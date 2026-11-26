# 2. Installation and nuclear data

## Building

You need [Rust](https://www.rust-lang.org/tools/install) (1.80 or newer).

```bash
git clone https://github.com/HarlleyHauradou/guaraci
cd guaraci
cargo install --path crates/guaraci-cli
```

The `guaraci` command is then available in the terminal.

## Nuclear data

Guaraci **does not redistribute** third-party data libraries. They are
downloaded from the official sources on your machine, checked by SHA-256 and
converted to an internal format.

In practice there is nothing to do: the first simulation that needs a library
that is not installed yet says so and downloads it. To download in advance
(for example, on a machine that will later be offline):

```bash
guaraci data install endfb8
```

| Library | Contents | Source | Download |
|---|---|---|---|
| `endfb8` (default) | photon interactions and radioactive decay | NNDC/BNL, ENDF/B-VIII.0 | automatic (36 MB + 14 MB) |
| `epics2017` | photon interactions | IAEA | manual |
| `epdl97` | photon interactions | IAEA | manual |

The IAEA website blocks automatic downloads. For EPICS2017 and EPDL97,
download the file with a web browser and install it with:

```bash
guaraci data install epics2017 --kind photoatomic --from path/to/file.zip
```

`--from` accepts a `.zip` file or a folder with the ENDF files.

### Where the data is stored

```bash
guaraci data path
```

By default in `~/.local/share/guaraci` (Linux). To use another place, such as a
shared disk of a cluster, set the `GUARACI_DATA` variable.

### What is installed

```bash
guaraci data list
```
