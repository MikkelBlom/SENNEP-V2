#![allow(clippy::field_reassign_with_default)]
//! Wall lines and moving-thing tracking: simulator with ground truth + Mikkel's room recording.
mod common;
use common::*;
use sennep_lidar::eval::SIM_EPOCH_S;
use sennep_lidar::geom::Pose2;
use sennep_lidar::pipeline::{Output, Pipeline, PipelineConfig};
use sennep_lidar::sim::*;
use sennep_lidar::source::{ByteSource, ReplaySource, SimSource};
use sennep_lidar::walls::WallOut;
use std::collections::{BTreeSet, HashMap};

struct Frame {
    t: f64,
    pose: Pose2,
    /// Confirmed tracks: (id, x, y, coasting)
    tracks: Vec<(u32, f64, f64, bool)>,
    walls: Option<Vec<WallOut>>,
}

fn drive(src: &mut dyn ByteSource, cfg: PipelineConfig) -> (Vec<Frame>, Pipeline) {
    let mut p = Pipeline::new(cfg);
    let mut buf = Vec::new();
    let mut frames = Vec::new();
    while let Some(t) = src.read_chunk(&mut buf).unwrap() {
        let mut done = Vec::new();
        p.feed(&buf, t, &mut |o| {
            if let Output::Scan(r) = o {
                done.push((r.t_end, r.pose, r.walls_updated));
            }
        });
        for (t, pose, walls) in done {
            let tracks = p.tracker.visible().map(|k| (k.id, k.pos().0, k.pos().1, k.coasting())).collect();
            frames.push(Frame { t, pose, tracks, walls: walls.then(|| p.walls.output()) });
        }
    }
    (frames, p)
}

fn ids(frames: &[Frame]) -> BTreeSet<u32> {
    frames.iter().flat_map(|f| f.tracks.iter().map(|t| t.0)).collect()
}

#[test]
fn driving_robot_sees_no_ghost_movers() {
    // The real risk: pose error while driving makes walls look like they moved.
    let world = World::apartment();
    let traj = Trajectory::tour_round_trip();
    let mut src = SimSource::new(Simulator::new(world, traj, SimConfig::default()), false);
    let (frames, _) = drive(&mut src, PipelineConfig::default());
    let ghost = ids(&frames);
    let ghost_scans = frames.iter().filter(|f| !f.tracks.is_empty()).count();
    println!("{} scans, ghost tracks {:?} in {} scans", frames.len(), ghost, ghost_scans);
    assert!(ghost_scans * 100 <= frames.len(), "ghost movers in {ghost_scans} of {} scans", frames.len());
}

#[test]
fn simulated_person_is_tracked() {
    let world = World::apartment().with_person();
    let traj = Trajectory::tour();
    let mover = world.movers[0];
    let mut src = SimSource::new(Simulator::new(world, traj.clone(), SimConfig::default()), false);
    let (frames, _) = drive(&mut src, PipelineConfig::default());
    // map frame -> world: world = truth0 ∘ est0⁻¹ ∘ map
    let to_world = traj.at(frames[0].t - SIM_EPOCH_S).compose(&frames[0].pose.inverse());
    let (mut near, mut seen, mut far) = (0, 0, 0);
    for f in &frames {
        let truth = mover.at(f.t - SIM_EPOCH_S);
        for &(_, x, y, coasting) in &f.tracks {
            if coasting {
                continue;
            }
            seen += 1;
            let (wx, wy) = to_world.apply(x, y);
            // The LiDAR only sees the near side of the 18 cm "person": the centroid sits in front of the centre.
            if (wx - truth.0).hypot(wy - truth.1) < 0.30 {
                near += 1;
            } else {
                far += 1;
            }
        }
    }
    println!("track detections {seen}: {near} on the person, {far} elsewhere; ids {:?}", ids(&frames));
    assert!(seen > frames.len() / 4, "person tracked in only {seen} of {} scans", frames.len());
    assert!(near as f64 >= seen as f64 * 0.95, "{far} of {seen} track positions off the person");
}

#[test]
fn walls_lie_on_real_walls() {
    let world = World::apartment();
    let traj = Trajectory::tour();
    let mut src = SimSource::new(Simulator::new(world.clone(), traj.clone(), SimConfig::default()), false);
    let (frames, p) = drive(&mut src, PipelineConfig::default());
    let to_world = traj.at(frames[0].t - SIM_EPOCH_S).compose(&frames[0].pose.inverse());
    let walls = p.walls.output();
    let total: f32 = walls.iter().map(|w| (w.b.0 - w.a.0).hypot(w.b.1 - w.a.1)).sum();
    let mut worst = 0.0f64;
    for w in &walls {
        for k in 0..=10 {
            let s = k as f32 / 10.0;
            let (x, y) = (w.a.0 + (w.b.0 - w.a.0) * s, w.a.1 + (w.b.1 - w.a.1) * s);
            worst = worst.max(world.static_distance(to_world.apply(x as f64, y as f64)));
        }
    }
    let real: f64 = world.segments.iter().map(|s| (s.b.0 - s.a.0).hypot(s.b.1 - s.a.1)).sum();
    println!(
        "{} walls, {:.1} m of {:.1} m real wall, worst point {:.1} mm off",
        walls.len(),
        total,
        real,
        worst * 1e3
    );
    assert!(worst < 0.08, "a wall line is {:.0} mm from any real wall", worst * 1e3);
    assert!(total as f64 > real * 0.5, "only {total:.1} m of walls found");
}

