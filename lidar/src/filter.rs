//! Point filtering for odometry and mapping. The safety zones deliberately do NOT use this:
//! a thin chair leg is "noise" for scan matching but very real for a bumper.

use crate::scan::ScanPoint;

/// Robot outline in the robot frame (centred on the robot origin). Default: Body v0, 800 x 650 mm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Footprint {
    pub length: f32,
    pub width: f32,
}

impl Default for Footprint {
    fn default() -> Self {
        Footprint { length: 0.80, width: 0.65 }
    }
}

impl Footprint {
    /// Distance from the outline to a point (0 inside).
    pub fn clearance(&self, x: f32, y: f32) -> f32 {
        let dx = (x.abs() - self.length * 0.5).max(0.0);
        let dy = (y.abs() - self.width * 0.5).max(0.0);
        dx.hypot(dy)
    }
    pub fn contains(&self, x: f32, y: f32, margin: f32) -> bool {
        x.abs() <= self.length * 0.5 + margin && y.abs() <= self.width * 0.5 + margin
    }
}

#[derive(Clone, Debug)]
pub struct FilterConfig {
    pub min_range: f32,
    pub max_range: f32,
    /// Drop returns inside the robot's own outline (mast, cables, shell edge) plus this margin.
    pub self_mask: Option<(Footprint, f32)>,
    /// LDROBOT-style near filter: isolated groups of < 3 points below this mean intensity are dropped.
    pub near_filter_intensity: Option<u8>,
}

impl Default for FilterConfig {
    fn default() -> Self {
        FilterConfig {
            min_range: 0.05,
            max_range: 12.0,
            self_mask: Some((Footprint::default(), 0.02)),
            near_filter_intensity: Some(220),
        }
    }
}

/// Points are grouped while consecutive bearings are within this (2 nominal steps at 10 Hz)...
const GROUP_MAX_DA: f32 = 1.5 * std::f32::consts::PI / 180.0;
/// ...and ranges within this fraction of each other (mixed pixels on edges break groups).
const GROUP_MAX_DR: f32 = 0.03;
const NEAR_FILTER_RANGE: f32 = 5.0;

pub fn filter_points(points: &[ScanPoint], cfg: &FilterConfig, out: &mut Vec<ScanPoint>) {
    out.clear();
    let keep = |p: &ScanPoint| {
        p.r >= cfg.min_range
            && p.r <= cfg.max_range
            && !cfg.self_mask.is_some_and(|(f, m)| f.contains(p.x, p.y, m))
    };
    let Some(thresh) = cfg.near_filter_intensity else {
        out.extend(points.iter().filter(|p| keep(p)));
        return;
    };
    // Walk groups of neighbouring points; a group is judged as a whole.
    let mut i = 0;
    while i < points.len() {
        let mut j = i + 1;
        while j < points.len() {
            let (a, b) = (&points[j - 1], &points[j]);
            let da = (a.a - b.a).abs();
            let da = da.min(std::f32::consts::TAU - da);
            if da > GROUP_MAX_DA || (a.r - b.r).abs() > GROUP_MAX_DR * a.r.max(b.r) {
                break;
            }
            j += 1;
        }
        let group = &points[i..j];
        let near = group.iter().any(|p| p.r < NEAR_FILTER_RANGE);
        let mean_int = group.iter().map(|p| p.intensity as u32).sum::<u32>() / group.len() as u32;
        let isolated_and_weak = group.len() < 3 && mean_int < thresh as u32;
        if !(near && isolated_and_weak) {
            out.extend(group.iter().filter(|p| keep(p)));
        }
        i = j;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pt(a_deg: f32, r: f32, intensity: u8) -> ScanPoint {
        let a = a_deg.to_radians();
        ScanPoint { x: r * a.cos(), y: r * a.sin(), r, a, intensity, t: 0.0 }
    }

    #[test]
    fn drops_isolated_weak_points_and_self_hits() {
        let mut pts: Vec<ScanPoint> = (0..20).map(|i| pt(i as f32 * 0.72, 2.0, 200)).collect();
        pts.push(pt(14.6, 1.0, 100)); // lone mixed pixel
        pts.extend((0..20).map(|i| pt(15.4 + i as f32 * 0.72, 3.0, 180)));
        pts.push(pt(40.0, 0.30, 255)); // inside the robot outline (mast)
        let mut out = Vec::new();
        filter_points(&pts, &FilterConfig::default(), &mut out);
        assert_eq!(out.len(), 40);
        assert!(out.iter().all(|p| p.r > 1.5));
    }

    #[test]
    fn clearance_to_outline() {
        let f = Footprint { length: 0.8, width: 0.6 };
        assert!((f.clearance(1.4, 0.0) - 1.0).abs() < 1e-6);
        assert!((f.clearance(0.0, -0.8) - 0.5).abs() < 1e-6);
        assert!((f.clearance(0.7, 0.7) - 0.5).abs() < 1e-6);
        assert_eq!(f.clearance(0.1, 0.1), 0.0);
    }
}
