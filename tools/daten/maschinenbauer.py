#!/usr/bin/env python3
"""Generates data/maschinenbauer.yaml and its texts (MAA).

Fictional machine builders, 5-8 per country/region. Tendency (-0.3..+0.3) shifts
only performance L. Names are invented from per-region stems; deterministic.
"""
import random, re, pathlib

root = pathlib.Path(__file__).resolve().parents[2]
codes = sorted(p.stem for p in (root / "data/laender").glob("*.yaml"))

stems = {
    "de": (["Rhein", "Elb", "Ruhr", "Nord", "Alt", "Kron", "Stahl", "Wend", "Hans", "Berg", "Fels"],
           ["Maschinenbau", "Werke", "Anlagenbau", "Technik", "Apparatebau"], ["& Söhne", "AG", "GmbH", ""]),
    "en": (["Thames", "Bridge", "Union", "Iron", "Crown", "Atlas", "Summit", "Harbor", "Pioneer", "Granite"],
           ["Engineering", "Machinery", "Works", "Industries", "Foundry"], ["& Sons", "Ltd.", "Co.", ""]),
    "fr": (["Loire", "Rhône", "Aurore", "Lorraine", "Vauban", "Comptoir", "Provence"],
           ["Constructions", "Mécanique", "Ateliers", "Forges"], ["& Fils", "S.A.", ""]),
    "es": (["Castilla", "Iberia", "Sierra", "Delta", "Andes", "Pampa", "Mar"],
           ["Maquinaria", "Talleres", "Ingeniería", "Industrias"], ["S.A.", "e Hijos", ""]),
    "it": (["Lombarda", "Adria", "Etna", "Pavia", "Torino", "Liguria"],
           ["Meccanica", "Officine", "Costruzioni"], ["S.p.A.", "& Figli", ""]),
    "sl": (["Wisła", "Kama", "Volga", "Ural", "Dnipro", "Tatra", "Baltik"],
           ["Mashstroi", "Zavod", "Technika", "Metalloprom"], ["Werke", ""]),
    "as": (["Sakura", "Hanjiang", "Lotus", "Mekong", "Jade", "Fuji", "Seoul", "Ganga", "Indus", "Orient"],
           ["Heavy Industries", "Machinery", "Engineering", "Works"], ["Co.", "Corp.", ""]),
    "ar": (["Nil", "Atlas", "Anatolia", "Zagros", "Sahara", "Tigris", "Levant"],
           ["Engineering", "Machinery", "Industrial Works", "Metalworks"], ["Co.", "& Brothers", ""]),
    "af": (["Congo", "Savanna", "Kilimanjaro", "Niger", "Cape", "Zambezi"],
           ["Engineering", "Machinery", "Works"], ["Ltd.", "Co.", ""]),
}
group = {}
for c in "DEU AUT CHE".split(): group[c] = "de"
for c in "GBR USA CAN AUS XNO XOA XZA".split(): group[c] = "en"
for c in "FRA".split(): group[c] = "fr"
for c in "ESP MEX ARG COL PER XSA XSN XSO XME XMG".split(): group[c] = "es"
for c in "ITA".split(): group[c] = "it"
for c in "POL RUS UKR XAR XLE".split(): group[c] = "sl"
for c in "CHN JPN KOR TWN VNM MYS PHL IDN IND PAK BGD".split(): group[c] = "as"
for c in "TUR IRN EGY XWA".split(): group[c] = "ar"
for c in "NGA ETH COD XAL".split(): group[c] = "af"

def slug(s):
    s = s.lower()
    for a, b in (("ä","ae"),("ö","oe"),("ü","ue"),("ß","ss"),("é","e"),("è","e"),("ô","o"),("ł","l"),("ê","e")):
        s = s.replace(a, b)
    return re.sub(r"[^a-z0-9]+", "_", s).strip("_")

rng = random.Random(1912)
entries, texts, used = [], [], set()
for code in codes:
    pool = stems[group.get(code, "en")]
    n = rng.randint(5, 8)
    k = 0
    tries = 0
    while k < n and tries < 200:
        tries += 1
        name = f"{rng.choice(pool[0])} {rng.choice(pool[1])} {rng.choice(pool[2])}".strip()
        key = f"{code.lower()}_{slug(name)}"
        if key in used:
            continue
        used.add(key)
        tend = round(rng.uniform(-0.3, 0.3), 2)
        ab = rng.choice([1900, 1900, 1900, 1920, 1950])
        entries.append(f"  - {{id: {key}, land: {code}, tendenz: {tend}, ab: {ab}}}")
        texts.append(f"maschinenbauer.{key}: {name}")
        k += 1

(root / "data/maschinenbauer.yaml").write_text(
    "# Erfundene Maschinenbauer für Ausschreibungen (MAA, docs/MASCHINENANGEBOTE.md §4).\n"
    "# Erzeugt von tools/daten/maschinenbauer.py. Tendenz verschiebt nur die Leistung L.\n\n"
    "maschinenbauer:\n" + "\n".join(entries) + "\n")
(root / "data/texte/de/maschinenbauer.yaml").write_text(
    "# Namen der Maschinenbauer (erfunden; erzeugt von tools/daten/maschinenbauer.py)\n\n"
    + "\n".join(texts) + "\n")
print(len(entries))
