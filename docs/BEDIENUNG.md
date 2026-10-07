# Bedienung – Prüfung und Zielbild (M18, 05.10.2026)

Prüfung aller Ansichten einer echten Partie (Werkstatt in Deutschland, 100 KI-Firmen,
zwei Monate gespielt) im Desktop- und iPhone-Format. Leitfrage je Stelle: **Was soll der
Spieler hier erkennen, und welche Entscheidung trifft er damit?** Daraus folgt das
Zielbild, das M18 bis M20 umsetzen.

Umsetzungsstand: Standorte und Werksansicht mit Personal und Preisen (M18b/c) sowie
Kopfbereich, Übersicht, Markt, Finanzen, Rundenbericht, Neues Spiel, Weltkarte und
Spielstände als Datei (M18d), der Technologiebaum (M19) und die geführte Einführung
bis zum ersten Verkauf (M20) sind umgesetzt.

## Erklärungen (M27)

Ein „ⓘ“ neben einem Wert zeigt, woraus er entsteht – als aufklappbare Rechnung, mit der
Tastatur und auf dem Handy (dort unten am Bildschirm) bedienbar:

- **Marktpreis** im Produktmarkt: Richtpreis, Preisniveau des Landes, Marktlage und die
  Lage im Vormonat.
- **Wer kauft?** im Produktmarkt: Einkommen, Kaufneigung, Bedarf oder Zielbesitz und
  Nachfrage je Kopf für jedes Einkommensfünftel.
- **Stückkosten** im Werk unter „Verkauf“: Material, Personal, Energie, Gemeinkosten,
  Pacht und Anlage.

## Hinweise und Kennzeichen (M18d)

Die Übersicht und der Rundenbericht zeigen unter „Zu erledigen“ bzw. „Jetzt zu tun“
Hinweise aus dem Kern (`views::hints`), jeweils mit Sprung an die Stelle, an der der
Spieler handeln kann (Werksansicht mit passendem Bereich, Forschung, Finanzen):
stillstehende Anlagen mit Ursache, Vorprodukte ohne Einkauf mit weniger als 7 Tagen
Reichweite, hergestellte, aber nicht angebotene Produkte, Verkauf unter Stückkosten,
Ware ohne Absatz im Vormonat, fehlendes Personal, Forschungszentrum ohne Projekt,
überzogenes Konto und eine Kasse, die bei einem Verlust wie im Vormonat keine 12 Monate
reicht. Die Märkte tragen Kennzeichen für Chancen: **Mangel** (weniger als 90 % der
Nachfrage von Verbrauchern und Staat bedient oder mehr als 10 % der Nachfrage ohne Ware),
**teuer** (Preis mindestens 20 % über dem Richtpreis) und **wenige Anbieter** (höchstens
zwei, und der Markt ist nicht gesättigt oder teurer als der Richtpreis). Die Schwellen
sind Anzeigehilfen und wirken nicht auf die Simulation.

## Grundsätze

1. **Zustand, Problem, Handlung:** Jede Ansicht beantwortet oben, wie es steht, was nicht
   läuft und was der Spieler jetzt tun kann. Warnungen führen per Klick zur Stelle.
2. **Entscheidungswissen neben dem Bedienelement:** Neben dem Preis stehen Marktpreis,
   Stückkosten und Marge; neben „Bauen“ stehen Leistung, Arbeitskräfte, Kosten und
   Amortisation; neben dem Einkauf die Reichweite.
3. **Jedes Feld hat eine sichtbare Beschriftung**, die zu ihm gehört (nicht nur ein
   Platzhalter, keine Sammelüberschrift über mehreren Feldern), und **jede Zahl ihre
   Einheit** (t, Stück, USD/t, %, Tage).
4. **Rückmeldung am Ort der Handlung:** Erfolg oder Fehler erscheint beim Knopf, nicht
   oben auf der Seite.
5. **Deutsche Zahleneingabe:** „1.800“ ist 1800 und „1,5“ ist 1,5; Felder zeigen ihren
   Wert formatiert und setzen schon beim Tippen die Tausenderpunkte („200.000.000“). Ein
   getippter Punkt trennt Tausender; nur nach einer Null („0.5“) und in Feldern ohne
   Tausenderpunkte (Prozent, Jahr, Zufallswert) gilt er als Komma. Eingefügter Text wird
   als Ganzes gelesen („1.5“ ist 1,5). Felder für ganze Zahlen nehmen kein Komma.
6. **Übersicht vor Detail:** Listen zeigen das Wichtigste je Zeile; ein Klick öffnet die
   Detailansicht (Werk, Produktmarkt, Land, Technologie).
7. **Handy:** Karten statt breiter Tabellen, Eingaben untereinander, ein kompakter
   Kopfbereich; nichts verlässt die Bildschirmbreite.

## Befunde und Zielbild je Ansicht

### Kopfbereich

- Ist: Firma, Datum, Kasse; Rundenlänge, „Runde beenden“; Speichern, Laden, Hauptmenü;
  darunter die Reiter. Auf dem Handy vier Zeilen hoch. Die Reiter waren beim Wechsel aus
  dem Startdialog halb verdeckt (behoben in M17).
- Ziel: eine Zeile mit Firma, Datum, Kasse (mit Trendpfeil zum Vormonat) und „Runde
  beenden“ mit Längenwahl; Speichern, Laden, Hauptmenü und Tastenhilfe in einem Menü
  „☰“. Warnungen (z. B. „Draht fehlt im Werk Deutschland“) als Zähler am Reiter
  Übersicht.
- Geld (M21): Im Menü „☰“ stehen zwei Wahlen für alle Beträge: **Währung der Beträge**
  (Landeswährung des Firmensitzes oder US-Dollar) und **Preise** (Kaufkraft 2026 ohne
  Inflation oder Preise der Zeit mit Inflation). Voreinstellung: Landeswährung,
  Kaufkraft 2026 – in Deutschland also Euro; mit „Preise der Zeit“ erscheinen Mark,
  Reichsmark, D-Mark und Euro, wie sie zum Spieldatum galten. Unter den Kennzahlen der
  Übersicht steht, worin die Beträge gerade stehen. Eingabefelder für Geld tragen das
  Zeichen der gezeigten Währung („€/t“) und rechnen die Eingabe zurück.

### Übersicht

- Ist: Kennzahlkacheln, Standorte mit Anlagen, Wettbewerber nach Eigenkapital.
  „0,7 Beschäftigte“ (Bruchteile von Personen), „Lager: Draht 5“ ohne Einheit. Was der
  Spieler als Nächstes tun sollte, steht nirgends.
- Ziel: oben **Zu erledigen** (Warnungen mit Sprung: Vorprodukt fehlt, Lager voll,
  Anlage steht, Kasse knapp, Forschung fertig), dann Kennzahlen mit Verlauf (kleine
  Linien für Kasse, Umsatz, Ergebnis), dann Standorte als Karten (Auslastung, Umsatz,
  Ergebnis, Engpass) mit Sprung zur Werksansicht. Wettbewerber auf eine eigene
  Unterseite „Wettbewerb“ mit Marktanteilen statt nur Eigenkapital.
