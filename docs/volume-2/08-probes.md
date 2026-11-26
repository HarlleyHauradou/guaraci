# 8. Probes

A **probe** measures something during the simulation. It is the equivalent of
the MCNP *tally* and the GATE *actor*.

```gci
[probe dose]
type = dose_grid
in = neck
voxel = [2, 2, 2] mm
```

| `type` | Keys | Result |
|---|---|---|
| `dose_grid` | `in`*, `voxel`* | 3D dose map over the bounding box of the volume (`.mhd` + `.raw`) and a map of the relative uncertainty |
| `volume_dose` | `volumes`* | energy and mean dose in each volume (`.csv`) |
| `spectrum` | `in`*, `emax` (1000 keV), `bins` (200) | photon fluence spectrum in the volume (`.csv`) |

`output` changes the name of the output file (default: `<name>.mhd` or `<name>.csv`).

## Details

- **Region of a volume:** its solid **minus** its daughter volumes. The dose in
  `neck` does not include the thyroid, which is its daughter; to include it,
  list both.
- **`dose_grid`:**
  - the grid covers the bounding box of the volume, including what lies outside the volume;
  - the mass of each voxel is computed from 27 points inside it, so a voxel cut
    by a boundary gets the mass it really contains;
  - the dose of a voxel is the mean over the materials inside it.
- **`spectrum`:**
  - uses the track-length estimator (fluence = sum of track lengths ÷ volume);
  - the `fluence_per_keV` column divides by the bin width.
