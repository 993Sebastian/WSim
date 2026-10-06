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
| M15 | Spielbarkeit Stufe 1 | ✅ |
| M16 | Markteintritt und Marketing | ✅ |
| – | Nacharbeit zu M16: Plausibilität und Versorgung | ✅ |
| M17 | Browser-Version (WebAssembly, GitHub Pages) | ✅ |
| M18 | Bedienung: Personal, Werksansicht, Preise, alle Ansichten überarbeitet | ✅ |
| M19 | Technologiebaum: Technologien, Verfahren, Produkte und Anlagen | ✅ |
| M20 | Geführte Einführung bis zum ersten Verkauf | ✅ |
| M21 | Landeswährungen: Kaufkraft 2026 oder Preise der Zeit | ✅ |
| M22 | Anlagen stilllegen und verkaufen; Balance-Runde | ✅ |
| M23 | Etappenziele | ✅ |
| M24 | Wettbewerb und Preise im Verlauf | ✅ |
| M25 | Produktionsketten | ✅ |
| M26 | Weiterlaufen bis … | ✅ |
| M27 | Ursachen erklären | ✅ |
| M28 | Geschichte erzählen | ✅ |
| M29 | Rang in der Übersicht | ✅ |
| M30 | Kaufangebote I: Standorte, Labore, Lizenzen | ✅ |
| M31 | Kaufangebote II: ganze Bereiche | ✅ |
| M32 | Produkte 1915–1939 | ✅ |
| M33 | Produkte 1940–1964 | ✅ |
| M34 | Regionen | ✅ |
| M35 | Grundstücke mit Lage | ✅ |
| M36 | Anlagen in fünf Größen | ✅ |
| M37 | Weiterentwicklung erforschter Produkte | ✅ |
| M38 | Pleiten: Standorte weitergeben | ✅ |
| M39 | Produkte 1965–1989 | ✅ |
| M40 | Produkte 1990–2009 | ✅ |
| M41 | Produkte 2010–2026 | in Arbeit |
| M42 | Produktnamen je Firma | in Arbeit (nach M41) |

Die Vorschläge aus `docs/OFFENE_PUNKTE.md`, Abschnitt E, sind freigegeben (Lastenheft
§18.3) und als M22–M29 umgesetzt. Kaufangebote zwischen Firmen und die Produkte bis 2026
(Lastenheft §18.4, Abschnitte F und G der offenen Punkte) folgen als M30–M33 und
M39–M41; M30–M39 sind umgesetzt. Regionen, Grundstücke, Anlagengrößen, Weiterentwicklung
und Pleiten (Abschnitte H–K, 06.10.2026) kommen als M34–M38 vor den letzten Epochen.

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
- M15 Balancing (Protokolle 1900–1930 mit 100 KI-Firmen):
  - Kohle als Nebenbrennstoff (0,002–1,2 t je Einheit) blockierte ganze Branchen
    (Werkzeug, Konserven, Mehl, Garn, Stoff …), weil kleine Mengen in vielen Ländern
    fehlten; sie ist nur noch Vorprodukt bei Stahl, Kupferhütte und Kraftwerk. Ebenso
    entfallen Nähgarn bei Kleidung und Garn bei Reifen (Lastenheft §17.2: lieber ein
    Eingangsmaterial weglassen).
  - KI baut auch Vorstufen aus, die sie selbst weiterverarbeitet (vorher nie), und
    diversifiziert: Je Quartal bauen bis zu vier reiche Firmen im größten Engpass der
    Welt. Vorher entstanden neue Hersteller nur als Ersatz für Pleiten.
  - Preisuntergrenze auf Vollkosten; Strom in der Rezeptwahl nach Netzanteil bewertet.
  - Höchstpreis auf das Vierfache des Richtpreises gesenkt (vorher 20-fach); die
    Preisspitzen machten Knappheit zur Goldgrube.
  - Förderung von Erz, Holz, Getreide, Kupfererz, Rohöl und Kautschuk mit historisch
    plausiblen Arbeitszeiten (vorher kostete Förderung fast nichts, Rohstoffe fielen auf
    4–46 % des Richtpreises). Richtpreise der Kleinteile an ihre Kosten angepasst
    (Nägel 1 800, Schrauben 2 800, Muttern 2 500 USD je t).
  - Ergebnis: Kleidung von 8–9 × auf unter 2 × Richtpreis, Autoproduktion von 365 auf
    über 7 000 im Jahr, Pleiten bis 1925 von 65 auf 23, Eisenerz und Holz nahe am
    Richtpreis. Offen: Baumwolle und Garn bleiben knapp und teuer.
  - Protokoll mit Aufschlüsselung nach Ländern (`laender.csv`) zum Finden von
    Handelsproblemen.
  - Testpartie 1900–1930 als Test (`crates/wsim-session/tests/testpartie.rs`, ignoriert,
    `--release`): Ein Spieler mit einfacher Strategie spielt 30 Jahre über die Sitzung.
- M16 Markteintritt und Marketing (Auftrag vom 04.10.2026: gesättigte Märkte zum Start,
  der Spieler gewinnt Marktanteile nur über Preis, Qualität, Marke, Präsenz und
  Industriekunden; Werbung für Endprodukte; abwählbares Tutorial):
  - Die Startbesetzung plant 115 % des Bedarfs (`marktdeckung`); etablierte Firmen sind
    dort bekannt, wo sie zum Start Endprodukte verkaufen (B = 0,5, historische 0,7).
  - Markenbekanntheit je Firma, Land und Warengruppe wächst mit Werbung (bestes
    verfügbares Werbemittel: Zeitung, Radio ab 1923, Fernsehen ab 1950, Internet ab
    1995) und Mundpropaganda (Marktanteil) und verblasst ohne beides. Die Parameter
    stehen in `marktmodell.marke` statt in einer eigenen Datei, weil sie die
    Anbieterwahl des Marktes betreffen. KI-Firmen werben mit 1–3 % ihres Umsatzes.
  - Präsenz in der Anbieterwahl: Ohne sie bekam ein Neuling mit einer Nagelmaschine bei
    gleichem Nutzen so viele Kunden wie ein großes Werk und verkaufte alles mit hohem
    Aufschlag (Protokoll: Werkstatt ohne eigenes Zutun 0,4–0,8 Mio. USD Gewinn im Jahr).
    Jetzt zählt jedes Angebot mit seiner Erzeugung und seinem Lager.
  - Preisgewicht der Endkunden von 2,0–0,6 auf 7–3: Massenware wird über den Preis
    verkauft; vorher hielt ein Anbieter mit doppeltem Preis noch über ein Drittel der
    Kunden.
  - KI-Lagerreichweite misst am Absatz statt an der eigenen Erzeugung; vorher trieb
    jede Drosselung die Reichweite hoch und die Auslastung bis zum Minimum, sodass die
    Welt nach einem halben Jahr nur die Hälfte der Nachfrage erzeugte.
  - Händlerlager zum Start: Länder ohne eigene Erzeugung bekommen 30 Tage ihrer Lücke,
    eingekauft im günstigsten Land mit Überschuss plus Transport; der Einfuhrpreis
    deckt diesen Einkauf, sodass der Handel vom ersten Tag an läuft (vorher kam der
    erste Nachschub erst nach Wochen des Mangels und leerte dann die Lager der
    Exporteure auf einen Schlag).
  - Baumwolle: Ertrag der Lagerstätten um 50 % erhöht. Baumwolle steht in Stufe 1 für
    alle Naturfasern (Wolle, Leinen, Seide), deren Bedarf die Kleidung mitträgt.
  - Ergebnis (Protokoll 1900–1930, 100 KI-Firmen): Die Werkstatt ohne eigenes Zutun
    verdient 30 000–70 000 USD im Jahr und ab etwa 1910 kaum noch etwas (vorher
    0,4–0,8 Mio. USD). Nach drei Monaten sind alle Endprodukte zu 88–100 % versorgt.
    Pleiten bis 1929: 31. In der Testpartie (Nägel, bis zu 20 Maschinen) steht der
    Spieler 1930 bei 0,66 Mio. USD Eigenkapital statt 18 Mio.
  - Oberfläche: Die Marktansicht zeigt je Produkt den Marktführer mit Anteil und den
    eigenen Anteil, darunter je Warengruppe die eigene Bekanntheit neben der
    bekanntesten Konkurrenz und das Werbebudget (Befehl `SetAdvertising`) sowie das
    beste Werbemittel und die Summe, die das Land einmal erreicht. Einfuhr und Ausfuhr
    stehen in einer Spalte, damit die Tabelle breit genug für die neuen Spalten bleibt.
  - Einführung (Lastenheft §14.4): im Dialog „Neues Spiel“ abwählbar (Standard: an),
    acht Schritte von der Lage als Neuling über Übersicht, Produktion, Markt, Werbung,
    Forschung und Finanzen bis zur Weltkarte. Sie liegt als Fenster neben dem Spiel
    und sperrt es nicht, wechselt bei jedem Schritt in die passende Ansicht und lässt
    sich über die Tastenhilfe („?“) neu starten. Sie gehört nur zur Oberfläche: Der Kern
    kennt sie nicht, und ein geladenes Spiel beginnt ohne sie.
  - Fehler behoben: Märkte von Gütern, die nur Firmen kaufen (Draht, Stahl, Holz …),
    schlossen ihren Monat nie ab; Markt- und Länderansicht zeigten dort keine Nachfrage
    und keinen Absatz. Die Simulation selbst las diese Werte nicht.
  - Bekannt, als Nächstes (Weltlauf-Diagnose): Nach dem ersten Quartal sinkt die
    Versorgung einzelner Konsumgüter weltweit auf 75–90 %, obwohl genug Anlagen da sind.
    Ursachen: (1) Händler zählen die Ware unterwegs gegen ihren 30-Tage-Vorrat; auf
    Seewegen über 30 Tage (China, Indien) kommt so dauerhaft nur ein Teil an. (2) Der
    Marktpreisindex eines Vorprodukts beginnt bei 0 und steigt nur langsam; darauf
    gestützte Gebote der KI erreichen die Verkäufer nicht (Möbelwerke ohne Leim).
    (3) Die KI ändert die Auslastung nur in Schritten von 0,1 und braucht Monate, um einer
    gestiegenen Nachfrage zu folgen. → behoben in der Nacharbeit (folgender Abschnitt).

### Nacharbeit zu M16: Plausibilität und Versorgung (05.10.2026)

Auftrag: generische Lösungen statt Einzelkorrekturen („das ganze Spiel sollte sich so
verhalten“), Plausibilitätsprüfungen, den Versorgungszweig gleich mit optimieren.
Grundsatz: Jede Regel gilt für alle Produkte und Länder gleich und hängt nur an der
Produktart, den Daten und Marktsignalen. Formeln: `docs/FORMELN.md`, Abschnitt
„Nacharbeit zu M16“; Entscheidungen zum Widersprechen: `docs/OFFENE_PUNKTE.md` 30–41.

  - Prüfungen: `validate` prüft die Richtpreise gegen die Herstellkosten des besten
    Rezepts (Marge 5–45 %, Förderung mit ihrer Pacht); `wsim rezepte` rechnet dasselbe
    für beliebige Länder und Jahre. Das Balance-Protokoll prüft neun Kennzahlen für alle
    Produkte und Länder gleich (Kern: `health`). Der Test
    `world_stays_plausible_in_the_first_year` lässt CI ein Jahr Weltlauf auf Brüche
    prüfen.
  - Kosten: Arbeitsproduktivität nach Wohlstand; Gemeinkosten als Zuschlag auf die
    Umwandlungskosten je Produktart; Pacht und Förderabgaben je Rohstoff (neue
    Kostenart); Preise der Waren je Land mit einem Anteil des Preisniveaus je Produktart
    (Waren sind handelbar), auch als Startpreis jedes Markts.
  - Markt und Handel: Index ab dem ersten Verkauf; Ausfuhr erst nach den Käufern im
    Land; Händlerbedarf mit Transporttagen; Händlerlager höchstens zum
    Wiederbeschaffungspreis; Knappheit im Ausland hebt Preise; freie Anlagen senken
    Preise; Staaten kaufen auch zu Weltpreisen; knapp ist nur, wofür ein Käufer mehr
    zahlen würde; Preisschritte 0,25 %/0,125 % je Tag statt 2 %/1 %.
  - KI: Auslastung nach Absatz und Lagerziel (erster Monat ohne Vormonat richtig
    gerechnet; ausverkauft über der Untergrenze → mehr; je Entscheidung höchstens ±0,3;
    Start im Gleichgewicht mit dem geplanten Absatz als Vormonat); Untergrenze aus
    Einstandskosten;
    Gebote bis zur Zahlungsbereitschaft aus dem Erlös; Eigenstrom, wenn das Netz bremst;
    kein Ausbau bei knappen Vorprodukten; Förderanlagen wachsen mit der Konzession.
  - Daten: Förderindex für Getreide, Baumwolle, Holz; Pacht für sieben Rohstoffe;
    Kleidung 5 statt 6 Stück je Kopf (Faserbedarf 1900); Fließband für Motor und Fahrgestell;
    Staatsnachfrage für sonstige Gummiwaren; Entsorgung überschüssiger Nebenprodukte
    (Benzin um 1900); Korrekturen aus der Margenprüfung (Nägel, Werkzeug, Schnittholz,
    Möbel, Glühlampe, Nähmaschine, Cracken, Stahl, Konserve, Fahrgestell, Montage).
  - Fehler behoben: Die Höchstförderung in der Ansicht „Lagerstätte erschließen“ war
    ohne Marktmaßstab angegeben (bei 100 KI-Firmen zehnmal zu hoch).
  - Fehler behoben: Kaufte ein Staat oberhalb seiner Preisgrenze nicht, galt das als
    Mangel; die Anbieter hoben ihre Preise weiter (Nägel und Mehl im ersten Jahr beim
    Drei- bis Vierfachen).

