//! Tests against the real ENDF/B-VIII.0 files when `GUARACI_TEST_ENDF` points
//! to a directory holding `ENDF-B-VIII.0_photoat/` and `ENDF-B-VIII.0_decay/`.

use crate::decay::*;
use crate::photon::*;
use std::path::PathBuf;

fn root() -> Option<PathBuf> {
    std::env::var_os("GUARACI_TEST_ENDF").map(PathBuf::from)
}

fn decay(name: &str) -> Option<NuclideDecay> {
    let p = root()?.join("ENDF-B-VIII.0_decay").join(name);
    let text = std::fs::read_to_string(&p).unwrap();
    read_decay(name, &text).unwrap()
}

fn close(a: f64, b: f64, rel: f64) -> bool {
    (a - b).abs() <= rel * b.abs()
}

#[test]
fn water_attenuation() {
    let Some(r) = root() else { return };
    let read = |n: &str| {
        let p = r.join("ENDF-B-VIII.0_photoat").join(n);
        read_photoatomic(n, &std::fs::read_to_string(p).unwrap()).unwrap()
    };
    let h = read("photoat-001_H_000.endf");
    let o = read("photoat-008_O_000.endf");
    assert_eq!(o.z, 8);
    assert!(close(o.atomic_mass, 15.999, 1e-3), "{}", o.atomic_mass);
    // mass attenuation coefficient of water at 140 keV (with coherent): ~0.154 cm2/g (XCOM 0.1538)
    let e = 0.140;
    let sigma =
        |x: &ElementPhoton| x.coherent.eval(e) + x.incoherent.eval(e) + x.photoelectric.eval(e) + x.pair.eval(e);
    let avogadro = 6.022_140_76e23;
    let mol = 2.0 * h.atomic_mass + o.atomic_mass;
    let mu_rho = (2.0 * sigma(&h) + sigma(&o)) * 1e-24 * avogadro / mol;
    assert!(close(mu_rho, 0.1538, 0.01), "mu/rho = {mu_rho}");
    // F(0) = Z and S(large x) = Z
    assert!(close(o.form_factor.eval(0.0), 8.0, 1e-9));
    assert!(close(o.scattering_function.eval(1e9), 8.0, 1e-6));
}

#[test]
fn iodine_131() {
    let Some(d) = decay("dec-053_I_131.endf") else { return };
    assert_eq!(d.id.to_string(), "I-131");
    assert!(close(d.half_life / 86_400.0, 8.0252, 1e-3), "{}", d.half_life / 86400.0);
    let main = d.photons[0];
    assert!(close(main.energy, 0.364490, 1e-4), "{main:?}");
    assert!(close(main.intensity, 0.815, 0.01), "{main:?}");
    assert!(
        close(d.mean_electron_energy, 0.1917, 0.01),
        "{}",
        d.mean_electron_energy
    );
}

#[test]
fn technetium_99m() {
    let Some(d) = decay("dec-043_Tc_099m1.endf") else {
        return;
    };
    assert_eq!(d.id.to_string(), "Tc-99m");
    assert!(close(d.half_life / 3600.0, 6.0067, 2e-3), "{}", d.half_life / 3600.0);
    assert!(close(d.photons[0].energy, 0.140511, 1e-4));
    assert!(close(d.photons[0].intensity, 0.885, 0.01), "{:?}", d.photons[0]);
}

#[test]
fn fluorine_18_annihilation() {
    let Some(d) = decay("dec-009_F_018.endf") else { return };
    let ann: f64 = d
        .photons
        .iter()
        .filter(|p| p.kind == PhotonKind::Annihilation)
        .map(|p| p.intensity)
        .sum();
    assert!(close(ann, 1.934, 0.01), "{ann} {:?}", d.photons);
}

#[test]
fn carbon_14_pure_beta() {
    let Some(d) = decay("dec-006_C_014.endf") else { return };
    assert!(d.photons.is_empty());
    assert!(
        close(d.mean_electron_energy, 0.0495, 0.01),
        "{}",
        d.mean_electron_energy
    );
}

fn electron(name: &str) -> Option<crate::electron::ElementElectron> {
    let p = root()?.join("ENDF-B-VIII.0_electrons").join(name);
    if !p.exists() {
        return None;
    }
    Some(crate::electron::read_electron(name, &std::fs::read_to_string(p).unwrap()).unwrap())
}

