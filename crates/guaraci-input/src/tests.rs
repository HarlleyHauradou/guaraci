use super::*;
use crate::ast::ValueKind;
use crate::config::*;

const MATERIALS: Builtin = Builtin {
    name: "materials",
    text: "[material Water]\ndensity = 1 g/cm3\nformula = H2O\n\n[material Air]\ndensity = 0.0012048\ncomposition = { C = 0.000124, N = 0.755268, O = 0.231781, Ar = 0.012827 }\n",
};

const MAIN: &str = r#"
# a lesion inside a water cylinder
[run]
histories = 1e7
seed = 12345

[world]
material = Air
size = [1, 1, 1] m

[volume phantom]
shape = cylinder
radius = 10            # cm by default
height = 20 cm
material = Water

[volume lesion]
parent = phantom
shape = sphere
radius = 15 mm
center = [3, 0, 0]
material = Water

[source]
nuclide = Tc99m
in = lesion
activity = 370 MBq

[probe dose]
type = dose_grid
in = phantom
voxel = [2 mm, 2MM, 0.25 cm]
"#;

fn config(text: &str) -> Result<Config, Vec<Diagnostic>> {
    let prog = load_str("main.gci", text, &[MATERIALS]).map_err(|(_, d)| d)?;
    Config::from_program(&prog)
}

fn first_error(text: &str) -> String {
    let errs = config(text).expect_err("expected an error");
    errs.iter().find(|d| d.is_error()).unwrap().message.clone()
}

#[test]
fn parses_example() {
    let c = config(MAIN).unwrap();
    assert_eq!(c.run.histories, 10_000_000);
    assert_eq!(c.run.seed, 12345);
    assert_eq!(c.world.size, [100.0, 100.0, 100.0]);
    let lesion = c.volume("lesion").unwrap();
    assert_eq!(lesion.shape, Shape::Sphere { radius: 1.5 });
    assert_eq!(lesion.center, [3.0, 0.0, 0.0]);
    assert_eq!(
        c.volume("phantom").unwrap().shape,
        Shape::Cylinder {
            radius: 10.0,
            height: 20.0
        }
    );
    let s = &c.sources[0];
    assert!((s.activity.unwrap() - 370e6).abs() < 1e-3);
    match &c.probes[0].kind {
        ProbeKind::DoseGrid { voxel, .. } => {
            for (a, b) in voxel.iter().zip([0.2, 0.2, 0.25]) {
                assert!((a - b).abs() < 1e-12);
            }
        }
        _ => panic!(),
    }
    assert_eq!(c.probes[0].output, "dose.mhd");
    // defaults are recorded for the report
    assert!(c
        .defaults
        .iter()
        .any(|d| d.key == "rng" && d.value == "philox"));
    // `radius = 10` and `center = [3, 0, 0]` used the default unit
    assert!(c.implicit_units.len() >= 4);
}

#[test]
fn number_forms() {
    let f = parser::parse(
        0,
        "[x]\na = 10cm\nb = -1.5e3 keV\nc = 10_000\nd = [1, 2, 3] mm\ne = 1.05 g/cm3\nf = 89 %\n",
    )
    .unwrap();
    let Item::Section(s) = &f.items[0] else {
        panic!()
    };
    let get = |k: &str| {
        &s.entries
            .iter()
            .find(|e| e.key.node == k)
            .unwrap()
            .value
            .kind
    };
    assert!(
        matches!(get("a"), ValueKind::Number { value, unit: Some(u) } if *value == 10.0 && u.node == "cm")
    );
    assert!(
        matches!(get("b"), ValueKind::Number { value, unit: Some(u) } if *value == -1500.0 && u.node == "keV")
    );
    assert!(matches!(get("c"), ValueKind::Number { value, unit: None } if *value == 10000.0));
    assert!(
        matches!(get("d"), ValueKind::Array { items, unit: Some(u) } if items.len() == 3 && u.node == "mm")
    );
    assert!(matches!(get("e"), ValueKind::Number { unit: Some(u), .. } if u.node == "g/cm3"));
    assert!(matches!(get("f"), ValueKind::Number { unit: Some(u), .. } if u.node == "%"));
}

