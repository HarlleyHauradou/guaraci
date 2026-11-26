//! Physical units and dimensional analysis for Guaraci.
//!
//! Values are stored internally in SI base units together with their
//! dimension. Unit spellings follow these rules:
//!
//! * the exact spelling is tried first (`MeV`, `cm`, `g/cm3`);
//! * otherwise a case-insensitive match is accepted **only** when it is
//!   unique (`CM` -> `cm`, `KEV` -> `keV`);
//! * if changing case changes the meaning, the user must write it exactly.
//!
//! Compound units are products and quotients of known units, with optional
//! integer exponents: `g/cm3`, `g/cm^3`, `MBq*h`, `MBq.s`, `kg*m^-3`.

use std::fmt;

mod table;
pub use table::UNITS;

/// Physical dimension as exponents of length, mass, time and plane angle.
///
/// Energy is `M L^2 T^-2`, activity is `T^-1`, absorbed dose is `L^2 T^-2`.
/// Plane angle is kept as its own dimension so that `radius = 30 deg` is an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Dim {
    pub length: i8,
    pub mass: i8,
    pub time: i8,
    pub angle: i8,
}

impl Dim {
    pub const NONE: Dim = Dim::new(0, 0, 0, 0);
    pub const fn new(length: i8, mass: i8, time: i8, angle: i8) -> Self {
        Dim {
            length,
            mass,
            time,
            angle,
        }
    }
    pub const fn mul(self, o: Dim) -> Dim {
        Dim::new(
            self.length + o.length,
            self.mass + o.mass,
            self.time + o.time,
            self.angle + o.angle,
        )
    }
    pub const fn pow(self, n: i8) -> Dim {
        Dim::new(self.length * n, self.mass * n, self.time * n, self.angle * n)
    }
}

impl fmt::Display for Dim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Dim::NONE {
            return write!(f, "dimensionless");
        }
        let mut parts = Vec::new();
        for (sym, e) in [
            ("L", self.length),
            ("M", self.mass),
            ("T", self.time),
            ("angle", self.angle),
        ] {
            match e {
                0 => {}
                1 => parts.push(sym.to_string()),
                _ => parts.push(format!("{sym}^{e}")),
            }
        }
        write!(f, "{}", parts.join(" "))
    }
}

/// What kind of physical quantity an input key expects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Length,
    Area,
    Volume,
    Mass,
    Time,
    Energy,
    Activity,
    Density,
    Angle,
    Dose,
    /// Pure number (fractions, intensities, counts). Accepts `%`.
    Dimensionless,
}

impl Kind {
    pub const fn dim(self) -> Dim {
        match self {
            Kind::Length => Dim::new(1, 0, 0, 0),
            Kind::Area => Dim::new(2, 0, 0, 0),
            Kind::Volume => Dim::new(3, 0, 0, 0),
            Kind::Mass => Dim::new(0, 1, 0, 0),
            Kind::Time => Dim::new(0, 0, 1, 0),
            Kind::Energy => Dim::new(2, 1, -2, 0),
            Kind::Activity => Dim::new(0, 0, -1, 0),
            Kind::Density => Dim::new(-3, 1, 0, 0),
            Kind::Angle => Dim::new(0, 0, 0, 1),
            Kind::Dose => Dim::new(2, 0, -2, 0),
            Kind::Dimensionless => Dim::NONE,
        }
    }

    /// Unit assumed when the user writes a bare number.
    pub const fn default_unit(self) -> &'static str {
        match self {
            Kind::Length => "cm",
            Kind::Area => "cm2",
            Kind::Volume => "cm3",
            Kind::Mass => "g",
            Kind::Time => "s",
            Kind::Energy => "keV",
            Kind::Activity => "MBq",
            Kind::Density => "g/cm3",
            Kind::Angle => "deg",
            Kind::Dose => "Gy",
            Kind::Dimensionless => "",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Kind::Length => "length",
            Kind::Area => "area",
            Kind::Volume => "volume",
            Kind::Mass => "mass",
            Kind::Time => "time",
            Kind::Energy => "energy",
            Kind::Activity => "activity",
            Kind::Density => "density",
            Kind::Angle => "angle",
            Kind::Dose => "absorbed dose",
            Kind::Dimensionless => "pure number",
        }
    }

    /// Kind whose dimension matches `dim`, if any. Used for error messages.
    pub fn from_dim(dim: Dim) -> Option<Kind> {
        ALL_KINDS.iter().copied().find(|k| k.dim() == dim)
    }
}

