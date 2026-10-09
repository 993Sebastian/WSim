"""Schreibt die Daten von P1 (Ketten 50–52, Lagerstätten, Texte) aus spec.py.

Aufruf aus dem Projektordner: python3 tools/daten/pakete/p1/gen.py
Die Arbeitsstunden-Faktoren und angepassten Richtpreise stehen in kalib.json; sie schreibt
kalib.py (danach gen.py erneut), bis `wsim validate data` ohne Margenwarnung endet.
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
 ("spruehtrocknung","Sprühtrocknung","lebensmittel",1901,[],5000),
 ("fetthaertung","Fetthärtung","chemie",1902,[],10000),
 ("flaschenblasmaschine","Flaschenblasmaschine","chemie",1903,[],10000),
 ("thermosflasche","Isolierflasche","chemie",1904,[],5000),
 ("bruehwuerfel","Brühwürfel","lebensmittel",1908,[],5000),
 ("alufolie","Aluminiumfolie","metall",1910,["schmelzflusselektrolyse"],8000),
 ("rostfreier_stahl","Rostfreier Stahl","metall",1912,["elektrostahl"],15000),
 ("gipskarton","Gipskarton","holz_bau",1916,[],5000),
 ("sportschuh","Sportschuh mit Gummisohle","textil",1917,["vulkanisation"],5000),
 ("gummistiefel","Gummistiefel aus der Form","chemie",1920,["vulkanisation"],5000),
 ("industriejoghurt","Industrielle Joghurtherstellung","lebensmittel",1919,[],5000),
 ("papiertaschentuch","Papiertaschentuch","holz_bau",1924,["sulfitverfahren"],5000),
 ("knabbergebaeck","Industrielle Knabberartikel","lebensmittel",1926,[],5000),
 ("industrielles_speiseeis","Industrielles Speiseeis","lebensmittel",1926,["kompressionskuehlschrank"],8000),
 ("einbaukueche","Einbauküche","holz_bau",1926,["holzbearbeitungsmaschinen"],6000),
 ("transportbeton","Transportbeton","holz_bau",1926,["lastkraftwagen"],6000),
 ("tiefkuehlkost","Tiefkühlkost","lebensmittel",1930,["kompressionskuehlschrank"],15000),
 ("mineralwolle","Mineralwolle","chemie",1930,[],8000),
 ("kuechenrolle","Küchenrolle","holz_bau",1931,["sulfitverfahren"],5000),
 ("loeslicher_kaffee","Löslicher Kaffee","lebensmittel",1938,["spruehtrocknung"],12000),
 ("spanplatte","Spanplatte","holz_bau",1941,["phenolharz"],10000),
 ("frischhaltedose","Frischhaltedose","chemie",1946,["spritzguss"],6000),
 ("frischhaltefolie","Frischhaltefolie","chemie",1949,["hochdruck_polyethylen"],8000),
 ("gefluegelmast","Geflügelmast","lebensmittel",1950,[],8000),
 ("betonfertigteil","Betonfertigteile","holz_bau",1950,[],10000),
 ("fertiggerichte","Fertiggerichte","lebensmittel",1953,["tiefkuehlkost"],10000),
 ("polyesterfaser","Polyesterfaser","chemie",1953,["dampfspaltung","polyamid"],30000),
 ("antihaftbeschichtung","Antihaftbeschichtung","chemie",1954,["hochdruck_polyethylen"],10000),
 ("flachpaketmoebel","Möbel im Flachpaket","holz_bau",1956,["spanplatte"],6000),
 ("floatglas","Floatglas","chemie",1959,[],30000),
 ("strumpfhose","Strumpfhose","textil",1959,["polyamid"],8000),
 ("wegwerfwindel","Wegwerfwindel","textil",1961,["hochdruck_polyethylen"],12000),
 ("tiefkuehlpizza","Tiefkühlpizza","lebensmittel",1962,["fertiggerichte"],8000),
 ("kunststoff_gartenmoebel","Kunststoff-Gartenmöbel","chemie",1965,["spritzguss"],6000),
 ("funktionsfaser","Funktionsfaser","textil",1975,["polyesterfaser"],15000),
 ("atmungsaktive_membran","Atmungsaktive Membran","chemie",1976,["polyesterfaser"],25000),
 ("laminat","Laminatboden","holz_bau",1977,["spanplatte"],10000),
 ("kapselkaffee","Kapselkaffee","lebensmittel",1986,["spritzguss"],20000),
 ("energiegetraenk","Energiegetränk","lebensmittel",1987,[],8000),
 ("pflanzendrink","Pflanzendrink","lebensmittel",1995,[],10000),
 ("pflanzliches_fleisch","Pflanzliches Fleisch","lebensmittel",2016,["pflanzendrink"],30000),
]
TECH_FILE = {"spruehtrocknung":0,"fetthaertung":0,"bruehwuerfel":0,"industriejoghurt":0,"knabbergebaeck":0,"industrielles_speiseeis":0,"tiefkuehlkost":0,"loeslicher_kaffee":0,"gefluegelmast":0,"fertiggerichte":0,"tiefkuehlpizza":0,"kapselkaffee":0,"energiegetraenk":0,"pflanzendrink":0,"pflanzliches_fleisch":0,"flaschenblasmaschine":0,
 "sportschuh":1,"gummistiefel":1,"polyesterfaser":1,"strumpfhose":1,"wegwerfwindel":1,"funktionsfaser":1,"atmungsaktive_membran":1}

FILES = [("50_nahrung_genuss.yaml", "Kette 50 – Lebensmittel, Getränke und Tabak (P1a)", spec.P1A),
         ("51_bekleidung_leder.yaml", "Kette 51 – Textil, Bekleidung, Schuhe und Leder (P1b)", spec.P1B),
         ("52_wohnen_bauen.yaml", "Kette 52 – Möbel, Haushaltswaren und Baustoffe (P1c)", spec.P1C)]

generate("p1", FILES, TECH, TECH_FILE, kal,
         "# P1 – Produktbreite (erzeugt, Ketten 50–52)", "# P1 – Lagerstätten der neuen Rohstoffe")
