#!/usr/bin/env python3
"""Compare the vial benchmark results: Guaraci, GATE 10, EGSnrc and FLUKA.

    python3 compare.py --inputs inputs --results results

`results` is the folder written by run_all.py; for each code the run with the
most threads/processes is used. Single folders can also be given:
--guaraci DIR, --gate DIR, --egsnrc DIR, --fluka DIR. MCNPX is outside the
cross-validation; with --mcnp OUTP its results are shown apart, as an extra.

Every result is per emitted particle: per photon in the photon mode, per β⁻
electron in the beta mode (detected from beta.csv in the inputs folder).

Writes comparison.csv, and comparison.png if matplotlib is installed.
"""

import argparse
import csv
import glob
import math
import os
import re
import struct

from make_inputs import CAV_H, CAV_R, MATERIALS, VIAL_H, VIAL_R, VOXEL, WAT_H, WAT_R, WAT_Z

MEV_J = 1.602176634e-13
KEV_PER_GY_KG = 1.0 / 1.602176634e-16           # keV in 1 J
WATER_MASS = MATERIALS["water"][0] * math.pi * WAT_R ** 2 * WAT_H
GLASS_MASS = MATERIALS["glass"][0] * math.pi * (VIAL_R ** 2 * VIAL_H - CAV_R ** 2 * CAV_H)
NZ = round(VIAL_H / VOXEL)
Z0 = -VIAL_H / 2
PROFILE_R = 0.4
# 1 mm slices of the vial filled with water (the axial profile is compared there:
# the air voxels above it hold too little mass for a meaningful dose)
WATER_SLICES = range(round((WAT_Z - WAT_H / 2 - Z0) / VOXEL), round((WAT_Z + WAT_H / 2 - Z0) / VOXEL))


def density(r, z):
    """Density (g/cm3) at radius r and height z (cm), from the geometry."""
    if r > VIAL_R or abs(z) > VIAL_H / 2:
        return MATERIALS["air"][0]
    if r > CAV_R or abs(z) > CAV_H / 2:
        return MATERIALS["glass"][0]
    if abs(z - WAT_Z) <= WAT_H / 2:
        return MATERIALS["water"][0]
    return MATERIALS["air"][0]


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


def ring_weights(dims, radius=PROFILE_R, sub=20):
    """Fraction of the area of each voxel column inside r < radius (sampled on a
    sub x sub grid), so that the voxel codes average the same cylinder as the
    R-Z codes."""
    nx, ny, _ = dims
    w = {}
    for i in range(nx):
        for j in range(ny):
            inside = 0
            for a in range(sub):
                for b in range(sub):
                    x = (i + (a + 0.5) / sub) * VOXEL - nx * VOXEL / 2
                    y = (j + (b + 0.5) / sub) * VOXEL - ny * VOXEL / 2
                    inside += x * x + y * y < radius * radius
            if inside:
                w[(i, j)] = inside / sub ** 2
    return w


def z_profile(dims, data):
    """Mean dose within r < 4 mm of the axis, per 1 mm slice."""
    nx, ny, nz = dims
    w = ring_weights(dims)
    total = sum(w.values())
    return [sum(f * data[i + nx * (j + ny * k)] for (i, j), f in w.items()) / total for k in range(nz)]


def particles_per_decay(inputs):
    path = os.path.join(inputs, "beta.csv")
    if os.path.exists(path):
        m = re.search(r"beta_minus_per_decay = ([\d.eE+-]+)", open(path).read())
        return (float(m.group(1)) if m else 1.0), "electron"
    rows = [l for l in open(os.path.join(inputs, "lines.csv")) if l[0].isdigit()]
    return sum(float(r.split(",")[1]) for r in rows), "photon"


def guaraci(path, per_decay):
    out = {}
    with open(os.path.join(path, "edep.csv")) as f:
        for r in csv.DictReader(f):
            e = float(r["energy_MeV_per_decay"]) / per_decay * 1e3
            name = {"vial": "glass"}.get(r["volume"], r["volume"])
            out[f"edep_{name}_keV"] = (e, float(r["uncertainty_percent"]))
    dims, data = read_mhd(os.path.join(path, "dose.mhd"))   # Gy per decay
    out["z_profile"] = [v / per_decay for v in z_profile(dims, data)]
    spec = os.path.join(path, "spectrum.csv")
    if os.path.exists(spec):
        out["spectrum"] = [float(r["fluence"]) / per_decay for r in csv.DictReader(open(spec))]
    return out