- **Etappen** (M23): unter „Zu erledigen“ die nächste Etappe mit Fortschrittsbalken
  (z. B. „1 von 2“ Anlagen, „12 % von 30 %“ Marktanteil) und „So geht's“; alle Etappen
  mit Datum der erreichten aufklappbar. „Etappen ausblenden“ blendet den Bereich aus,
  das Menü ☰ („Etappen zeigen“) wieder ein; die Wahl merkt sich der Browser. Erreichte
  Etappen stehen im Rundenbericht unter „Erreicht“; die Einführung zeigt den Bereich
  nach dem ersten Verkauf.
- **Rang** (M29): Unter „Wettbewerb“ steht der Platz der eigenen Firma unter allen Firmen,
  die nicht insolvent sind – nach Eigenkapital und nach Umsatz der letzten zwölf Monate,
  mit der Veränderung gegenüber dem Vorjahr („↑ 5 gegenüber dem Vorjahr (Platz 12)“).
  Zum Jahresende nennt der Rundenbericht beide Plätze.

### Produktion → Standorte und Werksansicht

- Ist: Alle Standorte untereinander in einer langen Seite, je Standort drei Tabellen
  (Anlagen, Vorprodukte, Verkauf) mit Eingabefeldern in Tabellenzellen. Mängel:
  - „Einkauf (Ziellager, Höchstpreis USD)“ steht über zwei Feldern ohne eigene
    Beschriftung; „Preisregel (USD)“ über Auswahl und Feld mit Platzhalter
    „Untergrenze“.
  - **Preis:** Im Modus „Marktpreis“ ist das einzige Feld die Untergrenze. Wer dort
    einen niedrigeren Preis einträgt, um zu unterbieten, ändert nichts – der Preis bleibt
    über der Untergrenze, wo er war. Die Rückmeldung erscheint oben außer Sicht, und
    „1.800“ wird als 1,8 gelesen. Das erklärt „Preisanpassungen gehen teilweise nicht“.
  - Keine Stückkosten, keine Marge, kein Vergleich mit dem Marktpreis – die wichtigste
    Grundlage der Preisentscheidung fehlt.
  - „Anlage bauen“ zeigt nur Preis und Bauzeit, nicht Leistung, Personalbedarf,
    Vorprodukte und Wirtschaftlichkeit.
  - Personal: nur „0,7 Beschäftigte“; Arbeitskräfte lassen sich weder einsehen noch
    beeinflussen.
  - Eigenes Vorprodukt (Draht) steht als Verkaufszeile da – verwirrend.
  - Handy: Tabellen mit Feldern laufen seitlich aus dem Bild.
- Ziel: Reiter **Standorte** mit einer Karte je Standort (Art, Land, Auslastung, Umsatz,
  Ergebnis des Vormonats, Engpass) und „Neuer Standort“. Ein Klick öffnet die
  **Werksansicht** mit Unterbereichen:
  - **Anlagen:** je Anlage Rezept (Eingänge → Ausgang je Tag), Auslastung (Regler),
    Ist/Plan, Engpass mit Ursache in Worten, Zustand; „Anlage bauen“ als Auswahl mit
    Leistung, Personal, Investition, Bauzeit.
  - **Stilllegen oder verkaufen** (M22): aufklappbar unter jeder fertigen Anlage. Zeigt
    Wartung im Betrieb und stillgelegt, Restbuchwert und Verkaufserlös; die Zahl der
    Einheiten ist wählbar. Verkaufen fragt nach, bei einem Erlös unter dem Restbuchwert
    mit dem Verlust. Eine stillgelegte Anlage zeigt „stillgelegt seit …“ und „Wieder
    anfahren“ mit Kosten und Dauer; die Übersicht zählt stillgelegte Anlagen.
  - **Einkauf:** je Vorprodukt Bedarf/Tag, Lager, Reichweite (Ampel), Marktpreis,
    Ziellager (Tage oder Menge) und Höchstpreis mit eigener Beschriftung.
  - **Verkauf:** je Erzeugnis Lager, Absatz Vormonat, **Stückkosten**, eigener Preis,
    Marktpreis, Marge; Preisart „Automatisch (folgt dem Absatz)“ mit Mindestpreis oder
    „Fester Preis“; Schnellknöpfe −5 %/+5 %; Rückmeldung am Knopf.
  - **Personal:** je Gruppe Bedarf, Beschäftigte, freie Kräfte im Land, Lohn je Stunde;
    **Lohnaufschlag** des Standorts mit Wirkung („bekommt bei Knappheit zuerst
    Arbeitskräfte“) und Kosten. Darunter **Schulung** (W1, ⓘ: Niveau steigt Monat für
    Monat zum Ziel, sinkt ohne Schulung langsam; Kosten nach dem Ziel, Wirkung nach dem
    Niveau): erreichtes Niveau, Schulungsziel mit Herkunft („eigenes Ziel des Standorts“,
    „aus der Strategie“, „kein Ziel“), Schulungskosten je Tag und „Wirkung jetzt“ (weniger
    Arbeit, Qualitätspunkte); Feld „Schulungsziel“ in %, „Übernehmen“ und – bei eigenem
    Ziel – „Ziel der Strategie folgen“.
  - **Kosten:** Stückkosten je Erzeugnis aufgeschlüsselt (Material, Personal, Energie,
    Anlage, Verwaltung und Vertrieb, Pacht) und Ergebnis des Vormonats.
- **Grundstücke (M35):** „Neuer Standort“ (auch das Forschungszentrum) zeigt unter Land
  und Art die freien Grundstücke des Landes: Gewerbefläche frei/gesamt, Bodenpreis je
  Lage, Filter nach Lage und Größe, eine Liste (Lage, Größe, Fläche, Kaufpreis, Pacht je
  Jahr; ohne Filter die vier größten je Lage) und „Kaufen oder pachten“. „Gründen“ geht
  erst mit gewähltem Grundstück; daneben stehen die Kosten jetzt. Was die Lagen bewirken,
  steht aufklappbar darüber. Förderstätten brauchen kein Grundstück.
  In der Werksansicht steht unter den Anlagen die Karte **Grundstück** mit Lage (und
  ihrer Wirkung), belegter Fläche als Balken, Eigentum oder Pacht und „Grundstück
  kaufen“. „Anlage bauen“ nennt die Fläche und wie viele Anlagen noch passen; Anlagen,
  die nicht mehr passen, sind in der Auswahl markiert, und „Bauen“ bleibt dann aus –
  mit dem Hinweis, einen weiteren Standort zu gründen.
  Flächen unter einem Hektar stehen in Quadratmetern („40 m²“ statt „0 ha“).
- **Anlagengrößen (M36):** „Anlage bauen“ hat neben Anlage und Anzahl die **Größe**
  (sehr klein bis sehr groß, je mit Leistungsfaktor und Preis; vorgewählt: mittel). Die
  Werte darunter gelten für die gewählte Größe: Investition, Bauzeit, „Arbeit je Stück“
  gegenüber mittel, Fläche mit „noch Platz für …“ und Leistung je Tag. Größen, die nicht
  mehr aufs Grundstück passen, sind markiert. Ein kurzer Hinweis erklärt den Tausch:
  große Anlagen sparen je Stück, brauchen aber Kapital, Fläche und Absatz. Die Karte
  einer Anlage nennt ihre Größe, wenn sie nicht mittel ist.