fn room_recording() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("recordings/2026-10-12_room_walkthrough.ldr")
}

#[test]
fn room_recording_one_walker_no_ghosts() {
    // LiDAR on a table; Mikkel walks in at ~11 s, around the LiDAR and out at ~20.5 s.
    let mut src = ReplaySource::open(&room_recording(), false, false).unwrap();
    let (frames, _) = drive(&mut src, bench_config());
    let t0 = frames[0].t;
    let before = frames.iter().filter(|f| f.t - t0 < 10.5 && !f.tracks.is_empty()).count();
    let during = frames.iter().filter(|f| (12.0..20.0).contains(&(f.t - t0)));
    let (n, tracked) = during.fold((0, 0), |(n, k), f| (n + 1, k + f.tracks.iter().any(|t| !t.3) as usize));
    println!(
        "before: {before} scans with tracks; during: tracked in {tracked} of {n}; ids {:?}",
        ids(&frames)
    );
    assert_eq!(before, 0);
    assert!(tracked * 100 >= n * 95, "walker tracked in {tracked} of {n} scans");
    assert_eq!(ids(&frames).len(), 1, "the walker must keep one id");
}

#[test]
fn room_recording_walls_hold_still() {
    let mut src = ReplaySource::open(&room_recording(), false, false).unwrap();
    let (frames, p) = drive(&mut src, bench_config());
    let t0 = frames[0].t;
    let mut last: HashMap<u32, WallOut> = HashMap::new();
    let mut worst = (0.0f32, 0u32, 0.0f64);
    let mut ids_late = BTreeSet::new();
    for f in &frames {
        let Some(ws) = &f.walls else { continue };
        let settled = f.t - t0 > 5.0; // the first seconds are the map filling in
        for w in ws {
            if let (true, Some(o)) = (settled, last.get(&w.id)) {
                let m = (w.a.0 - o.a.0).hypot(w.a.1 - o.a.1).max((w.b.0 - o.b.0).hypot(w.b.1 - o.b.1));
                if m > worst.0 {
                    worst = (m, w.id, f.t - t0);
                }
            }
            if settled {
                ids_late.insert(w.id);
            }
            last.insert(w.id, *w);
        }
    }
    let final_ids: BTreeSet<u32> = p.walls.output().iter().map(|w| w.id).collect();
    println!(
        "worst endpoint step after 5 s: {:.1} mm (wall {} at {:.1} s); walls {} at end, {} ids seen after 5 s",
        worst.0 * 1e3,
        worst.1,
        worst.2,
        final_ids.len(),
        ids_late.len()
    );
    // One end cell (5 cm) changing state moves an end by a few cm at most; the lines themselves hold to mm.
    assert!(worst.0 < 0.04, "a wall jumped {:.0} mm", worst.0 * 1e3);
    assert!(ids_late.len() <= final_ids.len() + 1, "walls appear and vanish: {ids_late:?} vs {final_ids:?}");
}

#[test]
fn room_2_sprints_and_bending_down_keep_one_id() {
    // Mikkel: walks in (~26 s), stands still (60–82 s), sprints to the far wall and back three times
    // (~3.5 m/s), bends down below the scan plane to fix the carpet (~91 s), walks out (~104 s).
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("recordings/2026-10-12_room_2.ldr");
    let mut src = ReplaySource::open(&path, false, false).unwrap();
    let (frames, _) = drive(&mut src, bench_config());
    let t0 = frames[0].t;
    let ids = ids(&frames);
    let before = frames.iter().filter(|f| f.t - t0 < 25.0 && !f.tracks.is_empty()).count();
    let still = frames.iter().filter(|f| (62.0..80.0).contains(&(f.t - t0)));
    let (n, seen) = still.fold((0, 0), |(n, k), f| (n + 1, k + f.tracks.iter().any(|t| !t.3) as usize));
    println!(
        "ids {ids:?}; before entering: {before} scans with tracks; standing still: seen in {seen} of {n}"
    );
    assert_eq!(before, 0);
    assert_eq!(ids.len(), 1, "one person, one id: {ids:?}");
    assert!(seen * 100 >= n * 95, "standing still: seen in {seen} of {n}");
}

#[test]
fn room_3_glass_kettle_and_corner_keep_one_id() {
    // Mikkel alone (~16 s to ~189 s, back in at ~213 s). A glass electric kettle flickers all the time
    // (laser partly through glass); a thin static thing near his path gives 3–7 point blobs; at ~81 s he
    // is out of view for ~3.5 s and reappears 0.5 m away.
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("recordings/2026-10-12_room_3.ldr");
    let mut src = ReplaySource::open(&path, false, false).unwrap();
    let (frames, _) = drive(&mut src, bench_config());
    let t0 = frames[0].t;
    let first_visit: BTreeSet<u32> = frames
        .iter()
        .filter(|f| (17.0..188.0).contains(&(f.t - t0)))
        .flat_map(|f| f.tracks.iter().map(|t| t.0))
        .collect();
    let all = ids(&frames);
    println!("first visit ids {first_visit:?}, all {all:?}");
    assert_eq!(first_visit.len(), 1, "one person, one id during the first visit");
    assert!(all.len() <= 2, "kettle or furniture became a mover: {all:?}");
}
