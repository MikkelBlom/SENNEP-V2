//! Scoring against simulator ground truth. Shared by the tests and the `bench` binary so both
//! measure exactly the same thing.

use crate::geom::{wrap_angle, Pose2};
use crate::grid::Grid;
use crate::sim::{Trajectory, World};
use crate::source::SIM_HOST_EPOCH_US;

pub const SIM_EPOCH_S: f64 = SIM_HOST_EPOCH_US as f64 * 1e-6;

#[derive(Clone, Copy, Debug, Default)]
pub struct Score {
    /// RMS position error over all scans (m), both trajectories relative to their first pose.
    pub ate_rms: f64,
    pub final_pos_err: f64,
    pub final_th_err: f64,
    pub path_len: f64,
    pub accepted: f64,
    pub map_precision: f64,
    pub map_recall: f64,
}

/// `poses` are (host time s, estimated pose) per scan.
pub fn score(
    poses: &[(f64, Pose2)],
    accepted: &[bool],
    grid: Option<&Grid>,
    traj: &Trajectory,
    world: &World,
) -> Score {
    let first = poses[0];
    let truth0 = traj.at(first.0 - SIM_EPOCH_S);
    let mut sq = 0.0;
    let mut last = (0.0, 0.0);
    for &(t, est) in poses {
        let rel_est = first.1.between(&est);
        let rel_true = truth0.between(&traj.at(t - SIM_EPOCH_S));
        let e = rel_est.between(&rel_true);
        sq += e.dist().powi(2);
        last = (e.dist(), wrap_angle(rel_true.th - rel_est.th).abs());
    }
    let (map_precision, map_recall) = grid.map_or((0.0, 0.0), |g| map_quality(g, &first.1, &truth0, world));
    Score {
        ate_rms: (sq / poses.len() as f64).sqrt(),
        final_pos_err: last.0,
        final_th_err: last.1,
        path_len: traj.length().0,
        accepted: accepted.iter().filter(|a| **a).count() as f64 / accepted.len().max(1) as f64,
        map_precision,
        map_recall,
    }
}

/// Precision: share of occupied cells within 10 cm of a real wall.
/// Recall: share of observed wall samples (every 5 cm along each wall) with an occupied cell within 10 cm.
pub fn map_quality(g: &Grid, est0: &Pose2, true0: &Pose2, world: &World) -> (f64, f64) {
    // map frame -> world: world = true0 ∘ est0⁻¹ ∘ map
    let to_world = true0.compose(&est0.inverse());
    let to_map = to_world.inverse();
    let occ = Grid::occupied_threshold();
    let (mut good, mut total) = (0usize, 0usize);
    for cy in 0..g.dim {
        for cx in 0..g.dim {
            if g.value(cx, cy) >= occ {
                let (mx, my) = g.cell_center(cx, cy);
                total += 1;
                if world.static_distance(to_world.apply(mx as f64, my as f64)) < 0.10 {
                    good += 1;
                }
            }
        }
    }
    let reach = (0.10 / g.cfg.resolution).ceil() as i64;
    let (mut found, mut seen) = (0usize, 0usize);
    for s in &world.segments {
        let len = (s.b.0 - s.a.0).hypot(s.b.1 - s.a.1);
        let n = (len / 0.05).ceil() as usize;
        for i in 0..=n {
            let k = i as f64 / n as f64;
            let (mx, my) = to_map.apply(s.a.0 + (s.b.0 - s.a.0) * k, s.a.1 + (s.b.1 - s.a.1) * k);
            let cx = ((mx as f32 - g.origin.0) / g.cfg.resolution) as i64;
            let cy = ((my as f32 - g.origin.1) / g.cfg.resolution) as i64;
            let inside = |x: i64, y: i64| x >= 0 && y >= 0 && (x as usize) < g.dim && (y as usize) < g.dim;
            if !inside(cx, cy) || g.value(cx as usize, cy as usize) == 0 {
                continue; // never observed (behind furniture, outside view)
            }
            seen += 1;
            let hit = (-reach..=reach).any(|dy| {
                (-reach..=reach).any(|dx| {
                    let (x, y) = (cx + dx, cy + dy);
                    inside(x, y) && g.value(x as usize, y as usize) >= occ && {
                        let (ox, oy) = g.cell_center(x as usize, y as usize);
                        (ox as f64 - mx).hypot(oy as f64 - my) < 0.10
                    }
                })
            });
            found += hit as usize;
        }
    }
    (good as f64 / total.max(1) as f64, found as f64 / seen.max(1) as f64)
}
