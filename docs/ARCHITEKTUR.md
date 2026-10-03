# Architektur WSim

Bezug: `docs/LASTENHEFT.md`. Umgesetzt wird nur Ausbaustufe 1 (§17), die Architektur
ist aber auf alle sechs Stufen ausgelegt. Technologie freigegeben am 03.10.2026.
Dieses Dokument wird mit jedem Meilenstein fortgeschrieben.

---

## 1 Technologie

### Vorschlag: Rust-Simulationskern + Tauri 2 + TypeScript/React-Oberfläche

| Baustein | Wahl | Begründung |
| --- | --- | --- |
| Simulationskern | **Rust** (Bibliothek `wsim-core`) | Native Geschwindigkeit ohne Garbage Collector – wichtig für „sehr viele KI-Firmen“ und Tages-Ticks über 200 Jahre (≈ 73 000 Schritte). Deterministisch beherrschbar (keine versteckte Nebenläufigkeit, feste Iterationsreihenfolge, Ganzzahl-Geld). `cargo test` macht den Kern ohne Oberfläche test- und lauffähig (§16.1). `serde` liefert robustes Laden von YAML/JSON und versionierte Spielstände. |
| Desktop-Hülle | **Tauri 2** | Erzeugt einen echten Windows-Installer (NSIS-`setup.exe` oder MSI), startet wie ein normales Programm, läuft komplett offline. Klein (≈ 10–20 MB statt 150 MB bei Electron). Die WebView2-Laufzeit wird als Offline-Installer eingebettet, damit auch Rechner ohne Internet installieren können. |
| Oberfläche | **TypeScript + React + Vite** | Tabellen, Diagramme und Karten sind die Stärke von Web-Technik: **TanStack Table** (sortieren, filtern, virtualisiert – zehntausende Zeilen), **Apache ECharts** (Zeitreihen, Balken, Sankey für Produktionsketten, Choroplethen-Weltkarte mit Ebenen, Zoom). Nüchtern-modernes Design ist mit CSS leicht erreichbar. |
| Kartendaten | **Natural Earth** Admin-0 (gemeinfrei), vereinfacht als TopoJSON | Heutige Grenzen, ISO-3166-Codes als Länder-IDs. |
| Datenformat | **YAML** für Spielinhalte, MessagePack + DEFLATE für Spielstände | YAML ist kommentierbar und gut von Hand editierbar; Spielstände kompakt und schnell, Kompression in reinem Rust (keine C-Abhängigkeit). |

**Warum diese Trennung zwei Sprachen rechtfertigt:** Das Lastenheft verlangt einen
Kern, der ohne Oberfläche lauffähig ist. Mit Rust-Kern und Web-Oberfläche ist diese
Trennung physisch erzwungen – die Oberfläche *kann* gar keine Spiellogik enthalten,
sie sendet nur Befehle und liest Sichten. Zudem ist alles in dieser Linux-Umgebung
baubar und testbar (Kern per `cargo test`, Oberfläche per Vitest und Playwright
gegen den Web-Build); der Windows-Installer entsteht in GitHub Actions auf einem
Windows-Runner.

### Geprüfte Alternativen

| Alternative | Warum nicht |
| --- | --- |
| C# / .NET + Avalonia | Ernsthafte Alternative (eine Sprache, schnell). Schwächer bei Tabellen-/Diagramm-/Kartenbibliotheken; Teile des Avalonia-Ökosystems (TreeDataGrid) sind inzwischen kommerziell. |
| C# + WPF | Nur unter Windows bau- und testbar – hier nicht prüfbar. |
| Electron + TypeScript überall | Einfachste Lösung, aber Simulation in JS 3–10× langsamer, Determinismus über Engine-Versionen schwerer zu garantieren, großer Installer. |
| Godot / Unity | Spiel-Engines lohnen sich bei Grafik; für Tabellen und Diagramme sind sie umständlich. |
| Python + Qt | Simulation zu langsam für viele KI-Firmen. |

---

## 2 Architektur

### 2.1 Gesamtbild

