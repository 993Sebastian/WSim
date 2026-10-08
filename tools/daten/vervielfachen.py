#!/usr/bin/env python3
"""Vervielfacht die Endprodukte einer Datenkopie für Last- und Skalierungstests (P0).

Aufruf: python3 tools/daten/vervielfachen.py <daten> <ziel> <faktor>

Kopiert <daten> nach <ziel> und legt jedes Endprodukt (art: endprodukt) mit seinen
Rezepten (faktor - 1)-mal zusätzlich an (`<id>_v2`, `<id>_v3`, …). Die Kopien nutzen
dieselben Anlagen, Technologien und Vorprodukte; Richtpreis und Nachfrage bleiben gleich.
Die Texte erhalten den Zusatz „(Variante n)“. Das Ergebnis ist keine Spielversion,
sondern ein Prüfstand: `wsim run --daten <ziel> --nur-welt …` misst, wie Rechenzeit
und Spielstandsgröße mit der Zahl der Produkte wachsen.
"""

import copy
import shutil
import sys
from pathlib import Path

import yaml


class _NoAliases(yaml.SafeDumper):
    # The loader rejects anchors; shared values are written out again.
    def ignore_aliases(self, data):
        return True


def dump(data) -> str:
    return yaml.dump(data, Dumper=_NoAliases, allow_unicode=True, sort_keys=False)


def main() -> None:
    if len(sys.argv) != 4:
        sys.exit(__doc__)
    source, target, factor = Path(sys.argv[1]), Path(sys.argv[2]), int(sys.argv[3])
    if factor < 1:
        sys.exit("Faktor muss mindestens 1 sein")
    if target.exists():
        shutil.rmtree(target)
    shutil.copytree(source, target)
    texts_path = target / "texte" / "de" / "ketten.yaml"
    texts = yaml.safe_load(texts_path.read_text(encoding="utf-8"))
    added_texts = {}
    products = recipes = 0
    for chain in sorted((target / "ketten").glob("*.yaml")):
        data = yaml.safe_load(chain.read_text(encoding="utf-8"))
        finals = [p for p in data.get("produkte", []) if p.get("art") == "endprodukt"]
        if not finals:
            continue
        ids = {p["id"] for p in finals}
        own = [r for r in data.get("rezepte", []) if r.get("produkt") in ids]
        for n in range(2, factor + 1):
            for p in finals:
                variant = dict(copy.deepcopy(p), id=f"{p['id']}_v{n}")
                data["produkte"].append(variant)
                name = texts.get(f"produkt.{p['id']}", p["id"])
                added_texts[f"produkt.{variant['id']}"] = f"{name} (Variante {n})"
                products += 1
            for r in own:
                variant = dict(
                    copy.deepcopy(r), id=f"{r['id']}_v{n}", produkt=f"{r['produkt']}_v{n}"
                )
                data["rezepte"].append(variant)
                name = texts.get(f"rezept.{r['id']}", r["id"])
                added_texts[f"rezept.{variant['id']}"] = f"{name} (Variante {n})"
                recipes += 1
        chain.write_text(dump(data), encoding="utf-8")
    with texts_path.open("a", encoding="utf-8") as f:
        f.write("\n")
        f.write(dump(added_texts))
    print(f"{products} Produkte und {recipes} Rezepte hinzugefügt nach {target}")


if __name__ == "__main__":
    main()
