#!/usr/bin/env python3
"""Erzeugt data/laender/*.yaml und data/texte/de/laender.yaml aus öffentlichen Quellen.

Quellen (werden nach tools/daten/.cache/ geladen):
- Gapminder Fast Track (CC BY 4.0): Bevölkerung, BIP je Kopf (KKP, int.-$ 2017), Gini
  https://github.com/open-numbers/ddf--gapminder--fasttrack
- Natural Earth 1:50m (gemeinfrei): Grenzen, Namen, Hauptstädte
  https://github.com/nvkelso/natural-earth-vector

Aufruf: python3 tools/daten/laender.py   (aus dem Projektverzeichnis)

Kleine Länder werden zu Regionen zusammengefasst (tools/daten/regionen.py, M34): Summen
für Bevölkerung, BIP und Fläche, bevölkerungsgewichtete Mittel für BIP je Kopf und Gini.

Alle Werte gelten in heutigen Grenzen (Lastenheft §3.2) und zeigen den realen
Verlauf (Entscheidung 13). Korrekturen und Schätzungen sind unten dokumentiert und
werden in den Dateien als Annäherung gekennzeichnet.
"""

import collections
import csv
import json
import math
import os
import re
import sys
import unicodedata
import urllib.request

sys.path.insert(0, os.path.dirname(__file__))
import regionen  # noqa: E402

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
CACHE = os.path.join(os.path.dirname(__file__), ".cache")
GAPMINDER = "https://raw.githubusercontent.com/open-numbers/ddf--gapminder--fasttrack/master"
NATURAL_EARTH = "https://raw.githubusercontent.com/nvkelso/natural-earth-vector/master/geojson"
SOURCES = {
    "pop.csv": f"{GAPMINDER}/countries_etc_datapoints/ddf--datapoints--pop--by--country--time.csv",
    "gdp_pcap.csv": f"{GAPMINDER}/countries_etc_datapoints/ddf--datapoints--gdp_pcap--by--country--time.csv",
    "gini.csv": f"{GAPMINDER}/countries_etc_datapoints/ddf--datapoints--gini--by--country--time.csv",
    "geo.csv": f"{GAPMINDER}/ddf--entities--geo--country.csv",
    "ne_countries.geojson": f"{NATURAL_EARTH}/ne_50m_admin_0_countries.geojson",
    "ne_places.geojson": f"{NATURAL_EARTH}/ne_50m_populated_places_simple.geojson",
}

# Int.-$ 2017 (KKP) → USD mit Kaufkraft 2026: US-Verbraucherpreise 2017 → 2026, geschätzt.
USD_2017_TO_2026 = 1.33

# Jahre in den Dateien: jährlich bis 2026, danach in Fünfjahresschritten (Projektionen).
YEARS = list(range(1900, 2027)) + list(range(2030, 2101, 5))

QUELLE = (
    "Bevölkerung, BIP je Kopf und Gini: Gapminder Fast Track (CC BY 4.0), BIP von int.-$ 2017 "
    f"mit Faktor {USD_2017_TO_2026} auf USD 2026 umgerechnet; Fläche, Nachbarn, Hauptstadt: Natural Earth. "
    "Steuern, Stabilität und Prägungen: eigene Schätzungen. Regionen: Summen bzw. "
    "bevölkerungsgewichtete Mittel ihrer Länder."
)

# ---------------------------------------------------------------------------------------
# Länderauswahl und Korrekturen auf heutige Grenzen
# ---------------------------------------------------------------------------------------

# Natural-Earth-Gebiete, die zu einem anderen Land gehören (heutige Grenzen).
NE_MERGE = {"SOL": "SOM", "CYN": "CYP", "KAS": "IND", "KOS": "XKX", "SAH": "ESH", "PSX": "PSE"}
# Gapminder-Gebiete, die zu einem anderen Land gezählt werden.
GAPMINDER_MERGE = {"hkg": "chn"}
EXCLUDED = {"hos"}  # Vatikanstadt: kein UN-Mitglied, wirtschaftlich ohne Bedeutung

