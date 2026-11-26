#!/usr/bin/env python3
"""Build gettext .po files from a translated copy of the documentation.

The translator writes each page in the target language with exactly the same
structure as the English page (same paragraphs, list items, table cells,
headings). This script extracts the messages of both versions with Sphinx's
gettext builder and pairs them in order, page by page, writing
docs/locale/<lang>/LC_MESSAGES/<page>.po.

    python3 tools/docs/align_translation.py --lang pt_BR --translated path/to/pt_BR

Pages whose number of messages differs are reported and skipped.
Requires: sphinx, myst-parser, sphinx-design, pydata-sphinx-theme, sphinx-copybutton.
"""

import argparse
import os
import re
import shutil
import subprocess
import sys
import tempfile


def gettext(src, out):
    subprocess.run([sys.executable, "-m", "sphinx", "-q", "-b", "gettext", "-E", src, out], check=True)


def doc_messages(out):
    """Messages of every page, in document order and with repetitions
    (the .pot files merge repeated messages, which breaks the pairing)."""
    import pickle
    from sphinx import addnodes
    from sphinx.util.nodes import extract_messages

    result = {}
    tree_dir = os.path.join(out, ".doctrees")
    for root, _, files in os.walk(tree_dir):
        for f in files:
            if not f.endswith(".doctree"):
                continue
            with open(os.path.join(root, f), "rb") as fh:
                doctree = pickle.load(fh)
            msgs = [m for _, m in extract_messages(doctree) if m]
            for toc in doctree.findall(addnodes.toctree):
                msgs += [title for title, _ in toc["entries"] if title]
            result[os.path.relpath(os.path.join(root, f), tree_dir)[: -len(".doctree")]] = msgs
    return result


def po_escape(s):
    s = s.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n")
    return f'"{s}"'


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--lang", required=True)
    ap.add_argument("--translated", required=True, help="folder with the translated pages (same names as docs/)")
    ap.add_argument("--docs", default=os.path.join(os.path.dirname(__file__), "..", "..", "docs"))
    a = ap.parse_args()
    docs = os.path.abspath(a.docs)

    with tempfile.TemporaryDirectory() as tmp:
        # the translated tree needs the same conf.py and static files
        # start from the English tree, then overlay the translated pages, so a
        # partial translation still builds (untranslated pages stay empty)
        tsrc = os.path.join(tmp, "src")
        shutil.copytree(docs, tsrc, ignore=shutil.ignore_patterns("_build", "locale"))
        shutil.copytree(a.translated, tsrc, dirs_exist_ok=True)
        # include files referenced relative to docs/ (e.g. ../PHILOSOPHY.md)
        for f in os.listdir(os.path.dirname(docs)):
            if f.endswith(".md"):
                shutil.copy(os.path.join(os.path.dirname(docs), f), tmp)
        gettext(docs, os.path.join(tmp, "en"))
        gettext(tsrc, os.path.join(tmp, "tr"))

        en_all = doc_messages(os.path.join(tmp, "en"))
        tr_all = doc_messages(os.path.join(tmp, "tr"))
        ok, bad = 0, []
        for doc in sorted(en_all):
            en, tr = en_all[doc], tr_all.get(doc)
            rel = doc + ".pot"
            if tr is None:
                bad.append(f"{doc}: no translated page")
                continue
            if en == tr:
                continue  # page not translated yet
            if len(en) != len(tr):
                n = min(len(en), len(tr))
                i = next((k for k in range(n) if (en[k] == tr[k]) != (en[k] == tr[k]) or en[k].startswith("`") != tr[k].startswith("`")), n)
                bad.append(f"{doc}: {len(en)} English messages, {len(tr)} translated (structures differ near {en[min(i, len(en) - 1)][:50]!r})")
                continue
            out = os.path.join(docs, "locale", a.lang, "LC_MESSAGES", rel[:-4] + ".po")
            os.makedirs(os.path.dirname(out), exist_ok=True)
            seen = set()
            with open(out, "w", encoding="utf-8") as po:
                po.write(f'msgid ""\nmsgstr ""\n"Content-Type: text/plain; charset=UTF-8\\n"\n"Language: {a.lang}\\n"\n\n')
                for x, y in zip(en, tr):
                    if x in seen:
                        continue
                    seen.add(x)
                    if y == x:
                        y = ""
                    # translations are parsed again as Markdown: "1. Title" would become a list
                    y = re.sub(r"^(\d+)\. ", r"\1\\. ", y)
                    po.write(f"msgid {po_escape(x)}\nmsgstr {po_escape(y)}\n\n")
            ok += 1
        print(f"{ok} pages written to docs/locale/{a.lang}/LC_MESSAGES")
        for b in bad:
            print("  NOT ALIGNED:", b)
        sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
