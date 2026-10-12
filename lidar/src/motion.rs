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
    /// Max distance from a track's predicted position to a new detection: `gate` + `gate_per_speed` × speed.
    /// A sprinter turning at a wall is 0.5 m off the prediction within a scan or two.
    pub gate: f32,
    pub gate_per_speed: f32,
    /// Detections needed before a track is shown...
    pub confirm_hits: u32,
    /// ...and how far it must have moved from where it was first seen, metres. Map artefacts
    /// (a door frame seen from a new angle) stand still; people and pets don't.
    pub confirm_move: f32,
    /// ...and have averaged at least this many points per detection. A person at chest height gives
    /// 10–40; a thin furniture leg that the beams sometimes miss gives 3–7 and must not become a mover.
    pub confirm_points: f32,
    /// How long an unconfirmed track survives without detections, s.
    pub tentative_life: f64,
    /// How long a confirmed track keeps predicting without detections, s (bending down below the
    /// scan plane takes ~2 s).
    pub coast: f64,
    /// While unseen, the speed decays with this time constant and the guess stays within
    /// `coast_max` metres of where it was last seen (so it does not fly through walls).
    pub coast_tau: f64,
    pub coast_max: f64,
    /// A new detection this close to where a lost track was last seen (or is predicted) takes over
    /// that track's id instead of starting a new one.
    pub reacquire: f64,
    /// A confirmed track not seen for longer than `coast` is hidden but remembered this long, s,
    /// so someone who stood out of view (behind furniture, too close to a wall) gets their id back
    /// when they reappear within `reacquire_dormant` metres of where they were last seen.
    pub dormant: f64,
    pub reacquire_dormant: f64,
    /// A new blob this close to a person who is being tracked right now is part of that person
    /// (an arm, a bag), not a new mover.
    pub satellite: f64,
    /// Clutter: small blobs (≤ `clutter_max_points`) that keep appearing in the same `clutter_cell`
    /// (a glass kettle, a glass cabinet door: the laser goes partly through glass) are ignored there
    /// once the spot has scored `clutter_limit` (each such blob +1, halving every `clutter_half_life` s).
    pub clutter_cell: f32,
    pub clutter_max_points: usize,
    pub clutter_limit: f32,
    pub clutter_half_life: f64,
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
            gate_per_speed: 0.3,
            confirm_hits: 3,
            confirm_move: 0.25,
            confirm_points: 6.0,
            tentative_life: 0.35,
            coast: 3.0,
            coast_tau: 0.5,
            coast_max: 1.0,
            reacquire: 1.2,
            dormant: 10.0,
            reacquire_dormant: 1.5,
            satellite: 0.7,
            clutter_cell: 0.15,
            clutter_max_points: 5,
            clutter_limit: 6.0,
            clutter_half_life: 30.0,
            accel_sigma: 5.0,
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
    /// Sum of points over all detections (for the average).
    pub point_sum: usize,
    pub last_seen: f64,
    /// Position of the last detection.
    pub seen_pos: (f64, f64),
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
    /// Includes tentative and dormant tracks; what to show is `visible()`.
    pub tracks: Vec<Track>,
    clutter: std::collections::HashMap<(i32, i32), (f32, f64)>,
    next_id: u32,
    last_t: Option<f64>,
}

impl Tracker {
    pub fn new(cfg: MotionConfig) -> Self {
        Tracker { cfg, tracks: Vec::new(), clutter: Default::default(), next_id: 1, last_t: None }
    }

    /// Confirmed tracks seen within the last `coast` seconds: what the UI and the robot should use.
    pub fn visible(&self) -> impl Iterator<Item = &Track> {
        self.tracks.iter().filter(|k| k.confirmed && k.t - k.last_seen <= self.cfg.coast)
    }

