# Vorgabe Manager-System

Stand: 06.10.2026. Erarbeitet mit dem Auftraggeber, abgeglichen mit
`claude/architektur-vorschlag` bis M39 (Commit `6f0838c`). Ergänzt Lastenheft §5.5,
§5.6 und §18.5. Bei Widerspruch gilt diese Datei vor §5.5/§5.6.

## 0 Entscheidungen des Auftraggebers (06.10.2026)

1. Das Manager-System wird aus Stufe 2 vorgezogen und **nach M41** umgesetzt
   (Meilensteine MA0–MA6, Abschnitt 11).
2. **Weltebene = Vorstandsebene.** Es gibt keine eigene Weltebene unter dem Vorstand.
3. **KI-Firmen** stellen Manager aus demselben Pool ein und werben ab (ab MA6). Ihr
   Verhalten wird weiter über Kompetenz und Aggressivität gesteuert.
4. **Fristablauf:** Bei einem unbeantworteten Anliegen bleibt alles, wie es ist. Nur bei
   Themen mit „Entscheide selbst“ setzt der Manager seine Empfehlung um.
5. **Fähigkeiten** sieht der Spieler als Stufen, nicht als Zahlen. Ein starkes
   Personalressort macht sie genauer sichtbar.
6. **Logistik-Stelle:** in den Daten angelegt, aktiv erst mit der Logistik (Stufe 2).
7. **Gehaltsfaktoren** 1,5 bis 15 × Akademikerlohn (Abschnitt 3) sind bestätigt.
8. **Budget:** Jeder Manager hat ein Standardbudget je Entscheidung und je Jahr. Darunter
   entscheidet er ohne Rückfrage (Abschnitt 5).

## 1 Ziel

Anfangs entscheidet der Spieler alles selbst. Mit wachsender Firma stellt er Manager
ein, die die Routine ihres Bereichs übernehmen und nur fragen, wenn eine Entscheidung
über ihr Budget oder ihre Befugnis hinausgeht. Diese Fragen heißen **Anliegen**.
Besetzt der Spieler eine höhere Ebene, laufen die Anliegen der unteren Ebenen dort auf;
der Spieler hört nur noch die größeren Themen der obersten besetzten Stelle. An der
Spitze steht ein CEO mit Strategieauftrag und regelmäßiger Strategierücksprache.

Später (Tochterfirmen, Stufe 2; Kapitalmarkt, Stufe 3) wechselt das Spiel in eine
Personenperspektive: Der Spieler ist Eigentümer mehrerer Firmen, jede mit eigenem CEO.
Das Datenmodell muss das schon jetzt zulassen (Abschnitt 9).

## 2 Begriffe

| Deutsch (Daten, Texte) | Code (Vorschlag) | Bedeutung |
| --- | --- | --- |
| Einheit | `OrgUnit` | Standort, Land, Kontinent oder Konzern einer Firma |
| Stelle | `Position` | Platz in einer Einheit: Leitung oder Fachstelle eines Bereichs |
| Leitung | `Head` | Generalist einer Einheit |
| Fachstelle | `Specialist` | Stelle für einen Bereich (Produktion, Einkauf …) |
| Bereich | `Function` | Produktion, Einkauf und Lager, Vertrieb und Marketing, Personal, Logistik, Forschung, Finanzen |
| Manager | `Manager` | Einzelperson mit Fähigkeiten und Gehalt |
| Anliegen | `Concern` | Frage eines Managers mit Optionen, Empfehlung und Frist |
| Option | `ConcernOption` | Liste von Befehlen mit geschätzter Wirkung |
| Auslöser | `Trigger` | Prüfung, die ein Anliegen oder eine Routineentscheidung erzeugt |
| Budget | `Budget` | Höchstbetrag je Entscheidung und je Jahr |
| Befugnis | `Authority` | Budget plus erlaubte Themen einer Stelle |
| Strategieauftrag | `Mandate` | Leitlinie, Ziele und Grenzen für den CEO |
| Strategierücksprache | `StrategyReview` | Bericht und Vorschläge des CEO im gewählten Abstand |

In `docs/GLOSSAR.md` übernehmen. Achtung: „Region“ bedeutet seit M34 zusammengefasste
Kleinstaaten; die Ebene über dem Land heißt deshalb **Kontinent** (Katalog:
`continents`, 6 Einträge).

