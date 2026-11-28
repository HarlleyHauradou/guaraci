#!/usr/bin/env python3
"""Write equivalent Guaraci, GATE 10 and MCNPX inputs for the vial benchmark.

A 10 mL borosilicate vial filled with water, with a radionuclide spread
uniformly in the water. Two modes:

- photons (default): the photon lines of the nuclide (default I-131); photons
  only, electron energy deposited where the electron is created, in all three codes;
- beta (--beta): the β⁻ spectrum of the nuclide (e.g. Y-90); electrons and
  photons transported in all three codes.

The source is taken from Guaraci's decay data (ENDF/B-VIII.0) with
`guaraci info nuclide <name> --csv` (photon lines) or `--beta-csv` (β⁻
histogram), so the three codes use exactly the same spectrum. Nothing from the
data library is stored in the repository.

    python3 make_inputs.py                                  # I-131 photons, writes ./inputs/
    python3 make_inputs.py --nuclide Tc-99m --histories 2e7 --out tc99m
    python3 make_inputs.py --nuclide Y-90 --beta --out inputs_y90
"""

import argparse
import csv
import io
import math
import os
import subprocess
import sys

# ---------------------------------------------------------------------------
# Geometry (cm). The vial is centred at the origin, axis along z.
WORLD = 30.0               # side of the air cube
VIAL_R, VIAL_H = 1.1, 4.5  # glass, outer
CAV_R, CAV_H = 1.0, 4.3    # air cavity: 1 mm wall, bottom and top
WAT_R, WAT_H = 1.0, 3.2    # water: 10.05 mL resting on the bottom
WAT_Z = -CAV_H / 2 + WAT_H / 2   # centre of the water = -0.55 cm
VOXEL = 0.1                # dose grid, 1 mm

# Materials: identical mass fractions and densities in the three codes
# (PNNL-15870 Rev. 1, as in Guaraci's database: Water, Air, Glass_Borosilicate).
MATERIALS = {
    "water": (0.998207, [("H", 1, 0.111894), ("O", 8, 0.888106)]),
    "air": (0.001205, [("C", 6, 0.000124), ("N", 7, 0.755268), ("O", 8, 0.231781), ("Ar", 18, 0.012827)]),
    "glass": (2.23, [("B", 5, 0.040064), ("O", 8, 0.539562), ("Na", 11, 0.028191),
                     ("Al", 13, 0.011644), ("Si", 14, 0.37722), ("K", 19, 0.003321)]),
}


def run_guaraci(guaraci, args):
    """Run `guaraci <args>` and return its output; on failure, show Guaraci's own message."""
    try:
        p = subprocess.run([guaraci, *args], capture_output=True, text=True)
    except FileNotFoundError:
        sys.exit(f"cannot run `{guaraci}`: is Guaraci installed and in the PATH? (or give --guaraci <path>)")
    if p.returncode != 0:
        msg = (p.stderr or p.stdout).strip()
        sys.exit(f"`{guaraci} {' '.join(args)}` failed:\n{msg}\n\n"
                 "If the decay data is missing or outdated: guaraci data install endfb8 --force")
    return p.stdout


def lines_from_guaraci(guaraci, nuclide):
    out = run_guaraci(guaraci, ["info", "nuclide", nuclide, "--csv"])
    header = [l for l in out.splitlines() if l.startswith("#")]
    rows = list(csv.DictReader(io.StringIO("\n".join(l for l in out.splitlines() if not l.startswith("#")))))
    lines = [(float(r["energy_keV"]), float(r["photons_per_decay"]), r["kind"]) for r in rows]
    return (header[0][2:] if header else nuclide), lines


