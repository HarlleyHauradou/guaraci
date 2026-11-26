//! Minimal reader for the ENDF-6 format (fixed 80-column records).
//!
//! Only what Guaraci needs is implemented: CONT/HEAD, LIST and TAB1 records,
//! and splitting a file into (MF, MT) sections.

use std::collections::BTreeMap;

#[derive(Debug, thiserror::Error)]
#[error("{file}: line {line}: {msg}")]
pub struct EndfError {
    pub file: String,
    pub line: usize,
    pub msg: String,
}

/// Parse an ENDF real number: `1.234567+5`, `-1.2-3`, `1.0E+9`, ` 8000.0`, blank = 0.
pub fn parse_real(field: &str) -> Option<f64> {
    let t = field.trim();
    if t.is_empty() {
        return Some(0.0);
    }
    if let Ok(v) = t.parse() {
        return Some(v);
    }
    // insert the missing `E` before an exponent sign that is not the leading sign
    let bytes = t.as_bytes();
    for i in 1..bytes.len() {
        if (bytes[i] == b'+' || bytes[i] == b'-') && !matches!(bytes[i - 1], b'E' | b'e') {
            let s = format!("{}E{}", &t[..i], &t[i..]);
            return s.parse().ok();
        }
    }
    None
}

pub fn parse_int(field: &str) -> Option<i64> {
    let t = field.trim();
    if t.is_empty() {
        Some(0)
    } else {
        t.parse().ok()
    }
}

/// The lines of one (MF, MT) section, without the MAT/MF/MT columns.
#[derive(Debug, Clone)]
pub struct Section<'a> {
    pub mf: u32,
    pub mt: u32,
    lines: Vec<(usize, &'a str)>,
    file: String,
}

/// Split an ENDF file into its sections, keyed by (MF, MT).
pub fn sections<'a>(file: &str, text: &'a str) -> BTreeMap<(u32, u32), Section<'a>> {
    let mut out: BTreeMap<(u32, u32), Section<'a>> = BTreeMap::new();
    for (i, line) in text.lines().enumerate() {
        if line.len() < 75 {
            continue;
        }
        let (Some(mf), Some(mt)) = (parse_int(&line[70..72]), parse_int(&line[72..75])) else {
            continue;
        };
        if mf <= 0 || mt <= 0 {
            continue; // SEND/FEND/MEND/TEND and the tape id
        }
        out.entry((mf as u32, mt as u32))
            .or_insert_with(|| Section {
                mf: mf as u32,
                mt: mt as u32,
                lines: Vec::new(),
                file: file.to_string(),
            })
            .lines
            .push((i + 1, &line[..66]));
    }
    out
}

/// CONT/HEAD record: C1, C2, L1, L2, N1, N2.
#[derive(Debug, Clone, Copy)]
pub struct Cont {
    pub c1: f64,
    pub c2: f64,
    pub l1: i64,
    pub l2: i64,
    pub n1: i64,
    pub n2: i64,
}

/// TAB1 record: a tabulated function y(x) with interpolation laws.
#[derive(Debug, Clone)]
pub struct Tab1 {
    pub cont: Cont,
    /// (NBT, INT) interpolation ranges
    pub ranges: Vec<(usize, u8)>,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
}

/// Sequential reader over a section.
pub struct Reader<'s, 'a> {
    sec: &'s Section<'a>,
    pos: usize,
}

impl<'a> Section<'a> {
    pub fn reader(&self) -> Reader<'_, 'a> {
        Reader { sec: self, pos: 0 }
    }
}

impl Reader<'_, '_> {
    fn err(&self, msg: impl Into<String>) -> EndfError {
        let line = self.sec.lines.get(self.pos.saturating_sub(1)).map(|l| l.0).unwrap_or(0);
        EndfError {
            file: self.sec.file.clone(),
            line,
            msg: format!("MF{} MT{}: {}", self.sec.mf, self.sec.mt, msg.into()),
        }
    }

