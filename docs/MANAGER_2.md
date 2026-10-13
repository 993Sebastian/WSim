<!-- ACHTUNG: Der Text der Vorgabe vom 13.10.2026 kam beim Auftrag abgeschnitten an: er bricht in
     §7.2 mitten im Wort ab ("Sic…"). §7.2 bis §10 sowie die Beschreibung von MG8–MG10 fehlen
     und werden vom Auftraggeber nachgeliefert; bis dahin wird nur umgesetzt, was hier steht. -->

# Vorgabe Manager mit Geschichte

Neue Vorgabe vom Auftraggeber (13.10.2026): Überarbeitung Manager-System „Manager mit Geschichte“
inklusive Strategieabteilung, Personalverantwortung und Branchenfreigabe (MG1–MG10).
Bei Widerspruch gilt diese Datei vor `MANAGER.md`, vor `BETEILIGUNGEN.md` §4 und vor den
Abschnitten ZA2, ZA3 und SU2 („Empfehlung der Strategieabteilung“) in `FORMELN.md`.
Formeln vorher in `FORMELN.md`, Zahlen in `data/parameter/management.yaml` bzw. `zentrale.yaml`,
alle Texte als Textschlüssel. Alte Spielstände müssen nicht kompatibel bleiben (Entscheidung
12.10.2026); die Version wird trotzdem erhöht.

## 0 Entscheidungen des Auftraggebers (13.10.2026)
1. Leitbild: „Du führst Menschen, nicht ein Formular.“ Manager haben eine nachvollziehbare
   Bilanz, ihre Fragen sind echte Abwägungen, und die Folgen kommen zurück.
2. Entscheidungen bekommen eine Geschichte: Nachbetrachtung, Trefferquote, Gedächtnis, Chronik.
3. Mehr Tiefe und Vielfalt der Anliegen: 2–4 Optionen mit echtem Zielkonflikt, mehr Anlässe.
4. Strategie wird gemeinsam gelenkt: CEO und höhere Ebenen machen Strategievorschläge,
   auch zu Branchen.
5. Branchenstärke je Manager mit höchstens 10 % Wirkung in der Produktion (vorsichtig, weil
   schon viele Produktionseffekte wirken; Gesamtdeckel siehe §5.4).
6. In Europa findet der Spieler kaum freie Manager, und das Abwerben ist schwer zu finden.
   Lösung über ein sichtbares Abwerbemenü und faireren Zugriff, nicht über einen größeren Pool.
   Die Poolgröße wird NICHT an die Zahl der Firmen gekoppelt.
7. Manager werden vermenschlicht: Jeder hat einen Stil, eine Meinung und widerspricht begründet.
8. Personal ist eine zentrale Entscheidung: Leitungen stellen selbst ein (auch kaskadierend
   nach unten), aber jede Einstellung hat eine Begründung, eine Akte und eine Nachbetrachtung.
9. Jede Leitung einer Zentralabteilung empfiehlt dem Spieler, wie viele Angestellte ihre
   Abteilung braucht, auch wenn sie selbst einstellen darf.
10. Strategieabteilung und Strategievorschläge (MG6) sind EIN System, nicht zwei.
11. Der Spieler legt je Ebene (Konzern, Kontinent, Land) fest, in welchen Branchen die Manager
    dort aktiv sein dürfen; einfach zu bedienen, schnell im Kern.

## 1 MG1 Abwerben sichtbar machen, Managermarkt fair (zuerst umsetzen)
Befund (Testlauf auf Stand GitHub 12.10.2026, Seed 7, 100 KI-Firmen, Start 1900, DEU; gegen den
aktuellen Stand prüfen): Europa hat ab dem 6. Monat dauerhaft 0 freie Kandidaten.
`staffing::month_start` (`ai_staffing`, `heads_hire`) läuft direkt nach dem Auffüllen in
`management::month_start`; die KI stellt die neuen Kandidaten ein, bevor der Spieler sie sieht.
Abwerben ist für den Spieler nur über Wettbewerb → einzelne Firma → „Führung“ erreichbar; eine
Übersicht über alle fremden Manager fehlt.

