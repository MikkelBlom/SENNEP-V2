//! Log-odds occupancy grid. Each cell holds an i8 log-odds value (scaled by 20, so ±127 = ±6.35).
//! Beams mark their end cell as hit and every cell they pass through as free, so anything that
//! moves away (a person) is erased by later beams instead of leaving a ghost trail.
//! The UI receives only tiles that changed since its last update.

use crate::geom::Pose2;

pub const TILE: usize = 32;

#[derive(Clone, Debug)]
pub struct GridConfig {
    pub resolution: f32,
    /// Map size in metres (square, centred on the start pose).
    pub size: f32,
    pub hit: i8,
    pub miss: i8,
    pub clamp: i8,
    /// Beams longer than this only clear space, they don't mark hits (far returns are noisy).
    pub max_hit_range: f32,
}

impl Default for GridConfig {
    fn default() -> Self {
        GridConfig { resolution: 0.05, size: 40.0, hit: 18, miss: -5, clamp: 100, max_hit_range: 8.0 }
    }
}

pub struct Grid {
    pub cfg: GridConfig,
    pub dim: usize,
    pub cells: Vec<i8>,
    /// World coordinates of cell (0, 0)'s corner.
    pub origin: (f32, f32),
    dirty: Vec<bool>,
    tiles_per_side: usize,
    stamp: Vec<u32>,
    sweep_id: u32,
}

impl Grid {
    pub fn new(cfg: GridConfig) -> Grid {
        let tiles = ((cfg.size / cfg.resolution) as usize).div_ceil(TILE);
        let dim = tiles * TILE;
        let half = dim as f32 * cfg.resolution * 0.5;
        Grid {
            cells: vec![0; dim * dim],
            stamp: vec![0; dim * dim],
            dirty: vec![false; tiles * tiles],
            tiles_per_side: tiles,
            origin: (-half, -half),
            dim,
            cfg,
            sweep_id: 0,
        }
    }

    pub fn tiles_per_side(&self) -> usize {
        self.tiles_per_side
    }

    fn cell_of(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        let cx = ((x - self.origin.0) / self.cfg.resolution).floor();
        let cy = ((y - self.origin.1) / self.cfg.resolution).floor();
        (cx >= 0.0 && cy >= 0.0 && (cx as usize) < self.dim && (cy as usize) < self.dim)
            .then_some((cx as usize, cy as usize))
    }

    pub fn cell_center(&self, cx: usize, cy: usize) -> (f32, f32) {
        (
            self.origin.0 + (cx as f32 + 0.5) * self.cfg.resolution,
            self.origin.1 + (cy as f32 + 0.5) * self.cfg.resolution,
        )
    }

    pub fn value(&self, cx: usize, cy: usize) -> i8 {
        self.cells[cy * self.dim + cx]
    }

    /// Integrates one scan. `points` are in the robot frame at `pose`; `sensor` is the LiDAR's
    /// position in the robot frame.
    pub fn integrate(&mut self, pose: &Pose2, sensor: (f32, f32), points: &[(f32, f32)]) {
        self.sweep_id = self.sweep_id.wrapping_add(1).max(1);
        let (sx, sy) = pose.apply(sensor.0 as f64, sensor.1 as f64);
        let (sx, sy) = (sx as f32, sy as f32);
        let res = self.cfg.resolution;
        // Pass 1: free space along every beam (each cell at most once per scan).
        for &(px, py) in points {
            let (wx, wy) = pose.apply(px as f64, py as f64);
            self.trace(sx, sy, wx as f32, wy as f32, res);
        }
        // Pass 2: hits win over misses from the same scan.
        for &(px, py) in points {
            let r = (px - sensor.0).hypot(py - sensor.1);
            if r > self.cfg.max_hit_range {
                continue;
            }
            let (wx, wy) = pose.apply(px as f64, py as f64);
            if let Some((cx, cy)) = self.cell_of(wx as f32, wy as f32) {
                let i = cy * self.dim + cx;
                let undo_miss = if self.stamp[i] == self.sweep_id { -(self.cfg.miss as i16) } else { 0 };
                self.stamp[i] = self.sweep_id.wrapping_add(1 << 31); // hit marker: don't hit twice
                let v = self.cells[i] as i16 + self.cfg.hit as i16 + undo_miss;
                self.set(i, cx, cy, v);
            }
        }
    }

