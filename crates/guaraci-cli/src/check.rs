//! `guaraci check <input>`: validate and describe a simulation.

use crate::{report, BUILTINS};
use guaraci_data::library::{data_dir, installed, library_info, load_decay, Kind};
use guaraci_input::config::{Composition, Emission, Placement, ProbeKind, Shape};
use guaraci_input::{load, Config};
use std::fmt::Write as _;
use std::path::Path;

pub fn run(input: &Path) -> Result<(), ()> {
    let prog = match load(input, BUILTINS) {
        Ok(p) => p,
        Err((prog, diags)) => {
            report(&diags, &prog.sources);
            return Err(());
        }
    };
    let config = match Config::from_program(&prog) {
        Ok(c) => c,
        Err(diags) => {
            report(&diags, &prog.sources);
            return Err(());
        }
    };
    report(&config.warnings, &prog.sources);
    print!("{}", describe(&config, &prog.sources));
    println!("\nInput is valid.");
    Ok(())
}

fn fmt_len(cm: f64) -> String {
    format!("{} cm", trim(cm))
}

fn trim(x: f64) -> String {
    let s = format!("{x:.6}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" {
        "0".into()
    } else {
        s.into()
    }
}

fn v3(v: [f64; 3]) -> String {
    format!("[{}, {}, {}] cm", trim(v[0]), trim(v[1]), trim(v[2]))
}

fn shape(s: &Shape) -> String {
    match s {
        Shape::Box { size } => format!("box {}", v3(*size)),
        Shape::Sphere { radius } => format!("sphere, radius {}", fmt_len(*radius)),
        Shape::Cylinder { radius, height } => {
            format!("cylinder, radius {}, height {}", fmt_len(*radius), fmt_len(*height))
        }
        Shape::Ellipsoid { semi_axes } => format!("ellipsoid, semi-axes {}", v3(*semi_axes)),
    }
}

/// Number with `n` significant digits, without trailing zeros.
fn sig(x: f64, n: usize) -> String {
    if x == 0.0 {
        return "0".into();
    }
    let mag = x.abs().log10().floor() as i32;
    if !(-4..6).contains(&mag) {
        return format!("{:.*e}", n.saturating_sub(1), x);
    }
    let decimals = (n as i32 - 1 - mag).max(0) as usize;
    trim_str(&format!("{x:.decimals$}"))
}

fn trim_str(s: &str) -> String {
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s.to_string()
    }
}

fn energy(mev: f64) -> String {
    if mev < 1.0 {
        format!("{} keV", sig(mev * 1e3, 6))
    } else {
        format!("{} MeV", sig(mev, 6))
    }
}

fn sig_energy(mev: f64) -> String {
    if mev < 1.0 {
        format!("{} keV", sig(mev * 1e3, 4))
    } else {
        format!("{} MeV", sig(mev, 4))
    }
}

fn activity(bq: f64) -> String {
    for (f, u) in [(1e12, "TBq"), (1e9, "GBq"), (1e6, "MBq"), (1e3, "kBq")] {
        if bq >= f {
            return format!("{} {u}", trim(bq / f));
        }
    }
    format!("{} Bq", trim(bq))
}

fn half_life(s: f64) -> String {
    if s.is_infinite() {
        return "stable".into();
    }
    for (f, u) in [(365.25 * 86400.0, "y"), (86400.0, "d"), (3600.0, "h"), (60.0, "min")] {
        if s >= f {
            return format!("{:.4} {u}", s / f)
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();
        }
    }
    format!("{} s", trim(s))
}

