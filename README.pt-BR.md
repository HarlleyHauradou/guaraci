<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/_static/brand/guaraci-assinatura-sobre-escuro.svg">
    <img src="docs/_static/brand/guaraci-assinatura.svg" width="380" alt="guaraci">
  </picture>
</p>

<p align="center">
  <a href="README.md">English</a> · <b>Português</b> · <a href="README.es.md">Español</a>
</p>

<p align="center">
  <b>Transporte de radiação por Monte Carlo em CPU e GPU</b><br>
  Com foco inicial em física médica e medicina nuclear.
</p>

<p align="center">
  <a href="https://guaraci.readthedocs.io/pt-br/latest/"><b>Documentação</b></a> ·
  <a href="docs/volume-2/03-first-simulation.md">Primeira simulação</a> ·
  <a href="docs/volume-1/09-validation.md">Validação</a> ·
  <a href="examples/">Exemplos</a>
</p>

<p align="center">
  <a href="https://github.com/HarlleyHauradou/guaraci/actions/workflows/ci.yml"><img src="https://github.com/HarlleyHauradou/guaraci/actions/workflows/ci.yml/badge.svg" alt="ci"></a>
  <a href="https://guaraci.readthedocs.io/pt-br/latest/"><img src="https://readthedocs.org/projects/guaraci/badge/?version=latest" alt="docs"></a>
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
- **Transporte de elétrons e pósitrons** (história condensada mista): poder de freamento de Bethe com as energias médias de excitação do ICRU 37, raios δ, bremsstrahlung, espalhamento múltiplo, espectros β pela teoria de Fermi, aniquilação.
- **Fontes radioativas** a partir dos dados de decaimento do ENDF/B-VIII.0 (3821 nuclídeos), baixados da fonte oficial no primeiro uso; nenhum dado de terceiros é redistribuído.
- **Entrada legível** com unidades (`15 mm`, `3.7 GBq`), inclusão de arquivos e mensagens de erro que apontam a linha e sugerem a correção.
- **372 materiais** do PNNL-15870 Rev. 1.
- **Probes:** mapas de dose 3D (MetaImage), dose em volumes e espectros de fluência, todos com incerteza estatística.
- **Validação:** μ/ρ a menos de 0,4 % do NIST XCOM; poder de freamento e alcance a menos de 1,1 % do NIST ESTAR; frações absorvidas de fótons e elétrons em esferas comparadas com Stabin & Konijnenberg (2000); conservação exata de energia; resultados independentes do número de threads.

Em desenvolvimento: fantomas voxelizados (CT/DICOM), backend CUDA e interface gráfica.

## Instalação

```bash
git clone https://github.com/HarlleyHauradou/guaraci
cd guaraci
cargo install --path crates/guaraci-cli
guaraci run examples/thyroid_i131
```

## Documentação

O manual está online, em inglês, português e espanhol, em **<https://guaraci.readthedocs.io/pt-br/latest/>**, em três volumes:

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
| `docs/` | fontes da documentação (Sphinx + tema PyData, Markdown com MyST), publicada no Read the Docs |
| `examples/` | arquivos de entrada |
| `validation/` | benchmarks e comparações com GATE, EGSnrc e FLUKA |
| `tools/` | geradores |

Veja também a [filosofia](PHILOSOPHY.md) do projeto.


## Autor

**Harlley Hauradou** · [ORCID 0000-0003-3560-9708](https://orcid.org/0000-0003-3560-9708) · <hhauradou@gmail.com>

## Como citar

Se você usar o Guaraci no seu trabalho, cite-o, por favor. O botão **Cite this
repository** do GitHub (gerado a partir do [`CITATION.cff`](CITATION.cff)) dá a
referência em APA e BibTeX:

```bibtex
@software{guaraci,
  author  = {Hauradou, Harlley},
  title   = {Guaraci: Monte Carlo radiation transport on CPU and GPU},
  version = {0.2.0},
  year    = {2026},
  url     = {https://github.com/HarlleyHauradou/guaraci}
}
```

## Licença

O Guaraci é licenciado, à sua escolha, sob

- a Apache License, versão 2.0 ([LICENSE-APACHE](LICENSE-APACHE)), ou
- a licença MIT ([LICENSE-MIT](LICENSE-MIT)).

Salvo declaração explícita em contrário, qualquer contribuição enviada
intencionalmente para inclusão no Guaraci, conforme definido na licença
Apache-2.0, será licenciada nessas duas licenças, sem termos ou condições
adicionais.

O nome, os logos e o símbolo do sol do Guaraci não estão cobertos por essas
licenças ([termos da marca](docs/_static/brand/LICENSE.md)). Os dados de outras
origens estão listados no [NOTICE](NOTICE).
