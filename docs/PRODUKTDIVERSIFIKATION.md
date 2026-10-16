# Vorgabe: Produktdiversifikation und Marktforschung

Stand: 15.10.2026 · Status: zur Umsetzung freigegeben

**Reihenfolge (Auftraggeber, 15.10.2026):** DM1–DM6 (Meilensteine dieser Vorgabe, von mir benannt) kommen **direkt nach RB4**, vor ZP1.
Gesamtreihenfolge: PD7–PD10 → MN6/PD1 → PD11 → Manager-Reste → PV1–PV8 → RB1–RB4 → E2E → DM1–DM6 → ZP1. WG1–WG5 bleiben zurückgestellt.

Bitte vor der Umsetzung den aktuellen Code-Stand prüfen und Namen, Strukturen und Rundenlogik an das Bestehende anpassen. Das betrifft vor allem das Nachfragemodell aus `docs/WELT_MARKT_BETRIEB.md`, die Produktentwicklung (Meilensteine M0–M7), die Strategieabteilung und das System der Maschinenangebote. Alte Spielstände müssen nicht kompatibel bleiben.

---

## 1. Ziel

Endprodukte unterscheiden sich künftig über ein kleines **Produktprofil**. Der Markt besteht aus Kundensegmenten, die diese Profile unterschiedlich bewerten. Der Spieler erfährt die Marktbedürfnisse über Marktforschung, entweder extern gegen Einmalzahlung oder intern über die Strategieabteilung, und stellt seine Produkte entsprechend ein.

Leitlinie: **wenig Merkmale, klare Entscheidungen.** Jedes Endprodukt lässt sich mit drei Zahlen und einem Preis beschreiben.

---

## 2. Merkmale der Endprodukte

Jedes Endprodukt hat genau drei Merkmale auf einer Skala von 0–100:

| Merkmal | Bedeutung | Beispiel Auto |
|---|---|---|
| **Qualität** | Haltbarkeit, Zuverlässigkeit, Verarbeitung | Zuverlässigkeit |
| **Leistung** | Was das Produkt kann, Ausstattung | Motorleistung, Ausstattung |
| **Effizienz** | Betriebskosten, Verbrauch, Wartungsaufwand | Verbrauch |

- Die Merkmale gelten für alle Endprodukte. Branchenspezifische Anzeigenamen sind optional und rein kosmetisch (Konfigurationsdatei).
- Der **Preis** ist kein Merkmal. Er wird wie bisher vom Spieler bzw. Manager gesetzt.
- Das Profil gehört zum **Produktstand eines Herstellers**. Zwei Firmen mit demselben Produkt können also unterschiedliche Profile haben.

### 2.1 Wie Werte entstehen
- **Grundprodukt bzw. neu erforschtes Produkt ohne Entwicklung:** Es startet mit der technologischen Grundlinie der aktuellen Epoche für alle drei Merkmale (siehe 4.2).
- **Neuentwicklung:** Bei M1 (Lastenheft) wird ein Zielprofil festgelegt. Bei M6 (Artikelfreigabe) steht das erreichte Profil fest. Es weicht je nach Team leicht vom Ziel ab (Vorschlag ±5 Punkte je Merkmal; ein besseres bzw. vollständigeres Team bedeutet kleinere Abweichung).
- **Weiterentwicklung:** Bei M1 (Optimierungsanalyse) wählt der Spieler ein Merkmal. Bei M3 (Freigabe) steigt es um einen Betrag, der von Projektgröße und Team abhängt (Vorschlag 4–10 Punkte).
- **Obergrenze:** Kein Merkmal kann über dem Technologiestand der Branche in der aktuellen Epoche liegen (`techMax(branche, jahr)`).

### 2.2 Kosten der Merkmale
Höhere Werte erhöhen die Herstellkosten, und zwar progressiv, je näher der Wert an `techMax` liegt.

Vorschlag:
```
kostenfaktor = 1 + Σ_m k_m · (wert_m / techMax)^2
herstellkosten = basisHerstellkosten · kostenfaktor
```
- Startwerte: `k_Qualität = 0,35`, `k_Leistung = 0,45`, `k_Effizienz = 0,30`.
- Höhere Zielwerte verlängern außerdem Entwicklungsprojekte. Das geht in die Prognose der Projektleiter bei M2 ein.
- Alle Werte kommen in eine Balancing-Konfiguration.

---

## 3. Kundensegmente

Je Endprodukt und Region gibt es drei Segmente:

