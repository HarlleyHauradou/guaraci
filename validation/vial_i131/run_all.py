#!/usr/bin/env python3
"""Run the vial benchmark with every code on this machine and time each run.

    python3 run_all.py                                  # inputs/ -> results/
    python3 run_all.py --inputs inputs_y90 --results results_y90
    python3 run_all.py --codes guaraci,gate --threads 1,16 --histories 1e6

For each code and each thread count:

- Guaraci and GATE 10 run with that many threads;
- EGSnrc (DOSRZnrc) and FLUKA run that many independent processes, each with
  its own random seed and 1/N of the histories (they are single-threaded);
- the wall-clock time from start to the end of the last process is measured
  the same way for every code (compilation of the FLUKA source routine is not
  included).

Writes results/machine.txt (CPU, memory, GPU, code versions), results/timing.csv
and one folder per run, e.g. results/gate_t16/. Then run compare.py.

Needs: `guaraci` in the PATH (or --guaraci); the Python of the opengate
environment (--gate-python, default: the Python running this script); EGSnrc
with EGS_HOME set and `dosrznrc` compiled; FLUKA 4 with FLUPRO set. MCNPX is
not run here: it is outside the cross-validation.
"""

import argparse
import csv
import os
import platform
import re
import shutil
import subprocess
import sys
import time


def sh(cmd, **kw):
    try:
        return subprocess.run(cmd, shell=isinstance(cmd, str), capture_output=True, text=True, **kw).stdout.strip()
    except Exception as e:
        return f"unavailable ({e})"


def machine_info(a):
    cpu = re.search(r"Model name:\s*(.*)", sh("lscpu"))
    mem = re.search(r"MemTotal:\s*(\d+)", open("/proc/meminfo").read()) if os.path.exists("/proc/meminfo") else None
    lines = [
        f"date              {time.strftime('%Y-%m-%d %H:%M:%S %z')}",
        f"system            {platform.platform()}",
        f"CPU               {cpu.group(1).strip() if cpu else platform.processor()}",
        f"logical CPUs      {os.cpu_count()}",
        f"memory            {int(mem.group(1)) / 1024 ** 2:.1f} GiB" if mem else "memory            unknown",
        f"GPU               {sh('nvidia-smi --query-gpu=name,memory.total,driver_version --format=csv,noheader') or 'none found'}",
        f"Guaraci           {sh([a.guaraci, '--version'])}",
        f"GATE (opengate)   {sh([a.gate_python, '-c', 'import opengate; print(opengate.__version__)'])}",
        f"EGSnrc            EGS_HOME={os.environ.get('EGS_HOME', 'not set')} HEN_HOUSE={os.environ.get('HEN_HOUSE', 'not set')}",
        f"FLUKA             FLUPRO={os.environ.get('FLUPRO', 'not set')}",
        "note              no code uses the GPU in this benchmark",
    ]
    return "\n".join(lines) + "\n"


def histories_of(inputs):
    m = re.search(r"^histories = (\S+)", open(os.path.join(inputs, "vial.gci")).read(), re.M)
    return int(float(m.group(1)))


def run_parallel(jobs):
    """Start every (cmd, cwd, env) job at once; wait for all; return the wall time."""
    t0 = time.perf_counter()
    procs = []
    for cmd, cwd, env, log in jobs:
        f = open(log, "w")
        procs.append((subprocess.Popen(cmd, cwd=cwd, env=env, stdout=f, stderr=subprocess.STDOUT), f, log))
    failed = []
    for p, f, log in procs:
        if p.wait() != 0:
            failed.append(log)
        f.close()
    wall = time.perf_counter() - t0
    if failed:
        print(f"    FAILED: see {', '.join(failed)}")
    return wall, not failed


# ------------------------------------------------------------------ codes
def guaraci(a, n, threads, out):
    os.makedirs(out, exist_ok=True)
    cmd = [a.guaraci, "run", os.path.join(a.inputs, "vial"), "-o", out, "--threads", str(threads), "--histories", str(n)]
    return run_parallel([(cmd, None, os.environ.copy(), os.path.join(out, "run.log"))])


def gate(a, n, threads, out):
    os.makedirs(out, exist_ok=True)
    env = dict(os.environ, GATE_THREADS=str(threads), GATE_OUTPUT=os.path.abspath(out), GATE_HISTORIES=str(n))
    cmd = [a.gate_python, os.path.join(a.inputs, "gate10_vial.py")]
    return run_parallel([(cmd, None, env, os.path.join(out, "run.log"))])


def egsnrc(a, n, procs, out):
    home = os.environ.get("EGS_HOME")
    if not home:
        print("    EGS_HOME is not set: skipping EGSnrc")
        return None, False
    work = os.path.join(home, "dosrznrc")
    os.makedirs(out, exist_ok=True)
    text = open(os.path.join(a.inputs, "egsnrc_vial.egsinp")).read()
    jobs = []
    tag = os.path.basename(os.path.abspath(out))
    for k in range(procs):
        name = f"guaraci_vial_{tag}_{k}"
        t = re.sub(r"NUMBER OF HISTORIES= \S+", f"NUMBER OF HISTORIES= {max(100, n // procs)}", text)
        t = re.sub(r"INITIAL RANDOM NO. SEEDS= .*", f"INITIAL RANDOM NO. SEEDS= {33 + k}, {97 + 13 * k}", t)
        with open(os.path.join(work, name + ".egsinp"), "w") as f:
            f.write(t)
        jobs.append((["dosrznrc", "-i", name], work, os.environ.copy(), os.path.join(out, f"run_{k}.log")))
    wall, ok = run_parallel(jobs)
    for k in range(procs):
        name = f"guaraci_vial_{tag}_{k}"
        for ext in (".egslst", ".egsinp"):
            src = os.path.join(work, name + ext)
            if os.path.exists(src):
                shutil.copy(src, os.path.join(out, name + ext))
    return wall, ok


