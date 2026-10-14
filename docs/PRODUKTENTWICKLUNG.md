# Produktentwicklung (PD1–PD11)

Vorgabe des Auftraggebers vom 13.10.2026. Sie ersetzt das bisherige „Produkt erforschen“ durch
Entwicklungsprojekte mit Meilensteinen. Umsetzung in den Meilensteinen **PD1–PD11** (Abschnitt 11;
der Auftraggeber nannte sie PE1–PE11, der Name PE ist durch `docs/PERSON.md` belegt, siehe
OFFENE_PUNKTE, Abschnitt PD). Formeln kommen vor der Umsetzung in `docs/FORMELN.md`.
Alle mit **(vorläufig)** markierten Werte stehen als vorläufige Entscheidungen in
`docs/OFFENE_PUNKTE.md` (Abschnitt PD) und liegen als Parameter in YAML.

## Rahmen

- Ein Projektleiter (PL) stellt an den Meilensteinen Zwischenergebnisse vor. Es gibt
  Entscheidungen, Probleme und Chancen.
- KI-Firmen entwickeln nach **exakt denselben Regeln**; der Spieler hat keinen Nachteil.
- Die Grundprodukte je Branche bleiben ohne Forschung verfügbar. Alle anderen Produkte werden nur
  über eine Neuentwicklung freigeschaltet. Die Epoche bestimmt weiterhin, was entwickelbar ist.
- **Namensregel:** Intern heißen die Projektmeilensteine **PM0–PM7** (Neuentwicklung) und
  **WM0–WM3** (Weiterentwicklung), damit sie nicht mit den Doku-Meilensteinen M1–M42 verwechselt
  werden. Im UI erscheinen sie als M0, M1 usw.
- Parameter, Ereigniskataloge und Rollenwerte sind datengetrieben (YAML).
- **Alte Spielstände dürfen ungültig werden.** Ein nicht ladbarer Spielstand führt zu einer
  sauberen Meldung (Textschlüssel), nicht zu einem Absturz.

## 1 Labor und Personalrollen

- Die pauschalen 20 Ingenieure je Labor entfallen. Bestehende Labore starten leer.
- Im Labor stellt der Spieler je Rolle über einen Button eine beliebige Anzahl ein oder entlässt
  sie. Die Laborgröße begrenzt die Arbeitsplätze.
- Rollen:
  - **Projektleiter:** einzelne Personen wie die Manager mit Alter, Ruhestand, Charakter (z. B.
    optimistisch, vorsichtig), Erfahrung, Branchenstärke und Bilanz. Abwerbbar und können
    abgeworben werden. Kapazität: 1 Neuentwicklung **oder** bis zu 2 Weiterentwicklungen gleichzeitig.
  - **Produktmanager (Pool):** Schwerpunkt PM0–PM1 und PM7. Bestimmen Genauigkeit der
    Marktprognose, Testmarkt/Pilotserie, Startbekanntheit und First-Mover-Bonus. In der
    Weiterentwicklung zuständig für die Ziele Qualität und Ausstattung.
  - **Entwickler (Pool):** Haupttreiber in PM2–PM6.
  - **Fertigungsplaner (Pool):** Schwerpunkt PM5–PM7. Bestimmen Startausschuss, Fertigungsrate
    und Inputmengen. In der Weiterentwicklung zuständig für Kosten, Ausschuss und Rate sowie die
    Umrüstzeit.
  - **Laborpersonal (Pool):** Tests und Erprobung. Senkt das Risiko versteckter Mängel und erhöht
    die Chance auf Durchbrüche, besonders in PM3 und PM6.
- Pools haben eine Durchschnittserfahrung statt Einzelpersonen (Leistung, Bedienbarkeit).
- Unverplantes Personal kostet Gehalt ohne Nutzen.
- Die Strategieabteilung empfiehlt dem Spieler immer, wie viel Personal je Rolle über alle
  Projekte gebraucht wird.
- Wer selbstständig einstellt und entlässt, regelt die Entwicklungsleitung (Abschnitt 10). Die KI
  stellt nach denselben Regeln ein.

## 2 Personalbedarf je Projekt

