//! Typed access to the entries of a section, collecting every problem.

use crate::ast::*;
use crate::diag::Diagnostic;
use guaraci_units::{Kind, Quantity};

/// A value that was not written by the user and took its default.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultNote {
    pub section: String,
    pub key: String,
    pub value: String,
}

/// A value that was written with no unit and got the default unit.
#[derive(Debug, Clone, PartialEq)]
pub struct ImplicitUnit {
    pub span: Span,
    pub unit: &'static str,
}

/// Shared sink for diagnostics and notes while building a config.
#[derive(Debug, Default)]
pub struct Notes {
    pub diags: Vec<Diagnostic>,
    pub defaults: Vec<DefaultNote>,
    pub implicit_units: Vec<ImplicitUnit>,
}

impl Notes {
    pub fn error(&mut self, d: Diagnostic) {
        self.diags.push(d);
    }
    pub fn has_errors(&self) -> bool {
        self.diags.iter().any(|d| d.is_error())
    }
}

/// Reads the entries of one section. Call [`Reader::finish`] at the end so
/// unknown keys are reported.
pub struct Reader<'a> {
    pub section: &'a Section,
    used: Vec<&'a str>,
    title: String,
}

impl<'a> Reader<'a> {
    pub fn new(section: &'a Section) -> Self {
        let title = match &section.name {
            Some(n) => format!("[{} {}]", section.kind.node, n.node),
            None => format!("[{}]", section.kind.node),
        };
        Reader {
            section,
            used: Vec::new(),
            title,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    /// The raw entry for `key`, marking it as used. Duplicates are reported.
    pub fn entry(&mut self, key: &'a str, notes: &mut Notes) -> Option<&'a Entry> {
        self.used.push(key);
        let mut found = self.section.entries.iter().filter(|e| e.key.node == key);
        let first = found.next()?;
        if let Some(dup) = found.next() {
            notes.error(
                Diagnostic::error(format!("`{key}` is given twice in {}", self.title))
                    .at(dup.key.span)
                    .label("second value")
                    .related(first.key.span, "first value")
                    .help("keep only one of them"),
            );
        }
        Some(first)
    }

    pub fn has(&self, key: &str) -> bool {
        self.section.entries.iter().any(|e| e.key.node == key)
    }

    fn default_note(&self, key: &str, value: String, notes: &mut Notes) {
        notes.defaults.push(DefaultNote {
            section: self.title.clone(),
            key: key.to_string(),
            value,
        });
    }

    // ---- numbers -------------------------------------------------------

    /// A quantity of the given kind, or `None` if absent or invalid.
    pub fn quantity(&mut self, key: &'a str, kind: Kind, notes: &mut Notes) -> Option<Quantity> {
        let e = self.entry(key, notes)?;
        quantity_of(&e.value, None, kind, key, notes)
    }

    /// A quantity, or `default` (expressed in the kind's default unit).
    pub fn quantity_or(&mut self, key: &'a str, kind: Kind, default: f64, notes: &mut Notes) -> Quantity {
        if self.has(key) {
            if let Some(q) = self.quantity(key, kind, notes) {
                return q;
            }
        } else {
            self.used.push(key);
            self.default_note(
                key,
                format!("{} {}", default, kind.default_unit()).trim().to_string(),
                notes,
            );
        }
        Quantity::new(default, None, kind).expect("valid default").0
    }

    /// A required quantity.
    pub fn require_quantity(&mut self, key: &'a str, kind: Kind, notes: &mut Notes) -> Option<Quantity> {
        if !self.has(key) {
            self.missing(key, &format!("{} = <{}>", key, kind.name()), notes);
            self.used.push(key);
            return None;
        }
        self.quantity(key, kind, notes)
    }

    /// A list of three quantities (`[1, 2, 3] cm`, `[1 cm, 2cm, 30 mm]`), or a
    /// single value used for all three.
    pub fn vec3(&mut self, key: &'a str, kind: Kind, notes: &mut Notes) -> Option<[Quantity; 3]> {
        let e = self.entry(key, notes)?;
        match &e.value.kind {
            ValueKind::Array { items, unit } => {
                if items.len() != 3 {
                    notes.error(
                        Diagnostic::error(format!("`{key}` needs 3 values (x, y, z), found {}", items.len()))
                            .at(e.value.span)
                            .help(format!("for example `{key} = [1, 2, 3] {}`", kind.default_unit())),
                    );
                    return None;
                }
                let qs: Vec<Option<Quantity>> = items
                    .iter()
                    .map(|v| quantity_of(v, unit.as_ref(), kind, key, notes))
                    .collect();
                Some([qs[0]?, qs[1]?, qs[2]?])
            }
            _ => {
                let q = quantity_of(&e.value, None, kind, key, notes)?;
                Some([q, q, q])
            }
        }
    }

    pub fn require_vec3(&mut self, key: &'a str, kind: Kind, notes: &mut Notes) -> Option<[Quantity; 3]> {
        if !self.has(key) {
            self.missing(key, &format!("{key} = [x, y, z] {}", kind.default_unit()), notes);
            self.used.push(key);
            return None;
        }
        self.vec3(key, kind, notes)
    }

    /// A list of quantities of one kind.
    pub fn quantity_list(&mut self, key: &'a str, kind: Kind, notes: &mut Notes) -> Option<Vec<Quantity>> {
        let e = self.entry(key, notes)?;
        match &e.value.kind {
            ValueKind::Array { items, unit } => {
                let qs: Vec<Option<Quantity>> = items
                    .iter()
                    .map(|v| quantity_of(v, unit.as_ref(), kind, key, notes))
                    .collect();
                qs.into_iter().collect()
            }
            _ => quantity_of(&e.value, None, kind, key, notes).map(|q| vec![q]),
        }
    }

    /// A non-negative whole number such as `histories = 1e7`.
    pub fn count(&mut self, key: &'a str, notes: &mut Notes) -> Option<u64> {
        let e = self.entry(key, notes)?;
        match &e.value.kind {
            ValueKind::Number { value, unit: None } if *value >= 0.0 && value.fract() == 0.0 && *value < 1.8e19 => {
                Some(*value as u64)
            }
            ValueKind::Number { value, unit: Some(u) } => {
                notes.error(
                    Diagnostic::error(format!("`{key}` is a plain count and takes no unit"))
                        .at(u.span)
                        .label("remove this")
                        .help(format!("write `{key} = {value}`")),
                );
                None
            }
            ValueKind::Number { .. } => {
                notes.error(
                    Diagnostic::error(format!("`{key}` must be a whole number, zero or more"))
                        .at(e.value.span)
                        .help("for example `1e7` or `10_000_000`"),
                );
                None
            }
            other => {
                notes.error(type_error(key, "a whole number", other, e.value.span));
                None
            }
        }
    }

    pub fn count_or(&mut self, key: &'a str, default: u64, notes: &mut Notes) -> u64 {
        if self.has(key) {
            self.count(key, notes).unwrap_or(default)
        } else {
            self.used.push(key);
            self.default_note(key, default.to_string(), notes);
            default
        }
    }

    // ---- words ---------------------------------------------------------

    /// A word or quoted text.
    pub fn text(&mut self, key: &'a str, notes: &mut Notes) -> Option<Spanned<String>> {
        let e = self.entry(key, notes)?;
        match &e.value.kind {
            ValueKind::Word(w) | ValueKind::Str(w) => Some(Spanned {
                node: w.clone(),
                span: e.value.span,
            }),
            other => {
                notes.error(type_error(key, "a name", other, e.value.span));
                None
            }
        }
    }

    pub fn require_text(&mut self, key: &'a str, hint: &str, notes: &mut Notes) -> Option<Spanned<String>> {
        if !self.has(key) {
            self.missing(key, &format!("{key} = {hint}"), notes);
            self.used.push(key);
            return None;
        }
        self.text(key, notes)
    }

    /// A list of words (`volumes = [lesion, liver]`) or a single word.
    pub fn text_list(&mut self, key: &'a str, notes: &mut Notes) -> Option<Vec<Spanned<String>>> {
        let e = self.entry(key, notes)?;
        let one = |v: &Value, notes: &mut Notes| match &v.kind {
            ValueKind::Word(w) | ValueKind::Str(w) => Some(Spanned {
                node: w.clone(),
                span: v.span,
            }),
            other => {
                notes.error(type_error(key, "a name", other, v.span));
                None
            }
        };
        match &e.value.kind {
            ValueKind::Array { items, .. } => items.iter().map(|v| one(v, notes)).collect(),
            _ => one(&e.value, notes).map(|w| vec![w]),
        }
    }

    /// One of a fixed set of words; case-insensitive.
    pub fn choice(
        &mut self,
        key: &'a str,
        options: &[&'static str],
        default: &'static str,
        notes: &mut Notes,
    ) -> &'static str {
        if !self.has(key) {
            self.used.push(key);
            self.default_note(key, default.to_string(), notes);
            return default;
        }
        let Some(w) = self.text(key, notes) else {
            return default;
        };
        if let Some(o) = options.iter().find(|o| o.eq_ignore_ascii_case(&w.node)) {
            return o;
        }
        notes.error(
            Diagnostic::error(format!("`{}` is not a valid {key}", w.node))
                .at(w.span)
                .label(format!("options: {}", options.join(", ")))
                .maybe_help(suggest(&w.node, options.iter().copied()).map(|s| format!("did you mean `{s}`?"))),
        );
        default
    }

    pub fn bool_or(&mut self, key: &'a str, default: bool, notes: &mut Notes) -> bool {
        let Some(e) = self.entry(key, notes) else {
            self.default_note(key, default.to_string(), notes);
            return default;
        };
        match &e.value.kind {
            ValueKind::Bool(b) => *b,
            other => {
                notes.error(type_error(key, "`on` or `off`", other, e.value.span));
                default
            }
        }
    }

    // ---- bookkeeping ---------------------------------------------------

    fn missing(&self, key: &str, example: &str, notes: &mut Notes) {
        notes.error(
            Diagnostic::error(format!("{} needs `{key}`", self.title))
                .at(self.section.header_span)
                .label(format!("`{key}` is missing in this section"))
                .help(format!("add a line like `{example}`")),
        );
    }

    /// Report keys that were never read.
    pub fn finish(self, notes: &mut Notes) {
        for e in &self.section.entries {
            if !self.used.contains(&e.key.node.as_str()) {
                let mut d = Diagnostic::error(format!("{} has no setting called `{}`", self.title, e.key.node))
                    .at(e.key.span)
                    .label("unknown setting");
                let mut known: Vec<&str> = self.used.clone();
                known.sort();
                known.dedup();
                d = match suggest(&e.key.node, known.iter().copied()) {
                    Some(s) => d.help(format!("did you mean `{s}`?")),
                    None => d.help(format!("settings here: {}", known.join(", "))),
                };
                notes.error(d);
            }
        }
    }
}

/// Convert a value to a quantity, applying an outer list unit if given.
pub fn quantity_of(
    v: &Value,
    outer: Option<&Spanned<String>>,
    kind: Kind,
    key: &str,
    notes: &mut Notes,
) -> Option<Quantity> {
    let ValueKind::Number { value, unit } = &v.kind else {
        notes.error(type_error(key, &format!("a {}", kind.name()), &v.kind, v.span));
        return None;
    };
    let unit = unit.as_ref().or(outer);
    if unit.is_none() && kind != Kind::Dimensionless {
        notes.implicit_units.push(ImplicitUnit {
            span: v.span,
            unit: kind.default_unit(),
        });
    }
    match Quantity::new(*value, unit.map(|u| u.node.as_str()), kind) {
        Ok((q, _)) => Some(q),
        Err(e) => {
            let span = unit.map(|u| u.span).unwrap_or(v.span);
            let help = e.help().or_else(|| match kind {
                Kind::Dimensionless => Some(format!("`{key}` is a pure number; remove the unit")),
                _ => Some(format!("use a unit of {}, e.g. `{}`", kind.name(), kind.default_unit())),
            });
            notes.error(Diagnostic::error(format!("`{key}`: {e}")).at(span).maybe_help(help));
            None
        }
    }
}

fn type_error(key: &str, expected: &str, found: &ValueKind, span: Span) -> Diagnostic {
    Diagnostic::error(format!("`{key}` should be {expected}, not {}", found.describe()))
        .at(span)
        .label("here")
}

/// Closest option, if reasonably close.
pub fn suggest<'b>(word: &str, options: impl Iterator<Item = &'b str>) -> Option<&'b str> {
    let w = word.to_lowercase();
    options
        .map(|o| (strsim::jaro_winkler(&w, &o.to_lowercase()), o))
        .filter(|(s, _)| *s > 0.8)
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, o)| o)
}