### Markt

- Ist: eine Tabelle mit 46 Produkten und zehn Spalten; Mengen ohne Einheit;
  „Verkauft“ größer als „Nachfrage“ (Ausfuhr eingerechnet) ohne Erklärung; Hinweis
  „Mengen des Vormonats“ erst unter der Tabelle; Marktführer abgeschnitten. Marke und
  Werbung darunter.
- Ziel: Unterreiter **Produkte** und **Marke und Werbung**. Die Produktliste zeigt je
  Zeile Preis gegen Richtpreis (Balken), Nachfrage und Deckung (Ampel: Mangel/Überfluss),
  Anbieter, Marktführer mit Anteil, eigener Anteil; Filter „nur eigene“, „Chancen“
  (Mangel, Preis über Richtpreis, wenige Anbieter). Ein Klick öffnet den
  **Produktmarkt**: Preisverlauf, Anbieter mit Preisen und Anteilen, Nachfrage nach
  Einkommensschicht, Einfuhr und Ausfuhr, eigenes Angebot.
- **Verlauf** (M24): Der Produktmarkt zeigt die letzten 24 Monate – bezahlter Preis mit
  dem Richtpreis als gestrichelter Linie, Absatz je Monat und den eigenen Anteil.
  Der Rundenbericht nennt unter „Wettbewerb“ neue und ausgeschiedene Anbieter in den
  eigenen Märkten und Preissenkungen ab 10 %; „Ansehen“ führt zum Markt.
- **Produktionsketten** (M25): dritter Unterreiter im Markt. Wählbar ist die Spitze einer
  Kette (zuerst die eigene, z. B. Nägel); der Baum zeigt je Stufe Menge je Einheit, Anlage,
  Stückkosten gegen Marktpreis im Land des Firmensitzes, Marge, fehlende Technologien und
  Kennzeichen „stellst du her“, „kaufst du ein“, „verkaufst du“, „hakt: …“. Die ersten
  zwei Stufen sind offen, tiefere klappen auf; ein Klick auf ein Produkt öffnet seinen
  Markt.
- **Warengruppe und Suche** (nach M38): Mit den Epochen bis 1989 wächst die Liste auf
  über 90 Produkte. Eine Auswahl „Warengruppe“ (mit der Zahl der Produkte je Gruppe) und
  ein Suchfeld „Produkt suchen“ grenzen sie ein; beides wirkt zusammen mit „Zeigen“ und
  der Sortierung.
- **Produktnamen** (M42): Bei Endprodukten (Geräte, Fahrzeuge, Kleidung, Lebensmittel …)
  zeigt die Anbieterliste den Namen, unter dem jede Firma verkauft („Kelvor M80“). Darunter
  „Dein Produktname“: das Feld mit dem ersten von drei Vorschlägen aus den Namensbausteinen,
  die anderen zwei als Knöpfe, „Namen speichern“ und „Namen entfernen“. Echte Produkt- und
  Markennamen und Namen, die eine andere Firma für dasselbe Produkt trägt, lehnt das Spiel
  mit einer Meldung ab. Ein verkauftes Endprodukt ohne Namen steht unter „Zu erledigen“.
  Nachfolgemodelle (B1): Erreicht ein benanntes Modell eine neue Entwicklungsstufe, schlägt
  der Rundenbericht das Nachfolgemodell vor („Aus Kelvor M80 könnte Kelvor M100 werden“);
  es steht dann als erster Vorschlag unter „Dein Produktname“. KI-Firmen benennen um.

### Wettbewerb (M30, M31)

Reiter „Wettbewerb“ (Taste 4) mit zwei Bereichen:

- **Angebote:** Kaufangebote und Lizenzanfragen anderer Firmen mit Preis, Grundwert (ⓘ:
  Ertragswert, Restwert, Lager, Buchwert) und Frist; „Annehmen“, „Ablehnen“ oder ein
  Gegenangebot mit eigenem Preis. Eigene Angebote warten auf die Antwort und lassen
  sich zurückziehen. Darunter die abgeschlossenen Angebote des letzten Jahres.
  Wartende Angebote stehen auch unter „Zu erledigen“ und als Zähler am Reiter; „bis zur
  nächsten Warnung“ hält bei einem neuen Angebot an.
- **Firmen:** alle Firmen mit Sitz, Eigenkapital, Umsatz und Zahl der Standorte. Eine
  Firma öffnet ihre Standorte (Produkte, Anlagen, Beschäftigte, Grundwert, „Neubau
  heute“) mit einem Preisfeld für ein eigenes Angebot und die Technologien, die dir
  fehlen, mit der ersparten Forschung als Anhalt für eine Lizenzanfrage. Braucht die
  Firma einen Standort selbst – den Strom eines Kraftwerks für ihre Werke im Land oder
  ihr einziges Labor –, steht das dabei: Unter dem Neubaupreis verkauft sie ihn nicht,
  und der Preisvorschlag beginnt dort.
- **Versteigerungen (M38):** Zahlungsunfähige Firmen bleiben in der Firmenliste, solange
  ihre Standorte versteigert werden („insolvent – Versteigerung bis …“). Ihre Seite
  erklärt den Ablauf; jeder Standort zeigt das Mindestgebot, das Preisfeld schlägt es vor,
  der Knopf heißt „Bieten“. Der Rundenbericht meldet Zuschlag oder Verlust.
- **Produkte unter eigenem Namen (M42):** Die Seite einer Firma nennt über ihren
  Standorten die Namen ihrer Produkte („Kelvor M80 – Kühlschrank“); der Rundenbericht
  nennt sie in den Meldungen über neue Anbieter, Preissenkungen und Rückzüge.
- **Bereiche (M31):** Unter den Standorten einer Firma stehen ihre Bereiche – je
  Warengruppe alle Standorte, die Waren der Gruppe herstellen oder anbieten, mit der
  Bekanntheit der Marke je Land, Grundwert (Standorte und Marke, ⓘ), Neubaupreis und
  Preisfeld. Angebote für Bereiche erscheinen wie die für Standorte unter „Angebote“.
- **Zentrale der Konkurrenz (ZA4):** Unter dem Kopf einer Firma steht ihre Zentrale
  („Zentrale: Finanzen, Marketing (3 Angestellte).“ oder „Keine Zentralabteilungen.“) und
  ein laufender Umzug des Hauptsitzes („Verlegt den Hauptsitz nach … (ab …)“). Der
  Rundenbericht meldet, wenn eine Firma ihren Sitz verlegt.
