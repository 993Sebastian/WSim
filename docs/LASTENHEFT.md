# Lastenheft Wirtschaftssimulation 1900–2100

Oct 3, 2026 · @Sebastian

## 1 Ziel und Rahmen

Ziel ist eine rundenbasierte Wirtschaftssimulation für Windows, in der der Spieler von 1900 bis 2100 eine kleine Werkstatt zu einem weltweiten Konzern ausbaut. Das Spiel ist ein Einzelspieler-Spiel und wird mit Claude Code umgesetzt.

- Vorbild für Spielmechanik und Abläufe ist GearCity, jedoch mit einer sehr viel breiteren Produktpalette über alle Branchen.
- 3D-Grafik spielt keine Rolle. Das Spiel lebt von Menüs, Tabellen, Diagrammen und einer 2D-Weltkarte.
- Leitprinzip in allen Bereichen ist eine möglichst realistische Welt: Länder, Rohstoffe, Löhne, Technologie, Geschichte und Firmen orientieren sich an der echten Welt.
- Spielsprache ist Deutsch.
- Dieses Lastenheft beschreibt das Was. Das Wie (Technik, Architektur, Formeln) schlägt Claude Code vor, begründet es und lässt es abnehmen.

## 2 Spielkonzept

Der Spieler ist Firmenchef und steuert sein Unternehmen gegen eine Vielzahl von KI-Firmen, die gemeinsam den gesamten Weltmarkt bilden. Es gibt kein festes Spielziel; die Partie endet am 31.12.2100.

- Die Spielerfigur ist in Version 1 personenunabhängig: kein Alter, kein Lebenslauf, keine Nachfolge.
- Start als kleine Werkstatt, die einfachste Teile herstellt oder mit ihnen handelt.
- Ausbau bis zum weltweiten Konzern mit hochkomplexen Produktionsketten und Vertriebswegen, z. B. Flugzeug- oder Raketenbau.
- Produktionsketten beginnen beim eigenen Rohstoffabbau, z. B. Erz, und reichen bis zum Endprodukt.
- Der Spieler entscheidet je Produkt, ob er selbst verkauft oder an Händler bzw. KI-Firmen verkauft.
- Die Komplexität wächst mit den Produkten: Einfache Produkte sind leicht zu beherrschen, Hochtechnologie ist tief und anspruchsvoll.
- Technologischer Fortschritt verdrängt Produkte: Kommt das Smartphone, sinkt die Nachfrage nach Handys über die Zeit.

## 3 Spielwelt

Die Welt besteht aus allen Ländern in ihren heutigen Grenzen; Städte spielen keine Rolle, die kleinste räumliche Einheit ist das Land.

### 3.1 Zeitraum

- Spielzeit vom Startjahr (Standard 1900, wählbar) bis 31.12.2100.
- Der technologische Stand friert mit dem Jahr 2026 ein. Die Datenstruktur muss trotzdem erlauben, später Zukunftstechnologien nachzupflegen.

### 3.2 Länder

- Die heutigen Grenzen gelten für den gesamten Zeitraum, auch für historische Ereignisse wie die Weltkriege.
- Jedes Land hat Merkmale entsprechend der echten Welt, die sich historisch verändern (Jahreswerte, dazwischen interpoliert).

| Merkmal | Bedeutung im Spiel |
| --- | --- |
| Bevölkerung und Kaufkraft | Grundlage der Endkunden-Nachfrage |
| Arbeitskräfte nach Qualifikation und Fachrichtung | Begrenzter Pool, um den alle Firmen konkurrieren |
| Lohnniveau je Qualifikation | Personalkosten, Abwerbung |
| Forschungsstärke je Fachgebiet | Effizienz von Forschungsstandorten |
| Prägung Handmontage oder Automatisierung | Produktivität und Eignung für bestimmte Fertigung |
| Infrastruktur (Häfen, Schiene, Straße, Flughäfen) | Transportwege, Kosten, Dauer |
| Steuern (Unternehmen, Dividenden, Zölle) | Gewinn nach Steuern, Standortwahl |
| Politische Stabilität | Risiko für Standorte und Handel |
| Währung | Wechselkurseffekte |

### 3.3 Weltkarte

- 2D-Weltkarte mit anklickbaren Ländern.
- Einblendbare Ebenen: Rohstoffe, Lohnniveau, eigene und fremde Standorte, Handelsrouten, Zölle und Handelsbeschränkungen, Konflikte.

### 3.4 Rohstoffe

- Lagerstätten sind real auf die Länder verteilt und endlich, sie können sich erschöpfen.
- Abbau erfordert Erschließung (Mine, Bohrfeld, Plantage) mit Investition und laufenden Kosten.

### 3.5 Handelsrouten

- Entfernungen und Wege nach realen Maßstäben: Seewege, Kanäle (z. B. Panamakanal erst ab 1914), Schiene, Straße, Luft.
- Verkehrsmittel und Infrastruktur stehen erst ab ihrer historischen Verfügbarkeit zur Verfügung.

### 3.6 Währung

- Intern rechnet das Spiel in einer Leitwährung (US-Dollar) auf Kaufkraftniveau 2026, ohne Inflation.
- Jedes Land hat seine historische Währung inklusive Umstellungen, z. B. Mark, Reichsmark, D-Mark, Euro.
- Wechselkurse zur Leitwährung ändern sich jährlich ungefähr wie in der Realität, jedoch ohne den Inflationsanteil.
- Wechselkurse wirken auf Exporte, Importe, Löhne und Gewinne.
- Anzeige standardmäßig in der Währung des Firmensitzes, umschaltbar.

## 4 Ereignisse

Bis 2026 laufen die realen historischen Ereignisse mit ihren wirtschaftlichen Folgen ab, danach erzeugt das Spiel zufällige Ereignisse.

