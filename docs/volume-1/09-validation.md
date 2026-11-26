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
- 5×10⁵ histories per case; the statistical uncertainty is typically below 1 %.

The case-by-case results are in `validation/stabin2000_comparison.csv`. To reproduce:

```bash
GUARACI_TEST_DATA=~/.local/share/guaraci cargo test --release -p guaraci-cpu stabin_spheres_full -- --ignored --nocapture
```

Deviation from the recommended value (selected masses):

| Energy | 1 g | 10 g | 100 g | 1000 g | 6000 g |
|---|---|---|---|---|---|
| 20 keV | +7.6 % | +5.0 % | +3.2 % | +1.4 % | +0.6 % |
| 30 keV | +8.8 % | +6.7 % | +4.9 % | +2.6 % | +1.0 % |
| 40 keV | +6.0 % | +6.5 % | +6.8 % | +2.5 % | +0.8 % |
| 60 keV | +3.5 % | +3.6 % | +6.1 % | +1.7 % | −0.1 % |
| 80 keV | +4.0 % | +2.7 % | +4.3 % | +0.6 % | 0.0 % |
| 100 keV | +1.9 % | +2.5 % | +3.9 % | +0.6 % | +0.3 % |
| 140 keV | −2.0 % | +1.5 % | +3.4 % | — | — |
| 160 keV | — | — | — | +0.4 % | −0.3 % |
| 364 keV | +1.0 % | +2.2 % | +3.1 % | +0.7 % | −0.1 % |
| 662 keV | +8.2 % | +4.2 % | +3.4 % | +1.1 % | 0.0 % |
| 1460 keV | +48.6 % | +17.4 % | +7.7 % | +3.8 % | +1.8 % |
| 2750 keV | +171.4 % | +46.9 % | +20.9 % | +7.6 % | +4.0 % |

### Reading the results

1. **From 80 to 364 keV, and above 300 g at any energy up to 662 keV:** agreement
   within 0–4 %. The two reference codes differ from each other by up to about
   6 % (as the paper points out, Table 3).
2. **Above 1 MeV, in small spheres:** Guaraci overestimates, from +8 % at 100 g
   to +171 % at 1 g and 2.75 MeV. The deviation falls with the size of the
   sphere, as expected. This is the limitation of the current version: Compton
   electrons (up to ~2.5 MeV, with a range of ~1 cm in tissue) are deposited on
   the spot, whereas the reference codes transport them and they escape from
   small spheres. The same effect, smaller, appears at 662 keV below 10 g.
   **This benchmark will be repeated with electron transport.**
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
   and Guaraci lies between the two reference codes, closer to EGS4.

The `stabin_spheres_selected` test, run with every `cargo test`, uses five cases
of region 1 with a tolerance of 6 % + 3σ.

## Comparison with other codes

The [vial benchmark](../examples.md) provides equivalent inputs for Guaraci,
GATE 10 and MCNPX.

## Pending

- Repeat the sphere benchmark with electron transport (photons and electrons).
- Results of the GATE 10 and MCNPX comparisons.
