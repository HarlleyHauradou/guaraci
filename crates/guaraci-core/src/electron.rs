//! Electron and positron physics: stopping powers, ranges, multiple
//! scattering, β spectra and photo-electron angular distribution.
//!
//! Mixed (class II) condensed history: knock-on electrons (δ-rays) above the
//! electron cut-off and bremsstrahlung photons above the photon cut-off are
//! produced one by one (Møller/Bhabha cross sections and the bremsstrahlung
//! spectra of the library); softer losses are continuous, with the restricted
//! stopping power. Each step ends with a deflection whose mean cosine matches
//! the elastic transport mean free path.
//!
//! Units: energy MeV, length cm, density g/cm³.

use crate::physics::{Grid, AVOGADRO, BARN, ELECTRON_MASS};
use crate::rng::Rng;
use crate::vec3::Vec3;
use guaraci_data::electron::{BremSpectrum, ElementElectron};
use guaraci_data::Table;

/// Mean excitation energies of the elements (eV), Z = 1..100, gas-phase values
/// of ICRU Report 37 (1984), as used by NIST ESTAR.
pub const ICRU37_I: [f64; 100] = [
    19.2, 41.8, 40.0, 63.7, 76.0, 81.0, 82.0, 95.0, 115.0, 137.0, // H..Ne
    149.0, 156.0, 166.0, 173.0, 173.0, 180.0, 174.0, 188.0, 190.0, 191.0, // Na..Ca
    216.0, 233.0, 245.0, 257.0, 272.0, 286.0, 297.0, 311.0, 322.0, 330.0, // Sc..Zn
    334.0, 350.0, 347.0, 348.0, 343.0, 352.0, 363.0, 366.0, 379.0, 393.0, // Ga..Zr
    417.0, 424.0, 428.0, 441.0, 449.0, 470.0, 470.0, 469.0, 488.0, 488.0, // Nb..Sn
    487.0, 485.0, 491.0, 482.0, 488.0, 491.0, 501.0, 523.0, 535.0, 546.0, // Sb..Nd
    560.0, 574.0, 580.0, 591.0, 614.0, 628.0, 650.0, 658.0, 674.0, 684.0, // Pm..Yb
    694.0, 705.0, 718.0, 727.0, 736.0, 746.0, 757.0, 790.0, 790.0, 800.0, // Lu..Hg
    810.0, 823.0, 823.0, 830.0, 825.0, 794.0, 827.0, 826.0, 841.0, 847.0, // Tl..Th
    878.0, 890.0, 902.0, 921.0, 934.0, 939.0, 952.0, 966.0, 980.0, 994.0, // Pa..Fm
];

/// Mean excitation energy (eV) of a mixture by the Bragg rule of ICRU Report 37:
/// in condensed materials H, C, N, O, F and Cl use 19.2, 81, 82, 106, 112 and
/// 180 eV, and the other elements 1.13 × their gas value.
pub fn bragg_mean_excitation(components: &[(u32, f64, f64)], condensed: bool) -> f64 {
    // components: (Z, mass fraction, atomic mass)
    let (mut num, mut den) = (0.0, 0.0);
    for &(z, w, a) in components {
        let i = if condensed {
            match z {
                1 => 19.2,
                6 => 81.0,
                7 => 82.0,
                8 => 106.0,
                9 => 112.0,
                17 => 180.0,
                _ => 1.13 * ICRU37_I[(z - 1) as usize],
            }
        } else {
            ICRU37_I[(z - 1) as usize]
        };
        let ne = w * z as f64 / a;
        num += ne * i.ln();
        den += ne;
    }
    (num / den).exp()
}

/// Sternheimer–Peierls (1971) general parametrisation of the density effect.
#[derive(Debug, Clone, Copy)]
pub struct DensityEffect {
    pub c: f64,
    pub x0: f64,
    pub x1: f64,
    pub a: f64,
    pub m: f64,
}

impl DensityEffect {
    /// `i_ev`: mean excitation energy; `z_over_a`: ⟨Z/A⟩ (mol/g); density g/cm³.
    pub fn new(i_ev: f64, z_over_a: f64, density: f64, condensed: bool) -> DensityEffect {
        let plasma = 28.816 * (density * z_over_a).sqrt(); // eV
        let c = 2.0 * (i_ev / plasma).ln() + 1.0;
        let (x0, x1) = if condensed {
            if i_ev < 100.0 {
                (if c <= 3.681 { 0.2 } else { 0.326 * c - 1.0 }, 2.0)
            } else {
                (if c <= 5.215 { 0.2 } else { 0.326 * c - 1.5 }, 3.0)
            }
        } else if c < 10.0 {
            (1.6, 4.0)
        } else if c < 10.5 {
            (1.7, 4.0)
        } else if c < 11.0 {
            (1.8, 4.0)
        } else if c < 11.5 {
            (1.9, 4.0)
        } else if c < 12.25 {
            (2.0, 4.0)
        } else if c < 13.804 {
            (2.0, 5.0)
        } else {
            (0.326 * c - 2.5, 5.0)
        };
        let m = 3.0;
        let a = (c - 4.606 * x0) / (x1 - x0).powf(m);
        DensityEffect { c, x0, x1, a, m }
    }

