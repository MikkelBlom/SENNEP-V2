//! `sennep-lidar`: runs the LiDAR stack and serves the UI.
//!
//!   sennep-lidar --serial /dev/serial/by-id/usb-Silicon_Labs_CP2102...   (the robot)
//!   sennep-lidar --replay recording.ldr [--loop]                          (PC development)
//!   sennep-lidar --sim [--person]                                         (no hardware at all)
//!
//! Options: --http ADDR (default 0.0.0.0:8080), --record FILE.ldr, --no-body (LiDAR not on the robot).

use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use sennep_lidar::filter::{FilterConfig, Footprint};
use sennep_lidar::pipeline::{Output, Pipeline, PipelineConfig};
use sennep_lidar::server::{serve, Hub};
use sennep_lidar::sim::{SimConfig, Simulator, Trajectory, World};
use sennep_lidar::source::{ByteSource, Recorder, ReplaySource, SerialSource, SimSource};
use sennep_lidar::sys::{rss_mb, CpuMeter};

fn usage() -> ! {
    eprintln!(
        "usage: sennep-lidar (--serial PATH | --replay FILE [--loop] | --sim [--person]) \
         [--http ADDR] [--record FILE.ldr] [--no-body]"
    );
    std::process::exit(2)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let has = |f: &str| args.iter().any(|a| a == f);
    let val = |f: &str| {
        args.iter().position(|a| a == f).map(|i| args.get(i + 1).cloned().unwrap_or_else(|| usage()))
    };

    let mut source: Box<dyn ByteSource> = if let Some(p) = val("--serial") {
        Box::new(SerialSource::open(&p).unwrap_or_else(|e| {
            eprintln!("cannot open {p}: {e}");
            std::process::exit(1)
        }))
    } else if let Some(p) = val("--replay") {
        Box::new(ReplaySource::open(&PathBuf::from(&p), true, has("--loop")).unwrap_or_else(|e| {
            eprintln!("cannot read {p}: {e}");
            std::process::exit(1)
        }))
    } else if has("--sim") {
        let world = if has("--person") { World::apartment().with_person() } else { World::apartment() };
        // Loop the tour forever by chaining runs.
        Box::new(LoopingSim::new(world))
    } else {
        usage()
    };

    let mut cfg = PipelineConfig::default();
    if has("--no-body") {
        cfg.filter = FilterConfig { self_mask: None, ..FilterConfig::default() };
        cfg.zones.footprint = Footprint { length: 0.0, width: 0.0 };
        cfg.zones.self_margin = -1.0;
    }
    let footprint = cfg.zones.footprint;
    let mut recorder =
        val("--record").map(|p| Recorder::create(&PathBuf::from(p)).expect("cannot create recording"));

    let hub = Arc::new(Hub::new());
    let addr = val("--http").unwrap_or_else(|| "0.0.0.0:8080".into());
    serve(&addr, hub.clone()).unwrap_or_else(|e| {
        eprintln!("cannot listen on {addr}: {e}");
        std::process::exit(1)
    });

    let mut pipeline = Pipeline::new(cfg);
    let mut buf = Vec::with_capacity(4096);
    let mut cpu = CpuMeter::new();
    let mut last_stats = Instant::now();
    let mut last = (0.0f64, 0.0f64, 0.0f64, false, 0.0f64, 0usize, 0.0f32);
    let describe = source.describe();
    eprintln!("source: {describe}");
    loop {
        let host_us = match source.read_chunk(&mut buf) {
            Ok(Some(t)) => t,
            Ok(None) => {
                eprintln!("source ended");
                break;
            }
            Err(e) => {
                eprintln!("read error: {e}");
                std::thread::sleep(Duration::from_millis(200));
                continue;
            }
        };
        if let Some(r) = &mut recorder {
            if let Err(e) = r.write(host_us, &buf) {
                eprintln!("recording stopped: {e}");
                recorder = None;
            }
        }
        if hub.reset_requested.swap(false, Ordering::Relaxed) {
            pipeline.reset_map();
            hub.reset_map();
        }
        let mut grid_changed = false;
        pipeline.feed(&buf, host_us, &mut |o| match o {
            Output::Zones(z) => hub.publish_zones(z),
            Output::Scan(r) => {
                hub.publish_scan(r);
                last = (
                    r.pose.x,
                    r.pose.y,
                    r.pose.th,
                    r.info.accepted,
                    r.info.inlier_ratio,
                    r.raw_count,
                    r.rev_hz,
                );
                grid_changed = true;
            }
        });
        if grid_changed {
            if let Some(g) = &mut pipeline.grid {
                hub.publish_grid(g);
            }
        }
        if last_stats.elapsed() >= Duration::from_secs(1) {
            last_stats = Instant::now();
            let (p50, p99, max) = pipeline.packet_us.summary();
            let (s50, s99, smax) = pipeline.scan_us.summary();
            let st = pipeline.parser.stats;
            let json = serde_json::json!({
                "source": describe,
                "rev_hz": last.6,
                "points": last.5,
                "pose": [last.0, last.1, last.2],
                "accepted": last.3,
                "inliers": last.4,
                "keyframes": pipeline.odom.keyframe_count(),
                "packet_us": [p50, p99, max],
                "scan_us": [s50, s99, smax],
                "cpu_pct": cpu.sample(),
                "rss_mb": rss_mb().map(|r| r.0),
                "parser": { "packets": st.packets, "crc_errors": st.crc_errors, "skipped_bytes": st.skipped_bytes },
                "clients": hub.clients.load(Ordering::Relaxed),
                "footprint": [footprint.length, footprint.width],
            });
            hub.publish_stats(json.to_string());
            if let Some(r) = &mut recorder {
                let _ = r.flush();
            }
        }
    }
    if let Some(r) = &mut recorder {
        let _ = r.flush();
    }
}

/// The simulator tour, restarted when it ends, paced in real time.
struct LoopingSim {
    world: World,
    inner: SimSource,
    offset_us: u64,
    last_us: u64,
    run: u64,
}

impl LoopingSim {
    fn new(world: World) -> Self {
        let inner = SimSource::new(
            Simulator::new(world.clone(), Trajectory::tour_round_trip(), SimConfig::default()),
            true,
        );
        LoopingSim { world, inner, offset_us: 0, last_us: 0, run: 0 }
    }
}

impl ByteSource for LoopingSim {
    fn read_chunk(&mut self, buf: &mut Vec<u8>) -> std::io::Result<Option<u64>> {
        loop {
            if let Some(t) = self.inner.read_chunk(buf)? {
                self.last_us = t + self.offset_us;
                return Ok(Some(self.last_us));
            }
            self.run += 1;
            self.offset_us = self.last_us;
            let cfg = SimConfig { seed: 1 + self.run, ..SimConfig::default() };
            self.inner =
                SimSource::new(Simulator::new(self.world.clone(), Trajectory::tour_round_trip(), cfg), true);
        }
    }
    fn describe(&self) -> String {
        "simulator (tour + return, looping)".into()
    }
}
