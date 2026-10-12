/**
 * Clusterer: DBSCAN (Density-Based Spatial Clustering of Applications with Noise).
 * Groups nearby points into clusters. A cluster of hundreds of points is
 * likely a wall; a cluster of 3 points is noise.
 *
 * Uses grid-based spatial indexing for O(N) average-case neighbor lookups.
 */
class Clusterer {
  constructor(config = {}) {
    this.config = {
      epsilon: 100,     // mm — max distance between two points in the same cluster
      minPoints: 5,     // Minimum points to form a dense cluster
      ...config,
    };
  }

  updateConfig(newConfig) {
    Object.assign(this.config, newConfig);
  }

  /**
   * @param {Array<{x: number, y: number, angle: number, distance: number}>} points
   * @returns {Array<Array>} Array of clusters, each being an array of points
   */
  process(points) {
    if (!points || points.length === 0) return [];

    // Ensure cartesian coords
    const withCartesian = points.map(p => {
      if (p.x !== undefined && p.y !== undefined) return { ...p };
      const rad = (p.angle - 90) * (Math.PI / 180);
      return {
        ...p,
        x: p.distance * Math.cos(rad),
        y: p.distance * Math.sin(rad),
      };
    });

    const { epsilon, minPoints } = this.config;
    const epsilonSq = epsilon * epsilon;

    // Build spatial index for range queries
    const cellSize = epsilon;
    const spatialIndex = new Map();

    for (let i = 0; i < withCartesian.length; i++) {
      const point = withCartesian[i];
      const cellX = Math.floor(point.x / cellSize);
      const cellY = Math.floor(point.y / cellSize);
      const key = `${cellX}_${cellY}`;
      if (!spatialIndex.has(key)) spatialIndex.set(key, []);
      spatialIndex.get(key).push(i);
    }

    // DBSCAN labels: -1 = noise, 0+ = cluster ID
    const labels = new Int32Array(withCartesian.length).fill(-2); // -2 = unvisited
    let clusterId = 0;

    const regionQuery = (pointIdx) => {
      const point = withCartesian[pointIdx];
      const cellX = Math.floor(point.x / cellSize);
      const cellY = Math.floor(point.y / cellSize);
      const neighbors = [];

      for (let dx = -1; dx <= 1; dx++) {
        for (let dy = -1; dy <= 1; dy++) {
          const candidates = spatialIndex.get(`${cellX + dx}_${cellY + dy}`);
          if (!candidates) continue;

          for (const candidateIdx of candidates) {
            if (candidateIdx === pointIdx) continue;
            const candidate = withCartesian[candidateIdx];
            const distSq = (point.x - candidate.x) ** 2 + (point.y - candidate.y) ** 2;
            if (distSq <= epsilonSq) {
              neighbors.push(candidateIdx);
            }
          }
        }
      }

      return neighbors;
    };

    for (let i = 0; i < withCartesian.length; i++) {
      if (labels[i] !== -2) continue; // Already processed

      const neighbors = regionQuery(i);

      if (neighbors.length < minPoints) {
        labels[i] = -1; // Noise
        continue;
      }

      // Start a new cluster
      labels[i] = clusterId;
      const seedQueue = [...neighbors];
      let queueIdx = 0;

      while (queueIdx < seedQueue.length) {
        const neighborIdx = seedQueue[queueIdx++];

        if (labels[neighborIdx] === -1) {
          // Was noise, now border point
          labels[neighborIdx] = clusterId;
          continue;
        }
        if (labels[neighborIdx] !== -2) continue; // Already in a cluster

        labels[neighborIdx] = clusterId;

        const newNeighbors = regionQuery(neighborIdx);
        if (newNeighbors.length >= minPoints) {
          seedQueue.push(...newNeighbors);
        }
      }

      clusterId++;
    }

    // Group points by cluster, skip noise
    const clusters = [];
    for (let id = 0; id < clusterId; id++) {
      const clusterPoints = [];
      for (let i = 0; i < withCartesian.length; i++) {
        if (labels[i] === id) {
          clusterPoints.push(withCartesian[i]);
        }
      }
      if (clusterPoints.length >= minPoints) {
        clusters.push(clusterPoints);
      }
    }

    return clusters;
  }
}

module.exports = Clusterer;