def beta_from_guaraci(guaraci, nuclide, bins):
    """β⁻ histogram: (origin, info line, [(E_low_MeV, E_high_MeV, probability)])."""
    out = run_guaraci(guaraci, ["info", "nuclide", nuclide, "--beta-csv", "--bins", str(bins)])
    comments = [l[2:] for l in out.splitlines() if l.startswith("#")]
    rows = list(csv.DictReader(io.StringIO("\n".join(l for l in out.splitlines() if not l.startswith("#")))))
    hist = [(float(r["E_low_MeV"]), float(r["E_high_MeV"]), float(r["probability"])) for r in rows]
    return comments[0], comments[1], hist


def wrap(prefix, items, width=78, indent="     "):
    """MCNP card continuation: lines of at most 80 columns."""
    out, cur = [], prefix
    for it in items:
        if len(cur) + 1 + len(it) > width:
            out.append(cur)
            cur = indent + it
        else:
            cur += " " + it
    out.append(cur)
    return "\n".join(out)


def guaraci_geometry():
    return f"""[material VialWater]
density = {MATERIALS['water'][0]} g/cm3
composition = {{ {", ".join(f"{s} = {w}" for s, _, w in MATERIALS['water'][1])} }}

[material VialAir]
density = {MATERIALS['air'][0]} g/cm3
composition = {{ {", ".join(f"{s} = {w}" for s, _, w in MATERIALS['air'][1])} }}

[material VialGlass]
density = {MATERIALS['glass'][0]} g/cm3
composition = {{ {", ".join(f"{s} = {w}" for s, _, w in MATERIALS['glass'][1])} }}

[world]
material = VialAir
size = {WORLD} cm

[volume vial]
shape = cylinder
radius = {VIAL_R} cm
height = {VIAL_H} cm
material = VialGlass

[volume cavity]
parent = vial
shape = cylinder
radius = {CAV_R} cm
height = {CAV_H} cm
material = VialAir

[volume water]
parent = cavity
shape = cylinder
radius = {WAT_R} cm
height = {WAT_H} cm
center = [0, 0, {WAT_Z:.4f}] cm
material = VialWater

"""


def guaraci_input(nuclide, origin, lines, histories):
    e = ", ".join(f"{l[0]:.4f}" for l in lines)
    i = ", ".join(f"{l[1]:.6e}" for l in lines)
    return f"""# Vial benchmark: 10 mL of water in a borosilicate vial, {nuclide} photon lines.
# Generated by validation/vial_i131/make_inputs.py
# Lines: {origin}
# Same geometry, materials and spectrum as gate10_vial.py and mcnpx_vial.inp.

[run]
histories = {histories:.0f}
seed = 2026

[physics]
# photons only: photoelectric, Compton (Klein-Nishina with S(x,Z)), Rayleigh, pair;
# electrons deposit their energy where they are created (as in the GATE and MCNPX inputs)
list = photon_local

{guaraci_geometry()}[source]
# photon lines only (no beta): energy in keV, intensity in photons per decay
energy = [{e}] keV
intensity = [{i}]
in = water

[probe edep]
type = volume_dose
volumes = [water, cavity, vial]

[probe dose]
type = dose_grid
in = vial
voxel = [{VOXEL}, {VOXEL}, {VOXEL}] cm

[probe spectrum]
type = spectrum
in = water
emax = 700 keV
bins = 700
"""


def guaraci_beta_input(nuclide, origin, info, histories):
    return f"""# Vial benchmark, beta mode: 10 mL of water in a borosilicate vial, {nuclide} β⁻ spectrum.
# Generated by validation/vial_i131/make_inputs.py
# Spectrum: {origin}
#           {info}
# Same geometry, materials and spectrum as gate10_vial.py and mcnpx_vial.inp.

[run]
histories = {histories:.0f}
seed = 2026

[physics]
# electrons and photons transported (mixed condensed history), as in the GATE and MCNPX inputs
list = photon_electron
electron_cut = 10 keV

{guaraci_geometry()}[source]
# the nuclide's decay: its β⁻ spectrum (the same histogram as beta.csv); its
# photon lines and conversion electrons carry less than 0.1 % of its energy:
# Guaraci emits them, the GATE and MCNPX inputs do not
nuclide = {nuclide}
beta = transport
in = water

[probe edep]
type = volume_dose
volumes = [water, cavity, vial]

[probe dose]
type = dose_grid
in = vial
voxel = [{VOXEL}, {VOXEL}, {VOXEL}] cm
"""


