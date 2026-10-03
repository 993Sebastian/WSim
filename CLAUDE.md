# WSim – Projektregeln

Rundenbasierte Wirtschaftssimulation 1900–2100 für Windows (Einzelspieler, Deutsch).
Anforderungen: `docs/LASTENHEFT.md`. Architektur: `docs/ARCHITEKTUR.md`.
Offene Fragen: `docs/OFFENE_PUNKTE.md`.

## Umfang und Arbeitsweise

- Umgesetzt wird **nur Ausbaustufe 1** (Lastenheft §17). Keine Abkürzung, die eine
  spätere Stufe verbaut (siehe Tabelle „Was Stufe 1 schon vorsieht“ in der Architektur).
- Gearbeitet wird in **Meilensteinen** (Architektur §4). Vor jedem neuen Meilenstein
  auf Freigabe des Auftraggebers warten.
- Formeln vor der Umsetzung in `docs/FORMELN.md` beschreiben; sie werden mit dem
  Meilenstein abgenommen.
- Unklarheiten nicht stillschweigend entscheiden: in `docs/OFFENE_PUNKTE.md`
  eintragen (mit Vorschlag) und nachfragen, wenn sie das Ergebnis ändern.

## Aufbau

- `crates/wsim-core` – Simulationskern. Keine Abhängigkeit zu UI, Tauri oder Datei-IO.
- `crates/wsim-data` – Laden und Prüfen der Datendateien, erzeugt den Katalog.
- `crates/wsim-cli` – Läufe ohne Oberfläche, `validate`, Balance-Protokolle;
  `land <ISO>` und `route <von> <nach>` zeigen Länderwerte und Transportwege.
- `app/src-tauri` – dünner Adapter zwischen Kern und Oberfläche.
- `ui/` – TypeScript + React. **Keine Spiellogik**, keine nachgerechneten Spielwerte.
- `data/` – alle Spielinhalte (YAML) und alle Texte (`data/texte/de/`).

## Kernregeln der Simulation

- **Determinismus:** gleicher Seed + gleiche Befehle = identischer Zustand.
  - Zufall nur über die Zufallsströme des Kerns (je Teilsystem/Firma), nie `rand::thread_rng`.
  - Keine Iteration über `HashMap`/`HashSet` im Kern; `Vec`, `BTreeMap` oder `IndexMap`.
  - Keine Systemzeit, keine von Thread-Reihenfolge abhängigen Ergebnisse.
  - `exp`, `ln`, `pow` usw. über `libm`.
- **Geld** ist `i64` (Hundertstel-Cent, USD Kaufkraft 2026), nie Gleitkomma.
- **Jede Geldbewegung** ist ein Buchungssatz (Konto, Kostenart, Kostenstelle).
- **Jede Handlung** von Spieler und KI ist ein `Befehl` und läuft durch dieselbe
  Prüfung. KI-Firmen bekommen keine Sonderregeln; Schwierigkeit ändert nur ihr Verhalten.
- **Keine Spielzahlen im Code:** Parameter und Inhalte gehören in `data/`.
- **Keine Anzeigetexte im Kern:** Meldungen als Textschlüssel + Parameter.

## Daten

- Datenfelder und IDs deutsch, `snake_case`, ohne Umlaute (`foerderung`, `roheisen`).
  Länder-IDs sind ISO-3166-alpha-3 (`DEU`, `SWE`).
- Code-Bezeichner englisch; Zuordnung in `docs/GLOSSAR.md` pflegen.
- Geschätzte Werte mit `annaeherung: true` kennzeichnen, recherchierte mit `quelle:`.
- Format aller Felder: `docs/DATENFORMAT.md` – bei jeder Schemaänderung mitpflegen.
- Jeder neue Datentyp braucht Prüfregeln mit verständlicher deutscher Fehlermeldung
  (Datei, Zeile, Pfad) und einen Test in `crates/wsim-data/tests/fehlerfaelle.rs`.
  Alle Prüfmeldungen stehen zentral in `crates/wsim-data/src/messages.rs`.
