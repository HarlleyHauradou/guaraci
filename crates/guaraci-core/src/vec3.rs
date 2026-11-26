//! Minimal 3D vector.

use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }
    pub fn from_array(a: [f64; 3]) -> Self {
        Vec3::new(a[0], a[1], a[2])
    }
    pub fn dot(self, o: Vec3) -> f64 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }
    pub fn norm(self) -> f64 {
        self.dot(self).sqrt()
    }
    pub fn mul_elem(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x * o.x, self.y * o.y, self.z * o.z)
    }
    pub fn div_elem(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x / o.x, self.y / o.y, self.z / o.z)
    }
    pub fn abs(self) -> Vec3 {
        Vec3::new(self.x.abs(), self.y.abs(), self.z.abs())
    }
    pub fn max_elem(self) -> f64 {
        self.x.max(self.y).max(self.z)
    }
    pub fn to_array(self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
    pub fn normalized(self) -> Vec3 {
        self / self.norm()
    }

    /// New direction after scattering by polar angle (cos θ) and azimuth φ
    /// relative to the unit vector `self`.
    pub fn rotate(self, cos_theta: f64, phi: f64) -> Vec3 {
        let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
        let (sin_phi, cos_phi) = phi.sin_cos();
        let (u, v, w) = (self.x, self.y, self.z);
        let a = (1.0 - w * w).sqrt();
        if a < 1e-10 {
            let s = w.signum();
            return Vec3::new(sin_theta * cos_phi, sin_theta * sin_phi, s * cos_theta);
        }
        Vec3::new(
            u * cos_theta + sin_theta * (u * w * cos_phi - v * sin_phi) / a,
            v * cos_theta + sin_theta * (v * w * cos_phi + u * sin_phi) / a,
            w * cos_theta - a * sin_theta * cos_phi,
        )
        .normalized()
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}
impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}
impl Mul<f64> for Vec3 {
    type Output = Vec3;
    fn mul(self, s: f64) -> Vec3 {
        Vec3::new(self.x * s, self.y * s, self.z * s)
    }
}
impl Div<f64> for Vec3 {
    type Output = Vec3;
    fn div(self, s: f64) -> Vec3 {
        Vec3::new(self.x / s, self.y / s, self.z / s)
    }
}
impl Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}
