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
  · Verdrängungsfaktor. Nachfrage je Tag = ((T_q − o_q)⁺ · Aneignung + o_q / Nutzungsdauer
  · min(1, T_q / o_q)) · Bevölkerung/5 / 365 – liegt der Besitz über dem Ziel, ersetzen
  nur so viele Besitzer ihr Gerät, wie das Ziel noch trägt; die anderen wechseln (§6.4).
  Monatlich: o_q += Käufe / (Bevölkerung/5) − o_q / (12 · Nutzungsdauer).
- **Besitz zum Spielstart:** Für Gebrauchsgüter, die es zum Start schon gibt (ein Rezept
  mit bekannter Technik oder der Staatsmarkt), beginnt o_q bei T_q; sonst bei 0.
- **Verdrängung:** Ersetzt Produkt B Produkt A, sinkt T_A um den Faktor
  (1 − o_B / max. Besitzquote_B) – Besitzer des Nachfolgers kaufen den Vorgänger nicht mehr.

### Anbieterwahl der Endkunden

    Nutzen_i = −Preisgewicht_q · ln(p_i / r) + Qualitätsgewicht_q · (Qualität_i − 50) / 25
    Anteil_i = exp(Nutzen_i) / Σ exp(Nutzen_j)

Ist ein Anbieter ausverkauft, wird der Rest auf die übrigen verteilt. Seit M16 kommen
Marke und Präsenz hinzu (Abschnitt M16). Das Preisgewicht ist dort auf 7 (ärmstes
Fünftel) bis 3 (reichstes) gestiegen: Massenware wird über den Preis verkauft, ein
doppelter Preis behält bei Gewicht 5 nur 1/32 der Kunden.

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

## M10 – KI-Firmen

Parameter: `data/parameter/kimodell.yaml`; reale Firmen und Namensbausteine in
`data/ki/`. Werte `{bei_0, bei_1}` gelten linear in Kompetenz *k* bzw. Aggressivität *a*
der Firma (0–1). Jede Firma weicht zufällig (eigener Zufallsstrom) um höchstens
± `streuung` von den Werten der Schwierigkeit ab.

### Marktmaßstab

Mit *N* KI-Firmen arbeiten die Märkte im Maßstab *s* = clamp(*N* / `firmen_bei_realer_groesse`,
`massstab.minimum`, `massstab.maximum`), bei 100 Firmen also 0,1. Skaliert werden
Marktbevölkerung (Verbrauchernachfrage), Staatsnachfrage, Arbeitskräfte (nie unter
`arbeitskraefte_min` je Gruppe), Förderung und Vorrat der Lagerstätten sowie deren
Erschließungskosten. Preise, Löhne, Einkommen je Kopf und Anlagengrößen bleiben real.
Ohne KI-Firmen gilt *s* = 1.

### Konzessionen

Eine Lagerstätte hat *n* = clamp(⌊Förderung_max · *s* / (`anlagen_je_konzession` ·
Jahresleistung einer Förderanlage)⌋, 1, `konzessionen_max`) gleich große Konzessionen.
Jede Konzession gehört höchstens einem Standort; ihre Jahresförderung ist höchstens
Förderung_max · *s* · Anteil, der Vorrat (× *s*) gilt für alle gemeinsam. Erschließen
kostet Erschließungskosten · *s* · Anteil. Spielstände vor M10 übernehmen die eine
erschlossene Lagerstätte als Konzession mit Anteil 1.

### Startbesetzung

1. **Rezept je Produkt:** das im Startjahr nutzbare Rezept mit den geringsten
   Stückkosten aus Vorprodukten zum Richtpreis, Arbeit zum `referenzlohn_usd`, Strom zum
   mittleren Strompreis der Länder und Abschreibung samt Wartung.
2. **Reale Firmen** (höchstens die Hälfte von *N*, gegründet bis zum Startjahr, Name
   nicht vom Spieler belegt) erhalten ihre Anlagen mit Anzahl max(1, round(Anzahl ·
   *s*)); ihre Leistung deckt Nachfrage, ihr Bedarf erhöht die der Vorprodukte.
3. **Bedarf:** Produkte werden so geordnet, dass jedes nach allen Produkten kommt, die
   es verbrauchen. Je Produkt und Land gilt Bedarf = Verbraucher- + Staatsnachfrage pro
   Tag + Vorproduktbedarf der schon geplanten Anlagen − Leistung realer Anlagen.
