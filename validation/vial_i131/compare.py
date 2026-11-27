#!/usr/bin/env python3
"""Compare Guaraci, GATE 10 and MCNPX results of the vial benchmark.

Every result is expressed per emitted particle: per photon in the photon mode,
per β⁻ electron in the beta mode (detected from beta.csv in the inputs folder).
Any of the three codes may be missing; it is simply left out of the table.

    python3 compare.py --inputs inputs \
        --guaraci vial_results --gate gate_output --mcnp outp

Writes comparison.csv (one row per quantity) and prints a readable table.
Only the Python standard library is used.
"""

import argparse
import csv
import glob
import math
import os
import re
import struct

from make_inputs import MATERIALS, CAV_H, CAV_R, VIAL_H, VIAL_R, WAT_H, WAT_R

WATER_MASS = MATERIALS["water"][0] * math.pi * WAT_R ** 2 * WAT_H
GLASS_MASS = MATERIALS["glass"][0] * math.pi * (VIAL_R ** 2 * VIAL_H - CAV_R ** 2 * CAV_H)


# ---------------------------------------------------------------- readers
def read_mhd(path):
    hdr = {}
    for line in open(path):
        if "=" in line:
            k, v = line.split("=", 1)
            hdr[k.strip()] = v.strip()
    dims = [int(x) for x in hdr["DimSize"].split()]
    kind = {"MET_FLOAT": "f", "MET_DOUBLE": "d"}[hdr["ElementType"]]
    raw = os.path.join(os.path.dirname(path), hdr["ElementDataFile"])
    n = dims[0] * dims[1] * dims[2]
    data = struct.unpack(f"<{n}{kind}", open(raw, "rb").read(n * struct.calcsize(kind)))
    return dims, data


def z_profile(dims, data, radius_vox=4):
    """Mean dose of the voxels within `radius_vox` voxels of the axis, per z slice."""
    nx, ny, nz = dims
    cx, cy = nx / 2, ny / 2
    ring = [(i, j) for i in range(nx) for j in range(ny) if (i + 0.5 - cx) ** 2 + (j + 0.5 - cy) ** 2 <= radius_vox ** 2]
    return [sum(data[i + nx * (j + ny * k)] for i, j in ring) / len(ring) for k in range(nz)]


def photons_per_decay(inputs):
    rows = [l for l in open(os.path.join(inputs, "lines.csv")) if not l.startswith("#") and not l.startswith("energy")]
    return sum(float(r.split(",")[1]) for r in rows)


def betas_per_decay(inputs):
    """Beta mode: β⁻ per decay, from the header of beta.csv (None in the photon mode)."""
    path = os.path.join(inputs, "beta.csv")
    if not os.path.exists(path):
        return None
    m = re.search(r"beta_minus_per_decay = ([\d.eE+-]+)", open(path).read())
    return float(m.group(1)) if m else 1.0


def guaraci(path, per_decay):
    out = {}
    with open(os.path.join(path, "edep.csv")) as f:
        for r in csv.DictReader(f):
            e = float(r["energy_MeV_per_decay"]) / per_decay * 1e3
            u = float(r["uncertainty_percent"])
            out[f"edep_{r['volume']}_keV"] = (e, u)
    dims, data = read_mhd(os.path.join(path, "dose.mhd"))
    # Guaraci's dose unit is Gy per decay here (no activity given)
    out["z_profile"] = [v / per_decay for v in z_profile(dims, data)]
    spec_path = os.path.join(path, "spectrum.csv")
    if os.path.exists(spec_path):
        spec = list(csv.DictReader(open(spec_path)))
        out["spectrum"] = [float(r["fluence"]) / per_decay for r in spec]
    return out


def gate(path, events=None):
    out = {}
    stats = os.path.join(path, "stats.txt")
    if not events and os.path.exists(stats):
        m = re.search(r"Events?\s*(?:\(\w+\))?\s*[:=]?\s*(\d+)", open(stats).read())
        events = int(m.group(1)) if m else None
    if not events:
        print("GATE: number of events not found in stats.txt; give it with --gate-events")
        return out
    edep = [p for p in glob.glob(os.path.join(path, "water*edep*.mhd")) if "uncertainty" not in p]
    if edep:
        _, data = read_mhd(edep[0])
        out["edep_water_keV"] = (sum(data) / events * 1e3, float("nan"))
    dose = [p for p in glob.glob(os.path.join(path, "dose_grid*dose*.mhd")) if "uncertainty" not in p]
    if dose:
        dims, data = read_mhd(dose[0])
        out["z_profile"] = [v / events for v in z_profile(dims, data)]
    return out