### 4.1 Historische Ereignisse (1900 bis 2026)

- Beispiele: Erster und Zweiter Weltkrieg, Weltwirtschaftskrise 1929, Ölkrisen 1973 und 1979, Kalter Krieg mit Embargos, Finanzkrise 2008, Pandemie 2020.
- Kriegsparteien und betroffene Gebiete werden auf die heutigen Ländergrenzen abgebildet.
- Mögliche Wirkungen: Nachfrageverschiebung (z. B. Rüstung statt Konsum), Handelssperren und Zölle, Mangel an Arbeitskräften, Rohstoffpreise, Börsencrash.
- Standorte in Kriegsgebieten können zerstört oder beschädigt, enteignet oder zwangsweise auf Kriegsproduktion umgestellt werden. Das gilt für Spieler und KI-Firmen gleichermaßen und auch für zufällige Konflikte ab 2027.

### 4.2 Zufällige Ereignisse (ab 2027)

- Wirtschaftskrisen, Konflikte, Naturkatastrophen, Pandemien, politische Umbrüche.
- Häufigkeit und Stärke sind bei Spielstart einstellbar.

### 4.3 Daten

- Alle Ereignisse und ihre Wirkungen liegen in bearbeitbaren Datendateien, sodass Ereignisse ergänzt oder angepasst werden können.

## 5 Unternehmen, Organisation und Personal

Anfangs steuert der Spieler alles selbst; mit wachsender Größe delegiert er an ein mehrstufiges Manager-System bis hin zum Vorstand.

### 5.1 Firmensitz und Tochterfirmen

- Der Firmensitz liegt im gewählten Startland und bestimmt Steuern und Startbedingungen. Er ist später verlegbar (mit Kosten).
- Der Spieler kann Tochterfirmen gründen, z. B. für Logistik, mit eigener Bilanz und eigenem Management.
- Tochterfirmen können eigene Anteile haben und an die Börse gebracht werden.
- Lieferungen und Leistungen zwischen Konzernfirmen werden intern verrechnet und im Controlling getrennt ausgewiesen.

### 5.2 Standorte

- Je Land beliebig viele Standorte: Minen und Förderstätten, Werke, Lager, Handelsniederlassungen, Forschungszentren.
- Ein Werk hat: Kapazität, Automatisierungsgrad, Belegschaft nach Qualifikation, Qualitätsniveau, Zustand und Alter der Anlagen.

### 5.3 Arbeitskräfte

- Arbeitskräfte werden als Gruppen nach Qualifikation geführt, nicht als Einzelpersonen.
- Jedes Land hat einen begrenzten Pool je Qualifikation, um den alle Firmen konkurrieren.
- Abwerbung von anderen Firmen über Gehalt und Arbeitsbedingungen, z. B. Urlaub, Arbeitszeit, Sozialleistungen. Umgekehrt kann auch der Spieler Personal verlieren.

### 5.4 Schulung

- Pauschales Schulungssystem: Jeder Standort hat ein Schulungsniveau, das Kosten verursacht und Produktivität und Qualität erhöht.
- Ein HR-Manager kann beauftragt werden, ein Zielniveau auf nationaler, kontinentaler oder weltweiter Ebene zu halten.
- Bei Kündigungen wird automatisch nachbesetzt.

### 5.5 Manager

- Es gibt jederzeit ein Kontingent an verfügbaren und abwerbbaren Managern mit unterschiedlichen Fähigkeiten und Gehaltserwartungen.
- Manager sind in Version 1 alterslos; sie können kündigen oder abgeworben werden.
- Rollen auf Standortebene: Einkauf, Produktion, Vertrieb und Marketing, Logistik, HR, Forschung.
- Ein Werksmanager besetzt die untergeordneten Rollen seines Werks selbst.
- Darüber folgen Länder-, Kontinental- und Weltebene sowie ein Vorstand mit Fachressorts.
- Der Spieler gibt Ziele und Regeln vor, z. B. Mindestlagerbestand, Budget, Preisstrategie, Qualitätsziel. Er kann jederzeit selbst eingreifen (siehe 5.6).
- Die Fähigkeiten eines Managers bestimmen, wie gut er die Vorgaben umsetzt.

&#91;embedded content: Manager-Hierarchie · 5 Ebenen, 6 Rollen im Werk\]

Jede Ebene führt die darunterliegende nach den Vorgaben von oben; wo noch kein Manager sitzt, entscheidet der Spieler selbst.

### 5.6 Strategievorgaben

In einer eigenen Strategieansicht trifft der Spieler strategische Entscheidungen auf jeder Ebene vom Werk bis weltweit; alle Manager richten ihr Handeln danach aus.

- Ebenen: Werk bzw. Standort, Land, Kontinent, weltweit, Gesamtkonzern sowie jede Tochterfirma.
- Vorgaben vererben sich nach unten. Eine untere Ebene kann eine geerbte Vorgabe gezielt überschreiben, z. B. ein einzelnes Werk mit anderer Preisstrategie.
- Der Spieler kann jede Einzelentscheidung auch direkt treffen; eine direkte Entscheidung hat Vorrang vor der Strategie.
- Die Ansicht zeigt für jede Stelle, welche Vorgabe gilt und von welcher Ebene sie stammt.
- Wie gut eine Vorgabe umgesetzt wird, hängt von den Fähigkeiten des zuständigen Managers ab.

