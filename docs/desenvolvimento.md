# Desenvolvimento

## Organização do código

| Pasta | Conteúdo |
|---|---|
| `crates/guaraci-units` | grandezas físicas e unidades |
| `crates/guaraci-input` | a linguagem `.gci` |
| `crates/guaraci-data` | leitor ENDF-6, download e cache dos dados |
| `crates/guaraci-core` | física, geometria, fontes e probes |
| `crates/guaraci-cpu` | motor paralelo em CPU |
| `crates/guaraci-cli` | o programa `guaraci` |
| `data/` | banco de materiais (`materials.gci`) |
| `docs/` | este site (MkDocs Material) |
| `examples/` | arquivos de entrada de exemplo |
| `tools/` | geradores (banco de materiais) |
| `validation/` | benchmarks e comparações com outros códigos |

Os detalhes de cada parte estão no [Volume I](volume-1/01-arquitetura.md).

## Compilar e testar

```bash
cargo build --release
cargo test --workspace
```

Os testes que precisam dos dados nucleares reais são pulados se as variáveis
abaixo não estiverem definidas:

```bash
guaraci data install endfb8
GUARACI_TEST_DATA=~/.local/share/guaraci cargo test --release --workspace
```

O benchmark completo das esferas de Stabin & Konijnenberg (231 casos) leva
cerca de 1,5 minuto em 4 núcleos:

```bash
GUARACI_TEST_DATA=~/.local/share/guaraci cargo test --release -p guaraci-cpu stabin_spheres_full -- --ignored --nocapture
```

## Documentação

O site é gerado a partir da pasta `docs/` com MkDocs Material e publicado no
GitHub Pages a cada push na branch `main`. Para ver localmente:

```bash
pip install mkdocs-material
mkdocs serve
```

## Regras do projeto

- Nenhum dado de terceiros é distribuído com o código; as bibliotecas são
  baixadas das fontes oficiais. A exceção, por decisão do projeto, é o banco
  de materiais gerado do PNNL-15870 Rev. 1.
- Toda funcionalidade nova vem com testes e com o capítulo correspondente do manual.
- Cada mensagem de erro aponta a linha da entrada e sugere a correção
  ([filosofia](filosofia.md), item 3).
