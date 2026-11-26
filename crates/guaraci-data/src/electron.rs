//! Electron interaction data per element (ENDF-6 electro-atomic sublibrary,
//! EEDL/EPICS2017): stopping powers and elastic transport cross section,
//! derived at conversion time from the cross sections (MF23) and the
//! energy-loss and angular data (MF26).
//!
//! Units in the converted data: energy MeV, cross sections barn, stopping
//! powers MeV·barn per atom (multiply by the atom density × 1e-24 to get MeV/cm).

use crate::endf::{self, EndfError, Section, Tab1};
use crate::photon::Table;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementElectron {
    pub z: u32,
    /// Energy grid (MeV), logarithmic.
    pub energy: Vec<f64>,
    /// Collision (ionisation + excitation) stopping power, MeV·barn per atom.
    pub stopping_collision: Vec<f64>,
    /// Radiative (bremsstrahlung) stopping power, MeV·barn per atom.
    pub stopping_radiative: Vec<f64>,
    /// Elastic transport cross section σ_tr = σ_el (1 − ⟨cos θ⟩), barn.
    pub elastic_transport: Vec<f64>,
    /// Total elastic cross section, barn.
    pub elastic: Vec<f64>,
    /// Bremsstrahlung cross section (all photon energies), barn.
    pub brem_cross_section: Vec<f64>,
    /// Bremsstrahlung photon spectra at the incident energies of the library.
    pub brem_spectra: Vec<BremSpectrum>,
}

/// Normalised bremsstrahlung photon spectrum at one incident energy, in the
/// reduced photon energy κ = k / E (0 < κ ≤ 1); the density is linear between points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BremSpectrum {
    /// Incident electron kinetic energy, MeV.
    pub energy: f64,
    pub kappa: Vec<f64>,
    /// Probability density in κ.
    pub density: Vec<f64>,
}

const EV: f64 = 1e-6; // eV -> MeV

fn lin(x: &[f64], y: &[f64], v: f64) -> f64 {
    Table {
        x: x.to_vec(),
        y: y.to_vec(),
    }
    .eval(v)
}

/// log-log interpolation for positive tables, lin-lin otherwise
fn interp(x: &[f64], y: &[f64], v: f64) -> f64 {
    if x.is_empty() {
        return 0.0;
    }
    if v < x[0] {
        return 0.0; // below threshold
    }
    lin(x, y, v)
}

fn mf23(secs: &BTreeMap<(u32, u32), Section>, mt: u32) -> Result<Option<(f64, Tab1)>, EndfError> {
    let Some(s) = secs.get(&(23, mt)) else { return Ok(None) };
    let mut r = s.reader();
    r.cont()?;
    let t = r.tab1()?;
    // C1 of the TAB1 is the binding energy for subshell ionisation (MT 534+)
    Ok(Some((t.cont.c1, t)))
}

/// One MF26 sub-section, reduced to what Guaraci needs.
enum Law {
    /// LAW=8: average energy transfer ET(E), eV.
    EnergyTransfer(Tab1),
    /// LAW=1 or LAW=2 tables: for each incident energy, (x, f(x)) pairs.
    Tables {
        energies: Vec<f64>,
        tables: Vec<(Vec<f64>, Vec<f64>)>,
    },
}

fn mf26(secs: &BTreeMap<(u32, u32), Section>, mt: u32) -> Result<Vec<(f64, Law)>, EndfError> {
    let Some(s) = secs.get(&(26, mt)) else {
        return Ok(vec![]);
    };
    let mut r = s.reader();
    let head = r.cont()?;
    let mut out = Vec::new();
    for _ in 0..head.n1.max(0) {
        let y = r.tab1()?; // yield
        let zap = y.cont.c1;
        let law = y.cont.l2;
        match law {
            8 => out.push((zap, Law::EnergyTransfer(r.tab1()?))),
            1 | 2 => {
                let (c, _) = r.tab2()?;
                let ne = c.n2.max(0) as usize;
                let mut energies = Vec::with_capacity(ne);
                let mut tables = Vec::with_capacity(ne);
                for _ in 0..ne {
                    let (lc, v) = r.list()?;
                    energies.push(lc.c2);
                    // LAW=1 (NA=0): pairs (E', f); LAW=2 (LANG=12): pairs (mu, p)
                    let (xs, ys) = v.chunks(2).filter(|p| p.len() == 2).map(|p| (p[0], p[1])).unzip();
                    tables.push((xs, ys));
                }
                out.push((zap, Law::Tables { energies, tables }));
            }
            other => {
                return Err(EndfError {
                    file: String::new(),
                    line: 0,
                    msg: format!("MF26 MT{mt}: unsupported LAW {other}"),
                })
            }
        }
    }
    Ok(out)
}

