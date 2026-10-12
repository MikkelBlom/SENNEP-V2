//! LiDAR odometry: where did the robot move since the last scan?
//!
//! SENNEP has no wheel encoders (mecanum wheels slip anyway), so this is the only motion source
//! until the gyro is fused. Approach, chosen to avoid what made the old app drift and jitter:
//!
//! 1. **Predict** with constant velocity from the previous scans.
//! 2. **De-skew**: a revolution takes 100 ms, during which the robot moves up to 4 cm. Each point
//!    is moved to where it would have been seen at the scan's end time, using its own timestamp.
//! 3. **Match against keyframes, not the previous scan.** A keyframe is a stored scan with surface
//!    normals. While the robot stands still every scan matches the same keyframe, so noise does
//!    not accumulate (zero drift, sub-mm jitter). New keyframes are added only after real motion.
//! 4. **Point-to-line ICP** (Gauss-Newton on point-to-normal distance, Huber weights), with a
//!    shrinking correspondence radius. Converges in a few iterations on indoor walls, where
//!    point-to-point ICP slides along them.
//! 5. If the match is poor (sudden acceleration ruined the prediction), retry from rotated guesses.

use crate::geom::{wrap_angle, Pose2};
use crate::scan::ScanPoint;
use crate::sim::range_sigma;
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Debug)]
pub struct OdomConfig {
    pub keyframe_dist: f64,
    pub keyframe_angle: f64,
    pub max_keyframes: usize,
    pub max_iterations: usize,
    /// Huber threshold on point-to-line distance (m).
    pub huber: f64,
    /// Fraction of points within `inlier_dist` needed to accept a match.
    pub min_inlier_ratio: f64,
    pub inlier_dist: f64,
    pub deskew: bool,
}

impl Default for OdomConfig {
    fn default() -> Self {
        OdomConfig {
            keyframe_dist: 0.25,
            keyframe_angle: 12f64.to_radians(),
            max_keyframes: 10,
            max_iterations: 25,
            huber: 0.02,
            min_inlier_ratio: 0.35,
            inlier_dist: 0.05,
            deskew: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct MapPoint {
    x: f32,
    y: f32,
    nx: f32,
    ny: f32,
}

struct Keyframe {
    pose: Pose2,
    points: Vec<MapPoint>,
}

const CELL: f32 = 0.25;

/// Spatial hash over the keyframe points for nearest-neighbour lookups.
#[derive(Default)]
struct LocalMap {
    points: Vec<MapPoint>,
    cells: HashMap<(i32, i32), (u32, u32)>,
}

impl LocalMap {
    fn build(keyframes: &VecDeque<Keyframe>) -> LocalMap {
        let mut pts: Vec<(i32, i32, MapPoint)> = keyframes
            .iter()
            .flat_map(|k| k.points.iter())
            .map(|p| ((p.x / CELL).floor() as i32, (p.y / CELL).floor() as i32, *p))
            .collect();
        pts.sort_unstable_by_key(|p| (p.0, p.1));
        let mut cells = HashMap::with_capacity(pts.len() / 4);
        let mut i = 0;
        while i < pts.len() {
            let mut j = i;
            while j < pts.len() && (pts[j].0, pts[j].1) == (pts[i].0, pts[i].1) {
                j += 1;
            }
            cells.insert((pts[i].0, pts[i].1), (i as u32, (j - i) as u32));
            i = j;
        }
        LocalMap { points: pts.into_iter().map(|p| p.2).collect(), cells }
    }

    fn nearest(&self, x: f32, y: f32, max_d: f32) -> Option<&MapPoint> {
        let (cx, cy) = ((x / CELL).floor() as i32, (y / CELL).floor() as i32);
        let mut best = None;
        let mut best_d2 = max_d * max_d;
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(&(s, n)) = self.cells.get(&(cx + dx, cy + dy)) {
                    for p in &self.points[s as usize..(s + n) as usize] {
                        let d2 = (p.x - x).powi(2) + (p.y - y).powi(2);
                        if d2 < best_d2 {
                            best_d2 = d2;
                            best = Some(p);
                        }
                    }
                }
            }
        }
        best
    }
}

/// Surface normals from neighbouring points in scan order (scan points are angularly sorted, so
/// neighbours in the array are neighbours on the surface unless there is a range jump).
/// Normals are oriented towards `origin` (the sensor), so the two faces of a thin wall can be
/// told apart later.
fn with_normals(points: &[(f32, f32)], origin: (f32, f32)) -> Vec<MapPoint> {
    let n = points.len();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let (px, py) = points[i];
        let (mut sx, mut sy, mut sxx, mut sxy, mut syy, mut k) = (0f32, 0f32, 0f32, 0f32, 0f32, 0f32);
        for o in -3i32..=3 {
            let j = (i as i32 + o).rem_euclid(n as i32) as usize;
            let (qx, qy) = points[j];
            if (qx - px).powi(2) + (qy - py).powi(2) > 0.12 * 0.12 {
                continue;
            }
            sx += qx;
            sy += qy;
            sxx += qx * qx;
            sxy += qx * qy;
            syy += qy * qy;
            k += 1.0;
        }
        if k < 4.0 {
            continue;
        }
        let (mx, my) = (sx / k, sy / k);
        let (cxx, cxy, cyy) = (sxx / k - mx * mx, sxy / k - mx * my, syy / k - my * my);
        // Smallest eigenvector of the 2x2 covariance = surface normal.
        let tr = cxx + cyy;
        let det = cxx * cyy - cxy * cxy;
        let disc = (tr * tr / 4.0 - det).max(0.0).sqrt();
        let (l_small, l_big) = (tr / 2.0 - disc, tr / 2.0 + disc);
        if l_big <= 0.0 || l_small > 0.05 * l_big {
            continue; // not line-like (corner, clutter): useless for point-to-line
        }
        let (mut nx, mut ny) = if cxy.abs() > 1e-9 {
            (cxy, l_small - cxx)
        } else if cxx < cyy {
            (1.0, 0.0)
        } else {
            (0.0, 1.0)
        };
        let len = (nx * nx + ny * ny).sqrt();
        nx /= len;
        ny /= len;
        if nx * (origin.0 - px) + ny * (origin.1 - py) < 0.0 {
            nx = -nx;
            ny = -ny;
        }
        out.push(MapPoint { x: px, y: py, nx, ny });
    }
    out
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MatchInfo {
    pub iterations: u32,
    pub inlier_ratio: f64,
    pub rms: f64,
    pub accepted: bool,
    pub retries: u32,
    pub new_keyframe: bool,
}

pub struct Odometry {
    pub cfg: OdomConfig,
    pub pose: Pose2,
    /// Body-frame velocity (vx, vy, w) estimated from the last two scans.
    pub vel: Pose2,
    last_t: Option<f64>,
    keyframes: VecDeque<Keyframe>,
    map: LocalMap,
    /// Scratch buffers, reused to avoid per-scan allocation.
    body: Vec<(f32, f32)>,
}

impl Odometry {
    pub fn new(cfg: OdomConfig) -> Self {
        Odometry {
            cfg,
            pose: Pose2::IDENTITY,
            vel: Pose2::IDENTITY,
            last_t: None,
            keyframes: VecDeque::new(),
            map: LocalMap::default(),
            body: Vec::new(),
        }
    }

