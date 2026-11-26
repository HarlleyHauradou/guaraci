# 9. Validation

The tests of `crates/guaraci-cpu/tests/benchmarks.rs` run with
`GUARACI_TEST_DATA` pointing to a data folder where ENDF/B-VIII.0 is installed:

```bash
GUARACI_TEST_DATA=~/.local/share/guaraci cargo test --release -p guaraci-cpu -- --nocapture
```

## Attenuation coefficients vs NIST XCOM

Total μ/ρ with coherent scattering (cm²/g), computed from the material tables
(ENDF/B-VIII.0 and PNNL-15870 Rev. 1 compositions):

| Material | Energy | Guaraci | XCOM | Deviation |
|---|---|---|---|---|
| Water | 10 keV | 5.3312 | 5.329 | +0.04 % |
| Water | 20 keV | 0.81073 | 0.8096 | +0.14 % |
| Water | 30 keV | 0.37609 | 0.3756 | +0.13 % |
| Water | 60 keV | 0.20583 | 0.2059 | −0.03 % |
| Water | 100 keV | 0.17064 | 0.1707 | −0.04 % |
| Water | 200 keV | 0.13698 | 0.1370 | −0.02 % |
| Water | 500 keV | 0.09681 | 0.09687 | −0.06 % |
| Water | 1 MeV | 0.07064 | 0.07072 | −0.12 % |
| Water | 2 MeV | 0.04933 | 0.04942 | −0.19 % |
| Water | 10 MeV | 0.02214 | 0.02219 | −0.25 % |
| Lead | 100 keV | 5.5500 | 5.549 | +0.02 % |
| Lead | 500 keV | 0.16119 | 0.1614 | −0.13 % |
| Lead | 1 MeV | 0.07076 | 0.07102 | −0.36 % |

Test tolerances: 2 % (water) and 3 % (lead).

## Uncollided fluence

A 140.5 keV point source at the centre of a water sphere of 10 cm radius, with
Rayleigh scattering off, surrounded by a vacuum shell from 10 to 10.5 cm. The
fluence of photons between 140.5 and 140.6 keV in the shell must be
e^(−μR) · ΔR / V_shell per emitted photon.

| Guaraci | Analytic | Deviation | Statistical σ |
|---|---|---|---|
| 1.68266×10⁻⁴ cm⁻² | 1.67848×10⁻⁴ cm⁻² | +0.25 % | 0.16 % |

This test exercises the geometry, the tables, the sampling of the free path and
the track-length estimator.

The bin must be narrow: a 140–141 keV bin also receives forward Compton-scattered
photons (θ < 9°) and overestimates the fluence by ~2 %.

## Electron stopping power and range vs NIST ESTAR

Liquid water (I = 75 eV from the Bragg rule), total stopping power in MeV cm²/g
and CSDA range in g/cm²:

| Energy | S (Guaraci) | S (ESTAR) | Deviation | R (Guaraci) | R (ESTAR) | Deviation |
|---|---|---|---|---|---|---|
| 10 keV | 22.546 | 22.56 | −0.06 % | — | — | — |
| 100 keV | 4.1171 | 4.121 | −0.10 % | 1.4322×10⁻² | 1.431×10⁻² | +0.08 % |
| 1 MeV | 1.8645 | 1.862 | +0.13 % | 0.43691 | 0.4367 | +0.05 % |
| 10 MeV | 2.1753 | 2.151 | +1.13 % | 4.9220 | 4.975 | −1.07 % |

The collision part is the Bethe formula, so the agreement checks the mean
excitation energy and the density effect. At 10 MeV, the small excess comes
from the radiative stopping power of EEDL. Tolerance of the test: 2 %.

## Compton sampling vs Klein-Nishina

For S(x) = Z (free electrons), the distribution of cos θ in 20 bins is compared
with the numerically integrated Klein-Nishina formula. The χ² test (19 degrees
of freedom) must stay below 43.8 (99.9 %) at 30 keV, 140.5 keV, 511 keV and 2 MeV.

## Energy conservation and thread independence

With an F-18 source in tissue containing cortical bone:

- emitted − deposited − escaped < 10⁻¹⁰ of the emitted energy;
- the results with 1 and with 4 threads are identical (relative difference < 10⁻¹²).

