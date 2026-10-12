#![allow(clippy::field_reassign_with_default, clippy::needless_range_loop)]
//! Simulator runs with known ground truth: drift, jitter, map quality, zone correctness.
mod common;
use common::*;
use sennep_lidar::eval::Score;
use sennep_lidar::geom::{wrap_angle, Pose2};
use sennep_lidar::pipeline::PipelineConfig;
use sennep_lidar::sim::*;
use sennep_lidar::source::{SimSource, SIM_HOST_EPOCH_US};

const EPOCH: f64 = SIM_HOST_EPOCH_US as f64 * 1e-6;

fn score(run: &Run, traj: &Trajectory, world: &World) -> Score {
    sennep_lidar::eval::score(&run.poses, &run.accepted, run.pipeline.grid.as_ref(), traj, world)
}

fn simulate(world: World, traj: Trajectory, cfg: SimConfig) -> (Run, Score) {
    let sim = Simulator::new(world.clone(), traj.clone(), cfg);
    let mut src = SimSource::new(sim, false);
    let run = run(&mut src, PipelineConfig::default());
    let s = score(&run, &traj, &world);
    println!(
        "scans {} accepted {:.1}% | ATE {:.1} mm | end error {:.1} mm ({:.2}% of {:.1} m), {:.2} deg | map precision {:.1}% recall {:.1}% | keyframes {}",
        run.poses.len(),
        s.accepted * 100.0,
        s.ate_rms * 1e3,
        s.final_pos_err * 1e3,
        s.final_pos_err / s.path_len.max(1e-9) * 100.0,
        s.path_len,
        s.final_th_err.to_degrees(),
        s.map_precision * 100.0,
        s.map_recall * 100.0,
        run.pipeline.odom.keyframe_count(),
    );
    (run, s)
}

#[test]
fn stationary_robot_does_not_drift() {
    let traj = Trajectory::stationary(Pose2::new(-2.0, -1.0, 0.4), 10.0);
    let (run, s) = simulate(World::apartment(), traj, SimConfig::default());
    let max = run.poses.iter().map(|p| run.poses[0].1.between(&p.1).dist()).fold(0.0, f64::max);
    let xs: Vec<f64> = run.poses.iter().map(|p| p.1.x).collect();
    let ys: Vec<f64> = run.poses.iter().map(|p| p.1.y).collect();
    println!("static: std x {:.2} mm y {:.2} mm, max {:.2} mm", std(&xs) * 1e3, std(&ys) * 1e3, max * 1e3);
    assert!(std(&xs) < 0.002 && std(&ys) < 0.002);
    assert!(max < 0.008, "max deviation {max}");
    assert!(s.final_th_err.to_degrees() < 0.1);
    assert!(s.map_precision > 0.95);
}

#[test]
fn tour_odometry_and_map_hold() {
    let (_, s) = simulate(World::apartment(), Trajectory::tour(), SimConfig::default());
    assert!(s.accepted > 0.98, "accepted {}", s.accepted);
    assert!(s.final_pos_err / s.path_len < 0.02, "drift {} m over {} m", s.final_pos_err, s.path_len);
    assert!(s.final_th_err.to_degrees() < 1.5, "heading error {}", s.final_th_err.to_degrees());
    assert!(s.map_precision > 0.90, "precision {}", s.map_precision);
    assert!(s.map_recall > 0.80, "recall {}", s.map_recall);
}

#[test]
fn tour_with_walking_person_and_wire_errors() {
    let cfg = SimConfig { byte_error_rate: 1e-4, seed: 7, ..SimConfig::default() };
    let (run, s) = simulate(World::apartment().with_person(), Trajectory::tour(), cfg);
    let st = run.pipeline.parser.stats;
    println!("parser {st:?}");
    assert!(st.crc_errors > 0, "errors were injected");
    assert!(s.accepted > 0.95);
    assert!(s.final_pos_err / s.path_len < 0.03);
    assert!(s.map_precision > 0.85, "person must not leave a trail: precision {}", s.map_precision);
}

#[test]
fn without_deskew_it_is_worse() {
    // Shows the de-skew earns its keep: same drive, de-skew off.
    let world = World::apartment();
    let traj = Trajectory::tour();
    let (_, with) = simulate(world.clone(), traj.clone(), SimConfig::default());
    let sim = Simulator::new(world.clone(), traj.clone(), SimConfig::default());
    let mut cfg = PipelineConfig::default();
    cfg.odom.deskew = false;
    let run = run(&mut SimSource::new(sim, false), cfg);
    let without = score(&run, &traj, &world);
    println!("ATE with de-skew {:.1} mm, without {:.1} mm", with.ate_rms * 1e3, without.ate_rms * 1e3);
    assert!(with.ate_rms < without.ate_rms);
}