Ergebnis im Balance-Protokoll (1900–1930, 100 KI-Firmen, Seed 1; Zahl der Verstöße, die
Grenzen gelten für alle Produkte und Länder gleich):

| Prüfung | Stand M16 | Zwischenstand (vor Pacht und Dämpfung) | Endstand |
| --- | --- | --- | --- |
| Versorgung von Verbrauchern und Staaten weltweit (≥ 90 %) | 488 | 39 | 18 |
| Versorgung je Land (≥ 75 %) | 5.774 | 550 | 311 |
| Erzeugung, die auf Vorprodukte wartet (≤ 10 %) | 131 | 114 | 12 |
| Erzeugung, die auf Arbeitskräfte wartet (≤ 10 %) | – | 2 | 1 |
| Preis gegen Richtpreis (0,5–2 ×) | 404 | 429 | 146 |
| Marge über Vollkosten (−20 bis 50 %) | – | 96 | 38 |
| Jahresergebnis des passiven Spielers (≤ 50 %) | 4 | 2 | 4 |
| Pleiten | 0 | 0 | 0 |
| Förderung gegen den Bedarf der Anlagen (≥ 90 %) | 78 | 68 | 27 |

Preise 1929 gegen den Richtpreis im Land (Endstand): Rohstoffe 0,81–1,07 (vorher
Getreide 0,36, Kautschuk 0,37, Rohöl 0,41), Mehl 0,90, Kupfer 0,80, Petroleum 0,86;
Baumwolle 1,06 und Kleidung 0,66 (vorher 3,83 und 1,40); Stahlkette 0,45–0,66 (real
fielen diese Preise bis 1929 ebenfalls); Auto 0,32 mit Fließband (der Preis des ersten Massenautos fiel 1909 bis
1925 real auf etwa ein Siebtel). Im ersten Jahr schwankt Getreide nicht mehr zwischen 0,55
und 3,9 des Richtpreises. Offen: `docs/OFFENE_PUNKTE.md` 38.

Vorgehen: Jede Änderung im Weltlauf gegen den vorherigen Stand gemessen; Varianten
parallel (Startpreis, Gleichgewichtsstart, Preisschritte 2/1, 1/0,5, 0,5/0,25 und
0,25/0,125 % je Tag, Niederstwertprinzip, Mindestschritt nach Richtpreis, Ausbau über der
Normalauslastung). Verworfen, was keine Verbesserung brachte (`docs/FORMELN.md`).

### M17: Browser-Version (05.10.2026)

Auftrag: eine Browser-Ansicht neben dem Windows-Programm, um neue Stände vom iPhone aus zu
prüfen. Entscheidungen des Auftraggebers: Lastenheft §18.2.

  - `crates/wsim-web`: die Spielsitzung als WebAssembly-Modul mit eingebauten Daten
    (1,9 MB YAML, Modul 5,3 MB). Schnittstelle: JSON-Anfragen mit denselben Namen wie die
    Befehle der Desktop-App; Spielstände als Bytes. Ohne wasm-bindgen: eine kleine
    Brücke mit `unsafe` (Speicher zwischen JavaScript und Modul), dafür keine weiteren
    Werkzeuge im Build. Das Crate hat eigene Lint-Regeln (`unsafe_code = "deny"`, im
    Modul `bridge` erlaubt); sonst gilt das Verbot im Workspace weiter.
  - Spielsitzung: Spielstände über `SaveStore` (Dateien am Desktop, Speicher im Browser);
    die Web-Version legt geschriebene Stände in IndexedDB ab und beim Start wieder ein.
  - Oberfläche: `ui/src/kern/web.ts` mit Web Worker (`kern.worker.ts`); Bauart „web“
    (`pnpm -C ui build:web`). Grundlayout fürs Handy: Kopfleiste und Reiter als ein
    fester Kopfbereich, Reiter zum Wischen, Dialoge in Bildschirmbreite, Tabellen in
    eigenem Rahmen (vorher machten breite Tabellen die ganze Seite breiter, der Browser
    zoomte heraus, und Knöpfe in Dialogen lagen außerhalb). Jede Ansicht beginnt oben.
  - Prüfungen in CI (Job „Browser-Version“): Clippy für das WebAssembly-Ziel; gleicher
    Zustands-Hash nach zwei Monaten mit 20 KI-Firmen in Browser und Desktop
    (`f8e22892a572d015`); Playwright gegen den echten Kern im Desktop- und iPhone-Format
    (Spiel beginnen, Woche spielen, speichern, Seite neu laden, Stand laden).
    Veröffentlichung nach jedem Push auf `claude/architektur-vorschlag` in den Zweig
    `gh-pages`.
  - Leistung (Node, ein Kern): Daten laden 0,26 s, neues Spiel mit 100 KI-Firmen 0,07 s,
    ein Monat 1,5–1,7 s; Speicher etwa 90 MB.

### M18: Bedienung, Personal und Werksansicht (05.10.2026)

Auftrag: Lesbarkeit und Zuordnung von Überschriften und Feldern prüfen, das einzelne Werk
detaillierter zeigen, Preisanpassungen reparieren, Arbeitskräfte einstellbar machen,
alle Ansichten auf die Entscheidungen des Spielers ausrichten. Entscheidungen des
Auftraggebers: Lastenheft §18.2 (Manager erst in Stufe 2; Personal sichtbar mit
Lohnaufschlag).

  - **M18a Prüfung:** `docs/BEDIENUNG.md` – Grundsätze und Befund je Ansicht. Ursache von
    „Preisanpassungen gehen teilweise nicht“: Im Modus Marktpreis war das einzige Feld
    die Untergrenze; ein niedrigerer Wert änderte den laufenden Preis nicht. Dazu
    erschien die Rückmeldung oben außer Sicht, und „1.800“ wurde als 1,8 gelesen.
  - **M18b Personal:** Lohnaufschlag je Standort (`SetWagePremium`, 0 bis
    `lohnaufschlag_max`); er gilt für Lohnkosten, den Wert der Erzeugnisse und die
    Kostenschätzungen der KI. Besetzung nach Aufschlag; wer mehr zahlt, wirbt bei
    Knappheit Arbeitskräfte von Standorten desselben Landes mit niedrigerem Aufschlag ab
    (Formeln: `docs/FORMELN.md`, M18). KI: Schritt nach oben bei Personalmangel, sonst
    zurück bis 0 (höchstens 30 %). Im Weltlauf 1900 ändert das nichts (keine
    Personalengpässe); erst 1917 trat im Protokoll ein Engpass auf (Kupfererz).
  - **M18c Werksansicht und Preise:** Reiter „Standorte“ mit einer Karte je Standort
    (Umsatz und Ergebnis des Vormonats, Beschäftigte, Engpässe); „Öffnen“ führt in die
    Werksansicht mit Anlagen, Einkauf, Verkauf, Personal sowie Kosten und Ergebnis.
    Neuer Befehl `SetPrice`: Der eingegebene Preis gilt sofort, auch im automatischen
    Modus (der Preis folgt danach wieder dem Markt); Schnellknöpfe −5 %/+5 %; neben dem
    Preis stehen Marktpreis, Richtpreis, Stückkosten und Marge auf einer Skala.
    Stückkosten je Erzeugnis aufgeschlüsselt (Material, Personal, Energie, Gemeinkosten,
    Pacht, Anlage); Ergebnis des Standorts im Vormonat je Kostenart und Rohertrag je
    Produkt. Dafür führt das Hauptbuch Ergebnisse je Kostenstelle und Kostenart (nur
    laufender und letzter abgeschlossener Monat und Jahr, Spielstände bleiben klein) und
    verrechnet die Löhne der genutzten Stunden intern auf die Produkte.
  - Bedienung allgemein: Jedes Feld hat eine eigene Beschriftung mit Einheit, Zahlen
    werden deutsch gelesen („1.800“ = 1800, „2,5“ = 2,5), Rückmeldungen erscheinen am
    Formular. Auf dem Handy werden Tabellen zu Karten; lange Beschriftungen brechen um,
    statt Werte aus der Karte zu schieben.
  - Spielstände: neue Felder mit Vorgabewerten, ältere Stände laden unverändert
    (keine neue Formatversion).
  - **M18d Ansichten:** Kopfbereich in einer Zeile mit Menü „☰“ (Speichern, Laden,
    Tastaturkürzel, Hauptmenü), Kassentrend und Zähler offener Warnungen am Reiter
    Übersicht. Übersicht: „Zu erledigen“ mit Sprung an die Stelle (Werk und Bereich),
    Kennzahlen mit Verlauf (Kasse, Umsatz, Ergebnis je Monat), Standorte als Karten,
    Wettbewerb aufklappbar. Markt: Unterreiter Produkte und Marke und Werbung; je
    Produkt Preis mit Trend zum Vormonat, Preis zum Richtpreis als Balken, Nachfrage mit
    Einheit, Versorgung als Ampel, Kennzeichen für Chancen und Filter; Klick öffnet den
    Produktmarkt (Anbieter mit Preisen und Anteilen, wer kauft: Einkommensfünftel,
    Staat, Firmen; Ein- und Ausfuhr). Finanzen: Verlauf und „Womit verdienst du Geld?“
    (Ergebnis je Standort, Rohertrag je Produkt, Vormonat oder Jahr). Rundenbericht:
    „Was lief“ je Produkt und „Jetzt zu tun“. Neues Spiel: Grundeinstellungen oben,
    Weiteres aufklappbar, Erklärung der Startformen. Weltkarte: Ebene
    „Absatzchancen“ für ein Produkt (Preis zum Richtpreis je Land, Nachfrage als Kreis);
    im Länderdetail „Standort hier gründen“. Browser-Version: Spielstände als Datei
    herunter- und hochladen (Wechsel zwischen iPhone und Rechner). Auf dem Handy
    werden Tabellen zu zweispaltigen Karten; die Marktliste ist fünfmal kürzer.
  - Kern: Hinweise, Monatsverlauf, Ergebnisse je Kostenstelle, Umsatz und Rohertrag
    je Produkt in der Runde, Produkt- und Weltmarkt als Sichten (mit Tests); neue
    Anfragen `produktmarkt` und `weltmarkt` in Desktop-App, Browser-Version und Vorschau.

### M19: Technologiebaum (05.10.2026)

