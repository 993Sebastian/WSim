# Formeln und Regeln der Simulation

Jede Regel wird hier beschrieben, bevor sie umgesetzt wird, und mit ihrem Meilenstein
abgenommen. Parameter stehen in den Datendateien, nicht im Code.

## M3 – Zeit, Runden, Zufall, Spielstände

### Kalender

- Gregorianischer Kalender, kleinster Schritt ist ein Tag.
- Die Partie beginnt am 1. Januar des Startjahres (1900 bis 2026) und endet mit dem
  31.12.2100. Danach sind keine Entscheidungen mehr möglich.

### Runden

Der Spieler wählt beim Beenden jeder Runde ihre Länge:

| Länge | Ende der Runde |
| --- | --- |
| Tag | nächster Tag |
| Woche | 7 Tage später |
| Monat | Beginn des nächsten Kalendermonats |
| Quartal | Beginn des nächsten Kalenderquartals (1.1., 1.4., 1.7., 1.10.) |

Monat und Quartal enden immer an Kalendergrenzen. Nach einer Wochenrunde am 8. Januar
läuft eine Monatsrunde also nur bis Ende Januar. So bleiben Monats- und
Quartalsabschlüsse (GuV, Berichte) an echten Kalenderperioden ausgerichtet.

Ablauf eines Tages: Systeme in fester Reihenfolge (Architektur §2.2), danach wird das
Datum weitergezählt und die Länderwerte für den neuen Tag berechnet. Zum Jahreswechsel
erscheint eine Meldung.

### Jahreswerte

Jahreswerte in den Daten gelten **zur Jahresmitte** (wie Bevölkerungs- und
BIP-Statistiken). Wert am Tag *d*:

    t = Jahr(d) + (Tag_im_Jahr(d) − 1) / Tage_im_Jahr − 0,5
    Wert(d) = lineare Interpolation der Jahreswerte an der Stelle t

Vor dem ersten und nach dem letzten Jahreswert gilt der jeweils nächste Wert.

### Zufall

- Ein Seed je Partie. Jedes Teilsystem und jede Firma hat einen eigenen Zufallsstrom
  (xoshiro256++, initialisiert über SplitMix64 aus Seed und Stromnummer).
- Neue Firmen verändern daher nicht die Zufallszahlen anderer Firmen.

### Reproduzierbarkeit

- Jede erfolgreiche Entscheidung und jede Runde wird im **Journal** festgehalten.
  Gleiche Spieldaten + gleiche Einstellungen + gleiches Journal ergeben exakt
  denselben Spielstand (geprüft über einen Zustands-Hash).
- Speichern, Laden und Weiterspielen ergibt denselben Zustand wie Durchspielen.

### Spielstände

- Datei: Kennung `WSIMSAVE`, lesbarer Kopf (Formatversion, Spielversion,
  Datenversion, Datum, Firmenname), danach der komprimierte Spielzustand.
- Verweise auf Spielinhalte werden mit ihren IDs (`DEU`) gespeichert. Ändern sich die
  Daten, werden sie beim Laden neu zugeordnet; neue Inhalte werden ergänzt. Fehlt ein
  Inhalt, den der Spielstand verwendet, meldet das Spiel, welcher.
- Ältere Formatversionen werden beim Laden schrittweise umgewandelt.

## M4 – Ländermodell

Aus den Jahreswerten eines Landes (Bevölkerung, BIP je Kopf zu Kaufkraftparität, Gini)
und den Parametern in `data/parameter/laendermodell.yaml` werden alle weiteren
Länderwerte abgeleitet. Sie werden **monatlich** (Werte des Monatsersten) neu berechnet
und nicht gespeichert, weil sie sich vollständig aus Daten und Datum ergeben.

Bezeichnungen: *y* = BIP je Kopf (KKP, USD 2026), *t* = Zeitpunkt der Jahreswerte
(Jahresmitte, siehe M3).

### Preisniveau und Einkommen

    P = (y / y_Bezugsland)^Elastizität, begrenzt auf [Minimum, Maximum]

Bezugsland ist die USA (P = 1). Marktpreise in ärmeren Ländern liegen unter der
Kaufkraftparität (Balassa-Samuelson); Löhne und Einkommen werden daher mit P in
Weltmarktpreise umgerechnet.

Einkommensschichten: Die Einkommen folgen einer Lognormalverteilung mit dem Gini des
Landes, σ = √2 · Φ⁻¹((Gini + 1) / 2). Anteil des Fünftels *q* am Einkommen:

    Anteil_q = Φ(z_q − σ) − Φ(z_{q−1} − σ),   z_q = Φ⁻¹(q/5)
    Einkommen je Kopf im Fünftel q = y · P · 5 · Anteil_q

### Arbeitskräfte und Löhne

    Erwerbspersonen = Bevölkerung · Erwerbsquote
    Anteil der Qualifikation = Tabelle „qualifikationsanteile“, logarithmisch in y interpoliert
    Anteil der Fachrichtung = Grundanteil · Ländergewicht, normiert auf 1
    Pool (Gruppe) = Erwerbspersonen · Anteil Qualifikation · Anteil Fachrichtung

    Durchschnittslohn je Erwerbsperson = Lohnquote · y / Erwerbsquote
    Jahreslohn (Qualifikation) = Durchschnittslohn · Faktor_q / Σ (Anteil_q · Faktor_q)
    Stundenlohn = Jahreslohn / Jahresarbeitsstunden(Jahr) · P

Die Faktoren („lohnabstand“) sinken mit dem Wohlstand: Fachkräfte und Akademiker waren
um 1900 relativ teurer als heute.

### Energie, Steuern, Infrastruktur, Stabilität

    Netzanteil = Stromnetz(Jahr) · (y / Bezug)^0,7, begrenzt auf [0, 1]
    Strompreis = Strompreis(Jahr) · P
    Entwicklung d = Lage von y zwischen „von_usd“ und „bis_usd“ (logarithmisch), 0–1
    Schiene = √d · Verfügbarkeit(Jahr), Straße = d · Verfügbarkeit(Jahr),
    Luft = d · Verfügbarkeit(Jahr), Hafen = max(√d, 0,2) (Binnenländer: 0)

Steuern und Stabilität: eigene Jahreswerte des Landes, sonst die Standardwerte.

### Forschung und Automatisierung

    Forschungseffizienz (Fachgebiet) = (y / Bezug)^Elastizität, begrenzt · Ländergewicht
    Prägung Automatisierung = Basis + je_Verdopplung · log₂(y / Bezug) + Länderzuschlag

Werte prüfen: `wsim land DEU --jahr 1900`.