## Decay data

See [chapter 4](04-data.md#reference-tests).

## Absorbed fractions in spheres (Stabin and Konijnenberg, 2000)

**Reference:** M.G. Stabin and M.W. Konijnenberg, *Re-evaluation of absorbed
fractions for photons and electrons in spheres of various sizes*, J Nucl Med
2000;41:149–160, Tables 2 and 3. Unit-density spheres with a uniform
monoenergetic photon source, inside an infinite unit-density medium. The
recommended values are the mean of EGS4 and MCNP4B.

The 231 values (21 masses from 1 to 6000 g; 12 energies from 20 keV to
2.75 MeV) are in `validation/stabin2000_photons.csv`. Rows garbled by the text
extraction of the PDF were transcribed from the page images, and every row was
checked in two ways: the radius matches unit density, and the recommended
value is the EGS4/MCNP mean.

**Model in Guaraci:**

- the sphere and the medium are `Tissue_Soft_ICRP` (density 1);
- the infinite medium is a 1 m cube;
- absorbed fraction = energy deposited in the sphere ÷ energy emitted;
- physics list `photon_electron` (the default): Compton, photo- and pair
  electrons are transported;
- 5×10⁵ histories per case; the statistical uncertainty is typically below 1 %.

The case-by-case results are in `validation/stabin2000_comparison.csv`. To reproduce:

```bash
GUARACI_TEST_DATA=~/.local/share/guaraci cargo test --release -p guaraci-cpu stabin_spheres_full -- --ignored --nocapture
```

Deviation from the recommended value (selected masses; over the 231 cases, the
mean deviation is +1.8 %, and 182 values lie between EGS4 and MCNP within the
rounding of the table):

| Energy | 1 g | 10 g | 100 g | 1000 g | 6000 g |
|---|---|---|---|---|---|
| 20 keV | +7.2 % | +5.0 % | +3.0 % | +1.6 % | +0.6 % |
| 30 keV | +9.5 % | +7.0 % | +5.4 % | +2.8 % | +1.2 % |
| 40 keV | +6.3 % | +6.5 % | +7.1 % | +2.6 % | +1.2 % |
| 60 keV | +3.3 % | +3.8 % | +5.8 % | +1.9 % | +0.3 % |
| 80 keV | +3.9 % | +2.1 % | +4.4 % | +0.9 % | +0.3 % |
| 100 keV | +1.0 % | +2.3 % | +3.7 % | +0.8 % | +0.5 % |
| 140 keV | −2.0 % | +2.2 % | +3.0 % | — | — |
| 160 keV | — | — | — | +0.4 % | −0.2 % |
| 364 keV | −1.8 % | +1.9 % | +2.0 % | +0.4 % | −0.5 % |
| 662 keV | −3.7 % | +0.4 % | +0.5 % | −0.3 % | −0.5 % |
| 1460 keV | −0.8 % | −0.7 % | +1.0 % | +1.1 % | +0.3 % |
| 2750 keV | +11.9 % | −3.2 % | +1.0 % | −0.5 % | −0.2 % |

### Reading the results

1. **From 80 keV to 2.75 MeV:** agreement within 0–5 %. The two reference codes
   differ from each other by up to about 6 % (as the paper points out, Table 3).
   The one exception, +11.9 % at 1 g and 2.75 MeV, is a value the paper gives
   with a single significant figure (0.004; EGS4 gives 0.005).
2. **Above 1 MeV, in small spheres: the effect of electron transport.** With
   the previous version, in which Compton electrons (up to ~2.5 MeV, with a
   range of ~1 cm in tissue) were deposited on the spot, Guaraci overestimated
   by +49 % at 1 g and 1460 keV and by +171 % at 1 g and 2.75 MeV. With the
   electrons transported, they escape from small spheres as in the reference
   codes, and the deviations fall to −0.8 % and +11.9 %. At 662 keV and 1 g,
   the deviation goes from +8.2 % to −3.7 %. With `list = photon_local`, the
   previous behaviour returns.
3. **From 20 to 60 keV:** a systematic deviation of +3 to +8 % in small spheres.
   In this range the photoelectric effect dominates, and it is very sensitive to
   the composition. The paper uses the tissue of MIRD Pamphlets 3 and 8, whose
   composition is not given in the text. For a 10 g sphere at 30 keV:

| Material | Absorbed fraction | Deviation |
|---|---|---|
| Tissue_Soft_ICRP | 0.1523 | +6.5 % |
| Tissue_Soft_ICRUFourComponent | 0.1507 | +5.4 % |
| Water | 0.1612 | +12.7 % |
| EGS4 (paper) | 0.149 | +4.2 % |
| MCNP (paper) | 0.137 | −4.2 % |

   Changing the material moves the result more than the observed deviation,
   and Guaraci lies between the two reference codes, closer to EGS4. (These
   values were computed with `photon_local`; at 30 keV, electron transport
   changes them by less than 0.5 %.)

The `stabin_spheres_selected` test, run with every `cargo test`, uses five cases
of region 1 with a tolerance of 6 % + 3σ.

## Electron absorbed fractions in spheres (Stabin and Konijnenberg, 2000)

Table 1 of the same paper: uniform monoenergetic electron sources in the same
spheres, from 0.01 to 1000 g and from 0.1 to 4 MeV (133 values, in
`validation/stabin2000_electrons.csv`). The rows at 4 MeV came out scrambled
in the text extraction of the PDF and were reassembled by hand; every row was
checked against the EGS4/MCNP mean. The model is the one used for photons,
with `particle = electron` and 10⁵ histories per case. The radius comes from
the mass, since the table rounds it to 0.01 cm.

```bash
GUARACI_TEST_DATA=~/.local/share/guaraci cargo test --release -p guaraci-cpu stabin_electron_spheres_full -- --ignored --nocapture
```

Deviation from the recommended value (selected masses; all cases in
`validation/stabin2000_electrons_comparison.csv`):

| Energy | 0.01 g | 0.1 g | 1 g | 10 g | 100 g | 1000 g |
|---|---|---|---|---|---|---|
| 0.1 MeV | −0.5 % | −0.2 % | −0.2 % | −0.1 % | −0.1 % | 0.0 % |
| 0.2 MeV | +0.5 % | +0.1 % | +0.1 % | −0.1 % | 0.0 % | +0.1 % |
| 0.4 MeV | −0.1 % | −0.4 % | −0.1 % | +0.1 % | 0.0 % | 0.0 % |
| 0.7 MeV | +2.1 % | −0.8 % | +0.1 % | 0.0 % | +0.1 % | +0.2 % |
| 1 MeV | +5.4 % | +0.1 % | +0.2 % | +0.2 % | 0.0 % | +0.1 % |
| 2 MeV | +5.0 % | +5.3 % | +2.6 % | +0.6 % | +0.5 % | +0.2 % |
| 4 MeV | +4.7 % | +4.3 % | +6.3 % | +3.2 % | +1.2 % | +0.8 % |

Over the 133 cases, the mean deviation is +0.7 % and the largest is 6.6 %.

- **Up to 0.7 MeV, and above 10 g at any energy:** agreement within 1 %, except
  one case at 2.1 %.
- **Where the electron range is comparable to the sphere** (1–4 MeV, below 10 g):
  the two reference codes differ from each other by 5 to 15 %, and the
  recommended value is their mean. 123 of the 133 Guaraci values lie between
  EGS4 and MCNP, within the rounding of the table. The other 10 are 1 to 2 % above EGS4, with the largest
  deviations at 4 MeV.

Two corrections of this version came from this benchmark:

1. The interpolation of the elastic angular data (chapter 7): with linear
   interpolation, λ₁ was 2.5 times too short at 1 MeV, and the 0.1 g sphere at
   1 MeV absorbed 12 % too much.
2. The δ-rays and bremsstrahlung photons as discrete events: with the
   continuous loss alone, the 0.01 g sphere at 2 MeV absorbed 11 % too much,
   because these secondaries carry energy out of small spheres.

The `stabin_electron_spheres_selected` test, run with every `cargo test`, uses
five cases with a tolerance of 3 % + the EGS4–MCNP difference + 3σ.

## Comparison with other codes

The [vial benchmark](../examples.md) provides equivalent inputs for Guaraci,
GATE 10 and MCNPX.

## Pending

- Results of the GATE 10 and MCNPX comparisons.
- Electron benchmarks in heterogeneous geometries (depth-dose curves, backscatter).
