//! Diagnostics: errors and warnings that point at the input text.

use crate::ast::Span;
use crate::source::SourceMap;
use miette::{GraphicalReportHandler, GraphicalTheme, LabeledSpan, NamedSource, SourceSpan};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// A message about the input, optionally pointing at a place in a file.
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub span: Option<Span>,
    pub label: Option<String>,
    pub help: Option<String>,
    /// Secondary location, e.g. the first definition of a duplicated name.
    pub related: Option<(Span, String)>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Diagnostic {
            severity: Severity::Error,
            message: message.into(),
            span: None,
            label: None,
            help: None,
            related: None,
        }
    }
    pub fn warning(message: impl Into<String>) -> Self {
        Diagnostic {
            severity: Severity::Warning,
            ..Diagnostic::error(message)
        }
    }
    pub fn at(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
    pub fn maybe_help(mut self, help: Option<String>) -> Self {
        self.help = help;
        self
    }
    pub fn related(mut self, span: Span, label: impl Into<String>) -> Self {
        self.related = Some((span, label.into()));
        self
    }
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }

    /// Render with source excerpt, colours optional.
    pub fn render(&self, sources: &SourceMap, color: bool) -> String {
        let report = Rendered { d: self, sources };
        let theme = if color {
            GraphicalTheme::unicode()
        } else {
            GraphicalTheme::unicode_nocolor()
        };
        let handler = GraphicalReportHandler::new_themed(theme).with_width(100);
        let mut out = String::new();
        handler.render_report(&mut out, &report).ok();
        out
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

struct Rendered<'a> {
    d: &'a Diagnostic,
    sources: &'a SourceMap,
}

impl fmt::Debug for Rendered<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.d.message)
    }
}
impl fmt::Display for Rendered<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.d.message)
    }
}
impl std::error::Error for Rendered<'_> {}

impl miette::Diagnostic for Rendered<'_> {
    fn severity(&self) -> Option<miette::Severity> {
        Some(match self.d.severity {
            Severity::Error => miette::Severity::Error,
            Severity::Warning => miette::Severity::Warning,
        })
    }
    fn help<'b>(&'b self) -> Option<Box<dyn fmt::Display + 'b>> {
        self.d
            .help
            .as_ref()
            .map(|h| Box::new(h) as Box<dyn fmt::Display>)
    }
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        let span = self.d.span?;
        Some(self.sources.named(span.file) as &dyn miette::SourceCode)
    }
    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        let span = self.d.span?;
        let mut labels = vec![LabeledSpan::new_with_span(
            self.d.label.clone(),
            to_source_span(span),
        )];
        // miette renders one source per report; only show related spans from the same file.
        if let Some((r, l)) = &self.d.related {
            if r.file == span.file {
                labels.push(LabeledSpan::new_with_span(
                    Some(l.clone()),
                    to_source_span(*r),
                ));
            }
        }
        Some(Box::new(labels.into_iter()))
    }
}

fn to_source_span(s: Span) -> SourceSpan {
    SourceSpan::new(s.start.into(), s.len())
}

pub(crate) type Named = NamedSource<String>;
