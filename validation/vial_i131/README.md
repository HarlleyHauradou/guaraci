# Vial benchmark: Guaraci × GATE 10 × MCNPX

A 10 mL borosilicate glass vial filled with water and a uniform radioactive
source. The three input files describe exactly the same problem, so the
differences between the results come from the physics and the cross sections
of each code. Two modes:

- **photons** (default): the photon lines of I-131, photon transport only;
- **beta** (`--beta`): the β⁻ spectrum of Y-90, electron and photon transport
  (see [Beta mode: Y-90](#beta-mode-y-90) below).

## Problem

| Item | Value |
|---|---|
| Vial (glass) | cylinder, radius 1.1 cm, height 4.5 cm, centred at the origin, axis z |
| Cavity (air) | cylinder, radius 1.0 cm, height 4.3 cm: 1 mm wall, bottom and top |
| Water | cylinder, radius 1.0 cm, height 3.2 cm (10.05 mL), on the bottom (centre z = −0.55 cm) |
| World | air cube, 30 cm side |
| Materials | water 0.998207 g/cm³, air 0.001205 g/cm³, borosilicate glass 2.23 g/cm³; mass fractions of PNNL-15870 Rev. 1, written explicitly in every input |
| Source | the 32 photon lines (γ and X) of I-131 from ENDF/B-VIII.0, uniform in the water, isotropic |
| Physics | photons only; electrons deposit their energy where they are created; no fluorescence; no Doppler broadening |

| Code | How "photons only, local electrons" is set |
|---|---|
| Guaraci | `list = photon_local`; the source has photon lines only, no β |
| GATE 10 | `G4EmLivermorePhysics`; fluo, Auger and PIXE off; electrons killed at creation (user limit `min_ekine`) |
| MCNPX | `MODE P`; `PHYS:P 100 1 0 0 1` (IDES = 1: no electrons; NODOP = 1: no Doppler) |

Remaining differences, to keep in mind when reading the results:

- **Cross sections:** Guaraci uses ENDF/B-VIII.0 (EPICS2014), GATE uses
  Livermore (EPDL97), MCNPX uses its mcplib library.
- **Fluorescence:** MCNPX keeps it in detailed photon physics, so a little
  energy goes into fluorescence photons there.
- **Compton scattering:** MCNPX and Guaraci use Klein-Nishina corrected by the
  incoherent scattering function; GATE's Livermore model also includes the
  Doppler broadening.

## Generating the inputs

The source is not stored in the repository: it is read from Guaraci's decay
data when the inputs are generated. `make_inputs.py` calls
`guaraci info nuclide <name> --csv` (or `--beta-csv`) by itself, so Guaraci
must be installed, but nothing needs to be run first; the decay data is
downloaded on first use. If Guaraci fails, the script shows its message.

```bash
python3 make_inputs.py                       # I-131, 1e7 photons, writes inputs/
python3 make_inputs.py --nuclide Tc-99m      # any nuclide of ENDF/B-VIII.0
```

This writes `inputs/vial.gci`, `inputs/gate10_vial.py`,
`inputs/mcnpx_vial.inp` and `inputs/lines.csv`.

## Running

```bash
guaraci run inputs/vial -o vial_results       # Guaraci
python3 inputs/gate10_vial.py                 # GATE 10, results in inputs/gate_output/
mcnpx i=inputs/mcnpx_vial.inp o=outp          # MCNPX
```

The GATE 10 and MCNPX inputs were written for this benchmark but could not be
run while it was prepared (neither code was available). If a parameter name
is rejected by your version, adjust that line; the intent of each one is
commented.

## Comparing

```bash
python3 compare.py --guaraci vial_results --gate inputs/gate_output --mcnp outp
```

Every result is converted to a value **per emitted photon**. GATE and MCNPX
emit one photon per event; Guaraci emits on average 1.0585 photons per decay,
the sum of the line intensities. The comparison covers:

1. **Energy deposited in the water and in the glass:**
   - Guaraci: `volume_dose` probe;
   - GATE: `DoseActor` with one voxel;
   - MCNPX: F6 tally × mass.
2. **Dose near the axis** (mean within 4 mm of the axis), from the 1 mm dose
   grids of Guaraci and GATE.
3. **Photon fluence in the water**: total, the 364 keV line and the scattered part:
   - Guaraci: `spectrum` probe;
   - MCNPX: F4 tally.

It prints a table and writes `comparison.csv`.

## Guaraci result (1e7 decays)

| Quantity | Value |
|---|---|
| Energy in the water | 11.227 keV per photon (± 0.09 %) |
| Energy in the glass | 3.818 keV per photon (± 0.16 %) |
| Photon fluence in the water | 9.595 × 10⁻² cm⁻² per photon |
| Energy balance (emitted − deposited − escaped) | −1 × 10⁻¹¹ % |

## Beta mode: Y-90

Y-90 is a pure β⁻ emitter (endpoint 2.28 MeV, range up to ~11 mm in water).
In the 1 cm radius vial most β particles stop in the water, but part of them
reach the glass and some leave the vial: the result tests electron transport,
and in particular the interfaces.

```bash
python3 make_inputs.py --nuclide Y-90 --beta --out inputs_y90
guaraci run inputs_y90/vial -o y90_results
python3 inputs_y90/gate10_vial.py
mcnpx i=inputs_y90/mcnpx_vial.inp o=outp_y90
python3 compare.py --inputs inputs_y90 --guaraci y90_results --gate inputs_y90/gate_output --mcnp outp_y90
```

| | Guaraci | GATE 10 | MCNPX |
|---|---|---|---|
| Source | `nuclide = Y-90`, `beta = transport` | `e-`, histogram of `beta.csv` | `PAR=3`, `SI3 H`/`SP3 D` from `beta.csv` |
| Physics | `photon_electron`, `electron_cut = 10 keV` | `G4EmStandardPhysics_option4`, cut 0.01 mm | `MODE E P`, `CUT:E J 0.01` |
| Energy deposited | `volume_dose` | `DoseActor` (water) | `*F8:E,P` (water, glass) |

- **Same spectrum:** the β⁻ histogram (200 bins, all branches) is written by
  `guaraci info nuclide Y-90 --beta-csv`; it is the spectrum Guaraci samples.
  Its mean, 945.8 keV, is 1.4 % above the ENDF value (933 keV), because the
  shape of first-forbidden unique transitions leaves out a Coulomb
  correction. This does not affect the comparison: all three codes use it.
- **What Guaraci adds:** the photon lines and conversion electrons of Y-90
  (less than 0.1 % of its energy), which the GATE and MCNPX inputs leave out.
- **Comparison:** `compare.py` finds `beta.csv` in the inputs folder and
  expresses every result per emitted electron. It compares the energy in the
  water and in the glass, and the axial dose profile of Guaraci and GATE.
  Differences of a few % between electron codes are usual, mostly near the
  interfaces.

Guaraci result (10⁶ decays, 47 s on 4 threads):

| Quantity | Value |
|---|---|
| Energy in the water | 786.9 keV per electron (± 0.05 %) |
| Energy in the glass | 113.9 keV per electron (± 0.16 %) |
| Energy escaping from the world (30 cm of air) | 43.0 keV per electron (4.5 %) |
| Energy balance (emitted − deposited − escaped) | 5 × 10⁻¹² % |

