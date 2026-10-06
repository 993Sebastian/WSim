# Vorgabe Hauptsitz, Zentralabteilungen und Start-ups

Stand: 06.10.2026. Erarbeitet mit dem Auftraggeber, aufbauend auf `docs/MANAGER.md`
(Branch `claude/manager-vorgabe`). Ergänzt Lastenheft §5.1, §5.6, §7.1, §11.2 und §18.6.
Bei Widerspruch gilt diese Datei vor dem Lastenheft, `docs/MANAGER.md` geht ihr vor.

Umgesetzt wird erst nach Freigabe durch den Auftraggeber (Abschnitt 9).

## 0 Entscheidungen des Auftraggebers (06.10.2026)

1. Start-ups arbeiten an **neuen Technologien** oder an **Verbesserungen** bestehender
   Technologien. Verbesserungen lösen Technologiesprünge bei Kosten, Qualität oder
   Effizienz aus; solche Start-ups gibt es vor und nach 2026.
2. Eine **Minderheitsbeteiligung** bringt nur finanziellen Gewinn, keinen Zugang zur
   Technologie.
3. **Namen:** historische Erfinder zu ihren realen Erfindungen und fiktive Gründer.
4. Der Spieler kann **eigene Ausgründungen** machen.
5. Die **Häufigkeit** gilt pro Jahr, die Anzahl ist beim neuen Spiel einstellbar.
6. Der **Hauptsitz** ist frei wählbar und verlegbar; er wirkt auf Steuern, Personal und
   Risiken in Kriegszeiten.
7. Neben der Strategieabteilung gibt es weitere **Zentralabteilungen** (Finanzen,
   Personal, Recht, Marketing); sie sind vollständig zu beschreiben.
8. Zentralabteilungen sind **ab Spielbeginn** einrichtbar, aber teuer; zu früh
   aufgebaut ruinieren sie das Unternehmen.
9. Bis zu welchem Betrag eine Zentralabteilung selbst entscheidet, legt der Spieler als
   **Strategievorgabe** fest.
10. Die Strategieabteilung empfiehlt **Start-ups und Übernahmen** etablierter Firmen.
11. Die **Trefferquote** jeder Abteilungsleitung ist sichtbar und wirkt **exponentiell**
    auf ihre Gehaltsforderung.

## 1 Ziel

Der Konzern bekommt einen Hauptsitz mit Zentralabteilungen, die konzernweit arbeiten
und dem Spieler begründete Empfehlungen geben. Die Strategieabteilung verwaltet
Beteiligungen an jungen Firmen, die an Technologien forschen. Viele davon scheitern;
erfolgreiche lösen einen Technologiesprung aus und können zu Tochterfirmen oder zu
neuen KI-Konkurrenten heranwachsen.

## 2 Begriffe

| Begriff | Code (Vorschlag) | Bedeutung |
| --- | --- | --- |
| Hauptsitz | `Headquarters` | Sitz von Vorstand und Zentralabteilungen; erweitert den Firmensitz aus §5.1 |
| Zentralabteilung | `CentralDepartment` | Konzernweite Abteilung im Hauptsitz mit Leitung und Angestellten |
| Empfehlung | `Recommendation` | Anliegen (`Concern`, `docs/MANAGER.md`) einer Zentralabteilung mit Begründung |
| Trefferquote | `HitRate` | Anteil der Empfehlungen einer Leitung, die sich nachträglich ausgezahlt haben |
| Start-up | `Venture` | Junge Firma oder Erfinder mit Zieltechnologie, Phase und Anteilseignern |
| Finanzierungsrunde | `FundingRound` | Kapitalbedarf einer Phase, für den Anteile oder Fördergeld angeboten werden |
| Ausgründung | `SpinOff` | Vom Spieler abgespaltenes Forschungsprojekt als eigenes Start-up |

## 3 Hauptsitz

- Der Firmensitz aus §5.1 wird zum Hauptsitz. Der Spieler wählt Land und Stadt frei,
  beim neuen Spiel als Startland.
- **Verlegen** kostet Geld und dauert einige Monate; ein Teil der Angestellten der
  Zentralabteilungen zieht nicht mit (Parameter).
- **Wirkungen des Standorts:**
  - Steuern des Konzerns nach dem Land des Hauptsitzes (§11.1).
  - Akademiker-Pool und Lohnniveau des Landes für die Angestellten der Abteilungen;
    Managerpool des Kontinents (`docs/MANAGER.md` 4.2).
  - Risiken in Kriegszeiten: Ausfall der Abteilungen oder Beschlagnahme, je nach
    Kriegslage des Landes (Stufe 4, bis dahin vorbereitet, aber ohne Wirkung).
