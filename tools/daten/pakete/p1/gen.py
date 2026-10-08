"""Schreibt die Daten von P1 (Ketten 50–52, Lagerstätten, Texte) aus spec.py.

Aufruf aus dem Projektordner: python3 tools/daten/pakete/p1/gen.py
Die Arbeitsstunden-Faktoren und angepassten Richtpreise stehen in kalib.json; sie schreibt
kalib.py (danach gen.py erneut), bis `wsim validate data` ohne Margenwarnung endet.
Änderungen an den Waren gehören in spec.py, nicht in die erzeugten YAML-Dateien.
"""
import json, math, sys
from pathlib import Path
import yaml
sys.path.insert(0, str(Path(__file__).parent))
import spec

ROOT = Path(__file__).resolve().parents[4] / "data"
KALIB = Path(__file__).parent / "kalib.json"
kal = json.loads(KALIB.read_text()) if KALIB.exists() else {"arbeit": {}, "preis": {}}

PROFILE = {
    "farm": {"ungelernt": 0.8, "angelernt": 0.2},
    # Food, textiles and wood around 1900 hardly employed academics (see P1 in docs).
    "lm": {"ungelernt": 0.25, "angelernt": 0.5, "fachkraft.lebensmittel": 0.25},
    "tx": {"ungelernt": 0.1, "angelernt": 0.6, "fachkraft.textil": 0.295, "akademiker.textil": 0.005},
    "hb": {"ungelernt": 0.2, "angelernt": 0.42, "fachkraft.holz_bau": 0.37, "akademiker.holz_bau": 0.01},
    "ch": {"ungelernt": 0.15, "angelernt": 0.47, "fachkraft.chemie": 0.35, "akademiker.chemie": 0.03},
    "me": {"ungelernt": 0.1, "angelernt": 0.51, "fachkraft.metall": 0.37, "akademiker.metall": 0.02},
}

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

def sig(x, n=3):
    if x == 0: return 0
    d = n - 1 - int(math.floor(math.log10(abs(x))))
    v = round(x, d)
    return int(v) if v == int(v) and abs(v) >= 1 else v

def labor(h, profil, f):
    out = {}
    for g, share in PROFILE[profil].items():
        v = sig(h * share * f, 3)
        if v: out[g] = v
    return out

FILES = [("50_nahrung_genuss.yaml", "Kette 50 – Lebensmittel, Getränke und Tabak (P1a)", spec.P1A),
         ("51_bekleidung_leder.yaml", "Kette 51 – Textil, Bekleidung, Schuhe und Leder (P1b)", spec.P1B),
         ("52_wohnen_bauen.yaml", "Kette 52 – Möbel, Haushaltswaren und Baustoffe (P1c)", spec.P1C)]