## 3 Ebenen und Stellen

| Ebene | Leitung | Fachstellen | Prüfrhythmus | Typische Themen | Gehalt Fach / Leitung (× Akademikerlohn des Einsatzlandes) |
| --- | --- | --- | --- | --- | --- |
| 1 Standort | Werks-, Betriebs-, Labor-, Kraftwerks- oder Niederlassungsleitung | je nach Standorttyp (unten) | wöchentlich | Auslastung, Engpässe, Preis, Lohnaufschlag, Rezept, Anlagengröße, Stilllegen | 1,5 / 2,5 |
| 2 Land | Landesleitung | Produktion, Einkauf und Lager, Vertrieb und Marketing, Personal, Logistik, Forschung | monatlich | Verteilung zwischen Standorten, Grundstück und neuer Standort im Land, Preis- und Werbelinie | 3 / 4 |
| 3 Kontinent | Kontinentvorstand | wie Land | monatlich | Markteintritt in Länder, Verlagern oder Verkaufen, Lagerstätten, Kraftwerke, Versteigerungen | 5 / 7 |
| 4 Vorstand | CEO | Ressorts Produktion und Technik, Einkauf und Logistik, Vertrieb und Marketing, Personal, Forschung und Entwicklung, Finanzen | quartalsweise | Portfolio und Ketten, Forschungsschwerpunkte, Kredite, Kaufangebote, Vorgaben | 10 / 15 |
| (5 Eigentümer) | Spieler | – | Rücksprachen | Strategieauftrag, später Beteiligungen | – |

Fachstellen je Standorttyp (`SiteType`, in den Daten änderbar):

| Standorttyp | Leitung | Fachstellen |
| --- | --- | --- |
| Werk (`Factory`) | Werksleitung | Produktion; Einkauf und Lager; Vertrieb und Marketing; Personal; Logistik |
| Förderstätte (`Extraction`) | Betriebsleitung | Produktion (Förderung); Vertrieb und Marketing; Personal |
| Kraftwerk (`PowerPlant`) | Kraftwerksleitung | Produktion; Einkauf und Lager |
| Lager (`Warehouse`) | Lagerleitung | Logistik |
| Niederlassung (`SalesOffice`) | Niederlassungsleitung | Vertrieb und Marketing |
| Forschungszentrum (`ResearchCenter`) | Laborleitung | – |

Regeln:

- Einheiten Land und Kontinent entstehen, sobald die Firma dort einen Standort hat.
- Eine Leitung ohne besetzte Fachstellen erledigt alle Bereiche ihrer Einheit selbst,
  mit einem Abschlag auf ihre Fachkompetenz (Parameter, Vorschlag 20 %).
- Schalter je Leitung „Leitung stellt ein“: Sie besetzt ihre Fachstellen selbst aus dem
  Pool, nach ihrem Urteilsvermögen und im Rahmen ihres Budgets (Lastenheft §5.5).
- Unbesetzte Stellen werden bei der Weiterleitung übersprungen (Abschnitt 6.5).
- Solange der CEO unbesetzt ist, führt der Spieler selbst.

## 4 Manager

### 4.1 Merkmale

| Merkmal | Wertebereich | Wirkung |
| --- | --- | --- |
| Fachkompetenz je Bereich | 0–100 | Güte der Routine und Genauigkeit der Prognosen |
| Erkennen | 0–100 | Ob und wie früh er eine Lage bemerkt |
| Urteilsvermögen | 0–100 | Wie oft seine Empfehlung die tatsächlich beste Option ist; Entscheidungen über weitergeleitete Anliegen |
| Führung | 0–100 | Nur Leitungen: hebt die Fähigkeiten der Unterstellten leicht |
| Risikoneigung | 0–100 (vorsichtig – mutig) | Bevorzugt Abwarten oder Ausbau |
| Fragefreude | 0–100 (zurückhaltend – gesprächig) | Wie oft er bei Themen **ohne Ausgabe** (Preis, Rezept) fragt, statt still nach Vorgabe zu entscheiden; innerhalb seines Budgets fragt er nie |
| Zufriedenheit | 0–100 | Sinkt bei Gehalt unter Markt, schlechtem Ergebnis der Einheit und häufigem Übergehen seiner Empfehlung; niedrig = Kündigungsgefahr |