    /// δ for a particle with kinetic energy `t` (MeV), electron mass.
    pub fn delta(&self, t: f64) -> f64 {
        let tau = t / ELECTRON_MASS;
        let x = (tau * (tau + 2.0)).sqrt().log10();
        if x < self.x0 {
            0.0
        } else if x < self.x1 {
            4.6052 * x + self.a * (self.x1 - x).powf(self.m) - self.c
        } else {
            4.6052 * x - self.c
        }
    }
}

/// Collision stopping power (MeV/cm) from the Bethe formula with the
/// Rohrlich–Carlson terms for electrons (`positron = false`) or positrons.
pub fn bethe_collision(
    t: f64,
    i_ev: f64,
    electron_density_per_g: f64,
    density: f64,
    de: &DensityEffect,
    positron: bool,
) -> f64 {
    // 2π r_e² m c² N_A = 0.153537 MeV cm²/mol
    const K: f64 = 0.153_537;
    let tau = t / ELECTRON_MASS;
    let gamma = tau + 1.0;
    let beta2 = 1.0 - 1.0 / (gamma * gamma);
    let i = i_ev * 1e-6 / ELECTRON_MASS;
    let f = if positron {
        let y = 1.0 / (tau + 2.0);
        2.0 * std::f64::consts::LN_2 - beta2 / 12.0 * (23.0 + 14.0 * y + 10.0 * y * y + 4.0 * y * y * y)
    } else {
        1.0 - beta2 + (tau * tau / 8.0 - (2.0 * tau + 1.0) * std::f64::consts::LN_2) / (gamma * gamma)
    };
    let bracket = (tau * tau * (tau + 2.0) / (2.0 * i * i)).ln() + f - de.delta(t);
    (K * electron_density_per_g / beta2 * bracket * density).max(0.0)
}

/// 2π r_e² m c² N_A, MeV cm²/mol.
const K_COLLISION: f64 = 0.153_537;

/// Energy-transfer dependent part of the restricted collision stopping power
/// (Møller for electrons, Bhabha for positrons), with the transfer `d` in units of m c².
fn restricted_term(tau: f64, d: f64, positron: bool) -> f64 {
    let gamma = tau + 1.0;
    let beta2 = 1.0 - 1.0 / (gamma * gamma);
    if positron {
        let y = 1.0 / (tau + 2.0);
        let (d2, d3, d4) = (0.5 * d * d, d * d * d / 3.0, 0.25 * d * d * d * d);
        (tau * d).ln() - beta2 / tau * (tau + 2.0 * d - y * (3.0 * d2 + y * (d - d3 + y * (d2 - tau * d3 + d4))))
    } else {
        -1.0 - beta2
            + ((tau - d) * d).ln()
            + tau / (tau - d)
            + (0.5 * d * d + (2.0 * tau + 1.0) * (1.0 - d / tau).ln()) / (gamma * gamma)
    }
}

/// Largest energy a knock-on electron can take (MeV): half the kinetic energy
/// for electrons (the faster one is the primary), all of it for positrons.
pub fn max_delta_energy(t: f64, positron: bool) -> f64 {
    if positron {
        t
    } else {
        0.5 * t
    }
}

/// Collision stopping power restricted to energy transfers below `cut` (MeV/cm):
/// the unrestricted Bethe value minus the share of the hard collisions.
pub fn restricted_collision(
    full: f64,
    t: f64,
    cut: f64,
    electron_density_per_g: f64,
    density: f64,
    positron: bool,
) -> f64 {
    let wmax = max_delta_energy(t, positron);
    if cut >= wmax {
        return full;
    }
    let tau = t / ELECTRON_MASS;
    let gamma = tau + 1.0;
    let beta2 = 1.0 - 1.0 / (gamma * gamma);
    let hard =
        restricted_term(tau, wmax / ELECTRON_MASS, positron) - restricted_term(tau, cut / ELECTRON_MASS, positron);
    (full - K_COLLISION * electron_density_per_g * density / beta2 * hard).max(0.0)
}

