//! Constructive solid geometry: nested volumes, each a simple solid.
//!
//! Volumes form a tree. A point belongs to the deepest volume that contains
//! it. Daughters must lie inside their mother and must not overlap each other
//! (checked by [`Geometry::check_overlaps`]).

use crate::vec3::Vec3;

pub const INF: f64 = f64::INFINITY;

/// A solid centred at the origin of its local frame.
#[derive(Debug, Clone, PartialEq)]
pub enum Solid {
    Box {
        half: Vec3,
    },
    Sphere {
        r: f64,
    },
    /// Axis along z.
    Cylinder {
        r: f64,
        half_h: f64,
    },
    Ellipsoid {
        a: Vec3,
    },
}

/// Parameter interval [t0, t1] along a ray inside a solid.
fn interval(solid: &Solid, p: Vec3, d: Vec3) -> Option<(f64, f64)> {
    match *solid {
        Solid::Box { half } => slabs(p, d, half),
        Solid::Sphere { r } => quadratic(d.dot(d), p.dot(d), p.dot(p) - r * r),
        Solid::Ellipsoid { a } => {
            let (ps, ds) = (p.div_elem(a), d.div_elem(a));
            quadratic(ds.dot(ds), ps.dot(ds), ps.dot(ps) - 1.0)
        }
        Solid::Cylinder { r, half_h } => {
            let (z0, z1) = slab(p.z, d.z, half_h)?;
            let a = d.x * d.x + d.y * d.y;
            let c = p.x * p.x + p.y * p.y - r * r;
            let (r0, r1) = if a < 1e-300 {
                if c <= 0.0 {
                    (-INF, INF)
                } else {
                    return None;
                }
            } else {
                quadratic(a, p.x * d.x + p.y * d.y, c)?
            };
            let (t0, t1) = (z0.max(r0), z1.min(r1));
            (t0 < t1).then_some((t0, t1))
        }
    }
}

/// Roots of a t² + 2 b t + c = 0.
fn quadratic(a: f64, b: f64, c: f64) -> Option<(f64, f64)> {
    let disc = b * b - a * c;
    if disc <= 0.0 {
        return None;
    }
    let s = disc.sqrt();
    // numerically stable form
    let q = if b > 0.0 { -(b + s) } else { -(b - s) };
    let (t0, t1) = (q / a, c / q);
    Some(if t0 < t1 { (t0, t1) } else { (t1, t0) })
}

fn slab(p: f64, d: f64, h: f64) -> Option<(f64, f64)> {
    if d.abs() < 1e-300 {
        return (p.abs() <= h).then_some((-INF, INF));
    }
    let (t0, t1) = ((-h - p) / d, (h - p) / d);
    Some(if t0 < t1 { (t0, t1) } else { (t1, t0) })
}

fn slabs(p: Vec3, d: Vec3, h: Vec3) -> Option<(f64, f64)> {
    let (ax0, ax1) = slab(p.x, d.x, h.x)?;
    let (ay0, ay1) = slab(p.y, d.y, h.y)?;
    let (az0, az1) = slab(p.z, d.z, h.z)?;
    let (t0, t1) = (ax0.max(ay0).max(az0), ax1.min(ay1).min(az1));
    (t0 < t1).then_some((t0, t1))
}

impl Solid {
    pub fn inside(&self, p: Vec3) -> bool {
        match *self {
            Solid::Box { half } => p.x.abs() <= half.x && p.y.abs() <= half.y && p.z.abs() <= half.z,
            Solid::Sphere { r } => p.dot(p) <= r * r,
            Solid::Cylinder { r, half_h } => p.z.abs() <= half_h && p.x * p.x + p.y * p.y <= r * r,
            Solid::Ellipsoid { a } => {
                let q = p.div_elem(a);
                q.dot(q) <= 1.0
            }
        }
    }

    /// Distance from an inside point to the surface along unit vector `d`.
    pub fn dist_out(&self, p: Vec3, d: Vec3) -> f64 {
        match interval(self, p, d) {
            Some((_, t1)) => t1.max(0.0),
            None => 0.0,
        }
    }

    /// Distance from an outside point to the surface along `d` (∞ if missed).
    pub fn dist_in(&self, p: Vec3, d: Vec3) -> f64 {
        match interval(self, p, d) {
            Some((t0, t1)) if t1 > 0.0 => t0.max(0.0),
            _ => INF,
        }
    }

    pub fn volume(&self) -> f64 {
        use std::f64::consts::PI;
        match *self {
            Solid::Box { half } => 8.0 * half.x * half.y * half.z,
            Solid::Sphere { r } => 4.0 / 3.0 * PI * r * r * r,
            Solid::Cylinder { r, half_h } => PI * r * r * 2.0 * half_h,
            Solid::Ellipsoid { a } => 4.0 / 3.0 * PI * a.x * a.y * a.z,
        }
    }

