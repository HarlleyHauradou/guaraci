# 1. Introduction

Guaraci is a Monte Carlo code for radiation transport in matter, aimed first
at medical physics and nuclear medicine. Like MCNP and GATE, it runs from the
terminal with input files (`.gci`), and it can compute on CPU or GPU.

## What this version does

- Transports photons on the CPU, in parallel: photoelectric effect, Compton,
  Rayleigh and pair production (`guaraci run`).
- Transports electrons and positrons: stopping powers, δ-rays,
  bremsstrahlung, multiple scattering and annihilation.
- Scores 3D dose grids, dose in volumes and fluence spectra, each with its
  statistical uncertainty.
- Reads and checks input files, with error messages that point at the line
  and suggest the fix (`guaraci check`).
- Downloads, verifies and converts the data libraries (ENDF/B-VIII.0).
- Ships a database of 372 materials (PNNL-15870 Rev. 1).
- Knows the decay data of 3821 nuclides.

Electrons and positrons are transported (mixed condensed history), including
β particles from the decay data. The GPU backend is **in development**.

## Principles

Guaraci follows the [project philosophy](../philosophy.md). Two practical
rules come from it:

- **Everything you do not write has a known default value**, and
  `guaraci check` lists every one of them.
- **Nothing is guessed.** An ambiguous name or unit is an error, with a suggestion.