/// Macroscopic cross section (1/cm) for knock-on electrons above `cut`
/// (Møller for electrons, Bhabha for positrons; atomic binding neglected).
pub fn delta_cross_section(t: f64, cut: f64, electron_density_per_g: f64, density: f64, positron: bool) -> f64 {
    let wmax = max_delta_energy(t, positron);
    if cut >= wmax {
        return 0.0;
    }
    let (xmin, xmax) = (cut / t, wmax / t);
    let gamma = t / ELECTRON_MASS + 1.0;
    let gamma2 = gamma * gamma;
    let beta2 = 1.0 - 1.0 / gamma2;
    let bracket = if positron {
        let y = 1.0 / (1.0 + gamma);
        let y2 = y * y;
        let y12 = 1.0 - 2.0 * y;
        let b1 = 2.0 - y2;
        let b2 = y12 * (3.0 + y2);
        let y122 = y12 * y12;
        let b4 = y122 * y12;
        let b3 = b4 + y122;
        (xmax - xmin)
            * (1.0 / (beta2 * xmin * xmax) + b2 - 0.5 * b3 * (xmin + xmax)
                + b4 * (xmin * xmin + xmin * xmax + xmax * xmax) / 3.0)
            - b1 * (xmax / xmin).ln()
    } else {
        let gg = (2.0 * gamma - 1.0) / gamma2;
        ((xmax - xmin) * (1.0 - gg + 1.0 / (xmin * xmax) + 1.0 / ((1.0 - xmin) * (1.0 - xmax)))
            - gg * (xmax * (1.0 - xmin) / (xmin * (1.0 - xmax))).ln())
            / beta2
    };
    (K_COLLISION * electron_density_per_g * density / t * bracket).max(0.0)
}

/// Kinetic energy (MeV) of a knock-on electron above `cut`, sampled from the
/// Møller or Bhabha cross section.
pub fn sample_delta_energy(t: f64, cut: f64, positron: bool, rng: &mut impl Rng) -> f64 {
    let (xmin, xmax) = (cut / t, max_delta_energy(t, positron) / t);
    let gamma = t / ELECTRON_MASS + 1.0;
    let gamma2 = gamma * gamma;
    let beta2 = 1.0 - 1.0 / gamma2;
    // 1/x² is sampled exactly; the rest of the cross section by rejection
    let draw = |rng: &mut dyn FnMut() -> f64| {
        let u = rng();
        xmin * xmax / (xmin * (1.0 - u) + xmax * u)
    };
    let mut uni = || rng.uniform();
    if positron {
        let y = 1.0 / (1.0 + gamma);
        let y2 = y * y;
        let y12 = 1.0 - 2.0 * y;
        let b1 = 2.0 - y2;
        let b2 = y12 * (3.0 + y2);
        let y122 = y12 * y12;
        let b4 = y122 * y12;
        let b3 = b4 + y122;
        let yy = xmax * xmax;
        let grej = 1.0 + (yy * yy * b4 - xmin * xmin * xmin * b3 + yy * b2 - xmin * b1) * beta2;
        loop {
            let x = draw(&mut uni);
            let x2 = x * x;
            let z = 1.0 + (x2 * x2 * b4 - x * x2 * b3 + x2 * b2 - x * b1) * beta2;
            if grej * uni() <= z {
                return x * t;
            }
        }
    } else {
        let gg = (2.0 * gamma - 1.0) / gamma2;
        let y = 1.0 - xmax;
        let grej = 1.0 - gg * xmax + xmax * xmax * (1.0 - gg + (1.0 - gg * y) / (y * y));
        loop {
            let x = draw(&mut uni);
            let y = 1.0 - x;
            let z = 1.0 - gg * x + x * x * (1.0 - gg + (1.0 - gg * y) / (y * y));
            if grej * uni() <= z {
                return x * t;
            }
        }
    }
}

/// Directions after a knock-on collision: (primary, δ-ray), from energy and
/// momentum conservation with the target electron at rest.
pub fn delta_directions(t: f64, w: f64, dir: Vec3, rng: &mut impl Rng) -> (Vec3, Vec3) {
    let m = ELECTRON_MASS;
    let p0 = (t * (t + 2.0 * m)).sqrt();
    let pd = (w * (w + 2.0 * m)).sqrt();
    let cos_d = (w * (t + 2.0 * m) / (pd * p0)).min(1.0);
    let phi = 2.0 * std::f64::consts::PI * rng.uniform();
    let dd = dir.rotate(cos_d, phi);
    let rest = dir * p0 - dd * pd;
    let n = rest.norm();
    (if n > 0.0 { rest * (1.0 / n) } else { dir }, dd)
}

/// One bremsstrahlung spectrum prepared for sampling (reduced photon energy κ = k/E).
#[derive(Debug, Clone)]
struct BremTable {
    ln_e: f64,
    kappa: Vec<f64>,
    density: Vec<f64>,
    /// ∫₀^κ p, and ∫₀^κ κ p, at the points.
    cdf: Vec<f64>,
    moment: Vec<f64>,
}

impl BremTable {
    fn new(s: &BremSpectrum) -> BremTable {
        let n = s.kappa.len();
        let (mut cdf, mut moment) = (vec![0.0; n], vec![0.0; n]);
        for i in 1..n {
            let (a, b, fa, fb) = (s.kappa[i - 1], s.kappa[i], s.density[i - 1], s.density[i]);
            cdf[i] = cdf[i - 1] + 0.5 * (fa + fb) * (b - a);
            moment[i] = moment[i - 1] + (b - a) / 6.0 * (fa * (2.0 * a + b) + fb * (a + 2.0 * b));
        }
        let norm = cdf[n - 1].max(1e-300);
        BremTable {
            ln_e: s.energy.ln(),
            kappa: s.kappa.clone(),
            density: s.density.iter().map(|v| v / norm).collect(),
            cdf: cdf.iter().map(|v| v / norm).collect(),
            moment: moment.iter().map(|v| v / norm).collect(),
        }
    }

