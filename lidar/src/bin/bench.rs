//! Benchmark + quality report. Runs unchanged on a PC or the Pi:
//!
//!   cargo run --release --bin bench -- [--out ../docs/lidar/BENCHMARK.md] [--long] [--recordings DIR]
//!
//! On the Pi with the cross-compiled binary: `./bench --recordings "<repo>/Lidar Test Enviornment/recordings"`.
//!
//! Every scenario's bytes are generated/loaded first, so only the pipeline itself is timed.

use std::path::{Path, PathBuf};
use std::time::Instant;

use sennep_lidar::eval::{score, Score};
use sennep_lidar::geom::Pose2;
use sennep_lidar::pipeline::{Output, Pipeline, PipelineConfig, Window};
use sennep_lidar::sim::{SimConfig, Simulator, Trajectory, World};
use sennep_lidar::source::{load_legacy_jsonl, ByteSource, SimSource};
use sennep_lidar::sys::{cpu_seconds, rss_mb};

struct Result {
    name: String,
    data_s: f64,
    scans: usize,
    packets: u64,
    crc_errors: u64,
    cpu_pct: Option<f64>,
    wall_pct: f64,
    packet_us: (u32, u32, u32),
    scan_ms: (f64, f64, f64),
    rss_delta_mb: Option<f64>,
    jitter: Option<(f64, f64, f64)>,
    sim: Option<Score>,
}

fn collect(src: &mut dyn ByteSource) -> Vec<(u64, Vec<u8>)> {
    let mut out = Vec::new();
    let mut buf = Vec::new();
    while let Some(t) = src.read_chunk(&mut buf).unwrap() {
        out.push((t, buf.clone()));
    }
    out
}

fn std(v: &[f64]) -> f64 {
    let m = v.iter().sum::<f64>() / v.len() as f64;
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64).sqrt()
}

fn run(
    name: &str,
    chunks: &[(u64, Vec<u8>)],
    cfg: PipelineConfig,
    truth: Option<(&Trajectory, &World)>,
    still: bool,
) -> Result {
    let rss0 = rss_mb().map(|r| r.0);
    let mut p = Pipeline::new(cfg);
    p.packet_us = Window::new(1 << 20);
    p.scan_us = Window::new(1 << 16);
    let (mut poses, mut accepted) = (Vec::new(), Vec::new());
    let cpu0 = cpu_seconds();
    let wall0 = Instant::now();
    for (t, b) in chunks {
        p.feed(b, *t, &mut |o| {
            if let Output::Scan(r) = o {
                poses.push((r.t_end, r.pose));
                accepted.push(r.info.accepted);
            }
        });
    }
    let wall = wall0.elapsed().as_secs_f64();
    let cpu = cpu0.and_then(|c0| cpu_seconds().map(|c1| c1 - c0));
    let data_s = (chunks.last().unwrap().0 - chunks[0].0) as f64 * 1e-6;
    let (s50, s99, smax) = p.scan_us.summary();
    let jitter = still.then(|| {
        let xs: Vec<f64> = poses.iter().map(|p| p.1.x).collect();
        let ys: Vec<f64> = poses.iter().map(|p| p.1.y).collect();
        let th: Vec<f64> = poses.iter().map(|p| p.1.th).collect();
        (std(&xs), std(&ys), std(&th))
    });
    let sim = truth.map(|(traj, world)| score(&poses, &accepted, p.grid.as_ref(), traj, world));
    Result {
        name: name.into(),
        data_s,
        scans: poses.len(),
        packets: p.parser.stats.packets,
        crc_errors: p.parser.stats.crc_errors,
        cpu_pct: cpu.map(|c| c / data_s * 100.0),
        wall_pct: wall / data_s * 100.0,
        packet_us: p.packet_us.summary(),
        scan_ms: (s50 as f64 / 1e3, s99 as f64 / 1e3, smax as f64 / 1e3),
        rss_delta_mb: rss0.and_then(|a| rss_mb().map(|b| (b.0 - a).max(0.0))),
        jitter,
        sim,
    }
}