GATE_PHYSICS_PHOTONS = """# ---- physics: photons only, local electron deposition
sim.physics_manager.physics_list_name = "G4EmLivermorePhysics"
em = sim.physics_manager.em_parameters
em.fluo = False
em.auger = False
em.pixe = False
# kill every electron at creation: its kinetic energy is deposited locally
# (Geant4 does not accept a user region on the world volume: the region is
# the air box `room`, which contains everything else)
region = sim.physics_manager.add_region("everything")
region.associate_volume("room")
region.user_limits["min_ekine"] = 1 * GeV
sim.physics_manager.user_limits_particles = ["electron"]"""

GATE_PHYSICS_BETA = """# ---- physics: electrons and photons transported
# option4: the standard list recommended for dosimetry (Goudsmit-Saunderson
# multiple scattering for electrons); no fluorescence, as in Guaraci
sim.physics_manager.physics_list_name = "G4EmStandardPhysics_option4"
em = sim.physics_manager.em_parameters
em.fluo = False
em.auger = False
em.pixe = False
# production cut 0.01 mm everywhere (of the order of 10 keV for electrons in
# water; Geant4 prints the energy equivalent at start-up). Guaraci: electron_cut = 10 keV
sim.physics_manager.global_production_cuts.all = 0.01 * mm"""


