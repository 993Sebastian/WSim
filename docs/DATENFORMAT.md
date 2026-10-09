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
  verkehrsmittel.yaml  Verkehrsmittel mit Kosten, Geschwindigkeit, Nutzlast und Kaufpreis
  waehrungen/          Preisindex, Währungen und ihre Zeiträume je Land (Anzeige)
  etappen.yaml         Etappenziele des Spielers nach der Einführung
  startups/            historische Erfinder zu ihren Technologien (SU1)
  texte/de/            alle Anzeigetexte
```

Die Aufteilung auf Dateien ist frei: Jede Datei besteht aus Abschnitten, der Lader
sammelt alle Dateien (`*.yaml`, nicht `*.yml`) und fügt die Abschnitte zusammen.
Erlaubte Abschnitte: `meta`, `laendermodell`, `produktionsmodell`, `finanzmodell`,
`marktmodell`, `transportmodell`, `forschungsmodell`, `einheiten`, `kontinente`, `branchen`, `warengruppen`,
`transportklassen`, `qualifikationen`, `fachrichtungen`, `laender`, `produkte`,
`anlagen`, `rezepte`, `technologien`, `lagerstaetten`, `verkehrsmittel`, `kimodell`,
`namensgruppen`, `produktnamen`, `reale_firmen`, `ereignisse`, `ereignisfolgen`, `preisindex`, `waehrungen`,
`landeswaehrungen`, `etappen`, `startups`, `erfinder`. Jeder Abschnitt außer `meta` ist eine Liste von Einträgen (`meta`,
`preisindex`, `produktnamen` und die Modelle in `parameter/` sind einmalige Zuordnungen).

## Allgemeine Regeln

- **IDs** bestehen aus Kleinbuchstaben, Ziffern und `_`, ohne Umlaute
  (`stahl_bessemer`). Länder verwenden ISO-3166-Codes aus drei Großbuchstaben (`DEU`).
  Jede ID gibt es je Art nur einmal.
- **Verweise** nennen die ID eines anderen Eintrags (`produkt: roheisen`).
- **Zahlen** mit Punkt als Dezimaltrennzeichen; `_` gliedert große Zahlen (`60_000_000`).
- **Geld** immer in USD mit Kaufkraft 2026, ohne Inflation (Lastenheft §3.6). Nur die
  Anzeige rechnet in Landeswährungen um (Abschnitte `preisindex`, `waehrungen`,
  `landeswaehrungen`).
- **Jahreswerte** werden als `{1900: 56.0, 1913: 67.0}` angegeben; dazwischen wird
  linear interpoliert, davor und danach gilt der nächste Wert.
- **Herkunft**: Geschätzte Werte bekommen `annaeherung: true`, recherchierte eine
  `quelle: "…"`. Beide Felder sind bei Ländern, Produkten, Anlagen, Rezepten,
  Technologien und Lagerstätten erlaubt.
- **Texte**: Jeder Eintrag braucht einen Anzeigenamen in `texte/de/`, Schlüssel
  `<art>.<id>`, z. B. `produkt.roheisen: Roheisen`. Arten: `einheit`, `kontinent`,
  `branche`, `warengruppe`, `transportklasse`, `qualifikation`, `fachrichtung`, `land`,
  `produkt`, `anlage`, `rezept`, `technologie`, `lagerstaette`, `verkehrsmittel`,
  `schwierigkeit`, `ereignis`, `waehrung`, außerdem `teilland` für die Länder in Regionen,
  `grundstuecksklasse` und `lage` für Grundstücke.
  Namen von Firmen sind Eigennamen und brauchen keinen Text.
  Zusätzliche Texte wie
  `produkt.eisenerz.info` sind erlaubt.

Pflichtfelder sind unten **fett** gesetzt, alle anderen sind optional.

## meta

| Feld | Bedeutung |
| --- | --- |
| **datenversion** | Ganze Zahl; steigt, wenn sich Inhalte so ändern, dass Spielstände angepasst werden müssen |

## Grundbegriffe

| Abschnitt | Felder |
| --- | --- |
| `einheiten` | **id**, gewicht_kg (Gewicht einer Einheit; fehlt es, braucht jedes Produkt mit dieser Einheit ein eigenes `gewicht_kg`). Vorhanden: `t`, `kg`, `l` (Liter, 1 kg; P1), `stueck`, `mwh`, `m3`, `m2`, `kwh`, `kwp` |
| `kontinente`, `branchen`, `warengruppen`, `fachrichtungen` | **id** |
| `transportklassen` | **id**, kostenfaktor (Transportkosten gegenüber Schüttgut, Standard 1) |
| `qualifikationen` | **id**, **stufe** (1–255, höher = qualifizierter), **mit_fachrichtung** (true/false) |

**Arbeitskräftegruppen** ergeben sich daraus: Qualifikationen ohne Fachrichtung heißen
wie die Qualifikation (`ungelernt`), mit Fachrichtung `qualifikation.fachrichtung`
(`fachkraft.metall`). Die Fachrichtungen sind zugleich die Fachgebiete der Forschung.

## laender

Die Länderdateien werden von `tools/daten/laender.py` aus Gapminder und Natural Earth
erzeugt; Änderungen gehören in das Skript (oder bewusst von Hand, dann im Skript
nachziehen). Welche Länder zu Regionen zusammengefasst werden, steht in
`tools/daten/regionen.py` (Formeln: `docs/FORMELN.md`, Abschnitt M34).

| Feld | Bedeutung |
| --- | --- |
| **id** | ISO-3166-Code, z. B. `DEU`; Regionen ohne namensgebendes Land einen frei verfügbaren Code `X..` (`XBA` Baltikum) |
| **kontinent** | Verweis auf einen Kontinent |
| **flaeche_km2** | Fläche |
| **hauptstadt** | `{breite, laenge}` in Grad; Bezugspunkt für Entfernungen |
| **binnenland** | `true` ohne Meereszugang |
| nachbarn | Länder mit gemeinsamer Landgrenze (muss beidseitig eingetragen sein) |
| umfasst | Nur bei Regionen (M34): ISO-Codes der zusammengefassten Länder, mindestens zwei, das namensgebende oder größte zuerst. Keins davon darf ein eigenes Land sein oder zu einer zweiten Region gehören. Jedes braucht einen Text `teilland.<ISO>`. |
| staedte | Städte für den Hauptsitz (W2), je `{id, einwohner, breite, laenge, hauptstadt}`: `id` snake_case ohne Umlaute, je Land einmal; `einwohner` > 0 (Agglomeration heute, gilt als Anteil an der Bevölkerung von `zentrale.stadt.einwohner_jahr`); höchstens eine `hauptstadt: true`. Jede braucht einen Text `stadt.<ISO>.<id>`. Ohne Städte zählt das Land als eine Stadt. Erzeugt von `tools/daten/laender.py` (Natural Earth, bis fünf je Land). |
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
| **produktivitaet** | Arbeitsproduktivität `{bezug_usd, elastizitaet, minimum, maximum}` (minimum ≤ maximum): Die Arbeitsstunden der Rezepte gelten beim BIP je Kopf `bezug_usd` |
| **automatisierung** | `{basis, je_verdopplung, bezug_usd}` |

## produktionsmodell

Ein einziger Abschnitt (in `parameter/produktionsmodell.yaml`); Formeln in
`docs/FORMELN.md` (M5).

| Feld | Bedeutung |
| --- | --- |
| **standortkosten_usd** | Kosten je Standorttyp: `foerderstaette`, `werk`, `kraftwerk`, `lager`, `niederlassung`, `forschungszentrum` (alle nötig) |
| **gebaeude_lebensdauer_jahre**, **erschliessung_lebensdauer_jahre** | Abschreibungsdauern |
| **foerderkurve_ab** | 0–1: Ist von einer Lagerstätte mit Vorrat weniger als dieser Anteil übrig, sinkt die zulässige Förderung im selben Verhältnis; 0 = volle Förderung bis zuletzt (C2) |
| **automatisierung** | `arbeitsersparnis` (0–1), `kostenanteil` |
| **qualitaet** | Gewichte `vorprodukte`, `automatisierung`, `zustand` |
| **zustand_minimum** | Untergrenze des Anlagenzustands (0–1) |
| `lohnaufschlag_max` | Höchster Lohnaufschlag eines Standorts über die Landeslöhne (0–5, Vorgabe 1 = 100 %; M18) |
| **stilllegung** | Stillgelegte Anlagen (M22): `instandhaltung_anteil` (0–1, Anteil der Wartung, den sie weiter kosten), `wiederanlauf_tage` (Dauer des Wiederanfahrens), `wiederanlauf_kosten` (0–1, Anteil der Investition) |
| **verkauf** | Verkaufte Anlagen (M22): `erloes_anteil` (0–1, Anteil des Restbuchwerts), `schrottwert` (0–1, Mindesterlös als Anteil der Investition) |
| **schulung** | Schulung je Standort (W1): `kosten_anteil_lohn` (0–1, Kosten je Tag als Anteil der Lohnsumme bei Ziel 100 %), `aufbau_je_monat` (0–1, so viel steigt das Niveau je Monat bis zum Ziel), `verlust_je_monat` (0–1, so viel sinkt es über dem Ziel), `arbeitsersparnis` (0–0,9, weniger Arbeitsstunden bei Niveau 100 %), `qualitaet_punkte` (0–50, Qualitätspunkte bei Niveau 100 %). Fehlt der Abschnitt, gibt es keine Schulung |
| strom | Produkt der Art `energie`, das für Eigenstrom steht (Kette 7) |
| **einspeiseverguetung** | Anteil (0–1) des Industriestrompreises für überschüssigen Eigenstrom |
| **gemeinkosten_anteil** | Verwaltung, Vertrieb und Logistik als Zuschlag (0–5) auf die Umwandlungskosten eines Laufs (Arbeit, Strom, Anlage), je Produktart: `rohstoff`, `halbzeug`, `komponente`, `endprodukt`, `energie` |
| **nebenprodukte_lager_tage** | Nebenprodukte über so viele Tage ihrer Erzeugung hinaus werden entsorgt (> 0) |
| **anlagengroessen** | Anlagengrößen (M36): **kapazitaet** je Größe `sehr_klein`, `klein`, `mittel`, `gross`, `sehr_gross` (alle nötig, > 0, mit der Größe wachsend, `mittel` = 1 – für sie gelten die Werte der Anlagen); **investition_exponent** (0–1,5), **arbeit_exponent** (−1–1, auf die Stunden je Durchlauf), **flaeche_exponent** (0–1,5), **bauzeit_exponent** (0–1). Namen als `anlagengroesse.<größe>`. |
| **richtpreis_marge** | `{minimum, maximum}` (je −1 bis 1): Bereich der Marge, den das beste Rezept eines Produkts zu Richtpreisen erreichen soll; außerhalb warnt die Prüfung (Förderung nur unter `minimum`) |
| **startformen** | `werkstatt` und `handel` (beide nötig), je: **standorttyp**, **gebaeude_usd**, `anlagen` (Liste `{anlage, rezept, auslastung}`), `einkauf` (Liste `{produkt, ziel, hoechstpreis_usd}`), `verkauf` (Liste von Produkten, Angebot zum Marktpreis). Anlagen und Rezepte müssen 1900 bekannt sein und zum Standorttyp passen. Kosten werden vom Startkapital bezahlt. |

## forschungsmodell

Ein einziger Abschnitt (in `parameter/forschungsmodell.yaml`); Formeln in
`docs/FORMELN.md` (M9).

| Feld | Bedeutung |
| --- | --- |
| **vorgriff_faktor** | Kostenfaktor (1–10) je Jahr vor dem historischen Erfindungsjahr |
| **nachzuegler** | `rabatt_je_jahr` und `minimum` (je 0–1) nach der Erfindung |
| **gemeingut_nach_jahren** | Jahre nach dem historischen Erfindungsjahr, ab denen jede Firma die Technologie nutzen darf |
| **forscher** | Qualifikation der Forscher (mit Fachrichtungen); die Fachrichtung ist das Fachgebiet der Technologie |
| **sachkosten_usd_je_forschertag** | Geräte und Material je Forscher und Tag bei Preisniveau 1 |
| **weiterentwicklung** | Weiterentwicklung erforschter Produkte (M37): `stufen` (1–10); `je_stufe` mit `qualitaet` (Punkte, 0–20), `arbeit` und `vorprodukte` (eingesparter Anteil je Durchlauf, 0–0,09); `aufwand` mit `anteil` (0,01–2), `wachstum` (1–3) und `grundaufwand` (Punkte, > 0): Stufe n kostet max(grundaufwand, größter Forschungsaufwand der Technologien des Produkts) · anteil · wachstum^(n − 1); `gemeingut_nach_jahren` (1–100); `fachgebiete`: Branche → Fachrichtung der Forscher für Produkte ohne Technologie (jede Branche eines solchen Produkts braucht einen Eintrag, die Fachrichtung eine Forschergruppe) |
| patente | Patente (P7, optional; ohne den Block keine): `laufzeit_jahre` (1–50) ab der ersten Anmeldung, `anmeldefrist_tage` (1–3650) ab der Erfindung, `kosten_je_land_usd` (≥ 0; einmalig je Land für die ganze Laufzeit, mal Preisniveau des Landes), `ki_groesste_maerkte` (0–200; KI-Firmen melden in ihren Standortländern und so vielen größten Märkten an), `annaeherung`/`quelle` |

## finanzmodell

Ein einziger Abschnitt (in `parameter/finanzmodell.yaml`); Formeln in `docs/FORMELN.md` (M6).

| Feld | Bedeutung |
| --- | --- |
| **realzins** | Realer Leitzins als Jahreswerte (−0,2 bis 0,5) |
| **risikoaufschlag** | `minimum` und `je_verschuldung` (Aufschlag je Verschuldungsgrad) |
| **beleihung** | Anteil der Sachwerte, bis zu dem Banken Kredit geben (0–1) |
| **dispo** | `anteil` der Bilanzsumme als Kreditlinie, `aufschlag` auf den Leitzins |
| **laufzeit_max_jahre** | Längste Kreditlaufzeit |
| **dividende** | Dividenden aller Firmen (PE4): `reserve_monate_min` (0–60: die Kasse behält so viele Monate laufender Kosten), `konzern_anteil` (0–1: darüber bekommt eine Firma ihren Anteil steuerfrei in die Rücklagen), `quote_min`, `quote_max` (0–1, min ≤ max: Anteil am Jahresüberschuss, den angriffslustige bzw. vorsichtige KI-Firmen und CEOs vorschlagen), `kasse_monate` (0–60: darunter schlagen KI und CEO nichts vor) |

## marktmodell

Ein einziger Abschnitt (in `parameter/marktmodell.yaml`); Formeln in `docs/FORMELN.md` (M7).

| Feld | Bedeutung |
| --- | --- |
| **preisgewicht**, **qualitaetsgewicht** | je 5 Werte (ärmstes Fünftel zuerst) für die Anbieterwahl |
| **aneignung_je_jahr** | Anteil der Lücke zur Ziel-Besitzquote, der je Jahr gekauft wird |
| **verlauf_monate** | Monate der Preis- und Absatzreihe je Markt für die Verläufe (1–120, M24) |
| **meldung_preissenkung** | Preissenkung eines Wettbewerbers seit seinem Bezugspreis, die der Rundenbericht meldet (0,01–0,9, M24) |
| **preisanpassung** | `hoch`, `runter` (je Tag), `lagertage`, `auslastung_normal` (0–1: darunter sinken automatische Preise auch, freie Anlagen werben um Kunden), `hoechstfaktor` (automatische Preise höchstens dieses Vielfache des Richtpreises im Land, 1–1000), `aufholen_max` (knappe Ware unter dem Richtpreis steigt um `hoch` · Richtpreis/Preis, höchstens dieses Vielfache; 1–1000, M22) |
| **staat_hoechstpreis** | Staaten zahlen höchstens dieses Vielfache des Richtpreises (im Land, mindestens des weltweiten) |
| **verdraengung_staat_jahre** | > 0: über so viele Jahre ab seiner Verfügbarkeit verdrängt ein Gut mit Staatsnachfrage den Staatsbedarf der Güter, die es `ersetzt` (M33) |
| **preisniveau_anteil** | Wie weit das Preisniveau eines Landes die Preise bestimmt (0: Weltpreis, 1: voll), je Produktart: `rohstoff`, `halbzeug`, `komponente`, `endprodukt`, `energie` (je 0–1) |
| **index_glaettung** | Gewicht des Tagesdurchschnitts im Marktpreis |
| **haendler** | KI-Händler (M8): `marge` (Aufschlag auf Einkauf und Transport), `vorrat_tage` (Lager für so viele Tage offener Nachfrage), `glaettung_tage` (1–365, Mittelung der offenen Nachfrage), `arbitrage` (C1): `abstand` (0–10: so weit muss der Marktpreis über Einstand und Marge aus dem Ausland liegen) und `anteil` (0–1: so viel des Absatzes bringen die Händler dann zusätzlich; 0 = keine Arbitrage) |
| **marke** | Marke und Werbung (M16): `markengewicht` (5 Werte ≥ 0, ärmstes Fünftel zuerst), `vergessen_je_monat` (0–1), `mundpropaganda` (0–1, Anteil der Lücke je Monat bei 100 % Marktanteil), `kosten_je_einwohner_usd` (> 0, Werbung, die ein Land bei Preisniveau 1 einmal erreicht), `bekanntheit_start` und `bekanntheit_start_real` (0–1, etablierte generierte bzw. historische Firmen, wo sie zum Start Endprodukte verkaufen), `bekanntheit_handel` und `bekanntheit_staatsmarkt` (0–1, Bekanntheit eingeführter Ware und des Staatsmarkts), `werbemittel` (Liste `{id, ab, wirkung}`, nicht leer, `id` eindeutig, `wirkung` > 0; es wirkt das beste im Jahr verfügbare; Text `werbemittel.<id>`) |

## transportmodell

Ein einziger Abschnitt (in `parameter/transportmodell.yaml`); Formeln in
`docs/FORMELN.md` (M8).

| Feld | Bedeutung |
| --- | --- |
| **umweg** | `land`, `see`, `luft` (je 1–5): Weglänge im Verhältnis zur Luftlinie zwischen den Hauptstädten |
| **umschlag** | `kosten_usd_je_t` und `tage` für Be- oder Entladen in einem Hafen oder Flughafen bei voll ausgebauter Infrastruktur |
| **mindestinfrastruktur** | Ausbaugrad (0–1), unter dem Straße, Schiene, Hafen oder Flughafen nicht nutzbar sind |

## ruestung

`data/parameter/ruestung.yaml` (H3; ohne den Abschnitt keine Kriegsstärke und keine Rüstungsgüter).

| Feld | Bedeutung |
| --- | --- |
| **militaerausgaben** | Militärausgaben als Anteil am BIP (0–1) je Land und Jahr: `standard` und `laender` (ISO-Code → Jahreswerte), wie `co2_preis` |
| **frieden**, **krieg** | Anteile (0–1, krieg > frieden): bis `frieden` Kriegsstärke 0, ab `krieg` 1, dazwischen linear |
| **bezug** | Anteil (0,001–1), bei dem der Bedarf je BIP der Rüstungsgüter gilt |
| annaeherung, quelle | Herkunft |

## produkte

**Werkzeuge für die Produktbreite (P0).** `python3 tools/daten/tabelle.py produkte.csv
data/ketten/NN_name.yaml --titel "Kette NN – …" [--anlagen anlagen.csv]
[--technologien technologien.csv] [--jahr 1950]` schreibt eine Kettendatei aus Tabellen
(CSV mit Semikolon; eine Zeile je Produkt mit einem Rezept, Listen als
`stahl:0.008 schrauben:0.0005`; Spalten im Kopf des Skripts), ergänzt die Namen in
`texte/de/ketten.yaml` und gibt einen Plausibilitätsbericht aus: Ergebnis von
`wsim validate` (darunter die Richtpreis-Margen) und die Zeilen der neuen Rezepte aus
`wsim rezepte`. Zellen ohne `quelle` erhalten `annaeherung: true`. Weitere Rezepte,
Nebenprodukte und Sonderfelder werden danach in der YAML-Datei ergänzt.
**Branchenpakete (P1 ff.):** Die Ketten 50 ff. und die Lagerstätten ihrer Rohstoffe erzeugt
`python3 tools/daten/pakete/<paket>/gen.py` (P1: Ketten 50–52, P2: 53–54, P3a: 55, P3b: 56) aus `spec.py`
(eine Zeile je Ware mit Preis, Nachfrage, Rezept, Anlage und Lagerstätten) über den
gemeinsamen Generator `tools/daten/pakete/generator.py`; `kalib.py` stimmt Arbeitsstunden
und Richtpreise auf die Plausibilitätsprüfung ab und schreibt `kalib.json`. Änderungen an
diesen Waren gehören in `spec.py`, die YAML-Dateien sind erzeugt. Eine Lagerstätte mit
sechstem Wert hat diesen Vorrat (Gruben), sonst ist sie erneuerbar; `staat` darf einen
Verlauf tragen und mit `"ruestung"` statt des Kriegsfaktors ein Rüstungsgut beschreiben (H3);
Technologien bis 1900 erhalten keinen Forschungsaufwand, `komplex` setzt `sehr_komplex`, `('e', zu, je_besitz, …)` beschreibt ein
Ergänzungsgut. Mit `WSIM_DATEN=<ordner>` schreiben
Generator und Kalibrierung in eine Kopie der Daten (Versuche).
`python3 tools/daten/vervielfachen.py data <ziel> <faktor>` legt in einer Kopie jedes
Endprodukt mit seinen Rezepten `faktor − 1`-mal zusätzlich an (`<id>_v2` …) – ein
Prüfstand für Rechenzeit und Spielstandsgröße, keine Spieldaten.

| Feld | Bedeutung |
| --- | --- |
| **id** | |
| **art** | `rohstoff`, `halbzeug`, `komponente`, `endprodukt`, `energie` |
| **branche** | Verweis auf eine Branche |
| **einheit** | Verweis auf eine Einheit |
| **verwendung** | `industrie`, `konsum` oder `beides` |
| **warengruppe** | Verweis; bestimmt später Zölle und Handelsbeschränkungen |
| **transportklasse** | Verweis; bestimmt später Transportmittel und -kosten |
| **richtpreis_usd** | Typischer Preis je Einheit um 1900 (bei Preisniveau 1); Startwert der Märkte |
| gewicht_kg | Gewicht einer Einheit; nötig, wenn die Einheit keines festlegt |
| heizwert_mwh | Energiegehalt je Einheit, wenn das Produkt als Brennstoff dient |
| nachfrage | Endkunden-Nachfrage, siehe unten |
| staatsnachfrage.**je_mio_usd_bip** | Staatlicher Bedarf in Einheiten je Jahr und Mio. USD BIP |
| staatsnachfrage.kriegsfaktor | Faktor bei voller Kriegsstärke (Standard 1); dazwischen 1 + (Faktor − 1) · Kriegsstärke des Landes (H3, Abschnitt `ruestung`) |
| staatsnachfrage.ruestung | `true`: Rüstungsgut (H3) – der Bedarf je BIP gilt bei Militärausgaben von `ruestung.bezug` und folgt ihnen; braucht den Abschnitt `ruestung`, kein `kriegsfaktor` |
| staatsnachfrage.verlauf | Jahreswerte eines Faktors auf den Bedarf je BIP, z. B. `{1913: 1.0, 1970: 8.0}`; dazwischen linear, davor und danach der erste bzw. letzte Wert; ohne Angabe 1 (M39) |
| staatsmarkt.**preis_usd** | Ware ist in jedem Land vom staatlichen Markt zu diesem Preis erhältlich |
| staatsmarkt.verfuegbar_ab / verfuegbar_bis | Jahre, in denen der Staatsmarkt die Ware anbietet |
| ersetzt | Liste von Produkten, die dieses Produkt nach und nach verdrängt: als Gebrauchsgut über seinen Besitz (M9), mit Staatsnachfrage deren Staatsbedarf über `verdraengung_staat_jahre` (M33) |
| foerderindex | Nur Rohstoffe: Jahreswerte, mit denen die Höchstförderung und der Vorrat aller Lagerstätten des Rohstoffs gegenüber ihren Datenwerten wachsen (mehr Fläche, bessere Erträge, Erkundung), z. B. `{1900: 1.0, 1930: 1.45}`; ohne Angabe 1 |
| pacht_anteil | Nur Rohstoffe, 0–0,9: Pacht und Förderabgaben je geförderter Einheit als Anteil am Richtpreis im Land (Bodenrente, Förderzins, Konzessionsabgaben); als Kostenart „Pacht und Förderabgaben“ gebucht; ohne Angabe 0 |
| sehr_komplex | `true`, wenn der Produktbaum fünf oder sechs Ebenen braucht (Lastenheft §17.2); sonst Warnung ab fünf Ebenen |

Jedes Produkt muss hergestellt (Rezept, auch als Nebenprodukt) oder vom
**Staatsmarkt** bezogen werden können. Der Staatsmarkt liefert Güter ohne eigene
Kette, z. B. Glas, Zinn oder die Pferdekutsche.

**nachfrage** (Konsumgüter; Formeln in `docs/FORMELN.md`, M7):

| Feld | Bedeutung |
| --- | --- |
| **bedarfsklasse** | `grundbedarf`, `gebrauchsgut`, `luxus` |
| verbrauch.**je_kopf_und_jahr** | Verbrauchsgut: Sättigungsbedarf je Einwohner und Jahr |
| gebrauch.**nutzungsdauer_jahre**, gebrauch.**max_besitzquote** | Gebrauchsgut: Lebensdauer und maximale Besitzquote je Einwohner (0–10) |
| ergaenzung.**zu**, ergaenzung.**je_besitz_und_jahr** | Ergänzungsgut: Verbrauch je besessenem Gebrauchsgut (z. B. Benzin je Auto) |
| **kaufschwelle** | Verhältnis Jahreseinkommen je Kopf zu Preis, bei dem die Hälfte einer Schicht kauft (z. B. 2) |
| **preisempfindlichkeit**, **einkommensempfindlichkeit** | Exponenten von Preis und Einkommen in der Kaufneigung (≥ 0) |
| saison | Zwölf Monatsfaktoren |
| netzabhaengig | `true`: nur Haushalte mit Stromanschluss kaufen (Nachfrage × Netzversorgung) |

Genau eines von `verbrauch`, `gebrauch` oder `ergaenzung` ist anzugeben; `zu` muss ein
Gebrauchsgut sein.

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
| flaeche_ha | Fläche je Einheit in ha (> 0), wo die Regel des Grundstücksmodells (Investition je ha) nicht passt, etwa bei Raffinerien (M35) |

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
| eingang | `{produkt: menge}` je Durchlauf; nicht das eigene Produkt; höchstens vier Vorprodukte (Strom zählt nicht). Der Produktbaum vom Rohstoff bis zum Produkt hat höchstens sechs Ebenen, mehr als vier nur bei `sehr_komplex` (Lastenheft §17.2). Hat ein Vorprodukt mehrere Rezepte, zählt sein einfachster Weg; ein Rohstoff mit Abbau-Rezept ist immer die erste Ebene |
| **arbeit_stunden** | `{arbeitskräftegruppe: stunden}` je Durchlauf, z. B. `fachkraft.metall: 1.8` |
| energie_mwh | Strom je Durchlauf (Standard 0) |
| **qualitaet_basis** | Grundqualität 0–100 vor Einfluss von Vorprodukten, Schulung, Automatisierung und Anlagenzustand |
| `co2_t` | t CO₂ je Durchlauf (0–1000, Standard 0; H2) |
| `schadstoff_kg` | kg Schadstoffe (Staub, Schwefel- und Stickoxide) je Durchlauf (0–100 000, Standard 0; H2); Nachrüstung senkt sie |

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
| vorrat | Gesamtvorrat in Produkteinheiten beim Förderindex 1; er wächst mit dem Förderindex des Rohstoffs (M41). Pflicht, außer bei erneuerbaren |
| erneuerbar | `true` für Wald, Plantagen, Ackerland: kein Vorrat, nur Höchstförderung |
| entdeckt | Jahr der Entdeckung; vorher nicht nutzbar |
| **erschliessung.investition_usd**, **erschliessung.dauer_tage** | Kosten und Dauer der Erschließung |
| **foerderung_max_je_jahr** | Höchstförderung |
| foerderkosten_faktor | Relative Förderkosten (Standard 1) |

## verkehrsmittel

| Feld | Bedeutung |
| --- | --- |
| **id** | |
| **weg** | `gelaende` (ohne ausgebaute Wege, z. B. Fuhrwerk), `strasse`, `schiene`, `see`, `luft` |
| **verfuegbar_ab**, verfuegbar_bis | Jahre, in denen das Verkehrsmittel genutzt wird |
| **transportklassen** | Liste der Transportklassen, die es befördert (mindestens eine) |
| **kosten_usd_je_tkm** | Jahreswerte: Kosten je Tonnenkilometer Schüttgut bei voll ausgebauter Infrastruktur |
| **km_je_tag** | Jahreswerte: Strecke je Tag einschließlich Wartezeiten (> 0) |
| nutzlast_t, kaufpreis_usd | Jahreswerte (> 0): Tonnen je Fahrzeug und Kaufpreis in USD für eigene Flotten (W5). Nur gemeinsam; ohne sie kauft niemand das Verkehrsmittel. Luftfahrzeuge zählen nicht zu Flotten. |

Eine Transportklasse ohne verfügbares Verkehrsmittel (in Stufe 1 `leitung`) lässt sich
nicht zwischen Ländern befördern.

## kimodell

Ein einziger Abschnitt (in `parameter/kimodell.yaml`); Formeln in `docs/FORMELN.md` (M10).
Werte der Form `{bei_0: …, bei_1: …}` hängen linear von Kompetenz bzw. Aggressivität
der Firma (0–1) ab.

| Feld | Bedeutung |
| --- | --- |
| **firmen_standard**, **firmen_max** | Vorgabe und Obergrenze der Zahl der KI-Firmen |
| **firmen_bei_realer_groesse** | Bei so vielen Firmen arbeiten die Märkte in realer Größe (Marktmaßstab 1) |
| **massstab** | `minimum` und `maximum` des Marktmaßstabs (je über 0 bis 1) |
| **arbeitskraefte_min** | Untergrenze jedes Arbeitsmarkts (Personen je Gruppe) |
| **anlagen_je_konzession**, **konzessionen_max** | Teilung der Lagerstätten in Konzessionen |
| **schwierigkeiten** | Liste mit `id`, `kompetenz`, `aggressivitaet` (je 0–1); Text `schwierigkeit.<id>` |
| **schwierigkeit_standard** | ID der vorgewählten Schwierigkeit |
| **streuung** | Zufällige Abweichung je Firma von den Werten der Schwierigkeit (0–0,5) |
| **start** | `auslastung`, `marktdeckung` (je Produktart 1–3: Vollleistung der Startanlagen im Verhältnis zum Bedarf; über 1 sind die Märkte zum Start gesättigt), `anlage_mindestanteil`, `lager_eingang_tage`, `lager_ausgang_tage`, `kasse_monate`, `gewicht_entwicklung` (je Produktart: `rohstoff`, `halbzeug`, `komponente`, `endprodukt`, `energie`), `referenzlohn_usd` |
| **verhalten** | `betrieb_alle_tage`, `lager_hoch_tage`, `lager_niedrig_tage` (kleiner als hoch), `lager_ziel_tage` (Lager der Erzeugnisse in Tagen des Abgangs), `lager_ausgleich_tage` (in so vielen Tagen wird die Lücke zum Ziel geschlossen), `auslastung_schritt`, `auslastung_min`, `auslastung_aenderung_max` (0,01–1: so stark ändert sich die geplante Auslastung je Entscheidung höchstens), `preisuntergrenze`, `schaetzfehler` (`bei_0`/`bei_1` nach Kompetenz, je 0–1: so weit liegen Stückkosten und Marge in der Schätzung einer KI-Firma je Produkt und Jahr daneben; B2), `werbeanteil` (`bei_0`/`bei_1` nach Aggressivität: Werbebudget als Anteil des Vormonatsumsatzes je Land und Warengruppe), `schulung` (`bei_0`/`bei_1` nach Kompetenz, je 0–1: Schulungsziel der Standorte einer KI-Firma, auf 5 % gerundet; W1), `einkauf_aufschlag`, `ausbau_auslastung`, `ausbau_marge`, `ausbau_markt_auslastung` (0–1, Standard 0: kein Ausbau eines Produkts, solange alle laufenden Anlagen dafür weltweit im Schnitt weniger ihrer Leistung planen; C4), `ausbau_vorprodukt_preis_max` (1–10: kein Ausbau, solange ein Vorprodukt im Land mehr als dieses Vielfache seines Richtpreises kostet), `ausbau_anteil_kasse_max`, `ausbau_je_pruefung_max` (1–50: so viele Ausbauten beginnt eine Firma je Prüfung, die mit der höchsten Marge zuerst; C2), `forschung_vorgriff_jahre`, `forschung_mindestumsatz_usd`, `forschung_mindestkompetenz`, `entwicklung_nutzen_je_stufe` (0–1, geschätzter Jahresnutzen einer Entwicklungsstufe als Anteil am Umsatz mit dem Produkt; M37), `entwicklung_amortisation_jahre` (0–50: so schnell muss sich die nächste Stufe bezahlt machen), `forschung_luecke_firmen` (so viele Firmen erforschen dieselbe Technologie für eine Marktlücke zugleich, 0 = keine; M32), `forschung_luecke_vorrang_umsatz` (≥ 0, Standard 0: eine Marktlücke, deren offener Bedarf im Jahr mindestens so viele Vorjahresumsätze der Firma wert ist, geht der Forschung in den eigenen Branchen vor; 0 = nie; L1), `forschung_vorlauf_jahre` (0–30, Standard 0: Produkte mit Nachfrage, die sich historisch in so vielen Jahren herstellen lassen, zählen samt ihren Vorprodukten schon als Marktlücke; M39), `kasse_min_monate` (kleiner als max), `kasse_max_monate`, `kredit_jahre`, `gruendungen_je_monat`, `diversifikationen_je_quartal` (Firmen je Quartal, die in einem fremden Engpass bauen), `einstiege_je_quartal` (0–100, Standard 0: so viele teure oder knappe Märkte mit 1 bis unter `einstieg_firmen_max` Herstellern bekommen je Quartal einen weiteren; C3), `einstieg_preisfaktor` (1–10), `einstieg_firmen_max` (0 = nie) und `einstieg_anteil` (0,01–1): zahlen die Käufer weltweit wenigstens dieses Vielfache des Richtpreises und stellen weniger Firmen das Produkt her, plant eine weitere Firma für diesen Anteil des Absatzes (M33), `vorrat_jahre_min` (≥ 0: eine neue Konzession nur, wenn der Restvorrat der Lagerstätte so viele Jahre ihrer Höchstförderung reicht; M33). Die Firmenzahlen `forschung_luecke_firmen`, `gruendungen_je_monat`, `diversifikationen_je_quartal`, `einstiege_je_quartal` und `einstieg_firmen_max` gelten für `firmen_standard` KI-Firmen und wachsen mit mehr Firmen im selben Verhältnis, `gruendung_kapitalfaktor`, `lohnaufschlag_schritt` (0–1: so weit hebt oder senkt eine KI-Firma den Lohnaufschlag eines Standorts je Entscheidung), `lohnaufschlag_max` (0–5: höchster Aufschlag der KI; M18), `stilllegen_auslastung` (0–1, kleiner als `stilllegen_zielauslastung`: darunter legt die KI überzählige Einheiten still), `stilllegen_zielauslastung` (0,1–1), `stilllegen_preis_max` (0,1–10: nur wo der Preis im Land unter diesem Vielfachen des Richtpreises liegt), `wiederanfahren_auslastung` (0,1–1, größer als `stilllegen_zielauslastung`: darüber fährt die KI stillgelegte Einheiten wieder an), `verkaufen_nach_monaten` (so lange stillgelegte Einheiten werden verkauft; M22) |

## kaufmodell

Ein einziger Abschnitt (in `parameter/kaufmodell.yaml`): Kaufangebote zwischen Firmen für
Standorte und Lizenzen (M30). Formeln in `docs/FORMELN.md` (M30). Werte der Form
`{bei_0: …, bei_1: …}` hängen linear von der Aggressivität der Firma ab.

| Feld | Bedeutung |
| --- | --- |
| **gueltig_monate** | 1–24: so lange wartet ein Angebot auf die Antwort |
| **sperre_monate** | 0–120: nach Ablehnung oder Verfall bietet derselbe Käufer erst danach wieder für denselben Gegenstand |
| **mindestalter_monate** | 0–120: ein Standort lässt sich erst so lange nach der Gründung kaufen |
| **ertragsfaktor** | 0,5–30 Jahre: Ertragswert = Jahresergebnis des Standorts × Faktor |
| **ertrag_mindestmonate** | 1–12: kürzer bestehende Standorte haben noch keinen Ertragswert |
| **firmenwert_jahre** | 1–40: Abschreibung eines gekauften Firmenwerts |
| **qualifiziert_ab_stufe** | Qualifikationsstufe, ab der Beschäftigte als qualifiziert zählen (eine Stufe aus `qualifikationen`) |
| **anteile** | Anteile kaufen und verkaufen (PE5): `praemie_min`, `praemie_max` (0–5, min ≤ max: Aufschlag auf den Firmenwert, den Halter verlangen, bei schwacher bzw. guter Ertragslage), `rendite_gut` (0,01–1: Jahresüberschuss je Eigenkapital, ab dem die Ertragslage als gut gilt), `abschlag_min`, `abschlag_max` (0–0,9, min ≤ max: Abschlag, mit dem Anleger der Person Anteile sofort abkaufen, bei guter bzw. schwacher Ertragslage) |
| **insolvenz** | Pleiten (M38): `tage` (0–365; so lange werden die Standorte einer zahlungsunfähigen KI-Firma versteigert, 0 = sofort aufgeben), `mindestpreis` (0–1, Mindestgebot als Anteil des Grundwerts) |
| **ki** | Verhalten der KI-Firmen: `angebot_chance` (je 0–1, Wahrscheinlichkeit je Monat), `offene_angebote_max` (0–20), `spieler_angebote_je_monat` (0–10, von allen KI-Firmen zusammen), `wettbewerb_aufschlag` (je 0–5), `fachkraefte_aufschlag` (0–5), `bauzeit_aufschlag` (0–5), `neubau_anteil` (0–1), `mindestvorteil` (0–5), `gebotsaufschlag` (je 0–5), `mindestpreis_usd` (≥ 0), `kasse_anteil_max` (0–1), `lizenz_gebot` (je 0–2), `lizenz_hoechst` (0–2), `verkaufsaufschlag` (je 0–5), `kern_anteil` (0–1), `kern_aufschlag` (0–10), `lizenz_mindest` (0–2), `wettbewerb_lizenz` (0–10), `gegen_schwelle` (0–1) |

## grundstuecksmodell

Ein einziger Abschnitt (in `parameter/grundstuecksmodell.yaml`): Grundstücke der
Standorte (M35). Formeln in `docs/FORMELN.md` (M35).

| Feld | Bedeutung |
| --- | --- |
| **flaeche_ha_je_mrd_bip** | > 0: Gewerbefläche eines Landes je Mrd. USD BIP (mal Marktmaßstab) |
| **frei_min_anteil** | 0–0,9, Standard 0: zum 1. Januar kommen mindestens so viele Grundstücke hinzu, dass dieser Anteil der Gewerbefläche frei bleibt (L1) |
| **wachstum** | `{ab_jahr, jahre}`: neue Grundstücke wachsen um den Faktor 1 + (Jahr − ab_jahr) / jahre |
| **wohlstand** | `{reich_ab_usd, arm_unter_usd}`: BIP je Kopf, ab dem ein Land reich bzw. unter dem es arm ist (arm < reich) |
| **klassen** | Größenklassen, mindestens eine: `id`, `flaeche_ha: {von, bis}` (> 0, von ≤ bis), `anteile: {reich, mittel, arm}` (je 0–1; je Wohlstand zusammen 1). Jede braucht den Text `grundstuecksklasse.<id>` |
| **lagen** | `stadt`, `hafen`, `land` mit `anteil` (0–1, zusammen 1), `flaeche` und `bodenpreis` (Faktoren 0,1–10), `anwerben` (−1 bis 1, wirkt wie ein Lohnaufschlag beim Anwerben), `fracht_see` (0,1–2, Faktor auf die Fracht über See), `lieferkosten` (0–0,5 des Umsatzes im eigenen Land). Texte `lage.<lage>` |
| **bodenpreis_usd_je_ha** | ≥ 0: Bodenpreis bei Preisniveau 1 |
| **knappheit** | 0–20: der Bodenpreis steigt um diesen Faktor mal den belegten Anteil der Gewerbefläche |
| **pacht_anteil** | 0–1: jährliche Pacht als Anteil des Bodenwerts |
| **anlagenflaeche** | `investition_je_ha_usd` (> 0), `zuschlag` (0–5, für Wege, Lager und Verwaltung), `mindestflaeche_ha` (≥ 0, je Standort) |
| **ki_reserve** | 0–5: die KI sucht ein Grundstück, das ihr Vorhaben mit dieser Reserve fasst |

## namensgruppen

Namensbausteine für erzeugte KI-Firmen (in `ki/`). Eine Firma nimmt die Gruppe ihres
Sitzlandes, sonst die eine Gruppe mit `standard: true`.

| Feld | Bedeutung |
| --- | --- |
| **id** | Schlüssel der Gruppe |
| laender | Länder der Gruppe; jedes Land gehört zu höchstens einer Gruppe |
| standard | `true` für genau eine Gruppe |
| **familiennamen**, **orte**, **rechtsformen** | Nicht leere Listen von Namensteilen |
| **vornamen** | Nicht leere Liste; Vornamen der Manager aus Ländern der Gruppe (MA1) |
| familienname_zuerst | Wahrheitswert, Vorgabe `false`: Namen der Manager mit dem Familiennamen zuerst (Ostasien) |
| **muster** | Namensmuster mit `{familienname}`, `{ort}`, `{rechtsform}`, `{branche}` |
| **branchen** | Wort für das Geschäft je Branche, z. B. `metallurgie: Hüttenwerke` |

## management

`data/parameter/management.yaml` (MA1; Regeln: `docs/FORMELN.md`, Vorgabe: `docs/MANAGER.md`).
Optional: ohne den Abschnitt gibt es keine Manager.

| Feld | Bedeutung |
|---|---|
| **bereiche** | Liste: `id` (Bereich, Text `bereich.<id>`), `themen` (Themen der Entscheidungen aus MA0, die eine Stelle des Bereichs erledigt; jedes Thema höchstens in einem Bereich; der Bereich mit `abwerbung` stellt die Anliegen zu Angeboten anderer Firmen an Manager, MA6) |
| **ebenen** | Genau die Ebenen `standort`, `land`, `kontinent`, `vorstand`: `pruefung_tage` (≥ 1), `gehalt_fach`, `gehalt_leitung` (> 0, Vielfaches des Jahreslohns der Lohngruppe), `budget_fach`, `budget_leitung` (je `[Entscheidung, Jahr]`, Anteile 0–1 am Umsatz der Einheit in zwölf Monaten, das erste höchstens das zweite); optional `fachstellen` (Bereiche der Fachstellen von Land, Kontinent und Vorstand – beim Vorstand die Ressorts, MA5 –, je höchstens einmal; nicht bei `standort`, dessen Fachstellen unter `standorttypen` stehen) und `themen` (Themen, die Stellen der Ebene selbst aufgreifen; jedes muss zu einem Bereich gehören, MA3) |
| routine_themen | Themen, bei denen die Stelle der Option der Regeln folgt (Routine); bei den übrigen empfiehlt sie nach ihrem Urteilsvermögen |
| regel_themen | Themen, deren Wirkung nur Kosten zählt (Kredite, Werbung): Die Stelle folgt der Option der Regeln, Anliegen dazu gelten als wichtig (MA5). Ein Thema steht höchstens in einer der beiden Listen |
| **budget_sockel_gehaelter** | `entscheidung`, `jahr` (≥ 0): Mindestbudget in Jahresgehältern des Managers |
| **anliegen** | `frist_tage`, `offen_je_stelle`, `wirkzeit_tage` (≥ 1), `sperre_tage` (≥ 0), `schaetzfehler` (0–1, größter Fehler der Prognose ohne Fachkompetenz), `empfehlung_grund` (0–1, Wahrscheinlichkeit einer besten Empfehlung ohne Urteilsvermögen), `buendel_ab` (≥ 1, ab so vielen gleichen Anliegen verschiedener Standorte an einem Tag fragt die Stelle darüber einmal für alle, MA3) |
| **strategie** | Strategievorgaben (MA4): `premium`, `kampfpreis` je `untergrenze` (> 0, Preisuntergrenze als Vielfaches der Vollkosten je Stück) und `aufschlag` (−0,9 bis 2, Startpreis über dem Marktpreis); größte einstellbare Werte `mindestmarge_max` (> 0, Anteil auf die Vollkosten), `lager_tage_max` (> 0 und mindestens die Lagertage der KI, die die Standardwerte sind), `liquiditaet_monate_max` (> 0, Monate laufender Kosten) |
| **strategieauftrag** | Strategieauftrag und Rücksprache (MA5): `leitlinien` mit `wachstum`, `ertrag`, `sicherheit`, `marktfuehrung` (je 0–1, Aggressivität der Regeln wie bei KI-Firmen); `antraege_max`, `chancen_risiken`, `ruecksprachen_behalten` (je ≥ 1); `personal_schaerfe` (0–1, wie viel genauer ein Personalressort mit voller Fachkompetenz die Fähigkeiten zeigt) |
| **markt** | Lebendiger Managermarkt (MA6): `erfahrung` (`chance_monat` 0–1, `spielraum` 0–100 Punkte über der höchsten Fachkompetenz bei der Ziehung); `zufriedenheit` (`start` 0–100, `basis` 0–100, `gehalt_gewicht`, `verlust_abzug`, `uebergangen_abzug` ≥ 0, `anpassung` 0–1, `stufen` zwei steigende Grenzen 0–100 für „unzufrieden“ und „gemischt“); `kuendigung` (`schwelle` 0–100, `chance_max` 0–1); `abwerbung` (`staerke_min` und `vorsprung` 0–100, `aufschlag` ≥ 0, `ignoriert_abzug` 0–100, `ki_gegen_max` 1–10, `sperre_monate` Monate bis zum nächsten Angebot an denselben Manager); `ki` (`einstellungen_monat`, `umsatz_ceo_usd` und `umsatz_standort_usd` ≥ 0, `gehalt_anteil`, `kompetenz_ceo`, `kompetenz_leitung` je 0–1) |
| **standorttypen** | Je Standorttyp (`werk`, `foerderstaette`, `kraftwerk`, `lager`, `niederlassung`, `forschungszentrum`) die Bereiche seiner Fachstellen; jeder Standort hat dazu eine Leitung |
| **leitung_ohne_fach_abschlag** | 0–1: so viel weniger Fachkompetenz hat eine Leitung in einem Bereich ohne besetzte Fachstelle |
| **bemerken_grund** | 0–1: Wahrscheinlichkeit, mit der eine Stelle ganz ohne Sorgfalt eine Lage am Prüftermin bemerkt; mit voller Sorgfalt 1 |
| **gehalt_lohngruppe** | Arbeitskräftegruppe, deren Lohn die Gehälter bestimmt (z. B. `akademiker.kaufmaennisch`) |
| **abfindung_monate** | ≥ 0: Monatsgehälter beim Entlassen |
| **pool** | `je_mio_akademiker` (> 0), `min` (≥ 1), `max` (≥ `min`): Kandidaten je Kontinent; `abgang_monat` (0–1) |
| **faehigkeiten** | `schwerpunkt`, `sonst`, `allgemein`: je `mittel` (0–100) und `streuung` (0–50); `eindruck_unschaerfe` (0–50) |
| **annaeherung**, **quelle** | wie bei anderen Daten |

## tochterfirmen

`data/parameter/tochterfirmen.yaml` (W6; Regeln: `docs/FORMELN.md`, Abschnitt W6). Ohne
Abschnitt kann der Spieler keine Tochterfirmen gründen. Ein Abschnitt `tochterfirmen` mit:

| Feld | Bedeutung |
|---|---|
| **mindestkapital_usd** | ≥ 0: kleinstes Startkapital einer Tochter |
| **gruendungskosten_usd** | ≥ 0: Notar, Register (Sonstiges der Mutter) |
| **geschaeftsfuehrung** | `kompetenz`, `aggressivitaet` (je 0–1): Charakter der eigenen Geschäftsführung, die nach den Regeln der KI handelt |
| **logistik** | `kasse_anteil` (0–1: so viel ihrer Kasse steckt eine Logistik-Tochter je Monat in Fahrzeuge), `rendite_min` (0–10: Jahresrendite, ab der sie ein Fahrzeug kauft) |
| **annaeherung**, **quelle** | wie bei anderen Daten |

## boerse

`data/parameter/boerse.yaml` (K1; Regeln: `docs/FORMELN.md`, Abschnitt K1). Ohne
Abschnitt gibt es keine Börse: keine Firma ist notiert, die Befehle der Börse werden
abgelehnt. Ein Abschnitt `boerse` mit:

| Feld | Bedeutung |
|---|---|
| **bewertung** | `gewicht_buchwert` (0–1: Anteil des Eigenkapitals am fairen Wert, der Rest ist Gewinn · KGV), `kgv` (> 0), `boden_buchwert` (0–1: nie unter diesem Anteil des Eigenkapitals), `rendite_annahme` (0–1: Jahresgewinn je Eigenkapital für Monate ohne Zahlen), `gewinn_monate` (1–24: Durchschnitt über so viele abgeschlossene Monate) |
| **stimmung** | `schwankung` (0–1: Standardabweichung von ln S je Monat), `rueckkehr` (0–1: so viel von ln S baut sich je Monat ab) |
| **traegheit** | 0–1: Anteil der Lücke zwischen Börsenwert und Ziel (logarithmisch), der sich je Monat schließt |
| **rauschen** | 0–1: eigene Schwankung je Firma und Monat |
| **boersengang** | `eigenkapital_min_usd` (≥ 0), `anteil_max` (0,01–0,9: höchstens so viele neue Aktien auf einmal), `abschlag` (0–1: Zeichnungsabschlag auf den Wert), `kosten_anteil` (0–1: Banken und Prospekt, Anteil am Erlös) |
| **handel** | `aufschlag`, `abschlag` (je 0–1: auf den Börsenwert beim Kauf bzw. Verkauf), `preiswirkung` (0–5: Kurswirkung je gehandeltem Anteil), `anteil_max` (0–1: so viel einer anderen Firma darf eine Firma über die Börse halten; darüber nur mit einem Übernahmeangebot) |
| **uebernahme** | `aufschlag` (0–5: Übernahmeprämie über dem Börsenwert), `kosten_anteil` (0–1: Banken und Berater, Anteil am Kaufpreis) (K3) |

Historische Börsenkrisen stehen seit H1 als Wirkung `boersenkrach` bei den Ereignissen.
| **rueckkauf** | `anteil_max` (0–0,9: je Aktienrückkauf höchstens so viel der Aktien) (K3) |
| **start** | `eigenkapital_min_usd` (≥ 0: KI-Firmen mit so viel Eigenkapital sind beim Start notiert), `streubesitz` (0–1: davon bei den Anlegern) |
| **ki** | `boersengang_chance` (0–1: je Monat, für KI-Firmen mit dem Eigenkapital von `start`), `boersengang_anteil` (0–0,9); KI-Anleger (K3): `depot_anteil_kasse` (0–1: so viel des Kassenüberschusses in Aktien), `depot_anteil_max` (0–1: höchstens so viel einer Firma), `unterbewertung` (0–0,9: kauft unter, verkauft über diesem Abstand zum fairen Wert), `uebernahme_chance` (0–1: je Monat, mal Aggressivität), `uebernahme_kasse_anteil` (0–1: Übernahmepreis höchstens so viel der Kasse) |
| **annaeherung**, **quelle** | wie bei anderen Daten |

## anleihen

`data/parameter/anleihen.yaml` (K2; Regeln: `docs/FORMELN.md`, Abschnitt K2). Ohne Abschnitt
gibt es keine Anleihen. Ein Abschnitt `anleihen` mit:

| Feld | Bedeutung |
|---|---|
| **eigenkapital_min_usd** | ≥ 0: kleinere Firmen finden keine Anleger |
| **volumen_min_usd** | ≥ 0: kleinste Anleihe |
| **laufzeit_jahre** | `min`, `max` (je 1–100, min ≤ max) |
| **kosten_anteil** | 0–1: Kosten der Ausgabe (Sonstiges), Anteil am Betrag |
| **rueckkauf_aufschlag** | 0–1: vorzeitiger Rückkauf über dem Nennwert (Zinsaufwand) |
| **gewinn_monate** | 1–24: Zinsdeckung aus so vielen abgeschlossenen Monaten |
| **bonitaet** | Liste der Stufen von der besten zur schlechtesten: `stufe` (`aaa`, `aa`, `a`, `bbb`, `bb`, `b` oder `ccc`, je höchstens einmal; Text `bonitaet.<stufe>`), `verschuldung_max` (0–1: Kredite und Anleihen je Bilanzsumme), `zinsdeckung_min` (0–100: EBIT je Zinsaufwand), `aufschlag` (0–0,5: über dem realen Leitzins). Jede Stufe hat einen größeren Aufschlag, eine höhere erlaubte Verschuldung und eine kleinere Zinsdeckung als die vorige. |
| **ki** | `laufzeit_jahre` (zwischen `laufzeit_jahre.min` und `.max`), `vorteil_min` (0–1: so weit muss der Kupon unter dem Kreditzins liegen) |
| **annaeherung**, **quelle** | wie bei anderen Daten |

## bank

`data/parameter/bank.yaml` (K4; Regeln: `docs/FORMELN.md`, Abschnitt K4). Ohne Abschnitt
gibt es keine Banken des Spielers (die Startform „Bank“ und der Schwerpunkt „Bank“ fehlen).
Ein Abschnitt `bank` mit:

| Feld | Bedeutung |
|---|---|
| **einlagen** | `hebel_max` (> 0: Einlagen höchstens so viel mal das Eigenkapital), `aufschlag_neutral` (−0,2–0,2: Einlagenzins über dem Leitzins, bei dem die Hälfte der Kapazität kommt), `elastizitaet` (0–1000: so viel mehr Anteil je Zinspunkt), `anpassung` (0–1: Anteil der Lücke zum Ziel, der sich je Monat schließt) |
| **mindestreserve** | 0–1: dieser Anteil der Einlagen bleibt in der Kasse |
| **start** | Einstellungen einer neuen Bank: `einlagen_aufschlag` (−0,2–0,2), `kreditnachlass` (0–0,9), `verschuldung_max` (0–1) |
| **annaeherung**, **quelle** | wie bei anderen Daten |

## zentrale

`data/parameter/zentrale.yaml` (ZA1–ZA4; Regeln: `docs/FORMELN.md`, Vorgabe:
`docs/BETEILIGUNGEN.md`). Optional: ohne den Abschnitt kostet ein Umzug nichts und dauert
bis zum nächsten Monatsanfang.

| Feld | Bedeutung |
|---|---|
| **hauptsitz** | `verlegung_monate` (0–60), `kosten_grund_usd` und `kosten_je_angestelltem_usd` (≥ 0, USD Kaufkraft 2026), `mitziehen` (0–1: Anteil der Angestellten der Zentrale, der beim Umzug mitkommt) |
| abteilungen | Liste der Zentralabteilungen (ZA2): `id` (`strategie`, `finanzen`, `personal`, `recht` oder `marketing`, je höchstens einmal – was sie tut, steht im Kern), `bereich` (ein Bereich aus `management`, der unter den Fachstellen der Ebene `vorstand` steht: das Ressort, dessen Manager die Abteilung leitet), `lohngruppe` (Arbeitskräftegruppe der Angestellten, Lohn im Land des Hauptsitzes), `buero_usd` (≥ 0: Bürokosten je Angestelltem und Jahr), `faelle` (≥ 0: was ein Angestellter im Monat bearbeitet), `wirkung` (0–1: Wirkung bei voller Güte und Abdeckung). Ohne Liste gibt es keine Abteilungen. |
| genauigkeit | 0–1: so viel kleiner werden Schätzfehler einer voll ausgestatteten Abteilung, und so viel der Lücke zum vollen Urteilsvermögen schließt sie (ZA2) |
| umschuldung | `mindestvorteil` (0–1, absolut: ab diesem Zinsvorteil lohnt ein neuer Kredit) und `gebuehr` (0–1: Anteil der Restschuld) (ZA3) |
| trefferquote | `bewertung_monate` (1–120), `mittelwert` (0–1), `vorgewicht` (≥ 0: Fälle, mit denen der Mittelwert zählt), `k` (0–10: Exponent der Gehaltsforderung) (ZA3) |
| stadt | Stadt des Hauptsitzes (W2): `akademiker_konzentration` (0–100: Akademiker einer Stadt = Pool des Landes · min(1, Wert · Anteil der Stadt an der Bevölkerung)), `anteil_zentralen` (0–1: davon offen für die Zentralen aller Firmen der Stadt), `buero_bezug_einwohner` (> 0) und `buero_elastizitaet` (0–1: Bürokosten · Preisniveau · (Einwohner / Bezug)^Elastizität), `umzug_im_land` (0–1: Anteil der Umzugskosten bei einer anderen Stadt desselben Landes), `einwohner_jahr` (1900–2100: Bezugsjahr der Einwohnerzahlen der Städte). Ohne den Block begrenzt keine Stadt die Zentrale. |
| ki | Zentrale der KI-Firmen (ZA4): `anteil_umsatz` (`bei_0`, `bei_1`, je 0–1: so viel ihres Umsatzes der letzten zwölf Monate dürfen alle Abteilungen zusammen im Jahr kosten, nach Kompetenz), `reihenfolge` (Liste von Abteilungen aus `abteilungen`, je höchstens einmal: die eingerichtet werden, in dieser Reihenfolge), `mindestlast` (Zuordnung Abteilung → Arbeitslast ≥ 0, ab der sie eingerichtet wird; ohne Angabe 1), `sitz` mit `anteil_umsatz_min` (0–1: so viel ihres Umsatzes muss ein Land bringen, um Sitz zu werden), `bip_anteil_min` (0–10: so viel des BIP je Kopf des heutigen Sitzlands muss das neue Land mindestens haben), `amortisation_jahre` (0–100: in so vielen Jahren muss die Ersparnis die Kosten der Verlegung decken) und `sperre_jahre` (0–100: so lange bleibt der Sitz nach einem Umzug). Ohne den Block richten KI-Firmen keine Abteilungen ein und verlegen ihren Sitz nicht. |
| **annaeherung**, **quelle** | wie bei anderen Daten |

Die Bereiche `strategie` (Themen `kaufangebot`, `antwort`) und `recht` (Thema `lizenz`)
stehen in `management.yaml` als Ressorts des Vorstands; ihre Texte `bereich.<id>` in der
Oberfläche, die der Abteilungen unter `abteilung.<id>`.

## startups

`data/parameter/startups.yaml` (SU1–SU3; Regeln: `docs/FORMELN.md`, Vorgabe:
`docs/BETEILIGUNGEN.md` §5). Optional: ohne den Abschnitt gibt es keine Start-ups. Eine
einmalige Zuordnung:

| Feld | Bedeutung |
|---|---|
| **bezeichnungen** | Liste `{id, ab}` (nicht leer, `ab` aufsteigend): wie das Spiel die Start-ups ab diesem Jahr nennt; Texte `startup.bezeichnung.<id>` |
| **je_jahr** | 0–1000: neue Start-ups je Jahr bei der Häufigkeit „normal“ |
| **haeufigkeiten** | Liste `{id, faktor}` (nicht leer, `faktor` 0–10): die Wahl beim neuen Spiel, Faktor auf `je_jahr`; Texte `startup.haeufigkeit.<id>` |
| **haeufigkeit_standard** | `id` einer Häufigkeit: die Vorgabe |
| **anteil_neu** | 0–1: Anteil, der an einer neuen Technologie arbeitet (sonst an einer Verbesserung) |
| **vorlauf_jahre_max** | 0–100: so viele Jahre darf eine neue Technologie ihrem historischen Jahr höchstens voraus sein |
| **phasen** | Liste `{id, monate, kapital_usd, chance, bewertung}` (nicht leer, `id` eindeutig, `monate` > 0, `kapital_usd` ≥ 0, `chance` 0–1, `bewertung` > 0: Wert vor der Runde als Vielfaches des Kapitals); Texte `startup.phase.<id>` |
| **bezug_bip_je_kopf_usd** | > 0: BIP je Kopf, bei dem `kapital_usd` gilt |
| **kapital_faktor_min**, **kapital_faktor_max** | > 0, min ≤ max: Grenzen des Einkommensfaktors |
| **vorlauf_kapital** | ≥ 0: mehr Kapital je Jahr Vorlauf (Anteil) |
| **vorlauf_chance** | 0–1: weniger Chance je Jahr Vorlauf (Anteil), höchstens bis `chance_min` (0–1) |
| **investoren_chance_monat** | 0–1: Wahrscheinlichkeit je Monat, dass Investoren eine offene Runde decken |
| **frist_monate** | > 0: so lange bleibt eine Runde offen |
| **unschaerfe** | 0–1: Unschärfe der gezeigten Chance ohne Strategieabteilung |
| **stufe_mittel_ab**, **stufe_hoch_ab** | 0–1, mittel ≤ hoch: ab diesen gezeigten Chancen heißt die Stufe „mittel“ und „hoch“ |
| **aufbewahren_jahre** | 0–100: so lange bleiben beendete Start-ups in der Liste |
| **beteiligung** | Zuordnung (SU2, Pflicht im Abschnitt), Felder siehe unten |
| **annaeherung**, **quelle** | wie bei anderen Daten |

Felder von `beteiligung` (Beteiligungen der Firmen, SU2):

| Feld | Bedeutung |
|---|---|
| **erfolg_faktor** | 0–100: Wert bei Erfolg als Vielfaches der Bewertung nach der letzten Runde |
| **kauf_aufschlag** | 0–10: Aufschlag auf den Wert beim Kauf zwischen den Runden und beim Eingliedern |
| **verkauf_abschlag** | 0–1: Abschlag auf den Wert beim Verkauf an Investoren |
| **foerderung_wirkung** | 0–1: Fördergeld in Höhe des Phasenkapitals schließt diesen Teil der Lücke der Chance zu 1 |
| **sperrminoritaet**, **mehrheit** | 0–1, Sperrminorität ≤ Mehrheit: ab der Sperrminorität kann keine andere Firma eingliedern; über der Mehrheit lenken, eingliedern, bei Erfolg Mutter werden |
| **forschungsbonus** | 0–1: Anteil des Forschungsaufwands, den die Mehrheit beim Scheitern gutgeschrieben bekommt |
| **chance_max** | 0–1: höchste Chance einer Phase nach Lenkung und Fördergeld |
| **lenkung** | Zuordnung `zuegig` und `gruendlich`, je `{monate, chance}` (0,1–10): Faktoren auf Dauer und Chance der Phasen; Texte `startup.lenkung.<normal\|zuegig\|gruendlich>` |
| **rendite_mindest** | 0–100: Mindestertrag je Dollar über 1, den die Strategieabteilung bei Risikobereitschaft 0 verlangt |
| **einsatz_kasse** | 0–1: höchster Einsatz einer Empfehlung als Anteil der Kasse |
| **ausgruendung_fortschritt_min** | 0–1: ab diesem Fortschritt (Punkte / Aufwand) lässt sich ein Forschungsprojekt ausgründen (SU3) |
| **ki** | Zuordnung (SU3): wie KI-Firmen sich beteiligen, Felder siehe unten |

Felder von `ki` (SU3):

| Feld | Bedeutung |
|---|---|
| **pruefen_chance** | 0–1: Wahrscheinlichkeit je Monat, dass eine KI-Firma die Start-ups prüft |
| **kasse_min_usd** | ≥ 0: erst ab dieser Kasse |
| **einsatz_kasse** | 0–1: höchstens dieser Anteil der Kasse für Zusagen im Monat |
| **rendite_mindest** | 0–100: Mindestertrag je Dollar über 1 bei Aggressivität 0 |
| **uebernahme_chance_min** | 0–1: nützliche Start-ups übernimmt sie erst ab dieser gezeigten Chance |
| **uebernahme_anteil_min** | 0–1: und nur, wenn sie schon mindestens diesen Anteil hält |
| **uebernahme_kasse** | 0–1: Kauf und Eingliedern zusammen höchstens dieser Anteil der Kasse |
| **ausgruenden_chance** | 0–1: Wahrscheinlichkeit zum Jahresbeginn, ein Forschungsprojekt auszugründen |
| **ausgruenden_verkauf** | 0–0,99: diesen Anteil verkauft sie dabei an Investoren |

## erfinder

`data/startups/erfinder.yaml` (SU1): historische Erfinder zu ihren Erfindungen. Entsteht
ein Start-up für die Technologie, trägt es den Namen und sitzt im Land des Erfinders
(einmal je Erfinder). Ohne Abschnitt `startups` warnt die Prüfung, dass sie ungenutzt
bleiben. Liste von Einträgen:

| Feld | Bedeutung |
|---|---|
| **technologie** | `id` einer Technologie, je Technologie höchstens ein Erfinder |
| **name** | nicht leer; keine Namen, die heute Marken sind |
| **land** | ISO-Code eines Landes (heutige Grenzen) |
| **annaeherung**, **quelle** | wie bei anderen Daten |

## zoelle

`data/parameter/zoelle.yaml` (W3; Regeln: `docs/FORMELN.md`, Abschnitt W3). Ohne Abschnitt
gibt es keine Zölle. Ein Abschnitt `zoelle` mit:

| Feld | Bedeutung |
|---|---|
| **standard** | Zeitreihe Jahr → Durchschnittszoll (Anteil am Warenwert, 0–5) der Länder ohne eigene Angabe; linear zwischen den Jahren |
| **laender** | Zuordnung ISO-Code → eigene Zeitreihe wie `standard` |
| **warengruppen** | Zuordnung Warengruppe → Faktor (≥ 0) auf den Zoll; ohne Angabe 1 |
| **produkte** | Zuordnung Produkt → Faktor (≥ 0) statt dem seiner Warengruppe (0 = zollfrei, z. B. Dünger) |
| **zonen** | Liste von Handelszonen: `id`, `faktor` (0–1, Zoll zwischen Mitgliedern mal Faktor), `mitglieder` (ISO-Code → `[Beitritt]` oder `[Beitritt, Austritt]`, Austritt nach dem Beitritt; Mitglied bis vor dem Austrittsjahr) |
| **sperren** | Liste von Handelssperren: `laender` (genau zwei verschiedene ISO-Codes), `von`, optional `bis` (Jahr, ausschließlich) |
| **dynamik** | Änderung nach dem letzten Datenjahr: `standardabweichung` (0–1, je Jahr), `minimum`, `maximum` (0–5), `stufen` (Liste aus `id` und `faktor` 0–10 auf die Standardabweichung, Wahl beim neuen Spiel), `standard` (`id` einer Stufe) |
| **annaeherung**, **quelle** | wie bei anderen Daten |

Texte: `zoll.zone.<id>` je Zone, `zoll.dynamik.<id>` je Stufe.

## vertraege

`data/parameter/vertraege.yaml` (W4; Regeln: `docs/FORMELN.md`, Abschnitt W4). Ohne Abschnitt
gibt es keine Lieferverträge. Ein Abschnitt `vertraege` mit:

| Feld | Bedeutung |
|---|---|
| **laufzeit_monate_max** | 1–600: längste Laufzeit |
| **laufzeit_standard** | 1 bis `laufzeit_monate_max`: Laufzeit der Angebote der KI |
| **strafe_max** | 0–10: höchste Vertragsstrafe (Anteil am Wert der fehlenden Menge) |
| **strafe_standard** | 0 bis `strafe_max`: Strafe der Angebote der KI und Vorgabe im Formular |
| **kuendigung_monate** | 0–600: wer kündigt, zahlt die Strafe auf so viele Monatsmengen |
| **angebot_tage** | 1–365: so lange wartet ein Angebot der KI auf die Antwort |
| **aufbewahren_monate** | 0–600: so lange bleiben beendete Verträge in der Liste |
| **ki** | `abschlag_verkauf` und `aufschlag_kauf` (0–1: so viel unter dem eigenen bzw. über dem Marktpreis nimmt die KI an), `anteil` (0–1: Anteil von Leistung oder Bedarf, der in Verträge geht), `strafe_max` (0–10), `angebot_chance` (0–1 je Monat und Produkt des Spielers) |
| **annaeherung**, **quelle** | wie bei anderen Daten |

Texte: `vertrag.abgelehnt.<grund>` (`preis`, `menge`, `qualitaet`, `strafe`).

## logistik

`data/parameter/logistik.yaml` (W5; Regeln: `docs/FORMELN.md`, Abschnitt W5). Ohne Abschnitt
fahren alle Ladungen über den Frachtmarkt. Ein Abschnitt `logistik` mit:

| Feld | Bedeutung |
|---|---|
| **staat** | `aufschlag` (0–10: so viel teurer als der Frachtmarkt), `risiko_faktor` (0–10: Faktor auf das Verlustrisiko) |
| **flotte** | `marge_frachtmarkt` (0–0,9: Marge der Logistikfirmen in der Marktfracht, die eine voll genutzte Flotte spart), `auslastung` (0,01–1: genutzter Anteil der Fahrleistung), `unterhalt_anteil` (0–1 je Jahr vom Kaufwert), `nutzungsdauer_jahre` (> 0: lineare Abschreibung), `verkauf_anteil` (0–1 des Buchwerts beim Verkauf), `vermietung_anteil` (0–1: Anteil des freien Platzes, der Ladung anderer findet), `vermietung_markt_anteil` (0–1: höchstens dieser Anteil der Tonnenkilometer des Frachtmarkts geht an Firmenflotten; W6) |
| **risiko** | `land`, `see`: Jahreswerte (0–1), Wahrscheinlichkeit, eine Ladung zu verlieren |
| **ki** | `anteil` (0–1: so viel ihrer Fracht soll die Flotte einer KI-Firma tragen), `kasse_anteil` (0–1: höchstens so viel der Kasse geht je Monat in Fahrzeuge) |
| **annaeherung**, **quelle** | wie bei anderen Daten |

## produktnamen

Bausteine für die erfundenen Namen, die Firmen ihren Endprodukten geben (in `ki/`, M42;
Regeln: `docs/FORMELN.md`, Abschnitt M42). Optional: ohne den Abschnitt tragen Produkte
keine Namen. Eine einmalige Zuordnung:

| Feld | Bedeutung |
| --- | --- |
| **hausmarke** | 0–1: Anteil, mit dem eine Firma für ein weiteres Produkt desselben Stils einen eigenen Stamm wiederverwendet |
| ausgeschlossen | Echte Produkt- und Markennamen. Kein Stamm, Buchstabe oder Zusatz darf eines dieser Wörter enthalten; Namen mit einem solchen Wort erzeugt das Spiel nicht und nimmt es vom Spieler nicht an (ohne Unterschied von Groß- und Kleinschreibung) |
| **stile** | Liste der Namensstile: `id` (eindeutig), `warengruppen` (nicht leer, jede Warengruppe in höchstens einem Stil; benannt werden die Endprodukte dieser Gruppen), `staemme` (erfundene Wörter, nicht leer, ohne Doppelte), `muster` (nicht leer, siehe unten), `zahlen` (über 0, ohne Doppelte), `buchstaben`, `zusaetze`, `nachfolger` (Kennzeichen der Nachfolgemodelle in Reihenfolge, z. B. `[II, III]`; einzelne Wörter ohne Ziffern, ohne Doppelte; leer: keine Nachfolgemodelle, B1) |

Ein Muster ist `{text, ab, bis}`: `text` mit `{stamm}` (Pflicht) und wahlweise `{zahl}`,
`{buchstabe}`, `{zusatz}` – jeder benutzte Platzhalter braucht eine nicht leere Liste;
`ab` und `bis` (Jahre, beide optional, `ab` ≤ `bis`) begrenzen, wann das Muster gilt.
Mindestens ein Muster je Stil gilt ohne Zeitgrenzen.

## reale_firmen

Historische Firmen mit ihrem Stand im frühesten Startjahr 1900 (in `ki/`). Sie zählen zu
den KI-Firmen; ihre Anlagenzahl wird mit dem Marktmaßstab verkleinert (mindestens 1).

| Feld | Bedeutung |
| --- | --- |
| **id** | Schlüssel |
| **name** | Firmenname (Eigenname, nicht übersetzt) |
| **sitz** | Land des Hauptsitzes |
| **gegruendet** | Gründungsjahr, höchstens 1900 |
| **standorte** | Liste mit `land`, `lagerstaette` (Pflicht bei Förderanlagen, im selben Land) und `anlagen`: Liste aus `anlage`, `anzahl` (über 0) und `rezept` (optional, muss zur Anlage passen). Alle Anlagen eines Standorts haben denselben Standorttyp; Anlagen und Rezepte dürfen keine Technologie nach 1900 brauchen. |
| kompetenz, aggressivitaet | Feste Werte (0–1) statt der Schwierigkeit |
| annaeherung, quelle | Herkunft |

## ereignisse

Historische Ereignisse (in `ereignisse/`, 1900–2026), Lastenheft §4.1. Sie erscheinen als
Weltereignis im Rundenbericht; ihre Wirkungen (H1) gelten für Spieler und KI-Firmen. Texte:
`ereignis.<id>` (Titel) und `ereignis.<id>.text` (Beschreibung), beide Pflicht.

| Feld | Bedeutung |
| --- | --- |
| **id** | Schlüssel |
| **datum** | Tag des Ereignisses, `"JJJJ-MM-TT"` (in Anführungszeichen) |
| **art** | `krieg`, `kriegsende`, `krise`, `revolution`, `staatsgruendung`, `abkommen`, `katastrophe`, `technik` oder `reform` (Wirtschaftspolitik, M28); Text `ereignisart.<art>` |
| laender | Betroffene Länder (heutige Grenzen) |
| wirkungen | Liste der Wirkungen (H1, unten) |
| annaeherung, quelle | Herkunft |

Eine **Wirkung** hat ein Feld `art` und je nach Art weitere Felder (Regeln: `docs/FORMELN.md`,
Abschnitt H1). `laender` (Länder, in denen sie wirkt) gilt für alle Arten außer
`boersenkrach`; ohne Angabe die Länder des Ereignisses. Wirkungen mit Dauer nehmen `bis`
(`"JJJJ-MM-TT"`, nach dem Ereignistag; ohne: bis zum Spielende). Andere Felder als die
genannten werden abgelehnt.

| art | Felder | Bedeutung |
| --- | --- | --- |
| `nachfrage` | `warengruppen` (ohne: alle), `konsum`, `staat` (je 0–10, Standard 1; mindestens einer ≠ 1), `bis` | Faktor auf Verbraucher- bzw. Staatsnachfrage |
| `handelssperre` | **gegen** (Länder, nicht zugleich in `laender`), `bis` | kein Handel zwischen den beiden Seiten |
| `zoll` | `gegen` (ohne: alle anderen), **aufschlag** (0–5), `bis` | zusätzlicher Zoll auf Einfuhren in die Länder |
| `arbeitskraefte` | **faktor** (0,05–2), `bis` | verfügbare Arbeitskräfte |
| `produktion` | `warengruppen` (ohne: alle), **faktor** (0–2), `bis` | höchstens so viel der geplanten Läufe |
| `abschottung` | `alle` (ja/nein, Standard nein), `bis` | keine neuen oder gekauften Standorte von Firmen anderer Länder; mit `alle` nur noch der Staatsbetrieb |
| `zerstoerung` | **anteil** (0–1) | einmalig: dieser Anteil der Anlagen und Lager |
| `enteignung` | `nur_auslaendische` (Standard ja), `entschaedigung` (0–1, Standard 0) | einmalig: Standorte gehen an den Staatsbetrieb |
| `boersenkrach` | **einbruch** (0–0,95) | Stimmung der Börse am folgenden Monatsersten |

Beispiel: `- {art: handelssperre, laender: [GBR, FRA], gegen: [DEU], bis: "1919-07-12"}`

## ereignisfolgen

`data/parameter/ereignisse.yaml` (H1), optional. Ein Abschnitt `ereignisfolgen` mit:

| Feld | Bedeutung |
|---|---|
| **staatsbetrieb** | `betriebskapital_anteil` (0–1: Kasse, die der Staat seinem Betrieb zu enteigneten Standorten gibt, als Anteil an deren Buchwert; ohne Abschnitt 0) |
| **annaeherung**, **quelle** | wie bei anderen Daten |


## lebenslauf

`data/parameter/lebenslauf.yaml` (PE1), optional; ohne den Abschnitt altern Manager nicht.
Ein Abschnitt `lebenslauf` mit:

| Feld | Bedeutung |
|---|---|
| **eintrittsalter** | Je Ebene `standort`, `land`, `kontinent`, `vorstand`: `von`, `bis` (16–100 Jahre, `von` ≤ `bis`), `mittel` (zwischen beiden), `streuung` (> 0) der abgeschnittenen Normalverteilung des Alters beim Eintritt in den Pool |
| **ebene_nach_staerke** | `land`, `kontinent`, `vorstand` (0–100, steigend): ab dieser Stärke wird ein neuer Kandidat für diese Ebene gezogen |
| **erfahrung** | `jung_bis`, `alt_ab` (Jahre), `jung`, `alt` (Faktoren 0–10 auf die Erfahrungschance, MA6) |
| **risiko** | `ab` (Jahre), `je_jahr` (0–10 Punkte), `hoechstens` (0–100 Punkte): Abnahme der Risikofreude |
| **abbau** | `ab` (Jahre), `chance` (0–1 je Geburtstag) für Erkennen und Führung |
| **ruhestand** | `abweichung` (0–20 Jahre, persönlich ±), `vorwarnung_monate` (1–60), `verlaengerung_jahre_max` (0–10), `verlaengerung_aufschlag` (0–5, Gehaltsaufschlag), `bezugsalter` (Jahre: das Ruhestandsalter dieses Lebensjahres gilt), `zusage_alter` (`[von, bis]`, steigend: die Zusage zur Verlängerung sinkt dazwischen von voll auf null) |
| **sterbetafel** | `ab` (Jahre), `chance` (0–1, Sterbechance je Monat bei der Lebenserwartung), `verdopplung_jahre` (> 0) |
| **ruhestandsalter** | Länder-Zeitreihe: `standard` (Jahr → Alter 40–90) und optional `laender` (ISO-Code → eigene Reihe) |
| **lebenserwartung** | Länder-Zeitreihe wie oben (30–110 Jahre): Alter, in dem die Sterbechance `chance` erreicht |
| **annaeherung**, **quelle** | wie bei anderen Daten |

Das Thema `nachfolge` gehört in `management.bereiche` zum Personalbereich: Dessen Stellen
fragen nach der Nachfolge.

## person

`data/parameter/person.yaml` (PE2, PE3), optional; ohne den Abschnitt bekommt die Person
keine Kinder, und Gründung, Lebensstil, Steuern und Zinsen kosten und bringen nichts. Ein
Abschnitt `person` mit:

| Feld | Bedeutung |
|---|---|
| **alter_start** | `standard`, `von`, `bis` (16–100 Jahre, `von` ≤ `standard` ≤ `bis`): Alter der Person beim Spielbeginn |
| **familie** | `kinder_ab`, `kinder_bis` (Jahre, steigend: Alter der Person, in dem Kinder kommen), `kinder_hoechstens` (0–20), `kinder_chance_jahr` (0–1), `managerkarte_ab` (Jahre: ab dann arbeitet ein Kind als Manager) |
| **gruendung** | `kosten_anteil` (0–1: Gründungskosten als Anteil der Einlage), `kosten_mindestens_monatsloehne` (0–120: mindestens so viele Akademiker-Monatslöhne im Sitzland), `einlage_vorschlag` (0–1: vorgeschlagene Einlage als Anteil des Privatkontos) |
| **gehalt** | `hoechstens_mitgesellschafter` (1–100: Gehalt der Person als CEO mit Mitgesellschaftern höchstens dieses Vielfache des Vorschlags) |
| **darlehen** | `zins_hoechstens` (0–1 je Jahr), `jahre_hoechstens` (1–100): Grenzen eines Gesellschafterdarlehens |
| **lebensstil** | `standard` (eine der Stufen), `wechsel_monate` (0–120: höchstens ein Wechsel in so vielen Monaten), `stufen` mit genau `bescheiden`, `buergerlich`, `gehoben`, `luxurioes`, je `kosten` (0–1000 Akademiker-Monatslöhne je Monat, von Stufe zu Stufe steigend), `zins` (−0,1–0,1: Auf- oder Abschlag auf neue Bankkredite), `gehaltsforderung` (−0,9–1: Anteil mehr oder weniger Forderung im Vorstand), `ausbildung` (0–50 Punkte auf die Fähigkeiten der Kinder), `sterblichkeit` (0–10: Faktor auf die Sterbechance der Person) |
| **einkommensteuer** | Satz je Land und Jahr (0–1) wie `lebenslauf.ruhestandsalter`: `standard` und `laender` mit eigenen Reihen |
| **sparzins** | Guthabenzins des Privatkontos je Land und Jahr (−0,5–0,5), Aufbau wie `einkommensteuer` |
| **erbe** | Erbe der Person (PE6): `neffe_alter: {von, bis}` (18–80 Jahre, von ≤ bis: so alt ist ein erzeugter Neffe oder eine Nichte, wenn kein Kind lebt), `hinweis_ab` (18–120: ab diesem Alter jeden Januar der Hinweis „Nachfolge bedenken“, solange kein Erbe bestimmt ist) |
| **erbschaftsteuer** | Erbschaft- und Schenkungsteuer je Land und Jahr (0–1), Aufbau wie `einkommensteuer`: ein Satz auf den ganzen Nachlass |
| **annaeherung**, **quelle** | wie bei anderen Daten |

## umwelt

Ein einziger Abschnitt (in `parameter/umwelt.yaml`): Umwelt und Regulierung (H2). Formeln in
`docs/FORMELN.md` (H2). Ohne den Abschnitt gibt es keine Nachrüstung, keinen CO₂-Preis,
keine Kartellgrenze und keinen Einfluss auf das Markenbild.

| Feld | Bedeutung |
| --- | --- |
| **nachruestung** | Stufen der Abgasreinigung, die erste zuerst: `{ab, minderung, kosten_anteil}` – ab dem Jahr verfügbar (Jahre aufsteigend), Minderung der Schadstoffe 0–1, Kosten als Anteil der Investition der Einheit 0–10 |
| **co2_preis** | USD (Kaufkraft 2026) je t CO₂ je Land und Jahr (0–10 000), Aufbau wie `person.einkommensteuer` |
| **kartell** | `marktanteil_max` (0–1): gemeinsamer Marktanteil, ab dem die Kartellaufsicht eine Übernahme untersagt |
| **markenbild** | `gewicht` (0–1): so stark biegt die Schadstoffintensität den Zuwachs an Bekanntheit |

## regulierungen

Liste (in `regulierungen.yaml`): Regeln je Land ab einem Datum (H2). Jede braucht den Text
`regulierung.<id>`.

| Feld | Bedeutung |
| --- | --- |
| **id** | Schlüssel (snake_case) |
| **datum** | JJJJ-MM-TT, gilt ab dem Monat |
| **laender** | Länder (ISO), mindestens eines |
| **art** | `auflage` (mit **stufe** 1 bis Zahl der Nachrüststufen und **frist_monate** 0–240), `arbeitsschutz` (mit **lohnaufschlag** 0–1), `verbot` (mit **produkte**, mindestens eines, und `herstellung` und/oder `verkauf`: true), `kartellaufsicht` (ohne weitere Felder). Felder einer anderen Art sind Fehler. |
| `annaeherung`, `quelle` | wie überall |

## preisindex

Verbraucherpreise der Leitwährung (in `waehrungen/preisindex.yaml`), einmalig. Sie
rechnen Spieldollar in Dollar der jeweiligen Zeit um (Anzeige „Preise der Zeit“,
docs/FORMELN.md M21).

| Feld | Bedeutung |
| --- | --- |
| **leitwaehrung** | Währung, in der das Spiel rechnet; ihre Kurse müssen alle 1 sein |
| **basisjahr** | Jahr, dessen Kaufkraft die Spieldollar haben; braucht einen Wert in `werte` |
| **werte** | Jahresmittel des Index, `{1900: 8.4, …}`, Jahre 1900–2100, Werte über 0 |
| **teuerung_danach** | Angenommene Teuerung je Jahr nach dem letzten Wert, zwischen −0,5 und 1 |
| annaeherung, quelle | Herkunft |

## waehrungen

Währungen mit ihrem Kurs zur Leitwährung (in `waehrungen/`). Text: `waehrung.<id>`.

| Feld | Bedeutung |
| --- | --- |
| **id** | Schlüssel, z. B. `reichsmark` |
| **zeichen** | Kurzzeichen der Anzeige, z. B. `RM`, `€`; nicht leer |
| kurse | Einheiten je US-Dollar der Zeit (nominal), `{1924: 4.2, "1933-03": 4.2}`. Schlüssel: Jahr (steht für das Jahresmittel) oder `"JJJJ-MM"` (Monatsmitte), 1900–2100; Werte über 0. Dazwischen logarithmisch interpoliert, davor und danach gilt der nächste Wert. |
| bindung | Statt `kurse`: fest an eine andere Währung gebunden, mit `an` (Währung mit eigenen Kursen, nicht selbst gebunden) und `faktor` (Einheiten je Einheit von `an`, über 0) |
| annaeherung, quelle | Herkunft |

Genau eines von `kurse` und `bindung` ist anzugeben. Eine Währung, die kein Land nutzt
und an die keine andere gebunden ist, ergibt eine Warnung.

## landeswaehrungen

Welche Währung ein Land wann nutzt (in `waehrungen/laender.yaml`). Jedes Land braucht
genau einen Eintrag. Vor einer eigenen Landeswährung gilt die Währung der damaligen
Kolonial- oder Vormacht (Annahme, im Kommentar begründet).

| Feld | Bedeutung |
| --- | --- |
| **land** | Land (ISO-3166-alpha-3) |
| **perioden** | Nicht leere Liste aus `ab` (Jahr = 1. Januar oder `"JJJJ-MM"` = 1. des Monats) und `waehrung`; aufsteigend nach `ab`, der erste Zeitraum beginnt spätestens 1900 |
| perioden[].umrechnung | Gesetzlicher Umstellungskurs (M28): Einheiten der bisherigen Währung für eine Einheit der neuen, > 0 (Euro: 1,95583 für die D-Mark). Nur bei einem Wechsel der Währung. Ohne ihn nennt die Meldung zur Währungsumstellung das Verhältnis der Wechselkurse |

## etappen

Etappenziele des Spielers nach der Einführung (M23), in der Reihenfolge der Übersicht.
Sie wirken nicht auf die Simulation. Texte: `etappe.<id>` (Name) und
`etappe.<id>.hinweis` (so erreicht man sie; `{anteil}` wird beim Marktführer durch den
geforderten Anteil ersetzt), beide Pflicht. Bewertung: `docs/FORMELN.md`, M23.

| Feld | Bedeutung |
| --- | --- |
| **id** | Schlüssel |
| **art** | `erster_verkauf`, `gewinnmonat`, `anlagen`, `eigenes_vorprodukt`, `laender`, `forschung`, `marktfuehrer` oder `eigenkapital` |
| wert | Nur und genau bei `anlagen` und `laender` (ganze Zahl 1–1000), `marktfuehrer` (Anteil am Absatz, 0,01–1) und `eigenkapital` (Vielfaches des Startkapitals, über 1 bis 1000) |

