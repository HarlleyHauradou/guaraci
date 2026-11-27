//! `guaraci run <input>`: build the problem, run it and write the results.

use crate::data::ensure;
use crate::{check, report, BUILTINS};
use guaraci_core::problem::{Normalization, Probe, Problem, MEV_TO_J};
use guaraci_cpu::{Options, Results, RngChoice};
use guaraci_data::library::{data_dir, load_decay, load_electron, load_photon, Kind, Provenance};
use guaraci_input::config::{Backend, Emission, RngKind};
use guaraci_input::{load, Config, Diagnostic};
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct RunArgs {
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub histories: Option<u64>,
    pub threads: Option<usize>,
    pub offline: bool,
}

pub fn run(args: RunArgs) -> Result<(), ()> {
    let prog = load(&args.input, BUILTINS).map_err(|(p, d)| report(&d, &p.sources))?;
    let mut config = Config::from_program(&prog).map_err(|d| report(&d, &prog.sources))?;
    report(&config.warnings, &prog.sources);
    if let Some(h) = args.histories {
        config.run.histories = h;
    }
    if let Some(t) = args.threads {
        config.run.threads = t;
    }
    if config.run.backend == Backend::Cuda {
        eprintln!("error: the CUDA backend is not available yet; use `backend = cpu`");
        return Err(());
    }
    let offline = args.offline || std::env::var_os("GUARACI_OFFLINE").is_some();

    // ---- data
    let lib = config.physics.data.clone();
    ensure(&lib, Kind::Photoatomic, offline)?;
    let needs_decay = config
        .sources
        .iter()
        .any(|s| matches!(s.emission, Emission::Nuclide { .. }));
    if needs_decay {
        ensure("endfb8", Kind::Decay, offline)?;
    }
    if config.physics.electrons {
        ensure("endfb8", Kind::Electron, offline)?;
    }
    let dir = data_dir();
    let (photon_prov, photon) = load_photon(&dir, &lib).map_err(|e| eprintln!("error: {e}"))?;
    let electron = if config.physics.electrons {
        Some(load_electron(&dir, "endfb8").map_err(|e| eprintln!("error: {e}"))?)
    } else {
        None
    };
    let decay = if needs_decay {
        Some(load_decay(&dir, "endfb8").map_err(|e| eprintln!("error: {e}"))?)
    } else {
        None
    };

    // ---- problem
    eprintln!("Preparing geometry and cross-section tables...");
    let problem = Problem::build(
        &config,
        &photon,
        decay.as_ref().map(|d| &d.1),
        electron.as_ref().map(|d| &d.1),
    )
    .map_err(|d: Vec<Diagnostic>| report(&d, &prog.sources))?;
    drop(photon);

    // ---- output directory
    let stem = args
        .input
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "guaraci".into());
    let out = args
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from(format!("{stem}_results")));
    std::fs::create_dir_all(&out).map_err(|e| eprintln!("error: cannot create {}: {e}", out.display()))?;

    // ---- run
    let opts = Options {
        histories: config.run.histories,
        batches: config.run.batches,
        seed: config.run.seed,
        rng: match config.run.rng {
            RngKind::Philox => RngChoice::Philox,
            RngKind::Xoshiro => RngChoice::Xoshiro,
            RngKind::Pcg => RngChoice::Pcg,
            RngKind::Mt19937 => RngChoice::Mt19937,
        },
        threads: config.run.threads,
    };
    let done = Arc::new(AtomicU64::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    {
        let stop = stop.clone();
        let _ = ctrlc::set_handler(move || {
            if stop.swap(true, Ordering::SeqCst) {
                std::process::exit(130);
            }
            eprintln!("\nStopping after the current batch (press Ctrl-C again to quit now)...");
        });
    }
    let finished = Arc::new(AtomicBool::new(false));
    let printer = {
        let (done, finished) = (done.clone(), finished.clone());
        let total = opts.histories;
        std::thread::spawn(move || progress(&done, &finished, total))
    };
    let results = guaraci_cpu::run(&problem, &opts, &done, &stop);
    finished.store(true, Ordering::SeqCst);
    let _ = printer.join();

    // ---- write
    let mut files = Vec::new();
    for (pi, probe) in problem.probes.iter().enumerate() {
        match write_probe(&out, &problem, &results, pi, probe) {
            Ok(mut f) => files.append(&mut f),
            Err(e) => {
                eprintln!("error: writing probe `{}`: {e}", probe.name());
                return Err(());
            }
        }
    }
    let input_copy = out.join("input");
    let _ = std::fs::create_dir_all(&input_copy);
    for f in prog.sources.files().iter().filter(|f| f.path.is_some()) {
        let name = Path::new(&f.display)
            .file_name()
            .map(|n| n.to_owned())
            .unwrap_or_default();
        let _ = std::fs::write(input_copy.join(name), f.text());
    }
    let text = report_text(
        &config,
        &prog,
        &problem,
        &results,
        &photon_prov,
        decay.as_ref().map(|d| &d.0),
        electron.as_ref().map(|d| &d.0),
        &files,
        &args.input,
    );
    let report_path = out.join("report.txt");
    std::fs::write(&report_path, &text).map_err(|e| eprintln!("error: {e}"))?;

    println!("{}", summary(&problem, &results));
    println!("Results in {}/ (report.txt, {})", out.display(), files.join(", "));
    if results.interrupted {
        println!(
            "Note: the run was stopped early; results use {} histories.",
            results.total.histories
        );
    }
    Ok(())
}

