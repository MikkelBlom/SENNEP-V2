/**
 * CalibrationRunner: Grid search over WallExtractor parameters.
 *
 * Uses recorded JSONL data to test parameter combinations and find the
 * most stable configuration. "Stable" = consistent wall count + minimal
 * endpoint drift across sweeps.
 *
 * Scoring:
 * - wallCountConsistency: Low variance in wall count = good
 * - coverageScore: Higher fraction of profile covered = good
 * - stabilityScore: Low average endpoint drift between consecutive sweeps = good
 * - fitscore: Low average RMS deviation of wall fits = good (tight fits)
 *
 * This runs in the main process and reports progress via a callback.
 */
const fs = require('fs');
const PreFilter = require('./PreFilter');
const WallExtractor = require('./WallExtractor');

class CalibrationRunner {
  constructor() {
    this.isRunning = false;
    this.progress = 0;
    this.results = [];
  }

  /**
   * Run calibration against a recording file.
   * @param {string} filePath - Path to a JSONL recording
   * @param {object} preFilterConfig - PreFilter config to use
   * @param {function} onProgress - Called with { progress: 0-1, message: string }
   * @returns {Promise<{ bestConfig: object, bestScore: number, results: Array }>}
   */
  async run(filePath, preFilterConfig, onProgress) {
    if (this.isRunning) throw new Error('Calibration already running');
    this.isRunning = true;
    this.results = [];

    try {
      // Load recording
      const content = fs.readFileSync(filePath, 'utf8');
      const lines = content.trim().split('\n');
      const sweeps = lines.map(line => JSON.parse(line).sweep);

      if (sweeps.length < 5) {
        throw new Error('Recording too short — need at least 5 sweeps');
      }

      // Use a subset for speed (max 30 sweeps)
      const maxSweeps = Math.min(30, sweeps.length);
      const testSweeps = sweeps.slice(0, maxSweeps);

      // Define parameter grid
      const grid = {
        emaAlpha:               [0.15, 0.3, 0.5],
        lineDeviationThreshold: [25, 50, 100, 150],
        minWallLength:          [25, 50, 100, 200],
        cornerAngleThreshold:   [10, 20, 35, 45],
        minWallBins:            [3, 6, 10],
      };

      // Generate all combinations
      const combos = this.cartesian(grid);
      const totalCombos = combos.length;

      onProgress?.({ progress: 0, message: `Testing ${totalCombos} parameter combinations...` });

      let bestScore = -Infinity;
      let bestConfig = null;

      for (let i = 0; i < combos.length; i++) {
        const combo = combos[i];

        const score = this.evaluateConfig(combo, testSweeps, preFilterConfig);

        this.results.push({ config: combo, score });

        if (score.total > bestScore) {
          bestScore = score.total;
          bestConfig = combo;
        }

        // Report progress every 10 combos
        if (i % 10 === 0 || i === combos.length - 1) {
          const progress = (i + 1) / totalCombos;
          onProgress?.({
            progress,
            message: `Tested ${i + 1}/${totalCombos} — best score: ${bestScore.toFixed(1)}`,
          });
        }

        // Yield to event loop occasionally
        if (i % 50 === 0) {
          await new Promise(resolve => setImmediate(resolve));
        }
      }

      // Sort results by total score
      this.results.sort((a, b) => b.score.total - a.score.total);

      this.isRunning = false;

      return {
        bestConfig: {
          binResolution: 0.5,
          maxGapBins: 3,
          enabled: true,
          ...bestConfig,
        },
        bestScore,
        topResults: this.results.slice(0, 5).map(r => ({
          config: r.config,
          score: r.score.total,
          breakdown: r.score,
        })),
      };
    } catch (err) {
      this.isRunning = false;
      throw err;
    }
  }

