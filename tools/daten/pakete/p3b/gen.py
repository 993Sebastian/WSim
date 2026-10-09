"""Schreibt die Daten von P3b (Kette 56, Lagerstätten, Texte) aus spec.py.

Aufruf aus dem Projektordner: python3 tools/daten/pakete/p3b/gen.py
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
 ("luftzerlegung","Luftverflüssigung nach Linde","chemie",1895,[],8000),
 ("starrluftschiff","Starrluftschiff","maschinenbau",1908,["ottomotor","luftzerlegung"],15000),
 ("fallschirm","Rucksackfallschirm","textil",1912,[],4000),
 ("wasserflugzeug","Wasserflugzeug","maschinenbau",1912,["flugmotor"],10000),
 ("bomber","Mehrmotoriges Großflugzeug","maschinenbau",1915,["flugmotor"],15000),
 ("segelflug","Segelflugzeug","holz_bau",1920,[],4000),
 ("leichtflugzeug","Leichtflugzeug in Serie","maschinenbau",1927,["ganzmetallflugzeug"],8000),
 ("radiosonde","Radiosonde","elektro",1929,["elektronenroehre"],5000),
 ("modellflug","Motorflugmodell","maschinenbau",1930,[],3000),
 ("radar","Funkmessung (Radar)","elektro",1935,["elektronenroehre"],25000),
 ("fluessigkeitsrakete","Flüssigkeitsrakete","maschinenbau",1942,["luftzerlegung"],40000),
 ("hubschrauber","Hubschrauber","maschinenbau",1944,["flugmotor"],25000),
 ("propellerturbine","Propellerturbine","maschinenbau",1945,["strahltriebwerk"],20000),
 ("hoehenforschungsrakete","Höhenforschungsrakete","maschinenbau",1946,["fluessigkeitsrakete"],10000),
 ("kroll_verfahren","Titan nach Kroll","metall",1948,[],20000),
 ("turbopropflugzeug","Turboprop-Verkehrsflugzeug","maschinenbau",1953,["propellerturbine"],25000),
 ("militaertransporter","Militärtransporter","maschinenbau",1954,["turbopropflugzeug"],20000),
 ("bordelektronik","Bordelektronik","elektro",1955,["transistor","radar"],20000),
 ("frachtflugzeug","Frachtflugzeug","maschinenbau",1955,["turbopropflugzeug"],15000),
 ("satellitentechnik","Satellitentechnik","elektro",1957,["fluessigkeitsrakete","bordelektronik"],50000),
 ("traegerrakete","Mehrstufige Trägerrakete","maschinenbau",1957,["fluessigkeitsrakete","bordelektronik"],60000),
 ("fluggastbruecke","Fluggastbrücke","maschinenbau",1959,["duesenverkehrsflugzeug"],5000),
 ("mantelstromtriebwerk","Mantelstromtriebwerk","maschinenbau",1960,["strahltriebwerk","kroll_verfahren"],50000),
 ("heissluftballon","Moderner Heißluftballon","textil",1960,["polyamid"],4000),
 ("wettersatellit","Wettersatellit","elektro",1960,["satellitentechnik"],20000),
 ("raumsonde","Raumsonde","elektro",1962,["satellitentechnik","traegerrakete"],40000),
 ("geschaeftsreisejet","Geschäftsreisejet","maschinenbau",1963,["duesenverkehrsflugzeug"],15000),
 ("kohlenstofffaser","Kohlenstofffaser","chemie",1963,["dampfspaltung"],20000),
 ("kommunikationssatellit","Geostationärer Nachrichtensatellit","elektro",1965,["satellitentechnik"],30000),
 ("flugsimulator","Digitaler Flugsimulator","elektro",1965,["integrierte_schaltung"],15000),
 ("kampfhubschrauber","Kampfhubschrauber","maschinenbau",1967,["hubschrauber","propellerturbine","bordelektronik"],20000),
 ("grossraumflugzeug","Großraumflugzeug","maschinenbau",1970,["mantelstromtriebwerk","duesenverkehrsflugzeug"],80000),
 ("erdbeobachtung","Erdbeobachtung aus dem All","elektro",1972,["satellitentechnik"],25000),
 ("marschflugkoerper","Marschflugkörper","maschinenbau",1975,["strahltriebwerk","bordelektronik"],30000),
 ("satellitennavigation","Satellitennavigation","elektro",1978,["satellitentechnik","integrierte_schaltung"],40000),
 ("ultraleichtflugzeug","Ultraleichtflugzeug","maschinenbau",1979,["leichtflugzeug"],5000),
 ("gleitschirm","Gleitschirm","textil",1985,["polyamid"],3000),
 ("satellitenfernsehen","Satellitenfernsehen","elektro",1988,["kommunikationssatellit"],10000),
 ("regionaljet","Regionaljet","maschinenbau",1992,["mantelstromtriebwerk"],30000),
 ("navigationsgeraet","Navigationsgerät","elektro",1995,["satellitennavigation","tft_lcd"],10000),
 ("militaerdrohne","Militärdrohne","elektro",1995,["mikroprozessor","kohlenstofffaser"],30000),
 ("satellitentelefonie","Satellitentelefonie","elektro",1998,["kommunikationssatellit","mobilfunk"],15000),
 ("kleinsatellit","Kleinsatellit","elektro",2003,["satellitentechnik","lithium_ionen_akku"],10000),
 ("multikopter","Multikopter","elektro",2010,["lithium_ionen_akku","mikroprozessor"],10000),
 ("wiederverwendbare_rakete","Wiederverwendbare Trägerrakete","maschinenbau",2015,["traegerrakete","kohlenstofffaser"],80000),
]
TECH_FILE = {}

FILES = [("56_luft_raumfahrt.yaml", "Kette 56 – Luft- und Raumfahrt (P3b)", spec.P3B)]

generate("p3b", FILES, TECH, TECH_FILE, kal,
         "# P3b – Produktbreite (erzeugt, Kette 56)", "# P3b – Lagerstätten der neuen Rohstoffe", dep_note="P3b")