#[test]
fn multiline_list_and_comments() {
    let f = parser::parse(0, "[x]\na = [\n  1,  # one\n  2,\n] cm\n").unwrap();
    let Item::Section(s) = &f.items[0] else {
        panic!()
    };
    assert!(matches!(&s.entries[0].value.kind, ValueKind::Array { items, .. } if items.len() == 2));
}

#[test]
fn wrong_unit_kind() {
    let msg = first_error(&MAIN.replace("radius = 15 mm", "radius = 15 keV"));
    assert!(msg.contains("expected length"), "{msg}");
}

#[test]
fn unknown_key_suggests() {
    let errs = config(&MAIN.replace("radius = 15 mm", "radios = 15 mm")).unwrap_err();
    let d = errs
        .iter()
        .find(|d| d.message.contains("radios"))
        .expect("unknown key reported");
    assert_eq!(d.help.as_deref(), Some("did you mean `radius`?"));
}

#[test]
fn unknown_volume_suggests() {
    let errs = config(&MAIN.replace("in = lesion", "in = lesio")).unwrap_err();
    let d = errs.iter().find(|d| d.message.contains("lesio")).unwrap();
    assert_eq!(d.help.as_deref(), Some("did you mean `lesion`?"));
}

#[test]
fn unknown_material() {
    let msg = first_error(&MAIN.replace(
        "material = Water\n\n[volume lesion]",
        "material = Watr\n\n[volume lesion]",
    ));
    assert!(msg.contains("no material called `Watr`"), "{msg}");
}

#[test]
fn colon_instead_of_equals() {
    let msg = first_error("[run]\nhistories: 10\n");
    assert_eq!(msg, "use `=` instead of `:`");
}

#[test]
fn entry_outside_section() {
    let msg = first_error("histories = 10\n");
    assert!(msg.contains("outside any section"));
}

#[test]
fn reports_many_errors_at_once() {
    let text = MAIN
        .replace("radius = 15 mm", "radius = 15 keV")
        .replace("in = lesion", "in = nowhere");
    let errs = config(&text).unwrap_err();
    assert!(errs.len() >= 2, "{errs:#?}");
}

#[test]
fn user_material_overrides_builtin() {
    let text = format!("{MAIN}\n[material Water]\ndensity = 0.998\nformula = H2O\n");
    let c = config(&text).unwrap();
    assert!((c.material("Water").unwrap().density - 0.998).abs() < 1e-12);
}

#[test]
fn includes_from_disk() {
    let dir = std::env::temp_dir().join(format!("guaraci-inc-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("geometry.gci"),
        "[volume phantom]\nshape = sphere\nradius = 10\nmaterial = Water\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("main.gci"),
        "include geometry\n[world]\nmaterial = Air\nsize = 1 m\n[source]\nenergy = 140.5 keV\nin = phantom\n",
    )
    .unwrap();
    let prog = load(&dir.join("main"), &[MATERIALS])
        .map_err(|(_, d)| d)
        .unwrap();
    let c = Config::from_program(&prog).unwrap();
    assert_eq!(c.volumes.len(), 1);
    // include cycle
    std::fs::write(dir.join("a.gci"), "include b\n").unwrap();
    std::fs::write(dir.join("b.gci"), "include a\n").unwrap();
    let (_, d) = load(&dir.join("a"), &[]).unwrap_err();
    assert!(d[0].message.contains("includes itself"));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn error_render_points_at_text() {
    let text = MAIN.replace("radius = 15 mm", "radius = 15 kev2");
    let prog = load_str("main.gci", &text, &[MATERIALS]).unwrap();
    let errs = Config::from_program(&prog).unwrap_err();
    let out = errs[0].render(&prog.sources, false);
    assert!(out.contains("main.gci"), "{out}");
    assert!(out.contains("kev2"), "{out}");
}
