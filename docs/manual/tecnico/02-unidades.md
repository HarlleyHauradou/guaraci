# 2. Unidades e análise dimensional

Crate `guaraci-units`.

## Dimensões

Cada valor é guardado em unidades SI junto com sua dimensão, um vetor de
expoentes inteiros `[L, M, T, A]`: comprimento, massa, tempo e ângulo plano.

| Grandeza | Dimensão |
|---|---|
| energia | M L² T⁻² |
| atividade | T⁻¹ |
| dose absorvida | L² T⁻² (J/kg) |
| densidade | M L⁻³ |

O ângulo é uma dimensão própria (no SI ele é adimensional), para que
`radius = 30 deg` seja recusado.

## Tabela de unidades

A tabela (`table.rs`) lista **apenas** unidades usuais em transporte de
radiação. Um símbolo incomum (como megâmetro) é mais provavelmente um erro de
digitação do que uma intenção, então não é aceito. Fatores exatos:
1 eV = 1,602176634×10⁻¹⁹ J (SI 2019) e 1 Ci = 3,7×10¹⁰ Bq; o ano é juliano (365,25 d).

## Resolução de um símbolo

1. A grafia exata vem primeiro.
2. Sem grafia exata, procuram-se os símbolos iguais ignorando maiúsculas e
   minúsculas. O símbolo é aceito se houver um só candidato, ou se todos os
   candidatos tiverem o mesmo fator e a mesma dimensão (`l`/`L`).
3. Se houver candidatos com significados diferentes, é um erro, e a mensagem
   lista as opções.

Um teste garante que a tabela atual não tem nenhum par de símbolos que difira
só pela caixa e tenha significado diferente.

## Unidades compostas

Uma expressão é um produto de átomos separados por `*`, `.` ou `·`, ou um
quociente separado por `/`. Cada átomo pode ter expoente inteiro (`cm3`,
`cm^3`, `m^-3`). Cada `/` inverte só o átomo seguinte, então `a/b/c` = a·b⁻¹·c⁻¹.

## Valores sem unidade

Cada chave da entrada declara a grandeza que espera (`Kind`). Um número sem
unidade recebe a unidade padrão dessa grandeza, e cada ocorrência é registrada
(`implicit_units`) para aparecer no `check` e no relatório.