def gate_events(path, inputs, override=None):
    if override:
        return override
    ev = os.path.join(path, "events.txt")
    if os.path.exists(ev):
        return int(open(ev).read().split()[0])
    st = os.path.join(path, "stats.json")
    if os.path.exists(st):
        m = re.search(r'"events"\s*:\s*\{?[^0-9]*(\d+)', open(st).read())
        if m:
            return int(m.group(1))
    # last resort: src.n (primaries per thread) x threads of the generated script
    text = open(os.path.join(inputs, "gate10_vial.py")).read()
    n = re.search(r'GATE_HISTORIES", "(\d+)"', text)
    return int(n.group(1)) if n else None


def first(path, pattern):
    hits = [p for p in glob.glob(os.path.join(path, pattern)) if "uncertainty" not in p and "squared" not in p]
    return sorted(hits)[0] if hits else None


def gate(path, inputs, events=None):
    out = {}
    events = gate_events(path, inputs, events)
    if not events:
        print("GATE: number of events unknown; give it with --gate-events")
        return out
    total = {}
    for name in ("water", "vial", "cavity"):
        f = first(path, f"{name}*edep*.mhd")
        if f:
            total[name] = sum(read_mhd(f)[1]) / events * 1e3      # keV per primary
    if "water" in total:
        out["edep_water_keV"] = (total["water"], float("nan"))
    if "vial" in total and "cavity" in total:
        out["edep_glass_keV"] = (total["vial"] - total["cavity"], float("nan"))
    f = first(path, "dose_grid*edep*.mhd")
    if f:
        # dose from the deposited energy and the densities of the geometry, so
        # that every code is converted to dose the same way
        dims, data = read_mhd(f)
        nx, ny, nz = dims
        mass = [[[density(math.hypot((i + 0.5) * VOXEL - VIAL_R, (j + 0.5) * VOXEL - VIAL_R), Z0 + (k + 0.5) * VOXEL)
                  * VOXEL ** 3 * 1e-3 for i in range(nx)] for j in range(ny)] for k in range(nz)]
        dose = [data[i + nx * (j + ny * k)] * MEV_J / mass[k][j][i] / events
                for k in range(nz) for j in range(ny) for i in range(nx)]
        out["z_profile"] = z_profile(dims, dose)
        out["grid_total_keV"] = sum(data) / events * 1e3
    return out


def rz_results(edep_kev, label):
    """From the energy (keV per primary) of an R-Z grid on the vial — rings
    [0, 0.4), [0.4, 1.0), [1.0, 1.1) cm or 0.1 cm bins, 45 slabs of 1 mm —
    compute water, glass and the axial profile (dose r < 0.4 cm, Gy)."""
    out = {}
    water = glass = 0.0
    prof_e = [0.0] * NZ
    prof_m = [0.0] * NZ
    for (r_lo, r_hi, k), e in edep_kev.items():
        z = Z0 + (k + 0.5) * VOXEL
        rho = density((r_lo + r_hi) / 2, z)
        if rho == MATERIALS["water"][0]:
            water += e
        elif rho == MATERIALS["glass"][0]:
            glass += e
        if r_hi <= PROFILE_R + 1e-9:
            prof_e[k] += e
            prof_m[k] += rho * math.pi * (r_hi ** 2 - r_lo ** 2) * VOXEL * 1e-3
    out["edep_water_keV"] = (water, float("nan"))
    out["edep_glass_keV"] = (glass, float("nan"))
    out["z_profile"] = [e / KEV_PER_GY_KG / m if m else 0.0 for e, m in zip(prof_e, prof_m)]
    return out


