# 4. A linguagem `.gci`

## Estrutura

Um arquivo `.gci` é uma sequência de **seções**. Cada seção começa com um
cabeçalho entre colchetes e tem linhas `chave = valor`:

```
[volume lesion]        # tipo da seção e nome
shape = sphere
radius = 15 mm
```

- Comentários começam com `#` e vão até o fim da linha.
- Uma informação por linha.
- `run`, `physics` e `world` aparecem uma vez e não têm nome.
- `volume`, `material` e `probe` precisam de nome; o nome de `source` é opcional.

## Valores

| Tipo | Exemplos |
|---|---|
| número | `10`, `-1.5e3`, `1e7`, `10_000_000` |
| número com unidade | `10 cm`, `10cm`, `140.5 keV`, `1.05 g/cm3`, `89 %` |
| palavra | `Water`, `sphere`, `I-131` |
| texto | `"Tissue, Soft (ICRP)"` (use aspas se houver espaços ou vírgulas) |
| sim/não | `on`/`off`, `yes`/`no`, `true`/`false` |
| lista | `[3, 0, 0]`, `[1 cm, 2cm, 30 mm]`, `[lesion, liver]` |
| tabela | `{ H = 0.112, O = 0.888 }` |

Listas podem ocupar várias linhas.

## Unidades

A unidade vem depois do número, com ou sem espaço. Numa lista, uma unidade
depois do `]` vale para todos os elementos que não têm unidade própria:

```
size  = [10, 10, 10] cm
voxel = [2 mm, 2 mm, 0.25 cm]
```

**Sem unidade, vale o padrão da grandeza:**

| Grandeza | Padrão | Outras unidades |
|---|---|---|
| comprimento | cm | nm, um, mm, m |
| energia | keV | eV, MeV, GeV, J |
| atividade | MBq | Bq, kBq, GBq, TBq, uCi, mCi, Ci |
| tempo | s | ns, us, ms, min, h, d, y |
| densidade | g/cm3 | kg/m3, g/cc, mg/cm3… |
| ângulo | deg | rad, mrad |

A lista completa é dada por `guaraci info units`. Unidades compostas são
aceitas: `g/cm3`, `g/cm^3`, `kg*m^-3`, `MBq*h`.

O `guaraci check` lista todo número escrito sem unidade e a unidade assumida
para cada um.

**Maiúsculas e minúsculas:**

- A grafia exata vem primeiro.
- `CM` e `KEV` são aceitos porque não há outra unidade com essas letras.
- Se a troca de caixa mudasse o significado, a grafia exata seria obrigatória.

**As unidades são conferidas:** `radius = 5 keV` é um erro ("esperava comprimento").

## Incluir outros arquivos

```
include materials, geometry/body
```

Isso lê `materials.gci` e `geometry/body.gci`; a extensão é opcional. O caminho
é relativo ao arquivo que contém o `include`. Um arquivo incluído duas vezes é
lido uma vez só, e inclusões circulares são detectadas.

## Erros

O Guaraci lê o arquivo inteiro e mostra **todos** os problemas de uma vez,
cada um apontando o trecho do texto:

```
  × `radius`: expected length, but `keV` is a unit of energy
    ╭─[tireoide.gci:13:12]
 13 │ radius = 6 keV
    ·            ───
    ╰────
  help: use a unit of length, e.g. `cm`
```

Nomes errados recebem sugestões: `in = tiroide` → *did you mean `thyroid`?*
