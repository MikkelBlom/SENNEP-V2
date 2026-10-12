/**
 * ScanMatcher: Iterative Closest Point (ICP) for frame-to-frame alignment.
 *
 * Compares the current scan to the previous scan and computes the optimal
 * rotation + translation that aligns them. This compensates for motor
 * micro-stutters (stationary) and will be critical for mobile operation.
 *
 * Algorithm:
 * 1. For each point in the current scan, find the closest point in the previous scan
 * 2. Compute the optimal rigid transformation (rotation + translation) that minimizes
 *    the sum of squared distances between matched point pairs
 * 3. Apply the transformation and repeat until convergence
 */
class ScanMatcher {
  constructor(config = {}) {
    this.config = {
      enabled: true,
      maxIterations: 30,
      convergenceTranslation: 1.0,  // mm — stop if translation delta < this
      convergenceRotation: 0.001,   // radians (~0.06°) — stop if rotation delta < this
      maxCorrespondenceDistance: 500, // mm — ignore matches further than this
      ...config,
    };

    this.previousScan = null;
    this.cumulativePose = { x: 0, y: 0, theta: 0 };
  }

  updateConfig(newConfig) {
    Object.assign(this.config, newConfig);
  }

  /**
   * @param {Array<{x: number, y: number}>} currentPoints - Current scan in sensor frame
   * @returns {{pose: {x, y, theta}, converged: boolean, iterations: number}}
   */
  process(currentPoints) {
    if (!this.config.enabled || !currentPoints || currentPoints.length < 10) {
      this.previousScan = currentPoints;
      return {
        pose: { ...this.cumulativePose },
        converged: false,
        iterations: 0,
      };
    }

    // Ensure cartesian coords
    const currentCartesian = currentPoints.map(p => {
      if (p.x !== undefined && p.y !== undefined) return { x: p.x, y: p.y };
      const rad = (p.angle - 90) * (Math.PI / 180);
      return { x: p.distance * Math.cos(rad), y: p.distance * Math.sin(rad) };
    });

    if (!this.previousScan || this.previousScan.length < 10) {
      this.previousScan = currentCartesian;
      return {
        pose: { ...this.cumulativePose },
        converged: true,
        iterations: 0,
      };
    }

    // Build spatial index for the previous scan (reference)
    const cellSize = 100;
    const refIndex = new Map();
    for (const point of this.previousScan) {
      const key = `${Math.floor(point.x / cellSize)}_${Math.floor(point.y / cellSize)}`;
      if (!refIndex.has(key)) refIndex.set(key, []);
      refIndex.get(key).push(point);
    }

    // ICP iteration
    let transformedPoints = currentCartesian.map(p => ({ ...p }));
    let totalRotation = 0;
    let totalTx = 0;
    let totalTy = 0;
    let converged = false;
    let iterations = 0;

    const { maxIterations, convergenceTranslation, convergenceRotation, maxCorrespondenceDistance } = this.config;
    const maxCorrDistSq = maxCorrespondenceDistance * maxCorrespondenceDistance;

    for (let iter = 0; iter < maxIterations; iter++) {
      iterations++;

      // Step 1: Find closest point correspondences
      const matches = [];

      for (const point of transformedPoints) {
        const best = this.findClosest(point, refIndex, cellSize, maxCorrDistSq);
        if (best) {
          matches.push({ source: point, target: best });
        }
      }

      if (matches.length < 5) break;

      // Step 2: Compute optimal rotation + translation (SVD-free closed-form for 2D)
      const transform = this.computeRigidTransform(matches);

      if (!transform) break;

      const { rotation, tx, ty } = transform;

      // Step 3: Apply transformation
      const cos = Math.cos(rotation);
      const sin = Math.sin(rotation);

      for (const point of transformedPoints) {
        const newX = cos * point.x - sin * point.y + tx;
        const newY = sin * point.x + cos * point.y + ty;
        point.x = newX;
        point.y = newY;
      }

      totalRotation += rotation;
      totalTx += tx;
      totalTy += ty;

      // Check convergence
      const translationDelta = Math.sqrt(tx * tx + ty * ty);
      if (translationDelta < convergenceTranslation && Math.abs(rotation) < convergenceRotation) {
        converged = true;
        break;
      }
    }

    // Update cumulative pose
    const cos = Math.cos(this.cumulativePose.theta);
    const sin = Math.sin(this.cumulativePose.theta);
    this.cumulativePose.x += cos * totalTx - sin * totalTy;
    this.cumulativePose.y += sin * totalTx + cos * totalTy;
    this.cumulativePose.theta += totalRotation;

    // Store current scan for next iteration
    this.previousScan = currentCartesian;

    return {
      pose: { ...this.cumulativePose },
      converged,
      iterations,
    };
  }

  /**
   * Find the closest point in the reference index to the query point.
   */
  findClosest(query, refIndex, cellSize, maxDistSq) {
    const cx = Math.floor(query.x / cellSize);
    const cy = Math.floor(query.y / cellSize);

    let bestPoint = null;
    let bestDistSq = maxDistSq;

    for (let dx = -1; dx <= 1; dx++) {
      for (let dy = -1; dy <= 1; dy++) {
        const candidates = refIndex.get(`${cx + dx}_${cy + dy}`);
        if (!candidates) continue;

        for (const candidate of candidates) {
          const distSq = (query.x - candidate.x) ** 2 + (query.y - candidate.y) ** 2;
          if (distSq < bestDistSq) {
            bestDistSq = distSq;
            bestPoint = candidate;
          }
        }
      }
    }

    return bestPoint;
  }

  /**
   * Compute optimal 2D rigid transformation from matched point pairs.
   * Uses the closed-form solution based on centroids and cross-covariance.
   */
  computeRigidTransform(matches) {
    const n = matches.length;
    if (n < 3) return null;

    // Compute centroids
    let srcCx = 0, srcCy = 0, tgtCx = 0, tgtCy = 0;
    for (const match of matches) {
      srcCx += match.source.x;
      srcCy += match.source.y;
      tgtCx += match.target.x;
      tgtCy += match.target.y;
    }
    srcCx /= n; srcCy /= n;
    tgtCx /= n; tgtCy /= n;

    // Compute cross-covariance elements
    let sxx = 0, sxy = 0, syx = 0, syy = 0;
    for (const match of matches) {
      const sx = match.source.x - srcCx;
      const sy = match.source.y - srcCy;
      const tx = match.target.x - tgtCx;
      const ty = match.target.y - tgtCy;

      sxx += sx * tx;
      sxy += sx * ty;
      syx += sy * tx;
      syy += sy * ty;
    }

    // Optimal rotation: theta = atan2(syx - sxy, sxx + syy)
    const rotation = Math.atan2(syx - sxy, sxx + syy);
    const cos = Math.cos(rotation);
    const sin = Math.sin(rotation);

    // Optimal translation
    const tx = tgtCx - (cos * srcCx - sin * srcCy);
    const ty = tgtCy - (sin * srcCx + cos * srcCy);

    return { rotation, tx, ty };
  }

  reset() {
    this.previousScan = null;
    this.cumulativePose = { x: 0, y: 0, theta: 0 };
  }
}

module.exports = ScanMatcher;
