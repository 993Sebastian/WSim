# Fortschritt Stufe 1

Auftrag vom 03.10.2026: alle Meilensteine M4–M15 nach den eigenen Vorschlägen
selbstständig umsetzen; Review durch den Auftraggeber danach. Diese Datei ist der
Einstiegspunkt beim Weiterarbeiten nach einer Unterbrechung.

## Stand

| Nr. | Meilenstein | Status |
| --- | --- | --- |
| M1 | Projektgerüst | ✅ |
| M2 | Datenformat und Prüfung | ✅ |
| M3 | Kern-Gerüst | ✅ |
| M4 | Länder-Grundwerte | ✅ |
| M5 | Rohstoffe und Produktion | in Arbeit |
| M6 | Buchführung und Grundfinanzen | offen |
| M7 | Markt und Preise (ein Land) | offen |
| M8 | Handel zwischen Ländern | offen |
| M9 | Alle 12 Ketten + Forschung | offen |
| M10 | KI-Firmen | offen |
| M11 | Rundenbericht und Meldungen | offen |
| M12 | Oberfläche I: Rundenablauf | offen |
| M13 | Oberfläche II: Weltkarte | offen |
| M14 | Oberfläche III: Spielen | offen |
| M15 | Spielbarkeit Stufe 1 | offen |

## Eigenständige Entscheidungen (für das Review)

Entscheidungen, die ohne Rückfrage getroffen wurden. Alle sind änderbar.

- M3: Monats- und Quartalsrunden enden an Kalendergrenzen; Jahreswerte gelten zur
  Jahresmitte (docs/FORMELN.md).
- Vorschläge aus docs/OFFENE_PUNKTE.md mit 🟡 werden wie vorgeschlagen umgesetzt.
- Installer bleibt bei rund 217 MB mit WebView2-Offline-Installer (Lastenheft: offline).
- M4: Länderdaten aus Gapminder Fast Track (Bevölkerung, BIP je Kopf, Gini) und Natural
  Earth (Fläche, Nachbarn, Hauptstadt, deutsche Namen), erzeugt mit
  `tools/daten/laender.py`. 197 Länder: UN-Mitglieder ohne Vatikan, plus Taiwan,
  Palästina, Kosovo, Westsahara; Hongkong zu China. Korrekturen auf heutige Grenzen für
  Deutschland (bis 1949) und Irland/Vereinigtes Königreich (bis 1921).
  Abhängige Gebiete (z. B. Puerto Rico, Grönland) zählen flächenmäßig zum Mutterland;
  ihre Bevölkerung fehlt in den Quellen.
- M4: Löhne, Arbeitskräfte, Preisniveau, Strom, Infrastruktur, Forschung und
  Automatisierung werden aus BIP je Kopf und Parametern berechnet (Formeln in
  docs/FORMELN.md); Steuern, Stabilität und Länderprägungen für wichtige Länder sind
  eigene Schätzungen im Erzeugungsskript.
- M4: Länderwerte werden monatlich statt täglich neu berechnet (Leistung) und nicht mehr
  gespeichert; Spielstand-Format 2 mit Umwandlung von Format 1.
- Wunsch vom 03.10.: Pop-ups bei wirtschaftlich wichtigen Ereignissen (Kriege,
  Staatsgründungen, Krisen). Umsetzung: Ereignisdatei mit historischen Ereignissen
  (M11, Meldungsart „Weltereignis“), Anzeige als Pop-up in der Oberfläche (M12). Die
  wirtschaftlichen Folgen stecken bis Stufe 4 bereits in den realen Länderwerten.