- **Führung und Abwerben (B1):** Unten auf der Seite einer Firma stehen ihre Manager
  (Vorstand zuerst) mit Stelle und Fähigkeiten (ⓘ). Je Manager wählst du „Für die
  Stelle“ eine deiner freien Stellen; daneben steht das Angebot im Jahr (rot „über der
  Kasse“, wenn es deine Kasse übersteigt). „Abwerben“ schickt das Angebot; die Firma
  antwortet am nächsten Tag, der Rundenbericht meldet Wechsel oder Gegenangebot. Läuft
  schon ein Angebot oder wurde der Manager eben umworben, steht dort, bis wann.

### Organisation (MA1–MA6)

Reiter „Organisation“ (Taste 7; Weltkarte jetzt 8, Berichte 9). Frage: „Wer erledigt was,
und was bleibt bei mir?“ Vier Unterreiter: „Stellen“, „Anliegen“ (mit Zähler der
offenen Anliegen; solange welche offen sind, öffnet die Ansicht dort), „Strategie“ und
„Rücksprache“. Der Reiter „Organisation“ trägt denselben Zähler.

- **Organigramm:** Kopfzeile mit Zahl der Manager, Gehältern je Jahr und Bewerbern (ⓘ:
  wie Stellen arbeiten). Darunter Kontinent → Land → Standort; je Standort eine Karte mit
  seinen Stellen (Leitung, z. B. „Werksleitung“, und Fachstellen), dem Inhaber
  (Schwerpunkt, Fachkompetenz der Stelle, Erkennen, seit wann; alle Fähigkeiten hinter
  ⓘ), den Themen, die die Stelle erledigt, und dem Gehalt. Die Leitung zeigt die Themen
  leerer Fachstellen, die sie mit übernimmt; „Entscheidest du selbst: …“ nennt, was
  niemand erledigt; „Nächste Prüfung“ den nächsten Prüftermin.
- **Besetzen:** öffnet den Managermarkt für die Stelle – Bewerber des eigenen Kontinents
  zuerst (Auswahl „Bewerber aus“: andere Kontinente oder alle), mit Schwerpunkt,
  Fachkompetenz der Stelle, Erkennen, Urteilsvermögen (bei Leitungen auch Führung) und
  der Gehaltsforderung für genau diese Stelle; „Einstellen“. Darunter eigene Manager mit
  „Hierher versetzen“ (Gehalt danach). Fähigkeiten erscheinen nur als Stufen von
  „schwach“ bis „herausragend“ – als Einschätzung, die um eine Stufe danebenliegen kann.
  Forderungen über der Kasse tragen „über der Kasse“, darüber steht dann eine Warnung
  mit dem Kassenstand (B1); ein Hinweis nennt das Abwerben unter „Wettbewerb“.
- **Entlassen:** fragt nach und nennt die Abfindung.
- **Zufriedenheit (MA6):** In der Spalte „Gehalt im Jahr“ unter dem Gehalt die
  Zufriedenheit als Stufe („zufrieden“ grün, „gemischt“ gelb, „unzufrieden“ rot) und, wenn
  eine andere Firma ihn abwerben will, „Angebot von Firma: Betrag – Antwort bis Datum“. In
  der aufklappbaren Zeile der Stelle der Abschnitt „Gehalt und Zufriedenheit“: Gehalt,
  Marktwert (seine Forderung heute), Zufriedenheit, Erklärung (Gehalt unter Marktwert,
  Verlust der Einheit, übergangene Empfehlungen; wer sehr unzufrieden ist, kündigt), Feld
  „Neues Gehalt im Jahr“ (vorbelegt mit dem Marktwert) und „Gehalt anpassen“ – Gehälter
  lassen sich nur erhöhen. Bei Leitungen der Schalter „Besetzt freie Fachstellen selbst“:
  Die Leitung stellt je Monat höchstens eine Fachstelle mit Aufgaben aus dem Managermarkt
  ein, im Rahmen ihres Jahresbudgets; die Meldung im Rundenbericht nennt Name und Gehalt.
- **Hauptsitz (ZA1):** Unter dem Vorstand die Überschrift „Hauptsitz und Zentrale“ mit der
  Karte „Hauptsitz: Deutschland“ (ⓘ: was der Sitz bewirkt): Gewinnsteuer und Lohnniveau der
  Manager im Sitzland; Formular „Hauptsitz verlegen“ mit Auswahl „Neues Land“ (je Land
  Steuer und Lohn, nach Namen sortiert), Kosten, Dauer und Anteil der Angestellten, der
  mitzieht. Während des Umzugs steht dort „Umzug nach … läuft – der neue Sitz gilt zum
  Monatsanfang ab …“; der Rundenbericht meldet den Abschluss und wie viele Angestellte nicht
  mitgezogen sind.
- **Stadt des Hauptsitzes (W2):** Die Karte heißt „Hauptsitz: Berlin, Deutschland“. Wer ein
  neues Land wählt, wählt darunter die „Stadt“ (vorgewählt die Hauptstadt). Darunter die
  Tabelle „Städte in Deutschland“ (ⓘ: große Stadt = mehr Akademiker, aber teurere Büros; alle
  Firmen einer Stadt teilen sich deren Akademiker) mit Einwohnern, „Akademiker für
  Zentralen“, „Gewünscht (alle Firmen)“ (rot, wenn mehr gewünscht als vorhanden), Büro je
  Angestelltem und Monat und „Nach … ziehen“ bzw. „Sitz“; darunter Kosten und Dauer eines
  Umzugs im Land. In den Zentralabteilungen steht unter dem Zahlenfeld „nur n besetzt – zu
  wenige Akademiker in der Stadt“, wenn die Stadt nicht alle Stellen füllt.
- **Zentralabteilungen (ZA2):** Darunter die Karte „Zentralabteilungen“ (ⓘ: was jede tut,
  dass sie nur mit Leitung arbeitet und auch ohne Arbeit kostet, wie die Abdeckung die
  Genauigkeit der Leitung hebt) mit der Summe „n Angestellte kosten zusammen … im Monat“ und
  einer Tabelle je Abteilung: Leitung (Name und Fachkompetenz als Stufe, sonst „Leitung frei
  – im Vorstand „…“ besetzen“, ggf. „entscheidet bis … allein“), Angestellte (Zahlenfeld
  und „Festlegen“), Abdeckung (z. B. „100 % (4 Fälle für 1 im Monat)“), Wirkung in Worten
  („Senkt den Risikoaufschlag neuer Kredite um bis zu 24 %“, „Beobachtet 2 weitere Länder
  nach Übernahmezielen“ …) und Kosten im Monat mit den Kosten je Angestelltem. Frage: „Lohnt
  sich die Zentrale schon, und arbeitet sie?“ Bei Strategie und Recht steht neben der
  Leitung ihre Trefferquote.
- **Trefferquote (ZA3):** In den Fähigkeiten jedes Managers (ⓘ im Organigramm und im
  Managermarkt) die Zeile „Trefferquote: 63 % (8 bewertet)“ bzw. „noch nichts bewertet“:
  Wie oft die Schätzungen von Übernahmezielen und Lizenzen den Wert nicht überstiegen.
  Eine hohe Quote hebt die Gehaltsforderung (exponentiell) und lockt Abwerber an.
