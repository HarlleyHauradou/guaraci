# 9. Comandos

| Comando | O que faz |
|---|---|
| `guaraci check <entrada>` | lê, valida e descreve a simulação, sem rodar |
| `guaraci run <entrada>` | roda a simulação |
| `guaraci run <entrada> -o pasta` | grava os resultados em `pasta` (padrão: `<entrada>_results`) |
| `guaraci run <entrada> --histories 1e6` | muda o número de histórias sem editar o arquivo |
| `guaraci run <entrada> --threads 8` | número de threads (padrão: todos os núcleos) |
| `guaraci run <entrada> --offline` | nunca baixa dados; falha se faltar algum |
| `guaraci data list` | bibliotecas disponíveis e instaladas |
| `guaraci data install <biblioteca>` | baixa, confere e converte uma biblioteca |
| `guaraci data install <bib> --kind <photoatomic\|decay> --from <arquivo>` | instala de um arquivo já baixado |
| `guaraci data path` | pasta onde os dados ficam |
| `guaraci info units` | todas as unidades |
| `guaraci info materials [filtro]` | materiais do banco |
| `guaraci info nuclide <nome>` | dados de decaimento de um nuclídeo |

## Variáveis de ambiente

| Variável | Efeito |
|---|---|
| `GUARACI_DATA` | pasta dos dados nucleares |
| `GUARACI_OFFLINE` | o mesmo que `--offline` |
| `NO_COLOR` | desliga as cores das mensagens |

Durante uma simulação, **Ctrl-C** termina o lote em andamento e grava os
resultados parciais. Um segundo Ctrl-C sai imediatamente.
