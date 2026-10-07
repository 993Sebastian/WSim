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
  verkehrsmittel.yaml  Verkehrsmittel mit Kosten und Geschwindigkeit
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
`namensgruppen`, `produktnamen`, `reale_firmen`, `ereignisse`, `preisindex`, `waehrungen`,
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
| `einheiten` | **id**, gewicht_kg (Gewicht einer Einheit; fehlt es, braucht jedes Produkt mit dieser Einheit ein eigenes `gewicht_kg`) |
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

## finanzmodell

Ein einziger Abschnitt (in `parameter/finanzmodell.yaml`); Formeln in `docs/FORMELN.md` (M6).

| Feld | Bedeutung |
| --- | --- |
| **realzins** | Realer Leitzins als Jahreswerte (−0,2 bis 0,5) |
| **risikoaufschlag** | `minimum` und `je_verschuldung` (Aufschlag je Verschuldungsgrad) |
| **beleihung** | Anteil der Sachwerte, bis zu dem Banken Kredit geben (0–1) |
| **dispo** | `anteil` der Bilanzsumme als Kreditlinie, `aufschlag` auf den Leitzins |
| **laufzeit_max_jahre** | Längste Kreditlaufzeit |

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
| **richtpreis_usd** | Typischer Preis je Einheit um 1900 (bei Preisniveau 1); Startwert der Märkte |
| gewicht_kg | Gewicht einer Einheit; nötig, wenn die Einheit keines festlegt |
| heizwert_mwh | Energiegehalt je Einheit, wenn das Produkt als Brennstoff dient |
| nachfrage | Endkunden-Nachfrage, siehe unten |
| staatsnachfrage.**je_mio_usd_bip** | Staatlicher Bedarf in Einheiten je Jahr und Mio. USD BIP |
| staatsnachfrage.kriegsfaktor | Faktor in Kriegszeiten (Standard 1, wirkt ab Stufe 4) |
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
| **verhalten** | `betrieb_alle_tage`, `lager_hoch_tage`, `lager_niedrig_tage` (kleiner als hoch), `lager_ziel_tage` (Lager der Erzeugnisse in Tagen des Abgangs), `lager_ausgleich_tage` (in so vielen Tagen wird die Lücke zum Ziel geschlossen), `auslastung_schritt`, `auslastung_min`, `auslastung_aenderung_max` (0,01–1: so stark ändert sich die geplante Auslastung je Entscheidung höchstens), `preisuntergrenze`, `schaetzfehler` (`bei_0`/`bei_1` nach Kompetenz, je 0–1: so weit liegen Stückkosten und Marge in der Schätzung einer KI-Firma je Produkt und Jahr daneben; B2), `werbeanteil` (`bei_0`/`bei_1` nach Aggressivität: Werbebudget als Anteil des Vormonatsumsatzes je Land und Warengruppe), `schulung` (`bei_0`/`bei_1` nach Kompetenz, je 0–1: Schulungsziel der Standorte einer KI-Firma, auf 5 % gerundet; W1), `einkauf_aufschlag`, `ausbau_auslastung`, `ausbau_marge`, `ausbau_vorprodukt_preis_max` (1–10: kein Ausbau, solange ein Vorprodukt im Land mehr als dieses Vielfache seines Richtpreises kostet), `ausbau_anteil_kasse_max`, `ausbau_je_pruefung_max` (1–50: so viele Ausbauten beginnt eine Firma je Prüfung, die mit der höchsten Marge zuerst; C2), `forschung_vorgriff_jahre`, `forschung_mindestumsatz_usd`, `forschung_mindestkompetenz`, `entwicklung_nutzen_je_stufe` (0–1, geschätzter Jahresnutzen einer Entwicklungsstufe als Anteil am Umsatz mit dem Produkt; M37), `entwicklung_amortisation_jahre` (0–50: so schnell muss sich die nächste Stufe bezahlt machen), `forschung_luecke_firmen` (so viele Firmen erforschen dieselbe Technologie für eine Marktlücke zugleich, 0 = keine; M32), `forschung_vorlauf_jahre` (0–30, Standard 0: Produkte mit Nachfrage, die sich historisch in so vielen Jahren herstellen lassen, zählen samt ihren Vorprodukten schon als Marktlücke; M39), `kasse_min_monate` (kleiner als max), `kasse_max_monate`, `kredit_jahre`, `gruendungen_je_monat`, `diversifikationen_je_quartal` (Firmen je Quartal, die in einem fremden Engpass bauen), `einstieg_preisfaktor` (1–10), `einstieg_firmen_max` (0 = nie) und `einstieg_anteil` (0,01–1): zahlen die Käufer weltweit wenigstens dieses Vielfache des Richtpreises und stellen weniger Firmen das Produkt her, plant eine weitere Firma für diesen Anteil des Absatzes (M33), `vorrat_jahre_min` (≥ 0: eine neue Konzession nur, wenn der Restvorrat der Lagerstätte so viele Jahre ihrer Höchstförderung reicht; M33). Die Firmenzahlen `forschung_luecke_firmen`, `gruendungen_je_monat`, `diversifikationen_je_quartal` und `einstieg_firmen_max` gelten für `firmen_standard` KI-Firmen und wachsen mit mehr Firmen im selben Verhältnis, `gruendung_kapitalfaktor`, `lohnaufschlag_schritt` (0–1: so weit hebt oder senkt eine KI-Firma den Lohnaufschlag eines Standorts je Entscheidung), `lohnaufschlag_max` (0–5: höchster Aufschlag der KI; M18), `stilllegen_auslastung` (0–1, kleiner als `stilllegen_zielauslastung`: darunter legt die KI überzählige Einheiten still), `stilllegen_zielauslastung` (0,1–1), `stilllegen_preis_max` (0,1–10: nur wo der Preis im Land unter diesem Vielfachen des Richtpreises liegt), `wiederanfahren_auslastung` (0,1–1, größer als `stilllegen_zielauslastung`: darüber fährt die KI stillgelegte Einheiten wieder an), `verkaufen_nach_monaten` (so lange stillgelegte Einheiten werden verkauft; M22) |

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
| **insolvenz** | Pleiten (M38): `tage` (0–365; so lange werden die Standorte einer zahlungsunfähigen KI-Firma versteigert, 0 = sofort aufgeben), `mindestpreis` (0–1, Mindestgebot als Anteil des Grundwerts) |
| **ki** | Verhalten der KI-Firmen: `angebot_chance` (je 0–1, Wahrscheinlichkeit je Monat), `offene_angebote_max` (0–20), `spieler_angebote_je_monat` (0–10, von allen KI-Firmen zusammen), `wettbewerb_aufschlag` (je 0–5), `fachkraefte_aufschlag` (0–5), `bauzeit_aufschlag` (0–5), `neubau_anteil` (0–1), `mindestvorteil` (0–5), `gebotsaufschlag` (je 0–5), `mindestpreis_usd` (≥ 0), `kasse_anteil_max` (0–1), `lizenz_gebot` (je 0–2), `lizenz_hoechst` (0–2), `verkaufsaufschlag` (je 0–5), `kern_anteil` (0–1), `kern_aufschlag` (0–10), `lizenz_mindest` (0–2), `wettbewerb_lizenz` (0–10), `gegen_schwelle` (0–1) |

