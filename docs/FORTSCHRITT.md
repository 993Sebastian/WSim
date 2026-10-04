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
| M10 | KI-Firmen | ✅ |
| M11 | Rundenbericht und Meldungen | ✅ |
| M12 | Oberfläche I: Rundenablauf | ✅ (vor M11 gezogen) |
| M13 | Oberfläche II: Weltkarte | ✅ |
| M14 | Oberfläche III: Spielen | ✅ |
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
  Staatsmarkt. Handelsfirmen als echte KI-Firmen sind nach M10 verschoben (siehe M10).
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
- M10: Standard 100 KI-Firmen (Auftraggeber); Märkte im Maßstab Firmen/1000, also 0,1.
  Preise, Löhne und Anlagengrößen bleiben real, damit Zahlen und Bilanzen vertraut
  wirken. Höchstens 10 000 Firmen.
- M10: Schwierigkeiten leicht/mittel/schwer setzen Kompetenz 0,25/0,5/0,8 und
  Aggressivität 0,3/0,5/0,7; jede Firma streut ± 0,15.
- M10: 29 reale Firmen mit ihrem Stand 1900 (Krupp, Thyssen, Carnegie Steel,
  Standard Oil, Branobel, Siemens & Halske, AEG, General Electric, Singer, Dunlop,
  Michelin, Mitsui, Tata …); Anlagenzahlen grob geschätzt (`annaeherung: true`).
  Höchstens die Hälfte aller KI-Firmen sind reale. Sie werden auch bei späterem
  Startjahr mit dem Stand 1900 eingesetzt.
- M10: Firmennamen aus acht Sprachräumen (Familienname, Ort, Rechtsform, Branche);
  Namen realer Firmen werden nie erzeugt.
- M10: Weitere Kohle- (12) und Eisenerzlagerstätten (9) sowie das Forschungslabor
  (500 000 USD, 20 Forscherplätze) für die KI-Forschung ergänzt.
- M10: KI-Entscheidungen stehen nicht im Journal; sie folgen beim Wiederholen aus dem
  Zustand (Test `ai_world_is_reproducible`).
- M10: Firmen können nicht bei sich selbst kaufen; die KI lagert eigene Vorprodukte
  per Warentransfer um.
- M10: Gefundene und behobene Marktfehler: Angebote ohne Lager erhöhten täglich ihren
  Preis (Preise bis 10¹¹ USD); jetzt nur bei ausverkaufter Ware und höchstens
  20 × Richtpreis (`marktmodell.preisanpassung.hoechstfaktor`). Händler sehen nun die
  Gebote der Einkaufsaufträge.
- M10: Weltlauf 1900–1930 mit 100 Firmen (Seed 1): Preise bleiben in der Nähe der
  Richtpreise, alle Ketten liefern, rund 2–3 % der Firmen pro Jahr gehen pleite und
  werden durch Gründungen an Engpässen ersetzt. Die Welt reagiert empfindlich auf
  Regeländerungen; das Feintuning (Nachfrageniveau Textil, Überkapazität bei Erz) ist
  Aufgabe von M15.
- M10: Leistung – Startbesetzung 0,2 s; ein Jahr mit 100 Firmen rund 4 s, mit
  1000 Firmen rund 6 s (Release, eine Kern-CPU).
- M10: CLI `run --ki <Anzahl> --schwierigkeit <id> --welt` für Weltläufe mit Bericht.
- M10 vereinfacht (offene Punkte 23–25): keine Handelsfirmen als KI, KI-Einkäufe
  ohne Kassenprüfung, kein Ersatz durch höher qualifizierte Arbeitskräfte.
- M12 vor M11 gezogen, damit der Auftraggeber früh ein spielbares Programm sieht
  (Absprache 04.10.2026). Der Rundenbericht zeigt vorerst Zeitraum, Kasse vorher/nachher,
  Veränderung des Eigenkapitals und die Meldungen des Kerns; M11 erweitert ihn.
- M12: Sichten (`wsim_core::views`) liefern der Oberfläche nur Schlüssel und fertige
  Zahlen (USD, ISO-Datum); die Oberfläche formatiert nur. Die Spielsitzung (neues
  Spiel, Runden, Spielstände) liegt in der Bibliothek `wsim-session`, damit der
  Tauri-Adapter dünn bleibt und alles ohne Tauri testbar ist.
- M12: Die Browser-Vorschau (Entwicklung, Playwright) antwortet mit Beispielsichten
  eines echten Spiels (`wsim beispielsichten ui/src/kern/beispiel.json`); jede Runde
  liefert denselben Bericht. Spielstände liegen im lokalen App-Datenordner
  (`spielstaende/*.wsim`), Namen bis 60 Zeichen.
- M12: Startjahr im Dialog 1900–1930 (offener Punkt 7); Zufallswert wird vorgeschlagen
  und ist änderbar. Der Marktmaßstab wird nicht in der Oberfläche vorausberechnet.