Außerdem: Name, Heimatland, Gehaltsforderung, Arbeitgeber, Stelle, Erfahrung je Bereich.
Manager sind alterslos (Lastenheft §5.5).

### 4.2 Markt

- Pool je Kontinent. Größe nach den Akademikern der Länder des Kontinents, Fähigkeiten
  nach Verteilungen aus den Daten, Gehaltsforderung nach Fähigkeiten und Lohnniveau.
- Der Pool erneuert sich monatlich; Kandidaten verschwinden auch, wenn andere Firmen sie
  einstellen (ab MA6 KI-Firmen).
- Namen aus einer Datendatei je Sprachraum (wie `data/ki/namen.yaml`).

### 4.3 Einstellen, Entlassen, Kosten

- Neue Befehle für Einstellen, Versetzen und Entlassen. Entlassen kostet eine Abfindung
  (Monatsgehälter, Parameter).
- Gehälter sind Kostenart Personal. Kostenstelle: der Standort bei Standortstellen, sonst
  Gemeinkosten (`Overhead`) der Firma; das Controlling weist Managementkosten je Ebene aus.
- Erfahrung: Die Fachkompetenz steigt langsam mit der Zeit im Bereich, bis zu einer
  persönlichen Obergrenze.
- Abwerbung (ab MA6): Andere Firmen bieten guten Managern mehr. Dann entsteht ein Anliegen
  der zuständigen Personal-Stelle: Gegenangebot oder gehen lassen.

### 4.4 Anzeige der Fähigkeiten

Fähigkeiten erscheinen als fünf Stufen („schwach“, „mäßig“, „solide“, „stark“,
„herausragend“). Die angezeigte Stufe ist eine Schätzung mit Unschärfe; je höher die
Fachkompetenz des besetzten Personalressorts, desto genauer. Der Kern liefert der
Oberfläche nur die Stufe, nie die Zahl.

## 5 Budget je Manager

Jeder Manager hat zwei Grenzen: einen Höchstbetrag **je Entscheidung** und einen
Höchstbetrag **je Kalenderjahr**. Liegt eine Entscheidung unter beiden, trifft er sie
ohne Rückfrage. Überschreitet sie eine davon, wird sie ein Anliegen an die nächste
besetzte Stelle darüber.

### 5.1 Was zählt

| Art der Entscheidung | Angerechneter Betrag | Beispiele (Befehle) |
| --- | --- | --- |
| Einmalige Ausgabe | voller Betrag | `BuildFacility`, `DevelopDeposit`, `FoundSite`, `FoundSiteOnPlot` (Kauf), `BuyPlot`, `MakeOffer`, Gebot bei Versteigerung |
| Laufende Mehrkosten | Mehrkosten für 12 Monate | `SetWagePremium` (höher), `SetPurchase` (höherer Höchstpreis), `SetAdvertising` (höher), Pacht bei `FoundSiteOnPlot` |
| Abgabe von Vermögen | Buchwert | `SellFacility`, `MothballFacility`, Annahme eines Kaufangebots (`AnswerOffer`) |
| Steuerung ohne Ausgabe | zählt nicht | `SetPrice`, `SetSale`, `SetProduction`, `SetResearch`, `SetDevelopment`, Rezeptwechsel, solange die Vorgaben eingehalten sind |

`TakeLoan` und `RepayLoan` darf nur das Ressort Finanzen oder der CEO, im Rahmen des
Strategieauftrags.

### 5.2 Standardwerte

Bezugsgröße ist der Umsatz der Einheit in den letzten 12 Monaten; feste Dollarbeträge
passen nicht über 200 Jahre.

| Stelle | Je Entscheidung | Je Jahr | Bezugsgröße |
| --- | --- | --- | --- |
| Fachstelle Standort | 2 % | 5 % | Umsatz des Standorts |
| Leitung Standort | 5 % | 10 % | Umsatz des Standorts |
| Fachstelle Land | 3 % | 6 % | Umsatz der Firma im Land |
| Landesleitung | 5 % | 12 % | Umsatz der Firma im Land |
| Fachstelle Kontinent | 3 % | 8 % | Umsatz der Firma im Kontinent |
| Kontinentvorstand | 6 % | 15 % | Umsatz der Firma im Kontinent |
| Vorstandsressort | 3 % | 8 % | Konzernumsatz |
| CEO | 8 % | 20 % | Konzernumsatz, höchstens Investitionsbudget des Strategieauftrags |

