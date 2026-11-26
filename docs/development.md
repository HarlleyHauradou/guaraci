# Development

## Code organisation

| Folder | Contents |
|---|---|
| `crates/guaraci-units` | physical quantities and units |
| `crates/guaraci-input` | the `.gci` language |
| `crates/guaraci-data` | ENDF-6 reader, download and cache of the data |
| `crates/guaraci-core` | physics, geometry, sources and probes |
| `crates/guaraci-cpu` | parallel CPU engine |
| `crates/guaraci-cli` | the `guaraci` program |
| `data/` | material database (`materials.gci`) |
| `docs/` | this website (Sphinx, PyData Sphinx Theme, MyST Markdown) |
| `examples/` | example input files |
| `tools/` | generators (material database) |
| `validation/` | benchmarks and comparisons with other codes |

The details of each part are in [Volume I](volume-1/01-architecture.md).

## Building and testing

```bash
cargo build --release
cargo test --workspace
```

Tests that need the real nuclear data are skipped unless this variable is set:

```bash
guaraci data install endfb8
GUARACI_TEST_DATA=~/.local/share/guaraci cargo test --release --workspace
```

The complete sphere benchmark of Stabin & Konijnenberg (231 cases) takes about
1.5 minutes on 4 cores:

```bash
GUARACI_TEST_DATA=~/.local/share/guaraci cargo test --release -p guaraci-cpu stabin_spheres_full -- --ignored --nocapture
```

## Documentation

The website is built from the `docs/` folder with Sphinx (PyData Sphinx
Theme; pages in Markdown with MyST) and published on GitHub Pages, in the
three languages, at every push to `main` (workflow `.github/workflows/docs.yml`).
To view it locally:

```bash
pip install -r docs/requirements.txt
sphinx-build -W docs docs/_build/html
```

Then open `docs/_build/html/index.html` in a browser.

English is the source language. The Portuguese and Spanish versions are
translations kept as gettext files in `docs/locale/` (see [Translations](#translations)).
To build another language, add `-D language=pt_BR` or `-D language=es`.

## Translations

```bash
pip install sphinx-intl
cd docs
sphinx-build -b gettext . _build/gettext          # extract the English text
sphinx-intl update -p _build/gettext -l pt_BR -l es   # create or update the .po files
# translate the msgstr entries in locale/<lang>/LC_MESSAGES/*.po
sphinx-build -D language=pt_BR . _build/html-pt_BR    # build one language
```

When the English text changes, `sphinx-intl update` marks the affected entries
as *fuzzy*, so it is always clear what must be revised.

## Project rules

- No third-party data is redistributed with the code; the libraries are
  downloaded from the official sources. The exception, by project decision,
  is the material database generated from PNNL-15870 Rev. 1.
- Code, messages, outputs, documentation and commits are written in English.
- Every new feature comes with tests and with the corresponding chapter of the manual.
- Every error message points at the line of the input and suggests the fix
  ([philosophy](philosophy.md), item 3).
