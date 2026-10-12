//! `sennep-lidar`: runs the LiDAR stack and serves the UI.
//!
//!   sennep-lidar --serial /dev/serial/by-id/usb-Silicon_Labs_CP2102...   (the robot)
//!   sennep-lidar --replay recording.ldr [--loop]                          (PC development)
//!   sennep-lidar --sim [--person]                                         (no hardware at all)
//!
//! Options: --http ADDR (default 0.0.0.0:8080), --no-body (LiDAR not on the robot),
//!   --record-dir DIR (default lidar/recordings or recordings), --no-record (don't start recording by itself),
//!   --record FILE.ldr (record into this file instead of auto/).
//! A live LiDAR is recorded automatically into <record-dir>/auto/; the page can keep, mark, stop and start,
//! and play any recording back (play/pause/jump/speed) and return to live.

use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use sennep_lidar::pipeline::{Output, Pipeline, PipelineConfig};
use sennep_lidar::player::{self, Player};
use sennep_lidar::recording::Recordings;
use sennep_lidar::server::{serve, Hub};
use sennep_lidar::sim::{SimConfig, Simulator, Trajectory, World};
use sennep_lidar::source::{ByteSource, ReplaySource, SerialSource, SimSource};
use sennep_lidar::sys::{rss_mb, CpuMeter};

