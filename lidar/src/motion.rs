//! Moving things: which points of a scan are new obstacles in known-free space, grouped and
//! tracked over time with position and velocity.
//!
//! A point is "moving" when it lands in a cell the map has seen as free many times (beams went
//! straight through it) and no solid cell is next to it. That needs no model of people or
//! furniture: anything that walks into empty space is caught, a chair that has stood still for a
//! while has become part of the map, and a wall is never flagged because its cells are solid.
//! Unknown cells (never seen) are not flagged: a doorway that opens up is not motion.
//!
//! Tracks keep predicting with their last velocity for a while after they disappear (behind the
//! robot's own body, behind furniture), so a follower can guess where a person went.

use std::collections::VecDeque;

use crate::geom::Pose2;
use crate::grid::Grid;

#[derive(Clone, Debug)]
pub struct MotionConfig {
    /// The cell must be at least this free (log-odds; the grid's miss is −5 per beam, so −40 ≈ 8 beams).
    pub free_max: i8,
    /// ...and no cell within `solid_radius` cells may be this solid (grazing beams mark the cells
    /// right next to a wall or door frame as free, so wall points can land there from a new angle).
    pub solid_min: i8,
    pub solid_radius: usize,
    /// Points closer than this belong to the same moving thing (two legs → one person).
    pub link: f32,
    pub min_points: usize,
    /// Max distance from a track's predicted position to a new detection, metres.
    pub gate: f32,
    /// Detections needed before a track is shown...
    pub confirm_hits: u32,
    /// ...and how far it must have moved from where it was first seen, metres. Map artefacts
    /// (a door frame seen from a new angle) stand still; people and pets don't.
    pub confirm_move: f32,
    /// How long an unconfirmed track survives without detections, s.
    pub tentative_life: f64,
    /// How long a confirmed track keeps predicting without detections, s.
    pub coast: f64,
    /// Kalman noise: acceleration (m/s²) and measurement (m), 1 σ.
    pub accel_sigma: f64,
    pub meas_sigma: f64,
    /// Seconds of history kept per track (the UI draws it as a trail).
    pub history_s: f64,
}

impl Default for MotionConfig {
    fn default() -> Self {
        MotionConfig {
            free_max: -40,
            solid_min: 20,
            solid_radius: 2,
            link: 0.35,
            min_points: 3,
            gate: 0.8,
            confirm_hits: 3,
            confirm_move: 0.25,
            tentative_life: 0.35,
            coast: 2.0,
            accel_sigma: 2.0,
            meas_sigma: 0.08,
            history_s: 4.0,
        }
    }
}

/// Marks which robot-frame `points` are moving, given the map *before* this scan is added.
pub fn classify(grid: &Grid, pose: &Pose2, points: &[(f32, f32)], cfg: &MotionConfig, out: &mut Vec<bool>) {
    out.clear();
    out.extend(points.iter().map(|&(px, py)| {
        let (wx, wy) = pose.apply(px as f64, py as f64);
        let Some((cx, cy)) = grid.cell_index(wx as f32, wy as f32) else { return false };
        let r = cfg.solid_radius;
        if cx < r || cy < r || cx + r >= grid.dim || cy + r >= grid.dim || grid.value(cx, cy) > cfg.free_max {
            return false;
        }
        for ny in cy - r..=cy + r {
            for nx in cx - r..=cx + r {
                if grid.value(nx, ny) >= cfg.solid_min {
                    return false;
                }
            }
        }
        true
    }));
}

#[derive(Clone, Debug)]
pub struct Track {
    pub id: u32,
    /// State [x, vx] and [y, vy] with 2 × 2 covariances (independent axes).
    sx: Kf,
    sy: Kf,
    pub hits: u32,
    pub confirmed: bool,
    pub first_t: f64,
    /// Where it was first seen, and the farthest it has been from there.
    pub origin: (f32, f32),
    pub travelled: f32,
    pub last_seen: f64,
    pub t: f64,
    /// Points in the last detection (size cue).
    pub points: usize,
    pub history: VecDeque<(f64, f32, f32)>,
}

impl Track {
    pub fn pos(&self) -> (f64, f64) {
        (self.sx.x[0], self.sy.x[0])
    }
    pub fn vel(&self) -> (f64, f64) {
        (self.sx.x[1], self.sy.x[1])
    }
    /// Not seen in the latest scan: position is a prediction.
    pub fn coasting(&self) -> bool {
        self.t - self.last_seen > 0.15
    }
}

