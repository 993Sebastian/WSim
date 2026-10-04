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

## M5 – Produktion

Parameter: `data/parameter/produktionsmodell.yaml`.

### Standorte und Anlagen

- Ein Standort kostet Grundstück und Gebäude (`standortkosten_usd` je Standorttyp);
  Abschreibung linear über `gebaeude_lebensdauer_jahre`.
- Eine Anlage kostet ihre Investition und produziert erst nach der Bauzeit
  (bis dahin „Anlagen im Bau“). Abschreibung linear über ihre Lebensdauer, Wartung
  täglich `Investition · wartung_je_jahr / 365`.
- Zustand: sinkt linear von 1 auf `zustand_minimum` über die Lebensdauer.
- Lagerstätten: Erschließung kostet und dauert laut Daten; danach Abbau bis zur
  Höchstförderung je Kalenderjahr und bis der Vorrat erschöpft ist. Die
  Erschließung wird über `erschliessung_lebensdauer_jahre` abgeschrieben.

### Tagesproduktion je Anlage

    geplante Durchläufe = Kapazität je Tag · Auslastung
    Durchläufe = min(geplant,
                     Lagerbestand(Eingang) / Menge je Durchlauf   für jeden Eingang,
                     freie Arbeitsstunden(Gruppe) / Stunden je Durchlauf   für jede Gruppe,
                     geplant · Netzanteil des Landes   (nur bei Strombedarf),
                     verbleibende Jahresförderung und Vorrat   (nur Abbau))

    Stunden je Durchlauf = Stunden laut Rezept · Arbeitsfaktor · Förderkostenfaktor (Abbau)
    Arbeitsfaktor = 1 − Automatisierung · Arbeitsersparnis · (0,5 + 0,5 · Automatisierungsprägung)

Die Anlagen eines Standorts teilen sich dessen Belegschaft in ihrer Reihenfolge.
Das Ergebnis eines Durchlaufs liegt nach `dauer_tage` Tagen im Lager (bei 1 Tag am
selben Tag).

### Qualität

    Q = Grundqualität + 0,3 · (Ø Qualität der Eingänge − 50) + 10 · Automatisierung
        − 20 · (1 − Zustand),   begrenzt auf 0–100

(Gewichte aus den Parametern.) Abbau-Rezepte haben keine Eingänge (Term = 0).

### Personal

- Jeder Beschäftigte leistet `Jahresarbeitsstunden / 365` Stunden je Kalendertag und
  bekommt dafür den Stundenlohn seiner Gruppe im Land.
- Am Monatsersten und nach jeder Änderung wird jeder Standort auf den Bedarf seiner
  geplanten Produktion eingestellt: Fehlende Kräfte werden aus dem freien Pool des
  Landes (Pool minus alle Beschäftigten im Land) eingestellt, überzählige entlassen.
  Ältere Standorte werden zuerst bedient. Vertretung durch höhere Qualifikationen
  und Abwerbung kommen später (M10).

### Bewertung und Buchung (Gesamtkostenverfahren)

- Verbrauchte Eingänge: Materialaufwand zum Durchschnittswert des Lagers.
- Löhne aller Beschäftigten: Personalaufwand (auch für ungenutzte Stunden).
- Strom: Energieaufwand zum Landespreis.
- Erzeugte Ware: Bestandserhöhung zu Herstellkosten = Eingänge + genutzte
  Arbeitsstunden · Lohn + Strom. Nebenprodukte erhalten Kosten anteilig nach Menge.
- Ungenutzte Arbeitszeit, Wartung und Abschreibung mindern das Ergebnis direkt.

## M6 – Finanzen

Parameter: `data/parameter/finanzmodell.yaml`. Das Spiel kennt keine Inflation; alle
Zinsen sind **reale** Zinsen (Nominalzins minus Inflation, historisch angenähert).

### Kredite

    Kreditrahmen = Beleihung · (Sachanlagen + Anlagen im Bau + Vorräte) − laufende Kredite
    Zins = Realzins(Jahr) + Mindestaufschlag + Aufschlag je Verschuldung · Verschuldungsgrad
    Verschuldungsgrad = (Kredite + neuer Kredit) / (Bilanzsumme + neuer Kredit)
    Monatsrate (Annuität) = K · i/12 / (1 − (1 + i/12)^−n),  n = Laufzeit in Monaten

Am Monatsende: Zinsen = Restschuld · i/12 (Zinsaufwand), Tilgung = Rate − Zinsen.
Vorzeitige Tilgung ist jederzeit möglich.