| Strategiefeld | Beispiele für Einstellungen |
| --- | --- |
| Verkaufswege | Nur eigener Vertrieb; KI-Händler dürfen kaufen; beides, mit Mindestpreis und Höchstmenge |
| Preis | Premium, Marktpreis, Kampfpreis, Mindestmarge |
| Qualität | Mindestqualität eigener Produkte und eingekaufter Vorprodukte |
| Eigenfertigung oder Zukauf | Halbzeuge selbst fertigen, zukaufen oder nach Kosten entscheiden |
| Lager | Mindest- und Höchstbestände, Sicherheitsreichweite |
| Partner | Konzernfirmen zuerst, bevorzugte Lieferanten und Kunden, gesperrte Firmen |
| Personal | Lohnniveau relativ zum Markt, Arbeitsbedingungen, Schulungsziel |
| Logistik | Eigene Flotte, staatlicher Transport, KI-Dienstleister |
| Investitionen | Budget, Wachstumsziele, Zielländer |
| Marketing | Budget, Zielmärkte, Markenpositionierung |
| Forschung | Schwerpunkte, Budget, Patente anmelden, Lizenzen vergeben |
| Finanzen und Risiko | Kreditrahmen, Liquiditätsreserve, Dividendenpolitik |
| Umwelt | Auflagen nur erfüllen oder übererfüllen |

Die Strategiefelder sind erweiterbar.

## 6 Produkte und Produktion

Produkte werden nicht frei entworfen, sondern durch Forschung freigeschaltet; die Breite reicht über alle Branchen vom Rohstoff bis zur Rakete.

### 6.1 Produktketten

- Kette: Rohstoff → Halbzeug (eine oder mehrere Stufen) → Komponente → Endprodukt.
- Halbzeuge und Komponenten muss der Spieler entwickeln und fertigen oder am Markt kaufen.
- Jedes Produkt hat ein Rezept: Eingangsstoffe und Mengen, Arbeitskräfte nach Qualifikation, Anlagen bzw. Automatisierung, Energie, Fertigungsdauer.
- Es gibt Industriegüter (für andere Firmen) und Konsumgüter (für Endkunden).
- Energie ist ein eigener Produktionsfaktor (z. B. Kohle, Strom, Öl, Gas). Standorte beziehen sie vom Markt bzw. Netz des Landes zu Landespreisen.
- Nach Erforschung der jeweiligen Technologie kann der Spieler Energie selbst erzeugen, z. B. mit eigenem Kraftwerk, eigener Kohleförderung oder Ölförderung, und Überschüsse verkaufen.

### 6.2 Branchen (Zielumfang)

Bergbau und Rohstoffe, Metallurgie, Chemie, Energie, Holz und Papier, Textil, Lebensmittel, Bau, Maschinenbau, Fahrzeuge, Schiffbau, Luftfahrt, Raumfahrt, Elektrotechnik und Elektronik, Konsumgüter, Pharma, Rüstung.

### 6.3 Qualität

- Jedes gefertigte Produkt hat einen Qualitätswert.
- Er hängt ab von: Qualität der Vorprodukte, Technologiestand, Schulungsniveau, Automatisierung, Zustand der Anlagen.
- Qualität beeinflusst erzielbaren Preis, Nachfrage und Markenimage.

### 6.4 Produktlebenszyklus

- Neue Technologien bringen Nachfolgeprodukte hervor, die Vorgänger verdrängen.
- Die Nachfrage nach dem alten Produkt sinkt schrittweise mit der Verbreitung des Nachfolgers.

### 6.5 Rüstung

- Spieler und KI-Firmen können Rüstungsgüter herstellen.
- Die Nachfrage hängt stark von Kriegen und Spannungen ab.

## 7 Forschung, Patente und Lizenzen

Der Technologiebaum bildet die reale Entwicklung ab; wer der Geschichte vorauseilen will, zahlt dafür so viel, dass extreme Vorgriffe praktisch in die Pleite führen.

### 7.1 Technologiebaum

- Realistischer Baum mit Abhängigkeiten über alle Branchen, jede Technologie mit ihrem historischen Erfindungsjahr.
- Forschung vor dem historischen Jahr ist möglich. Die Kosten steigen mit jedem Jahr Vorsprung stark an (z. B. exponentiell).
- Balancing-Vorgabe: Ein Smartphone 1960 ist technisch erreichbar, aber so teuer, dass ein Konzern daran normalerweise zerbricht. Der Kostenfaktor ist in den Daten einstellbar.
- Forschungsstandorte arbeiten je nach Land und Fachgebiet unterschiedlich effizient (siehe 3.2).
- KI-Firmen forschen ebenfalls. Ohne Zutun des Spielers entwickelt sich die Welt ungefähr historisch.
- Ist eine Technologie einmal erfunden, wird sie für Nachzügler günstiger.
- Ab 2026 kommen keine neuen Technologien hinzu.

### 7.2 Patente und Lizenzen

- Wer eine Technologie zuerst erforscht, kann sie patentieren (begrenzte Laufzeit, Kosten je abgedecktem Land).
- Patentierte Technologien dürfen andere nur mit Lizenz nutzen.
- Lizenzen können gekauft und verkauft werden, um Forschung abzukürzen oder Einnahmen zu erzielen.

## 8 Logistik und Handel

Waren bewegen sich über reale Entfernungen mit den zur jeweiligen Zeit verfügbaren Verkehrsmitteln; Zölle und Handelsbeschränkungen verändern sich dynamisch.

### 8.1 Transport

- Verkehrsmittel nach Epoche, z. B. Fuhrwerk, Dampfschiff, Eisenbahn, LKW, Containerschiff, Frachtflugzeug, Pipeline.
- Drei Wege, Waren zu bewegen:
  - eigene Flotte aufbauen und betreiben,
  - eine staatliche Transportorganisation beauftragen (überall verfügbar, aber teuer),
  - KI-Logistikfirmen zu Marktpreisen beauftragen.
- Der Spieler kann selbst Transportdienste für andere anbieten und damit Geld verdienen, z. B. über eine Logistik-Tochter.
- Jede Route hat Dauer, Kosten, Kapazität und Risiko.
- Lager an Standorten puffern Ware zwischen Produktion, Transport und Verkauf.

