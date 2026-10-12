/**
 * OutlierRemoval: Statistical Outlier Removal (SOR).
 * For each point, checks if its distance to K-nearest neighbors
 * is within a statistical threshold. Points that are isolated
 * (far from their neighbors) get flagged as noise and dropped.
 *
 * Uses a simple grid-based spatial index instead of a full KD-tree
 * for O(N) average-case performance on 2D LiDAR data.
 */
class OutlierRemoval {
  constructor(config = {}) {
    this.config = {
      enabled: true,
      neighborCount: 8,       // K neighbors to check
      stddevMultiplier: 1.5,  // Points beyond mean + multiplier*stddev are outliers
      gridSize: 200,          // Spatial index cell size (mm)
      ...config,
    };
  }

  updateConfig(newConfig) {
    Object.assign(this.config, newConfig);
  }

  /**
   * @param {Array<{angle: number, distance: number, x: number, y: number}>} points
   * @returns {Array} Points with outliers removed
   */
  process(points) {
    if (!this.config.enabled || !points || points.length < 10) return points;

    // Ensure cartesian coords exist
    const withCartesian = points.map(p => {
      if (p.x !== undefined && p.y !== undefined) return p;
      const rad = (p.angle - 90) * (Math.PI / 180);
      return {
        ...p,
        x: p.distance * Math.cos(rad),
        y: p.distance * Math.sin(rad),
      };
    });

    const { neighborCount, stddevMultiplier, gridSize } = this.config;

    // Build spatial index
    const spatialIndex = new Map();
    for (const point of withCartesian) {
      const cellX = Math.floor(point.x / gridSize);
      const cellY = Math.floor(point.y / gridSize);
      const key = `${cellX}_${cellY}`;
      if (!spatialIndex.has(key)) spatialIndex.set(key, []);
      spatialIndex.get(key).push(point);
    }

    // Calculate mean neighbor distance for each point
    const meanDistances = new Float64Array(withCartesian.length);

    for (let i = 0; i < withCartesian.length; i++) {
      const point = withCartesian[i];
      const cellX = Math.floor(point.x / gridSize);
      const cellY = Math.floor(point.y / gridSize);

      // Gather candidate neighbors from adjacent cells
      const candidates = [];
      for (let dx = -1; dx <= 1; dx++) {
        for (let dy = -1; dy <= 1; dy++) {
          const neighbors = spatialIndex.get(`${cellX + dx}_${cellY + dy}`);
          if (neighbors) candidates.push(...neighbors);
        }
      }

      // Find K nearest distances
      const distances = [];
      for (const candidate of candidates) {
        if (candidate === point) continue;
        const distSq = (point.x - candidate.x) ** 2 + (point.y - candidate.y) ** 2;
        distances.push(Math.sqrt(distSq));
      }

      distances.sort((a, b) => a - b);
      const kNearest = distances.slice(0, neighborCount);

      if (kNearest.length === 0) {
        meanDistances[i] = Infinity;
        continue;
      }

      const sum = kNearest.reduce((acc, d) => acc + d, 0);
      meanDistances[i] = sum / kNearest.length;
    }

    // Calculate global stats
    let globalSum = 0;
    let validCount = 0;
    for (let i = 0; i < meanDistances.length; i++) {
      if (isFinite(meanDistances[i])) {
        globalSum += meanDistances[i];
        validCount++;
      }
    }

    if (validCount === 0) return withCartesian;

    const globalMean = globalSum / validCount;

    let varianceSum = 0;
    for (let i = 0; i < meanDistances.length; i++) {
      if (isFinite(meanDistances[i])) {
        varianceSum += (meanDistances[i] - globalMean) ** 2;
      }
    }
    const globalStddev = Math.sqrt(varianceSum / validCount);
    const threshold = globalMean + stddevMultiplier * globalStddev;

    // Filter
    return withCartesian.filter((_, i) => meanDistances[i] <= threshold);
  }
}

module.exports = OutlierRemoval;