4. **Anlagen:** *F* = Bedarf / (Tagesleistung · `start.auslastung`); unter
   `anlage_mindestanteil` keine. Verteilung auf Länder mit Gewicht Bedarf ·
   Entwicklung^*w* (*w* je Produktart aus `gewicht_entwicklung`), bei Rohstoffen auf
   Lagerstätten mit freien Konzessionen nach Förderung / Kostenfaktor (höchstens die
   Förderung). Ganze Zahlen nach dem größten Rest, Summe round(*F*), mindestens 1.
5. **Vorprodukte begrenzen:** Von den Rohstoffen aufwärts wird jede Stufe auf den
   Anteil verkleinert, den ihre Vorprodukte weltweit decken (Staatsmarkt und Strom
   unbegrenzt) – sonst entstünden Werke ohne Material.
6. **Firmen:** Anlagen werden je Branche und Land gebündelt. Sind es mehr Bündel als
   Firmen, geht das kleinste in das kleinste derselben Branche auf; sind es weniger,
   wird das größte geteilt (Anlagen oder Anzahl halbiert).
7. **Ausstattung je Firma:** ein Standort je Land und Standorttyp, je Konzession ein
   Förderstandort, eigenes Kraftwerk im Land für den Strom, den das Netz nicht liefert
   (Bedarf · (1 − Netzanteil)). Lager: `lager_ausgang_tage` der Erzeugung (zu
   Herstellkosten) und `lager_eingang_tage` der Vorprodukte (zum Marktpreis). Kasse:
   `kasse_monate` der laufenden Kosten. Angebot im Marktpreis-Modus mit Untergrenze aus
   den variablen Kosten, Einkaufsaufträge für alle Vorprodukte. Eigenkapital = Summe
   aller Vermögenswerte.

### Verhalten

Alle Entscheidungen sind Befehle mit derselben Prüfung wie beim Spieler; abgelehnte
Befehle unterbleiben. Sie werden am Tagesanfang getroffen und nicht ins Journal
geschrieben, weil sie sich bei der Wiederholung aus dem Zustand ergeben.

- **Betrieb** alle `betrieb_alle_tage` (*k*) Tage, erster Termin zufällig gestreut:
  - Lagerreichweite = (Lager − Rückhalt) / Absatz je Tag im Vormonat; ohne Absatz im
    Vormonat / (Vollleistung · Auslastung). (Bis M16 stand hier immer die eigene
    Erzeugung im Nenner: Jede Drosselung verlängerte die Reichweite und trieb die
    Auslastung bis zum Minimum.) Über
    `lager_hoch_tage` sinkt die Auslastung um `auslastung_schritt` (nicht unter
    `auslastung_min`), unter `lager_niedrig_tage` steigt sie, wenn die Anlage nicht
    durch fehlende Vorprodukte oder Arbeitskräfte gebremst war.
  - Neue Anlagen bekommen das günstigste bekannte Rezept; bekannte bessere Rezepte auf
    derselben Anlage ersetzen alte (z. B. nach Forschung). Strom zählt dabei mit
    Strompreis / Netzanteil des Landes (eigenes Kraftwerk im Land: Netzanteil 1), damit
    elektrische Verfahren nur gewählt werden, wo genug Strom da ist (M15).
  - Preisuntergrenze = Vollkosten je Stück · `preisuntergrenze` (*a*): variable Kosten
    (Vorprodukte zum Marktpreis, Arbeit, Strom) bei der laufenden Auslastung plus
    Fixkosten (Abschreibung, Instandhaltung, bei Förderung die Erschließung über
    `erschliessung_lebensdauer_jahre`) bei der Normalauslastung `start.auslastung`
    (M15; vorher nur variable Kosten). Darüber sucht der Marktpreis-Modus den Preis. Ein
    neuer `SetSale` mit gleichem Aufschlag setzt die Preissuche nicht zurück.
  - Erzeugnisse ohne Angebot, die der Standort nicht selbst braucht (z. B. Benzin als
    Nebenprodukt der Raffinerie), werden zum Marktpreis ohne Untergrenze angeboten.
  - Einkauf: Ziel = `lager_eingang_tage` · Tagesbedarf, Höchstpreis = Marktpreis ·
    (1 + `einkauf_aufschlag`); fehlt Ware (unter `lager_niedrig_tage`), steigt das Gebot
    je Durchgang um `auslastung_schritt` bis zum Dreifachen des Marktpreises.
  - Eigene Ware: Eine Firma kann auf dem Markt nicht bei sich selbst kaufen. Fehlende
    Vorprodukte holt sie per `TransferGoods` von eigenen Standorten, die sie anbieten
    (zuerst im selben Land, sonst per Fracht).
