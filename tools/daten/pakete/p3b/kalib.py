"""Stimmt Arbeitsstunden und Richtpreise der Rezepte von P1 auf die Plausibilitätsprüfung ab
(Marge 5–45 %, Ziel 30 %). Braucht target/release/wsim; danach gen.py erneut aufrufen."""
import json, os, re, subprocess, sys
from pathlib import Path
import yaml
HERE = Path(__file__).parent
ROOT = Path(__file__).resolve().parents[4]
WSIM = [str(ROOT / "target/release/wsim")]
DATA = Path(os.environ.get("WSIM_DATEN") or ROOT / "data")
kf = HERE / "kalib.json"
kal = json.loads(kf.read_text()) if kf.exists() else {"arbeit": {}, "preis": {}}
sys.path.insert(0, str(HERE))
spec = __import__("spec")
base_price = {p["id"]: p["preis"] for p in spec.P3B}
texts = yaml.safe_load((DATA / "texte/de/ketten.yaml").read_text())
files = {f: yaml.safe_load((DATA / "ketten" / f).read_text()) for f in ["56_luft_raumfahrt.yaml"]}
out = subprocess.run(WSIM + ["validate", str(DATA)], capture_output=True, text=True); out = (out.stdout + out.stderr).splitlines()
warn = []
for i, l in enumerate(out):
    m = re.match(r"Warnung in .*?/ketten/(56_[a-z_]+\.yaml), Zeile \d+ \(rezepte\[(\d+)\]\.id\)", l)
    if not m: continue
    t = out[i + 1]
    n = re.search(r"„(\w+)“ (\d+) zu Richtpreisen für ([\d.]+) USD je Einheit her, der Richtpreis ist ([\d.]+) USD", t)
    if not n: continue
    rec = files[m.group(1)]["rezepte"][int(m.group(2))]
    warn.append((rec, int(n.group(2)), float(n.group(3)), float(n.group(4))))
shares = {}
for year in sorted({w[1] for w in warn}):
    tab = subprocess.run(WSIM + ["rezepte", "--jahr", str(year), "--daten", str(DATA)], capture_output=True, text=True).stdout
    for row in tab.splitlines():
        if row.startswith("| ") and "%" in row:
            c = [x.strip() for x in row.strip("|").split("|")]
            try: shares[(year, c[0])] = float(c[-1].rstrip(" %").replace(",", ".")) / 100
            except ValueError: pass
changed = 0
for rec, year, cost, price in warn:
    rid, pid = rec["id"], rec["produkt"]
    s = shares.get((year, texts.get(f"rezept.{rid}", rid)), 0.4)
    target = 0.7 * price
    L, R = s * cost, (1 - s) * cost
    f0 = kal["arbeit"].get(rid, 1.0)
    if L > 0 and R < 0.85 * target:
        f = max(0.1, min(10.0, (target - R) / L))
        kal["arbeit"][rid] = round(f0 * f, 4)
        if not (0.1 < f < 10.0):  # labour alone cannot reach it: move the price
            kal["preis"][pid] = (cost - L + L * f) / 0.7
    else:
        kal["preis"][pid] = cost / 0.7
    changed += 1
    print(f"{rid:32} {year} Kosten {cost:12.2f} Preis {price:12.2f} Arbeit {s:4.0%} → Faktor {kal['arbeit'].get(rid,1):.3f} Preis {kal['preis'].get(pid, price):.2f}")
kf.write_text(json.dumps(kal, indent=1, ensure_ascii=False))
print("geändert:", changed)
