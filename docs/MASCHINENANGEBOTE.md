# Maschinenangebote (Vorgabe vom 12.10.2026)

Neues Paket, Meilenstein **MAA** am Ende der Kette (nach FW1; WG1–WG5 bleiben bis zur
Aufforderung zurückgestellt). Annahmen und offene Fragen: `docs/OFFENE_PUNKTE.md`,
Abschnitt W. Alte Spielstände müssen nicht kompatibel bleiben.

## 1. Nachtrag zu FW1 (Anlagengrößen)

- Der Anlagenkauf läuft in zwei Schritten: erst das Produkt, dann die Größe. Pro Produkt
  stehen alle Größen aus M36 zur Wahl.
- Beim Umstellen auf ein anderes Produkt der Branche bleibt die Größe erhalten.
  - Dauer: halbe Katalog-Bauzeit der Größe.
  - Umstellungskosten richten sich ebenfalls nach dieser Größe, nicht nach der
    individuellen Bauzeit des ursprünglichen Angebots.
- Ein Größenwechsel ist keine Umstellung. Dafür wird neu gekauft oder die alte Anlage
  verkauft.

## 2. Ausschreibung statt Direktkauf

- Der Spieler wählt Produkt und Größe. Die Auswahl zeigt wie bisher die Katalogwerte:
  Preis, Fläche, Output, Personalbedarf, Input, Ausschussrate und Bauzeit.
- Statt zu kaufen, startet er eine Ausschreibung (Befehl z. B.
  `RequestMachineOffers { site, product, size }`). Sie ist kostenlos und läuft 28 Tage.
- In jeder der vier Wochen gehen an zufälligen Tagen 1–5 neue Angebote von
  Maschinenbauern ein (Parameter `angebote_je_woche: [1, 5]`).
- Die Angebote erscheinen an der Stelle, an der die Anlage ausgewählt wurde. Dazu kommen
  eine Meldung „Neue Maschinenangebote“ und ein Status „Ausschreibung läuft – Tag X von
  28“.
- Jedes Angebot gilt 14 Tage und kann angenommen oder abgelehnt werden.
- Mit der Annahme startet der Bau mit den Werten des Angebots. Die Ausschreibung endet,
  alle übrigen Angebote verfallen.
- Ohne Annahme verfällt die Ausschreibung. Eine neue kann jederzeit gestartet werden.
- Je Standort, Produkt und Größe läuft höchstens eine Ausschreibung gleichzeitig.

## 3. Streuung der Angebote

Jedes Angebot zieht drei verdeckte, voneinander unabhängige Werte im Bereich −1 bis +1:
Leistung L, Preislage P und Bauzeit Z.

- **Verteilung** (für alle drei gleich):
  - 85 % Normalverteilung um 0 mit Standardabweichung 0,35.
  - 15 % Ausreißer mit Betrag 0,7–1,0 und zufälligem Vorzeichen.
  - Begrenzt auf −1 bis +1.
- **Kennzahlen relativ zum Katalogwert.** Jede Kennzahl außer der Bauzeit bekommt ein
  eigenes Rauschen von ±3 %. Jede Kennzahl ist auf ±20 % begrenzt.

  | Kennzahl | Faktor |
  | --- | --- |
  | Output | × (1 + 0,20 · L) |
  | Personalbedarf | × (1 − 0,15 · L) |
  | Input je Stück | × (1 − 0,10 · L) |
  | Ausschussrate | × (1 − 0,20 · L) |
  | Fläche | × (1 ± 0,10), unabhängig gezogen |
  | Preis | × (1 + 0,12 · L + 0,08 · P) |
  | Bauzeit | × (1 + 0,20 · Z), auf ganze Tage gerundet |

- **Folgen:**
  - Premiummaschinen (hohes L) kosten mehr und leisten deutlich mehr.
  - Es gibt Schnäppchen (hohes L, niedriges P) und schlechte Angebote (niedriges L,
    hohes P).
  - Schnelle Lieferung (Z) kann für ein schwächeres Angebot sprechen.
- Die angebotene Bauzeit ist verbindlich.
- Alle Parameter stehen in `data/parameter/maschinenangebote.yaml`.
- Eigener Zufallsstrom „beschaffung“.

## 4. Maschinenbauer

- Fiktive Anbieter je Land oder Region und Epoche, 5–8 Namen je Land
  (`data/maschinenbauer.yaml`).
- Jeder Anbieter hat eine Tendenz von −0,3 bis +0,3, die den Mittelwert von L verschiebt.
  Z verschiebt sie nicht.
- Der Anbietername steht am Angebot, die Tendenz ist nicht sichtbar.

## 5. Anzeige

- Die Angebotsliste zeigt alle Kennzahlen absolut und als farbig markierte Abweichung vom
  Katalogwert in Prozent.
- **Bauzeit:** in Tagen, mit Abweichung zum Katalogwert in Prozent, und die
  voraussichtliche Fertigstellung bei Annahme heute.
- **Materialkosten je Stück** = Σ (Inputmenge je Stück des Angebots / (1 − Ausschussrate
  des Angebots)) · Marktpreis des Vorprodukts.
  - Marktpreis am Angebotsdatum im Land des Standorts. Gibt es dort keinen, gilt die
    Einstandsschätzung wie bei B2 (inkl. Transport bzw. staatlicher Markt).
  - Der Wert wird beim Eingang des Angebots fest berechnet und nicht mehr aktualisiert.
    Das Angebot zeigt „Materialkosten je Stück zu Marktpreisen vom TT.MM.JJJJ“.
  - Vergleichswert: derselbe Wert für den Katalogwert, zum gleichen Datum. Die
    Abweichung steht absolut und in Prozent daneben, farbig markiert.
  - Die Aufschlüsselung je Vorprodukt (Menge, Preis, Betrag) lässt sich aufklappen.
- **Gesamtkosten:** Die geschätzten Stückkosten bei Vollauslastung (Material, Personal,
  Abschreibung, Gemeinkosten) stehen als eigene Zeile daneben.

## 6. Gebaute Anlage

- Die Anlage speichert ihre Faktoren gegenüber dem Katalog und zeigt sie in der
  Werksansicht samt Anbieter.
- Beim Umstellen auf ein anderes Produkt bleiben die Faktoren erhalten.
- Lernkurve, Zustand, Instandhaltung und Ausfälle wirken zusätzlich.
- Überholung und Verkaufswert beziehen sich auf den gezahlten Preis.

## 7. KI-Firmen und Manager

- KI-Firmen und Manager nutzen dieselbe Ausschreibung.
- **Bewertung:**
  - Stückkosten über die Lebensdauer, mit denselben Materialkosten wie die Anzeige.
  - Bauzeit: Bei Engpass oder Nachfrage ohne Angebot entgeht bis zur Fertigstellung
    Deckungsbeitrag. Den rechnen sie gegen den Vorteil des Angebots.
  - Der Schätzfehler nach Kompetenz (B2 bzw. Manager-Formeln) wirkt nur auf ihre eigene
    Einschätzung, nicht auf die Anzeige.
- Sie nehmen das beste Angebot an, sobald es eine Schwelle unterschreitet, spätestens
  am Ende der Ausschreibung.
- Liegt der Preis über dem Manager-Budget, entsteht ein Anliegen „Maschinenangebot
  annehmen?“ mit Empfehlung.
- Die Auslöser „Anlage bauen/Kapazität ausbauen“ starten künftig eine Ausschreibung statt
  eines Direktkaufs.