- **Empfehlungen der Zentrale (ZA3):** Anliegen „Umschuldung“ (Schritt „Kredit über …
  umschulden: 4,5 % statt 9,5 % Zins, gleiche Restlaufzeit“, Begründung „Spart rund … Zinsen
  im Jahr; die Gebühr kostet einmal …“) und „Gehaltsrunde“ („Gehalt von … auf … erhöhen“,
  „… ist unzufrieden und verdient 10 % unter dem Marktwert – ohne Erhöhung droht die
  Kündigung“). Kaufangebote des Vorstands begründen Übernahmen mit „Passt zum eigenen
  Geschäft (…)“ oder „Streut das Risiko …“, Lizenzen mit „Erspart die eigene Forschung an
  …“.
- **Abwerbung (MA6):** Will eine KI-Firma einen Manager abwerben, kommt ein Anliegen
  „Abwerbung“ (wichtig) der Personalstelle seiner Einheit, sonst der nächsten darüber, sonst
  des Managers selbst: „Gegenangebot“ (Gehalt auf das Angebot, er bleibt zufrieden) oder
  „Gehen lassen“ (er wechselt, die Stelle wird frei, ohne Abfindung). Unbeantwortet
  verfällt das Angebot, er bleibt, ist aber enttäuscht (Zufriedenheit sinkt). Kündigungen
  und Wechsel stehen im Rundenbericht.
- Stellen ohne Aufgaben (Logistik) stehen da, lassen sich aber erst besetzen, wenn sie
  Aufgaben bekommen. Laborleitungen wählen das nächste Forschungsziel (MA2).
- **Budget und Entscheidungen (MA2):** Unter den Stellen eines Standorts je besetzter
  Stelle eine aufklappbare Zeile „Werksleitung · Budget 18.500 USD je Entscheidung,
  37.000 USD im Jahr“ (oder „fragt bei jeder Ausgabe“) mit dem Zähler ihrer offenen
  Anliegen. Darin: Budget je Entscheidung und im Jahr mit Anteil, im Jahr verbraucht,
  Umsatz des Standorts in zwölf Monaten; Erklärung des Budgets (Anteil am Umsatz, Sockel
  in Jahresgehältern, 0 % = immer fragen); zwei Prozentfelder mit „Budget übernehmen“ und
  „Standard wiederherstellen (5 % / 10 %)“; „Was die Stelle selbst entschieden hat“ (die
  letzten Entscheidungen mit Datum, Thema, Produkt, Option, Betrag und erwarteter
  Wirkung); „Fragt nicht nach“ mit stummgeschalteten und abgelehnten Themen und je einem
  Knopf „Wieder fragen“; „Zu den Anliegen (n)“.
- **Anliegen (MA2):** Hinweistext (was ein Anliegen ist, Frist). Je offenes Anliegen eine
  Karte: Thema und Produkt (Marke „wichtig“ außerhalb der Routine), Stelle · Standort ·
  Land · „Name fragt“ · „Antwort bis …“, der Grund („Kostet mehr als das Budget je
  Entscheidung …“, „Dafür braucht es einen Kredit …“) und das letzte Monatsergebnis des
  Standorts. Tabelle der Optionen: Name (Marke „empfohlen“) mit ihren Schritten in Worten
  („1 × Nagelmaschine bauen (klein)“), Betrag aufs Budget, Ergebnis im Jahr als Spanne,
  einmalige Wirkung, Knopf „Umsetzen“. Darunter „Empfehlung: … “ mit Begründung und die
  Knöpfe „Entscheide selbst“, „Nicht mehr fragen“, „Ablehnen“. Gleiche Anliegen mehrerer
  Standorte stehen in einer Gruppe mit „Empfehlung für alle übernehmen“. Unten
  aufklappbar „Erledigte Anliegen“ mit dem Ausgang (gewählt, von der Stelle entschieden,
  abgelehnt, Frist verstrichen, anderweitig entschieden).
- **Rundenbericht und Übersicht:** Neue Anliegen, verfallene Anliegen und die Rückmeldung
  zur Wirkung erscheinen im Bericht mit Sprung zur Organisation; „Zu erledigen“ nennt die
  Zahl der offenen Anliegen und die erste Frist. Hinweise zu Bereichen, die eine besetzte
  Stelle übernimmt, entfallen.
- **Länder und Kontinente (MA3):** Unter jeder Kontinent-Überschrift eine Karte
  „Kontinent · Europa“ mit Kontinentvorstand und Fachstellen, unter jeder Länder-Überschrift
  „Land · Deutschland“ mit Landesleitung und Fachstellen, darunter die Standorte. Gleiche
  Spalten, „Besetzen“ öffnet den Managermarkt der Stelle („Produktion · Land · Deutschland“),
  Budget und Protokoll wie bei Standorten. „Entscheidest du selbst“ nennt nur, was keine
  Stelle darüber übernimmt.
- **Budget-Vorgaben (MA3):** Aufklappbar über dem Organigramm „Budget-Vorgaben für
  Stellentypen (n)“ (ⓘ: Reihenfolge eigene Einstellung → Land → Kontinent → Firma →
  Standard, Deckel durch die Leitung): Liste der Vorgaben („Werk: Produktion · ganze
  Firma: 3 % / 8 %“) mit „Entfernen“, darunter Stellentyp, „Gilt für“ (ganze Firma, ein
  Kontinent oder Land der Firma), zwei Prozentfelder und „Vorgabe setzen“.
- **Anliegen über mehrere Stellen (MA3):** Die Karte nennt die fragende Stelle
  („Landesleitung · Land · Deutschland · Name fragt“), „Betrifft Werk · Deutschland“ und den
  Weg mit den Empfehlungen („Weg: Werksleitung (Werk · Deutschland, empfiehlt Ausbauen) →
  Landesleitung (…)“). Ein strategisches Anliegen zeigt eine Tabelle „Standorte“ mit den
  Schritten, Beträgen und Prognosen je Standort; „Umsetzen“ oder „Entscheide selbst“ gilt
  für alle.
- **Strategie (MA4):** Frage: „Welche Vorgabe gilt wo, woher stammt sie, und wer setzt sie
  um?“ Hinweistext (ⓘ: Vererbung Standort → Land → Kontinent → Firma → Standard, Vorrang
  eigener Entscheidungen, Wirkung der Fähigkeiten). Tabelle „Wo gilt was“: je Einheit
  (Firma, Kontinent, Land, Standort, eingerückt) die sieben Felder Preis, Lager, Personal,
  Eigenfertigung oder Zukauf, Investitionsbudget, Liquiditätsreserve, Schulung (W1) – je Zelle der
  geltende Wert (fett, wenn hier festgelegt) und darunter „hier festgelegt“, „von Europa“
  oder „Standard“; „Bearbeiten“ wählt die Einheit. Darunter „Vorgaben: <Einheit>“ mit
  Auswahlfeld der Einheit und sieben Karten (je ⓘ mit der Wirkung des Felds): „Gilt: …“ mit
  Herkunft, an Standorten „Umgesetzt von: Werksleitung (Name)“ oder „Keine Stelle besetzt –
  hier entscheidest du selbst“, beim Investitionsbudget der Rest des Jahres und welches
  Budget bindet, bei der Reserve die laufenden Kosten im Monat. Eingaben je Feld
  (Preisstrategie mit Untergrenze und Startaufschlag, Mindestmarge in %, Reichweiten in
  Tagen, Lohnaufschlag in %, Lieferungen, Budget in der angezeigten Währung, Reserve in
  Monaten, Schulungsziel in %), „Hier festlegen“ und – bei eigener Vorgabe – „Vorgabe hier entfernen“ (bei der
  Firma „Auf Standard zurücksetzen“). Werte außerhalb der Grenzen meldet die Karte vor dem
  Senden.