def gate_input(nuclide, origin, lines, histories, beta=None):
    """`beta`: β⁻ histogram [(E_low, E_high, p)] for the beta mode, else photon lines."""
    what = f"{nuclide} β⁻ spectrum" if beta else f"{nuclide} photon lines"
    # where the energy of a step is scored: at the interaction point ("post") for
    # photons, whose steps are centimetres long; anywhere on the (short) step for electrons
    hit = "random" if beta else "post"
    if beta:
        physics_doc = """Physics: G4EmStandardPhysics_option4, electrons and photons transported;
fluorescence, Auger and PIXE off; production cut 0.01 mm."""
        physics = GATE_PHYSICS_BETA
        spectrum = f"""EDGES_MEV = {[beta[0][0]] + [round(h[1], 6) for h in beta]!r}
PROBS = {[h[2] for h in beta]!r}"""
        source = f"""# ---- source: one electron per event, uniform in the water, isotropic, β⁻ histogram
src = sim.add_source("GenericSource", "electrons")
src.particle = "e-"
src.n = PRIMARIES_PER_THREAD
src.position.type = "box"
src.position.size = [{2 * WAT_R} * cm, {2 * WAT_R} * cm, {WAT_H} * cm]
src.position.translation = [0, 0, {WAT_Z:.4f} * cm]
src.position.confine = "water"
src.direction.type = "iso"
# histogram: probability PROBS[k] between EDGES_MEV[k] and EDGES_MEV[k+1], flat inside the bin
src.energy.type = "spectrum_histogram"
src.energy.spectrum_energy_bin_edges = [e * MeV for e in EDGES_MEV]
src.energy.spectrum_weights = PROBS"""
    else:
        physics_doc = """Physics: G4EmLivermorePhysics; fluorescence, Auger and PIXE off; electrons are
killed as soon as they are created (user limit min_ekine), so their energy is
deposited at the interaction point, as in Guaraci and in MCNP with IDES=1."""
        physics = GATE_PHYSICS_PHOTONS
        spectrum = f"""ENERGIES_KEV = {[round(l[0], 4) for l in lines]!r}
WEIGHTS = {[l[1] for l in lines]!r}"""
        source = f"""# ---- source: one photon per event, uniform in the water, isotropic
src = sim.add_source("GenericSource", "photons")
src.particle = "gamma"
src.n = PRIMARIES_PER_THREAD
src.position.type = "box"
src.position.size = [{2 * WAT_R} * cm, {2 * WAT_R} * cm, {WAT_H} * cm]
src.position.translation = [0, 0, {WAT_Z:.4f} * cm]
src.position.confine = "water"
src.direction.type = "iso"
try:  # opengate >= 10.0 (newer naming)
    src.energy.type = "spectrum_discrete"
    src.energy.spectrum_energies = [e * keV for e in ENERGIES_KEV]
    src.energy.spectrum_weights = WEIGHTS
except Exception:  # older 10.x naming
    src.energy.type = "spectrum_lines"
    src.energy.spectrum_energy = [e * keV for e in ENERGIES_KEV]
    src.energy.spectrum_weight = WEIGHTS"""
    mats = "\n".join(
        f'db.add_material_weights("Vial_{k.capitalize()}", {[s for s, _, _ in v[1]]!r}, {[w for _, _, w in v[1]]!r}, {v[0]} * gcm3)'
        for k, v in MATERIALS.items()
    )
    nx = round(2 * VIAL_R / VOXEL)
    nz = round(VIAL_H / VOXEL)
    return f'''#!/usr/bin/env python3
"""GATE 10 (opengate) version of the vial benchmark: {what} in 10 mL of water.

Generated by validation/vial_i131/make_inputs.py. Source: {origin}
Same geometry, materials and spectrum as vial.gci and mcnpx_vial.inp.

{physics_doc}

Run:  python3 gate10_vial.py      (results in gate_output/, next to this file)
This file was written for opengate 10.x; parameter names of the energy
spectrum and of the user limits changed between minor versions; if a name is
rejected, see the opengate documentation for your version (the intent of each
line is commented).
"""

import os

import opengate as gate
from opengate.utility import g4_units

cm, mm, keV, MeV, km, GeV = g4_units.cm, g4_units.mm, g4_units.keV, g4_units.MeV, g4_units.km, g4_units.GeV
gcm3 = g4_units.g_cm3

{spectrum}

# threads and output folder can be set from outside (run_all.py does);
# in opengate, src.n is the number of primaries of EACH thread
THREADS = int(os.environ.get("GATE_THREADS", "4"))
HISTORIES = int(float(os.environ.get("GATE_HISTORIES", "{histories:.0f}")))
PRIMARIES_PER_THREAD = max(1, HISTORIES // THREADS)
OUTPUT = os.environ.get("GATE_OUTPUT", os.path.join(os.path.dirname(os.path.abspath(__file__)), "gate_output"))

sim = gate.Simulation()
sim.number_of_threads = THREADS
sim.random_seed = int(os.environ.get("GATE_SEED", "2026"))
sim.output_dir = OUTPUT
sim.visu = False

# ---- materials: same mass fractions and densities as Guaraci and MCNP
db = sim.volume_manager.material_database
{mats}

# ---- geometry
# the world is vacuum; the air box `room` inside it is the {WORLD} cm air cube of
# vial.gci and mcnpx_vial.inp (a particle leaving it never comes back)
sim.world.size = [{WORLD + 2} * cm] * 3
sim.world.material = "G4_Galactic"

room = sim.add_volume("Box", "room")
room.size = [{WORLD} * cm] * 3
room.material = "Vial_Air"

vial = sim.add_volume("Tubs", "vial")
vial.mother = "room"
vial.rmin, vial.rmax, vial.dz = 0, {VIAL_R} * cm, {VIAL_H / 2} * cm       # dz = half length
vial.material = "Vial_Glass"

cavity = sim.add_volume("Tubs", "cavity")
cavity.mother = "vial"
cavity.rmin, cavity.rmax, cavity.dz = 0, {CAV_R} * cm, {CAV_H / 2} * cm
cavity.material = "Vial_Air"

water = sim.add_volume("Tubs", "water")
water.mother = "cavity"
water.rmin, water.rmax, water.dz = 0, {WAT_R} * cm, {WAT_H / 2} * cm
water.translation = [0, 0, {WAT_Z:.4f} * cm]
water.material = "Vial_Water"

{physics}

{source}

# ---- actors
stats = sim.add_actor("SimulationStatisticsActor", "stats")
stats.output_filename = "stats.json"

# energy deposited in the water, in the vial (glass + everything inside) and in
# the cavity (air + water): one voxel each; glass = vial - cavity
for name, vol, size in [
    ("water", "water", [{2 * WAT_R} * cm, {2 * WAT_R} * cm, {WAT_H} * cm]),
    ("vial", "vial", [{2 * VIAL_R} * cm, {2 * VIAL_R} * cm, {VIAL_H} * cm]),
    ("cavity", "cavity", [{2 * CAV_R} * cm, {2 * CAV_R} * cm, {CAV_H} * cm]),
]:
    a = sim.add_actor("DoseActor", f"{{name}}_edep")
    a.attached_to = vol
    a.size = [1, 1, 1]
    a.spacing = size
    a.output_filename = f"{{name}}.mhd"
    a.hit_type = "{hit}"

# 1 mm dose grid over the vial bounding box ({nx} x {nx} x {nz}), as Guaraci's `dose` probe
d = sim.add_actor("DoseActor", "dose_grid")
d.attached_to = "vial"
d.size = [{nx}, {nx}, {nz}]
d.spacing = [{VOXEL * 10} * mm] * 3
d.output_filename = "dose_grid.mhd"
d.hit_type = "{hit}"   # photons: "post" (at the interaction point); electrons: "random" (along the step)
d.dose.active = True
d.dose_uncertainty.active = True

sim.run()
print(stats)
os.makedirs(OUTPUT, exist_ok=True)
with open(os.path.join(OUTPUT, "events.txt"), "w") as f:
    f.write(f"{{PRIMARIES_PER_THREAD * THREADS}}\\n")
print(f"Results in {{OUTPUT}} (water_edep.mhd: energy in the water, MeV; dose_grid_dose.mhd: Gy)")
'''


