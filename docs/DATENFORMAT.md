# Datenformat

Alle Spielinhalte liegen als YAML-Dateien in `data/`. Neue Produkte, Rezepte oder
Technologien entstehen durch Ergänzen dieser Dateien, ohne Programmierung
(Lastenheft §16.2). Prüfen mit:

```sh
cargo run -p wsim-cli -- validate data
```

Die Prüfung meldet jeden Fehler mit Datei, Zeile und Stelle, z. B.:

```
Fehler in data/ketten/01_eisen_stahl.yaml, Zeile 216 (rezepte[3].eingang.roheisn):
  Produkt „roheisn“ ist nicht definiert. Meinten Sie „roheisen“?
```

Fehler verhindern das Laden, Warnungen nicht.

## Aufbau

```
data/
  meta.yaml            Datenversion
  grundlagen.yaml      Einheiten, Kontinente, Branchen, Warengruppen, Transportklassen
  arbeitskraefte.yaml  Qualifikationen und Fachrichtungen
  laender/             ein Land je Datei, z. B. DEU.yaml (erzeugt)
  parameter/           Parameter der Modelle, z. B. laendermodell.yaml
  ketten/              eine Produktionskette je Datei
  lagerstaetten/       Rohstoffvorkommen, eine Datei je Rohstoff
  texte/de/            alle Anzeigetexte
```

Die Aufteilung auf Dateien ist frei: Jede Datei besteht aus Abschnitten, der Lader
sammelt alle Dateien (`*.yaml`, nicht `*.yml`) und fügt die Abschnitte zusammen.
Erlaubte Abschnitte: `meta`, `laendermodell`, `einheiten`, `kontinente`, `branchen`, `warengruppen`,
`transportklassen`, `qualifikationen`, `fachrichtungen`, `laender`, `produkte`,
`anlagen`, `rezepte`, `technologien`, `lagerstaetten`. Jeder Abschnitt außer `meta`
ist eine Liste von Einträgen (`meta` und `laendermodell` sind einmalige Zuordnungen).

## Allgemeine Regeln

- **IDs** bestehen aus Kleinbuchstaben, Ziffern und `_`, ohne Umlaute
  (`stahl_bessemer`). Länder verwenden ISO-3166-Codes aus drei Großbuchstaben (`DEU`).
  Jede ID gibt es je Art nur einmal.
- **Verweise** nennen die ID eines anderen Eintrags (`produkt: roheisen`).
- **Zahlen** mit Punkt als Dezimaltrennzeichen; `_` gliedert große Zahlen (`60_000_000`).
- **Geld** immer in USD mit Kaufkraft 2026, ohne Inflation (Lastenheft §3.6).
- **Jahreswerte** werden als `{1900: 56.0, 1913: 67.0}` angegeben; dazwischen wird
  linear interpoliert, davor und danach gilt der nächste Wert.
- **Herkunft**: Geschätzte Werte bekommen `annaeherung: true`, recherchierte eine
  `quelle: "…"`. Beide Felder sind bei Ländern, Produkten, Anlagen, Rezepten,
  Technologien und Lagerstätten erlaubt.
- **Texte**: Jeder Eintrag braucht einen Anzeigenamen in `texte/de/`, Schlüssel
  `<art>.<id>`, z. B. `produkt.roheisen: Roheisen`. Arten: `einheit`, `kontinent`,
  `branche`, `warengruppe`, `transportklasse`, `qualifikation`, `fachrichtung`, `land`,
  `produkt`, `anlage`, `rezept`, `technologie`, `lagerstaette`. Zusätzliche Texte wie
  `produkt.eisenerz.info` sind erlaubt.

Pflichtfelder sind unten **fett** gesetzt, alle anderen sind optional.

## meta

| Feld | Bedeutung |
| --- | --- |
| **datenversion** | Ganze Zahl; steigt, wenn sich Inhalte so ändern, dass Spielstände angepasst werden müssen |

## Grundbegriffe

| Abschnitt | Felder |
| --- | --- |
| `einheiten` | **id**, gewicht_kg (Gewicht einer Einheit; fehlt es, braucht jedes Produkt mit dieser Einheit ein eigenes `gewicht_kg`) |
| `kontinente`, `branchen`, `warengruppen`, `transportklassen`, `fachrichtungen` | **id** |
| `qualifikationen` | **id**, **stufe** (1–255, höher = qualifizierter), **mit_fachrichtung** (true/false) |

**Arbeitskräftegruppen** ergeben sich daraus: Qualifikationen ohne Fachrichtung heißen
wie die Qualifikation (`ungelernt`), mit Fachrichtung `qualifikation.fachrichtung`
(`fachkraft.metall`). Die Fachrichtungen sind zugleich die Fachgebiete der Forschung.