- M12: Sprungziele der Meldungen führen vorerst zur Übersicht; weitere Ansichten folgen
  mit M13/M14.
- M11: 42 historische Ereignisse 1900–1939 (`data/ereignisse/`) erscheinen am
  Ereignistag als Weltereignis; die Oberfläche zeigt sie nach der Runde als eigene
  Fenster vor dem Bericht (Wunsch des Auftraggebers: Popups bei Kriegen und neuen
  Ländern). In Stufe 1 ohne eigene Wirkungen: Die Folgen stecken in den Länderwerten
  (Lastenheft §18), Wirkungen folgen mit Stufe 4.
- M11: Rundenbericht mit Finanzergebnis der Runde (aus dem Hauptbuch, auch über den
  Jahreswechsel) und Vergleich mit der Vorrunde, Abschnitten Weltgeschehen, Warnungen,
  Wettbewerb und Forschung (Fortschritt je Projekt).
- M11: Wettbewerbermeldungen: Gründungen, Pleiten, erste Erfindungen und Ausbauten in
  Produkten, die der Spieler herstellt oder anbietet (nicht jeder Ausbau aller
  100 Firmen). Warnung, wenn Anlagen wegen fehlender Vorprodukte stillstehen.
- M11: Nach jeder Runde automatische Speicherung unter „Automatisch“ (ein Stand, wird
  überschrieben); manuelle Spielstände beliebig. Ein Meldungsarchiv folgt mit M14.
- M11: Die Browser-Vorschau zeigt jetzt einen Start 1914 mit dem Juli-Bericht
  (Kriegsbeginn), damit Popups und Vergleich sichtbar sind.
- M13: Weltkarte als eigenes SVG statt ECharts-Geo (Architektur §2.4): keine zusätzliche
  Bibliothek, volle Kontrolle über Farben in hellem und dunklem Design. Die Umrisse
  erzeugt `python3 tools/daten/karte.py` aus Natural Earth 1:50 Mio. (gleichabstandige
  Projektion, vereinfacht, 151 KB); 21 Kleinstaaten ohne Fläche erscheinen als Punkt
  an der Hauptstadt.
- M13: Ebenen Lohnniveau (Stundenlohn Ungelernter), BIP je Kopf, Bevölkerung (je fünf
  Klassen nach Quantilen), Rohstoffe (Lagerstätten je Rohstoff an der Hauptstadt, Größe
  nach Jahresförderung, Farbe nach freien Konzessionen) und Standorte (eigene und
  fremde). Ein Klick oder Enter auf ein Land öffnet das Länderdetail: Grundwerte,
  Arbeitskräfte und Löhne je Gruppe, Lagerstätten mit Konzessionen, Firmen im Land und
  Märkte des Vormonats.
- M13: Routen auf der Karte (Lastenheft §14.1) folgen mit der Logistik in Stufe 2; in
  Stufe 1 zeigt `wsim route` die Wege.
- M14: Sechs Spielansichten neben Übersicht und Weltkarte: Produktion, Markt,
  Forschung, Finanzen, Berichte; Wechsel mit den Zifferntasten 1–7. Alle Eingaben werden
  als `Befehl` (JSON) über die Sitzung an `Game::apply` geschickt und laufen durch
  dieselbe Prüfung wie bei KI-Firmen; Ablehnungen erscheinen als Textschlüssel.
- M14: „Produktionsketten“ (Lastenheft §14.1) zeigt Stufe 1 je Standort als Tabelle statt
  als Grafik: Anlagen mit Rezept, Auslastung, Ist-/Planmenge und Engpass, darunter die
  Vorprodukte mit Reichweite in Tagen (rot unter 1, gelb unter 7) und Einkauf, dann
  Verkauf und Lager. Eine grafische Kettenansicht lohnt erst mit mehr Standorten und
  Logistik (Stufe 2).
- M14: Engpass-Ursachen kommen aus dem Kern: Die Produktion merkt sich je Anlage, was
  am letzten Tag begrenzt hat (`Slot::limit`: Vorprodukt, Arbeitskräftegruppe, Strom,
  Lagerstätte); dazu im Bau, ohne Rezept, stillgelegt. Der Tooltip erklärt, was hilft.
- M14: Markttabelle sortierbar (Klick auf den Spaltenkopf) und auf eigene Produkte
  filterbar; Technologien nach Fachgebiet filterbar (Lastenheft §14.3).
- M14: Tastaturkürzel an einer Stelle (`Spiel.tsx`) und in der Hilfe (`?` oder F1):
  Strg+Enter Runde beenden, 1–7 Ansichten, Strg+S Speichern, Strg+O Laden, Esc schließt.