- Rohschema (`raw.rs`, deutsche Feldnamen per `serde(rename)`) und Katalog im Kern
  (`wsim_core::catalog`, englisch) sind getrennt; nur `wsim-data` übersetzt.
- Länderdaten gelten in **heutigen Grenzen** für den ganzen Zeitraum.
- `data/laender/*.yaml` und `data/texte/de/laender.yaml` erzeugt
  `python3 tools/daten/laender.py` aus Gapminder und Natural Earth (Quellen werden nach
  `tools/daten/.cache/` geladen). Korrekturen und Schätzungen gehören in das Skript.
- Fortschritt und eigenständige Entscheidungen: `docs/FORTSCHRITT.md`.

## Spielstände

- Formatversion `SAVE_FORMAT_VERSION` in `crates/wsim-core/src/save/mod.rs`. Jede
  inkompatible Änderung an `GameState`: Version erhöhen, das alte Format in
  `save/legacy.rs` typisiert einlesen und umwandeln, neuen Beispielstand mit dem
  ignorierten Test `write_fixture` erzeugen und in `FIXTURES` eintragen. Alte
  Beispielstände unter `crates/wsim-core/tests/fixtures/saves/` bleiben liegen.
- Verweise auf Inhalte (IDs) werden beim Speichern automatisch als Schlüssel
  geschrieben; Zustand je Inhalt gehört in `PerId<Id, T>` (wird als Zuordnung
  Schlüssel → Wert gespeichert). Neue Einträge nach dem Laden: `fit_to_catalog`.
- Abgeleitete Werte (z. B. Länderwerte) nicht speichern (`#[serde(skip)]`), sondern
  nach dem Laden neu berechnen.
- Alles, was die Simulation verändert, läuft über `Game::apply`/`advance` und steht
  damit im Journal; `Game::replay` muss denselben Zustand ergeben.
- Jede Buchung hält die Bilanz ausgeglichen (`Ledger::is_balanced`); Tests prüfen das.

## Prüfen vor jedem Commit

- Rust: `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`
  (deckt `crates/*` ab; die Tauri-Hülle braucht zusätzlich `pnpm -C ui build` und
  `cargo clippy -p wsim-app --all-targets -- -D warnings`)
- Daten: `cargo run -p wsim-cli -- validate data` (muss ohne Fehler und Warnungen enden)
- UI: `pnpm -C ui typecheck`, `pnpm -C ui lint`, `pnpm -C ui test`, `pnpm -C ui e2e`
  (`pnpm -C ui format` behebt Formatierungsfehler)
- Die Reproduzierbarkeits-Tests (`crates/wsim-core/src/determinism_tests.rs`,
  `crates/wsim-cli/tests/lauf.rs`) müssen grün sein.
- Neue Kernlogik kommt mit Tests (Szenario- oder Eigenschaftstest).
- CI (`.github/workflows/ci.yml`) führt dasselbe aus und baut den Windows-Installer.

## Umgebung

- Rust-Version ist in `rust-toolchain.toml` fest eingestellt (gleiche Gleitkomma-
  Ergebnisse); nur bewusst und mit Determinismus-Test anheben.
- Tauri unter Linux braucht `libwebkit2gtk-4.1-dev` und `libgtk-3-dev`. Die App lässt
  sich hier mit `pnpm -C app tauri build --debug --no-bundle` bauen und unter
  `xvfb-run` starten.
- Playwright ist auf 1.56.1 festgelegt, passend zum vorinstallierten Chromium.

## Sprache

- Kommunikation, Dokumentation, Commit-Nachrichten und Spieltexte auf Deutsch.
- Code-Kommentare auf Englisch, knapp, nur wo das Warum nicht offensichtlich ist.