    fn next_line(&mut self) -> Result<&str, EndfError> {
        let l = self
            .sec
            .lines
            .get(self.pos)
            .ok_or_else(|| self.err("unexpected end of section"))?
            .1;
        self.pos += 1;
        Ok(l)
    }

    pub fn at_end(&self) -> bool {
        self.pos >= self.sec.lines.len()
    }

    fn field(line: &str, i: usize) -> &str {
        let s = i * 11;
        if s >= line.len() {
            ""
        } else {
            &line[s..(s + 11).min(line.len())]
        }
    }

    pub fn cont(&mut self) -> Result<Cont, EndfError> {
        let l = self.next_line()?.to_string();
        let r = |i| parse_real(Self::field(&l, i));
        let n = |i| parse_int(Self::field(&l, i));
        match (r(0), r(1), n(2), n(3), n(4), n(5)) {
            (Some(c1), Some(c2), Some(l1), Some(l2), Some(n1), Some(n2)) => Ok(Cont { c1, c2, l1, l2, n1, n2 }),
            _ => Err(self.err(format!("cannot read record `{l}`"))),
        }
    }

    /// `n` real numbers, 6 per line.
    fn reals(&mut self, n: usize) -> Result<Vec<f64>, EndfError> {
        let mut out = Vec::with_capacity(n);
        while out.len() < n {
            let l = self.next_line()?.to_string();
            for i in 0..6 {
                if out.len() == n {
                    break;
                }
                out.push(parse_real(Self::field(&l, i)).ok_or_else(|| self.err(format!("bad number in `{l}`")))?);
            }
        }
        Ok(out)
    }

    fn ints(&mut self, n: usize) -> Result<Vec<i64>, EndfError> {
        let mut out = Vec::with_capacity(n);
        while out.len() < n {
            let l = self.next_line()?.to_string();
            for i in 0..6 {
                if out.len() == n {
                    break;
                }
                out.push(parse_int(Self::field(&l, i)).ok_or_else(|| self.err(format!("bad integer in `{l}`")))?);
            }
        }
        Ok(out)
    }

    /// LIST record: CONT with NPL = N1 values following.
    pub fn list(&mut self) -> Result<(Cont, Vec<f64>), EndfError> {
        let c = self.cont()?;
        let v = self.reals(c.n1.max(0) as usize)?;
        Ok((c, v))
    }

    /// TAB2 header: CONT plus NR interpolation ranges (the NZ sub-records follow).
    pub fn tab2(&mut self) -> Result<(Cont, Vec<(usize, u8)>), EndfError> {
        let c = self.cont()?;
        let r = self.ints(2 * c.n1.max(0) as usize)?;
        Ok((c, r.chunks(2).map(|p| (p[0] as usize, p[1] as u8)).collect()))
    }

    pub fn tab1(&mut self) -> Result<Tab1, EndfError> {
        let c = self.cont()?;
        let nr = c.n1.max(0) as usize;
        let np = c.n2.max(0) as usize;
        let r = self.ints(2 * nr)?;
        let ranges = r.chunks(2).map(|p| (p[0] as usize, p[1] as u8)).collect();
        let xy = self.reals(2 * np)?;
        let (x, y) = xy.chunks(2).map(|p| (p[0], p[1])).unzip();
        Ok(Tab1 { cont: c, ranges, x, y })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reals() {
        assert_eq!(parse_real(" 1.916906+5"), Some(1.916906e5));
        assert_eq!(parse_real("-7.872955-3"), Some(-7.872955e-3));
        assert_eq!(parse_real(" 1.00000E+9"), Some(1e9));
        assert_eq!(parse_real(" .001000000"), Some(0.001));
        assert_eq!(parse_real(" 897244443."), Some(897244443.0));
        assert_eq!(parse_real("           "), Some(0.0));
    }
}