    /// (∫₀^x p, ∫₀^x κ p)
    fn integrals(&self, x: f64) -> (f64, f64) {
        let n = self.kappa.len();
        if x <= self.kappa[0] {
            return (0.0, 0.0);
        }
        if x >= self.kappa[n - 1] {
            return (1.0, self.moment[n - 1]);
        }
        let i = self.kappa.partition_point(|&v| v <= x) - 1;
        let (a, fa) = (self.kappa[i], self.density[i]);
        let slope = (self.density[i + 1] - fa) / (self.kappa[i + 1] - a);
        let h = x - a;
        let fx = fa + slope * h;
        let c = self.cdf[i] + 0.5 * (fa + fx) * h;
        let m = self.moment[i] + h / 6.0 * (fa * (2.0 * a + x) + fx * (a + 2.0 * x));
        (c, m)
    }

    /// κ sampled above `x0`.
    fn sample_above(&self, x0: f64, rng: &mut impl Rng) -> f64 {
        let c0 = self.integrals(x0).0;
        let u = c0 + rng.uniform() * (1.0 - c0);
        let n = self.kappa.len();
        let i = (self.cdf.partition_point(|&v| v <= u)).clamp(1, n - 1) - 1;
        let (a, fa) = (self.kappa[i], self.density[i]);
        let h = self.kappa[i + 1] - a;
        let slope = (self.density[i + 1] - fa) / h;
        let r = u - self.cdf[i];
        // fa·x + slope·x²/2 = r
        let x = if slope.abs() * h < 1e-9 * fa.max(1e-300) {
            r / fa.max(1e-300)
        } else {
            let disc = (fa * fa + 2.0 * slope * r).max(0.0);
            2.0 * r / (fa + disc.sqrt()).max(1e-300)
        };
        (a + x.clamp(0.0, h)).max(x0)
    }
}

/// Bremsstrahlung spectra of one element.
#[derive(Debug, Clone)]
pub struct BremSampler {
    tables: Vec<BremTable>,
}

impl BremSampler {
    pub fn new(spectra: &[BremSpectrum]) -> BremSampler {
        BremSampler {
            tables: spectra
                .iter()
                .filter(|s| s.kappa.len() >= 2)
                .map(BremTable::new)
                .collect(),
        }
    }

    /// Bracketing tables and the weight of the upper one (interpolation in ln E).
    fn bracket(&self, e: f64) -> (usize, usize, f64) {
        let n = self.tables.len();
        let le = e.ln();
        if n == 1 || le <= self.tables[0].ln_e {
            return (0, 0, 0.0);
        }
        if le >= self.tables[n - 1].ln_e {
            return (n - 1, n - 1, 0.0);
        }
        let i = self.tables.partition_point(|t| t.ln_e <= le) - 1;
        let f = (le - self.tables[i].ln_e) / (self.tables[i + 1].ln_e - self.tables[i].ln_e);
        (i, i + 1, f)
    }

    /// (fraction of the photons above `cut`, fraction of the radiated energy below `cut`) at energy `e`.
    pub fn fractions(&self, e: f64, cut: f64) -> (f64, f64) {
        if self.tables.is_empty() {
            return (0.0, 1.0);
        }
        let x = cut / e;
        let (i, j, f) = self.bracket(e);
        let at = |t: &BremTable| {
            let (c, m) = t.integrals(x);
            (1.0 - c, m / t.moment[t.moment.len() - 1].max(1e-300))
        };
        let (hi, si) = at(&self.tables[i]);
        let (hj, sj) = at(&self.tables[j]);
        if x >= 1.0 {
            return (0.0, 1.0);
        }
        ((1.0 - f) * hi + f * hj, (1.0 - f) * si + f * sj)
    }

    /// Photon energy (MeV) above `cut` for an electron of energy `e`.
    pub fn sample(&self, e: f64, cut: f64, rng: &mut impl Rng) -> f64 {
        let (i, j, f) = self.bracket(e);
        let t = if rng.uniform() < f {
            &self.tables[j]
        } else {
            &self.tables[i]
        };
        (t.sample_above(cut / e, rng) * e).clamp(cut, e)
    }
}

/// Direction of a bremsstrahlung photon: a dipole distribution in the rest
/// frame of the electron, boosted to the laboratory (typical angle m c²/E).
pub fn brem_photon_direction(t: f64, dir: Vec3, rng: &mut impl Rng) -> Vec3 {
    let gamma = t / ELECTRON_MASS + 1.0;
    let beta = (1.0 - 1.0 / (gamma * gamma)).sqrt();
    let c0 = 2.0 * rng.uniform() - 1.0;
    let cos = ((c0 + beta) / (1.0 + beta * c0)).clamp(-1.0, 1.0);
    dir.rotate(cos, 2.0 * std::f64::consts::PI * rng.uniform())
}

