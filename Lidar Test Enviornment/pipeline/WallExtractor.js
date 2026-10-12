/**
 * WallExtractor: Robust wall detection from accumulated scan data.
 *
 * WHY THIS REPLACES RANSAC:
 * The STP-19P data is extremely stable (stddev < 5mm at most angles).
 * The jitter was caused by RANSAC randomly sampling different subsets each
 * frame and producing different walls. That's the wrong tool.
 *
 * NEW APPROACH:
 * 1. Maintain a running average polar profile: for each 0.5° angle bin,
 *    store the exponentially-smoothed distance.
 * 2. Detect wall segments by finding runs of angle bins where the XY
 *    points are collinear (same wall surface).
 * 3. Use least-squares line fitting (not random sampling) on each run.
 * 4. Merge collinear segments across small gaps (doorways, corners).
 *
 * Benefits:
 * - Deterministic — same data always produces same walls
 * - Uses ALL points, not random samples
 * - Naturally smooth — the profile is EMA-averaged over multiple sweeps
 * - Handles the actual data characteristics (ordered angular scan)
 */
class WallExtractor {
  constructor(config = {}) {
    this.config = {
      binResolution: 0.5,         // degrees per bin
      emaAlpha: 0.3,              // Profile smoothing (0.3 = blend 30% new, 70% old)
      minWallBins: 6,             // Minimum angle bins for a wall segment (~3°)
      maxGapBins: 3,              // Bridge across up to N empty bins
      lineDeviationThreshold: 25, // mm — max perpendicular deviation from fitted line
      minWallLength: 120,         // mm — reject tiny wall fragments
      cornerAngleThreshold: 15,   // degrees — detect corners when wall direction changes
      maxRange: 12000,            // mm — ignore bins beyond this range
      profileProtectionThreshold: 100, // mm — abs diff to classify as transient
      profileProtectionRelative: 0.15, // ratio — relative diff to classify as transient
      cooldownSweeps: 5,               // sweeps before classification activates
      enabled: true,
      ...config,
    };

    // Polar profile: angle bin → smoothed distance
    this.binCount = Math.ceil(360 / this.config.binResolution);
    this.profile = new Float64Array(this.binCount);      // Smoothed distance
    this.profileCount = new Uint16Array(this.binCount);   // Hit count per bin
    this.profileValid = new Uint8Array(this.binCount);    // 1 = has data
    this.sweepsProcessed = 0;
  }

  updateConfig(newConfig) {
    Object.assign(this.config, newConfig);
  }

  /**
   * Update the polar profile with a new batch of filtered points.
   * @param {Array<{angle: number, distance: number}>} points
   */
  updateProfile(points) {
    if (!points || points.length === 0) return;

    const { binResolution, emaAlpha, maxRange } = this.config;
    const isFirstSweep = this.sweepsProcessed === 0;

    for (const point of points) {
      if (point.distance <= 0 || point.distance > maxRange) continue;

      const binIndex = Math.floor(point.angle / binResolution) % this.binCount;

      if (!this.profileValid[binIndex] || isFirstSweep) {
        // First measurement for this bin — accept directly
        this.profile[binIndex] = point.distance;
        this.profileValid[binIndex] = 1;
      } else {
        // EMA: blend new reading with historical average
        this.profile[binIndex] = this.profile[binIndex] * (1 - emaAlpha) + point.distance * emaAlpha;
      }

      this.profileCount[binIndex]++;
    }

    this.sweepsProcessed++;
  }

  /**
   * Classify incoming points as stable (background/wall) or transient (moving object).
   * Compares against the CURRENT profile before it is updated — call this before updateProfile().
   *
   * Transient: point is significantly CLOSER than the established profile (object in front of wall).
   * Stable: everything else (matches wall, farther than wall, or no baseline yet).
   *
   * Transient points carry { diff, profileDist, binIndex } metadata for downstream use.
   *
   * @param {Array} points - Filtered points to classify
   * @returns {{ stable: Array, transient: Array }}
   */
  classifyPoints(points) {
    const stable = [];
    const transient = [];

    if (!points || points.length === 0) return { stable, transient };

    // During cooldown there is no reliable baseline — treat everything as stable
    if (this.sweepsProcessed < this.config.cooldownSweeps) {
      return { stable: points, transient };
    }

    const { binResolution, profileProtectionThreshold, profileProtectionRelative } = this.config;

    for (const point of points) {
      if (point.distance <= 0 || point.distance > this.config.maxRange) {
        stable.push(point);
        continue;
      }

      const binIndex = Math.floor(point.angle / binResolution) % this.binCount;

      if (!this.profileValid[binIndex]) {
        // No established baseline for this bin — treat as stable
        stable.push(point);
        continue;
      }

      const profileDist = this.profile[binIndex];
      const diff = point.distance - profileDist; // negative = closer than wall

      if (diff < 0) {
        const absDiff = Math.abs(diff);
        const relativeDiff = absDiff / profileDist;
        if (absDiff >= profileProtectionThreshold || relativeDiff >= profileProtectionRelative) {
          // Attach metadata consumed by processTransients() and grid-bin filtering
          transient.push({ ...point, diff, profileDist, binIndex });
          continue;
        }
      }

      stable.push(point);
    }

    return { stable, transient };
  }

