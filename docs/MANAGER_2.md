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
Jeder Manager hat 1–2 bevorzugte Begründungsschlüssel (z. B. Rendite, Marktanteil, Sicherheit,
Technologievorsprung), die er häufiger anführt. Texte je Stil unterschiedlich formuliert
(Textschlüssel je Stil, mindestens 3 Varianten je Anlass).
### 7.3 Einwand
Entscheidet der Spieler bei einem Anliegen über einer Schwelle (Betrag, Parameter) gegen die
Empfehlung, hält der Manager einen kurzen Einwand mit Zahl in der Akte fest („Ich halte das für
falsch: ...“). Kein zusätzlicher Klick nötig; MG3 greift den Einwand später auf. Querdenker und
Offensive erheben Einwände häufiger, Vorsichtige seltener.
### 7.4 KI-Firmen
Stil wirkt über ihre Kompetenz nicht; nur Anzeige im Abwerbereiter.

## 8 MG8 Personalverantwortung der Manager (erweitert MANAGER.md 3 und FORMELN „Leitung stellt ein“)
Befund (Stand GitHub 12.10.2026, gegen aktuellen Stand prüfen):
- `SetHiringByHead` besetzt nur Fachstellen der eigenen Einheit, höchstens eine je Monat, ohne
  Rückfrage, Meldung erst danach.
- Der CEO besetzt damit nur Vorstandsressorts, keine Kontinent-, Landes- oder Standortleitungen;
  der Schalter muss je Leitung einzeln gesetzt werden.
- Die Zahl der Angestellten der Zentralabteilungen setzt nur der Spieler (`StaffDepartment`), ohne
  Empfehlung; KI-Firmen rechnen n = ⌈Arbeitslast / faelle⌉ automatisch.
- Die Zentralabteilung Personal wirkt laut BETEILIGUNGEN 4.1 bei der Managerauswahl mit, in
  FORMELN ZA2/ZA3 aber nur über Schulung und Gehaltsrunde.

### 8.1 Kaskade
- Eine Leitung mit „stellt ein“ besetzt freie Fachstellen ihrer Einheit UND freie Leitungen der
  direkt untergeordneten Einheiten (CEO → Kontinent → Land → Standort).
- Eine so eingestellte Leitung erbt den Schalter (Vorgabe an), der Spieler kann ihn je Leitung
  abschalten. Neuer Schalter am CEO: „gesamte Organisation selbst aufbauen“ setzt ihn für alle.
- Reihenfolge: zuerst Stellen mit der größten Wirkung (Umsatz der Einheit); Leitungen der Ebene
  darunter erst, wenn die eigene Einheit arbeitet.
- Höchstens `einstellungen_monat` (Vorschlag 2) je Leitung und Monat; Gehalt muss ins
  Jahresbudget der einstellenden Leitung passen (wie bisher).
- Für Werksstellen zählt die Branchenstärke in der Werksbranche (MG5); für Land/Kontinent die
  Branchenstärke in den dort freigegebenen Branchen (§10).

### 8.2 Einstellen als Entscheidung mit Akte (nutzt MG2)
- Die einstellende Leitung prüft 3 Kandidaten (mit arbeitender Personalabteilung bis zu
  3 + ⌊K_Personal / 2⌋), freie Bewerber UND abwerbbare Manager fremder Firmen (MG1-Reiter),
  bewertet nach Fachkompetenz, Branchenstärke, Stil, Trefferquote, Gehalt. Eindruck und Fehler
  wie bisher nach Urteilsvermögen.
- Unter der neuen Freigabegrenze „Personal“ (je Ebene, Maßstab Jahresgehalt, bei den übrigen
  Freigabegrenzen in der Strategieansicht) stellt sie selbst ein und meldet mit Begründung
  („gewählt wegen …, Alternativen: …“). Grenze 0 = jede Einstellung als Anliegen.
- Darüber oder ohne Schalter entsteht ein Personalanliegen: 2–3 Kandidaten als Optionen, je
  Option Gehalt, Stärken/Schwächen, Stil, Begründung der Leitung, ihre Empfehlung; dazu
  „Stelle offen lassen“ mit Prognose (wer übernimmt die Aufgaben, mit welchem Abschlag).
