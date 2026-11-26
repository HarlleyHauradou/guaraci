//! The Guaraci input language.
//!
//! A simulation is described by one or more `.gci` files. This crate reads
//! them (following `include` statements), reports every problem it finds with
//! a pointer into the text, and produces a typed [`config::Config`].

#![allow(clippy::result_large_err)]

pub mod ast;
pub mod config;
pub mod diag;
pub mod parser;
pub mod reader;
pub mod source;

pub use ast::{Section, Span, Spanned};
pub use config::Config;
pub use diag::{Diagnostic, Severity};
pub use source::{SourceMap, EXTENSION};

use ast::{Item, Section as Sec};
use std::path::{Path, PathBuf};

/// Built-in input text shipped with Guaraci (e.g. the default material database).
pub struct Builtin {
    pub name: &'static str,
    pub text: &'static str,
}

/// Every section of a simulation, in reading order, with the files they came from.
#[derive(Debug, Default)]
pub struct Program {
    pub sources: SourceMap,
    pub sections: Vec<Sec>,
    /// Number of files that are built-in (they come first in `sources`).
    pub builtin_files: usize,
}

impl Program {
    pub fn is_builtin(&self, span: Span) -> bool {
        span.file < self.builtin_files
    }
}

/// Read `path` (the `.gci` extension is optional) and everything it includes.
pub fn load(path: &Path, builtins: &[Builtin]) -> Result<Program, (Program, Vec<Diagnostic>)> {
    let mut prog = Program::default();
    let mut diags = Vec::new();
    for b in builtins {
        let id = prog
            .sources
            .add(format!("<built-in {}>", b.name), None, b.text.to_string());
        match parser::parse(id, b.text) {
            Ok(f) => prog
                .sections
                .extend(f.items.into_iter().filter_map(|i| match i {
                    Item::Section(s) => Some(s),
                    Item::Include(_) => None,
                })),
            Err(d) => diags.push(d),
        }
    }
    prog.builtin_files = prog.sources.files().len();

    let base = path.parent().unwrap_or(Path::new(".")).to_path_buf();
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    match source::resolve(&base, &name) {
        Ok(p) => {
            let mut stack = Vec::new();
            load_file(
                &mut prog,
                &mut diags,
                &p,
                path.display().to_string(),
                &mut stack,
            );
        }
        Err(tried) => diags.push(
            Diagnostic::error(format!("cannot find input file `{}`", path.display())).help(
                format!(
                    "looked for: {}",
                    tried
                        .iter()
                        .map(|t| t.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ),
        ),
    }
    if diags.iter().any(|d| d.is_error()) {
        Err((prog, diags))
    } else {
        Ok(prog)
    }
}

/// Parse text that does not come from disk (tests, GUI editor buffers).
pub fn load_str(
    name: &str,
    text: &str,
    builtins: &[Builtin],
) -> Result<Program, (Program, Vec<Diagnostic>)> {
    let mut prog = Program::default();
    let mut diags = Vec::new();
    for (n, t) in builtins
        .iter()
        .map(|b| (format!("<built-in {}>", b.name), b.text))
        .chain([(name.to_string(), text)])
    {
        let id = prog.sources.add(n, None, t.to_string());
        match parser::parse(id, t) {
            Ok(f) => {
                for item in f.items {
                    match item {
                        Item::Section(s) => prog.sections.push(s),
                        Item::Include(inc) => diags.push(
                            Diagnostic::error("`include` is not available here")
                                .at(inc.target.span)
                                .help("save the file to disk so included files can be found"),
                        ),
                    }
                }
            }
            Err(d) => diags.push(d),
        }
    }
    prog.builtin_files = builtins.len();
    if diags.iter().any(|d| d.is_error()) {
        Err((prog, diags))
    } else {
        Ok(prog)
    }
}

fn load_file(
    prog: &mut Program,
    diags: &mut Vec<Diagnostic>,
    path: &PathBuf,
    display: String,
    stack: &mut Vec<PathBuf>,
) {
    if prog.sources.find_path(path).is_some() {
        return; // already read (diamond include): read each file once
    }
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            diags.push(Diagnostic::error(format!("cannot read `{display}`: {e}")));
            return;
        }
    };
    let id = prog.sources.add(display, Some(path.clone()), text);
    let file = match parser::parse(id, prog.sources.get(id).text()) {
        Ok(f) => f,
        Err(d) => {
            diags.push(d);
            return;
        }
    };
    stack.push(path.clone());
    let dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
    for item in file.items {
        match item {
            Item::Section(s) => prog.sections.push(s),
            Item::Include(inc) => {
                let t = &inc.target;
                match source::resolve(&dir, &t.node) {
                    Ok(p) if stack.contains(&p) => diags.push(
                        Diagnostic::error(format!(
                            "`{}` includes itself (directly or through other files)",
                            t.node
                        ))
                        .at(t.span)
                        .label("include cycle")
                        .help(format!(
                            "chain: {}",
                            stack
                                .iter()
                                .chain([&p])
                                .map(|p| p.display().to_string())
                                .collect::<Vec<_>>()
                                .join(" -> ")
                        )),
                    ),
                    Ok(p) => {
                        let shown = p
                            .strip_prefix(std::env::current_dir().unwrap_or_default())
                            .unwrap_or(&p)
                            .display()
                            .to_string();
                        load_file(prog, diags, &p, shown, stack)
                    }
                    Err(tried) => diags.push(
                        Diagnostic::error(format!("cannot find included file `{}`", t.node))
                            .at(t.span)
                            .label("not found")
                            .help(format!(
                                "looked for: {}",
                                tried
                                    .iter()
                                    .map(|p| p.display().to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )),
                    ),
                }
            }
        }
    }
    stack.pop();
}

#[cfg(test)]
mod tests;