def egsnrc(path):
    """DOSRZnrc listings (.egslst): dose (Gy per incident particle) of each
    region, from the dose summary. All the listings of the folder are averaged."""
    radii = [0.0, PROFILE_R, CAV_R, VIAL_R]
    acc, runs = {}, 0
    # "  Z#  P#  C#   T(Gy/particle) +/- x%   T-S ...": zone, planar slab, ring, total dose
    row = re.compile(r"^\s*\d+\s+(\d+)\s+(\d+)\s+(\d\.\d+E[-+]\d+)\s*\+/-\s*([\d.]+)\s*%")
    for lst in sorted(glob.glob(os.path.join(path, "*.egslst"))):
        text = open(lst, errors="replace").read()
        found = 0
        for line in text.splitlines():
            m = row.match(line)
            if not m:
                continue
            iz, ix, dose = int(m.group(1)), int(m.group(2)), float(m.group(3))
            if 2 <= iz <= NZ + 1 and 1 <= ix <= 3:
                k = iz - 2
                r_lo, r_hi = radii[ix - 1], radii[ix]
                mass = density((r_lo + r_hi) / 2, Z0 + (k + 0.5) * VOXEL) * math.pi * (r_hi ** 2 - r_lo ** 2) * VOXEL * 1e-3
                acc[(r_lo, r_hi, k)] = acc.get((r_lo, r_hi, k), 0.0) + dose * mass * KEV_PER_GY_KG
                found += 1
        if found:
            runs += 1
        else:
            print(f"EGSnrc: no dose table found in {lst}; please send this file")
    if not runs:
        return {}
    return rz_results({k: v / runs for k, v in acc.items()}, "EGSnrc")


def fluka_numbers(text):
    return [float(x) for x in re.findall(r"-?\d\.\d+E[-+]\d+", text)]


def fluka(path):
    """FLUKA formatted outputs of every run_k: USRBIN (unit 21, GeV/cm3 per
    primary on the R-Z grid) and USRTRACK (unit 22, photons / GeV / cm2 per primary)."""
    nr = round(VIAL_R / VOXEL)
    acc, spec, runs = {}, None, 0
    for d in sorted(glob.glob(os.path.join(path, "run_*"))):
        f21 = sorted(glob.glob(os.path.join(d, "*_fort.21")))
        if not f21:
            continue
        text = open(f21[0]).read()
        # the data follow the last header line
        body = text.split("accurate deposition")[-1] if "accurate deposition" in text else text
        vals = fluka_numbers(body)[-nr * NZ:]
        if len(vals) != nr * NZ:
            print(f"FLUKA: could not read the R-Z grid of {f21[0]}; please send this file")
            continue
        runs += 1
        for k in range(NZ):
            for i in range(nr):
                r_lo, r_hi = i * VOXEL, (i + 1) * VOXEL
                vol = math.pi * (r_hi ** 2 - r_lo ** 2) * VOXEL
                key = (r_lo, r_hi, k)
                acc[key] = acc.get(key, 0.0) + vals[i + nr * k] * vol * 1e6        # GeV -> keV
        f22 = sorted(glob.glob(os.path.join(d, "*_fort.22")))
        if f22:
            t = open(f22[0]).read()
            part = t.split("Flux (Part/GeV/cmq/pr)")[-1] if "Flux (Part/GeV/cmq/pr)" in t else t
            v = fluka_numbers(part)[:700]
            if len(v) == 700:
                v = [x * 1e-6 for x in v]       # per GeV -> per 1 keV bin
                spec = v if spec is None else [a + b for a, b in zip(spec, v)]
    if not runs:
        return {}
    out = rz_results({k: v / runs for k, v in acc.items()}, "FLUKA")
    if spec:
        out["spectrum"] = [x / runs for x in spec]
    return out


def mcnp(path):
    """Extra, outside the cross-validation: F6 / *F8 tallies (cells 1 and 3) and F4."""
    text = open(path).read()
    out = {}
    for b in re.split(r"\n1tally\s+", text):
        if re.match(r"6\s", b) or re.match(r"8\s", b):
            for cell, val, err in re.findall(r"cell\s+(\d+)\s*\n\s*([\d.E+-]+)\s+([\d.]+)", b):
                name = "water" if cell == "1" else "glass"
                mass = 1.0 if b.startswith("8") else (WATER_MASS if cell == "1" else GLASS_MASS)
                out[f"edep_{name}_keV"] = (float(val) * mass * 1e3, float(err) * 100)
        if re.match(r"4\s", b):
            rows = re.findall(r"^\s+(\d\.\d+E[+-]\d+)\s+([\d.]+E[+-]\d+)\s+([\d.]+)\s*$", b, re.M)
            out["spectrum"] = [float(v) for _, v, _ in rows]   # bin k: (k, k+1] keV
    return out


