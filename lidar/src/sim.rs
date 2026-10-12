//! Simulated LD19 in a simulated room, producing the raw byte stream the real sensor would.
//!
//! Recordings from the old app are all stationary, so they cannot show whether odometry and the
//! map hold up while driving. The simulator knows the true pose at every instant, which turns
//! "the map looks fine" into numbers (drift, map IoU, zone correctness).
//! Noise follows the STL-19P datasheet: STD 2 mm < 0.5 m, 4 mm < 2 m, 15 mm beyond.

use crate::geom::Pose2;
use crate::protocol::{Packet, RawPoint, POINTS_PER_PACKET};

/// SplitMix64: tiny, deterministic, good enough for noise.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed)
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    pub fn uniform(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
    pub fn gauss(&mut self) -> f64 {
        let u1 = self.uniform().max(1e-12);
        let u2 = self.uniform();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Segment {
    pub a: (f64, f64),
    pub b: (f64, f64),
}

#[derive(Clone, Copy, Debug)]
pub struct Circle {
    pub c: (f64, f64),
    pub r: f64,
}

/// A circle that moves back and forth between two points (a person walking).
#[derive(Clone, Copy, Debug)]
pub struct Mover {
    pub from: (f64, f64),
    pub to: (f64, f64),
    pub r: f64,
    pub period_s: f64,
}

impl Mover {
    pub fn at(&self, t: f64) -> (f64, f64) {
        let phase = (t / self.period_s).fract();
        let k = if phase < 0.5 { phase * 2.0 } else { 2.0 - phase * 2.0 };
        (self.from.0 + (self.to.0 - self.from.0) * k, self.from.1 + (self.to.1 - self.from.1) * k)
    }
}

#[derive(Clone, Debug, Default)]
pub struct World {
    pub segments: Vec<Segment>,
    pub circles: Vec<Circle>,
    pub movers: Vec<Mover>,
}

fn ray_segment(o: (f64, f64), d: (f64, f64), s: &Segment) -> Option<f64> {
    let e = (s.b.0 - s.a.0, s.b.1 - s.a.1);
    let den = d.0 * e.1 - d.1 * e.0;
    if den.abs() < 1e-12 {
        return None;
    }
    let w = (s.a.0 - o.0, s.a.1 - o.1);
    let t = (w.0 * e.1 - w.1 * e.0) / den;
    let u = (w.0 * d.1 - w.1 * d.0) / den;
    (t > 1e-6 && (0.0..=1.0).contains(&u)).then_some(t)
}

fn ray_circle(o: (f64, f64), d: (f64, f64), c: (f64, f64), r: f64) -> Option<f64> {
    let w = (o.0 - c.0, o.1 - c.1);
    let b = w.0 * d.0 + w.1 * d.1;
    let cc = w.0 * w.0 + w.1 * w.1 - r * r;
    let disc = b * b - cc;
    if disc < 0.0 {
        return None;
    }
    let t = -b - disc.sqrt();
    (t > 1e-6).then_some(t)
}

impl World {
    /// Distance along the ray from `o` in unit direction `d`, at time `t` (for movers).
    pub fn cast(&self, o: (f64, f64), d: (f64, f64), t: f64) -> Option<f64> {
        let mut best: Option<f64> = None;
        let mut take = |v: Option<f64>| {
            if let Some(v) = v {
                if best.is_none_or(|b| v < b) {
                    best = Some(v);
                }
            }
        };
        for s in &self.segments {
            take(ray_segment(o, d, s));
        }
        for c in &self.circles {
            take(ray_circle(o, d, c.c, c.r));
        }
        for m in &self.movers {
            take(ray_circle(o, d, m.at(t), m.r));
        }
        best
    }

    /// Distance from a point to the nearest static surface (for map ground truth).
    pub fn static_distance(&self, p: (f64, f64)) -> f64 {
        let mut best = f64::INFINITY;
        for s in &self.segments {
            let e = (s.b.0 - s.a.0, s.b.1 - s.a.1);
            let l2 = e.0 * e.0 + e.1 * e.1;
            let k = (((p.0 - s.a.0) * e.0 + (p.1 - s.a.1) * e.1) / l2).clamp(0.0, 1.0);
            let q = (s.a.0 + e.0 * k, s.a.1 + e.1 * k);
            best = best.min((p.0 - q.0).hypot(p.1 - q.1));
        }
        for c in &self.circles {
            best = best.min(((p.0 - c.c.0).hypot(p.1 - c.c.1) - c.r).abs());
        }
        best
    }

    fn rect(segs: &mut Vec<Segment>, x0: f64, y0: f64, x1: f64, y1: f64) {
        segs.push(Segment { a: (x0, y0), b: (x1, y0) });
        segs.push(Segment { a: (x1, y0), b: (x1, y1) });
        segs.push(Segment { a: (x1, y1), b: (x0, y1) });
        segs.push(Segment { a: (x0, y1), b: (x0, y0) });
    }

    /// An 8 x 5 m room split by a partition wall with a doorway, furniture, table legs.
    /// Asymmetric on purpose so scan matching has something to lock onto.
    pub fn apartment() -> World {
        let mut s = Vec::new();
        Self::rect(&mut s, -4.0, -2.5, 4.0, 2.5);
        // 10 cm partition wall at x = 0.5 with a 1.4 m doorway (real walls have thickness; a
        // zero-thickness wall seen end-on is invisible to a sampled sensor).
        Self::rect(&mut s, 0.45, -2.5, 0.55, -0.5);
        Self::rect(&mut s, 0.45, 0.9, 0.55, 2.5);
        // sofa, shelf, cabinet
        Self::rect(&mut s, -3.6, 1.6, -1.6, 2.4);
        Self::rect(&mut s, 2.6, -2.4, 3.9, -2.0);
        Self::rect(&mut s, 3.2, 0.9, 3.9, 1.9);
        // a slanted wall corner
        s.push(Segment { a: (-4.0, -1.5), b: (-3.0, -2.5) });
        let mut circles = Vec::new();
        for (x, y) in [(-3.5, -0.3), (-2.9, -0.3), (-3.5, 0.3), (-2.9, 0.3)] {
            circles.push(Circle { c: (x, y), r: 0.02 }); // table legs
        }
        circles.push(Circle { c: (2.0, 1.5), r: 0.15 }); // plant pot
        World { segments: s, circles, movers: Vec::new() }
    }

    pub fn with_person(mut self) -> World {
        // Walks across the lower part of the left room, in view most of the tour but never into the robot.
        self.movers.push(Mover { from: (-1.8, -2.0), to: (-0.3, -1.8), r: 0.18, period_s: 8.0 });
        self
    }
}

/// Robot motion as body-frame velocity commands (mecanum: vx, vy and yaw rate are independent).
#[derive(Clone, Copy, Debug)]
pub struct Leg {
    pub dur_s: f64,
    pub vx: f64,
    pub vy: f64,
    pub w: f64,
}

/// Pose as a function of time, integrated at 1 kHz.
#[derive(Clone, Debug)]
pub struct Trajectory {
    poses: Vec<Pose2>,
    pub dt: f64,
}

impl Trajectory {
    pub fn from_legs(start: Pose2, legs: &[Leg]) -> Trajectory {
        let dt = 1e-3;
        let mut poses = vec![start];
        let mut p = start;
        for leg in legs {
            let n = (leg.dur_s / dt).round() as usize;
            for _ in 0..n {
                p = p.compose(&Pose2::new(leg.vx * dt, leg.vy * dt, leg.w * dt));
                poses.push(p);
            }
        }
        Trajectory { poses, dt }
    }

    pub fn stationary(p: Pose2, dur_s: f64) -> Trajectory {
        Self::from_legs(p, &[Leg { dur_s, vx: 0.0, vy: 0.0, w: 0.0 }])
    }

    /// A tour through both rooms: forward, rotate, strafe, diagonal, through the doorway and back.
    /// Speeds are what a 50 kg mecanum robot indoors would do (0.4 m/s, 60 °/s).
    pub fn tour() -> Trajectory {
        Self::from_legs(Self::TOUR_START, &Self::tour_legs())
    }

    /// The tour and then the same legs backwards, ending exactly where it started (seamless loop).
    pub fn tour_round_trip() -> Trajectory {
        let fwd = Self::tour_legs();
        let back = fwd.iter().rev().map(|l| Leg { dur_s: l.dur_s, vx: -l.vx, vy: -l.vy, w: -l.w });
        let legs: Vec<Leg> = fwd.iter().copied().chain(back).collect();
        Self::from_legs(Self::TOUR_START, &legs)
    }

    const TOUR_START: Pose2 = Pose2 { x: -2.5, y: -0.8, th: 0.0 };

    fn tour_legs() -> Vec<Leg> {
        let r = 60f64.to_radians();
        vec![
            Leg { dur_s: 1.0, vx: 0.0, vy: 0.0, w: 0.0 },
            Leg { dur_s: 4.0, vx: 0.4, vy: 0.0, w: 0.0 },
            Leg { dur_s: 1.5, vx: 0.0, vy: 0.0, w: r },
            Leg { dur_s: 3.0, vx: 0.0, vy: -0.3, w: 0.0 },
            Leg { dur_s: 3.0, vx: 0.3, vy: 0.0, w: 0.2 },
            Leg { dur_s: 3.0, vx: 0.25, vy: 0.15, w: -0.3 },
            Leg { dur_s: 1.5, vx: 0.0, vy: 0.0, w: -r },
            Leg { dur_s: 5.0, vx: 0.4, vy: 0.0, w: 0.0 },
            Leg { dur_s: 3.0, vx: 0.0, vy: 0.0, w: r },
            Leg { dur_s: 6.0, vx: 0.4, vy: 0.0, w: 0.05 },
            Leg { dur_s: 1.0, vx: 0.0, vy: 0.0, w: 0.0 },
        ]
    }

    /// Continues with `other`'s motion from where this trajectory ends.
    pub fn append(&self, other: &Trajectory) -> Trajectory {
        let end = *self.poses.last().unwrap();
        let start = other.poses[0];
        let mut poses = self.poses.clone();
        poses.extend(other.poses.iter().skip(1).map(|p| end.compose(&start.between(p))));
        Trajectory { poses, dt: self.dt }
    }

    pub fn duration(&self) -> f64 {
        (self.poses.len() - 1) as f64 * self.dt
    }

    pub fn at(&self, t: f64) -> Pose2 {
        let i = (t / self.dt).floor().clamp(0.0, (self.poses.len() - 1) as f64) as usize;
        let j = (i + 1).min(self.poses.len() - 1);
        let k = (t / self.dt - i as f64).clamp(0.0, 1.0);
        let (a, b) = (self.poses[i], self.poses[j]);
        Pose2::new(
            a.x + (b.x - a.x) * k,
            a.y + (b.y - a.y) * k,
            a.th + crate::geom::wrap_angle(b.th - a.th) * k,
        )
    }

    /// Total path length (m) and total absolute rotation (rad).
    pub fn length(&self) -> (f64, f64) {
        let mut d = 0.0;
        let mut r = 0.0;
        for w in self.poses.windows(2) {
            d += w[0].between(&w[1]).dist();
            r += crate::geom::wrap_angle(w[1].th - w[0].th).abs();
        }
        (d, r)
    }
}

#[derive(Clone, Debug)]
pub struct SimConfig {
    pub rev_hz: f64,
    pub sample_hz: f64,
    /// LiDAR position on the robot (x, y, yaw of the sensor's 0° mark), matches `scan::Mount`.
    pub mount: Pose2,
    pub noise: bool,
    pub dropout: f64,
    /// Probability per byte of a bit flip on the wire.
    pub byte_error_rate: f64,
    pub seed: u64,
    /// Offset of the sensor's ms counter relative to sim time (it is not zero at boot in practice).
    pub lidar_clock_offset_ms: u64,
}

impl Default for SimConfig {
    fn default() -> Self {
        SimConfig {
            rev_hz: 10.0,
            sample_hz: 5_000.0,
            mount: Pose2::IDENTITY,
            noise: true,
            dropout: 0.005,
            byte_error_rate: 0.0,
            seed: 1,
            lidar_clock_offset_ms: 12_345,
        }
    }
}

pub fn range_sigma(r: f64) -> f64 {
    if r < 0.5 {
        0.002
    } else if r < 2.0 {
        0.004
    } else {
        0.015
    }
}

/// Generates packets in time order.
pub struct Simulator {
    pub world: World,
    pub traj: Trajectory,
    pub cfg: SimConfig,
    rng: Rng,
    sample: u64,
}

impl Simulator {
    pub fn new(world: World, traj: Trajectory, cfg: SimConfig) -> Simulator {
        let rng = Rng::new(cfg.seed);
        Simulator { world, traj, cfg, rng, sample: 0 }
    }

    pub fn sample_time(&self, n: u64) -> f64 {
        n as f64 / self.cfg.sample_hz
    }

    /// Sensor angle (clockwise degrees) of sample `n`. Starts at 123° so the stream begins mid-revolution.
    fn sensor_angle(&self, n: u64) -> f64 {
        (123.0 + 360.0 * self.cfg.rev_hz * self.sample_time(n)) % 360.0
    }

    /// Next packet and the sim time (s) of its last point, or None when the trajectory ends.
    pub fn next_packet(&mut self) -> Option<(Packet, f64)> {
        let n0 = self.sample;
        let t_last = self.sample_time(n0 + POINTS_PER_PACKET as u64 - 1);
        if t_last > self.traj.duration() {
            return None;
        }
        let mut points = [RawPoint::default(); POINTS_PER_PACKET];
        for (i, p) in points.iter_mut().enumerate() {
            let n = n0 + i as u64;
            let t = self.sample_time(n);
            let sensor = self.traj.at(t).compose(&self.cfg.mount);
            let a = sensor.th - self.sensor_angle(n).to_radians();
            let d = (a.cos(), a.sin());
            let hit = self.world.cast((sensor.x, sensor.y), d, t).filter(|&r| (0.03..12.0).contains(&r));
            *p = match hit {
                Some(r) if !(self.cfg.noise && self.rng.uniform() < self.cfg.dropout) => {
                    let r = if self.cfg.noise { r + self.rng.gauss() * range_sigma(r) } else { r };
                    let intensity = (230.0 - 12.0 * r).clamp(40.0, 255.0) as u8;
                    RawPoint { dist_mm: (r * 1000.0).round().clamp(1.0, 65_535.0) as u16, intensity }
                }
                _ => RawPoint::default(),
            };
        }
        let start = self.sensor_angle(n0);
        let end = self.sensor_angle(n0 + POINTS_PER_PACKET as u64 - 1);
        let t0 = self.sample_time(n0);
        let ts = ((t0 * 1000.0) as u64 + self.cfg.lidar_clock_offset_ms) % 30_000;
        self.sample += POINTS_PER_PACKET as u64;
        let pkt = Packet {
            speed_dps: (360.0 * self.cfg.rev_hz).round() as u16,
            start_cdeg: ((start * 100.0).round() as u32 % 36_000) as u16,
            end_cdeg: ((end * 100.0).round() as u32 % 36_000) as u16,
            timestamp_ms: ts as u16,
            points,
            host_us: 0,
        };
        Some((pkt, t_last))
    }

    /// Encodes a packet, optionally corrupting bytes the way a noisy wire would.
    pub fn wire_bytes(&mut self, p: &Packet, out: &mut Vec<u8>) {
        let bytes = p.encode();
        for &b in &bytes {
            let b = if self.cfg.byte_error_rate > 0.0 && self.rng.uniform() < self.cfg.byte_error_rate {
                b ^ (1 << (self.rng.next_u64() % 8))
            } else {
                b
            };
            out.push(b);
        }
    }
}
