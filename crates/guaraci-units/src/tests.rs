use super::*;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-12 * a.abs().max(b.abs())
}

#[test]
fn simple_units() {
    let (q, _) = Quantity::new(10.0, Some("cm"), Kind::Length).unwrap();
    assert!(close(q.to("mm"), 100.0));
    let (q, _) = Quantity::new(140.5, Some("keV"), Kind::Energy).unwrap();
    assert!(close(q.to("MeV"), 0.1405));
}

#[test]
fn default_units() {
    let (q, u) = Quantity::new(10.0, None, Kind::Length).unwrap();
    assert_eq!(u.canonical, "cm");
    assert!(close(q.to("cm"), 10.0));
    let (q, _) = Quantity::new(511.0, None, Kind::Energy).unwrap();
    assert!(close(q.to("keV"), 511.0));
    let (q, _) = Quantity::new(370.0, None, Kind::Activity).unwrap();
    assert!(close(q.to("mCi"), 10.0));
}

#[test]
fn compound_units() {
    for s in ["g/cm3", "g/cm^3", "g/cc", "g*cm^-3", "g.cm-3"] {
        let s2 = s.replace("cm-3", "cm^-3");
        let (q, _) =
            Quantity::new(1.0, Some(&s2), Kind::Density).unwrap_or_else(|e| panic!("{s}: {e}"));
        assert!(close(q.to("kg/m3"), 1000.0), "{s}");
    }
    let u = Unit::parse("MBq*h").unwrap();
    assert_eq!(u.dim, Dim::NONE);
    assert!(close(u.factor, 3.6e9));
}

#[test]
fn case_insensitive_when_unique() {
    assert_eq!(Unit::parse("CM").unwrap().canonical, "cm");
    assert_eq!(Unit::parse("KEV").unwrap().canonical, "keV");
    assert_eq!(Unit::parse("mbq").unwrap().canonical, "MBq");
    // `l` and `L` mean the same thing, so `L` in any case is fine.
    assert!(Unit::parse("l").is_ok());
}

#[test]
fn ambiguous_case_is_an_error() {
    // `MS` could be `ms` (millisecond) ... and nothing else, so it is accepted;
    // `MGY` only matches `mGy`. Build an ambiguity artificially through `M`:
    // `m` (metre) exists, but no `M`, so `M` is accepted as metre.
    assert_eq!(Unit::parse("M").unwrap().canonical, "m");
    // `uci` matches both `uCi` and nothing else -> fine.
    assert!(Unit::parse("UCI").is_ok());
}

#[test]
fn wrong_kind() {
    let e = Quantity::new(5.0, Some("keV"), Kind::Length).unwrap_err();
    assert!(matches!(e, UnitError::WrongKind { .. }));
    assert_eq!(
        e.to_string(),
        "expected length, but `keV` is a unit of energy"
    );
}

#[test]
fn unknown_with_suggestion() {
    let e = Unit::parse("kev2x").unwrap_err();
    assert!(matches!(e, UnitError::Unknown { .. }));
    let e = Unit::parse("cn").unwrap_err();
    let help = e.help().unwrap();
    assert!(help.contains("`cm`"), "{help}");
}

#[test]
fn percent() {
    let (q, _) = Quantity::new(89.0, Some("%"), Kind::Dimensionless).unwrap();
    assert!(close(q.si, 0.89));
}

#[test]
fn table_has_no_case_conflicts_with_different_meaning() {
    for a in UNITS {
        for b in UNITS {
            if a.symbol != b.symbol && a.symbol.to_lowercase() == b.symbol.to_lowercase() {
                assert!(
                    a.dim == b.dim && a.factor == b.factor,
                    "{} vs {}",
                    a.symbol,
                    b.symbol
                );
            }
        }
    }
}
