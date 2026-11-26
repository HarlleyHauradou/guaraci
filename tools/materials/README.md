# Material database generator

`data/materials.gci` is generated from the PNNL-15870 compendium
(Williams, Gesh, Pagh, *Compendium of Material Composition Data for Radiation
Transport Modeling*, PNNL, 2006).

```bash
pdftotext -layout PNNL-15870.pdf pnnl.txt
python3 pnnl15870_to_gci.py pnnl.txt names.json aliases.json ../../data/materials.gci
```

- `names.json` — short identifiers for materials whose automatic name is awkward.
- `aliases.json` — other names listed in the compendium ("See Lucite."), written as `same_as`.

Isotopic compositions (U, Pu) are summed by element, since photon transport does
not depend on the isotope.