- Sockel: mindestens ein eigenes Jahresgehalt je Entscheidung und drei je Jahr.
- Standorte ohne Umsatz (Labor, Kraftwerk, Lager): Bezugsgröße sind ihre Jahreskosten.
- Alle Werte in `data/`, als `annaeherung: true` gekennzeichnet. Wie der Umsatz der
  letzten 12 Monate je Einheit ermittelt wird (das Hauptbuch hält heute nur laufenden und
  letzten Monat/Jahr je Kostenstelle), schlägt Claude Code in `docs/FORMELN.md` vor.

### 5.3 Regeln

- **Anpassen:** Der Spieler ändert das Budget je Stelle oder per Vorgabe für alle Stellen
  eines Typs, z. B. „alle Werksleitungen in Europa: 3 % / 8 %“. Vererbung wie bei
  anderen Vorgaben (Abschnitt 7). 0 heißt „immer fragen“.
- **Deckel:** Das Budget einer Stelle ist nie höher als das ihrer Leitung. Was eine
  Fachstelle ausgibt, zählt nicht zusätzlich auf das Budget der Leitung.
- **Jahreswechsel:** Am 1. Januar beginnt das Jahresbudget neu; Unverbrauchtes verfällt.
- **Aufgebraucht:** Jede weitere zählende Entscheidung wird ein Anliegen; der Manager fragt
  dabei einmal, ob sein Budget erhöht werden soll.
- **Freigaben:** Gibt eine höhere Stelle oder der Spieler eine Option frei, zählt der
  Betrag nicht auf das Budget des fragenden Managers.
- **Anzeige:** Organisationsansicht je Stelle: Budget, Verbrauch im Jahr, zuletzt selbst
  getroffene Entscheidungen mit Betrag.

## 6 Routine und Anliegen

### 6.1 Grundsatz

Manager handeln nur über Befehle (`Command`), geprüft wie Spieler und KI. Sie fragen
nicht nach Kalender, sondern wenn ein Auslöser anschlägt und die Entscheidung Budget
oder Befugnis übersteigt.

### 6.2 Ablauf je Prüftermin

1. **Prüftermin** nach Rhythmus der Ebene, gestaffelt über die Tage wie die KI-Firmen.
2. **Auslöser prüfen:** Katalog in den Daten (Abschnitt 12), mit Parametern wie „Anlage
   seit 3 Monaten unter 50 % ausgelastet“. Ob der Manager die Lage bemerkt, entscheidet
   *Erkennen* über seinen eigenen Zufallsstrom.
3. **Optionen bilden:** Die Entscheidungsbausteine aus MA0 liefern 2–4 Optionen, je Option
   Befehlsliste, Kosten, angerechneter Budgetbetrag und geschätzte Ergebniswirkung als
   Bandbreite. Schätzfehler und Bandbreite hängen von der Fachkompetenz ab.
4. **Wählen:** Der Manager ordnet die Optionen nach seiner Schätzung, Risikoneigung und
   den Vorgaben. Mit Wahrscheinlichkeit nach *Urteilsvermögen* trifft seine Empfehlung
   die nach den wahren Werten beste Option.
5. **Im Rahmen** (Budget, Themen, Vorgaben): ausführen und im Protokoll „Was das
   Management entschieden hat“ vermerken.
6. **Sonst Anliegen** mit Lage (Kennzahlen), Optionen, Empfehlung mit kurzer Begründung
   (Textschlüssel + Parameter), Frist und Verhalten bei Fristablauf.

### 6.3 Antworten des Spielers

- Option wählen: wird als Befehlsliste ausgeführt (Journal, Replay).
- „Entscheide selbst“ für dieses eine Anliegen.
- „Zu diesem Thema nicht mehr fragen“: Das Thema wird in die Befugnis der Stelle
  aufgenommen (das Budget bleibt).
- Ablehnen: dasselbe Thema kommt erst nach einer Sperrfrist wieder.
- Eine direkte Entscheidung des Spielers in einer Ansicht hat Vorrang und schließt
  passende offene Anliegen.

### 6.4 Rückmeldung zu Folgen

Nach einer Wirkzeit (Parameter, z. B. 3 Monate) meldet der Manager die tatsächliche
Wirkung gegenüber seiner Prognose. Falsche Entscheidungen brauchen keine eigene
Strafe: Die Befehle wirken in der Simulation.

