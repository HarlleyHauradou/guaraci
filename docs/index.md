---
html_theme.sidebar_secondary.remove: true
---

# Guaraci

**Transporte de radiação por Monte Carlo em CPU e GPU**, com foco inicial em
física médica e medicina nuclear.

*Guaraci é o Sol na mitologia tupi, a fonte da radiação que dá vida a tudo.*

::::{grid} 1 2 2 2
:gutter: 3

:::{grid-item-card} {octicon}`download` Instalar
:link: volume-2/02-instalacao
:link-type: doc

Compile com Cargo. Os dados nucleares (ENDF/B-VIII.0) são baixados das
fontes oficiais na primeira simulação.
:::

:::{grid-item-card} {octicon}`play` Primeira simulação
:link: volume-2/03-primeira-simulacao
:link-type: doc

I-131 na tireoide em 30 linhas legíveis, do arquivo de entrada ao mapa de dose.
:::

:::{grid-item-card} {octicon}`book` Volume II — Manual do Usuário
:link: volume-2/index
:link-type: doc

A linguagem `.gci`, materiais, fontes, probes, comandos e resultados.
:::

:::{grid-item-card} {octicon}`beaker` Volume I — Manual Técnico
:link: volume-1/index
:link-type: doc

Física, dados nucleares, motor de transporte, estatística e validação.
:::

::::

## Um arquivo de entrada

```ini
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
center = [0, 2, 0]                # sem unidade = cm
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

```bash
guaraci check tireoide     # mostra o que o Guaraci entendeu, sem rodar
guaraci run tireoide       # roda e grava dose, incertezas e relatório
```

## O que já funciona

| | |
|---|---|
| **Física** | efeito fotoelétrico, Compton (Klein-Nishina com S(x,Z)), Rayleigh (fator de forma), produção de pares |
| **Fontes** | 3821 nuclídeos do ENDF/B-VIII.0, linhas definidas pelo usuário, atividade e duração |
| **Geometria** | caixa, esfera, cilindro e elipsoide aninhados, com verificação de sobreposição |
| **Materiais** | 372 materiais do PNNL-15870 Rev. 1, além de fórmulas químicas e composições próprias |
| **Probes** | mapa de dose 3D (`.mhd`), dose por volume e espectro de fluência, todos com incerteza |
| **Desempenho** | milhões de histórias por segundo em CPU multinúcleo; resultados idênticos com qualquer número de threads |
| **Validação** | μ/ρ a menos de 0,4 % do NIST XCOM; frações absorvidas em 231 esferas comparadas com Stabin & Konijnenberg (2000) |

Em desenvolvimento: fantomas voxelizados (CT/DICOM), transporte de elétrons,
backend CUDA e a interface gráfica ([Volume III](volume-3/index.md)).

## Os três volumes

| Volume | Para quem |
|---|---|
| [**I — Manual Técnico**](volume-1/index.md) | quem quer saber *como* o Guaraci calcula: teoria, dados, métodos e validação |
| [**II — Manual do Usuário**](volume-2/index.md) | quem monta e roda simulações; é por aqui que se aprende |
| [**III — Interface Gráfica**](volume-3/index.md) | quem usa o programa gráfico (editor, visualização da geometria e dos resultados) |

```{toctree}
:hidden:
:maxdepth: 2

Vol. I · Técnico <volume-1/index>
Vol. II · Usuário <volume-2/index>
Vol. III · Interface <volume-3/index>
Exemplos <exemplos>
Desenvolvimento <desenvolvimento>
Filosofia <filosofia>
```
