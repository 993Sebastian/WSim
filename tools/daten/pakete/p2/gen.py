"""Schreibt die Daten von P2 (Ketten 53–54, Lagerstätten, Texte) aus spec.py.

Aufruf aus dem Projektordner: python3 tools/daten/pakete/p2/gen.py
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
 ("wegwerfrasierklinge","Wegwerf-Rasierklinge","metall",1904,[],5000),
 ("waschpulver","Selbsttätiges Waschpulver","chemie",1907,[],6000),
 ("emulsionscreme","Emulsionscreme","chemie",1911,[],5000),
 ("lippenstifthuelse","Lippenstift in der Hülse","chemie",1915,[],5000),
 ("handbohrmaschine","Elektrische Handbohrmaschine","maschinenbau",1917,["elektrischer_einzelantrieb"],8000),
 ("motorrasenmaeher","Motorrasenmäher","maschinenbau",1919,["ottomotor"],6000),
 ("impfstoffproduktion","Industrielle Impfstoffherstellung","chemie",1921,[],12000),
 ("insulin","Insulin","chemie",1922,[],15000),
 ("armbanduhr","Armbanduhr mit Automatik","metall",1923,[],6000),
 ("gabelstapler","Gabelstapler","maschinenbau",1923,["lastkraftwagen"],8000),
 ("fluessigshampoo","Flüssiges Shampoo","chemie",1927,[],5000),
 ("polystyrol","Polystyrol","chemie",1930,[],20000),
 ("pvc_herstellung","PVC-Herstellung","chemie",1931,[],20000),
 ("synthetische_tenside","Synthetische Tenside","chemie",1932,[],15000),
 ("alleskleber","Alleskleber","chemie",1932,[],5000),
 ("vitaminsynthese","Vitaminsynthese","chemie",1935,[],12000),
 ("haushaltsreiniger","Flüssige Haushaltsreiniger","chemie",1935,["synthetische_tenside"],5000),
 ("kunststoffrohr","Kunststoffrohre","chemie",1936,["pvc_herstellung"],8000),
 ("maehdrescher","Selbstfahrender Mähdrescher","maschinenbau",1938,["ackerschlepper"],12000),
 ("synthetische_insektizide","Synthetische Insektizide","chemie",1939,[],15000),
 ("pvc_schlauch","Kunststoffschlauch","chemie",1940,["pvc_herstellung"],5000),
 ("vinylschallplatte","Vinyl-Schallplatte","chemie",1948,["pvc_herstellung"],6000),
 ("fluessiges_spuelmittel","Flüssiges Spülmittel","chemie",1950,["synthetische_tenside"],5000),
 ("kunststoffbodenbelag","Kunststoff-Bodenbelag","chemie",1950,["pvc_herstellung"],6000),
 ("kunststoffspielzeug","Kunststoffspielzeug","chemie",1950,["spritzguss","polystyrol"],6000),
 ("motorkettensaege","Motorkettensäge","maschinenbau",1950,["ottomotor"],8000),
 ("verpackungsautomat","Verpackungsautomat","maschinenbau",1950,["elektrischer_einzelantrieb"],10000),
 ("expandiertes_polystyrol","Expandiertes Polystyrol","chemie",1951,["polystyrol"],8000),
 ("deoroller","Deoroller","chemie",1952,["hochdruck_polyethylen"],5000),
 ("polypropylen","Polypropylen","chemie",1957,["dampfspaltung"],25000),
 ("hormonpraeparate","Hormonpräparate","chemie",1960,[],20000),
 ("muellbeutel","Müllbeutel","chemie",1960,["hochdruck_polyethylen"],4000),
 ("industrieroboter","Industrieroboter","maschinenbau",1961,["transistor"],30000),
 ("betablocker","Betablocker","chemie",1964,[],25000),
 ("plastiktuete","Kunststoff-Tragetasche","chemie",1965,["hochdruck_polyethylen"],4000),
 ("quarzuhr","Quarzuhr","elektro",1969,["integrierte_schaltung"],15000),
 ("cnc_steuerung","CNC-Steuerung","maschinenbau",1975,["mikroprozessor"],30000),
 ("gentechnische_wirkstoffe","Gentechnische Wirkstoffe","chemie",1982,["insulin"],60000),
]
TECH_FILE = {t[0]: 0 for t in TECH if t[2] == "chemie"}

FILES = [("53_chemie_pharma.yaml", "Kette 53 – Chemie, Pharma, Kosmetik und Kunststoffwaren (P2a)", spec.P2A),
         ("54_metall_maschinen.yaml", "Kette 54 – Metallwaren, Werkzeuge und Maschinen (P2b)", spec.P2B)]

generate("p2", FILES, TECH, TECH_FILE, kal,
         "# P2 – Produktbreite (erzeugt, Ketten 53–54)", "# P2 – Lagerstätten der neuen Rohstoffe", dep_note="P2")
