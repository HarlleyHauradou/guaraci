# 10. Results

`guaraci run thyroid` creates the folder `thyroid_results/` with:

| File | Contents |
|---|---|
| `report.txt` | complete report of the simulation |
| `input/` | copy of every input file used |
| `<probe>.mhd` + `.raw` | dose map (`dose_grid`) |
| `<probe>_uncertainty.mhd` + `.raw` | relative uncertainty (1σ) of every voxel |
| `<probe>.csv` | `volume_dose` and `spectrum` |

## Units of the results

The unit depends on what the input says about the activity:

| Sources | Result | Dose unit |
|---|---|---|
| no `activity` | per decay | Gy per decay |
| with `activity` | rate | Gy/s |
| with `activity` and `duration` | total | Gy |

With several sources, each simulated decay comes from a source chosen in
proportion to its activity (or activity × duration). Either every source has
an `activity`, or none has.

## Uncertainty

The histories are split into `batches` batches (default 10). The uncertainty is
the standard deviation of the mean between batches (1σ).

- **Values above ~5 % call for more histories.** The uncertainty falls as
  1/√N: 4× more histories halve it.
- **For dose maps, the summary shows the maximum only among the voxels with an
  uncertainty ≤ 10 %.** The raw maximum is almost always a voxel with very
  little mass and a single deposit.

## The report

`report.txt` is meant to be read as a lab notebook:

- the whole simulation as Guaraci understood it (the same output as `check`);
- the origin of the data, with SHA-256 and installation date;
- the normalisation used;
- the **energy balance**: emitted = deposited + escaped; the difference should
  be of the order of 10⁻¹⁰ % or less;
- the results, the performance (histories/s) and the limitations of the version.

## Opening the `.mhd` maps

- **3D Slicer:** *File → Add Data*, choose the `.mhd`.
- **ImageJ/Fiji:** *File → Import → MHD/MHA* (or the *MetaImage* plugin).
- **Amide:** *File → Import File (generic)*.
- **Python:**

```python
import SimpleITK as sitk
dose = sitk.GetArrayFromImage(sitk.ReadImage("thyroid_results/dose.mhd"))  # [z, y, x]
```

Distances in the `.mhd` are in **mm**, as these programs expect, and the
`Offset` is the centre of the first voxel.
