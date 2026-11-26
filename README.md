# Guaraci

**Monte Carlo radiation transport on CPU and GPU.**

Focused initially on medical physics and nuclear medicine.

> *Guaraci* is the Sun in Tupi mythology — the source of the radiation that gives life to everything.

## Status

Early development. Available so far:

- **Photon transport on CPU** (multi-threaded): photoelectric effect, Compton scattering (Klein-Nishina with incoherent scattering function), Rayleigh scattering (form factors) and pair production. Electrons deposit their energy locally.
- **Probes**: 3D dose grids (MetaImage `.mhd`), dose in volumes, photon fluence spectra — each with its statistical uncertainty.
- **Radionuclide sources** from ENDF/B-VIII.0 decay data (3821 nuclides), user-defined photon lines, activity and duration.
- **Validation**: μ/ρ within 0.4 % of NIST XCOM (10 keV–10 MeV), uncollided fluence within 0.25 % of the analytic value, exact energy conservation, results independent of the number of threads.
- `guaraci check` explains what an input describes; `guaraci run` writes a readable report and a copy of the inputs.
- Nuclear data downloaded from the official sources on first use (`guaraci data install endfb8`).
- 372 materials from PNNL-15870 Rev. 1.

Coming next: S-value benchmarks, voxelised phantoms, CUDA backend, electron transport, desktop interface.

## Quick start

```bash
cargo install --path crates/guaraci-cli
guaraci check examples/thyroid_i131
guaraci run examples/thyroid_i131
```

## Manual

The manual has two volumes (in Portuguese for now): a [User Manual](docs/manual/usuario/README.md) and a [Technical Manual](docs/manual/tecnico/README.md).

## A taste of the input

```
[world]
material = Air
size = [1, 1, 1] m

[volume phantom]
shape = cylinder
radius = 10            # no unit = cm
height = 20 cm
material = Water

[volume lesion]
parent = phantom
shape = sphere
radius = 15 mm
center = [3, 0, 0]
material = Water

[source]
nuclide = Tc-99m
in = lesion
activity = 370 MBq

[probe dose]
type = dose_grid
in = phantom
voxel = [2, 2, 2] mm
```

Default units when none is written: cm, keV, MBq, s, g/cm³, degrees.

## Nuclear data

Guaraci does not redistribute nuclear data libraries. They are downloaded from their official sources on first use (or with `guaraci data install`) and converted locally.

## Philosophy

See [PHILOSOPHY.md](PHILOSOPHY.md).

## Building

```bash
cargo build --release
cargo test
```
