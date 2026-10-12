//! Walls as straight segments, fitted to the occupancy grid (not to single scans).
//!
//! Why the old app's lines jumped: it fitted lines to each revolution (~450 noisy points) and threw
//! them away 0.1 s later. Here the input is the grid, which already averages hundreds of scans at
//! corrected poses, and every wall keeps an id: each update first re-fits the walls it already has
//! (to the cells near them) and only then looks for new ones in what is left. Endpoints and line
//! parameters are smoothed, so a wall only moves when the map itself changes.
//!
//! What is left after the walls (occupied cells that are not part of a straight run) becomes
//! "objects": furniture, plant pots, chair legs, drawn as outlines.

use crate::grid::Grid;

#[derive(Clone, Debug)]
pub struct WallConfig {
    /// Cells at or above this log-odds count as solid (the grid's "occupied" is 20; walls need more evidence).
    pub min_value: i8,
    /// Max distance from the line for a cell centre to belong to a wall (> half a diagonal cell step).
    pub inlier_tol: f32,
    /// Shortest wall, metres. Shorter straight runs stay objects.
    pub min_len: f32,
    pub min_cells: usize,
    /// A gap longer than this along the line splits it into two walls (doorways).
    pub max_gap: f32,
    /// How far an existing wall may grow per update, metres.
    pub grow: f32,
    /// Updates a wall may go unconfirmed before it is dropped.
    pub keep_missing: u32,
    /// Smoothing of line and endpoints per update (0 = frozen, 1 = no smoothing).
    pub alpha: f32,
    /// Endpoints of two walls closer than this (and at an angle) are joined at their intersection.
    pub corner_snap: f32,
}