### Kontoüberziehung (Kreditlinie)

    Kreditlinie = Dispo-Anteil · Bilanzsumme
    Überziehungszinsen am Monatsende = −Kasse · (Realzins + Dispo-Aufschlag) / 12

### Steuern

Am 31. Dezember: Gewinnsteuer am Firmensitz auf das Jahresergebnis vor Steuern.
Verluste werden unbegrenzt vorgetragen und mit späteren Gewinnen verrechnet.
Dividendensteuer folgt mit Dividenden (Stufe 3).

### Zahlungsunfähigkeit (Lastenheft §11.3)

Am Monatsende ist eine Firma zahlungsunfähig, wenn die Überziehung die Kreditlinie
übersteigt **und** ein neuer Kredit die Lücke nicht schließen könnte. Für den Spieler
endet dann die Partie; solange ein Kredit möglich ist, warnt das Spiel nur.

### Berichte

GuV nach Kostenarten (Gesamtkostenverfahren), Bilanz aus den Kontensalden,
Kapitalfluss nach dem Gegenkonto jeder Kassenbewegung: Sachanlagen/Anlagen im Bau →
Investitionen, Kredite/Eigenkapital → Finanzierung, alles andere → operativ.

## M7 – Markt und Preise

Parameter: `data/parameter/marktmodell.yaml`. Je Produkt und Land gibt es einen Markt.

### Teilnehmer

- **Anbieter:** Standorte mit Verkaufsangebot (Festpreis oder automatischer Preis;
  ein Teil des Lagers kann zurückbehalten werden) und – für Güter mit `staatsmarkt` –
  der Staatsmarkt zum Datenpreis · Preisniveau, in beliebiger Menge, Qualität 50.
- **Nachfrager**, in dieser Reihenfolge bedient:
  1. Einkaufsaufträge von Standorten (Industrie): Ziel-Lagerbestand, Höchstpreis,
     Mindestqualität; höchste Zahlungsbereitschaft zuerst, jeweils beim billigsten
     passenden Anbieter, nie bei der eigenen Firma.
  2. Staat: täglich `je_mio_usd_bip · BIP (Marktpreise) / 10⁶ / 365`, billigste
     Anbieter bis `staat_hoechstpreis · Richtpreis · Preisniveau`.
  3. Endkunden, reichste Schicht zuerst.

### Endkunden-Nachfrage (monatlich)

Je Einkommensfünftel q mit Einkommen y_q (Marktpreise) und Marktpreis p,
Richtpreis r (· Preisniveau):

    a_q = (y_q / (Kaufschwelle · r))^Einkommensempfindlichkeit · (r / p)^Preisempfindlichkeit
    Kaufneigung S_q = a_q / (1 + a_q)

Liegt das Einkommen beim `kaufschwelle`-Fachen des Preises, kauft die Hälfte der
Schicht. Preissenkungen erhöhen S_q gerade in den unteren Schichten (Massenmärkte).

- **Verbrauchsgüter:** Nachfrage je Tag = Bedarf je Kopf · S_q · Bevölkerung/5 / 365 · Saison.
- **Gebrauchsgüter:** Besitzquote o_q je Einwohner. Zielquote T_q = max. Besitzquote · S_q
  · Verdrängungsfaktor. Nachfrage je Tag = ((T_q − o_q)⁺ · Aneignung + o_q / Nutzungsdauer)
  · Bevölkerung/5 / 365. Monatlich: o_q += Käufe / (Bevölkerung/5) − o_q / (12 · Nutzungsdauer).
- **Verdrängung:** Ersetzt Produkt B Produkt A, sinkt T_A um den Faktor
  (1 − o_B / max. Besitzquote_B) – Besitzer des Nachfolgers kaufen den Vorgänger nicht mehr.

### Anbieterwahl der Endkunden

    Nutzen_i = −Preisgewicht_q · ln(p_i / r) + Qualitätsgewicht_q · (Qualität_i − 50) / 25
    Anteil_i = exp(Nutzen_i) / Σ exp(Nutzen_j)

Ist ein Anbieter ausverkauft, wird der Rest auf die übrigen verteilt.

### Preise

- Automatischer Preis: Start beim Marktpreis · (1 + Aufschlag). Täglich +`hoch`, wenn
  der Anbieter alles verkauft hat und Nachfrage offen blieb; −`runter`, wenn er weniger
  als 1/`lagertage` seines Angebots verkauft hat; nie unter der Preisuntergrenze.
