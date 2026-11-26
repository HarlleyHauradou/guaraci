//! Hand-written parser for `.gci` files.
//!
//! ```text
//! # comment
//! include materials, phantoms/body
//!
//! [volume lesion]
//! shape  = sphere
//! radius = 15 mm
//! center = [3, 0, 0]          # unit-less numbers use the default unit (cm)
//! size   = [10 cm, 10cm, 100 mm]
//! composition = { H = 0.11, O = 0.89 }
//! ```

use crate::ast::*;
use crate::diag::Diagnostic;

pub fn parse(file: FileId, text: &str) -> Result<File, Diagnostic> {
    let mut p = Parser { text, pos: 0, file };
    p.file()
}

struct Parser<'a> {
    text: &'a str,
    pos: usize,
    file: FileId,
}

fn is_word_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}
fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | '+')
}
fn is_key_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | '-')
}
fn is_unit_start(c: char) -> bool {
    c.is_alphabetic() || matches!(c, 'µ' | 'μ' | '°' | '%')
}
fn is_unit_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, 'µ' | 'μ' | '°' | '%' | '/' | '*' | '.' | '^' | '·')
}

impl<'a> Parser<'a> {
    fn span(&self, start: usize, end: usize) -> Span {
        Span {
            file: self.file,
            start,
            end,
        }
    }
    fn peek(&self) -> Option<char> {
        self.text[self.pos..].chars().next()
    }
    fn peek_at(&self, n: usize) -> Option<char> {
        self.text[self.pos..].chars().nth(n)
    }
    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }
    fn eat(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.pos += c.len_utf8();
            true
        } else {
            false
        }
    }
    fn take_while(&mut self, f: impl Fn(char) -> bool) -> &'a str {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if !f(c) {
                break;
            }
            self.pos += c.len_utf8();
        }
        &self.text[start..self.pos]
    }
    /// Spaces and tabs only.
    fn skip_inline_ws(&mut self) {
        self.take_while(|c| c == ' ' || c == '\t' || c == '\r');
    }
    fn skip_comment(&mut self) {
        if self.peek() == Some('#') {
            self.take_while(|c| c != '\n');
        }
    }
    /// Whitespace, newlines and comments.
    fn skip_all_ws(&mut self) {
        loop {
            self.take_while(char::is_whitespace);
            if self.peek() == Some('#') {
                self.skip_comment();
            } else {
                break;
            }
        }
    }
    /// After a statement: optional spaces/comment, then newline or end of file.
    fn end_of_line(&mut self, what: &str) -> Result<(), Diagnostic> {
        self.skip_inline_ws();
        self.skip_comment();
        match self.peek() {
            None => Ok(()),
            Some('\n') => {
                self.pos += 1;
                Ok(())
            }
            Some(_) => {
                let start = self.pos;
                let rest = self.take_while(|c| c != '\n' && c != '#').trim_end();
                Err(Diagnostic::error(format!("unexpected text after {what}"))
                    .at(self.span(start, start + rest.len()))
                    .label("not expected here")
                    .help("put one statement per line; comments start with `#`"))
            }
        }
    }

    fn file(&mut self) -> Result<File, Diagnostic> {
        let mut file = File::default();
        let mut current: Option<Section> = None;
        loop {
            self.skip_all_ws();
            let Some(c) = self.peek() else { break };
            if c == '[' {
                if let Some(s) = current.take() {
                    file.items.push(Item::Section(s));
                }
                current = Some(self.header()?);
                continue;
            }
            if self.at_keyword("include") {
                if let Some(s) = current.take() {
                    file.items.push(Item::Section(s));
                }
                for inc in self.include()? {
                    file.items.push(Item::Include(inc));
                }
                continue;
            }
            let start = self.pos;
            let entry = self.entry()?;
            self.end_of_line("the value")?;
            match current.as_mut() {
                Some(s) => s.entries.push(entry),
                None => {
                    return Err(Diagnostic::error(format!(
                        "`{}` is outside any section",
                        entry.key.node
                    ))
                    .at(self.span(start, entry.value.span.end))
                    .label("which section does this belong to?")
                    .help("start a section first, e.g. `[run]` or `[volume phantom]`"))
                }
            }
        }
        if let Some(s) = current.take() {
            file.items.push(Item::Section(s));
        }
        Ok(file)
    }

    fn at_keyword(&self, kw: &str) -> bool {
        let rest = &self.text[self.pos..];
        rest.starts_with(kw)
            && rest[kw.len()..]
                .chars()
                .next()
                .is_none_or(|c| c == ' ' || c == '\t')
    }

    fn include(&mut self) -> Result<Vec<Include>, Diagnostic> {
        let kw_start = self.pos;
        self.pos += "include".len();
        let mut out = Vec::new();
        loop {
            self.skip_inline_ws();
            match self.peek() {
                None | Some('\n') | Some('#') => break,
                Some(',') => {
                    self.pos += 1;
                    continue;
                }
                Some('"') => {
                    let (s, span) = self.string()?;
                    out.push(Include {
                        target: Spanned { node: s, span },
                    });
                }
                Some(_) => {
                    let start = self.pos;
                    let w = self.take_while(|c| !c.is_whitespace() && c != ',' && c != '#');
                    out.push(Include {
                        target: Spanned {
                            node: w.to_string(),
                            span: self.span(start, self.pos),
                        },
                    });
                }
            }
        }
        if out.is_empty() {
            return Err(Diagnostic::error("`include` needs a file name")
                .at(self.span(kw_start, self.pos))
                .help("for example: `include materials` reads `materials.gci`"));
        }
        self.end_of_line("include")?;
        Ok(out)
    }

    fn header(&mut self) -> Result<Section, Diagnostic> {
        let start = self.pos;
        self.bump(); // [
        self.skip_inline_ws();
        let kind = self.ident("section type", is_key_char)?;
        self.skip_inline_ws();
        let name = match self.peek() {
            Some(']') => None,
            Some('"') => {
                let (s, span) = self.string()?;
                Some(Spanned { node: s, span })
            }
            Some(c) if is_word_start(c) || c.is_ascii_digit() => {
                let s = self.pos;
                let w = self.take_while(is_word_char);
                Some(Spanned {
                    node: w.to_string(),
                    span: self.span(s, self.pos),
                })
            }
            _ => None,
        };
        self.skip_inline_ws();
        if !self.eat(']') {
            let here = self.pos;
            return Err(Diagnostic::error("section header is not closed")
                .at(self.span(start, here.max(start + 1)))
                .label("expected `]`")
                .help("a header looks like `[volume phantom]`: type, optional name, then `]`"));
        }
        let header_span = self.span(start, self.pos);
        self.end_of_line("the section header")?;
        Ok(Section {
            kind,
            name,
            entries: Vec::new(),
            header_span,
        })
    }

    fn ident(&mut self, what: &str, f: fn(char) -> bool) -> Result<Spanned<String>, Diagnostic> {
        let start = self.pos;
        match self.peek() {
            Some(c) if is_word_start(c) => {}
            _ => {
                return Err(Diagnostic::error(format!("expected a {what}"))
                    .at(self.span(start, (start + 1).min(self.text.len())))
                    .label(format!("a {what} starts with a letter")))
            }
        }
        let w = self.take_while(f);
        Ok(Spanned {
            node: w.to_string(),
            span: self.span(start, self.pos),
        })
    }

    fn entry(&mut self) -> Result<Entry, Diagnostic> {
        let key = self.ident("name", is_key_char)?;
        self.skip_inline_ws();
        if !self.eat('=') {
            let here = self.pos;
            let msg = if self.peek() == Some(':') {
                "use `=` instead of `:`".to_string()
            } else {
                format!("expected `=` after `{}`", key.node)
            };
            return Err(Diagnostic::error(msg)
                .at(self.span(here, (here + 1).min(self.text.len()).max(here)))
                .help(format!("write it as `{} = value`", key.node)));
        }
        self.skip_inline_ws();
        let value = self.value()?;
        Ok(Entry { key, value })
    }

    fn value(&mut self) -> Result<Value, Diagnostic> {
        let start = self.pos;
        match self.peek() {
            None | Some('\n') | Some('#') => Err(Diagnostic::error("missing value")
                .at(self.span(start.saturating_sub(1), start))
                .label("a value is expected after `=`")),
            Some('"') => {
                let (s, span) = self.string()?;
                Ok(Value { kind: ValueKind::Str(s), span })
            }
            Some('[') => self.array(),
            Some('{') => self.table(),
            Some(c) if c.is_ascii_digit() || ((c == '-' || c == '+' || c == '.') && self.peek_at(1).is_some_and(|d| d.is_ascii_digit() || d == '.')) => {
                self.number()
            }
            Some(c) if is_word_start(c) => {
                let w = self.take_while(is_word_char);
                let span = self.span(start, self.pos);
                let kind = match w {
                    "true" | "yes" | "on" => ValueKind::Bool(true),
                    "false" | "no" | "off" => ValueKind::Bool(false),
                    _ => ValueKind::Word(w.to_string()),
                };
                Ok(Value { kind, span })
            }
            Some(c) => Err(Diagnostic::error(format!("unexpected `{c}`"))
                .at(self.span(start, start + c.len_utf8()))
                .label("cannot start a value")
                .help("values are numbers (`10 cm`), words (`Water`), text (`\"a b\"`), lists (`[1, 2]`) or tables (`{ H = 1 }`)")),
        }
    }

    fn string(&mut self) -> Result<(String, Span), Diagnostic> {
        let start = self.pos;
        self.bump(); // "
        let mut out = String::new();
        loop {
            match self.bump() {
                None | Some('\n') => {
                    return Err(Diagnostic::error("text is not closed")
                        .at(self.span(start, self.pos.min(self.text.len())))
                        .label("missing closing `\"`"))
                }
                Some('"') => break,
                Some('\\') => match self.bump() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some(c) => out.push(c),
                    None => {}
                },
                Some(c) => out.push(c),
            }
        }
        Ok((out, self.span(start, self.pos)))
    }

    fn number(&mut self) -> Result<Value, Diagnostic> {
        let start = self.pos;
        if matches!(self.peek(), Some('+' | '-')) {
            self.bump();
        }
        self.take_while(|c| c.is_ascii_digit() || c == '_');
        if self.peek() == Some('.') && self.peek_at(1).is_none_or(|c| !c.is_alphabetic()) {
            self.bump();
            self.take_while(|c| c.is_ascii_digit() || c == '_');
        }
        // exponent: only if followed by digits, so `5e` stays a unit error rather than a silent parse
        if matches!(self.peek(), Some('e' | 'E')) {
            let save = self.pos;
            self.bump();
            if matches!(self.peek(), Some('+' | '-')) {
                self.bump();
            }
            if self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.take_while(|c| c.is_ascii_digit());
            } else {
                self.pos = save;
            }
        }
        let num_end = self.pos;
        let raw: String = self.text[start..num_end]
            .chars()
            .filter(|&c| c != '_')
            .collect();
        let value: f64 = raw.parse().map_err(|_| {
            Diagnostic::error(format!("`{}` is not a number", &self.text[start..num_end]))
                .at(self.span(start, num_end))
        })?;
        let unit = self.maybe_unit();
        let end = unit.as_ref().map(|u| u.span.end).unwrap_or(num_end);
        Ok(Value {
            kind: ValueKind::Number { value, unit },
            span: self.span(start, end),
        })
    }

    /// A unit written after a number or list, on the same line: `10 cm`, `10cm`.
    fn maybe_unit(&mut self) -> Option<Spanned<String>> {
        let save = self.pos;
        self.skip_inline_ws();
        match self.peek() {
            Some(c) if is_unit_start(c) => {
                let start = self.pos;
                let mut prev = ' ';
                while let Some(c) = self.peek() {
                    if is_unit_char(c) || (c == '-' && prev == '^') {
                        prev = c;
                        self.pos += c.len_utf8();
                    } else {
                        break;
                    }
                }
                Some(Spanned {
                    node: self.text[start..self.pos].to_string(),
                    span: self.span(start, self.pos),
                })
            }
            _ => {
                self.pos = save;
                None
            }
        }
    }

    fn array(&mut self) -> Result<Value, Diagnostic> {
        let start = self.pos;
        self.bump(); // [
        let mut items = Vec::new();
        loop {
            self.skip_all_ws();
            match self.peek() {
                Some(']') => {
                    self.bump();
                    break;
                }
                None => {
                    return Err(Diagnostic::error("list is not closed")
                        .at(self.span(start, start + 1))
                        .label("this `[` has no matching `]`"))
                }
                _ => {}
            }
            items.push(self.value()?);
            self.skip_all_ws();
            match self.peek() {
                Some(',') => {
                    self.bump();
                }
                Some(']') => {}
                Some(c) => {
                    let p = self.pos;
                    return Err(Diagnostic::error("expected `,` or `]` in list")
                        .at(self.span(p, p + c.len_utf8()))
                        .label("separate items with commas"));
                }
                None => {
                    return Err(Diagnostic::error("list is not closed")
                        .at(self.span(start, start + 1))
                        .label("this `[` has no matching `]`"))
                }
            }
        }
        let unit = self.maybe_unit();
        let end = unit.as_ref().map(|u| u.span.end).unwrap_or(self.pos);
        Ok(Value {
            kind: ValueKind::Array { items, unit },
            span: self.span(start, end),
        })
    }

    fn table(&mut self) -> Result<Value, Diagnostic> {
        let start = self.pos;
        self.bump(); // {
        let mut entries = Vec::new();
        loop {
            self.skip_all_ws();
            match self.peek() {
                Some('}') => {
                    self.bump();
                    break;
                }
                None => {
                    return Err(Diagnostic::error("table is not closed")
                        .at(self.span(start, start + 1))
                        .label("this `{` has no matching `}`"))
                }
                _ => {}
            }
            entries.push(self.entry()?);
            self.skip_all_ws();
            if !self.eat(',') && self.peek() != Some('}') {
                let p = self.pos;
                return Err(Diagnostic::error("expected `,` or `}` in table")
                    .at(self.span(p, (p + 1).min(self.text.len())))
                    .label("separate entries with commas"));
            }
        }
        Ok(Value {
            kind: ValueKind::Table(entries),
            span: self.span(start, self.pos),
        })
    }
}