  /**
   * Evaluate a single parameter config against the test sweeps.
   */
  evaluateConfig(config, sweeps, preFilterConfig) {
    const preFilter = new PreFilter(preFilterConfig);
    const extractor = new WallExtractor({
      binResolution: 0.5,
      maxGapBins: 3,
      enabled: true,
      ...config,
    });

    const wallCountsPerSweep = [];
    const wallsPerSweep = [];

    for (const sweep of sweeps) {
      const filtered = preFilter.process(sweep);
      const walls = extractor.process(filtered);
      wallCountsPerSweep.push(walls.length);
      wallsPerSweep.push(walls);
    }

    // --- Score 1: Wall Count Consistency (0-30 points) ---
    // Low variance in wall count across sweeps = stable
    const skipFirst = 3; // Skip warmup sweeps
    const stableCounts = wallCountsPerSweep.slice(skipFirst);
    const countConsistency = this.scoreConsistency(stableCounts, 30);

    // --- Score 2: Coverage (0-25 points) ---
    // Higher average wall count = better coverage (up to a limit)
    const avgWalls = stableCounts.length > 0
      ? stableCounts.reduce((s, v) => s + v, 0) / stableCounts.length
      : 0;
    // Sweet spot: 8-15 walls. Penalize both too few and too many.
    const coverageScore = Math.max(0, 25 - Math.abs(avgWalls - 12) * 2);

    // --- Score 3: Endpoint Stability (0-30 points) ---
    // Compare walls between consecutive sweeps (after warmup)
    let totalDrift = 0;
    let driftSamples = 0;

    for (let i = skipFirst + 1; i < wallsPerSweep.length; i++) {
      const prevWalls = wallsPerSweep[i - 1];
      const currWalls = wallsPerSweep[i];

      for (const curr of currWalls) {
        // Find closest wall from previous sweep
        let bestDist = Infinity;
        for (const prev of prevWalls) {
          const d = this.wallCenterDistance(curr, prev);
          if (d < bestDist) bestDist = d;
        }
        if (isFinite(bestDist)) {
          totalDrift += bestDist;
          driftSamples++;
        }
      }
    }

    const avgDrift = driftSamples > 0 ? totalDrift / driftSamples : 1000;
    // Lower drift = higher score. 0mm drift = 30pts, 50mm = 15pts, 100mm+ = 0pts
    const stabilityScore = Math.max(0, 30 * (1 - avgDrift / 100));

    // --- Score 4: Fit Quality (0-15 points) ---
    // Lower average RMS deviation in wall fits = tighter lines
    let totalRms = 0;
    let rmsSamples = 0;
    for (const walls of wallsPerSweep.slice(skipFirst)) {
      for (const w of walls) {
        if (w.rmsDev !== undefined) {
          totalRms += w.rmsDev;
          rmsSamples++;
        }
      }
    }
    const avgRms = rmsSamples > 0 ? totalRms / rmsSamples : 100;
    // Lower RMS = higher score. 0mm = 15pts, 50mm = 7.5pts, 100mm+ = 0pts
    const fitScore = Math.max(0, 15 * (1 - avgRms / 100));

    return {
      total: Math.round((countConsistency + coverageScore + stabilityScore + fitScore) * 10) / 10,
      countConsistency: Math.round(countConsistency * 10) / 10,
      coverageScore: Math.round(coverageScore * 10) / 10,
      stabilityScore: Math.round(stabilityScore * 10) / 10,
      fitScore: Math.round(fitScore * 10) / 10,
      avgWalls: Math.round(avgWalls * 10) / 10,
      avgDrift: Math.round(avgDrift * 10) / 10,
      avgRms: Math.round(avgRms * 10) / 10,
    };
  }

  /**
   * Score consistency of an array of values. Lower variance = higher score.
   */
  scoreConsistency(values, maxScore) {
    if (values.length < 2) return 0;
    const mean = values.reduce((s, v) => s + v, 0) / values.length;
    if (mean === 0) return 0;
    const variance = values.reduce((s, v) => s + (v - mean) ** 2, 0) / values.length;
    const cv = Math.sqrt(variance) / mean; // Coefficient of variation
    // CV of 0 = full score, CV of 0.5+ = 0
    return Math.max(0, maxScore * (1 - cv * 2));
  }

  wallCenterDistance(wallA, wallB) {
    const dx = wallA.center.x - wallB.center.x;
    const dy = wallA.center.y - wallB.center.y;
    return Math.sqrt(dx * dx + dy * dy);
  }

  /**
   * Cartesian product of a parameter grid object.
   * { a: [1,2], b: [3,4] } → [{ a:1, b:3 }, { a:1, b:4 }, { a:2, b:3 }, { a:2, b:4 }]
   */
  cartesian(grid) {
    const keys = Object.keys(grid);
    const results = [];

    const recurse = (depth, current) => {
      if (depth === keys.length) {
        results.push({ ...current });
        return;
      }
      const key = keys[depth];
      for (const val of grid[key]) {
        current[key] = val;
        recurse(depth + 1, current);
      }
    };

    recurse(0, {});
    return results;
  }
}

module.exports = CalibrationRunner;
