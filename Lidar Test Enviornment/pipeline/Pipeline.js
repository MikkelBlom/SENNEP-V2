/**
 * Pipeline: Orchestrator for the SLAM processing chain.
 *
 * Chain: PreFilter → Downsampler → OutlierRemoval → ScanMatcher →
 *        SensorFusion → OccupancyGrid → WallExtractor
 *
 * Key design change: Walls are extracted from an accumulated polar profile
 * using least-squares fitting, NOT from per-frame RANSAC. This is
 * deterministic and jitter-free.
 */
const fs = require('fs');
const path = require('path');

const PreFilter = require('./PreFilter');
const Downsampler = require('./Downsampler');
const OutlierRemoval = require('./OutlierRemoval');
const OccupancyGrid = require('./OccupancyGrid');
const ScanMatcher = require('./ScanMatcher');
const SensorFusion = require('./SensorFusion');
const WallExtractor = require('./WallExtractor');
const WallTracker = require('./WallTracker');
const MovingObjectDetector = require('./MovingObjectDetector');

class Pipeline {
  constructor(config = {}) {
    this.preFilter = new PreFilter(config.preFilter);
    this.downsampler = new Downsampler(config.downsampler);
    this.outlierRemoval = new OutlierRemoval(config.outlierRemoval);
    this.occupancyGrid = new OccupancyGrid(config.occupancyGrid);
    this.scanMatcher = new ScanMatcher(config.scanMatcher);
    this.sensorFusion = new SensorFusion(config.sensorFusion);
    this.wallExtractor = new WallExtractor(config.wallExtractor);
    this.wallTracker = new WallTracker(config.wallTracker);
    this.movingObjectDetector = new MovingObjectDetector(config.movingObjectDetector);

    // Frequency limiting
    this.sweepCount = 0;
    this.updateRate = config.updateRate || 2;

    // Performance tracking
    this.lastProcessTime = 0;

    // Data recording
    this.recording = false;
    this.recordingStream = null;
    this.recordingPath = null;

    // Cached output for throttled frames
    this.cachedOutput = null;
  }

  updateConfig(moduleId, config) {
    const moduleMap = {
      preFilter: this.preFilter,
      downsampler: this.downsampler,
      outlierRemoval: this.outlierRemoval,
      occupancyGrid: this.occupancyGrid,
      scanMatcher: this.scanMatcher,
      sensorFusion: this.sensorFusion,
      wallExtractor: this.wallExtractor,
      wallTracker: this.wallTracker,
      movingObjectDetector: this.movingObjectDetector,
    };

    if (moduleId === 'global') {
      if (config.updateRate !== undefined) this.updateRate = config.updateRate;
      return;
    }

    const target = moduleMap[moduleId];
    if (target) {
      target.updateConfig(config);
    }
  }

