//! Bytes in, zones + poses + map out. Single-threaded and deterministic: the same bytes always
//! give the same result, which is what lets recordings and the simulator act as regression tests.

use std::time::Instant;

use crate::filter::{filter_points, FilterConfig};
use crate::geom::Pose2;
use crate::grid::{Grid, GridConfig};
use crate::odom::{MatchInfo, OdomConfig, Odometry};
use crate::protocol::{Packet, Parser, POINTS_PER_PACKET};
use crate::scan::{Mount, Scan, ScanAssembler, ScanPoint};
use crate::zones::{ZoneConfig, ZoneReport, Zones};

#[derive(Clone, Debug)]
pub struct PipelineConfig {
    pub mount: Mount,
    pub filter: FilterConfig,
    pub zones: ZoneConfig,
    pub odom: OdomConfig,
    pub grid: Option<GridConfig>,
    /// Zone reports per second (the Mega link budget is ~20–50 Hz).
    pub zone_rate_hz: f64,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        PipelineConfig {
            mount: Mount::default(),
            filter: FilterConfig::default(),
            zones: ZoneConfig::default(),
            odom: OdomConfig::default(),
            grid: Some(GridConfig::default()),
            zone_rate_hz: 50.0,
        }
    }
}

/// Fixed-size window of recent timings for live diagnostics.
#[derive(Clone, Debug)]
pub struct Window {
    buf: Vec<u32>,
    next: usize,
    full: bool,
}

impl Window {
    pub fn new(n: usize) -> Self {
        Window {
            buf: vec![0; n],
            next: 0,
            full: false,
        }
    }
    pub fn push(&mut self, v: u32) {
        self.buf[self.next] = v;
        self.next = (self.next + 1) % self.buf.len();
        self.full |= self.next == 0;
    }
    /// (p50, p99, max) of the window.
    pub fn summary(&self) -> (u32, u32, u32) {
        let n = if self.full { self.buf.len() } else { self.next };
        if n == 0 {
            return (0, 0, 0);
        }
        let mut v = self.buf[..n].to_vec();
        v.sort_unstable();
        (v[n / 2], v[(n * 99 / 100).min(n - 1)], v[n - 1])
    }
}

pub struct ScanResult {
    pub seq: u64,
    pub t_end: f64,
    pub pose: Pose2,
    pub info: MatchInfo,
    pub rev_hz: f32,
    pub raw_count: usize,
    pub no_return: u32,
    /// Filtered, de-skewed points in the robot frame at `t_end`.
    pub points: Vec<(f32, f32)>,
    /// All valid points of the revolution (robot frame), for display.
    pub raw: Vec<ScanPoint>,
    /// Processing time for this scan (filter + odometry + grid), µs.
    pub proc_us: u32,
}

pub enum Output<'a> {
    Zones(&'a ZoneReport),
    Scan(&'a ScanResult),
}

pub struct Pipeline {
    pub cfg: PipelineConfig,
    pub parser: Parser,
    pub assembler: ScanAssembler,
    pub zones: Zones,
    pub odom: Odometry,
    pub grid: Option<Grid>,
    pkt_points: Vec<ScanPoint>,
    filtered: Vec<ScanPoint>,
    last_zone_t: f64,
    pub packet_us: Window,
    pub scan_us: Window,
    pub scans: u64,
}

impl Pipeline {
    pub fn new(cfg: PipelineConfig) -> Pipeline {
        Pipeline {
            parser: Parser::new(),
            assembler: ScanAssembler::new(cfg.mount),
            zones: Zones::new(cfg.zones.clone()),
            odom: Odometry::new(cfg.odom.clone()),
            grid: cfg.grid.clone().map(Grid::new),
            pkt_points: Vec::with_capacity(POINTS_PER_PACKET),
            filtered: Vec::with_capacity(600),
            last_zone_t: f64::NEG_INFINITY,
            packet_us: Window::new(4096),
            scan_us: Window::new(256),
            scans: 0,
            cfg,
        }
    }

    pub fn feed(&mut self, bytes: &[u8], host_us: u64, out: &mut impl FnMut(Output)) {
        // The parser hands packets to a closure; collect them first so `self` is free afterwards.
        let mut packets: [Option<Packet>; 64] = [None; 64];
        let mut n = 0;
        let mut rest = bytes;
        while !rest.is_empty() {
            let take = rest.len().min(47 * 60);
            let (chunk, tail) = rest.split_at(take);
            rest = tail;
            self.parser.push(chunk, host_us, &mut |p| {
                packets[n] = Some(p);
                n += 1;
            });
            for p in packets[..n].iter_mut().filter_map(Option::take) {
                self.on_packet(&p, out);
            }
            n = 0;
        }
    }

    fn on_packet(&mut self, p: &Packet, out: &mut impl FnMut(Output)) {
        let t0 = Instant::now();
        let finished = self.assembler.push(p, &mut self.pkt_points);
        let t_pkt = self.assembler_last_time();
        let yaw = self.cfg.mount.yaw;
        let half_step = 0.5f32.to_radians();
        let a_first = yaw - p.angle_deg(0).to_radians() + half_step;
        let a_last = yaw - p.angle_deg(POINTS_PER_PACKET - 1).to_radians() - half_step;
        self.zones
            .update(&self.pkt_points, Some((a_first, a_last)), t_pkt);
        self.packet_us.push(t0.elapsed().as_micros() as u32);
        if t_pkt - self.last_zone_t >= 1.0 / self.cfg.zone_rate_hz {
            self.last_zone_t = t_pkt;
            let report = self.zones.report(t_pkt);
            out(Output::Zones(&report));
        }
        if let Some(scan) = finished {
            let r = self.process_scan(scan);
            out(Output::Scan(&r));
        }
    }

    fn assembler_last_time(&self) -> f64 {
        self.pkt_points
            .last()
            .map(|p| p.t)
            .unwrap_or(self.last_zone_t.max(0.0))
    }

    pub fn process_scan(&mut self, scan: Scan) -> ScanResult {
        let t0 = Instant::now();
        filter_points(&scan.points, &self.cfg.filter, &mut self.filtered);
        let (pose, info) = self.odom.process(&self.filtered, scan.t_end);
        let points = self.odom.deskewed().to_vec();
        if let Some(g) = &mut self.grid {
            if info.accepted {
                g.integrate(&pose, (self.cfg.mount.x, self.cfg.mount.y), &points);
            }
        }
        self.scans += 1;
        let proc_us = t0.elapsed().as_micros() as u32;
        self.scan_us.push(proc_us);
        ScanResult {
            seq: scan.seq,
            t_end: scan.t_end,
            pose,
            info,
            rev_hz: scan.rev_hz,
            raw_count: scan.points.len(),
            no_return: scan.no_return,
            points,
            raw: scan.points,
            proc_us,
        }
    }
}