/// Electron and positron tables of one material.
#[derive(Debug, Clone)]
pub struct ElectronMaterial {
    pub mean_excitation_ev: f64,
    /// Knock-on electrons above this energy are produced (MeV).
    pub delta_cut: f64,
    /// Bremsstrahlung photons above this energy are produced (MeV).
    pub brem_cut: f64,
    /// Restricted stopping power (soft collisions + soft radiative), MeV/cm.
    pub stopping_minus: Vec<f64>,
    pub stopping_plus: Vec<f64>,
    /// Unrestricted (total) stopping power, MeV/cm, for reports and checks.
    pub stopping_full_minus: Vec<f64>,
    pub stopping_full_plus: Vec<f64>,
    /// 1/λ₁, the inverse elastic transport mean free path, 1/cm.
    pub inv_transport: Vec<f64>,
    /// Range with the restricted stopping power, from the cut-off energy, cm.
    pub range_minus: Vec<f64>,
    pub range_plus: Vec<f64>,
    /// CSDA range (unrestricted stopping power), cm.
    pub csda_range_minus: Vec<f64>,
    /// Macroscopic cross sections of the hard events, 1/cm.
    pub sigma_delta_minus: Vec<f64>,
    pub sigma_delta_plus: Vec<f64>,
    pub sigma_brem: Vec<f64>,
    /// Hard bremsstrahlung cross section of each element, 1/cm.
    brem_partial: Vec<Vec<f64>>,
    brem: Vec<BremSampler>,
}

#[inline]
fn lerp(v: &[f64], (i, f): (usize, f64)) -> f64 {
    v[i] + (v[i + 1] - v[i]) * f
}

/// Energy for a given range by inverting a monotonic range table.
fn energy_at_range(grid: &Grid, range: &[f64], r: f64) -> f64 {
    if r <= range[0] {
        return grid.energies[0] * (r / range[0].max(1e-300)).max(0.0);
    }
    let n = range.len();
    if r >= range[n - 1] {
        return grid.energies[n - 1];
    }
    let i = range.partition_point(|&v| v <= r) - 1;
    let f = (r - range[i]) / (range[i + 1] - range[i]);
    // interpolate in ln E, consistent with the grid
    (grid.energies[i].ln() + f * (grid.energies[i + 1].ln() - grid.energies[i].ln())).exp()
}

impl ElectronMaterial {
    /// `components`: (Z, mass fraction, atomic mass, atom density per cm³, electron data).
    /// `delta_cut` and `brem_cut` (MeV) separate continuous from discrete losses.
    pub fn new(
        grid: &Grid,
        density: f64,
        components: &[(u32, f64, f64, f64, &ElementElectron)],
        mean_excitation_override: Option<f64>,
        delta_cut: f64,
        brem_cut: f64,
    ) -> ElectronMaterial {
        let condensed = density > 0.01;
        let bragg: Vec<(u32, f64, f64)> = components.iter().map(|c| (c.0, c.1, c.2)).collect();
        let i_ev = mean_excitation_override.unwrap_or_else(|| bragg_mean_excitation(&bragg, condensed));
        let z_over_a: f64 = components.iter().map(|c| c.1 * c.0 as f64 / c.2).sum();
        let de = DensityEffect::new(i_ev, z_over_a, density, condensed);
        let n = grid.energies.len();
        let mut m = ElectronMaterial {
            mean_excitation_ev: i_ev,
            delta_cut,
            brem_cut,
            stopping_minus: vec![0.0; n],
            stopping_plus: vec![0.0; n],
            stopping_full_minus: vec![0.0; n],
            stopping_full_plus: vec![0.0; n],
            inv_transport: vec![0.0; n],
            range_minus: vec![0.0; n],
            range_plus: vec![0.0; n],
            csda_range_minus: vec![0.0; n],
            sigma_delta_minus: vec![0.0; n],
            sigma_delta_plus: vec![0.0; n],
            sigma_brem: vec![0.0; n],
            brem_partial: vec![vec![0.0; n]; components.len()],
            brem: components.iter().map(|c| BremSampler::new(&c.4.brem_spectra)).collect(),
        };
        for (k, &e) in grid.energies.iter().enumerate() {
            let (mut s_rad, mut s_rad_soft, mut inv_tr, mut sig_brem) = (0.0, 0.0, 0.0, 0.0);
            for (ci, c) in components.iter().enumerate() {
                let el = c.4;
                let tab = |v: &Vec<f64>| {
                    Table {
                        x: el.energy.clone(),
                        y: v.clone(),
                    }
                    .eval(e)
                };
                let rad = c.3 * BARN * tab(&el.stopping_radiative);
                let (hard, soft) = m.brem[ci].fractions(e, brem_cut);
                let sb = c.3 * BARN * tab(&el.brem_cross_section) * hard;
                s_rad += rad;
                s_rad_soft += rad * soft;
                inv_tr += c.3 * BARN * tab(&el.elastic_transport);
                m.brem_partial[ci][k] = sb;
                sig_brem += sb;
            }
            m.inv_transport[k] = inv_tr.max(1e-30);
            m.sigma_brem[k] = sig_brem;
            let col_minus = bethe_collision(e, i_ev, z_over_a, density, &de, false);
            let col_plus = bethe_collision(e, i_ev, z_over_a, density, &de, true);
            m.stopping_full_minus[k] = col_minus + s_rad;
            m.stopping_full_plus[k] = col_plus + s_rad;
            m.stopping_minus[k] = restricted_collision(col_minus, e, delta_cut, z_over_a, density, false) + s_rad_soft;
            m.stopping_plus[k] = restricted_collision(col_plus, e, delta_cut, z_over_a, density, true) + s_rad_soft;
            m.sigma_delta_minus[k] = delta_cross_section(e, delta_cut, z_over_a, density, false);
            m.sigma_delta_plus[k] = delta_cross_section(e, delta_cut, z_over_a, density, true);
        }
        // ranges: R(E0) assumes S constant below the first point (tiny), then trapezoid in E
        for (range, s) in [
            (&mut m.range_minus, &m.stopping_minus),
            (&mut m.range_plus, &m.stopping_plus),
            (&mut m.csda_range_minus, &m.stopping_full_minus),
        ] {
            range[0] = grid.energies[0] / s[0].max(1e-30);
            for k in 1..n {
                let de = grid.energies[k] - grid.energies[k - 1];
                range[k] = range[k - 1] + 0.5 * de * (1.0 / s[k].max(1e-30) + 1.0 / s[k - 1].max(1e-30));
            }
        }
        m
    }

