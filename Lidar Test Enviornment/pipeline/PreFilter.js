/**
 * PreFilter: Distance, confidence, and angle gating.
 * First stage in the SLAM pipeline — removes obviously bad data
 * before any expensive computation touches it.
 */
class PreFilter {
  constructor(config = {}) {
    this.config = {
      minRange: 150,          // mm — internal reflections below this
      maxRange: 12000,        // mm — STP-19P reliable limit
      minConfidence: 100,     // 0-255 from the sensor
      excludedSectors: [],    // Array of { startAngle, endAngle } to mask
      ...config,
    };
  }

  updateConfig(newConfig) {
    Object.assign(this.config, newConfig);
  }

  /**
   * @param {Array<{angle: number, distance: number, confidence: number}>} points
   * @returns {Array} Filtered points
   */
  process(points) {
    if (!points || points.length === 0) return [];

    const { minRange, maxRange, minConfidence, excludedSectors } = this.config;

    return points.filter(point => {
      if (point.distance < minRange || point.distance > maxRange) return false;
      if (point.confidence < minConfidence) return false;

      // Sector exclusion (e.g., cable exit zone)
      for (const sector of excludedSectors) {
        if (this.isAngleInSector(point.angle, sector.startAngle, sector.endAngle)) {
          return false;
        }
      }

      return true;
    });
  }

  isAngleInSector(angle, start, end) {
    // Handle wraparound (e.g., sector from 350° to 10°)
    if (start <= end) {
      return angle >= start && angle <= end;
    }
    return angle >= start || angle <= end;
  }
}

module.exports = PreFilter;