/// Mean of x under the piecewise-linear density f(x).
fn mean_x(x: &[f64], f: &[f64]) -> f64 {
    let (mut norm, mut first) = (0.0, 0.0);
    for i in 1..x.len() {
        let (a, b, fa, fb) = (x[i - 1], x[i], f[i - 1], f[i]);
        let h = b - a;
        norm += 0.5 * (fa + fb) * h;
        first += h / 6.0 * (fa * (2.0 * a + b) + fb * (a + 2.0 * b));
    }
    if norm > 0.0 {
        first / norm
    } else {
        0.0
    }
}

/// Mean of 1 − x under the piecewise-linear density f(x) (exact for linear f in each panel).
/// Works with 1 − x directly, so no precision is lost when x is close to 1.
fn mean_one_minus_x(x: &[f64], f: &[f64]) -> f64 {
    let (mut norm, mut first) = (0.0, 0.0);
    for i in 1..x.len() {
        let (a, b, fa, fb) = (1.0 - x[i - 1], 1.0 - x[i], f[i - 1], f[i]);
        let h = x[i] - x[i - 1];
        norm += 0.5 * (fa + fb) * h;
        // ∫ (1 − x) f dx for linear f on the panel
        first += h / 6.0 * (fa * (2.0 * a + b) + fb * (a + 2.0 * b));
    }
    if norm > 0.0 {
        first / norm
    } else {
        0.0
    }
}

/// Quantity tabulated at incident energies, interpolated linearly in ln E.
fn at_energy(energies: &[f64], values: &[f64], e: f64) -> f64 {
    let n = energies.len();
    if n == 0 {
        return 0.0;
    }
    if e <= energies[0] {
        return values[0];
    }
    if e >= energies[n - 1] {
        return values[n - 1];
    }
    let i = energies.partition_point(|&v| v <= e) - 1;
    let (l0, l1) = (energies[i].ln(), energies[i + 1].ln());
    values[i] + (values[i + 1] - values[i]) * (e.ln() - l0) / (l1 - l0)
}

/// Positive quantity tabulated at incident energies, interpolated log-log.
fn at_energy_loglog(energies: &[f64], values: &[f64], e: f64) -> f64 {
    let n = energies.len();
    if n == 0 {
        return 0.0;
    }
    if e <= energies[0] {
        return values[0];
    }
    if e >= energies[n - 1] {
        return values[n - 1];
    }
    let i = energies.partition_point(|&v| v <= e) - 1;
    let f = (e / energies[i]).ln() / (energies[i + 1] / energies[i]).ln();
    (values[i].ln() + f * (values[i + 1] / values[i]).ln()).exp()
}