  /**
   * Extract wall segments from the current accumulated profile.
   * @returns {Array<WallSegment>}
   */
  extractWalls() {
    if (this.sweepsProcessed < 2) return [];

    const { binResolution, lineDeviationThreshold, minWallBins, maxGapBins, minWallLength, cornerAngleThreshold } = this.config;

    // Step 1: Convert valid profile bins to XY points
    const profilePoints = [];
    for (let i = 0; i < this.binCount; i++) {
      if (!this.profileValid[i]) continue;

      const angleDeg = (i + 0.5) * binResolution;
      const angleRad = (angleDeg - 90) * (Math.PI / 180);
      const distance = this.profile[i];

      profilePoints.push({
        binIndex: i,
        angle: angleDeg,
        distance,
        x: distance * Math.cos(angleRad),
        y: distance * Math.sin(angleRad),
      });
    }

    if (profilePoints.length < minWallBins) return [];

    // Step 2: Segment the profile into wall runs using angular collinearity
    const segments = this.segmentProfile(profilePoints, lineDeviationThreshold, minWallBins, maxGapBins, cornerAngleThreshold);

    // Step 3: Fit least-squares lines to each segment
    const walls = [];
    for (const segment of segments) {
      if (segment.length < minWallBins) continue;

      const wall = this.fitWallSegment(segment);
      if (wall && wall.length >= minWallLength) {
        walls.push(wall);
      }
    }

    return walls;
  }

  /**
   * Process: Combined update + extract. Called by Pipeline.
   * @param {Array} points - Filtered points from the pipeline
   * @returns {Array<WallSegment>}
   */
  process(points) {
    if (!this.config.enabled) return [];

    this.updateProfile(points);
    return this.extractWalls();
  }

  /**
   * Segment profile points into groups that belong to the same wall.
   * A new segment starts when:
   * - Large gap in angle bins (no data)
   * - Large distance jump (different surface)
   * - Direction change (corner)
   */
  segmentProfile(points, deviationThreshold, minBins, maxGapBins, cornerThresholdDeg) {
    const segments = [];
    let currentSegment = [points[0]];

    for (let i = 1; i < points.length; i++) {
      const prev = points[i - 1];
      const curr = points[i];

      // Check bin gap
      const binGap = curr.binIndex - prev.binIndex;
      if (binGap > maxGapBins) {
        if (currentSegment.length >= minBins) segments.push(currentSegment);
        currentSegment = [curr];
        continue;
      }

      // Check distance jump (different wall surface)
      const distJump = Math.abs(curr.distance - prev.distance);
      const relativeJump = distJump / Math.max(curr.distance, prev.distance);
      if (relativeJump > 0.15 && distJump > 50) {
        if (currentSegment.length >= minBins) segments.push(currentSegment);
        currentSegment = [curr];
        continue;
      }

      // Check direction change (corner detection)
      if (currentSegment.length >= 3) {
        const segStart = currentSegment[0];
        const segDir = Math.atan2(prev.y - segStart.y, prev.x - segStart.x);
        const newDir = Math.atan2(curr.y - prev.y, curr.x - prev.x);
        let dirDiff = Math.abs(segDir - newDir) * (180 / Math.PI);
        if (dirDiff > 180) dirDiff = 360 - dirDiff;

        if (dirDiff > cornerThresholdDeg) {
          if (currentSegment.length >= minBins) segments.push(currentSegment);
          currentSegment = [curr];
          continue;
        }
      }

      currentSegment.push(curr);
    }

    if (currentSegment.length >= minBins) segments.push(currentSegment);

    return segments;
  }

  /**
   * Fit a least-squares line to a segment and return wall endpoints.
   * Uses total least squares (principal axis) — more robust than OLS
   * when both X and Y have error.
   */
  fitWallSegment(segment) {
    const n = segment.length;
    if (n < 3) return null;

    // Compute centroid
    let cx = 0, cy = 0;
    for (const p of segment) { cx += p.x; cy += p.y; }
    cx /= n; cy /= n;

    // Compute covariance matrix elements
    let sxx = 0, sxy = 0, syy = 0;
    for (const p of segment) {
      const dx = p.x - cx;
      const dy = p.y - cy;
      sxx += dx * dx;
      sxy += dx * dy;
      syy += dy * dy;
    }

    // Principal direction from eigenvector of covariance matrix
    const theta = 0.5 * Math.atan2(2 * sxy, sxx - syy);
    const dirX = Math.cos(theta);
    const dirY = Math.sin(theta);

    // Normal direction
    const normX = -dirY;
    const normY = dirX;

    // Check fit quality: max perpendicular deviation
    let maxDev = 0;
    let totalDevSq = 0;
    for (const p of segment) {
      const dev = Math.abs((p.x - cx) * normX + (p.y - cy) * normY);
      if (dev > maxDev) maxDev = dev;
      totalDevSq += dev * dev;
    }

    const rmsDev = Math.sqrt(totalDevSq / n);
    if (rmsDev > this.config.lineDeviationThreshold) return null;

    // Project all points onto the line direction to find endpoints
    let minT = Infinity, maxT = -Infinity;
    for (const p of segment) {
      const t = (p.x - cx) * dirX + (p.y - cy) * dirY;
      if (t < minT) minT = t;
      if (t > maxT) maxT = t;
    }

    const startX = cx + dirX * minT;
    const startY = cy + dirY * minT;
    const endX = cx + dirX * maxT;
    const endY = cy + dirY * maxT;

    const length = Math.sqrt((endX - startX) ** 2 + (endY - startY) ** 2);
    const angle = Math.atan2(endY - startY, endX - startX);

    return {
      start: { x: startX, y: startY },
      end: { x: endX, y: endY },
      length,
      angle,
      center: { x: (startX + endX) / 2, y: (startY + endY) / 2 },
      pointCount: n,
      rmsDev: Math.round(rmsDev * 10) / 10,
      confidence: Math.min(1.0, this.sweepsProcessed / 10), // Builds up over time
    };
  }

  clear() {
    this.profile.fill(0);
    this.profileCount.fill(0);
    this.profileValid.fill(0);
    this.sweepsProcessed = 0;
  }
}

module.exports = WallExtractor;