#[derive(Clone, Debug)]
struct Kf {
    x: [f64; 2],
    p: [[f64; 2]; 2],
}

impl Kf {
    fn new(pos: f64, meas: f64) -> Kf {
        Kf { x: [pos, 0.0], p: [[meas * meas, 0.0], [0.0, 1.0]] }
    }
    fn predict(&mut self, dt: f64, q: f64) {
        let [p00, p01] = self.p[0];
        let [p10, p11] = self.p[1];
        self.x[0] += self.x[1] * dt;
        // F P Fᵀ + Q (white-noise acceleration)
        let (dt2, dt3, dt4) = (dt * dt, dt * dt * dt, dt * dt * dt * dt);
        self.p = [
            [p00 + dt * (p10 + p01) + dt2 * p11 + q * dt4 / 4.0, p01 + dt * p11 + q * dt3 / 2.0],
            [p10 + dt * p11 + q * dt3 / 2.0, p11 + q * dt2],
        ];
    }
    fn update(&mut self, z: f64, r: f64) {
        let s = self.p[0][0] + r;
        let (k0, k1) = (self.p[0][0] / s, self.p[1][0] / s);
        let y = z - self.x[0];
        self.x[0] += k0 * y;
        self.x[1] += k1 * y;
        let [p00, p01] = self.p[0];
        self.p = [[(1.0 - k0) * p00, (1.0 - k0) * p01], [self.p[1][0] - k1 * p00, self.p[1][1] - k1 * p01]];
    }
}

/// A detection: centroid of one group of moving points (world frame).
#[derive(Clone, Copy, Debug)]
pub struct Blob {
    pub x: f32,
    pub y: f32,
    pub n: usize,
}

pub struct Tracker {
    pub cfg: MotionConfig,
    pub tracks: Vec<Track>,
    next_id: u32,
    last_t: Option<f64>,
}

impl Tracker {
    pub fn new(cfg: MotionConfig) -> Self {
        Tracker { cfg, tracks: Vec::new(), next_id: 1, last_t: None }
    }

    /// Groups world-frame moving points into blobs.
    #[allow(clippy::needless_range_loop)]
    pub fn blobs(&self, pts: &[(f32, f32)]) -> Vec<Blob> {
        let n = pts.len();
        let mut parent: Vec<usize> = (0..n).collect();
        fn find(p: &mut [usize], i: usize) -> usize {
            let mut r = i;
            while p[r] != r {
                r = p[r];
            }
            let mut i = i;
            while p[i] != r {
                let next = p[i];
                p[i] = r;
                i = next;
            }
            r
        }
        let l2 = self.cfg.link * self.cfg.link;
        for i in 0..n {
            for j in i + 1..n {
                if (pts[i].0 - pts[j].0).powi(2) + (pts[i].1 - pts[j].1).powi(2) <= l2 {
                    let (a, b) = (find(&mut parent, i), find(&mut parent, j));
                    parent[a] = b;
                }
            }
        }
        let mut sums: std::collections::BTreeMap<usize, (f32, f32, usize)> = Default::default();
        for i in 0..n {
            let r = find(&mut parent, i);
            let e = sums.entry(r).or_insert((0.0, 0.0, 0));
            e.0 += pts[i].0;
            e.1 += pts[i].1;
            e.2 += 1;
        }
        sums.into_values()
            .filter(|s| s.2 >= self.cfg.min_points)
            .map(|(x, y, k)| Blob { x: x / k as f32, y: y / k as f32, n: k })
            .collect()
    }

