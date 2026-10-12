/**
 * WallTracker: Temporal persistence and smoothing for RANSAC wall segments.
 *
 * Without this, RANSAC produces slightly different wall endpoints every frame
 * because of random sampling + sensor noise. This module solves jitter by:
 *
 * 1. Matching new wall candidates to existing tracked walls by spatial proximity
 * 2. Smoothing endpoint positions with EMA (Exponential Moving Average)
 * 3. Maintaining wall lifetime — walls persist even when briefly occluded
 * 4. Merging near-parallel overlapping walls into single segments
 *
 * The result: walls lock in place and stop vibrating.
 */
class WallTracker {
  constructor(config = {}) {
    this.config = {
      smoothing: 0.15,           // EMA alpha — lower = more stable, higher = more responsive
      persistence: 60,           // Frames before an unmatched wall disappears
      matchDistanceThreshold: 300, // mm — max centroid distance to consider a match
      matchAngleThreshold: 15,   // degrees — max angle difference to consider a match
      mergeDistanceThreshold: 150, // mm — merge walls closer than this
      mergeAngleThreshold: 10,   // degrees — merge walls within this angle
      minLifetimeToShow: 3,      // Don't render walls until they've been confirmed N times
      ...config,
    };

    this.trackedWalls = [];
    this.nextId = 0;
  }

  updateConfig(newConfig) {
    Object.assign(this.config, newConfig);
  }

  /**
   * Takes raw RANSAC walls and returns temporally-smoothed tracked walls.
   * @param {Array} candidateWalls - Fresh walls from FeatureExtractor
   * @returns {Array} Stable, smoothed walls for rendering
   */
  process(candidateWalls) {
    // Age all existing walls
    for (const wall of this.trackedWalls) {
      wall.lifetime--;
      wall.matched = false;
    }

    // Match each candidate to the nearest tracked wall
    for (const candidate of candidateWalls) {
      const match = this.findBestMatch(candidate);

      if (match) {
        this.smoothWall(match, candidate);
        match.lifetime = this.config.persistence;
        match.matched = true;
        match.hitCount = Math.min(200, (match.hitCount || 0) + 1);
      } else {
        // New wall — add to tracking
        this.trackedWalls.push({
          id: this.nextId++,
          start: { ...candidate.start },
          end: { ...candidate.end },
          length: candidate.length,
          angle: candidate.angle,
          center: { ...candidate.center },
          pointCount: candidate.pointCount,
          lifetime: this.config.persistence,
          matched: true,
          hitCount: 1,
        });
      }
    }

    // Prune dead walls
    this.trackedWalls = this.trackedWalls.filter(w => w.lifetime > 0);

    // Merge overlapping near-parallel walls
    this.mergeOverlapping();

    // Only return walls that have been confirmed enough times
    return this.trackedWalls
      .filter(w => w.hitCount >= this.config.minLifetimeToShow)
      .map(w => ({
        id: w.id,
        start: { x: w.start.x, y: w.start.y },
        end: { x: w.end.x, y: w.end.y },
        length: w.length,
        angle: w.angle,
        center: { x: w.center.x, y: w.center.y },
        pointCount: w.pointCount,
        confidence: Math.min(1.0, w.hitCount / 20), // 0-1 confidence for UI
      }));
  }

  /**
   * Find the best matching tracked wall for a candidate.
   * Matches by centroid distance AND angle similarity.
   */
  findBestMatch(candidate) {
    let bestWall = null;
    let bestScore = Infinity;

    const { matchDistanceThreshold, matchAngleThreshold } = this.config;

    for (const tracked of this.trackedWalls) {
      if (tracked.matched) continue; // Already claimed by another candidate

      // Centroid distance
      const distSq = (tracked.center.x - candidate.center.x) ** 2 +
                      (tracked.center.y - candidate.center.y) ** 2;
      const dist = Math.sqrt(distSq);

      if (dist > matchDistanceThreshold) continue;

      // Angle difference (handle wraparound)
      let angleDiff = Math.abs(tracked.angle - candidate.angle);
      // Lines are directionless — 0° and 180° are the same wall orientation
      if (angleDiff > Math.PI / 2) angleDiff = Math.PI - angleDiff;
      const angleDiffDeg = angleDiff * (180 / Math.PI);

      if (angleDiffDeg > matchAngleThreshold) continue;

      // Combined score: distance + angle penalty
      const score = dist + angleDiffDeg * 10;

      if (score < bestScore) {
        bestScore = score;
        bestWall = tracked;
      }
    }

    return bestWall;
  }

  /**
   * Smooth a tracked wall toward the new candidate using EMA.
   */
  smoothWall(tracked, candidate) {
    const alpha = this.config.smoothing;

    tracked.start.x = this.lerp(tracked.start.x, candidate.start.x, alpha);
    tracked.start.y = this.lerp(tracked.start.y, candidate.start.y, alpha);
    tracked.end.x = this.lerp(tracked.end.x, candidate.end.x, alpha);
    tracked.end.y = this.lerp(tracked.end.y, candidate.end.y, alpha);

    // Recalculate derived properties from smoothed endpoints
    tracked.center.x = (tracked.start.x + tracked.end.x) / 2;
    tracked.center.y = (tracked.start.y + tracked.end.y) / 2;
    tracked.length = Math.sqrt(
      (tracked.end.x - tracked.start.x) ** 2 +
      (tracked.end.y - tracked.start.y) ** 2
    );
    tracked.angle = Math.atan2(
      tracked.end.y - tracked.start.y,
      tracked.end.x - tracked.start.x
    );
    tracked.pointCount = Math.round(
      this.lerp(tracked.pointCount, candidate.pointCount, alpha)
    );
  }