def mcnp(path):
    """Photon mode: F6 (cells 1 and 3, MeV/g per photon) and F4 (cell 1 spectrum).
    Beta mode: *F8 (cells 1 and 3, MeV per source electron)."""
    text = open(path).read()
    out = {}
    blocks = re.split(r"\n1tally\s+", text)
    for b in blocks:
        if re.match(r"8\s", b):
            for cell, val, err in re.findall(r"cell\s+(\d+)\s*\n\s*([\d.E+-]+)\s+([\d.]+)", b):
                name = "water" if cell == "1" else "vial"
                out[f"edep_{name}_keV"] = (float(val) * 1e3, float(err) * 100)
        if re.match(r"6\s", b):
            for cell, val, err in re.findall(r"cell\s+(\d+)\s*\n\s*([\d.E+-]+)\s+([\d.]+)", b):
                mass = WATER_MASS if cell == "1" else GLASS_MASS
                name = "water" if cell == "1" else "vial"
                out[f"edep_{name}_keV"] = (float(val) * mass * 1e3, float(err) * 100)
        if re.match(r"4\s", b):
            rows = re.findall(r"^\s+(\d\.\d+E[+-]\d+)\s+([\d.]+E[+-]\d+)\s+([\d.]+)\s*$", b, re.M)
            out["spectrum"] = [float(v) for _, v, _ in rows][1:]  # first bin is 0-1 keV
    return out


# ---------------------------------------------------------------- report
def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--inputs", default="inputs")
    ap.add_argument("--guaraci", default="vial_results")
    ap.add_argument("--gate", default="gate_output")
    ap.add_argument("--gate-events", type=int)
    ap.add_argument("--mcnp", default="outp")
    a = ap.parse_args()

    betas = betas_per_decay(a.inputs)
    per_decay = betas if betas is not None else photons_per_decay(a.inputs)
    particle = "electron" if betas is not None else "photon"
    codes = {}
    if os.path.isdir(a.guaraci):
        codes["Guaraci"] = guaraci(a.guaraci, per_decay)
    if os.path.isdir(a.gate):
        codes["GATE 10"] = gate(a.gate, a.gate_events)
    if os.path.isfile(a.mcnp):
        codes["MCNPX"] = mcnp(a.mcnp)
    if not codes:
        raise SystemExit("no results found")
    names = list(codes)
    ref = names[0]
    rows = []

    print(f"{particle.capitalize()}s per decay in the source: {per_decay:.5f}; results per emitted {particle}.\n")
    print(f"Energy deposited per emitted {particle} (keV); ± = 1σ in %; Δ relative to {ref}")
    for q, label in [("edep_water_keV", "water"), ("edep_vial_keV", "glass"), ("edep_cavity_keV", "air above water")]:
        cells = []
        base = codes[ref].get(q, (None,))[0]
        for n in names:
            v = codes[n].get(q)
            if v is None:
                cells.append(f"{n}: —")
                continue
            d = f" Δ {100 * (v[0] - base) / base:+.2f} %" if base and n != ref else ""
            u = f" ± {v[1]:.2f} %" if not math.isnan(v[1]) else ""
            cells.append(f"{n}: {v[0]:.4f}{u}{d}")
            rows.append({"quantity": q, "code": n, "value": v[0], "uncertainty_percent": v[1]})
        print(f"  {label:<16} " + "   ".join(cells))

    # dose profile along the axis
    profs = {n: c["z_profile"] for n, c in codes.items() if "z_profile" in c}
    if profs:
        print(f"\nDose near the axis (mean within r = 4 mm, Gy per {particle}), every 3 mm, z from the bottom of the vial:")
        print("  z (mm)  " + "  ".join(f"{n:>12}" for n in profs))
        n_z = min(len(p) for p in profs.values())
        for k in range(1, n_z, 3):
            print(f"  {k + 0.5:6.1f}  " + "  ".join(f"{profs[n][k]:12.4e}" for n in profs))
            for n in profs:
                rows.append({"quantity": f"dose_axis_z{k + 0.5}mm", "code": n, "value": profs[n][k], "uncertainty_percent": ""})

    # spectrum
    specs = {n: c["spectrum"] for n, c in codes.items() if "spectrum" in c}
    if specs:
        # main line of the source: the bin of the most intense photon line
        lines = [l.split(",") for l in open(os.path.join(a.inputs, "lines.csv")) if l[0].isdigit()]
        main_kev = int(max(lines, key=lambda r: float(r[1]))[0].split(".")[0])
        print("\nPhoton fluence in the water (cm^-2 per photon):")
        print("  " + "  ".join(f"{n:>12}" for n in specs)
              + f"   (total, {main_kev}-{main_kev + 1} keV peak, 1-{main_kev} keV scattered)")
        for n, s in specs.items():
            total = sum(s)
            peak = s[main_kev] if len(s) > main_kev else float("nan")
            scattered = sum(s[:main_kev])
            print(f"  {n:<10} {total:12.4e} {peak:12.4e} {scattered:12.4e}")
            rows += [
                {"quantity": "fluence_total", "code": n, "value": total, "uncertainty_percent": ""},
                {"quantity": f"fluence_{main_kev}keV_bin", "code": n, "value": peak, "uncertainty_percent": ""},
            ]

    with open("comparison.csv", "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=["quantity", "code", "value", "uncertainty_percent"])
        w.writeheader()
        w.writerows(rows)
    print("\nWritten comparison.csv")


if __name__ == "__main__":
    main()