def mcnp_input(nuclide, origin, lines, histories, beta=None):
    """`beta`: β⁻ histogram [(E_low, E_high, p)] for the beta mode, else photon lines."""
    def m_card(n, key):
        return wrap(f"M{n}", [f"{z * 1000} -{w}" for _, z, w in MATERIALS[key][1]])

    h = WORLD / 2
    if beta:
        return mcnp_beta_input(nuclide, origin, histories, beta, m_card, h)
    erg = wrap("SI3 L", [f"{l[0] / 1000:.7f}" for l in lines])
    prob = wrap("SP3 D", [f"{l[1]:.6e}" for l in lines])
    return f"""Vial benchmark: 10 mL water, {nuclide} photon lines (Guaraci comparison)
c Generated by validation/vial_i131/make_inputs.py
c Lines: {origin[:70]}
c Same geometry, materials and spectrum as vial.gci and gate10_vial.py.
c Photons only (MODE P); IDES=1: no electrons, their energy is deposited
c locally; NODOP=1: no Doppler broadening (as in Guaraci).
c ---------------------------------------------------------------- cells
1  1 -{MATERIALS['water'][0]}  -3          imp:p=1   $ water
2  2 -{MATERIALS['air'][0]}  -2 3        imp:p=1   $ air above the water
3  3 -{MATERIALS['glass'][0]}      -1 2        imp:p=1   $ glass
4  2 -{MATERIALS['air'][0]}  -4 1        imp:p=1   $ room air
5  0             4          imp:p=0   $ outside

c ------------------------------------------------------------- surfaces
1  RCC 0 0 {-VIAL_H / 2}  0 0 {VIAL_H}  {VIAL_R}
2  RCC 0 0 {-CAV_H / 2}  0 0 {CAV_H}  {CAV_R}
3  RCC 0 0 {WAT_Z - WAT_H / 2:.4f}  0 0 {WAT_H}  {WAT_R}
4  RPP {-h} {h} {-h} {h} {-h} {h}

c ----------------------------------------------------------------- data
MODE P
{m_card(1, 'water')}
{m_card(2, 'air')}
{m_card(3, 'glass')}
c      emcpf ides nocoh ispn nodop
PHYS:P 100   1    0     0    1
c source: uniform in the water cylinder, isotropic, discrete lines
SDEF CEL=1 POS=0 0 {WAT_Z - WAT_H / 2:.4f} AXS=0 0 1 RAD=D1 EXT=D2 ERG=D3 PAR=2
SI1 0 {WAT_R}
SP1 -21 1
SI2 0 {WAT_H}
SP2 -21 0
{erg}
{prob}
c energy deposition (MeV/g per photon) in water and glass
F6:P 1 3
c photon fluence in the water, 1 keV bins up to 700 keV (cm^-2 per photon)
F4:P 1
E4 0.001 698I 0.700
NPS {histories:.0f}
PRDMP 2J 1
"""