def fluka_build(a):
    """Compile the source routine and link a FLUKA executable (not timed)."""
    flupro = os.environ.get("FLUPRO")
    if a.fluka_exe:
        return os.path.abspath(a.fluka_exe)
    if not flupro:
        print("    FLUPRO is not set: skipping FLUKA")
        return None
    build = os.path.abspath(os.path.join(a.results, "fluka_build"))
    os.makedirs(build, exist_ok=True)
    shutil.copy(os.path.join(a.inputs, "source.f"), build)
    tool = lambda t: next((p for p in (os.path.join(flupro, "bin", t), os.path.join(flupro, "flutil", t)) if os.path.exists(p)), t)
    for cmd in ([tool("fff"), "source.f"], [tool("lfluka"), "-m", "fluka", "-o", "flukavial", "source.o"]):
        p = subprocess.run(cmd, cwd=build, capture_output=True, text=True)
        if p.returncode != 0:
            print(f"    FLUKA build failed: {' '.join(cmd)}\n{p.stdout}{p.stderr}")
            return None
    return os.path.join(build, "flukavial")


def fluka(a, n, procs, out, exe):
    flupro = os.environ.get("FLUPRO")
    if not exe or not flupro:
        return None, False
    rfluka = next((p for p in (os.path.join(flupro, "bin", "rfluka"), os.path.join(flupro, "flutil", "rfluka")) if os.path.exists(p)), "rfluka")
    text = open(os.path.join(a.inputs, "fluka_vial.inp")).read()
    jobs = []
    for k in range(procs):
        d = os.path.abspath(os.path.join(out, f"run_{k}"))
        os.makedirs(d, exist_ok=True)
        t = re.sub(r"^RANDOMIZ .*$", f"RANDOMIZ  {1.0:>10}{float(k + 1):>10}", text, flags=re.M)
        t = re.sub(r"^START .*$", f"START     {float(max(1, n // procs)):>10}", t, flags=re.M)
        with open(os.path.join(d, "fluka_vial.inp"), "w") as f:
            f.write(t)
        jobs.append(([rfluka, "-e", exe, "-N0", "-M1", "fluka_vial"], d, os.environ.copy(), os.path.join(d, "run.log")))
    return run_parallel(jobs)


# ------------------------------------------------------------------ main
def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--inputs", default="inputs")
    ap.add_argument("--results", default="results")
    ap.add_argument("--codes", default="guaraci,gate,egsnrc,fluka")
    ap.add_argument("--threads", default=f"1,{os.cpu_count()}", help="thread / process counts, e.g. 1,16")
    ap.add_argument("--histories", type=float, help="histories of every run (default: those of the inputs)")
    ap.add_argument("--guaraci", default=os.environ.get("GUARACI", "guaraci"))
    ap.add_argument("--gate-python", default=sys.executable, help="Python of the opengate environment")
    ap.add_argument("--fluka-exe", help="an already linked FLUKA executable with source.f")
    a = ap.parse_args()

    n = int(a.histories) if a.histories else histories_of(a.inputs)
    codes = [c.strip() for c in a.codes.split(",") if c.strip()]
    counts = [int(t) for t in a.threads.split(",")]
    os.makedirs(a.results, exist_ok=True)
    info = machine_info(a)
    with open(os.path.join(a.results, "machine.txt"), "w") as f:
        f.write(info)
    print(info)

    timing_path = os.path.join(a.results, "timing.csv")
    new = not os.path.exists(timing_path)
    exe = fluka_build(a) if "fluka" in codes else None
    with open(timing_path, "a", newline="") as f:
        w = csv.writer(f)
        if new:
            w.writerow(["code", "threads", "histories", "wall_s", "histories_per_s", "folder", "ok"])
        for code in codes:
            for t in counts:
                out = os.path.join(a.results, f"{code}_t{t}")
                print(f"{code}: {n:.3g} histories, {t} thread(s)/process(es) -> {out}")
                if code == "guaraci":
                    wall, ok = guaraci(a, n, t, out)
                elif code == "gate":
                    wall, ok = gate(a, n, t, out)
                elif code == "egsnrc":
                    wall, ok = egsnrc(a, n, t, out)
                elif code == "fluka":
                    wall, ok = fluka(a, n, t, out, exe)
                else:
                    print(f"    unknown code `{code}` (guaraci, gate, egsnrc, fluka)")
                    continue
                if wall is None:
                    continue
                print(f"    {wall:.1f} s, {n / wall:.4g} histories/s")
                w.writerow([code, t, n, f"{wall:.3f}", f"{n / wall:.6g}", out, ok])
                f.flush()
    print(f"\nTimes in {timing_path}. Compare: python3 compare.py --inputs {a.inputs} --results {a.results}")


if __name__ == "__main__":
    main()