### 8.2 Zölle und Handelsbeschränkungen

- Zölle und Beschränkungen gelten je Länderpaar und Warengruppe.
- Bis 2026 folgen sie grob der Geschichte (z. B. Embargos, Kriegszeiten).
- Darüber hinaus verändern sie sich dynamisch; die Stärke dieser Dynamik ist im Spiel einstellbar.

## 9 Markt, Vertrieb und Verträge

Nachfrage entsteht aus Endkunden und aus dem Bedarf der Industrie; Preise bilden sich frei über Angebot und Nachfrage.

### 9.1 Nachfrage und Preise

Die Endkunden-Nachfrage wird je Land aus Einkommensschichten berechnet, damit sie sich wie in der realen Welt mit Wohlstand, Einkommensverteilung und Sättigung entwickelt.

- **Fünf Einkommensschichten je Land** (je ein Fünftel der Haushalte) mit Durchschnittseinkommen je Schicht. Grundlage sind historische Jahreswerte für Bevölkerung, Pro-Kopf-Einkommen und Einkommensverteilung, z. B. angelehnt an die Maddison Project Database und UN-Bevölkerungsdaten.
- **Nachfragemerkmale je Produkt** (in den Produktdaten): Bedarfsklasse (Grundbedarf, Gebrauchsgut, Luxus), Einkommensschwelle, Preis- und Einkommensempfindlichkeit, Nutzungsdauer, maximale Besitzquote, optional ein Saisonverlauf.
- **Verbrauchsgüter** wie Lebensmittel oder Kleidung: laufender Bedarf pro Kopf, der mit dem Einkommen bis zu einer Sättigung steigt.
- **Gebrauchsgüter** wie Möbel, Fahrrad oder Auto: Das Spiel führt je Land und Schicht eine Besitzquote. Nachfrage = Erstkäufer (bis zur Sättigung, typischer S-Kurven-Verlauf) + Ersatzkäufe nach Ablauf der Nutzungsdauer.
- **Kaufkraftgrenze:** Eine Schicht kauft erst, wenn ihr Einkommen im Verhältnis zum Preis die Schwelle erreicht. Preissenkungen öffnen neue Schichten und schaffen Massenmärkte, wie beim Ford Modell T.
- **Anbieterwahl:** Die Nachfrage verteilt sich nach Attraktivität auf die Anbieter: Preis, Qualität, Markenbekanntheit, Verfügbarkeit im Land. Untere Schichten achten stärker auf den Preis, obere stärker auf Qualität und Marke.
- **Gesättigte Märkte zum Start** (Vorgabe des Auftraggebers vom 04.10.2026): Zum Spielstart decken etablierte Firmen die Nachfrage aller Märkte, und ihre Marken sind dort bekannt. Der Spieler kommt als neuer Anbieter hinzu und gewinnt Marktanteile auf klassischen Wegen: günstigerer Preis, bessere Qualität, Werbung und Markenbekanntheit, Präsenz in weiteren Ländern (Niederlassungen), Lieferungen an Firmen, Händler und den Staat.
- **Verdrängung:** Ein Nachfolgeprodukt gewinnt Kaufentscheidungen schrittweise; Besitzer des alten Produkts wechseln beim nächsten Ersatzkauf.
- **Länderprägung:** optionaler Vorliebefaktor je Land und Produktgruppe, z. B. Fahrrad in den Niederlanden.
- **Staatliche Nachfrage:** Staaten kaufen abhängig von Wirtschaftsleistung und Lage, z. B. Rüstung, Schienen, Baustoffe, Fahrzeuge; in Kriegen stark steigend.
- **Industrie-Nachfrage:** aus dem Bedarf aller Firmen nach Rohstoffen, Halbzeugen, Komponenten und Energie.
- **Konjunktur und Ereignisse** verändern die Einkommen der Schichten und damit die Nachfrage.
- **Preise** bilden sich je Land über Angebot und Nachfrage; Transportkosten und Zölle verbinden die Märkte.

### 9.2 Verkaufswege

- Eigener Vertrieb über Niederlassungen und Händlernetz, oder Verkauf an KI-Händler und andere Firmen.
- Der Spieler legt fest, ob und was KI-Händler und andere Firmen von ihm kaufen dürfen: je Produkt, je Land oder pauschal, optional mit Mindestpreis und Höchstmenge.
- Diese Entscheidung trifft er entweder selbst oder gibt sie als Strategie vor, nach der seine Manager handeln (siehe 5.6).

### 9.3 Lieferverträge

- Neben dem freien Markt (Spotmarkt) gibt es langfristige Verträge mit KI-Firmen, als Käufer und als Verkäufer.
- Vertragsinhalte: Menge, Preis, Laufzeit, Mindestqualität, Vertragsstrafen.

### 9.4 Marketing

- Werbung und Markenbekanntheit je Land und Produktgruppe; Image wird auch von Qualität und Umweltverhalten beeinflusst.
- Werbemittel nach Epoche, z. B. Zeitung, Radio, Fernsehen, Internet.
- Marketing und Vertrieb lassen sich an Manager delegieren.
- Vorgezogen nach Stufe 1 (Auftraggeber 04.10.2026): Werbebudget je Land und Warengruppe,
  Markenbekanntheit von 0 bis 1, die durch Werbung und eigene Verkäufe wächst und ohne
  beides langsam verblasst; es wirkt automatisch das beste Werbemittel der Epoche. KI-Firmen
  werben nach denselben Regeln. Wahl einzelner Werbemittel, Image durch Qualität und
  Umwelt sowie Delegation bleiben in Stufe 2 bzw. 4.

## 10 KI-Konkurrenten