pub const ALL_KINDS: [Kind; 11] = [
    Kind::Length,
    Kind::Area,
    Kind::Volume,
    Kind::Mass,
    Kind::Time,
    Kind::Energy,
    Kind::Activity,
    Kind::Density,
    Kind::Angle,
    Kind::Dose,
    Kind::Dimensionless,
];

/// One entry of the unit table.
#[derive(Debug, Clone, Copy)]
pub struct UnitDef {
    pub symbol: &'static str,
    /// Value of one of this unit in SI base units.
    pub factor: f64,
    pub dim: Dim,
    /// Human-readable name, shown in `guaraci info units`.
    pub name: &'static str,
}

/// A parsed unit expression such as `g/cm3`.
#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    pub factor: f64,
    pub dim: Dim,
    /// Canonical spelling (e.g. `cm` when the user wrote `CM`).
    pub canonical: String,
}

impl Unit {
    /// Parse a unit expression. See the crate docs for the accepted syntax.
    pub fn parse(text: &str) -> Result<Unit, UnitError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(UnitError::Empty);
        }
        let mut factor = 1.0;
        let mut dim = Dim::NONE;
        let mut canonical = String::new();
        let mut sign: i8 = 1;
        let mut first = true;
        let mut rest = text;
        loop {
            // separator
            if !first {
                let c = rest.chars().next().unwrap();
                match c {
                    '/' => sign = -1,
                    '*' | '.' | '·' => sign = 1,
                    _ => return Err(UnitError::Syntax(text.to_string())),
                }
                canonical.push(if c == '/' { '/' } else { '·' });
                rest = &rest[c.len_utf8()..];
            }
            first = false;
            let end = rest.find(['/', '*', '.', '·']).unwrap_or(rest.len());
            let atom = &rest[..end];
            rest = &rest[end..];
            let (sym, exp) = split_exponent(atom).ok_or_else(|| UnitError::Syntax(text.to_string()))?;
            let def = lookup(sym)?;
            let e = exp * sign;
            factor *= def.factor.powi(e as i32);
            dim = dim.mul(def.dim.pow(e));
            canonical.push_str(def.symbol);
            if exp != 1 {
                canonical.push_str(&exp.to_string());
            }
            if rest.is_empty() {
                break;
            }
        }
        Ok(Unit { factor, dim, canonical })
    }
}

/// Split `cm3`, `cm^3`, `m^-3` into symbol and exponent.
fn split_exponent(atom: &str) -> Option<(&str, i8)> {
    if atom.is_empty() {
        return None;
    }
    if let Some((s, e)) = atom.split_once('^') {
        return Some((s, e.parse().ok()?));
    }
    let digits = atom.len() - atom.trim_end_matches(|c: char| c.is_ascii_digit()).len();
    if digits > 0 && digits < atom.len() {
        let (s, e) = atom.split_at(atom.len() - digits);
        return Some((s, e.parse().ok()?));
    }
    Some((atom, 1))
}