```
┌──────────────────── Windows-Programm (Tauri) ────────────────────┐
│  ui/  TypeScript + React                                         │
│       Ansichten, Tabellen, Diagramme, Weltkarte, Texte (i18n)    │
│       ── sendet Befehle, liest Sichten, enthält KEINE Spiellogik │
│                         ▲  JSON über Tauri-IPC                   │
│  app/src-tauri/  dünner Adapter: Sim-Thread, Fortschritt, Dateien│
└─────────────────────────┼────────────────────────────────────────┘
                          ▼
      crates/wsim-core   Simulationskern (kein UI, kein Datei-IO)
              ▲
      crates/wsim-data   YAML laden → prüfen → kompilierter Katalog
              ▲
      crates/wsim-cli    Kommandozeile: Daten prüfen, Läufe ohne UI,
                         Balance- und Leistungsprotokolle
      data/              Spielinhalte (YAML) + Texte, getrennt vom Code
```

### 2.2 Simulationskern (`wsim-core`)

**Katalog und Zustand getrennt.**
- *Katalog*: unveränderliche Spielinhalte aus `data/` (Länder-Zeitreihen, Produkte,
  Rezepte, Anlagen, Technologien, Lagerstätten, Parameter). Beim Laden werden
  Text-IDs in kompakte Zahlen-IDs (`ProduktId(u16)`, `LandId(u16)` …) übersetzt.
- *Zustand*: alles, was sich im Spiel ändert (Firmen, Standorte, Lager, Märkte,
  Konten, Forschung, Kalender, Zufallsgeneratoren). Nur der Zustand wird gespeichert.

**Zeit und Runden (§13).**
- Interner Schritt = 1 Tag. Eine Runde = Tag, Woche, Monat oder Quartal.
- Feste Systemreihenfolge je Tag:
  1. Kalender, Länderwerte interpolieren (Jahreswerte → Tageswert)
  2. Befehle anwenden (Spieler zu Rundenbeginn; KI-Firmen in ihrem Rhythmus)
  3. Rohstoffabbau, Erschöpfung von Lagerstätten
  4. Produktion (Rezept, Arbeitskräfte, Energie, Anlagenzustand → Menge + Qualität)
  5. Transport (in Stufe 1 vereinfacht, siehe offene Punkte)
  6. Märkte: Nachfrage bilden, Angebote zuteilen, Preise anpassen
  7. Finanzen: Löhne, Energie, Zinsen, Abschreibungen, Steuern buchen
  8. Forschung
  9. Warnungen und Meldungen erzeugen
- KI-Firmen entscheiden gestaffelt (Firma *i* z. B. wöchentlich an Tag `(t+i) mod 7`),
  damit die Last gleichmäßig verteilt ist und lange Runden mehrere KI-Entscheidungen
  enthalten.
- `advance(tage, fortschritt_callback)` liefert einen Rundenbericht zurück.

**Ein Regelwerk für alle (§10).** Jede Handlung – Werk bauen, Preis setzen,
Forschung starten, Kredit aufnehmen – ist ein `Befehl`, der zentral geprüft und
ausgeführt wird. Spieler und KI-Firmen nutzen denselben Weg; später erzeugen auch
Manager (Stufe 2) nur Befehle. Schwierigkeit verändert die KI-Entscheider, nie die
Regeln.

**Vorgaben mit Vererbung (§5.6) – Gerüst ab Stufe 1.** Einstellungen wie
„KI-Händler dürfen kaufen, Mindestpreis X“ werden als `Vorgabe` an einer *Stelle*
(Konzern, später Kontinent/Land/Werk/Tochter) gespeichert. Die Abfrage
„welche Vorgabe gilt hier und woher stammt sie“ existiert von Anfang an; Stufe 1
nutzt nur Konzern- und Produktebene.

**Doppelte Buchführung ab Stufe 1 (§11.4, §14.2).** Jede Geldbewegung ist ein
Buchungssatz mit Konto, Kostenart (Material, Personal, Energie, Transport, Zölle,
Steuern, Marketing, Forschung, Zinsen) und Kostenstelle (Firma, Standort, Produkt,
Land). Bilanz, GuV, Kapitalfluss und später das ganze Controlling, Tochterfirmen
und konzerninterne Verrechnung sind dann Auswertungen – kein Umbau.

**Modifikatoren.** Wirkungen werden als datengetriebene Modifikatoren auf Größen
gelegt (z. B. „Nachfrage Konsumgüter in FRA × 0,7“, „Arbeitskräftepool −20 %“).
Stufe 1 nutzt sie für Spieleinstellungen; Ereignisse und Regulierungen (Stufe 4/6)
erzeugen später nur noch Modifikatoren.

**Determinismus (§16.1).**
- Ein Seed → je Teilsystem und Firma ein eigener Zufallsstrom (ChaCha8), abgeleitet
  aus Seed + Strom-ID. Neue Firmen verschieben dadurch nicht die Zufallszahlen anderer.
