//! Assembles packets into whole revolutions and converts them to the robot frame.
//!
//! The old app split revolutions on *packet* boundaries, so every other sweep overlapped the
//! previous one by a few degrees (492/504 points alternating). Here the split happens per point,
//! at the exact 360 -> 0 crossing, like LDROBOT's own SDK. Every point also gets its own
//! measurement time, which motion de-skewing and IMU fusion need.

use crate::clock::LidarClock;
use crate::protocol::{Packet, POINTS_PER_PACKET};

/// Where the LiDAR sits on the robot. `yaw` is the robot-frame angle of the sensor's 0° mark.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Mount {
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScanPoint {
    /// Robot frame, metres.
    pub x: f32,
    pub y: f32,
    /// Range from the LiDAR (m) and bearing in the robot frame (rad, CCW).
    pub r: f32,
    pub a: f32,
    pub intensity: u8,
    /// Host monotonic time of the measurement, seconds.
    pub t: f64,
}

#[derive(Clone, Debug, Default)]
pub struct Scan {
    pub seq: u64,
    pub t_start: f64,
    pub t_end: f64,
    /// Valid points (distance > 0), in measurement order.
    pub points: Vec<ScanPoint>,
    /// Points the sensor reported as 0 (no return).
    pub no_return: u32,
    /// Revolution rate reported by the sensor, Hz.
    pub rev_hz: f32,
}

/// Safety valve: a stalled motor would otherwise grow one "revolution" forever.
const MAX_POINTS_PER_SCAN: usize = 2_000;

pub struct ScanAssembler {
    pub mount: Mount,
    pub clock: LidarClock,
    cur: Scan,
    last_angle: f32,
    seen_wrap: bool,
    seq: u64,
    speed_sum: f32,
    speed_n: u32,
    pub partial_dropped: u32,
}

impl ScanAssembler {
    pub fn new(mount: Mount) -> Self {
        ScanAssembler {
            mount,
            clock: LidarClock::new(),
            cur: Scan::default(),
            last_angle: -1.0,
            seen_wrap: false,
            seq: 0,
            speed_sum: 0.0,
            speed_n: 0,
            partial_dropped: 0,
        }
    }

    /// Feeds one packet. Its valid points (robot frame) are written to `packet_points` so the
    /// safety zones can react without waiting for the revolution to finish. Returns a scan when
    /// this packet completed one.
    pub fn push(&mut self, p: &Packet, packet_points: &mut Vec<ScanPoint>) -> Option<Scan> {
        packet_points.clear();
        let t_pkt = self.clock.to_host(p.timestamp_ms, p.host_us as f64 * 1e-6);
        let speed = (p.speed_dps.max(1)) as f64;
        let step_deg = p.span_cdeg() as f64 / 100.0 / (POINTS_PER_PACKET - 1) as f64;
        let mut done = None;
        for i in 0..POINTS_PER_PACKET {
            let angle = p.angle_deg(i);
            if angle < self.last_angle - 180.0 || self.cur.points.len() >= MAX_POINTS_PER_SCAN {
                done = self.finish();
            }
            self.last_angle = angle;
            let rp = p.points[i];
            let t = t_pkt + i as f64 * step_deg / speed;
            if self.cur.points.is_empty() && self.cur.no_return == 0 {
                self.cur.t_start = t;
            }
            self.cur.t_end = t;
            if rp.dist_mm == 0 {
                self.cur.no_return += 1;
                continue;
            }
            let r = rp.dist_mm as f32 * 1e-3;
            // Sensor angles grow clockwise; the robot frame is counter-clockwise.
            let a = self.mount.yaw - angle.to_radians();
            let (s, c) = a.sin_cos();
            let sp = ScanPoint {
                x: self.mount.x + r * c,
                y: self.mount.y + r * s,
                r,
                a,
                intensity: rp.intensity,
                t,
            };
            self.cur.points.push(sp);
            packet_points.push(sp);
        }
        self.speed_sum += p.speed_dps as f32;
        self.speed_n += 1;
        done
    }

    fn finish(&mut self) -> Option<Scan> {
        let mut scan = std::mem::take(&mut self.cur);
        self.cur.points = Vec::with_capacity(scan.points.capacity().max(600));
        let rev_hz = if self.speed_n > 0 {
            self.speed_sum / self.speed_n as f32 / 360.0
        } else {
            0.0
        };
        self.speed_sum = 0.0;
        self.speed_n = 0;
        if !self.seen_wrap {
            // The first revolution started wherever the stream happened to start.
            self.seen_wrap = true;
            self.partial_dropped += 1;
            return None;
        }
        scan.seq = self.seq;
        scan.rev_hz = rev_hz;
        self.seq += 1;
        Some(scan)
    }
}
