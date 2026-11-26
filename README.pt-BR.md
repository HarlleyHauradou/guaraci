<p align="center">
  <img src="docs/_static/logo.svg" width="90" alt="">
</p>

<h1 align="center">Guaraci</h1>

<p align="center">
  <a href="README.md">English</a> · <b>Português</b> · <a href="README.es.md">Español</a>
</p>

<p align="center">
  <b>Transporte de radiação por Monte Carlo em CPU e GPU</b><br>
  Com foco inicial em física médica e medicina nuclear.
</p>

<p align="center">
  <a href="https://harlleyhauradou.github.io/guaraci/pt-br/"><b>Documentação</b></a> ·
  <a href="docs/volume-2/03-first-simulation.md">Primeira simulação</a> ·
  <a href="docs/volume-1/08-validation.md">Validação</a> ·
  <a href="examples/">Exemplos</a>
</p>

<p align="center">
  <a href="https://github.com/HarlleyHauradou/guaraci/actions/workflows/ci.yml"><img src="https://github.com/HarlleyHauradou/guaraci/actions/workflows/ci.yml/badge.svg" alt="ci"></a>
  <a href="https://github.com/HarlleyHauradou/guaraci/actions/workflows/docs.yml"><img src="https://github.com/HarlleyHauradou/guaraci/actions/workflows/docs.yml/badge.svg" alt="docs"></a>
</p>

---

*Guaraci é o Sol na mitologia tupi, a fonte da radiação que dá vida a tudo.*

```ini
[volume thyroid]
parent = neck
shape = ellipsoid
semi_axes = [1, 1.5, 2.5] cm
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
guaraci check thyroid    # o que o Guaraci entendeu, sem rodar
guaraci run thyroid      # mapa de dose, incertezas e um relatório legível
```

## Recursos

- **Transporte de fótons** em CPUs multinúcleo: efeito fotoelétrico, Compton (Klein-Nishina com S(x,Z)), Rayleigh (fatores de forma), produção de pares.
- **Fontes radioativas** a partir dos dados de decaimento do ENDF/B-VIII.0 (3821 nuclídeos), baixados da fonte oficial no primeiro uso; nenhum dado de terceiros é redistribuído.
- **Entrada legível** com unidades (`15 mm`, `3.7 GBq`), inclusão de arquivos e mensagens de erro que apontam a linha e sugerem a correção.
- **372 materiais** do PNNL-15870 Rev. 1.
- **Probes:** mapas de dose 3D (MetaImage), dose em volumes e espectros de fluência, todos com incerteza estatística.
- **Validação:** μ/ρ a menos de 0,4 % do NIST XCOM; frações absorvidas em 231 esferas comparadas com Stabin & Konijnenberg (2000); conservação exata de energia; resultados independentes do número de threads.

Em desenvolvimento: fantomas voxelizados (CT/DICOM), transporte de elétrons, backend CUDA e interface gráfica.

## Instalação

```bash
git clone https://github.com/HarlleyHauradou/guaraci
cd guaraci
cargo install --path crates/guaraci-cli
guaraci run examples/thyroid_i131
```

## Documentação

O manual está online, em inglês, português e espanhol, em **<https://harlleyhauradou.github.io/guaraci/pt-br/>**, em três volumes:

| Volume | Conteúdo |
|---|---|
| I — Manual Técnico | física, dados nucleares, motor de transporte, estatística, validação |
| II — Manual do Usuário | a linguagem `.gci`, materiais, fontes, probes, comandos, resultados |
| III — Interface Gráfica | o programa desktop (em desenvolvimento) |

O programa, suas mensagens e suas saídas são em inglês, a língua da comunidade científica internacional.

## Repositório

| Pasta | Conteúdo |
|---|---|
| `crates/` | os crates Rust (`guaraci-units`, `-input`, `-data`, `-core`, `-cpu`, `-cli`) |
| `data/` | banco de materiais |
| `docs/` | fontes da documentação (Sphinx + tema PyData, Markdown com MyST), publicada no GitHub Pages |
| `examples/` | arquivos de entrada |
| `validation/` | benchmarks e comparações com GATE e MCNPX |
| `tools/` | geradores |

Veja também a [filosofia](PHILOSOPHY.md) do projeto.
