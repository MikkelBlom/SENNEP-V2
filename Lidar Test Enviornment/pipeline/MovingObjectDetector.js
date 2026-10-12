/**
 * MovingObjectDetector: Detects objects that differ from the stable environment.
 *
 * How it works:
 * The WallExtractor already maintains a stable polar distance profile.
 * By comparing each incoming point's distance against the profile at that
 * angle, we can identify points that are significantly closer (something
 * blocking the beam) or significantly farther (something that moved away).
 *
 * Only "closer" anomalies are interesting — they represent objects in front
 * of walls. "Farther" anomalies usually mean a door opened or the wall
 * profile hasn't converged yet.
 *
 * Anomalous points are clustered into moving objects by angular proximity.
 * Objects are tracked across frames by nearest-centroid matching so each
 * object keeps a persistent ID and a position trail.
 */
class MovingObjectDetector {
  constructor(config = {}) {
    this.config = {
      enabled: true,
      distanceThreshold: 100,    // mm — how far from profile to be "anomalous"
      relativeThreshold: 0.15,   // 15% — relative distance difference
      minObjectPoints: 3,        // Minimum points to form an object
      maxObjectGapDeg: 5,        // Max angular gap between points in same object
      onlyCloser: true,          // Only detect objects closer than walls (not farther)
      cooldownSweeps: 5,         // Don't detect until profile has N sweeps
      maxTrailLength: 80,        // Max number of past positions to keep per object
      matchThreshold: 600,       // mm — max centroid distance to match across frames
      maxMissedFrames: 15,       // Frames before an unmatched track is deleted
      ...config,
    };

    // Persistent tracking state
    this.trackedObjects = new Map(); // trackId → { center, trail, missedFrames }
    this.nextId = 0;
  }

  updateConfig(newConfig) {
    Object.assign(this.config, newConfig);
  }

  // ---------------------------------------------------------------------------
  // Internal helpers
  // ---------------------------------------------------------------------------

  /**
   * Cluster an array of anomaly points by angular proximity.
   * Points must have { angle } field. Returns arrays of clusters.
   */
  _cluster(points) {
    const { minObjectPoints, maxObjectGapDeg } = this.config;
    const sorted = points.slice().sort((a, b) => a.angle - b.angle);

    const clusters = [];
    let current = [sorted[0]];

    for (let i = 1; i < sorted.length; i++) {
      if (sorted[i].angle - sorted[i - 1].angle <= maxObjectGapDeg) {
        current.push(sorted[i]);
      } else {
        if (current.length >= minObjectPoints) clusters.push(current);
        current = [sorted[i]];
      }
    }
    if (current.length >= minObjectPoints) clusters.push(current);

    return clusters;
  }

  /**
   * Build a raw object descriptor from a cluster of anomaly points.
   * The cluster points must have { x, y, angle, diff } fields.
   */
  _buildDescriptor(cluster) {
    let cx = 0, cy = 0;
    let minAngle = Infinity, maxAngle = -Infinity;

    for (const p of cluster) {
      cx += p.x; cy += p.y;
      if (p.angle < minAngle) minAngle = p.angle;
      if (p.angle > maxAngle) maxAngle = p.angle;
    }
    cx /= cluster.length;
    cy /= cluster.length;

    let maxR = 0;
    for (const p of cluster) {
      const r = Math.sqrt((p.x - cx) ** 2 + (p.y - cy) ** 2);
      if (r > maxR) maxR = r;
    }

    return {
      center: { x: cx, y: cy },
      radius: Math.max(30, maxR),
      pointCount: cluster.length,
      angleSpan: maxAngle - minAngle,
      avgDiff: cluster.reduce((s, p) => s + p.diff, 0) / cluster.length,
      points: cluster.map(p => ({ x: p.x, y: p.y })),
    };
  }