- **Kasse** am Monatsanfang: unter `kasse_min_monate` laufender Kosten ein Kredit über
  `kredit_jahre` bis zur Mitte zwischen Minimum und Maximum (höchstens der
  Kreditrahmen), über `kasse_max_monate` Tilgung.
- **Ausbau** am letzten Tag jedes Quartals: das Produkt mit der höchsten Marge
  (Angebotspreis, ohne Angebot Marktpreis / Stückkosten − 1), dessen Auslastung
  mindestens `ausbau_auslastung` (*a*) und Marge mindestens `ausbau_marge` (*a*) ist und
  von dessen Erzeugung im Monat mindestens 90 % verkauft oder am selben Standort
  weiterverarbeitet wurden (M15: integrierte Werke), erhält 25 % mehr Anlagen
  (mindestens 1) mit dem günstigsten bekannten Rezept, bezahlt bis zu
  `ausbau_anteil_kasse_max` von Kasse + Kreditrahmen. Sonst erschließen Förderfirmen
  eine freie Konzession, wenn ihr Rohstoff weltweit offen nachgefragt ist und sich
  nirgends stapelt.
- **Forschung** zum Jahresbeginn bei *k* ≥ `forschung_mindestkompetenz` und
  Vorjahresumsatz ≥ `forschung_mindestumsatz_usd`: die Technologie mit dem geringsten
  Aufwand unter denen, die ein Rezept oder eine Anlage der eigenen Branchen betreffen
  und bis Jahr + `forschung_vorgriff_jahre` (*k*) erfunden sind; ohne Forschungszentrum
  wird eines mit einem Labor am Sitz gebaut.
- **Pleite:** Eine zahlungsunfähige KI-Firma scheidet aus; ihre Belegschaft wird frei,
  Angebote und Aufträge enden, ihre Konzessionen werden frei.
- **Neugründung:** Sind weniger als *N* KI-Firmen aktiv, entstehen je Monat bis zu
  `gruendungen_je_monat` neue. Gesucht wird das Produkt mit der größten offenen
  Nachfrage (Wert); fehlt einem neuen Werk ein Vorprodukt (offene Nachfrage über die
  Hälfte seines Bedarfs), wird stattdessen dieser Engpass gegründet (bis zu sechs
  Stufen). Ausgelassen werden Produkte, deren Lager weltweit mehr als
  `lager_hoch_tage` Erzeugung umfassen, und Rohstoffe ohne freie Konzession. Kapital =
  `gruendung_kapitalfaktor` · Investition; Aufbau über `FoundSite`, `DevelopDeposit`,
  `BuildFacility`, `SetProduction`, `SetSale`.
- **Diversifizierung** (M15) am letzten Tag jedes Quartals: Bis zu
  `diversifikationen_je_quartal` KI-Firmen, die reichsten zuerst (Budget =
  `ausbau_anteil_kasse_max` · (Kasse + Kreditrahmen)), bauen einen neuen Standort für
  denselben Engpass wie bei der Neugründung, jede Firma in einer anderen Kette; die
  Anlagenzahl wird so weit verringert, wie das Budget reicht.

### Händler und Preise (Ergänzungen zu M7/M8)

- Händler prüfen den Einstandspreis gegen max(Marktpreis, höchstes Gebot der
  Einkaufsaufträge im Zielland); ein Markt ohne Verkäufer bewegt sonst keinen Preis.
- Ein Angebot erhöht seinen Preis nur, wenn es Ware hatte und ausverkauft wurde.
- Automatische Preise (Anbieter und Händler) bleiben unter `hoechstfaktor` ·
  Richtpreis im Land.

## M16 – Markteintritt, Marke und Werbung

Parameter: `marktmodell.marke` in `data/parameter/marktmodell.yaml`; KI:
`kimodell.verhalten.werbeanteil`; Startbesetzung: `kimodell.start.marktdeckung`.

### Gesättigte Märkte zum Start

Die Startbesetzung plant je Produkt Anlagen für `marktdeckung` (z. B. 1,15) mal den
Bedarf bei Normalauslastung, statt für den Bedarf allein. Etablierte Firmen beginnen mit
Markenbekanntheit B_start in jedem Land und jeder Warengruppe, in denen sie zum Start
Endprodukte anbieten: `bekanntheit_start` für generierte, `bekanntheit_start_real` für
historische Firmen. Der Spieler beginnt überall mit B = 0.

