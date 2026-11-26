# 3. Primeira simulação

Crie o arquivo `tireoide.gci`:

```
# I-131 na tireoide, dentro de um pescoço simplificado.

[run]
histories = 1e7
seed = 2026

[world]
material = Air
size = [50, 50, 50] cm

[volume neck]
shape = cylinder
radius = 6 cm
height = 15 cm
material = Tissue_Soft_ICRP

[volume thyroid]
parent = neck
shape = ellipsoid
semi_axes = [1, 1.5, 2.5] cm
center = [0, 2, 0]
material = Tissue_Soft_ICRP

[source]
nuclide = I-131
in = thyroid
activity = 3.7 GBq

[probe dose]
type = dose_grid
in = neck
voxel = [2, 2, 2] mm
```

Confira o que o Guaraci entendeu, sem rodar:

```bash
guaraci check tireoide
```

A extensão `.gci` é opcional. A saída mostra:

- a geometria em árvore;
- os materiais com composição e origem;
- as linhas gama do I-131 vindas do ENDF/B-VIII.0;
- os valores padrão aplicados;
- os números escritos sem unidade e a unidade assumida para cada um.

Para rodar: `guaraci run tireoide` **(em desenvolvimento)**.
