# Vial benchmark: Guaraci × GATE 10 × MCNPX

A 10 mL borosilicate glass vial filled with water and a uniform I-131 source,
photon transport only. The three input files describe exactly the same
problem, so the differences between the results come from the physics and
the cross sections of each code.

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
| Guaraci | default (`photon_local` physics); the source has photon lines only, no β |
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

The photon lines are not stored in the repository: they are read from
Guaraci's decay data when the inputs are generated.

```bash
python3 make_inputs.py                       # I-131, 1e7 photons, writes inputs/
python3 make_inputs.py --nuclide Tc-99m      # any nuclide of ENDF/B-VIII.0
```

This writes `inputs/vial.gci`, `inputs/gate10_vial.py`,
`inputs/mcnpx_vial.inp` and `inputs/lines.csv`.

## Running

```bash
guaraci run inputs/vial -o vial_results       # Guaraci
(cd inputs && python3 gate10_vial.py)         # GATE 10, results in inputs/gate_output/
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