- Akte mit Wiedervorlage nach `personal_wirkzeit` (12 Monate): Wie hat sich der Eingestellte
  gemacht (seine Trefferquote, Kennzahlen seiner Einheit gegen Vorjahr/Plan)? Meldung der
  einstellenden Leitung. Das Ergebnis zählt in ihr neues Merkmal „Personalgespür“ (Trefferquote
  nur aus Personalakten), sichtbar wie die Trefferquote.
- Entlassungen durch Leitungen nur als Anliegen (nie still), Abfindung als Kosten sofort.

### 8.3 Zentralabteilung Personal
- Mit arbeitender Abteilung: mehr Kandidaten je Prüfung (8.2), genauerer Eindruck fremder und
  freier Manager (Schätzfehler × (1 − genauigkeit · A)), und sie schlägt Abwerbeziele vor
  (Manager fremder Firmen mit niedriger Zufriedenheit und passender Stärke) als Anliegen.

### 8.4 Personalbedarf der Zentralabteilungen (alle fünf)
- Die Leitung jeder Zentralabteilung schätzt ihre Arbeitslast (Schätzfehler nach
  Urteilsvermögen) und empfiehlt eine Sollzahl:
      n_empf = ⌈geschätzte Last / faelle⌉ · Stilfaktor (offensiv 1,2, vorsichtig 0,85, sonst 1)
  begrenzt durch den freien Akademikerpool der Stadt (dann mit Hinweis „Stadt ausgeschöpft,
  Umzug des Hauptsitzes würde helfen“).
- Personalvorschlag mit: Ist, Soll, Kosten je Jahr (Lohn + Büro), was sich ändert (Abdeckung,
  Genauigkeit in %, zusätzliche Fälle, Länder, Branchen) und was bei Abbau verloren geht; bei
  der Strategie zusätzlich die empfohlene Verteilung je Branche (§9.2).
- Wann: in jeder Strategierücksprache (MANAGER.md 8.2) immer, dazu außer der Reihe, wenn die
  Abdeckung unter `unterdeckt_ab` (0,8) fällt oder die Last unter `ueberbesetzt_ab` (0,6 der
  Kapazität) liegt.
- Mit Schalter „Leitung stellt ein“ setzt die Leitung die Sollzahl selbst (`StaffDepartment`),
  solange die Mehrkosten im Jahr unter ihrer Freigabegrenze liegen, sonst Anliegen. Ohne Schalter
  immer Anliegen. In beiden Fällen Akte (Wiedervorlage 12 Monate: hat sich die Abteilung
  gerechnet?).
- Ohne Leitung meldet niemand Bedarf; die Organisationsansicht zeigt dann „ohne Leitung, kein
  Bedarf bekannt“.

## 9 MG9 Strategieabteilung als Denkstelle des Konzerns (ein System mit MG6)
### 9.1 Rolle und Fallarten
- Strategievorschläge (MG6) von CEO, Kontinent- und Landesleitungen sind Anstöße. Arbeitet die
  Strategieabteilung, prüft sie jeden Anstoß als Fall, rechnet ihn mit ihrer Genauigkeit nach
  (engere Spanne) und legt ihn vor; der Vorschlagende bleibt als Vertreter sichtbar, die
  Leitung Strategie gibt eine Stellungnahme ab (unterstützt / rät ab, mit Begründung). Ohne
  Abteilung kommen Vorschläge direkt mit der Genauigkeit des Vorschlagenden.
- Eigene Fallarten der Abteilung (alle als Strategievorschlag, ein Datentyp, eine Ansicht):

| Fall | Begründungen |
|---|---|
| Branche erschließen, ausbauen, verlassen | Versorgungslücke, Preis über Richtpreis, wenige Anbieter, Passung, Streuung |
| Land betreten, verlassen | Nachfrage, Lohnniveau, Zölle, Konkurrenz |
| Übernahme (Standort, Bereich, Firma über Börse) | Passung, Streuung, Abwehr |
| Start-up: Zusage, Nachschießen, Aufstocken | Rendite, Passung (Technologie ergänzt eigene Ketten), Streuung |
| Start-up/Beteiligung: Verkaufen, Ausstieg, Bieterverfahren | Gewinne sichern, Chance gesunken, Kapital für Besseres |
| Abwehr: Konkurrent will zugreifen (Bieterverfahren, Runde, Übernahmeziel) | Abwehr eines Konkurrenten |

  Die Begründung „Abwehr“ wird tatsächlich erzeugt, sobald eine fremde Firma auf ein Ziel bietet
  oder eine Runde zeichnet, das die Abteilung beobachtet.

