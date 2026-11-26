# 9. Comandos

| Comando | O que faz |
|---|---|
| `guaraci check <entrada>` | lê, valida e descreve a simulação, sem rodar |
| `guaraci run <entrada>` | roda a simulação **(em desenvolvimento)** |
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
| `NO_COLOR` | desliga as cores das mensagens |