fn machine() -> String {
    let cpu = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("model name") || l.starts_with("Model"))
                .map(|l| l.split(':').nth(1).unwrap_or("").trim().to_string())
        })
        .unwrap_or_else(|| "unknown".into());
    let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0);
    format!("{} ({}), {cores} cores", cpu, std::env::consts::ARCH)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = args.iter().position(|a| a == "--out").and_then(|i| args.get(i + 1)).map(PathBuf::from);
    let long = args.iter().any(|a| a == "--long");
    let rec_dir = args
        .iter()
        .position(|a| a == "--recordings")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../Lidar Test Enviornment/recordings")
        });
    let mut results = Vec::new();

    let mut recs: Vec<PathBuf> = std::fs::read_dir(&rec_dir)
        .map(|d| {
            d.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "jsonl")).collect()
        })
        .unwrap_or_default();
    recs.sort();
    for path in &recs {
        let chunks = load_legacy_jsonl(path).unwrap();
        let name = format!("Recording {}", &path.file_name().unwrap().to_string_lossy()[16..35]);
        results.push(run(&name, &chunks, PipelineConfig::off_robot(), None, true));
    }

    let world = World::apartment();
    let person = World::apartment().with_person();
    let sims: Vec<(&str, World, Trajectory, SimConfig, bool)> = vec![
        (
            "Sim: standing still 30 s",
            world.clone(),
            Trajectory::stationary(Pose2::new(-2.0, -1.0, 0.4), 30.0),
            SimConfig::default(),
            true,
        ),
        ("Sim: trip 8.7 m", world.clone(), Trajectory::tour(), SimConfig::default(), false),
        (
            "Sim: trip + person + noise on the wire",
            person.clone(),
            Trajectory::tour(),
            SimConfig { byte_error_rate: 1e-4, seed: 7, ..SimConfig::default() },
            false,
        ),
        (
            "Sim: trip there and back",
            person.clone(),
            Trajectory::tour_round_trip(),
            SimConfig { seed: 3, ..SimConfig::default() },
            false,
        ),
    ];
    for (name, w, traj, cfg, still) in sims {
        let chunks = collect(&mut SimSource::new(Simulator::new(w.clone(), traj.clone(), cfg), false));
        results.push(run(name, &chunks, PipelineConfig::default(), Some((&traj, &w)), still));
    }
    if long {
        // Ten round trips back to back (~11 min of driving) to look for slow drift or growth.
        let legs_traj = {
            let one = Trajectory::tour_round_trip();
            let mut t = one.clone();
            for _ in 1..10 {
                t = t.append(&one);
            }
            t
        };
        let chunks = collect(&mut SimSource::new(
            Simulator::new(person.clone(), legs_traj.clone(), SimConfig::default()),
            false,
        ));
        results.push(run(
            "Sim: 10 trips there and back",
            &chunks,
            PipelineConfig::default(),
            Some((&legs_traj, &person)),
            false,
        ));
    }

    let mut md = String::new();
    md += "# LiDAR stack v2 — benchmark\n\n";
    md += &format!(
        "Machine: {}. Build: release. Generated by `cargo run --release --bin bench`.\n\n",
        machine()
    );
    if std::env::consts::ARCH != "aarch64" {
        md += "> **Not measured on the Pi.** The Pi 5 (Cortex-A76) is expected to be about 1.5–2.5× slower per core than this machine. \
               Run `bench` on the Pi for the real numbers (see `lidar/README.md`).\n\n";
    }
    md += "All sources are replayed as fast as possible; only the pipeline (parser → zones → scan assembly → filter → odometry → map) is timed.\n";
    md +=
        "*CPU* = CPU time / data duration, i.e. how large a share of one core the stack uses in real time.\n";
    md += "*Per packet* = parser + zone update for 12 points (what the safety zones wait for). *Per revolution* = filter + odometry + map, after the revolution's last point.\n\n";
    md += "## Performance\n\n| Scenario | Data | Revs | Packets | CPU (1 core) | Per packet µs (median/p99/max) | Per revolution ms (median/p99/max) | ΔRAM |\n|---|---:|---:|---:|---:|---:|---:|---:|\n";
    for r in &results {
        md += &format!(
            "| {} | {:.1} s | {} | {}{} | {} | {} / {} / {} | {:.2} / {:.2} / {:.2} | {} |\n",
            r.name,
            r.data_s,
            r.scans,
            r.packets,
            if r.crc_errors > 0 { format!(" ({} CRC errors)", r.crc_errors) } else { String::new() },
            r.cpu_pct.map_or(format!("~{:.1} %", r.wall_pct), |c| format!("{c:.1} %")),
            r.packet_us.0,
            r.packet_us.1,
            r.packet_us.2,
            r.scan_ms.0,
            r.scan_ms.1,
            r.scan_ms.2,
            r.rss_delta_mb.map_or("–".into(), |m| format!("{m:.1} MB")),
        );
    }
    md += "\n## Stability when the robot stands still\n\n| Scenario | STD x | STD y | STD heading |\n|---|---:|---:|---:|\n";
    for r in results.iter().filter(|r| r.jitter.is_some()) {
        let (x, y, th) = r.jitter.unwrap();
        md += &format!("| {} | {:.2} mm | {:.2} mm | {:.3}° |\n", r.name, x * 1e3, y * 1e3, th.to_degrees());
    }
    md += "\n## Odometry and map against the simulator's ground truth\n\n| Scenario | Driven | Matches accepted | ATE (RMS) | Final error | Final heading error | Map precision | Map coverage |\n|---|---:|---:|---:|---:|---:|---:|---:|\n";
    for r in results.iter().filter(|r| r.sim.is_some()) {
        let s = r.sim.unwrap();
        md += &format!(
            "| {} | {:.1} m | {:.1} % | {:.1} mm | {:.1} mm{} | {:.2}° | {:.1} % | {:.1} % |\n",
            r.name,
            s.path_len,
            s.accepted * 100.0,
            s.ate_rms * 1e3,
            s.final_pos_err * 1e3,
            if s.path_len > 0.1 {
                format!(" ({:.2} %)", s.final_pos_err / s.path_len * 100.0)
            } else {
                String::new()
            },
            s.final_th_err.to_degrees(),
            s.map_precision * 100.0,
            s.map_recall * 100.0
        );
    }
    md += "\nMap precision = share of occupied cells less than 10 cm from a real wall. Map coverage = share of the seen walls that are in the map.\n";
    print!("{md}");
    if let Some(p) = out {
        std::fs::write(&p, &md).unwrap();
        eprintln!("wrote {}", p.display());
    }
}
