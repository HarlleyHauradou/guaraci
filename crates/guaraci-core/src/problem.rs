//! A complete, validated simulation: geometry, materials, physics tables,
//! sources and probes, built from a [`Config`] and the nuclear data.

use crate::electron::{BetaSpectrum, ElectronMaterial};
use crate::geometry::{Geometry, Solid, Volume};
use crate::physics::{Component, ElementSampling, Grid, Material, Processes, AVOGADRO};
use crate::vec3::Vec3;
use guaraci_data::decay::BetaKind;
use guaraci_data::elements::{symbol, z_of};
use guaraci_data::library::{DecayData, ElectronData, PhotonData};
use guaraci_data::{NuclideId, PhotonKind};
use guaraci_input::config::{
    BetaMode, Composition, Config, Emission, MaterialDef, Placement, ProbeKind, Shape, SourceParticle,
};
use guaraci_input::{Diagnostic, Span};
use std::collections::BTreeMap;

/// MeV -> J
pub const MEV_TO_J: f64 = 1.602_176_634e-13;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineKind {
    Single,
    /// Emitted in back-to-back pairs (positron annihilation).
    Annihilation,
    /// A positron line (only in `Source::electron_lines`).
    Positron,
}

#[derive(Debug, Clone)]
pub struct Line {
    pub energy: f64,
    /// Photons per decay.
    pub intensity: f64,
    pub kind: LineKind,
}

#[derive(Debug, Clone)]
pub enum Where {
    Point(Vec3),
    /// Uniform in the region of a volume (its solid minus its daughters).
    Region {
        volume: usize,
    },
}

/// A β branch ready for sampling.
#[derive(Debug, Clone)]
pub struct BetaEmitter {
    pub spectrum: BetaSpectrum,
    /// Particles per decay.
    pub intensity: f64,
    pub positron: bool,
}

#[derive(Debug, Clone)]
pub struct Source {
    pub name: String,
    pub label: String,
    pub lines: Vec<Line>,
    /// β branches emitted as particles (electron transport on).
    pub betas: Vec<BetaEmitter>,
    /// Conversion and Auger electrons, and electron or positron sources,
    /// emitted as particles (`LineKind::Positron` marks positrons).
    pub electron_lines: Vec<Line>,
    /// Energy deposited at the decay point, MeV per decay.
    pub local_energy: f64,
    pub place: Where,
    /// Relative weight used to choose the source of each decay.
    pub weight: f64,
}

/// How results per simulated decay are converted to the reported quantity.
#[derive(Debug, Clone, PartialEq)]
pub enum Normalization {
    /// Results per decay.
    PerDecay,
    /// × total activity (Bq): rates per second.
    PerSecond { activity: f64 },
    /// × number of decays.
    Total { decays: f64 },
}

#[derive(Debug, Clone)]
pub enum Probe {
    DoseGrid {
        name: String,
        output: String,
        origin: Vec3,
        spacing: Vec3,
        dims: [usize; 3],
        mass_kg: Vec<f64>,
    },
    VolumeDose {
        name: String,
        output: String,
        volumes: Vec<usize>,
        mass_kg: Vec<f64>,
    },
    Spectrum {
        name: String,
        output: String,
        region: usize,
        emax: f64,
        bins: usize,
        volume_cm3: f64,
    },
}

