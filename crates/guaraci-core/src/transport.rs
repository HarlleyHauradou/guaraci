//! The history loop: sample a decay, follow every photon, score the probes.

use crate::electron::{
    brem_photon_direction, compton_electron_direction, delta_directions, sample_delta_energy,
    sample_multiple_scattering, sample_sauter,
};
use crate::geometry::PUSH;
use crate::physics::{interact, Interaction, ELECTRON_MASS};
use crate::problem::{LineKind, Probe, Problem, Where};
use crate::rng::Rng;
use crate::vec3::Vec3;

/// Accumulated scores (energies in MeV, track lengths in cm), per simulated decay sum.
#[derive(Debug, Clone)]
pub struct Tally {
    /// One array per probe.
    pub cells: Vec<Vec<f64>>,
    /// Energy deposited anywhere in the world.
    pub deposited: f64,
    /// Energy carried out of the world by photons.
    pub escaped: f64,
    /// Energy emitted (photons + local electrons).
    pub emitted: f64,
    pub histories: u64,
    pub photons: u64,
}

impl Tally {
    pub fn new(p: &Problem) -> Tally {
        Tally {
            cells: p.probes.iter().map(|pr| vec![0.0; pr.cells()]).collect(),
            deposited: 0.0,
            escaped: 0.0,
            emitted: 0.0,
            histories: 0,
            photons: 0,
        }
    }

    pub fn add(&mut self, o: &Tally) {
        for (a, b) in self.cells.iter_mut().zip(&o.cells) {
            for (x, y) in a.iter_mut().zip(b) {
                *x += y;
            }
        }
        self.deposited += o.deposited;
        self.escaped += o.escaped;
        self.emitted += o.emitted;
        self.histories += o.histories;
        self.photons += o.photons;
    }

    pub fn clear(&mut self) {
        for c in &mut self.cells {
            c.iter_mut().for_each(|x| *x = 0.0);
        }
        self.deposited = 0.0;
        self.escaped = 0.0;
        self.emitted = 0.0;
        self.histories = 0;
        self.photons = 0;
    }

    #[inline]
    fn deposit(&mut self, p: &Problem, pos: Vec3, vol: usize, e: f64) {
        if e <= 0.0 {
            return;
        }
        self.deposited += e;
        for &(pi, cell) in &p.volume_slots[vol] {
            self.cells[pi][cell] += e;
        }
        for (pi, probe) in p.probes.iter().enumerate() {
            if let Probe::DoseGrid {
                origin, spacing, dims, ..
            } = probe
            {
                let r = (pos - *origin).div_elem(*spacing);
                if r.x >= 0.0 && r.y >= 0.0 && r.z >= 0.0 {
                    let (i, j, k) = (r.x as usize, r.y as usize, r.z as usize);
                    if i < dims[0] && j < dims[1] && k < dims[2] {
                        self.cells[pi][i + dims[0] * (j + dims[1] * k)] += e;
                    }
                }
            }
        }
    }

