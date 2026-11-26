# 2. Instalação e dados nucleares

## Compilar

É preciso o [Rust](https://www.rust-lang.org/tools/install) (1.80 ou mais novo).

```bash
git clone https://github.com/HarlleyHauradou/guaraci
cd guaraci
cargo install --path crates/guaraci-cli
```

O comando `guaraci` fica disponível no terminal.

## Dados nucleares

O Guaraci **não distribui** bibliotecas de dados de terceiros. Elas são baixadas
das fontes oficiais na sua máquina, conferidas por SHA-256 e convertidas para
um formato interno.

Na prática você não precisa fazer nada: na primeira simulação que precisar de
uma biblioteca ainda não instalada, o Guaraci avisa e baixa. Para baixar antes
(por exemplo, numa máquina que depois ficará sem internet):

```bash
guaraci data install endfb8
```

| Biblioteca | Conteúdo | Fonte | Download |
|---|---|---|---|
| `endfb8` (padrão) | interações de fótons e decaimento radioativo | NNDC/BNL, ENDF/B-VIII.0 | automático (36 MB + 14 MB) |
| `epics2017` | interações de fótons | IAEA | manual |
| `epdl97` | interações de fótons | IAEA | manual |

O site da IAEA bloqueia downloads automáticos. Para EPICS2017 e EPDL97, baixe o
arquivo pelo navegador e instale com:

```bash
guaraci data install epics2017 --kind photoatomic --from caminho/do/arquivo.zip
```

`--from` aceita um arquivo `.zip` ou uma pasta com os arquivos ENDF.

### Onde os dados ficam

```bash
guaraci data path
```

Por padrão em `~/.local/share/guaraci` (Linux). Para usar outro lugar, como
um disco compartilhado de um cluster, defina a variável `GUARACI_DATA`.

### Ver o que está instalado

```bash
guaraci data list
```
