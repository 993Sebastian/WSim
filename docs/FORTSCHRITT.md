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
fielen diese Preise bis 1929 ebenfalls); Auto 0,32 mit Fließband (Ford: Model T 1909 bis
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