/// Stopping powers of liquid water (MeV cm²/g) vs NIST ESTAR.
#[test]
fn water_stopping_power() {
    let (Some(h), Some(o)) = (electron("e-001_H_000.endf"), electron("e-008_O_000.endf")) else {
        return;
    };
    let (wh, wo) = (0.111894, 0.888106);
    let na = 6.022_140_76e23;
    let per_g = |el: &crate::electron::ElementElectron,
                 a: f64,
                 w: f64,
                 v: &dyn Fn(&crate::electron::ElementElectron, usize) -> f64,
                 e: f64| {
        let t = crate::photon::Table {
            x: el.energy.clone(),
            y: (0..el.energy.len()).map(|i| v(el, i)).collect(),
        };
        w * na / a * t.eval(e) * 1e-24
    };
    let col = |el: &crate::electron::ElementElectron, i: usize| el.stopping_collision[i];
    let rad = |el: &crate::electron::ElementElectron, i: usize| el.stopping_radiative[i];
    for (e, estar_col, estar_rad) in [
        (0.01, 22.56, 3.898e-3),
        (0.1, 4.115, 6.229e-3),
        (1.0, 1.849, 1.293e-2),
        (10.0, 1.968, 0.1832),
    ] {
        let c = per_g(&h, 1.008, wh, &col, e) + per_g(&o, 15.999, wo, &col, e);
        let r = per_g(&h, 1.008, wh, &rad, e) + per_g(&o, 15.999, wo, &rad, e);
        eprintln!("{e:>6} MeV  collision {c:.4} (ESTAR {estar_col}, {:+.2} %)  radiative {r:.4e} (ESTAR {estar_rad}, {:+.2} %)", 100.0 * (c - estar_col) / estar_col, 100.0 * (r - estar_rad) / estar_rad);
    }
}

#[test]
fn beta_branches() {
    let Some(d) = decay("dec-053_I_131.endf") else { return };
    let main = d
        .betas
        .iter()
        .max_by(|a, b| a.intensity.total_cmp(&b.intensity))
        .unwrap();
    assert!(
        close(main.endpoint, 0.6063, 1e-3) && close(main.intensity, 0.896, 0.01),
        "{main:?}"
    );
    assert!(close(d.mean_beta_energy, 0.18185, 1e-3), "{}", d.mean_beta_energy);
    assert!(!d.electrons.is_empty());
    let Some(f) = decay("dec-009_F_018.endf") else { return };
    let plus: Vec<_> = f.betas.iter().filter(|b| b.kind == BetaKind::Plus).collect();
    let intensity: f64 = plus.iter().map(|b| b.intensity).sum();
    assert!(close(intensity, 0.967, 0.01), "{intensity} {:?}", f.betas);
    // F-18 positron endpoint: 633.5 keV
    assert!(close(plus[0].endpoint, 0.6335, 2e-3), "{:?}", plus[0]);
}

/// The bremsstrahlung spectra are normalised and their mean photon energy,
/// times the cross section, gives back the radiative stopping power.
#[test]
fn bremsstrahlung_spectra_consistent() {
    let Some(o) = electron("e-008_O_000.endf") else { return };
    assert!(o.brem_spectra.len() > 5, "{}", o.brem_spectra.len());
    for s in &o.brem_spectra {
        let (mut norm, mut mean) = (0.0, 0.0);
        for i in 1..s.kappa.len() {
            let (a, b, fa, fb) = (s.kappa[i - 1], s.kappa[i], s.density[i - 1], s.density[i]);
            norm += 0.5 * (fa + fb) * (b - a);
            mean += (b - a) / 6.0 * (fa * (2.0 * a + b) + fb * (a + 2.0 * b));
        }
        assert!((norm - 1.0).abs() < 1e-3, "{} MeV: norm {norm}", s.energy);
        let k = o.energy.iter().position(|&e| (e / s.energy - 1.0).abs() < 0.02);
        if let Some(k) = k {
            let s_rad = o.brem_cross_section[k] * mean * o.energy[k];
            let rel = s_rad / o.stopping_radiative[k] - 1.0;
            eprintln!("{:.4e} MeV: σ⟨k⟩ / S_rad − 1 = {rel:+.4}", s.energy);
            assert!(rel.abs() < 0.03, "{} MeV: {rel}", s.energy);
        }
    }
}
