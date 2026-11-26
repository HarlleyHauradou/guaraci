# 1. Architecture

Guaraci is a Cargo *workspace* with one crate per responsibility:

| Crate | Role | Status |
|---|---|---|
| `guaraci-units` | physical quantities with dimensions; unit table | done |
| `guaraci-input` | the `.gci` language: parser, `include`, diagnostics, typed configuration | done |
| `guaraci-data` | ENDF-6 reader, download, conversion and cache of the data | done |
| `guaraci-cli` | the `guaraci` program | done |
| `guaraci-core` | physics, geometry, sources, probes (no I/O) | done (photons) |
| `guaraci-cpu` | parallel CPU engine | done |
| `guaraci-cuda` | GPU engine (CUDA) | planned |
| `guaraci-gui` | graphical interface | planned |

## Flow

```text
.gci files ──parser──▶ AST with positions ──reader──▶ Config (internal units)
                                                             │
ENDF data (cache) ───────────────────────────────────────────▶ Problem ──▶ Backend (CPU/GPU) ──▶ probes
```

- The `Config` uses fixed units: cm, MeV, g/cm³, s, Bq, rad.
- The `Problem` is a flat structure, without pointers, that can be copied as
  is to the GPU. Both backends implement the same trait.

## Conventions

- No third-party data is embedded in the code, except the material database
  generated from PNNL-15870 Rev. 1 (a project decision) and the element symbols.
- Every message to the user that refers to the input carries the position in
  the text (`Span`).