def mcnp_beta_input(nuclide, origin, histories, beta, m_card, h):
    edges = wrap("SI3 H", [f"{beta[0][0]:.6f}"] + [f"{b[1]:.6f}" for b in beta])
    probs = wrap("SP3 D", ["0"] + [f"{b[2]:.6e}" for b in beta])
    return f"""Vial benchmark: 10 mL water, {nuclide} beta-minus spectrum (Guaraci comparison)
c Generated by validation/vial_i131/make_inputs.py
c Spectrum: {origin[:66]}
c Same geometry, materials and spectrum as vial.gci and gate10_vial.py.
c Electrons and photons transported (MODE E P); electron cut-off 10 keV
c (as electron_cut in Guaraci); energy deposition from the *F8 tally.
c ---------------------------------------------------------------- cells
1  1 -{MATERIALS['water'][0]}  -3          imp:e,p=1   $ water
2  2 -{MATERIALS['air'][0]}  -2 3        imp:e,p=1   $ air above the water
3  3 -{MATERIALS['glass'][0]}      -1 2        imp:e,p=1   $ glass
4  2 -{MATERIALS['air'][0]}  -4 1        imp:e,p=1   $ room air
5  0             4          imp:e,p=0   $ outside

c ------------------------------------------------------------- surfaces
1  RCC 0 0 {-VIAL_H / 2}  0 0 {VIAL_H}  {VIAL_R}
2  RCC 0 0 {-CAV_H / 2}  0 0 {CAV_H}  {CAV_R}
3  RCC 0 0 {WAT_Z - WAT_H / 2:.4f}  0 0 {WAT_H}  {WAT_R}
4  RPP {-h} {h} {-h} {h} {-h} {h}

c ----------------------------------------------------------------- data
MODE E P
{m_card(1, 'water')}
{m_card(2, 'air')}
{m_card(3, 'glass')}
c electrons below 10 keV deposit their energy locally
CUT:E J 0.01
c source: electrons, uniform in the water cylinder, isotropic, beta histogram
SDEF CEL=1 POS=0 0 {WAT_Z - WAT_H / 2:.4f} AXS=0 0 1 RAD=D1 EXT=D2 ERG=D3 PAR=3
SI1 0 {WAT_R}
SP1 -21 1
SI2 0 {WAT_H}
SP2 -21 0
{edges}
{probs}
c energy deposited (MeV per source electron) in the water and in the glass,
c by electrons and photons together
*F8:E,P 1 3
NPS {histories:.0f}
PRDMP 2J 1
"""


