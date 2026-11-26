//! Physics benchmarks against reference values.
//!
//! They need installed data: set `GUARACI_TEST_DATA` to a Guaraci data
//! directory where `guaraci data install endfb8` was run. Without it the tests
//! are skipped.

use guaraci_core::physics::{Grid, Material};
use guaraci_core::problem::{Probe, Problem};
use guaraci_cpu::{run, Options, RngChoice};
use guaraci_data::library::{load_decay, load_electron, load_photon, DecayData, ElectronData, PhotonData};
use guaraci_input::{load_str, Builtin, Config};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::OnceLock;

const MATERIALS: Builtin = Builtin {
    name: "materials",
    text: include_str!("../../../data/materials.gci"),
};

fn data() -> Option<&'static (PhotonData, DecayData)> {
    static DATA: OnceLock<Option<(PhotonData, DecayData)>> = OnceLock::new();
    DATA.get_or_init(|| {
        let dir = PathBuf::from(std::env::var_os("GUARACI_TEST_DATA")?);
        Some((load_photon(&dir, "endfb8").ok()?.1, load_decay(&dir, "endfb8").ok()?.1))
    })
    .as_ref()
}

fn electron_data() -> Option<&'static ElectronData> {
    static DATA: OnceLock<Option<ElectronData>> = OnceLock::new();
    DATA.get_or_init(|| {
        let dir = PathBuf::from(std::env::var_os("GUARACI_TEST_DATA")?);
        Some(load_electron(&dir, "endfb8").ok()?.1)
    })
    .as_ref()
}

fn problem(text: &str) -> Option<Problem> {
    let (photon, decay) = data()?;
    let prog = load_str("bench.gci", text, &[MATERIALS])
        .map_err(|(_, d)| d)
        .expect("input");
    let c = Config::from_program(&prog).expect("config");
    Some(Problem::build(&c, photon, Some(decay), electron_data()).expect("problem"))
}

fn options(histories: u64, threads: usize) -> Options {
    Options {
        histories,
        batches: 10,
        seed: 42,
        rng: RngChoice::Philox,
        threads,
    }
}

fn material<'a>(p: &'a Problem, name: &str) -> &'a Material {
    p.materials.iter().find(|m| m.name == name).unwrap()
}

/// μ/ρ (cm²/g) with coherent scattering, NIST XCOM.
#[test]
fn attenuation_matches_xcom() {
    let text = "[world]\nmaterial = Water\nsize = 10\n[volume pb]\nshape = sphere\nradius = 1\nmaterial = Lead\n\
                [source]\nenergy = 10 MeV\nposition = [0, 0, 4]\n";
    let Some(p) = problem(text) else { return };
    let water = [
        (0.010, 5.329),
        (0.020, 0.8096),
        (0.030, 0.3756),
        (0.060, 0.2059),
        (0.100, 0.1707),
        (0.200, 0.1370),
        (0.500, 0.09687),
        (1.0, 0.07072),
        (2.0, 0.04942),
        (10.0, 0.02219),
    ];
    let lead = [(0.100, 5.549), (0.500, 0.1614), (1.0, 0.07102)];
    let check = |m: &Material, g: &Grid, table: &[(f64, f64)], tol: f64| {
        for &(e, xcom) in table {
            let v = m.mass_attenuation(g, e);
            let dev = (v - xcom) / xcom;
            eprintln!(
                "{:<6} {:>8.1} keV  {v:>9.5}  XCOM {xcom:>9.5}  {:+.2} %",
                m.name,
                e * 1e3,
                dev * 100.0
            );
            assert!(
                dev.abs() < tol,
                "{} at {} keV: {v:.5} vs XCOM {xcom} ({:+.2} %)",
                m.name,
                e * 1e3,
                dev * 100.0
            );
        }
    };
    check(material(&p, "Water"), &p.grid, &water, 0.02);
    check(material(&p, "Lead"), &p.grid, &lead, 0.03);
}

