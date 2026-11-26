//! Radioactive decay data (ENDF-6 MF8 MT457).

use crate::elements::NuclideId;
use crate::endf::{self, EndfError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhotonKind {
    Gamma,
    XRay,
    /// 511 keV photons from positron annihilation.
    Annihilation,
}

/// A discrete photon line.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct PhotonLine {
    /// MeV
    pub energy: f64,
    /// Photons per decay.
    pub intensity: f64,
    pub kind: PhotonKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BetaKind {
    Minus,
    Plus,
}

/// One β branch: the spectrum is computed from the endpoint (Fermi theory).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct BetaBranch {
    pub kind: BetaKind,
    /// Endpoint (maximum kinetic) energy, MeV.
    pub endpoint: f64,
    /// Emitted particles per decay.
    pub intensity: f64,
    /// ENDF transition type: 1 allowed, 2 first-forbidden unique, ...
    pub forbiddenness: u8,
}

/// A discrete electron line (internal conversion or Auger).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ElectronLine {
    /// MeV
    pub energy: f64,
    /// Electrons per decay.
    pub intensity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecayMode {
    /// ENDF RTYP, e.g. 1 = β⁻, 2 = β⁺/EC, 3 = IT, 4 = α; 1.5 = β⁻ then n.
    pub rtyp: f64,
    pub branching: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NuclideDecay {
    pub id: NuclideId,
    /// Seconds; `f64::INFINITY` for stable nuclides.
    pub half_life: f64,
    /// Mean energy per decay carried by light particles (β⁻, β⁺, conversion and
    /// Auger electrons), MeV.
    pub mean_electron_energy: f64,
    /// Mean energy per decay carried by photons (γ, X, annihilation), MeV.
    pub mean_photon_energy: f64,
    /// Mean energy per decay carried by heavy particles (α, recoil...), MeV.
    pub mean_heavy_energy: f64,
    pub modes: Vec<DecayMode>,
    pub photons: Vec<PhotonLine>,
    /// True if the file gives a continuous photon spectrum that is not included in `photons`.
    pub has_continuous_photons: bool,
    /// β⁻ and β⁺ branches.
    pub betas: Vec<BetaBranch>,
    /// Conversion and Auger electrons.
    pub electrons: Vec<ElectronLine>,
    /// Mean β energy per decay given by the library (ERAV × FD of STYP 1 and 2), MeV.
    pub mean_beta_energy: f64,
}

impl NuclideDecay {
    pub fn is_stable(&self) -> bool {
        self.half_life.is_infinite()
    }
    /// Sum of the energy of the discrete photon lines per decay (MeV).
    pub fn discrete_photon_energy(&self) -> f64 {
        self.photons.iter().map(|p| p.energy * p.intensity).sum()
    }
}

const ANNIHILATION_EV: f64 = 510_998.95;
const ELECTRON_MASS_MEV: f64 = 0.510_998_950;

/// Read an ENDF-6 decay file. Returns `None` for files without MF8 MT457.
pub fn read_decay(file: &str, text: &str) -> Result<Option<NuclideDecay>, EndfError> {
    let secs = endf::sections(file, text);
    let Some(sec) = secs.get(&(8, 457)) else {
        return Ok(None);
    };
    let mut r = sec.reader();
    let head = r.cont()?;
    let za = head.c1.round() as u32;
    let id = NuclideId {
        z: za / 1000,
        a: za % 1000,
        isomer: head.l2.max(0) as u32,
    };
    let nsp = head.n2.max(0) as usize;
    let stable = head.n1 == 1; // NST

    let (c, energies) = r.list()?;
    let half_life = if stable || c.c1 <= 0.0 { f64::INFINITY } else { c.c1 };
    let e = |i: usize| energies.get(2 * i).copied().unwrap_or(0.0) * 1e-6;
    let (mean_electron_energy, mean_photon_energy, mean_heavy_energy) = (e(0), e(1), e(2));

    let (_, modes_raw) = r.list()?;
    let modes = modes_raw
        .chunks(6)
        .map(|m| DecayMode {
            rtyp: m[0],
            branching: m[4],
        })
        .collect();

    let mut photons = Vec::new();
    let mut has_continuous_photons = false;
    let mut betas = Vec::new();
    let mut electrons = Vec::new();
    let mut mean_beta_energy = 0.0;
    for _ in 0..nsp {
        let (sc, sv) = r.list()?;
        let styp = sc.c2.round() as i64;
        let lcon = sc.l1;
        let ner = sc.n2.max(0) as usize;
        let fd = sv.first().copied().unwrap_or(1.0);
        if matches!(styp, 1 | 2) {
            mean_beta_energy += fd * sv.get(2).copied().unwrap_or(0.0) * 1e-6;
        }
        if lcon != 1 {
            for _ in 0..ner {
                let (lc, lv) = r.list()?;
                let ri = lv.get(2).copied().unwrap_or(0.0);
                let kind = match styp {
                    0 => Some(PhotonKind::Gamma),
                    9 if (lc.c1 - ANNIHILATION_EV).abs() < 50.0 => Some(PhotonKind::Annihilation),
                    9 => Some(PhotonKind::XRay),
                    _ => None,
                };
                if let Some(kind) = kind {
                    if ri > 0.0 {
                        photons.push(PhotonLine {
                            energy: lc.c1 * 1e-6,
                            intensity: fd * ri,
                            kind,
                        });
                    }
                }
                let ty = lv.get(1).copied().unwrap_or(1.0).round().max(1.0) as u8;
                match styp {
                    1 if ri > 0.0 => betas.push(BetaBranch {
                        kind: BetaKind::Minus,
                        endpoint: lc.c1 * 1e-6,
                        intensity: fd * ri,
                        forbiddenness: ty,
                    }),
                    2 => {
                        // RI counts β⁺ and EC together; RIS is the positron intensity
                        let ris = lv.get(4).copied().unwrap_or(0.0);
                        if ris > 0.0 {
                            // for β⁺ the library gives the decay energy Q_EC of the branch;
                            // the positron endpoint is Q_EC − 2 mₑc²
                            betas.push(BetaBranch {
                                kind: BetaKind::Plus,
                                endpoint: (lc.c1 * 1e-6 - 2.0 * ELECTRON_MASS_MEV).max(0.0),
                                intensity: fd * ris,
                                forbiddenness: ty,
                            });
                        }
                    }
                    8 if ri > 0.0 => electrons.push(ElectronLine {
                        energy: lc.c1 * 1e-6,
                        intensity: fd * ri,
                    }),
                    _ => {}
                }
            }
        }
        if lcon != 0 {
            if matches!(styp, 0 | 9) {
                has_continuous_photons = true;
            }
            let t = r.tab1()?; // continuous spectrum
            if t.cont.l2 != 0 {
                // LCOV != 0: covariance data follows
                r.list()?;
            }
        }
    }
    photons.sort_by(|a, b| b.intensity.total_cmp(&a.intensity));
    Ok(Some(NuclideDecay {
        id,
        half_life,
        mean_electron_energy,
        mean_photon_energy,
        mean_heavy_energy,
        modes,
        photons,
        has_continuous_photons,
        betas,
        electrons,
        mean_beta_energy,
    }))
}