    fn set(&mut self, i: usize, cx: usize, cy: usize, v: i16) {
        let c = self.cfg.clamp as i16;
        let v = v.clamp(-c, c) as i8;
        if self.cells[i] != v {
            self.cells[i] = v;
            self.dirty[(cy / TILE) * self.tiles_per_side + cx / TILE] = true;
        }
    }

    /// Amanatides–Woo voxel traversal; marks every cell before the end cell as free.
    fn trace(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, res: f32) {
        let Some((mut cx, mut cy)) = self.cell_of(x0, y0) else {
            return;
        };
        let end = self.cell_of(x1, y1);
        let (dx, dy) = (x1 - x0, y1 - y0);
        let len = dx.hypot(dy);
        if len < 1e-6 {
            return;
        }
        let (step_x, step_y) = (if dx >= 0.0 { 1i64 } else { -1 }, if dy >= 0.0 { 1i64 } else { -1 });
        let next_bx = self.origin.0 + (cx as f32 + if dx >= 0.0 { 1.0 } else { 0.0 }) * res;
        let next_by = self.origin.1 + (cy as f32 + if dy >= 0.0 { 1.0 } else { 0.0 }) * res;
        let mut t_max_x = if dx.abs() > 1e-9 { (next_bx - x0) / dx } else { f32::INFINITY };
        let mut t_max_y = if dy.abs() > 1e-9 { (next_by - y0) / dy } else { f32::INFINITY };
        let t_dx = if dx.abs() > 1e-9 { res / dx.abs() } else { f32::INFINITY };
        let t_dy = if dy.abs() > 1e-9 { res / dy.abs() } else { f32::INFINITY };
        let max_steps = (len / res) as usize * 2 + 2;
        for _ in 0..max_steps {
            if Some((cx, cy)) == end {
                return;
            }
            let i = cy * self.dim + cx;
            if self.stamp[i] != self.sweep_id && self.stamp[i] != self.sweep_id.wrapping_add(1 << 31) {
                self.stamp[i] = self.sweep_id;
                let v = self.cells[i] as i16 + self.cfg.miss as i16;
                self.set(i, cx, cy, v);
            }
            if t_max_x < t_max_y {
                if t_max_x > 1.0 {
                    return;
                }
                t_max_x += t_dx;
                let n = cx as i64 + step_x;
                if n < 0 || n >= self.dim as i64 {
                    return;
                }
                cx = n as usize;
            } else {
                if t_max_y > 1.0 {
                    return;
                }
                t_max_y += t_dy;
                let n = cy as i64 + step_y;
                if n < 0 || n >= self.dim as i64 {
                    return;
                }
                cy = n as usize;
            }
        }
    }

    /// Takes the list of changed tiles (tx, ty) and clears the dirty flags.
    pub fn take_dirty(&mut self) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        for (i, d) in self.dirty.iter_mut().enumerate() {
            if *d {
                *d = false;
                out.push((i % self.tiles_per_side, i / self.tiles_per_side));
            }
        }
        out
    }

    pub fn mark_all_dirty(&mut self) {
        // Only tiles with content matter to a newly connected viewer.
        for ty in 0..self.tiles_per_side {
            for tx in 0..self.tiles_per_side {
                let any = (0..TILE).any(|r| {
                    let row = (ty * TILE + r) * self.dim + tx * TILE;
                    self.cells[row..row + TILE].iter().any(|&v| v != 0)
                });
                if any {
                    self.dirty[ty * self.tiles_per_side + tx] = true;
                }
            }
        }
    }

    /// Copies one tile's cells (row-major, TILE x TILE).
    pub fn tile(&self, tx: usize, ty: usize, out: &mut Vec<u8>) {
        for r in 0..TILE {
            let row = (ty * TILE + r) * self.dim + tx * TILE;
            out.extend(self.cells[row..row + TILE].iter().map(|&v| v as u8));
        }
    }

    pub fn occupied_threshold() -> i8 {
        20
    }
}
