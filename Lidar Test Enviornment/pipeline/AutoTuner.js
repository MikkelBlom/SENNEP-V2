/**
 * AutoTuner: Analyzes incoming scan data characteristics and adjusts
 * pipeline parameters automatically.
 *
 * Runs a calibration phase on the first N sweeps, then periodically
 * re-evaluates to adapt to changing environments.
 *
 * What it measures:
 * - Point density (pts/sweep) → adjusts downsampler cell size
 * - Noise level (SOR rejection rate) → adjusts outlier multiplier
 * - Average neighbor distance → adjusts DBSCAN epsilon
 * - Wall stability (tracking churn rate) → adjusts smoothing alpha
 * - Processing latency → adjusts update rate to stay within budget
 *
 * The user can override any auto-tuned value via the UI, which "locks"
 * that parameter and prevents the auto-tuner from changing it.
 */
class AutoTuner {
  constructor(pipeline, config = {}) {
    this.pipeline = pipeline;

    this.config = {
      enabled: true,
      calibrationSweeps: 20,      // Number of sweeps for initial calibration
      reevaluateInterval: 50,     // Re-check every N sweeps
      targetLatencyMs: 30,        // Target processing time budget
      maxLatencyMs: 80,           // If latency exceeds this, aggressively downsample
      ...config,
    };

    // Metrics accumulators for the calibration window
    this.metrics = {
      rawCounts: [],
      filteredCounts: [],
      cleanedCounts: [],
      processTimes: [],
      wallCounts: [],
      gridCells: [],
      neighborDistances: [],
    };

    this.sweepsSinceLastTune = 0;
    this.calibrationComplete = false;
    this.lockedParams = new Set(); // Parameters manually set by the user
    this.tuneHistory = [];         // Log of automatic adjustments
  }

  updateConfig(newConfig) {
    Object.assign(this.config, newConfig);
  }

  /**
   * Lock a parameter so auto-tuner won't change it.
   * Called when user manually adjusts a slider.
   */
  lockParam(paramKey) {
    this.lockedParams.add(paramKey);
  }

  unlockParam(paramKey) {
    this.lockedParams.delete(paramKey);
  }

  /**
   * Feed diagnostics from the pipeline after each processed sweep.
   * @param {object} diagnostics - From Pipeline.process() output
   * @param {Array} cleanedPoints - Points after SOR for neighbor analysis
   */
  ingest(diagnostics, cleanedPoints) {
    if (!this.config.enabled) return;

    this.metrics.rawCounts.push(diagnostics.rawCount);
    this.metrics.filteredCounts.push(diagnostics.filteredCount);
    this.metrics.cleanedCounts.push(diagnostics.cleanedCount);
    this.metrics.processTimes.push(diagnostics.processTimeMs);
    this.metrics.wallCounts.push(diagnostics.wallCount);
    this.metrics.gridCells.push(diagnostics.gridCellCount);

    // Sample neighbor distances for epsilon tuning (only every 5th sweep to save CPU)
    if (cleanedPoints && cleanedPoints.length > 20 && this.sweepsSinceLastTune % 5 === 0) {
      const avgNeighborDist = this.sampleAverageNeighborDistance(cleanedPoints);
      this.metrics.neighborDistances.push(avgNeighborDist);
    }

    this.sweepsSinceLastTune++;

    // Initial calibration
    if (!this.calibrationComplete && this.sweepsSinceLastTune >= this.config.calibrationSweeps) {
      this.calibrationComplete = true;
      this.tune('calibration');
      return;
    }

    // Periodic re-evaluation
    if (this.calibrationComplete && this.sweepsSinceLastTune >= this.config.reevaluateInterval) {
      this.tune('periodic');
    }
  }

