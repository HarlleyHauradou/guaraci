# Vial benchmark: Guaraci × GATE 10 × EGSnrc × FLUKA

A 10 mL borosilicate glass vial filled with water and a uniform radioactive
source. The four codes describe exactly the same problem, so the differences
between the results come from the physics and the cross sections of each code.
Every run is timed on the same machine. Two modes:

- **photons** (default): the photon lines of I-131, photon transport only;
- **beta** (`--beta`): the β⁻ spectrum of Y-90, electron and photon transport.

MCNPX inputs are also written (`mcnpx_vial.inp`), as an optional extra for
those who have a licence. They are not part of the cross-validation, nor of
the timing.

## Problem

| Item | Value |
|---|---|
| Vial (glass) | cylinder, radius 1.1 cm, height 4.5 cm, centred at the origin, axis z |
| Cavity (air) | cylinder, radius 1.0 cm, height 4.3 cm: 1 mm wall, bottom and top |
| Water | cylinder, radius 1.0 cm, height 3.2 cm (10.05 mL), on the bottom (centre z = −0.55 cm) |
| Around | air cube, 30 cm side (EGSnrc: air cylinder of 15 cm radius and 30 cm height, see below) |
| Materials | water 0.998207 g/cm³, air 0.001205 g/cm³, borosilicate glass 2.23 g/cm³; mass fractions of PNNL-15870 Rev. 1, written explicitly in every input |
| Source | uniform in the water, isotropic; the same spectrum in every code (below) |

**Same spectrum everywhere.** `make_inputs.py` takes the source from Guaraci's
decay data (ENDF/B-VIII.0) and writes it into every input:

- photons: the 32 γ and X lines of I-131 (`guaraci info nuclide I-131 --csv`);
- beta: the β⁻ spectrum of Y-90 as a 200-bin histogram (`--beta-csv`).

EGSnrc and FLUKA read the lines as 1 eV wide histogram bins, as EGSnrc's own
line spectra do. In FLUKA the primaries are sampled by the user routine
`source.f`, which `make_inputs.py` writes from the FLUKA 4 template.

### Physics

| Code | Photon mode | Beta mode |
|---|---|---|
| Guaraci | `list = photon_local` | `list = photon_electron`, `electron_cut = 10 keV` |
| GATE 10 | `G4EmLivermorePhysics`; electrons killed at creation (user limit in the air box `room`) | `G4EmStandardPhysics_option4`, production cut 0.01 mm |
| EGSnrc (DOSRZnrc) | `Global ECUT= 50` (electrons deposit locally), `Bound Compton scattering= simple` | `Global ECUT= 0.521` (10 keV), `PCUT= 0.001` |
| FLUKA 4 | `DEFAULTS PRECISIO`; `EMFCUT` −10 MeV: electrons deposit locally | `EMFCUT` 10 keV / 1 keV, also as production cuts |

In every code:
- no fluorescence (GATE `fluo` off, EGSnrc `Atomic relaxations= Off`, FLUKA `EMFFLUO` off);
- Rayleigh on;
- Compton with binding (S(x,Z)) and without Doppler broadening, where the code allows it.

The cross sections are each code's own: ENDF/B-VIII.0 (Guaraci), Livermore/EPDL
(GATE), XCOM (EGSnrc) and FLUKA's EPDL-based data.

### What each code scores

| Quantity | Guaraci | GATE 10 | EGSnrc | FLUKA |
|---|---|---|---|---|
| Energy in water and glass | `volume_dose` | `DoseActor` of one voxel on water, vial and cavity (glass = vial − cavity) | dose of every R-Z region × its mass | `USRBIN` R-Z, 1 mm bins aligned with the materials |
| Axial dose, r < 4 mm | 1 mm dose grid | 1 mm energy grid ÷ mass | regions of the 0.4 cm ring | `USRBIN` bins with r < 0.4 cm |
| Photon fluence in the water (photon mode) | `spectrum` | — | — | `USRTRACK` |

- The axial profile is compared in the water slices only. In the voxel codes,
  each voxel is weighted by the part of its area inside r < 4 mm, so that all
  four codes average over the same cylinder.
- For GATE, `compare.py` also checks that the energy of its dose grid equals
  water + glass.

## 1. Generate the inputs

```bash
python3 make_inputs.py --histories 1e7                                 # I-131, photons -> inputs/
python3 make_inputs.py --nuclide Y-90 --beta --histories 1e7 --out inputs_y90
```