- In der Werkstatt gibt es noch keine Zentralabteilung; der Spieler erledigt alles
  selbst. Der Hauptsitz wird mit der ersten Zentralabteilung sichtbar.

## 4 Zentralabteilungen

### 4.1 Abteilungen

| Abteilung | Zuordnung | Aufgaben | Beispiele für Empfehlungen |
| --- | --- | --- | --- |
| Strategie | neues Ressort, berichtet an den CEO | Beteiligungen und Start-ups verwalten (Abschnitt 5), Übernahmeziele suchen, Märkte und Konkurrenz beobachten | Anteile an einem Start-up kaufen, aufstocken oder verkaufen; Mehrheit an einer etablierten Firma über die Börse übernehmen; Kaufangebot nach §18.4 |
| Finanzen | Ressort Finanzen | Kredite, Anleihen, Liquidität, Währungen, Dividenden | Umschuldung, Kapitalerhöhung, Währungsabsicherung |
| Personal | Ressort Personal | Konzernweite Managerauswahl, Gehaltsniveau, Schulungsprogramme; bündelt die HR-Aufgaben aus §5.4 | Manager abwerben, Gehälter anpassen, Schulung ausweiten |
| Recht | neues Ressort, berichtet an den CEO | Patente, Lizenzen, Verträge, Regulierung, Kartellfragen | Patent anmelden, Lizenz anbieten, Risiken einer Übernahme |
| Marketing | Ressort Vertrieb und Marketing | Marken und konzernweite Kampagnen | Kampagne starten, Marke neu positionieren |

- Strategie und Recht sind neue Vorstandsressorts (Ebene 4 in `docs/MANAGER.md` 3, gleiche
  Gehaltsfaktoren). Ist der CEO unbesetzt, gehen ihre Anliegen an den Spieler.
- Die Liste ist in den Daten erweiterbar.

### 4.2 Aufbau

- Jede Abteilung hat eine **Leitung**: den Manager des zugehörigen Ressorts, mit den
  Merkmalen aus `docs/MANAGER.md` 4.1.
- Dazu kommen **Angestellte** als Gruppe von Akademikern der passenden Fachrichtung
  (wie §5.3, keine Einzelpersonen), eingestellt im Land des Hauptsitzes.
- Die **Anzahl** der Angestellten bestimmt, wie viel die Abteilung bearbeiten kann (z. B.
  wie viele Start-ups und Märkte die Strategie beobachtet). Ihre **Qualifikation** und
  die Fachkompetenz der Leitung bestimmen, wie genau die Einschätzungen sind.
- **Kosten:** Gehälter der Leitung und der Angestellten plus Bürokosten je Angestelltem,
  gebucht als Gemeinkosten des Hauptsitzes. Die Werte sind so hoch, dass eine
  Abteilung erst ab einer gewissen Firmengröße trägt.

### 4.3 Empfehlungen

- Eine Empfehlung ist ein Anliegen nach `docs/MANAGER.md` 6 mit zusätzlich:
  - Ziel und Art (z. B. Anteile kaufen, Fördergeld, Übernahme, Verkauf),
  - Betrag,
  - Einschätzung von Chance und Risiko mit Unsicherheit,
  - Begründung als Textschlüssel: lohnende Rendite, Passung zum Geschäftsmodell
    (Technologie ergänzt eigene Ketten), Abwehr eines Konkurrenten, Streuung des
    Risikos.
- Der Spieler nimmt an, lehnt ab oder stellt zurück (`AnswerConcern`).
- **Freigabegrenze:** Im neuen Strategiefeld „Beteiligungen“ (§5.6) legt der Spieler je
  Abteilung fest, bis zu welchem Betrag sie selbst entscheidet. Darunter handelt sie
  ohne Rückfrage, darüber entsteht eine Empfehlung. Das ergänzt das Budget je
  Entscheidung und je Jahr aus `docs/MANAGER.md` 5.
- Schwache Leitungen empfehlen auch Fehlgriffe und übersehen Chancen; die Qualität folgt
  aus Urteilsvermögen und Fachkompetenz.

### 4.4 Trefferquote und Gehalt

