# 1. Arquitetura

O Guaraci é um *workspace* Cargo com um crate por responsabilidade:

| Crate | Papel | Situação |
|---|---|---|
| `guaraci-units` | grandezas físicas com dimensão; tabela de unidades | pronto |
| `guaraci-input` | linguagem `.gci`: parser, `include`, diagnósticos, configuração tipada | pronto |
| `guaraci-data` | leitor ENDF-6, download, conversão e cache de dados | pronto |
| `guaraci-cli` | o programa `guaraci` | pronto |
| `guaraci-core` | física, geometria, fontes, probes (sem E/S) | pronto (fótons) |
| `guaraci-cpu` | motor paralelo em CPU | pronto |
| `guaraci-cuda` | motor em GPU (CUDA) | planejado |
| `guaraci-gui` | interface gráfica | planejado |

## Fluxo

```
arquivos .gci ──parser──▶ AST com posições ──leitor──▶ Config (unidades internas)
                                                              │
dados ENDF (cache) ──────────────────────────────────────────▶ Problem ──▶ Backend (CPU/GPU) ──▶ probes
```

- A `Config` usa unidades fixas: cm, MeV, g/cm³, s, Bq, rad.
- O `Problem` será uma estrutura "achatada", sem ponteiros, que pode ser
  copiada como está para a GPU. Os dois backends implementam o mesmo trait.

## Convenções

- Nenhum dado de terceiros é embutido no código, exceto o banco de materiais
  gerado do PNNL-15870 Rev. 1 (por decisão do projeto) e os símbolos dos elementos.
- Toda mensagem ao usuário que se refere à entrada carrega a posição no
  texto (`Span`).