- **Verkaufswege (M8, in der Strategie):** Abschnitt „Verkaufswege“ unter den Karten
  (ⓘ: genaueste Regel gilt, ohne Regel dürfen alle kaufen, Mindestpreis und Höchstmenge je
  Angebot und Monat): Tabelle „Regeln“ (Käufer, Gilt für – ganze Firma, Land, Produkt
  oder „Produkt in Land“ –, Regel „erlaubt, ab 1.700 USD/t, höchstens 500 t im Monat“ oder
  „gesperrt“) mit „Entfernen“; Formular „Regel festlegen“ mit Käufer (KI-Händler, andere
  Firmen), Land und Produkt (je „alle“ oder eines der eigenen), Kaufen erlaubt/gesperrt,
  Mindestpreis in der angezeigten Währung je Einheit und Höchstmenge im Monat (leer: keine
  Grenze).
- **Beteiligungen (ZA2, in der Strategie):** Abschnitt „Beteiligungen“ (ⓘ: wofür Budget
  und Freigabegrenzen gelten, Risikobereitschaft für Start-ups) mit „In diesem Jahr gekauft:
  …; offene Gebote: …. Übrig: …“ und einem Formular: „Budget im Jahr“ (leer: ohne
  Grenze), „Risikobereitschaft“ in %, darunter „Freigabegrenzen der Zentralabteilungen“ je
  Abteilung (leer: nur ihr Budget je Entscheidung), „Beteiligungen festlegen“. Werte
  außerhalb der Grenzen meldet das Formular vor dem Senden. Reicht das Budget oder die
  Freigabegrenze nicht, kommt ein Anliegen mit dem Grund „Das Budget für Beteiligungen
  reicht in diesem Jahr nicht mehr (übrig: …)“ bzw. „Kostet mehr als die Freigabegrenze der
  Abteilung (…)“.
- **Anliegen wegen Vorgaben (MA4):** „Danach läge die Kasse unter der Liquiditätsreserve
  (…)“ oder „Das Investitionsbudget (Deutschland) reicht nicht mehr (übrig: …)“.
- **Vorstand (MA5):** Über den Kontinenten eine Karte „Vorstand“ (Hinweis: was CEO und
  Ressorts tun) mit CEO und den Ressorts („Ressort Finanzen“, „Ressort Produktion“ …),
  gleiche Spalten wie bei Standorten. Das Ressort Personal hat keine Themen, lässt sich
  aber besetzen („Schätzt Bewerber und Manager genauer ein“). „Entscheidest du selbst“
  nennt ohne Vorstand Kredite, Kaufangebote und die Antworten darauf. Was der Vorstand
  selbst entscheidet (Kredit, Gebot, Verkauf, Ablehnung, Gegenangebot), steht im
  Rundenbericht („Der Vorstand verkauft …“).
- **Anliegen des Vorstands (MA5):** „CEO · Vorstand · Name fragt“; neue Gründe „Danach
  wären die Kredite höher, als der Strategieauftrag erlaubt (Spielraum: …)“ und „Ein
  Antrag des CEO aus der Strategierücksprache – unabhängig vom Budget“. Schritte von
  Kaufangeboten in Worten („Angebot von X annehmen: 1,2 Mio. USD für den Standort (Werk in
  Deutschland)“, „… verlangen“, „… ablehnen“, „… bieten“).
- **Rücksprache (MA5):** Oben das Formular „Strategieauftrag an den Vorstand“ (ⓘ: Wirkung
  der Leitlinie auf alle Stellen, Ziele, Grenzen; Investitionsbudget und Reserve stehen in
  der Strategie): Leitlinie (Wachstum, Ertrag, Sicherheit, Marktführerschaft mit Auswahl der
  Warengruppe; darunter, was sie bewirkt, mit der Aggressivität), Takt der Rücksprache,
  Ziele (Umsatzwachstum, Umsatzrendite, Eigenkapitalquote in %, Rang nach Umsatz; leer =
  ohne Ziel), Grenzen (Kredite höchstens in % der Bilanzsumme, gesperrte Länder und
  Warengruppen als Liste mit „×“ und „hinzufügen …“), „Auftrag übernehmen“; Werte
  außerhalb der Grenzen meldet das Formular vor dem Senden. Darunter „Strategierücksprachen“:
  ohne CEO der Hinweis, wie es dazu kommt; sonst „Nächste Rücksprache mit Name: Datum“,
  „Ziele heute“ (Vorgabe, Ist, erreicht/verfehlt) und die letzte Rücksprache als Karte
  („Rücksprache 01.01.1914 – 31.03.1914“, „Bericht von Name · Rücksprache quartalsweise“):
  Tabelle Umsatz und Ergebnis (Firma, je Kontinent, Gemeinkosten), Tabelle Warengruppen
  (Umsatz, Marge), Ziele des Abschnitts, Chancen (Produkte mit der höchsten Marge je Umsatz,
  Anträge) und Risiken (Standorte mit Verlust, verfehlte Ziele, Kasse unter der Reserve,
  Verschuldung über der Grenze), „Zu den Anträgen (n)“. Ältere Rücksprachen aufklappbar mit
  Umsatz und Ergebnis in der Zeile.
- **Anhalten:** Im Menü (☰) „Bei Anliegen anhalten: bei allen / bei wichtigen / nie“
  (Vorgabe: bei wichtigen; im Browser gemerkt). „Bis Jahresende“ und „bis zur nächsten
  Meldung“ halten danach an („Angehalten wegen eines Anliegens deiner Manager“). Eine
  Strategierücksprache hält immer an („Angehalten: Strategierücksprache“); ein Kaufangebot
  an die Firma hält nicht mehr an, solange der Vorstand Antworten übernimmt.
- Übersicht: Ab drei Standorten ohne einen Manager weist „Zu erledigen“ auf die
  Organisation hin. Rundgang der Einführung: Schritt „Organisation“ (nennt Budget und
  Anliegen, den Vorstand und die Rücksprache).

### Beteiligungen (SU1)

Frage des Spielers: „Wer arbeitet woran, wie weit sind sie, wie stehen die Chancen, und
wem gehören sie?“

- Neue Hauptansicht „Beteiligungen“ (Taste 8) nach der Organisation. Überschrift nach der
  Epoche: „Erfinder und Gründungen“, ab 1970 „Wagniskapital“, ab 1990 „Start-ups“.