## laender

Die Länderdateien werden von `tools/daten/laender.py` aus Gapminder und Natural Earth
erzeugt; Änderungen gehören in das Skript (oder bewusst von Hand, dann im Skript
nachziehen).

| Feld | Bedeutung |
| --- | --- |
| **id** | ISO-3166-Code, z. B. `DEU` (Kosovo: `XKX`) |
| **kontinent** | Verweis auf einen Kontinent |
| **flaeche_km2** | Fläche |
| **hauptstadt** | `{breite, laenge}` in Grad; Bezugspunkt für Entfernungen |
| **binnenland** | `true` ohne Meereszugang |
| nachbarn | Länder mit gemeinsamer Landgrenze (muss beidseitig eingetragen sein) |
| **werte.bevoelkerung** | Einwohner, Jahreswerte |
| **werte.bip_je_kopf_usd** | BIP je Einwohner zu Kaufkraftparität in USD, Jahreswerte |
| **werte.gini** | Gini-Koeffizient der Einkommen (0–0,95), Jahreswerte |
| werte.stabilitaet | Politische Stabilität 0–1; sonst Standardwert des Ländermodells |
| werte.steuer_unternehmen, werte.steuer_dividenden | Steuersätze 0–1; sonst Standardverlauf |
| praegung.automatisierung | Zuschlag zur Automatisierungsprägung (−1 bis 1) |
| praegung.fachrichtungen | Gewicht je Fachrichtung (1 = Durchschnitt), z. B. `{textil: 1.5}` |
| praegung.forschung | Forschungsstärke je Fachgebiet (1 = Durchschnitt) |

Alle Werte gelten in **heutigen Grenzen** und zeigen den **realen Verlauf**
einschließlich Kriegen und Krisen: jährlich bis 2026, danach Projektionen in
Fünfjahresschritten.

## laendermodell

Ein einziger Abschnitt (in `parameter/laendermodell.yaml`) mit den Parametern, aus denen
die übrigen Länderwerte berechnet werden; Formeln in `docs/FORMELN.md` (M4).

| Feld | Bedeutung |
| --- | --- |
| **preisniveau** | `referenzland`, `elastizitaet`, `minimum`, `maximum` |
| **erwerbsquote**, **lohnquote** | Anteile 0–1 |
| **jahresarbeitsstunden** | Jahreswerte |
| **qualifikationsanteile** | Zeilen `{bip_je_kopf_usd, anteile: {qualifikation: anteil}}`, aufsteigend; jede Qualifikation, Summe 1 |
| **lohnabstand** | Zeilen `{bip_je_kopf_usd, faktoren: {qualifikation: faktor}}`, aufsteigend |
| **fachrichtungsanteile** | je Qualifikation mit Fachrichtung: `{fachrichtung: anteil}`, Summe 1 |
| **strompreis_usd_je_mwh**, **stromnetz**, **stromnetz_bezug_usd** | Strom für die Industrie |
| **steuer_unternehmen**, **steuer_dividenden** | Standardverläufe 0–1 |
| **entwicklung** | `{von_usd, bis_usd}` für den Entwicklungsstand |
| **verkehrstraeger** | Verfügbarkeit 0–1 nach Jahr: `schiene`, `strasse`, `luft`, `hafen` |
| **stabilitaet** | Standardwert 0–1 |
| **forschung** | `{bezug_usd, elastizitaet, minimum, maximum}` |
| **automatisierung** | `{basis, je_verdopplung, bezug_usd}` |

## produkte

| Feld | Bedeutung |
| --- | --- |
| **id** | |
| **art** | `rohstoff`, `halbzeug`, `komponente`, `endprodukt`, `energie` |
| **branche** | Verweis auf eine Branche |
| **einheit** | Verweis auf eine Einheit |
| **verwendung** | `industrie`, `konsum` oder `beides` |
| **warengruppe** | Verweis; bestimmt später Zölle und Handelsbeschränkungen |
| **transportklasse** | Verweis; bestimmt später Transportmittel und -kosten |
| gewicht_kg | Gewicht einer Einheit; nötig, wenn die Einheit keines festlegt |
| heizwert_mwh | Energiegehalt je Einheit, wenn das Produkt als Brennstoff dient |
| nachfrage | Endkunden-Nachfrage, siehe unten |
| staatsnachfrage.**je_mio_usd_bip** | Staatlicher Bedarf in Einheiten je Jahr und Mio. USD BIP |
| staatsnachfrage.kriegsfaktor | Faktor in Kriegszeiten (Standard 1, wirkt ab Stufe 4) |
| staatsmarkt.**preis_usd** | Ware ist in jedem Land vom staatlichen Markt zu diesem Preis erhältlich |
| staatsmarkt.verfuegbar_ab / verfuegbar_bis | Jahre, in denen der Staatsmarkt die Ware anbietet |
| ersetzt | Liste von Produkten, die dieses Produkt nach und nach verdrängt |