### 1.1 Abwerbemenü (Hauptpunkt)
- Im Managermarkt zwei Reiter: „Freie Kandidaten“ und „Manager anderer Firmen“.
- Der zweite Reiter listet alle Manager fremder Firmen (KI wie andere), die der Spieler kennen
  kann: Name, Firma, Stelle, Land/Kontinent, Funktion, Stufen der Fähigkeiten, Trefferquote
  (ab MG2), Branchenstärke (ab MG5), Stil (ab MG7), Zufriedenheit als Stufe, aktuelles Gehalt
  soweit bekannt, voraussichtliche Forderung, Sperrfrist nach letztem Umwerben.
- Filter: Kontinent, Land, Funktion, Branche, Mindeststufe, „wechselbereit“ (niedrige
  Zufriedenheit). Sortierung: passende Funktion und Kontinent der Stelle zuerst.
- Button „Abwerben“ direkt in der Liste; nutzt den bestehenden Befehl `PoachManager` und die
  bestehende Logik (Forderung, Aufschlag, Antwort am nächsten Tag, Sperre).
- Einstiege: im Hauptmenü bzw. in der Organisation sichtbar „Managermarkt“; an jeder freien
  Stelle zwei Buttons „Kandidaten suchen“ und „Manager abwerben“ (öffnet den zweiten Reiter mit
  Filter auf diese Stelle). Ohne freie Stelle: Hinweis wie bisher, aber mit Link zur
  Organisation, um eine Stelle zu schaffen.
- Die bestehende Firmenseite unter Wettbewerb → „Führung“ bleibt und verlinkt ins Menü.
- Wie genau der Spieler fremde Manager einschätzen kann, folgt den bestehenden Regeln
  (Stufen, Eindruck, Personalressort; ab MG8 genauer mit Personalabteilung).

### 1.2 Pool
- Poolgröße bleibt wie bisher an die Akademiker je Kontinent gebunden (`je_mio_akademiker`, min,
  max), keine Kopplung an Zahl oder Größe der Firmen.
- Prüfen, dass die Akademikerzahlen je Land historisch langsam wachsen und der Pool damit über
  die Jahrzehnte mitwächst. Falls nicht: langsame historische Wachstumsrate der Akademiker aus
  den Länderdaten bzw. einen Parameter einführen; Werte in `FORMELN.md` belegen.

### 1.3 Fairness beim Zugriff
- Jeder Kandidat bekommt das Datum seines Markteintritts. KI-Firmen dürfen erst nach
  `ki_wartetage` (10) Tagen auf dem Markt zugreifen, der Spieler sofort. Der Pool wird dadurch
  nicht größer.
- Managermarkt zeigt „neu“ und die Tage bis zum möglichen KI-Zugriff.

### 1.4 Prüfung
- Szenariotest (Aufbau wie Befund): Am Tag 1–10 jedes Monats sieht der Spieler in Europa die
  in diesem Monat neu erschienenen Kandidaten.
- UI-Test: Von einer freien Stelle gelangt man mit einem Klick in die Liste fremder Manager
  und kann ein Abwerbeangebot abgeben.
- Neues CLI-Protokoll „managermarkt“ je Kontinent und Jahr: Poolziel, frei, neu, von KI
  eingestellt, abgeworben. Damit sieht der Auftraggeber, ob über die Jahrzehnte ein Engpass
  entsteht.

## 2 MG2 Entscheidungsakte und Nachbetrachtung (erweitert MANAGER.md 6.4)
- Jedes Anliegen mit Wirkung über Routine erzeugt beim Abschluss eine Entscheidungsakte:
  Anliegen, Stelle, Manager, Optionen, empfohlene und gewählte Option, Prognose je Option
  (Kennzahl, Spanne), Wiedervorlagedatum.
- Wirkzeit je Thema in den Daten (3–24 Monate; z. B. Preis 3, Personal 6, Ausbau 12,
  Branche/Land 24).
- Bei Wiedervorlage misst der Kern die tatsächliche Kennzahl und schätzt für die nicht
  gewählte empfohlene Option eine Gegenrechnung mit einfacher Regel je Thema (z. B. abgelehnter
  Ausbau: nicht bediente Nachfrage × Deckungsbeitrag je Stück). Ergebnis als Meldung des
  Managers mit Zahlen: „Ausbau im März 1907 abgelehnt, seitdem 1.800 t Nachfrage nicht bedient,
  entgangener Deckungsbeitrag ca. …“.