/// Point source at the centre of a water sphere: the uncollided fluence in a
/// thin air shell just outside must be exp(-μR)/(4πR²) per photon.
#[test]
fn uncollided_fluence_through_water_sphere() {
    let text = "[physics]\nrayleigh = off\n\
                [world]\nmaterial = Vacuum\nsize = 40\n\
                [material Vacuum]\ndensity = 1e-20\ncomposition = { H = 1 }\n\
                [volume shell]\nshape = sphere\nradius = 10.5\nmaterial = Vacuum\n\
                [volume ball]\nparent = shell\nshape = sphere\nradius = 10\nmaterial = Water\n\
                [source]\nenergy = 140.5 keV\nposition = [0, 0, 0]\n\
                [probe flu]\ntype = spectrum\nin = shell\nemax = 140.6 keV\nbins = 1406\n";
    let Some(p) = problem(text) else { return };
    let r = run(&p, &options(2_000_000, 0), &AtomicU64::new(0), &AtomicBool::new(false));
    let mu = material(&p, "Water").mass_attenuation(&p.grid, 0.1405);
    let Probe::Spectrum { volume_cm3, .. } = &p.probes[0] else {
        panic!()
    };
    let (m, e) = r.mean_and_error(0, 1405); // 140.5–140.6 keV: only uncollided photons
    let fluence = m / volume_cm3;
    // mean of exp(-μR)/(4πr²) over the shell r ∈ [10, 10.5]: track length / volume
    let expected = (-mu * 10.0f64).exp() * 0.5 / (4.0 / 3.0 * std::f64::consts::PI * (10.5f64.powi(3) - 1000.0));
    let dev = (fluence - expected) / expected;
    let sigma = e / m;
    eprintln!(
        "uncollided fluence {fluence:.5e} vs analytic {expected:.5e}: {:+.3} % (σ {:.3} %)",
        dev * 100.0,
        sigma * 100.0
    );
    assert!(
        dev.abs() < 4.0 * sigma + 0.002,
        "fluence {fluence:.5e} vs {expected:.5e} ({:+.3} %, σ {:.3} %)",
        dev * 100.0,
        sigma * 100.0
    );
}

/// Energy is conserved and results do not depend on the number of threads.
#[test]
fn conservation_and_thread_independence() {
    let text = "[world]\nmaterial = Air\nsize = 30\n\
                [volume body]\nshape = cylinder\nradius = 8\nheight = 20\nmaterial = Tissue_Soft_ICRP\n\
                [volume bone]\nparent = body\nshape = box\nsize = [2, 2, 10]\nmaterial = Bone_Cortical\n\
                [source]\nnuclide = F-18\nin = body\n\
                [probe organs]\ntype = volume_dose\nvolumes = [body, bone]\n";
    let Some(p) = problem(text) else { return };
    let a = run(&p, &options(200_000, 1), &AtomicU64::new(0), &AtomicBool::new(false));
    let b = run(&p, &options(200_000, 4), &AtomicU64::new(0), &AtomicBool::new(false));
    let t = &a.total;
    let rel = (t.emitted - t.deposited - t.escaped).abs() / t.emitted;
    assert!(rel < 1e-10, "energy not conserved: {rel}");
    for k in 0..2 {
        let (x, y) = (a.mean_and_error(0, k).0, b.mean_and_error(0, k).0);
        assert!(
            ((x - y) / x).abs() < 1e-12,
            "thread count changed the result: {x} vs {y}"
        );
    }
    // F-18: 1.93 annihilation photons and ~0.25 MeV of positron energy per decay
    let n = t.histories as f64;
    assert!(
        (t.emitted / n - (1.934 * 0.511 + 0.2498)).abs() < 0.02,
        "{}",
        t.emitted / n
    );
}

/// Reference rows of Stabin & Konijnenberg (2000), Tables 2 and 3.
struct StabinRow {
    mass: f64,
    radius: f64,
    energy: f64,
    egs4: f64,
    mcnp: f64,
    recommended: f64,
}

fn parse_stabin(csv: &str) -> Vec<StabinRow> {
    csv.lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with("mass"))
        .map(|l| {
            let v: Vec<f64> = l.split(',').map(|x| x.parse().unwrap()).collect();
            StabinRow {
                mass: v[0],
                radius: v[1],
                energy: v[2],
                egs4: v[3],
                mcnp: v[4],
                recommended: v[5],
            }
        })
        .collect()
}

fn stabin_rows() -> Vec<StabinRow> {
    parse_stabin(include_str!("../../../validation/stabin2000_photons.csv"))
}

fn stabin_electron_rows() -> Vec<StabinRow> {
    parse_stabin(include_str!("../../../validation/stabin2000_electrons.csv"))
}

/// Absorbed fraction for a uniform monoenergetic photon source in a sphere of
/// soft tissue (unit density) inside a 1 m cube of the same tissue
/// (the "infinite medium" of the reference). Returns (fraction, relative σ).
fn absorbed_fraction(radius: f64, energy: f64, histories: u64) -> Option<(f64, f64)> {
    absorbed_fraction_of("photon", radius, energy, histories)
}