  /**
   * Merge wall segments that belong to the same physical surface.
   *
   * The old approach (centroid distance) fails when a wall is split into two
   * halves by an obstacle — those halves can have centroids 1m+ apart even
   * though they're on the same line. The correct test is collinearity:
   *
   * Two segments belong to the same wall if:
   *   1. Their angles are similar (already the same orientation)
   *   2. The PERPENDICULAR distance from one segment's line to the other
   *      segment's centroid is small (they're on the same line in space)
   *   3. They're not too far apart ALONG the wall direction (not across the room)
   */
  mergeOverlapping() {
    const mergeAngleRad = this.config.mergeAngleThreshold * (Math.PI / 180);
    const maxPerpDist  = 80;   // mm — how collinear they must be
    const maxLongGap   = 800;  // mm — max gap between the two segments along the wall

    for (let i = 0; i < this.trackedWalls.length; i++) {
      const wallA = this.trackedWalls[i];
      if (!wallA) continue;

      for (let j = i + 1; j < this.trackedWalls.length; j++) {
        const wallB = this.trackedWalls[j];
        if (!wallB) continue;

        // 1. Angle similarity (lines are directionless — 0° == 180°)
        let angleDiff = Math.abs(wallA.angle - wallB.angle);
        if (angleDiff > Math.PI / 2) angleDiff = Math.PI - angleDiff;
        if (angleDiff > mergeAngleRad) continue;

        // Use wallA's direction as the reference axis
        const dirX = Math.cos(wallA.angle);
        const dirY = Math.sin(wallA.angle);
        const normX = -dirY;  // perpendicular to wall direction
        const normY =  dirX;

        // 2. Perpendicular distance from wallB's centroid to wallA's line
        const dx = wallB.center.x - wallA.center.x;
        const dy = wallB.center.y - wallA.center.y;
        const perpDist = Math.abs(dx * normX + dy * normY);
        if (perpDist > maxPerpDist) continue;

        // 3. Longitudinal gap — project both segments onto wallA's direction
        //    and measure the gap between their extents
        const refT = wallA.center.x * dirX + wallA.center.y * dirY;
        const aT = [
          wallA.start.x * dirX + wallA.start.y * dirY - refT,
          wallA.end.x   * dirX + wallA.end.y   * dirY - refT,
        ];
        const bT = [
          wallB.start.x * dirX + wallB.start.y * dirY - refT,
          wallB.end.x   * dirX + wallB.end.y   * dirY - refT,
        ];
        const aMin = Math.min(...aT), aMax = Math.max(...aT);
        const bMin = Math.min(...bT), bMax = Math.max(...bT);
        // Gap is positive when they don't overlap, 0 when they do
        const longGap = Math.max(0, Math.max(aMin, bMin) - Math.min(aMax, bMax));
        if (longGap > maxLongGap) continue;

        // --- Merge: keep the wall with more hits, absorb the other ---
        const keeper   = wallA.hitCount >= wallB.hitCount ? wallA : wallB;
        const absorbed = keeper === wallA ? wallB : wallA;

        // Reproject absorbed endpoints onto keeper's axis to find full extent
        const kDirX = Math.cos(keeper.angle);
        const kDirY = Math.sin(keeper.angle);
        const kRefT = keeper.center.x * kDirX + keeper.center.y * kDirY;

        const allT = [
          keeper.start.x  * kDirX + keeper.start.y  * kDirY - kRefT,
          keeper.end.x    * kDirX + keeper.end.y    * kDirY - kRefT,
          absorbed.start.x * kDirX + absorbed.start.y * kDirY - kRefT,
          absorbed.end.x   * kDirX + absorbed.end.y   * kDirY - kRefT,
        ];
        const minT = Math.min(...allT);
        const maxT = Math.max(...allT);

        keeper.start.x = keeper.center.x + kDirX * minT;
        keeper.start.y = keeper.center.y + kDirY * minT;
        keeper.end.x   = keeper.center.x + kDirX * maxT;
        keeper.end.y   = keeper.center.y + kDirY * maxT;
        keeper.length  = Math.sqrt(
          (keeper.end.x - keeper.start.x) ** 2 +
          (keeper.end.y - keeper.start.y) ** 2
        );
        keeper.center.x = (keeper.start.x + keeper.end.x) / 2;
        keeper.center.y = (keeper.start.y + keeper.end.y) / 2;
        keeper.hitCount += absorbed.hitCount;
        keeper.lifetime  = Math.max(keeper.lifetime, absorbed.lifetime);

        this.trackedWalls.splice(j, 1);
        j--;
      }
    }
  }

  lerp(a, b, t) {
    return a + (b - a) * t;
  }

  clear() {
    this.trackedWalls = [];
    this.nextId = 0;
  }
}

module.exports = WallTracker;
