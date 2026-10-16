# Vorgabe: Rundenbericht überarbeiten (RB1–RB4)

Stand: 15.10.2026 · Auftraggeber: Sebastian · Ersetzt Lastenheft §13.2 (dort verweist §13.2 auf dieses Dokument).

**Reihenfolge (Auftraggeber, 15.10.2026, geändert am 16.10.2026):** RB1–RB4 kommen **vor der Gesamtprüfung E2E** (direkt nach der Auswertung MN6/PD1); danach E2E, DM1–DM6, ZP1.
Gesamtreihenfolge: PD7–PD10 → Auswertung MN6/PD1 → PD11 → Manager-Reste → PV1–PV8 → RB1–RB4 → E2E → DM1–DM6 → ZP1.
WG1–WG5 bleiben zurückgestellt.

## Ziel

Der Rundenbericht soll **schlanker, aber zielgerichteter** werden. Er soll auf einen Blick ein gutes Bild der letzten Runde geben, ohne zu erschlagen. Das Wichtigste steht oben. Details gibt es nur per Klick in den jeweiligen Detailansichten.

Vorher bitte die bestehende Rundenbericht-Ansicht und ihre Datenquellen prüfen und umbauen statt neu zu bauen, wo es sinnvoll ist. Alte Spielstände müssen nicht kompatibel bleiben.

## Zielbild

```
RUNDE 14 · Q2 1912                                     [Enter] weiter
Ergebnis  +8.200 (▲ 12 %)          Kasse 312.000    Privat +1.900 ▲

PRODUKTE      Prod.  Verk.   Umsatz  Deckungsb.  Anteil Land  Kont.  Welt
Stahlträger   1.200  1.150   92.000  +21.400 ▲   DE 18 %       6 %   2,1 %
Schrauben     8.000  6.100   30.500   +4.900 ▼   DE 31 %       9 %   1,4 %
Fahrräder       300    300   45.000   −3.800 ▼   DE  4 %       1 %   0,2 %
+ 2 weitere
Summe                       171.900  +22.500
Fixkosten                            −14.300

AUSLASTUNG
▸ Werk Essen    ████████░░  88 %
▾ Werk Bochum   ████░░░░░░  41 %  Material fehlte
    Anlage Schrauben 1   ███░░░░░░░  30 %  Material fehlte
    Anlage Schrauben 2   ██████░░░░  58 %  Lager voll
    Anlage Draht 1       –  in Umstellung (noch 3 Wochen)

BITTE ANSEHEN (3)
! Fahrräder: negativer Deckungsbeitrag, Preis unter Herstellkosten
! Liquidität reicht voraussichtlich 6 Wochen
! Konkurrent senkt Preis für Schrauben in Deutschland um 15 %

4 weitere Meldungen im Meldungsfach →
```

## 1. Kopfzeile

- Rundennummer und Zeitraum (passend zur Rundenlänge).
- Ergebnis der Runde mit Trendpfeil und Veränderung zur Vorrunde in Prozent.
- Kassenstand.
- Veränderung des Privatvermögens der Spielerfigur in einer Zeile bzw. einem Feld (ersetzt den bisherigen Abschnitt „Person“; Ursachen per Klick in der Personenansicht).
- Hinweis „[Enter] weiter“ sichtbar.

## 2. Produkte (Kernstück)

Eine Zeile je Produkt mit:

| Spalte | Inhalt |
|---|---|
| Prod. | in der Runde produzierte Menge |
| Verk. | in der Runde verkaufte Menge |
| Umsatz | Umsatz der Runde |
| Deckungsb. | Deckungsbeitrag (Definition unten) mit Trendpfeil zur Vorrunde |
| Anteil Land | Marktanteil im Land der Fertigung, mit Länderkürzel in der Zelle |
| Kont. | Marktanteil im Kontinent dieses Landes |
| Welt | Marktanteil weltweit |