    #[inline]
    pub fn range(&self, grid: &Grid, e: f64, positron: bool) -> f64 {
        lerp(
            if positron { &self.range_plus } else { &self.range_minus },
            grid.locate(e),
        )
    }

    #[inline]
    pub fn energy_after(&self, grid: &Grid, range: f64, positron: bool) -> f64 {
        energy_at_range(grid, if positron { &self.range_plus } else { &self.range_minus }, range)
    }

    #[inline]
    pub fn transport_mfp(&self, grid: &Grid, e: f64) -> f64 {
        1.0 / lerp(&self.inv_transport, grid.locate(e))
    }

    /// Restricted stopping power (MeV/cm) at energy `e`.
    pub fn stopping(&self, grid: &Grid, e: f64, positron: bool) -> f64 {
        lerp(
            if positron {
                &self.stopping_plus
            } else {
                &self.stopping_minus
            },
            grid.locate(e),
        )
    }

    /// Unrestricted (total) stopping power (MeV/cm) at energy `e`.
    pub fn total_stopping(&self, grid: &Grid, e: f64, positron: bool) -> f64 {
        lerp(
            if positron {
                &self.stopping_full_plus
            } else {
                &self.stopping_full_minus
            },
            grid.locate(e),
        )
    }

    /// CSDA range (cm) of an electron of energy `e`.
    pub fn csda_range(&self, grid: &Grid, e: f64) -> f64 {
        lerp(&self.csda_range_minus, grid.locate(e))
    }

    /// Macroscopic cross sections (1/cm) of (knock-on, bremsstrahlung) hard events.
    #[inline]
    pub fn hard(&self, grid: &Grid, e: f64, positron: bool) -> (f64, f64) {
        let at = grid.locate(e);
        let d = lerp(
            if positron {
                &self.sigma_delta_plus
            } else {
                &self.sigma_delta_minus
            },
            at,
        );
        (d.max(0.0), lerp(&self.sigma_brem, at).max(0.0))
    }

    /// Bremsstrahlung photon energy (MeV) for an electron of energy `e`.
    pub fn sample_brem(&self, grid: &Grid, e: f64, rng: &mut impl Rng) -> f64 {
        let at = grid.locate(e);
        let total: f64 = self.brem_partial.iter().map(|v| lerp(v, at)).sum();
        let mut u = rng.uniform() * total;
        let mut pick = self.brem.len() - 1;
        for (i, v) in self.brem_partial.iter().enumerate() {
            u -= lerp(v, at);
            if u <= 0.0 {
                pick = i;
                break;
            }
        }
        self.brem[pick].sample(e, self.brem_cut, rng)
    }
}

/// Mass stopping power helper for reporting (MeV cm²/g).
pub fn per_density(s: f64, density: f64) -> f64 {
    s / density
}