- Geld als Ganzzahl (`i64`, Hundertstel-Cent in USD Kaufkraft 2026). Mengen als `f64`
  mit festen Rechenreihenfolgen; Exponential-/Log-Funktionen über die
  plattformunabhängige `libm`.
- Keine `HashMap`-Iteration im Kern, keine Systemzeit, Parallelisierung nur mit
  deterministischem Zusammenführen.
- Pflichttest: Startzustand + Befehlsprotokoll zweimal abspielen → identischer
  Zustands-Hash.

**Skalierung (§16.4).** Daten spaltenweise in Vektoren (struct-of-arrays) statt
Objektgeflecht; Märkte je Land × Produkt aggregiert; Länder und Firmen
parallelisierbar (rayon) mit fester Zusammenführungsreihenfolge. Ein Benchmark mit
1 000 bzw. 5 000 KI-Firmen gehört zur Testsuite.

**Erklärungen statt nackter Zahlen (§14.3).** Wichtige Größen (Preis, Qualität,
Stückkosten, Nachfrage) können eine Aufschlüsselung ihrer Ursachen liefern; die
Oberfläche zeigt sie im Tooltip.

**Meldungen und Texte (§13.3, §16.4).** Der Kern erzeugt nie deutsche Sätze,
sondern `Meldung { art, schluessel, parameter, ziel }`. Alle Texte liegen zentral in
`data/texte/de/`; eine Übersetzung ist ein weiterer Ordner.

**Spielstände (§16.3).** Kopf mit Formatversion, Spielversion und Datenversion,
danach der Zustand (MessagePack, DEFLATE-komprimiert). Verweise auf Inhalte werden mit ihren
Text-IDs gespeichert, damit geänderte Daten ladbar bleiben. Jede Formatänderung
bringt eine Migrationsfunktion und einen Test mit einem alten Beispielstand mit.

**Protokolle (§16.4).** Je Runde optional Kennzahlen als CSV/JSONL (Preise, Mengen,
Firmenergebnisse) für Balance-Analysen; `wsim-cli run --seed 42 --bis 1930` erzeugt
sie ohne Oberfläche.

### 2.3 Datenformat (`data/`, geprüft von `wsim-data`)

```
data/
  meta.yaml              Datenversion, Einheiten, Grundannahmen
  parameter/             Formel- und Balancing-Parameter (keine Zahlen im Code)
  laender/DEU.yaml …     ein Land je Datei, Jahreswerte → Interpolation
  lagerstaetten/         je Rohstoff eine Datei
  ketten/                je Produktionskette eine Datei (Produkte, Rezepte,
                         Anlagen, Technologien der Kette zusammen)
  firmen/                Startbesetzung KI-Firmen
  texte/de/              alle Anzeigetexte
```

- Jede Datei enthält typisierte Abschnitte (`produkte:`, `rezepte:`, `anlagen:`,
  `technologien:`, `lagerstaetten:`). Der Lader sammelt alle Dateien und führt sie
  nach Typ zusammen. So kann eine neue Kette als *eine* Datei ergänzt werden.
- Verweise nur über lesbare IDs (`roheisen`, `DEU`); doppelte oder unbekannte IDs,
  Zyklen im Technologiebaum, fehlende Texte, unplausible Werte (negative Mengen,
  Anteile ≠ 100 %) werden beim Laden mit Datei, Zeile und Pfad gemeldet, mit
  Vorschlag bei Tippfehlern.
- Geschätzte Werte tragen `annaeherung: true`, recherchierte eine `quelle:`.
- Güter ohne eigene Kette (Glas, Zinn, Pferdekutsche …) bietet der **Staatsmarkt**
  in jedem Land zu einem Preis aus den Daten an (`staatsmarkt:` am Produkt).
- Feldnamen und IDs der Daten sind deutsch (ohne Umlaute), damit Inhalte ohne
  Programmierkenntnisse lesbar sind; der Programmcode verwendet englische Bezeichner.
  Ein Glossar (`docs/GLOSSAR.md`) ordnet beides zu.

### 2.4 Oberfläche (`ui/`)

- Ansichten nach §14.1; Stufe 1: Neues Spiel, Übersicht, Weltkarte, Länderdetail,
  Standorte/Produktion, Produktionsketten, Markt und Preise, Forschung, Finanzen,
  Rundenbericht, Speichern/Laden.
