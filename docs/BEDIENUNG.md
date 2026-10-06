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
    Arbeitskräfte“) und Kosten.
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

- In Ordnung; Laden zeigt Firma und Datum. Ziel: Spielstände in der Browser-Version
  zusätzlich als Datei herunterladen und hochladen (Wechsel zwischen Geräten).
