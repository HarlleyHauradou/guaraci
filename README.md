<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/_static/brand/guaraci-assinatura-sobre-escuro.svg">
    <img src="docs/_static/brand/guaraci-assinatura.svg" width="380" alt="guaraci">
  </picture>
</p>

<p align="center">
  <b>English</b> · <a href="README.pt-BR.md">Português</a> · <a href="README.es.md">Español</a>
</p>

<p align="center">
  <b>Monte Carlo radiation transport on CPU and GPU</b><br>
  Focused initially on medical physics and nuclear medicine.
</p>

<p align="center">
  <a href="https://guaraci.readthedocs.io/en/latest/"><b>Documentation</b></a> ·
  <a href="docs/volume-2/03-first-simulation.md">First simulation</a> ·
  <a href="docs/volume-1/09-validation.md">Validation</a> ·
  <a href="examples/">Examples</a>
</p>

<p align="center">
  <a href="https://github.com/HarlleyHauradou/guaraci/actions/workflows/ci.yml"><img src="https://github.com/HarlleyHauradou/guaraci/actions/workflows/ci.yml/badge.svg" alt="ci"></a>
  <a href="https://guaraci.readthedocs.io/en/latest/"><img src="https://readthedocs.org/projects/guaraci/badge/?version=latest" alt="docs"></a>
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
- **Electron and positron transport** (mixed condensed history): Bethe stopping powers with ICRU 37 mean excitation energies, δ-rays, bremsstrahlung, multiple scattering, β spectra from Fermi theory, annihilation.
- **Radionuclide sources** from ENDF/B-VIII.0 decay data (3821 nuclides), downloaded from the official source on first use — no third-party data is redistributed.
- **Human-readable input** with units (`15 mm`, `3.7 GBq`), includes, and error messages that point at the line and suggest the fix.
- **372 materials** from PNNL-15870 Rev. 1.
- **Probes**: 3D dose maps (MetaImage), dose in volumes, fluence spectra — all with statistical uncertainties.
- **Validated**: μ/ρ within 0.4 % of NIST XCOM; stopping powers and ranges within 1.1 % of NIST ESTAR; photon and electron absorbed fractions in spheres compared with Stabin & Konijnenberg (2000); exact energy conservation; results independent of the thread count.

In development: voxelised phantoms (CT/DICOM), CUDA backend, graphical interface.

## Install

```bash
git clone https://github.com/HarlleyHauradou/guaraci
cd guaraci
cargo install --path crates/guaraci-cli
guaraci run examples/thyroid_i131
```

## Documentation

The manual is online, in English, Portuguese and Spanish, at **<https://guaraci.readthedocs.io/en/latest/>**, in three volumes:

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
| `docs/` | documentation sources (Sphinx + PyData theme, Markdown with MyST), published on Read the Docs |
| `examples/` | input files |
| `validation/` | benchmarks and comparisons with GATE, EGSnrc and FLUKA |
| `tools/` | generators |

See also the project [philosophy](PHILOSOPHY.md).


## Author

**Harlley Hauradou** · [ORCID 0000-0003-3560-9708](https://orcid.org/0000-0003-3560-9708) · <hhauradou@gmail.com>

## How to cite

If you use Guaraci in your work, please cite it. GitHub's **Cite this
repository** button (from [`CITATION.cff`](CITATION.cff)) gives the reference
in APA and BibTeX:

```bibtex
@software{guaraci,
  author  = {Hauradou, Harlley},
  title   = {Guaraci: Monte Carlo radiation transport on CPU and GPU},
  version = {0.2.0},
  year    = {2026},
  url     = {https://github.com/HarlleyHauradou/guaraci}
}
```

## License

Guaraci is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE)), or
- MIT license ([LICENSE-MIT](LICENSE-MIT)),

at your option. Unless you explicitly state otherwise, any contribution
intentionally submitted for inclusion in Guaraci by you, as defined in the
Apache-2.0 license, shall be dual licensed as above, without any additional
terms or conditions.

The Guaraci name, logos and sun symbol are not covered by these licenses
([brand terms](docs/_static/brand/LICENSE.md)). Data of other origin is listed
in [NOTICE](NOTICE).
