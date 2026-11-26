<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/_static/brand/guaraci-assinatura-sobre-escuro.svg">
    <img src="docs/_static/brand/guaraci-assinatura.svg" width="380" alt="guaraci">
  </picture>
</p>

<p align="center">
  <a href="README.md">English</a> · <a href="README.pt-BR.md">Português</a> · <b>Español</b>
</p>

<p align="center">
  <b>Transporte de radiación por Monte Carlo en CPU y GPU</b><br>
  Enfocado inicialmente en física médica y medicina nuclear.
</p>

<p align="center">
  <a href="https://guaraci.readthedocs.io/es/latest/"><b>Documentación</b></a> ·
  <a href="docs/volume-2/03-first-simulation.md">Primera simulación</a> ·
  <a href="docs/volume-1/08-validation.md">Validación</a> ·
  <a href="examples/">Ejemplos</a>
</p>

<p align="center">
  <a href="https://github.com/HarlleyHauradou/guaraci/actions/workflows/ci.yml"><img src="https://github.com/HarlleyHauradou/guaraci/actions/workflows/ci.yml/badge.svg" alt="ci"></a>
  <a href="https://guaraci.readthedocs.io/es/latest/"><img src="https://readthedocs.org/projects/guaraci/badge/?version=latest" alt="docs"></a>
</p>

---

*Guaraci es el Sol en la mitología tupí, la fuente de la radiación que da vida a todo.*

```ini
[volume thyroid]
parent = neck
shape = ellipsoid
semi_axes = [1, 1.5, 2.5] cm
material = Tissue_Soft_ICRP

[source]
nuclide = I-131
in = thyroid
activity = 3.7 GBq

[probe dose]
type = dose_grid
in = neck
voxel = [2, 2, 2] mm
```

```bash
guaraci check thyroid    # lo que Guaraci entendió, sin ejecutar
guaraci run thyroid      # mapa de dosis, incertidumbres y un informe legible
```

## Características

- **Transporte de fotones** en CPU multinúcleo: efecto fotoeléctrico, Compton (Klein-Nishina con S(x,Z)), Rayleigh (factores de forma), producción de pares.
- **Fuentes radiactivas** a partir de los datos de decaimiento de ENDF/B-VIII.0 (3821 nucleidos), descargados de la fuente oficial en el primer uso; no se redistribuye ningún dato de terceros.
- **Entrada legible** con unidades (`15 mm`, `3.7 GBq`), inclusión de archivos y mensajes de error que señalan la línea y sugieren la corrección.
- **372 materiales** del PNNL-15870 Rev. 1.
- **Probes:** mapas de dosis 3D (MetaImage), dosis en volúmenes y espectros de fluencia, todos con incertidumbre estadística.
- **Validación:** μ/ρ dentro del 0,4 % de NIST XCOM; fracciones absorbidas en 231 esferas comparadas con Stabin & Konijnenberg (2000); conservación exacta de la energía; resultados independientes del número de hilos.

En desarrollo: fantomas vóxel (CT/DICOM), transporte de electrones, backend CUDA e interfaz gráfica.

## Instalación

```bash
git clone https://github.com/HarlleyHauradou/guaraci
cd guaraci
cargo install --path crates/guaraci-cli
guaraci run examples/thyroid_i131
```

## Documentación

El manual está en línea, en inglés, portugués y español, en **<https://guaraci.readthedocs.io/es/latest/>**, en tres volúmenes:

| Volumen | Contenido |
|---|---|
| I — Manual Técnico | física, datos nucleares, motor de transporte, estadística, validación |
| II — Manual del Usuario | el lenguaje `.gci`, materiales, fuentes, probes, comandos, resultados |
| III — Interfaz Gráfica | el programa de escritorio (en desarrollo) |

El programa, sus mensajes y sus salidas están en inglés, el idioma de la comunidad científica internacional.

## Repositorio

| Carpeta | Contenido |
|---|---|
| `crates/` | los crates de Rust (`guaraci-units`, `-input`, `-data`, `-core`, `-cpu`, `-cli`) |
| `data/` | base de datos de materiales |
| `docs/` | fuentes de la documentación (Sphinx + tema PyData, Markdown con MyST), publicada en Read the Docs |
| `examples/` | archivos de entrada |
| `validation/` | benchmarks y comparaciones con GATE y MCNPX |
| `tools/` | generadores |

Vea también la [filosofía](PHILOSOPHY.md) del proyecto.
