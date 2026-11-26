# 7. Transport engine and statistics

## Geometry

Module `geometry.rs`. The volumes form a tree whose root is the world.

- **Location:** starting from the world, the search goes down to the first
  daughter that contains the point, and so on. The point belongs to the deepest volume.
- **Distance to the boundary:** in the region of volume V, it is the smallest of
  the exit distance from V and the entry distance into each daughter of V. The
  solids (box, sphere, cylinder, ellipsoid) compute these distances from the
  parametric interval of the ray inside them.
- **Crossing:** the photon moves to the boundary plus a small push
  (10⁻⁹ × the size of the world) and is located again.
- **Overlap check:** before running, 5000 points of each surface are tested.
  A point outside the mother means the volume sticks out of its mother; a point
  inside a sister means an overlap. The idea is that of Geant4's `CheckOverlaps`.

## Random number generators

Module `rng.rs`. All of them produce uniform numbers in (0, 1) with 53 bits.

| `rng` | Algorithm | Verification |
|---|---|---|
| `philox` | Philox4x32-10 (Salmon et al., 2011), counter-based | Random123 known-answer tests |
| `xoshiro` | Xoshiro256++ (Blackman and Vigna) | known output for the state [1, 2, 3, 4] |
| `pcg` | PCG64 XSL-RR 128/64 (O'Neill) | statistical moments |
| `mt19937` | 32-bit Mersenne Twister | 10000th output for seed 5489 (C++ standard) |

## Parallelism and reproducibility

Histories are split into fixed blocks of 1000. Block b always uses stream b of
the generator: for Philox, the counter (b, i) with the key equal to the seed;
for the others, the state is derived from (seed, b) by SplitMix64. Therefore
**the result does not depend on the number of threads**, and a test checks it.
Blocks are distributed among threads with rayon. Each thread accumulates in a
`Tally` of its own, and the tallies are summed at the end of each batch.

## Uncertainty

Blocks are grouped in B batches (`batches`). For every cell of every probe,
the total sum and the sum of the squares of the per-history means of each
batch are kept. The uncertainty of the mean is

  σ²(x̄) = [ (1/B) Σ x̄_b² − x̄² ] · B/(B − 1) / B

## Scoring

- **Deposited energy:** goes to the grid cell where it falls and to the volume
  probes of the current region.
- **Track length:** each path segment in a region is added to the energy bin of
  the `spectrum` probes of that region.
- **Balance:** the `Tally` also accumulates the energy emitted, deposited and
  escaped from the world, to check energy conservation.

## Dose

D = E_dep / m, with E in J (1 MeV = 1.602176634×10⁻¹³ J) and m in kg.

- **Volumes:** m = ρ × (volume of the solid − volumes of the daughters), computed analytically.
- **Voxels:** m = ⟨ρ⟩ × voxel volume, with ⟨ρ⟩ the mean over 3×3×3 points inside the voxel.