# Länder ohne Gapminder-Werte: Bevölkerung (Stützjahre) und Bezugsland für BIP und Gini.
ADDED = {
    "XKX": {  # Kosovo, Volkszählungen und Schätzungen
        "name": "Kosovo",
        "pop": {1900: 400e3, 1921: 440e3, 1931: 550e3, 1948: 730e3, 1961: 960e3, 1971: 1.24e6,
                1981: 1.58e6, 1991: 1.95e6, 2000: 1.70e6, 2011: 1.74e6, 2020: 1.78e6, 2026: 1.60e6,
                2100: 1.2e6},
        "like": "srb", "gdp_factor": 0.5,
    },
    "ESH": {  # Westsahara
        "name": "Westsahara",
        "pop": {1900: 20e3, 1950: 14e3, 1970: 76e3, 1990: 220e3, 2010: 480e3, 2020: 600e3,
                2026: 620e3, 2100: 900e3},
        "like": "mar", "gdp_factor": 0.6,
    },
    "LIE": {"name": None, "pop": None, "like": "che", "gdp_factor": 1.0},
}

def german_reich_factor(year):
    """Gapminder führt Deutschland bis 1949 in früheren Grenzen (bis etwa 1913 ungefähr
    Kaiserreich ohne Elsass-Lothringen, ab etwa 1925 Grenzen von 1937). Anteil der
    Bevölkerung im heutigen Gebiet: 0,785 bzw. 0,855; Übergänge linear, damit kein
    Sprung entsteht."""
    points = {1913: 0.785, 1925: 0.855, 1944: 0.855, 1950: 1.0}
    return interpolate(points, year)


NORTHERN_IRELAND = 1.25e6  # um 1900–1921

# ---------------------------------------------------------------------------------------
# Schätzungen (Annäherungen) für Steuern, Stabilität und Prägungen
# ---------------------------------------------------------------------------------------

# Gewinnsteuer auf Unternehmensebene, effektiv (Anteil). Fehlt ein Land, gilt der
# Standardverlauf aus data/parameter/laendermodell.yaml.
CORPORATE_TAX = {
    "USA": {1900: 0.0, 1909: 0.01, 1916: 0.02, 1917: 0.06, 1918: 0.12, 1922: 0.125, 1926: 0.135,
            1929: 0.11, 1930: 0.12, 1936: 0.15, 1940: 0.24, 1942: 0.40, 1952: 0.52, 1979: 0.46,
            1987: 0.38, 1993: 0.39, 2018: 0.26, 2026: 0.26},
    "GBR": {1900: 0.06, 1914: 0.06, 1916: 0.25, 1918: 0.30, 1922: 0.25, 1930: 0.225, 1940: 0.425,
            1946: 0.45, 1965: 0.40, 1973: 0.52, 1984: 0.35, 2000: 0.30, 2010: 0.28, 2017: 0.19,
            2023: 0.25, 2026: 0.25},
    "DEU": {1900: 0.04, 1914: 0.05, 1920: 0.10, 1925: 0.15, 1928: 0.20, 1939: 0.40, 1946: 0.60,
            1953: 0.50, 1977: 0.56, 1990: 0.50, 2001: 0.39, 2008: 0.30, 2026: 0.30},
    "FRA": {1900: 0.04, 1917: 0.08, 1920: 0.10, 1930: 0.15, 1948: 0.24, 1958: 0.50, 1993: 0.333,
            2020: 0.28, 2022: 0.25, 2026: 0.25},
}