Auftrag: ein einsehbarer Technologiebaum – welche Technologien, Verfahren und Produkte
erforscht werden können und welche Betriebsmittel (Anlagen) sie brauchen.

  - **Baum:** Spalten nach Zeitabschnitt der Erfindung, Zeilen nach Fachgebiet (die
    Fachgebiete bleiben beim seitlichen Blättern stehen), Linien zu den Voraussetzungen;
    die gewählte Technologie hebt ihre Linien hervor, der Baum öffnet sich an der
    Forschungsgrenze. Stand je Knoten: bekannt, in Arbeit, erforschbar, Voraussetzungen
    fehlen (Farbe und Rahmen, dazu eine Legende). Wahlweise **Liste** nach Fachgebiet;
    auf dem Handy ist sie voreingestellt, die Details öffnen sich unter dem Eintrag.
  - **Je Technologie:** Fortschritt in Punkten, Aufwandsfaktor (Vorgriff teurer,
    Nachzügler günstiger), Dauer mit den eigenen Zentren bzw. mit einem voll besetzten
    Labor und die Kosten dafür (Formeln: `docs/FORMELN.md`, M19); Voraussetzungen und
    „Führt zu“ als Sprünge; **Schaltet frei:** Anlagen (Art, Investition, Bauzeit),
    Verfahren (Ein- und Ausgänge je Anlage und Tag, nötige Anlage und gegebenenfalls die
    Technologie, die ihr noch fehlt) und Produkte; bei bekannten Technologien führt
    „Anlage bauen“ zu den Standorten. Forschung startet direkt aus dem Baum; ohne
    Zentrum mit Labor führt ein Hinweis zu den Forschungszentren.
  - **Forschungszentren:** eigener Unterreiter mit Stand (Forscher auf Plätzen, Labor im
    Bau), Fortschritt des Projekts, Projektwahl, Auslastung je Labor, Labor bauen und
    neues Zentrum gründen.
  - Mengen werden mit drei gültigen Stellen gezeigt („0,021 t Kupferdraht“ statt „0 t“).
    Ein Test prüft, dass es jeden festen Textschlüssel der Oberfläche gibt.
  - Tests: Kern (Stand, Schätzung, „Führt zu“, Freischaltungen), Vorschau (Baum, Detail,
    Sprung zu Voraussetzungen, Liste) und Browser-Version mit echtem Kern auf Desktop und
    iPhone (Zentrum gründen, Labor bauen, Forschung starten).



### M20: Geführte Einführung (05.10.2026)

Auftrag: Die Einführung soll intensiver begleiten, bis die ersten Produkte verkauft sind,
und die konkret zu benutzenden Schaltflächen hervorheben.

  - **Schritt für Schritt zum ersten Verkauf:** Jeder Schritt hebt die Schaltfläche
    hervor, die der Spieler benutzen soll (pulsierender Rahmen, der Rest des Bildschirms
    abgedunkelt), und geht von selbst weiter, sobald sie benutzt wurde. Ist der Spieler
    woanders, zeigt der Rahmen den Weg dorthin (Reiter, „Öffnen“, Bereich). Nichts wird
    gesperrt; „Überspringen“, „Zurück“ und „×“ sind immer da, die Tafel lässt sich auf
    eine Zeile einklappen.
  - **Zwei Wege je nach Startform:** Werkstatt – Standorte, Werk öffnen, Anlage,
    Einkauf (Draht), Verkauf (Preis, z. B. −5 %), Personal, Runde beenden, Bericht.
    Handelsniederlassung – Niederlassung öffnen, Ware einkaufen, Ware anbieten, Runde,
    Bericht. Ohne Verkauf in der Runde nennt die Einführung die üblichen Ursachen und
    führt zur nächsten Runde; nach dem ersten Umsatz folgen „Zu erledigen“ und ein
    kurzer Rundgang durch Markt, Marke, Forschung, Finanzen und Weltkarte.
  - Dialoge lassen der Tafel Platz (auf breiten Bildschirmen daneben, sonst darüber);
    auf breiten Bildschirmen hält auch die Ansicht rechts eine Spalte frei.
  - Gefunden beim Durchspielen: Eine Handelsniederlassung sah für neu gewählte Waren
    keinen Marktpreis (die Werksansicht liefert jetzt die Preise aller Produkte im
    Land); das Ziellager beginnt beim Handel leer statt mit 0. Der Rundenbericht zeigt
    die **Lagerveränderung** (zu Herstellkosten), damit Umsatz − Kosten + Lager =
    Ergebnis aufgeht.
  - Tests: Einführung in der Vorschau (alle Schritte, Zurück, Einklappen, Neustart),
    mit echtem Kern auf Desktop und iPhone bis zum ersten Umsatz; Unit-Test, dass es zu
    jedem Schritt Texte und jede Markierung in den Ansichten gibt; Kern-Test, dass die
    Zeilen des Rundenberichts aufgehen.

### M21: Landeswährungen (05.10.2026)

Auftrag: Landeswährungen mit angenommenen Wechselkursen und historischem Verständnis;
Währungen, die es 1900 noch nicht gab, nach Annahmen. Entscheidung (§18.2): vorgezogen,
Anzeige umschaltbar zwischen Kaufkraft 2026 und zeitgenössischen Preisen.

  - **Daten** (`data/waehrungen/`): 263 Währungen mit Kurzzeichen und Kursen je
    US-Dollar 1900–2026 (Goldparitäten bis 1914, Abwertungen und Währungsreformen auf
    den Monat genau, Paritäten von Bretton Woods, danach gerundete Jahresmittel, die
    deutsche Hyperinflation bis 4,2 Billionen Mark je Dollar) oder fest an eine andere
    Währung gebunden (CFA-Franc, Rand-Raum, ostkaribischer Dollar, …); für alle 197
    Länder die Zeiträume ihrer Währungen, z. B. Deutschland: Mark, ab Dezember 1923
    Reichsmark, ab Juni 1948 D-Mark, ab 1999 Euro. Dazu der Verbraucherpreisindex der
    USA 1900–2026 (danach 2 % Teuerung im Jahr). Prüfregeln mit deutschen Meldungen und
    Tests für alle Fehlerfälle; ein Test prüft alle Länder und Jahre 1900–2100.
  - **Anzeige:** Im Menü ☰ wählt der Spieler die Währung (Landeswährung des
    Firmensitzes oder US-Dollar) und die Preise (Kaufkraft 2026 ohne Inflation oder
    Preise der Zeit mit Inflation). Voreinstellung: Landeswährung, Kaufkraft 2026. Die
    Übersicht nennt, worin die Beträge stehen. Eingaben in Formularen (Preise,
    Höchstpreise, Kredite, Tilgung, Werbebudget) gelten in der gezeigten Währung.
    Formeln: `docs/FORMELN.md`, M21.
  - Das Länderdetail der Weltkarte nennt die Währung mit dem Kurs der Zeit („Mark (M) ·
    4,22 M je US-Dollar (1914)“) und ihre Abfolge („Mark, ab Dezember 1923 Reichsmark,
    ab Juni 1948 D-Mark, ab 1999 Euro“).
  - Kleine Beträge einer starken Währung werden mit drei gültigen Stellen gezeigt
    („0,00512 £“), sehr große gekürzt bis „Trill.“ (Hyperinflation).
  - Tests: Kern (Kurse, Bindung, Teuerung, Optionen der Übersicht), Daten (Fehlerfälle,
    alle Länder und Jahre), Oberfläche (Formate), Vorschau (Umschalten, Eingabe in Euro)
    und Browser-Version mit echtem Kern (Mark zu Preisen von 1900).

Eigenständige Entscheidungen:

  - Die Währung wirkt nur auf die Anzeige; die Simulation, die Daten und der Spielstand
    bleiben in US-Dollar mit der Kaufkraft 2026 (Wechselkurswirkungen auf Handel und
    Löhne bleiben in Stufe 4). Die Wahl der Anzeige merkt sich der Browser bzw. die App,
    sie gehört nicht zum Spielstand.
  - „Kaufkraft 2026“ zeigt die Landeswährung von 2026 (in Deutschland den Euro), denn
    eine Mark „mit der Kaufkraft von 2026“ gibt es nicht; Mark, Reichsmark und D-Mark
    erscheinen mit „Preise der Zeit“ (offener Punkt 3).
  - Alle Beträge eines Bildschirms werden mit dem Faktor des aktuellen Spieldatums
    umgerechnet, auch Verläufe – sonst wären die Monate vor und nach einer Reform oder
    während der Hyperinflation nicht vergleichbar.
  - Vor einer eigenen Währung gilt die der Kolonial- oder Vormacht; Übergangswährungen
    sind zusammengefasst, wo sie nur kurz galten (Rentenmark als Reichsmark,
    Cruzado novo, Syli, Ekwele); bei gespaltenen Kursen gilt der Kurs, zu dem
    tatsächlich gehandelt wurde. Annahmen stehen als Kommentar in den Daten.
  - Außerhalb einer Partie (Neues Spiel, Startkapital) bleibt die Anzeige in US-Dollar.

### M22: Anlagen stilllegen und verkaufen (05.10.2026)

Vorschlag 1 aus `docs/OFFENE_PUNKTE.md`, Abschnitt E (freigegeben, Lastenheft §18.3).
Formeln: `docs/FORMELN.md`, M22.

  - **Befehle für Spieler und KI:** Anlagen ganz oder teilweise (Zahl der Einheiten)
    stilllegen, wieder anfahren oder verkaufen. Stillgelegt: keine Erzeugung, kein
    Personal, kein Verschleiß, ein Viertel der Wartung. Wieder anfahren: 2 % der
    Investition, 30 Tage. Verkauf: halber Restbuchwert, mindestens 3 % der Investition
    als Schrottwert; Gewinn oder Verlust gegen den Buchwert wird gebucht.
  - **Werksansicht:** „Stilllegen oder verkaufen“ unter jeder Anlage mit Wartung,
    Restbuchwert, Erlös und Rückfrage vor dem Verkauf; stillgelegte Anlagen zeigen
    „stillgelegt seit …“ und „Wieder anfahren“. Die Übersicht zählt stillgelegte Anlagen;
    die Ursache „stillgelegt“ bzw. „fährt wieder an“ steht beim Engpass.
  - **KI:** vierteljährlich je Standort und Produkt nach dem Abgang: unter der Hälfte der
    Leistung und bei einem Preis unter dem Richtpreis legt sie still, über 95 % fährt sie
    wieder an (Abstand gegen Pendeln); nach 24 Monaten Stillstand verkauft sie. Der
    Rundenbericht meldet, wenn ein Wettbewerber in den Märkten des Spielers stilllegt
    oder verkauft.
  - **Balance-Runde** (Weltlauf 1900–1930 gegen M21): Überkapazität über zehn Jahre bei 23
    statt 30 Produkten, Benzin 1929 bei 1,18 statt 0,00 × Richtpreis, Autoreifen ohne
    Monopolmarge (14 % statt 49 %), 4 statt 13 Pleiten. Dazu: knappe Waren weit unter dem
    Richtpreis holen schneller auf; Nebenprodukte nicht unter ihrem Brennwert; Benzin mit
    übriger Verwendung; die KI baut ab 1925 Crackanlagen (Tabelle in FORMELN).
  - Tests: sieben Szenariotests der Befehle (Kosten, Buchung, Teilstilllegung, Prüfung,
    Speichern und Wiederholen), vier KI-Tests (Stilllegen und Verkauf, Untergrenze nach
    dem Brennwert), Prüfregeln der neuen Parameter, Vorschau (Stilllegen, Verkaufen mit
    Rückfrage) und Browser-Version mit echtem Kern (Stilllegen und wieder anfahren).

Eigenständige Entscheidungen:

  - Verkaufserlös und Kosten des Stillstands sind Annahmen (Parameter in
    `data/parameter/produktionsmodell.yaml`); der Spieler bekommt dieselben Bedingungen
    wie die KI.
  - Förderanlagen legt die KI nicht still (sie hängen an ihrer Konzession), Strom und
    Forschung ebenfalls nicht; der Spieler darf alle Anlagen stilllegen.
  - Die Prüfung „Förderung gegen Bedarf der Anlagen“ zeigt mehr Verstöße (26 → 90,
    schlechtester Wert 74 %): Ohne Überkapazität schwankt der Einkauf der Anlagen
    stärker als die Förderung. Hingenommen, weil die Märkte selbst besser versorgt sind.
  - Benzins übrige Verwendung läuft wie bei Gummi über die Staatsnachfrage, bis
    Lösungsmittel und Motoren eigene Abnehmer bekommen.
  - Gummi bleibt 1902–1912 knapp (Kautschukboom); dass die Kautschukförderung dabei unter
    ihre Kapazität fällt, ist als offener Punkt 38 vermerkt.

### M23: Etappenziele (05.10.2026)

