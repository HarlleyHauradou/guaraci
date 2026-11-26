# 7. Electron and positron physics

Module `guaraci-core/src/electron.rs`, used by `transport.rs`. Units: MeV, cm,
g/cm³. Electrons and positrons are transported with the physics list
`photon_electron` (the default); with `photon_local` they deposit their energy
where they are created.

## Mixed condensed history

A charged particle undergoes thousands of collisions per millimetre. As in
EGS, PENELOPE and Geant4, Guaraci groups them into steps (a *condensed
history*) and separates two kinds of events:

- **Soft events** (energy transfer below a threshold): their total effect is
  continuous, with the *restricted* stopping power L(E).
- **Hard events** (above the threshold): simulated one by one, with their
  secondaries:
  - knock-on electrons (δ-rays) above `electron_cut`;
  - bremsstrahlung photons above the photon threshold k_c = max(`photon_cut`, 1 keV).

Elastic scattering is condensed: the direction changes at the end of each
step, by an angle sampled so that the mean deflection is exact.

## Tables per material

For each material, a logarithmic grid with 200 points per decade, from the
cut-off up to the highest energy of the problem, holds:

- the restricted stopping power L(E) for electrons and for positrons;
- the unrestricted stopping power S(E) and the CSDA range (for reports and checks);
- the range R(E) = ∫ dE / L(E), from the cut-off;
- the macroscopic cross sections of the hard events, Σ_δ(E) and Σ_b(E);
- the inverse transport mean free path, 1/λ₁(E).

## Collision stopping power

The Bethe formula, with the Rohrlich–Carlson terms for electrons and positrons
(ICRU Report 37):

  S_col/ρ = K (Z/A) / β² · [ ln(τ²(τ + 2) / (2 (I/mₑc²)²)) + F^±(τ) − δ ]

with K = 2π r_e² mₑc² N_A = 0.153537 MeV cm²/mol and τ = T/mₑc².

- **Mean excitation energy I:**
  - Bragg rule over the elements, ln I = Σ wᵢ (Zᵢ/Aᵢ) ln Iᵢ / Σ wᵢ (Zᵢ/Aᵢ);
  - Iᵢ from ICRU Report 37 (the values used by NIST ESTAR);
  - in condensed media, H, C, N, O, F and Cl take 19.2, 81, 82, 106, 112 and
    180 eV, and the other elements 1.13 × their gas value;
  - `mean_excitation` in a `[material]` replaces the Bragg rule (water: 75 eV).
- **Density effect δ:** the parametrisation of Sternheimer and Peierls (1971),
  with its constants computed from I and the plasma energy of the material.

The collision stopping power derived from the library (EEDL) is not used: in
water it is about 7 % above ESTAR at 1 MeV.

## Radiative stopping power and bremsstrahlung spectra

From the electro-atomic library (EEDL, ENDF-6 MF23/MF26 MT527):

- the cross section σ_b(E) and the mean photon energy ⟨k⟩(E) give
  S_rad = Σᵢ nᵢ σ_b,i ⟨k⟩ᵢ;
- the photon spectra p(k | E), tabulated at a few incident energies, are kept
  in the reduced variable κ = k/E and interpolated in ln E between them.

Photons below k_c are part of L(E): their share of S_rad is
∫₀^κc κ p dκ / ∫₀¹ κ p dκ. Above k_c, the number of photons per unit length is
Σ_b = Σᵢ nᵢ σ_b,i ∫_κc¹ p dκ.

## Knock-on electrons (δ-rays)

The Møller (electrons) and Bhabha (positrons) cross sections for free electrons
at rest, as in Geant4's `G4MollerBhabhaModel`:

- largest energy transfer: T/2 for electrons (the faster electron is called the
  primary) and T for positrons;
- Σ_δ(E) is the integral of the cross section from `electron_cut` up to that maximum;
- the restricted collision stopping power subtracts the energy carried by the
  hard collisions:

  L_col = S_col − K (Z/A) ρ / β² · [ G(τ, W_max) − G(τ, W_cut) ]

  with G the transfer-dependent part of the restricted Bethe formula
  (Seltzer and Berger). A unit test checks that L_col + Σ_δ⟨W⟩ = S_col within 0.2 %.