# Politische Stabilität (0 = Bürgerkrieg, 1 = sehr stabil). Standard 0,8.
STABILITY = {
    "RUS": {1900: 0.7, 1904: 0.6, 1905: 0.35, 1907: 0.6, 1914: 0.55, 1917: 0.1, 1918: 0.05,
            1921: 0.3, 1923: 0.5, 1928: 0.55, 1930: 0.45, 1937: 0.4, 1941: 0.35, 1946: 0.6,
            1953: 0.7, 1989: 0.5, 1991: 0.3, 1994: 0.35, 2000: 0.5, 2008: 0.55, 2022: 0.4, 2026: 0.4},
    "UKR": {1900: 0.65, 1905: 0.35, 1907: 0.6, 1914: 0.5, 1917: 0.1, 1918: 0.05, 1921: 0.25,
            1923: 0.45, 1932: 0.3, 1934: 0.4, 1941: 0.1, 1945: 0.4, 1953: 0.65, 1991: 0.4,
            2004: 0.45, 2014: 0.3, 2022: 0.1, 2026: 0.15},
    "CHN": {1900: 0.25, 1902: 0.45, 1911: 0.15, 1913: 0.3, 1916: 0.15, 1928: 0.35, 1937: 0.1,
            1949: 0.2, 1952: 0.5, 1959: 0.3, 1962: 0.45, 1966: 0.25, 1977: 0.5, 1989: 0.55,
            1993: 0.7, 2026: 0.7},
    "MEX": {1900: 0.7, 1910: 0.15, 1920: 0.35, 1929: 0.55, 1940: 0.7, 2026: 0.6},
    "DEU": {1900: 0.9, 1914: 0.8, 1918: 0.3, 1919: 0.35, 1920: 0.4, 1923: 0.35, 1924: 0.65,
            1929: 0.6, 1932: 0.4, 1934: 0.6, 1939: 0.5, 1945: 0.2, 1949: 0.7, 1955: 0.9, 2026: 0.9},
    "AUT": {1900: 0.8, 1918: 0.3, 1920: 0.45, 1927: 0.5, 1934: 0.35, 1938: 0.4, 1945: 0.3,
            1955: 0.85, 2026: 0.9},
    "HUN": {1900: 0.8, 1918: 0.3, 1919: 0.15, 1920: 0.5, 1944: 0.2, 1948: 0.5, 1956: 0.2,
            1957: 0.55, 1990: 0.75, 2026: 0.75},
    "POL": {1900: 0.5, 1905: 0.35, 1907: 0.5, 1914: 0.3, 1918: 0.25, 1921: 0.5, 1926: 0.55,
            1939: 0.05, 1945: 0.3, 1948: 0.55, 1981: 0.4, 1990: 0.7, 2026: 0.8},
    "TUR": {1900: 0.5, 1908: 0.4, 1912: 0.35, 1918: 0.2, 1922: 0.3, 1924: 0.6, 1960: 0.45,
            1961: 0.6, 1980: 0.45, 1983: 0.6, 2016: 0.45, 2026: 0.5},
    "IRL": {1900: 0.75, 1916: 0.5, 1919: 0.3, 1923: 0.5, 1924: 0.75, 2026: 0.9},
    "ITA": {1900: 0.75, 1919: 0.5, 1922: 0.5, 1925: 0.65, 1943: 0.2, 1946: 0.6, 1950: 0.75, 2026: 0.75},
    "ESP": {1900: 0.65, 1909: 0.55, 1923: 0.6, 1931: 0.5, 1936: 0.05, 1939: 0.4, 1950: 0.6,
            1975: 0.6, 1981: 0.75, 2026: 0.8},
    "FRA": {1900: 0.85, 1940: 0.3, 1944: 0.4, 1946: 0.7, 1958: 0.7, 1968: 0.65, 1970: 0.85, 2026: 0.85},
    "JPN": {1900: 0.85, 1945: 0.3, 1950: 0.8, 2026: 0.9},
    "KOR": {1900: 0.5, 1910: 0.45, 1945: 0.4, 1950: 0.05, 1953: 0.35, 1961: 0.5, 1987: 0.75, 2026: 0.8},
    "IND": {1900: 0.6, 1919: 0.5, 1947: 0.3, 1950: 0.55, 2026: 0.65},
    "PAK": {1900: 0.6, 1947: 0.3, 1950: 0.45, 2026: 0.4},
    "BGD": {1900: 0.6, 1947: 0.35, 1971: 0.15, 1973: 0.4, 2026: 0.45},
    "GRC": {1900: 0.65, 1912: 0.5, 1922: 0.4, 1924: 0.55, 1941: 0.2, 1946: 0.15, 1950: 0.5,
            1967: 0.45, 1974: 0.65, 2026: 0.75},
    "IRN": {1900: 0.5, 1905: 0.35, 1911: 0.4, 1925: 0.55, 1979: 0.2, 1981: 0.4, 2026: 0.4},
    "BRA": {1900: 0.65, 1930: 0.5, 1937: 0.55, 1964: 0.5, 1985: 0.65, 2026: 0.65},
    "ARG": {1900: 0.75, 1930: 0.55, 1955: 0.45, 1976: 0.35, 1983: 0.6, 2001: 0.45, 2003: 0.6, 2026: 0.6},
    "USA": {1900: 0.9, 2026: 0.85},
    "GBR": {1900: 0.9, 2026: 0.9},
    "CHE": {1900: 0.95, 2026: 0.95},
    "SWE": {1900: 0.9, 2026: 0.95},
}

