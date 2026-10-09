"""Schreibt die Daten von P4a (Kette 57, Lagerstätten, Texte) aus spec.py.

Aufruf aus dem Projektordner: python3 tools/daten/pakete/p4a/gen.py
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
 ("elektrische_beleuchtung","Elektrische Wohnraumbeleuchtung","elektro",1900,[],0),
 ("taschenlampe","Taschenlampe","elektro",1899,[],0),
 ("elektrisches_buegeleisen","Elektrisches Bügeleisen","elektro",1907,["heizleiter"],5000),
 ("heizleiter","Chromnickel-Heizleiter","metall",1906,[],8000),
 ("toaster","Toaster","elektro",1909,["heizleiter"],4000),
 ("tischventilator","Tischventilator","elektro",1902,[],3000),
 ("heizdecke","Heizdecke","elektro",1912,["heizleiter"],3000),
 ("hochspannungstechnik","Hochspannungstechnik","elektro",1912,[],20000),
 ("elektroherd","Elektroherd","elektro",1913,["heizleiter"],6000),
 ("lichterkette","Elektrische Lichterkette","elektro",1917,[],2000),
 ("kuechenmaschine","Küchenmaschine","elektro",1919,[],5000),
 ("haartrockner","Haartrockner","elektro",1920,["phenolharz","heizleiter"],4000),
 ("heizluefter","Heizlüfter","elektro",1925,["heizleiter"],3000),
 ("lichtbogenschweissen","Lichtbogenschweißen","elektro",1920,[],8000),
 ("warmwasserspeicher","Elektrischer Warmwasserspeicher","elektro",1920,["heizleiter"],4000),
 ("wasserkocher","Elektrischer Wasserkocher","elektro",1922,["heizleiter"],3000),
 ("schutzkontakt","Schutzkontaktsteckdose","elektro",1926,["phenolharz"],3000),
 ("notstromaggregat","Notstromaggregat","maschinenbau",1930,[],6000),
 ("trockenrasierer","Elektrischer Trockenrasierer","elektro",1931,["phenolharz"],5000),
 ("standmixer","Standmixer","elektro",1937,[],4000),
 ("leuchtstofflampe","Leuchtstofflampe","elektro",1938,[],10000),
 ("gefriergeraet","Gefriergerät","elektro",1940,["kompressionskuehlschrank"],8000),
 ("geschirrspueler","Geschirrspüler","elektro",1950,["waschvollautomat"],10000),
 ("raumklimageraet","Raumklimagerät","elektro",1950,["kompressionskuehlschrank"],10000),
 ("elektrische_zahnbuerste","Elektrische Zahnbürste","elektro",1954,["spritzguss"],4000),
 ("kaffeemaschine","Filterkaffeemaschine","elektro",1954,["spritzguss","heizleiter"],4000),
 ("waeschetrockner","Wäschetrockner","elektro",1955,["waschvollautomat"],8000),
 ("halogenlampe","Halogenlampe","elektro",1959,["wolframgluehlampe"],6000),
 ("dunstabzug","Dunstabzugshaube","elektro",1960,[],3000),
 ("nachtspeicherheizung","Nachtspeicherheizung","elektro",1962,["heizleiter"],5000),
 ("elektrogrill","Elektrogrill","elektro",1965,["heizleiter"],3000),
 ("fritteuse","Elektrische Fritteuse","elektro",1976,["spritzguss"],3000),
 ("kompaktleuchtstofflampe","Energiesparlampe","elektro",1980,["leuchtstofflampe","transistor"],10000),
 ("luftreiniger","Luftreiniger","elektro",1985,["integrierte_schaltung"],4000),
 ("brotbackautomat","Brotbackautomat","elektro",1986,["mikroprozessor"],4000),
 ("induktionskochfeld","Induktionskochfeld","elektro",1987,["mikroprozessor"],8000),
 ("akkuschrauber","Akkuschrauber","elektro",1992,["lithium_ionen_akku"],5000),
 ("saugroboter","Saugroboter","elektro",2002,["mikroprozessor","lithium_ionen_akku"],10000),
 ("rolltreppe","Rolltreppe","maschinenbau",1911,["drehstrommotor"],6000),
 ("grosstransformator","Großtransformator","elektro",1904,["drehstrommotor"],10000),
]
TECH_FILE = {}

FILES = [("57_elektro_haushalt.yaml", "Kette 57 – Elektrotechnik und Haushaltsgeräte (P4a)", spec.P4A)]

generate("p4a", FILES, TECH, TECH_FILE, kal,
         "# P4a – Produktbreite (erzeugt, Kette 57)", "# P4a – Lagerstätten der neuen Rohstoffe", dep_note="P4a")
