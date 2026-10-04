#!/usr/bin/env python3
"""Erzeugt die Weltkarte der Oberfläche (ui/src/karte/welt.json) aus Natural Earth.

Je Land (ISO-3166-alpha-3, nur Länder aus data/laender/) ein SVG-Pfad in einer
gleichabstandigen Projektion: x = Länge, y = −Breite, Fläche 360 × 180, vereinfacht
(Douglas-Peucker) und auf 0,1° gerundet. Gebiete ohne eigenes Land zählen wie in
laender.py zum Mutterland.

Aufruf: python3 tools/daten/karte.py
"""

import json
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
import laender  # noqa: E402

ROOT = os.path.join(os.path.dirname(__file__), "..", "..")
OUT = os.path.join(ROOT, "ui", "src", "karte", "welt.json")
TOLERANZ = 0.12  # Grad


def vereinfache(punkte, toleranz):
    if len(punkte) < 3:
        return punkte
    (x1, y1), (x2, y2) = punkte[0], punkte[-1]
    dx, dy = x2 - x1, y2 - y1
    laenge = (dx * dx + dy * dy) ** 0.5
    weitester, abstand = 0, -1.0
    for i in range(1, len(punkte) - 1):
        x, y = punkte[i]
        if laenge == 0:
            d = ((x - x1) ** 2 + (y - y1) ** 2) ** 0.5
        else:
            d = abs(dy * x - dx * y + x2 * y1 - y2 * x1) / laenge
        if d > abstand:
            weitester, abstand = i, d
    if abstand <= toleranz:
        return [punkte[0], punkte[-1]]
    links = vereinfache(punkte[: weitester + 1], toleranz)
    rechts = vereinfache(punkte[weitester:], toleranz)
    return links[:-1] + rechts


def pfad(ring):
    punkte = vereinfache([(lon, -lat) for lon, lat in ring], TOLERANZ)
    punkte = [(round(x + 180, 1), round(y + 90, 1)) for x, y in punkte]
    eindeutig = [p for i, p in enumerate(punkte) if i == 0 or p != punkte[i - 1]]
    if len(eindeutig) < 4:
        return ""
    return "M" + "L".join(f"{x:g} {y:g}" for x, y in eindeutig) + "Z"


def main():
    laender.fetch()
    iso_codes = {f[:-5] for f in os.listdir(os.path.join(ROOT, "data", "laender"))}
    ne = json.load(open(os.path.join(laender.CACHE, "ne_countries.geojson")))
    sovereign = {}
    for f in ne["features"]:
        p = f["properties"]
        iso = laender.NE_MERGE.get(p["ADM0_A3"], p["ISO_A3_EH"])
        if iso in iso_codes:
            sovereign.setdefault(p["SOV_A3"], iso)
    pfade = {}
    for f in ne["features"]:
        p = f["properties"]
        iso = laender.NE_MERGE.get(p["ADM0_A3"], p["ISO_A3_EH"])
        if iso not in iso_codes:
            iso = sovereign.get(p["SOV_A3"])
        if iso is None:
            continue
        for polygon in laender.polygons(f["geometry"]):
            d = pfad(polygon[0])
            if d:
                pfade.setdefault(iso, []).append(d)
    fehlend = sorted(iso_codes - set(pfade))
    karte = {iso: "".join(teile) for iso, teile in sorted(pfade.items())}
    with open(OUT, "w") as f:
        json.dump(karte, f, separators=(",", ":"))
        f.write("\n")
    print(f"{OUT}: {len(karte)} Länder, {os.path.getsize(OUT) // 1024} KB")
    if fehlend:
        print("Ohne Fläche in Natural Earth 1:50 Mio.:", ", ".join(fehlend))


if __name__ == "__main__":
    main()