Vorschlag 2 aus `docs/OFFENE_PUNKTE.md`, Abschnitt E. Formeln: `docs/FORMELN.md`, M23.

  - **Daten** (`data/etappen.yaml`): acht Etappen in dieser Reihenfolge – erster Verkauf,
    erster Monat mit Gewinn, zwei Anlagen, eigenes Vorprodukt, zweites Land, erste
    eigene Erfindung, Marktführer in einem Land (mindestens 30 % des Absatzes),
    Eigenkapital verdoppelt. Name und „So geht's“ als Texte; Prüfregeln mit deutschen
    Meldungen (Art, Wert, Texte) und Fehlerfall-Tests.
  - **Kern:** bewertet die Etappen des Spielers am Ende jedes Tages, merkt sich das Datum
    (Spielstand, nach Schlüsseln gespeichert) und meldet jede einmal als Erfolg. Keine
    Wirkung auf die Simulation; KI-Firmen haben keine Etappen.
  - **Oberfläche:** Bereich „Etappen“ in der Übersicht (nächste Etappe mit Fortschritt und
    Hinweis, alle Etappen aufklappbar), ausblendbar und über das Menü ☰ wieder
    einblendbar; Rundenbericht mit der Gruppe „Erreicht“; die Einführung führt nach dem
    ersten Verkauf zu den Etappen.
  - Tests: Kern (einmalige Meldung mit Datum, bleibt nach Verkauf der Anlagen erreicht,
    Speichern und Wiederholen, Eigenkapital, Marktführerschaft gegen einen Wettbewerber,
    Länder und eigenes Vorprodukt), Daten (Fehlerfälle), Vorschau (Anzeige, Ausblenden,
    Menü) und Browser-Version mit echtem Kern („Etappe erreicht: Erster Verkauf“).

Eigenständige Entscheidungen:

  - Vorschlag war eine Bewertung nur als Sicht. Sie läuft nun im Tageslauf des Kerns,
    damit eine erreichte Etappe ihr Datum behält und genau einmal im Rundenbericht steht;
    sie verändert nichts außer der Liste der erreichten Etappen.
  - „Erster Verkauf“ ist die erste Etappe, auch wenn die Einführung dorthin führt: Wer
    ohne Einführung spielt, bekommt so dieselbe Richtung.
  - „Zwei Anlagen“ zählt fertige Einheiten (auch stillgelegte): Eine zweite Nagelmaschine
    am selben Standort zählt wie eine neue Anlage.
  - Marktführer heißt: im Vormonat in einem Land mehr verkauft als jede andere Firma und
    mindestens 30 % des ganzen Absatzes dort, also auch gegen Einfuhren der Händler und
    den Staatsmarkt. Ein knapper Vorsprung unter vielen kleinen Anbietern zählt so noch
    nicht.
  - Ob die Etappen gezeigt werden, merkt sich der Browser bzw. die App (wie die
    Geldanzeige), nicht der Spielstand.

### M24: Wettbewerb und Preise im Verlauf (05.10.2026)

Vorschlag 3 aus `docs/OFFENE_PUNKTE.md`, Abschnitt E. Formeln: `docs/FORMELN.md`, M24.

  - **Monatsreihe je Markt** im Spielstand: bezahlter Preis, Absatz und eigener Absatz der
    letzten 24 Monate (`marktmodell.verlauf_monate`), nur für Märkte mit Absatz.
  - **Produktmarkt:** Bereich „Verlauf“ mit drei kleinen Diagrammen – Preis gegen den
    Richtpreis (gestrichelt), Absatz je Monat, eigener Anteil. Monate ohne Verkauf
    schreiben den letzten Preis fort.
  - **Rundenbericht** (Gruppe „Wettbewerb“, Sprung zum Markt): neue und ausgeschiedene
    Anbieter in den Märkten, in denen der Spieler anbietet, und Preissenkungen ab 10 %
    (`marktmodell.meldung_preissenkung`) seit dem letzten Höchststand.
  - Tests: Kern (Reihe mit Kürzung, Lücken und eigenem Absatz; Meldungen für neue,
    ausgeschiedene und billigere Anbieter, auch bei schrittweiser Senkung), Daten
    (Fehlerfälle der Parameter), Oberfläche (Fortschreiben fehlender Preise) und Vorschau
    (Diagramme im Produktmarkt).

Eigenständige Entscheidungen:

  - Keine neue Formatversion: Die Reihen sind neue Felder mit Standardwert; ältere
    Spielstände beginnen ihre Reihen beim nächsten Monatsende.
  - Ein Spielstand 1903 mit 100 KI-Firmen wird dadurch rund 23 % größer (5,1 statt
    4,2 MB): Mehrere Tausend Märkte haben Absatz, auch über den Staatsmarkt und Einfuhren.
    Mengen werden einfach genau (f32) gespeichert, Preise als Geldbetrag.
  - Preissenkungen werden gegen den Höchststand seit der letzten Meldung gemessen, nicht
    gegen den Vormonat: Automatische Preise sinken höchstens etwa 4 % im Monat, ein
    Vergleich mit dem Vormonat meldete also nur feste Preise.

### M25: Produktionsketten (05.10.2026)

Vorschlag 4 aus `docs/OFFENE_PUNKTE.md`, Abschnitt E. Formeln: `docs/FORMELN.md`, M25.

  - **Kern:** neue Sicht `chains` – für jede Spitze einer Kette das günstigste nutzbare
    Rezept je Stufe bis zu den Rohstoffen, Stückkosten und Marge im Land des
    Firmensitzes, fehlende Technologien, was der Spieler herstellt, einkauft oder
    verkauft, und wo seine Anlagen haken. Über Sitzung, Tauri-Befehl `ketten`, die
    Browser-Version und die Vorschau-Beispieldaten verfügbar.
  - **Oberfläche:** Markt → „Produktionsketten“ als aufklappbarer Baum; die eigene Kette
    steht vorne, ein Klick auf ein Produkt öffnet seinen Markt im Heimatland.
  - Tests: Kern (Kette, Rezeptwahl, Kosten und Marge, eigene Anlage mit Engpass),
    Vorschau (Baum, Aufklappen, Wechsel der Kette, Sprung zum Markt) und Browser-Version
    mit echtem Kern.

Eigenständige Entscheidungen:

  - Ketten beginnen nicht nur bei Endprodukten, sondern bei jedem Produkt, aus dem nichts
    weiter hergestellt wird – sonst fehlte die Kette der Startwerkstatt (Nägel gehen an
    das Bauwesen).
  - Unterreiter im Markt statt eigener Hauptreiter: Die Ketten beantworten „Was lohnt
    sich?“ mit Preisen und Margen und führen in die Produktmärkte.
  - Die Ansicht deckt Ungleichgewichte der Daten auf (1914: Motor 80 %, Fahrgestell 83 %
    Marge mit dem Fließband); sie bleiben, wie im Weltlauf beobachtet, ein Restpunkt der
    Balance (offener Punkt 38).

### M26: Weiterlaufen bis … (05.10.2026)

Vorschlag 5 aus `docs/OFFENE_PUNKTE.md`, Abschnitt E. Regeln: `docs/FORMELN.md`, M26.

  - **Sitzung:** `end_round_until` reiht Monatsrunden aneinander – bis Jahresende oder bis
    zur ersten Runde mit Warnung, Krise oder Weltereignis (höchstens ein Jahr) – und
    liefert einen Bericht über den ganzen Zeitraum mit Zahl der Runden und Haltegrund.
    Tauri-Befehl und Browser-Version nehmen dafür `bis` entgegen.
  - **Oberfläche:** zwei Einträge unter „Mehrere Monate“ in der Rundenlänge; Fortschritt
    mit Rundenzähler; Bericht mit „n Runden am Stück“ und dem Haltegrund. Weltereignisse
    erscheinen wie gewohnt vor dem Bericht, jedes in einem eigenen Fenster.
  - Tests: Sitzung (Jahresende mit elf Monaten in einem Bericht, Halt bei Meldung,
    unbekanntes Ziel), Vorschau (Fortschritt, Bericht) und Browser-Version mit echtem Kern.

Eigenständige Entscheidungen:

  - Mehrere Runden laufen immer in Monatsschritten; die Auswahl ersetzt die Rundenlänge,
    statt ein zweites Auswahlfeld in die volle Kopfleiste zu setzen (Handy).
  - „Warnung“ heißt: eine Meldung der Art Warnung oder Krise (z. B. Konto überzogen,
    Vorprodukt fehlt) oder ein Weltereignis. Erfolge und Wettbewerbsmeldungen halten nicht
    an. Nach einem Jahr ohne Warnung hält das Spiel trotzdem, damit der Spieler nachsehen
    kann.

### M27: Ursachen erklären (05.10.2026)

Vorschlag 6 aus `docs/OFFENE_PUNKTE.md`, Abschnitt E. Formeln: `docs/FORMELN.md`, M27.

  - **Kern:** Die Sicht des Produktmarkts liefert die Teile des Marktpreises
    (Richtpreis, Preisniveau, Marktlage, Lage im Vormonat) und der Nachfrage der
    Verbraucher je Einkommensfünftel (Einkommen, Kaufneigung, Bedarf, Zielbesitz und
    Besitz). Die Verdrängung durch Nachfolger nutzen Simulation und Erklärung aus
    derselben Funktion.
  - **Oberfläche:** „ⓘ“ bei Marktpreis und „Wer kauft?“ im Produktmarkt und bei den
    Stückkosten im Werk; die Vorschau enthält dafür zusätzlich den Möbelmarkt.
  - Tests: Kern (Teile des Preises und der Nachfrage ergeben das Ganze) und Vorschau
    (alle drei Erklärungen).

Eigenständige Entscheidungen:

  - Die Erklärung steht in einem aufklappbaren Element statt in einem Tooltip beim
    Überfahren: So funktioniert sie auch mit der Tastatur und auf dem Handy.
  - Die Kaufneigung wird beim heutigen Marktpreis gezeigt; die Nachfrage des Monats
    wurde zu Monatsbeginn mit dem damaligen Preis gesetzt. Zu Monatsbeginn stimmen beide
    genau überein, im Monat können sie leicht abweichen.

### M28: Geschichte erzählen (05.10.2026)

Vorschlag 7 aus `docs/OFFENE_PUNKTE.md`, Abschnitt E. Regeln: `docs/FORMELN.md`, M28.

  - **Daten:** 82 Weltereignisse von 1940 bis 2026 (`data/ereignisse/1940_2026.yaml`) mit
    Titel und Beschreibung, neue Ereignisart „Wirtschaftspolitik“ (`reform`). Gesetzliche
    Umstellungskurse (`umrechnung`) an 137 Währungswechseln: alle Euro-Einführungen mit
    den amtlichen Kursen, Rentenmark, D-Mark, Schilling, Neuer Franc, die Rubel von 1961
    und 1998 und weitere Währungsschnitte nach Inflationen.
  - **Kern:** Meldung „Währungsumstellung“ am ersten Tag des Zeitraums, für das Land des
    Firmensitzes und alle Länder mit eigenem Standort; sie erscheint wie ein Weltereignis
    in einem eigenen Fenster und im Rundenbericht.
  - **Oberfläche:** Das Ereignisfenster zeigt bei Währungen den Umstellungskurs und den
    Hinweis, dass sich das Vermögen nicht ändert. Zahlen in Meldungen mit bis zu sechs
    gültigen Stellen (1,95583), sehr große in Worten (400 Quadrilliarden).
  - Tests: Kern (Meldung nur für Sitz- und Standortländer, Kurs aus Gesetz oder
    Wechselkursen), Echtdaten (Euro 1999 als Weltereignis und als Umstellung mit
    1,95583), Prüfregeln für `umrechnung`, Oberfläche (Fenster und Zahlenformat).

Eigenständige Entscheidungen:

  - Ohne gesetzlichen Kurs nennt die Meldung das Verhältnis der Wechselkurse; wo dieses
    deutlich vom bekannten Umstellungskurs abwich (z. B. Euro, D-Mark 1948, Zloty 1995),
    steht der gesetzliche Kurs jetzt in den Daten. Für die D-Mark gilt der Satz für
    Bargeld und Guthaben (10 RM : 1 DM); Löhne, Mieten und Preise wurden 1 : 1
    umgestellt (Kommentar in den Daten).
  - Gemeldet wird nur, wo der Spieler sitzt oder einen Standort hat; sonst kämen über
    das Jahrhundert mehrere hundert Meldungen zusammen.
  - Die Monatsgenauigkeit der Daten bestimmt den Tag der Meldung (D-Mark: 1. Juni 1948
    statt 20. Juni); der Text nennt deshalb nur Monat und Jahr.

### M29: Rang in der Übersicht (05.10.2026)

