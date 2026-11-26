//! Syntax tree of a `.gci` file. Every node remembers where it came from.

/// Index of a source file inside a [`crate::SourceMap`].
pub type FileId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub file: FileId,
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn join(self, other: Span) -> Span {
        Span {
            file: self.file,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
    pub fn len(&self) -> usize {
        self.end - self.start
    }
    pub fn is_empty(&self) -> bool {
        self.end == self.start
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Include {
    pub target: Spanned<String>,
}

/// `[kind name]` followed by `key = value` lines.
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub kind: Spanned<String>,
    pub name: Option<Spanned<String>>,
    pub entries: Vec<Entry>,
    pub header_span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub key: Spanned<String>,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    pub kind: ValueKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueKind {
    /// `10`, `10 cm`, `10cm`, `-1.5e3 keV`
    Number {
        value: f64,
        unit: Option<Spanned<String>>,
    },
    /// `"text"`
    Str(String),
    /// Bare word: `Water`, `Tc99m`, `dose.mhd`, `photon_local`
    Word(String),
    Bool(bool),
    /// `[1, 2, 3] cm` — `unit` applies to elements written without a unit.
    Array {
        items: Vec<Value>,
        unit: Option<Spanned<String>>,
    },
    /// `{ H = 0.11, O = 0.89 }`
    Table(Vec<Entry>),
}

impl ValueKind {
    pub fn describe(&self) -> &'static str {
        match self {
            ValueKind::Number { .. } => "a number",
            ValueKind::Str(_) => "a text",
            ValueKind::Word(_) => "a word",
            ValueKind::Bool(_) => "true/false",
            ValueKind::Array { .. } => "a list",
            ValueKind::Table(_) => "a table",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Include(Include),
    Section(Section),
}

/// One parsed file, in the order it was written.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct File {
    pub items: Vec<Item>,
}