Jedes Produkt muss hergestellt (Rezept, auch als Nebenprodukt) oder vom
**Staatsmarkt** bezogen werden können. Der Staatsmarkt liefert Güter ohne eigene
Kette, z. B. Glas, Zinn oder die Pferdekutsche.

**nachfrage** (Konsumgüter; die Formeln folgen mit M7):

| Feld | Bedeutung |
| --- | --- |
| **bedarfsklasse** | `grundbedarf`, `gebrauchsgut`, `luxus` |
| verbrauch.**je_kopf_und_jahr** | Verbrauchsgut: Sättigungsbedarf je Einwohner und Jahr |
| gebrauch.**nutzungsdauer_jahre**, gebrauch.**max_besitzquote** | Gebrauchsgut: Lebensdauer und maximale Besitzquote je Haushalt (0–10) |
| **einkommensschwelle_usd** | Pro-Kopf-Einkommen, ab dem eine Einkommensschicht kauft |
| **preisempfindlichkeit**, **einkommensempfindlichkeit** | Elastizitäten als Beträge (≥ 0) |
| saison | Zwölf Monatsfaktoren |

Genau eines von `verbrauch` oder `gebrauch` ist anzugeben.

## anlagen

| Feld | Bedeutung |
| --- | --- |
| **id** | |
| **standorttyp** | `foerderstaette`, `werk`, `kraftwerk`, `lager`, `niederlassung`, `forschungszentrum` |
| **investition_usd** | Baukosten |
| **bauzeit_tage** | |
| **kapazitaet_je_tag** | Rezept-Durchläufe je Tag bei voller Auslastung |
| **lebensdauer_jahre** | mindestens 1 |
| **wartung_je_jahr** | Anteil der Investition (0–1) |
| **automatisierung_max** | Höchster erreichbarer Automatisierungsgrad (0–1) |
| technologie | Nötige Technologie, um die Anlage zu bauen |

## rezepte

| Feld | Bedeutung |
| --- | --- |
| **id** | |
| **produkt** | Hauptprodukt |
| **menge** | Ausstoß je Durchlauf (> 0) |
| nebenprodukte | `{produkt: menge}` je Durchlauf |
| **dauer_tage** | Fertigungsdauer eines Durchlaufs, mindestens 1 |
| **anlage** | Anlage, auf der das Rezept läuft |
| technologie | Nötige Technologie |
| abbau | `true`: fördert einen Rohstoff aus einer Lagerstätte am Standort |
| eingang | `{produkt: menge}` je Durchlauf; nicht das eigene Produkt |
| **arbeit_stunden** | `{arbeitskräftegruppe: stunden}` je Durchlauf, z. B. `fachkraft.metall: 1.8` |
| energie_mwh | Strom je Durchlauf (Standard 0) |
| **qualitaet_basis** | Grundqualität 0–100 vor Einfluss von Vorprodukten, Schulung, Automatisierung und Anlagenzustand |

Mehrere Rezepte für dasselbe Produkt bilden technischen Fortschritt ab
(Bessemer-, Siemens-Martin-, Elektrostahl).

## technologien

| Feld | Bedeutung |
| --- | --- |
| **id** | |
| **fachgebiet** | Verweis auf eine Fachrichtung |
| **erfindungsjahr** | Historisches Jahr der Erfindung |
| voraussetzungen | Liste von Technologien; Kreise sind nicht erlaubt |
| forschungsaufwand | Forschungspunkte zum historischen Jahr; Pflicht für Technologien nach 1900 |

Technologien bis zum frühesten Startjahr 1900 sind bei Spielbeginn allen bekannt.

## lagerstaetten

| Feld | Bedeutung |
| --- | --- |
| **id** | |
| **land** | Verweis auf ein Land (heutige Grenzen) |
| **rohstoff** | Verweis auf ein Produkt der Art `rohstoff` |
| vorrat | Gesamtvorrat in Produkteinheiten; Pflicht, außer bei erneuerbaren |
| erneuerbar | `true` für Wald, Plantagen, Ackerland: kein Vorrat, nur Höchstförderung |
| entdeckt | Jahr der Entdeckung; vorher nicht nutzbar |
| **erschliessung.investition_usd**, **erschliessung.dauer_tage** | Kosten und Dauer der Erschließung |
| **foerderung_max_je_jahr** | Höchstförderung |
| foerderkosten_faktor | Relative Förderkosten (Standard 1) |
