//! `guaraci info ...`

use crate::{report, BUILTINS};
use clap::Subcommand;
use guaraci_core::electron::BetaSpectrum;
use guaraci_data::decay::BetaKind;
use guaraci_data::library::{data_dir, load_decay, Kind};
use guaraci_data::NuclideId;
use guaraci_input::config::Composition;

#[derive(Subcommand)]
pub enum InfoCommand {
    /// Every unit Guaraci understands.
    Units,
    /// Built-in materials; give a word to filter (e.g. `tissue`, `bone`, `LYSO`).
    Materials { filter: Option<String> },
    /// Decay data of a nuclide (e.g. `I-131`, `Tc-99m`).
    Nuclide {
        name: String,
        /// Print the photon lines as CSV (energy_keV,photons_per_decay,kind),
        /// e.g. to build the source of another Monte Carlo code.
        #[arg(long)]
        csv: bool,
        /// Print the β⁻ spectrum (all branches together) as a CSV histogram
        /// (E_low_MeV,E_high_MeV,probability), from Fermi theory as used in transport.
        #[arg(long, conflicts_with = "csv")]
        beta_csv: bool,
        /// Number of bins of the β histogram.
        #[arg(long, default_value_t = 200, requires = "beta_csv")]
        bins: usize,
    },
}

