"""Stellt die Verweise der Spieldaten auf zusammengefasste Länder um (Regionen, 11.10.2026).

Nach einer Änderung in tools/daten/regionen.py und dem Neuerzeugen der Länder
(laender.py, karte.py) ersetzt dieses Skript in allen übrigen Datendateien die Codes
aufgegangener Länder und früherer Regionen durch ihre Region:

- Listen ([A, B, C]) verlieren doppelte Einträge;
- eine Zuordnung je Land (Zeile „  CODE: …“) behält den Eintrag des Leitlands bzw. den
  ersten;
- Landeswährungen (`- land: CODE` mit Perioden) behalten die des Leitlands.

Aufruf: python3 tools/daten/regionen_umstellen.py
"""

import glob
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)
import regionen  # noqa: E402


def mapping():
    current = {os.path.basename(f)[:-5] for f in glob.glob(os.path.join(ROOT, "data", "laender", "*.yaml"))}
    m = {k: v for k, v in regionen.region_of().items()}
    m.update(regionen.FRUEHER)
    return {k: v for k, v in m.items() if k not in current}


def lead_of(region):
    return regionen.LEITLAND.get(region, region)


def dedupe_lists(text):
    def fix(match):
        items = [i.strip() for i in match.group(1).split(",")]
        seen = []
        for i in items:
            if i and i not in seen:
                seen.append(i)
        return "[" + ", ".join(seen) + "]"

    return re.sub(r"\[([A-Z]{3}(?:\s*,\s*[A-Z]{3})*)\]", fix, text)


def replace_codes(text, m, keep_original=False):
    pat = re.compile(r"\b(" + "|".join(sorted(m)) + r")\b")
    return pat.sub(lambda x: m[x.group(1)], text)


def dedupe_keyed_lines(lines, m):
    """Lines „<indent>CODE: …“ in one block: keep the lead country's line, else the first."""
    out = []
    i = 0
    key_re = re.compile(r"^(\s+)([A-Z]{3}):(.*)$")
    while i < len(lines):
        block = []
        j = i
        while j < len(lines) and key_re.match(lines[j]):
            block.append(lines[j])
            j += 1
        if not block:
            out.append(lines[i])
            i += 1
            continue
        chosen = {}
        order = []
        for line in block:
            indent, code, rest = key_re.match(line).groups()
            region = m.get(code, code)
            is_lead = lead_of(region) == code or code == region
            if region not in chosen:
                order.append(region)
                chosen[region] = (is_lead, f"{indent}{region}:{rest}")
            elif is_lead and not chosen[region][0]:
                chosen[region] = (True, f"{indent}{region}:{rest}")
        out += [chosen[r][1] for r in order]
        i = j
    return out


def currencies(path, m):
    text = open(path).read()
    head, _, body = text.partition("landeswaehrungen:\n")
    entries = re.split(r"(?m)^(?=  - land: )", body)
    keep = {}
    order = []
    prefix = entries[0] if not entries[0].startswith("  - land:") else ""
    for e in entries:
        mm = re.match(r"  - land: ([A-Z]{3})", e)
        if not mm:
            continue
        code = mm.group(1)
        region = m.get(code, code)
        is_lead = code == region or lead_of(region) == code
        e = e.replace(f"land: {code}", f"land: {region}", 1)
        if region not in keep:
            order.append(region)
            keep[region] = (is_lead, e)
        elif is_lead and not keep[region][0]:
            keep[region] = (True, e)
    with open(path, "w") as f:
        f.write(head + "landeswaehrungen:\n" + prefix + "".join(keep[r][1] for r in sorted(order)))


def main():
    m = mapping()
    files = [f for f in glob.glob(os.path.join(ROOT, "data", "**", "*.yaml"), recursive=True)
             if "/laender/" not in f and not f.endswith("texte/de/laender.yaml")]
    for path in sorted(files):
        if path.endswith(os.path.join("waehrungen", "laender.yaml")):
            currencies(path, m)
            continue
        text = open(path).read()
        if not re.search(r"\b(" + "|".join(sorted(m)) + r")\b", text):
            continue
        lines = text.split("\n")
        lines = dedupe_keyed_lines(lines, m)
        text = replace_codes("\n".join(lines), m)
        text = dedupe_lists(text)
        with open(path, "w") as f:
            f.write(text)
        print("umgestellt:", os.path.relpath(path, ROOT))


if __name__ == "__main__":
    main()