def other_inputs(a, origin, lines, hist):
    """EGSnrc and FLUKA inputs (other_codes.py). MCNPX is written too, but only
    as an optional extra: it is not part of the cross-validation."""
    import other_codes as O

    spectrum = os.path.abspath(os.path.join(a.out, "egsnrc_vial.spectrum"))
    return {
        "egsnrc_vial.spectrum": O.egsnrc_spectrum(a.nuclide, origin, lines, hist),
        "egsnrc_vial.egsinp": O.egsnrc_input(a.nuclide, origin, a.histories, spectrum, beta=bool(hist)),
        "fluka_vial.inp": O.fluka_input(a.nuclide, origin, a.histories, beta=hist),
        "source.f": O.fluka_source(a.nuclide, lines, hist),
    }


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--nuclide", default="I-131")
    ap.add_argument("--histories", type=float, default=1e7)
    ap.add_argument("--out", default="inputs")
    ap.add_argument("--guaraci", default=os.environ.get("GUARACI", "guaraci"), help="path to the guaraci program")
    ap.add_argument("--beta", action="store_true", help="beta mode: the β⁻ spectrum of the nuclide, electrons transported")
    ap.add_argument("--bins", type=int, default=200, help="bins of the β⁻ histogram (beta mode)")
    a = ap.parse_args()

    if a.beta:
        origin, info, hist = beta_from_guaraci(a.guaraci, a.nuclide, a.bins)
        os.makedirs(a.out, exist_ok=True)
        with open(os.path.join(a.out, "beta.csv"), "w") as f:
            f.write(f"# {origin}\n# {info}\nE_low_MeV,E_high_MeV,probability\n")
            f.writelines(f"{lo},{hi},{p}\n" for lo, hi, p in hist)
        files = {
            "vial.gci": guaraci_beta_input(a.nuclide, origin, info, a.histories),
            "gate10_vial.py": gate_input(a.nuclide, origin, None, a.histories, beta=hist),
            **other_inputs(a, origin, None, hist),
            "mcnpx_vial.inp": mcnp_input(a.nuclide, origin, None, a.histories, beta=hist),
        }
        for name, text in files.items():
            with open(os.path.join(a.out, name), "w") as f:
                f.write(text)
        mean = sum(p * (lo + hi) / 2 for lo, hi, p in hist)
        print(f"{a.nuclide}: β⁻ histogram of {len(hist)} bins up to {hist[-1][1]:.4f} MeV, mean {mean * 1e3:.1f} keV")
        print(f"files written to {a.out}/: {', '.join(['beta.csv'] + list(files))}")
        return

    origin, lines = lines_from_guaraci(a.guaraci, a.nuclide)
    if not lines:
        sys.exit(f"{a.nuclide} has no photon lines")
    os.makedirs(a.out, exist_ok=True)
    with open(os.path.join(a.out, "lines.csv"), "w") as f:
        f.write(f"# {origin}\nenergy_keV,photons_per_decay,kind\n")
        f.writelines(f"{e},{i},{k}\n" for e, i, k in lines)
    files = {
        "vial.gci": guaraci_input(a.nuclide, origin, lines, a.histories),
        "gate10_vial.py": gate_input(a.nuclide, origin, lines, a.histories),
        **other_inputs(a, origin, lines, None),
        "mcnpx_vial.inp": mcnp_input(a.nuclide, origin, lines, a.histories),
    }
    for name, text in files.items():
        with open(os.path.join(a.out, name), "w") as f:
            f.write(text)
    total = sum(l[1] for l in lines)
    mean_e = sum(l[0] * l[1] for l in lines) / total
    vol = math.pi * WAT_R ** 2 * WAT_H
    print(f"{a.nuclide}: {len(lines)} photon lines, {total:.4f} photons per decay, mean energy {mean_e:.2f} keV")
    print(f"water volume {vol:.3f} cm3; files written to {a.out}/: {', '.join(['lines.csv'] + list(files))}")


if __name__ == "__main__":
    main()