## grundstuecksmodell

Ein einziger Abschnitt (in `parameter/grundstuecksmodell.yaml`): Grundstücke der
Standorte (M35). Formeln in `docs/FORMELN.md` (M35).

| Feld | Bedeutung |
| --- | --- |
| **flaeche_ha_je_mrd_bip** | > 0: Gewerbefläche eines Landes je Mrd. USD BIP (mal Marktmaßstab) |
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

Historische Ereignisse (in `ereignisse/`, 1900–2026), Lastenheft §4.1. In Stufe 1 erscheinen sie
als Weltereignis im Rundenbericht; Wirkungen folgen mit Stufe 4. Texte:
`ereignis.<id>` (Titel) und `ereignis.<id>.text` (Beschreibung), beide Pflicht.

| Feld | Bedeutung |
| --- | --- |
| **id** | Schlüssel |
| **datum** | Tag des Ereignisses, `"JJJJ-MM-TT"` (in Anführungszeichen) |
| **art** | `krieg`, `kriegsende`, `krise`, `revolution`, `staatsgruendung`, `abkommen`, `katastrophe`, `technik` oder `reform` (Wirtschaftspolitik, M28); Text `ereignisart.<art>` |
| laender | Betroffene Länder (heutige Grenzen) |
| annaeherung, quelle | Herkunft |


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