- Gemeinsame Bausteine: Tabelle (sortier-/filterbar, Spaltenauswahl), Zeitreihen-
  diagramm, Kennzahl-Kachel, Erklär-Tooltip, Navigationsziel (jede Meldung führt per
  Klick zur Detailansicht).
- Weltkarte: ECharts-Geo mit Natural-Earth-Daten; Ebenen als umschaltbare
  Choroplethen/Symbole (Stufe 1: Rohstoffe, Lohnniveau, eigene und fremde Standorte).
- Die Oberfläche fragt Sichten seitenweise beim Kern ab, hält keine Spiellogik und
  rechnet keine Spielwerte nach.
- Tastaturkürzel zentral registriert; Texte nur über Textschlüssel.

### 2.5 Was Stufe 1 schon für spätere Stufen vorsieht

| Spätere Funktion | Vorbereitet durch |
| --- | --- |
| Manager, Strategie-Ansicht (2) | Befehle als einziger Handlungsweg; Vorgaben mit Stellen-Hierarchie |
| Tochterfirmen, Controlling (2) | Doppelte Buchführung mit Kostenstellen, Firma als Konzernknoten |
| Logistik, Zölle (2) | Transport als eigenes System; Warengruppen und Länderpaare im Datenmodell |
| Börse, Übernahmen (3) | Eigentümer-Struktur je Firma (Anteile), Bilanzdaten |
| Ereignisse, Regulierung (4) | Modifikatoren, Ereignis-/Regulierungsdateien als eigener Datentyp |
| Währungen (4) | Alle Beträge intern in Leitwährung; Anzeige läuft über eine Umrechnungsschicht |
| Breite, Patente (5) | Ketten-Dateien, Technologiebaum mit Erfindungsjahr, Rezept-Alternativen |
| Zufallsereignisse, Töne (6) | Eigene Zufallsströme, Meldungsarten |

---

## 3 Beispiel: Kette 1 – Eisen und Stahl

Die vollständige Kette steht in `data/ketten/01_eisen_stahl.yaml`, die Lagerstätten in
`data/lagerstaetten/`, die Namen in `data/texte/de/`. Alle Felder beschreibt
`docs/DATENFORMAT.md`. Auszug:

```yaml
rezepte:
  # Zwei Rezepte für dasselbe Produkt: Fortschritt durch Technologie.
  - id: stahl_bessemer
    produkt: stahl
    menge: 1
    dauer_tage: 1
    anlage: stahlwerk_konverter
    technologie: bessemer_verfahren
    eingang:
      roheisen: 1.12
    arbeit_stunden:               # Personenstunden je Durchlauf
      ungelernt: 2.0
      angelernt: 1.5
      fachkraft.metall: 1.5
      akademiker.metall: 0.05
    qualitaet_basis: 45
    annaeherung: true

  - id: stahl_siemens_martin
    produkt: stahl
    eingang:
      roheisen: 1.10
      kohle: 0.25
    qualitaet_basis: 65
    # …

lagerstaetten:
  - id: lothringen_minette
    land: FRA                     # heutige Grenzen – 1900 zum großen Teil deutsch
    rohstoff: eisenerz
    vorrat: 2_500_000_000         # t, normiert auf 60 % Fe
    entdeckt: 1850
    erschliessung: {investition_usd: 80_000_000, dauer_tage: 540}
    foerderung_max_je_jahr: 25_000_000
    foerderkosten_faktor: 1.3
    annaeherung: true
```

---

## 4 Meilensteine für Stufe 1

**Stand:** M1 bis M4 abgeschlossen; Fortschritt in `docs/FORTSCHRITT.md`.

Jeder Meilenstein endet mit grünen automatischen Tests und einer kurzen Abnahme
durch dich. Formeln werden vor der Umsetzung in `docs/FORMELN.md` beschrieben und
mit dem Meilenstein freigegeben.