pub fn run(cmd: InfoCommand) -> Result<(), ()> {
    match cmd {
        InfoCommand::Units => {
            for k in guaraci_units::ALL_KINDS {
                let units: Vec<&str> = guaraci_units::UNITS
                    .iter()
                    .filter(|u| u.dim == k.dim())
                    .map(|u| u.symbol)
                    .collect();
                if units.is_empty() {
                    continue;
                }
                let def = if k.default_unit().is_empty() {
                    String::new()
                } else {
                    format!("  (default {})", k.default_unit())
                };
                println!("{:<14} {}{def}", k.name(), units.join(" "));
            }
            println!("\nCompound units: g/cm3, g/cm^3, kg/m3, MBq*h, MBq.s ...");
            Ok(())
        }
        InfoCommand::Materials { filter } => {
            let text = "[world]\nmaterial = Air\nsize = 1\n[source]\nenergy = 1\nposition = [0,0,0]\n";
            let prog = guaraci_input::load_str("<info>", text, BUILTINS).map_err(|(p, d)| report(&d, &p.sources))?;
            let c = guaraci_input::Config::from_program(&prog).map_err(|d| report(&d, &prog.sources))?;
            let f = filter.map(|s| s.to_lowercase());
            let mut n = 0;
            for m in &c.materials {
                let hay = format!("{} {}", m.name.node, m.title.clone().unwrap_or_default()).to_lowercase();
                if f.as_ref().is_some_and(|f| !hay.contains(f.as_str())) {
                    continue;
                }
                n += 1;
                let elems = match &m.composition {
                    Composition::MassFractions(v) => v.iter().map(|e| e.0.node.as_str()).collect::<Vec<_>>().join(" "),
                    Composition::Formula(f) => f.node.clone(),
                };
                let alias = m.copied_from.as_ref().map(|t| format!(" = {t}")).unwrap_or_default();
                println!(
                    "{:<36} {:>10} g/cm3  {:<40} {elems}",
                    format!("{}{alias}", m.name.node),
                    format!("{:.6}", m.density).trim_end_matches('0').trim_end_matches('.'),
                    m.title.clone().unwrap_or_default()
                );
            }
            println!("\n{n} materials");
            Ok(())
        }
        InfoCommand::Nuclide {
            name,
            csv,
            beta_csv,
            bins,
        } => {
            let Some(id) = NuclideId::parse(&name) else {
                eprintln!("error: `{name}` is not a nuclide name (e.g. I-131, Tc-99m)");
                return Err(());
            };
            // first use: download the decay data, as `run` does
            crate::data::ensure("endfb8", Kind::Decay, crate::data::offline_env())?;
            let (prov, data) = match load_decay(&data_dir(), "endfb8") {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("error: {e}");
                    return Err(());
                }
            };
            let Some(n) = data.get(&id) else {
                eprintln!("error: {id} is not in {}", prov.title);
                return Err(());
            };
            let spectra: Vec<(f64, bool, BetaSpectrum)> = n
                .betas
                .iter()
                .map(|b| {
                    let positron = b.kind == BetaKind::Plus;
                    // charge of the daughter nucleus
                    let zd = if positron { id.z.saturating_sub(1) } else { id.z + 1 };
                    (
                        b.intensity,
                        positron,
                        BetaSpectrum::new(b.endpoint, zd, positron, b.forbiddenness),
                    )
                })
                .collect();
            if beta_csv {
                let minus: Vec<&(f64, bool, BetaSpectrum)> = spectra.iter().filter(|s| !s.1).collect();
                let total: f64 = minus.iter().map(|s| s.0).sum();
                if minus.is_empty() || total <= 0.0 {
                    eprintln!("error: {id} has no β⁻ branches");
                    return Err(());
                }
                let emax = minus.iter().map(|s| s.2.endpoint()).fold(0.0, f64::max);
                let mean = minus.iter().map(|s| s.0 * s.2.mean).sum::<f64>() / total;
                let plus: f64 = spectra.iter().filter(|s| s.1).map(|s| s.0).sum::<f64>() + 0.0;
                println!(
                    "# {id} beta-minus spectrum, {} ({}, sha256 {})",
                    prov.title, prov.source, prov.sha256
                );
                println!(
                    "# beta_minus_per_decay = {total:.6e}; mean_energy_MeV = {mean:.6}; branches = {}; beta_plus_per_decay = {plus:.3e}",
                    minus.len()
                );
                println!("E_low_MeV,E_high_MeV,probability");
                let bins = bins.max(1);
                for k in 0..bins {
                    let (lo, hi) = (emax * k as f64 / bins as f64, emax * (k + 1) as f64 / bins as f64);
                    let p: f64 = minus
                        .iter()
                        .map(|s| s.0 * (s.2.cdf_at(hi) - s.2.cdf_at(lo)))
                        .sum::<f64>()
                        / total;
                    println!("{lo:.6},{hi:.6},{p:.6e}");
                }
                return Ok(());
            }
            if csv {
                println!(
                    "# {id} photon lines, {} ({}, sha256 {})",
                    prov.title, prov.source, prov.sha256
                );
                println!("energy_keV,photons_per_decay,kind");
                for l in &n.photons {
                    println!("{:.4},{:.6e},{:?}", l.energy * 1e3, l.intensity, l.kind);
                }
                return Ok(());
            }
            println!("{id}   ({})", prov.title);
            println!(
                "  half-life                {}",
                if n.is_stable() {
                    "stable".into()
                } else {
                    format!("{:.6e} s", n.half_life)
                }
            );
            println!(
                "  mean energy per decay    electrons {:.4} MeV, photons {:.4} MeV, heavy particles {:.4} MeV",
                n.mean_electron_energy, n.mean_photon_energy, n.mean_heavy_energy
            );
            println!("  photon lines             {}", n.photons.len());
            for l in &n.photons {
                println!(
                    "    {:>10.3} keV  {:>9.5} %  {:?}",
                    l.energy * 1e3,
                    l.intensity * 100.0,
                    l.kind
                );
            }
            if !n.betas.is_empty() {
                println!("  β branches               {}", n.betas.len());
                for (intensity, positron, s) in &spectra {
                    println!(
                        "    {}  endpoint {:>9.4} keV  mean {:>9.4} keV  {:>9.5} %",
                        if *positron { "β⁺" } else { "β⁻" },
                        s.endpoint() * 1e3,
                        s.mean * 1e3,
                        intensity * 100.0
                    );
                }
            }
            if !n.electrons.is_empty() {
                let e: f64 = n.electrons.iter().map(|l| l.energy * l.intensity).sum();
                println!(
                    "  conversion/Auger lines   {} ({:.4} keV per decay)",
                    n.electrons.len(),
                    e * 1e3
                );
            }
            if n.has_continuous_photons {
                println!("  note: the file also has a continuous photon spectrum, not used yet");
            }
            Ok(())
        }
    }
}
