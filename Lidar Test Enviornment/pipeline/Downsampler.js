/**
 * Downsampler: Voxel grid thinning.
 * Reduces point count by averaging points that fall within the same
 * spatial cell. Preserves geometric shape while cutting compute cost.
 */
class Downsampler {
  constructor(config = {}) {
    this.config = {
      cellSize: 30,   // mm — voxel grid resolution
      enabled: true,
      ...config,
    };
  }

  updateConfig(newConfig) {
    Object.assign(this.config, newConfig);
  }

  /**
   * Converts polar points to cartesian, bins into a voxel grid,
   * returns centroids as polar points.
   * @param {Array<{angle: number, distance: number, confidence: number}>} points
   * @returns {Array} Downsampled points (polar, with averaged confidence)
   */
  process(points) {
    if (!this.config.enabled || !points || points.length === 0) return points;

    const { cellSize } = this.config;
    const bins = new Map();

    for (const point of points) {
      const rad = (point.angle - 90) * (Math.PI / 180);
      const cartX = point.distance * Math.cos(rad);
      const cartY = point.distance * Math.sin(rad);

      const gridX = Math.floor(cartX / cellSize);
      const gridY = Math.floor(cartY / cellSize);
      const key = `${gridX}_${gridY}`;

      let bin = bins.get(key);
      if (!bin) {
        bin = { sumX: 0, sumY: 0, sumConf: 0, count: 0 };
        bins.set(key, bin);
      }

      bin.sumX += cartX;
      bin.sumY += cartY;
      bin.sumConf += (point.confidence || 200);
      bin.count++;
    }

    const result = [];
    for (const bin of bins.values()) {
      const avgX = bin.sumX / bin.count;
      const avgY = bin.sumY / bin.count;
      const angle = (Math.atan2(avgY, avgX) * (180 / Math.PI) + 90 + 360) % 360;
      const distance = Math.sqrt(avgX * avgX + avgY * avgY);

      result.push({
        angle,
        distance,
        confidence: Math.round(bin.sumConf / bin.count),
        // Carry cartesian for downstream modules to avoid re-converting
        x: avgX,
        y: avgY,
      });
    }

    return result;
  }
}

module.exports = Downsampler;