The energy W of the δ-ray is sampled from 1/W² and accepted by rejection with
the rest of the cross section. The directions of the δ-ray and of the primary
follow from energy and momentum conservation.

## Elastic scattering and the transport mean free path

  1/λ₁ = Σᵢ nᵢ σ_el,i (1 − ⟨cos θ⟩ᵢ)

with the large-angle elastic cross section (MT525) and the mean cosine of its
angular distribution (MF26). The angular tables of EEDL are sparse at high
energy (for oxygen: 256 keV, then 10 MeV), and 1 − ⟨cos θ⟩ falls by a factor of
40 between them. It is therefore interpolated log-log in energy. A linear
interpolation would make λ₁ 2.5 times too short at 1 MeV.

At the end of a step of length s, the deflection is sampled from a screened
Rutherford distribution,

  p(μ) ∝ 1/(1 − μ + 2η)²,

whose screening parameter η is chosen so that ⟨1 − cos θ⟩ = 1 − e^(−s/λ₁), the
exact mean after a path s (Lewis, 1950).

## Step

The step s is the smallest of:

1. the path that loses 15 % of the energy (from the range table);
2. 0.3 λ₁;
3. the residual range down to the cut-off;
4. the distance to the next hard event, s_h = −ln ξ / Σ_max, where Σ_max is
   the largest Σ_δ + Σ_b over the energies of the step;
5. the distance to the next boundary.

Along the step:

- the energy after the step comes from the range: E₁ = R⁻¹(R(E₀) − s);
- the loss E₀ − E₁ is deposited at a uniform point of the segment;
- at a boundary, the particle crosses and continues with the tables of the new
  material.

When the step ends on a hard event, the event happens with probability
Σ(E₁)/Σ_max, and is then a δ-ray or a photon in proportion to Σ_δ and Σ_b.
Otherwise it is a virtual event, and nothing happens: this keeps the sampling
exact although Σ changes along the step.

## Cut-off and positrons

- Below `electron_cut` (default 10 keV, minimum 1 keV), the particle deposits
  its energy on the spot.
- A positron below the cut-off, or at the end of its range, annihilates at
  rest: two photons of 511 keV leave in opposite, isotropic directions.
  Annihilation in flight is not simulated.

## Electrons created by photons

| Interaction | Electron |
|---|---|
| photoelectric | all the photon energy (binding included, since fluorescence is not emitted), direction from the Sauter–Gavrila distribution (K shell) |
| Compton | E − E′, direction from momentum conservation |
| pair production | E − 2mₑc² shared uniformly between the electron and the positron, both close to the photon direction (θ ≈ mₑc²/E) |

## Electrons from radioactive decay

With `beta = transport` and the list `photon_electron`:

- **β⁻ and β⁺ branches:** one spectrum per branch, from the endpoint of the
  decay library and Fermi theory:

  N(T) ∝ F(Z, W) · p W (W₀ − W)² · C(W)

  - W is the total energy in units of mₑc² and p the momentum.
  - F is the Fermi function in the usual relativistic approximation:
    2πη/(1 − e^(−2πη)) · p^(2(γ₀−1)), with η = ±αZW/p and γ₀ = √(1 − (αZ)²).
    Z is the charge of the daughter nucleus, and the sign is negative for β⁺.
  - C = 1 for allowed transitions, and p² + q² for first-forbidden unique ones.
  - For β⁺, the library gives the decay energy of the branch, and the positron
    endpoint is that energy minus 2mₑc².
- **Conversion and Auger electrons:** discrete lines with their intensities.
- **Annihilation photons:** the 511 keV lines of the decay library are left
  out, since the transported positrons make their own.

Check: the mean β energy of the main I-131 branch (endpoint 606.3 keV) is
191.6 keV (ENSDF), and the F-18 positron spectrum has a mean of 250 keV.

## Limitations

- No energy-loss straggling below the δ-ray and bremsstrahlung thresholds:
  soft losses are their mean value.
- δ-rays are produced on free electrons (atomic binding is neglected), and no
  fluorescence or Auger cascade follows the ionisation.
- Annihilation in flight is not simulated.
