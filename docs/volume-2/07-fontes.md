# 7. Fontes e radionuclídeos

```ini
[source]
nuclide = I-131
in = thyroid
activity = 3.7 GBq
```

## O que emite

Escolha **uma** das formas:

| Chave | Exemplo | Descrição |
|---|---|---|
| `nuclide` | `I-131` | emissões do nuclídeo, tiradas da biblioteca de decaimento ENDF/B-VIII.0 |
| `energy` (+ `intensity`) | `energy = [140.5, 364.5] keV`, `intensity = [0.89, 0.81]` | linhas definidas por você (intensidade = fótons por decaimento; padrão 1) |

O nome do nuclídeo pode ser escrito de várias formas, todas equivalentes:
`I-131`, `I131`, `131I`, `i-131`; para isômeros, `Tc-99m`, `Tc99m`, `99mTc`.
Para ver os dados de um nuclídeo:

```bash
guaraci info nuclide Tc-99m
```

Qualquer nuclídeo da biblioteca de decaimento pode ser usado (3821 no ENDF/B-VIII.0).

## Onde está

| Chave | Descrição |
|---|---|
| `in = <volume>` | distribuída uniformemente no volume |
| `position = [x, y, z]` | fonte pontual |

O material do volume e o nuclídeo são independentes: o volume diz onde está a
atividade, e o material diz do que é feito o meio.

## Quanto

| Chave | Descrição |
|---|---|
| `activity` | atividade (padrão de unidade: MBq) |
| `duration` | tempo de aquisição/exposição; com `activity`, dá o número de decaimentos |

Sem `activity`, os resultados são por decaimento.

## Tratamento das emissões nesta versão

| Emissão | Exemplo | Tratamento |
|---|---|---|
| gamas e raios X | I-131 364 keV, Tc-99m 140 keV | transportados |
| aniquilação de pósitron | F-18, Ga-68 | fótons de 511 keV transportados a partir do ponto de decaimento |
| β⁻, β⁺ (energia cinética), elétrons Auger e de conversão | I-131, Lu-177, C-14 | energia depositada no ponto de emissão |

A deposição local é uma boa aproximação quando o voxel é bem maior que o
alcance do elétron (cerca de 0,3 mm para o C-14 e de até 2–3 mm para o I-131 no
tecido). O transporte de elétrons virá numa versão futura.