fn absorbed_fraction_of(particle: &str, radius: f64, energy: f64, histories: u64) -> Option<(f64, f64)> {
    let text = format!(
        "[world]\nmaterial = Tissue_Soft_ICRP\nsize = 100 cm\n\
         [volume sphere]\nshape = sphere\nradius = {radius} cm\nmaterial = Tissue_Soft_ICRP\n\
         [source]\nparticle = {particle}\nenergy = {energy} MeV\nin = sphere\n\
         [probe af]\ntype = volume_dose\nvolumes = [sphere]\n"
    );
    let p = problem(&text)?;
    let r = run(&p, &options(histories, 0), &AtomicU64::new(0), &AtomicBool::new(false));
    let (m, e) = r.mean_and_error(0, 0);
    Some((m / energy, e / m))
}

/// A few spheres and energies, run with every `cargo test`.
#[test]
fn stabin_spheres_selected() {
    if data().is_none() {
        return;
    }
    let rows = stabin_rows();
    for (mass, energy) in [
        (10.0, 0.14),
        (100.0, 0.364),
        (1000.0, 0.662),
        (100.0, 0.06),
        (3000.0, 0.1),
    ] {
        let row = rows.iter().find(|r| r.mass == mass && r.energy == energy).unwrap();
        let (af, rel) = absorbed_fraction(row.radius, energy, 200_000).unwrap();
        let dev = (af - row.recommended) / row.recommended;
        // the two reference codes differ by up to ~6 % (paper); the table is rounded to 3 digits
        let tol = 0.06 + 3.0 * rel + 0.0005 / row.recommended;
        eprintln!(
            "{mass:>6} g {:>6.0} keV  Guaraci {af:.4} ± {:.2} %  ref {:.3}  {:+.1} %",
            energy * 1e3,
            rel * 100.0,
            row.recommended,
            dev * 100.0
        );
        assert!(
            dev.abs() < tol,
            "{mass} g, {energy} MeV: {af:.4} vs {} ({:+.1} %)",
            row.recommended,
            dev * 100.0
        );
    }
}

/// The whole table. Slow: run with
/// `GUARACI_TEST_DATA=... cargo test --release -p guaraci-cpu stabin_spheres_full -- --ignored --nocapture`.
/// Writes validation/stabin2000_comparison.csv.
#[test]
#[ignore]
fn stabin_spheres_full() {
    if data().is_none() {
        return;
    }
    let mut out = String::from("mass_g,energy_MeV,guaraci,guaraci_rel_sigma,egs4,mcnp,recommended,deviation_percent\n");
    let mut devs = Vec::new();
    for row in stabin_rows() {
        let (af, rel) = absorbed_fraction(row.radius, row.energy, 500_000).unwrap();
        let dev = 100.0 * (af - row.recommended) / row.recommended;
        devs.push((row.energy, row.mass, dev));
        eprintln!(
            "{:>6} g {:>6.0} keV  Guaraci {af:.4} ± {:.2} %  EGS4 {:.3}  MCNP {:.3}  rec {:.3}  {dev:+.1} %",
            row.mass,
            row.energy * 1e3,
            rel * 100.0,
            row.egs4,
            row.mcnp,
            row.recommended
        );
        out.push_str(&format!(
            "{},{},{af:.5},{rel:.5},{},{},{},{dev:.2}\n",
            row.mass, row.energy, row.egs4, row.mcnp, row.recommended
        ));
    }
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../validation/stabin2000_comparison.csv"
    );
    std::fs::write(path, out).unwrap();
    let mean = devs.iter().map(|d| d.2).sum::<f64>() / devs.len() as f64;
    let max = devs.iter().map(|d| d.2.abs()).fold(0.0, f64::max);
    eprintln!(
        "{} cases: mean deviation {mean:+.2} %, largest |deviation| {max:.1} %",
        devs.len()
    );
}

