# 6. Materiais

## Banco embutido

O Guaraci traz 372 materiais do *Compendium of Material Composition Data for
Radiation Transport Modeling* (PNNL-15870 Rev. 1, 2011), mais 147 nomes
alternativos. Para procurar:

```bash
guaraci info materials            # todos
guaraci info materials tissue     # filtra pelo nome ou título
guaraci info materials LYSO
```

Alguns exemplos:

| Uso | Nomes |
|---|---|
| tecidos | `Tissue_Soft_ICRP`, `Tissue_Soft_ICRUFourComponent`, `Tissue_Lung`, `Tissue_Adipose`, `Tissue_Breast`, `Blood`, `Brain`, `EyeLens`, `Skin`, `Muscle_Skeletal` |
| ossos | `Bone_Cortical`, `Bone_Compact` |
| simuladores | `Water`, `PMMA` (= `Lucite`), `A150TissueEquivalentPlastic`, `BoneEquivalentPlastic_B100` |
| detectores | `NaI`, `CsI`, `BGO`, `LYSO`, `LSO`, `GSO`, `CdTe` |
| blindagem | `Lead`, `Tungsten`, `Concrete_Ordinary_NIST`, `Steel_Stainless304` |

Nomes ambíguos do compêndio não viraram atalhos. "Soft Tissue", por exemplo,
existe nas versões ICRP e ICRU, e por isso você escolhe explicitamente entre
`Tissue_Soft_ICRP` e `Tissue_Soft_ICRUFourComponent`.

## Definir um material

Por fração de massa (o Guaraci normaliza a soma para 1):

```ini
[material MyGel]
density = 1.05 g/cm3
composition = { H = 0.10, C = 0.12, O = 0.75, N = 0.03 }
```

Copiando outro material com outra densidade:

```ini
[material Isopor]
same_as = Polystyrene
density = 0.03 g/cm3
```

Uma definição sua com o mesmo nome de um material do banco **substitui** o do
banco. Para valer em vários arquivos, junte as definições num arquivo seu e use
`include`.

### Chaves

| Chave | Descrição |
|---|---|
| `density` | densidade (obrigatória, exceto com `same_as`) |
| `composition` | tabela `{ elemento = fração de massa }` |
| `formula` | fórmula química, ex. `H2O` **(em desenvolvimento)** |
| `same_as` | copia composição (e densidade, se não for dada) de outro material |
| `title`, `comment`, `reference`, `number` | informação descritiva, mostrada nos relatórios |

### Elemento puro

Um material de um só elemento é definido como qualquer outro. Muitos
elementos já estão no banco (`Lead`, `Aluminum`, `Iron`, `Gold`, `Tungsten`…);
para os outros, dê a densidade:

```ini
[material Iodine]
density = 4.93 g/cm3
composition = { I = 1 }
```

### Isótopos

No transporte de fótons, só o número atômico importa: C-14 atenua como
carbono comum. Por isso as composições do banco dadas por isótopo (U, Pu, Li,
B…) foram somadas por elemento.

O nuclídeo radioativo de uma fonte **não** entra no material; isso é feito na
seção `[source]` (capítulo 7).
