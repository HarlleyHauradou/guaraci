//! Physics benchmarks against reference values.
//!
//! They need installed data: set `GUARACI_TEST_DATA` to a Guaraci data
//! directory where `guaraci data install endfb8` was run. Without it the tests
//! are skipped.

use guaraci_core::physics::{Grid, Material};
use guaraci_core::problem::{Probe, Problem};
use guaraci_cpu::{run, Options, RngChoice};
use guaraci_data::library::{load_decay, load_photon, DecayData, PhotonData};
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

fn problem(text: &str) -> Option<Problem> {
    let (photon, decay) = data()?;
    let prog = load_str("bench.gci", text, &[MATERIALS])
        .map_err(|(_, d)| d)
        .expect("input");
    let c = Config::from_program(&prog).expect("config");
    Some(Problem::build(&c, photon, Some(decay)).expect("problem"))
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
