//! `guaraci` — command-line entry point.

mod check;
mod data;
mod info;
mod run;

use clap::{Parser, Subcommand};
use guaraci_input::{Builtin, Diagnostic, SourceMap};
use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::ExitCode;

/// Built-in input files shipped with Guaraci.
pub const BUILTINS: &[Builtin] = &[Builtin {
    name: "materials",
    text: include_str!("../../../data/materials.gci"),
}];

#[derive(Parser)]
#[command(
    name = "guaraci",
    version,
    about = "Guaraci: Monte Carlo radiation transport on CPU and GPU"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Read an input file and show the simulation it describes, without running it.
    Check {
        /// Input file (`.gci` extension optional).
        input: PathBuf,
    },
    /// Run a simulation.
    Run {
        /// Input file (`.gci` extension optional).
        input: PathBuf,
        /// Folder for the results (default: `<input>_results`).
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Override the number of histories, e.g. `--histories 1e6`.
        #[arg(long, value_parser = parse_count)]
        histories: Option<u64>,
        /// Number of CPU threads (default: all cores).
        #[arg(long)]
        threads: Option<usize>,
        /// Never download data; fail if something is missing.
        #[arg(long)]
        offline: bool,
    },
    /// Manage nuclear data libraries.
    Data {
        #[command(subcommand)]
        command: data::DataCommand,
    },
    /// Reference information: units, materials, nuclides.
    Info {
        #[command(subcommand)]
        command: info::InfoCommand,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Check { input } => check::run(&input),
        Command::Run {
            input,
            output,
            histories,
            threads,
            offline,
        } => run::run(run::RunArgs {
            input,
            output,
            histories,
            threads,
            offline,
        }),
        Command::Data { command } => data::run(command),
        Command::Info { command } => info::run(command),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => ExitCode::FAILURE,
    }
}

fn parse_count(s: &str) -> Result<u64, String> {
    let v: f64 = s
        .replace('_', "")
        .parse()
        .map_err(|_| format!("`{s}` is not a number"))?;
    if v < 1.0 || v.fract() != 0.0 {
        return Err(format!("`{s}` must be a whole number of at least 1"));
    }
    Ok(v as u64)
}

pub fn color() -> bool {
    std::io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none()
}

/// Print diagnostics to stderr, errors last so they stay on screen.
pub fn report(diags: &[Diagnostic], sources: &SourceMap) {
    let color = color();
    let mut sorted: Vec<&Diagnostic> = diags.iter().collect();
    sorted.sort_by_key(|d| d.is_error());
    for d in sorted {
        eprint!("{}", d.render(sources, color));
    }
    let errors = diags.iter().filter(|d| d.is_error()).count();
    if errors > 0 {
        eprintln!("{errors} error{} found", if errors == 1 { "" } else { "s" });
    }
}