- Trefferquote für alle Manager (bestehende Felder `judged`/`hits` wiederverwenden): Treffer,
  wenn seine Empfehlung im Ergebnis mindestens so gut war wie die Alternative (Toleranz als
  Parameter). Sichtbar im Managermarkt, in der Organisation und im Anliegen. Bei Stellenwechsel
  nimmt der Manager seine Trefferquote mit.
- Gehaltsforderung folgt der Trefferquote wie in `BETEILIGUNGEN.md` 4.4, jetzt für alle
  (für die Strategie siehe §9.4).

## 3 MG3 Gedächtnis und Chronik
- Überstimmt der Spieler einen Manager und der Manager hatte recht: Beim nächsten Anliegen im
  selben Thema verweist er darauf (Textschlüssel mit Datum und Ergebnis). Der bestehende
  Zufriedenheitsabzug „übergangen“ greift nur noch in diesem Fall.
- Lag der Manager falsch und der Spieler hatte recht: kein Abzug; mit Chance steigt sein
  Urteilsvermögen um 1 (er lernt), seine Risikoneigung nähert sich der des Spielers an.
- Chronik je Einheit und je Manager: alle Entscheidungsakten mit Ergebnis, filterbar nach
  Thema, Zeitraum, „gegen Empfehlung entschieden“. Neue Ansicht im Organisationsbereich und
  Link aus jedem Anliegen auf frühere Akten zum selben Thema.

## 4 MG4 Tiefe und Vielfalt der Anliegen
- Jedes Anliegen hat 2–4 Optionen. Jede Option zeigt: Kosten sofort, Kosten laufend je Jahr,
  erwartete Wirkung (Spanne), Risiko, Zeit bis zur Wirkung. Mindestens eine Option ist
  „nichts tun“ mit ihrer eigenen Prognose. Reine Ja/Nein-Anliegen nur, wo es fachlich nicht
  anders geht.
- Neue Anlässe (nur aktiv, wenn das zugrunde liegende System existiert, sonst in den Daten
  angelegt und inaktiv): Lieferantenwechsel, Qualität gegen Menge, Lohnforderung,
  Preissenkung eines Konkurrenten, Großüberholung gegen Ersatzinvestition, Lageraufbau vor
  erwartetem Preisanstieg, Expansion ins Nachbarland, Reaktion auf Konjunktureinbruch.
- Widerspruch: Empfehlen zwei Stellen derselben Einheit im selben Prüftermin gegensätzliche
  Optionen (z. B. Vertrieb Preis senken, Produktion Qualität halten), entsteht ein Anliegen,
  in dem jede Option zeigt, welcher Manager sie vertritt und mit welcher Begründung.
- Die Mengenbegrenzung aus `MANAGER.md` 6.6 gilt weiter.

## 5 MG5 Branchenstärke
### 5.1 Merkmal
Jeder Manager hat Branchenerfahrung 0–100 je Branche (`data/grundlagen.yaml` branchen). Beim
Erzeugen: eine Heimatbranche (Mittel 55, Streuung 15), mit Chance eine zweite (Mittel 35), übrige
0–10. Anzeige als Stufen wie die Fähigkeiten.
### 5.2 Wachstum
Je Monat in einer Stelle, deren Einheit Werke dieser Branche hat, mit Chance +1 (wie Erfahrung im
Markt, `chance_monat` 0,3), bis zum Potenzial.
### 5.3 Wirkung im Werk
Maßgeblich ist die Fachstelle Produktion; ist sie leer, die Werksleitung mit halber Wirkung. Nur
die feste Werksbranche zählt. Stärke s = max(0, E − 50) / 50 (0 bis 1).
- Arbeitsstunden je Durchlauf × (1 − 0,10 · s)
- Ausschussquote × (1 − 0,10 · s) (sobald Ausschuss aus WELT_MARKT_BETRIEB 13.4 umgesetzt ist)
- Materialeinsatz × (1 − 0,03 · s)

