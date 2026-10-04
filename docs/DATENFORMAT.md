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
  texte/de/            alle Anzeigetexte
```

Die Aufteilung auf Dateien ist frei: Jede Datei besteht aus Abschnitten, der Lader
sammelt alle Dateien (`*.yaml`, nicht `*.yml`) und fügt die Abschnitte zusammen.
Erlaubte Abschnitte: `meta`, `laendermodell`, `produktionsmodell`, `finanzmodell`,
`marktmodell`, `transportmodell`, `forschungsmodell`, `einheiten`, `kontinente`, `branchen`, `warengruppen`,
`transportklassen`, `qualifikationen`, `fachrichtungen`, `laender`, `produkte`,
`anlagen`, `rezepte`, `technologien`, `lagerstaetten`, `verkehrsmittel`, `kimodell`,
`namensgruppen`, `reale_firmen`, `ereignisse`. Jeder Abschnitt außer `meta`
ist eine Liste von Einträgen (`meta` und die Modelle in `parameter/` sind einmalige Zuordnungen).

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
  `produkt`, `anlage`, `rezept`, `technologie`, `lagerstaette`, `verkehrsmittel`,
  `schwierigkeit`, `ereignis`. Namen von Firmen sind Eigennamen und brauchen keinen Text.
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

## produktionsmodell

Ein einziger Abschnitt (in `parameter/produktionsmodell.yaml`); Formeln in
`docs/FORMELN.md` (M5).

| Feld | Bedeutung |
| --- | --- |
| **standortkosten_usd** | Kosten je Standorttyp: `foerderstaette`, `werk`, `kraftwerk`, `lager`, `niederlassung`, `forschungszentrum` (alle nötig) |
| **gebaeude_lebensdauer_jahre**, **erschliessung_lebensdauer_jahre** | Abschreibungsdauern |
| **automatisierung** | `arbeitsersparnis` (0–1), `kostenanteil` |
| **qualitaet** | Gewichte `vorprodukte`, `automatisierung`, `zustand` |
| **zustand_minimum** | Untergrenze des Anlagenzustands (0–1) |
| strom | Produkt der Art `energie`, das für Eigenstrom steht (Kette 7) |
| **einspeiseverguetung** | Anteil (0–1) des Industriestrompreises für überschüssigen Eigenstrom |
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
| **preisanpassung** | `hoch`, `runter` (je Tag), `lagertage`, `hoechstfaktor` (automatische Preise höchstens dieses Vielfache des Richtpreises im Land, 1–1000) |
| **staat_hoechstpreis** | Staaten zahlen höchstens dieses Vielfache des Richtpreises |
| **index_glaettung** | Gewicht des Tagesdurchschnitts im Marktpreis |
| **haendler** | KI-Händler (M8): `marge` (Aufschlag auf Einkauf und Transport), `vorrat_tage` (Lager für so viele Tage offener Nachfrage), `glaettung_tage` (1–365, Mittelung der offenen Nachfrage) |

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
| staatsmarkt.**preis_usd** | Ware ist in jedem Land vom staatlichen Markt zu diesem Preis erhältlich |
| staatsmarkt.verfuegbar_ab / verfuegbar_bis | Jahre, in denen der Staatsmarkt die Ware anbietet |
| ersetzt | Liste von Produkten, die dieses Produkt nach und nach verdrängt |
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
| eingang | `{produkt: menge}` je Durchlauf; nicht das eigene Produkt; höchstens vier Vorprodukte (Strom zählt nicht). Der Produktbaum vom Rohstoff bis zum Produkt hat höchstens sechs Ebenen, mehr als vier nur bei `sehr_komplex` (Lastenheft §17.2) |
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
| **start** | `auslastung`, `anlage_mindestanteil`, `lager_eingang_tage`, `lager_ausgang_tage`, `kasse_monate`, `gewicht_entwicklung` (je Produktart: `rohstoff`, `halbzeug`, `bauteil`, `endprodukt`, `energie`), `referenzlohn_usd` |
| **verhalten** | `betrieb_alle_tage`, `lager_hoch_tage`, `lager_niedrig_tage` (kleiner als hoch), `auslastung_schritt`, `auslastung_min`, `preisuntergrenze`, `einkauf_aufschlag`, `ausbau_auslastung`, `ausbau_marge`, `ausbau_anteil_kasse_max`, `forschung_vorgriff_jahre`, `forschung_mindestumsatz_usd`, `forschung_mindestkompetenz`, `kasse_min_monate` (kleiner als max), `kasse_max_monate`, `kredit_jahre`, `gruendungen_je_monat`, `diversifikationen_je_quartal` (Firmen je Quartal, die in einem fremden Engpass bauen), `gruendung_kapitalfaktor` |

## namensgruppen

Namensbausteine für erzeugte KI-Firmen (in `ki/`). Eine Firma nimmt die Gruppe ihres
Sitzlandes, sonst die eine Gruppe mit `standard: true`.

| Feld | Bedeutung |
| --- | --- |
| **id** | Schlüssel der Gruppe |
| laender | Länder der Gruppe; jedes Land gehört zu höchstens einer Gruppe |
| standard | `true` für genau eine Gruppe |
| **familiennamen**, **orte**, **rechtsformen** | Nicht leere Listen von Namensteilen |
| **muster** | Namensmuster mit `{familienname}`, `{ort}`, `{rechtsform}`, `{branche}` |
| **branchen** | Wort für das Geschäft je Branche, z. B. `metallurgie: Hüttenwerke` |

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

Historische Ereignisse (in `ereignisse/`), Lastenheft §4.1. In Stufe 1 erscheinen sie
als Weltereignis im Rundenbericht; Wirkungen folgen mit Stufe 4. Texte:
`ereignis.<id>` (Titel) und `ereignis.<id>.text` (Beschreibung), beide Pflicht.

| Feld | Bedeutung |
| --- | --- |
| **id** | Schlüssel |
| **datum** | Tag des Ereignisses, `"JJJJ-MM-TT"` (in Anführungszeichen) |
| **art** | `krieg`, `kriegsende`, `krise`, `revolution`, `staatsgruendung`, `abkommen`, `katastrophe` oder `technik`; Text `ereignisart.<art>` |
| laender | Betroffene Länder (heutige Grenzen) |
| annaeherung, quelle | Herkunft |