    /// Spots that keep producing small flickering blobs. Returns true if this blob is clutter.
    fn clutter(&mut self, b: &Blob, t: f64) -> bool {
        if b.n > self.cfg.clutter_max_points {
            return false;
        }
        let c = self.cfg.clutter_cell;
        let key = ((b.x / c).floor() as i32, (b.y / c).floor() as i32);
        let hl = self.cfg.clutter_half_life;
        let mut worst = 0.0f32;
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some((s, lt)) = self.clutter.get(&(key.0 + dx, key.1 + dy)) {
                    worst = worst.max(s * 0.5f32.powf(((t - lt) / hl) as f32));
                }
            }
        }
        let e = self.clutter.entry(key).or_insert((0.0, t));
        e.0 = e.0 * 0.5f32.powf(((t - e.1) / hl) as f32) + 1.0;
        e.1 = t;
        let score = e.0;
        if self.clutter.len() > 4096 {
            self.clutter.retain(|_, (s, lt)| *s * 0.5f32.powf(((t - *lt) / hl) as f32) > 0.5);
        }
        worst.max(score) > self.cfg.clutter_limit
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
            if tr.coasting() {
                let k = (-dt / self.cfg.coast_tau).exp();
                tr.sx.x[1] *= k;
                tr.sy.x[1] *= k;
                let (dx, dy) = (tr.sx.x[0] - tr.seen_pos.0, tr.sy.x[0] - tr.seen_pos.1);
                let d = dx.hypot(dy);
                if d > self.cfg.coast_max {
                    let f = self.cfg.coast_max / d;
                    tr.sx.x[0] = tr.seen_pos.0 + dx * f;
                    tr.sy.x[0] = tr.seen_pos.1 + dy * f;
                }
            }
        }
        if let Some(pts) = moving {
            let all = self.blobs(pts);
            let mut blobs = Vec::with_capacity(all.len());
            for b in all {
                if !self.clutter(&b, t) {
                    blobs.push(b);
                }
            }
            let coast = self.cfg.coast;
            // Greedy nearest-first association inside the gate.
            let mut pairs: Vec<(f64, usize, usize)> = Vec::new();
            for (ti, tr) in self.tracks.iter().enumerate() {
                if t - tr.last_seen > coast {
                    continue; // dormant: only re-acquired, below
                }
                let (px, py) = tr.pos();
                let (vx, vy) = tr.vel();
                let gate = self.cfg.gate as f64 + self.cfg.gate_per_speed as f64 * vx.hypot(vy);
                for (bi, b) in blobs.iter().enumerate() {
                    let d = (b.x as f64 - px).hypot(b.y as f64 - py);
                    if d <= gate {
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
                tr.seen_pos = (b.x as f64, b.y as f64);
                tr.points = b.n;
                tr.travelled = tr.travelled.max((b.x - tr.origin.0).hypot(b.y - tr.origin.1));
                tr.point_sum += b.n;
                tr.confirmed |= tr.hits >= self.cfg.confirm_hits
                    && tr.travelled >= self.cfg.confirm_move
                    && tr.point_sum as f32 >= self.cfg.confirm_points * tr.hits as f32;
            }
            for (bi, b) in blobs.iter().enumerate() {
                if b_used[bi] {
                    continue;
                }
                let (bx, by) = (b.x as f64, b.y as f64);
                // Part of someone tracked right now (an arm, a bag)?
                let satellite = (0..t_used.len()).any(|ti| {
                    let tr = &self.tracks[ti];
                    t_used[ti]
                        && tr.confirmed
                        && (bx - tr.seen_pos.0).hypot(by - tr.seen_pos.1) <= self.cfg.satellite
                });
                if satellite {
                    continue;
                }
                // Re-acquire: a lost confirmed track near here keeps its id. Coasting tracks: near the
                // prediction or where last seen; dormant ones: near where last seen.
                let reach = |tr: &Track| -> Option<f64> {
                    let ds = (bx - tr.seen_pos.0).hypot(by - tr.seen_pos.1);
                    if t - tr.last_seen > coast {
                        (ds <= self.cfg.reacquire_dormant).then_some(ds)
                    } else {
                        let (px, py) = tr.pos();
                        let d = (bx - px).hypot(by - py).min(ds);
                        (d <= self.cfg.reacquire).then_some(d)
                    }
                };
                let lost = (0..t_used.len()) // tracks created in this loop are new, never lost
                    .filter(|&ti| !t_used[ti] && self.tracks[ti].confirmed && self.tracks[ti].coasting())
                    .filter_map(|ti| reach(&self.tracks[ti]).map(|d| (d, ti)))
                    .min_by(|a, b| a.0.total_cmp(&b.0));
                if let Some((_, ti)) = lost {
                    t_used[ti] = true;
                    let tr = &mut self.tracks[ti];
                    tr.sx = Kf::new(b.x as f64, self.cfg.meas_sigma);
                    tr.sy = Kf::new(b.y as f64, self.cfg.meas_sigma);
                    tr.hits += 1;
                    tr.last_seen = t;
                    tr.seen_pos = (b.x as f64, b.y as f64);
                    tr.points = b.n;
                    continue;
                }
                {
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
                        point_sum: b.n,
                        last_seen: t,
                        seen_pos: (b.x as f64, b.y as f64),
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
                unseen <= cfg.dormant
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
        self.clutter.clear();
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
            let pts: Vec<(f32, f32)> = (0..8).map(|i| (x, i as f32 * 0.02)).collect();
            tr.update(k as f64 * 0.1, Some(&pts));
        }
        tr.update(1.5, Some(&[]));
        assert_eq!(tr.tracks.len(), 1);
        assert!(tr.tracks[0].coasting());
        let p = tr.tracks[0].pos().0;
        assert!(p > 1.0 && p < 1.9 + 0.01, "kept moving while unseen, but slowing and within 1 m: {p}");
        tr.update(5.0, Some(&[]));
        assert_eq!(tr.visible().count(), 0, "hidden after the coast time");
        tr.update(12.0, Some(&[]));
        assert!(tr.tracks.is_empty(), "forgotten after the dormant time");
    }

    #[test]
    fn reappearing_after_bending_down_keeps_the_id() {
        let mut tr = Tracker::new(MotionConfig::default());
        for k in 0..10 {
            let x = k as f32 * 0.08;
            let pts: Vec<(f32, f32)> = (0..8).map(|i| (x, i as f32 * 0.03)).collect();
            tr.update(k as f64 * 0.1, Some(&pts));
        }
        let id = tr.tracks[0].id;
        for k in 10..30 {
            tr.update(k as f64 * 0.1, Some(&[])); // 2 s below the scan plane
        }
        tr.update(3.0, Some(&[(0.85, 0.1), (0.85, 0.13), (0.85, 0.16)]));
        assert_eq!(tr.tracks.len(), 1);
        assert_eq!(tr.tracks[0].id, id);
        assert!(!tr.tracks[0].coasting());
    }
}

#[cfg(test)]
mod clutter_tests {
    use super::*;

    #[test]
    fn flickering_glass_is_ignored_but_a_person_passing_it_is_not() {
        let mut tr = Tracker::new(MotionConfig::default());
        // A glass kettle: 3 points at the same spot every other scan for 10 s.
        for k in 0..100 {
            let pts: &[(f32, f32)] =
                if k % 2 == 0 { &[(1.40, 1.83), (1.41, 1.84), (1.42, 1.83)] } else { &[] };
            tr.update(k as f64 * 0.1, Some(pts));
        }
        assert_eq!(tr.visible().count(), 0);
        let before = tr.tracks.iter().filter(|k| k.hits > 3).count();
        assert_eq!(before, 0, "the flicker never builds a track");
        // A person (20 points) walking right past the same spot is still tracked.
        for k in 0..20 {
            let x = 0.6 + k as f32 * 0.08;
            let pts: Vec<(f32, f32)> =
                (0..20).map(|i| (x + (i % 5) as f32 * 0.03, 1.7 + (i / 5) as f32 * 0.05)).collect();
            tr.update(10.0 + k as f64 * 0.1, Some(&pts));
        }
        assert_eq!(tr.visible().count(), 1);
    }

    #[test]
    fn an_arm_next_to_a_tracked_person_is_not_a_new_mover() {
        let mut tr = Tracker::new(MotionConfig::default());
        for k in 0..30 {
            let x = k as f32 * 0.06;
            let mut pts: Vec<(f32, f32)> =
                (0..15).map(|i| (x + (i % 5) as f32 * 0.03, (i / 5) as f32 * 0.04)).collect();
            if k > 10 {
                pts.extend([(x, 0.55), (x + 0.02, 0.56), (x + 0.04, 0.55)]); // 45 cm away: beyond the blob link
            }
            tr.update(k as f64 * 0.1, Some(&pts));
        }
        assert_eq!(
            tr.tracks.iter().filter(|k| k.confirmed).count(),
            1,
            "{:?}",
            tr.tracks.iter().map(|k| k.id).collect::<Vec<_>>()
        );
    }
}