texts, dep_texts = {}, {}
deposits = {}
for i, (fname, title, prods) in enumerate(FILES):
    chain = {"produkte": [], "anlagen": [], "rezepte": [], "technologien": []}
    seen_fac = set()
    for p in prods:
        pid = p["id"]
        price = kal["preis"].get(pid, p["preis"])
        e = {"id": pid, "art": p["art"], "branche": p["branche"], "einheit": p["einheit"]}
        if p.get("gewicht") is not None: e["gewicht_kg"] = p["gewicht"]
        e.update({"verwendung": p["verw"], "warengruppe": p["wg"], "transportklasse": p.get("tk", "stueckgut"),
                  "richtpreis_usd": sig(price, 3)})
        n = p.get("nach")
        if n:
            if n[0] == "v":
                _, kopf, X, pe, ie, kl = n
                d = {"bedarfsklasse": kl, "verbrauch": {"je_kopf_und_jahr": kopf}}
            else:
                _, dauer, quote, X, pe, ie, kl = n
                d = {"bedarfsklasse": kl, "gebrauch": {"nutzungsdauer_jahre": dauer, "max_besitzquote": quote}}
            d.update({"kaufschwelle": sig(X / price, 3), "preisempfindlichkeit": pe, "einkommensempfindlichkeit": ie})
            if p.get("saison"): d["saison"] = p["saison"]
            e["nachfrage"] = d
        if p.get("staat"):
            je, kf = p["staat"]
            e["staatsnachfrage"] = {"je_mio_usd_bip": je, "kriegsfaktor": kf}
        if p.get("sm"): e["staatsmarkt"] = {"preis_usd": p["sm"]}
        if p.get("ersetzt"): e["ersetzt"] = p["ersetzt"]
        if p.get("index"): e["foerderindex"] = p["index"]
        if p.get("pacht"): e["pacht_anteil"] = p["pacht"]
        e["annaeherung"] = True
        chain["produkte"].append(e)
        texts[f"produkt.{pid}"] = p["name"]
        r = p.get("rez")
        if not r: continue
        aid, aname, typ, umsatz = r["anlage"]
        menge = r.get("menge", 1)
        if aid not in seen_fac:
            seen_fac.add(aid)
            per_day = umsatz * 1e6 / (price * menge * 365)
            cap = sig(per_day, 2)
            farm = typ == "foerderstaette"
            chain["anlagen"].append({"id": aid, "standorttyp": typ,
                "investition_usd": int(sig(umsatz * 1e6 * (0.6 if farm else 0.9), 2)),
                "bauzeit_tage": 120 if farm else 180, "kapazitaet_je_tag": cap,
                "lebensdauer_jahre": 25, "wartung_je_jahr": 0.04,
                "automatisierung_max": 0.3 if farm else 0.5,
                **({"technologie": r["tech"]} if r.get("tech") else {}), "annaeherung": True})
            texts[f"anlage.{aid}"] = aname
        rid = r.get("rid") or (f"{pid}_gewinnen" if r.get("abbau") else f"{pid}_herstellen")
        rec = {"id": rid, "produkt": pid, "menge": menge, "dauer_tage": r.get("tage", 1), "anlage": aid}
        if r.get("tech"): rec["technologie"] = r["tech"]
        if r.get("abbau"): rec["abbau"] = True
        if r.get("ein"): rec["eingang"] = {k: v for k, v in r["ein"].items() if v}
        if r.get("neben"): rec["nebenprodukte"] = r["neben"]
        rec["arbeit_stunden"] = labor(r["h"], r["profil"], kal["arbeit"].get(rid, 1.0))
        rec["qualitaet_basis"] = 50
        rec["annaeherung"] = True
        chain["rezepte"].append(rec)
        texts[f"rezept.{rid}"] = r.get("rname") or f"{p['name']} herstellen"
        if p.get("dep"):
            deposits[pid] = (p["name"], p["dep"])
    for tid, tname, fg, year, pre, eff in TECH:
        f = TECH_FILE.get(tid, 2)
        if f == i:
            t = {"id": tid, "fachgebiet": fg, "erfindungsjahr": year}
            if pre: t["voraussetzungen"] = pre
            t["forschungsaufwand"] = eff
            t["annaeherung"] = True
            chain["technologien"].append(t)
            texts[f"technologie.{tid}"] = tname
    chain = {k: v for k, v in chain.items() if v}
    class D(yaml.SafeDumper):
        def ignore_aliases(self, d): return True
    body = yaml.dump(chain, Dumper=D, allow_unicode=True, sort_keys=False, width=110)
    (ROOT / "ketten" / fname).write_text(
        f"# {title}\n# Erzeugt mit tools/daten/pakete/p1/gen.py – Änderungen dort in spec.py.\n# Breite der Produkte (Stufe 5). Alle Zahlen sind Annäherungen; Richtpreise um 1900\n"
        f"# oder zum Erfindungsjahr, Arbeitsstunden auf die Plausibilitätsprüfung abgestimmt\n"
        f"# (5–45 % Marge, Ziel 30 %). Kaufschwelle = Einkommen, bei dem die Hälfte kauft / Preis.\n\n" + body,
        encoding="utf-8")

for pid, (pname, deps) in deposits.items():
    items = []
    for did, dname, land, maxi, faktor in deps:
        items.append({"id": did, "land": land, "rohstoff": pid, "erneuerbar": True,
                      "erschliessung": {"investition_usd": 4_000_000 if maxi > 1e6 else 2_000_000, "dauer_tage": 180},
                      "foerderung_max_je_jahr": int(maxi), "foerderkosten_faktor": faktor, "annaeherung": True})
        dep_texts[f"lagerstaette.{did}"] = dname
    (ROOT / "lagerstaetten" / f"{pid}.yaml").write_text(
        f"# {pname} (P1, erneuerbar: Weiden, Felder, Plantagen, Fanggründe, Gruben mit praktisch\n"
        f"# unerschöpflichem Vorrat). Höchstförderung um 1900 in t je Jahr; der Förderindex des\n"
        f"# Rohstoffs hebt sie über die Zeit. Grobe Annäherungen.\n\n"
        + yaml.dump({"lagerstaetten": items}, allow_unicode=True, sort_keys=False, width=110), encoding="utf-8")

def write_texts(path, marker, entries):
    p = ROOT / "texte" / "de" / path
    s = p.read_text(encoding="utf-8")
    if marker in s:
        s = s[: s.index(marker)].rstrip("\n") + "\n"
    s += f"\n{marker}\n" + "".join(f"{k}: {json.dumps(v, ensure_ascii=False)}\n" for k, v in entries.items())
    p.write_text(s, encoding="utf-8")

write_texts("ketten.yaml", "# P1 – Produktbreite (erzeugt, Ketten 50–52)", texts)
write_texts("lagerstaetten.yaml", "# P1 – Lagerstätten der neuen Rohstoffe", dep_texts)
print(len(texts), len(dep_texts))
