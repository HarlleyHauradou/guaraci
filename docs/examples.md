# Examples

The files are in the
[`examples/`](https://github.com/HarlleyHauradou/guaraci/tree/main/examples) folder of the repository.

## I-131 in the thyroid

[`examples/thyroid_i131.gci`](https://github.com/HarlleyHauradou/guaraci/blob/main/examples/thyroid_i131.gci):
an ellipsoidal thyroid inside a cylindrical neck of soft tissue, with 3.7 GBq of I-131.

```bash
guaraci run examples/thyroid_i131
```

Result with 2×10⁶ decays: mean dose rate in the thyroid of 7.43×10⁻³ Gy/s.
If all the β energy and part of the photon energy stayed in the thyroid, the
hand calculation would give 3.7 GBq × 0.2055 MeV per decay ÷ 15.7 g =
7.76×10⁻³ Gy/s, which is also what Guaraci gives with `list = photon_local`.
With electron transport, the β particles emitted near the surface leave the
thyroid and the result is about 4 % lower.

## Y-90 microspheres in a liver tumour

[`examples/y90_microspheres.gci`](https://github.com/HarlleyHauradou/guaraci/blob/main/examples/y90_microspheres.gci):
1 GBq of Y-90, a pure β⁻ emitter, for 1 h in a tumour of 15 mm radius inside
the liver, as in radioembolisation. The dose comes almost entirely from
electrons with a range of up to 11 mm, so electron transport is essential.
Result with 10⁵ decays: 33.2 Gy in the tumour, 91 % of the 36.6 Gy that full
absorption would give.

## Tc-99m lesion in a water phantom

[`examples/tc99m_lesion.gci`](https://github.com/HarlleyHauradou/guaraci/blob/main/examples/tc99m_lesion.gci):
a sphere of 15 mm radius in a water cylinder, with results per decay. It
includes a fluence spectrum showing the 140.5 keV peak and the Compton continuum.

## I-131 vial: Guaraci vs GATE 10 vs EGSnrc vs FLUKA

[`validation/vial_i131/`](https://github.com/HarlleyHauradou/guaraci/tree/main/validation/vial_i131):
a 10 mL vial of water with I-131, photon transport only. A generator writes
equivalent inputs for Guaraci, GATE 10, EGSnrc and FLUKA, a script runs and
times them all on the same machine, and another one compares the results and
draws the plots. With `--beta`, the same vial holds a β⁻ emitter such as Y-90
and the four codes transport the electrons:
`python3 make_inputs.py --nuclide Y-90 --beta --out inputs_y90`.

```bash
cd validation/vial_i131
python3 make_inputs.py
python3 run_all.py
python3 compare.py
```