| Nr. | Meilenstein | Inhalt | Prüfbar durch |
| --- | --- | --- | --- |
| M1 | Projektgerüst | Cargo-Workspace, Tauri-Gerüst, leere UI, CI (Linux-Tests, Windows-Installer als Artefakt), Formatierung/Linter | CI grün; Installer installiert und startet ein leeres Fenster |
| M2 | Datenformat und Prüfung | Schemas für Länder, Produkte, Rezepte, Anlagen, Technologien, Lagerstätten, Texte; Lader mit verständlichen Fehlern; Kette 1 als Daten; `wsim validate` | Tests je Fehlerart mit erwarteter Meldung; Kette 1 lädt fehlerfrei |
| M3 | Kern-Gerüst | Katalog/Zustand, Kalender, Tages-Ticks, Rundenlängen, Befehle, Zufallsströme, Fortschritt, Speichern/Laden mit Version; `wsim run` | Determinismus-Test (gleicher Hash), Speichern→Laden→Weiterrechnen identisch |
| M4 | Länder-Grundwerte | Alle Länder in heutigen Grenzen 1900–1930: Bevölkerung, Pro-Kopf-Einkommen, Einkommensfünftel, Arbeitskräfte und Löhne je Qualifikation, Energiepreis, Steuersatz, Infrastruktur, Stabilität, Entfernungen | Vollständigkeits- und Plausibilitätstests; Interpolation |
| M5 | Rohstoffe und Produktion | Lagerstätten, Erschließung, Abbau, Erschöpfung; Standorte (Mine, Werk, Lager); Rezepte, Arbeitskräftepool, Energiebezug, Qualität, Lagerbestände | Szenario-Tests mit exakt berechnetem Ergebnis |
| M6 | Buchführung und Grundfinanzen | Doppelte Buchführung, Kostenarten/-stellen, GuV, Bilanz, Kapitalfluss, Bankkredit, Gewinnsteuer, Zahlungsunfähigkeit | Bilanz immer ausgeglichen (Eigenschaftstest); Beispielrechnungen |
| M7 | Markt und Preise (ein Land) | Endkunden-Nachfrage aus Einkommensfünfteln (Verbrauchs-/Gebrauchsgüter, Besitzquote, Kaufkraftgrenze), Industrie- und Staatsnachfrage, Anbieterwahl, Preisbildung | Preis konvergiert; Preissenkung erschließt neue Schichten |
| M8 | Handel zwischen Ländern | Entfernungen, epochengerechte Transportkosten und -dauer (vereinfacht), KI-Händler, Verkaufswege je Produkt (Mindestpreis, Höchstmenge) | Preisunterschiede ≤ Transportkosten; Verkaufssperre wirkt |
| M9 | Alle 12 Ketten + Forschung | Daten der Ketten 2–12; Technologiebaum Stufe 1; Forschungsprojekte, Kostenaufschlag für Vorgriffe, Nachzügler-Rabatt; Verdrängung (Glühlampe, Automobil) | Jede Kette im Kopflos-Szenario vom Rohstoff bis zum Endprodukt; Verdrängungskurve |
| M10 | KI-Firmen | Startbesetzung, Grundverhalten (produzieren, Preise, investieren, forschen, expandieren, pleitegehen) über dieselben Befehle; Leistungstest | Welt läuft 1900–1930 ohne Spieler mit plausiblen Größen; Benchmark 1 000 Firmen |
| M11 | Rundenbericht und Meldungen | Finanzergebnis und Vergleich, Weltereignisse, Wettbewerberaktionen, Warnungen, Forschung; Autospeichern, manuelle Spielstände | Kern-Tests; Bericht aus Kopflos-Lauf |
| M12 | Oberfläche I | Hauptmenü, Neues Spiel (Einstellungen §15, soweit Stufe 1), Übersicht, Runde beenden mit Länge und Fortschritt, Rundenbericht mit Sprungzielen, Speichern/Laden | Playwright-Tests gegen Web-Build |
| M13 | Oberfläche II: Weltkarte | Karte, Länderdetail, Ebenen Rohstoffe/Lohnniveau/Standorte | Playwright-Tests, Sichtprüfung |
| M14 | Oberfläche III: Spielen | Standorte bauen, Produktion, Produktionsketten mit Engpässen, Markt und Preise, Forschung, Finanzen; Tooltips mit Ursachen; Tastaturkürzel | Playwright-Tests; durchgespielte Partie 1900–1905 |
| M15 | Spielbarkeit Stufe 1 | Balancing 1900–1930, Windows-Installer offline, Protokoll-Auswertung | Testpartie 1900–1930 ohne Abbruch; Abnahme durch dich |

M12–M14 können nach M11 auch verzahnt mit M9/M10 laufen, wenn du früh etwas sehen
möchtest.

---

## 5 Unklare Stellen im Lastenheft

Siehe `docs/OFFENE_PUNKTE.md` (22 Punkte mit Vorschlag). Nach deiner Entscheidung werden
die Punkte in §18 des Lastenhefts übernommen.
