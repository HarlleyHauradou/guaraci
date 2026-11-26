# Examples

The files are in the
[`examples/`](https://github.com/HarlleyHauradou/guaraci/tree/main/examples) folder of the repository.

## I-131 in the thyroid

[`examples/thyroid_i131.gci`](https://github.com/HarlleyHauradou/guaraci/blob/main/examples/thyroid_i131.gci):
an ellipsoidal thyroid inside a cylindrical neck of soft tissue, with 3.7 GBq of I-131.

```bash
guaraci run examples/thyroid_i131
```

Result with 10⁷ decays: mean dose rate in the thyroid of 7.756×10⁻³ Gy/s,
which is what the hand calculation gives: 3.7 GBq × 0.2055 MeV per decay ÷ 15.7 g.

## Tc-99m lesion in a water phantom

[`examples/tc99m_lesion.gci`](https://github.com/HarlleyHauradou/guaraci/blob/main/examples/tc99m_lesion.gci):
a sphere of 15 mm radius in a water cylinder, with results per decay. It
includes a fluence spectrum showing the 140.5 keV peak and the Compton continuum.

## I-131 vial: Guaraci vs GATE 10 vs MCNPX

[`validation/vial_i131/`](https://github.com/HarlleyHauradou/guaraci/tree/main/validation/vial_i131):
a 10 mL vial of water with I-131, photon transport only. A generator writes
three equivalent inputs (Guaraci, GATE 10 and MCNPX), and a script compares the results.

```bash
cd validation/vial_i131
python3 make_inputs.py
guaraci run inputs/vial -o vial_results
python3 compare.py
```