- Bei PM0/WM0 ermittelt der PL den Bedarf je Rolle und je Phase.
- Der Spieler bzw. die zuständige Entwicklungsleitung weist Personal aus dem Labor zu; die Zuweisung ist
  während des Projekts änderbar.
- **Unterbesetzung** einer Rolle: Die Phasen dieser Rolle dauern länger (etwa proportional zur
  Lücke), das Problemrisiko steigt, die Endkennzahlen in ihrem Bereich fallen schlechter aus.
- **Überbesetzung** beschleunigt mit abnehmendem Ertrag, höchstens etwa 30 % schneller (vorläufig).
- Der PL zeigt immer Szenarien: nach Bedarf (Dauer X, Risiko normal), +n je Rolle (kürzer/besser),
  −n je Rolle (länger, Risiko höher, Kennzahlenabschlag).

## 3 Neuentwicklung PM0–PM7

Gesamtdauer und Gesamtkosten hängen vom Produkt ab; vorläufig aus den bisherigen Forschungswerten
abgeleitet, später justiert.

Verteilung (Kosten / Dauer, vorläufig): PM0→PM1 5 % / 10 %; PM1→PM2 10 % / 10 %;
PM2→PM3 10 % / 10 %; PM4 Gate (Freigabe des Hauptbudgets); PM4→PM5 30 % / 30 %;
PM5→PM6 25 % / 25 %; PM6→PM7 15 % / 15 %.

- **PM0 Projektstart:** Produkt wählen, PL zuweisen, Budgetrahmen. Tempo: gründlich / normal /
  beschleunigt (mehr Tempo = kürzer, mehr Risiko). Zielausstattung: Basis / Standard / Premium.
  Der PL ermittelt den Personalbedarf.
- **PM1 Lastenheft:** Marktsicht (Zielregionen, Nachfrageprognose mit Unschärfe, Zielpreis,
  Konkurrenzlage). Entscheidung: Zielprofil, Zielkonflikt Kosten gegen Qualität.
- **PM2 Pflichtenheft:** 2–3 technische Lösungswege mit Kosten, Risiko, erwarteten Kennzahlen.
  Entscheidung: Lösungsweg.
- **PM3 Konzeptfreigabe:** Machbarkeitsnachweis, Wirtschaftlichkeitsrechnung (Amortisation).
  Entscheidung: weiter / mit Auflagen / Schleife zu PM2 / Abbruch.
- **PM4 Entwicklungsfreigabe:** Gate über das Hauptbudget von ca. 70 %.
- **PM5 Komponentenfreigabe:** Prüfen, ob alle Vorprodukte selbst herstellbar oder am Markt
  beschaffbar sind. Bei Problemen: Zukauf (teurer), Ersatzkomponente (Qualitätsabschlag) oder
  Nachentwicklung (Verzug).
- **PM6 Artikelfreigabe:** Finale Kennzahlen (Qualität, Inputmengen, Startausschuss,
  Fertigungsrate); sie streuen um höchstens ±10 % (vorläufig), abhängig von Projektverlauf,
  Besetzung, PL und Branchenstärke. Ab hier können Maschinenangebote eingeholt werden.
- **PM7 Vertriebsfreigabe:** Das Produkt ist frei, bekommt den Firmen-Produktnamen und eine
  Startbekanntheit.

## 4 Kennzahlen und Review

- Der PL stellt bei PM2, PM4 und PM6 immer dieselben Kennzahlen vor: Umsatzprognose (je Jahr und
  über den Lebenszyklus), Herstellkosten je Stück, Projektaufwand (Kosten, Dauer, Personal),
  Investitionsaufwand (Anlagen in benötigter Größe nach Absatzprognose), daraus Deckungsbeitrag
  und Amortisation.
- Die PM2-Werte werden als **Projekt-Baseline** eingefroren.
- Bei PM4 und PM6 zeigt der Bericht je Kennzahl den Wert bei PM2, den aktuellen Wert und die
  Abweichung samt Ursachen (Ereignisse, Entscheidungen, Personal, Marktpreise).
- Stammt das Projekt aus einem Produktvorschlag (Abschnitt 5), werden die Vorschlagswerte als
  zweite Referenz gezeigt.