| Segment | Charakter | Gewichtung Q / L / E / Preis (Startvorschlag) | Mindestschwelle |
|---|---|---|---|
| **Preisbewusst** | kauft fast nur über den Preis, achtet etwas auf Betriebskosten | 0,15 / 0,10 / 0,20 / 0,55 | Qualität ≥ 30 % der Erwartung |
| **Mittelklasse** | ausgewogenes Preis-Leistungs-Verhältnis | 0,25 / 0,25 / 0,20 / 0,30 | Qualität ≥ 60 % der Erwartung |
| **Premium** | zahlt für Qualität und Leistung | 0,35 / 0,40 / 0,10 / 0,15 | Qualität und Leistung ≥ 90 % der Erwartung |

- Segmentgrößen (Anteil an der Gesamtnachfrage) sind je Region und Epoche unterschiedlich und verschieben sich langsam. Startvorschlag: 40 / 45 / 15 %. Wohlhabende Regionen haben mehr Premium, ärmere mehr Preisbewusst. Bei aktivierter Inflation koppeln, sofern sinnvoll.
- Die Gewichte dürfen je Produkt leicht variieren (Konfiguration, ±0,05). Die Grundstruktur bleibt gleich.
- Kein Kano-Modell, keine weiteren Segmente. Ein Öko-Segment ist erst mit dem Themenfeld Umwelt vorgesehen.

---

## 4. Absatzverteilung

### 4.1 Grundsatz
Die **Gesamtnachfrage** je Produkt und Region kommt weiterhin aus dem bestehenden Nachfragemodell. Das neue System **verteilt** diese Nachfrage auf Segmente und innerhalb jedes Segments auf die Anbieter.

### 4.2 Erwartung (Marktmaßstab)
Merkmale werden relativ zur aktuellen Markterwartung bewertet:
```
erwartung_m(produkt, region, t) = max( absatzgewichteter Durchschnitt aller angebotenen Profile,
                                       grundlinie_m(branche, jahr) )
```
- `grundlinie` steigt mit dem technologischen Fortschritt der Epoche (Vorschlag ca. 1–2 Punkte pro Jahr, je Branche konfigurierbar).
- Dadurch veralten Produkte von selbst, und dauerhaftes Weiterentwickeln bringt einen Vorsprung.

### 4.3 Nutzen und Marktanteil je Segment
Vorschlag:
```
nutzen(p, s) = Σ_m w_s,m · (wert_p,m − erwartung_m) / 10
             − w_s,Preis · ln(preis_p / referenzpreis) · c_Preis
```
- `referenzpreis` = absatzgewichteter Marktpreis, Startwert `c_Preis = 4`.
- Liegt das Profil unter der Mindestschwelle eines Segments, bekommt das Produkt in diesem Segment keinen Absatz.
- Marktanteil innerhalb des Segments per Logit: `anteil_p = exp(nutzen_p) / Σ exp(nutzen)`.
- Lieferfähigkeit bleibt maßgeblich: Kann ein Anbieter nicht liefern, geht die Restnachfrage nach dem bestehenden Mechanismus an andere Anbieter.

### 4.4 Balancing-Prüfung
Simulationstests über mehrere Jahrzehnte mit KI-Firmen. Prüfkriterien:
- Kein einzelnes Profil gewinnt dauerhaft alle drei Segmente.
- Ein Billiganbieter und ein Premiumanbieter können beide profitabel sein.
- Ein Produkt ohne Weiterentwicklung verliert über ca. 5–10 Jahre spürbar Marktanteil.

---

## 5. Sichtbarkeit ohne Marktforschung

Ohne jede Studie sieht der Spieler:
- eigene Absätze, Preise und Profile,
- Gesamtabsatz des Marktes und Marktpreise der Konkurrenz,
- **nicht**: Segmentgrößen, Segmentgewichte, Mindestschwellen und Profile der Konkurrenz (Anzeige „?“).

---

## 6. Marktforschung extern (Einmalauftrag)

Für Spieler ohne Strategieabteilung oder für Regionen, die die eigene Abteilung nicht abdeckt.

### 6.1 Ablauf (analog zu den Maschinenangeboten)
1. Der Spieler wählt **Studientyp**, **Produkt** und **Region** und fragt an.
2. Innerhalb von ca. einer Woche Spielzeit kommen **1–3 Angebote** von Marktforschungsinstituten.
3. Die Angebote unterscheiden sich in **Preis**, **Dauer** (2–8 Wochen) und **Genauigkeit**:
   - grob: Werte als Spanne ±25 %
   - solide: ±10 %
   - präzise: ±3 %

   Präzise ist deutlich teurer. Die Verteilung ist nicht gleichmäßig, gelegentlich gibt es auch sehr gute oder sehr schlechte Angebote.
