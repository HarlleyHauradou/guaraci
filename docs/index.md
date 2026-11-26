---
html_theme.sidebar_secondary.remove: true
---

{.hero-title}
# Guaraci

```{image} _static/brand/guaraci-assinatura.svg
:alt: guaraci
:class: only-light hero-logo
```

```{image} _static/brand/guaraci-assinatura-sobre-escuro.svg
:alt: guaraci
:class: only-dark hero-logo
```

**Monte Carlo radiation transport on CPU and GPU**, focused initially on
medical physics and nuclear medicine.

*Guaraci is the Sun in Tupi mythology — the source of the radiation that gives life to everything.*

::::{grid} 1 2 2 2
:gutter: 3

:::{grid-item-card} {octicon}`download` Install
:link: volume-2/02-installation
:link-type: doc

Build with Cargo. The nuclear data (ENDF/B-VIII.0) is downloaded from the
official sources on the first simulation.
:::

:::{grid-item-card} {octicon}`play` First simulation
:link: volume-2/03-first-simulation
:link-type: doc

I-131 in the thyroid in 30 readable lines, from the input file to the dose map.
:::

:::{grid-item-card} {octicon}`book` Volume II — User Manual
:link: volume-2/index
:link-type: doc

The `.gci` language, materials, sources, probes, commands and results.
:::

:::{grid-item-card} {octicon}`beaker` Volume I — Technical Manual
:link: volume-1/index
:link-type: doc

Physics, nuclear data, transport engine, statistics and validation.
:::

::::

## An input file

```gci
[world]
material = Air
size = [50, 50, 50] cm

[volume neck]
shape = cylinder
radius = 6 cm
height = 15 cm
material = Tissue_Soft_ICRP

[volume thyroid]
parent = neck
shape = ellipsoid
semi_axes = [1, 1.5, 2.5] cm
center = [0, 2, 0]                # no unit = cm
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
guaraci check thyroid     # what Guaraci understood, without running
guaraci run thyroid       # runs and writes dose, uncertainties and a report
```

## What works today

| | |
|---|---|
| **Physics** | photons: photoelectric effect, Compton (Klein-Nishina with S(x,Z)), Rayleigh (form factors), pair production; electrons and positrons: mixed condensed history with δ-rays, bremsstrahlung, multiple scattering and annihilation |
| **Sources** | 3821 nuclides of ENDF/B-VIII.0 (γ, X, β⁻, β⁺, conversion and Auger electrons), user-defined photon, electron and positron lines, activity and duration |
| **Geometry** | nested boxes, spheres, cylinders and ellipsoids, with overlap checks |
| **Materials** | 372 materials of PNNL-15870 Rev. 1, chemical formulas and custom compositions |
| **Probes** | 3D dose maps (`.mhd`), dose in volumes and fluence spectra, all with uncertainties |
| **Performance** | millions of histories per second on multi-core CPUs; identical results with any number of threads |
| **Validation** | μ/ρ within 0.4 % of NIST XCOM; stopping powers and ranges within 1.1 % of NIST ESTAR; photon and electron absorbed fractions in spheres compared with Stabin & Konijnenberg (2000) |

In development: voxelised phantoms (CT/DICOM), CUDA backend and the graphical
interface ([Volume III](volume-3/index.md)).

## The three volumes

| Volume | For whom |
|---|---|
| [**I — Technical Manual**](volume-1/index.md) | those who want to know *how* Guaraci computes: theory, data, methods and validation |
| [**II — User Manual**](volume-2/index.md) | those who set up and run simulations; this is where to learn |
| [**III — Graphical Interface**](volume-3/index.md) | users of the desktop program (editor, geometry and results viewers) |

```{toctree}
:hidden:
:maxdepth: 2

Vol. I · Technical <volume-1/index>
Vol. II · User <volume-2/index>
Vol. III · Interface <volume-3/index>
Examples <examples>
Development <development>
Philosophy <philosophy>
```
