# 5. Section reference

Keys marked with * are required. The default value applies when the key is omitted.

## `[run]` — the run

| Key | Type | Default | Description |
|---|---|---|---|
| `histories` | whole number | 1000000 | histories (simulated decays) |
| `seed` | whole number | 1 | seed of the random number generator |
| `rng` | `philox`, `xoshiro`, `pcg`, `mt19937` | `philox` | random number generator |
| `backend` | `cpu`, `cuda` | `cpu` | where to compute |
| `threads` | whole number | 0 (all cores) | CPU threads |
| `batches` | whole number | 10 | batches used to estimate the uncertainty |

## `[physics]` — physics

| Key | Type | Default | Description |
|---|---|---|---|
| `list` | `photon_electron`, `photon_local` | `photon_electron` | ready-made physics list (see below) |
| `data` | `endfb8`, `epics2017`, `epdl97` | `endfb8` | photon interaction library |
| `photoelectric` | on/off | on | photoelectric effect |
| `compton` | `klein_nishina_sf`, `klein_nishina`, `off` | `klein_nishina_sf` | Compton model |
| `rayleigh` | on/off | on | coherent scattering |
| `pair` | on/off | on | pair production |
| `photon_cut` | energy | 1 keV | below this, a photon deposits its energy on the spot; bremsstrahlung photons are produced above it (at least 1 keV) |
| `electron_cut` | energy | 10 keV | below this, an electron or positron deposits its energy on the spot; knock-on electrons (δ-rays) are produced above it. Minimum 1 keV |
| `particles` | list | from the list | transported particles: `photon`, `electron`, `positron` (like the MODE card of MCNP) |

| List | What it does |
|---|---|
| `photon_electron` | photons, electrons and positrons are transported. Use it whenever the electron range matters at the scale of your voxels or volumes: β emitters, small volumes, interfaces |
| `photon_local` | only photons are transported. Electrons deposit their energy where they are created. Faster; good when the voxels are much larger than the electron range |

The list sets the defaults, and any key written replaces them. `check` shows
the physics actually used.

## `[world]` — the world

| Key | Type | Description |
|---|---|---|
| `material`* | name | material filling the world |
| `size`* | length or list of 3 | sides of the box (centred at the origin) |

## `[volume <name>]` — volumes

| Key | Type | Description |
|---|---|---|
| `shape`* | `box`, `sphere`, `cylinder`, `ellipsoid` | shape |
| `material`* | name | material |
| `parent` | name | the volume that contains this one (default: the world) |
| `center` | list of 3 lengths | centre relative to the centre of the parent (default `[0, 0, 0]`) |

Dimensions of each shape:

| Shape | Keys |
|---|---|
| `box` | `size` (sides) |
| `sphere` | `radius` |
| `cylinder` | `radius`, `height` (axis along z) |
| `ellipsoid` | `semi_axes` (semi-axes along x, y, z) |

## `[material <name>]`

See [chapter 6](06-materials.md).

## `[source]` or `[source <name>]`

See [chapter 7](07-sources.md).

## `[probe <name>]`

See [chapter 8](08-probes.md).