Vorschlag 8 aus `docs/OFFENE_PUNKTE.md`, Abschnitt E. Regeln: `docs/FORMELN.md`, M29.

  - **Kern:** Platz des Spielers unter allen aktiven Firmen nach Eigenkapital und nach
    Umsatz der letzten zwölf Monate (`ranking`). Die Plätze zu Spielbeginn und nach
    jedem Monatsende stehen im Spielstand (`standings`, die letzten 24; ältere
    Spielstände beginnen ohne). Zum Jahresende meldet der Rundenbericht unter
    „Wettbewerb“ die Plätze mit denen des Vorjahrs.
  - **Oberfläche:** In der Übersicht unter „Wettbewerb“ „Dein Rang unter n Firmen“ mit
    beiden Plätzen und der Veränderung gegenüber dem Vorjahr, dazu eine Erklärung (ⓘ).
  - Tests: Kern (Plätze, gleiche Werte, Insolvenz, Meldung zum Jahresende, Vergleich mit
    dem Vorjahr), Sicht, Vorschau.

Eigenständige Entscheidungen:

  - Umsatz über die letzten zwölf abgeschlossenen Monate statt des laufenden Jahres:
    Der Rang springt dann nicht jeden Januar.
  - Gleiche Werte teilen sich einen Platz; ohne Umsatz gibt es keinen Platz nach Umsatz.
  - Der Vergleich gilt dem Stand ein Jahr zuvor im selben Monat, nicht dem Jahresende:
    So zeigt er auch im Januar eine Veränderung über ein ganzes Jahr.


### M30: Kaufangebote I – Standorte, Labore und Lizenzen (05.10.2026)

Auftrag vom 05.10.2026 (Lastenheft §18.4, `docs/OFFENE_PUNKTE.md`, Abschnitt F).
Formeln: `docs/FORMELN.md`, M30; Parameter: `data/parameter/kaufmodell.yaml`.

  - **Befehle für alle Firmen:** Angebot für einen Standort (jede Art, auch Labore) oder
    eine Lizenz auf eine Technologie; Antwort mit Annehmen, Ablehnen oder einmal einem
    Gegenangebot; Rücknahme. Frist zwei Monate, danach zwölf Monate Sperre für denselben
    Käufer und Gegenstand; Standorte erst ab zwölf Monaten Alter.
  - **Bewertung:** Grundwert = höherer Wert aus Ertragswert (Ergebnis der letzten zwölf
    Monate × 5) und Restwert der Anlagen, dazu Anlagen im Bau und Lager. Lizenzwert =
    ersparte Forschung des Käufers. Die KI zahlt mehr für Wettbewerb im selben Markt,
    knappe Fachkräfte, eigenes Geschäft (Bauzeit) und einen teureren Neubau.
  - **Übergabe:** Der Standort wechselt mit Anlagen, Gebäude, Konzession, Lager,
    Belegschaft, Angeboten und Forschungsprojekt; beide Seiten buchen doppelt. Was über
    den Buchwerten gezahlt wird, ist Firmenwert (neues Konto, zehn Jahre abgeschrieben);
    Lizenzen buchen als eigene Kostenart. Forschungspunkte, Marke und Kredite bleiben
    bei der Firma.
  - **KI:** prüft monatlich mit 2–6 % Wahrscheinlichkeit ein Geschäft und wählt das mit
    dem größten Spielraum je Dollar; antwortet am Tag nach Eingang. Dem Spieler bieten
    alle KI-Firmen zusammen höchstens einmal im Monat etwas Neues an.
  - **Oberfläche:** Reiter „Wettbewerb“ (Taste 4) mit „Angebote“ (beantworten,
    Gegenangebot, zurückziehen, abgeschlossene des letzten Jahres) und „Firmen“ (alle
    Firmen, Standorte mit Grundwert und Neubaupreis, fehlende Technologien mit
    Lizenzwert). Offene Angebote zählen am Reiter und unter „Zu erledigen“; „bis zur
    nächsten Warnung“ hält bei einem neuen Angebot.
  - **Weltlauf 1900–1930** (Seed 1, 100 KI-Firmen): 324 Standortkäufe (222 Werke,
    98 Förderstandorte, 4 Labore), 16 Lizenzen, 8 Angebote an den passiven Spieler,
    4 Pleiten wie ohne Angebote. Gegen denselben Lauf ohne Angebote: Versorgung weltweit
    19 → 15 Verstöße, je Land 313 → 294, Preis gegen Richtpreis 140 → 148, Marge
    43 → 47; Kohle 1929 bei 1,04 × Richtpreis.
  - Tests: neun Szenariotests (Übergabe mit Buchungen und Firmenwert, Prüfregeln,
    Lizenz nach Gegenangebot, Antworten und Verfall, KI-Angebot an den Spieler, Hinweise
    und Sichten, Kraftwerke und Labore, die der Eigentümer braucht), KI-Test zu
    Kraftwerken im Bau, Sitzung, Prüfregeln des Kaufmodells, Vorschau (beantworten und
    bieten) und Browser-Version mit echtem Kern (Firmen und Standorte).

Eigenständige Entscheidungen:

  - Die KI wählt nach Spielraum je Dollar (H − P) / P statt nach dem größten Spielraum:
    Sonst gewannen stets große Werke, und an den Spieler mit seiner Werkstatt ging nie
    ein Angebot.
  - Der erste Weltlauf zeigte doppelten Kohlebedarf (Kohle 1929 bei 1,84 × Richtpreis):
    KI-Firmen verkauften Kraftwerke, deren Strom ihre Werke brauchten, und bauten neu –
    dabei kam während der Bauzeit von 360 Tagen jedes Quartal eine Einheit dazu (1 → 6),
    weil Kraftwerke im Bau nicht mitzählten. Behoben in der KI (Kraftwerke im Bau zählen
    gegen den Fehlbedarf, Formeln M10) und in den Kaufangeboten: Was der Eigentümer
    selbst braucht (Kraftwerk für die eigenen Werke im Land, einziges Labor, solange er
    forscht oder als KI forschen will), gibt er nur zum Neubaupreis ab. Kraftwerke kauft
    die KI nur, wo ihr Strom fehlt (Strom lässt sich nicht handeln), Labore nur, wenn
    sie keines hat. Vorher wechselten bis 1930 358 Labore den Besitzer, meist in der
    Pause zwischen zwei Forschungszielen.
  - Die Oberfläche zeigt bei fremden Standorten „Neubau heute“ und, wenn die Firma den
    Standort selbst braucht, einen Hinweis; der Preisvorschlag beginnt dann beim
    Neubaupreis. Den Mindestpreis der KI (Verkaufsaufschlag) zeigt sie nicht.
  - Mindestalter zwölf Monate, Firmenwert über zehn Jahre und die Antwort am Folgetag
    sind Annahmen (Parameter). Die KI-Antwort hängt nur vom Angebot und den eigenen
    Werten ab, nicht vom Zufall.
  - Knapp drei Viertel der KI-Angebote werden abgelehnt (Gebot 5–20 % über dem Grundwert,
    Mindestpreis 10–30 %, Kernstandorte 50 % mehr): ein normales Ergebnis von
    Verhandlungen, das nur eine Sperre für denselben Gegenstand hinterlässt.

### M31: Kaufangebote II – ganze Bereiche (05.10.2026)

Lastenheft §18.4 („alle Standorte einer Warengruppe mit der Markenbekanntheit“).
Formeln: `docs/FORMELN.md`, M31.

  - **Bereich:** je Firma und Warengruppe alle Förderstandorte, Werke, Lager und
    Verkaufsbüros, die Waren der Gruppe herstellen oder anbieten – jeweils ganz –, dazu
    die Bekanntheit der Marke in allen Ländern. Kraftwerke und Labore gehören zu keinem
    Bereich.
  - **Bewertung:** Grundwert = Grundwerte der Standorte + Markenwert; der Markenwert ist
    die Werbung, die dieselbe Bekanntheit aufbauen würde (Werbemodell M16). Höchstpreis
    und KI-Motive wie bei Standorten, über alle Standorte des Bereichs zusammen.
  - **Übergabe:** Der Preis wird nach den Grundwerten auf die Standorte verteilt, jeder
    Standort wie in M30 gebucht. Der Käufer übernimmt je Land die höhere Bekanntheit;
    der Verkäufer verliert Marke und Werbebudgets der Gruppe. Offene Angebote für die
    übergebenen Standorte und für Bereiche des Verkäufers verfallen.
  - **KI:** prüft Bereiche zusammen mit Standorten und Lizenzen (Spielraum je Dollar),
    nur solche mit mindestens zwei Standorten oder einer Marke. Sperren und offene
    Angebote gelten jetzt je Käufer, Verkäufer und Gegenstand.
  - **Oberfläche:** In der Firmenansicht stehen die Bereiche mit ihren Standorten, der
    Bekanntheit je Land, Grundwert (Standorte und Marke, ⓘ), Neubaupreis und Preisfeld;
    Angebote, Meldungen und Hinweise nennen Warengruppe und Zahl der Standorte.
  - **Weltlauf 1900–1930:** 15 Bereichskäufe (der größte mit 23 Standorten, Fahrzeuge),
    328 Standortkäufe, 16 Lizenzen. Gegen M30: Versorgung weltweit 15 → 18 Verstöße,
    je Land 294 → 309, Preis gegen Richtpreis 148 → 139, Marge 47 → 47. 7 statt 4
    Pleiten, davon 3 im Jahr 1925; keine der Firmen hatte einen Bereich gekauft oder
    verkauft (vor allem Stahlfirmen bei niedrigen Stahlpreisen). Die Grenzen der
    Prüfung „Pleiten“ werden eingehalten.
  - Tests: vier Szenariotests (Übergabe mit Preisaufteilung und Marke, Kernaufschlag für
    die ganze Erzeugung, KI bietet für den Bereich eines Wettbewerbers, Sichten und
    Hinweise), Vorschau (Bereich mit Marke, Grundwert, Angebot).

Eigenständige Entscheidungen:

  - Ein Standort geht ganz zu einem Bereich, auch wenn er Waren anderer Gruppen herstellt;
    Standorte lassen sich nicht teilen. Gehört ein Standort zu mehreren Bereichen, steht
    er in jedem.
  - Umfasst ein Bereich die ganze Erzeugung einer KI-Firma, verlangt sie den
    Kernaufschlag wie für ihren einzigen Standort mit Anlagen (M30), statt abzulehnen.
  - Bekanntheit wird bis 99 % bewertet (volle Bekanntheit würde unendlich viel Werbung
    kosten). Die Marke geht als Teil des Preises in den Firmenwert der Standorte ein,
    soweit der Preis die Buchwerte übersteigt; ein eigenes Markenkonto gibt es nicht.
  - Ein Bereich lässt sich bieten, sobald sein ältester Standort das Mindestalter hat.

### M32: Produkte 1915–1939 (05.10.2026)

