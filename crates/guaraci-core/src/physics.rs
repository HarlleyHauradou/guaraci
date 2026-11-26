//! Photon physics: material cross-section tables and interaction sampling.
//!
//! Units: energy MeV, length cm, density g/cm³, cross sections barn.

use crate::rng::Rng;
use crate::vec3::Vec3;
use guaraci_data::{ElementPhoton, Table};

pub const AVOGADRO: f64 = 6.022_140_76e23;
pub const BARN: f64 = 1e-24;
/// Electron rest energy, MeV (CODATA 2018).
pub const ELECTRON_MASS: f64 = 0.510_998_950_00;
/// h c in MeV·cm (CODATA 2018).
pub const HC: f64 = 1.239_841_984e-10;

/// Log-uniform energy grid.
#[derive(Debug, Clone)]
pub struct Grid {
    pub ln_min: f64,
    pub inv_step: f64,
    pub energies: Vec<f64>,
}

impl Grid {
    pub fn new(emin: f64, emax: f64, points_per_decade: usize) -> Grid {
        let decades = (emax / emin).log10();
        let n = ((decades * points_per_decade as f64).ceil() as usize).max(2) + 1;
        let (lmin, lmax) = (emin.ln(), emax.ln());
        let step = (lmax - lmin) / (n - 1) as f64;
        let energies = (0..n).map(|i| (lmin + step * i as f64).exp()).collect();
        Grid {
            ln_min: lmin,
            inv_step: 1.0 / step,
            energies,
        }
    }

    /// Index and fraction for interpolation (clamped to the grid).
    #[inline]
    pub fn locate(&self, e: f64) -> (usize, f64) {
        let t = (e.ln() - self.ln_min) * self.inv_step;
        let n = self.energies.len();
        if t <= 0.0 {
            return (0, 0.0);
        }
        let i = t as usize;
        if i >= n - 1 {
            return (n - 2, 1.0);
        }
        (i, t - i as f64)
    }
}

#[inline]
fn lerp(v: &[f64], (i, f): (usize, f64)) -> f64 {
    v[i] + (v[i + 1] - v[i]) * f
}

/// Shared per-element sampling data.
#[derive(Debug, Clone)]
pub struct ElementSampling {
    pub z: u32,
    /// Incoherent scattering function S(x), x in 1/cm.
    pub scattering_function: Table,
    /// x² and cumulative ∫F² d(x²) for coherent scattering.
    pub ff_x2: Vec<f64>,
    pub ff_cum: Vec<f64>,
}

impl ElementSampling {
    pub fn new(e: &ElementPhoton) -> Self {
        let x2: Vec<f64> = e.form_factor.x.iter().map(|x| x * x).collect();
        let f2: Vec<f64> = e.form_factor.y.iter().map(|f| f * f).collect();
        let mut cum = vec![0.0; x2.len()];
        for i in 1..x2.len() {
            cum[i] = cum[i - 1] + 0.5 * (f2[i] + f2[i - 1]) * (x2[i] - x2[i - 1]);
        }
        ElementSampling {
            z: e.z,
            scattering_function: e.scattering_function.clone(),
            ff_x2: x2,
            ff_cum: cum,
        }
    }

    fn cum_at(&self, x2: f64) -> f64 {
        let (xs, ys) = (&self.ff_x2, &self.ff_cum);
        let n = xs.len();
        if x2 >= xs[n - 1] {
            return ys[n - 1];
        }
        let i = xs.partition_point(|&v| v <= x2).max(1);
        let (a, b) = (xs[i - 1], xs[i]);
        if b > a {
            ys[i - 1] + (ys[i] - ys[i - 1]) * (x2 - a) / (b - a)
        } else {
            ys[i]
        }
    }
}

/// Element inside a material.
#[derive(Debug, Clone)]
pub struct Component {
    /// Index into the problem's element list.
    pub element: usize,
    pub z: u32,
    pub mass_fraction: f64,
    /// atoms / cm³
    pub atom_density: f64,
}

/// Which processes are simulated.
#[derive(Debug, Clone, Copy)]
pub struct Processes {
    pub photoelectric: bool,
    pub compton: bool,
    /// Use the incoherent scattering function S(x, Z) in Compton sampling.
    pub compton_sf: bool,
    pub rayleigh: bool,
    pub pair: bool,
}

/// Macroscopic cross sections of one material on the energy grid (1/cm).
#[derive(Debug, Clone)]
pub struct Material {
    pub name: String,
    pub density: f64,
    pub components: Vec<Component>,
    pub mu_photo: Vec<f64>,
    pub mu_incoh: Vec<f64>,
    pub mu_coh: Vec<f64>,
    pub mu_pair: Vec<f64>,
    pub mu_total: Vec<f64>,
    /// Per component: incoherent and coherent macroscopic cross sections,
    /// used to choose the target atom.
    pub comp_incoh: Vec<Vec<f64>>,
    pub comp_coh: Vec<Vec<f64>>,
}