    /// Half sizes of the bounding box.
    pub fn half_extent(&self) -> Vec3 {
        match *self {
            Solid::Box { half } => half,
            Solid::Sphere { r } => Vec3::new(r, r, r),
            Solid::Cylinder { r, half_h } => Vec3::new(r, r, half_h),
            Solid::Ellipsoid { a } => a,
        }
    }

    /// A point on the surface from two uniform numbers in (0, 1)
    /// (not uniformly distributed for every solid; used for overlap checks).
    pub fn surface_point(&self, u1: f64, u2: f64, u3: f64) -> Vec3 {
        let sphere = |u1: f64, u2: f64| {
            let c = 2.0 * u1 - 1.0;
            let s = (1.0 - c * c).sqrt();
            let phi = 2.0 * std::f64::consts::PI * u2;
            Vec3::new(s * phi.cos(), s * phi.sin(), c)
        };
        match *self {
            Solid::Sphere { r } => sphere(u1, u2) * r,
            Solid::Ellipsoid { a } => sphere(u1, u2).mul_elem(a),
            Solid::Box { half } => {
                let face = (u3 * 6.0) as usize % 6;
                let (a, b) = (2.0 * u1 - 1.0, 2.0 * u2 - 1.0);
                let p = match face / 2 {
                    0 => Vec3::new(1.0, a, b),
                    1 => Vec3::new(a, 1.0, b),
                    _ => Vec3::new(a, b, 1.0),
                };
                let p = if face.is_multiple_of(2) { p } else { -p };
                p.mul_elem(half)
            }
            Solid::Cylinder { r, half_h } => {
                let phi = 2.0 * std::f64::consts::PI * u1;
                if u3 < 0.5 {
                    Vec3::new(r * phi.cos(), r * phi.sin(), (2.0 * u2 - 1.0) * half_h)
                } else {
                    let rr = r * u2.sqrt();
                    let z = if u3 < 0.75 { half_h } else { -half_h };
                    Vec3::new(rr * phi.cos(), rr * phi.sin(), z)
                }
            }
        }
    }
}

/// One volume of the tree.
#[derive(Debug, Clone)]
pub struct Volume {
    pub name: String,
    pub solid: Solid,
    /// Centre in world coordinates.
    pub center: Vec3,
    pub material: usize,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
}

impl Volume {
    #[inline]
    pub fn local(&self, p: Vec3) -> Vec3 {
        p - self.center
    }
    #[inline]
    pub fn contains(&self, p: Vec3) -> bool {
        self.solid.inside(self.local(p))
    }
}

/// The volume tree. Index 0 is the world.
#[derive(Debug, Clone)]
pub struct Geometry {
    pub volumes: Vec<Volume>,
}

/// Small push across a boundary, relative to the world size.
pub const PUSH: f64 = 1e-9;

impl Geometry {
    pub fn world(&self) -> &Volume {
        &self.volumes[0]
    }