- Jeder Bericht enthält Ampeln für Termin, Budget, Qualität, Risiken und die Empfehlung des PL.
  Der Ton hängt vom Charakter des PL ab; ein optimistischer PL prognostiziert systematisch zu gut,
  das zeigt sich später in seiner Bilanz.
- Optionen an Gates: weiter / mit Auflagen / Schleife / pausieren / abbrechen.

## 5 Produktvorschläge der Strategieabteilung

- Die Strategieabteilung schlägt Produkte vor, die zum Unternehmen passen (Werksbranchen,
  erforschte Komponenten, Märkte, Managerstärken). Die Vorschläge sind Teil des **gemeinsamen
  Strategiesystems** mit den Strategievorschlägen der Manager (`docs/MANAGER_2.md` §6, §9), kein
  zweites System. Solange die Abteilung fehlt, kommen sie über die Manager-Strategievorschläge
  (CEO/Vorstand); sobald sie existiert, übernimmt sie.
- Jeder Vorschlag enthält Umsatzprognose, Herstellkostenprognose, Projektaufwand (Kosten, Dauer,
  Personalbedarf je Rolle), Investitionsaufwand, Amortisation und eine kurze Begründung. Die
  Prognosegüte hängt von der Besetzung der Abteilung ab.
- Wird ein Vorschlag als Projekt gestartet, werden die Vorschlagswerte eingefroren; PL und
  Strategieabteilung werden daran gemessen. Projekte ohne Vorschlag bleiben frei startbar (dann
  gilt nur die PM2-Baseline).
- **Nachkalkulation:** 2 Jahre nach PM7 (vorläufig) wird Ist gegen Prognose verglichen. Das fließt
  in die Bilanz des PL (Termin-, Budget-, Kennzahlentreue, Prognosegüte) und in die Trefferquote
  der Strategieabteilung. Die Bilanz beeinflusst den Marktwert des PL und ist in der
  Personalansicht sichtbar.

## 6 Probleme und Chancen

- Ereignisse mit kurzer Geschichte und 2–3 Optionen, je Phase aus einem YAML-Katalog:
  technisches Problem (mehr Budget / Zielabstriche / Verzug), Kostenüberschreitung,
  Lieferantenausfall, Schlüsselentwickler abgeworben, Konkurrenz entwickelt dasselbe Produkt
  (als Gerücht), Durchbruch (Phase kürzer oder Qualität besser).
- Wahrscheinlichkeiten hängen ab von Tempo, Besetzung, PL-Erfahrung, Branchenstärke, Budgetpuffer.
- **Versteckte Mängel:** Durchwinken mit Auflagen, Fast-Track oder wenig Laborpersonal erhöhen das
  Risiko. Wirkung nach PM7 als Rückruf bzw. Reputationsverlust.

## 7 Delegation und Fairness

- Standard-Delegation: Neuentwicklung – der Spieler entscheidet bei PM2, PM4, PM6, PM7, der PL
  die übrigen. Weiterentwicklung – der Spieler entscheidet bei WM0, WM2, WM3. Einstellbar global
  und je Projekt: „alles vorlegen“ / Standard / „PL entscheidet alles“.
- Ereignisse mit hohem Schweregrad gehen im Standard an den Spieler, leichte entscheidet der PL
  (vorläufig).
- Unbeantwortete Entscheidungen entscheidet der PL zum Rundenende mit seiner Empfehlung, genau wie
  die KI (vorläufig).
- **Gleichstellung:** Die KI nutzt dieselben Projekte, Ereigniskataloge, Wahrscheinlichkeiten,
  Kosten und Personalregeln und kennt keine verdeckten Werte, die der Spieler nicht sieht.
  KI-Projekte erscheinen beim Spieler als Gerüchte bzw. Nachrichten und umgekehrt. Die KI-Logik
  wird in jedem PD-Meilenstein gleich mitgebaut.

## 8 Weiterentwicklung WM0–WM3

- **WM0 Projektstart:** Ziel wählen (Herstellkosten, Qualität, Ausschuss, Fertigungsrate,
  Inputmenge oder neue Ausstattungsstufe). Der PL ermittelt den Personalbedarf.
