# 7. Sources and radionuclides

```gci
[source]
nuclide = I-131
in = thyroid
activity = 3.7 GBq
```

## What it emits

Choose **one** of:

| Key | Example | Description |
|---|---|---|
| `nuclide` | `I-131` | emissions of the nuclide, from the ENDF/B-VIII.0 decay library |
| `energy` (+ `intensity`) | `energy = [140.5, 364.5] keV`, `intensity = [0.89, 0.81]` | lines of your own (intensity = photons per decay; default 1) |

The nuclide name can be written in several equivalent ways: `I-131`, `I131`,
`131I`, `i-131`; for isomers, `Tc-99m`, `Tc99m`, `99mTc`. To see the data of a
nuclide:

```bash
guaraci info nuclide Tc-99m
guaraci info nuclide Tc-99m --csv     # photon lines as CSV, e.g. for other codes
```

Any nuclide of the decay library can be used (3821 in ENDF/B-VIII.0).

## Where it is

| Key | Description |
|---|---|
| `in = <volume>` | spread uniformly over the volume (its region: the solid minus its daughters) |
| `position = [x, y, z]` | point source |

The material of the volume and the nuclide are independent: the volume says
where the activity is, and the material says what the medium is made of.

## How much

| Key | Description |
|---|---|
| `activity` | activity (default unit: MBq) |
| `duration` | acquisition/exposure time; with `activity`, it gives the number of decays |

Without `activity`, results are per decay.

## Electrons

| Key | Default | Description |
|---|---|---|
| `beta` | `local` | `local`: the mean energy of β⁻, β⁺, Auger and conversion electrons is deposited at the decay point; `off`: ignored (to separate the photon contribution) |

## How emissions are treated in this version

| Emission | Example | Treatment |
|---|---|---|
| gamma and X-rays | I-131 364 keV, Tc-99m 140 keV | transported |
| positron annihilation | F-18, Ga-68 | back-to-back 511 keV photons transported from the decay point |
| β⁻, β⁺ (kinetic energy), Auger and conversion electrons | I-131, Lu-177, C-14 | energy deposited at the emission point |

Local deposition is a good approximation when the voxel is much larger than
the electron range (about 0.3 mm for C-14 and up to 2–3 mm for I-131 in
tissue). Electron transport will come in a future version.
