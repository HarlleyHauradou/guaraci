# 4. Dados atômicos e nucleares

Crate `guaraci-data`.

## Política

Os arquivos de dados não são redistribuídos. Cada biblioteca do catálogo
(`library.rs`) registra:

- a URL oficial e a página de download;
- o SHA-256 esperado e o tamanho.

O processo de instalação:

1. baixa (via `HTTPS_PROXY`, se definido) ou usa o arquivo dado em `--from`;
2. confere o SHA-256 (só para downloads automáticos; um `--from` diferente do
   catálogo gera um aviso, não um erro);
3. converte os arquivos ENDF-6 para estruturas Rust serializadas (bincode);
4. grava `<GUARACI_DATA>/<biblioteca>/<tipo>.bin`, com um cabeçalho de
   procedência (fonte, SHA-256, data, versão do formato, número de entradas);
5. apaga o arquivo baixado.

| Biblioteca | Arquivo | SHA-256 |
|---|---|---|
| ENDF/B-VIII.0 photoat | `ENDF-B-VIII.0_photoat.zip` (35 776 679 B) | `61aee2b8…0248e835d`ᵃ |
| ENDF/B-VIII.0 decay | `ENDF-B-VIII.0_decay.zip` (14 173 710 B) | `9087395f…f8d03d5e`ᵃ |

ᵃ Os valores completos estão em `crates/guaraci-data/src/library.rs`.

## Leitor ENDF-6

`endf.rs` implementa os registros CONT/HEAD, LIST e TAB1 do formato de 80
colunas:

- Os números reais aceitam a forma compacta sem `E` (`1.234567+5`) e campos em branco (= 0).
- O arquivo é dividido em seções `(MF, MT)` pelas colunas 71–75.

## Dados fotoatômicos (MF23, MF27)

| Seção | Grandeza | Unidade no Guaraci |
|---|---|---|
| MF23 MT502 | seção de choque coerente (Rayleigh) | barn × MeV |
| MF23 MT504 | seção de choque incoerente (Compton) | barn × MeV |
| MF23 MT516 | produção de pares (núcleo + elétron) | barn × MeV |
| MF23 MT522 | fotoelétrico, todas as camadas | barn × MeV |
| MF27 MT502 | fator de forma F(x, Z) | x = sen(θ/2)/λ em cm⁻¹ |
| MF27 MT504 | função de espalhamento incoerente S(x, Z) | idem |

- Energias são convertidas de eV para MeV.
- A massa atômica vem do AWR do arquivo: m = AWR × mₙ, com
  mₙ = 1,00866491595 u (CODATA 2018). Assim nenhuma tabela de massas
  atômicas é embutida.
- Todas as tabelas do ENDF/B-VIII.0 usam interpolação linear-linear (INT = 2).
  O conversor recusa outras leis em vez de interpolá-las errado.

## Dados de decaimento (MF8 MT457)

De cada nuclídeo são guardados:

- a meia-vida e os modos de decaimento (RTYP, razão de ramificação);
- as energias médias por decaimento: partículas leves ⟨E_LP⟩ (β⁻, β⁺,
  elétrons Auger e de conversão), eletromagnética ⟨E_EM⟩ e partículas pesadas ⟨E_HP⟩;
- as linhas discretas de fótons, com intensidade absoluta = FD × RI:
  - STYP 0 vira **gama**;
  - STYP 9 vira **raio X**, ou **aniquilação** quando a energia está a 50 eV de 510,999 keV.

Espectros contínuos de fótons (raros) são marcados, mas ainda não usados.

## Testes de referência

Com `GUARACI_TEST_ENDF` apontando para os arquivos descompactados, os testes conferem:

| Caso | Valor esperado |
|---|---|
| μ/ρ da água a 140 keV | 0,1538 cm²/g (NIST XCOM), tolerância de 1% |
| I-131 | T½ = 8,025 d; 364,49 keV com 81,5% |
| Tc-99m | T½ = 6,007 h; 140,51 keV com 88,5% |
| F-18 | 1,934 fóton de aniquilação por decaimento |
| C-14 | nenhum fóton; ⟨E_LP⟩ = 49,5 keV |