- Kopf: ein Satz, was Start-ups sind und was ihr Erfolg bringt; Zahlen (neue im Jahr,
  gegründet seit Spielbeginn, beendet mit Erfolg und gescheitert); Hinweis, ob die
  Strategieabteilung die Chancen schätzt.
- Unterreiter „Laufend“ (mit Zähler) und „Beendet“. Laufend: Gründer (Kennzeichen
  „Erfinder“ für historische Erfinder), Land, Ziel („Technologie (neu, n Jahre vor der
  Zeit)“ oder „Produkt: Stufe n“), Phase („Prototyp (2 von 3)“), Finanzierung („Runde
  offen: sucht … bis …“ oder „… finanziert, Entscheidung am …“), Erfolgschance (Prozent mit
  arbeitender Strategieabteilung, sonst gering/mittel/hoch), Eigner mit Anteilen.
  Beendet: Ergebnis (Erfolg, Gescheitert, Kein Geld, Überholt – mit Erklärung beim
  Darüberfahren) und Datum.
- Auf dem Handy als Karten (Spaltenname über dem Wert).
- Meldungen: wenn ein Start-up eine Technologie früher als erwartet zur Marktreife
  bringt, und wenn es ein Produkt weiterentwickelt, das du verkaufst.
- Rundgang der Einführung: Schritt „Beteiligungen“ nach „Organisation“ (23 Schritte).

### Beteiligungen (SU2)

Frage des Spielers: „Wo stecke ich mein Geld hinein, was ist meine Beteiligung wert, und
was darf ich mit ihr?“

- Kopf: Zeile „Deine Beteiligungen (n): Buchwert …, heutiger Wert …“, ohne Beteiligung ein
  Hinweis, wie man einsteigt.
- Unterreiter „Laufend“, „Deine“ (Start-ups mit Anteil, Zusage oder als Tochterfirma, mit
  Zähler) und „Beendet“. Die Tabelle zeigt statt aller Eigner die Spalte „Dein Anteil“
  (Prozent, „… zugesagt“), eigene Zeilen hervorgehoben, Kennzeichen „Tochter“; je Zeile
  die Schaltfläche „Handeln“.
- „Handeln“ öffnet das Detail („Zurück zur Liste“): Land, Phase und Finanzierung,
  Erfolgschance (mit Strategieabteilung und offener Runde: „im Mittel das x-Fache des
  Einsatzes“), Wert heute und bei Erfolg, alle Eigner, dein Anteil mit Buchwert und
  Fördergeld; darunter die Rechte (Sperrminorität, Mehrheit) in einem Satz.
- Karten im Detail, nur wenn sie gehen:
  - **In der Runde zusagen** (offene Runde, „Höchstens“ der offene Rest) oder **Anteile
    kaufen** (zwischen den Runden, mit Aufschlag) – Betrag mit Tausenderpunkten;
  - **Fördergeld geben** (immer bei laufenden Start-ups, mit Wirkung in einem Satz);
  - **An Investoren verkaufen** (mit eigenem Anteil): Teil deines Anteils in Prozent,
    sofort, mit Abschlag; Erlös für alles in der Erklärung;
  - **An Firmen verkaufen** (ZA4, mit eigenem Anteil): der ganze Anteil mit
    „Mindestpreis“; die Erklärung nennt Anteil, heutigen Wert, den höchsten Aufschlag der
    Firmen und den Tag, an dem das beste Gebot kauft (nächster Monatsanfang). Danach zeigt
    die Karte das laufende Angebot mit „Angebot zurückziehen“. Der Rundenbericht meldet
    Käufer und Preis, sonst das beste Gebot (oder dass keine Firma bot);
  - **Lenken** (Mehrheit): Tempo normal, zügig, gründlich als Auswahl – wirkt sofort;
  - **Eingliedern** (Mehrheit, keine fremde Sperrminorität): Preis auf der Schaltfläche;
    hält eine andere Firma eine Sperrminorität, steht dort der Grund.
- Jede Aktion meldet ihr Ergebnis in der Karte; Fehler des Kerns (z. B. Kasse reicht
  nicht, Runde schon gedeckt) erscheinen dort als Text.
- Beendet: beim Erfolg der Ausgang („– übernommen von …“, „– Börsengang, daraus wurde …“).
- Meldungen im Rundenbericht: Rückzahlung einer Zusage, verlorene Beteiligung,
  Forschungsbonus, Erfolg der Tochterfirma, Auszahlung, Übernahme durch eine andere Firma,
  Börsengang, neue Firma aus einem Start-up.
- Empfehlungen der Strategieabteilung (Thema „Start-up“, Option „Beteiligen“) kommen als
  Anliegen in den Posteingang, wenn sie über der Freigabegrenze liegen.

### Ausgründen (SU3)

Frage des Spielers: „Welches meiner Forschungsprojekte kann ich auslagern, und was bringt
es mir?“

- Unterreiter „Ausgründen“ (Zähler: Projekte, die gehen) in der Ansicht „Beteiligungen“:
  ein Satz, was Ausgründen heißt (Investoren zahlen mit, das Forschungszentrum ist frei,
  über der Mehrheit bleibt es Tochterfirma), dann je Forschungszentrum mit Projekt eine
  Karte „Projekt …“ mit Land und Fortschritt.
- Geht es, zeigt die Karte die Startphase, den Wert und den Erlös für alles, die Restdauer
  der Phasen und bei Technologien den Vorlauf zur Geschichte; Eingabe „An Investoren
  verkaufen“ (%, Vorgabe 40) und „Ausgründen“. Warnungen, wenn die Geschichte (Vorlauf
  kürzer als die Phasen) oder andere Firmen am selben Ziel schneller sein dürften. Geht es
  nicht, steht der Grund dort (zu früh mit der Schwelle, oder schon erfunden bzw.
  erreicht).
- Ausgründungen tragen in den Listen das Kennzeichen „Ausgründung“, im Detail „Ausgründung
  von …“.
- Meldungen: wenn eine KI-Firma ein Projekt ausgründet (das Start-up sucht Geld), wenn sie
  ein Start-up übernimmt, und wenn sie dabei deinen Anteil kauft (mit Betrag).

### Forschung → Technologiebaum (M19)

- Ist: Liste der Technologien mit Erfindungsjahr, Stand, Voraussetzungen, „ermöglicht“.
  Kein Baum, keine Kosten- oder Dauerschätzung, keine Verbindung zu Produkten und
  Anlagen.
- Ziel: **Technologiebaum** nach Fachgebiet und Zeit: Knoten mit Stand (bekannt, in
  Arbeit, möglich, gesperrt), Kanten zu Voraussetzungen; je Technologie Kosten und
  Dauer bei eigenem Forschungszentrum, und was sie freischaltet: **Verfahren**
  (Rezepte mit Ein- und Ausgängen), **Produkte**, **Anlagen** (Betriebsmittel mit
  Investition und Leistung) – jeweils mit Sprung zu „Anlage bauen“.
- Umgesetzt (M19): Baum mit feststehenden Fachgebieten und Linien zu den
  Voraussetzungen, alternativ Liste (Handy: voreingestellt, Details unter dem Eintrag);
  Detail mit Fortschritt, Aufwandsfaktor, Dauer und Kosten je Labor, „Schaltet frei“ und
  Forschung starten; Forschungszentren als eigener Unterreiter.