Lastenheft §18.4, Abschnitt G der offenen Punkte („gleich schrittweise bis 2026“).
Formeln: `docs/FORMELN.md`, M32 (und M10, Forschung).

  - **Daten** (`data/ketten/14_*.yaml` bis `19_*.yaml`): 20 Produkte, 21 Rezepte und 15
    Technologien (erfunden 1874–1920) in sechs Ketten – Aluminium (Bauxit → Tonerde nach
    Bayer → Schmelzflusselektrolyse → Kochtopf), Kunstseide (Zellstoff → Viskose →
    Strümpfe), Elektrogeräte (Phenolharz, Radioröhre, Röhrenradio, Kleinmotor, Staubsauger,
    Kühlschrank), Nutzfahrzeuge (Lastwagen, Traktor), Luftfahrt (Flugmotor,
    Ganzmetall-Verkehrsflugzeug) und Stickstoff (Chilesalpeter, Ammoniak nach
    Haber-Bosch, Dünger auf beiden Wegen). Dazu elf Bauxit-Lagerstätten und die
    Salpeterfelder der Atacama. Alle Werte sind als Annäherung gekennzeichnet; die
    Margen zum Richtpreis liegen bei 5–45 %, die Produktbäume bei höchstens fünf Ebenen
    (Verkehrsflugzeug `sehr_komplex`).
  - **Nachfrage erst ab Verfügbarkeit:** Endkunden und Staat fragen ein Produkt erst nach,
    wenn es sich herstellen lässt; der Markt eines Landes listet nur solche Waren.
  - **KI für neue Produkte** (für alle Firmen gleich): Pioniere bauen, was gefragt ist,
    aber noch nirgends hergestellt wird; Marktlücken werden erforscht (höchstens zwei
    Firmen je Technologie, `forschung_luecke_firmen`); Anlagen im Bau zählen gegen die
    offene Nachfrage; die Engpass-Suche weicht auf das nächste knappe Vorprodukt aus;
    Werke für Vorprodukte entstehen, wo die Fracht am geringsten ist (Tonerde beim
    Bauxit in Frankreich); was eigene Standorte verbrauchen, zählt für die Auslastung.
  - **Weltlauf 1900–1940** (Seed 1, 100 KI-Firmen): Alle neuen Produkte werden
    hergestellt – Strümpfe und Staubsauger ab 1907/08, Kochtopf (über Bauxit und Tonerde
    aus Frankreich) ab 1912, Lastwagen 1913, Ammoniak und Traktor 1918, Kühlschrank 1919,
    Röhrenradio 1921, Verkehrsflugzeug 1925. Nach zwei bis vier Anlaufjahren sind Kochtopf, Radio,
    Kühlschrank, Staubsauger, Lastwagen, Traktor und Dünger zu 99–100 % versorgt,
    Verkehrsflugzeuge zu 48–86 %. Gegen den ersten M32-Stand: Versorgung weltweit 192 →
    65 Verstöße, je Land 4.681 → 1.378, Erzeugung ohne Vorprodukte 171 → 51, keine
    Pleitewelle (6 Pleiten bis 1939).
  - **1900–1930 gegen M31:** Die älteren Ketten laufen wie zuvor (Verstöße je Land bei
    den alten Produkten 322 statt 309, Gummi und Schnittholz wie bisher); dazu kommen
    die Anlaufjahre der neuen Produkte (weltweit 18 → 55, je Land 309 → 1.175, davon
    216 Verkehrsflugzeuge). Margen außerhalb −20 bis 50 %: 47 → 107, vor allem
    Aluminium und Tonerde (Verlust bei zu großen Hütten für den Anlauf) sowie Kleinmotor
    und Radioröhre (Pioniergewinne der ersten Jahre).
  - Tests: Nachfrage wartet auf die Technologie, Markt ohne unverfügbare Waren, sechs
    KI-Tests (Engpass ohne Wissen über das Endprodukt, Forschung für eine Marktlücke,
    Pionier mit Anlage im Bau, Standort nach Fracht für Vorprodukte und nicht für
    Staatsgüter, Verbrauch eigener Standorte als Absatz).

Eigenständige Entscheidungen:

  - Lastwagen, Traktoren, Verkehrsflugzeuge und Dünger kaufen in Stufe 1 die Staaten (je
    BIP); eigene Kunden wie Speditionen und Fluggesellschaften folgen mit späteren
    Stufen. Der Düngerbedarf entspricht 1913 (0,3 t je Mio. USD BIP), sein späterer
    Anstieg kommt mit den nächsten Epochen.
  - Aluminiumhütten stehen am eigenen Wasserkraftwerk (in der Investition). Die neuen
    Anlagen sind einzelne Linien (Tonerde 30 t, Hütte 10 t, Flugmotoren 0,5 Stück, Zellstoff
    50 t je Tag), damit sie zu den kleinen Märkten passen.
  - Die Standortwahl nach Fracht gilt nur für Vorprodukte: Mühlen nahe am Getreide, aber
    fern der Nachfrage, wurden sonst Quartal für Quartal neu gebaut (Händler bringen
    Mehl kaum in Länder mit niedrigem Preisniveau).
  - Bekannte Grenzen (offene Punkte, Abschnitt G): dünner Markt für Verkehrsflugzeuge,
    stillstehende Aluminiumhütten nach dem Anlauf, Chilesalpeter wird auch ohne Käufer
    weiter auf Mindestauslastung gefördert (Förderung legt die KI nicht still, M22).

### M33: Produkte 1940–1964 (06.10.2026)

Lastenheft §18.4, Abschnitt G der offenen Punkte. Formeln: `docs/FORMELN.md`, M33 (und
M10: Lagerstätten, Konzessionen, Forschung).

  - **Daten** (`data/ketten/20_*.yaml` bis `27_*.yaml`): 13 Produkte, 15 Rezepte und 12
    Technologien (erfunden 1934–1954) in acht Ketten – Kunststoffe (Ethylen aus dem
    Dampfspalter → Polyethylen → Kunststoffwaren im Spritzguss), Polyamid (aus Benzin oder
    aus Kohle, mit Ammoniak) für Strümpfe, Fernsehen (Bildröhre → Schwarzweiß-Fernseher,
    in Serie ab 1946), Waschmaschine, Transistor (Germanium → Transistor →
    Transistorradio, verdrängt das Röhrenradio), Düsenflugzeug (Strahltriebwerk →
    Düsenverkehrsflugzeug, verdrängt das Propellerflugzeug), Penicillin und
    Synthesekautschuk. Dazu drei Germanium-Lagerstätten (Tsumeb, Kipushi, Tri-State).
    Alle Werte sind als Annäherung gekennzeichnet; die Margen zum Richtpreis liegen bei
    5–45 %, die Produktbäume bei höchstens fünf Ebenen (Düsenflugzeug `sehr_komplex`).
  - **Verdrängung beim Staatsbedarf:** Ersetzt ein Produkt mit Staatsnachfrage ein
    anderes, sinkt der Staatsbedarf des alten über `verdraengung_staat_jahre` (15) auf
    null, ab dem Jahr, in dem sich das neue herstellen ließ.
  - **KI** (für alle Firmen gleich): Verfahrenswahl mit Knappheit (teure Vorprodukte
    zählen mit ihrem Aufpreis); Ausbau nach dem Absatz seit Beginn des Vormonats;
    **Einstieg in teure Märkte** (zahlen Käufer das 1,3-Fache des Richtpreises und
    stellen weniger als vier Firmen ein Produkt her, steigt eine weitere ein und
    erforscht dafür auch dessen Verfahren); Gruben bekommen so viele Anlagen, wie ihre
    Konzession beschäftigt; neue Konzessionen nur, wo der Restvorrat noch 10 Jahre
    reicht, an der Lagerstätte mit der billigsten Fracht; als Halde zählen nur die Lager
    der Verkäufer.
  - **Förderung übers Jahr verteilt:** Eine Konzession darf bis zu einem Tag höchstens
    den bisherigen Anteil ihrer Jahresmenge fördern.
  - **Prüfung der Daten:** Für die Ebenen des Produktbaums zählt bei mehreren Rezepten
    eines Vorprodukts der einfachste Weg; ein Rohstoff mit Abbau-Rezept ist die erste
    Ebene.
  - **Weltlauf 1900–1965** (Seed 1, 100 KI-Firmen): Alle neuen Produkte werden
    hergestellt; Fernseher, Waschmaschinen, Strümpfe, Gummi, Kunststoffwaren (ab 1954),
    Transistorradios und Penicillin (ab 1951) sind zu 100 % versorgt, Öl kostet
    durchgehend den Richtpreis (im ersten M33-Stand 1959–1964 das Drei- bis Vierfache,
    weil die Raffinerien mit vollen Eingangslagern den Mangel verdeckten und neue
    Ölfelder zwanzig Bohrtürme für zwei bekamen). Gegen den ersten M33-Stand:
    Mangeljahre 1940–1964 97 → 66, Versorgung weltweit 171 → 134 Verstöße, je Land
    3 351 → 3 098, Erzeugung ohne Vorprodukte 140 → 85, ohne Arbeitskräfte 111 → 64,
    Förderung gegen Bedarf 233 → 168. 1900–1939 wie bei M32 (65 → 67 Mangeljahre, davon
    zwei der neuen Waschmaschine).
  - Tests: Staatsbedarf weicht dem Nachfolger, Förderung über das Jahr verteilt, KI-Tests
    für teure Märkte (Einstieg und Forschung), Gruben passend zur Konzession und zum
    Vorrat, Lager nur von Verkäufern; Prüfregeln für die neuen Parameter und für den
    einfachsten Weg im Produktbaum.

Eigenständige Entscheidungen:

  - Einführungsjahre der Serienfertigung (Fernseher 1946 statt der ersten Sendungen
    1936); Penicillin als Grundbedarf, Kunststoffwaren als Sammelgut in kg; Germanium aus
    eigenen Lagerstätten (Zink gibt es noch nicht).
  - Teure Märkte: ab dem 1,3-Fachen des Richtpreises, bis vier Hersteller, ein Viertel
    des Absatzes für den Neuen (`kimodell`); Lagerstätten brauchen 10 Jahre Restvorrat.
  - Bekannte Grenzen (offene Punkte, Abschnitt G): Kupfererz ist ab 1954 knapp (das
    1,8- bis 2,4-Fache des Richtpreises, ein bis zwei Förderer), Kupferdraht für den
    Staat fällt dann zeitweise aus; Düsenflugzeuge baut bis 1964 nur eine Firma (zu
    10–30 % versorgt); Pleiten geben Konzessionen frei, die neu erschlossen werden müssen
    (Abschnitt K).

### M34: Regionen (06.10.2026)

Auftrag: kleine Länder zusammenfassen, Ziel um 100 Länder und Regionen (offene Punkte,
Abschnitt I).

- **Daten:** 197 Länder → 111 Länder und Regionen (`tools/daten/regionen.py`). Die
  Länderdateien, die Namen und die Karte erzeugen `tools/daten/laender.py` und
  `karte.py` aus denselben Quellen; Regionen tragen `umfasst` mit ihren Ländern, deren
  Namen als `teilland.<ISO>` und auf der Karte einen Umriss ohne innere Grenzen.
  Lagerstätten, reale Firmen, Ereignisse, Namensgruppen und Währungen verweisen auf die
  Region; 75 Währungen ohne Land entfallen.
- **Prüfung:** `umfasst` braucht gültige Codes, mindestens zwei Länder, keins davon ein
  eigenes Land oder in zwei Regionen, und für jedes einen Namen.
- **Spielstände:** Die Codes der aufgegangenen Länder lesen sich als ihre Region
  (`KeyTable::add_alias`); bei Werten je Land gilt der Eintrag der Region selbst bzw.
  des führenden Landes.
- **Oberfläche:** Das Länderdetail nennt die Länder einer Region („Umfasst“).
- Tests: Prüfregeln (`regionen_werden_geprueft`), echte Daten (111 Einträge, Belgien
  mit Luxemburg, Baltikum), Spielstände mit zusammengefassten Ländern (Kern), Vorrang
  des eigenen Eintrags beim Laden.
- **Weltlauf 1900–1965** (Seed 1, 100 KI-Firmen): 21 statt 34 Minuten Rechenzeit.
  Mangeljahre 1900–1939 wie bei M33 (67), 1940–1964 69 statt 66; Versorgung weltweit 136
  statt 134 Verstöße. Düsenflugzeuge kommen erst nach 1964 (bei M33 1964 mit 9 %
  Versorgung; bekannte Grenze, offene Punkte G).

### Mehr KI-Firmen (06.10.2026)

Wunsch: „Mehr KI-Gegner. Mache einen Simulationslauf mit 500 Gegnern.“

- **Erster Lauf mit 500 KI-Firmen** (1900–1930, vor den Änderungen): 17 Minuten, gut
  34 s je Spieljahr. Etablierte Märkte hatten fünfmal so viele Anbieter wie mit 100 Firmen,
  neue Märkte aber weiter einen: Kühlschrank und Radio kosteten das Doppelte des
  Richtpreises, 71 Mangeljahre (100 Firmen: 54).
- **Firmenzahlen wachsen mit:** Forscher je Marktlücke, Neugründungen je Monat,
  Diversifizierungen je Quartal und Hersteller teurer Märkte gelten für 100 Firmen und
  wachsen mit mehr Firmen im selben Verhältnis (`ai::per_companies`). Zweiter Lauf:
  45 Mangeljahre, Kühlschrank 2,8 statt 0,9 Hersteller zum 0,78-Fachen statt zum
  2,05-Fachen des Richtpreises, Versorgung weltweit 45 statt 71 Verstöße.
- **Tempo:** Markträumung mit Listen je Produkt und Land, Wege je Herkunftsland statt je
  Anbieter, Händlerauswahl über einen Haufen statt Sortieren, Anziehung der Angebote
  einmal je Schicht, Preisfaktoren einmal je Monat, Engpass-Suche mit einer
  Bestandsaufnahme für alle Firmen, bis gebaut wird. Bei 500 Firmen sinkt die Rechenzeit
  für 1900–1901 von 32 auf 22 Sekunden. Die Ergebnisse bleiben gleich (gleiche
  Zustands-Hashes bei 100 und 500 Firmen).