    /// One scan's moving points (world frame) at time `t`. `None` = the pose is unreliable this
    /// scan: tracks only predict.
    pub fn update(&mut self, t: f64, moving: Option<&[(f32, f32)]>) {
        let dt = self.last_t.map_or(0.0, |l| (t - l).clamp(0.0, 1.0));
        self.last_t = Some(t);
        let q = self.cfg.accel_sigma * self.cfg.accel_sigma;
        for tr in &mut self.tracks {
            tr.sx.predict(dt, q);
            tr.sy.predict(dt, q);
            tr.t = t;
        }
        if let Some(pts) = moving {
            let blobs = self.blobs(pts);
            // Greedy nearest-first association inside the gate.
            let mut pairs: Vec<(f64, usize, usize)> = Vec::new();
            for (ti, tr) in self.tracks.iter().enumerate() {
                let (px, py) = tr.pos();
                for (bi, b) in blobs.iter().enumerate() {
                    let d = (b.x as f64 - px).hypot(b.y as f64 - py);
                    if d <= self.cfg.gate as f64 {
                        pairs.push((d, ti, bi));
                    }
                }
            }
            pairs.sort_by(|a, b| a.0.total_cmp(&b.0));
            let (mut t_used, mut b_used) = (vec![false; self.tracks.len()], vec![false; blobs.len()]);
            let r = self.cfg.meas_sigma * self.cfg.meas_sigma;
            for (_, ti, bi) in pairs {
                if t_used[ti] || b_used[bi] {
                    continue;
                }
                t_used[ti] = true;
                b_used[bi] = true;
                let (tr, b) = (&mut self.tracks[ti], blobs[bi]);
                tr.sx.update(b.x as f64, r);
                tr.sy.update(b.y as f64, r);
                tr.hits += 1;
                tr.last_seen = t;
                tr.points = b.n;
                tr.travelled = tr.travelled.max((b.x - tr.origin.0).hypot(b.y - tr.origin.1));
                tr.confirmed |= tr.hits >= self.cfg.confirm_hits && tr.travelled >= self.cfg.confirm_move;
            }
            for (bi, b) in blobs.iter().enumerate() {
                if !b_used[bi] {
                    let id = self.next_id;
                    self.next_id += 1;
                    self.tracks.push(Track {
                        id,
                        sx: Kf::new(b.x as f64, self.cfg.meas_sigma),
                        sy: Kf::new(b.y as f64, self.cfg.meas_sigma),
                        hits: 1,
                        confirmed: false,
                        first_t: t,
                        origin: (b.x, b.y),
                        travelled: 0.0,
                        last_seen: t,
                        t,
                        points: b.n,
                        history: VecDeque::new(),
                    });
                }
            }
        }
        let cfg = &self.cfg;
        self.tracks.retain(|tr| {
            let unseen = t - tr.last_seen;
            if tr.confirmed {
                unseen <= cfg.coast
            } else {
                unseen <= cfg.tentative_life
            }
        });
        for tr in &mut self.tracks {
            let (x, y) = tr.pos();
            tr.history.push_back((t, x as f32, y as f32));
            while tr.history.front().is_some_and(|h| t - h.0 > cfg.history_s) {
                tr.history.pop_front();
            }
        }
    }

    pub fn reset(&mut self) {
        self.tracks.clear();
        self.last_t = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_walker_gets_one_track_with_the_right_velocity() {
        let mut tr = Tracker::new(MotionConfig::default());
        for k in 0..40 {
            let t = k as f64 * 0.1;
            let x = 1.0 + 0.8 * t as f32; // 0.8 m/s along x
                                          // two legs, 25 cm apart, a few points each, with some noise
            let wob = ((k * 7919) % 13) as f32 / 13.0 * 0.04 - 0.02;
            let pts = [
                (x, 0.12 + wob),
                (x + 0.02, 0.13),
                (x - 0.02, 0.11),
                (x, -0.12),
                (x + 0.03, -0.11),
                (x - 0.01, -0.13),
            ];
            tr.update(t, Some(&pts));
        }
        assert_eq!(tr.tracks.len(), 1);
        let t = &tr.tracks[0];
        assert!(t.confirmed);
        let (vx, vy) = t.vel();
        assert!((vx - 0.8).abs() < 0.1 && vy.abs() < 0.1, "v = {vx:.2}, {vy:.2}");
    }

    #[test]
    fn coasts_then_expires() {
        let mut tr = Tracker::new(MotionConfig::default());
        for k in 0..10 {
            let x = k as f32 * 0.1;
            tr.update(k as f64 * 0.1, Some(&[(x, 0.0), (x, 0.02), (x, 0.04)]));
        }
        tr.update(1.5, Some(&[]));
        assert_eq!(tr.tracks.len(), 1);
        assert!(tr.tracks[0].coasting());
        assert!(tr.tracks[0].pos().0 > 1.2, "kept moving while unseen");
        tr.update(3.5, Some(&[]));
        assert!(tr.tracks.is_empty());
    }
}