- Für jede Leitung wird festgehalten, wie sich ihre Empfehlungen entwickelt haben
  (angenommene wie abgelehnte; Bewertung nach einer Frist aus den Daten, z. B. Wert der
  Beteiligung, Erfolg des Start-ups).
- Die Trefferquote ist im Managermarkt und in der Organisationsansicht sichtbar.
- Die **Gehaltsforderung** steigt exponentiell mit der Trefferquote, z. B.
  Forderung = Grundgehalt × e^(k × (Trefferquote − Mittelwert)), `k` in den Daten.
- Leitungen mit hoher Trefferquote werden häufiger abgeworben (`docs/MANAGER.md` MA6).

## 5 Start-ups

### 5.1 Bezeichnung je Epoche

- Um 1900 „Erfinder“ und „Gründungen“, ab etwa 1970 „Wagniskapital“, ab den 1990ern
  „Start-ups“. Die Bezeichnungen und Jahre stehen in den Daten.

### 5.2 Entstehung

- Start-ups entstehen zufällig über einen eigenen Zufallsstrom, jedes mit Land, Namen,
  Zieltechnologie, Phase und Anteilseignern.
- Häufigkeit pro Jahr, unabhängig von der Rundenlänge; die Anzahl ist beim neuen Spiel
  einstellbar (§15).
- **Namen:** historische Erfinder zu ihren realen Erfindungen (in den Daten an die
  Technologie gebunden) und fiktive Gründer aus Namenslisten je Land.
- **Arten:**
  - **Neue Technologie:** Ziel liegt knapp vor dem aktuellen Stand im Baum. Je weiter sie
    dem historischen Jahr voraus ist, desto höher Kapitalbedarf und Risiko (§7.1).
    Ab 2026 gibt es diese Art nicht mehr.
  - **Verbesserung:** Ziel ist eine bereits erfundene Technologie; bei Erfolg gibt es
    einen Sprung bei Kosten, Qualität oder Effizienz. Gibt es vor und nach 2026.

### 5.3 Phasen und Finanzierung

- Drei Phasen: Idee, Prototyp, Marktreife. Jede hat einen Kapitalbedarf und eine
  Finanzierungsrunde.
- Ohne ausreichendes Kapital stockt ein Start-up und geht nach einer Frist ein.
- Die Erfolgschance ist nur als Einschätzung sichtbar: grob ohne Strategieabteilung,
  genauer, je besser sie besetzt ist (4.2).
- Misserfolgsquote insgesamt hoch, Richtwert 60 bis 70 %, in den Daten einstellbar.
- KI-Investoren und KI-Firmen bieten nach denselben Regeln mit (§10) und kaufen
  Start-ups weg.

### 5.4 Wege der Beteiligung

- **Anteile kaufen:** in einer Finanzierungsrunde oder von anderen Eignern. Anteile
  laufen über das bestehende Modell `Stake`/`Holder`. Wer in späteren Runden nicht
  nachschießt, wird verwässert.
- **Fördergeld ohne Anteile:** erhöht die Erfolgschance. Der Geber erhält keine Rechte;
  sein Nutzen ist, dass die Technologie früher in der Welt ist und Nachforschen damit
  günstiger wird (§7.1).

### 5.5 Rechte nach Anteil

| Anteil | Rechte |
| --- | --- |
| unter 25 % | Gewinnbeteiligung und Erlös bei einem Verkauf |
| ab 25 % | zusätzlich Sperrminorität: Verkauf an Konkurrenten blockieren |
| über 50 % | aktive Lenkung (Budget, Ausrichtung, Börsengang, Verkauf), Aufstocken, Eingliedern als Tochterfirma |

- Eine Minderheitsbeteiligung bringt nur finanziellen Gewinn, keinen Zugang zur
  Technologie.
- Erst als Tochterfirma (über 50 %) darf der ganze Konzern das Patent nutzen.

### 5.6 Erfolg und Scheitern

- **Erfolg:** Die Technologie gilt als erfunden bzw. verbessert; das Patent liegt beim
  Start-up (§7.2). Andere lizenzieren sie oder forschen günstiger nach.
- Ein erfolgreiches Start-up ohne beherrschenden Eigner wird zu einer neuen KI-Firma
  (§10) mit der Technologie als Startvorteil.
- **Scheitern:** Die Anteile sind wertlos. Der Mehrheitseigner erhält einen kleinen
  Forschungsbonus auf die Zieltechnologie (Parameter).

### 5.7 Eigene Ausgründungen

