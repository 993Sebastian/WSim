#!/usr/bin/env python3
"""Erzeugt eine Kettendatei aus Tabellen und prüft sie auf Plausibilität (P0).

Aufruf:
    python3 tools/daten/tabelle.py <produkte.csv> <kette.yaml> [--titel TEXT]
        [--anlagen anlagen.csv] [--technologien technologien.csv]
        [--daten data] [--jahr 1950] [--bericht bericht.md]

Für die Produktbreite (Stufe 5) werden Hunderte Produkte nach demselben Muster erfasst.
Eine Zeile der Produkttabelle beschreibt ein Produkt mit genau einem Rezept; weitere
Rezepte, Nebenprodukte oder Sonderfelder werden danach in der YAML-Datei ergänzt.

Tabellen: CSV mit Semikolon, erste Zeile Spaltennamen, Dezimalpunkt oder -komma.
Leere Zellen werden weggelassen. Listen in einer Zelle: `stahl:0.008 schrauben:0.0005`.

produkte.csv:  id; name; art; branche; einheit; gewicht_kg; verwendung; warengruppe;
               transportklasse; richtpreis_usd; bedarfsklasse; nutzungsdauer_jahre;
               max_besitzquote; kaufschwelle; preisempfindlichkeit;
               einkommensempfindlichkeit; rezept_id; rezept_name; anlage; menge;
               dauer_tage; technologie; eingang; arbeit_stunden; qualitaet_basis; quelle
anlagen.csv:   id; name; standorttyp; investition_usd; bauzeit_tage; kapazitaet_je_tag;
               lebensdauer_jahre; wartung_je_jahr; automatisierung_max; quelle
technologien.csv: id; name; fachgebiet; erfindungsjahr; forschungsaufwand; voraussetzungen;
               quelle

Ohne `quelle` wird `annaeherung: true` gesetzt. Die Namen gehen nach
`texte/de/ketten.yaml` (unter der Überschrift `# <titel>`).

Danach prüft `wsim validate` die Daten (Fehler und Warnungen, darunter die
Richtpreis-Margen), und `wsim rezepte` rechnet die Stückkosten der neuen Rezepte; der
Bericht listet die neuen Rezepte mit Marge und markiert Ausreißer.
"""

import argparse
import csv
import subprocess
import sys
from pathlib import Path

import yaml

PRODUCT_FIELDS = [
    "art", "branche", "einheit", "gewicht_kg", "verwendung", "warengruppe",
    "transportklasse", "richtpreis_usd",
]
DEMAND_FIELDS = ["bedarfsklasse", "kaufschwelle", "preisempfindlichkeit", "einkommensempfindlichkeit"]
FACILITY_FIELDS = [
    "standorttyp", "investition_usd", "bauzeit_tage", "kapazitaet_je_tag",
    "lebensdauer_jahre", "wartung_je_jahr", "automatisierung_max",
]


class _NoAliases(yaml.SafeDumper):
    # The loader rejects anchors.
    def ignore_aliases(self, data):
        return True


def dump(data) -> str:
    return yaml.dump(data, Dumper=_NoAliases, allow_unicode=True, sort_keys=False, width=100)


def number(text: str):
    """A cell as int or float if it is one, else the text."""
    t = text.strip().replace("_", "")
    for convert in (int, float):
        try:
            return convert(t.replace(",", "."))
        except ValueError:
            pass
    return text.strip()


def pairs(text: str) -> dict:
    """`stahl:0.008 schrauben:0.0005` → {stahl: 0.008, schrauben: 0.0005}."""
    out = {}
    for item in text.split():
        key, _, value = item.partition(":")
        if not value:
            sys.exit(f"„{item}“: erwartet Schlüssel:Menge")
        out[key] = number(value)
    return out


def rows(path: Path) -> list[dict]:
    with path.open(encoding="utf-8-sig", newline="") as f:
        reader = csv.DictReader(f, delimiter=";")
        return [
            {k.strip(): (v or "").strip() for k, v in row.items() if k}
            for row in reader
            if any((v or "").strip() for v in row.values())
        ]


def source(row: dict, entry: dict) -> None:
    if row.get("quelle"):
        entry["quelle"] = row["quelle"]
    else:
        entry["annaeherung"] = True


def product(row: dict) -> dict:
    p = {"id": row["id"]}
    for f in PRODUCT_FIELDS:
        if row.get(f):
            p[f] = number(row[f])
    if row.get("bedarfsklasse"):
        demand = {f: number(row[f]) for f in DEMAND_FIELDS if row.get(f)}
        if row.get("nutzungsdauer_jahre"):
            demand["gebrauch"] = {
                "nutzungsdauer_jahre": number(row["nutzungsdauer_jahre"]),
                "max_besitzquote": number(row.get("max_besitzquote") or "1"),
            }
        p["nachfrage"] = demand
    source(row, p)
    return p