impl Material {
    /// Build tables. `elements[c.element]` must be the photon data of component `c`.
    pub fn new(
        name: String,
        density: f64,
        components: Vec<Component>,
        elements: &[&ElementPhoton],
        grid: &Grid,
        p: Processes,
    ) -> Material {
        let n = grid.energies.len();
        let mut m = Material {
            name,
            density,
            mu_photo: vec![0.0; n],
            mu_incoh: vec![0.0; n],
            mu_coh: vec![0.0; n],
            mu_pair: vec![0.0; n],
            mu_total: vec![0.0; n],
            comp_incoh: vec![vec![0.0; n]; components.len()],
            comp_coh: vec![vec![0.0; n]; components.len()],
            components,
        };
        for (k, c) in m.components.iter().enumerate() {
            let el = elements[c.element];
            for (i, &e) in grid.energies.iter().enumerate() {
                let nb = c.atom_density * BARN;
                if p.photoelectric {
                    m.mu_photo[i] += nb * el.photoelectric.eval(e);
                }
                if p.compton {
                    let s = nb * el.incoherent.eval(e);
                    m.mu_incoh[i] += s;
                    m.comp_incoh[k][i] = s;
                }
                if p.rayleigh {
                    let s = nb * el.coherent.eval(e);
                    m.mu_coh[i] += s;
                    m.comp_coh[k][i] = s;
                }
                if p.pair && e > 2.0 * ELECTRON_MASS {
                    m.mu_pair[i] += nb * el.pair.eval(e);
                }
            }
        }
        for i in 0..n {
            m.mu_total[i] = m.mu_photo[i] + m.mu_incoh[i] + m.mu_coh[i] + m.mu_pair[i];
        }
        m
    }

    #[inline]
    pub fn mu(&self, at: (usize, f64)) -> f64 {
        lerp(&self.mu_total, at)
    }

    /// Mass attenuation coefficient μ/ρ (cm²/g) at energy `e`.
    pub fn mass_attenuation(&self, grid: &Grid, e: f64) -> f64 {
        self.mu(grid.locate(e)) / self.density
    }

    fn pick_component(&self, table: &[Vec<f64>], at: (usize, f64), total: f64, rng: &mut impl Rng) -> usize {
        let target = rng.uniform() * total;
        let mut acc = 0.0;
        for (k, t) in table.iter().enumerate() {
            acc += lerp(t, at);
            if target < acc {
                return k;
            }
        }
        table.len() - 1
    }
}

/// Outcome of one interaction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Interaction {
    /// Photon absorbed; all energy deposited.
    Photoelectric,
    /// Photon continues with new energy and direction; `deposit` goes to the electron.
    Compton {
        energy: f64,
        dir: Vec3,
        deposit: f64,
    },
    Rayleigh {
        dir: Vec3,
    },
    /// Photon absorbed; `deposit` = kinetic energy of the pair; two 511 keV photons follow.
    Pair {
        deposit: f64,
    },
}

/// Sample the interaction of a photon of energy `e` moving along `dir`.
pub fn interact(
    m: &Material,
    elements: &[ElementSampling],
    grid: &Grid,
    e: f64,
    dir: Vec3,
    use_sf: bool,
    rng: &mut impl Rng,
) -> Interaction {
    let at = grid.locate(e);
    let photo = lerp(&m.mu_photo, at);
    let incoh = lerp(&m.mu_incoh, at);
    let coh = lerp(&m.mu_coh, at);
    let pair = lerp(&m.mu_pair, at);
    let r = rng.uniform() * (photo + incoh + coh + pair);
    if r < photo {
        Interaction::Photoelectric
    } else if r < photo + incoh {
        let k = m.pick_component(&m.comp_incoh, at, incoh, rng);
        let el = &elements[m.components[k].element];
        let (cos, e1) = sample_compton(e, el, use_sf, rng);
        let phi = 2.0 * std::f64::consts::PI * rng.uniform();
        Interaction::Compton {
            energy: e1,
            dir: dir.rotate(cos, phi),
            deposit: e - e1,
        }
    } else if r < photo + incoh + coh {
        let k = m.pick_component(&m.comp_coh, at, coh, rng);
        let cos = sample_rayleigh(e, &elements[m.components[k].element], rng);
        let phi = 2.0 * std::f64::consts::PI * rng.uniform();
        Interaction::Rayleigh {
            dir: dir.rotate(cos, phi),
        }
    } else {
        Interaction::Pair {
            deposit: e - 2.0 * ELECTRON_MASS,
        }
    }
}

