# 8. Validação

Os testes de `crates/guaraci-cpu/tests/benchmarks.rs` rodam com
`GUARACI_TEST_DATA` apontando para uma pasta de dados com o ENDF/B-VIII.0
instalado:

```bash
GUARACI_TEST_DATA=~/.local/share/guaraci cargo test --release -p guaraci-cpu -- --nocapture
```

## Coeficientes de atenuação × NIST XCOM

μ/ρ total com espalhamento coerente (cm²/g), calculado a partir das tabelas dos
materiais (ENDF/B-VIII.0 e composições do PNNL-15870 Rev. 1):

| Material | Energia | Guaraci | XCOM | Desvio |
|---|---|---|---|---|
| Água | 10 keV | 5,3312 | 5,329 | +0,04 % |
| Água | 20 keV | 0,81073 | 0,8096 | +0,14 % |
| Água | 30 keV | 0,37609 | 0,3756 | +0,13 % |
| Água | 60 keV | 0,20583 | 0,2059 | −0,03 % |
| Água | 100 keV | 0,17064 | 0,1707 | −0,04 % |
| Água | 200 keV | 0,13698 | 0,1370 | −0,02 % |
| Água | 500 keV | 0,09681 | 0,09687 | −0,06 % |
| Água | 1 MeV | 0,07064 | 0,07072 | −0,12 % |
| Água | 2 MeV | 0,04933 | 0,04942 | −0,19 % |
| Água | 10 MeV | 0,02214 | 0,02219 | −0,25 % |
| Chumbo | 100 keV | 5,5500 | 5,549 | +0,02 % |
| Chumbo | 500 keV | 0,16119 | 0,1614 | −0,13 % |
| Chumbo | 1 MeV | 0,07076 | 0,07102 | −0,36 % |

Tolerâncias do teste: 2 % (água) e 3 % (chumbo).

## Fluência não colidida

Uma fonte pontual de 140,5 keV no centro de uma esfera de água de 10 cm, com
o espalhamento Rayleigh desligado, cercada por uma casca de vácuo de 10 a 10,5 cm.
A fluência de fótons com energia entre 140,5 e 140,6 keV na casca deve ser
e^(−μR) · ΔR / V_casca por fóton emitido.

| Guaraci | Analítico | Desvio | σ estatístico |
|---|---|---|---|
| 1,68266×10⁻⁴ cm⁻² | 1,67848×10⁻⁴ cm⁻² | +0,25 % | 0,16 % |

Esse teste exercita a geometria, as tabelas, o sorteio do caminho livre e o
estimador de comprimento de trajetória.

O bin precisa ser estreito: um bin de 140–141 keV recebe também fótons com
espalhamento Compton para a frente (θ < 9°) e superestima a fluência em ~2 %.

## Amostragem Compton × Klein-Nishina

Para S(x) = Z (elétrons livres), a distribuição de cos θ em 20 bins é comparada
com a fórmula de Klein-Nishina integrada numericamente. O teste χ² (19 graus de
liberdade) deve ficar abaixo de 43,8 (99,9 %), em 30 keV, 140,5 keV, 511 keV e
2 MeV.

## Conservação de energia e independência das threads

Numa fonte de F-18 em tecido com osso cortical:

- emitida − depositada − escapada < 10⁻¹⁰ da emitida;
- os resultados com 1 e com 4 threads são idênticos (diferença relativa < 10⁻¹²).

## Dados de decaimento

Veja o [capítulo 4](04-dados.md#testes-de-referência).

## Pendente

- Frações absorvidas e S-values em esferas, comparados com valores publicados
  (MIRD; Stabin e Konijnenberg, 2000).
- Comparação com GATE e MCNP em geometrias simples.