### 9.2 Echte Arbeitslast, Analysten mit Branchenschwerpunkt
- Arbeitslast Strategie = beobachtete Branchen + beobachtete Länder + laufende Beteiligungen +
  Anstöße im Quartal / 3 (Gewichte als Parameter); nicht mehr fest 1.
- Angestellte werden Branchen zugeteilt (`StaffDepartment` erhält optional eine Verteilung je
  Branche; ohne Angabe gleichmäßig auf die freigegebenen Branchen mit eigenen Werken und die
  Zielbranchen offener Fälle). Abdeckung und Genauigkeit gelten je Branche:
      A_b = min(1, K_b / Last_b); Genauigkeit in Branche b aus Fachkompetenz und Branchenstärke
      der Leitung in b (MG5) und A_b
- Branchen ohne Analysten: Start-ups und Ziele dort nur als Stufe (gering/mittel/hoch), keine
  eigenen Vorschläge der Abteilung, nur Anstöße der Linie.

### 9.3 Strategieklausur (erweitert die Strategierücksprache MANAGER.md 8.2, kein neues Intervall)
- Mit arbeitender Strategieabteilung wird die Rücksprache zur Klausur, Ablauf als eigene Ansicht:
  1. Lagebild: je Branche eigene Stellung, Trend, Versorgungslücken; Beteiligungsportfolio mit
     Wert und Entwicklung; größte Bewegungen der Konkurrenz; fällige Wiedervorlagen (MG2).
  2. 2–4 Vorschläge mit echtem Zielkonflikt (MG4-Format), je mit Vertreter, Stellungnahme der
     Leitung Strategie und ihrem Stil (MG7) erkennbar.
  3. Personalvorschläge aller Zentralabteilungen (8.4).
  4. Branchenfreigaben im Überblick mit Freigabe-Vorschlägen der Manager (§10.5).
  5. Entscheidungen des Spielers mit Akte und Wiedervorlage.
- Ohne Abteilung bleibt die bisherige Rücksprache mit den MG6-Vorschlägen (plus 3. und 4.).

### 9.4 Trefferquote der Strategie: zwei Werte
- Treffsicherheit: wie MG2 (Empfehlung mindestens so gut wie die Alternative) und bei Preisen
  „nicht überzahlt“.
- Spürsinn: Anteil der lohnenden Gelegenheiten im Blickfeld (beobachtete Branchen/Länder), die
  die Abteilung empfohlen hat. Lohnend = Start-up erfolgreich bzw. Ziel nach Wirkzeit über dem
  damals gebotenen Preis. Abgelehnte Empfehlungen, die sich gelohnt hätten, zählen als Treffer
  der Leitung; nicht erkannte lohnende Gelegenheiten zählen gegen sie.
- Gehaltsforderung (BETEILIGUNGEN 4.4) nach dem Mittel beider Werte. Damit ist behoben, dass zu
  niedriges Bieten bisher immer als Treffer zählt.

### 9.5 Konkurrenzbeobachtung
- In beobachteten Ländern und Branchen meldet die Abteilung Bewegungen der Konkurrenz (Neubau,
  Übernahme, Markteintritt, Gebot auf ein Start-up mit eigenem Anteil) als Nachricht mit
  Einschätzung; bei Bedarf entsteht daraus ein Abwehrvorschlag (9.1).

### 9.6 KI-Firmen
- Nutzen Arbeitslast, Branchenverteilung und Fallarten über ihre Regeln (n = ⌈Last / faelle⌉);
  keine Klausur, keine Anliegen.

## 10 MG10 Branchenfreigabe je Ebene
Befund: Heute gibt es nur im Strategieauftrag firmenweit gesperrte Warengruppen und Länder
(`SetMandate`, `mandate::banned`); keine Freigabe je Kontinent oder Land, keine Positivliste, und
die Sperren beziehen sich auf Warengruppen statt auf die Werksbranchen (`grundlagen.yaml` branchen).

### 10.1 Stufen je Branche (neues Strategiefeld „Branchenfreigabe“ im Vorgabensystem MANAGER.md 7)
- frei: Die Stellen dürfen alles im Rahmen ihrer Budgets (Neubau, Ausbau, Übernahme, Betrieb).
- mit Rückfrage: Betrieb frei; jede Expansion (Neubau, Ausbau, Übernahme, Markteintritt) wird
  als Anliegen vorgelegt, unabhängig vom Budget.