    /// The last processed scan's points, de-skewed into the robot frame at its end time.
    pub fn deskewed(&self) -> &[(f32, f32)] {
        &self.body
    }

    pub fn keyframe_count(&self) -> usize {
        self.keyframes.len()
    }

    /// Processes one filtered scan; returns the robot pose at `t_end`.
    pub fn process(&mut self, points: &[ScanPoint], t_end: f64) -> (Pose2, MatchInfo) {
        let dt = self.last_t.map_or(0.1, |t| (t_end - t).clamp(1e-3, 1.0));
        let predicted = self.pose.compose(&self.vel.scale(dt));

        // De-skew into the robot frame at t_end.
        self.body.clear();
        for p in points {
            if self.cfg.deskew {
                let tau = p.t - t_end; // <= 0
                let (x, y) = self.vel.scale(tau).apply(p.x as f64, p.y as f64);
                self.body.push((x as f32, y as f32));
            } else {
                self.body.push((p.x, p.y));
            }
        }

        let mut info = MatchInfo::default();
        if self.keyframes.is_empty() {
            if self.body.len() >= 20 {
                self.add_keyframe(predicted);
                info.accepted = true;
                info.new_keyframe = true;
            }
            self.last_t = Some(t_end);
            return (self.pose, info);
        }

        let (mut est, mut m) = self.icp(predicted);
        if !m.accepted {
            // Sudden acceleration: try rotated starting guesses (no gyro yet).
            for (k, dth) in [5.0f64, -5.0, 10.0, -10.0, 15.0, -15.0].iter().enumerate() {
                let guess = Pose2::new(predicted.x, predicted.y, predicted.th + dth.to_radians());
                let (e, mm) = self.icp(guess);
                if mm.accepted && mm.inlier_ratio > m.inlier_ratio {
                    est = e;
                    m = mm;
                    m.retries = k as u32 + 1;
                    break;
                }
            }
        }
        info = m;
        if info.accepted {
            let step = self.pose.between(&est);
            self.vel = step.scale(1.0 / dt);
            self.pose = est;
        } else {
            // Keep coasting on the prediction but decay velocity, so a long dropout doesn't fly away.
            self.pose = predicted;
            self.vel = self.vel.scale(0.5);
        }
        self.last_t = Some(t_end);

        let last = self.keyframes.back().map(|k| k.pose).unwrap_or(self.pose);
        let d = last.between(&self.pose);
        if info.accepted
            && (d.dist() > self.cfg.keyframe_dist
                || d.th.abs() > self.cfg.keyframe_angle
                || info.inlier_ratio < self.cfg.min_inlier_ratio + 0.2)
        {
            self.add_keyframe(self.pose);
            info.new_keyframe = true;
        }
        (self.pose, info)
    }

