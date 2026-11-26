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