#[test]
fn zones_match_ground_truth() {
    let world = World::apartment().with_person();
    let traj = Trajectory::tour();
    let sim = Simulator::new(world.clone(), traj.clone(), SimConfig::default());
    let run = run(&mut SimSource::new(sim, false), PipelineConfig::default());
    let cfg = sennep_lidar::zones::ZoneConfig::default();
    let n = cfg.sectors;
    let hw = cfg.beam_half_width_deg as f64;
    let width = 360.0 / n as f64;
    // `widen` > 0: a ray counts for a sector if it is within `widen` degrees of it (beam width).
    // `widen` < 0: only if it is at least that far inside (an object that merely grazes a sector
    // edge may legitimately not be there yet). `with_thin`: include objects narrower than ~3
    // samples (a 4 cm chair leg at 2-4 m), which a real LiDAR hits only sometimes.
    let truth_at = |t: f64, with_thin: bool, widen: f64| {
        let pose = traj.at(t);
        let mut visible = world.clone();
        visible.circles.retain(|c| {
            let d = (c.c.0 - pose.x).hypot(c.c.1 - pose.y);
            with_thin || (2.0 * c.r / d).to_degrees() > 2.2
        });
        let mut truth = vec![cfg.max_clearance as f64; n];
        for k in 0..3600 {
            let a = k as f64 * 0.1f64.to_radians();
            let dir = (pose.th + a).sin_cos();
            if let Some(r) = visible.cast((pose.x, pose.y), (dir.1, dir.0), t).filter(|&r| r < 12.0) {
                let c = cfg.footprint.clearance((r * a.cos()) as f32, (r * a.sin()) as f32) as f64;
                let deg = a.to_degrees();
                for s in 0..n {
                    // signed distance (deg) from the ray to the sector's edges, positive inside
                    let off = (deg - s as f64 * width + 180.0).rem_euclid(360.0) - 180.0;
                    if width / 2.0 - off.abs() >= -widen {
                        truth[s] = truth[s].min(c);
                    }
                }
            }
        }
        truth
    };
    let (mut worst_opt, mut worst_cons, mut checked, mut stale) = (0.0f64, 0.0f64, 0, 0);
    for z in run.zones.iter().skip(20).step_by(5) {
        // A sector holds the latest pass over it, i.e. something from the last revolution
        // (100 ms). So the honest truth is the range of values over that window.
        let t = z.t - EPOCH;
        let turn = wrap_angle(traj.at(t).th - traj.at(t - 0.1).th).abs().to_degrees();
        let snaps: Vec<f64> = (0..=4).map(|i| t - 0.025 * i as f64).collect();
        let lo_w: Vec<Vec<f64>> = snaps.iter().map(|&ts| truth_at(ts, true, hw)).collect();
        let hi_w: Vec<Vec<f64>> = snaps.iter().map(|&ts| truth_at(ts, false, -(hw + turn))).collect();
        for s in 0..n {
            let lo = lo_w.iter().map(|w| w[s]).fold(f64::INFINITY, f64::min);
            let hi = hi_w.iter().map(|w| w[s]).fold(0.0, f64::max);
            let v = z.clearance[s] as f64;
            // Sensor noise biases a minimum low (datasheet STD 15 mm beyond 2 m), and 0.72°
            // sampling can step past the very closest point of a corner.
            let tol = 0.02 + 3.0 * range_sigma(v + 0.4) + 0.006 * v;
            let optimistic = (v - hi - tol).max(0.0); // more room than there is: dangerous
            let conservative = (lo - v - tol).max(0.0); // less room than there is: safe, but bounded
            if optimistic > 0.0 || conservative > 0.0 {
                println!("t {t:.2} sector {s}: reported {v:.3}, truth over last rev {lo:.3}..{hi:.3}");
            }
            worst_opt = worst_opt.max(optimistic);
            worst_cons = worst_cons.max(conservative);
            checked += 1;
            stale += z.stale[s] as usize;
        }
    }
    println!(
        "checked {checked} sector values; beyond truth window + noise: optimistic {:.1} mm, conservative {:.1} mm; stale {stale}",
        worst_opt * 1e3,
        worst_cons * 1e3
    );
    assert_eq!(stale, 0);
    assert_eq!(worst_opt, 0.0, "zones reported more room than there was");
    assert!(worst_cons < 0.10, "conservative by {worst_cons}");
}

#[test]
fn returning_to_known_places_cancels_drift() {
    // Out and back the same way: the robot must re-lock onto the keyframes it made on the way out.
    let (_, s) =
        simulate(World::apartment().with_person(), Trajectory::tour_round_trip(), SimConfig::default());
    assert!(s.final_pos_err < 0.02, "back at start but {} m off", s.final_pos_err);
    assert!(s.final_th_err.to_degrees() < 0.3);
    assert!(s.map_precision > 0.90);
}