Die KI-Firmen bilden den gesamten Markt; ihre Anzahl ist einstellbar und kann sehr hoch sein.

- Startbesetzung aus realen historischen Firmen mit ihren tatsächlichen Tätigkeitsfeldern zum Startjahr, z. B. Krupp, Siemens, Standard Oil.
- Weitere reale Firmen entstehen zu ihrem historischen Gründungsjahr, z. B. Boeing 1916. Bei Bedarf füllen generierte Firmen auf die eingestellte Anzahl auf.
- Rollen: Rohstoffförderer, Hersteller, Transporteure, Händler, Investoren. Eine Firma kann eine, mehrere oder alle Rollen haben.
- Ausrichtung: spezialisiert auf einzelne Märkte oder diversifiziert.
- Verhalten: forschen, investieren, expandieren, Preise anpassen, auf den Spieler reagieren, übernehmen und übernommen werden, pleitegehen.
- Manche Firmen sind privat und gehen erst im Lauf der Jahre an die Börse, wo bekannt zum historischen Zeitpunkt.
- KI-Firmen spielen nach denselben Regeln wie der Spieler. Der Schwierigkeitsgrad verändert ihre Kompetenz und Aggressivität, nicht die Regeln.

## 11 Finanzen und Börse

Der Spieler finanziert sich über Eigenkapital, Kredite und den Kapitalmarkt; eine interne Börse handelt Anteile aller börsennotierten Firmen.

### 11.1 Finanzierung

- Startkapital frei eingebbar.
- Bankkredite mit Zins, Laufzeit und Bonitätsprüfung; Kreditlinien.
- Anleihen ausgeben.
- Eigener Börsengang, Kapitalerhöhungen, Dividenden, Aktienrückkäufe; auch für Tochterfirmen.
- Steuern je Land auf Gewinne und Dividenden.

### 11.2 Börse und Investoren

- Kurse bilden sich aus Gewinn, Wachstum, Dividende, Marktstimmung und Ereignissen; Crashs sind möglich (z. B. 1929).
- KI-Investoren kaufen und verkaufen Anteile, auch am Unternehmen des Spielers.
- Der Spieler kann Anteile anderer Firmen kaufen, Beteiligungen aufbauen und Mehrheiten übernehmen.
- Übernahmen, auch feindliche, sowie Fusionen.
- Auch das Unternehmen des Spielers kann übernommen werden, sobald es an der Börse ist und Investoren oder KI-Firmen die Mehrheit kaufen. Verliert der Spieler so die Kontrolle, endet das Spiel.

### 11.3 Insolvenz

- Firmen, die zahlungsunfähig werden, gehen pleite; ihre Standorte und Marken können versteigert oder übernommen werden.
- Für den Spieler endet das Spiel erst, wenn er zahlungsunfähig ist und weder neue Kredite erhält noch Unternehmensanteile verkaufen kann.
- Solange einer dieser Wege offen ist, kann er sich selbst retten.

### 11.4 Rechnungswesen

- Bilanz, Gewinn- und Verlustrechnung und Kapitalfluss für Konzern und jede Tochterfirma.

## 12 Regulierung und Umwelt

Auflagen und Verbote greifen historisch passend und je Land unterschiedlich.

- Beispiele: Arbeitsschutzgesetze, Verbot von Asbest, FCKW und verbleitem Benzin, Emissionsgrenzwerte, CO₂-Bepreisung (z. B. EU-Emissionshandel ab 2005).
- Wirkungen: zusätzliche Kosten, Nachrüstpflichten für Anlagen, Produktions- oder Verkaufsverbote.
- Standorte verursachen Emissionen; Umweltverhalten beeinflusst das Markenimage.
- Nach 2026 können neue Regulierungen als zufällige Ereignisse entstehen (siehe 4.2).
- Regulierungen liegen wie Ereignisse in bearbeitbaren Datendateien.

## 13 Rundenablauf und Rundenbericht

Der Spieler plant, wählt die Länge der nächsten Runde und lässt die Welt laufen; längere Runden sparen Zeit, geben der Konkurrenz aber mehr Gelegenheit zu reagieren.

### 13.1 Ablauf einer Runde

1. Der Spieler trifft beliebig viele Entscheidungen.
2. Beim Beenden wählt er die Rundenlänge: Tag, Woche, Monat oder Quartal.
3. Die Simulation rechnet den Zeitraum intern in kleinen Schritten (Vorschlag: Tage). KI-Firmen entscheiden in ihrem eigenen Rhythmus, also bei langen Runden mehrfach, der Spieler erst wieder am Rundenende.
4. Manager des Spielers handeln innerhalb der Runde nach ihren Vorgaben weiter.
5. Das Spiel speichert automatisch und zeigt den Rundenbericht.

### 13.2 Rundenbericht

- Finanzergebnis der Runde und Veränderung zum Vorzeitraum.
- Wichtige Ereignisse in der Welt.
- Aktionen der Wettbewerber (neue Produkte, Preissenkungen, Übernahmen, Pleiten).
- Warnungen, z. B. Lager leer, Liquidität knapp, Vertrag gefährdet, Manager kündigt.
- Forschungsfortschritte und Börsenbewegungen.
- Jeder Eintrag führt per Klick zur passenden Detailansicht.

### 13.3 Meldungen und Töne

- Meldungen sind nach Art unterschieden, z. B. Erfolg, Warnung, Krise, Weltereignis, Börse.
- Jede Art hat einen passenden eigenen Ton. Lautstärke einstellbar, Töne abschaltbar.
- Keine Musik in Version 1.

## 14 Benutzeroberfläche und Controlling

Die Oberfläche ist nüchtern und modern; ein ausgeprägtes Controlling zeigt genau, womit, wo und wodurch Geld verdient oder verloren wird.