  /**
   * Analyze accumulated metrics and adjust pipeline parameters.
   */
  tune(reason) {
    const adjustments = {};

    // --- 1. Point Density → Downsampler Cell Size ---
    if (!this.lockedParams.has('downsampler.cellSize')) {
      const avgRaw = this.median(this.metrics.rawCounts);
      const avgCleaned = this.median(this.metrics.cleanedCounts);

      // Aim for 600-1000 cleaned points
      if (avgCleaned > 1200) {
        const currentCell = this.pipeline.downsampler.config.cellSize;
        const newCell = Math.min(100, currentCell + 10);
        if (newCell !== currentCell) {
          this.pipeline.downsampler.updateConfig({ cellSize: newCell });
          adjustments['downsampler.cellSize'] = { from: currentCell, to: newCell, reason: `${avgCleaned} pts too dense` };
        }
      } else if (avgCleaned < 400 && avgRaw > 600) {
        const currentCell = this.pipeline.downsampler.config.cellSize;
        const newCell = Math.max(10, currentCell - 5);
        if (newCell !== currentCell) {
          this.pipeline.downsampler.updateConfig({ cellSize: newCell });
          adjustments['downsampler.cellSize'] = { from: currentCell, to: newCell, reason: `${avgCleaned} pts too sparse` };
        }
      }
    }

    // --- 2. Noise Level → Outlier Removal Multiplier ---
    if (!this.lockedParams.has('outlierRemoval.stddevMultiplier')) {
      const avgFiltered = this.median(this.metrics.filteredCounts);
      const avgCleaned = this.median(this.metrics.cleanedCounts);

      if (avgFiltered > 0) {
        const rejectionRate = 1 - (avgCleaned / avgFiltered);

        // If rejecting too many points (>30%), loosen the filter
        if (rejectionRate > 0.3) {
          const current = this.pipeline.outlierRemoval.config.stddevMultiplier;
          const newVal = Math.min(3.0, current + 0.2);
          if (newVal !== current) {
            this.pipeline.outlierRemoval.updateConfig({ stddevMultiplier: newVal });
            adjustments['outlierRemoval.stddevMultiplier'] = { from: current, to: newVal, reason: `${(rejectionRate * 100).toFixed(0)}% rejection too high` };
          }
        }
        // If rejecting too few (<5%), tighten it
        if (rejectionRate < 0.05) {
          const current = this.pipeline.outlierRemoval.config.stddevMultiplier;
          const newVal = Math.max(0.8, current - 0.1);
          if (newVal !== current) {
            this.pipeline.outlierRemoval.updateConfig({ stddevMultiplier: newVal });
            adjustments['outlierRemoval.stddevMultiplier'] = { from: current, to: newVal, reason: `${(rejectionRate * 100).toFixed(0)}% rejection too low` };
          }
        }
      }
    }

    // --- 3. Neighbor Distance → DBSCAN Epsilon ---
    if (!this.lockedParams.has('clusterer.epsilon') && this.metrics.neighborDistances.length >= 3) {
      const avgNeighborDist = this.median(this.metrics.neighborDistances);
      // Epsilon should be ~2-3x the average neighbor distance
      const idealEpsilon = Math.round(avgNeighborDist * 2.5);
      const clampedEpsilon = Math.max(30, Math.min(500, idealEpsilon));

      const currentEpsilon = this.pipeline.clusterer.config.epsilon;
      const diff = Math.abs(clampedEpsilon - currentEpsilon);

      // Only adjust if the difference is significant (>20%)
      if (diff > currentEpsilon * 0.2) {
        this.pipeline.clusterer.updateConfig({ epsilon: clampedEpsilon });
        adjustments['clusterer.epsilon'] = { from: currentEpsilon, to: clampedEpsilon, reason: `avg neighbor dist ${avgNeighborDist.toFixed(0)}mm` };
      }
    }

    // --- 4. Processing Latency → Update Rate ---
    if (!this.lockedParams.has('global.updateRate')) {
      const avgLatency = this.median(this.metrics.processTimes);

      if (avgLatency > this.config.maxLatencyMs && this.pipeline.updateRate < 5) {
        const newRate = this.pipeline.updateRate + 1;
        this.pipeline.updateRate = newRate;
        adjustments['global.updateRate'] = { from: newRate - 1, to: newRate, reason: `${avgLatency.toFixed(0)}ms exceeds budget` };
      } else if (avgLatency < this.config.targetLatencyMs * 0.5 && this.pipeline.updateRate > 1) {
        const newRate = this.pipeline.updateRate - 1;
        this.pipeline.updateRate = newRate;
        adjustments['global.updateRate'] = { from: newRate + 1, to: newRate, reason: `${avgLatency.toFixed(0)}ms under budget` };
      }
    }

    // --- 5. Wall Stability → Tracker Smoothing ---
    if (!this.lockedParams.has('wallTracker.smoothing')) {
      const wallCountVariance = this.variance(this.metrics.wallCounts);
      const avgWallCount = this.median(this.metrics.wallCounts);

      // High variance in wall count = high churn = need more smoothing
      if (avgWallCount > 0) {
        const cv = Math.sqrt(wallCountVariance) / avgWallCount; // Coefficient of variation

        if (cv > 0.5) {
          // Highly unstable — slow down smoothing
          const current = this.pipeline.wallTracker?.config?.smoothing || 0.15;
          const newVal = Math.max(0.05, current - 0.03);
          if (this.pipeline.wallTracker && newVal !== current) {
            this.pipeline.wallTracker.updateConfig({ smoothing: newVal });
            adjustments['wallTracker.smoothing'] = { from: current, to: newVal, reason: `high churn CV=${cv.toFixed(2)}` };
          }
        } else if (cv < 0.15) {
          // Very stable — can increase responsiveness
          const current = this.pipeline.wallTracker?.config?.smoothing || 0.15;
          const newVal = Math.min(0.4, current + 0.02);
          if (this.pipeline.wallTracker && newVal !== current) {
            this.pipeline.wallTracker.updateConfig({ smoothing: newVal });
            adjustments['wallTracker.smoothing'] = { from: current, to: newVal, reason: `stable CV=${cv.toFixed(2)}` };
          }
        }
      }
    }

    // Log adjustments
    if (Object.keys(adjustments).length > 0) {
      const entry = {
        timestamp: Date.now(),
        reason,
        sweepCount: this.sweepsSinceLastTune,
        adjustments,
      };
      this.tuneHistory.push(entry);
      console.log(`[AutoTuner] ${reason}: ${Object.entries(adjustments).map(([k, v]) => `${k}: ${v.from} → ${v.to} (${v.reason})`).join(', ')}`);
    }

    // Reset accumulators
    this.metrics.rawCounts = [];
    this.metrics.filteredCounts = [];
    this.metrics.cleanedCounts = [];
    this.metrics.processTimes = [];
    this.metrics.wallCounts = [];
    this.metrics.gridCells = [];
    this.metrics.neighborDistances = [];
    this.sweepsSinceLastTune = 0;
  }