  /**
   * Process a complete 360° sweep.
   * @param {Array} rawSweep - Raw points from LidarParser
   * @returns {object|null} Processed output for the UI
   */
  process(rawSweep) {
    if (!rawSweep || rawSweep.length === 0) return this.cachedOutput;

    this.sweepCount++;

    // Record raw data if active
    if (this.recording && this.recordingStream) {
      try {
        this.recordingStream.write(JSON.stringify({
          timestamp: Date.now(),
          sweep: rawSweep,
        }) + '\n');
      } catch (err) {
        console.error('[Pipeline] Recording write error:', err.message);
      }
    }

    // Frequency limiting
    if (this.sweepCount % this.updateRate !== 0) {
      return this.cachedOutput;
    }

    const startTime = performance.now();

    // === STAGE 1: Pre-Filter ===
    const filtered = this.preFilter.process(rawSweep);

    // Add cartesian x/y to filtered points so downstream stages (wall classification,
    // moving object detection) can use them without re-converting from polar.
    for (const p of filtered) {
      const rad = (p.angle - 90) * (Math.PI / 180);
      p.x = p.distance * Math.cos(rad);
      p.y = p.distance * Math.sin(rad);
    }

    // === STAGE 2: Downsample ===
    const downsampled = this.downsampler.process(filtered);

    // === STAGE 3: Outlier Removal ===
    const cleaned = this.outlierRemoval.process(downsampled);

    // === STAGE 4: Scan Matching (ICP) ===
    const icpResult = this.scanMatcher.process(cleaned);

    // === STAGE 5: Sensor Fusion ===
    const fusedPose = this.sensorFusion.process(icpResult.pose);

    // === STAGE 7: Wall Extraction — Two-Phase Classification ===
    // Phase 1: Classify points against the CURRENT (pre-update) stable profile.
    // This must run before updateProfile() so we compare against uncontaminated data.
    // Use `filtered` (full angular density) for the best wall profile accuracy.
    // filtered points now carry x/y from the cartesian conversion step above.
    const classified = this.wallExtractor.classifyPoints(filtered);

    // Phase 2: Update profile using only stable (non-object) points.
    // Moving objects are now excluded — the profile stays fixed to wall distances.
    this.wallExtractor.updateProfile(classified.stable);

    // Phase 3: Extract walls from the stable profile.
    const rawWalls = this.wallExtractor.extractWalls();

    // Phase 4: Temporally track walls across frames — smooths endpoints (EMA),
    // suppresses one-frame flicker (minLifetimeToShow), and merges nearby
    // parallel fragments into single segments.
    const walls = this.wallTracker.process(rawWalls);

    // === STAGE 6: Occupancy Grid ===
    // Exclude transient bins so moving objects don't leave ghost trails in the map.
    let gridPoints = cleaned;
    if (classified.transient.length > 0) {
      const transientBins = new Set(classified.transient.map(p => p.binIndex));
      const filteredForGrid = cleaned.filter(p => {
        const bin = Math.floor(p.angle / this.wallExtractor.config.binResolution) %
                    this.wallExtractor.binCount;
        return !transientBins.has(bin);
      });
      // Startup robustness: if nearly everything is transient, fall back to full cleaned
      gridPoints = filteredForGrid.length >= 20 ? filteredForGrid : cleaned;
    }
    this.occupancyGrid.update(gridPoints, fusedPose);
    const gridSnapshot = this.occupancyGrid.getSnapshot();

    // === STAGE 8: Moving Object Detection ===
    // Transient points were pre-classified against the stable profile — cluster them directly.
    const movingObjects = this.movingObjectDetector.processTransients(classified.transient);

    this.lastProcessTime = performance.now() - startTime;

    // Build output
    this.cachedOutput = {
      points: cleaned.map(p => ({
        x: p.x,
        y: p.y,
        angle: p.angle,
        distance: p.distance,
      })),

      walls,
      movingObjects,
      grid: gridSnapshot,
      pose: fusedPose,

      diagnostics: {
        rawCount: rawSweep.length,
        filteredCount: filtered.length,
        downsampledCount: downsampled.length,
        cleanedCount: cleaned.length,
        wallCount: walls.length,
        transientCount: classified.transient.length,
        gridCellCount: gridSnapshot.totalCells,
        processTimeMs: Math.round(this.lastProcessTime * 10) / 10,
        icpConverged: icpResult.converged,
        icpIterations: icpResult.iterations,
        sweepNumber: this.sweepCount,
        profileSweeps: this.wallExtractor.sweepsProcessed,
      },
    };

    return this.cachedOutput;
  }

  // --- Recording ---

  startRecording(directory) {
    if (this.recording) return;
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    this.recordingPath = path.join(directory, `lidar_recording_${timestamp}.jsonl`);
    try {
      this.recordingStream = fs.createWriteStream(this.recordingPath, { flags: 'a' });
      this.recording = true;
      console.log(`[Pipeline] Recording started: ${this.recordingPath}`);
    } catch (err) {
      console.error('[Pipeline] Failed to start recording:', err.message);
    }
  }

  stopRecording() {
    if (!this.recording) return null;
    this.recording = false;
    if (this.recordingStream) {
      this.recordingStream.end();
      this.recordingStream = null;
    }
    const savedPath = this.recordingPath;
    this.recordingPath = null;
    console.log(`[Pipeline] Recording stopped: ${savedPath}`);
    return savedPath;
  }

  async replayRecording(filePath, onSweep, delayMs = 100) {
    const content = fs.readFileSync(filePath, 'utf8');
    const lines = content.trim().split('\n');
    for (const line of lines) {
      try {
        const entry = JSON.parse(line);
        const result = this.process(entry.sweep);
        if (result && onSweep) onSweep(result);
        if (delayMs > 0) await new Promise(resolve => setTimeout(resolve, delayMs));
      } catch (err) {
        console.error('[Pipeline] Replay parse error:', err.message);
      }
    }
  }

  clearMap() {
    this.occupancyGrid.clear();
    this.scanMatcher.reset();
    this.sensorFusion.reset();
    this.wallExtractor.clear();
    this.wallTracker.clear();
    this.movingObjectDetector.clear();
    this.cachedOutput = null;
    this.sweepCount = 0;
  }
}

module.exports = Pipeline;
