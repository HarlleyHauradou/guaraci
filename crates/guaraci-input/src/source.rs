//! Source files and include resolution.

use crate::ast::{FileId, Span};
use crate::diag::Named;
use miette::NamedSource;
use std::path::{Path, PathBuf};

/// Extension of Guaraci input files.
pub const EXTENSION: &str = "gci";

#[derive(Debug)]
pub struct SourceFile {
    /// Path as shown to the user.
    pub display: String,
    /// Canonical path on disk, `None` for built-in files.
    pub path: Option<PathBuf>,
    pub named: Named,
}

impl SourceFile {
    pub fn text(&self) -> &str {
        self.named.inner()
    }
}

/// All files read for one simulation.
#[derive(Debug, Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn add(&mut self, display: String, path: Option<PathBuf>, text: String) -> FileId {
        let named = NamedSource::new(display.clone(), text);
        self.files.push(SourceFile { display, path, named });
        self.files.len() - 1
    }
    pub fn get(&self, id: FileId) -> &SourceFile {
        &self.files[id]
    }
    pub(crate) fn named(&self, id: FileId) -> &Named {
        &self.files[id].named
    }
    pub fn files(&self) -> &[SourceFile] {
        &self.files
    }
    pub fn find_path(&self, path: &Path) -> Option<FileId> {
        self.files.iter().position(|f| f.path.as_deref() == Some(path))
    }
    /// `file:line:column` (1-based) of the start of a span.
    pub fn location(&self, span: Span) -> String {
        let f = &self.files[span.file];
        let before = &f.text()[..span.start.min(f.text().len())];
        let line = before.matches('\n').count() + 1;
        let col = before.rsplit('\n').next().map(|l| l.chars().count()).unwrap_or(0) + 1;
        format!("{}:{}:{}", f.display, line, col)
    }
    /// Text covered by a span.
    pub fn slice(&self, span: Span) -> &str {
        &self.files[span.file].text()[span.start..span.end]
    }
}

/// Resolve the file a user means by `name`, relative to `base_dir`.
///
/// `main`, `main.gci` and `sub/dir/main` all work; the `.gci` extension is
/// optional. Returns the candidates tried when nothing exists.
pub fn resolve(base_dir: &Path, name: &str) -> Result<PathBuf, Vec<PathBuf>> {
    let raw = base_dir.join(name);
    let mut tried = Vec::new();
    let with_ext = {
        let mut p = raw.clone().into_os_string();
        p.push(".");
        p.push(EXTENSION);
        PathBuf::from(p)
    };
    // Prefer `name.gci` when `name` has no extension, so that a directory or an
    // unrelated file called `materials` does not shadow `materials.gci`.
    let order = if raw.extension().is_some() {
        [raw.clone(), with_ext]
    } else {
        [with_ext, raw.clone()]
    };
    for cand in order {
        if cand.is_file() {
            return Ok(cand.canonicalize().unwrap_or(cand));
        }
        tried.push(cand);
    }
    Err(tried)
}