- Nach M38: Die Spalten folgen bis 1919 den Jahrzehnten, danach den Produktepochen
  (1920–1939, 1940–1964, 1965–1989, ab 1990); vorher stand alles ab 1920 in einer
  Spalte, mit den Epochen bis 1989 gut zwanzig Technologien untereinander.

### Forschung → Weiterentwicklung (M37)

- Ziel: Der Spieler sieht für jedes Produkt, das er herstellt (umschaltbar: alle, die er
  herstellen darf), wo er steht und was die nächste Stufe bringt und kostet – und wo die
  Wettbewerber stehen.
- Umgesetzt: eigener Unterreiter zwischen Technologiebaum und Forschungszentren. Oben
  die Erklärung mit der Wirkung je Stufe; je Produkt eine Karte mit Stufe (Punkte,
  „Stufe 2 von 5“, Hinweis auf Gemeingut), Wirkung jetzt, bester Wettbewerber, nächste
  Stufe mit Wirkung, Fortschritt, Dauer (mit den eigenen Zentren oder mit einem Labor,
  dann mit Kosten) und Knopf „Weiterentwickeln“ mit Wahl des Zentrums. Ohne Zentrum führt
  die Karte zu den Forschungszentren.
- Forschungszentren: Das Projekt wählt eine Technologie oder ein Produkt („Nägel →
  Stufe 2“); „kein Projekt“ hält beides an. Der Stand zeigt den Fortschritt der Stufe.
- Mit „alle Produkte“ (Häkchen „Nur eigene Produkte“ aus) sucht ein Feld nach dem Namen.
- Markt: Die Anbieter eines Produkts zeigen ihre Entwicklungsstufe (Spalte „Stufe“).
- Rundgang der Einführung: Schritt „Produkte weiterentwickeln“ zeigt auf den Unterreiter.

### Finanzen

- Ist: Bilanz, Erfolgsrechnung (Vormonat, Jahr, Vorjahr), Geldfluss, Kredite. Klar
  gegliedert.
- Ziel: zusätzlich Verlauf (Kasse, Umsatz, Ergebnis je Monat) und **Ergebnis je
  Standort und Produkt** (Deckungsbeitrag), damit sichtbar ist, womit Geld verdient
  wird (Lastenheft §14.2).

### Weltkarte und Länderdetail

- Ist: Ebenen Lohnniveau, BIP je Kopf, Bevölkerung, Rohstoffe, Standorte; Klick öffnet
  das Land. Gut lesbar.
- Ziel: Ebene **Absatzchancen** für ein gewähltes Produkt (Nachfrage, Preis gegen
  Richtpreis); im Länderdetail Sprung „Standort hier gründen“.
- M21: Das Länderdetail nennt die Währung des Landes zum Spieldatum mit ihrem Kurs je
  US-Dollar der Zeit und die Abfolge der Währungen (Umstellungen und Reformen).
- M34/M35: Bei Regionen steht „Umfasst“ mit den Ländern der Region. **Gewerbeflächen**
  zeigt die Fläche des Landes und ihren belegten Anteil, je Lage die Zahl der freien
  Grundstücke, das größte und den Bodenpreis je ha.

### Rundenbericht

- Ist: Kasse, Eigenkapital, Finanzergebnis mit Vorrunde, Ereignisse.
- Ziel: zusätzlich „Was lief“ (Erzeugung und Absatz je Produkt, Engpässe der Runde)
  mit Sprung zur Werksansicht.
- **Weiterlaufen bis …** (M26): Die Auswahl „Rundenlänge“ hat unter „Mehrere Monate“
  zusätzlich „bis Jahresende“ und „bis zur nächsten Warnung“ (höchstens ein Jahr). Der
  Fortschritt zeigt „Runde n · Tag x von y“; ein Bericht deckt alle Runden ab, nennt ihre
  Zahl („3 Runden am Stück“) und den Grund des Halts.
- **Geschichte** (M28): Weltereignisse von 1900 bis 2026 erscheinen vor dem Bericht in
  eigenen Fenstern. Wechselt im Land des Firmensitzes oder eines eigenen Standorts die
  Währung, kommt ein Fenster „Währungsumstellung“ mit dem Umstellungskurs („1 € =
  1,95583 DM“) und dem Hinweis, dass sich das Vermögen nicht ändert.

### Neues Spiel

- Ist: lange Liste von Feldern; auf dem Handy steht das Kästchen „Einführung zeigen“
  über seinem Text.
- Ziel: Grundeinstellungen oben (Name, Land, Startform), Weiteres aufklappbar; Kästchen
  links neben dem Text.
- SU1: Unter „Weitere Einstellungen“ die Häufigkeit der Start-ups (keine, wenige, normal,
  viele – mit der Zahl je Jahr), Vorgabe „normal“.

### Einführung (M20)

- Ist: acht Textschritte, die die Ansicht wechseln; der Text spricht von der Spalte
  „Ursache“, die Spalte heißt „Engpass“; das Fenster verdeckt auf dem Desktop die
  Preisfelder. Keine Hervorhebung, kein Mitmachen.
- Ziel: geführte Aufgaben bis zum ersten Verkauf; jede Aufgabe hebt die zu benutzende
  Schaltfläche hervor (Rahmen und Abdunklung des Rests) und geht erst weiter, wenn der
  Spieler sie ausgeführt hat (Spielzustand), z. B. Werksansicht öffnen → Einkauf von
  Draht prüfen → Preis setzen → Runde beenden → im Bericht den ersten Verkauf sehen.
- Umgesetzt (M20): zwei Wege (Werkstatt, Handelsniederlassung) mit Rahmen, Abdunklung
  und Weiterschalten nach Benutzung; ist der Spieler woanders, zeigt der Rahmen den Weg.
  Ohne Verkauf folgt eine weitere Runde mit den üblichen Ursachen, danach ein Rundgang.
  Die Tafel verdeckt nichts: Dialoge und auf breiten Bildschirmen auch die Ansicht
  lassen ihr Platz; auf dem Handy lässt sie sich einklappen.
- M35: Der Rundgang zeigt nach den Etappen „Standorte und Grundstücke“ mit dem
  Gründungsformular (Lagen, Kauf oder Pacht, volle Grundstücke).
- Nach M38: Der Rundgang zeigt nach „Marke und Werbung“ den Reiter „Wettbewerb“
  (Kaufangebote für Standorte, Bereiche und Lizenzen, Angebote an den Spieler,
  Versteigerungen nach Pleiten); die Einführung hat damit 21 Schritte.

### Tastenhilfe, Speichern und Laden

- Ansichten mit 1–9 und 0 (SU1: 7 Organisation, 8 Beteiligungen, 9 Weltkarte, 0 Berichte).

- In Ordnung; Laden zeigt Firma und Datum. Ziel: Spielstände in der Browser-Version
  zusätzlich als Datei herunterladen und hochladen (Wechsel zwischen Geräten).
