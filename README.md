# Guaraci

**Monte Carlo radiation transport on CPU and GPU.**

Focused initially on medical physics and nuclear medicine.

> *Guaraci* is the Sun in Tupi mythology — the source of the radiation that gives life to everything.

## Status

Early development. Available so far:

- `guaraci-units` — physical units with dimensional checking (`10 cm`, `[1, 2, 3] mm`, `1.05 g/cm3`, `370 MBq`).
- `guaraci-input` — the `.gci` input language: parser, includes, typed configuration and error messages that point at the text.

Coming next: photon transport on CPU (photoelectric, Compton, Rayleigh, pair production), dose grids with uncertainty, radionuclide sources, then a CUDA backend and a desktop interface.

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
nuclide = Tc99m
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