/// Read an ENDF-6 electro-atomic file and derive the transport quantities.
pub fn read_electron(file: &str, text: &str) -> Result<ElementElectron, EndfError> {
    let secs = endf::sections(file, text);
    let err = |msg: &str| EndfError {
        file: file.to_string(),
        line: 0,
        msg: msg.to_string(),
    };
    let head = secs
        .get(&(23, 526))
        .ok_or_else(|| err("not an electro-atomic file (no MF23 MT526)"))?
        .reader()
        .cont()?;
    let z = (head.c1 / 1000.0).round() as u32;

    // grid: 100 eV .. 100 MeV, 40 points per decade (energies in eV here)
    let grid: Vec<f64> = (0..=240).map(|i| 1e2 * 10f64.powf(i as f64 / 40.0)).collect();

    // --- elastic: total (526) and large-angle (525) with its angular distribution
    let (_, el_tot) = mf23(&secs, 526)?.ok_or_else(|| err("missing MT526"))?;
    let (_, el_large) = mf23(&secs, 525)?.ok_or_else(|| err("missing MT525"))?;
    // 1 − ⟨μ⟩ of the large-angle distribution at the tabulated energies
    let mut mu_e = Vec::new();
    let mut one_minus_mu = Vec::new();
    for (_, law) in mf26(&secs, 525)? {
        if let Law::Tables { energies, tables } = law {
            for (e, (mu, p)) in energies.iter().zip(&tables) {
                mu_e.push(*e);
                one_minus_mu.push(mean_one_minus_x(mu, p).max(1e-30));
            }
        }
    }

    // --- bremsstrahlung: cross section and mean photon energy (LAW=8 sub-section)
    let brem = mf23(&secs, 527)?;
    let mut brem_et: Option<Tab1> = None;
    let mut brem_spectra = Vec::new();
    for (_, law) in mf26(&secs, 527)? {
        match law {
            Law::EnergyTransfer(t) => brem_et = Some(t),
            // the only tabulated sub-section of MT527: photon spectra in photon energy, eV
            Law::Tables { energies, tables } => {
                for (e, (k, f)) in energies.iter().zip(tables) {
                    if *e <= 0.0 || k.len() < 2 {
                        continue;
                    }
                    brem_spectra.push(BremSpectrum {
                        energy: e * EV,
                        kappa: k.iter().map(|v| v / e).collect(),
                        density: f.iter().map(|v| v * e).collect(),
                    });
                }
            }
        }
    }

    // --- excitation: cross section and mean energy loss
    let exc = mf23(&secs, 528)?;
    let mut exc_et: Option<Tab1> = None;
    for (_, law) in mf26(&secs, 528)? {
        if let Law::EnergyTransfer(t) = law {
            exc_et = Some(t);
        }
    }

    // --- ionisation per subshell: cross section, binding energy, mean recoil energy
    struct Shell {
        binding: f64,
        xs: Tab1,
        rec_e: Vec<f64>,
        rec_mean: Vec<f64>,
    }
    let mut shells = Vec::new();
    for mt in 534..=599 {
        let Some((binding, xs)) = mf23(&secs, mt)? else {
            continue;
        };
        let mut rec_e = Vec::new();
        let mut rec_mean = Vec::new();
        for (zap, law) in mf26(&secs, mt)? {
            if zap.round() as i64 != 11 {
                continue;
            }
            if let Law::Tables { energies, tables } = law {
                for (e, (x, f)) in energies.iter().zip(&tables) {
                    rec_e.push(*e);
                    rec_mean.push(mean_x(x, f));
                }
            }
        }
        shells.push(Shell {
            binding,
            xs,
            rec_e,
            rec_mean,
        });
    }

    let mut out = ElementElectron {
        z,
        energy: Vec::with_capacity(grid.len()),
        stopping_collision: Vec::with_capacity(grid.len()),
        stopping_radiative: Vec::with_capacity(grid.len()),
        elastic_transport: Vec::with_capacity(grid.len()),
        elastic: Vec::with_capacity(grid.len()),
        brem_cross_section: Vec::with_capacity(grid.len()),
        brem_spectra,
    };
    for &e in &grid {
        // collision: excitation + ionisation (binding + mean recoil energy)
        let mut s_col = 0.0;
        if let (Some((_, x)), Some(et)) = (&exc, &exc_et) {
            s_col += interp(&x.x, &x.y, e) * lin(&et.x, &et.y, e);
        }
        for sh in &shells {
            let sigma = interp(&sh.xs.x, &sh.xs.y, e);
            if sigma > 0.0 {
                s_col += sigma * (sh.binding + at_energy(&sh.rec_e, &sh.rec_mean, e));
            }
        }
        let mut s_rad = 0.0;
        let mut sig_brem = 0.0;
        if let Some((_, x)) = &brem {
            sig_brem = interp(&x.x, &x.y, e);
            if let Some(et) = &brem_et {
                s_rad = sig_brem * lin(&et.x, &et.y, e);
            }
        }
        let sig_large = interp(&el_large.x, &el_large.y, e);
        // the angular tables are sparse (e.g. 256 keV, then 10 MeV) and 1 − ⟨μ⟩
        // falls roughly as 1/E: interpolate it log-log
        let tr = sig_large * at_energy_loglog(&mu_e, &one_minus_mu, e);
        out.energy.push(e * EV);
        out.stopping_collision.push(s_col * EV);
        out.stopping_radiative.push(s_rad * EV);
        out.elastic_transport.push(tr);
        out.elastic.push(interp(&el_tot.x, &el_tot.y, e));
        out.brem_cross_section.push(sig_brem);
    }
    Ok(out)
}
