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

Veja o [capítulo 4](04-dados.md#testes-de-referencia).

## Frações absorvidas em esferas (Stabin e Konijnenberg, 2000)

**Referência:** M.G. Stabin e M.W. Konijnenberg, *Re-evaluation of absorbed
fractions for photons and electrons in spheres of various sizes*, J Nucl Med
2000;41:149–160, Tabelas 2 e 3. São esferas de densidade 1 com fonte de fótons
monoenergética uniforme, dentro de um meio infinito de densidade 1. Os valores
recomendados são a média de EGS4 e MCNP4B.

Os 231 valores (21 massas, de 1 a 6000 g; 12 energias, de 20 keV a 2,75 MeV)
estão em `validation/stabin2000_photons.csv`. As linhas embaralhadas na
extração do texto do PDF foram transcritas das imagens das páginas, e cada
linha foi conferida de duas formas: o raio corresponde a densidade 1, e o valor
recomendado é a média EGS4/MCNP.

**Modelo no Guaraci:**

- a esfera e o meio são de `Tissue_Soft_ICRP` (densidade 1);
- o meio infinito é um cubo de 1 m;
- fração absorvida = energia depositada na esfera ÷ energia emitida;
- 5×10⁵ histórias por caso; a incerteza estatística é tipicamente < 1 %.

O resultado caso a caso está em `validation/stabin2000_comparison.csv`. Para
reproduzir:

```bash
GUARACI_TEST_DATA=~/.local/share/guaraci cargo test --release -p guaraci-cpu stabin_spheres_full -- --ignored --nocapture
```

Desvio em relação ao valor recomendado (seleção de massas):

| Energia | 1 g | 10 g | 100 g | 1000 g | 6000 g |
|---|---|---|---|---|---|
| 20 keV | +7.6 % | +5.0 % | +3.2 % | +1.4 % | +0.6 % |
| 30 keV | +8.8 % | +6.7 % | +4.9 % | +2.6 % | +1.0 % |
| 40 keV | +6.0 % | +6.5 % | +6.8 % | +2.5 % | +0.8 % |
| 60 keV | +3.5 % | +3.6 % | +6.1 % | +1.7 % | -0.1 % |
| 80 keV | +4.0 % | +2.7 % | +4.3 % | +0.6 % | -0.0 % |
| 100 keV | +1.9 % | +2.5 % | +3.9 % | +0.6 % | +0.3 % |
| 140 keV | -2.0 % | +1.5 % | +3.4 % | — | — |
| 160 keV | — | — | — | +0.4 % | -0.3 % |
| 364 keV | +1.0 % | +2.2 % | +3.1 % | +0.7 % | -0.1 % |
| 662 keV | +8.2 % | +4.2 % | +3.4 % | +1.1 % | +0.0 % |
| 1460 keV | +48.6 % | +17.4 % | +7.7 % | +3.8 % | +1.8 % |
| 2750 keV | +171.4 % | +46.9 % | +20.9 % | +7.6 % | +4.0 % |

### Leitura dos resultados

1. **De 80 a 364 keV, e acima de 300 g em qualquer energia até 662 keV:** a
   concordância é de 0 a 4 %. A diferença entre os dois códigos de referência
   chega a cerca de 6 % (o próprio artigo aponta isso, na Tabela 3).
2. **Acima de 1 MeV, em esferas pequenas:** o Guaraci superestima, de +8 % em
   100 g a +171 % em 1 g a 2,75 MeV. O desvio diminui com o tamanho da esfera,
   como esperado. É o efeito da limitação desta versão: os elétrons Compton
   (até ~2,5 MeV, com alcance de ~1 cm no tecido) são depositados no local,
   enquanto nos códigos de referência eles são transportados e escapam de
   esferas pequenas. O mesmo efeito, menor, aparece a 662 keV abaixo de 10 g.
   **Este benchmark será refeito com o transporte de elétrons.**
3. **De 20 a 60 keV:** há um desvio sistemático de +3 a +8 % em esferas
   pequenas. Nessa faixa domina o efeito fotoelétrico, muito sensível à
   composição. O artigo usa o tecido dos MIRD Pamphlets 3 e 8, que não aparece
   no texto. Num caso de 10 g a 30 keV:

| Material | Fração absorvida | Desvio |
|---|---|---|
| Tissue_Soft_ICRP | 0,1523 | +6,5 % |
| Tissue_Soft_ICRUFourComponent | 0,1507 | +5,4 % |
| Water | 0,1612 | +12,7 % |
| EGS4 (artigo) | 0,149 | +4,2 % |
| MCNP (artigo) | 0,137 | −4,2 % |

   A mudança de material desloca o resultado mais do que o desvio observado,
   e o Guaraci fica entre os dois códigos de referência, mais perto do EGS4.

O teste `stabin_spheres_selected`, rodado em todo `cargo test`, usa cinco casos
da região 1 com tolerância de 6 % + 3σ.

## Pendente

- Repetir o benchmark de esferas com o transporte de elétrons (fótons e elétrons).
- Comparação com GATE e MCNP em geometrias simples.
