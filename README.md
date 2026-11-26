<p align="center">
  <img src="docs/assets/favicon.svg" width="90" alt="">
</p>

<h1 align="center">Guaraci</h1>

<p align="center">
  <b>Monte Carlo radiation transport on CPU and GPU</b><br>
  Focused initially on medical physics and nuclear medicine.
</p>

<p align="center">
  <a href="https://harlleyhauradou.github.io/guaraci/"><b>Documentation</b></a> ·
  <a href="https://harlleyhauradou.github.io/guaraci/volume-2/03-primeira-simulacao/">First simulation</a> ·
  <a href="https://harlleyhauradou.github.io/guaraci/volume-1/08-validacao/">Validation</a> ·
  <a href="examples/">Examples</a>
</p>

<p align="center">
  <a href="https://github.com/HarlleyHauradou/guaraci/actions/workflows/ci.yml"><img src="https://github.com/HarlleyHauradou/guaraci/actions/workflows/ci.yml/badge.svg" alt="ci"></a>
  <a href="https://github.com/HarlleyHauradou/guaraci/actions/workflows/docs.yml"><img src="https://github.com/HarlleyHauradou/guaraci/actions/workflows/docs.yml/badge.svg" alt="docs"></a>
</p>

---

*Guaraci is the Sun in Tupi mythology — the source of the radiation that gives life to everything.*

```ini
[volume thyroid]
parent = neck
shape = ellipsoid
semi_axes = [1, 1.5, 2.5] cm
material = Tissue_Soft_ICRP

[source]
nuclide = I-131
in = thyroid
activity = 3.7 GBq

[probe dose]
type = dose_grid
in = neck
voxel = [2, 2, 2] mm
```

```bash
guaraci check thyroid    # what Guaraci understood, without running
guaraci run thyroid      # dose map, uncertainties and a readable report
```

## Features

- **Photon transport** on multi-core CPUs: photoelectric effect, Compton (Klein-Nishina with S(x,Z)), Rayleigh (form factors), pair production.
- **Radionuclide sources** from ENDF/B-VIII.0 decay data (3821 nuclides), downloaded from the official source on first use — no third-party data is redistributed.
- **Human-readable input** with units (`15 mm`, `3.7 GBq`), includes, and error messages that point at the line and suggest the fix.
- **372 materials** from PNNL-15870 Rev. 1.
- **Probes**: 3D dose maps (MetaImage), dose in volumes, fluence spectra — all with statistical uncertainties.
- **Validated**: μ/ρ within 0.4 % of NIST XCOM; absorbed fractions in 231 spheres compared with Stabin & Konijnenberg (2000); exact energy conservation; results independent of the thread count.

In development: voxelised phantoms (CT/DICOM), electron transport, CUDA backend, graphical interface.

## Install

```bash
git clone https://github.com/HarlleyHauradou/guaraci
cd guaraci
cargo install --path crates/guaraci-cli
guaraci run examples/thyroid_i131
```

## Documentation

The manual is online at **<https://harlleyhauradou.github.io/guaraci/>** (in Portuguese), in three volumes:

| Volume | Contents |
|---|---|
| I — Technical Manual | physics, nuclear data, transport engine, statistics, validation |
| II — User Manual | the `.gci` language, materials, sources, probes, commands, results |
| III — Graphical Interface | the desktop program (in development) |

## Repository

| Folder | Contents |
|---|---|
| `crates/` | the Rust crates (`guaraci-units`, `-input`, `-data`, `-core`, `-cpu`, `-cli`) |
| `data/` | material database |
| `docs/` | documentation site sources (MkDocs Material) |
| `examples/` | input files |
| `validation/` | benchmarks and comparisons with GATE and MCNPX |
| `tools/` | generators |

See also the project [philosophy](PHILOSOPHY.md).
