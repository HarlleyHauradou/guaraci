# 8. Probes

Um **probe** mede algo durante a simulação. É o equivalente ao *tally* do MCNP
e ao *actor* do GATE.

```
[probe dose]
type = dose_grid
in = neck
voxel = [2, 2, 2] mm
```

| `type` | Chaves | Resultado |
|---|---|---|
| `dose_grid` | `in`*, `voxel`* | mapa 3D de dose sobre a caixa que envolve o volume (`.mhd` + `.raw`) e mapa da incerteza relativa |
| `volume_dose` | `volumes`* | energia e dose média em cada volume (`.csv`) |
| `spectrum` | `in`*, `emax` (1000 keV), `bins` (200) | espectro de fluência de fótons no volume (`.csv`) |

`output` muda o nome do arquivo de saída (padrão: `<nome>.mhd` ou `<nome>.csv`).

## Detalhes

- **Região de um volume:** é o sólido dele **menos** os volumes filhos. A dose em
  `neck` não inclui a tireoide, que é filha dele; para incluí-la, liste os dois.
- **`dose_grid`:**
  - a grade cobre a caixa que envolve o volume, inclusive o que está fora dele;
  - a massa de cada voxel é calculada com 27 pontos internos, então um voxel
    cortado por uma fronteira recebe a massa que realmente contém;
  - a dose do voxel é a média sobre os materiais dentro dele.
- **`spectrum`:**
  - usa o estimador de comprimento de trajetória (fluência = soma dos comprimentos ÷ volume);
  - a coluna `fluence_per_keV` divide pela largura do bin.