/// Electron stopping power and CSDA range in liquid water against NIST ESTAR.
#[test]
fn electron_stopping_and_range_match_estar() {
    if electron_data().is_none() {
        return;
    }
    let text = "[physics]\nelectron_cut = 1 keV\n[world]\nmaterial = Water\nsize = 30\n\
                [source]\nenergy = 12 MeV\nposition = [0, 0, 0]\n";
    let Some(p) = problem(text) else { return };
    let k = p.materials.iter().position(|m| m.name == "Water").unwrap();
    let (m, rho) = (&p.emats[k], p.materials[k].density);
    // total stopping power (MeV cm²/g) and CSDA range (g/cm²), ESTAR
    for (e, s_estar, r_estar) in [
        (0.01, 22.56, None),
        (0.1, 4.121, Some(1.431e-2)),
        (1.0, 1.862, Some(4.367e-1)),
        (10.0, 2.151, Some(4.975)),
    ] {
        let s = m.total_stopping(&p.egrid, e, false) / rho;
        let ds = 100.0 * (s - s_estar) / s_estar;
        eprintln!("water {e:>5} MeV  S {s:.4} (ESTAR {s_estar}, {ds:+.2} %)");
        assert!(ds.abs() < 2.0, "stopping power at {e} MeV: {s} vs {s_estar}");
        if let Some(r_estar) = r_estar {
            let r = m.csda_range(&p.egrid, e) * rho;
            let dr = 100.0 * (r - r_estar) / r_estar;
            eprintln!("water {e:>5} MeV  R {r:.4e} (ESTAR {r_estar:.4e}, {dr:+.2} %)");
            assert!(dr.abs() < 2.0, "CSDA range at {e} MeV: {r} vs {r_estar}");
        }
    }
}

/// Radius (cm) of a unit-density sphere of `mass` grams. The electron table
/// rounds radii to 0.01 cm, too coarse for the smallest spheres.
fn unit_density_radius(mass: f64) -> f64 {
    (3.0 * mass / (4.0 * std::f64::consts::PI)).cbrt()
}

/// Electron absorbed fractions, Stabin & Konijnenberg (2000), Table 1: a few spheres and energies.
#[test]
fn stabin_electron_spheres_selected() {
    if data().is_none() || electron_data().is_none() {
        return;
    }
    let rows = stabin_electron_rows();
    for (mass, energy) in [(0.01, 0.4), (0.1, 1.0), (1.0, 2.0), (10.0, 4.0), (0.01, 2.0)] {
        let row = rows.iter().find(|r| r.mass == mass && r.energy == energy).unwrap();
        let (af, rel) = absorbed_fraction_of("electron", unit_density_radius(mass), energy, 40_000).unwrap();
        let dev = (af - row.recommended) / row.recommended;
        // EGS4 and MCNP differ by up to ~10 % in the smallest spheres (paper)
        let spread = (row.egs4 - row.mcnp).abs() / row.recommended;
        let tol = 0.03 + spread + 3.0 * rel + 0.0005 / row.recommended;
        eprintln!(
            "{mass:>6} g {:>6.0} keV  Guaraci {af:.4} ± {:.2} %  ref {:.3} (EGS4 {:.3}, MCNP {:.3})  {:+.1} %",
            energy * 1e3,
            rel * 100.0,
            row.recommended,
            row.egs4,
            row.mcnp,
            dev * 100.0
        );
        assert!(
            dev.abs() < tol,
            "{mass} g, {energy} MeV: {af:.4} vs {} ({:+.1} %)",
            row.recommended,
            dev * 100.0
        );
    }
}

/// The whole electron table. Writes validation/stabin2000_electrons_comparison.csv.
#[test]
#[ignore]
fn stabin_electron_spheres_full() {
    if data().is_none() || electron_data().is_none() {
        return;
    }
    let mut out = String::from("mass_g,energy_MeV,guaraci,guaraci_rel_sigma,egs4,mcnp,recommended,deviation_percent\n");
    let mut devs = Vec::new();
    for row in stabin_electron_rows() {
        let (af, rel) = absorbed_fraction_of("electron", unit_density_radius(row.mass), row.energy, 100_000).unwrap();
        let dev = 100.0 * (af - row.recommended) / row.recommended;
        eprintln!(
            "{:>6} g {:>5} MeV  {af:.4}  ref {:.3}  {dev:+.1} %",
            row.mass, row.energy, row.recommended
        );
        out += &format!(
            "{},{},{af:.5},{rel:.5},{},{},{},{dev:.2}\n",
            row.mass, row.energy, row.egs4, row.mcnp, row.recommended
        );
        devs.push(dev);
    }
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../validation/stabin2000_electrons_comparison.csv"
    );
    std::fs::write(path, out).unwrap();
    let mean = devs.iter().sum::<f64>() / devs.len() as f64;
    let max = devs.iter().map(|d| d.abs()).fold(0.0, f64::max);
    eprintln!(
        "{} cases: mean deviation {mean:+.2} %, largest |deviation| {max:.1} %",
        devs.len()
    );
}