### 6.5 Weiterleitung

- Kette: Fachstelle → Leitung derselben Einheit → Fachstelle desselben Bereichs eine Ebene
  höher → deren Leitung → … → Vorstandsressort → CEO → Spieler.
- Unbesetzte Stellen werden übersprungen.
- Die erste besetzte Stelle mit ausreichendem Budget und Befugnis entscheidet selbst, mit
  ihrem eigenen Urteilsvermögen. Sonst gibt sie das Anliegen mit ihrem Kommentar weiter.
- Beim Spieler erscheint das Anliegen als Anliegen der obersten besetzten Stelle, mit dem
  Weg dorthin („von Vertrieb Werk Essen über Landesleitung Deutschland“).
- Häufen sich gleiche Anliegen aus mehreren Einheiten, macht die höhere Stelle daraus ein
  eigenes strategisches Anliegen.

### 6.6 Menge begrenzen

- Sperrfrist je Thema und Stelle; höchstens zwei offene Anliegen je Stelle (Parameter).
- Gleiche Anliegen mehrerer Standorte erscheinen gebündelt mit „für alle übernehmen“.
- Einstellung „Runde anhalten bei Anliegen: wichtige / alle / nie“. „Weiterlaufen bis …“
  (M26) hält entsprechend an, damit keine Frist übersprungen wird.
- Viele unbetreute Einheiten erzeugen viele Fragen. Das ist gewollt: Es ist der Anreiz,
  die nächste Ebene zu besetzen.
- Für Bereiche mit besetzter Stelle entfallen die passenden Hinweise aus
  `views/hints.rs` („Zu erledigen“).

## 7 Vorgaben (Lastenheft §5.6)

- `policy::Scope` bekommt weitere Geltungsbereiche: Standort, Kontinent, Konzern (=
  Vorstand), später Tochterfirma. Vererbung von allgemein nach speziell bleibt.
- Neue Strategiefelder zuerst: Preis (Premium, Marktpreis, Kampfpreis, Mindestmarge),
  Lager (Mindest- und Höchstreichweite), Personal (Lohnaufschlag-Linie), Eigenfertigung
  oder Zukauf, Investitionsbudget, Liquiditätsreserve, Budget und Befugnis je Stellentyp.
- Die Ansicht zeigt je Stelle, welche Vorgabe gilt und von welcher Ebene sie stammt.
- Manager können Vorgabe-Änderungen als Anliegen vorschlagen.

## 8 Vorstand, CEO, Strategierücksprache

### 8.1 Strategieauftrag

| Teil | Beispiele |
| --- | --- |
| Leitlinie | Wachstum, Ertrag, Sicherheit, Marktführerschaft in einer Warengruppe |
| Ziele | Umsatzwachstum % je Jahr, Umsatzrendite %, Eigenkapitalquote %, Rang |
| Grenzen | Investitionsbudget je Jahr, höchste Verschuldung, Liquiditätsreserve, gesperrte Branchen oder Länder |
| Rücksprache | monatlich, quartalsweise, halbjährlich oder jährlich |

### 8.2 Ablauf der Rücksprache

1. Fällig zum gewählten Termin; die Runde hält dort an.
2. Bericht: Ziele gegen Ist, Ergebnis je Kontinent und Warengruppe, größte Chancen und
   Risiken (aus Controlling, Markt, Rang).
3. Ein bis drei strategische Anträge, je als Anliegen mit Optionen.
4. Der Spieler genehmigt, ändert oder lehnt ab und kann den Auftrag anpassen.

Zwischen den Rücksprachen meldet der CEO nur Dringendes: Kaufangebote über einer
Schwelle, drohende Zahlungsunfähigkeit, Abwerbung eines Vorstands.

## 9 Vorbereitung Personenperspektive

- Stellen hängen an einer Firma (`CompanyId`), nicht am Spieler.
- Die Zustellung eines Anliegens endet an einem Empfänger: einer Stelle oder dem
  Eigentümer (`Holder::Player` ist schon von der Firma getrennt).
- Tochterfirmen (Stufe 2) bekommen eigenes Management bis zum CEO; ihre Anliegen gehen an
  den CEO der Mutter, wenn besetzt.