Der Handel läuft vom ersten Tag an: Fehlt einem Land ein Gut (Bedarf der Anlagen,
Verbraucher und des Staates über der eigenen Erzeugung), halten die Händler dort
`vorrat_tage` dieser Lücke auf Lager. Eingekauft ist die Ware zum Richtpreis des
günstigsten Landes mit Überschuss plus Transport (E); Einfuhrpreis und Marktpreis des
Landes beginnen bei max(Richtpreis im Land, E · (1 + `haendler.marge`)). So decken die
Händler den Einkauf weiter, sobald das erste Lager verkauft ist, statt erst nach Wochen
des Mangels.

### Markenbekanntheit

B ∈ [0, 1] je Firma, Land und Warengruppe. Monatlich (Monatsanfang, für den Vormonat):

    K = kosten_je_einwohner_usd · Bevölkerung · Preisniveau · Marktmaßstab
    Werbung   = (1 − B) · (1 − exp(−w · A / K))
    Absatz    = (1 − B) · mundpropaganda · s
    B'        = min(1, B · (1 − vergessen) + Werbung + Absatz)

- A: Werbeausgaben des Monats im Land für die Warengruppe (USD), gebucht als Kostenart
  Marketing.
- w: Wirkung des besten Werbemittels, das im Jahr verfügbar ist (`werbemittel`, z. B.
  Zeitung ab 1800 mit 1,0, Radio ab 1923 mit 1,4).
- K: Ausgaben, mit denen die Werbung ein Land einmal erreicht; mit dem Marktmaßstab
  verkleinert wie die Märkte.
- s: Anteil der Firma an den Verkäufen von Endprodukten der Warengruppe an Verbraucher
  im Land im Vormonat (Mundpropaganda: Wer verkauft, wird bekannt).
- Einträge mit B < 0,001 ohne Budget entfallen.

### Anbieterwahl mit Marke und Präsenz

Der Nutzen der Endkunden (M7) bekommt einen Markenteil, und jedes Angebot zählt mit
seiner Präsenz im Handel:

    Nutzen_i  = −Preisgewicht_q · ln(p_i / r) + Qualitätsgewicht_q · (Qualität_i − 50) / 25
                + Markengewicht_q · B_i
    Gewicht_i = P_i · exp(Nutzen_i)
    Anteil_i  = Gewicht_i / Σ Gewicht

- P_i: Präsenz, was der Anbieter je Tag in die Läden bringt. Standort: geplante Erzeugung
  des Produkts je Tag (Haupt- und Nebenprodukt fertiger Anlagen bei ihrer Auslastung)
  + verfügbares Lager / `lagertage`. Händler: Importlager / `lagertage`. Staatsmarkt:
  die gesamte Verbrauchernachfrage je Tag.
- Ohne Präsenz bekäme ein Neuling mit einer kleinen Anlage bei gleichem Nutzen so viele
  Kunden wie ein großer Etablierter und verkaufte jede Menge zu jedem Preis. Mit
  Präsenz verkauft er bei gleichem Nutzen ungefähr seine Erzeugung; teurer, unbekannt
  oder schlechter bleibt er auf seiner Ware sitzen.

- B_i: Bekanntheit des Anbieters im Land für die Warengruppe des Produkts. Händlerware
  (Einfuhr) hat `bekanntheit_handel`, der Staatsmarkt `bekanntheit_staatsmarkt`.
- `markengewicht` je Einkommensfünftel, ärmstes zuerst; obere Schichten achten stärker
  auf die Marke.
- Industrie und Staat kaufen weiter nach Preis (M7); dort zählt die Marke nicht.

Beispiel: Ein Neuling (B = 0, Präsenz 1 t/Tag) neben einer bekannten Marke (B = 0,6,
Präsenz 30 t/Tag) bei gleichem Preis und gleicher Qualität, Markengewicht 1,2,
Preisgewicht 4: Anteil = 1 / (1 + 30 · exp(0,72)) ≈ 1,6 % der Nachfrage – bei einem
Markt von 31 t/Tag 0,5 t, die halbe Erzeugung. Mit 15 % niedrigerem Preis
(exp(−4 · ln 0,85) ≈ 1,9) sind es etwa 3 % – die ganze Erzeugung.

### Werbebudget

- Befehl `SetAdvertising { country, group, budget }`: Budget je Monat (≥ 0; 0 beendet die
  Werbung). Gezahlt wird am Monatsanfang aus der Kasse, auch ins Minus wie bei Einkäufen.
- KI: Am Monatsanfang setzt jede KI-Firma für jedes Land und jede Warengruppe, in denen
  sie Endprodukte an Verbraucher verkauft, das Budget = `werbeanteil` (*a*) · Umsatz der
  Warengruppe im Land im Vormonat.