def recipe(row: dict) -> dict | None:
    if not row.get("anlage"):
        return None
    r = {
        "id": row.get("rezept_id") or f"{row['id']}_herstellen",
        "produkt": row["id"],
        "menge": number(row.get("menge") or "1"),
        "dauer_tage": number(row.get("dauer_tage") or "1"),
        "anlage": row["anlage"],
    }
    if row.get("technologie"):
        r["technologie"] = row["technologie"]
    if row.get("eingang"):
        r["eingang"] = pairs(row["eingang"])
    if row.get("arbeit_stunden"):
        r["arbeit_stunden"] = pairs(row["arbeit_stunden"])
    if row.get("qualitaet_basis"):
        r["qualitaet_basis"] = number(row["qualitaet_basis"])
    source(row, r)
    return r


def facility(row: dict) -> dict:
    a = {"id": row["id"]}
    for f in FACILITY_FIELDS:
        if row.get(f):
            a[f] = number(row[f])
    source(row, a)
    return a


def technology(row: dict) -> dict:
    t = {"id": row["id"]}
    for f in ("fachgebiet", "erfindungsjahr", "forschungsaufwand"):
        if row.get(f):
            t[f] = number(row[f])
    if row.get("voraussetzungen"):
        t["voraussetzungen"] = row["voraussetzungen"].split()
    return t


def report(wsim: list[str], data: Path, year: int, recipes: list[str], names: dict) -> str:
    """Validation result and the margins of the new recipes."""
    check = subprocess.run(wsim + ["validate", str(data)], capture_output=True, text=True)
    lines = [f"# Plausibilität\n", "## Prüfung\n", "```", check.stdout.strip() or check.stderr.strip(), "```\n"]
    table = subprocess.run(
        wsim + ["rezepte", "--jahr", str(year), "--daten", str(data)],
        capture_output=True,
        text=True,
    )
    wanted = {names.get(f"rezept.{r}", r) for r in recipes}
    rows = [l for l in table.stdout.splitlines() if l.startswith("| ")]
    head, body = rows[:2], [l for l in rows[2:] if l.split("|")[1].strip() in wanted]
    lines.append(f"## Neue Rezepte {year}\n")
    if body:
        lines += head + body
        flagged = sum("⚠" in l for l in body)
        lines.append(f"\n{len(body)} Rezepte, {flagged} mit Marge außerhalb des Bands (⚠).")
    else:
        lines.append(f"Keines der neuen Rezepte erscheint {year} in der Rechnung (Fehler oben, Erfindung später oder Daten unvollständig).")
    return "\n".join(lines) + "\n"


def main() -> None:
    a = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    a.add_argument("produkte", type=Path)
    a.add_argument("kette", type=Path, help="Zieldatei, z. B. data/ketten/49_moebel.yaml")
    a.add_argument("--titel", default=None, help="Überschrift in Datei und Texten")
    a.add_argument("--anlagen", type=Path)
    a.add_argument("--technologien", type=Path)
    a.add_argument("--daten", type=Path, default=Path("data"))
    a.add_argument("--jahr", type=int, default=1950)
    a.add_argument("--bericht", type=Path)
    a.add_argument("--wsim", default="cargo run -q --release -p wsim-cli --")
    args = a.parse_args()

    title = args.titel or args.kette.stem
    chain = {"produkte": [], "anlagen": [], "rezepte": [], "technologien": []}
    texts = {}
    for row in rows(args.produkte):
        chain["produkte"].append(product(row))
        texts[f"produkt.{row['id']}"] = row.get("name") or row["id"]
        r = recipe(row)
        if r:
            chain["rezepte"].append(r)
            name = texts[f"produkt.{row['id']}"]
            texts[f"rezept.{r['id']}"] = row.get("rezept_name") or f"{name} herstellen"
    if args.anlagen:
        for row in rows(args.anlagen):
            chain["anlagen"].append(facility(row))
            texts[f"anlage.{row['id']}"] = row.get("name") or row["id"]
    if args.technologien:
        for row in rows(args.technologien):
            chain["technologien"].append(technology(row))
            texts[f"technologie.{row['id']}"] = row.get("name") or row["id"]
    chain = {k: v for k, v in chain.items() if v}

    args.kette.write_text(
        f"# {title}\n# Erzeugt mit tools/daten/tabelle.py aus {args.produkte.name}.\n\n" + dump(chain),
        encoding="utf-8",
    )
    texts_path = args.daten / "texte" / "de" / "ketten.yaml"
    known = yaml.safe_load(texts_path.read_text(encoding="utf-8")) or {}
    new = {k: v for k, v in texts.items() if k not in known}
    if new:
        with texts_path.open("a", encoding="utf-8") as f:
            f.write(f"\n# {title}\n" + dump(new))
    print(f"{args.kette}: {len(chain.get('produkte', []))} Produkte, "
          f"{len(chain.get('rezepte', []))} Rezepte; {len(new)} Texte ergänzt")

    names = {**known, **texts}
    text = report(args.wsim.split(), args.daten, args.jahr, [r["id"] for r in chain.get("rezepte", [])], names)
    if args.bericht:
        args.bericht.write_text(text, encoding="utf-8")
        print(f"Bericht: {args.bericht}")
    else:
        print(text)


if __name__ == "__main__":
    main()