- Ein Strategieauftrag je gesteuerter Firma; Personenebene mit Vermögen, Beteiligungen,
  Dividende, Abberufung eines CEO folgt mit Stufe 3.
- Manager sind ein eigener Bestand im `GameState`, nicht Teil einer Firma, damit sie
  zwischen Firmen (auch KI) wechseln können.

## 10 Technische Leitplanken

- **Determinismus:** eigener Zufallsstrom je Manager (abgeleitet aus Seed + Manager-ID) und
  einer für den Managermarkt. Keine `HashMap`-Iteration. Gleicher Seed + gleiche Befehle =
  gleicher Zustand, auch mit Anliegen.
- **Befehle (Vorschlag):** `HireManager`, `MoveManager`, `DismissManager`,
  `AnswerConcern { concern, answer }`, `SetBudget`, `SetAuthority`, `SetHiringByHead`,
  `SetMandate`, `SetReviewInterval`, `SetConcernHalt`. Ausführungen durch Manager laufen
  als normale Befehle im Namen der Firma; das Journal vermerkt die auslösende Stelle.
- **Tagesablauf:** Manager-Prüfungen in Schritt 2 („Befehle anwenden“) nach den
  Spielerbefehlen und vor den KI-Firmen.
- **Daten:** neue Datei, z. B. `data/management.yaml` (Ebenen, Stellen je Standorttyp,
  Gehaltsfaktoren, Budget-Standardwerte, Prüfrhythmus, Auslöser mit Parametern,
  Fähigkeitsverteilungen) und `data/manager/namen.yaml`; Texte in
  `data/texte/de/`. Prüfregeln und Fehlerfälle nach `CLAUDE.md`; `docs/DATENFORMAT.md`
  pflegen.
- **Spielstand:** neue Formatversion mit Umwandlung (alte Stände laden ohne Manager,
  Spieler führt alles selbst), Beispielstand ergänzen.
- **Sichten:** Organisation (Organigramm, Stellen, Budget, Verbrauch, Protokoll),
  Managermarkt, Postfach Anliegen (Zähler im Kopfbereich), Strategie, Rücksprache.
  Keine Spiellogik in der Oberfläche.
- **Weltlauf:** `wsim-cli run --welt` und Protokolle müssen ohne Manager unverändert
  bleiben, solange KI-Firmen keine Manager haben (bis MA6).

## 11 Meilensteine

| Nr. | Inhalt | Prüfbar durch |
| --- | --- | --- |
| MA0 | Entscheidungsbausteine: Prüfungen in `ai.rs` (`operate`, `expand`, `retire`, `restart_where_short`, `supply_own`, `advertise`, `research_plan`, `development_target`, `manage_cash`, `open_deposit`, `own_power`, `diversify`, `pioneer` …) liefern Vorschläge (Befehlsliste, Kosten, geschätzte Wirkung). KI-Firmen führen weiter den besten sofort aus. | KI-Verhalten bitgleich: Determinismus-Tests, `lauf.rs` und Weltlauf-Protokoll unverändert |
| MA1 | Stellen und Manager: Datenmodell, Pool je Kontinent, Einstellen/Versetzen/Entlassen, Gehaltsbuchung, Organisationsansicht mit Organigramm; Standortstellen erledigen Routine nach Vorgaben | Szenario: Werk mit Werksleitung produziert und verkauft ohne Spielerbefehle; Bilanz ausgeglichen |
| MA2 | Anliegen auf Standortebene: Auslöser-Katalog, Budget je Stelle mit Standardwerten, Anliegen mit Optionen, Prognose, Empfehlung und Frist, `AnswerConcern`, Postfach, Runden-Halt, Bündelung, Rückmeldung zu Folgen | Szenario: mit Budget legt die Werksleitung eine schwache Anlage still, ohne Budget entsteht ein Anliegen; Fristablauf ändert nichts |
| MA3 | Land und Kontinent: höhere Ebenen, Weiterleitung, Budget-Vorgaben je Stellentyp, Deckel durch die Leitung, strategische Anliegen aus gehäuften Meldungen | Szenario: Anliegen landet bei der Landesleitung, nicht beim Spieler; ohne Budget dort beim Spieler mit Weg |
| MA4 | Strategieansicht: neue Geltungsbereiche, Strategiefelder Preis, Lager, Personal, Eigenfertigung, Investition, Liquidität; Anzeige „gilt von welcher Ebene“ | Vererbungs- und Überschreibungstests |
| MA5 | Vorstand und CEO: Ressorts, Strategieauftrag, Strategierücksprache | Szenario: quartalsweise Rücksprache hält die Runde an, Bericht stimmt mit Controlling überein |
| MA6 | Lebendiger Managermarkt: Erfahrung, Zufriedenheit, Kündigung, Abwerbung, KI-Firmen stellen ein | Weltlauf mit Managern; Benchmark mit 1 000 KI-Firmen ohne deutlichen Leistungsverlust |

