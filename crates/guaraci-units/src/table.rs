//! The table of known units. Only units that make sense in radiation transport
//! are listed on purpose: an unusual symbol is more likely a typo than intent.

use crate::{Dim, UnitDef};

const L: Dim = Dim::new(1, 0, 0, 0);
const M: Dim = Dim::new(0, 1, 0, 0);
const T: Dim = Dim::new(0, 0, 1, 0);
const A: Dim = Dim::new(0, 0, 0, 1);
const E: Dim = Dim::new(2, 1, -2, 0);
const ACT: Dim = Dim::new(0, 0, -1, 0);
const DOSE: Dim = Dim::new(2, 0, -2, 0);
const VOL: Dim = Dim::new(3, 0, 0, 0);
const NONE: Dim = Dim::NONE;

const EV: f64 = 1.602_176_634e-19;
const CI: f64 = 3.7e10;
const YEAR: f64 = 365.25 * 86_400.0;

macro_rules! u {
    ($s:expr, $f:expr, $d:expr, $n:expr) => {
        UnitDef {
            symbol: $s,
            factor: $f,
            dim: $d,
            name: $n,
        }
    };
}

pub static UNITS: &[UnitDef] = &[
    // length
    u!("nm", 1e-9, L, "nanometre"),
    u!("um", 1e-6, L, "micrometre"),
    u!("µm", 1e-6, L, "micrometre"),
    u!("μm", 1e-6, L, "micrometre"),
    u!("mm", 1e-3, L, "millimetre"),
    u!("cm", 1e-2, L, "centimetre"),
    u!("m", 1.0, L, "metre"),
    // volume
    u!("cc", 1e-6, VOL, "cubic centimetre"),
    u!("mL", 1e-6, VOL, "millilitre"),
    u!("ml", 1e-6, VOL, "millilitre"),
    u!("L", 1e-3, VOL, "litre"),
    u!("l", 1e-3, VOL, "litre"),
    // mass
    u!("mg", 1e-6, M, "milligram"),
    u!("g", 1e-3, M, "gram"),
    u!("kg", 1.0, M, "kilogram"),
    // time
    u!("ns", 1e-9, T, "nanosecond"),
    u!("us", 1e-6, T, "microsecond"),
    u!("µs", 1e-6, T, "microsecond"),
    u!("ms", 1e-3, T, "millisecond"),
    u!("s", 1.0, T, "second"),
    u!("min", 60.0, T, "minute"),
    u!("h", 3600.0, T, "hour"),
    u!("d", 86_400.0, T, "day"),
    u!("y", YEAR, T, "year (Julian)"),
    u!("yr", YEAR, T, "year (Julian)"),
    // energy
    u!("eV", EV, E, "electronvolt"),
    u!("keV", 1e3 * EV, E, "kiloelectronvolt"),
    u!("MeV", 1e6 * EV, E, "megaelectronvolt"),
    u!("GeV", 1e9 * EV, E, "gigaelectronvolt"),
    u!("J", 1.0, E, "joule"),
    // activity
    u!("Bq", 1.0, ACT, "becquerel"),
    u!("kBq", 1e3, ACT, "kilobecquerel"),
    u!("MBq", 1e6, ACT, "megabecquerel"),
    u!("GBq", 1e9, ACT, "gigabecquerel"),
    u!("TBq", 1e12, ACT, "terabecquerel"),
    u!("uCi", 1e-6 * CI, ACT, "microcurie"),
    u!("µCi", 1e-6 * CI, ACT, "microcurie"),
    u!("mCi", 1e-3 * CI, ACT, "millicurie"),
    u!("Ci", CI, ACT, "curie"),
    // absorbed dose
    u!("uGy", 1e-6, DOSE, "microgray"),
    u!("µGy", 1e-6, DOSE, "microgray"),
    u!("mGy", 1e-3, DOSE, "milligray"),
    u!("cGy", 1e-2, DOSE, "centigray"),
    u!("Gy", 1.0, DOSE, "gray"),
    // angle
    u!("mrad", 1e-3, A, "milliradian"),
    u!("rad", 1.0, A, "radian"),
    u!("deg", std::f64::consts::PI / 180.0, A, "degree"),
    u!("°", std::f64::consts::PI / 180.0, A, "degree"),
    // pure numbers
    u!("%", 1e-2, NONE, "percent"),
];