# Gewichte der Fachrichtungen (1 = Durchschnitt), Zuschlag zur Automatisierungsprägung
# und Forschungsstärke je Fachgebiet. Grobe Schätzungen für wichtige Industrieländer.
PROFILE = {
    "GBR": {"fachrichtungen": {"textil": 1.5, "bergbau": 1.3, "maschinenbau": 1.2, "metall": 1.2},
            "forschung": {"metall": 1.2, "maschinenbau": 1.2, "elektro": 1.1}},
    "DEU": {"fachrichtungen": {"chemie": 1.5, "metall": 1.3, "elektro": 1.4, "maschinenbau": 1.2, "bergbau": 1.2},
            "forschung": {"chemie": 1.7, "elektro": 1.4, "metall": 1.3, "maschinenbau": 1.2}},
    "USA": {"fachrichtungen": {"maschinenbau": 1.3, "elektro": 1.3, "metall": 1.2},
            "forschung": {"elektro": 1.4, "maschinenbau": 1.3}, "automatisierung": 0.15},
    "FRA": {"fachrichtungen": {"textil": 1.2, "chemie": 1.1},
            "forschung": {"chemie": 1.2, "maschinenbau": 1.1}},
    "BEL": {"fachrichtungen": {"bergbau": 1.4, "metall": 1.4}},
    "SWE": {"fachrichtungen": {"metall": 1.4, "bergbau": 1.3},
            "forschung": {"metall": 1.3, "elektro": 1.2}},
    "CHE": {"fachrichtungen": {"chemie": 1.5, "maschinenbau": 1.3},
            "forschung": {"chemie": 1.5, "maschinenbau": 1.2}},
    "CZE": {"fachrichtungen": {"maschinenbau": 1.2, "metall": 1.2}},
    "JPN": {"fachrichtungen": {"textil": 1.4}},
    "IND": {"fachrichtungen": {"textil": 1.3}},
    "RUS": {"fachrichtungen": {"bergbau": 1.2}},
}

# ---------------------------------------------------------------------------------------

def fetch():
    os.makedirs(CACHE, exist_ok=True)
    for name, url in SOURCES.items():
        path = os.path.join(CACHE, name)
        if not os.path.exists(path):
            print("lade", url)
            urllib.request.urlretrieve(url, path)


def read_series(name, column):
    data = collections.defaultdict(dict)
    with open(os.path.join(CACHE, name)) as f:
        for row in csv.DictReader(f):
            data[row["country"]][int(row["time"])] = float(row[column])
    return data


def interpolate(points, year):
    years = sorted(points)
    if year <= years[0]:
        return points[years[0]]
    if year >= years[-1]:
        return points[years[-1]]
    for a, b in zip(years, years[1:]):
        if a <= year <= b:
            return points[a] + (points[b] - points[a]) * (year - a) / (b - a)
    raise AssertionError


def ring_area_km2(ring):
    """Fläche eines Polygonrings auf der Kugel (km²)."""
    r = 6371.0088
    area = 0.0
    for (lon1, lat1), (lon2, lat2) in zip(ring, ring[1:] + ring[:1]):
        area += math.radians(lon2 - lon1) * (2 + math.sin(math.radians(lat1)) + math.sin(math.radians(lat2)))
    return abs(area * r * r / 2)


def polygons(geometry):
    if geometry["type"] == "Polygon":
        return [geometry["coordinates"]]
    return geometry["coordinates"]


