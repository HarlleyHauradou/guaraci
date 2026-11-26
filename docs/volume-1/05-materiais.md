# 5. Banco de materiais

`data/materials.gci` é gerado por `tools/materials/pnnl15870rev1_to_gci.py` a
partir do texto (`pdftotext -layout`) do PNNL-15870 Rev. 1.

## Regras de conversão

- **Composição:** usam-se as frações de massa da tabela de cada material.
  Isótopos são somados por elemento (pela coluna Photon ZA).
- **Elementos com fração zero** na fonte (o Am em compostos de Pu) são omitidos.
- **Identificadores:** o nome do compêndio vira CamelCase, e cada parte
  separada por vírgula vira um bloco ligado por `_` (`Tissue, Soft (ICRP)` →
  `Tissue_Soft_ICRP`). O texto entre parênteses só entra quando é preciso para
  distinguir materiais. `names.json` corrige casos especiais (`Water, Liquid` → `Water`).
- **Nomes alternativos** viram seções com `same_as`. Eles vêm de três fontes:
  - o índice do compêndio ("Perspex (see Lucite)");
  - as abreviações entre parênteses (BGO, LYSO, PMMA);
  - `aliases.json` (NaI, CsI, Quartz…).

  Entradas ambíguas do índice são puladas de propósito.
- **Normalização:** quando as frações de massa da fonte não somam 1, o fato
  fica registrado na referência do material, e o Guaraci normaliza.
