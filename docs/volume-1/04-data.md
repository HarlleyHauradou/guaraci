# 4. Atomic and nuclear data

Crate `guaraci-data`.

## Policy

Data files are not redistributed. Each library of the catalogue
(`library.rs`) records:

- the official URL and the download page;
- the expected SHA-256 and the size.

The installation process:

1. downloads the file (through `HTTPS_PROXY`, if set) or uses the file given with `--from`;
2. checks the SHA-256 (for automatic downloads only; a `--from` file that
   differs from the catalogue gives a warning, not an error);
3. converts the ENDF-6 files to serialised Rust structures (bincode);
4. writes `<GUARACI_DATA>/<library>/<kind>.bin`, with a provenance header
   (source, SHA-256, date, format version, number of entries);
5. deletes the downloaded file.

| Library | File | SHA-256 |
|---|---|---|
| ENDF/B-VIII.0 photoat | `ENDF-B-VIII.0_photoat.zip` (35 776 679 B) | `61aee2b8…0248e835d`[^sha] |
| ENDF/B-VIII.0 decay | `ENDF-B-VIII.0_decay.zip` (14 173 710 B) | `9087395f…f8d03d5e`[^sha] |
| ENDF/B-VIII.0 electrons | `ENDF-B-VIII.0_electrons.zip` (9 060 819 B) | `98f5ce1e…e694c5d4`[^sha] |

[^sha]: The full values are in `crates/guaraci-data/src/library.rs`.

## ENDF-6 reader

`endf.rs` implements the CONT/HEAD, LIST, TAB1 and TAB2 records of the 80-column format:

- Real numbers accept the compact form without `E` (`1.234567+5`) and blank fields (= 0).
- The file is split into `(MF, MT)` sections by columns 71–75.

## Photo-atomic data (MF23, MF27)

| Section | Quantity | Unit in Guaraci |
|---|---|---|
| MF23 MT502 | coherent (Rayleigh) cross section | barn vs MeV |
| MF23 MT504 | incoherent (Compton) cross section | barn vs MeV |
| MF23 MT516 | pair production (nuclear + electron field) | barn vs MeV |
| MF23 MT522 | photoelectric, all shells | barn vs MeV |
| MF27 MT502 | form factor F(x, Z) | x = sin(θ/2)/λ in cm⁻¹ |
| MF27 MT504 | incoherent scattering function S(x, Z) | same |

- Energies are converted from eV to MeV.
- The atomic mass comes from the AWR of the file: m = AWR × mₙ, with
  mₙ = 1.00866491595 u (CODATA 2018). No table of atomic masses is embedded.
- Every table of ENDF/B-VIII.0 uses linear-linear interpolation (INT = 2). The
  converter rejects other laws rather than interpolating them wrongly.

## Decay data (MF8 MT457)

For each nuclide, Guaraci keeps:

- the half-life and the decay modes (RTYP, branching ratio);
- the mean energies per decay: light particles ⟨E_LP⟩ (β⁻, β⁺, Auger and
  conversion electrons), electromagnetic ⟨E_EM⟩ and heavy particles ⟨E_HP⟩;
- the discrete photon lines, with absolute intensity = FD × RI:
  - STYP 0 becomes **gamma**;
  - STYP 9 becomes **X-ray**, or **annihilation** when the energy is within 50 eV of 510.999 keV.
- the β branches:
  - STYP 1 (β⁻): endpoint = ER, intensity = FD × RI;
  - STYP 2 (β⁺): endpoint = ER − 2mₑc², because for this type ER is the decay
    energy of the branch; intensity = FD × RIS (RI counts β⁺ and electron capture together);
  - the transition type (allowed, first-forbidden unique...) for the spectrum shape;
- the discrete electrons (STYP 8: conversion and Auger) and the mean β energy per decay.

Continuous photon spectra (rare) are flagged but not used yet.

## Electro-atomic data (MF23, MF26)

The ENDF/B-VIII.0 electron sublibrary is EEDL (EPICS2017). For each element,
Guaraci derives on a grid from 100 eV to 100 MeV (40 points per decade):

| Section | Use |
|---|---|
| MF23 MT525, MF26 MT525 | large-angle elastic cross section and angular distribution → transport cross section σ_el (1 − ⟨cos θ⟩) |
| MF23 MT526 | total elastic cross section |
| MF23/MF26 MT527 | bremsstrahlung: cross section, mean photon energy (LAW = 8) and photon spectra (LAW = 1) |
| MF23/MF26 MT528, MT534+ | excitation and subshell ionisation (converted, but not used for transport: see chapter 7) |

The value 1 − ⟨cos θ⟩ is interpolated log-log between the tabulated energies.

## Reference tests

With `GUARACI_TEST_ENDF` pointing to the unpacked files, the tests check:

| Case | Expected value |
|---|---|
| μ/ρ of water at 140 keV | 0.1538 cm²/g (NIST XCOM), 1 % tolerance |
| I-131 | T½ = 8.025 d; 364.49 keV with 81.5 % |
| Tc-99m | T½ = 6.007 h; 140.51 keV with 88.5 % |
| F-18 | 1.934 annihilation photons per decay |
| C-14 | no photons; ⟨E_LP⟩ = 49.5 keV |
| I-131 β⁻ | main branch: endpoint 606.3 keV, 89.6 %; mean β energy 181.9 keV per decay |
| F-18 β⁺ | 96.7 % positrons, endpoint 633.5 keV |
| water, EEDL | radiative stopping power vs ESTAR (printed); bremsstrahlung spectra normalised, σ_b⟨k⟩ = S_rad within 3 % |