- nur halten: Betrieb und Ersatz bestehender Anlagen; keine Expansion, keine Vorschläge zur
  Expansion.
- gesperrt: keine Expansion; bestehende Standorte laufen weiter. Ein Rückzug (Verkauf,
  Stilllegung) geschieht nie automatisch, sondern nur als Vorschlag „Branche verlassen“.
- Zusätzlich je Ebene eine Stufe für „alle übrigen bzw. neuen Branchen“ (Vorgabe: frei). Damit
  geht „nur diese Branchen“: gewünschte Branchen frei, „übrige“ gesperrt.

### 10.2 Geltung und Vererbung
- Geltungsbereiche Konzern, Kontinent, Land (Standorte haben ihre feste Branche und folgen ihrem
  Land). Vererbung von allgemein nach speziell wie alle Vorgaben; jede Zelle ist „geerbt“ oder
  „eigener Wert“.
- Die Freigabe hängt an der Einheit, nicht an der Person: Wechselt der Manager, bleibt sie.
- Die bisherigen gesperrten Warengruppen des Strategieauftrags werden zur Konzern-Ebene dieses
  Feldes (Zuordnung Warengruppe → Branche über die Produkte); gesperrte Länder bleiben im
  Strategieauftrag.
- Schalter je Ebene „gilt auch für Beteiligungen“ (Vorgabe aus): dann empfiehlt und zeichnet
  die Strategieabteilung keine Start-ups und Übernahmen in nicht freien Branchen.
- Der Spieler selbst darf weiter alles (wie bisher bei Sperren).

### 10.3 Wirkung im Kern (schnell)
- Speicherung je Geltungsbereich als zwei Bitmasken über die Branchen-Indizes (Stufe 2 Bit je
  Branche) plus Maske „gesetzt/geerbt“. Die wirksame Stufe je Einheit wird beim Ändern einer
  Vorgabe und beim Entstehen einer Einheit aufgelöst und zwischengespeichert; Abfrage im
  Prüftermin O(1). Kein HashMap-Iterieren, deterministisch.
- Prüfpunkt an genau einer Stelle: dort, wo heute `mandate::banned` greift, wird
  `branchenfreigabe(einheit, branche)` abgefragt. Gilt für Entscheidungen der Stellen,
  Strategievorschläge (MG6/MG9), Gelegenheiten, Übernahmegebote und Analystenverteilung (9.2).
- Zurückgehaltene Entscheidungen werden je Einheit und Monat gezählt (für 10.5 und die Anzeige),
  nicht jede einzeln gemeldet.

### 10.4 Bedienung
- Ansicht „Branchenfreigabe“ (Strategie) als Matrix: Zeilen Branchen, Spalten Konzern und die
  Kontinente mit eigener Einheit, ausklappbar auf Länder. Zelle zeigt die Stufe als Farbe und
  Symbol; geerbte Werte blass mit Herkunft im Tooltip.
- Ein Tipp/Klick wechselt reihum frei → mit Rückfrage → nur halten → gesperrt → geerbt.
  Langes Drücken/Rechtsklick öffnet die Auswahl direkt.
- Schnellaktionen je Spalte: „nur diese Branchen …“ (Mehrfachauswahl, übrige gesperrt),
  „alles erben“, „wie Konzern“; je Zeile: „überall gleich setzen“.
- Spalte zeigt je Branche kleine Kennzahlen: eigene Werke, Umsatzanteil, Branchenstärke der
  zuständigen Leitung, Zahl zurückgehaltener Entscheidungen im letzten Jahr.
- Auf jeder Managerkarte (Organisation, Leitung von Konzern/Kontinent/Land) die Zeile
  „Aktiv in: Maschinenbau, Chemie (2 von 12)“ mit Link direkt in die passende Spalte.
- Im Anliegen und im Strategievorschlag: Hinweis, wenn eine Option nur wegen der Freigabe fehlt.

### 10.5 Manager mit Stimme
- Sieht eine Stelle in einer nicht freien Branche eine große Chance (erwarteter Wert über
  `freigabe_vorschlag_ab`, Parameter) oder hat sie dort im Jahr mehr als n zurückgehaltene
  Entscheidungen, darf sie höchstens einmal je Branche und Jahr begründet die Freigabe
  vorschlagen (Vorgabe-Änderung als Anliegen, MANAGER.md 7, mit Akte und Wiedervorlage).