fn usage() -> ! {
    eprintln!(
        "usage: sennep-lidar (--serial PATH | --replay FILE [--loop] | --sim [--person]) \
         [--http ADDR] [--no-body] [--record-dir DIR] [--no-record] [--record FILE.ldr]"
    );
    std::process::exit(2)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let has = |f: &str| args.iter().any(|a| a == f);
    let val = |f: &str| {
        args.iter().position(|a| a == f).map(|i| args.get(i + 1).cloned().unwrap_or_else(|| usage()))
    };

    let serial_path = val("--serial");
    let open_serial = |p: &str| -> Box<dyn ByteSource> {
        Box::new(SerialSource::open(p).unwrap_or_else(|e| {
            eprintln!("cannot open {p}: {e}");
            std::process::exit(1)
        }))
    };
    let mut mode = if let Some(p) = &serial_path {
        Mode::Live(open_serial(p))
    } else if let Some(p) = val("--replay") {
        if p.ends_with(".jsonl") {
            // The old Electron app's recordings: no player controls, just paced replay.
            Mode::Live(Box::new(ReplaySource::open(&PathBuf::from(&p), true, has("--loop")).unwrap_or_else(
                |e| {
                    eprintln!("cannot read {p}: {e}");
                    std::process::exit(1)
                },
            )))
        } else {
            Mode::Play(Player::open(&PathBuf::from(&p), has("--loop")).unwrap_or_else(|e| {
                eprintln!("cannot read {p}: {e}");
                std::process::exit(1)
            }))
        }
    } else if has("--sim") {
        let world = if has("--person") { World::apartment().with_person() } else { World::apartment() };
        // Loop the tour forever by chaining runs.
        Mode::Live(Box::new(LoopingSim::new(world)))
    } else {
        usage()
    };

    let cfg = if has("--no-body") { PipelineConfig::off_robot() } else { PipelineConfig::default() };
    let footprint = cfg.zones.footprint;
    let live = serial_path.is_some();
    let dir = val("--record-dir").map(PathBuf::from).unwrap_or_else(Recordings::default_dir);
    let mut recordings = Recordings::new(dir, live || val("--record").is_some());
    let auto_record = live && !has("--no-record") && val("--record").is_none();
    if let Some(p) = val("--record") {
        recordings.start_at(&PathBuf::from(p));
    } else if auto_record {
        recordings.start();
    }

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
    let mut files = (Instant::now() - Duration::from_secs(60), Vec::new());
    let mut last = (0.0f64, 0.0f64, 0.0f64, false, 0.0f64, 0usize, 0.0f32);
    eprintln!("source: {}", mode.describe());
    loop {
        // Commands from the page first, so they also work while playback is paused.
        let mut restart = false;
        for cmd in std::mem::take(&mut *hub.commands.lock().unwrap()) {
            let (verb, arg) = cmd.split_once(' ').unwrap_or((cmd.as_str(), ""));
            match (verb, &mut mode) {
                ("play:open", _) => match Player::open(&PathBuf::from(arg), false) {
                    Ok(p) => {
                        eprintln!("playing {arg}");
                        recordings.stop();
                        mode = Mode::Play(p);
                        restart = true;
                    }
                    Err(e) => eprintln!("cannot play {arg}: {e}"),
                },
                ("play:live", Mode::Play(_)) => {
                    if let Some(p) = &serial_path {
                        mode = Mode::Live(open_serial(p));
                        restart = true;
                        if auto_record {
                            recordings.start();
                        }
                    }
                }
                ("play:toggle", Mode::Play(p)) => {
                    let from_start = p.paused && p.at_end();
                    p.set_paused(!p.paused);
                    restart |= from_start;
                }
                ("play:speed", Mode::Play(p)) => p.set_speed(arg.parse().unwrap_or(1.0)),
                ("play:seek" | "play:step", Mode::Play(p)) => {
                    let v: f64 = arg.parse().unwrap_or(0.0);
                    let target = if verb == "play:step" { p.pos_s() + v } else { v };
                    pipeline.reset_map();
                    hub.reset_map();
                    p.seek(target, |b, t| pipeline.feed(b, t, &mut |_| {}));
                    if let Some(g) = &mut pipeline.grid {
                        g.mark_all_dirty();
                        hub.publish_grid(g);
                    }
                    hub.publish_walls(&pipeline.walls);
                    last_stats = Instant::now() - Duration::from_secs(2);
                }
                (_, Mode::Live(_)) if cmd.starts_with("rec:") => recordings.command(&cmd),
                _ => {}
            }
        }
        if restart || hub.reset_requested.swap(false, Ordering::Relaxed) {
            pipeline.reset_map();
            hub.reset_map();
        }
        let host_us = match &mut mode {
            Mode::Live(src) => match src.read_chunk(&mut buf) {
                Ok(Some(t)) => Some(t),
                Ok(None) => {
                    eprintln!("source ended");
                    break;
                }
                Err(e) => {
                    eprintln!("read error: {e}");
                    std::thread::sleep(Duration::from_millis(200));
                    None
                }
            },
            Mode::Play(p) => p.poll(&mut buf),
        };
        if let Some(host_us) = host_us {
            if matches!(mode, Mode::Live(_)) {
                recordings.write(host_us, &buf);
            }
            let mut grid_changed = false;
            let mut walls_changed = false;
            let mut motion: Option<(f64, Vec<(f32, f32)>)> = None;
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
                    walls_changed |= r.walls_updated;
                    motion = Some((r.t_end, r.moving.clone()));
                }
            });
            if grid_changed {
                if let Some(g) = &mut pipeline.grid {
                    hub.publish_grid(g);
                }
            }
            if walls_changed {
                hub.publish_walls(&pipeline.walls);
            }
            if let Some((t, moving)) = motion {
                hub.publish_motion(t, &pipeline.tracker, &moving);
            }
        }
        if last_stats.elapsed() >= Duration::from_secs(1) {
            last_stats = Instant::now();
            if files.0.elapsed() > Duration::from_secs(5) {
                files = (Instant::now(), player::list(&recordings.keep_dir));
            }
            let (p50, p99, max) = pipeline.packet_us.summary();
            let (s50, s99, smax) = pipeline.scan_us.summary();
            let st = pipeline.parser.stats;
            let player = match &mode {
                Mode::Play(p) => serde_json::json!({
                    "file": p.path.file_stem().map(|s| s.to_string_lossy().to_string()),
                    "path": p.path.display().to_string(),
                    "t": p.pos_s(),
                    "dur": p.duration_s(),
                    "paused": p.paused,
                    "speed": p.speed,
                    "marks": p.marks,
                }),
                Mode::Live(_) => serde_json::Value::Null,
            };
            let json = serde_json::json!({
                "source": mode.describe(),
                "rev_hz": last.6,
                "points": last.5,
                "pose": [last.0, last.1, last.2],
                "accepted": last.3,
                "inliers": last.4,
                "keyframes": pipeline.odom.keyframe_count(),
                "walls": pipeline.walls.walls.len(),
                "objects": pipeline.walls.objects.len(),
                "movers": pipeline.tracker.visible().count(),
                "packet_us": [p50, p99, max],
                "scan_us": [s50, s99, smax],
                "cpu_pct": cpu.sample(),
                "rss_mb": rss_mb().map(|r| r.0),
                "parser": { "packets": st.packets, "crc_errors": st.crc_errors, "skipped_bytes": st.skipped_bytes },
                "clients": hub.clients.load(Ordering::Relaxed),
                "footprint": [footprint.length, footprint.width],
                "rec": if matches!(mode, Mode::Live(_)) { recordings.status() } else { serde_json::Value::Null },
                "player": player,
                "live_available": live,
                "files": files.1,
            });
            hub.publish_stats(json.to_string());
            recordings.flush();
        }
    }
    recordings.stop();
}

/// Where the bytes come from: a live source (LiDAR, simulator) or a recording under UI control.
enum Mode {
    Live(Box<dyn ByteSource>),
    Play(Player),
}

impl Mode {
    fn describe(&self) -> String {
        match self {
            Mode::Live(s) => s.describe(),
            Mode::Play(p) => format!("playback {}", p.path.display()),
        }
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
