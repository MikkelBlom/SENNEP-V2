#![allow(clippy::field_reassign_with_default, clippy::needless_range_loop)]
//! The old app's recordings (static LiDAR on a desk), replayed through the real parser.
mod common;
use common::*;
use sennep_lidar::source::ReplaySource;

#[test]
fn every_revolution_is_whole_and_unique() {
    for path in recordings() {
        let mut src = ReplaySource::open(&path, false, false).unwrap();
        let run = run(&mut src, bench_config());
        let st = run.pipeline.parser.stats;
        assert_eq!(st.crc_errors, 0, "{path:?}");
        assert!(run.scans.len() >= 20, "{path:?}: {} scans", run.scans.len());
        for (i, s) in run.scans.iter().enumerate() {
            // Old app: 492/504 alternating with overlap. Now: one point per sample slot, no overlap.
            let mut angles: Vec<f32> = s
                .iter()
                .map(|p| p.a.rem_euclid(std::f32::consts::TAU))
                .collect();
            angles.sort_by(f32::total_cmp);
            let gaps = angles.windows(2).map(|w| w[1] - w[0]).fold(0f32, f32::max);
            let wrap = std::f32::consts::TAU - angles.last().unwrap() + angles[0];
            assert!(
                gaps.max(wrap).to_degrees() < 3.0,
                "{path:?} scan {i}: hole {}",
                gaps.max(wrap).to_degrees()
            );
            // The seam: last point of the revolution must sit exactly one step before the first
            // (bearings decrease as the sensor turns clockwise). The old app overlapped here.
            let seam = (s.last().unwrap().a - s[0].a)
                .rem_euclid(std::f32::consts::TAU)
                .to_degrees();
            assert!(
                seam > 0.0 && seam < 3.0,
                "{path:?} scan {i}: seam {seam} deg"
            );
            // Elsewhere the sensor itself occasionally overlaps two packets by a fraction of a degree.
            let dup = angles
                .windows(2)
                .filter(|w| (w[1] - w[0]).to_degrees() < 0.3)
                .count();
            assert!(dup <= 3, "{path:?} scan {i}: {dup} overlapping points");
        }
    }
}

#[test]
fn static_pose_is_rock_steady() {
    for path in recordings() {
        let mut src = ReplaySource::open(&path, false, false).unwrap();
        let run = run(&mut src, bench_config());
        let xs: Vec<f64> = run.poses.iter().map(|p| p.1.x).collect();
        let ys: Vec<f64> = run.poses.iter().map(|p| p.1.y).collect();
        let ths: Vec<f64> = run.poses.iter().map(|p| p.1.th).collect();
        let max_dev = run.poses.iter().map(|p| p.1.dist()).fold(0.0, f64::max);
        println!(
            "{:?}: std x {:.2} mm, y {:.2} mm, th {:.3} deg; max |xy| {:.2} mm; keyframes {}",
            path.file_name().unwrap(),
            std(&xs) * 1e3,
            std(&ys) * 1e3,
            std(&ths).to_degrees(),
            max_dev * 1e3,
            run.pipeline.odom.keyframe_count()
        );
        assert!(
            run.accepted.iter().filter(|a| **a).count() >= run.accepted.len() - 1,
            "{path:?}"
        );
        assert!(std(&xs) < 0.005 && std(&ys) < 0.005, "{path:?}");
        assert!(std(&ths).to_degrees() < 0.2, "{path:?}");
        assert!(max_dev < 0.02, "{path:?}");
    }
}