impl Default for WallConfig {
    fn default() -> Self {
        WallConfig {
            min_value: 40,
            inlier_tol: 0.055,
            min_len: 0.35,
            min_cells: 6,
            max_gap: 0.22,
            grow: 0.6,
            keep_missing: 4,
            alpha: 0.35,
            corner_snap: 0.18,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Wall {
    pub id: u32,
    /// Unit direction and a point on the line (centroid of its cells).
    pub dir: (f32, f32),
    pub c: (f32, f32),
    /// Extent along `dir`, relative to `c`.
    pub t0: f32,
    pub t1: f32,
    pub cells: usize,
    pub missed: u32,
    pub age: u32,
}

impl Wall {
    pub fn a(&self) -> (f32, f32) {
        (self.c.0 + self.dir.0 * self.t0, self.c.1 + self.dir.1 * self.t0)
    }
    pub fn b(&self) -> (f32, f32) {
        (self.c.0 + self.dir.0 * self.t1, self.c.1 + self.dir.1 * self.t1)
    }
    pub fn len(&self) -> f32 {
        self.t1 - self.t0
    }
    fn dist(&self, p: (f32, f32)) -> f32 {
        let (dx, dy) = (p.0 - self.c.0, p.1 - self.c.1);
        (dx * -self.dir.1 + dy * self.dir.0).abs()
    }
    fn proj(&self, p: (f32, f32)) -> f32 {
        (p.0 - self.c.0) * self.dir.0 + (p.1 - self.c.1) * self.dir.1
    }
}

/// A drawn wall: endpoints after corner joining.
#[derive(Clone, Copy, Debug)]
pub struct WallOut {
    pub id: u32,
    pub a: (f32, f32),
    pub b: (f32, f32),
}

#[derive(Clone, Debug)]
pub struct Object {
    /// Convex outline, counter-clockwise, metres.
    pub hull: Vec<(f32, f32)>,
    pub cells: usize,
}

pub struct WallMap {
    pub cfg: WallConfig,
    pub walls: Vec<Wall>,
    pub objects: Vec<Object>,
    next_id: u32,
    pts: Vec<(f32, f32)>,
    used: Vec<bool>,
    /// Corner joins from the last update: (wall id, end 0/1) -> other wall id. Kept with a looser
    /// distance so a corner does not flicker between joined and open.
    joins: std::collections::HashMap<(u32, u8), u32>,
    out: Vec<WallOut>,
}

impl WallMap {
    pub fn new(cfg: WallConfig) -> Self {
        WallMap {
            cfg,
            walls: Vec::new(),
            objects: Vec::new(),
            next_id: 1,
            pts: Vec::new(),
            used: Vec::new(),
            joins: Default::default(),
            out: Vec::new(),
        }
    }

    /// Re-fits the walls to the current grid. ~1–3 ms for a flat; call a few times per second.
    pub fn update(&mut self, grid: &Grid) {
        self.collect(grid);
        let res = grid.cfg.resolution;
        self.used.clear();
        self.used.resize(self.pts.len(), false);

        // 1. Existing walls first (longest first), so the same cells keep the same id.
        let mut order: Vec<usize> = (0..self.walls.len()).collect();
        order.sort_by(|&a, &b| self.walls[b].len().total_cmp(&self.walls[a].len()));
        let mut keep = vec![true; self.walls.len()];
        for wi in order {
            let w = self.walls[wi].clone();
            let near: Vec<usize> = (0..self.pts.len())
                .filter(|&i| !self.used[i])
                .filter(|&i| {
                    let t = w.proj(self.pts[i]);
                    w.dist(self.pts[i]) <= self.cfg.inlier_tol * 1.5
                        && t >= w.t0 - self.cfg.grow
                        && t <= w.t1 + self.cfg.grow
                })
                .collect();
            match self.fit_run(&near, Some(&w), res) {
                Some((fit, members)) => {
                    let w = &mut self.walls[wi];
                    // The line (angle + offset) is smoothed. Along the line, growth is taken at once
                    // (the map shows more wall) while shrinking is smoothed, so one noisy end cell
                    // cannot make the wall flicker.
                    let fdir = if fit.dir.0 * w.dir.0 + fit.dir.1 * w.dir.1 >= 0.0 {
                        fit.dir
                    } else {
                        (-fit.dir.0, -fit.dir.1)
                    };
                    let (fa, fb) = if fdir == fit.dir { (fit.a(), fit.b()) } else { (fit.b(), fit.a()) };
                    let (oa, ob) = (w.a(), w.b());
                    let k = self.cfg.alpha;
                    w.dir = norm(lerp2(w.dir, fdir, k));
                    w.c = lerp2(w.c, fit.c, k);
                    let (ta, tb) = (w.proj(fa), w.proj(fb));
                    let (oa, ob) = (w.proj(oa), w.proj(ob));
                    w.t0 = if ta < oa { ta } else { lerp(oa, ta, k) };
                    w.t1 = if tb > ob { tb } else { lerp(ob, tb, k) };
                    w.cells = members.len();
                    w.missed = 0;
                    w.age += 1;
                    for i in members {
                        self.used[i] = true;
                    }
                }
                None => {
                    let w = &mut self.walls[wi];
                    w.missed += 1;
                    keep[wi] = w.missed <= self.cfg.keep_missing;
                }
            }
        }
        let mut k = keep.into_iter();
        self.walls.retain(|_| k.next().unwrap());

        // 2. New walls in what is left, cluster by cluster.
        let free: Vec<usize> = (0..self.pts.len()).filter(|&i| !self.used[i]).collect();
        for cluster in clusters(&self.pts, &free, res * 2.05) {
            let mut rest = cluster;
            for _ in 0..12 {
                if rest.len() < self.cfg.min_cells {
                    break;
                }
                let Some((fit, members)) = self.ransac(&rest, res) else { break };
                for &i in &members {
                    self.used[i] = true;
                }
                rest.retain(|i| !self.used[*i]);
                self.add_or_merge(fit, members.len());
            }
        }

        // 3. Objects: everything solid that is not a wall. Cells hugging a wall (the staircase of a
        // diagonal wall, a skirting board) belong to that wall, not to a separate object.
        let hug = self.cfg.inlier_tol * 2.0;
        let left: Vec<usize> = (0..self.pts.len())
            .filter(|&i| !self.used[i])
            .filter(|&i| {
                let p = self.pts[i];
                !self.walls.iter().any(|w| {
                    let t = w.proj(p);
                    w.dist(p) <= hug && t >= w.t0 - hug && t <= w.t1 + hug
                })
            })
            .collect();
        self.objects = clusters(&self.pts, &left, res * 2.05)
            .into_iter()
            .filter(|c| c.len() >= 2)
            .map(|c| Object {
                hull: hull(&c.iter().map(|&i| self.pts[i]).collect::<Vec<_>>(), res * 0.5),
                cells: c.len(),
            })
            .collect();
        self.join_corners();
    }

    /// Walls for drawing, with nearby corners joined (as of the last `update`).
    pub fn output(&self) -> Vec<WallOut> {
        self.out.clone()
    }

    /// Each wall end joins at most one other wall: the one whose intersection is closest. A join
    /// from the previous update is kept up to 1.6 × the normal distance (hysteresis).
    #[allow(clippy::needless_range_loop)]
    fn join_corners(&mut self) {
        let snap = self.cfg.corner_snap;
        let mut out: Vec<WallOut> =
            self.walls.iter().map(|w| WallOut { id: w.id, a: w.a(), b: w.b() }).collect();
        let mut joins = std::collections::HashMap::new();
        for i in 0..self.walls.len() {
            for end in 0..2u8 {
                let wi = &self.walls[i];
                let pi = if end == 0 { wi.a() } else { wi.b() };
                let mut best: Option<(f32, usize, (f32, f32))> = None;
                for j in 0..self.walls.len() {
                    let wj = &self.walls[j];
                    if i == j || (wi.dir.0 * wj.dir.1 - wi.dir.1 * wj.dir.0).abs() < 0.5 {
                        continue; // same wall or less than 30° apart: not a corner
                    }
                    let Some(x) = intersect(wi, wj) else { continue };
                    let dj = d2(wj.a(), x).sqrt().min(d2(wj.b(), x).sqrt());
                    let d = d2(pi, x).sqrt().max(dj);
                    let limit = if self.joins.get(&(wi.id, end)) == Some(&wj.id) { snap * 1.6 } else { snap };
                    if d < limit && best.is_none_or(|b| d < b.0) {
                        best = Some((d, j, x));
                    }
                }
                if let Some((_, j, x)) = best {
                    if end == 0 {
                        out[i].a = x
                    } else {
                        out[i].b = x
                    }
                    joins.insert((wi.id, end), self.walls[j].id);
                }
            }
        }
        self.joins = joins;
        self.out = out;
    }

    fn collect(&mut self, grid: &Grid) {
        self.pts.clear();
        let min = self.cfg.min_value;
        for cy in 0..grid.dim {
            let row = &grid.cells[cy * grid.dim..(cy + 1) * grid.dim];
            if !row.iter().any(|&v| v >= min) {
                continue;
            }
            for (cx, &v) in row.iter().enumerate() {
                if v >= min {
                    self.pts.push(grid.cell_center(cx, cy));
                }
            }
        }
    }

    /// Least-squares line through `idx`, then the longest gap-free run along it.
    /// With `prev`, the run that overlaps the previous extent most is taken instead.
    fn fit_run(&self, idx: &[usize], prev: Option<&Wall>, res: f32) -> Option<(Wall, Vec<usize>)> {
        if idx.len() < self.cfg.min_cells {
            return None;
        }
        let line = pca(&self.pts, idx)?;
        let tol = self.cfg.inlier_tol;
        let mut on: Vec<(f32, usize)> = idx
            .iter()
            .filter(|&&i| line.dist(self.pts[i]) <= tol)
            .map(|&i| (line.proj(self.pts[i]), i))
            .collect();
        on.sort_by(|a, b| a.0.total_cmp(&b.0));
        // Split into runs at gaps.
        let mut runs: Vec<&[(f32, usize)]> = Vec::new();
        let mut s = 0;
        for k in 1..=on.len() {
            if k == on.len() || on[k].0 - on[k - 1].0 > self.cfg.max_gap {
                runs.push(&on[s..k]);
                s = k;
            }
        }
        let score = |r: &[(f32, usize)]| -> f32 {
            match prev {
                Some(w) => {
                    // Overlap with the old extent (in the old wall's coordinates).
                    let ts: Vec<f32> = r.iter().map(|&(_, i)| w.proj(self.pts[i])).collect();
                    let lo = ts.iter().cloned().fold(f32::MAX, f32::min);
                    let hi = ts.iter().cloned().fold(f32::MIN, f32::max);
                    (hi.min(w.t1) - lo.max(w.t0)).max(0.0) + r.len() as f32 * 1e-4
                }
                None => r.len() as f32,
            }
        };
        let run = runs.into_iter().max_by(|a, b| score(a).total_cmp(&score(b)))?;
        let members: Vec<usize> = run.iter().map(|&(_, i)| i).collect();
        let len = run.last()?.0 - run.first()?.0 + res;
        let min_len = if prev.is_some() { self.cfg.min_len * 0.7 } else { self.cfg.min_len };
        // Walls are thin: a straight run must be mostly filled, not a line through a blob.
        if members.len() < self.cfg.min_cells || len < min_len || (members.len() as f32) < len / res * 0.55 {
            return None;
        }
        let fit = pca(&self.pts, &members)?;
        let ts: Vec<f32> = members.iter().map(|&i| fit.proj(self.pts[i])).collect();
        let t0 = ts.iter().cloned().fold(f32::MAX, f32::min) - res * 0.5;
        let t1 = ts.iter().cloned().fold(f32::MIN, f32::max) + res * 0.5;
        Some((Wall { t0, t1, cells: members.len(), ..fit }, members))
    }

    /// Deterministic RANSAC: candidate lines through pairs of cells, best support wins.
    fn ransac(&self, idx: &[usize], res: f32) -> Option<(Wall, Vec<usize>)> {
        let n = idx.len();
        let tol = self.cfg.inlier_tol;
        let mut rng = 0x9E37_79B9u32 ^ n as u32;
        let mut next = || {
            rng ^= rng << 13;
            rng ^= rng >> 17;
            rng ^= rng << 5;
            rng as usize
        };
        let mut best: Option<(usize, Wall)> = None;
        for _ in 0..(n * 2).clamp(40, 300) {
            let (i, j) = (idx[next() % n], idx[next() % n]);
            let (p, q) = (self.pts[i], self.pts[j]);
            let d = d2(p, q).sqrt();
            if d < res * 3.0 {
                continue;
            }
            let line = Wall {
                id: 0,
                dir: ((q.0 - p.0) / d, (q.1 - p.1) / d),
                c: p,
                t0: 0.0,
                t1: 0.0,
                cells: 0,
                missed: 0,
                age: 0,
            };
            let support = idx.iter().filter(|&&k| line.dist(self.pts[k]) <= tol).count();
            if best.as_ref().is_none_or(|b| support > b.0) {
                best = Some((support, line));
            }
        }
        let (_, line) = best?;
        let near: Vec<usize> = idx.iter().copied().filter(|&k| line.dist(self.pts[k]) <= tol * 1.5).collect();
        self.fit_run(&near, None, res)
    }

    fn add_or_merge(&mut self, fit: Wall, cells: usize) {
        // A new run that continues an existing wall (same line, small gap) extends it instead.
        for w in &mut self.walls {
            let cos = (w.dir.0 * fit.dir.0 + w.dir.1 * fit.dir.1).abs();
            if cos < 0.995 || w.dist(fit.a()) > self.cfg.inlier_tol || w.dist(fit.b()) > self.cfg.inlier_tol {
                continue;
            }
            let (ta, tb) = (w.proj(fit.a()), w.proj(fit.b()));
            let (lo, hi) = (ta.min(tb), ta.max(tb));
            if lo <= w.t1 + self.cfg.max_gap && hi >= w.t0 - self.cfg.max_gap {
                w.t0 = w.t0.min(lo);
                w.t1 = w.t1.max(hi);
                w.cells += cells;
                return;
            }
        }
        let id = self.next_id;
        self.next_id += 1;
        self.walls.push(Wall { id, cells, missed: 0, age: 0, ..fit });
    }
}

fn pca(pts: &[(f32, f32)], idx: &[usize]) -> Option<Wall> {
    if idx.len() < 2 {
        return None;
    }
    let n = idx.len() as f64;
    let (mut mx, mut my) = (0.0f64, 0.0f64);
    for &i in idx {
        mx += pts[i].0 as f64;
        my += pts[i].1 as f64;
    }
    mx /= n;
    my /= n;
    let (mut sxx, mut syy, mut sxy) = (0.0f64, 0.0f64, 0.0f64);
    for &i in idx {
        let (dx, dy) = (pts[i].0 as f64 - mx, pts[i].1 as f64 - my);
        sxx += dx * dx;
        syy += dy * dy;
        sxy += dx * dy;
    }
    let th = 0.5 * (2.0 * sxy).atan2(sxx - syy);
    Some(Wall {
        id: 0,
        dir: (th.cos() as f32, th.sin() as f32),
        c: (mx as f32, my as f32),
        t0: 0.0,
        t1: 0.0,
        cells: idx.len(),
        missed: 0,
        age: 0,
    })
}

fn intersect(a: &Wall, b: &Wall) -> Option<(f32, f32)> {
    let den = a.dir.0 * b.dir.1 - a.dir.1 * b.dir.0;
    if den.abs() < 1e-6 {
        return None;
    }
    let (dx, dy) = (b.c.0 - a.c.0, b.c.1 - a.c.1);
    let t = (dx * b.dir.1 - dy * b.dir.0) / den;
    Some((a.c.0 + a.dir.0 * t, a.c.1 + a.dir.1 * t))
}

/// Single-linkage clusters of `idx` (cells closer than `link`), via a coarse hash grid.
fn clusters(pts: &[(f32, f32)], idx: &[usize], link: f32) -> Vec<Vec<usize>> {
    use std::collections::HashMap;
    let key = |p: (f32, f32)| ((p.0 / link).floor() as i32, (p.1 / link).floor() as i32);
    let mut buckets: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
    for (k, &i) in idx.iter().enumerate() {
        buckets.entry(key(pts[i])).or_default().push(k);
    }
    let mut label = vec![usize::MAX; idx.len()];
    let mut out = Vec::new();
    for start in 0..idx.len() {
        if label[start] != usize::MAX {
            continue;
        }
        let id = out.len();
        let mut members = vec![idx[start]];
        let mut stack = vec![start];
        label[start] = id;
        while let Some(k) = stack.pop() {
            let p = pts[idx[k]];
            let (bx, by) = key(p);
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if let Some(list) = buckets.get(&(bx + dx, by + dy)) {
                        for &m in list {
                            if label[m] == usize::MAX && d2(p, pts[idx[m]]) <= link * link {
                                label[m] = id;
                                members.push(idx[m]);
                                stack.push(m);
                            }
                        }
                    }
                }
            }
        }
        out.push(members);
    }
    out
}

/// Convex hull (monotone chain), grown by `pad` so single cells get a visible outline.
fn hull(p: &[(f32, f32)], pad: f32) -> Vec<(f32, f32)> {
    let mut pts: Vec<(f32, f32)> = Vec::with_capacity(p.len() * 4);
    for &(x, y) in p {
        pts.extend([(x - pad, y - pad), (x + pad, y - pad), (x + pad, y + pad), (x - pad, y + pad)]);
    }
    pts.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    pts.dedup();
    let cross =
        |o: (f32, f32), a: (f32, f32), b: (f32, f32)| (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0);
    let mut h: Vec<(f32, f32)> = Vec::new();
    for pass in 0..2 {
        let start = h.len();
        let iter: Box<dyn Iterator<Item = &(f32, f32)>> =
            if pass == 0 { Box::new(pts.iter()) } else { Box::new(pts.iter().rev()) };
        for &q in iter {
            while h.len() >= start + 2 && cross(h[h.len() - 2], h[h.len() - 1], q) <= 0.0 {
                h.pop();
            }
            h.push(q);
        }
        h.pop();
    }
    h
}

fn d2(a: (f32, f32), b: (f32, f32)) -> f32 {
    (a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)
}
fn lerp(a: f32, b: f32, k: f32) -> f32 {
    a + (b - a) * k
}
fn lerp2(a: (f32, f32), b: (f32, f32), k: f32) -> (f32, f32) {
    (lerp(a.0, b.0, k), lerp(a.1, b.1, k))
}
fn norm(v: (f32, f32)) -> (f32, f32) {
    let n = v.0.hypot(v.1).max(1e-9);
    (v.0 / n, v.1 / n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::Pose2;
    use crate::grid::GridConfig;

    /// A 4 × 3 m room rotated by 25°, with a box in it, seen from the centre.
    fn room_grid(rot: f64) -> Grid {
        let mut g = Grid::new(GridConfig { size: 12.0, ..GridConfig::default() });
        let (s, c) = (rot.sin() as f32, rot.cos() as f32);
        let r = |x: f32, y: f32| (c * x - s * y, s * x + c * y);
        let mut pts = Vec::new();
        let mut k = 0.0f32;
        while k <= 1.0 {
            for (a, b) in [
                ((-2.0, -1.5), (2.0, -1.5)),
                ((2.0, -1.5), (2.0, 1.5)),
                ((2.0, 1.5), (-2.0, 1.5)),
                ((-2.0, 1.5), (-2.0, -1.5)),
            ] {
                let p = r(a.0 + (b.0 - a.0) * k, a.1 + (b.1 - a.1) * k);
                pts.push(p);
            }
            k += 0.004;
        }
        for i in 0..10 {
            pts.push(r(0.8 + i as f32 * 0.02, 0.6)); // a 20 cm box face
        }
        for _ in 0..8 {
            g.integrate(&Pose2::IDENTITY, (0.0, 0.0), &pts);
        }
        g
    }

    #[test]
    fn rotated_room_gives_four_stable_walls() {
        let g = room_grid(25f64.to_radians());
        let mut wm = WallMap::new(WallConfig::default());
        for _ in 0..5 {
            wm.update(&g);
        }
        let out = wm.output();
        assert_eq!(out.len(), 4, "{out:?}");
        for w in &out {
            let len = d2(w.a, w.b).sqrt();
            assert!(len > 2.8, "short wall {w:?}");
            let ang = (w.b.1 - w.a.1).atan2(w.b.0 - w.a.0).to_degrees().rem_euclid(90.0);
            assert!((ang - 25.0).abs() < 1.0, "angle {ang}");
        }
        assert_eq!(wm.objects.len(), 1, "{:?}", wm.objects);
        // Same grid again: same ids, nothing moves.
        let before = wm.output();
        wm.update(&g);
        for (a, b) in before.iter().zip(wm.output()) {
            assert_eq!(a.id, b.id);
            assert!(d2(a.a, b.a).sqrt() < 0.005 && d2(a.b, b.b).sqrt() < 0.005);
        }
    }

    #[test]
    fn hull_of_one_cell_is_a_square() {
        let h = hull(&[(1.0, 1.0)], 0.025);
        assert_eq!(h.len(), 4);
    }
}