- Marktpreis (Index) = (1 − g) · bisheriger Index + g · Durchschnittspreis des Tages,
  g = `index_glaettung`; vor dem ersten Verkauf Richtpreis · Preisniveau.

### Buchungen

Verkauf: Umsatzerlöse (Kasse) und Bestandsminderung zu Herstellkosten. Einkauf:
Vorräte zum gezahlten Preis (keine Ergebniswirkung bis zum Verbrauch).

## M8 – Handel zwischen Ländern

Parameter: `data/parameter/transportmodell.yaml`, `data/verkehrsmittel.yaml`,
`marktmodell.haendler`.

### Frachtdienst

Jedes Land ist ein Knoten an seiner Hauptstadt; innerhalb eines Landes kosten
Warenbewegungen nichts und dauern keinen Tag. d(A, B) ist die Großkreisentfernung der
Hauptstädte. Je Transportklasse k mit Kostenfaktor f_k und Jahr gilt je Weg das
günstigste verfügbare Verkehrsmittel, das k befördert (Kosten c je tkm, Strecke v je Tag).

- **Landweg** zwischen Nachbarn, Länge L = d · `umweg.land`. Infrastruktur I = Mittel
  beider Länder (Schiene bzw. Straße; Gelände = 1), nur nutzbar ab `mindestinfrastruktur`:

      Kosten = L · c · f_k / I        Dauer = L / (v · I)

  Gewählt wird das günstigste von Fuhrwerk, Straße und Schiene.
- **Seeweg** zwischen allen Ländern mit Hafen (Infrastruktur H ≥ Mindestwert), Länge
  S = d · `umweg.see`:

      Kosten = S · c · f_k + U(A) + U(B)      U(X) = Umschlagkosten · f_k / H(X)
      Dauer  = S / v + Umschlagtage / H(A) + Umschlagtage / H(B)

- **Luftweg** wie der Seeweg, mit Flughafen-Infrastruktur und `umweg.luft`.
- Die **Route** ist der kostengünstigste Weg über beliebige Teilstrecken (Dijkstra);
  Binnenländer erreichen die See über ihre Nachbarn. Die Infrastruktur gilt mit den
  Werten vom 1. Januar für das ganze Jahr.
- Kosten je Einheit = Route (USD/t) · Gewicht der Einheit (kg) / 1000; die Dauer wird auf
  ganze Tage aufgerundet (mindestens 1).

### Warentransfer der Firmen

Transfer zwischen eigenen Standorten in verschiedenen Ländern: Die Transportkosten
werden sofort als Kostenart Transport gebucht (Kostenstelle: Abgangsstandort, Produkt),
die Ware ist bis zur Ankunft mit ihrem Lagerwert unterwegs (Konto Vorräte) und kommt
am Ankunftstag vor der Produktion an.

### KI-Händler

Ein wettbewerblicher Händlermarkt verbindet die Länder (Gewinne verlassen das Spiel).

- **Offene Nachfrage** eines Markts je Tag:

      o = (Endkunden + Staat − davon von Firmen im Land bedient)
          + Industriekäufe aus Importen + offene Industrienachfrage / vorrat_tage

  geglättet: ō ← ō + (o − ō) / `glaettung_tage`.
- **Bedarf** eines Ziellands B: vorrat_tage · ō − Importlager − unterwegs.
- **Einkauf:** Händler kaufen nur aus Verkaufsangeboten von Firmen, die ihre
  Verkaufsfreigabe erlaubt (siehe unten). Für jedes Angebot im Land A ist der
  Einstandspreis E = Angebotspreis + Transport(A → B). Gekauft wird nur, wenn
  `Marktpreis(B) ≥ E · (1 + marge)`. Ziele mit der höchsten relativen Spanne werden zuerst
  bedient, jeweils aus den günstigsten Angeboten. Die Käufe finden bei der Räumung des
  Markts A statt, nach der Industrie und vor Staat und Endkunden.
- **Verkauf** im Zielland: Das Importlager ist ein Angebot wie jedes andere. Sein Preis
  steigt um `hoch`, wenn es ausverkauft ist und Nachfrage offen bleibt, sonst sinkt er um
  `runter`, nie unter durchschnittlicher Einstandspreis · (1 + marge).
- Folge: Im Gleichgewicht gilt Preis(B) ≤ (Preis(A) + Transport) · (1 + marge).

