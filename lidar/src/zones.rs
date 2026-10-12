//! Safety zones: for each of N sectors around the robot, the clearance (gap between the robot's
//! outline and the nearest obstacle). Updated per packet, i.e. every ~2.4 ms, so a sector's value
//! is never older than one revolution (100 ms) plus processing.
//!
//! Designed to be sent to the Mega (see docs/electronics/PI_MEGA_LINK.md), which caps speed per
//! direction together with its ToF sensors. A sector that has not been refreshed recently is
//! reported as stale; the Mega must treat stale as "blocked" (fail-safe if the LiDAR stalls).

use crate::filter::Footprint;
use crate::scan::ScanPoint;
use std::f32::consts::TAU;

/// 0.25° bins: fine enough that sector boundaries (at e.g. 11.25°) are not smeared.
const BINS: usize = 1440;

#[derive(Clone, Debug)]
pub struct ZoneConfig {
    pub sectors: usize,
    pub footprint: Footprint,
    /// Returns closer than this to the outline count as the robot itself and are ignored
    /// (negative = inside the outline).
    pub self_margin: f32,
    /// Intensity below which a single return is ignored (LDROBOT treats <= 15 as noise).
    pub min_intensity: u8,
    /// A sector older than this is stale.
    pub stale_after_s: f64,
    /// Clearance reported when nothing is seen (no return within range).
    pub max_clearance: f32,
    /// Each return stands for a beam this wide on either side of its bearing (half the 0.72°
    /// sample spacing, plus margin), so slivers at sector boundaries are never missed.
    pub beam_half_width_deg: f32,
}

impl Default for ZoneConfig {
    fn default() -> Self {
        ZoneConfig {
            sectors: 16,
            footprint: Footprint::default(),
            self_margin: 0.02,
            min_intensity: 16,
            stale_after_s: 0.25,
            max_clearance: 12.0,
            beam_half_width_deg: 0.4,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ZoneReport {
    pub t: f64,
    /// Sector 0 is centred straight ahead; sectors go counter-clockwise (left first).
    pub clearance: Vec<f32>,
    pub stale: Vec<bool>,
}

impl ZoneReport {
    /// Compact payload for the Pi -> Mega link: `t_ms u32 | n u8 | stale bitmask u32 | n x clearance_mm u16`.
    /// 16 sectors = 41 bytes, inside the link's 60-byte payload limit.
    pub fn encode(&self) -> Vec<u8> {
        let mut b = Vec::with_capacity(9 + 2 * self.clearance.len());
        b.extend_from_slice(&((self.t * 1000.0) as u64 as u32).to_le_bytes());
        b.push(self.clearance.len() as u8);
        let mask = self.stale.iter().enumerate().fold(0u32, |m, (i, &s)| m | ((s as u32) << i));
        b.extend_from_slice(&mask.to_le_bytes());
        for &c in &self.clearance {
            b.extend_from_slice(&((c * 1000.0).round().clamp(0.0, 65_535.0) as u16).to_le_bytes());
        }
        b
    }

    pub fn min_clearance(&self) -> f32 {
        self.clearance.iter().copied().fold(f32::INFINITY, f32::min)
    }
}

/// Receives zone updates; the Mega link will implement this.
pub trait ObstacleSink: Send {
    fn on_zones(&mut self, report: &ZoneReport);
}

pub struct Zones {
    pub cfg: ZoneConfig,
    /// Per 0.25° bearing bin (robot frame): nearest clearance seen in the latest pass and when.
    bin_clear: [f32; BINS],
    bin_time: [f64; BINS],
}

impl Zones {
    pub fn new(cfg: ZoneConfig) -> Self {
        Zones { cfg, bin_clear: [f32::INFINITY; BINS], bin_time: [f64::NEG_INFINITY; BINS] }
    }

    fn bin_of(a: f32) -> usize {
        ((a.rem_euclid(TAU) / TAU * BINS as f32) as usize).min(BINS - 1)
    }

    /// Feeds the points of one packet. `swept` is the robot-frame bearing range (start, end) the
    /// packet covered, including directions with no return, so emptied directions clear.
    pub fn update(&mut self, points: &[ScanPoint], swept: Option<(f32, f32)>, t: f64) {
        if let Some((a0, a1)) = swept {
            // Sensor turns clockwise = decreasing robot bearing: walk from a1 up to a0.
            let (b0, b1) = (Self::bin_of(a1), Self::bin_of(a0));
            let mut b = b0;
            loop {
                self.bin_clear[b] = f32::INFINITY;
                self.bin_time[b] = t;
                if b == b1 {
                    break;
                }
                b = (b + 1) % BINS;
            }
        }
        for p in points {
            if p.intensity < self.cfg.min_intensity {
                continue;
            }
            if self.cfg.footprint.contains(p.x, p.y, self.cfg.self_margin) {
                continue;
            }
            let c = self.cfg.footprint.clearance(p.x, p.y);
            // Binned by LiDAR bearing, the same angle the sweep clearing uses (the LiDAR sits at
            // the robot origin on SENNEP, so this is also the bearing from the robot centre).
            let hw = self.cfg.beam_half_width_deg.to_radians();
            let (b0, b1) = (Self::bin_of(p.a - hw), Self::bin_of(p.a + hw));
            let mut b = b0;
            loop {
                if self.bin_time[b] < t {
                    self.bin_clear[b] = c;
                    self.bin_time[b] = t;
                } else {
                    self.bin_clear[b] = self.bin_clear[b].min(c);
                }
                if b == b1 {
                    break;
                }
                b = (b + 1) % BINS;
            }
        }
    }

    pub fn report(&self, now: f64) -> ZoneReport {
        let n = self.cfg.sectors;
        let mut clearance = vec![self.cfg.max_clearance; n];
        let mut stale = vec![false; n];
        let width = BINS as f32 / n as f32;
        for b in 0..BINS {
            // Sector 0 is centred on bearing 0: shift by half a sector. Use the bin centre.
            let s = (((b as f32 + 0.5 + width * 0.5) / width) as usize) % n;
            if now - self.bin_time[b] > self.cfg.stale_after_s {
                stale[s] = true;
            }
            clearance[s] = clearance[s].min(self.bin_clear[b]);
        }
        ZoneReport { t: now, clearance, stale }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pt(x: f32, y: f32, t: f64) -> ScanPoint {
        ScanPoint { x, y, r: x.hypot(y), a: y.atan2(x), intensity: 200, t }
    }

    #[test]
    fn sectors_and_staleness() {
        let mut z = Zones::new(ZoneConfig::default());
        // Full sweep with an obstacle 0.5 m in front of the bumper and one on the left.
        z.update(&[pt(0.9, 0.0, 1.0), pt(0.0, 1.0, 1.0)], Some((TAU - 0.001, 0.0)), 1.0);
        let r = z.report(1.05);
        assert!((r.clearance[0] - 0.5).abs() < 1e-5);
        assert!((r.clearance[4] - (1.0 - 0.325)).abs() < 1e-5);
        assert_eq!(r.clearance[8], 12.0);
        assert!(r.stale.iter().all(|s| !s));
        // The obstacle in front leaves: the next pass over that direction clears it.
        z.update(&[], Some((0.1, -0.1)), 1.1);
        assert_eq!(z.report(1.1).clearance[0], 12.0);
        // Sensor stops: everything becomes stale.
        assert!(z.report(2.0).stale.iter().all(|&s| s));
        assert_eq!(r.encode().len(), 9 + 32);
    }
}