    fn add_keyframe(&mut self, pose: Pose2) {
        let world: Vec<(f32, f32)> = self
            .body
            .iter()
            .map(|&(x, y)| {
                let (wx, wy) = pose.apply(x as f64, y as f64);
                (wx as f32, wy as f32)
            })
            .collect();
        self.keyframes
            .push_back(Keyframe { pose, points: with_normals(&world, (pose.x as f32, pose.y as f32)) });
        while self.keyframes.len() > self.cfg.max_keyframes {
            self.keyframes.pop_front();
        }
        self.map = LocalMap::build(&self.keyframes);
        self.pose = pose;
    }

    fn icp(&self, init: Pose2) -> (Pose2, MatchInfo) {
        let mut t = init;
        let mut info = MatchInfo::default();
        let n = self.body.len().max(1) as f64;
        for it in 0..self.cfg.max_iterations {
            let max_d = if it < 4 {
                0.25
            } else if it < 8 {
                0.15
            } else {
                0.08
            };
            let (s, c) = t.th.sin_cos();
            let mut h = [[0f64; 3]; 3];
            let mut g = [0f64; 3];
            let (mut inl, mut sq) = (0usize, 0f64);
            for &(bx, by) in &self.body {
                let (bx, by) = (bx as f64, by as f64);
                let qx = t.x + c * bx - s * by;
                let qy = t.y + s * bx + c * by;
                let Some(m) = self.map.nearest(qx as f32, qy as f32, max_d) else {
                    continue;
                };
                let (nx, ny) = (m.nx as f64, m.ny as f64);
                // The surface must face the sensor; otherwise this is the back side of a wall.
                if nx * (t.x - m.x as f64) + ny * (t.y - m.y as f64) < 0.0 {
                    continue;
                }
                let e = nx * (qx - m.x as f64) + ny * (qy - m.y as f64);
                let j = [nx, ny, nx * -(qy - t.y) + ny * (qx - t.x)];
                // Maximum-likelihood weighting: far returns are noisier (datasheet: 4 mm STD < 2 m,
                // 15 mm beyond), then Huber to tame wrong correspondences.
                let w_noise = (0.004 / range_sigma((bx * bx + by * by).sqrt())).powi(2).min(1.0);
                let w = w_noise * if e.abs() <= self.cfg.huber { 1.0 } else { self.cfg.huber / e.abs() };
                for r in 0..3 {
                    g[r] += w * j[r] * e;
                    for col in 0..3 {
                        h[r][col] += w * j[r] * j[col];
                    }
                }
                if e.abs() < self.cfg.inlier_dist {
                    inl += 1;
                    sq += e * e;
                }
            }
            // Tiny damping keeps degenerate cases (long corridor) from exploding along the free axis.
            for (r, row) in h.iter_mut().enumerate() {
                row[r] += 1e-3 * n * if r == 2 { 0.1 } else { 1.0 };
            }
            let Some(delta) = solve3(&h, &[-g[0], -g[1], -g[2]]) else {
                break;
            };
            t = Pose2::new(t.x + delta[0], t.y + delta[1], t.th + delta[2]);
            info.iterations = it as u32 + 1;
            info.inlier_ratio = inl as f64 / n;
            info.rms = if inl > 0 { (sq / inl as f64).sqrt() } else { f64::INFINITY };
            if it >= 8 && delta[0].abs() < 1e-5 && delta[1].abs() < 1e-5 && delta[2].abs() < 1e-5 {
                break;
            }
        }
        info.accepted = info.inlier_ratio >= self.cfg.min_inlier_ratio;
        // Reject implausible jumps even if they "fit" (e.g. sliding to a parallel wall).
        let jump = init.between(&t);
        if jump.dist() > 0.5 || wrap_angle(jump.th).abs() > 30f64.to_radians() {
            info.accepted = false;
        }
        (t, info)
    }
}

fn solve3(a: &[[f64; 3]; 3], b: &[f64; 3]) -> Option<[f64; 3]> {
    let det = |m: &[[f64; 3]; 3]| {
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    };
    let d = det(a);
    if d.abs() < 1e-12 {
        return None;
    }
    let mut x = [0f64; 3];
    for (i, xi) in x.iter_mut().enumerate() {
        let mut m = *a;
        for r in 0..3 {
            m[r][i] = b[r];
        }
        *xi = det(&m) / d;
    }
    Some(x)
}