- **Weltlauf 1900–1990 mit 500 KI-Firmen** (Stand M39, Seed 1): 184 Minuten, im Mittel
  2 Minuten je Spieljahr (100 Firmen: 43 Minuten). Am Ende 582 Firmen, davon 82 pleite,
  9 724 Standorte. Gegen den Lauf mit 100 Firmen: weniger Versorgungslücken (weltweit
  158 statt 204 Verstöße, je Land 4 096 statt 5 440), mehr Preise außerhalb des 0,5- bis
  2-Fachen des Richtpreises (1 583 statt 1 262; mehr Wettbewerb drückt die Preise).
  Rohstoffe bleiben bis 1970 wie mit 100 Firmen beim 0,9- bis 1,1-Fachen; ab 1980 werden
  Holz (2,4-fach), Getreide (1,3- bis 1,7-fach) und Baumwolle (1,6-fach) knapp,
  Schnittholz ist 1989 nur zu 55 % versorgt – dieselbe Holzknappheit wie mit 100 Firmen,
  nur zehn Jahre früher. Das prüft der Weltlauf bis 2010 (M40).

### M35: Grundstücke (06.10.2026)

Auftrag: begrenzte, verschieden große Grundstücke je Land, kaufen oder pachten, ein
kleiner Wettbewerb um gute Grundstücke, volle Grundstücke zwingen zu einem neuen
Standort (offene Punkte, Abschnitt H; entschieden: Lage gleich mit).

- **Daten und Formeln:** `parameter/grundstuecksmodell.yaml` (Fläche je Mrd. USD BIP,
  vier Größenklassen nach Wohlstand, Lagen Stadt/Hafen/Land, Bodenpreis mit Knappheit,
  Pacht, Fläche der Anlagen); `anlagen[].flaeche_ha` für flächenhungrige Anlagen.
  Formeln: `docs/FORMELN.md`, Abschnitt M35. Prüfregeln mit Fehlerfall-Test.
- **Kern:** Grundstücke im Spielstand (`plots`), Angebot jedes Jahr am 1. Januar,
  Standort auf genau einem Grundstück (außer Förderstätten), Flächenprüfung beim Bauen,
  Befehle `FoundSiteOnPlot` und `BuyPlot`, Konto „Grundstücke“, monatliche Pacht, Lage
  wirkt auf Anwerben, Seefracht und Lieferkosten. Kaufangebote (M30) übergeben das
  Grundstück mit; der Grundwert enthält ein gekauftes Grundstück. Pleiten geben
  Grundstücke frei. Alte Spielstände bekommen gekaufte Grundstücke.
- **KI:** wählt das Grundstück mit den geringsten jährlichen Kosten (Pacht und
  Lieferkosten), das ihr Vorhaben mit Reserve fasst; kauft, wenn die Kasse es trägt;
  baut auf einem neuen Grundstück weiter, wenn das alte voll ist.
- **Oberfläche:** Gründen mit Grundstückswahl (Filter Lage und Größe, Kauf oder Pacht,
  Kosten), auch für Forschungszentren; Werksansicht mit Karte „Grundstück“ (belegte
  Fläche, Besitz, „Grundstück kaufen“); Bauen zeigt Fläche und „noch Platz für …“ und
  sperrt, was nicht passt; Länderdetail mit Gewerbeflächen; Grundwert nennt das
  Grundstück; Rundgang der Einführung mit „Standorte und Grundstücke“.
- Tests: acht Kerntests (`plots_tests`), Prüfregeln, Oberfläche (Gründen auf gewähltem
  Grundstück, Kauf, volle Grundstücke, Länderdetail, Einführung).
- **Weltläufe 1900–1965** (Seed 1, 100 KI-Firmen): Im ersten Lauf wurde fast jeder
  Ausbau ein eigener Standort (6 507 statt 2 143 Standorte 1965, 47 statt 21 Minuten
  Rechenzeit): Der volle älteste Standort blieb der beste Kandidat, neue Grundstücke
  fassten keinen weiteren Ausbau. Seitdem baut die KI auf freier Fläche eines anderen
  eigenen Standorts im Land weiter, und neue Grundstücke fassen das Dreifache des
  Vorhabens (`ki_reserve`). Zweiter Lauf: 23 Minuten, 12 Pleiten bis 1965 (M34: 9),
  Versorgung weltweit 209 Verstöße (M34: 136) – Ausbauten auf neuen Grundstücken
  brauchen länger; die Anlagengrößen (M36) gleichen das wieder aus.

### M36: Anlagen in fünf Größen (06.10.2026)

Auftrag: „verschiedene Größen von Werken … jeweils fünf Größen“ (offene Punkte,
Abschnitt H, Punkt 9).

- **Daten und Formeln:** `produktionsmodell.anlagengroessen` – Kapazität 0,25 / 0,5 / 1 /
  2 / 4 der Datenwerte (sehr klein bis sehr groß); Investition, Fläche und Bauzeit
  wachsen mit Kapazität^0,7 bzw. ^0,3, die Arbeitsstunden je Durchlauf sinken mit
  Kapazität^−0,15. Prüfregeln mit Fehlerfall-Test (alle Größen, wachsend, mittel = 1).
  Formeln: `docs/FORMELN.md`, Abschnitt M36.
- **Kern:** Größe je Anlage (`Slot::size`, Spielstände ohne Größe sind mittel),
  `BuildFacility` mit Größe; Kapazität, Arbeit, Kapitalkosten, Stückkosten, Fläche,
  Grundwert und Forscherzahl rechnen mit der Größe.
- **KI:** plant eine gewünschte Kapazität und wählt die größte Größe, die sie nicht
  übersteigt; bei knappem Geld oder Platz eine kleinere. Ausbau: kleine Werke
  verdoppeln sich, große wachsen um ein Viertel. Meldungen nennen die Größe.
- **Oberfläche:** Bauformular mit Größenwahl (Leistung, Preis, Bauzeit, Arbeit je
  Stück, Fläche, „noch Platz für …“, Hinweis auf den Tausch); die Anlagenkarte nennt
  die Größe.
- Tests: Größenmodell, große Anlage (Kosten, Leistung, Personal, Stückkosten),
  Spielstände und alte Befehle, Größenwahl der KI bei Geld- und Platzmangel.
- **Weltlauf 1900–1965** (Seed 1, 100 KI-Firmen): 22 Minuten, 2 412 Standorte, 11 Pleiten.
  Gegenüber dem zweiten M35-Lauf: Versorgung weltweit 147 statt 209 Verstöße, Erzeugung
  ohne Vorprodukte 68 statt 138, ohne Arbeitskräfte 25 statt 52 (M34: 91), Margen außerhalb
  −20 % bis 50 % 234 statt 363 (M34: 309). Billig bleiben Automobil, Fahrgestell, Blech
  und Glühlampe (unter dem halben Richtpreis).
- Nachtrag (mit M38): Balance-Protokoll und Weltbericht der Kommandozeile rechneten die
  Kapazität und den Bedarf an Vorprodukten noch ohne Anlagengröße; die Rohstoff-Prüfung
  dieses Laufs (Kohle 57 %, Bauxit 74 %) war dadurch verzerrt. Behoben.

### M38: Pleiten – Standorte werden versteigert (06.10.2026)

Auftrag aus dem Weltlauf (offene Punkte, Abschnitt K): Geht eine KI-Firma pleite, sollen
ihre Werke nicht einfach verschwinden.

- **Daten und Formeln:** `kaufmodell.insolvenz` (30 Tage Versteigerung, Mindestgebot der
  halbe Grundwert; 0 Tage = sofort aufgeben wie bisher). Formeln: `docs/FORMELN.md`,
  Abschnitt M38. Prüfregeln mit Fehlerfall-Test.
- **Kern:** Eine zahlungsunfähige KI-Firma ruht (keine Erzeugung, kein Personal, keine
  Angebote und Einkäufe); ihre Standorte werden versteigert. Gebote des Spielers über
  `MakeOffer` (mindestens das Mindestgebot, sonst Fehler), die KI bietet am Ende ihren
  Höchstpreis aus M30, soweit die Kasse reicht. Zuschlag zum zweithöchsten Gebot; die
  Übergabe bucht wie ein Kauf (Anlagen, Lager, Grundstück, Konzession). Ohne Gebot wird
  der Standort aufgegeben. Meldungen für Pleite, Zuschlag, Verlust und Verkauf.
- **Oberfläche:** Firmenliste mit „insolvent – Versteigerung bis …“, Firmenseite mit
  Erklärung, Mindestgebot je Standort und Knopf „Bieten“; fremde Standorte nennen die
  Größe ihrer Anlagen.
- Tests: Zuschlag zum zweiten Preis, Übernahme durch die KI zum Mindestgebot, Aufgabe ohne
  Gebot (Kern); Versteigerung in der Wettbewerbsansicht (Oberfläche).
- **Nachbesserung nach den Weltläufen (mit M37 committet):** Das Balance-Protokoll zählt
  jetzt versteigerte und aufgegebene Standorte und die Pleiten von Firmen, die vorher
  ersteigert hatten. Erster Vergleich mit und ohne Versteigerung (je zwei Seeds,
  1900–1958): mit Versteigerung 14 statt 9 bzw. 14 statt 7 Pleiten. Eine vorsichtigere
  Gebotsregel (die KI bietet wie bei Kaufangeboten und nur, wo sie im Land vertreten ist
  und einen Vorteil hat) änderte daran nichts (15 bzw. 20). Nur 3 der 20 Pleitefirmen
  hatten selbst ersteigert; die Ursache war die vorübergehende Lücke: Während der
  Versteigerung gründete sich sofort eine neue Firma in die stillstehende Nachfrage, nach
  dem Zuschlag lief beides. Jetzt zählt eine Firma in der Versteigerung bei Neugründungen
  noch mit, und ihre Werke gelten bei der Suche nach Nachfrage als kommende Versorgung.
  Ergebnis Seed 2: 12 statt 20 Pleiten (ohne Versteigerung 7), Versorgung weltweit 123
  statt 138 Verstöße, Erzeugung ohne Vorprodukte 74 statt 110; Seed 1 bis 1965: 14 Pleiten
  (ohne Versteigerung 11), Versorgung 128 statt 147 Verstöße, 34 Standorte versteigert,
  13 ohne Gebot aufgegeben.

### M37: Weiterentwicklung erforschter Produkte (06.10.2026)

Auftrag: „Wenn ein erforschtes Produkt weiter erforscht wird, dann soll das betreffende
Produkt weiterentwickelt und verbessert werden können. Das gilt auch für Vormaterialien
oder Halbzeuge.“ (Offene Punkte, Abschnitt J.)

- **Daten und Formeln:** `forschungsmodell.weiterentwicklung` (5 Stufen; je Stufe +4
  Qualität, −3 % Arbeit, −2 % Vorprodukte; Aufwand der Stufe n = B · 1,7^(n − 1) mit B =
  größter Forschungsaufwand der Technologien des Produkts, mindestens 10 000 Punkte;
  Gemeingut nach 15 Jahren; Fachgebiete je Branche für Produkte ohne Technologie),
  `kimodell.verhalten.entwicklung_*` (Nutzen 3 % des Umsatzes je Stufe und Jahr,
  Amortisation 5 Jahre). Formeln: `docs/FORMELN.md`, Abschnitt M37. Prüfregeln mit
  Fehlerfall-Test.
- **Kern:** Stufen und Punkte je Firma und Produkt, Projekt je Forschungszentrum
  (Technologie oder Produkt), Tag der ersten Erreichung je Stufe; Befehl `SetDevelopment`;
  die Stufe wirkt in Produktion (Arbeit, Vorprodukte, Qualität), Stückkosten, Personal,
  Vollkosten und der Planung der KI. Nachzügler forschen billiger, Stufen werden Gemeingut.
  Meldungen für erreichte Stufen und für Wettbewerber, die als erste eine Stufe erreichen.
- **KI:** Wer nichts zu erforschen hat, entwickelt sein umsatzstärkstes Produkt weiter,
  wenn sich die nächste Stufe in fünf Jahren bezahlt macht.