- **Trendpfeile statt eigener Vergleichsspalten:** ▲/▼ gegenüber der Vorrunde, genauer Vorrundenwert als Tooltip beim Darüberfahren.
- **Sortierung:** nach Umsatz absteigend.
- **Viele Produkte:** Es werden die Top 5 nach Umsatz gezeigt. Produkte mit negativem Deckungsbeitrag werden **immer** gezeigt, auch außerhalb der Top 5. Der Rest steht in einer Zeile „+ n weitere“, die sich aufklappen lässt.
- **Summenzeile** über alle Produkte (nicht nur die sichtbaren), darunter eine Zeile Fixkosten. Summe Deckungsbeitrag minus Fixkosten ergibt das Ergebnis in der Kopfzeile. Falls weitere Posten das Ergebnis beeinflussen (Zinsen, Steuern, Sondererträge), als eine zusätzliche Zeile „Sonstiges“ zeigen, damit die Rechnung aufgeht.
- Klick auf eine Produktzeile öffnet die Produkt- bzw. Marktdetailansicht.

### Deckungsbeitrag – Definition

Deckungsbeitrag = Umsatz − Materialkosten − variable Fertigungskosten − **direkt zugeordnete Personalkosten der Fertigung**.

- Direkt zugeordnet ist das Personal, das an den Anlagen des Produkts arbeitet (eine Anlage fertigt jeweils ein Produkt, ihre Personalkosten gehen also vollständig in dieses Produkt).
- Nicht im Deckungsbeitrag, sondern in den Fixkosten: Werksleitung, Verwaltung, Manager, Forschung/Labor, Strategieabteilung, Abschreibungen, Zinsen usw.
- Eine Vollkostenrechnung mit umgelegten Fixkosten wird im Rundenbericht **nicht** gezeigt.

### Marktanteile – Definition

- Marktanteil = eigene verkaufte Menge des Produkts in der Region ÷ gesamte verkaufte Menge dieses Produkts aller Anbieter in der Region, jeweils in der abgelaufenen Runde. 🟡 (mengenbasiert; wertbasiert wäre die Alternative)
- **Land** = Land der Fertigung. Wird ein Produkt in mehreren Ländern gefertigt, zeigt die Zeile das Land mit der größten Produktionsmenge. Die Zeile lässt sich dann je Fertigungsland aufklappen (Land und Kontinent je Unterzeile). 🟡
- **Kontinent** = Kontinent, zu dem dieses Land gehört.
- **Welt** = weltweiter Markt des Produkts.
- Anzeige als Prozentzahl, sinnvoll gerundet (ab 10 % ganze Zahl, darunter eine Nachkommastelle). Kein Absatz in der Region: „–“.
- Ziel: Der Spieler soll selbst einordnen können, wie viel Markt noch zu holen ist.

## 3. Auslastung

- **Je Werk** eine Zeile mit Balken und Prozentwert, **aufklappbar auf die einzelnen Anlagen** (Standard: zugeklappt; Werke mit Engpass dürfen aufgeklappt starten 🟡).
- Auslastung einer Anlage = tatsächlich produzierte Menge ÷ mögliche Menge bei voller Laufzeit im Rundenzeitraum (Nennkapazität inkl. aller aktiven Produktionseffekte).
- Auslastung eines Werks = Summe Ist-Menge ÷ Summe mögliche Menge aller **betriebsbereiten** Anlagen. Anlagen im Bau oder in Umstellung zählen nicht in den Durchschnitt, sondern erscheinen aufgeklappt mit Status und Restzeit.
- Farben: ab 85 % grün, 60–84 % gelb, unter 60 % rot. 🟡
- **Engpassgrund:** Sobald eine Anlage bzw. ein Werk nicht grün ist, steht dahinter der Hauptgrund für die Unterauslastung, z. B. „Material fehlte“, „Lager voll / kein Absatz“, „Personal fehlt“, „Umstellung“, „Störung/Wartung“, „manuell gedrosselt“. Bitte auf die Ursachen abbilden, die die Simulation tatsächlich kennt. Beim Werk steht der Grund, der die meiste Menge gekostet hat.
- Klick auf ein Werk bzw. eine Anlage öffnet die Werks- bzw. Anlagenansicht.

