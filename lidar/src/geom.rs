//! 2D rigid transforms. Robot frame: x forward, y left, angles counter-clockwise (ROS REP 103).

use std::f64::consts::PI;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pose2 {
    pub x: f64,
    pub y: f64,
    pub th: f64,
}

pub fn wrap_angle(a: f64) -> f64 {
    let mut a = (a + PI) % (2.0 * PI);
    if a < 0.0 {
        a += 2.0 * PI;
    }
    a - PI
}

impl Pose2 {
    pub const IDENTITY: Pose2 = Pose2 { x: 0.0, y: 0.0, th: 0.0 };

    pub fn new(x: f64, y: f64, th: f64) -> Self {
        Pose2 { x, y, th: wrap_angle(th) }
    }

    /// `self ∘ other`: apply `other` in the frame of `self`.
    pub fn compose(&self, o: &Pose2) -> Pose2 {
        let (s, c) = self.th.sin_cos();
        Pose2::new(self.x + c * o.x - s * o.y, self.y + s * o.x + c * o.y, self.th + o.th)
    }

    pub fn inverse(&self) -> Pose2 {
        let (s, c) = self.th.sin_cos();
        Pose2::new(-(c * self.x + s * self.y), s * self.x - c * self.y, -self.th)
    }

    /// Pose of `o` expressed in the frame of `self`.
    pub fn between(&self, o: &Pose2) -> Pose2 {
        self.inverse().compose(o)
    }

    pub fn apply(&self, x: f64, y: f64) -> (f64, f64) {
        let (s, c) = self.th.sin_cos();
        (self.x + c * x - s * y, self.y + s * x + c * y)
    }

    /// Linear interpolation from identity (t=0) to self (t=1); fine for the small motions within one scan.
    pub fn scale(&self, t: f64) -> Pose2 {
        Pose2::new(self.x * t, self.y * t, self.th * t)
    }

    pub fn dist(&self) -> f64 {
        self.x.hypot(self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_inverse_roundtrip() {
        let a = Pose2::new(1.0, -2.0, 0.7);
        let b = Pose2::new(-0.3, 0.5, -2.9);
        let ab = a.compose(&b);
        let back = a.between(&ab);
        assert!((back.x - b.x).abs() < 1e-12 && (back.y - b.y).abs() < 1e-12);
        assert!(wrap_angle(back.th - b.th).abs() < 1e-12);
        let id = a.compose(&a.inverse());
        assert!(id.x.abs() < 1e-12 && id.y.abs() < 1e-12 && id.th.abs() < 1e-12);
    }
}
