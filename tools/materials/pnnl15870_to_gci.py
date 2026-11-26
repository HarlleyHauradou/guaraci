import re, sys, json
lines = open(sys.argv[1], encoding='utf-8').read().split('\n')
hdr = re.compile(r'^\s*Element\s+Weight Fraction')
row = re.compile(r'^\s*([A-Z][a-z]?)(?:-(\d+))?\s+([0-9.Ee+-]+)\s+([0-9.Ee+-]+)\s+([0-9.Ee+-]+)\s*$')
mats = []
for i, l in enumerate(lines):
    if not hdr.match(l):
        continue
    j = i - 1
    while lines[j].strip() == '':
        j -= 1
    name_lines = []
    while lines[j].strip() and not re.fullmatch(r'\s*\d+\s*', lines[j]):
        name_lines.insert(0, lines[j].strip()); j -= 1
    name = ' '.join(name_lines)
    k = i + 1; elems = {}; iso = False; density = None; comments = []; ref = []; mode = None
    while 'MCNP Form' not in lines[k]:
        s = lines[k]; m = row.match(s)
        if m and mode is None:
            el = m.group(1); iso = iso or bool(m.group(2))
            elems[el] = elems.get(el, 0.0) + float(m.group(3))
        elif 'Density' in s and '=' in s:
            density = float(s.split('=')[1].split()[0])
        elif s.strip().startswith('Comments:'):
            mode = 'c'; comments.append(s.strip()[9:].strip())
        elif s.strip().startswith('Reference:'):
            mode = 'r'; ref.append(s.strip()[10:].strip())
        elif mode == 'c': comments.append(s.strip())
        elif mode == 'r': ref.append(s.strip())
        k += 1
    mats.append(dict(name=name, elems=elems, iso=iso, density=density,
                     comment=' '.join(c for c in comments if c), ref=re.sub(r'\s+\d+$', '', ' '.join(r for r in ref if r))))

# group prefixes: "Concrete 1: Ordinary Concrete", then "2: Barite ..."
group = None
for m in mats:
    g = re.match(r'^(.*?)\s+1:\s*(.*)$', m['name'])
    n = re.match(r'^(\d+):\s*(.*)$', m['name'])
    if g:
        group = g.group(1); m['group'] = group; m['item'] = g.group(2)
    elif n and group:
        m['group'] = group; m['item'] = n.group(2)
    else:
        group = None; m['group'] = None; m['item'] = m['name']
    m['title'] = f"{m['group']}: {m['item']}" if m['group'] else m['item']

def camel(s):
    s = re.sub(r'\(.*?\)', ' ', s)
    words = re.findall(r'[A-Za-z0-9]+', s)
    return ''.join(w[0].upper() + w[1:] for w in words)

override = json.load(open(sys.argv[2]))
seen = set()
for m in mats:
    if m['group']:
        item = camel(m['item']); g = camel(m['group'])
        item = item.replace(g, '') or item
        auto = g + '_' + item
    else:
        auto = camel(m['item'])
    m['id'] = override.get(m['title'], auto)
    assert m['id'] not in seen, m['id']; seen.add(m['id'])

aliases = json.load(open(sys.argv[3]))
title_to_id = {m['title']: m['id'] for m in mats}
by_plain = {}
for m in mats:
    by_plain[m['item']] = m['id']; by_plain[m['title']] = m['id']

def q(s): return '"' + s.replace('\\', '\\\\').replace('"', '\\"') + '"'
def fmt(x): return f"{x:.6f}".rstrip('0').rstrip('.') if x < 1 else f"{x:g}"

out = []
out.append("# Guaraci material database")
out.append("#")
out.append("# Compositions are mass fractions; Guaraci normalises them to sum to one.")
out.append("# Source: R.G. Williams III, C.J. Gesh, R.T. Pagh, \"Compendium of Material")
out.append("# Composition Data for Radiation Transport Modeling\", PNNL-15870,")
out.append("# Pacific Northwest National Laboratory, April 2006.")
out.append("#")
out.append("# Densities are typical values; use the real density of your object when known:")
out.append("#   [material MyFoam]")
out.append("#   same_as = Polystyrene")
out.append("#   density = 0.05 g/cm3")
out.append("")
for m in mats:
    out.append(f"[material {m['id']}]")
    out.append(f"title = {q(m['title'])}")
    out.append(f"density = {m['density']:g} g/cm3")
    comp = ', '.join(f"{el} = {fmt(w)}" for el, w in m['elems'].items())
    out.append(f"composition = {{ {comp} }}")
    notes = m['comment']
    if m['iso']:
        notes = (notes + ' ' if notes else '') + "Isotopic composition summed by element (photon transport does not depend on the isotope)."
    s = sum(m['elems'].values())
    if abs(s - 1) > 2e-3:
        notes = (notes + ' ' if notes else '') + f"Fractions in the source sum to {s:.4f}."
    if notes: out.append(f"comment = {q(notes)}")
    if m['ref']: out.append(f"reference = {q(m['ref'])}")
    out.append("")
out.append("# ---- other names used in the compendium ----")
out.append("")
for alias, target in aliases.items():
    tid = by_plain.get(target) or title_to_id.get(target)
    assert tid, (alias, target)
    out.append(f"[material {alias}]\nsame_as = {tid}\n")
open(sys.argv[4], 'w').write('\n'.join(out))
for m in mats: print(f"{m['id']:40s} <- {m['title']}")
