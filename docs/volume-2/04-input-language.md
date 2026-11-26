# 4. The `.gci` input language

## Structure

A `.gci` file is a sequence of **sections**. Each section starts with a header
in square brackets, followed by `key = value` lines:

```ini
[volume lesion]        # section type and name
shape = sphere
radius = 15 mm
```

- Comments start with `#` and run to the end of the line.
- One setting per line.
- `run`, `physics` and `world` appear once and have no name.
- `volume`, `material` and `probe` need a name; the name of a `source` is optional.

## Values

| Type | Examples |
|---|---|
| number | `10`, `-1.5e3`, `1e7`, `10_000_000` |
| number with unit | `10 cm`, `10cm`, `140.5 keV`, `1.05 g/cm3`, `89 %` |
| word | `Water`, `sphere`, `I-131` |
| text | `"Tissue, Soft (ICRP)"` (use quotes for spaces or commas) |
| yes/no | `on`/`off`, `yes`/`no`, `true`/`false` |
| list | `[3, 0, 0]`, `[1 cm, 2cm, 30 mm]`, `[lesion, liver]` |
| table | `{ H = 0.112, O = 0.888 }` |

Lists may span several lines.

## Units

The unit comes after the number, with or without a space. In a list, a unit
after the `]` applies to every element that has no unit of its own:

```ini
size  = [10, 10, 10] cm
voxel = [2 mm, 2 mm, 0.25 cm]
```

**Without a unit, the default unit of the quantity applies:**

| Quantity | Default | Other units |
|---|---|---|
| length | cm | nm, um, mm, m |
| energy | keV | eV, MeV, GeV, J |
| activity | MBq | Bq, kBq, GBq, TBq, uCi, mCi, Ci |
| time | s | ns, us, ms, min, h, d, y |
| density | g/cm3 | kg/m3, g/cc, mg/cm3… |
| angle | deg | rad, mrad |

`guaraci info units` gives the complete list. Compound units are accepted:
`g/cm3`, `g/cm^3`, `kg*m^-3`, `MBq*h`.

`guaraci check` lists every number written without a unit, with the unit
assumed for it.

**Upper and lower case:**

- The exact spelling comes first.
- `CM` and `KEV` are accepted because no other unit has those letters.
- If changing the case changed the meaning, the exact spelling would be required.

**Units are checked:** `radius = 5 keV` is an error ("expected length").

## Including other files

```text
include materials, geometry/body
```

This reads `materials.gci` and `geometry/body.gci`; the extension is optional.
The path is relative to the file that contains the `include`. A file included
twice is read only once, and circular includes are detected.

## Errors

Guaraci reads the whole file and reports **every** problem at once, each one
pointing at the text:

```text
  × `radius`: expected length, but `keV` is a unit of energy
    ╭─[thyroid.gci:13:12]
 13 │ radius = 6 keV
    ·            ───
    ╰────
  help: use a unit of length, e.g. `cm`
```

Misspelt names get suggestions: `in = thyriod` → *did you mean `thyroid`?*
