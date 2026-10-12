/**
 * FeatureExtractor: RANSAC line fitting per cluster.
 * Takes clustered point groups and extracts best-fit line segments.
 * Turns fuzzy point clouds into clean, mathematical wall lines.
 */
class FeatureExtractor {
  constructor(config = {}) {
    this.config = {
      ransacIterations: 100,
      ransacThreshold: 40,     // mm — max distance from line to be an inlier
      minWallPoints: 8,        // Min inliers for a valid wall
      minWallLength: 150,      // mm — minimum wall segment length
      ...config,
    };
  }

  updateConfig(newConfig) {
    Object.assign(this.config, newConfig);
  }

  /**
   * @param {Array<Array<{x: number, y: number}>>} clusters
   * @returns {Array<WallSegment>} Extracted wall segments
   */
  process(clusters) {
    if (!clusters || clusters.length === 0) return [];

    const walls = [];

    for (const cluster of clusters) {
      if (cluster.length < this.config.minWallPoints) continue;

      // A single cluster may contain multiple walls (e.g., an L-shaped corner).
      // Extract walls iteratively until remaining points cant form one.
      let remainingPoints = [...cluster];

      while (remainingPoints.length >= this.config.minWallPoints) {
        const result = this.ransac(remainingPoints);
        if (!result) break;

        const { inliers, outliers, wall } = result;

        if (wall.length >= this.config.minWallLength) {
          walls.push(wall);
        }

        remainingPoints = outliers;
      }
    }

    return walls;
  }

  /**
   * RANSAC: Random Sample Consensus for line fitting.
   * Picks two random points, fits a line, counts inliers.
   */
  ransac(points) {
    const { ransacIterations, ransacThreshold, minWallPoints } = this.config;

    let bestInlierIndices = [];
    let bestLine = null;

    for (let iter = 0; iter < ransacIterations; iter++) {
      // Pick two random distinct points
      const idxA = Math.floor(Math.random() * points.length);
      let idxB = Math.floor(Math.random() * (points.length - 1));
      if (idxB >= idxA) idxB++;

      const pointA = points[idxA];
      const pointB = points[idxB];

      // Skip if points are too close (degenerate line)
      const lineDx = pointB.x - pointA.x;
      const lineDy = pointB.y - pointA.y;
      const lineLen = Math.sqrt(lineDx * lineDx + lineDy * lineDy);
      if (lineLen < 10) continue;

      // Line equation: ax + by + c = 0  (normalized)
      const lineA = -lineDy / lineLen;
      const lineB = lineDx / lineLen;
      const lineC = -(lineA * pointA.x + lineB * pointA.y);

      // Count inliers
      const inlierIndices = [];
      for (let i = 0; i < points.length; i++) {
        const dist = Math.abs(lineA * points[i].x + lineB * points[i].y + lineC);
        if (dist <= ransacThreshold) {
          inlierIndices.push(i);
        }
      }

      if (inlierIndices.length > bestInlierIndices.length) {
        bestInlierIndices = inlierIndices;
        bestLine = { a: lineA, b: lineB, c: lineC };
      }
    }

    if (bestInlierIndices.length < minWallPoints || !bestLine) return null;

    // Separate inliers and outliers
    const inlierSet = new Set(bestInlierIndices);
    const inliers = [];
    const outliers = [];

    for (let i = 0; i < points.length; i++) {
      if (inlierSet.has(i)) inliers.push(points[i]);
      else outliers.push(points[i]);
    }

    // Project inliers onto the line and find segment endpoints
    const wall = this.fitLineSegment(inliers, bestLine);

    return { inliers, outliers, wall };
  }

  /**
   * Projects inlier points onto the best-fit line and extracts
   * the segment endpoints from the projection extremes.
   */
  fitLineSegment(inliers, line) {
    // Direction vector of the line: perpendicular to normal (a, b)
    const dirX = -line.b;
    const dirY = line.a;

    // Project each inlier onto the line direction
    let minT = Infinity;
    let maxT = -Infinity;

    for (const point of inliers) {
      const t = point.x * dirX + point.y * dirY;
      if (t < minT) minT = t;
      if (t > maxT) maxT = t;
    }

    // Reconstruct endpoints in world space
    // Find the point on the line closest to origin, then offset by t
    const originDist = -line.c; // signed distance from origin to line
    const baseX = line.a * originDist;
    const baseY = line.b * originDist;

    const startX = baseX + dirX * minT;
    const startY = baseY + dirY * minT;
    const endX = baseX + dirX * maxT;
    const endY = baseY + dirY * maxT;

    const length = Math.sqrt((endX - startX) ** 2 + (endY - startY) ** 2);
    const centerX = (startX + endX) / 2;
    const centerY = (startY + endY) / 2;
    const angle = Math.atan2(endY - startY, endX - startX);

    return {
      start: { x: startX, y: startY },
      end: { x: endX, y: endY },
      length,
      angle,
      center: { x: centerX, y: centerY },
      inlierCount: inliers.length,
      inlierRatio: inliers.length / (inliers.length + 1), // Approximate
    };
  }
}

module.exports = FeatureExtractor;
