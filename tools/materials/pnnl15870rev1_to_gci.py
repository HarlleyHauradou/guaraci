"""Build data/materials.gci from the text of PNNL-15870 Rev. 1 (pdftotext -layout)."""
import re, sys, json
from collections import Counter

text_path, names_path, out_path = sys.argv[1:4]
L = open(text_path, encoding='utf-8').read().replace('\f', '\n').split('\n')
footer = re.compile(r'^\s*(Page \d+ of \d+|PIET-43741-TM-963|PNNL-15870 Rev\. 1)\s*$')
L = [l for l in L if not footer.match(l)]

head = re.compile(r'^(\d{1,3})\s+(\S.*?)\s*$')
row = re.compile(r'^\s*([A-Z][a-z]?)(?:-\d+m?)?\s+(?:\d+|-)\s+(\d+)\s+([\d.Ee+-]+)\s+([\d.Ee+-]+)\s+([\d.Ee+-]+)\s*$')

def next_nonempty(i):
    i += 1
    while i < len(L) and not L[i].strip():
        i += 1
    return i

starts = []
for i, l in enumerate(L):
    m = head.match(l)
    if m and any(L[j].startswith('Formula =') for j in range(i + 1, min(i + 4, len(L)))):
        starts.append((i, int(m.group(1)), m.group(2)))

mats = []
for k, (i, num, name) in enumerate(starts):
    end = starts[k + 1][0] if k + 1 < len(starts) else len(L)
    block = L[i:end]
    density = float(re.search(r'Density \(g/cm3\) =\s+([\d.Ee+-]+)', '\n'.join(block)).group(1))
    elems = {}
    iso = False
    in_table = False
    comments = []
    in_comm = False
    for l in block:
        if 'Element' in l and 'Neutron ZA' in l:
            in_table = True; continue
        if in_table:
            if l.strip().startswith('Total'):
                in_table = False; continue
            m = row.match(l)
            if m:
                el = m.group(1)
                iso = iso or bool(re.match(r'^\s*[A-Z][a-z]?-\d', l))
                elems[el] = elems.get(el, 0.0) + float(m.group(3))
        if l.startswith('Comments and References'):
            in_comm = True; continue
        if in_comm and l.strip():
            comments.append(l.strip())
    elems = {el: w for el, w in elems.items() if w > 0}  # e.g. Am listed with fraction 0
    assert elems, (num, name)
    mats.append(dict(num=num, name=name,
                     density=density, elems=elems, iso=iso, notes=' '.join(comments)))

# Alternate names from the index: "Trans-stilbene (see Stilbene)"
alias_re = re.compile(r'^(\S.*?)\s+\(see (.+?)\)\s*$')
aliases = []
for l in L[:starts[0][0]]:
    m = alias_re.match(l.strip())
    if m:
        aliases.append((m.group(1), m.group(2)))

# ---- identifiers ----------------------------------------------------------
def camel(s):
    words = re.findall(r'[A-Za-z0-9]+', s)
    return ''.join(w[0].upper() + w[1:] for w in words)

def base_id(name):
    main = re.sub(r'\s*\(.*?\)', '', name)
    parts = [camel(p) for p in main.split(',') if camel(p)]
    return '_'.join(parts)

def paren(name):
    m = re.findall(r'\((.*?)\)', name)
    return camel(' '.join(m)) if m else ''

overrides = json.load(open(names_path))
ids = Counter(base_id(m['name']) for m in mats)
used = set()
for m in mats:
    if m['name'] in overrides:
        mid = overrides[m['name']]
    else:
        mid = base_id(m['name'])
        if ids[mid] > 1 and paren(m['name']):
            mid += '_' + paren(m['name'])
    if mid in used:
        mid = f"{mid}_{m['num']}"
    assert mid not in used, mid
    used.add(mid)
    m['id'] = mid

