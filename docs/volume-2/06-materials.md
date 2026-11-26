# 6. Materials

## Built-in database

Guaraci ships the 372 materials of the *Compendium of Material Composition
Data for Radiation Transport Modeling* (PNNL-15870 Rev. 1, 2011), plus 147
alternative names. To search it:

```bash
guaraci info materials            # all of them
guaraci info materials tissue     # filter by name or title
guaraci info materials LYSO
```

Some examples:

| Use | Names |
|---|---|
| tissues | `Tissue_Soft_ICRP`, `Tissue_Soft_ICRUFourComponent`, `Tissue_Lung`, `Tissue_Adipose`, `Tissue_Breast`, `Blood`, `Brain`, `EyeLens`, `Skin`, `Muscle_Skeletal` |
| bone | `Bone_Cortical`, `Bone_Compact` |
| phantoms | `Water`, `PMMA` (= `Lucite`), `A150TissueEquivalentPlastic`, `BoneEquivalentPlastic_B100` |
| detectors | `NaI`, `CsI`, `BGO`, `LYSO`, `LSO`, `GSO`, `CdTe` |
| shielding | `Lead`, `Tungsten`, `Concrete_Ordinary_NIST`, `Steel_Stainless304` |

Ambiguous names of the compendium did not become shortcuts. "Soft Tissue",
for example, exists in an ICRP and an ICRU version, so you choose explicitly
between `Tissue_Soft_ICRP` and `Tissue_Soft_ICRUFourComponent`.

## Defining a material

By mass fractions (Guaraci normalises the sum to 1):

```ini
[material MyGel]
density = 1.05 g/cm3
composition = { H = 0.10, C = 0.12, O = 0.75, N = 0.03 }
```

By chemical formula:

```ini
[material Acetone]
density = 0.7899 g/cm3
formula = C3H6O
```

Copying another material with a different density:

```ini
[material Styrofoam]
same_as = Polystyrene
density = 0.03 g/cm3
```

A definition of yours with the same name as a database material **replaces**
the database one. To use your definitions in several simulations, put them in
a file of your own and `include` it.

### Keys

| Key | Description |
|---|---|
| `density` | density (required, except with `same_as`) |
| `composition` | table `{ element = mass fraction }` |
| `formula` | chemical formula, e.g. `H2O`, `Ca(OH)2`; atomic masses come from the data library |
| `same_as` | copies the composition (and the density, if not given) of another material |
| `title`, `comment`, `reference`, `number` | descriptive information, shown in the reports |

### Pure elements

A single-element material is defined like any other. Many elements are
already in the database (`Lead`, `Aluminum`, `Iron`, `Gold`, `Tungsten`…); for
the others, give the density:

```ini
[material Iodine]
density = 4.93 g/cm3
composition = { I = 1 }
```

### Isotopes

In photon transport only the atomic number matters: C-14 attenuates like
ordinary carbon. Isotopes are accepted in compositions (`{ C-14 = 1 }`) and
treated as the element; the compositions of the database given by isotope
(U, Pu, Li, B…) were summed by element.

The radioactive nuclide of a source does **not** go into the material; that is
done in the `[source]` section ([chapter 7](07-sources.md)).