def main():
    fetch()
    pop = read_series("pop.csv", "pop")
    gdp = read_series("gdp_pcap.csv", "gdp_pcap")
    gini = read_series("gini.csv", "gini")
    geo = {r["country"]: r for r in csv.DictReader(open(os.path.join(CACHE, "geo.csv")))}

    # --- Gapminder-Länder zusammenstellen ---------------------------------------------
    series = {}
    for code in pop:
        if code in EXCLUDED or code in GAPMINDER_MERGE:
            continue
        iso = code.upper()
        if code not in gdp or code not in gini:
            if iso not in ADDED:
                continue
        series[iso] = {"pop": dict(pop[code]), "gdp": dict(gdp.get(code, {})), "gini": dict(gini.get(code, {}))}

    for source, target in GAPMINDER_MERGE.items():
        t = series[target.upper()]
        for y in list(t["pop"]):
            p_t, p_s = t["pop"][y], pop[source].get(y, 0)
            if y in t["gdp"] and y in gdp[source]:
                t["gdp"][y] = (t["gdp"][y] * p_t + gdp[source][y] * p_s) / (p_t + p_s)
            t["pop"][y] = p_t + p_s

    for iso, spec in ADDED.items():
        like = spec["like"]
        entry = series.setdefault(iso, {"pop": {}, "gdp": {}, "gini": {}})
        if spec["pop"]:
            entry["pop"] = {y: interpolate(spec["pop"], y) for y in range(1800, 2101)}
        entry["gdp"] = {y: v * spec["gdp_factor"] for y, v in gdp[like].items()}
        entry["gini"] = dict(gini[like])

    # Deutschland: historische → heutige Grenzen.
    for y, v in series["DEU"]["pop"].items():
        series["DEU"]["pop"][y] = v * german_reich_factor(y)
    # Irland und Großbritannien vor der Teilung (Gapminder zählt Nordirland doppelt).
    for y in series["IRL"]["pop"]:
        if series["IRL"]["pop"][y] > 3.5e6:
            republic = series["IRL"]["pop"][y] - NORTHERN_IRELAND
            series["IRL"]["pop"][y] = republic
            series["GBR"]["pop"][y] -= republic

    # --- Natural Earth: Fläche, Nachbarn, Hauptstadt, Namen ------------------------------
    ne = json.load(open(os.path.join(CACHE, "ne_countries.geojson")))
    sovereign = {}
    for f in ne["features"]:
        p = f["properties"]
        iso = NE_MERGE.get(p["ADM0_A3"], p["ISO_A3_EH"])
        if iso in series:
            sovereign.setdefault(p["SOV_A3"], iso)
    area = collections.Counter()
    vertices = collections.defaultdict(set)
    names = {}
    continents = {}
    for f in ne["features"]:
        p = f["properties"]
        iso = NE_MERGE.get(p["ADM0_A3"], p["ISO_A3_EH"])
        if iso not in series:
            iso = sovereign.get(p["SOV_A3"])
        if iso is None:
            continue
        own = NE_MERGE.get(p["ADM0_A3"], p["ISO_A3_EH"]) == iso
        if own:
            names.setdefault(iso, p["NAME_DE"])
            continents.setdefault(iso, (p["CONTINENT"], p["REGION_UN"], p["SUBREGION"]))
        for polygon in polygons(f["geometry"]):
            outer, *holes = polygon
            area[iso] += ring_area_km2(outer) - sum(ring_area_km2(h) for h in holes)
            for ring in polygon:
                for lon, lat in ring:
                    vertices[iso].add((round(lon, 3), round(lat, 3)))
    by_vertex = collections.defaultdict(set)
    for iso, points in vertices.items():
        for point in points:
            by_vertex[point].add(iso)
    shared = collections.Counter()
    for owners in by_vertex.values():
        for a in owners:
            for b in owners:
                if a < b:
                    shared[(a, b)] += 1
    neighbours = collections.defaultdict(set)
    for (a, b), count in shared.items():
        if count >= 2:
            neighbours[a].add(b)
            neighbours[b].add(a)

    capitals = {}
    places = json.load(open(os.path.join(CACHE, "ne_places.geojson")))
    # Städte je Land (W2): alle Orte mit Einwohnerzahl, die Hauptstadt markiert.
    towns = collections.defaultdict(list)
    for f in places["features"]:
        p = f["properties"]
        if p["pop_max"] and p["pop_max"] > 0:
            iso = NE_MERGE.get(p["adm0_a3"], p["adm0_a3"])
            towns[iso].append({
                "name": " ".join(p["name"].split()),
                "ascii": " ".join(p["nameascii"].split()),
                "pop": int(p["pop_max"]),
                "lat": p["latitude"],
                "lon": p["longitude"],
                "capital": p["featurecla"] == "Admin-0 capital" and p.get("adm0cap") == 1,
            })
    for f in places["features"]:
        p = f["properties"]
        if p["featurecla"] != "Admin-0 capital" or p.get("adm0cap") != 1:
            continue
        iso = NE_MERGE.get(p["adm0_a3"], p["adm0_a3"])
        best = capitals.get(iso)
        if best is None or p["pop_max"] > best[2]:
            capitals[iso] = (p["latitude"], p["longitude"], p["pop_max"])

    # --- Regionen (M34) ------------------------------------------------------------------
    missing = sorted(iso for iso in series if iso not in names)
    if missing:
        raise SystemExit(f"Ohne Natural-Earth-Gebiet: {missing}")
    region_of = regionen.region_of()
    groups = collections.defaultdict(list)
    for iso in sorted(series):
        groups[region_of.get(iso, iso)].append(iso)
    unknown = sorted(set(region_of) - set(series))
    if unknown:
        raise SystemExit(f"Regionen mit unbekannten Ländern: {unknown}")

    def landlocked_of(iso):
        g = geo.get(iso.lower(), {})
        return g.get("landlocked") == "landlocked" if g else iso == "XKX"

    def capital_of(iso):
        lat, lon = capitals.get(iso, (None, None, None))[:2]
        if lat is None:
            g = geo[iso.lower()]
            lat, lon = float(g["latitude"]), float(g["longitude"])
        return lat, lon

    regions = {}
    for code, isos in groups.items():
        pops = {iso: {y: interpolate(series[iso]["pop"], y) for y in YEARS} for iso in isos}
        pop_y = {y: sum(pops[iso][y] for iso in isos) for y in YEARS}
        weighted = lambda key, factor: {
            y: sum(pops[iso][y] * interpolate(series[iso][key], y) for iso in isos) / pop_y[y] * factor
            for y in YEARS
        }
        largest = max(isos, key=lambda iso: (pops[iso][2000], iso))
        head = code if code in isos else largest
        # Das namensgebende bzw. größte Land zuerst: Alte Spielstände übernehmen dessen
        # Zustand für die Region.
        ordered = [head] + sorted((iso for iso in isos if iso != head), key=lambda iso: (-pops[iso][2000], iso))
        regions[code] = {
            "members": ordered,
            "pop": pop_y,
            "gdp": weighted("gdp", USD_2017_TO_2026),
            "gini": weighted("gini", 0.01),
            "area": sum(area[iso] for iso in isos),
            "capital": capital_of(head),
            "landlocked": all(landlocked_of(iso) for iso in isos),
            "continent": continent_of(*continents[head]),
            "neighbours": sorted({region_of.get(n, n) for iso in isos for n in neighbours[iso]} - {code}),
            "name": regionen.NAMEN.get(code) or german_name(code, names[code]),
            "cities": cities_of(head, ordered, towns),
        }

    # --- Dateien schreiben ---------------------------------------------------------------
    out_dir = os.path.join(ROOT, "data", "laender")
    for old in os.listdir(out_dir):
        if old.endswith(".yaml"):
            os.remove(os.path.join(out_dir, old))
    texts = []
    member_texts = []
    city_texts = []
    for iso in sorted(regions):
        r = regions[iso]
        lat, lon = r["capital"]
        lines = [
            "laender:",
            f"  - id: {iso}",
            f"    kontinent: {r['continent']}",
            f"    flaeche_km2: {group(round(r['area']))}",
            f"    hauptstadt: {{breite: {lat:.2f}, laenge: {lon:.2f}}}",
            f"    binnenland: {'true' if r['landlocked'] else 'false'}",
            f"    nachbarn: [{', '.join(r['neighbours'])}]",
        ]
        if r["cities"]:
            lines.append("    staedte:")
            for c in r["cities"]:
                capital = ", hauptstadt: true" if c["capital"] else ""
                lines.append(
                    f"      - {{id: {c['id']}, einwohner: {group(c['pop'])}, "
                    f"breite: {c['lat']:.2f}, laenge: {c['lon']:.2f}{capital}}}"
                )
                city_texts.append(f"stadt.{iso}.{c['id']}: {c['german']}")
        if len(r["members"]) > 1:
            lines.append(f"    umfasst: [{', '.join(r['members'])}]")
        lines += [
            "    werte:",
            f"      bevoelkerung: {series_yaml(r['pop'], 0)}",
            f"      bip_je_kopf_usd: {series_yaml(r['gdp'], 0)}",
            f"      gini: {series_yaml(r['gini'], 3)}",
        ]
        if iso in CORPORATE_TAX:
            lines.append(f"      steuer_unternehmen: {inline(CORPORATE_TAX[iso], 3)}")
        if iso in STABILITY:
            lines.append(f"      stabilitaet: {inline(STABILITY[iso], 2)}")
        profile = PROFILE.get(iso)
        if profile:
            lines.append("    praegung:")
            if "automatisierung" in profile:
                lines.append(f"      automatisierung: {profile['automatisierung']}")
            for key in ("fachrichtungen", "forschung"):
                if key in profile:
                    items = ", ".join(f"{k}: {v}" for k, v in profile[key].items())
                    lines.append(f"      {key}: {{{items}}}")
        lines.append("    annaeherung: true")
        lines.append(f"    quelle: \"{QUELLE}\"")
        with open(os.path.join(out_dir, f"{iso}.yaml"), "w") as f:
            f.write("\n".join(lines) + "\n")
        texts.append(f"land.{iso}: {r['name']}")
        if len(r["members"]) > 1:
            member_texts += [f"teilland.{m}: {german_name(m, names[m])}" for m in r["members"]]
    with open(os.path.join(ROOT, "data", "texte", "de", "laender.yaml"), "w") as f:
        f.write("# Erzeugt von tools/daten/laender.py – Namen nach Natural Earth (NAME_DE).\n")
        f.write("\n".join(texts) + "\n")
        f.write("\n# Länder innerhalb der Regionen (tools/daten/regionen.py)\n")
        f.write("\n".join(sorted(member_texts)) + "\n")
        f.write("\n# Städte (W2; Natural Earth, deutsche Namen nach STADT_DE)\n")
        f.write("\n".join(city_texts) + "\n")
    print(f"{len(texts)} Länder und Regionen geschrieben.")
    world = lambda y: sum(interpolate(s["pop"], y) for s in series.values()) / 1e9
    print(f"Weltbevölkerung 1900: {world(1900):.2f} Mrd., 1930: {world(1930):.2f} Mrd., 2020: {world(2020):.2f} Mrd.")
    for iso in ("DEU", "GBR", "IRL", "POL", "FRA", "XBA"):
        print(iso, round(regions[iso]["pop"][1900] / 1e6, 1), "Mio. (1900)",
              "Nachbarn:", ", ".join(regions[iso]["neighbours"]))