- M14: Das Meldungsarchiv („Berichte“) hält die letzten 120 Rundenberichte der laufenden
  Sitzung im Speicher der Oberfläche; es wird nicht gespeichert. Ein gespeichertes Archiv
  bräuchte die Meldungen im Spielstand und folgt bei Bedarf.
- M14: Sprungziele der Meldungen: fehlende Vorprodukte → Produktion, Forschung →
  Forschung, Geld (Warnung, Krise, Erfolg) → Finanzen.
- M14: Fehler behoben: Startanlagen (Werkstatt des Spielers, Startbesetzung der KI)
  wurden doppelt aktiviert – direkt als Sachanlage und am ersten Tag noch einmal aus
  „Anlagen im Bau“. Die Summe stimmte, „Anlagen im Bau“ war aber negativ. Jetzt laufen
  Anlagen und Lagerstätten über „Anlagen im Bau“ und werden am ersten Tag fertig.
- M14: Die durchgespielte Partie 1900–1905 läuft als Test der Sitzung
  (`crates/wsim-session/tests/partie.rs`): Bauen, Produktion, Einkauf, Verkauf, Kredit,
  60 Monatsrunden, Wiederholung aus dem Journal ergibt denselben Zustand.
- M14 (Review): Ein unabhängiges Review des M14-Diffs (vier Blickwinkel, jeder Befund
  gegengeprüft) fand 14 Fehler, alle behoben:
  - Forschungslabore liefen nach dem Bau mit 0 % Auslastung, auch bei KI-Firmen – es
    wurde nie geforscht. Labore arbeiten jetzt ab Fertigstellung voll; die Auslastung
    lässt sich in der Forschungsansicht ändern, weitere Labore lassen sich bauen.
  - Lagerstätten ließen sich in der Oberfläche nicht erschließen. Förderstätten ohne
    Lagerstätte zeigen jetzt die entdeckten Lagerstätten des Landes mit freier
    Konzession (Kosten, Dauer, Jahresförderung); während der Erschließung lautet die
    Ursache „Lagerstätte in Erschließung“.
  - Absatz und Einkauf zeigten nach Monatsrunden immer 0 (der laufende Zähler wird am
    Monatsersten zurückgesetzt). Angebote und Einkäufe merken sich jetzt den Vormonat
    (`sold_last_month`, `bought_last_month`, alte Spielstände laden mit 0).
  - Rundungsfehler galten als Engpass („Zu wenig Angelernte“ bei voller Leistung);
    Ursachen zählen erst ab einer echten Unterschreitung des Plans.
  - Die Erfolgsrechnung ordnete die Spalten nach Position statt nach Kostenart.
  - Einkauf und Verkauf jedes Produkts an jedem Standort (z. B. Handelsniederlassung
    der Startform „Handel“), nicht nur der Vorprodukte und eigenen Erzeugnisse.
  - Zahlen statt Schlüssel in Befehlen werden gegen den Katalog geprüft (vorher Absturz).
  - Nach „Laden“ beginnen alle Ansichten neu; das Berichtsarchiv wird geleert.
  - Kleinere Formularfehler (Vorgabewerte, Sondertilgung je Kredit, Laborbau im Bau).
- M15: Produktbäume nach Lastenheft §17.2 (Vorgabe des Auftraggebers): höchstens vier
  Vorprodukte je Rezept, höchstens sechs Ebenen, mehr als vier nur mit `sehr_komplex`.
  Die Datenprüfung erzwingt das und zeigt die Verteilung der Endprodukte nach Ebenen.
  Umbau dafür: Hochofen und Stahlwerk sind ein Hüttenwerk (Roheisen entfällt, Stahl aus
  Erz und Kohle); Weißblech, Schrauben, Muttern, Motor und Fahrgestell aus Stahl; Reifen
  aus Gummi, Garn und Stahl; Investitionsgüter (Nähmaschine in der Konfektion,
  Elektromotor in der Weberei) sind keine Vorprodukte mehr; Fahrrad, Automobil,
  Elektromotor, Karosserie und Möbel mit höchstens vier Vorprodukten. Ergebnis: acht
  Endprodukte mit vier Ebenen, Handwerkzeug mit drei.
- M15: Vorbereitung auf Investor, Bank und Tochterfirmen (Lastenheft §17.3): Jede Firma
  hat Eigentümer mit Anteilen (`Holder`: Spieler, andere Firma, Privatbesitz). Die
  Spielerfirma gehört zu 100 % dem Spieler, KI-Firmen sind in Privatbesitz; ältere
  Spielstände bekommen das beim Laden. Das Hauptbuch nimmt neue Konten beim Laden mit
  Saldo 0 auf, sodass Beteiligungen und vergebene Kredite (Stufe 3) kein neues
  Spielstandformat brauchen. Befehle bleiben je Firma; der Spieler steuert die Firmen,
  an denen er die Mehrheit hält.
