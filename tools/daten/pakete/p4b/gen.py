"""Schreibt die Daten von P4b (Kette 58, Texte) aus spec.py.

Aufruf aus dem Projektordner: python3 tools/daten/pakete/p4b/gen.py
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
 ("sprossenradrechner","Sprossenradrechner in Serie","maschinenbau",1905,[],4000),
 ("tabelliermaschine","Lochkarten-Tabelliermaschine","elektro",1906,["drehstrommotor"],8000),
 ("waehlamt","Selbstwählamt","elektro",1910,[],15000),
 ("dynamischer_lautsprecher","Dynamischer Lautsprecher","elektro",1925,["elektronenroehre"],6000),
 ("musikautomat","Musikautomat","elektro",1927,["dynamischer_lautsprecher"],4000),
 ("fernschreiber","Fernschreiber","elektro",1930,["elektronenroehre"],8000),
 ("autoradio","Autoradio","elektro",1930,["rundfunk"],5000),
 ("sprechfunk","Tragbarer Sprechfunk","elektro",1940,["elektronenroehre"],8000),
 ("elektrischer_plattenspieler","Elektrischer Plattenspieler","elektro",1948,["dynamischer_lautsprecher"],5000),
 ("tonband","Tonbandgerät","elektro",1950,["elektronenroehre"],8000),
 ("leiterplatte","Gedruckte Schaltung","elektro",1950,[],8000),
 ("hifi_stereo","Stereo-Hi-Fi","elektro",1958,["transistor"],6000),
 ("xerografie","Xerografie","elektro",1959,["transistor"],12000),
 ("kompaktkassette","Kompaktkassette","elektro",1963,["transistor","spritzguss"],6000),
 ("grossrechner","Großrechner in Serie","elektro",1964,["integrierte_schaltung"],40000),
 ("geldautomat","Geldautomat","elektro",1967,["integrierte_schaltung"],10000),
 ("sps","Speicherprogrammierbare Steuerung","elektro",1969,["integrierte_schaltung"],12000),
 ("einbruchmeldeanlage","Elektronische Alarmanlage","elektro",1970,["integrierte_schaltung"],5000),
 ("videoueberwachung","Videoüberwachung","elektro",1973,["integrierte_schaltung"],6000),
 ("lichtwellenleiter","Lichtwellenleiter","elektro",1977,["reinstsilizium"],20000),
 ("videospielkonsole","Videospielkonsole","elektro",1977,["mikroprozessor"],10000),
 ("tragbarer_kassettenspieler","Tragbarer Kassettenspieler","elektro",1979,["kompaktkassette"],5000),
 ("telefax","Telefax","elektro",1980,["integrierte_schaltung"],8000),
 ("festplatte","Festplattenlaufwerk","elektro",1980,["integrierte_schaltung"],20000),
 ("computermonitor","Computerbildschirm","elektro",1981,["personal_computer"],5000),
 ("anrufbeantworter","Anrufbeantworter","elektro",1983,["integrierte_schaltung"],4000),
 ("camcorder","Videokamera","elektro",1983,["videorekorder"],10000),
 ("funkruf","Funkruf (Pager)","elektro",1984,["integrierte_schaltung"],5000),
 ("laserdrucker","Laserdrucker","elektro",1984,["mikroprozessor"],10000),
 ("flachbettscanner","Flachbettscanner","elektro",1987,["mikroprozessor"],5000),
 ("flashspeicher","Flash-Speicher","elektro",1988,["integrierte_schaltung"],20000),
 ("handheld_konsole","Tragbare Spielkonsole","elektro",1989,["mikroprozessor"],6000),
 ("schnurlostelefon","Schnurlostelefon","elektro",1990,["integrierte_schaltung"],5000),
 ("videoprojektor","Videoprojektor","elektro",1992,["tft_lcd"],8000),
 ("server","Serverrechner","elektro",1995,["mikroprozessor"],15000),
 ("mp3_spieler","MP3-Spieler","elektro",1998,["flashspeicher"],5000),
 ("dsl_router","DSL-Router","elektro",1999,["mikroprozessor"],5000),
 ("usb_speicher","USB-Speicher","elektro",2000,["flashspeicher"],3000),
 ("externe_festplatte","Externe Festplatte","elektro",2003,["festplatte"],3000),
 ("blu_ray","Blu-ray","elektro",2006,["dvd"],8000),
 ("elektronisches_papier","Elektronisches Papier","elektro",2007,["tft_lcd"],8000),
 ("fitnessarmband","Fitnessarmband","elektro",2013,["smartphone"],4000),
 ("sprachassistent","Sprachassistent","elektro",2014,["smartphone"],10000),
 ("smartwatch","Smartwatch","elektro",2015,["smartphone"],8000),
 ("funkkopfhoerer","Funkkopfhörer","elektro",2016,["smartphone"],5000),
]
TECH_FILE = {}

FILES = [("58_elektronik_it.yaml", "Kette 58 – Elektronik, IT und Kommunikation (P4b)", spec.P4B)]

generate("p4b", FILES, TECH, TECH_FILE, kal,
         "# P4b – Produktbreite (erzeugt, Kette 58)", "# P4b – Lagerstätten der neuen Rohstoffe", dep_note="P4b")