by_name = {m['name'].lower(): m for m in mats}
def find(target):
    t = target.lower()
    if t in by_name:
        return by_name[t]
    # "Explosive Compound, NC" -> match on the abbreviation in parentheses or a prefix
    cands = [m for m in mats if m['name'].lower().startswith(t)]
    if len(cands) == 1:
        return cands[0]
    abbr = t.split(',')[-1].strip()
    cands = [m for m in mats if re.search(r'\(' + re.escape(abbr) + r'\)', m['name'].lower())
             or m['name'].lower().endswith(', ' + abbr)]
    return cands[0] if len(cands) == 1 else None

alias_out = []
for a, t in aliases:
    m = find(t)
    aid = camel(re.sub(r'\s*\(.*?\)', '', a)) if ',' not in a else base_id(a)
    if not m or not aid or aid in used:
        print(f"skip alias {a!r} -> {t!r}", file=sys.stderr)
        continue
    used.add(aid)
    alias_out.append((aid, a, m['id']))
for aid, target in json.load(open(sys.argv[4])).items():
    m = by_name[target.lower()]
    if aid not in used:
        used.add(aid)
        alias_out.append((aid, aid, m['id']))
# Abbreviations in parentheses (BGO, LYSO, PMMA, ...) become short names too.
for m in mats:
    for ab in re.findall(r'\(([A-Za-z0-9-]+)\)', m['name']):
        ab_id = ab.replace('-', '')
        if ab_id.isupper() and len(ab_id) >= 2 and ab_id not in used:
            used.add(ab_id)
            alias_out.append((ab_id, ab, m['id']))

# ---- write ------------------------------------------------------------------
def q(s):
    return '"' + s.replace('\\', '\\\\').replace('"', '\\"') + '"'

def fmt(x):
    return f"{x:.6f}".rstrip('0').rstrip('.')

out = [
    "# Guaraci material database",
    "#",
    "# Source: R.J. McConn Jr, C.J. Gesh, R.T. Pagh, R.A. Rucker, R.G. Williams III,",
    "# \"Compendium of Material Composition Data for Radiation Transport Modeling\",",
    "# PNNL-15870 Rev. 1 (PIET-43741-TM-963), Pacific Northwest National Laboratory, March 2011.",
    "#",
    "# Compositions are mass fractions (Guaraci normalises them to sum to one).",
    "# `number` is the material number in the compendium.",
    "# Densities are typical values; when you know the real density, copy the material:",
    "#   [material MyFoam]",
    "#   same_as = Polystyrene",
    "#   density = 0.05 g/cm3",
    "",
]
for m in mats:
    out.append(f"[material {m['id']}]")
    out.append(f"title = {q(m['name'])}")
    out.append(f"number = {m['num']}")
    out.append(f"density = {m['density']:g} g/cm3")
    out.append("composition = { " + ', '.join(f"{el} = {fmt(w)}" for el, w in m['elems'].items()) + " }")
    notes = m['notes']
    if m['iso']:
        notes += " [Guaraci: isotopes summed by element; photon transport does not depend on the isotope.]"
    s = sum(m['elems'].values())
    if abs(s - 1) > 2e-3:
        notes += f" [Guaraci: fractions in the source sum to {s:.4f}; normalised.]"
    if notes.strip():
        out.append(f"reference = {q(notes.strip())}")
    out.append("")
out += ["# ---- abbreviations and other names used in the compendium ----", ""]
for aid, a, target in alias_out:
    out.append(f"[material {aid}]  # {a}")
    out.append(f"same_as = {target}")
    out.append("")
open(out_path, 'w', encoding='utf-8').write('\n'.join(out))
print(len(mats), "materials,", len(alias_out), "alternative names", file=sys.stderr)
for m in mats:
    print(f"{m['num']:3d} {m['id']:45s} {m['name']}")
for aid, a, t in alias_out:
    print(f"    {aid:30s} -> {t}   ({a})")
