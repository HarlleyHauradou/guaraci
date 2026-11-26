# 5. Referência das seções

Chaves marcadas com * são obrigatórias. O valor padrão vale quando a chave é omitida.

## `[run]` — execução

| Chave | Tipo | Padrão | Descrição |
|---|---|---|---|
| `histories` | número inteiro | 1000000 | histórias (decaimentos simulados) |
| `seed` | número inteiro | 1 | semente do gerador aleatório |
| `rng` | `philox`, `xoshiro`, `pcg`, `mt19937` | `philox` | gerador de números aleatórios |
| `backend` | `cpu`, `cuda` | `cpu` | onde calcular |
| `threads` | número inteiro | 0 (todos os núcleos) | threads na CPU |
| `batches` | número inteiro | 10 | lotes para estimar a incerteza |

## `[physics]` — física

| Chave | Tipo | Padrão | Descrição |
|---|---|---|---|
| `list` | `photon_local` | `photon_local` | lista de física pronta |
| `data` | `endfb8`, `epics2017`, `epdl97` | `endfb8` | biblioteca de interações de fótons |
| `photoelectric` | on/off | on | efeito fotoelétrico |
| `compton` | `klein_nishina_sf`, `klein_nishina`, `off` | `klein_nishina_sf` | modelo Compton |
| `rayleigh` | on/off | on | espalhamento coerente |
| `pair` | on/off | on | produção de pares |
| `photon_cut` | energia | 1 keV | abaixo disso, o fóton deposita a energia no local |
| `particles` | lista | `[photon]` | partículas transportadas (só fótons nesta versão) |

A lista define os padrões, e qualquer chave escrita os substitui (como o cartão
MODE do MCNP). O `check` mostra a física efetivamente usada.

## `[world]` — o mundo

| Chave | Tipo | Descrição |
|---|---|---|
| `material`* | nome | material que preenche o mundo |
| `size`* | comprimento ou lista de 3 | lados da caixa (centrada na origem) |

## `[volume <nome>]` — volumes

| Chave | Tipo | Descrição |
|---|---|---|
| `shape`* | `box`, `sphere`, `cylinder`, `ellipsoid` | forma |
| `material`* | nome | material |
| `parent` | nome | volume que contém este (padrão: o mundo) |
| `center` | lista de 3 comprimentos | centro em relação ao centro do pai (padrão `[0, 0, 0]`) |

Dimensões por forma:

| Forma | Chaves |
|---|---|
| `box` | `size` (lados) |
| `sphere` | `radius` |
| `cylinder` | `radius`, `height` (eixo ao longo de z) |
| `ellipsoid` | `semi_axes` (semieixos em x, y, z) |

## `[material <nome>]`

Veja o [capítulo 6](06-materiais.md).

## `[source]` ou `[source <nome>]`

Veja o [capítulo 7](07-fontes.md).

## `[probe <nome>]`

Veja o [capítulo 8](08-probes.md).