4. Der Spieler nimmt an oder lehnt ab. Bei Annahme wird **einmalig gezahlt**, und das Ergebnis kommt nach der Dauer.

### 6.2 Studientypen
- **Marktstudie** (Produkt × Region): Segmentgrößen, Gewichtungen, Mindestschwellen, aktuelle Erwartung je Merkmal.
- **Wettbewerbsanalyse** (Produkt × Region): Qualität, Leistung, Effizienz, Preis und Marktanteil der Konkurrenzprodukte.

### 6.3 Ergebnis
- Ergebnisse sind reine Daten, **keine Empfehlung**.
- Jede Studie ist eine Momentaufnahme mit Datum, zum Beispiel „Studie von 03/1985“. Nach 12 Monaten wird sie als veraltet markiert, bleibt aber einsehbar.
- Die Werte enthalten den Fehler der jeweiligen Genauigkeit. Der Fehler wird beim Erstellen einmal gewürfelt und gespeichert, damit dieselbe Studie nicht bei jedem Öffnen andere Werte zeigt.

---

## 7. Marktforschung intern (Strategieabteilung)

In die bestehende Strategieabteilung integrieren, **kein separates System**.

- Neue Rolle **Marktforscher**, per Button einstellbar wie die übrigen Rollen.
- Jeder Marktforscher deckt eine bestimmte Zahl von Produkt-Region-Kombinationen ab (Vorschlag 4). Die abgedeckten Kombinationen erhalten **jedes Quartal** automatisch aktualisierte Markt- und Wettbewerbsdaten.
- Die Genauigkeit hängt von Anzahl und Fähigkeit der Marktforscher ab. Bei Unterbesetzung wird es ungenauer.
- Die Leitung der Strategieabteilung **empfiehlt die nötige Anzahl Marktforscher**, passend zu den bisherigen Vorgaben zu Personalempfehlungen.
- **Empfehlungen** laufen über den bestehenden Empfehlungsweg der Strategieabteilung und der Manager. Beispiele:
  - „Modell X ist in der Mittelklasse zu teuer: Qualität +10 oder Preis −8 %.“
  - „Effizienz ist im Segment Preisbewusst unter der Erwartung, Weiterentwicklung empfohlen.“
- Jede Empfehlung enthält Begründung, Kosten und erwartete Absatzwirkung. Empfehlungen beruhen auf den (fehlerbehafteten) Daten der Abteilung und können deshalb danebenliegen.
- Externe Aufträge bleiben zusätzlich möglich.

---

## 8. Einbindung in die Produktentwicklung

| Meilenstein | Neu |
|---|---|
| Neuentwicklung **M1 Lastenheft** | Zielprofil (Q/L/E), Zielpreis, Zielsegment |
| Neuentwicklung **M2 Pflichtenheft** | Herstellkosten und Projektdauer für das Zielprofil |
| Neuentwicklung **M4 / M6** | Ist-Profil im Vergleich zu M2 |
| Weiterentwicklung **M1 Optimierungsanalyse** | Auswahl des Merkmals, das angehoben wird |
| Weiterentwicklung **M3 Freigabe** | neues Profil |

Die Produktvorschläge der Strategieabteilung enthalten künftig auch ein empfohlenes Zielprofil und Zielsegment.

---

## 9. KI-Firmen

- Jede KI-Firma bekommt je Branche eine Strategie: **Kostenführer**, **Mittelklasse** oder **Premium**. Eine Nische kann über eine Kombination abgebildet werden.
- KI-Firmen nutzen dieselben Institute und dieselbe interne Marktforschung mit denselben Kosten und derselben Ungenauigkeit. Der Spieler darf keinen Nachteil haben.

---

## 10. Vorprodukte: vorbereiten, aber deaktiviert

Das System für Vorprodukte wird **vollständig eingebaut, aber über einen Schalter deaktiviert**, damit es später nur noch eingeschaltet werden muss.

- Konfigurationsschalter `vorproduktMerkmale` mit Standardwert `false`.
- Vorprodukte haben genau **ein Merkmal: Güte** (0–100), je Hersteller und Produkt.
- **Lagerbestände** tragen eine durchschnittliche Güte. Beim Einlagern wird der mengengewichtete Mittelwert gebildet.
- **Wirkung, wenn aktiv:** Die durchschnittliche Güte der eingesetzten Vorprodukte verschiebt die Qualität des Endprodukts:
  ```
  qualitätsbonus = (gewichtete Input-Güte − 50) · f_Güte
  ```
  - Startwert `f_Güte = 0,1`, also maximal ±5 Punkte.
  - Der Bonus wirkt nur auf die produzierte Ware, nicht auf das entwickelte Profil.