impl Probe {
    pub fn name(&self) -> &str {
        match self {
            Probe::DoseGrid { name, .. } | Probe::VolumeDose { name, .. } | Probe::Spectrum { name, .. } => name,
        }
    }
    pub fn output(&self) -> &str {
        match self {
            Probe::DoseGrid { output, .. } | Probe::VolumeDose { output, .. } | Probe::Spectrum { output, .. } => {
                output
            }
        }
    }
    /// Number of accumulator cells.
    pub fn cells(&self) -> usize {
        match self {
            Probe::DoseGrid { dims, .. } => dims[0] * dims[1] * dims[2],
            Probe::VolumeDose { volumes, .. } => volumes.len(),
            Probe::Spectrum { bins, .. } => *bins,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Problem {
    pub geometry: Geometry,
    pub grid: Grid,
    pub materials: Vec<Material>,
    pub elements: Vec<ElementSampling>,
    pub sources: Vec<Source>,
    /// Cumulative source weights, normalised to 1.
    pub source_cdf: Vec<f64>,
    pub probes: Vec<Probe>,
    pub normalization: Normalization,
    pub photon_cut: f64,
    pub compton_sf: bool,
    /// Per volume: (probe index, cell) of every volume-dose probe that scores it.
    pub volume_slots: Vec<Vec<(usize, usize)>>,
    /// Per volume: spectrum probes scoring track length there.
    pub spectrum_slots: Vec<Vec<usize>>,
    pub max_energy: f64,
    /// Electrons and positrons are transported.
    pub electrons: bool,
    pub electron_cut: f64,
    /// Energy grid of the electron tables.
    pub egrid: Grid,
    /// Electron tables, one per entry of `materials` (empty without electron transport).
    pub emats: Vec<ElectronMaterial>,
}

/// Mass fractions by Z from a material definition.
fn composition(m: &MaterialDef, photon: &PhotonData, errs: &mut Vec<Diagnostic>) -> Option<Vec<(u32, f64)>> {
    let mass = |z: u32| photon.get(&z).map(|e| e.atomic_mass);
    let mut by_z: BTreeMap<u32, f64> = BTreeMap::new();
    match &m.composition {
        Composition::MassFractions(list) => {
            for (sym, w) in list {
                let Some(z) = element_of(&sym.node) else {
                    errs.push(
                        Diagnostic::error(format!("`{}` is not a chemical element", sym.node))
                            .at(sym.span)
                            .help(
                                "use element symbols such as `H`, `O`, `Ca` (isotopes like `C-14` are also accepted)",
                            ),
                    );
                    return None;
                };
                *by_z.entry(z).or_default() += w;
            }
        }
        Composition::Formula(f) => {
            let atoms = match parse_formula(&f.node) {
                Ok(a) => a,
                Err(msg) => {
                    errs.push(
                        Diagnostic::error(format!("cannot read formula `{}`: {msg}", f.node))
                            .at(f.span)
                            .help("e.g. `H2O`, `C3H6O`, `Ca(OH)2`"),
                    );
                    return None;
                }
            };
            for (z, count) in atoms {
                let Some(a) = mass(z) else {
                    errs.push(Diagnostic::error(format!("no data for element Z = {z}")).at(f.span));
                    return None;
                };
                *by_z.entry(z).or_default() += count * a;
            }
        }
    }
    let total: f64 = by_z.values().sum();
    if total <= 0.0 {
        errs.push(Diagnostic::error(format!("material `{}` has an empty composition", m.name.node)).at(m.name.span));
        return None;
    }
    Some(by_z.into_iter().map(|(z, w)| (z, w / total)).collect())
}

/// `O`, `Ca`, and isotopes `C-14`, `C14` → Z.
fn element_of(s: &str) -> Option<u32> {
    let sym: String = s.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
    let rest = &s[sym.len()..];
    let rest = rest.strip_prefix('-').unwrap_or(rest);
    if !rest.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    z_of(&sym)
}

/// Parse a chemical formula into (Z, number of atoms).
pub fn parse_formula(f: &str) -> Result<Vec<(u32, f64)>, String> {
    fn group(chars: &[char], i: &mut usize, out: &mut BTreeMap<u32, f64>, mult: f64) -> Result<(), String> {
        while *i < chars.len() {
            let c = chars[*i];
            if c == ')' {
                return Ok(());
            }
            if c == '(' {
                *i += 1;
                let mut inner = BTreeMap::new();
                group(chars, i, &mut inner, 1.0)?;
                if *i >= chars.len() || chars[*i] != ')' {
                    return Err("missing `)`".into());
                }
                *i += 1;
                let n = number(chars, i).unwrap_or(1.0);
                for (z, k) in inner {
                    *out.entry(z).or_default() += k * n * mult;
                }
                continue;
            }
            if !c.is_ascii_uppercase() {
                return Err(format!("unexpected `{c}`"));
            }
            let mut sym = c.to_string();
            *i += 1;
            while *i < chars.len() && chars[*i].is_ascii_lowercase() {
                sym.push(chars[*i]);
                *i += 1;
            }
            let z = z_of(&sym).ok_or(format!("`{sym}` is not an element"))?;
            let n = number(chars, i).unwrap_or(1.0);
            *out.entry(z).or_default() += n * mult;
        }
        Ok(())
    }
    fn number(chars: &[char], i: &mut usize) -> Option<f64> {
        let start = *i;
        while *i < chars.len() && (chars[*i].is_ascii_digit() || chars[*i] == '.') {
            *i += 1;
        }
        (*i > start)
            .then(|| chars[start..*i].iter().collect::<String>().parse().ok())
            .flatten()
    }
    let chars: Vec<char> = f.chars().collect();
    let mut out = BTreeMap::new();
    let mut i = 0;
    group(&chars, &mut i, &mut out, 1.0)?;
    if i != chars.len() {
        return Err("unbalanced `)`".into());
    }
    if out.is_empty() {
        return Err("no elements".into());
    }
    Ok(out.into_iter().collect())
}

fn solid(s: &Shape) -> Solid {
    match *s {
        Shape::Box { size } => Solid::Box {
            half: Vec3::from_array(size) * 0.5,
        },
        Shape::Sphere { radius } => Solid::Sphere { r: radius },
        Shape::Cylinder { radius, height } => Solid::Cylinder {
            r: radius,
            half_h: height / 2.0,
        },
        Shape::Ellipsoid { semi_axes } => Solid::Ellipsoid {
            a: Vec3::from_array(semi_axes),
        },
    }
}

impl Problem {
    pub fn build(
        c: &Config,
        photon: &PhotonData,
        decay: Option<&DecayData>,
        electron: Option<&ElectronData>,
    ) -> Result<Problem, Vec<Diagnostic>> {
        let mut errs = Vec::new();
        let electrons_on = c.physics.electrons && electron.is_some();

        // ---- sources first: they decide the energy range of the tables
        let mut sources = Vec::new();
        for s in &c.sources {
            let transport_e = electrons_on && s.beta == BetaMode::Transport;
            let (label, lines, local, betas, electron_lines) = match &s.emission {
                Emission::Nuclide { id, span } => match decay.and_then(|d| d.get(id)) {
                    Some(n) => {
                        let positrons = n.betas.iter().any(|b| b.kind == BetaKind::Plus);
                        let lines: Vec<Line> = n
                            .photons
                            .iter()
                            // transported positrons create their own annihilation photons
                            .filter(|l| !(transport_e && positrons && l.kind == PhotonKind::Annihilation))
                            .map(|l| Line {
                                energy: l.energy,
                                intensity: l.intensity,
                                kind: if l.kind == PhotonKind::Annihilation {
                                    LineKind::Annihilation
                                } else {
                                    LineKind::Single
                                },
                            })
                            .collect();
                        if n.is_stable() {
                            errs.push(Diagnostic::error(format!("{id} is stable: it does not decay")).at(*span));
                        }
                        let local = match s.beta {
                            BetaMode::Off => 0.0,
                            BetaMode::Local => n.mean_electron_energy,
                            BetaMode::Transport if !electrons_on => n.mean_electron_energy,
                            BetaMode::Transport => 0.0,
                        };
                        let (betas, elines) = if transport_e {
                            (
                                n.betas
                                    .iter()
                                    .map(|b| {
                                        let positron = b.kind == BetaKind::Plus;
                                        let zd = if positron { id.z.saturating_sub(1) } else { id.z + 1 };
                                        BetaEmitter {
                                            spectrum: BetaSpectrum::new(b.endpoint, zd, positron, b.forbiddenness),
                                            intensity: b.intensity,
                                            positron,
                                        }
                                    })
                                    .collect(),
                                n.electrons
                                    .iter()
                                    .map(|e| Line {
                                        energy: e.energy,
                                        intensity: e.intensity,
                                        kind: LineKind::Single,
                                    })
                                    .collect(),
                            )
                        } else {
                            (vec![], vec![])
                        };
                        (id.to_string(), lines, local, betas, elines)
                    }
                    None => {
                        errs.push(nuclide_missing(*id, *span, decay.is_some()));
                        continue;
                    }
                },
                Emission::Lines { particle, lines: ls } => {
                    let kind = match particle {
                        SourceParticle::Positron => LineKind::Positron,
                        _ => LineKind::Single,
                    };
                    let lines: Vec<Line> = ls
                        .iter()
                        .map(|&(e, i)| Line {
                            energy: e,
                            intensity: i,
                            kind,
                        })
                        .collect();
                    let label = format!("{} lines", particle.name());
                    if *particle == SourceParticle::Photon {
                        (label, lines, 0.0, vec![], vec![])
                    } else if electrons_on {
                        (label, vec![], 0.0, vec![], lines)
                    } else {
                        errs.push(
                            Diagnostic::error(format!(
                                "{} emits {}s, but electron transport is off",
                                s.name,
                                particle.name()
                            ))
                            .at(s.span)
                            .help("use `list = photon_electron` in [physics], or a photon source"),
                        );
                        continue;
                    }
                }
            };
            sources.push((s, label, lines, local, betas, electron_lines));
        }
        let max_energy = sources
            .iter()
            .flat_map(|(_, _, l, _, b, el)| {
                l.iter()
                    .map(|x| x.energy)
                    .chain(
                        b.iter()
                            .map(|x: &BetaEmitter| *x.spectrum.energies.last().unwrap_or(&0.0)),
                    )
                    .chain(el.iter().map(|x| x.energy))
            })
            .fold(0.0f64, f64::max)
            .max(0.02);
        let emin = c.physics.photon_cut.clamp(1e-6, 1e-3);
        let grid = Grid::new(emin * 0.999, max_energy * 1.01, 1000);

        // ---- elements and materials
        let processes = Processes {
            photoelectric: c.physics.photoelectric,
            compton: c.physics.compton.is_some(),
            compton_sf: matches!(
                c.physics.compton,
                Some(guaraci_input::config::ComptonModel::KleinNishinaSf)
            ),
            rayleigh: c.physics.rayleigh,
            pair: c.physics.pair,
        };
        let mut used_materials: Vec<&str> = vec![c.world.material.node.as_str()];
        for v in &c.volumes {
            if !used_materials.contains(&v.material.node.as_str()) {
                used_materials.push(&v.material.node);
            }
        }
        let mut element_z: Vec<u32> = Vec::new();
        let mut materials = Vec::new();
        let mut comps_all = Vec::new();
        for name in &used_materials {
            let m = c.material(name).expect("checked by the config");
            let Some(fracs) = composition(m, photon, &mut errs) else {
                continue;
            };
            let mut comps = Vec::new();
            for (z, w) in fracs {
                let Some(el) = photon.get(&z) else {
                    errs.push(
                        Diagnostic::error(format!("the photon data has no element {} (Z = {z})", symbol(z)))
                            .at(m.name.span),
                    );
                    continue;
                };
                let idx = element_z.iter().position(|&e| e == z).unwrap_or_else(|| {
                    element_z.push(z);
                    element_z.len() - 1
                });
                comps.push(Component {
                    element: idx,
                    z,
                    mass_fraction: w,
                    atom_density: m.density * w * AVOGADRO / el.atomic_mass,
                });
            }
            comps_all.push((name.to_string(), m.density, comps));
        }
        let element_data: Vec<&guaraci_data::ElementPhoton> = element_z.iter().map(|z| &photon[z]).collect();
        for (name, density, comps) in comps_all {
            materials.push(Material::new(name, density, comps, &element_data, &grid, processes));
        }
        let elements: Vec<ElementSampling> = element_data.iter().map(|e| ElementSampling::new(e)).collect();

        // ---- electron tables
        let electron_cut = c.physics.electron_cut;
        let egrid = Grid::new(electron_cut * 0.999, max_energy.max(electron_cut * 2.0) * 1.01, 200);
        let mut emats = Vec::new();
        if electrons_on {
            let edata = electron.expect("checked");
            for m in &materials {
                let mut comps = Vec::new();
                for comp in &m.components {
                    match edata.get(&comp.z) {
                        Some(el) => comps.push((
                            comp.z,
                            comp.mass_fraction,
                            photon[&comp.z].atomic_mass,
                            comp.atom_density,
                            el,
                        )),
                        None => errs.push(Diagnostic::error(format!(
                            "the electron data has no element {} (Z = {})",
                            symbol(comp.z),
                            comp.z
                        ))),
                    }
                }
                let i_override = c.material(&m.name).and_then(|d| d.mean_excitation).map(|e| e * 1e6);
                // δ-rays above the electron cut-off, bremsstrahlung photons above the photon cut-off
                let brem_cut = c.physics.photon_cut.max(1e-3);
                emats.push(ElectronMaterial::new(
                    &egrid,
                    m.density,
                    &comps,
                    i_override,
                    electron_cut,
                    brem_cut,
                ));
            }
        }
        let mat_index = |name: &str| materials.iter().position(|m: &Material| m.name == name).unwrap_or(0);

        // ---- geometry
        let mut volumes = vec![Volume {
            name: "world".into(),
            solid: Solid::Box {
                half: Vec3::from_array(c.world.size) * 0.5,
            },
            center: Vec3::ZERO,
            material: mat_index(&c.world.material.node),
            parent: None,
            children: vec![],
        }];
        // add volumes parents-first
        let mut pending: Vec<&guaraci_input::config::Volume> = c.volumes.iter().collect();
        while !pending.is_empty() {
            let before = pending.len();
            pending.retain(|v| {
                let parent = v.parent.as_ref().map(|p| p.node.as_str()).unwrap_or("world");
                let Some(pi) = (if parent == "world" {
                    Some(0)
                } else {
                    volumes.iter().position(|x| x.name == parent)
                }) else {
                    return true;
                };
                let center = volumes[pi].center + Vec3::from_array(v.center);
                volumes.push(Volume {
                    name: v.name.node.clone(),
                    solid: solid(&v.shape),
                    center,
                    material: mat_index(&v.material.node),
                    parent: Some(pi),
                    children: vec![],
                });
                let id = volumes.len() - 1;
                volumes[pi].children.push(id);
                false
            });
            if pending.len() == before {
                break; // cycles are reported by the config
            }
        }
        let geometry = Geometry { volumes };
        for (i, msg) in geometry.check_overlaps(5000) {
            let span = c.volume(&geometry.volumes[i].name).map(|v| v.name.span);
            let mut d = Diagnostic::error(msg)
                .help("move or resize the volume; daughters must lie inside their parent and must not overlap");
            if let Some(s) = span {
                d = d.at(s);
            }
            errs.push(d);
        }

        // ---- sources: placement and weights
        let all_activity = c.sources.iter().all(|s| s.activity.is_some());
        let any_activity = c.sources.iter().any(|s| s.activity.is_some());
        let all_duration = c.sources.iter().all(|s| s.duration.is_some());
        if any_activity && !all_activity {
            for s in c.sources.iter().filter(|s| s.activity.is_none()) {
                errs.push(
                    Diagnostic::error("some sources have `activity` and this one does not")
                        .at(s.span)
                        .help("give an activity to every source, or to none (results per decay)"),
                );
            }
        }
        let mut out_sources = Vec::new();
        for (s, label, lines, local, betas, electron_lines) in sources {
            let place = match &s.placement {
                Placement::Point(p) => {
                    let p = Vec3::from_array(*p);
                    if geometry.locate(p).is_none() {
                        errs.push(Diagnostic::error("the source position is outside the world").at(s.span));
                    }
                    Where::Point(p)
                }
                Placement::In(v) => match geometry.index(&v.node) {
                    Some(i) => {
                        if geometry.region_volume(i) <= 1e-12 * geometry.volumes[i].solid.volume() {
                            errs.push(
                                Diagnostic::error(format!(
                                    "`{}` is completely filled by its daughters; the source has no room",
                                    v.node
                                ))
                                .at(v.span),
                            );
                        }
                        Where::Region { volume: i }
                    }
                    None => continue,
                },
            };
            let weight = match (s.activity, s.duration) {
                (Some(a), Some(t)) if all_duration => a * t,
                (Some(a), _) => a,
                _ => 1.0,
            };
            out_sources.push(Source {
                name: s.name.clone(),
                label,
                lines,
                betas,
                electron_lines,
                local_energy: local,
                place,
                weight,
            });
        }
        let wsum: f64 = out_sources.iter().map(|s| s.weight).sum();
        let mut acc = 0.0;
        let source_cdf = out_sources
            .iter()
            .map(|s| {
                acc += s.weight / wsum;
                acc
            })
            .collect();
        let normalization = if all_activity && all_duration {
            Normalization::Total { decays: wsum }
        } else if all_activity {
            Normalization::PerSecond { activity: wsum }
        } else {
            Normalization::PerDecay
        };

        // ---- probes
        let nvol = geometry.volumes.len();
        let mut probes = Vec::new();
        let mut volume_slots = vec![Vec::new(); nvol];
        let mut spectrum_slots = vec![Vec::new(); nvol];
        for p in &c.probes {
            let pi = probes.len();
            match &p.kind {
                ProbeKind::DoseGrid { region, voxel } => {
                    let Some(v) = geometry.index(&region.node) else {
                        continue;
                    };
                    let vol = &geometry.volumes[v];
                    let half = vol.solid.half_extent();
                    let spacing = Vec3::from_array(*voxel);
                    // ceil with a tolerance: 2.2 / 0.1 = 22.000000000000004 must give 22 voxels
                    let count = |len: f64, step: f64| (((len / step) - 1e-9).ceil() as usize).max(1);
                    let dims = [
                        count(2.0 * half.x, spacing.x),
                        count(2.0 * half.y, spacing.y),
                        count(2.0 * half.z, spacing.z),
                    ];
                    let n = dims[0] * dims[1] * dims[2];
                    if n > 200_000_000 {
                        errs.push(
                            Diagnostic::error(format!("probe `{}` would have {n} voxels", p.name.node))
                                .at(p.name.span)
                                .help("use larger voxels"),
                        );
                        continue;
                    }
                    let size = Vec3::new(
                        dims[0] as f64 * spacing.x,
                        dims[1] as f64 * spacing.y,
                        dims[2] as f64 * spacing.z,
                    );
                    let origin = vol.center - size * 0.5;
                    let vvol = spacing.x * spacing.y * spacing.z;
                    // Mass of each voxel from the mean density at SUB³ points, so that
                    // voxels cut by a boundary get the mass they really contain.
                    const SUB: usize = 3;
                    let mut mass_kg = vec![0.0; n];
                    for k in 0..dims[2] {
                        for j in 0..dims[1] {
                            for i in 0..dims[0] {
                                let corner = origin
                                    + Vec3::new(i as f64 * spacing.x, j as f64 * spacing.y, k as f64 * spacing.z);
                                let mut rho = 0.0;
                                for (a, b, cc) in
                                    (0..SUB * SUB * SUB).map(|q| (q % SUB, q / SUB % SUB, q / (SUB * SUB)))
                                {
                                    let f = |x: usize| (x as f64 + 0.5) / SUB as f64;
                                    let pt = corner + Vec3::new(f(a) * spacing.x, f(b) * spacing.y, f(cc) * spacing.z);
                                    rho += geometry
                                        .locate(pt)
                                        .map(|v| materials[geometry.volumes[v].material].density)
                                        .unwrap_or(0.0);
                                }
                                mass_kg[i + dims[0] * (j + dims[1] * k)] = rho / (SUB * SUB * SUB) as f64 * vvol * 1e-3;
                            }
                        }
                    }
                    probes.push(Probe::DoseGrid {
                        name: p.name.node.clone(),
                        output: p.output.clone(),
                        origin,
                        spacing,
                        dims,
                        mass_kg,
                    });
                }
                ProbeKind::VolumeDose { volumes } => {
                    let mut ids = Vec::new();
                    let mut mass = Vec::new();
                    for v in volumes {
                        let Some(i) = geometry.index(&v.node) else { continue };
                        volume_slots[i].push((pi, ids.len()));
                        ids.push(i);
                        mass.push(geometry.region_volume(i) * materials[geometry.volumes[i].material].density * 1e-3);
                    }
                    probes.push(Probe::VolumeDose {
                        name: p.name.node.clone(),
                        output: p.output.clone(),
                        volumes: ids,
                        mass_kg: mass,
                    });
                }
                ProbeKind::Spectrum { region, emax, bins } => {
                    let Some(v) = geometry.index(&region.node) else {
                        continue;
                    };
                    spectrum_slots[v].push(pi);
                    probes.push(Probe::Spectrum {
                        name: p.name.node.clone(),
                        output: p.output.clone(),
                        region: v,
                        emax: *emax,
                        bins: *bins,
                        volume_cm3: geometry.region_volume(v),
                    });
                }
            }
        }

        if !errs.is_empty() {
            return Err(errs);
        }
        Ok(Problem {
            geometry,
            grid,
            materials,
            elements,
            sources: out_sources,
            source_cdf,
            probes,
            normalization,
            photon_cut: c.physics.photon_cut.max(1e-6),
            compton_sf: processes.compton_sf,
            volume_slots,
            spectrum_slots,
            max_energy,
            electrons: electrons_on,
            electron_cut,
            egrid,
            emats,
        })
    }

    /// Mean photon energy emitted per decay, averaged over sources (MeV).
    pub fn mean_emitted_photon_energy(&self) -> f64 {
        let wsum: f64 = self.sources.iter().map(|s| s.weight).sum();
        self.sources
            .iter()
            .map(|s| s.weight / wsum * s.lines.iter().map(|l| l.energy * l.intensity).sum::<f64>())
            .sum()
    }
}

fn nuclide_missing(id: NuclideId, span: Span, have_data: bool) -> Diagnostic {
    if have_data {
        Diagnostic::error(format!("{id} is not in the decay data"))
            .at(span)
            .help("check the mass number, or `guaraci info nuclide <name>`")
    } else {
        Diagnostic::error("decay data is not available").at(span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formulas() {
        assert_eq!(parse_formula("H2O").unwrap(), vec![(1, 2.0), (8, 1.0)]);
        assert_eq!(parse_formula("Ca(OH)2").unwrap(), vec![(1, 2.0), (8, 2.0), (20, 1.0)]);
        assert_eq!(parse_formula("C3H6O").unwrap(), vec![(1, 6.0), (6, 3.0), (8, 1.0)]);
        assert!(parse_formula("H2X").is_err());
        assert!(parse_formula("Ca(OH").is_err());
        assert_eq!(element_of("C-14"), Some(6));
        assert_eq!(element_of("C14"), Some(6));
        assert_eq!(element_of("Ca"), Some(20));
        assert_eq!(element_of("Xx"), None);
    }
}
