"""Gemeinsamer Generator der Branchenpakete (Ketten 50 ff.): schreibt Ketten, Lagerstätten
und Texte aus der Spezifikation eines Pakets (spec.py). Aufgerufen von <paket>/gen.py.
"""
import json, math, os
from pathlib import Path
import yaml

# WSIM_DATEN: another data folder (e.g. a copy for trials); default the project's data.
ROOT = Path(os.environ.get("WSIM_DATEN") or Path(__file__).resolve().parents[3] / "data")

PROFILE = {
    "farm": {"ungelernt": 0.8, "angelernt": 0.2},
    # Food, textiles and wood around 1900 hardly employed academics (see P1 in docs).
    "lm": {"ungelernt": 0.25, "angelernt": 0.5, "fachkraft.lebensmittel": 0.25},
    "tx": {"ungelernt": 0.1, "angelernt": 0.6, "fachkraft.textil": 0.295, "akademiker.textil": 0.005},
    "hb": {"ungelernt": 0.2, "angelernt": 0.42, "fachkraft.holz_bau": 0.37, "akademiker.holz_bau": 0.01},
    "ch": {"ungelernt": 0.15, "angelernt": 0.47, "fachkraft.chemie": 0.35, "akademiker.chemie": 0.03},
    "me": {"ungelernt": 0.1, "angelernt": 0.51, "fachkraft.metall": 0.37, "akademiker.metall": 0.02},
    "mb": {"ungelernt": 0.1, "angelernt": 0.48, "fachkraft.maschinenbau": 0.39, "akademiker.maschinenbau": 0.03},
}


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


def write_texts(path, marker, entries):
    p = ROOT / "texte" / "de" / path
    s = p.read_text(encoding="utf-8")
    if marker in s:
        s = s[: s.index(marker)].rstrip("\n") + "\n"
    s += f"\n{marker}\n" + "".join(f"{k}: {json.dumps(v, ensure_ascii=False)}\n" for k, v in entries.items())
    p.write_text(s, encoding="utf-8")


def generate(paket, files, tech, tech_file, kal, text_marker, dep_marker, dep_note="P1"):
    """files: [(Dateiname, Titel, Produktliste)]; tech: [(id, Name, Fachgebiet, Jahr,
    Voraussetzungen, Aufwand)]; tech_file: Technologie → Index der Datei (sonst die letzte)."""
    texts, dep_texts = {}, {}
    deposits = {}
    for i, (fname, title, prods) in enumerate(files):
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
                elif n[0] == "e":
                    _, zu, je, X, pe, ie, kl = n
                    d = {"bedarfsklasse": kl, "ergaenzung": {"zu": zu, "je_besitz_und_jahr": je}}
                else:
                    _, dauer, quote, X, pe, ie, kl = n
                    d = {"bedarfsklasse": kl, "gebrauch": {"nutzungsdauer_jahre": dauer, "max_besitzquote": quote}}
                d.update({"kaufschwelle": sig(X / price, 3), "preisempfindlichkeit": pe, "einkommensempfindlichkeit": ie})
                if p.get("saison"): d["saison"] = p["saison"]
                e["nachfrage"] = d
            if p.get("staat"):
                je, kf = p["staat"][:2]
                e["staatsnachfrage"] = {"je_mio_usd_bip": je, "kriegsfaktor": kf}
                if len(p["staat"]) > 2: e["staatsnachfrage"]["verlauf"] = p["staat"][2]
            if p.get("sm"): e["staatsmarkt"] = {"preis_usd": p["sm"]}
            if p.get("ersetzt"): e["ersetzt"] = p["ersetzt"]
            if p.get("komplex"): e["sehr_komplex"] = True
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
        for tid, tname, fg, year, pre, eff in tech:
            f = tech_file.get(tid, len(files) - 1)
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
            f"# {title}\n# Erzeugt mit tools/daten/pakete/{paket}/gen.py – Änderungen dort in spec.py.\n# Breite der Produkte (Stufe 5). Alle Zahlen sind Annäherungen; Richtpreise um 1900\n"
            f"# oder zum Erfindungsjahr, Arbeitsstunden auf die Plausibilitätsprüfung abgestimmt\n"
            f"# (5–45 % Marge, Ziel 30 %). Kaufschwelle = Einkommen, bei dem die Hälfte kauft / Preis.\n\n" + body,
            encoding="utf-8")

    for pid, (pname, deps) in deposits.items():
        items = []
        for did, dname, land, maxi, faktor, *vorrat in deps:
            # A sixth value is the reserve of a mine (t); without it the deposit renews.
            stock = {"vorrat": int(vorrat[0])} if vorrat else {"erneuerbar": True}
            items.append({"id": did, "land": land, "rohstoff": pid, **stock,
                          "erschliessung": {"investition_usd": 4_000_000 if maxi > 1e6 else 2_000_000, "dauer_tage": 180},
                          "foerderung_max_je_jahr": int(maxi), "foerderkosten_faktor": faktor, "annaeherung": True})
            dep_texts[f"lagerstaette.{did}"] = dname
        head = (f"# {pname} ({dep_note}, erneuerbar: Weiden, Felder, Plantagen, Fanggründe, Gruben mit praktisch\n"
                f"# unerschöpflichem Vorrat). Höchstförderung um 1900 in t je Jahr; der Förderindex des\n"
                f"# Rohstoffs hebt sie über die Zeit. Grobe Annäherungen.\n\n")
        if any("vorrat" in i for i in items):
            head = (f"# {pname} ({dep_note}). Vorrat und Höchstförderung um 1900 in t; der Förderindex des\n"
                    f"# Rohstoffs hebt die Förderung über die Zeit. Grobe Annäherungen.\n\n")
        (ROOT / "lagerstaetten" / f"{pid}.yaml").write_text(
            head
            + yaml.dump({"lagerstaetten": items}, allow_unicode=True, sort_keys=False, width=110), encoding="utf-8")

    def write_texts(path, marker, entries):
        p = ROOT / "texte" / "de" / path
        s = p.read_text(encoding="utf-8")
        if marker in s:
            s = s[: s.index(marker)].rstrip("\n") + "\n"
        s += f"\n{marker}\n" + "".join(f"{k}: {json.dumps(v, ensure_ascii=False)}\n" for k, v in entries.items())
        p.write_text(s, encoding="utf-8")

    write_texts("ketten.yaml", text_marker, texts)
    if dep_texts:
        write_texts("lagerstaetten.yaml", dep_marker, dep_texts)
    print(len(texts), len(dep_texts))