- **Markt, wenn aktiv:** Industriekunden haben nur Preis und eine Mindestgüte. Es gibt keine Segmente.
- Güte entsteht wie Qualität bei Endprodukten über Entwicklung bzw. Weiterentwicklung.
- **Wenn deaktiviert:** Alle Vorprodukte gelten als Güte 50. Es gibt keinen Effekt und keine Anzeige in der Oberfläche. Das Datenmodell ist trotzdem vorhanden.
- Für beide Schalterstellungen gibt es Tests.

---

## 11. Oberfläche

- **Produktansicht:** Profil (Q/L/E) mit Erwartung des Marktes zum Vergleich und Herstellkostenfaktor.
- **Marktansicht je Produkt und Region:** Liste aller Anbieter mit Q/L/E, Preis und Marktanteil. Unbekannte Werte erscheinen als „?“ oder Spanne. Datenquelle und Alter werden angezeigt. Optional ein Diagramm Qualität × Leistung.
- **Marktforschung:** Menü „Studie beauftragen“ mit Angebotsliste (Preis, Dauer, Genauigkeit), dazu eine Liste vorhandener Studien mit Alter.
- Alle Werte, die auf Studien beruhen, zeigen ihre Unsicherheit als Spanne.

---

## 12. Nicht im Umfang (spätere Ausbaustufen)

- Produktvarianten wie Basis und Premium auf derselben Anlage (bis zu drei je Produkt)
- Öko-Segment bzw. Nachhaltigkeit (mit dem Themenfeld Umwelt)
- Einfluss der Maschinen- bzw. Fertigungsqualität auf das Profil
- Kano-Modell und weitere Merkmale

---

## 13. Akzeptanzkriterien

1. Jedes Endprodukt jedes Herstellers hat ein Profil (Q/L/E) und dazu passende Herstellkosten.
2. Die Absatzverteilung erfolgt je Segment nach Nutzen, Mindestschwelle und Preis. Die Gesamtnachfrage kommt aus dem bestehenden Modell.
3. Die Erwartung steigt über die Zeit, und Produkte ohne Weiterentwicklung verlieren Anteile.
4. Externe Studien sind beauftragbar: 1–3 Angebote, Einmalzahlung, Dauer, Genauigkeit, gespeicherte Ergebnisse mit Alter.
5. Die Rolle Marktforscher in der Strategieabteilung liefert quartalsweise Daten und Empfehlungen und gibt eine Personalempfehlung.
6. Zielprofile sind in der Produktentwicklung bei M1 bzw. bei der Optimierungsanalyse wählbar und werden bei M2, M4 und M6 ausgewiesen.
7. KI-Firmen verfolgen unterschiedliche Profilstrategien und nutzen dieselben Mechaniken.
8. Vorprodukt-Güte ist implementiert, per Schalter deaktiviert und in beiden Zuständen getestet.
9. Die Balancing-Simulation (Abschnitt 4.4) ist bestanden.
10. Alle Parameter liegen in einer Balancing-Konfiguration und nicht im Code verstreut.

---

## Meilensteine (von mir vorgeschlagen, Vorgabe nennt keine)

| Nr. | Inhalt |
|---|---|
| DM1 | Profil (Q/L/E) je Produktstand und Hersteller, Grundlinie/techMax, Kostenfaktor, Erwartung; Daten, Prüfregeln, Spielstand |
| DM2 | Segmente und Absatzverteilung (Nutzen, Schwellen, Logit) auf das bestehende Nachfragemodell; Sichtbarkeit „?“ |
| DM3 | Externe Marktforschung: Anfrage, 1–3 Angebote, Studien mit gewürfeltem Fehler, Alterung; Sichten, Oberfläche |
| DM4 | Interne Marktforschung: Rolle Marktforscher, Quartalsdaten, Personalempfehlung, Empfehlungen mit Begründung |
| DM5 | Einbindung in die Produktentwicklung (M1/M2/M4/M6, Weiterentwicklung) und Produktvorschläge; KI-Profilstrategien |
| DM6 | Vorprodukt-Güte mit Schalter `vorproduktMerkmale` (beide Stellungen getestet), Oberfläche, Balancing nach 4.4 |
