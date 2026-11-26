# 1. Introdução

O Guaraci é um código de Monte Carlo para transporte de radiação na matéria,
voltado inicialmente à física médica e à medicina nuclear. Ele roda pelo
terminal a partir de arquivos de entrada (`.gci`), como o MCNP e o GATE, e
pode calcular em CPU ou GPU.

## O que esta versão faz

- Transporta fótons em CPU, em paralelo: efeito fotoelétrico, Compton,
  Rayleigh e produção de pares (`guaraci run`).
- Calcula dose em grade 3D, dose em volumes e espectro de fluência, cada um
  com sua incerteza estatística.
- Lê e valida arquivos de entrada, com mensagens de erro que apontam a linha
  e sugerem a correção (`guaraci check`).
- Baixa, verifica e converte as bibliotecas de dados (ENDF/B-VIII.0).
- Traz um banco de 372 materiais (PNNL-15870 Rev. 1).
- Conhece os dados de decaimento de 3821 nuclídeos.

Os elétrons ainda não são transportados: depositam a energia onde são criados.
O backend em GPU está **(em desenvolvimento)**.

## Princípios

O Guaraci segue a [filosofia do projeto](../filosofia.md). Duas regras
práticas decorrem dela:

- **Tudo que você não escreve tem um valor padrão conhecido**, e o
  `guaraci check` lista cada um deles.
- **Nada é adivinhado.** Um nome ou unidade ambíguo é um erro, com sugestão.
