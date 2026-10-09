"""Schreibt die Daten von P5 (Kette 59, Lagerstätten, Texte) aus spec.py.

Aufruf aus dem Projektordner: python3 tools/daten/pakete/p5/gen.py
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
 ("pluesch_spielzeug","Plüschtier","textil",1903,[],2000),
 ("wasserkraftturbine","Moderne Wasserkraftturbine","maschinenbau",1905,[],10000),
 ("roentgentechnik","Röntgentechnik","elektro",1905,[],10000),
 ("blutdruckmessung","Blutdruckmessgerät","elektro",1905,[],3000),
 ("verstellbares_krankenbett","Verstellbares Krankenbett","maschinenbau",1909,[],3000),
 ("elektrokardiograf","Elektrokardiograf","elektro",1910,[],8000),
 ("belebtschlamm","Belebtschlammverfahren","chemie",1914,[],10000),
 ("rettungswagen","Rettungswagen","maschinenbau",1925,["lieferwagen"],4000),
 ("oelfeuerung","Ölfeuerung","maschinenbau",1930,[],5000),
 ("faltrollstuhl","Faltrollstuhl","maschinenbau",1933,[],3000),
 ("gasturbinenkraftwerk","Gasturbinenkraftwerk","maschinenbau",1939,["strahltriebwerk"],30000),
 ("elektrogitarre","E-Gitarre","elektro",1950,["dynamischer_lautsprecher"],3000),
 ("beatmung","Beatmungsgerät","elektro",1952,[],8000),
 ("transistorhoergeraet","Transistor-Hörgerät","elektro",1952,["transistor"],5000),
 ("kernkraftwerk","Kernkraftwerk","chemie",1956,["kroll_verfahren"],80000),
 ("einwegspritze","Einwegspritze","chemie",1956,["spritzguss"],3000),
 ("kunststoff_surfbrett","Kunststoff-Surfbrett","chemie",1958,["polyesterfaser"],2000),
 ("muellverbrennung","Moderne Müllverbrennung","maschinenbau",1960,[],15000),
 ("herzschrittmacher","Herzschrittmacher","elektro",1960,["transistor","kroll_verfahren"],15000),
 ("hueftendoprothese","Hüftendoprothese","metall",1962,["kroll_verfahren"],10000),
 ("haemodialyse","Hämodialyse","elektro",1964,["integrierte_schaltung"],12000),
 ("synthesizer","Synthesizer","elektro",1964,["transistor"],5000),
 ("gasheizkessel","Gasheizkessel","maschinenbau",1965,[],5000),
 ("sonografie","Sonografie","elektro",1970,["integrierte_schaltung"],15000),
 ("computertomografie","Computertomografie","elektro",1972,["integrierte_schaltung"],30000),
 ("rauchgasentschwefelung","Rauchgasentschwefelung","chemie",1974,[],15000),
 ("solarthermie","Solarthermie","maschinenbau",1975,[],5000),
 ("kraft_waerme_kopplung","Blockheizkraftwerk","maschinenbau",1975,[],8000),
 ("abgaskatalysator","Abgaskatalysator","chemie",1975,["automobil"],10000),
 ("magnetresonanz","Magnetresonanztomografie","elektro",1980,["computertomografie"],40000),
 ("zahnimplantat","Zahnimplantat","metall",1982,["kroll_verfahren"],6000),
 ("snowboard","Snowboard","holz_bau",1985,[],2000),
 ("biogas","Biogasanlage","chemie",1990,[],8000),
 ("laufband","Heimtrainer-Laufband","maschinenbau",1990,["mikroprozessor"],4000),
 ("pelletfeuerung","Pelletheizung","maschinenbau",1997,["mikroprozessor"],5000),
 ("netzwechselrichter","Netzwechselrichter","elektro",2000,["mikroprozessor"],8000),
 ("pedelec","Elektrofahrrad","elektro",2008,["lithium_ionen_akku"],6000),
 ("ladestation","Ladestation","elektro",2011,["elektroauto"],5000),
 ("heimspeicher","Heimspeicher","elektro",2013,["traktionsbatterie"],8000),
 ("virtuelle_realitaet","Virtuelle Realität","elektro",2016,["smartphone"],10000),
 ("pem_elektrolyse","PEM-Elektrolyse","chemie",2018,[],30000),
]
TECH_FILE = {}

FILES = [("59_energie_medizin_freizeit.yaml", "Kette 59 – Energie- und Umwelttechnik, Medizintechnik, Freizeit (P5)", spec.P5)]

generate("p5", FILES, TECH, TECH_FILE, kal,
         "# P5 – Produktbreite (erzeugt, Kette 59)", "# P5 – Lagerstätten der neuen Rohstoffe", dep_note="P5")