Kein Malus unter 50. Alle drei Werte als Parameter. Höhere Ebenen (Land, Kontinent, Vorstand):
keine Produktionswirkung; Branchenstärke macht ihre Prognosen und Strategievorschläge in dieser
Branche genauer (§6, §9).
### 5.4 Gesamtdeckel gegen Überschwingen
Alle Produktionsfaktoren (Lernkurve, Schulung, Anlagengröße, Kennzahlen aus den
Maschinenangeboten, Branchenstärke) werden multipliziert und dann je Größe auf eine Untergrenze
begrenzt (Parameter, Vorschlag: Arbeitsstunden 0,40, Material 0,80, Ausschuss 0,30 des
Grundwerts). Neues CLI-Protokoll „produktionsfaktoren“: Verteilung der Einzel- und Gesamtfaktoren
über alle Werke, damit der Auftraggeber Überschwinger sieht. Test: Kein Werk unterschreitet die
Untergrenzen.
### 5.5 Anzeige
Managermarkt (beide Reiter) und Organisation zeigen die Branchenstärke; Filter „Branche“;
Sortierung für eine Werksstelle: passende Branche zuerst.
### 5.6 KI
KI-Firmen werten Branchenstärke bei der Auswahl mit (über ihre Kompetenz).

## 6 MG6 Strategievorschläge
- CEO sowie Kontinent- und Landesleitungen machen von sich aus Strategievorschläge: Branche
  erschließen, ausbauen oder verlassen; Land betreten oder verlassen; Übernahme; Großinvestition.
- Grundlage sind Marktdaten (Versorgungslücken, Preise über Richtpreis, wenige Anbieter, wie in
  der Ansicht „Chancen“), die eigene Stellung und die Branchenstärke des Vorschlagenden.
  Genauigkeit aus Urteilsvermögen und Branchenstärke; schwache Manager liegen öfter daneben.
- Jeder Vorschlag: Art, Ziel, Betrag, Zeithorizont, Chance und Risiko mit Spanne, Begründung
  (Textschlüssel), wie sicher sich der Manager ist.
- Antworten: annehmen (erzeugt die passenden Vorgaben bzw. Folgeanliegen mit konkreten
  Befehlen), ablehnen, anpassen (Betrag, Land, Zeitpunkt), zurückstellen.
- Häufigkeit: in jeder Strategierücksprache (`MANAGER.md` 8.2), dazu höchstens ein Vorschlag je
  Quartal außer der Reihe (Parameter).
- Jeder Vorschlag bekommt eine Entscheidungsakte (MG2) mit Wiedervorlage und zählt zur
  Trefferquote.
- Datenmodell gemeinsam mit den Empfehlungen der Zentralabteilungen (`BETEILIGUNGEN.md` 4.3):
  ein Typ, eine Ansicht „Strategievorschläge“, eine Trefferquote. Siehe §9.
- Vorschläge beachten die Branchenfreigabe (§10).

## 7 MG7 Stil und Einwand (alle Manager)
### 7.1 Stil
Neues Merkmal „Stil“ je Manager, beim Erzeugen gezogen (Anteile in den Daten):
- offensiv: bevorzugt Wachstum und größere Beträge, Risikoneigung +, Spannen eher optimistisch
- vorsichtig: bevorzugt „kleiner anfangen“ und „abwarten“, Risikoneigung −
- zahlengetrieben: engere Spannen, empfiehlt die Option mit dem besten Erwartungswert, wenig
  Mut zu Neuem (keine Vorschläge für neue Branchen ohne Datenbasis)
- Querdenker: bringt ungewöhnliche Optionen (z. B. Nachbarbranche, Nischenland), größere
  Streuung: mehr Volltreffer und mehr Fehlgriffe

Der Stil verschiebt nur, welche Option empfohlen wird, und die Spannen; er ersetzt keine
Fähigkeit. Sichtbar im Managermarkt (beide Reiter), in der Organisation und im Anliegen.
### 7.2 Lieblingsbegründung
Jeder Manager hat 1–2 bevorzugte Begründungsschlüssel (z. B. Rendite, Marktanteil, Sic

<!-- [Text der Vorgabe an dieser Stelle abgeschnitten] -->