## 4. Bitte ansehen

- **Höchstens 5 Hinweise** 🟡, sortiert nach Dringlichkeit (Krise vor Warnung vor Info), jeder anklickbar zur passenden Detailansicht.
- Typische Kandidaten: negativer Deckungsbeitrag, Liquidität knapp, Lager leer oder voll, Werk unter 60 % ausgelastet, Manager kündigt, Vertrag gefährdet, offene Entscheidung wartet, Marktanteil eines Produkts stark gefallen (Schwelle 🟡).
- **Welt und Wettbewerb** erscheinen nur, wenn sie die eigenen Produkte oder eine Branche betreffen, in der der Spieler aktiv ist (z. B. Preissenkung eines Konkurrenten bei einem eigenen Produkt, Rohstoffschock bei einem eigenen Vorprodukt, neuer Wettbewerber in der eigenen Branche).
- Alles andere (allgemeine Weltereignisse, Wettbewerberaktionen in fremden Branchen, Forschungsstände, Börsenbewegungen) kommt nicht in den Bericht, sondern ins Meldungsfach. Am Ende des Berichts steht eine Zeile „n weitere Meldungen im Meldungsfach →“.

## 5. Bedienung

- **Enter** und **Esc** schließen den Bericht und führen ins Spiel.
- Jede Zeile ist klickbar und führt zur passenden Detailansicht; danach ist der Bericht über das Meldungsfach bzw. einen Button „Letzter Rundenbericht“ wieder aufrufbar.
- Einstellung „Rundenbericht nur zeigen, wenn es Hinweise gibt“ (Standard: aus). Ist sie an und gibt es keine Hinweise, geht die nächste Runde direkt los, der Bericht bleibt trotzdem abrufbar.
- Der Bericht soll ohne Scrollen auf einen Bildschirm passen (bei Standardauflösung, Top-5-Regel und zugeklappten Werken).

## 6. Bewusst nicht im Bericht

- **Entgangener Absatz / nicht bediente Nachfrage:** gehört nicht in den Rundenbericht. Diese Auswertung soll später an eine strategische Stelle geknüpft werden (Strategieabteilung bzw. Produktmanagement, passend zur Idee der Produktdiversifikation). Bitte als offenen Punkt vormerken, nicht jetzt umsetzen.
- Vollkostenrechnung je Produkt.
- Eigene Abschnitte für Welt, Wettbewerb, Forschung, Börse und Person.

## Meilensteine

| Nr. | Inhalt |
|---|---|
| RB1 | Datengrundlage: Deckungsbeitrag je Produkt nach obiger Definition, Auslastung je Anlage und Werk inkl. Engpassgrund, Marktanteile Land/Kontinent/Welt je Produkt und Runde, Vorrundenwerte für Trendpfeile |
| RB2 | Neue Berichtsansicht: Kopfzeile, Produkttabelle mit Top-5-Regel und Summen, Auslastung mit Aufklappen |
| RB3 | Hinweise und Filter: Priorisierung, Höchstzahl, Relevanzfilter für Welt/Wettbewerb, Verweis aufs Meldungsfach |
| RB4 | Bedienung: Enter/Esc, Klickziele, Button „Letzter Rundenbericht“, Einstellung „nur bei Hinweisen“; Lastenheft §13.2 und FORTSCHRITT.md anpassen |

## Abnahme

- Summe Deckungsbeitrag − Fixkosten (± Sonstiges) ergibt exakt das Ergebnis der Kopfzeile.
- Marktanteile aller Anbieter einer Region summieren sich auf 100 %.
- Eine Anlage ohne Materialengpass, ohne Absatzgrenze und mit vollem Personal zeigt rund 100 % Auslastung.
- Test mit vielen Produkten (> 10) und vielen Werken: Bericht bleibt auf einem Bildschirm und öffnet ohne spürbare Verzögerung.
- Alle mit 🟡 markierten Werte bitte in OFFENE_PUNKTE.md eintragen.
