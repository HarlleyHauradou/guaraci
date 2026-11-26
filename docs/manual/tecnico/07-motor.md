# 7. Motor de transporte e estatística

## Geometria

Módulo `geometry.rs`. Os volumes formam uma árvore cuja raiz é o mundo.

- **Localização:** a partir do mundo, desce-se ao primeiro filho que contém o
  ponto, e assim por diante. O ponto pertence ao volume mais profundo.
- **Distância à fronteira:** na região do volume V, é o menor valor entre a
  distância de saída de V e a distância de entrada em cada filho de V. Os
  sólidos (caixa, esfera, cilindro, elipsoide) calculam essas distâncias pelo
  intervalo paramétrico da reta dentro deles.
- **Travessia:** o fóton avança até a fronteira mais um pequeno empurrão
  (10⁻⁹ × o tamanho do mundo) e é relocalizado.
- **Verificação de sobreposição:** antes de rodar, 5000 pontos de cada
  superfície são testados. Um ponto fora do pai significa que o volume vaza do
  pai; um ponto dentro de um irmão significa sobreposição. A ideia é a mesma do
  `CheckOverlaps` do Geant4.

## Geradores de números aleatórios

Módulo `rng.rs`. Todos produzem números uniformes em (0, 1) com 53 bits.

| `rng` | Algoritmo | Verificação |
|---|---|---|
| `philox` | Philox4x32-10 (Salmon et al., 2011), baseado em contador | vetores de teste do Random123 |
| `xoshiro` | Xoshiro256++ (Blackman e Vigna) | saída conhecida para o estado [1, 2, 3, 4] |
| `pcg` | PCG64 XSL-RR 128/64 (O'Neill) | momentos estatísticos |
| `mt19937` | Mersenne Twister 32 bits | 10000ª saída da semente 5489 (padrão C++) |

## Paralelismo e reprodutibilidade

As histórias são divididas em blocos fixos de 1000. O bloco b sempre usa o
fluxo b do gerador: no Philox, o contador (b, i) com a chave igual à semente;
nos demais, o estado é derivado de (semente, b) por SplitMix64. Por isso
**o resultado não depende do número de threads**, e um teste verifica isso.
Os blocos são distribuídos entre as threads com rayon. Cada thread acumula num
`Tally` próprio, e os `Tally` são somados no fim de cada lote.

## Incerteza

Os blocos são agrupados em B lotes (`batches`). Para cada célula de cada
probe, guarda-se a soma total e a soma dos quadrados das médias por história
de cada lote. A incerteza da média é:

  σ²(x̄) = [ (1/B) Σ x̄_b² − x̄² ] · B/(B − 1) / B

## Pontuação

- **Energia depositada:** vai para a célula da grade em que caiu e para os
  probes de volume da região atual.
- **Comprimento de trajetória:** cada trecho percorrido numa região é somado
  no bin de energia dos probes `spectrum` daquela região.
- **Balanço:** o `Tally` também acumula as energias emitida, depositada e
  escapada do mundo, para conferir a conservação de energia.

## Dose

D = E_dep / m, com E em J (1 MeV = 1,602176634×10⁻¹³ J) e m em kg.

- **Volumes:** m = ρ × (volume do sólido − volumes dos filhos), calculado
  analiticamente.
- **Voxels:** m = ⟨ρ⟩ × volume do voxel, com ⟨ρ⟩ a média em 3×3×3 pontos internos.
