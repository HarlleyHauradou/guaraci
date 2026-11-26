//! The history loop: sample a decay, follow every photon, score the probes.

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

#[derive(Clone, Copy)]
pub struct Photon {
    pos: Vec3,
    dir: Vec3,
    energy: f64,
    vol: usize,
}

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

/// Simulate one decay.
pub fn history(p: &Problem, rng: &mut impl Rng, t: &mut Tally, stack: &mut Vec<Photon>) {
    t.histories += 1;
    let u = rng.uniform();
    let si = p.source_cdf.iter().position(|&c| u < c).unwrap_or(p.sources.len() - 1);
    let src = &p.sources[si];
    let pos = match src.place {
        Where::Point(q) => q,
        Where::Region { volume } => sample_in_region(p, volume, rng),
    };
    let Some(vol) = p.geometry.locate(pos) else { return };

    if src.local_energy > 0.0 {
        t.emitted += src.local_energy;
        t.deposit(p, pos, vol, src.local_energy);
    }
    for line in &src.lines {
        let base = line.intensity.floor();
        let n = base as u64 + u64::from(rng.uniform() < line.intensity - base);
        match line.kind {
            LineKind::Single => {
                for _ in 0..n {
                    stack.push(Photon {
                        pos,
                        dir: isotropic(rng),
                        energy: line.energy,
                        vol,
                    });
                }
            }
            LineKind::Annihilation => {
                let mut left = n;
                while left > 0 {
                    let d = isotropic(rng);
                    stack.push(Photon {
                        pos,
                        dir: d,
                        energy: line.energy,
                        vol,
                    });
                    if left >= 2 {
                        stack.push(Photon {
                            pos,
                            dir: -d,
                            energy: line.energy,
                            vol,
                        });
                        left -= 2;
                    } else {
                        left -= 1;
                    }
                }
            }
        }
        t.emitted += n as f64 * line.energy;
    }

    while let Some(ph) = stack.pop() {
        transport(p, ph, rng, t, stack);
    }
}

fn transport(p: &Problem, mut ph: Photon, rng: &mut impl Rng, t: &mut Tally, stack: &mut Vec<Photon>) {
    t.photons += 1;
    let push = PUSH * p.geometry.world().solid.half_extent().max_elem().max(1.0);
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
                    t.deposit(p, ph.pos, ph.vol, ph.energy);
                    return;
                }
                Interaction::Compton { energy, dir, deposit } => {
                    t.deposit(p, ph.pos, ph.vol, deposit);
                    ph.energy = energy;
                    ph.dir = dir;
                }
                Interaction::Rayleigh { dir } => ph.dir = dir,
                Interaction::Pair { deposit } => {
                    t.deposit(p, ph.pos, ph.vol, deposit);
                    let d = isotropic(rng);
                    stack.push(Photon {
                        pos: ph.pos,
                        dir: d,
                        energy: ELECTRON_MASS,
                        vol: ph.vol,
                    });
                    stack.push(Photon {
                        pos: ph.pos,
                        dir: -d,
                        energy: ELECTRON_MASS,
                        vol: ph.vol,
                    });
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

/// Run `count` histories with one random stream and add them to `t`.
pub fn run_block(p: &Problem, rng: &mut impl Rng, count: u64, t: &mut Tally) {
    let mut stack = Vec::with_capacity(16);
    for _ in 0..count {
        history(p, rng, t, &mut stack);
    }
}