/// Look up a single unit symbol following the case rules in the crate docs.
pub fn lookup(sym: &str) -> Result<&'static UnitDef, UnitError> {
    if let Some(d) = UNITS.iter().find(|d| d.symbol == sym) {
        return Ok(d);
    }
    let lower = sym.to_lowercase();
    let candidates: Vec<&UnitDef> = UNITS.iter().filter(|d| d.symbol.to_lowercase() == lower).collect();
    match candidates.as_slice() {
        [] => {
            let max_dist = (lower.chars().count() / 3).max(1);
            let mut sugg: Vec<(usize, &str)> = UNITS
                .iter()
                .map(|d| (strsim::levenshtein(&lower, &d.symbol.to_lowercase()), d.symbol))
                .filter(|(dist, _)| *dist <= max_dist)
                .collect();
            sugg.sort_by_key(|s| s.0);
            sugg.dedup_by_key(|s| s.1);
            Err(UnitError::Unknown {
                unit: sym.to_string(),
                suggestions: sugg.into_iter().take(3).map(|s| s.1).collect(),
            })
        }
        [one] => Ok(one),
        many => {
            let first = many[0];
            if many.iter().all(|d| d.dim == first.dim && d.factor == first.factor) {
                Ok(first)
            } else {
                Err(UnitError::Ambiguous {
                    unit: sym.to_string(),
                    options: many.iter().map(|d| (d.symbol, d.name)).collect(),
                })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum UnitError {
    #[error("empty unit")]
    Empty,
    #[error("cannot read unit `{0}`")]
    Syntax(String),
    #[error("unknown unit `{unit}`")]
    Unknown {
        unit: String,
        suggestions: Vec<&'static str>,
    },
    #[error("unit `{unit}` is ambiguous")]
    Ambiguous {
        unit: String,
        options: Vec<(&'static str, &'static str)>,
    },
    #[error("expected {expected}, but `{unit}` is {found}")]
    WrongKind {
        expected: &'static str,
        unit: String,
        found: String,
    },
}

impl UnitError {
    /// A friendly hint telling the user how to fix the problem.
    pub fn help(&self) -> Option<String> {
        match self {
            UnitError::Unknown { suggestions, .. } if !suggestions.is_empty() => {
                Some(format!("did you mean {}?", join_quoted(suggestions)))
            }
            UnitError::Unknown { .. } => Some("run `guaraci info units` to see every known unit".into()),
            UnitError::Ambiguous { options, .. } => Some(format!(
                "write it exactly: {}",
                options
                    .iter()
                    .map(|(s, n)| format!("`{s}` ({n})"))
                    .collect::<Vec<_>>()
                    .join(" or ")
            )),
            UnitError::Syntax(_) => Some("units look like `cm`, `g/cm3`, `MBq*h` or `kg*m^-3`".into()),
            UnitError::WrongKind { .. } | UnitError::Empty => None,
        }
    }
}

fn join_quoted(items: &[&str]) -> String {
    items.iter().map(|s| format!("`{s}`")).collect::<Vec<_>>().join(", ")
}

/// A physical value: magnitude in SI base units plus its dimension.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quantity {
    pub si: f64,
    pub dim: Dim,
}

impl Quantity {
    /// Build a quantity from a number and an optional unit, checking that it is
    /// of the expected kind. A missing unit means the kind's default unit.
    pub fn new(value: f64, unit: Option<&str>, expected: Kind) -> Result<(Quantity, Unit), UnitError> {
        let unit_text = unit.unwrap_or(expected.default_unit());
        let u = if unit_text.is_empty() {
            Unit {
                factor: 1.0,
                dim: Dim::NONE,
                canonical: String::new(),
            }
        } else {
            Unit::parse(unit_text)?
        };
        if u.dim != expected.dim() {
            let found = match Kind::from_dim(u.dim) {
                Some(k) => format!("a unit of {}", k.name()),
                None => format!("a unit of dimension {}", u.dim),
            };
            return Err(UnitError::WrongKind {
                expected: expected.name(),
                unit: unit_text.to_string(),
                found,
            });
        }
        Ok((
            Quantity {
                si: value * u.factor,
                dim: u.dim,
            },
            u,
        ))
    }

    /// Value expressed in `unit` (must have the same dimension).
    pub fn in_unit(&self, unit: &str) -> Result<f64, UnitError> {
        let u = if unit.is_empty() {
            Unit {
                factor: 1.0,
                dim: Dim::NONE,
                canonical: String::new(),
            }
        } else {
            Unit::parse(unit)?
        };
        if u.dim != self.dim {
            return Err(UnitError::WrongKind {
                expected: Kind::from_dim(self.dim).map(|k| k.name()).unwrap_or("?"),
                unit: unit.to_string(),
                found: format!("dimension {}", u.dim),
            });
        }
        Ok(self.si / u.factor)
    }

    /// Value in `unit`; panics if the dimension does not match.
    /// Use only with literal units known to be compatible.
    pub fn to(&self, unit: &str) -> f64 {
        self.in_unit(unit).expect("incompatible unit conversion")
    }
}

#[cfg(test)]
mod tests;