fn progress(done: &AtomicU64, finished: &AtomicBool, total: u64) {
    let start = Instant::now();
    let tty = std::io::IsTerminal::is_terminal(&std::io::stderr());
    let mut last_print = Instant::now() - Duration::from_secs(10);
    loop {
        let fin = finished.load(Ordering::SeqCst);
        let d = done.load(Ordering::Relaxed);
        let interval = if tty {
            Duration::from_millis(250)
        } else {
            Duration::from_secs(10)
        };
        if fin || last_print.elapsed() >= interval {
            last_print = Instant::now();
            let t = start.elapsed().as_secs_f64();
            let rate = d as f64 / t.max(1e-9);
            let frac = d as f64 / total.max(1) as f64;
            let eta = if d > 0 {
                (total - d.min(total)) as f64 / rate
            } else {
                f64::NAN
            };
            let width = 30;
            let filled = (frac * width as f64) as usize;
            let line = format!(
                "  [{}{}] {:>5.1}%  {:.3e} histories/s  elapsed {}  left {}",
                "=".repeat(filled.min(width)),
                " ".repeat(width - filled.min(width)),
                frac * 100.0,
                rate,
                clock(t),
                if eta.is_finite() { clock(eta) } else { "?".into() }
            );
            if tty {
                eprint!("\r{line}");
            } else {
                eprintln!("{line}");
            }
            let _ = std::io::stderr().flush();
        }
        if fin {
            if tty {
                eprintln!();
            }
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Relative uncertainty as a percentage with two significant digits.
fn pct(rel: f64) -> String {
    if !rel.is_finite() {
        return "–".into();
    }
    let p = rel * 100.0;
    if p == 0.0 {
        "0 %".into()
    } else if p < 0.1 {
        format!("{p:.1e} %")
    } else {
        format!("{p:.2} %")
    }
}

fn clock(s: f64) -> String {
    let s = s.round() as u64;
    if s >= 3600 {
        format!("{}h{:02}m", s / 3600, s % 3600 / 60)
    } else if s >= 60 {
        format!("{}m{:02}s", s / 60, s % 60)
    } else {
        format!("{s}s")
    }
}

/// Factor and unit name for doses.
fn dose_unit(p: &Problem) -> (f64, &'static str) {
    match p.normalization {
        Normalization::PerDecay => (1.0, "Gy per decay"),
        Normalization::PerSecond { activity } => (activity, "Gy/s"),
        Normalization::Total { decays } => (decays, "Gy"),
    }
}

fn write_probe(out: &Path, p: &Problem, r: &Results, pi: usize, probe: &Probe) -> std::io::Result<Vec<String>> {
    let (scale, unit) = dose_unit(p);
    match probe {
        Probe::DoseGrid {
            output,
            origin,
            spacing,
            dims,
            mass_kg,
            ..
        } => {
            let n = mass_kg.len();
            let mut dose = vec![0f32; n];
            let mut unc = vec![0f32; n];
            for i in 0..n {
                let (m, e) = r.mean_and_error(pi, i);
                if mass_kg[i] > 0.0 && m > 0.0 {
                    dose[i] = (m * MEV_TO_J / mass_kg[i] * scale) as f32;
                    unc[i] = (e / m) as f32;
                }
            }
            let base = output.trim_end_matches(".mhd");
            let unc_name = format!("{base}_uncertainty.mhd");
            write_mhd(
                &out.join(output),
                &dose,
                origin,
                spacing,
                dims,
                &format!("absorbed dose, {unit}"),
            )?;
            write_mhd(
                &out.join(&unc_name),
                &unc,
                origin,
                spacing,
                dims,
                "relative standard uncertainty (1 sigma) of the dose",
            )?;
            Ok(vec![output.clone(), unc_name])
        }
        Probe::VolumeDose {
            output,
            volumes,
            mass_kg,
            ..
        } => {
            let mut s = "volume,material,mass_g,energy_MeV_per_decay,dose,dose_unit,uncertainty_percent\n".to_string();
            for (k, &v) in volumes.iter().enumerate() {
                let (m, e) = r.mean_and_error(pi, k);
                let vol = &p.geometry.volumes[v];
                let dose = if mass_kg[k] > 0.0 {
                    m * MEV_TO_J / mass_kg[k] * scale
                } else {
                    0.0
                };
                let _ = writeln!(
                    s,
                    "{},{},{:.6e},{:.6e},{:.6e},{unit},{:.3}",
                    vol.name,
                    p.materials[vol.material].name,
                    mass_kg[k] * 1e3,
                    m,
                    dose,
                    if m > 0.0 { 100.0 * e / m } else { f64::NAN }
                );
            }
            std::fs::write(out.join(output), s)?;
            Ok(vec![output.clone()])
        }
        Probe::Spectrum {
            output,
            emax,
            bins,
            volume_cm3,
            ..
        } => {
            let (fscale, funit) = match p.normalization {
                Normalization::PerDecay => (1.0, "cm^-2 per decay"),
                Normalization::PerSecond { activity } => (activity, "cm^-2 s^-1"),
                Normalization::Total { decays } => (decays, "cm^-2"),
            };
            let mut s = "energy_low_keV,energy_high_keV,fluence,fluence_per_keV,unit,uncertainty_percent\n".to_string();
            let w = emax / *bins as f64 * 1e3;
            for b in 0..*bins {
                let (m, e) = r.mean_and_error(pi, b);
                let f = m / volume_cm3 * fscale;
                let _ = writeln!(
                    s,
                    "{:.4},{:.4},{:.6e},{:.6e},{funit},{:.3}",
                    b as f64 * w,
                    (b + 1) as f64 * w,
                    f,
                    f / w,
                    if m > 0.0 { 100.0 * e / m } else { f64::NAN }
                );
            }
            std::fs::write(out.join(output), s)?;
            Ok(vec![output.clone()])
        }
    }
}

fn write_mhd(
    path: &Path,
    data: &[f32],
    origin: &guaraci_core::vec3::Vec3,
    spacing: &guaraci_core::vec3::Vec3,
    dims: &[usize; 3],
    what: &str,
) -> std::io::Result<()> {
    let raw = path.with_extension("raw");
    let raw_name = raw.file_name().unwrap().to_string_lossy().to_string();
    // MetaImage uses mm and the centre of the first voxel as offset
    let off = *origin + *spacing * 0.5;
    let header = format!(
        "ObjectType = Image\nNDims = 3\nBinaryData = True\nBinaryDataByteOrderMSB = False\nCompressedData = False\n\
         TransformMatrix = 1 0 0 0 1 0 0 0 1\nOffset = {} {} {}\nCenterOfRotation = 0 0 0\nAnatomicalOrientation = RAI\n\
         ElementSpacing = {} {} {}\nDimSize = {} {} {}\nElementType = MET_FLOAT\nElementDataFile = {raw_name}\n",
        off.x * 10.0,
        off.y * 10.0,
        off.z * 10.0,
        spacing.x * 10.0,
        spacing.y * 10.0,
        spacing.z * 10.0,
        dims[0],
        dims[1],
        dims[2]
    );
    let _ = what;
    std::fs::write(path, header)?;
    let mut bytes = Vec::with_capacity(data.len() * 4);
    for v in data {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    std::fs::write(raw, bytes)
}

fn summary(p: &Problem, r: &Results) -> String {
    let mut o = String::new();
    let t = &r.total;
    let secs = r.elapsed.as_secs_f64();
    let _ = writeln!(
        o,
        "\nDone: {} histories in {} ({:.3e} histories/s, {} threads)",
        t.histories,
        clock(secs),
        t.histories as f64 / secs.max(1e-9),
        r.threads
    );
    let (scale, unit) = dose_unit(p);
    for (pi, probe) in p.probes.iter().enumerate() {
        match probe {
            Probe::VolumeDose {
                volumes, mass_kg, name, ..
            } => {
                let _ = writeln!(o, "  probe {name}:");
                for (k, &v) in volumes.iter().enumerate() {
                    let (m, e) = r.mean_and_error(pi, k);
                    let dose = if mass_kg[k] > 0.0 {
                        m * MEV_TO_J / mass_kg[k] * scale
                    } else {
                        0.0
                    };
                    let _ = writeln!(
                        o,
                        "    {:<16} {:.4e} {unit}  ± {}",
                        p.geometry.volumes[v].name,
                        dose,
                        pct(if m > 0.0 { e / m } else { f64::NAN })
                    );
                }
            }
            Probe::DoseGrid { name, mass_kg, .. } => {
                // the maximum of noisy voxels is meaningless: only voxels known to 10 % or better
                let mut best: Option<(f64, f64, usize)> = None;
                let mut reliable = 0usize;
                for (i, &mk) in mass_kg.iter().enumerate() {
                    let (m, e) = r.mean_and_error(pi, i);
                    let rel = e / m;
                    if mk <= 0.0 || m <= 0.0 || rel.is_nan() || rel > 0.10 {
                        continue;
                    }
                    reliable += 1;
                    let d = m / mk;
                    if best.is_none_or(|b| d > b.0) {
                        best = Some((d, e / m, i));
                    }
                }
                match best {
                    Some((d, rel, _)) => {
                        let _ = writeln!(
                            o,
                            "  probe {name}: maximum dose {:.4e} {unit} ± {} (among the {reliable} voxels with uncertainty ≤ 10 %)",
                            d * MEV_TO_J * scale,
                            pct(rel)
                        );
                    }
                    None => {
                        let _ = writeln!(
                            o,
                            "  probe {name}: no voxel has an uncertainty below 10 %; run more histories"
                        );
                    }
                }
            }
            Probe::Spectrum { name, volume_cm3, .. } => {
                let total: f64 = (0..probe.cells()).map(|b| r.mean_and_error(pi, b).0).sum();
                let _ = writeln!(
                    o,
                    "  probe {name}: total photon fluence {:.4e} cm^-2 per decay",
                    total / volume_cm3
                );
            }
        }
    }
    o
}

#[allow(clippy::too_many_arguments)]
fn report_text(
    c: &Config,
    prog: &guaraci_input::Program,
    p: &Problem,
    r: &Results,
    photon: &Provenance,
    decay: Option<&Provenance>,
    electron: Option<&Provenance>,
    files: &[String],
    input: &Path,
) -> String {
    let mut o = String::new();
    let line = "=".repeat(78);
    let _ = writeln!(
        o,
        "{line}\nGuaraci {} — simulation report\n{line}",
        guaraci_core::VERSION_LINE
    );
    let _ = writeln!(o, "Input       {}", input.display());
    let _ = writeln!(
        o,
        "Files read  {}",
        prog.sources
            .files()
            .iter()
            .filter(|f| f.path.is_some())
            .map(|f| f.display.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    let _ = writeln!(o, "A copy of every input file is in input/.\n");

    let _ = writeln!(o, "{}", check::describe(c, &prog.sources));

    let _ = writeln!(o, "Data");
    let _ = writeln!(o, "  photon interactions  {} — {}", photon.title, photon.source);
    let _ = writeln!(
        o,
        "                       sha256 {}, installed {}",
        photon.sha256, photon.installed
    );
    if let Some(d) = decay {
        let _ = writeln!(o, "  radioactive decay    {} — {}", d.title, d.source);
        let _ = writeln!(
            o,
            "                       sha256 {}, installed {}",
            d.sha256, d.installed
        );
    }
    if let Some(e) = electron {
        let _ = writeln!(o, "  electrons            {} — {}", e.title, e.source);
        let _ = writeln!(
            o,
            "                       sha256 {}, installed {}",
            e.sha256, e.installed
        );
        let _ = writeln!(
            o,
            "  mean excitation      ICRU Report 37 (Bragg rule, condensed-phase values)"
        );
    }
    let _ = writeln!(
        o,
        "  materials            PNNL-15870 Rev. 1 (built-in database) and user definitions"
    );

    let _ = writeln!(o, "\nNormalisation");
    let _ = writeln!(
        o,
        "  {}",
        match p.normalization {
            Normalization::PerDecay => "Results are per decay (no activity given).".to_string(),
            Normalization::PerSecond { activity } =>
                format!("Results are per second for a total activity of {activity:.6e} Bq (dose rates)."),
            Normalization::Total { decays } =>
                format!("Results are totals for {decays:.6e} decays (activity × duration)."),
        }
    );

    let t = &r.total;
    let n = t.histories.max(1) as f64;
    let _ = writeln!(o, "\nEnergy balance (per decay)");
    let _ = writeln!(o, "  emitted     {:.6e} MeV", t.emitted / n);
    let _ = writeln!(
        o,
        "  deposited   {:.6e} MeV  ({:.3} %)",
        t.deposited / n,
        100.0 * t.deposited / t.emitted.max(1e-300)
    );
    let _ = writeln!(
        o,
        "  escaped     {:.6e} MeV  ({:.3} %)",
        t.escaped / n,
        100.0 * t.escaped / t.emitted.max(1e-300)
    );
    let _ = writeln!(
        o,
        "  difference  {:.3e} %  (should be ~0)",
        100.0 * (t.emitted - t.deposited - t.escaped) / t.emitted.max(1e-300)
    );

    let _ = writeln!(o, "\nResults{}", summary(p, r));
    let _ = writeln!(o, "Files: {}", files.join(", "));
    let _ = writeln!(
        o,
        "  .mhd/.raw images open in 3D Slicer, ImageJ/Fiji and Amide (spacing and offset in mm)."
    );
    let _ = writeln!(
        o,
        "  Uncertainties are 1 standard deviation of the mean, from {} batches.",
        r.batches
    );

    let _ = writeln!(o, "\nRun");
    let _ = writeln!(o, "  photons tracked  {}", t.photons);
    let _ = writeln!(
        o,
        "  time             {:.2} s on {} threads",
        r.elapsed.as_secs_f64(),
        r.threads
    );
    let _ = writeln!(
        o,
        "  speed            {:.4e} histories/s",
        n / r.elapsed.as_secs_f64().max(1e-9)
    );
    if r.interrupted {
        let _ = writeln!(o, "  NOTE             stopped early by the user");
    }
    let _ = writeln!(o, "\nLimitations of this version");
    if p.electrons {
        let _ = writeln!(
            o,
            "  - Electrons below the cut-off deposit their energy on the spot; no energy-loss straggling below"
        );
        let _ = writeln!(
            o,
            "    the δ-ray and bremsstrahlung thresholds; positrons do not annihilate in flight."
        );
        let _ = writeln!(
            o,
            "  - Auger and conversion electrons are emitted with the decay, but not after photoelectric absorption."
        );
    } else {
        let _ = writeln!(o, "  - Electrons (β, Auger, conversion, Compton and photo-electrons) deposit their energy at the point where they are created.");
    }
    let _ = writeln!(
        o,
        "  - No fluorescence after photoelectric absorption; no Doppler broadening in Compton scattering."
    );
    let _ = writeln!(o, "  - The dose in a voxel cut by a boundary is the mean over the materials it contains (mass from 27 points per voxel).");
    o
}
