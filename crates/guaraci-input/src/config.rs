//! Typed configuration built from the sections of a [`Program`].
//!
//! Units inside the config are fixed: lengths in cm, energies in MeV,
//! densities in g/cm³, times in s, activities in Bq, angles in rad.

use crate::ast::{Section, Span, Spanned, ValueKind};
use crate::diag::Diagnostic;
use crate::reader::{quantity_of, suggest, DefaultNote, ImplicitUnit, Notes, Reader};
use crate::Program;
use guaraci_units::{Kind, Quantity};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Cpu,
    Cuda,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RngKind {
    Philox,
    Xoshiro,
    Pcg,
    Mt19937,
}

impl RngKind {
    pub fn name(self) -> &'static str {
        match self {
            RngKind::Philox => "Philox4x32-10",
            RngKind::Xoshiro => "Xoshiro256++",
            RngKind::Pcg => "PCG64",
            RngKind::Mt19937 => "MT19937",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Run {
    pub histories: u64,
    pub seed: u64,
    pub backend: Backend,
    pub rng: RngKind,
    /// 0 = all cores.
    pub threads: usize,
    pub batches: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComptonModel {
    /// Klein-Nishina on a free electron at rest.
    KleinNishina,
    /// Klein-Nishina corrected by the incoherent scattering function S(x,Z).
    KleinNishinaSf,
}

#[derive(Debug, Clone)]
pub struct Physics {
    pub list: String,
    pub data: String,
    pub data_path: Option<String>,
    pub photoelectric: bool,
    pub compton: Option<ComptonModel>,
    pub rayleigh: bool,
    pub pair: bool,
    /// Photons below this energy deposit locally (MeV).
    pub photon_cut: f64,
}

#[derive(Debug, Clone)]
pub struct World {
    pub material: Spanned<String>,
    /// Full side lengths (cm).
    pub size: [f64; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    /// Full side lengths (cm).
    Box {
        size: [f64; 3],
    },
    Sphere {
        radius: f64,
    },
    /// Axis along z; full height (cm).
    Cylinder {
        radius: f64,
        height: f64,
    },
    /// Semi-axes (cm).
    Ellipsoid {
        semi_axes: [f64; 3],
    },
}

#[derive(Debug, Clone)]
pub struct Volume {
    pub name: Spanned<String>,
    pub parent: Option<Spanned<String>>,
    pub shape: Shape,
    pub material: Spanned<String>,
    /// Centre relative to the parent's centre (cm).
    pub center: [f64; 3],
}

#[derive(Debug, Clone)]
pub enum Composition {
    /// Element symbol -> mass fraction (normalised later).
    MassFractions(Vec<(Spanned<String>, f64)>),
    /// Element symbol -> number of atoms, e.g. `H2O`.
    Formula(Spanned<String>),
}

#[derive(Debug, Clone)]
pub struct MaterialDef {
    pub name: Spanned<String>,
    /// g/cm³
    pub density: f64,
    pub composition: Composition,
    pub builtin: bool,
}

#[derive(Debug, Clone)]
pub enum Emission {
    Nuclide(Spanned<String>),
    /// (energy MeV, photons per decay)
    Lines(Vec<(f64, f64)>),
}

#[derive(Debug, Clone)]
pub enum Placement {
    /// Uniformly inside a volume.
    In(Spanned<String>),
    /// A point (cm, world coordinates).
    Point([f64; 3]),
}

#[derive(Debug, Clone)]
pub struct Source {
    pub name: String,
    pub span: Span,
    pub emission: Emission,
    pub placement: Placement,
    /// Bq
    pub activity: Option<f64>,
    /// s; with `activity`, gives the number of decays
    pub duration: Option<f64>,
}

#[derive(Debug, Clone)]
pub enum ProbeKind {
    /// 3D dose map over the bounding box of a volume.
    DoseGrid {
        region: Spanned<String>,
        voxel: [f64; 3],
    },
    /// Energy and mean dose in whole volumes.
    VolumeDose { volumes: Vec<Spanned<String>> },
    /// Photon fluence spectrum in a volume (track-length estimator).
    Spectrum {
        region: Spanned<String>,
        emax: f64,
        bins: usize,
    },
}

#[derive(Debug, Clone)]
pub struct Probe {
    pub name: Spanned<String>,
    pub kind: ProbeKind,
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub run: Run,
    pub physics: Physics,
    pub world: World,
    pub volumes: Vec<Volume>,
    pub materials: Vec<MaterialDef>,
    pub sources: Vec<Source>,
    pub probes: Vec<Probe>,
    /// Values the user did not write (shown in `check` and the report).
    pub defaults: Vec<DefaultNote>,
    /// Numbers written without a unit that took the default unit.
    pub implicit_units: Vec<ImplicitUnit>,
    pub warnings: Vec<Diagnostic>,
}

const SECTIONS: &[&str] = &[
    "run", "physics", "world", "volume", "material", "source", "probe",
];

fn cm(q: Quantity) -> f64 {
    q.to("cm")
}
fn cm3(q: [Quantity; 3]) -> [f64; 3] {
    [cm(q[0]), cm(q[1]), cm(q[2])]
}

impl Config {
    /// Build the typed configuration, reporting every problem found.
    pub fn from_program(prog: &Program) -> Result<Config, Vec<Diagnostic>> {
        let mut n = Notes::default();
        let mut run = None;
        let mut physics = None;
        let mut world = None;
        let mut volumes = Vec::new();
        let mut materials: Vec<MaterialDef> = Vec::new();
        let mut sources = Vec::new();
        let mut probes = Vec::new();
        let mut singles: HashMap<&str, Span> = HashMap::new();

        for s in &prog.sections {
            let kind = s.kind.node.as_str();
            let builtin = prog.is_builtin(s.header_span);
            if matches!(kind, "run" | "physics" | "world") {
                if let Some(first) = singles.get(kind) {
                    n.error(
                        Diagnostic::error(format!("`[{kind}]` appears more than once"))
                            .at(s.header_span)
                            .related(*first, "first one")
                            .help(format!(
                                "merge both into a single `[{kind}]` section (first at {})",
                                prog.sources.location(*first)
                            )),
                    );
                    continue;
                }
                singles.insert(kind, s.header_span);
                if let Some(name) = &s.name {
                    n.error(
                        Diagnostic::error(format!("`[{kind}]` does not take a name"))
                            .at(name.span)
                            .help(format!("write just `[{kind}]`")),
                    );
                }
            } else if SECTIONS.contains(&kind) && s.name.is_none() && kind != "source" {
                n.error(
                    Diagnostic::error(format!("`[{kind}]` needs a name"))
                        .at(s.header_span)
                        .help(format!("for example `[{kind} {}]`", example_name(kind))),
                );
                continue;
            }
            match kind {
                "run" => run = Some(read_run(s, &mut n)),
                "physics" => physics = Some(read_physics(s, &mut n)),
                "world" => world = read_world(s, &mut n),
                "volume" => volumes.extend(read_volume(s, &mut n)),
                "material" => {
                    if let Some(m) = read_material(s, builtin, &mut n) {
                        // user definitions replace built-in ones with the same name
                        if let Some(i) = materials.iter().position(|o| o.name.node == m.name.node) {
                            if materials[i].builtin {
                                materials[i] = m;
                            } else {
                                n.error(
                                    Diagnostic::error(format!(
                                        "material `{}` is defined twice",
                                        m.name.node
                                    ))
                                    .at(m.name.span)
                                    .related(materials[i].name.span, "first definition")
                                    .help(format!(
                                        "first defined at {}",
                                        prog.sources.location(materials[i].name.span)
                                    )),
                                );
                            }
                        } else {
                            materials.push(m);
                        }
                    }
                }
                "source" => sources.extend(read_source(s, sources.len(), &mut n)),
                "probe" => probes.extend(read_probe(s, &mut n)),
                other => {
                    let mut d =
                        Diagnostic::error(format!("unknown section `[{other}]`")).at(s.kind.span);
                    d = match suggest(other, SECTIONS.iter().copied()) {
                        Some(sug) => d.help(format!("did you mean `[{sug}]`?")),
                        None => d.help(format!("sections are: {}", SECTIONS.join(", "))),
                    };
                    n.error(d);
                }
            }
        }

        let run = run.unwrap_or_else(|| read_run(&empty("run"), &mut n));
        let physics = physics.unwrap_or_else(|| read_physics(&empty("physics"), &mut n));
        if world.is_none() && !singles.contains_key("world") {
            n.error(Diagnostic::error("the simulation has no `[world]`").help(
                "add the box that contains everything, e.g.\n[world]\nmaterial = Air\nsize = [1, 1, 1] m",
            ));
        }
        if sources.is_empty() {
            n.error(
                Diagnostic::error("the simulation has no `[source]`")
                    .help("add one, e.g.\n[source]\nnuclide = Tc99m\nin = lesion"),
            );
        }

        check_references(
            &volumes,
            &materials,
            &sources,
            &probes,
            world.as_ref(),
            &mut n,
        );

        if n.has_errors() {
            return Err(n.diags);
        }
        let (errors, warnings): (Vec<_>, Vec<_>) = n.diags.into_iter().partition(|d| d.is_error());
        debug_assert!(errors.is_empty());
        Ok(Config {
            run,
            physics,
            world: world.expect("checked above"),
            volumes,
            materials,
            sources,
            probes,
            defaults: n.defaults,
            implicit_units: n.implicit_units,
            warnings,
        })
    }

    pub fn material(&self, name: &str) -> Option<&MaterialDef> {
        self.materials.iter().find(|m| m.name.node == name)
    }
    pub fn volume(&self, name: &str) -> Option<&Volume> {
        self.volumes.iter().find(|v| v.name.node == name)
    }
}

fn example_name(kind: &str) -> &'static str {
    match kind {
        "volume" => "phantom",
        "material" => "MyTissue",
        "probe" => "dose",
        _ => "name",
    }
}

fn empty(kind: &str) -> Section {
    let span = Span {
        file: 0,
        start: 0,
        end: 0,
    };
    Section {
        kind: Spanned {
            node: kind.into(),
            span,
        },
        name: None,
        entries: vec![],
        header_span: span,
    }
}

fn read_run(s: &Section, n: &mut Notes) -> Run {
    let mut r = Reader::new(s);
    let histories = r.count_or("histories", 1_000_000, n);
    let seed = r.count_or("seed", 1, n);
    let backend = match r.choice("backend", &["cpu", "cuda"], "cpu", n) {
        "cuda" => Backend::Cuda,
        _ => Backend::Cpu,
    };
    let rng = match r.choice("rng", &["philox", "xoshiro", "pcg", "mt19937"], "philox", n) {
        "xoshiro" => RngKind::Xoshiro,
        "pcg" => RngKind::Pcg,
        "mt19937" => RngKind::Mt19937,
        _ => RngKind::Philox,
    };
    let threads = r.count_or("threads", 0, n) as usize;
    let batches = r.count_or("batches", 10, n).max(1) as u32;
    if batches < 2 {
        n.diags.push(
            Diagnostic::warning(
                "with fewer than 2 batches no statistical uncertainty can be estimated",
            )
            .at(s.header_span),
        );
    }
    r.finish(n);
    Run {
        histories,
        seed,
        backend,
        rng,
        threads,
        batches,
    }
}

fn read_physics(s: &Section, n: &mut Notes) -> Physics {
    let mut r = Reader::new(s);
    let list = r
        .choice("list", &["photon_local"], "photon_local", n)
        .to_string();
    let data = r
        .choice("data", &["epics2017", "endfb8", "epdl97"], "epics2017", n)
        .to_string();
    let data_path = r.text("data_path", n).map(|t| t.node);
    // Process switches: the list sets the defaults, the user may override them.
    let photoelectric = r.bool_or("photoelectric", true, n);
    let rayleigh = r.bool_or("rayleigh", true, n);
    let pair = r.bool_or("pair", true, n);
    let compton = match r.choice(
        "compton",
        &["klein_nishina_sf", "klein_nishina", "off"],
        "klein_nishina_sf",
        n,
    ) {
        "klein_nishina" => Some(ComptonModel::KleinNishina),
        "off" => None,
        _ => Some(ComptonModel::KleinNishinaSf),
    };
    let photon_cut = r.quantity_or("photon_cut", Kind::Energy, 1.0, n).to("MeV");
    let particles = r.text_list("particles", n);
    if let Some(ps) = particles {
        for p in ps {
            if !matches!(p.node.as_str(), "photon" | "gamma") {
                n.error(
                    Diagnostic::error(format!("particle `{}` is not transported yet", p.node))
                        .at(p.span)
                        .help("this version follows photons only; electrons deposit their energy locally"),
                );
            }
        }
    }
    r.finish(n);
    Physics {
        list,
        data,
        data_path,
        photoelectric,
        compton,
        rayleigh,
        pair,
        photon_cut,
    }
}

fn read_world(s: &Section, n: &mut Notes) -> Option<World> {
    let mut r = Reader::new(s);
    let material = r.require_text("material", "Air", n);
    let size = r.require_vec3("size", Kind::Length, n);
    r.finish(n);
    let size = cm3(size?);
    if size.iter().any(|&x| x <= 0.0) {
        n.error(Diagnostic::error("world size must be positive").at(s.header_span));
    }
    Some(World {
        material: material?,
        size,
    })
}

fn read_shape(r: &mut Reader, n: &mut Notes) -> Option<Shape> {
    let shape = r.require_text("shape", "sphere", n)?;
    let shape_name = shape.node.to_lowercase();
    let sec = r.section;
    let positive = |q: Option<Quantity>, key: &str, n: &mut Notes| -> Option<f64> {
        let v = cm(q?);
        if v <= 0.0 {
            let span = sec
                .entries
                .iter()
                .find(|e| e.key.node == key)
                .map(|e| e.value.span)
                .unwrap_or(sec.header_span);
            n.error(Diagnostic::error(format!("`{key}` must be greater than zero")).at(span));
            return None;
        }
        Some(v)
    };
    match shape_name.as_str() {
        "box" | "cube" => {
            let size = r.require_vec3("size", Kind::Length, n);
            Some(Shape::Box { size: cm3(size?) })
        }
        "sphere" => {
            let q = r.require_quantity("radius", Kind::Length, n);
            Some(Shape::Sphere {
                radius: positive(q, "radius", n)?,
            })
        }
        "cylinder" => {
            let rq = r.require_quantity("radius", Kind::Length, n);
            let hq = r.require_quantity("height", Kind::Length, n);
            let radius = positive(rq, "radius", n);
            let height = positive(hq, "height", n);
            Some(Shape::Cylinder {
                radius: radius?,
                height: height?,
            })
        }
        "ellipsoid" => {
            let a = r.require_vec3("semi_axes", Kind::Length, n);
            Some(Shape::Ellipsoid { semi_axes: cm3(a?) })
        }
        _ => {
            let opts = ["box", "sphere", "cylinder", "ellipsoid"];
            n.error(
                Diagnostic::error(format!("unknown shape `{}`", shape.node))
                    .at(shape.span)
                    .label(format!("shapes: {}", opts.join(", ")))
                    .maybe_help(
                        suggest(&shape.node, opts.iter().copied())
                            .map(|s| format!("did you mean `{s}`?")),
                    ),
            );
            None
        }
    }
}

fn r_span(r: &Reader, key: &str) -> Span {
    r.section
        .entries
        .iter()
        .find(|e| e.key.node == key)
        .map(|e| e.value.span)
        .unwrap_or(r.section.header_span)
}

fn read_volume(s: &Section, n: &mut Notes) -> Option<Volume> {
    let name = s.name.clone()?;
    let mut r = Reader::new(s);
    let parent = r.text("parent", n);
    let shape = read_shape(&mut r, n);
    let material = r.require_text("material", "Water", n);
    let center = r
        .vec3("center", Kind::Length, n)
        .map(cm3)
        .unwrap_or([0.0; 3]);
    r.finish(n);
    if name.node.eq_ignore_ascii_case("world") {
        n.error(Diagnostic::error("`world` is reserved for the `[world]` section").at(name.span));
    }
    Some(Volume {
        name,
        parent,
        shape: shape?,
        material: material?,
        center,
    })
}

fn read_material(s: &Section, builtin: bool, n: &mut Notes) -> Option<MaterialDef> {
    let name = s.name.clone()?;
    let mut r = Reader::new(s);
    let density = r
        .require_quantity("density", Kind::Density, n)
        .map(|q| q.to("g/cm3"));
    let has_comp = r.has("composition");
    let has_formula = r.has("formula");
    let composition = match (has_comp, has_formula) {
        (true, true) => {
            n.error(
                Diagnostic::error(format!(
                    "material `{}` has both `composition` and `formula`",
                    name.node
                ))
                .at(s.header_span)
                .help("use only one: `formula = H2O` or `composition = { H = 0.112, O = 0.888 }`"),
            );
            None
        }
        (false, false) => {
            n.error(
                Diagnostic::error(format!("material `{}` needs `formula` or `composition`", name.node))
                    .at(s.header_span)
                    .help("e.g. `formula = H2O` or `composition = { H = 0.112, O = 0.888 }` (mass fractions)"),
            );
            None
        }
        (false, true) => r.text("formula", n).map(Composition::Formula),
        (true, false) => {
            let e = r.entry("composition", n)?;
            match &e.value.kind {
                ValueKind::Table(entries) => {
                    let mut out = Vec::new();
                    for en in entries {
                        if let Some(q) =
                            quantity_of(&en.value, None, Kind::Dimensionless, &en.key.node, n)
                        {
                            if q.si <= 0.0 {
                                n.error(
                                    Diagnostic::error("mass fractions must be greater than zero")
                                        .at(en.value.span),
                                );
                            }
                            out.push((en.key.clone(), q.si));
                        }
                    }
                    Some(Composition::MassFractions(out))
                }
                other => {
                    n.error(
                        Diagnostic::error(format!(
                            "`composition` should be a table, not {}",
                            other.describe()
                        ))
                        .at(e.value.span)
                        .help("write `composition = { H = 0.112, O = 0.888 }`"),
                    );
                    None
                }
            }
        }
    };
    r.finish(n);
    let density = density?;
    if density <= 0.0 {
        n.error(Diagnostic::error("density must be greater than zero").at(s.header_span));
    }
    Some(MaterialDef {
        name,
        density,
        composition: composition?,
        builtin,
    })
}

fn read_source(s: &Section, index: usize, n: &mut Notes) -> Option<Source> {
    let mut r = Reader::new(s);
    let name = s
        .name
        .as_ref()
        .map(|n| n.node.clone())
        .unwrap_or_else(|| format!("source{}", index + 1));
    let nuclide = r.text("nuclide", n);
    let energies = r.quantity_list("energy", Kind::Energy, n);
    let intensities = r.quantity_list("intensity", Kind::Dimensionless, n);
    let emission = match (nuclide, energies) {
        (Some(nu), None) => {
            if intensities.is_some() {
                n.error(
                    Diagnostic::error("`intensity` goes with `energy`, not with `nuclide`")
                        .at(r_span(&r, "intensity")),
                );
            }
            Some(Emission::Nuclide(nu))
        }
        (None, Some(es)) => {
            let ints = intensities.unwrap_or_else(|| {
                vec![
                    Quantity {
                        si: 1.0,
                        dim: guaraci_units::Dim::NONE
                    };
                    es.len()
                ]
            });
            if ints.len() != es.len() {
                n.error(
                    Diagnostic::error(format!(
                        "{} energies but {} intensities",
                        es.len(),
                        ints.len()
                    ))
                    .at(r_span(&r, "intensity"))
                    .help("give one intensity (photons per decay) for each energy"),
                );
                None
            } else {
                Some(Emission::Lines(
                    es.iter()
                        .zip(&ints)
                        .map(|(e, i)| (e.to("MeV"), i.si))
                        .collect(),
                ))
            }
        }
        (Some(nu), Some(_)) => {
            n.error(
                Diagnostic::error("give either `nuclide` or `energy`, not both")
                    .at(nu.span)
                    .help("`nuclide = Tc99m` uses the nuclide's emission lines; `energy = 140.5 keV` a single line"),
            );
            None
        }
        (None, None) => {
            n.error(
                Diagnostic::error(format!("{} needs `nuclide` or `energy`", r.title()))
                    .at(s.header_span)
                    .help("e.g. `nuclide = Tc99m` or `energy = 140.5 keV`"),
            );
            None
        }
    };
    let inside = r.text("in", n);
    let position = r.vec3("position", Kind::Length, n);
    let placement = match (inside, position) {
        (Some(v), None) => Some(Placement::In(v)),
        (None, Some(p)) => Some(Placement::Point(cm3(p))),
        (Some(v), Some(_)) => {
            n.error(Diagnostic::error("give either `in` or `position`, not both").at(v.span));
            None
        }
        (None, None) => {
            n.error(
                Diagnostic::error(format!("where is {}?", r.title()))
                    .at(s.header_span)
                    .help("add `in = <volume>` (spread over a volume) or `position = [x, y, z]` (a point)"),
            );
            None
        }
    };
    let activity = r
        .quantity("activity", Kind::Activity, n)
        .map(|q| q.to("Bq"));
    let duration = r.quantity("duration", Kind::Time, n).map(|q| q.to("s"));
    if duration.is_some() && activity.is_none() {
        n.error(Diagnostic::error("`duration` needs an `activity`").at(r_span(&r, "duration")));
    }
    r.finish(n);
    Some(Source {
        name,
        span: s.header_span,
        emission: emission?,
        placement: placement?,
        activity,
        duration,
    })
}

fn read_probe(s: &Section, n: &mut Notes) -> Option<Probe> {
    let name = s.name.clone()?;
    let mut r = Reader::new(s);
    let kind_word = r.require_text("type", "dose_grid", n)?;
    let types = ["dose_grid", "volume_dose", "spectrum"];
    let kind = match kind_word.node.as_str() {
        "dose_grid" => {
            let region = r.require_text("in", "phantom", n);
            let voxel = r.require_vec3("voxel", Kind::Length, n).map(cm3);
            if let Some(v) = voxel {
                if v.iter().any(|&x| x <= 0.0) {
                    n.error(
                        Diagnostic::error("voxel size must be greater than zero")
                            .at(r_span(&r, "voxel")),
                    );
                }
            }
            Some(ProbeKind::DoseGrid {
                region: region?,
                voxel: voxel?,
            })
        }
        "volume_dose" => {
            let vols = r.text_list("volumes", n);
            if vols.is_none() && !r.has("volumes") {
                n.error(
                    Diagnostic::error(format!("{} needs `volumes`", r.title()))
                        .at(s.header_span)
                        .help("e.g. `volumes = [lesion, phantom]`"),
                );
            }
            Some(ProbeKind::VolumeDose { volumes: vols? })
        }
        "spectrum" => {
            let region = r.require_text("in", "phantom", n);
            let emax = r.quantity_or("emax", Kind::Energy, 1000.0, n).to("MeV");
            let bins = r.count_or("bins", 200, n).max(1) as usize;
            Some(ProbeKind::Spectrum {
                region: region?,
                emax,
                bins,
            })
        }
        other => {
            n.error(
                Diagnostic::error(format!("unknown probe type `{other}`"))
                    .at(kind_word.span)
                    .label(format!("types: {}", types.join(", ")))
                    .maybe_help(
                        suggest(other, types.iter().copied())
                            .map(|s| format!("did you mean `{s}`?")),
                    ),
            );
            None
        }
    };
    let default_out = match kind {
        Some(ProbeKind::DoseGrid { .. }) => format!("{}.mhd", name.node),
        _ => format!("{}.csv", name.node),
    };
    let output = r.text("output", n).map(|t| t.node).unwrap_or(default_out);
    r.finish(n);
    Some(Probe {
        name,
        kind: kind?,
        output,
    })
}

fn check_references(
    volumes: &[Volume],
    materials: &[MaterialDef],
    sources: &[Source],
    probes: &[Probe],
    world: Option<&World>,
    n: &mut Notes,
) {
    let vol_names: Vec<&str> = volumes.iter().map(|v| v.name.node.as_str()).collect();
    let mat_names: Vec<&str> = materials.iter().map(|m| m.name.node.as_str()).collect();

    let check_vol = |name: &Spanned<String>, n: &mut Notes, allow_world: bool| {
        if allow_world && name.node == "world" {
            return;
        }
        if !vol_names.contains(&name.node.as_str()) {
            let mut d = Diagnostic::error(format!("there is no volume called `{}`", name.node))
                .at(name.span)
                .label("unknown volume");
            d = match suggest(&name.node, vol_names.iter().copied()) {
                Some(s) => d.help(format!("did you mean `{s}`?")),
                None if vol_names.is_empty() => {
                    d.help("define it with a `[volume <name>]` section")
                }
                None => d.help(format!("volumes: {}", vol_names.join(", "))),
            };
            n.error(d);
        }
    };
    let check_mat = |name: &Spanned<String>, n: &mut Notes| {
        if !mat_names.contains(&name.node.as_str()) {
            let mut d = Diagnostic::error(format!("there is no material called `{}`", name.node))
                .at(name.span)
                .label("unknown material");
            d = match suggest(&name.node, mat_names.iter().copied()) {
                Some(s) => d.help(format!("did you mean `{s}`?")),
                None => d.help(
                    "define it with a `[material <name>]` section, or run `guaraci info materials`",
                ),
            };
            n.error(d);
        }
    };

    if let Some(w) = world {
        check_mat(&w.material, n);
    }
    for (i, v) in volumes.iter().enumerate() {
        if let Some(prev) = volumes[..i].iter().find(|o| o.name.node == v.name.node) {
            n.error(
                Diagnostic::error(format!("volume `{}` is defined twice", v.name.node))
                    .at(v.name.span)
                    .related(prev.name.span, "first definition"),
            );
        }
        check_mat(&v.material, n);
        if let Some(p) = &v.parent {
            check_vol(p, n, true);
            if p.node == v.name.node {
                n.error(Diagnostic::error("a volume cannot be its own parent").at(p.span));
            }
        }
    }
    // parent cycles
    for v in volumes {
        let mut seen = vec![v.name.node.as_str()];
        let mut cur = v;
        while let Some(p) = &cur.parent {
            let Some(next) = volumes.iter().find(|o| o.name.node == p.node) else {
                break;
            };
            if seen.contains(&next.name.node.as_str()) {
                n.error(
                    Diagnostic::error(format!(
                        "volumes `{}` are each other's parents",
                        seen.join("`, `")
                    ))
                    .at(v.name.span),
                );
                break;
            }
            seen.push(&next.name.node);
            cur = next;
        }
    }
    for s in sources {
        if let Placement::In(v) = &s.placement {
            check_vol(v, n, true);
        }
    }
    for (i, p) in probes.iter().enumerate() {
        if let Some(prev) = probes[..i].iter().find(|o| o.name.node == p.name.node) {
            n.error(
                Diagnostic::error(format!("probe `{}` is defined twice", p.name.node))
                    .at(p.name.span)
                    .related(prev.name.span, "first definition"),
            );
        }
        match &p.kind {
            ProbeKind::DoseGrid { region, .. } | ProbeKind::Spectrum { region, .. } => {
                check_vol(region, n, true)
            }
            ProbeKind::VolumeDose { volumes } => volumes.iter().for_each(|v| check_vol(v, n, true)),
        }
    }
}