Guaraci must be installed: the script calls `guaraci info nuclide` by itself,
and the decay data is downloaded on first use. If Guaraci fails, the script
shows its message.

## 2. Run and time everything: `run_all.py`

```bash
python3 run_all.py --inputs inputs --results results --gate-python ~/Gate/opengate_env/bin/python
python3 run_all.py --inputs inputs_y90 --results results_y90 --gate-python ~/Gate/opengate_env/bin/python
```

**What it does:**
- **One run at a time:** code after code, and for each code first with 1
  thread, then with every logical CPU (`--threads 1,16` by default on a
  16-thread machine). Two codes never run at the same time.
- **Guaraci and GATE** run with that many threads.
- **EGSnrc and FLUKA** are single-threaded, so they run that many independent
  processes. Each process has its own seed and 1/N of the histories, and the
  results are averaged.
- **Timing:** the wall-clock time from the start to the end of the last
  process is measured the same way for every code. It includes the
  initialisation of each code, so use enough histories (10⁷) for it to be
  small. Compiling FLUKA's `source.f` is not timed.
- **Output:**
  - `results/machine.txt`: CPU, memory, GPU, code versions;
  - `results/timing.csv`: one line per run;
  - one folder per run, e.g. `results/fluka_t16/`.

**Useful options:**
- `--codes guaraci,gate`: only some codes;
- `--threads 1,8`: other thread counts;
- `--histories 1e5`: a short test run;
- `--fluka-exe`: an already linked FLUKA executable.

**Before running:**
- **EGSnrc:** `EGS_HOME` and `HEN_HOUSE` set, and `dosrznrc` compiled. The
  media are defined in the input itself (pegsless), so no PEGS4 file is needed.
- **FLUKA:** `FLUPRO` set. The script compiles `source.f` with `fff` and links
  it with `lfluka` (from `$FLUPRO/bin` or `$FLUPRO/flutil`).
- **GATE:** `--gate-python` is the Python of the opengate environment.

Do a short run first (`--histories 1e5`): it catches input errors of a code
in seconds.

## 3. Compare

```bash
python3 compare.py --inputs inputs --results results
```

For each code, the run with the most threads is used (it has the most
histories per second, and the same total number of histories).

**What it prints:**
- energy in the water and in the glass per emitted particle, with the
  deviation of each code from the mean of the other three;
- the axial dose profile in the water;
- the photon fluence in the water;
- the speed table.

**What it writes:**
- `comparison.csv`;
- `comparison.png`, with four plots: energy, axial profile, fluence spectrum
  and histories per second. It needs matplotlib, which the opengate
  environment has.

**MCNPX (extra):** run it by hand and pass it with `--mcnp outp`. It is shown
apart, in the table only.

**If a code shows nothing:**
- EGSnrc: send the `.egslst` of its folder;
- FLUKA: send the `*_fort.21` file of one `run_k` folder.

The readers depend on the exact layout of these files, which could not be
checked without the codes installed.

## Guaraci results

Photons, I-131 (10⁷ decays):

| Quantity | Value |
|---|---|
| Energy in the water | 11.227 keV per photon (± 0.09 %) |
| Energy in the glass | 3.818 keV per photon (± 0.16 %) |
| Photon fluence in the water | 9.595 × 10⁻² cm⁻² per photon |
| Energy balance (emitted − deposited − escaped) | −1 × 10⁻¹¹ % |

Beta, Y-90 (10⁶ decays):

| Quantity | Value |
|---|---|
| Energy in the water | 786.9 keV per electron (± 0.05 %) |
| Energy in the glass | 113.9 keV per electron (± 0.16 %) |
| Energy escaping from the world (30 cm of air) | 43.0 keV per electron (4.5 %) |
| Energy balance | 5 × 10⁻¹² % |

**Y-90 spectrum:** the mean of the histogram, 945.8 keV, is 1.4 % above the
ENDF value (933 keV). Guaraci's shape for first-forbidden unique transitions
leaves out a Coulomb correction. All four codes use the same histogram, so the
comparison is not affected. Guaraci also emits the photon lines and conversion
electrons of Y-90 (less than 0.1 % of its energy); the other inputs leave them out.

## First comparison (I-131, before EGSnrc and FLUKA)

With 10⁷ histories, Guaraci and MCNPX agreed within 0.06 % (water), 0.25 %
(glass) and 0.09 % (photon fluence).