    /// Deepest volume containing `p`, or `None` outside the world.
    pub fn locate(&self, p: Vec3) -> Option<usize> {
        if !self.volumes[0].contains(p) {
            return None;
        }
        let mut cur = 0;
        'down: loop {
            for &c in &self.volumes[cur].children {
                if self.volumes[c].contains(p) {
                    cur = c;
                    continue 'down;
                }
            }
            return Some(cur);
        }
    }

    /// Distance to the next boundary of the region of volume `v` (its own
    /// surface or the surface of one of its daughters).
    pub fn distance(&self, v: usize, p: Vec3, d: Vec3) -> f64 {
        let vol = &self.volumes[v];
        let mut s = vol.solid.dist_out(vol.local(p), d);
        for &c in &vol.children {
            let cv = &self.volumes[c];
            s = s.min(cv.solid.dist_in(cv.local(p), d));
        }
        s
    }

    /// Volume of the region that belongs to `v` (its solid minus its daughters), cm³.
    pub fn region_volume(&self, v: usize) -> f64 {
        let vol = &self.volumes[v];
        vol.solid.volume()
            - vol
                .children
                .iter()
                .map(|&c| self.volumes[c].solid.volume())
                .sum::<f64>()
    }

    pub fn index(&self, name: &str) -> Option<usize> {
        if name == "world" {
            return Some(0);
        }
        self.volumes.iter().position(|v| v.name == name)
    }

    /// Check, by sampling points on every surface, that each volume lies inside
    /// its mother and does not overlap its sisters. Returns human-readable problems.
    pub fn check_overlaps(&self, samples: usize) -> Vec<(usize, String)> {
        let mut problems = Vec::new();
        let mut state = 0x5eed_u64;
        let mut u = || ((crate::rng::splitmix64(&mut state) >> 11) as f64 + 0.5) / (1u64 << 53) as f64;
        for (i, v) in self.volumes.iter().enumerate().skip(1) {
            let mother = &self.volumes[v.parent.unwrap_or(0)];
            let scale = v.solid.half_extent().max_elem();
            let tol = 1e-9 * scale.max(1.0);
            let mut outside = None;
            let mut overlap: Option<&str> = None;
            for _ in 0..samples {
                let p = v.center + v.solid.surface_point(u(), u(), u());
                // shrink the test point very slightly towards the centre
                let q = v.center + (p - v.center) * (1.0 - tol / scale.max(tol));
                if outside.is_none() && !mother.contains(q) {
                    outside = Some(q);
                }
                if overlap.is_none() {
                    for &s in &mother.children {
                        if s != i {
                            let sv = &self.volumes[s];
                            let inner = sv.center + (q - sv.center) * (1.0 + 1e-12);
                            if sv.solid.inside(sv.local(q)) && sv.solid.inside(sv.local(inner)) {
                                overlap = Some(&sv.name);
                            }
                        }
                    }
                }
            }
            if let Some(q) = outside {
                problems.push((
                    i,
                    format!(
                        "volume `{}` sticks out of `{}` (e.g. the point [{:.4}, {:.4}, {:.4}] cm)",
                        v.name, mother.name, q.x, q.y, q.z
                    ),
                ));
            }
            if let Some(o) = overlap {
                problems.push((i, format!("volumes `{}` and `{o}` overlap", v.name)));
            }
        }
        problems
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn distances() {
        let z = Vec3::new(0.0, 0.0, 1.0);
        let x = Vec3::new(1.0, 0.0, 0.0);
        let s = Solid::Sphere { r: 2.0 };
        assert!(close(s.dist_out(Vec3::ZERO, z), 2.0));
        assert!(close(s.dist_in(Vec3::new(0.0, 0.0, -5.0), z), 3.0));
        assert_eq!(s.dist_in(Vec3::new(0.0, 0.0, -5.0), -z), INF);
        let b = Solid::Box {
            half: Vec3::new(1.0, 2.0, 3.0),
        };
        assert!(close(b.dist_out(Vec3::ZERO, x), 1.0));
        assert!(close(b.dist_in(Vec3::new(-4.0, 0.0, 0.0), x), 3.0));
        let c = Solid::Cylinder { r: 1.0, half_h: 5.0 };
        assert!(close(c.dist_out(Vec3::ZERO, x), 1.0));
        assert!(close(c.dist_out(Vec3::ZERO, z), 5.0));
        assert!(close(c.dist_in(Vec3::new(0.0, 0.0, -7.0), z), 2.0));
        assert!(close(c.dist_in(Vec3::new(-3.0, 0.0, 0.0), x), 2.0));
        let e = Solid::Ellipsoid {
            a: Vec3::new(1.0, 2.0, 3.0),
        };
        assert!(close(e.dist_out(Vec3::ZERO, z), 3.0));
        assert!(close(e.dist_in(Vec3::new(-4.0, 0.0, 0.0), x), 3.0));
    }

    fn two_level() -> Geometry {
        Geometry {
            volumes: vec![
                Volume {
                    name: "world".into(),
                    solid: Solid::Box {
                        half: Vec3::new(10.0, 10.0, 10.0),
                    },
                    center: Vec3::ZERO,
                    material: 0,
                    parent: None,
                    children: vec![1],
                },
                Volume {
                    name: "a".into(),
                    solid: Solid::Sphere { r: 5.0 },
                    center: Vec3::ZERO,
                    material: 1,
                    parent: Some(0),
                    children: vec![2],
                },
                Volume {
                    name: "b".into(),
                    solid: Solid::Sphere { r: 1.0 },
                    center: Vec3::new(2.0, 0.0, 0.0),
                    material: 1,
                    parent: Some(1),
                    children: vec![],
                },
            ],
        }
    }

    #[test]
    fn locate_and_distance() {
        let g = two_level();
        assert_eq!(g.locate(Vec3::new(2.0, 0.0, 0.0)), Some(2));
        assert_eq!(g.locate(Vec3::new(0.0, 0.0, 0.0)), Some(1));
        assert_eq!(g.locate(Vec3::new(8.0, 0.0, 0.0)), Some(0));
        assert_eq!(g.locate(Vec3::new(11.0, 0.0, 0.0)), None);
        // from the centre along +x: daughter b starts at x = 1
        assert!((g.distance(1, Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0)) - 1.0).abs() < 1e-12);
        let expect = 4.0 / 3.0 * std::f64::consts::PI * (125.0 - 1.0);
        assert!((g.region_volume(1) - expect).abs() < 1e-9);
    }

    #[test]
    fn overlaps() {
        let mut g = two_level();
        assert!(g.check_overlaps(2000).is_empty());
        g.volumes[2].center = Vec3::new(4.5, 0.0, 0.0); // sticks out of `a`
        let p = g.check_overlaps(2000);
        assert!(p.iter().any(|(_, m)| m.contains("sticks out")), "{p:?}");
    }
}