# Abweichende deutsche Kurznamen
NAME_OVERRIDES = {
    "USA": "Vereinigte Staaten", "GBR": "Vereinigtes Königreich", "CZE": "Tschechien",
    "PSE": "Palästina", "XKX": "Kosovo", "ESH": "Westsahara", "CYP": "Zypern", "SOM": "Somalia",
    "COD": "Demokratische Republik Kongo", "COG": "Republik Kongo", "PRK": "Nordkorea", "KOR": "Südkorea",
    "CHN": "China", "TWN": "Taiwan", "MDA": "Moldau", "FSM": "Mikronesien",
}


def german_name(iso, name):
    return NAME_OVERRIDES.get(iso, name)


# Städte (W2): höchstens so viele je Land oder Region, außer der Hauptstadt nur Orte ab
# dieser Größe.
MAX_STAEDTE = 5
MIN_EINWOHNER = 100_000

# Deutsche Namen der Städte, wo sie vom Namen in Natural Earth abweichen
STADT_DE = {
    "Munich": "München", "Cologne": "Köln", "Nuremberg": "Nürnberg", "Vienna": "Wien",
    "Moscow": "Moskau", "St. Petersburg": "Sankt Petersburg", "Rome": "Rom", "Milan": "Mailand",
    "Naples": "Neapel", "Florence": "Florenz", "Venice": "Venedig", "Genoa": "Genua",
    "Geneva": "Genf", "Warsaw": "Warschau", "Kraków": "Krakau", "Gdańsk": "Danzig",
    "Seville": "Sevilla", "Cairo": "Kairo", "Mexico City": "Mexiko-Stadt", "Prague": "Prag",
    "København": "Kopenhagen", "Lisbon": "Lissabon", "Brussels": "Brüssel", "Athens": "Athen",
    "The Hague": "Den Haag", "Beijing": "Peking", "Tokyo": "Tokio", "Ōsaka": "Osaka",
    "Bur Said": "Port Said", "İzmir": "Izmir", "Belgrade": "Belgrad", "Bucharest": "Bukarest",
    "Kiev": "Kiew", "Kyiv": "Kiew", "Algiers": "Algier", "Damascus": "Damaskus",
    "Baghdad": "Bagdad", "Tehran": "Teheran", "Riyadh": "Riad", "Singapore": "Singapur",
    "Ho Chi Minh City": "Ho-Chi-Minh-Stadt", "Pyongyang": "Pjöngjang", "Hong Kong": "Hongkong",
    "Havana": "Havanna", "Panama City": "Panama-Stadt", "Guatemala": "Guatemala-Stadt",
    "Addis Ababa": "Addis Abeba", "Cape Town": "Kapstadt", "Tripoli": "Tripolis",
    "Khartoum": "Khartum", "Karachi": "Karatschi", "Tashkent": "Taschkent",
    "Antwerp": "Antwerpen", "Gothenburg": "Göteborg", "Luxembourg": "Luxemburg",
    "Nicosia": "Nikosia", "Tbilisi": "Tiflis", "Yerevan": "Jerewan", "Mecca": "Mekka",
    "Jeddah": "Dschidda", "Kuwait": "Kuwait-Stadt", "Calcutta": "Kalkutta",
    "Saint Petersburg": "Sankt Petersburg", "Thessaloniki": "Thessaloniki",
}


