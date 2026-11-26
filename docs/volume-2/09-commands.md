# 9. Commands

| Command | What it does |
|---|---|
| `guaraci check <input>` | reads, checks and describes the simulation, without running it |
| `guaraci run <input>` | runs the simulation |
| `guaraci run <input> -o folder` | writes the results in `folder` (default: `<input>_results`) |
| `guaraci run <input> --histories 1e6` | changes the number of histories without editing the file |
| `guaraci run <input> --threads 8` | number of threads (default: all cores) |
| `guaraci run <input> --offline` | never downloads data; fails if something is missing |
| `guaraci data list` | available and installed libraries |
| `guaraci data install <library>` | downloads, checks and converts a library |
| `guaraci data install <lib> --kind <photoatomic\|decay> --from <file>` | installs from a file already downloaded |
| `guaraci data path` | folder where the data is stored |
| `guaraci info units` | every unit |
| `guaraci info materials [filter]` | materials of the database |
| `guaraci info nuclide <name> [--csv]` | decay data of a nuclide |

## Environment variables

| Variable | Effect |
|---|---|
| `GUARACI_DATA` | folder of the nuclear data |
| `GUARACI_OFFLINE` | same as `--offline` |
| `NO_COLOR` | turns off colours in messages |

During a simulation, **Ctrl-C** finishes the current batch and writes the
partial results. A second Ctrl-C quits immediately.