- Der Spieler kann ein eigenes Forschungsprojekt oder ein Forschungszentrum als Start-up
  abspalten und Anteile an Investoren verkaufen. So holt er Kapital herein und teilt das
  Risiko. Behält er über 50 %, bleibt es Tochterfirma.
- KI-Firmen können ebenfalls ausgründen.

## 6 Ansichten

- **Beteiligungen** (neue Hauptansicht, §14.1): Start-ups mit Phase, Kapitalbedarf,
  Einschätzung und Eignern; eigenes Portfolio; Empfehlungen der Strategieabteilung.
- **Organisation:** Hauptsitz mit Zentralabteilungen, Angestellten, Kosten, Trefferquote
  der Leitungen.
- **Strategie:** neues Strategiefeld „Beteiligungen“ (Budget für Start-ups und
  Übernahmen, Risikobereitschaft, Freigabegrenzen der Zentralabteilungen).
- Keine Spiellogik in der Oberfläche.

## 7 Technische Leitplanken

- **Determinismus:** eigener Zufallsstrom für die Entstehung von Start-ups und je
  Start-up für Fortschritt und Erfolg. Keine `HashMap`-Iteration.
- **Befehle (Vorschlag):** `SetHeadquarters`, `StaffDepartment`, `InvestInVenture`,
  `GrantVenture`, `SellVentureStake`, `SteerVenture`, `IntegrateVenture`, `SpinOff`.
  KI-Firmen und Abteilungen nutzen dieselben Befehle.
- **Daten:** z. B. `data/beteiligungen.yaml` (Abteilungen, Kosten, Epochenbezeichnungen,
  Häufigkeit, Phasen, Kapitalbedarf, Erfolgsquoten, Gehaltsfaktor `k`, Bewertungsfrist)
  und `data/startups/erfinder.yaml` (historische Erfinder je Technologie). Prüfregeln,
  Fehlerfälle und `docs/DATENFORMAT.md` nach `CLAUDE.md`.
- **Spielstand:** neue Formatversion mit Umwandlung (alte Stände ohne Hauptsitz und
  Start-ups), Beispielstand ergänzen.
- **Buchungen:** Beteiligungen als Finanzanlagen, Fördergeld als Aufwand, Erträge aus
  Ausschüttungen und Verkäufen; Bilanz bleibt ausgeglichen.

## 8 Meilensteine (Vorschlag)

| Nr. | Inhalt | Prüfbar durch |
| --- | --- | --- |
| ZA1 | Hauptsitz: Wahl, Verlegung, Wirkung auf Steuern und Personal | Szenario: Verlegung kostet, Steuer folgt dem neuen Land |
| ZA2 | Zentralabteilungen: Ressorts Strategie und Recht, Angestellte, Kosten, Freigabegrenze im Strategiefeld „Beteiligungen“ | Szenario: zu früh aufgebaute Abteilungen drücken die Werkstatt in die Verlustzone |
| ZA3 | Empfehlungen mit Begründung, Trefferquote, Gehaltsforderung exponentiell | Test der Gehaltsformel; Szenario: schwache Leitung empfiehlt häufiger Fehlgriffe |
| SU1 | Start-ups: Entstehung, Arten, Phasen, Finanzierungsrunden, Erfolg und Scheitern | Weltlauf: Anteil gescheiterter Start-ups im Richtwert; Determinismus |
| SU2 | Beteiligungen: Anteile, Fördergeld, Verwässerung, Rechte nach Anteil, Eingliedern als Tochterfirma, neue KI-Firma bei Erfolg | Szenario je Schwelle 25 % und 50 % |
| SU3 | Ausgründungen, KI-Investoren und KI-Firmen bieten mit | Weltlauf mit KI-Beteiligungen; Benchmark ohne deutlichen Leistungsverlust |

Jeder Meilenstein: Formeln vorher in `docs/FORMELN.md`, Tests, Eintrag in
`docs/FORTSCHRITT.md`, Bedienung nach `docs/BEDIENUNG.md`.

## 9 Offene Punkte

- **Zeitpunkt:** Vorschlag ZA1–ZA3 nach dem Manager-System (nach MA6), SU1–SU3 mit
  Stufe 3 (Kapitalmarkt). Der Auftraggeber legt den Zeitpunkt fest.
- Richtwerte für Kosten der Abteilungen, Kapitalbedarf je Phase und Häufigkeit schlägt
  Claude Code mit den Formeln vor.
