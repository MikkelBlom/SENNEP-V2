#![allow(dead_code, clippy::field_reassign_with_default)]
use sennep_lidar::filter::{FilterConfig, Footprint};
use sennep_lidar::geom::Pose2;
use sennep_lidar::pipeline::{Output, Pipeline, PipelineConfig};
use sennep_lidar::scan::ScanPoint;
use sennep_lidar::source::ByteSource;
use sennep_lidar::zones::ZoneReport;
use std::path::PathBuf;

pub struct Run {
    pub poses: Vec<(f64, Pose2)>,
    pub accepted: Vec<bool>,
    pub scans: Vec<Vec<ScanPoint>>,
    pub zones: Vec<ZoneReport>,
    pub pipeline: Pipeline,
}

pub fn run(src: &mut dyn ByteSource, cfg: PipelineConfig) -> Run {
    let mut p = Pipeline::new(cfg);
    let (mut poses, mut accepted, mut scans, mut zones) = (vec![], vec![], vec![], vec![]);
    let mut buf = Vec::new();
    while let Some(t) = src.read_chunk(&mut buf).unwrap() {
        p.feed(&buf, t, &mut |o| match o {
            Output::Scan(r) => {
                poses.push((r.t_end, r.pose));
                accepted.push(r.info.accepted);
                scans.push(r.raw.clone());
            }
            Output::Zones(z) => zones.push(z.clone()),
        });
    }
    Run { poses, accepted, scans, zones, pipeline: p }
}

/// The old recordings were made with the LiDAR on a desk, not on the robot: no body mask.
pub fn bench_config() -> PipelineConfig {
    let mut cfg = PipelineConfig::default();
    cfg.filter = FilterConfig { self_mask: None, ..FilterConfig::default() };
    cfg.zones.footprint = Footprint { length: 0.0, width: 0.0 };
    cfg.zones.self_margin = -1.0;
    cfg
}

pub fn recordings() -> Vec<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../Lidar Test Enviornment/recordings");
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
    v.retain(|p| p.extension().is_some_and(|e| e == "jsonl"));
    v.sort();
    assert!(!v.is_empty());
    v
}

pub fn std(v: &[f64]) -> f64 {
    let m = v.iter().sum::<f64>() / v.len() as f64;
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64).sqrt()
}
