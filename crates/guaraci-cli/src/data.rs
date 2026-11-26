//! `guaraci data ...`

use clap::Subcommand;
use guaraci_data::library::{self, data_dir, installed, Kind, Progress, CATALOG};
use std::io::Write;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum DataCommand {
    /// Show the available libraries and which are installed.
    List,
    /// Download, verify and convert a library (e.g. `endfb8`).
    Install {
        /// Library id (see `guaraci data list`).
        library: String,
        /// Only this part: `photoatomic` or `decay`.
        #[arg(long)]
        kind: Option<String>,
        /// Use a file already downloaded (zip archive or directory of ENDF files).
        #[arg(long)]
        from: Option<PathBuf>,
        /// Reinstall even if already installed.
        #[arg(long)]
        force: bool,
    },
    /// Show where data is stored.
    Path,
}

pub struct Bar {
    last: u64,
}

impl Bar {
    pub fn new() -> Self {
        Bar { last: u64::MAX }
    }
}

impl Progress for Bar {
    fn message(&mut self, msg: &str) {
        if self.last != u64::MAX {
            eprintln!();
            self.last = u64::MAX;
        }
        eprintln!("  {msg}");
    }
    fn bytes(&mut self, done: u64, total: u64) {
        let pct = (done * 100).checked_div(total).unwrap_or(0);
        if pct != self.last {
            self.last = pct;
            eprint!(
                "\r  {:>6.1} MB of {:.1} MB ({pct}%)",
                done as f64 / 1e6,
                total as f64 / 1e6
            );
            let _ = std::io::stderr().flush();
        }
    }
}

fn parse_kind(s: &str) -> Option<Kind> {
    match s {
        "photoatomic" | "photon" => Some(Kind::Photoatomic),
        "decay" => Some(Kind::Decay),
        "electron" | "electrons" => Some(Kind::Electron),
        _ => None,
    }
}

pub fn run(cmd: DataCommand) -> Result<(), ()> {
    let dir = data_dir();
    match cmd {
        DataCommand::Path => {
            println!("{}", dir.display());
            Ok(())
        }
        DataCommand::List => {
            println!("Data directory: {}\n", dir.display());
            for lib in CATALOG {
                println!("{:<10} {}  ({})", lib.id, lib.title, lib.publisher);
                for c in lib.components {
                    let state = match installed(&dir, lib.id, c.kind) {
                        Some(p) => format!("installed {} ({} entries)", p.installed, p.entries),
                        None if c.url.is_some() => format!("not installed — {:.0} MB download", c.size as f64 / 1e6),
                        None => format!("not installed — manual download from {}", c.page),
                    };
                    println!("           {:<12} {state}", c.kind.name());
                }
            }
            Ok(())
        }
        DataCommand::Install {
            library: lib,
            kind,
            from,
            force,
        } => {
            let Some(info) = library::library_info(&lib) else {
                eprintln!("error: unknown library `{lib}`");
                eprintln!(
                    "  libraries: {}",
                    CATALOG.iter().map(|l| l.id).collect::<Vec<_>>().join(", ")
                );
                return Err(());
            };
            let kinds: Vec<Kind> = match kind.as_deref() {
                None => info.components.iter().map(|c| c.kind).collect(),
                Some(k) => match parse_kind(k) {
                    Some(k) => vec![k],
                    None => {
                        eprintln!("error: unknown kind `{k}` (use `photoatomic`, `decay` or `electron`)");
                        return Err(());
                    }
                },
            };
            if from.is_some() && kinds.len() > 1 {
                eprintln!(
                    "error: with --from, say which part the file contains: --kind photoatomic, decay or electron"
                );
                return Err(());
            }
            let mut ok = true;
            for k in kinds {
                if !force && installed(&dir, info.id, k).is_some() {
                    eprintln!(
                        "{} {} data is already installed (use --force to reinstall)",
                        info.title,
                        k.name()
                    );
                    continue;
                }
                eprintln!("{} — {} data", info.title, k.name());
                if let Err(e) = library::install(&dir, info.id, k, from.as_deref(), &mut Bar::new()) {
                    eprintln!("\nerror: {e}");
                    ok = false;
                }
            }
            if ok {
                Ok(())
            } else {
                Err(())
            }
        }
    }
}