pub fn describe(c: &Config, sources: &guaraci_input::SourceMap) -> String {
    let mut o = String::new();
    let r = &c.run;
    let _ = writeln!(o, "Run");
    let _ = writeln!(o, "  histories  {}", r.histories);
    let _ = writeln!(o, "  batches    {}", r.batches);
    let _ = writeln!(o, "  random     {} (seed {})", r.rng.name(), r.seed);
    let backend = match r.backend {
        guaraci_input::config::Backend::Cpu => "CPU",
        guaraci_input::config::Backend::Cuda => "CUDA (GPU)",
    };
    let _ = writeln!(
        o,
        "  backend    {backend}{}",
        if r.threads == 0 {
            ", all CPU cores".to_string()
        } else {
            format!(", {} threads", r.threads)
        }
    );

    let p = &c.physics;
    let _ = writeln!(o, "\nPhysics  (list {})", p.list);
    let dir = data_dir();
    let lib = library_info(&p.data);
    let title = lib.map(|l| l.title).unwrap_or("?");
    let state = |kind| match installed(&dir, &p.data, kind) {
        Some(_) => "installed".to_string(),
        None => match lib.and_then(|l| l.components.iter().find(|c| c.kind == kind)) {
            Some(comp) if comp.url.is_some() => "will be downloaded on the first run".to_string(),
            Some(comp) => format!(
                "NOT INSTALLED — download it from {} and run `guaraci data install {} --from <file>`",
                comp.page, p.data
            ),
            None => "not available in this library".to_string(),
        },
    };
    let _ = writeln!(
        o,
        "  data           {title}, photon interactions: {}",
        state(Kind::Photoatomic)
    );
    let onoff = |b: bool| if b { "on" } else { "off" };
    let _ = writeln!(o, "  photoelectric  {}", onoff(p.photoelectric));
    let compton = match p.compton {
        Some(guaraci_input::config::ComptonModel::KleinNishinaSf) => {
            "on, Klein-Nishina with incoherent scattering function S(x,Z)"
        }
        Some(guaraci_input::config::ComptonModel::KleinNishina) => "on, Klein-Nishina (free electron at rest)",
        None => "off",
    };
    let _ = writeln!(o, "  compton        {compton}");
    let _ = writeln!(o, "  rayleigh       {}", onoff(p.rayleigh));
    let _ = writeln!(o, "  pair           {}", onoff(p.pair));
    let _ = writeln!(o, "  photon cut     {}", energy(p.photon_cut));
    let _ = writeln!(
        o,
        "  electrons      deposited where they are created (not transported in this version)"
    );

    let _ = writeln!(o, "\nGeometry");
    let _ = writeln!(o, "  world  box {}  {}", v3(c.world.size), c.world.material.node);
    fn tree(c: &Config, parent: Option<&str>, depth: usize, o: &mut String) {
        for v in c
            .volumes
            .iter()
            .filter(|v| v.parent.as_ref().map(|p| p.node.as_str()).filter(|p| *p != "world") == parent)
        {
            let _ = writeln!(
                o,
                "  {}{}  {}  {}  at {}",
                "  ".repeat(depth + 1),
                v.name.node,
                shape(&v.shape),
                v.material.node,
                v3(v.center)
            );
            tree(c, Some(&v.name.node), depth + 1, o);
        }
    }
    tree(c, None, 0, &mut o);

    let used: std::collections::BTreeSet<&str> = std::iter::once(c.world.material.node.as_str())
        .chain(c.volumes.iter().map(|v| v.material.node.as_str()))
        .collect();
    let _ = writeln!(o, "\nMaterials");
    for name in used {
        let Some(m) = c.material(name) else { continue };
        let comp = match &m.composition {
            Composition::Formula(f) => format!("formula {}", f.node),
            Composition::MassFractions(v) => {
                let total: f64 = v.iter().map(|x| x.1).sum();
                v.iter()
                    .map(|(e, w)| format!("{} {}", e.node, sig(w / total, 4)))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        };
        let from = m
            .copied_from
            .as_ref()
            .map(|f| format!(" (copy of {f})"))
            .unwrap_or_default();
        let _ = writeln!(o, "  {name}{from}: {} g/cm3", trim(m.density));
        if let Some(t) = &m.title {
            let _ = writeln!(
                o,
                "      {t}{}",
                m.number
                    .map(|n| format!(", PNNL-15870 Rev. 1 no. {n}"))
                    .unwrap_or_default()
            );
        }
        let _ = writeln!(o, "      {comp}");
    }

    let _ = writeln!(o, "\nSources");
    let decay = c
        .sources
        .iter()
        .any(|s| matches!(s.emission, Emission::Nuclide { .. }))
        .then(|| load_decay(&dir, "endfb8").ok())
        .flatten();
    for s in &c.sources {
        let place = match &s.placement {
            Placement::In(v) => format!("uniform in {}", v.node),
            Placement::Point(p) => format!("point at {}", v3(*p)),
        };
        let act = match (s.activity, s.duration) {
            (Some(a), Some(t)) => format!(", {} for {} s ({:.4e} decays)", activity(a), trim(t), a * t),
            (Some(a), None) => format!(", {}", activity(a)),
            _ => ", results per decay".into(),
        };
        match &s.emission {
            Emission::Nuclide { id, .. } => {
                let _ = writeln!(o, "  {}  {id}  {place}{act}", s.name);
                match &decay {
                    Some((prov, d)) => match d.get(id) {
                        Some(n) => {
                            let _ = writeln!(
                                o,
                                "      half-life {}; per decay: {} in photons, {} in electrons ({})",
                                half_life(n.half_life),
                                sig_energy(n.discrete_photon_energy()),
                                sig_energy(n.mean_electron_energy),
                                prov.title
                            );
                            for l in n.photons.iter().take(4) {
                                let _ = writeln!(
                                    o,
                                    "      {:>12}  {:>8.4} %  {:?}",
                                    energy(l.energy),
                                    l.intensity * 100.0,
                                    l.kind
                                );
                            }
                            if n.photons.len() > 4 {
                                let _ = writeln!(o, "      ... {} more lines", n.photons.len() - 4);
                            }
                            if n.photons.is_empty() {
                                let _ =
                                    writeln!(o, "      no photons: all energy is deposited locally in this version");
                            }
                        }
                        None => {
                            let _ = writeln!(o, "      WARNING: {id} is not in {}", prov.title);
                        }
                    },
                    None => {
                        let _ = writeln!(
                            o,
                            "      decay data (ENDF/B-VIII.0) will be downloaded on the first run"
                        );
                    }
                }
            }
            Emission::Lines(lines) => {
                let ls = lines
                    .iter()
                    .map(|(e, i)| format!("{} ({}/decay)", energy(*e), trim(*i)))
                    .collect::<Vec<_>>()
                    .join(", ");
                let _ = writeln!(o, "  {}  {ls}  {place}{act}", s.name);
            }
        }
    }

    let _ = writeln!(o, "\nProbes");
    for pr in &c.probes {
        let what = match &pr.kind {
            ProbeKind::DoseGrid { region, voxel } => format!("dose grid over {}, voxels {}", region.node, v3(*voxel)),
            ProbeKind::VolumeDose { volumes } => format!(
                "dose in {}",
                volumes.iter().map(|v| v.node.as_str()).collect::<Vec<_>>().join(", ")
            ),
            ProbeKind::Spectrum { region, emax, bins } => {
                format!("photon spectrum in {}, 0–{} in {bins} bins", region.node, energy(*emax))
            }
        };
        let _ = writeln!(o, "  {}  {what}  -> {}", pr.name.node, pr.output);
    }

    let user_defaults: Vec<_> = c.defaults.iter().collect();
    if !user_defaults.is_empty() {
        let _ = writeln!(o, "\nDefaults applied (not written in the input)");
        for d in user_defaults {
            let _ = writeln!(o, "  {} {} = {}", d.section, d.key, d.value);
        }
    }
    if !c.implicit_units.is_empty() {
        let _ = writeln!(o, "\nNumbers written without a unit");
        for u in &c.implicit_units {
            let _ = writeln!(
                o,
                "  {}  `{}` read as {}",
                sources.location(u.span),
                sources.slice(u.span),
                u.unit
            );
        }
    }
    o
}
