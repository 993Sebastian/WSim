"""Schreibt die Daten von P3a (Kette 55, Lagerstätten, Texte) aus spec.py.

Aufruf aus dem Projektordner: python3 tools/daten/pakete/p3a/gen.py
Abstimmung wie bei P1: kalib.py schreibt kalib.json, danach gen.py erneut.
Änderungen an den Waren gehören in spec.py, nicht in die erzeugten YAML-Dateien.
"""
import json, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
sys.path.insert(0, str(Path(__file__).parent.parent))
import spec
from generator import generate

KALIB = Path(__file__).parent / "kalib.json"
kal = json.loads(KALIB.read_text()) if KALIB.exists() else {"arbeit": {}, "preis": {}}

TECH = [  # id, name, fachgebiet, jahr, voraussetzungen, aufwand
 ("elektrolokomotive","Elektrolokomotive","elektro",1905,["elektrischer_einzelantrieb"],20000),
 ("motorschiff","Motorschiff","maschinenbau",1912,[],25000),
 ("elektrischer_anlasser","Elektrischer Anlasser","elektro",1912,["automobil"],8000),
 ("omnibus","Omnibus","maschinenbau",1919,["lastkraftwagen"],8000),
 ("lieferwagen","Lieferwagen","maschinenbau",1925,["lastkraftwagen"],8000),
 ("diesellokomotive","Diesellokomotive","maschinenbau",1935,["motorschiff"],25000),
 ("kleinwagen","Kleinwagen aus Großserie","maschinenbau",1938,["fliessband","ganzstahlkarosserie"],15000),
 ("motorroller","Motorroller","maschinenbau",1946,[],6000),
 ("sportwagen","Sportwagen","maschinenbau",1950,["ganzstahlkarosserie"],10000),
 ("sportboot","Kunststoffboot","maschinenbau",1955,["polyesterfaser"],8000),
 ("wohnmobil","Wohnmobil","maschinenbau",1956,["lieferwagen"],6000),
 ("containerschiff","Containerschiff","maschinenbau",1956,["motorschiff"],30000),
 ("kreuzfahrtschiff","Kreuzfahrtschiff","maschinenbau",1966,["motorschiff"],20000),
 ("hochgeschwindigkeitszug","Hochgeschwindigkeitszug","elektro",1964,["elektrolokomotive"],60000),
 ("suv","Geländelimousine","maschinenbau",1990,["ganzstahlkarosserie"],12000),
 ("hybridantrieb","Hybridantrieb","maschinenbau",1997,["lithium_ionen_akku"],40000),
]
TECH_FILE = {}

FILES = [("55_fahrzeuge.yaml", "Kette 55 – Fahrzeuge, Bahn und Schiffe (P3a)", spec.P3A)]

generate("p3a", FILES, TECH, TECH_FILE, kal,
         "# P3a – Produktbreite (erzeugt, Kette 55)", "# P3a – Lagerstätten der neuen Rohstoffe", dep_note="P3a")