### 14.1 Hauptansichten

| Ansicht | Inhalt |
| --- | --- |
| Übersicht | Kennzahlen, Warnungen, letzte Meldungen |
| Weltkarte | Länder, Standorte, Routen, Rohstoffe, Ebenen nach 3.3 |
| Produktionsketten | Ketten vom Rohstoff bis zum Endprodukt, Engpässe |
| Forschung | Technologiebaum, laufende Projekte, Patente, Lizenzen |
| Markt und Preise | Nachfrage, Preise, Marktanteile je Land und Produkt |
| Börse | Kurse, Beteiligungen, Investoren, Übernahmen |
| Finanzen | Bilanz, GuV, Kapitalfluss, Kredite |
| Controlling | Siehe 14.2 |
| Strategie | Strategievorgaben je Ebene nach 5.6 |
| Organisation | Manager, Personal, Schulung, Tochterfirmen |
| Logistik | Flotten, Routen, Transportaufträge, Lager |
| Verträge | Lieferverträge, Lizenzen |
| Berichte | Rundenberichte und Meldungsarchiv |

### 14.2 Controlling

- Gewinn und Deckungsbeitrag je Produkt, Werk, Standort, Handelsniederlassung, Land, Kontinent, Tochterfirma und Transportroute.
- Kosten nach Kostenarten (Material, Personal, Energie, Transport, Zölle, Steuern, Marketing, Forschung, Zinsen).
- Durchklicken von oben nach unten: Konzern → Kontinent → Land → Standort → Produkt.
- Zeitreihen und Vergleich mit Vorperioden; Vergleich mit Wettbewerbern, soweit öffentlich bekannt.

### 14.3 Bedienung

- Tabellen sortier- und filterbar, Diagramme für Zeitreihen.
- Tooltips, die Werte und ihre Ursachen erklären.
- Tastaturkürzel für häufige Aktionen.

### 14.4 Einführung (Tutorial)

Vorgabe des Auftraggebers vom 04.10.2026:

- Beim Start eines neuen Spiels im Dialog an- und abwählbar (Standard: an).
- Führt Schritt für Schritt durch die ersten Entscheidungen: Lage als Neuling im
  gesättigten Markt, Übersicht, Produktion, Einkauf und Verkauf, Markt mit Anteilen,
  Werbung, Runde beenden, Rundenbericht, Finanzen.
- Jederzeit überspringbar; über die Tastenhilfe erneut startbar.

## 15 Spieleinstellungen bei neuem Spiel

Die Einstellmöglichkeiten orientieren sich am Umfang von Capitalism Lab; ein Schwierigkeitsgrad setzt sinnvolle Voreinstellungen, die einzeln überschrieben werden können.

- Startjahr (Standard 1900)
- Startland bzw. Firmensitz
- Startkapital (frei eingebbar)
- Startform der Firma: Werkstatt mit Fertigung oder Handel
- Schwierigkeitsgrad als Voreinstellung
- Anzahl der KI-Firmen (bis sehr hoch)
- Kompetenz und Aggressivität der KI-Firmen
- Preis- und Qualitätsempfindlichkeit der Kunden
- Stärke der Konjunkturschwankungen
- Dynamik von Zöllen und Handelsbeschränkungen
- Häufigkeit und Stärke zufälliger Ereignisse ab 2027
- Kostenfaktor für Forschung vor dem historischen Jahr
- Zufallswert (Seed) für reproduzierbare Partien

## 16 Technische Anforderungen

Das Spiel ist ein installierbares Windows-Programm, das offline läuft; Spielinhalte liegen in bearbeitbaren Datendateien, getrennt vom Programmcode.

### 16.1 Plattform und Technik

- Windows-Desktop-Spiel mit Installer, Start wie ein gewöhnliches installiertes Spiel.
- Die Wahl der Technologie trifft Claude Code und begründet sie. Anforderungen: gute Darstellung vieler Tabellen und Diagramme, 2D-Weltkarte, leistungsfähige Simulation.
- Simulationskern und Oberfläche sind getrennt. Der Kern ist ohne Oberfläche lauffähig und testbar.
- Die Simulation ist bei gleichem Seed und gleichen Entscheidungen reproduzierbar.

### 16.2 Daten und Erweiterbarkeit

- Alle Spielinhalte liegen in lesbaren, bearbeitbaren Dateien (z. B. JSON oder YAML): Länder je Jahr, Rohstofflagerstätten, Produkte und Rezepte, Technologien, Transportmittel, Firmen, Ereignisse, Regulierungen, Währungen und Wechselkurse.
- Neue Produkte oder Technologien werden durch Ergänzen der Daten hinzugefügt, ohne Programmierung.
- Daten werden beim Laden geprüft; Fehler werden verständlich gemeldet.
- Realistische Startdaten erfordern Recherche. Wo Claude Code Näherungswerte einsetzt, werden sie als Annäherung gekennzeichnet.

### 16.3 Speichern

- Automatisches Speichern nach jeder Runde.
- Zusätzlich beliebig viele manuelle Spielstände.
- Spielstände tragen eine Versionsnummer, damit sie nach Updates weiter ladbar bleiben.

### 16.4 Leistung und Analyse

- Zunächst kein festes Leistungsziel; die Architektur muss aber auf sehr viele KI-Firmen ausgelegt sein.
- Während der Rundenberechnung zeigt das Spiel einen Fortschritt.
- Simulations-Protokolle für spätere Balance- und Leistungsanalysen.
- Automatische Tests für den Simulationskern.
- Alle Texte zentral gehalten, damit eine spätere Übersetzung möglich bleibt.

## 17 Ausbaustufen

Das Spiel wird in sechs Stufen umgesetzt; jede Stufe ist für sich spielbar, und das Fundament aus Stufe 1 trägt alle späteren.