Jeder Meilenstein: Formeln vorher in `docs/FORMELN.md`, Tests, Eintrag in
`docs/FORTSCHRITT.md`, Bedienung nach `docs/BEDIENUNG.md`.

## 12 Auslöser-Katalog (erste Fassung)

| Stelle | Auslöser | Optionen | Vorlage im Code |
| --- | --- | --- | --- |
| Standort Produktion | Anlage lange schwach ausgelastet | stilllegen; verkaufen; Preis senken; abwarten | `retire`, `MothballFacility`, `SellFacility` |
| Standort Produktion | Engpass durch Vorprodukt (`Limit::Input`) | Einkauf erhöhen; eigene Anlage bauen; anderes Rezept | `supply_own`, `best_recipe` |
| Standort Produktion | besseres Rezept oder Verfahren verfügbar | umstellen; beibehalten | `best_recipe` |
| Standort Produktion | Ausbau lohnt sich, freie Fläche vorhanden | Anlage in passender Größe bauen; abwarten | `expand`, `BuildFacility` (Größe, M36) |
| Standort Einkauf | Vorprodukt knapp oder teuer | Höchstpreis anheben; Vorrat aufbauen; staatlicher Markt | `SetPurchase` |
| Standort Vertrieb | Lager wächst | Preis senken; Händler zulassen; Produktion drosseln | `piling`, `SetSalesPolicy` |
| Standort Vertrieb | ausverkauft, Preis weit über Kosten | Preis erhöhen; Kapazität ausbauen | `SetPrice`, `BuildFacility` |
| Standort Personal | Personalmangel (`Limit::Labor`) | Lohnaufschlag erhöhen; automatisieren | `next_wage_premium`, `SetAutomation` |
| Standort Leitung | Grundstück gepachtet, Kauf günstiger als Pacht | kaufen; weiter pachten | `BuyPlot` (M35) |
| Labor / Forschung | Forschung oder Weiterentwicklung abgeschlossen | nächstes Ziel aus 2–3 Kandidaten | `research_plan`, `development_target`, `SetDevelopment` (M37) |
| Land / Kontinent | Nachfrage ohne Angebot, Engpass im Markt | Standort auf Grundstück gründen (Kauf oder Pacht); Lagerstätte erschließen; Kraftwerk | `opportunity`, `FoundSiteOnPlot`, `open_deposit`, `own_power` |
| Land / Kontinent | Versteigerung eines Standorts einer Pleitefirma | mitbieten bis Betrag X; nicht bieten | Versteigerung (M38) |
| Land / Kontinent | Kaufangebot einer anderen Firma | annehmen; Gegenangebot; ablehnen | `AnswerOffer` |
| Vorstand Finanzen | Kasse reicht nicht oder liegt brach | Kredit; Sondertilgung; Investitionen strecken | `manage_cash`, `TakeLoan`, `RepayLoan` |
| CEO | neue Kette oder Epoche lohnt sich | einsteigen (Pionier); abwarten | `diversify`, `pioneer` |
| Personal (alle Ebenen) | Manager wird abgeworben (ab MA6) | Gegenangebot; gehen lassen | – |

## 13 Was Claude Code selbst vorschlägt

In `docs/OFFENE_PUNKTE.md` (neuer Abschnitt, je mit Vorschlag) bzw. `docs/FORMELN.md`:

- Formeln für Schätzfehler, Erkennen, Urteilsvermögen, Erfahrung, Zufriedenheit, Pool.
- Umsatz der letzten 12 Monate je Einheit (Abschnitt 5.2).
- Wie Manager-Fähigkeiten bei KI-Firmen wirken (ab MA6), ohne Sonderregeln (§10).
- Abweichungen von dieser Vorgabe, wenn der Code es nahelegt, mit Begründung.
