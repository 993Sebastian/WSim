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
| M5 | Rohstoffe und Produktion | ✅ |
| M6 | Buchführung und Grundfinanzen | ✅ |
| M7 | Markt und Preise (ein Land) | ✅ |
| M8 | Handel zwischen Ländern | ✅ |
| M9 | Alle 12 Ketten + Forschung | ✅ |
| M10 | KI-Firmen | in Arbeit |
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
- M5: Doppelte Buchführung (Konten, Kostenarten nach §14.2, Kostenstellen Standort/
  Produkt, Monats- und Jahresergebnisse) schon in M5 statt M6, weil die Produktion
  Geldbewegungen erzeugt. Bewertung nach dem Gesamtkostenverfahren.
- M5: Spielstände schreiben alle Verweise auf Inhalte als IDs (`roheisen`), damit sie
  nach Datenänderungen ladbar bleiben; ältere Formate werden typisiert eingelesen
  (Format 3, Umwandlung von 1 und 2).
- M5: Umlagern von Waren nur innerhalb eines Landes und ohne Kosten; Transporte
  zwischen Ländern kommen mit M8.
- M5: Strombedarf wird höchstens bis zum Netzanteil des Landes gedeckt (Eigenstrom ab
  Kette 7 in M9). Walzen und Drahtziehen laufen um 1900 mit Dampf (Kohle als Eingang),
  daher ohne Strom.
- M5: Personal wird monatlich und nach Änderungen automatisch auf den Bedarf eingestellt
  (Lastenheft §5.4 „automatisch nachbesetzen“); ältere Standorte zuerst. Vertretung
  durch höhere Qualifikationen und Abwerbung folgen mit den KI-Firmen.
- M5: Startform (Werkstatt/Handel) wirkt erst mit Kette 2 (M9): dann erhält der
  Spieler eine kleine Werkstatt bzw. Niederlassung.
- M6: Zinsen sind reale Zinsen (das Spiel hat keine Inflation); Realzins-Verlauf
  1900–2026 grob angenähert. Kredite nur gegen Sicherheiten (60 % der Sachwerte),
  dazu eine Kreditlinie von 10 % der Bilanzsumme. Gewinnsteuer jährlich mit
  unbegrenztem Verlustvortrag.
- M6: Zahlungsunfähigkeit wird monatlich geprüft; der Spieler verliert nur, wenn auch
  ein neuer Kredit die Lücke nicht schließen könnte (§11.3). Anleihen und Anteile
  folgen in Stufe 3.
- M7: Die Kaufschwelle ist ein Verhältnis Einkommen/Preis statt eines Dollarbetrags
  (Lastenheft §9.1 „Einkommen im Verhältnis zum Preis“); jedes Produkt hat einen
  Richtpreis um 1900. Endkunden-Nachfrage wird monatlich berechnet, Märkte täglich
  geräumt (Reihenfolge: Industrie, Staat, Endkunden). Verkauf an Endkunden ist von jedem
  Standort im Land möglich; Niederlassungen werden mit dem Handel zwischen Ländern wichtig.
- M7: Automatische Preise passen sich täglich um +2 %/−1 % an (Parameter); Premium-,
  Kampfpreis- und Margenstrategien folgen mit der Strategieansicht (Stufe 2).
- M8: Frachtdienst zwischen Hauptstädten (Land-, See-, Luftwege; Dijkstra je Jahr und
  Abgangsland). Seewege nach Luftlinie × Umwegfaktor 1,4 – echte Seewege und Kanäle
  (Panama ab 1914, Suez) folgen mit der Logistik in Stufe 2. Binnenschifffahrt fehlt
  noch. Standorte haben innerhalb eines Landes keine Lage: Transfers im Land sind
  sofort und kostenlos.
- M8: Verkehrsmittel-Kosten aus Frachtraten um 1900 mit dem Verbraucherpreisindex auf
  2026 umgerechnet (Eisenbahn 0,17 USD/tkm, Dampfschiff 0,012, Fuhrwerk 3,0); Kohle
  Großbritannien → USA kostet so etwa 112 USD/t und 26 Tage.
- M8: KI-Händler sind in Stufe 1 ein wettbewerblicher Händlermarkt ohne eigene Bilanz
  (ihre Spanne verlässt das Spiel). Sie kaufen nur aus Firmenangeboten, nicht vom
  Staatsmarkt. Handelsfirmen als echte KI-Firmen folgen mit M10.
- M8: Verkaufsfreigaben je Abnehmergruppe mit vier Geltungsbereichen (pauschal, Land,
  Produkt, Produkt im Land); die Höchstmenge gilt je Angebot (Standort und Produkt)
  und Monat.
- M9: Preise der neuen Ketten sind Preise um 1900 × 38 (Verbraucherpreisindex bis 2026).
  Kautschuk ist dadurch um 1900 sehr teuer (Wildkautschuk); Plantagen in Malaya, auf
  Sumatra und Ceylon werden ab 1905/1910 nutzbar (lange Erschließung) und senken den
  Preis, wenn KI-Firmen sie erschließen.
- M9: Güter ohne eigene Kette vom Staatsmarkt: Leim, Zinn, Konserveninhalt, Glas,
  Schwefel, Pferdekutsche. Benzin entsteht um 1900 als Nebenprodukt der
  Petroleum-Raffinerie; das Cracken (1913) kehrt das Verhältnis um.
- M9: Forschung als Punkte je Forschertag; Aufwand ab 1900 erfundener Technologien in
  den Daten (z. B. Fließband 30 000 Punkte). Vorgriff × 1,25 je Jahr, Nachzügler −10 %
  je Jahr bis 20 %, Gemeingut nach 25 Jahren (Patente erst Stufe 5).
- M9: Eigenstrom kann nicht gelagert werden; Überschuss geht zu 50 % des
  Industriestrompreises ins Netz. Kraftwerke versorgen nur Standorte derselben Firma
  im selben Land.
- M9: Startform Werkstatt = kleines Werk (20 000 USD) mit Nagelmaschine (40 000 USD),
  Einkaufsauftrag für Draht und Verkaufsangebot für Nägel; Startform Handel =
  Niederlassung (15 000 USD). Beides wird vom Startkapital bezahlt.
- M9: Ergänzungsgüter (Benzin je Auto, Petroleum je Petroleumlampe, Ersatzreifen) und
  netzabhängige Nachfrage (Glühlampe) ergänzen das Nachfragemodell; so sinkt der
  Petroleumbedarf mit der Verdrängung der Lampe.
- M9: Herstellkosten im Kopflos-Szenario (Test `ketten.rs`) liegen bei Rohstoffen aus
  eigener Förderung weit unter den Richtpreisen (z. B. Gummi); das ist Stoff für das
  Balancing in M15.
- M9: Leistung – Märkte ohne Handel werden gesammelt gebucht; 30 Jahre ohne Spieler
  laufen in unter 1 s (vorher 3,5 s mit allen Ketten).