1. **Fundament:** Simulationskern, Datenformat, alle Länder mit Grundwerten, Rohstoffe, 12 Produktionsketten nach 17.1, Rundenablauf mit wählbarer Länge, Markt- und Preisbildung, KI-Firmen mit Grundverhalten, Grundfinanzen, Weltkarte, Rundenbericht, Speichern. Spielbar etwa 1900 bis 1930.
2. **Wachstum:** Manager-System, Schulung, Tochterfirmen, Logistik mit eigener Flotte, staatlichem und KI-Transport, Zölle, Lieferverträge, Marketing, ausgebautes Controlling.
3. **Kapitalmarkt:** Börse, Börsengänge, Investoren, Beteiligungen, Übernahmen, Insolvenzen, Anleihen; Spiel als reiner Investor oder als Bank (17.3).
4. **Geschichte:** historische Ereignisse, Regulierung und Umwelt, Währungen und Wechselkurse.
5. **Breite:** vollständige Produkt- und Technologiedaten bis 2026 (Ziel rund 500 Endprodukte über alle Epochen, Regeln nach 17.2), inklusive Elektronik, Luft- und Raumfahrt, Rüstung; reale Firmen mit Gründungsjahren; Patente und Lizenzen.
6. **Feinschliff:** zufällige Ereignisse ab 2027, Töne, Balancing, Leistungsoptimierung.

### 17.1 Produktionsketten in Stufe 1

Stufe 1 umfasst 8 Rohstoffe und 12 Ketten, die sich über gemeinsame Halbzeuge (vor allem Stahl) verbinden und jede Kernmechanik mindestens einmal abdecken.

Rohstoffe: Eisenerz, Kohle, Kupfererz, Holz, Baumwolle, Getreide, Kautschuk, Rohöl.

| Nr. | Kette | Weg vom Rohstoff zum Produkt | Zweck im Spiel |
| --- | --- | --- | --- |
| 1 | Eisen und Stahl | Eisenerz + Kohle → Stahl (Hüttenwerk) → Blech, Draht, Stabstahl | Rückgrat fast aller anderen Ketten |
| 2 | Kleinteile | Draht, Stabstahl → Nägel, Schrauben, Muttern | Startprodukt der Werkstatt |
| 3 | Werkzeug | Stahl + Holz → Handwerkzeuge | Einfaches Endprodukt aus zwei Ketten |
| 4 | Holz | Holz → Schnittholz → Möbel | Konsumgut |
| 5 | Textil | Baumwolle → Garn → Stoff → Kleidung | Mehrstufige Konsumkette |
| 6 | Lebensmittel | Getreide → Mehl; Weißblech + Lebensmittel → Konserven | Verbindet Landwirtschaft mit Stahl |
| 7 | Energie | Kohle → Strom im eigenen Kraftwerk | Energie als Produktionsfaktor |
| 8 | Kupfer und Elektro | Kupfererz → Kupfer → Kupferdraht → Elektromotor | Industriegut für andere Ketten |
| 9 | Licht | Rohöl → Petroleum → Petroleumlampe; Kupfer + Glas → Glühlampe | Verdrängung: Glühlampe ersetzt Petroleumlampe |
| 10 | Gummi | Kautschuk → Gummi → Reifen | Komponente für Fahrrad und Auto |
| 11 | Nähmaschine und Fahrrad | Stahl, Kleinteile, Reifen → Nähmaschine, Fahrrad | Massenprodukte, Industrie- und Konsumgut |
| 12 | Automobil | Motor, Fahrgestell, Karosserie, Reifen; Rohöl → Benzin | Erste komplexe Kette mit Forschung; Verdrängung der Pferdekutsche |

Rüstungsgüter folgen erst mit den historischen Ereignissen in Stufe 4.

### 17.2 Regeln für Produktbäume (alle Stufen)

Vorgabe des Auftraggebers vom 04.10.2026 (Ebenen am selben Tag von vier auf sechs
erweitert), gilt für alle heutigen und künftigen Produkte:

- Ein Rezept hat **höchstens vier Vorprodukte**. Lieber ein Eingangsmaterial weglassen als
  einen zu breiten Baum bauen; z. B. wird ein Benzinmotor nur aus Stahl gefertigt.
- Der Baum zu einem Zielprodukt hat **höchstens sechs Ebenen einschließlich Zielebene**,
  gezählt vom Rohstoff, z. B. Rohstoff → Halbzeug → Bauteil → Endprodukt (vier Ebenen).
  Sechs Ebenen reizen nur sehr komplexe Produkte aus (Linienflugzeug, Rakete); die meisten
  Produkte liegen bei drei bis vier Ebenen, weniger ist immer erlaubt. Güter ohne eigenes
  Rezept (vom staatlichen Markt bezogen, z. B. Glas, Zinn) zählen als Rohstoffebene. Strom
  ist Produktionsfaktor und zählt nicht als Vorprodukt.
- Produkte mit fünf oder sechs Ebenen tragen in den Daten `sehr_komplex: true`; ohne diese
  Angabe meldet die Datenprüfung eine Warnung.
- Die Datenprüfung (`wsim validate`) erzwingt beide Regeln für jedes Rezept und zeigt die
  Verteilung der Endprodukte nach Ebenen.
- Ziel über alle Epochen: **rund 500 Endprodukte**, also Breite statt Tiefe. Viele
  Endprodukte teilen sich wenige Halbzeuge und Bauteile.

### 17.3 Vorbereitung auf Investor, Bank und Tochterfirmen

Vorgabe des Auftraggebers vom 04.10.2026: Spätere Spielweisen werden schon im Fundament
berücksichtigt:

- Der Spieler kann sich nur als **Investor** beteiligen (ohne eigene Produktion), am
  **Aktienmarkt** handeln oder als **Bank** auftreten (Kredite vergeben, Einlagen).
- Der Spieler kann solche Tochterfirmen gründen (Beteiligungs-, Investment-, Bankfirma)
  neben Produktions-, Handels- und Logistiktöchtern.
- Umsetzung mit Stufe 2 (Tochterfirmen) und Stufe 3 (Kapitalmarkt); das Datenmodell aus
  Stufe 1 darf dem nicht im Weg stehen (siehe Architektur §2.5).

## 18 Offene Punkte

Alle bisher offenen Punkte sind geklärt und in die jeweiligen Abschnitte eingearbeitet.

Neue Fragen, die bei der Umsetzung auftauchen, werden hier gesammelt. Die vollständige
Liste mit Vorschlägen steht in `docs/OFFENE_PUNKTE.md`.

### 18.4 Entscheidungen vom 05.10.2026 (Kaufangebote, Produkte bis 2026)

- **Kaufangebote zwischen Firmen** (vorgezogen; ergänzt die Übernahmen über Anteile aus
  Stufe 3 und die Lizenzen aus Stufe 5): Firmen können einander **Standorte** (Werke,
  Minen, Kraftwerke, Forschungslabore), **ganze Bereiche** (alle Standorte einer
  Warengruppe mit der Markenbekanntheit) und **Technologien als Lizenz** abkaufen.
  - Jede Firma kann jeder anderen ein Angebot machen: KI an Spieler, Spieler an KI und
    KI-Firmen untereinander, nach denselben Regeln (§10).
  - Die Gegenseite nimmt an, lehnt ab oder macht ein Gegenangebot.
  - KI-Firmen bieten, wenn ein Objekt wirtschaftlich ist, eine qualifizierte Belegschaft
    oder Forscher mitbringt, eigene Forschung oder einen Neubau erspart oder Wettbewerb
    vermeidet.
- **Produkte bis 2026:** Die Breite aus Stufe 5 wird Epoche für Epoche vorgezogen
  (1915–1939, 1940–1964, 1965–1989, 1990–2009, 2010–2026), jeweils mit Rohstoffen,
  Technologien, Ketten und Verdrängung nach den Regeln aus 17.2. Rüstungsgüter bleiben
  bei Stufe 4 (17.1).

### 18.3 Entscheidungen vom 05.10.2026 (Verbesserungen)

Die acht Vorschläge aus `docs/OFFENE_PUNKTE.md`, Abschnitt E, werden umgesetzt
(Meilensteine M22–M29):

- Anlagen lassen sich stilllegen und verkaufen, vom Spieler und von der KI; danach eine
  Balance-Runde (Überkapazität, Benzin, Gummi, Reifen).
- Etappenziele nach der Einführung.
- Wettbewerb und Preise im Verlauf (Rundenbericht, Preis- und Anteilsverlauf im Markt).
- Ansicht Produktionsketten (§14.1).
- Weiterlaufen bis zum Jahresende oder zur nächsten Warnung.
- Erklärungen, aus welchen Teilen sich ein Wert zusammensetzt (§14.3).
- Geschichte erzählen: Währungsreformen und Weltereignisse bis 2026 als Meldungen.
- Rang der eigenen Firma in der Übersicht.

### 18.2 Entscheidungen vom 05.10.2026

- **Browser-Version:** Neben dem Windows-Programm gibt es eine Browser-Version zum Prüfen
  neuer Stände (auch vom iPhone). Der Simulationskern läuft dort als WebAssembly; sie
  wird nach jedem Push über GitHub Pages veröffentlicht.
- **Manager:** bleiben in Stufe 2, wie geplant.
- **Arbeitskräfte:** Eingestellt wird weiter nach dem Produktionsplan. Die Werksansicht
  zeigt Bedarf, Belegschaft, Lohn und freie Kräfte je Gruppe; je Standort setzt der
  Spieler einen Lohnaufschlag. Sind Arbeitskräfte knapp, bekommen die besten Zahler sie
  zuerst (vorgezogen aus 5.3).
- **Währungen:** werden vorgezogen (sonst Stufe 4). Anzeige umschaltbar: in Kaufkraft
  2026 ohne Inflation (Standard, wie 3.6) oder mit zeitgenössischen Preisen
  einschließlich Inflation. Für Währungen, die es 1900 noch nicht gab, gelten Annahmen.
- **Technologiebaum, Werksansicht, Tutorial:** Die Oberfläche wird auf Verständlichkeit
  geprüft und überarbeitet; der Technologiebaum zeigt Technologien, Verfahren, Produkte
  und Anlagen; das Tutorial begleitet mit hervorgehobenen Schaltflächen bis zum ersten
  verkauften Produkt.

### 18.1 Entscheidungen vom 03.10.2026

- **Staatlicher Markt:** Güter ohne eigene Produktionskette (z. B. Glas, Zinn, Schwefel, Inhalt von Konserven, Pferdekutsche) können vom staatlichen Markt bezogen werden. Er bietet die Ware in jedem Land zu einem Preis aus den Datendateien an.
- **Länderwerte:** Die Jahreswerte der Länder bilden den realen Verlauf ab, einschließlich der Einbrüche durch Kriege und Krisen. Historische Ereignisse (Stufe 4) erzeugen diese Einbrüche nicht ein zweites Mal.
- **Sprache:** Programmcode verwendet englische Bezeichner; Datendateien und alle Texte sind deutsch.
- **Arbeitskräfte:** vier Qualifikationen (Ungelernte, Angelernte, Fachkräfte, Akademiker), bei Fachkräften und Akademikern neun Fachrichtungen, die zugleich die Fachgebiete der Forschung sind (Vorschlag umgesetzt, Freigabe ausstehend).
