//! `guaraci info ...`

use crate::{report, BUILTINS};
use clap::Subcommand;
use guaraci_data::library::{data_dir, load_decay};
use guaraci_data::NuclideId;
use guaraci_input::config::Composition;

#[derive(Subcommand)]
pub enum InfoCommand {
    /// Every unit Guaraci understands.
    Units,
    /// Built-in materials; give a word to filter (e.g. `tissue`, `bone`, `LYSO`).
    Materials { filter: Option<String> },
    /// Decay data of a nuclide (e.g. `I-131`, `Tc-99m`).
    Nuclide { name: String },
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
        InfoCommand::Nuclide { name } => {
            let Some(id) = NuclideId::parse(&name) else {
                eprintln!("error: `{name}` is not a nuclide name (e.g. I-131, Tc-99m)");
                return Err(());
            };
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
            if n.has_continuous_photons {
                println!("  note: the file also has a continuous photon spectrum, not used yet");
            }
            Ok(())
        }
    }
}