- Branchenstärke des Managers fließt in Begründung und Genauigkeit ein; Stil wirkt (Offensive
  und Querdenker schlagen öfter vor).

### 10.6 Prüfung
- Szenario: Kontinentleitung Europa mit „nur Maschinenbau“ baut in 24 Monaten kein Werk anderer
  Branchen, obwohl Gelegenheiten bestehen; Landesleitung Deutschland mit eigenem Wert „Chemie
  frei“ darf Chemie (spezieller schlägt allgemeiner).
- Szenario: Branche „mit Rückfrage“ erzeugt bei Ausbauwunsch ein Anliegen trotz Budget.
- Szenario: große Chance in gesperrter Branche erzeugt genau einen Freigabevorschlag im Jahr.
- Leistungstest: 100 KI-Firmen, 50 Jahre, Laufzeit mit und ohne gesetzte Freigaben weicht um
  höchstens 2 % ab.
- UI-Test: Von einer Managerkarte mit einem Klick in die Matrix, Stufe ändern, Wirkung beim
  nächsten Prüftermin.

## 11 Ansichten (Überblick)
- Managermarkt mit Reitern „Freie Kandidaten“ und „Manager anderer Firmen“; neu, Trefferquote,
  Branchenstärke, Stil, Filter; Einstiege von jeder freien Stelle.
- Anliegen: Optionen mit Zielkonflikt, Vertreter je Option, Einwand, Verweis auf frühere Akten.
- Chronik je Einheit und je Manager.
- Strategievorschläge: offen, angenommen, abgelehnt, mit späterem Ergebnis.
- Strategieklausur (mit Strategieabteilung) bzw. Strategierücksprache.
- Organisation: Personalgespür, Personalbedarf der Zentralabteilungen, Analysten je Branche,
  „Aktiv in“ je Leitung.
- Branchenfreigabe als Matrix.

## 12 Meilensteine
- MG1 Abwerbemenü, Pool-Prüfung, faire Zugriffszeit (sofort, vor allem anderen)
- MG2 Entscheidungsakte, Nachbetrachtung, Trefferquote für alle
- MG3 Gedächtnis und Chronik
- MG4 Optionen mit Zielkonflikt, neue Anlässe, Widerspruch
- MG5 Branchenstärke mit Gesamtdeckel und Protokoll „produktionsfaktoren“
- MG6 Strategievorschläge
- MG7 Stil, Lieblingsbegründung, Einwand
- MG8 Kaskade beim Einstellen, Personalanliegen mit Akte und Personalgespür, Personalabteilung,
  Personalbedarf aller Zentralabteilungen
- MG9 Strategieabteilung als Denkstelle: ein Typ mit MG6, Fallarten inkl. Abwehr und Verkauf,
  Arbeitslast und Branchenschwerpunkt, Strategieklausur, zwei Trefferquoten,
  Konkurrenzbeobachtung
- MG10 Branchenfreigabe je Ebene mit Matrix, Vererbung, Freigabe-Vorschlägen

Reihenfolge: MG1 zuerst; MG7, MG8 und MG10 dürfen nach MG2 laufen; MG9 nach MG6.
Einordnung in den Gesamtplan: Vorschlag in `docs/OFFENE_PUNKTE.md` eintragen und nachfragen.
Jeder Meilenstein mit Szenariotests, z. B.:
- „Abgelehnter Ausbau erzeugt nach 12 Monaten eine Nachbetrachtung mit entgangenem
  Deckungsbeitrag.“
- „Produktionsleiter mit Maschinenbau 100 senkt Arbeitsstunden im Maschinenbauwerk um 10 %, im
  Chemiewerk nicht.“
- „CEO mit Schalter ‚gesamte Organisation‘ besetzt binnen 12 Monaten Kontinent-, Landes- und
  Werksleitungen; jede Einstellung hat eine Akte mit Begründung.“
- „Strategieabteilung mit 1 Angestelltem und 3 eigenen Branchen meldet Unterdeckung und
  empfiehlt eine Sollzahl mit Kosten; nach Umsetzung steigt die Genauigkeit wie angekündigt.“
- „Leitung, die stets 30 % unter Wert bietet und alle Erfolge übersieht, hat niedrigen Spürsinn.“
- „Fremde Firma bietet auf ein beobachtetes Start-up mit eigenem Anteil → Abwehrvorschlag.“
