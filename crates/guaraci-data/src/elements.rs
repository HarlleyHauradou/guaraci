//! Chemical element symbols and names (Z = 1..100).
//!
//! Only symbols and names live here; atomic masses come from the downloaded
//! data libraries (the AWR of each ENDF photo-atomic file).

pub const SYMBOLS: [&str; 100] = [
    "H", "He", "Li", "Be", "B", "C", "N", "O", "F", "Ne", "Na", "Mg", "Al", "Si", "P", "S", "Cl", "Ar", "K", "Ca",
    "Sc", "Ti", "V", "Cr", "Mn", "Fe", "Co", "Ni", "Cu", "Zn", "Ga", "Ge", "As", "Se", "Br", "Kr", "Rb", "Sr", "Y",
    "Zr", "Nb", "Mo", "Tc", "Ru", "Rh", "Pd", "Ag", "Cd", "In", "Sn", "Sb", "Te", "I", "Xe", "Cs", "Ba", "La", "Ce",
    "Pr", "Nd", "Pm", "Sm", "Eu", "Gd", "Tb", "Dy", "Ho", "Er", "Tm", "Yb", "Lu", "Hf", "Ta", "W", "Re", "Os", "Ir",
    "Pt", "Au", "Hg", "Tl", "Pb", "Bi", "Po", "At", "Rn", "Fr", "Ra", "Ac", "Th", "Pa", "U", "Np", "Pu", "Am", "Cm",
    "Bk", "Cf", "Es", "Fm",
];

pub const NAMES: [&str; 100] = [
    "hydrogen",
    "helium",
    "lithium",
    "beryllium",
    "boron",
    "carbon",
    "nitrogen",
    "oxygen",
    "fluorine",
    "neon",
    "sodium",
    "magnesium",
    "aluminium",
    "silicon",
    "phosphorus",
    "sulfur",
    "chlorine",
    "argon",
    "potassium",
    "calcium",
    "scandium",
    "titanium",
    "vanadium",
    "chromium",
    "manganese",
    "iron",
    "cobalt",
    "nickel",
    "copper",
    "zinc",
    "gallium",
    "germanium",
    "arsenic",
    "selenium",
    "bromine",
    "krypton",
    "rubidium",
    "strontium",
    "yttrium",
    "zirconium",
    "niobium",
    "molybdenum",
    "technetium",
    "ruthenium",
    "rhodium",
    "palladium",
    "silver",
    "cadmium",
    "indium",
    "tin",
    "antimony",
    "tellurium",
    "iodine",
    "xenon",
    "caesium",
    "barium",
    "lanthanum",
    "cerium",
    "praseodymium",
    "neodymium",
    "promethium",
    "samarium",
    "europium",
    "gadolinium",
    "terbium",
    "dysprosium",
    "holmium",
    "erbium",
    "thulium",
    "ytterbium",
    "lutetium",
    "hafnium",
    "tantalum",
    "tungsten",
    "rhenium",
    "osmium",
    "iridium",
    "platinum",
    "gold",
    "mercury",
    "thallium",
    "lead",
    "bismuth",
    "polonium",
    "astatine",
    "radon",
    "francium",
    "radium",
    "actinium",
    "thorium",
    "protactinium",
    "uranium",
    "neptunium",
    "plutonium",
    "americium",
    "curium",
    "berkelium",
    "californium",
    "einsteinium",
    "fermium",
];

/// Z of an element symbol (exact case: `Co` is cobalt, `CO` is not an element).
pub fn z_of(symbol: &str) -> Option<u32> {
    SYMBOLS.iter().position(|s| *s == symbol).map(|i| i as u32 + 1)
}

pub fn symbol(z: u32) -> &'static str {
    SYMBOLS[(z - 1) as usize]
}

/// A nuclide such as I-131 or Tc-99m.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct NuclideId {
    pub z: u32,
    pub a: u32,
    /// 0 = ground state, 1 = first isomer (`m`), 2 = second (`m2`)...
    pub isomer: u32,
}

impl std::fmt::Display for NuclideId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", symbol(self.z), self.a)?;
        match self.isomer {
            0 => Ok(()),
            1 => write!(f, "m"),
            n => write!(f, "m{n}"),
        }
    }
}

impl NuclideId {
    /// Read a nuclide name. Accepted forms (case-insensitive for the element
    /// when unambiguous): `I131`, `I-131`, `131I`, `Tc99m`, `Tc-99m`, `99mTc`,
    /// `Lu-177m2`.
    pub fn parse(text: &str) -> Option<NuclideId> {
        let t: String = text.trim().chars().filter(|c| !matches!(c, '-' | '_' | ' ')).collect();
        if t.is_empty() {
            return None;
        }
        let (sym, rest) = if t.chars().next()?.is_ascii_digit() {
            // 131I, 99mTc, 177m2Lu
            let digits: String = t.chars().take_while(|c| c.is_ascii_digit()).collect();
            let after = &t[digits.len()..];
            let (iso, sym) = split_isomer_prefix(after);
            (sym.to_string(), format!("{digits}{iso}"))
        } else {
            let sym: String = t.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
            (sym.clone(), t[sym.len()..].to_string())
        };
        let z = z_of(&sym).or_else(|| {
            let mut c = sym.chars();
            let first = c.next()?.to_ascii_uppercase();
            let norm: String = std::iter::once(first)
                .chain(c.map(|c| c.to_ascii_lowercase()))
                .collect();
            z_of(&norm)
        })?;
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        let a: u32 = digits.parse().ok()?;
        let tail = &rest[digits.len()..];
        let isomer = match tail {
            "" => 0,
            "m" | "M" => 1,
            _ if tail.starts_with(['m', 'M']) => tail[1..].parse().ok()?,
            _ => return None,
        };
        if a < z {
            return None;
        }
        Some(NuclideId { z, a, isomer })
    }
}

fn split_isomer_prefix(s: &str) -> (&str, &str) {
    // "mTc" -> ("m", "Tc"); "m2Lu" -> ("m2", "Lu"); "I" -> ("", "I")
    if s.len() > 1 && s.starts_with('m') {
        let rest = &s[1..];
        let digits = rest.chars().take_while(|c| c.is_ascii_digit()).count();
        let sym = &rest[digits..];
        if !sym.is_empty() && sym.chars().next().unwrap().is_ascii_uppercase() {
            return (&s[..1 + digits], sym);
        }
    }
    ("", s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nuclide_names() {
        let i131 = NuclideId {
            z: 53,
            a: 131,
            isomer: 0,
        };
        for s in ["I131", "I-131", "131I", "i-131", "I 131"] {
            assert_eq!(NuclideId::parse(s), Some(i131), "{s}");
        }
        let tc = NuclideId {
            z: 43,
            a: 99,
            isomer: 1,
        };
        for s in ["Tc99m", "Tc-99m", "99mTc", "TC-99M", "tc99m"] {
            assert_eq!(NuclideId::parse(s), Some(tc), "{s}");
        }
        assert_eq!(
            NuclideId::parse("Lu-177m2"),
            Some(NuclideId {
                z: 71,
                a: 177,
                isomer: 2
            })
        );
        assert_eq!(NuclideId::parse("C14").unwrap().to_string(), "C-14");
        assert_eq!(tc.to_string(), "Tc-99m");
        assert_eq!(NuclideId::parse("Xx-12"), None);
        assert_eq!(NuclideId::parse("I"), None);
    }
}