    #[inline]
    fn track(&mut self, p: &Problem, vol: usize, e: f64, len: f64) {
        for &pi in &p.spectrum_slots[vol] {
            if let Probe::Spectrum { emax, bins, .. } = &p.probes[pi] {
                let b = (e / emax * *bins as f64) as usize;
                if b < *bins {
                    self.cells[pi][b] += len;
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Photon,
    Electron,
    Positron,
}

#[derive(Clone, Copy, Debug)]
pub struct Particle {
    pub kind: Kind,
    pub pos: Vec3,
    pub dir: Vec3,
    /// Kinetic energy, MeV.
    pub energy: f64,
    pub vol: usize,
}

/// Kept for code that refers to the photon stack by its former name.
pub type Photon = Particle;

fn isotropic(rng: &mut impl Rng) -> Vec3 {
    let c = 2.0 * rng.uniform() - 1.0;
    let s = (1.0 - c * c).sqrt();
    let phi = 2.0 * std::f64::consts::PI * rng.uniform();
    Vec3::new(s * phi.cos(), s * phi.sin(), c)
}

/// Uniform point in the region of volume `v` (rejection sampling in its bounding box).
fn sample_in_region(p: &Problem, v: usize, rng: &mut impl Rng) -> Vec3 {
    let vol = &p.geometry.volumes[v];
    let h = vol.solid.half_extent();
    loop {
        let q = vol.center
            + Vec3::new(
                (2.0 * rng.uniform() - 1.0) * h.x,
                (2.0 * rng.uniform() - 1.0) * h.y,
                (2.0 * rng.uniform() - 1.0) * h.z,
            );
        if vol.contains(q) && !vol.children.iter().any(|&c| p.geometry.volumes[c].contains(q)) {
            return q;
        }
    }
}

/// Number of particles of a line of intensity `i` (per decay) in one decay.
fn count(i: f64, rng: &mut impl Rng) -> u64 {
    let base = i.floor();
    base as u64 + u64::from(rng.uniform() < i - base)
}

/// Simulate one decay.
pub fn history(p: &Problem, rng: &mut impl Rng, t: &mut Tally, stack: &mut Vec<Particle>) {
    t.histories += 1;
    let u = rng.uniform();
    let si = p.source_cdf.iter().position(|&c| u < c).unwrap_or(p.sources.len() - 1);
    let src = &p.sources[si];
    let pos = match src.place {
        Where::Point(q) => q,
        Where::Region { volume } => sample_in_region(p, volume, rng),
    };
    let Some(vol) = p.geometry.locate(pos) else { return };
    let new = |kind, dir, energy| Particle {
        kind,
        pos,
        dir,
        energy,
        vol,
    };

    if src.local_energy > 0.0 {
        t.emitted += src.local_energy;
        t.deposit(p, pos, vol, src.local_energy);
    }
    for line in &src.lines {
        let n = count(line.intensity, rng);
        match line.kind {
            LineKind::Single => {
                for _ in 0..n {
                    stack.push(new(Kind::Photon, isotropic(rng), line.energy));
                }
            }
            LineKind::Annihilation | LineKind::Positron => {
                let mut left = n;
                while left > 0 {
                    let d = isotropic(rng);
                    stack.push(new(Kind::Photon, d, line.energy));
                    if left >= 2 {
                        stack.push(new(Kind::Photon, -d, line.energy));
                        left -= 2;
                    } else {
                        left -= 1;
                    }
                }
            }
        }
        t.emitted += n as f64 * line.energy;
    }
    for beta in &src.betas {
        for _ in 0..count(beta.intensity, rng) {
            let e = beta.spectrum.sample(rng);
            t.emitted += e;
            let kind = if beta.positron { Kind::Positron } else { Kind::Electron };
            stack.push(new(kind, isotropic(rng), e));
        }
    }
    for line in &src.electron_lines {
        let n = count(line.intensity, rng);
        let kind = if line.kind == LineKind::Positron {
            Kind::Positron
        } else {
            Kind::Electron
        };
        for _ in 0..n {
            stack.push(new(kind, isotropic(rng), line.energy));
        }
        t.emitted += n as f64 * line.energy;
    }

    while let Some(part) = stack.pop() {
        match part.kind {
            Kind::Photon => transport_photon(p, part, rng, t, stack),
            Kind::Electron | Kind::Positron => transport_charged(p, part, rng, t, stack),
        }
    }
}

fn push_distance(p: &Problem) -> f64 {
    PUSH * p.geometry.world().solid.half_extent().max_elem().max(1.0)
}

/// An electron set in motion by a photon: transported if electron transport
/// is on and it is above the cut-off, otherwise deposited on the spot.
fn emit_electron(p: &Problem, pos: Vec3, dir: Vec3, energy: f64, vol: usize, t: &mut Tally, stack: &mut Vec<Particle>) {
    if p.electrons && energy > p.electron_cut {
        stack.push(Particle {
            kind: Kind::Electron,
            pos,
            dir,
            energy,
            vol,
        });
    } else {
        t.deposit(p, pos, vol, energy);
    }
}

/// Positron at rest: two back-to-back 511 keV photons. Their rest-mass
/// energy enters the energy balance here.
fn annihilate_with(pos: Vec3, vol: usize, t: &mut Tally, stack: &mut Vec<Particle>, rng: &mut impl Rng) {
    let d = isotropic(rng);
    t.emitted += 2.0 * ELECTRON_MASS;
    stack.push(Particle {
        kind: Kind::Photon,
        pos,
        dir: d,
        energy: ELECTRON_MASS,
        vol,
    });
    stack.push(Particle {
        kind: Kind::Photon,
        pos,
        dir: -d,
        energy: ELECTRON_MASS,
        vol,
    });
}

fn transport_photon(p: &Problem, mut ph: Particle, rng: &mut impl Rng, t: &mut Tally, stack: &mut Vec<Particle>) {
    t.photons += 1;
    let push = push_distance(p);
    loop {
        if ph.energy < p.photon_cut {
            t.deposit(p, ph.pos, ph.vol, ph.energy);
            return;
        }
        let mat = &p.materials[p.geometry.volumes[ph.vol].material];
        let at = p.grid.locate(ph.energy);
        let mu = mat.mu(at);
        let s = if mu > 0.0 {
            -rng.uniform().ln() / mu
        } else {
            f64::INFINITY
        };
        let d = p.geometry.distance(ph.vol, ph.pos, ph.dir);
        if s < d {
            t.track(p, ph.vol, ph.energy, s);
            ph.pos = ph.pos + ph.dir * s;
            match interact(mat, &p.elements, &p.grid, ph.energy, ph.dir, p.compton_sf, rng) {
                Interaction::Photoelectric => {
                    if p.electrons && ph.energy > p.electron_cut {
                        let cos = sample_sauter(ph.energy, rng);
                        let dir = ph.dir.rotate(cos, 2.0 * std::f64::consts::PI * rng.uniform());
                        stack.push(Particle {
                            kind: Kind::Electron,
                            pos: ph.pos,
                            dir,
                            energy: ph.energy,
                            vol: ph.vol,
                        });
                    } else {
                        t.deposit(p, ph.pos, ph.vol, ph.energy);
                    }
                    return;
                }
                Interaction::Compton { energy, dir, deposit } => {
                    let edir = compton_electron_direction(ph.energy, ph.dir, energy, dir);
                    emit_electron(p, ph.pos, edir, deposit, ph.vol, t, stack);
                    ph.energy = energy;
                    ph.dir = dir;
                }
                Interaction::Rayleigh { dir } => ph.dir = dir,
                Interaction::Pair { deposit } => {
                    if p.electrons {
                        // kinetic energy shared uniformly; both leave close to the photon direction
                        let te = deposit * rng.uniform();
                        let tp = deposit - te;
                        let phi = 2.0 * std::f64::consts::PI * rng.uniform();
                        for (kind, tk, ph_angle) in [
                            (Kind::Electron, te, phi),
                            (Kind::Positron, tp, phi + std::f64::consts::PI),
                        ] {
                            let cos = (1.0 - (ELECTRON_MASS / (tk + ELECTRON_MASS)).powi(2)).max(0.0).sqrt();
                            let dir = ph.dir.rotate(cos, ph_angle);
                            if kind == Kind::Electron {
                                emit_electron(p, ph.pos, dir, tk, ph.vol, t, stack);
                            } else if tk > p.electron_cut {
                                stack.push(Particle {
                                    kind,
                                    pos: ph.pos,
                                    dir,
                                    energy: tk,
                                    vol: ph.vol,
                                });
                            } else {
                                t.deposit(p, ph.pos, ph.vol, tk);
                                annihilate_with(ph.pos, ph.vol, t, stack, rng);
                            }
                        }
                        // the pair's rest mass leaves the balance until the positron annihilates
                        t.emitted -= 2.0 * ELECTRON_MASS;
                    } else {
                        t.deposit(p, ph.pos, ph.vol, deposit);
                        let d = isotropic(rng);
                        stack.push(Particle {
                            kind: Kind::Photon,
                            pos: ph.pos,
                            dir: d,
                            energy: ELECTRON_MASS,
                            vol: ph.vol,
                        });
                        stack.push(Particle {
                            kind: Kind::Photon,
                            pos: ph.pos,
                            dir: -d,
                            energy: ELECTRON_MASS,
                            vol: ph.vol,
                        });
                    }
                    return;
                }
            }
        } else {
            t.track(p, ph.vol, ph.energy, d);
            ph.pos = ph.pos + ph.dir * (d + push);
            match p.geometry.locate(ph.pos) {
                Some(v) => ph.vol = v,
                None => {
                    t.escaped += ph.energy;
                    return;
                }
            }
        }
    }
}

/// Largest fraction of its energy a charged particle may lose in one step.
const MAX_ENERGY_LOSS: f64 = 0.15;
/// Largest step in units of the elastic transport mean free path.
const MAX_STEP_LAMBDA: f64 = 0.3;

/// Condensed-history transport of an electron or positron (class I, CSDA):
/// continuous energy loss from the range table, deflection at the end of
/// each step, energy deposited at a uniformly sampled point of the step.
fn transport_charged(p: &Problem, mut e: Particle, rng: &mut impl Rng, t: &mut Tally, stack: &mut Vec<Particle>) {
    let positron = e.kind == Kind::Positron;
    let push = push_distance(p);
    loop {
        if e.energy <= p.electron_cut {
            t.deposit(p, e.pos, e.vol, e.energy);
            if positron {
                annihilate_with(e.pos, e.vol, t, stack, rng);
            }
            return;
        }
        let em = &p.emats[p.geometry.volumes[e.vol].material];
        let g = &p.egrid;
        let r0 = em.range(g, e.energy, positron);
        let r_cut = em.range(g, p.electron_cut, positron);
        let e_low = e.energy * (1.0 - MAX_ENERGY_LOSS);
        let s_energy = r0 - em.range(g, e_low, positron);
        let lambda1 = em.transport_mfp(g, e.energy);
        // distance to the next hard event, with the largest cross section along
        // the step (the energy changes by at most MAX_ENERGY_LOSS); the event is
        // then accepted with probability Σ(E at the end) / Σ_max
        let (d0, b0) = em.hard(g, e.energy, positron);
        let (d1, b1) = em.hard(g, e_low.max(p.electron_cut), positron);
        let sigma_max = (d0 + b0).max(d1 + b1);
        let s_hard = if sigma_max > 0.0 {
            -(1.0 - rng.uniform()).ln() / sigma_max
        } else {
            f64::INFINITY
        };
        let mut s = s_energy.min(MAX_STEP_LAMBDA * lambda1).min(r0 - r_cut).max(1e-12);
        let hard_step = s_hard < s;
        if hard_step {
            s = s_hard;
        }
        let d = p.geometry.distance(e.vol, e.pos, e.dir);
        let crosses = d < s;
        if crosses {
            s = d;
        }
        let e1 = if r0 - s <= r_cut {
            p.electron_cut
        } else {
            em.energy_after(g, r0 - s, positron)
        };
        let lost = (e.energy - e1).max(0.0);
        if lost > 0.0 {
            let at = e.pos + e.dir * (s * rng.uniform());
            t.deposit(p, at, e.vol, lost);
        }
        e.pos = e.pos + e.dir * s;
        e.energy = e1;
        if crosses {
            e.pos = e.pos + e.dir * push;
            match p.geometry.locate(e.pos) {
                Some(v) => e.vol = v,
                None => {
                    t.escaped += e.energy;
                    return;
                }
            }
        }
        if s > 0.0 {
            let cos = sample_multiple_scattering(s, lambda1, rng);
            e.dir = e.dir.rotate(cos, 2.0 * std::f64::consts::PI * rng.uniform());
        }
        if hard_step && !crosses && e.energy > p.electron_cut {
            let (sd, sb) = em.hard(g, e.energy, positron);
            let u = rng.uniform() * sigma_max;
            if u < sd {
                // knock-on electron (δ-ray)
                let w = sample_delta_energy(e.energy, em.delta_cut, positron, rng);
                let (d_primary, d_delta) = delta_directions(e.energy, w, e.dir, rng);
                stack.push(Particle {
                    kind: Kind::Electron,
                    pos: e.pos,
                    dir: d_delta,
                    energy: w,
                    vol: e.vol,
                });
                e.energy -= w;
                e.dir = d_primary;
            } else if u < sd + sb {
                // bremsstrahlung photon
                let k = em.sample_brem(g, e.energy, rng).min(e.energy);
                let dir = brem_photon_direction(e.energy, e.dir, rng);
                stack.push(Particle {
                    kind: Kind::Photon,
                    pos: e.pos,
                    dir,
                    energy: k,
                    vol: e.vol,
                });
                e.energy -= k;
            }
            // otherwise a rejected (virtual) event: nothing happens
        }
    }
}

/// Run `count` histories with one random stream and add them to `t`.
pub fn run_block(p: &Problem, rng: &mut impl Rng, count: u64, t: &mut Tally) {
    let mut stack = Vec::with_capacity(32);
    for _ in 0..count {
        history(p, rng, t, &mut stack);
    }
}