  /**
   * Match raw descriptors to the existing tracked objects, update trails,
   * age out lost tracks, and return the final output list with stable IDs
   * and trail arrays.
   */
  _matchAndTrack(rawObjects) {
    const { matchThreshold, maxTrailLength, maxMissedFrames } = this.config;
    const matched = new Set();
    const result = [];

    for (const obj of rawObjects) {
      // Find the nearest existing track within matchThreshold
      let bestId = null;
      let bestDist = matchThreshold;

      for (const [id, tracked] of this.trackedObjects) {
        if (matched.has(id)) continue;
        const dx = obj.center.x - tracked.center.x;
        const dy = obj.center.y - tracked.center.y;
        const dist = Math.sqrt(dx * dx + dy * dy);
        if (dist < bestDist) { bestDist = dist; bestId = id; }
      }

      if (bestId !== null) {
        // Continuing an existing track
        matched.add(bestId);
        const tracked = this.trackedObjects.get(bestId);
        const trail = [...tracked.trail, { x: obj.center.x, y: obj.center.y }]
          .slice(-maxTrailLength);
        this.trackedObjects.set(bestId, { center: obj.center, trail, missedFrames: 0 });
        result.push({ ...obj, id: bestId, trail });
      } else {
        // New detection — start a fresh track
        const id = this.nextId++;
        const trail = [{ x: obj.center.x, y: obj.center.y }];
        this.trackedObjects.set(id, { center: obj.center, trail, missedFrames: 0 });
        result.push({ ...obj, id, trail });
      }
    }

    // Age out tracks that weren't matched this frame
    for (const [id, tracked] of this.trackedObjects) {
      if (!matched.has(id)) {
        tracked.missedFrames = (tracked.missedFrames || 0) + 1;
        if (tracked.missedFrames > maxMissedFrames) {
          this.trackedObjects.delete(id);
        }
      }
    }

    return result;
  }

  // ---------------------------------------------------------------------------
  // Public API
  // ---------------------------------------------------------------------------

  /**
   * Detect moving objects by comparing current sweep to the stable profile.
   * Legacy entry point — still used if called directly (e.g. tests).
   * @param {Array} currentPoints - Filtered points from this sweep (angle, distance, x, y)
   * @param {WallExtractor} wallExtractor - Reference to get the stable profile
   * @returns {Array<MovingObject>}
   */
  process(currentPoints, wallExtractor) {
    if (!this.config.enabled) return [];
    if (!wallExtractor || wallExtractor.sweepsProcessed < this.config.cooldownSweeps) return [];
    if (!currentPoints || currentPoints.length === 0) return [];

    const { distanceThreshold, relativeThreshold, onlyCloser } = this.config;
    const { binResolution, profile, profileValid, binCount } = wallExtractor;

    const anomalies = [];
    for (const point of currentPoints) {
      if (!point.angle || point.distance <= 0) continue;
      const binIndex = Math.floor(point.angle / binResolution) % binCount;
      if (!profileValid[binIndex]) continue;

      const profileDist = profile[binIndex];
      const diff = point.distance - profileDist;
      const absDiff = Math.abs(diff);
      const relativeDiff = absDiff / profileDist;

      if (absDiff < distanceThreshold && relativeDiff < relativeThreshold) continue;
      if (onlyCloser && diff > 0) continue;

      anomalies.push({ angle: point.angle, distance: point.distance, x: point.x, y: point.y, diff, profileDist });
    }

    if (anomalies.length < this.config.minObjectPoints) return [];

    const clusters = this._cluster(anomalies);
    const raw = clusters.map(c => this._buildDescriptor(c));
    return this._matchAndTrack(raw);
  }

  /**
   * Detect moving objects from pre-classified transient points.
   *
   * Points already carry { diff, profileDist, binIndex } from WallExtractor.classifyPoints().
   * Classification was done against the stable (pre-update) profile, so this is
   * inherently accurate. Skips the profile-comparison step and goes straight to clustering.
   *
   * Also maintains persistent object IDs and builds a position trail for each object.
   *
   * @param {Array} transientPoints - Output of WallExtractor.classifyPoints().transient
   * @returns {Array<MovingObject>}   Each object has { id, center, radius, avgDiff, points, trail }
   */
  processTransients(transientPoints) {
    if (!this.config.enabled) return [];
    if (!transientPoints || transientPoints.length === 0) {
      // Still age out stale tracks even when nothing is detected
      this._matchAndTrack([]);
      return [];
    }

    const clusters = this._cluster(transientPoints);
    if (clusters.length === 0) {
      this._matchAndTrack([]);
      return [];
    }

    const raw = clusters.map(c => this._buildDescriptor(c));
    return this._matchAndTrack(raw);
  }

  clear() {
    this.trackedObjects.clear();
    this.nextId = 0;
  }
}

module.exports = MovingObjectDetector;
