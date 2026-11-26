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

Para rodar:

```bash
guaraci run tireoide
```

Na primeira execução, o Guaraci baixa os dados do ENDF/B-VIII.0 (cerca de 50 MB, só
uma vez). Depois ele mostra o progresso e, ao final, um resumo:

```
Done: 10000000 histories in 4s (2.594e6 histories/s, 4 threads)
  probe dose: maximum dose 8.2042e-3 Gy/s ± 1.18 % (among the 10564 voxels with uncertainty ≤ 10 %)
  probe organs:
    thyroid          7.7560e-3 Gy/s  ± 4.1e-3 %
    neck             2.4018e-5 Gy/s  ± 0.05 %
```

Como a fonte tem `activity`, os resultados são **taxas de dose** (Gy/s). Os
arquivos ficam em `tireoide_results/` (veja o [capítulo 10](10-resultados.md)).

Para um teste rápido, reduza as histórias sem editar o arquivo:

```bash
guaraci run tireoide --histories 1e5
```