def city_id(german, taken):
    """ID aus dem deutschen Namen: snake_case ohne Umlaute (München → muenchen)."""
    text = german.lower()
    for a, b in (("ä", "ae"), ("ö", "oe"), ("ü", "ue"), ("ß", "ss")):
        text = text.replace(a, b)
    text = unicodedata.normalize("NFKD", text).encode("ascii", "ignore").decode()
    base = re.sub(r"[^a-z0-9]+", "_", text).strip("_") or "stadt"
    key, n = base, 2
    while key in taken:
        key, n = f"{base}_{n}", n + 1
    taken.add(key)
    return key


def cities_of(head, members, towns):
    """Die Hauptstadt des namensgebenden Landes und die größten Orte aller Mitglieder."""
    all_towns = [t for iso in members for t in towns.get(iso, [])]
    chosen = []
    capital = max((t for t in towns.get(head, []) if t["capital"]), key=lambda t: t["pop"], default=None)
    if capital:
        chosen.append(dict(capital, capital=True))
    for t in sorted(all_towns, key=lambda t: (-t["pop"], t["ascii"])):
        if len(chosen) >= MAX_STAEDTE:
            break
        if t["pop"] < MIN_EINWOHNER and chosen:
            break
        if capital and t is capital:
            continue
        if any(c["ascii"] == t["ascii"] for c in chosen):
            continue
        chosen.append(dict(t, capital=False))
    chosen.sort(key=lambda t: (-t["pop"], t["ascii"]))
    taken = set()
    for c in chosen:
        c["german"] = STADT_DE.get(c["name"], c["name"])
        c["id"] = city_id(c["german"], taken)
    return chosen


