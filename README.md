# Guaraci

**Monte Carlo radiation transport on CPU and GPU.**

Focused initially on medical physics and nuclear medicine.

> *Guaraci* is the Sun in Tupi mythology — the source of the radiation that gives life to everything.

## Status

Early development. Available so far:

- `guaraci check <input>` — reads a `.gci` input, reports every problem with a pointer into the text, and describes the simulation (geometry tree, materials, nuclide emissions, defaults applied).
- `guaraci data install endfb8` — downloads ENDF/B-VIII.0 (photo-atomic and decay data) from NNDC, verifies its SHA-256 and converts it locally.
- `guaraci info units | materials | nuclide <name>`.
- A database of 372 materials from PNNL-15870 Rev. 1.

Coming next: photon transport on CPU (photoelectric, Compton, Rayleigh, pair production), dose grids with uncertainty, then a CUDA backend and a desktop interface.

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