### Verkaufswege (Vorgaben)

Je Abnehmergruppe (KI-Händler, andere Firmen) und Geltungsbereich (pauschal, Land,
Produkt, Produkt im Land; der genaueste gilt): erlaubt ja/nein, Mindestpreis,
Höchstmenge je Angebot und Monat. Ohne Vorgabe dürfen alle kaufen. Endkunden und
Staaten kaufen immer.

## M9 – Alle Ketten, Forschung, Eigenstrom

Parameter: `data/parameter/forschungsmodell.yaml`, `produktionsmodell.strom`,
`produktionsmodell.einspeiseverguetung`, `produktionsmodell.startformen`.

### Forschung

- Ein Forschungszentrum arbeitet an einer Technologie. Seine Labore haben
  `kapazitaet_je_tag` Forscherplätze × Auslastung; besetzt werden sie mit Arbeitskräften
  der Gruppe `forscher.<Fachgebiet>` (z. B. `akademiker.metall`) aus dem Landespool.
- Forschungspunkte je Tag = Forscher · Forschungseffizienz des Landes im Fachgebiet.
  Die Punkte gehören der Firma (mehrere Zentren können an derselben Technologie arbeiten).
- Kosten: Löhne der Forscher und Sachkosten (Forscher · `sachkosten` · Preisniveau),
  beides als Kostenart Forschung.
- Aufwand zum Zeitpunkt t (in Jahren, mit Bruchteil), Erfindung E = historisches Jahr
  oder früherer Erfolg im Spiel:

      vor E:   Punkte = Aufwand · vorgriff_faktor^((Jahr − t) · Spieleinstellung)
      ab E:    Punkte = Aufwand · max(minimum, (1 − rabatt_je_jahr)^(t − E))

  „Jahr“ ist das historische Erfindungsjahr; die Spieleinstellung „Kostenfaktor für
  Forschung vor dem historischen Jahr“ (0,25–4) verstärkt oder dämpft den Vorgriff.
  Erreichen die gesammelten Punkte den aktuellen Aufwand, ist die Technologie erforscht.
- Gemeingut: `gemeingut_nach_jahren` nach dem historischen Erfindungsjahr darf jede Firma
  die Technologie nutzen; bis zum Startjahr erfundene Technologien kennt jeder.

### Eigenstrom (Kette 7)

- Kraftwerke produzieren vor allen anderen Standorten. Ein Rezept mit Strombedarf e je
  Durchlauf läuft höchstens

      Durchläufe ≤ geplant · Netzanteil + Eigenstrom / e

  Eigenstrom ist der Strom auf allen Standorten der Firma im selben Land. Er wird zuerst
  verbraucht (Kostenart Energie zu Herstellkosten), der Rest kommt aus dem Netz zum
  Landespreis.
- Strom lässt sich nicht lagern: Was am Tagesende übrig ist, geht ins Netz zu
  Strompreis · `einspeiseverguetung` (Umsatzerlös).

### Nachfrage (Ergänzungen zu M7)

- Ergänzungsgut (`ergaenzung`): Nachfrage je Tag und Schicht q =
  Besitzquote_q(Gebrauchsgut) · Bevölkerung/5 · je_besitz_und_jahr · S_q / 365 · Saison.
- Netzabhängig: Verbrauch, Ziel-Besitzquote bzw. Ergänzung werden mit der
  Netzversorgung des Landes multipliziert.

### Ruhende Märkte (Rechenweise, ab M9)

Märkte ohne Firmenangebote, Einkaufsaufträge, Importe und Exporte „ruhen“: Nur
Endkunden und Staat fragen nach, und höchstens der Staatsmarkt verkauft. Ihre Tage
werden am Monatsende (vor dem Wechsel der Länderwerte) oder bei Aktivierung gesammelt
gebucht – mit denselben Formeln wie täglich, für d gleiche Tage:

    offene Nachfrage ō ← D + (ō − D) · (1 − 1/glaettung_tage)^d
    Marktpreis      p ← p_Staat + (p − p_Staat) · (1 − index_glaettung)^d

Die Ergebnisse hängen nicht von der Rundenlänge ab.

### Startformen

Die neue Firma erhält den Standort ihrer Startform (Werkstatt oder Niederlassung) mit
den Anlagen, Einkaufsaufträgen und Verkaufsangeboten aus den Daten; Gebäude und Anlagen
werden vom Startkapital bezahlt (reicht es nicht, lässt sich das Spiel nicht starten).
