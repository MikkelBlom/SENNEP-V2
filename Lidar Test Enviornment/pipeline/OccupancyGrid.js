/**
 * OccupancyGrid: Probabilistic map using log-odds Bayesian updating.
 *
 * Each cell stores a log-odds value representing the probability of being occupied.
 * - Beams that HIT a cell increase its log-odds (more likely a wall).
 * - Beams that PASS THROUGH a cell decrease its log-odds (more likely empty).
 * - Moving objects produce transient hits that decay as beams clear them.
 *
 * Uses Bresenham's line algorithm for raycasting (O(1) per cell traversed)
 * and a Map-based sparse representation — only occupied/uncertain cells are stored.
 */
class OccupancyGrid {
  constructor(config = {}) {
    this.config = {
      cellSize: 50,            // mm per grid cell
      hitIncrement: 0.9,       // Log-odds increase on hit (~+20% probability)
      missDecrement: 0.4,      // Log-odds decrease on miss (~-10% probability)
      logOddsMax: 5.0,         // Clamp ceiling (prevents permanent walls)
      logOddsMin: -2.0,        // Clamp floor (prevents permanent empty)
      occupiedThreshold: 1.5,  // Log-odds above this = "solid" cell
      decayRate: 0.02,         // Natural decay per sweep (slow forgetting)
      decayInterval: 5,        // Run decay every N sweeps
      raycastEnabled: true,
      ...config,
    };

    // Sparse grid: key "gx_gy" → { logOdds, x, y }
    this.grid = new Map();
    this.sweepCount = 0;
  }

  updateConfig(newConfig) {
    if (newConfig.cellSize && newConfig.cellSize !== this.config.cellSize) {
      this.grid.clear(); // Resolution change invalidates the grid
    }
    Object.assign(this.config, newConfig);
  }

  /**
   * Update the grid with a new sweep of filtered points.
   * @param {Array<{x: number, y: number, angle: number, distance: number}>} points
   * @param {{x: number, y: number, theta: number}} sensorPose
   */
  update(points, sensorPose = { x: 0, y: 0, theta: 0 }) {
    if (!points || points.length === 0) return;

    this.sweepCount++;
    const { cellSize, hitIncrement, missDecrement, logOddsMax, logOddsMin, raycastEnabled } = this.config;

    for (let i = 0; i < points.length; i++) {
      const point = points[i];
      
      // Points already have x,y in sensor-relative coordinates from PreFilter.
      // For a stationary sensor this IS the world frame.
      // When mobile, the Pipeline will pre-transform points into world frame
      // before calling update() — so we never apply pose offsets here.
      let worldX, worldY;
      if (point.x !== undefined && point.y !== undefined) {
        worldX = point.x;
        worldY = point.y;
      } else {
        const rad = (point.angle - 90) * (Math.PI / 180);
        worldX = point.distance * Math.cos(rad);
        worldY = point.distance * Math.sin(rad);
      }

      // HIT: Increase occupancy at the endpoint
      const hitGx = Math.floor(worldX / cellSize);
      const hitGy = Math.floor(worldY / cellSize);
      this.incrementCell(hitGx, hitGy, hitIncrement, logOddsMax, cellSize);

      // MISS: Raycast from sensor to hit point, decrease cells along the beam
      // Only process every 3rd point to save CPU
      if (raycastEnabled && i % 3 === 0) {
        const sensorGx = 0;
        const sensorGy = 0;
        this.bresenhamRaycast(sensorGx, sensorGy, hitGx, hitGy, missDecrement, logOddsMin, cellSize);
      }
    }

    // Periodic decay — prevents stale data from lingering forever
    if (this.sweepCount % this.config.decayInterval === 0) {
      this.decay();
    }
  }

  incrementCell(gx, gy, increment, clamp, cellSize) {
    const key = `${gx}_${gy}`;
    let cell = this.grid.get(key);

    if (!cell) {
      cell = {
        logOdds: 0,
        x: (gx + 0.5) * cellSize,
        y: (gy + 0.5) * cellSize,
      };
    }

    cell.logOdds = Math.min(clamp, cell.logOdds + increment);
    this.grid.set(key, cell);
  }

  /**
   * Bresenham's line algorithm: efficiently walks the grid cells
   * between the sensor and the hit point, decrementing each one.
   */
  bresenhamRaycast(x0, y0, x1, y1, decrement, clamp, cellSize) {
    let dx = Math.abs(x1 - x0);
    let dy = Math.abs(y1 - y0);
    const sx = x0 < x1 ? 1 : -1;
    const sy = y0 < y1 ? 1 : -1;
    let err = dx - dy;

    // Don't process the hit cell (that gets a HIT, not a MISS)
    while (x0 !== x1 || y0 !== y1) {
      // Decrement this cell (beam passed through it)
      const key = `${x0}_${y0}`;
      const cell = this.grid.get(key);
      if (cell) {
        cell.logOdds = Math.max(clamp, cell.logOdds - decrement);
        if (cell.logOdds <= clamp) {
          this.grid.delete(key); // Prune fully-empty cells
        }
      }

      const e2 = 2 * err;
      if (e2 > -dy) { err -= dy; x0 += sx; }
      if (e2 < dx) { err += dx; y0 += sy; }
    }
  }

  /**
   * Natural decay: slowly reduces all cells to prevent stale data.
   */
  decay() {
    const { decayRate } = this.config;
    for (const [key, cell] of this.grid) {
      cell.logOdds -= decayRate;
      if (cell.logOdds <= this.config.logOddsMin) {
        this.grid.delete(key);
      }
    }
  }

  /**
   * Returns the current grid state for rendering.
   * Only includes cells above a visibility threshold.
   */
  getSnapshot() {
    const cells = [];
    const visibilityThreshold = -1.0; // Show uncertain cells too

    for (const cell of this.grid.values()) {
      if (cell.logOdds > visibilityThreshold) {
        // Convert log-odds to probability: p = 1 / (1 + exp(-logOdds))
        const probability = 1.0 / (1.0 + Math.exp(-cell.logOdds));
        cells.push({
          x: cell.x,
          y: cell.y,
          probability,
          logOdds: cell.logOdds,
        });
      }
    }

    return {
      cells,
      cellSize: this.config.cellSize,
      totalCells: this.grid.size,
    };
  }

  clear() {
    this.grid.clear();
    this.sweepCount = 0;
  }
}

module.exports = OccupancyGrid;
