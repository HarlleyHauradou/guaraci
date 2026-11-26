# Material database generator

`data/materials.gci` is generated from the PNNL-15870 Rev. 1 compendium
(McConn, Gesh, Pagh, Rucker, Williams, *Compendium of Material Composition Data
for Radiation Transport Modeling*, PNNL-15870 Rev. 1, PNNL, 2011).

```bash
pdftotext -layout PNNL-15870Rev1.pdf pnnl.txt
python3 pnnl15870rev1_to_gci.py pnnl.txt names.json ../../data/materials.gci aliases.json
```

- Identifiers are built from the compendium names: `Tissue, Soft (ICRP)` becomes
  `Tissue_Soft_ICRP`; the text in parentheses is kept only when needed to tell
  materials apart.
- `names.json` overrides identifiers (e.g. `Water, Liquid` -> `Water`).
- Alternative names come from the compendium index (`Perspex (see Lucite)`), from
  abbreviations in parentheses (`BGO`, `LYSO`, `PMMA`) and from `aliases.json`.
  Ambiguous index entries (e.g. `Soft Tissue`, which has an ICRP and an ICRU
  version) are skipped on purpose: the user must choose.
- Isotopic compositions (U, Pu, Li, B, ...) are summed by element, since photon
  transport does not depend on the isotope.
