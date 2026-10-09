"""Auffälligkeiten der Produkte von P3b in einem Balance-Protokoll: auswertung.py <produkte.csv> <jahr> [neu]"""
from pathlib import Path
import csv,os,sys,yaml
from collections import Counter
f, year = sys.argv[1], sys.argv[2]
new=set()
for fn in ['56_luft_raumfahrt']:
    d=yaml.safe_load(open(Path(os.environ.get('WSIM_DATEN') or Path(__file__).resolve().parents[4] / 'data') / 'ketten' / f'{fn}.yaml'))
    new|={p['id'] for p in d['produkte']}
rows=[r for r in csv.DictReader(open(f),delimiter=';') if r['jahr']==year]
c=Counter(); cn=Counter()
for r in rows:
    for a in r['auffaellig'].split(','):
        if a: (cn if r['produkt'] in new else c)[a]+=1
print('neu:',dict(cn)); print('bestehend:',dict(c))
for r in rows:
    if r['auffaellig'] and (len(sys.argv)<4 or r['produkt'] in new):
        print(f"{r['produkt']:22} bed {r['bedarf']:>14} prod {r['produktion']:>14} p {r['preis_usd']:>10} rp {r['richtpreis_usd']:>10} n {r['hersteller']:>3} v {r['versorgung']:>5} {r['auffaellig']}")