# ---------------------------------------------------------------- discovery
def latest_runs(results):
    """For each code, the folder of the run with the most threads/processes."""
    best = {}
    timing = os.path.join(results, "timing.csv")
    if os.path.exists(timing):
        for r in csv.DictReader(open(timing)):
            if r.get("ok", "True") != "True":
                continue
            if r["code"] not in best or int(r["threads"]) >= int(best[r["code"]]["threads"]):
                best[r["code"]] = r
    return {k: v["folder"] for k, v in best.items()}


# ---------------------------------------------------------------- report
def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--inputs", default="inputs")
    ap.add_argument("--results", default="results")
    ap.add_argument("--guaraci")
    ap.add_argument("--gate")
    ap.add_argument("--gate-events", type=int)
    ap.add_argument("--egsnrc")
    ap.add_argument("--fluka")
    ap.add_argument("--mcnp", help="MCNPX output (extra, outside the cross-validation)")
    a = ap.parse_args()

    per_decay, particle = particles_per_decay(a.inputs)
    runs = latest_runs(a.results) if os.path.isdir(a.results) else {}
    folders = {
        "Guaraci": a.guaraci or runs.get("guaraci"),
        "GATE 10": a.gate or runs.get("gate"),
        "EGSnrc": a.egsnrc or runs.get("egsnrc"),
        "FLUKA": a.fluka or runs.get("fluka"),
    }
    codes = {}
    for name, folder in folders.items():
        if not folder or not os.path.exists(folder):
            continue
        r = {"Guaraci": lambda: guaraci(folder, per_decay), "GATE 10": lambda: gate(folder, a.inputs, a.gate_events),
             "EGSnrc": lambda: egsnrc(folder), "FLUKA": lambda: fluka(folder)}[name]()
        if r:
            codes[name] = r
    extra = {"MCNPX (extra)": mcnp(a.mcnp)} if a.mcnp and os.path.isfile(a.mcnp) else {}
    if not codes:
        raise SystemExit("no results found (run run_all.py first, or give the folders)")
    rows = []
    names = list(codes)

    print(f"{particle.capitalize()}s per decay in the source: {per_decay:.5f}; results per emitted {particle}.")
    print("Δ: deviation from the mean of the other codes of the cross-validation.\n")
    print(f"Energy deposited per emitted {particle} (keV); ± = 1σ in %")
    for q, label in [("edep_water_keV", "water"), ("edep_glass_keV", "glass")]:
        print(f"  {label}")
        vals = {n: codes[n][q][0] for n in names if q in codes[n]}
        for n in names + list(extra):
            v = (codes.get(n) or extra.get(n) or {}).get(q)
            if v is None:
                continue
            others = [x for m, x in vals.items() if m != n]
            ref = sum(others) / len(others) if others else None
            d = f"   Δ {100 * (v[0] - ref) / ref:+.2f} %" if ref else ""
            u = f" ± {v[1]:.2f} %" if not math.isnan(v[1]) else ""
            print(f"    {n:<16} {v[0]:10.4f}{u}{d}")
            rows.append({"quantity": q, "code": n, "value": v[0], "uncertainty_percent": v[1]})
    for n, c in codes.items():
        if "grid_total_keV" in c and "edep_water_keV" in c and "edep_glass_keV" in c:
            s = c["edep_water_keV"][0] + c["edep_glass_keV"][0]
            print(f"  check {n}: energy of the dose grid {c['grid_total_keV']:.4f} keV, water + glass {s:.4f} keV"
                  f" ({100 * (c['grid_total_keV'] - s) / s:+.2f} %)")

    profs = {n: {k: c["z_profile"][k] for k in WATER_SLICES} for n, c in codes.items() if "z_profile" in c}
    if profs:
        print(f"\nDose in the water near the axis (r < 4 mm), Gy per {particle}, every 3 mm from the bottom of the vial:")
        print("  z (mm)  " + "  ".join(f"{n:>12}" for n in profs))
        for k in list(WATER_SLICES)[::3]:
            print(f"  {k + 0.5:6.1f}  " + "  ".join(f"{profs[n][k]:12.4e}" for n in profs))
        for n, p in profs.items():
            for k, v in p.items():
                rows.append({"quantity": f"dose_axis_z{k + 0.5}mm", "code": n, "value": v, "uncertainty_percent": ""})

    specs = {n: c["spectrum"] for n, c in list(codes.items()) + list(extra.items()) if c.get("spectrum")}
    main_kev = None
    if specs and particle == "photon":
        lines = [l.split(",") for l in open(os.path.join(a.inputs, "lines.csv")) if l[0].isdigit()]
        main_kev = int(float(max(lines, key=lambda r: float(r[1]))[0]))
        print(f"\nPhoton fluence in the water (cm^-2 per photon): total, {main_kev}-{main_kev + 1} keV bin, below it")
        for n, s in specs.items():
            total, peak, below = sum(s), s[main_kev] if len(s) > main_kev else float("nan"), sum(s[:main_kev])
            print(f"  {n:<16} {total:12.4e} {peak:12.4e} {below:12.4e}")
            rows += [{"quantity": "fluence_total", "code": n, "value": total, "uncertainty_percent": ""},
                     {"quantity": f"fluence_{main_kev}keV_bin", "code": n, "value": peak, "uncertainty_percent": ""}]

    timing = []
    tpath = os.path.join(a.results, "timing.csv")
    if os.path.exists(tpath):
        timing = [r for r in csv.DictReader(open(tpath)) if r.get("ok", "True") == "True"]
        print("\nSpeed (wall clock, this machine; see machine.txt)")
        print(f"  {'code':<10} {'threads':>7} {'histories':>11} {'time (s)':>10} {'histories/s':>12}")
        for r in timing:
            print(f"  {r['code']:<10} {r['threads']:>7} {float(r['histories']):>11.3g} {float(r['wall_s']):>10.1f} {float(r['histories_per_s']):>12.4g}")

    with open("comparison.csv", "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=["quantity", "code", "value", "uncertainty_percent"])
        w.writeheader()
        w.writerows(rows)
    print("\nWritten comparison.csv")
    plot(codes, extra, profs, specs, main_kev, timing, particle)


def plot(codes, extra, profs, specs, main_kev, timing, particle):
    try:
        import matplotlib
        matplotlib.use("Agg")
        import matplotlib.pyplot as plt
    except ImportError:
        print("matplotlib is not installed: no comparison.png (pip install matplotlib)")
        return
    fig, ax = plt.subplots(2, 2, figsize=(12, 9))
    names = list(codes)
    # 1. energy in water and glass
    a = ax[0][0]
    width = 0.8 / max(1, len(names))
    for i, n in enumerate(names):
        vals = [codes[n].get(q, (float("nan"),))[0] for q in ("edep_water_keV", "edep_glass_keV")]
        a.bar([x + i * width for x in range(2)], vals, width, label=n)
    a.set_xticks([x + width * (len(names) - 1) / 2 for x in range(2)], ["water", "glass"])
    a.set_ylabel(f"keV per emitted {particle}")
    a.set_title("Energy deposited")
    a.legend()
    # 2. axial dose profile
    a = ax[0][1]
    for n, p in profs.items():
        a.plot([k + 0.5 for k in p], list(p.values()), "-o", ms=3, label=n)
    a.set_xlabel("z from the bottom of the vial (mm)")
    a.set_ylabel(f"dose, r < 4 mm (Gy per {particle})")
    a.set_title("Dose along the axis, in the water")
    a.legend()
    # 3. photon fluence spectrum
    a = ax[1][0]
    if specs:
        for n, s in specs.items():
            a.step(range(len(s)), s, where="post", label=n)
        a.set_yscale("log")
        a.set_xlabel("photon energy (keV)")
        a.set_ylabel(f"fluence per 1 keV (cm⁻² per {particle})")
        a.set_title("Photon fluence in the water")
        a.legend()
    else:
        a.axis("off")
    # 4. speed
    a = ax[1][1]
    if timing:
        labels = [f"{r['code']}\n{r['threads']} thr." for r in timing]
        a.bar(range(len(timing)), [float(r["histories_per_s"]) for r in timing])
        a.set_xticks(range(len(timing)), labels, fontsize=8)
        a.set_yscale("log")
        a.set_ylabel("histories per second")
        a.set_title("Speed on this machine")
    else:
        a.axis("off")
    fig.tight_layout()
    fig.savefig("comparison.png", dpi=130)
    print("Written comparison.png")


if __name__ == "__main__":
    main()