def continent_of(continent, region, subregion):
    if continent == "South America":
        return "suedamerika"
    if continent == "North America":
        return "nordamerika"
    if continent == "Europe":
        return "europa"
    if continent == "Africa":
        return "afrika"
    if continent == "Oceania":
        return "ozeanien"
    if continent == "Asia":
        return "asien"
    # Inselstaaten im offenen Ozean (Malediven, Mauritius, Seychellen …)
    return {"Africa": "afrika", "Asia": "asien", "Oceania": "ozeanien", "Americas": "nordamerika"}[region]


def group(n):
    return f"{n:_}"


def number(value, decimals):
    if decimals == 0:
        return group(int(round(value)))
    return f"{value:.{decimals}f}".rstrip("0").rstrip(".")


def series_yaml(points, decimals):
    """Zeitreihe als mehrzeilige Flow-Map, 8 Jahre je Zeile."""
    items = [f"{y}: {number(v, decimals)}" for y, v in sorted(points.items())]
    rows = [", ".join(items[i:i + 8]) for i in range(0, len(items), 8)]
    return "{\n        " + ",\n        ".join(rows) + "}"


def inline(points, decimals):
    return "{" + ", ".join(f"{y}: {number(v, decimals)}" for y, v in sorted(points.items())) + "}"


if __name__ == "__main__":
    main()
