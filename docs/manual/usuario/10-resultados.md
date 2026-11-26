# 10. Resultados

`guaraci run tireoide` cria a pasta `tireoide_results/` com:

| Arquivo | Conteúdo |
|---|---|
| `report.txt` | relatório completo da simulação |
| `input/` | cópia de todos os arquivos de entrada usados |
| `<probe>.mhd` + `.raw` | mapa de dose (`dose_grid`) |
| `<probe>_uncertainty.mhd` + `.raw` | incerteza relativa (1σ) de cada voxel |
| `<probe>.csv` | `volume_dose` e `spectrum` |

## Unidade dos resultados

A unidade depende do que a entrada diz sobre a atividade:

| Fontes | Resultado | Unidade da dose |
|---|---|---|
| sem `activity` | por decaimento | Gy por decaimento |
| com `activity` | taxa | Gy/s |
| com `activity` e `duration` | total | Gy |

Com várias fontes, cada decaimento simulado sai de uma fonte escolhida na
proporção da sua atividade (ou da atividade × duração). Ou todas as fontes
têm `activity`, ou nenhuma tem.

## Incerteza

As histórias são divididas em `batches` lotes (padrão 10). A incerteza é o
desvio padrão da média entre os lotes (1σ).

- **Valores acima de ~5 % pedem mais histórias.** A incerteza cai com 1/√N:
  4× mais histórias deixam a incerteza 2× menor.
- **No mapa de dose, o resumo mostra o máximo só entre os voxels com incerteza
  ≤ 10 %.** O máximo bruto quase sempre é um voxel com pouquíssima massa e um
  único depósito.

## O relatório

O `report.txt` foi feito para ser lido como um caderno de laboratório:

- a simulação inteira como o Guaraci a entendeu (a mesma saída do `check`);
- a origem dos dados, com SHA-256 e data de instalação;
- a normalização usada;
- o **balanço de energia**: emitida = depositada + escapada; a diferença deve
  ser da ordem de 10⁻¹⁰ % ou menor;
- os resultados, o desempenho (histórias/s) e as limitações da versão.

## Abrir os mapas `.mhd`

- **3D Slicer:** *File → Add Data*. Escolha o `.mhd`.
- **ImageJ/Fiji:** *File → Import → MHD/MHA* (ou o plugin *MetaImage*).
- **Amide:** *File → Import File (generic)*.
- **Python:**

```python
import SimpleITK as sitk
dose = sitk.GetArrayFromImage(sitk.ReadImage("tireoide_results/dose.mhd"))  # [z, y, x]
```

As distâncias no `.mhd` estão em **mm**, como esperam esses programas, e o
`Offset` é o centro do primeiro voxel.