/// Sample cos θ for a step: screened-Rutherford shape whose mean cosine is
/// exp(−s/λ₁) (Goudsmit–Saunderson first moment).
pub fn sample_multiple_scattering(step: f64, lambda1: f64, rng: &mut impl Rng) -> f64 {
    let t = 1.0 - (-step / lambda1).exp(); // target ⟨1 − cos θ⟩
    if t >= 0.999 {
        return 2.0 * rng.uniform() - 1.0;
    }
    if t <= 1e-12 {
        return 1.0;
    }
    // ⟨1−μ⟩(η) = 2η[(1+η) ln((1+η)/η) − 1], increasing in η: bisection in ln η
    let mean = |eta: f64| 2.0 * eta * ((1.0 + eta) * ((1.0 + eta) / eta).ln() - 1.0);
    let (mut lo, mut hi) = ((1e-14f64).ln(), (1e6f64).ln());
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if mean(mid.exp()) < t {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let eta = (0.5 * (lo + hi)).exp();
    let xi = rng.uniform();
    (1.0 - 2.0 * eta * xi / (1.0 - xi + eta)).clamp(-1.0, 1.0)
}

/// Photo-electron direction: Sauter–Gavrila K-shell distribution (method of
/// Geant4's G4SauterGavrilaAngularDistribution). Returns cos θ relative to the photon.
pub fn sample_sauter(t: f64, rng: &mut impl Rng) -> f64 {
    let tau = t / ELECTRON_MASS;
    if tau > 50.0 {
        return 1.0;
    }
    let gamma = tau + 1.0;
    let beta = (tau * (tau + 2.0)).sqrt() / gamma;
    let a = (1.0 - beta) / beta;
    let ap2 = a + 2.0;
    let b = 0.5 * beta * gamma * (gamma - 1.0) * (gamma - 2.0);
    let grej = 2.0 * (1.0 + a * b) / a;
    loop {
        let q = rng.uniform();
        let z = 2.0 * a * (2.0 * q + ap2 * q.sqrt()) / (ap2 * ap2 - 4.0 * q);
        let g = (2.0 - z) * (1.0 / (a + z) + b);
        if g >= rng.uniform() * grej {
            return (1.0 - z).clamp(-1.0, 1.0);
        }
    }
}

/// Fine-structure constant.
const ALPHA: f64 = 1.0 / 137.035_999_084;

/// Cumulative β spectrum of one branch, sampled by inversion.
#[derive(Debug, Clone)]
pub struct BetaSpectrum {
    pub energies: Vec<f64>,
    pub cdf: Vec<f64>,
    pub mean: f64,
}

impl BetaSpectrum {
    /// Allowed (or first-forbidden unique, `forbiddenness == 2`) shape with the
    /// Fermi function in the usual relativistic approximation
    /// F ∝ 2πη/(1 − e^(−2πη)) · p^(2(γ₀−1)), γ₀ = √(1 − (αZ)²).
    /// `z_daughter` is the charge of the daughter nucleus; `positron` selects β⁺.
    pub fn new(endpoint: f64, z_daughter: u32, positron: bool, forbiddenness: u8) -> BetaSpectrum {
        let n = 400;
        let w0 = endpoint / ELECTRON_MASS + 1.0;
        let gamma0 = (1.0 - (ALPHA * z_daughter as f64).powi(2)).max(0.0).sqrt();
        let density = |t: f64| -> f64 {
            let w = t / ELECTRON_MASS + 1.0;
            let p = (w * w - 1.0).max(0.0).sqrt();
            if p <= 0.0 {
                return 0.0;
            }
            let q = (w0 - w).max(0.0);
            let eta = if positron { -1.0 } else { 1.0 } * ALPHA * z_daughter as f64 * w / p;
            let x = 2.0 * std::f64::consts::PI * eta;
            let fermi = if x.abs() < 1e-8 { 1.0 } else { x / (1.0 - (-x).exp()) } * p.powf(2.0 * (gamma0 - 1.0));
            let shape = if forbiddenness == 2 { p * p + q * q } else { 1.0 };
            fermi * p * w * q * q * shape
        };
        let energies: Vec<f64> = (0..=n).map(|i| endpoint * i as f64 / n as f64).collect();
        let pdf: Vec<f64> = energies.iter().map(|&t| density(t)).collect();
        let mut cdf = vec![0.0; n + 1];
        let mut first = 0.0;
        for i in 1..=n {
            let area = 0.5 * (pdf[i] + pdf[i - 1]) * (energies[i] - energies[i - 1]);
            cdf[i] = cdf[i - 1] + area;
            first += 0.5 * (pdf[i] * energies[i] + pdf[i - 1] * energies[i - 1]) * (energies[i] - energies[i - 1]);
        }
        let total = cdf[n].max(1e-300);
        let mean = first / total;
        cdf.iter_mut().for_each(|c| *c /= total);
        BetaSpectrum { energies, cdf, mean }
    }

    pub fn sample(&self, rng: &mut impl Rng) -> f64 {
        let u = rng.uniform();
        let i = self.cdf.partition_point(|&c| c < u).clamp(1, self.cdf.len() - 1);
        let (c0, c1) = (self.cdf[i - 1], self.cdf[i]);
        let f = if c1 > c0 { (u - c0) / (c1 - c0) } else { 0.5 };
        self.energies[i - 1] + f * (self.energies[i] - self.energies[i - 1])
    }
}

/// Direction of the recoil electron in Compton scattering, from momentum conservation.
pub fn compton_electron_direction(e0: f64, dir0: Vec3, e1: f64, dir1: Vec3) -> Vec3 {
    let p = dir0 * e0 - dir1 * e1;
    let n = p.norm();
    if n > 0.0 {
        p / n
    } else {
        dir0
    }
}

/// Atom density helper.
pub fn atom_density(density: f64, mass_fraction: f64, atomic_mass: f64) -> f64 {
    density * mass_fraction * AVOGADRO / atomic_mass
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::{Philox, StreamRng};

    #[test]
    fn bragg_rule_water_and_air() {
        let water = bragg_mean_excitation(&[(1, 0.111894, 1.008), (8, 0.888106, 15.999)], true);
        assert!((water - 75.3).abs() < 0.5, "{water}");
        let air = bragg_mean_excitation(
            &[
                (6, 0.000124, 12.011),
                (7, 0.755268, 14.007),
                (8, 0.231781, 15.999),
                (18, 0.012827, 39.948),
            ],
            false,
        );
        assert!((air - 85.7).abs() < 0.5, "{air}");
    }

    #[test]
    fn bethe_water_vs_estar() {
        // liquid water, I = 75 eV: ESTAR collision stopping powers (MeV cm²/g)
        let za = 0.111894 / 1.008 + 0.888106 * 8.0 / 15.999;
        let de = DensityEffect::new(75.0, za, 1.0, true);
        for (t, estar) in [(0.01, 22.56), (0.1, 4.115), (1.0, 1.849), (10.0, 1.968)] {
            let s = bethe_collision(t, 75.0, za, 1.0, &de, false);
            assert!(((s - estar) / estar).abs() < 0.015, "{t} MeV: {s} vs {estar}");
        }
    }

    #[test]
    fn multiple_scattering_mean() {
        let mut rng = Philox::stream(3, 0);
        for ratio in [0.01, 0.1, 0.5, 2.0] {
            let n = 200_000;
            let mean: f64 = (0..n)
                .map(|_| sample_multiple_scattering(ratio, 1.0, &mut rng))
                .sum::<f64>()
                / n as f64;
            let expect = (-ratio).exp();
            assert!((mean - expect).abs() < 0.005, "s/λ = {ratio}: {mean} vs {expect}");
        }
    }

    #[test]
    fn sauter_forward_at_high_energy() {
        let mut rng = Philox::stream(4, 0);
        let lo: f64 = (0..50_000).map(|_| sample_sauter(0.01, &mut rng)).sum::<f64>() / 50_000.0;
        let hi: f64 = (0..50_000).map(|_| sample_sauter(1.0, &mut rng)).sum::<f64>() / 50_000.0;
        assert!(lo < hi && hi > 0.5, "{lo} {hi}");
    }

    #[test]
    fn beta_spectrum_i131_main_branch() {
        // I-131 main branch: endpoint 606.3 keV, daughter Xe (Z = 54);
        // mean energy of this branch ≈ 191.6 keV (ENSDF/NNDC)
        let s = BetaSpectrum::new(0.6063, 54, false, 1);
        assert!((s.mean - 0.1916).abs() < 0.006, "{}", s.mean);
    }

    #[test]
    fn restricted_stopping_limits() {
        let za = 0.111894 / 1.008 + 0.888106 * 8.0 / 15.999;
        let de = DensityEffect::new(75.0, za, 1.0, true);
        for positron in [false, true] {
            for t in [0.05, 0.5, 5.0] {
                let full = bethe_collision(t, 75.0, za, 1.0, &de, positron);
                // a cut at the largest transfer leaves the full stopping power
                let same = restricted_collision(full, t, max_delta_energy(t, positron), za, 1.0, positron);
                assert!((same - full).abs() < 1e-12 * full);
                // the energy lost in hard collisions, Σ_δ⟨W⟩, closes the balance
                let cut = 0.01;
                let soft = restricted_collision(full, t, cut, za, 1.0, positron);
                let wmax = max_delta_energy(t, positron);
                let n = 20_000;
                let (mut hard, mut prev) = (0.0, cut);
                for k in 1..=n {
                    let w = cut * (wmax / cut).powf(k as f64 / n as f64);
                    let dsig =
                        delta_cross_section(t, prev, za, 1.0, positron) - delta_cross_section(t, w, za, 1.0, positron);
                    hard += dsig * 0.5 * (prev + w);
                    prev = w;
                }
                let rel = (soft + hard) / full - 1.0;
                assert!(
                    rel.abs() < 2e-3,
                    "positron {positron}, {t} MeV: soft {soft} + hard {hard} vs {full} ({rel})"
                );
            }
        }
    }

    #[test]
    fn delta_sampling_matches_cross_section() {
        let mut rng = Philox::stream(5, 0);
        for positron in [false, true] {
            let (t, cut) = (1.0, 0.01);
            let n = 200_000;
            let w_split = 0.05;
            let above = (0..n)
                .filter(|_| sample_delta_energy(t, cut, positron, &mut rng) > w_split)
                .count() as f64
                / n as f64;
            let expect =
                delta_cross_section(t, w_split, 1.0, 1.0, positron) / delta_cross_section(t, cut, 1.0, 1.0, positron);
            assert!(
                (above - expect).abs() < 0.004,
                "positron {positron}: {above} vs {expect}"
            );
        }
    }
}
