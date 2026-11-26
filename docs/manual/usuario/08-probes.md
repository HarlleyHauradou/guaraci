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
| `dose_grid` | `in`*, `voxel`* | mapa 3D de dose sobre a caixa que envolve o volume (`.mhd` + `.raw`, abre no 3D Slicer, ImageJ, Amide) |
| `volume_dose` | `volumes`* | energia e dose média em cada volume (`.csv`) |
| `spectrum` | `in`*, `emax` (1000 keV), `bins` (200) | espectro de fluência de fótons (`.csv`) |

`output` muda o nome do arquivo de saída (padrão: `<nome>.mhd` ou `<nome>.csv`).

O cálculo dos probes está **(em desenvolvimento)**.
