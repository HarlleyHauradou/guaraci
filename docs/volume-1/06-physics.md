# 6. Photon physics

Module `guaraci-core/src/physics.rs`. Units: MeV, cm, g/cm³, barn.

## Tables per material

For each material used in the geometry, the macroscopic cross sections
μ = Σᵢ nᵢ σᵢ(E) (cm⁻¹) are tabulated on a logarithmic energy grid with 1000
points per decade, from the energy cut-off up to 1.01 × the highest emitted
energy. Between points, interpolation is linear in the logarithmic index.

- **Atom density:** nᵢ = ρ wᵢ N_A / Aᵢ, with wᵢ the mass fraction and Aᵢ the
  atomic mass (from the ENDF AWR).
- **Element cross sections:** read from the ENDF tables with linear-linear interpolation.
- **Process switched off:** with `rayleigh = off`, for example, the process is
  simply left out of μ, which is the same as removing it from the physics.
- **Choice of the target atom:** the Compton and Rayleigh parts of each element
  are also stored. The atom is sampled with probability nᵢσᵢ(E)/Σⱼnⱼσⱼ(E).

## Free path

The distance to the next interaction is s = −ln(ξ)/μ(E). If s is larger than
the distance to the next boundary, the photon moves to the boundary, crosses it
and continues in the new medium. The process is memoryless, so the remaining
path does not need to be kept.

## Photoelectric effect

The photon is absorbed and all its energy is deposited on the spot. This
includes the binding energy, since fluorescence is not emitted in this version.

## Compton scattering

Sampling follows the Klein-Nishina formula, with the composition and rejection
method of Geant4's `G4KleinNishinaCompton`. For ε = E′/E in [ε₀, 1],
ε₀ = 1/(1 + 2k), k = E/mₑc²:

1. With probability α₁/(α₁ + α₂), α₁ = −ln ε₀ and α₂ = (1 − ε₀²)/2, sample
   ε = ε₀^ξ; otherwise ε² is uniform in [ε₀², 1].
2. Accept with probability g = 1 − ε sin²θ / (1 + ε²).

With `compton = klein_nishina_sf` (the default), a second rejection uses the
incoherent scattering function: accept with probability S(x, Z)/Z, where
x = (E/hc) sin(θ/2). This reduces scattering at small angles, where the
binding of the electron matters. The energy E − E′ goes to the electron and is
deposited on the spot. Doppler broadening is not simulated.

## Rayleigh scattering

Sampling follows dσ/dΩ ∝ (1 + cos²θ)/2 · F²(x, Z):

1. x² is sampled from the distribution F²(x²) between 0 and x²_max = (E/hc)²,
   by inverting the cumulative integral ∫F² d(x²), precomputed with the
   trapezoidal rule for each element.
2. cos θ = 1 − 2 x²/x²_max.
3. Accept with probability (1 + cos²θ)/2.

The photon keeps its energy.

## Pair production

The kinetic energy of the pair, E − 2mₑc², is deposited on the spot. Two
photons of mₑc² leave in opposite, isotropic directions.

## Energy cut-off

A photon below `photon_cut` (default 1 keV) deposits its energy on the spot.

## Emission of radioactive sources

For each decay:

- **Photon lines:** a line of intensity I (photons per decay) emits ⌊I⌋
  photons, plus one with probability I − ⌊I⌋. Lines are independent of each
  other; γ-γ correlations are not reproduced.
- **Annihilation:** the 511 keV photons leave in collinear pairs from the decay
  point (without the positron range).
- **Electrons:** with `beta = local`, the mean energy per decay carried by
  light particles, ⟨E_LP⟩ from ENDF (β⁻, β⁺, Auger and conversion electrons),
  is deposited at the decay point.
- **Position and direction:**
  - for a source with `in = volume`, the position is uniform in the region of
    the volume (the solid minus its daughters), sampled by rejection in its
    bounding box;
  - the direction of each photon is isotropic.

## Constants

| Constant | Value | Source |
|---|---|---|
| mₑc² | 0.51099895000 MeV | CODATA 2018 |
| hc | 1.239841984×10⁻¹⁰ MeV·cm | CODATA 2018 |
| N_A | 6.02214076×10²³ mol⁻¹ | SI 2019 (exact) |
| 1 MeV | 1.602176634×10⁻¹³ J | SI 2019 (exact) |
| mₙ | 1.00866491595 u | CODATA 2018 |
