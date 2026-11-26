# Exemplos

Os arquivos estão na pasta
[`examples/`](https://github.com/HarlleyHauradou/guaraci/tree/main/examples) do repositório.

## I-131 na tireoide

[`examples/thyroid_i131.gci`](https://github.com/HarlleyHauradou/guaraci/blob/main/examples/thyroid_i131.gci):
uma tireoide elipsoidal dentro de um pescoço cilíndrico de tecido mole, com 3,7 GBq de I-131.

```bash
guaraci run examples/thyroid_i131
```

Resultado com 10⁷ decaimentos: taxa de dose média na tireoide de
7,756×10⁻³ Gy/s. É o que dá a conta manual: 3,7 GBq × 0,2055 MeV por
decaimento ÷ 15,7 g.

## Lesão de Tc-99m em simulador de água

[`examples/tc99m_lesion.gci`](https://github.com/HarlleyHauradou/guaraci/blob/main/examples/tc99m_lesion.gci):
uma esfera de 15 mm de raio num cilindro de água, com resultados por decaimento.
Inclui um espectro de fluência, em que aparecem o pico de 140,5 keV e o
contínuo do espalhamento Compton.

## Frasco com I-131: Guaraci × GATE 10 × MCNPX

[`validation/vial_i131/`](https://github.com/HarlleyHauradou/guaraci/tree/main/validation/vial_i131):
um frasco de 10 mL de água com I-131, só transporte de fótons. Um gerador
escreve três arquivos de entrada equivalentes (Guaraci, GATE 10 e MCNPX), e
um script compara os resultados.

```bash
cd validation/vial_i131
python3 make_inputs.py
guaraci run inputs/vial -o vial_results
python3 compare.py
```