- **WM1 Optimierungsanalyse:** 2–4 Ansatzpunkte mit erwartetem Gewinn und Kosten; der Spieler wählt.
- **WM2 Optimierung durchgeführt:** Tatsächliches Ergebnis, über oder unter der Erwartung.
- **WM3 Freigabe:** Die Verbesserung gilt in allen Werken. Umrüstung bestehender Anlagen dauert
  ein Viertel der Bauzeit (vorläufig); „alle sofort“ oder „gestaffelt“.
- Je Kennzahl 3 deutliche Stufen; danach bleibt Weiterentwickeln mit kleinem Restertrag möglich
  (vorläufig 20 % der dritten Stufe).
- Der bestehende Vorteilsmechanismus (M37) bleibt, die Weiterentwicklung baut darauf auf.
  Abgrenzung zur Lernkurve (BP1): Lernkurve je Standort, Weiterentwicklung je Produkt; die Effekte
  dürfen sich nicht doppeln.

## 9 Zusatzfunktionen

- **Gate-Gremium** bei PM3, PM4, PM7: CEO bzw. Bereichsleiter äußern sich, teils mit Gegenmeinung.
- **Fast-Track** (Phasen überlappen): nur auf Vorschlag des PL; kürzer, deutlich mehr Risiko.
- **Pilotserie/Testmarkt** zwischen PM6 und PM7: nur auf Vorschlag des PL; kleine Menge in einer
  Region; deckt Mängel auf oder hebt die Qualität, kostet Zeit.
- **Teilwissen bei Abbruch:** Freigegebene Komponenten bleiben erhalten und verkürzen spätere Projekte.
- **Plattformstrategie:** Gemeinsame, bereits freigegebene Komponenten verkürzen PM4–PM5.
- **Lizenz statt Entwicklung:** Know-how kaufen; überspringt bis PM5 oder PM6, teurer, Lizenzgebühr
  je Stück. Die KI kauft und verkauft Lizenzen; Anfragen an den Spieler kommen als Anliegen.
- **Entwicklungspartnerschaft:** Zwei Firmen teilen Kosten und Ergebnis, beide erhalten das
  Produkt bei PM7.
- **Start-up-Anbindung:** Eine Beteiligung kann ein Projekt mitbringen (bei PM2/PM3). Falls SU
  noch nicht umgesetzt ist, nur die Schnittstelle vorbereiten.
- **First-Mover-Bonus:** Wer ein Produkt in einer Region zuerst freigibt, erhält Bekanntheits- und
  Reputationsbonus.
- **PL-Bilanz:** Erfahrung und Erfolgsquote; Alterung und Ruhestand über das Lebenszyklus-System
  (`docs/PERSON.md`).
- **Förderprogramme:** Epochenabhängige staatliche Programme übernehmen Teilkosten (YAML),
  verknüpft mit den Ausschreibungen.
- **Projektportfolio-Ansicht:** alle Projekte mit Phase, Ampel, Budgetverbrauch, Personal, nächstem Gate.

## 10 Entwicklungsleitung

Angelehnt an das Manager-System (`docs/MANAGER_2.md`): alle Positionen haben Charakter, Erfahrung,
Branchenstärke, Alter/Ruhestand, Bilanz und sind abwerbbar.

- **Ebenen:**
  - *Vorstand Entwicklung* (unter dem CEO): Gesamtbudget und Schwerpunkte, Verteilung auf
    Kontinente/Länder, Konflikte zwischen Regionen.
  - *Kontinent-Entwicklungsleitung:* gleicht Personal und Projekte zwischen Ländern aus.
  - *Landes-Entwicklungsleitung:* gleicht zwischen Laboren im Land aus, verschiebt Projektleiter
    und Personal.
  - *Laborleiter* (je Labor): stellt die Rollen nach dem Bedarf der Projekte ein und entlässt sie,
    plant vorausschauend (vor Phasenbeginn einstellen, nach Phasen- bzw. Projektende freigeben oder
    umsetzen) und weist Personal und PL nach Priorität zu.
- **Besetzung:** Ein CEO mit Delegation stellt den Vorstand Entwicklung ein; dieser mit Delegation
  die untergeordneten Leitungen und Laborleiter. Besetzt wird in der Manager-Übersicht.