- **Oberfläche:** Forschung mit drittem Unterreiter „Weiterentwicklung“ (Karte je Produkt:
  Stufe, Wirkung, bester Wettbewerber, nächste Stufe mit Fortschritt, Dauer und Kosten,
  „Weiterentwickeln“); Forschungszentren wählen Technologie oder Produkt; Markt mit Spalte
  „Stufe“ je Anbieter; Anlagenkarte mit der Stufe; Rundgang mit „Produkte
  weiterentwickeln“. Außerdem: freie Grundstücke nach Kaufpreis sortiert (die günstigen
  zuerst).
- Tests: Aufwand, Gemeingut, Forschen Stufe für Stufe, Wirkung auf Erz, Arbeit, Qualität
  und Kosten, Befehl und Spielstand, Wahl der KI (Kern); Reiter und Projektwahl
  (Komponententest), Reiter und Marktspalte (Browser), Rundgang.
- **Weltläufe 1900–1965** (Seed 1, 100 KI-Firmen): Im ersten Lauf (Stufe 1 = 20 % von B,
  Wachstum 1,6) hatten die großen Firmen alle Stufen ihrer Produkte nach etwa fünf Jahren,
  bis 1930 war fast alles Gemeingut, und Preise unter dem halben Richtpreis kamen doppelt
  so oft vor (1 121 statt 522 Produktjahre). Mit Stufe 1 = B und Wachstum 1,7 verteilt
  sich die Entwicklung über Jahrzehnte: Kohle Stufe 5 seit 1932, Nägel 1944, Stahl Stufe 4
  seit 1927, Automobil Stufe 3 seit 1956, 1965 sind 47 Produkte weiterentwickelt. Gegenüber
  dem Lauf ohne Weiterentwicklung: 13 statt 14 Pleiten, Versorgung 147 statt 128 Verstöße,
  teurer als das Doppelte des Richtpreises 42 statt 72 Produktjahre, billiger als die
  Hälfte 717 statt 524 – die gesunkenen Kosten drücken die Preise unter den festen
  Richtpreis. 22 Minuten Rechenzeit wie zuvor.

### M39: Produkte 1965–1989 (06.10.2026)

Auftrag: Produktepochen bis 2026 (Lastenheft §18.4), hier 1965–1989.

- **Daten:** Ketten 28–34 (Halbleiter mit Quarz, Reinstsilizium und Mikrochip,
  Taschenrechner, Farbfernsehen, Mikrowelle, Personal Computer, Videorekorder,
  CD-Spieler), Quarz-Lagerstätten; Transistoren auch aus Silizium. Formeln:
  `docs/FORMELN.md`, Abschnitt M39.
- **Rohstoffe:** In den ersten Weltläufen bis 1990 (beide Seeds) waren nach 1965 alle
  Eisenerzgruben ausgelastet (Kapazität 1950 und 1975 gleich, Erz bis zum Vierfachen des
  Richtpreises), Kupfererz kostete das 1,5- bis 3,3-Fache bei nur zwei bis drei Förderern,
  und Kupferdraht für Staaten und Verbraucher war 1970–1978 kaum oder gar nicht versorgt.
  Dazu kommen die großen Funde nach 1950 (12 Eisenerz-, 13 Kupfererz-, 16 Ölfelder) mit
  Entdeckungsjahr und ein Förderindex für Eisen- und Kupfererz.
- **Staatsbedarf im Zeitverlauf** (`staatsnachfrage.verlauf`, neu): Stickstoffdünger folgt
  dem Verbrauch je BIP (1913 = 1, bis 1980 auf das Zehnfache).
- **KI:** Vorlauf für kommende Produkte (`forschung_vorlauf_jahre: 5`): Ohne ihn gab es
  Taschenrechner erst 1975 (Seed 1) bzw. bis 1976 gar nicht (Seed 2); Mikrochips wurden
  erst erforscht, als schon jemand Taschenrechner kaufen wollte.
- **Daten nachgestellt:** Siliziumwerk (500 kg je Tag, 30 Mio. USD) und Quarzbruch
  (5 t je Tag) passen zum kleinen Markt; vorher liefen sie mit 5 % Auslastung und Verlust.
- **Weltläufe:** Der passive Spieler (Werkstatt ohne Entscheidungen) ging 1976 bzw. 1979
  pleite, damit endete das Spiel und der Lauf. Lange Weltläufe starten ihn deshalb mit
  10 Mio. USD (`--kapital 10000000`).
- **Bedienung:** Markt mit Auswahl „Warengruppe“ (mit Zahl der Produkte) und Suchfeld;
  Technologiebaum mit Spalten je Produktepoche (vorher alles ab 1920 in einer Spalte);
  Suchfeld bei „Weiterentwicklung“ mit allen Produkten; Rundgang mit dem Schritt
  „Wettbewerb“ (21 Schritte); Flächen unter einem Hektar in m² („40 m²“ statt „0 ha“).
- **Browser-Version:** Der Test „Forschungszentrum gründen“ wählt jetzt ein Grundstück
  (seit M35 nötig; die CI meldete ihn seither rot, ohne abzubrechen).
- Tests: Staatsbedarf mit Verlauf (Kern), Prüfregeln für `verlauf` und
  `forschung_vorlauf_jahre`, Vorlauf der Forschung (KI), Marktfilter und Suche
  (Komponententests), Rundgang (Browser).
- **Weltläufe 1900–1990** mit allen Änderungen (100 KI-Firmen, 43 Minuten je Lauf):

  | | Seed 1 | Seed 2 |
  | --- | --- | --- |
  | Taschenrechner (1971): erste Ware / voll versorgt | 1973 / 1977 | 1972 / 1973 |
  | Farbfernseher (1965) | 1970 / 1971 | 1970 / 1972 |
  | Personal Computer (1977) | 1980 / 1983 | 1980 / 1983 |
  | Videorekorder (1976) | 1983 / 1983 | 1979 / 1982 |
  | CD-Spieler (1982) | 1986 / 1989 | 1986 / 1987 |
  | Pleiten bis 1990 (KI-Firmen gesamt) | 21 (121) | 20 (120) |

  Eisenerz bleibt bis 1989 beim 0,9- bis 1,1-Fachen des Richtpreises (die Gruben wachsen
  von 18 auf 77 Mio. t Kapazität), Kupfererz beim 0,9-Fachen, Kupferdraht ist durchweg
  versorgt (vorher 1970–1978 zu 0–32 %). Stickstoffdünger wächst von 3,6 Mio. t (1964)
  auf 17,6 Mio. t (1989) und bleibt versorgt. Erdöl kostet 1975–1980 vorübergehend das
  Zwei- bis 2,7-Fache, bis neue Felder fördern; Mikrochips 1980–1986 bis zum Dreifachen,
  als Personal Computer und Videorekorder zugleich anlaufen.
- **Bekannte Grenzen:** Neue Elektronik braucht nach dem Erfindungsjahr drei bis sechs
  Jahre bis zur Versorgung (Forschung, dann Chipwerk, Endmontage); Düsenflugzeuge bleiben
  bis 1985 knapp; Holz wird ab 1986 teuer (Seed 1: bis 2,8-fach, Schnittholz 1986 nur zu
  53 % versorgt) – das prüft der Weltlauf bis 2010 (M40). Siliziumwerk und Quarzbruch
  sind nach diesen Läufen verkleinert.

### M40: Produkte 1990–2009 (06.10.2026)

- **Daten** (Entwurf eines Hilfsagenten im Scratchpad, geprüft und übernommen): Ketten
  35–41 – Lithium-Ionen-Akku mit Lithium und Kobalt (10 bzw. 8 Lagerstätten),
  LCD-Panel in m² Bildfläche (neue Einheit `m2`), Mobiltelefon, Laptop, Digitalkamera,
  Flachbildfernseher (verdrängt den Farbfernseher), DVD-Spieler (verdrängt den
  Videorekorder). Formeln: `docs/FORMELN.md`, Abschnitt M40.
- **Nachbesserung M39:** Im Weltlauf des Agenten bis 2010 standen alle Chipwerke in den
  USA, wo Phenolharz (Chipgehäuse) das Vierfache kostete; nach der Ausbauregel baute keine
  Firma mehr aus, und Chips kosteten 1999–2009 das Vierfache – alle neuen Geräte
  hungerten. Das Gehäuse ist jetzt Glas und Keramik (Staatsmarkt); das Chipwerk bekommt
  10 ha statt 25 ha Fläche.
- **Oberfläche:** Beispieldaten der Vorschau mit den Daten bis 2009 erneuert; die
  Oberflächentests nennen die neuen Werte (Marge, Kaufangebot, Standortnummer).
- **Weltläufe 1900–2010** (100 KI-Firmen, je gut eine Stunde; am Ende 120 Firmen, davon
  20 pleite). Erste Ware / zu 90 % versorgt, Preis 2009 gegen den Richtpreis:

  | | Seed 1 | Seed 2 |
  | --- | --- | --- |
  | Mobiltelefon (1992) | 1997 / 2004, 2,4-fach | 1997 / 2001, 0,8-fach |
  | Laptop (1989) | 1998 / 2005, 1,2-fach | 1999 / 2003, 1,3-fach |
  | Digitalkamera (1991) | 1998 / 2004, 1,0-fach | 1997 / 2000, 0,8-fach |
  | Flachbildfernseher (2001) | 2004 / 2008, 2,7-fach | 2006 / 2009, 2,7-fach |
  | DVD-Spieler (1996) | 1998 / nie (2009: 30 %), 3,3-fach | 1997 / 1999, 0,7-fach |
  | LCD-Panel, Akku, Mikrochip 2009 | 3,0- / 2,3- / 1,0-fach | 3,9- / 1,7- / 0,9-fach |

- **Ursache der hohen Preise** (neues Diagnosekommando `wsim angebote <spielstand>
  <produkt>`: Angebote mit Preis, Preisboden, Auslastung, Lager und die Firmen, die das
  Verfahren kennen): 2009 kennen 25 Firmen die LCD-Technik, drei bauen Panels. In den USA
  kosten sie 11 000 USD je m², in China 3 000–5 800 USD bei halb leeren Werken. Händler
  beliefern nur Märkte mit offener Nachfrage; ein Land, das sich zu hohen Preisen selbst
  versorgt, bekommt keine billigere Ware. Vorschlag (Arbitragehandel) in
  `docs/OFFENE_PUNKTE.md`, Abschnitt G – erst nach Freigabe, weil er alle Märkte ändert.
- **Holz und Baumwolle:** Holz kostete ab 1997 das 2- bis 2,5-Fache, Schnittholz war 2009
  nur zu 42–67 % versorgt, Baumwolle bis zum 3,6-Fachen. Der Förderindex der Baumwolle
  folgt jetzt der Welternte (2010 das 6,9-Fache von 1900 statt 5,0); Holz bekommt ab 1980
  einen Zuschlag für Altpapier, Sägenebenprodukte und Plantagenholz (2000: 4,0 statt 3,4).
  Die Wirkung prüft der Weltlauf bis 2026 (M41).

### Keine echten Produktnamen (06.10.2026)

Auftrag: keine echten Produkte, Modelle oder Marken im Spiel, nur Gattungsbegriffe; eigene
Produktnamen je Firma folgen mit M42.

- **Weltereignisse:** „Ford Modell T“ heißt jetzt „Das Auto für alle“, „Fließband bei
  Ford“ „Fließband im Autobau“, „Boeing 747 im Liniendienst“ „Großraumflugzeuge im
  Liniendienst“, „IBM Personal Computer“ „Der Personal Computer wird Standard“, „Das
  iPhone“ „Das Smartphone“; auch die Texte nennen kein Produkt und keinen Hersteller mehr.
- **Produkte und Verfahren:** Bakelit → Phenolharz, Nylon → Polyamid (Technologie
  „Polyamidfaser“, Strümpfe aus Polyamid), Buna → Synthesekautschuk aus Butadien,
  Siemens-Verfahren → Gasphasenabscheidung, Compact Disc → Optische Speicherplatte (CD).
- **Kommentare in den Daten:** 76 Stellen nennen statt Modell und Hersteller nur Jahr und
  Sachverhalt („das erste Massenauto 1909: 850 USD“).
- **Bleibt (eigene Entscheidung):** Verfahren, die nach ihren Erfindern heißen
  (Siemens-Martin, Bayer, Haber-Bosch), Namen von Erfindern, die realen Firmen der
  Startbesetzung (Lastenheft §10) sowie Firmen und Schiffe in Weltereignissen (Lehman
  Brothers, Ever Given). Die IDs (`nylon`, `bakelit`, `ford_model_t` …) bleiben, damit
  Spielstände gültig bleiben; angezeigt werden sie nirgends.
