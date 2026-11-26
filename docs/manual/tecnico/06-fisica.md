# 6. Física de fótons

Módulo `guaraci-core/src/physics.rs`. Unidades: MeV, cm, g/cm³, barn.

## Tabelas por material

Para cada material usado na geometria, as seções de choque macroscópicas
μ = Σᵢ nᵢ σᵢ(E) (cm⁻¹) são tabeladas numa grade logarítmica de energia, com
1000 pontos por década, do corte de energia até 1,01 × a maior energia emitida.
Entre os pontos, a interpolação é linear no índice logarítmico.

- **Densidade atômica:** nᵢ = ρ wᵢ N_A / Aᵢ, com wᵢ a fração de massa e Aᵢ a
  massa atômica (do AWR do ENDF).
- **Seções de choque elementares:** são lidas das tabelas ENDF com
  interpolação linear-linear.
- **Processo desligado:** com `rayleigh = off`, por exemplo, ele simplesmente
  não entra em μ. Isso equivale a removê-lo da física.
- **Escolha do átomo-alvo:** guardam-se também as parcelas de Compton e
  Rayleigh de cada elemento. O átomo é sorteado com probabilidade
  nᵢσᵢ(E)/Σnⱼσⱼ(E).

## Caminho livre

A distância até a próxima interação é s = −ln(ξ)/μ(E). Se s é maior que a
distância até a próxima fronteira, o fóton vai até a fronteira, atravessa e
continua no novo meio. O processo não tem memória, então não é preciso
guardar o caminho restante.

## Efeito fotoelétrico

O fóton é absorvido, e toda a energia é depositada no ponto. Isso inclui a
energia de ligação, porque não há emissão de fluorescência nesta versão.

## Espalhamento Compton

A amostragem usa a fórmula de Klein-Nishina, pelo método de amostragem
composta e rejeição do `G4KleinNishinaCompton` do Geant4. Para ε = E′/E em
[ε₀, 1], ε₀ = 1/(1 + 2k), k = E/mₑc²:

1. Com probabilidade α₁/(α₁ + α₂), α₁ = −ln ε₀ e α₂ = (1 − ε₀²)/2, sorteia-se
   ε = ε₀^ξ; caso contrário, ε² é uniforme em [ε₀², 1].
2. Aceita-se com probabilidade g = 1 − ε sen²θ / (1 + ε²).

Com `compton = klein_nishina_sf` (padrão), há uma segunda rejeição pela função
de espalhamento incoerente: aceita-se com probabilidade S(x, Z)/Z, onde
x = (E/hc) sen(θ/2). Isso reduz o espalhamento em ângulos pequenos, onde a
ligação do elétron importa. A energia E − E′ vai para o elétron e é depositada
no ponto. O alargamento Doppler não é simulado.

## Espalhamento Rayleigh

A amostragem segue a forma dσ/dΩ ∝ (1 + cos²θ)/2 · F²(x, Z):

1. Sorteia-se x² da distribuição F²(x²) entre 0 e x²_max = (E/hc)², invertendo
   a integral cumulativa ∫F² d(x²), que é pré-calculada por regra do trapézio
   para cada elemento.
2. Calcula-se cos θ = 1 − 2 x²/x²_max.
3. Aceita-se com probabilidade (1 + cos²θ)/2.

O fóton mantém a energia.

## Produção de pares

A energia cinética do par, E − 2mₑc², é depositada no ponto. Dois fótons de
mₑc² saem em direções opostas, isotrópicas.

## Corte de energia

Um fóton abaixo de `photon_cut` (padrão 1 keV) deposita a energia no ponto.

## Emissão das fontes radioativas

Para cada decaimento:

- **Linhas de fótons:** cada linha de intensidade I (fótons por decaimento)
  emite ⌊I⌋ fótons, mais um com probabilidade I − ⌊I⌋. As linhas são
  independentes entre si; as correlações γ-γ não são reproduzidas.
- **Aniquilação:** os fótons de 511 keV saem em pares colineares, a partir do
  ponto de decaimento (sem o alcance do pósitron).
- **Elétrons:** com `beta = local`, deposita-se no ponto de decaimento a energia
  média por decaimento carregada por partículas leves, ⟨E_LP⟩ do ENDF
  (β⁻, β⁺, Auger e conversão).
- **Posição:**
  - para uma fonte com `in = volume`, a posição é uniforme na região do volume
    (o sólido menos os filhos), sorteada por rejeição na caixa envolvente;
  - a direção de cada fóton é isotrópica.

## Constantes

| Constante | Valor | Fonte |
|---|---|---|
| mₑc² | 0,51099895000 MeV | CODATA 2018 |
| hc | 1,239841984×10⁻¹⁰ MeV·cm | CODATA 2018 |
| N_A | 6,02214076×10²³ mol⁻¹ | SI 2019 (exata) |
| 1 MeV | 1,602176634×10⁻¹³ J | SI 2019 (exata) |
| mₙ | 1,00866491595 u | CODATA 2018 |
