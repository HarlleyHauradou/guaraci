# 3. First simulation

Create the file `thyroid.gci`:

```ini
# I-131 in the thyroid, inside a simplified neck.

[run]
histories = 1e7
seed = 2026

[world]
material = Air
size = [50, 50, 50] cm

[volume neck]
shape = cylinder
radius = 6 cm
height = 15 cm
material = Tissue_Soft_ICRP

[volume thyroid]
parent = neck
shape = ellipsoid
semi_axes = [1, 1.5, 2.5] cm
center = [0, 2, 0]
material = Tissue_Soft_ICRP

[source]
nuclide = I-131
in = thyroid
activity = 3.7 GBq

[probe dose]
type = dose_grid
in = neck
voxel = [2, 2, 2] mm
```

Check what Guaraci understood, without running:

```bash
guaraci check thyroid
```

The `.gci` extension is optional. The output shows:

- the geometry as a tree;
- the materials, with composition and origin;
- the gamma lines of I-131 from ENDF/B-VIII.0;
- the default values that were applied;
- the numbers written without a unit, and the unit assumed for each.

To run:

```bash
guaraci run thyroid
```

On the first run, Guaraci downloads the ENDF/B-VIII.0 data (about 50 MB, only
once). It then shows the progress and, at the end, a summary:

```text
Done: 10000000 histories in 4s (2.594e6 histories/s, 4 threads)
  probe dose: maximum dose 8.2042e-3 Gy/s ± 1.18 % (among the 10564 voxels with uncertainty ≤ 10 %)
  probe organs:
    thyroid          7.7560e-3 Gy/s  ± 4.1e-3 %
    neck             2.4018e-5 Gy/s  ± 0.05 %
```

Because the source has an `activity`, the results are **dose rates** (Gy/s).
The files are in `thyroid_results/` (see [chapter 10](10-results.md)).

For a quick test, reduce the number of histories without editing the file:

```bash
guaraci run thyroid --histories 1e5
```
