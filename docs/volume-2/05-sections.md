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
| `list` | `photon_local` | `photon_local` | ready-made physics list |
| `data` | `endfb8`, `epics2017`, `epdl97` | `endfb8` | photon interaction library |
| `photoelectric` | on/off | on | photoelectric effect |
| `compton` | `klein_nishina_sf`, `klein_nishina`, `off` | `klein_nishina_sf` | Compton model |
| `rayleigh` | on/off | on | coherent scattering |
| `pair` | on/off | on | pair production |
| `photon_cut` | energy | 1 keV | below this, a photon deposits its energy on the spot |
| `particles` | list | `[photon]` | transported particles (photons only in this version) |

The list sets the defaults, and any key written replaces them (like the MODE
card of MCNP). `check` shows the physics actually used.

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
