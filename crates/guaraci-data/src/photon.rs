//! Photo-atomic data per element (ENDF-6 MF23 cross sections, MF27 form factors).

use crate::endf::{self, EndfError, Tab1};
use serde::{Deserialize, Serialize};

/// Neutron mass in atomic mass units (CODATA 2018); converts ENDF AWR to u.
pub const NEUTRON_MASS_U: f64 = 1.008_664_915_95;

/// A tabulated function. Interpolation is linear in both x and y, which is the
/// law used by every MF23/MF27 table of the supported libraries (checked when
/// converting).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Table {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
}

impl Table {
    /// Linear interpolation; values outside the table are clamped to the ends.
    pub fn eval(&self, x: f64) -> f64 {
        let n = self.x.len();
        if n == 0 {
            return 0.0;
        }
        if x <= self.x[0] {
            return self.y[0];
        }
        if x >= self.x[n - 1] {
            return self.y[n - 1];
        }
        // last index with self.x[i] <= x
        let i = self.x.partition_point(|&v| v <= x) - 1;
        let (x0, x1, y0, y1) = (self.x[i], self.x[i + 1], self.y[i], self.y[i + 1]);
        if x1 == x0 {
            return y1;
        }
        y0 + (y1 - y0) * (x - x0) / (x1 - x0)
    }
}

/// Photon interaction data of one element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementPhoton {
    pub z: u32,
    /// Atomic mass (u), from the ENDF AWR.
    pub atomic_mass: f64,
    /// Coherent (Rayleigh) cross section: energy (MeV) -> barn.
    pub coherent: Table,
    /// Incoherent (Compton) cross section: MeV -> barn.
    pub incoherent: Table,
    /// Pair production, nuclear + electron field: MeV -> barn.
    pub pair: Table,
    /// Photoelectric absorption, all shells: MeV -> barn.
    pub photoelectric: Table,
    /// Coherent form factor F(x, Z); x = sin(θ/2)/λ in 1/cm.
    pub form_factor: Table,
    /// Incoherent scattering function S(x, Z); x in 1/cm.
    pub scattering_function: Table,
}

fn tab(
    secs: &std::collections::BTreeMap<(u32, u32), endf::Section>,
    mf: u32,
    mt: u32,
    file: &str,
) -> Result<Tab1, EndfError> {
    let s = secs.get(&(mf, mt)).ok_or_else(|| EndfError {
        file: file.to_string(),
        line: 0,
        msg: format!("section MF{mf} MT{mt} is missing"),
    })?;
    let mut r = s.reader();
    r.cont()?; // HEAD
    let t = r.tab1()?;
    if let Some(&(_, law)) = t.ranges.iter().find(|(_, law)| *law != 2) {
        return Err(EndfError {
            file: file.to_string(),
            line: 0,
            msg: format!("MF{mf} MT{mt} uses interpolation law {law}; only linear-linear (2) is supported"),
        });
    }
    Ok(t)
}

/// Read an ENDF-6 photo-atomic file.
pub fn read_photoatomic(file: &str, text: &str) -> Result<ElementPhoton, EndfError> {
    let secs = endf::sections(file, text);
    let head = secs
        .get(&(23, 502))
        .ok_or_else(|| EndfError {
            file: file.to_string(),
            line: 0,
            msg: "not a photo-atomic file (no MF23)".into(),
        })?
        .reader()
        .cont()?;
    let z = (head.c1 / 1000.0).round() as u32;
    let ev_to_mev = |t: Tab1| Table {
        x: t.x.iter().map(|e| e * 1e-6).collect(),
        y: t.y,
    };
    let plain = |t: Tab1| Table { x: t.x, y: t.y };
    Ok(ElementPhoton {
        z,
        atomic_mass: head.c2 * NEUTRON_MASS_U,
        coherent: ev_to_mev(tab(&secs, 23, 502, file)?),
        incoherent: ev_to_mev(tab(&secs, 23, 504, file)?),
        pair: ev_to_mev(tab(&secs, 23, 516, file)?),
        photoelectric: ev_to_mev(tab(&secs, 23, 522, file)?),
        form_factor: plain(tab(&secs, 27, 502, file)?),
        scattering_function: plain(tab(&secs, 27, 504, file)?),
    })
}