/// Compton scattering: Klein-Nishina (method of Geant4's G4KleinNishinaCompton),
/// optionally weighted by S(x, Z)/Z. Returns (cos θ, scattered energy).
pub fn sample_compton(e: f64, el: &ElementSampling, use_sf: bool, rng: &mut impl Rng) -> (f64, f64) {
    let k = e / ELECTRON_MASS;
    let eps0 = 1.0 / (1.0 + 2.0 * k);
    let eps0sq = eps0 * eps0;
    let alpha1 = -eps0.ln();
    let alpha2 = alpha1 + 0.5 * (1.0 - eps0sq);
    let z = el.z as f64;
    loop {
        let (eps, epssq) = if alpha1 > alpha2 * rng.uniform() {
            let eps = (-alpha1 * rng.uniform()).exp();
            (eps, eps * eps)
        } else {
            let epssq = eps0sq + (1.0 - eps0sq) * rng.uniform();
            (epssq.sqrt(), epssq)
        };
        let onecost = (1.0 - eps) / (eps * k);
        let sint2 = onecost * (2.0 - onecost);
        let greject = 1.0 - eps * sint2 / (1.0 + epssq);
        if greject < rng.uniform() {
            continue;
        }
        let cos = 1.0 - onecost;
        if use_sf {
            let x = e / HC * (0.5 * onecost).max(0.0).sqrt();
            if rng.uniform() * z > el.scattering_function.eval(x) {
                continue;
            }
        }
        return (cos, eps * e);
    }
}

/// Rayleigh scattering: x² sampled from F²(x) by inversion, then Thomson
/// rejection (1 + cos²θ)/2. Returns cos θ.
pub fn sample_rayleigh(e: f64, el: &ElementSampling, rng: &mut impl Rng) -> f64 {
    let x2max = (e / HC) * (e / HC);
    let cmax = el.cum_at(x2max);
    loop {
        let target = rng.uniform() * cmax;
        let i = el
            .ff_cum
            .partition_point(|&c| c <= target)
            .clamp(1, el.ff_cum.len() - 1);
        let (c0, c1) = (el.ff_cum[i - 1], el.ff_cum[i]);
        let (a, b) = (el.ff_x2[i - 1], el.ff_x2[i]);
        let x2 = if c1 > c0 {
            a + (b - a) * (target - c0) / (c1 - c0)
        } else {
            a
        };
        let cos = 1.0 - 2.0 * (x2 / x2max).min(1.0);
        if 2.0 * rng.uniform() <= 1.0 + cos * cos {
            return cos;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::{Philox, StreamRng};

    fn dummy_element(z: u32) -> ElementSampling {
        // S(x) = Z everywhere (free electrons), F = 0 except near 0
        ElementSampling {
            z,
            scattering_function: Table {
                x: vec![0.0, 1e12],
                y: vec![z as f64, z as f64],
            },
            ff_x2: vec![0.0, 1.0],
            ff_cum: vec![0.0, 1.0],
        }
    }

    /// Klein-Nishina dσ/dcosθ (unnormalised).
    fn kn(k: f64, cos: f64) -> f64 {
        let p = 1.0 / (1.0 + k * (1.0 - cos));
        p * p * (p + 1.0 / p - (1.0 - cos * cos))
    }

    #[test]
    fn compton_follows_klein_nishina() {
        let el = dummy_element(1);
        let mut rng = Philox::stream(1, 0);
        for e in [0.03, 0.1405, 0.511, 2.0] {
            let k = e / ELECTRON_MASS;
            let bins = 20;
            let n = 200_000;
            let mut h = vec![0usize; bins];
            for _ in 0..n {
                let (cos, e1) = sample_compton(e, &el, true, &mut rng);
                // Compton relation
                let expect = e / (1.0 + k * (1.0 - cos));
                assert!((e1 - expect).abs() < 1e-9 * e);
                let b = (((cos + 1.0) / 2.0 * bins as f64) as usize).min(bins - 1);
                h[b] += 1;
            }
            // expected counts by numerical integration
            let mut exp = vec![0.0; bins];
            let sub = 200;
            for (b, ex) in exp.iter_mut().enumerate() {
                for s in 0..sub {
                    let c = -1.0 + 2.0 * (b as f64 + (s as f64 + 0.5) / sub as f64) / bins as f64;
                    *ex += kn(k, c);
                }
            }
            let tot: f64 = exp.iter().sum();
            let chi2: f64 = h
                .iter()
                .zip(&exp)
                .map(|(&o, &x)| {
                    let x = x / tot * n as f64;
                    (o as f64 - x).powi(2) / x
                })
                .sum();
            // 19 degrees of freedom; 99.9% quantile ≈ 43.8
            assert!(chi2 < 43.8, "E = {e}: chi2 = {chi2}");
        }
    }

    #[test]
    fn grid_interpolation() {
        let g = Grid::new(1e-3, 10.0, 100);
        let (i, f) = g.locate(g.energies[37]);
        assert!(i == 37 && f.abs() < 1e-9 || i == 36 && (f - 1.0).abs() < 1e-9);
        assert_eq!(g.locate(1e-6), (0, 0.0));
    }
}