- **Unbesetzte Ebenen:** Alle Ebenen sind optional; eine unbesetzte übernimmt die nächsthöhere.
  Ohne jede Besetzung entscheidet der Spieler bzw. CEO wie bisher.
- **Eskalation:** Bedarf über dem Budget einer Ebene geht als Anliegen an die nächsthöhere, mit
  Begründung und den Szenarien der PL.
- **Rahmen je Ebene:** Personalbudget (von oben verteilt), Reservepuffer (vorläufig 10 %),
  Entlassen erlaubt ja/nein, Modus „stellt selbst ein“ oder „nur Empfehlung“. Im Modus „nur
  Empfehlung“ empfiehlt die Strategieabteilung den Bedarf, eingestellt wird durch den Spieler bzw.
  die nächsthöhere Ebene.
- **Branchenfreigabe:** je Leitung festlegbar, wie bei den übrigen Managern (MG10).
- **Charakter:** Ein sparsamer Leiter besetzt knapp und riskiert Unterbesetzung; ein großzügiger
  hält Reserve und kostet mehr.
- **Gegen Hin-und-her:** Einstellungen haben eine Vorlaufzeit (vorläufig 2 Runden), Entlassungen
  kosten Abfindung (vorläufig 3 Monatsgehälter), Verschieben zwischen Laboren kostet Einarbeitung
  (vorläufig 1 Runde ohne Wirkung).
- **Bilanz je Ebene:** Auslastung, Personalkosten, durch Unterbesetzung verursachter Verzug.
- **Fairness:** KI-Firmen nutzen dieselbe Hierarchie nach denselben Regeln.
- **Oberfläche:** Im Labor und im Portfolio ist sichtbar, welche Ebene gerade einstellt, entlässt
  oder verschiebt, und warum.
- **Schnittstelle:** Bis PD11 liefert die „zuständige Stelle“ für Zuweisung und Entscheidung (PD2,
  PD3) nur Spieler oder PL; PD11 ersetzt sie durch die Entwicklungsleitung, ohne Umbau der Projekte.

## 11 Umsetzungs-Meilensteine

| Nr. | Inhalt |
| --- | --- |
| PD1 | Labor-Rollen: Einstellen per Button, Pools, PL als Person, Arbeitsplätze nach Laborgröße, Entfall der 20 Ingenieure |
| PD2 | Neuentwicklung PM0–PM7 ersetzt Forschung: Kosten- und Dauerverteilung, Personalbedarf mit Wirkung und Szenarien |
| PD3 | Meilensteinberichte: Kennzahlen PM2/PM4/PM6, Review gegen PM2, Entscheidungen, Delegation |
| PD4 | Ereigniskatalog, Kennzahlenstreuung, versteckte Mängel und Rückrufe |
| PD5 | Weiterentwicklung WM0–WM3 mit Stufen und Restertrag |
| PD6 | Produktvorschläge im Strategiesystem (mit MG6/MG9), eingefrorene Vorschlagswerte, Nachkalkulation, Bilanz von PL und Strategieabteilung |
| PD7 | Gate-Gremium, Fast-Track- und Pilotserien-Vorschläge des PL |
| PD8 | Teilwissen, Plattform, Lizenzen, Entwicklungspartnerschaft, Start-up-Schnittstelle |
| PD9 | First-Mover, Förderprogramme, Portfolio-Ansicht |
| PD10 | Balancing-Durchlauf und Tests zur Gleichstellung von KI und Spieler, auch für die Hierarchie von PD11 (gleiche Seeds, gleiche Ergebnisse bei gleichen Entscheidungen) |
| PD11 | Entwicklungsleitung (Vorstand Entwicklung, Kontinent-, Landes-Entwicklungsleitung, Laborleiter) mit Budgetkaskade, Eskalation, Branchenfreigabe und KI-Gleichstellung; nach MG8 (Einstellkaskade) |

Jeder Meilenstein: Formeln in `docs/FORMELN.md` (Abschnitt PD), Daten und Prüfregeln, Kern mit
KI-Logik, Sichten und Oberfläche, Tests, Weltlauf-Vergleich, Doku, Einzel-Commit.