  /**
   * Sample average nearest-neighbor distance from a point cloud.
   * Uses a random subset for performance.
   */
  sampleAverageNeighborDistance(points) {
    const sampleSize = Math.min(50, points.length);
    const step = Math.max(1, Math.floor(points.length / sampleSize));
    let totalDist = 0;
    let count = 0;

    for (let i = 0; i < points.length; i += step) {
      const p = points[i];
      let minDist = Infinity;

      // Find nearest neighbor (brute force on sample — fine for 50 points)
      for (let j = 0; j < points.length; j += step) {
        if (i === j) continue;
        const dist = Math.sqrt((p.x - points[j].x) ** 2 + (p.y - points[j].y) ** 2);
        if (dist < minDist) minDist = dist;
      }

      if (isFinite(minDist)) {
        totalDist += minDist;
        count++;
      }
    }

    return count > 0 ? totalDist / count : 100;
  }

  /**
   * Get the latest tuning status for the UI diagnostics.
   */
  getStatus() {
    return {
      enabled: this.config.enabled,
      calibrated: this.calibrationComplete,
      lockedParams: Array.from(this.lockedParams),
      lastAdjustments: this.tuneHistory.length > 0 ? this.tuneHistory[this.tuneHistory.length - 1] : null,
      totalAdjustments: this.tuneHistory.length,
    };
  }

  // --- Math Helpers ---

  median(arr) {
    if (arr.length === 0) return 0;
    const sorted = [...arr].sort((a, b) => a - b);
    const mid = Math.floor(sorted.length / 2);
    return sorted.length % 2 !== 0 ? sorted[mid] : (sorted[mid - 1] + sorted[mid]) / 2;
  }

  variance(arr) {
    if (arr.length < 2) return 0;
    const mean = arr.reduce((a, b) => a + b, 0) / arr.length;
    return arr.reduce((sum, v) => sum + (v - mean) ** 2, 0) / arr.length;
  }
}

module.exports = AutoTuner;
