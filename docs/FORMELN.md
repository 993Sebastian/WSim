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
  Höchstförderung je Kalenderjahr und bis der Vorrat erschöpft ist (der Vorrat wächst mit
  dem Förderindex, M41). Die Jahresmenge
  verteilt sich über das Jahr: Bis zum Tag *t* darf höchstens der Anteil *t* / Tage des
  Jahres gefördert sein (M33). Die Erschließung wird über
  `erschliessung_lebensdauer_jahre` abgeschrieben.

### Tagesproduktion je Anlage

    geplante Durchläufe = Kapazität je Tag · Auslastung
    Durchläufe = min(geplant,
                     Lagerbestand(Eingang) / Menge je Durchlauf   für jeden Eingang,
                     freie Arbeitsstunden(Gruppe) / Stunden je Durchlauf   für jede Gruppe,
                     geplant · Netzanteil des Landes   (nur bei Strombedarf),
                     verbleibende Jahresförderung und Vorrat   (nur Abbau))

    Stunden je Durchlauf = Stunden laut Rezept · Arbeitsfaktor · Förderkostenfaktor (Abbau)
                           / Arbeitsproduktivität des Landes (Nacharbeit zu M16)
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
- Gemeinkosten (Verwaltung, Vertrieb, Logistik): Aufwand aus der Kasse je Durchlauf
  (Nacharbeit zu M16).
- Erzeugte Ware: Bestandserhöhung zu Herstellkosten = Eingänge + genutzte
  Arbeitsstunden · Lohn + Strom + Gemeinkosten. Nebenprodukte erhalten Kosten anteilig
  nach Menge. Läge ein Nebenprodukt danach über `nebenprodukte_lager_tage` seiner
  Tageserzeugung auf Lager, wird der Überschuss entsorgt (Nacharbeit zu M16: um 1900
  wurde Benzin abgefackelt); seine Kosten trägt die behaltene Ware.
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
  der Staatsmarkt zum Datenpreis · Preisfaktor des Landes (Abschnitt „Preise der Waren je
  Land“), in beliebiger Menge, Qualität 50.
- **Nachfrager**, in dieser Reihenfolge bedient:
  1. Einkaufsaufträge von Standorten (Industrie): Ziel-Lagerbestand, Höchstpreis,
     Mindestqualität; höchste Zahlungsbereitschaft zuerst, jeweils beim billigsten
     passenden Anbieter, nie bei der eigenen Firma.
  2. Staat: täglich `je_mio_usd_bip · BIP (Marktpreise) / 10⁶ / 365`, billigste
     Anbieter bis `staat_hoechstpreis · max(Richtpreis im Land, Richtpreis)`: Staaten
     ärmerer Länder kaufen Schienen oder Blech auch zu Weltpreisen.
  3. Endkunden, reichste Schicht zuerst.
  4. Händler für die Ausfuhr, aus dem, was die Käufer im Land übrig lassen (M8).

### Endkunden-Nachfrage (monatlich)

Je Einkommensfünftel q mit Einkommen y_q (Marktpreise) und Marktpreis p,
Richtpreis im Land r (Richtpreis · Preisfaktor):

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
  der Anbieter alles verkauft hat und ein Käufer ohne Ware blieb, der mehr als seinen
  Preis gezahlt hätte (im Land oder bei den Händlern, die für die Ausfuhr nicht genug
  fanden; Nacharbeit zu M16, siehe dort); −`runter`, wenn er weniger als
  1/`lagertage` seines Angebots verkauft hat oder seine Anlagen für das Produkt unter
  `auslastung_normal` geplant sind (freie Anlagen werben um Kunden, M16); nie unter der
  Preisuntergrenze und nie über `hoechstfaktor` · Richtpreis im Land.
- Marktpreis (Index) = (1 − g) · bisheriger Index + g · Durchschnittspreis des Tages,
  g = `index_glaettung`. Der erste Verkauf setzt den Index; vorher gilt der Richtpreis im
  Land. (Bis M16 begann die Glättung bei 0: Der Index lag dann wochenlang weit unter den
  Preisen, und Gebote nach dem Index erreichten keinen Verkäufer.)

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
- **Bedarf** eines Ziellands B: (vorrat_tage + Transporttage) · ō − Importlager −
  unterwegs; Transporttage des günstigsten Herkunftslands. (Bis M16 ohne die
  Transporttage: Bei Seewegen länger als `vorrat_tage` kam dann dauerhaft nur ein Teil
  der Nachfrage an, z. B. 60 % in China und Indien.)
- **Einkauf:** Händler kaufen nur aus Verkaufsangeboten von Firmen, die ihre
  Verkaufsfreigabe erlaubt (siehe unten). Für jedes Angebot im Land A ist der
  Einstandspreis E = Angebotspreis + Transport(A → B). Gekauft wird nur, wenn
  `Marktpreis(B) ≥ E · (1 + marge)`. Ziele mit der höchsten relativen Spanne werden zuerst
  bedient, jeweils aus den günstigsten Angeboten. Die Käufe finden bei der Räumung des
  Markts A statt, nach Industrie, Staat und Endkunden des Landes A (bis M16 vor Staat und
  Endkunden: Die Ausfuhr leerte die Märkte der Erzeugerländer). Angebote kommen nur in
  Frage, wenn E · (1 + marge) ≤ max(Marktpreis(B), `hoechstfaktor` · Richtpreis in B):
  Offene Nachfrage ist bis zum höchsten Preis, den der Markt annimmt, eine Lieferung
  wert; ein Markt ohne Verkäufer bewegt seinen Index sonst nie.
- **Verkauf** im Zielland: Das Importlager ist ein Angebot wie jedes andere. Sein Preis
  steigt um `hoch`, wenn es ausverkauft ist und Nachfrage offen bleibt, sonst sinkt er um
  `runter`, nie unter min(durchschnittlicher Einstandspreis, Wiederbeschaffungspreis) ·
  (1 + marge). Wiederbeschaffungspreis: der niedrigste Einstandspreis frischer Ware aus
  einem anderen Land (Angebotspreis + Transport). Andere Händler unterböten teuer
  eingekaufte Ware; bis M16 blieb solche Ware dauerhaft liegen, während die Werke daneben
  stillstanden.
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
Förderung_max · *s* · Anteil, bis zum Tag *t* des Jahres höchstens dieser Wert · *t* /
Tage des Jahres (M33), der Vorrat (× *s*) gilt für alle gemeinsam. Erschließen
kostet Erschließungskosten · *s* · Anteil. Spielstände vor M10 übernehmen die eine
erschlossene Lagerstätte als Konzession mit Anteil 1.

### Startbesetzung

1. **Rezept je Produkt:** das im Startjahr nutzbare Rezept mit den geringsten
   Stückkosten aus Vorprodukten zum Richtpreis, Arbeit zum `referenzlohn_usd`, Strom zum
   mittleren Strompreis der Länder und Abschreibung samt Wartung.
2. **Reale Firmen** (höchstens die Hälfte von *N*, gegründet bis zum Startjahr, Name
   nicht vom Spieler belegt) erhalten ihre Anlagen mit der Leistung Anzahl · *s* (in
   Einheiten der Datengröße), in der passenden Größe (M36: die größte, die nicht mehr
   leistet, so viele davon, wie gerundet passen, mindestens eine; C4 – vorher mindestens
   eine mittelgroße je Anlage); ihre Leistung deckt Nachfrage, ihr Bedarf erhöht die der
   Vorprodukte.
3. **Bedarf:** Produkte werden so geordnet, dass jedes nach allen Produkten kommt, die
   es verbrauchen. Je Produkt und Land gilt Bedarf = Verbraucher- + Staatsnachfrage pro
   Tag + Vorproduktbedarf der schon geplanten Anlagen − Leistung realer Anlagen.
4. **Anlagen:** *F* = Bedarf · `marktdeckung` / Tagesleistung bei Vollleistung (M16;
   vorher Bedarf / (Tagesleistung · `start.auslastung`)); unter `anlage_mindestanteil`
   keine. Die Anlagen beginnen mit `start.auslastung`. Verteilung auf Länder mit Gewicht Bedarf ·
   Entwicklung^*w* (*w* je Produktart aus `gewicht_entwicklung`), bei Rohstoffen auf
   Lagerstätten mit freien Konzessionen nach Förderung / Kostenfaktor (höchstens die
   Förderung). Ganze Zahlen nach dem größten Rest, Summe round(*F*), mindestens 1.
   **Vorprodukte geplanter Anlagen** (Nacharbeit zu M41): Liegt *F* unter
   `anlage_mindestanteil`, brauchen aber schon geplante oder reale Anlagen das Produkt,
   gibt es trotzdem eine Anlage, und zwar in der kleinsten Größe (M36), deren Leistung
   *F* deckt. Sonst fehlte das Vorprodukt ganz, und Schritt 5 verkleinerte alle Stufen
   darüber auf null: Bei einem Start ab 1920 fehlten so Bauxit und Phenolharz (Aluminium,
   Kochtöpfe, Flugzeuge, Röhrenradios), ab 1970 Reinstsilizium (Transistoren,
   Farbfernseher) und ab 2000 Lithium (Akkus, Mobiltelefone, Laptops, Digitalkameras).
5. **Vorprodukte begrenzen:** Von den Rohstoffen aufwärts wird jede Stufe auf den
   Anteil verkleinert, den ihre Vorprodukte weltweit decken (Staatsmarkt und Strom
   unbegrenzt) – sonst entstünden Werke ohne Material. Gedeckt heißt (Nacharbeit zu M16):
   Erzeugung der Vorprodukte bei Vollleistung ≥ Verbrauch bei `start.auslastung` ·
   `marktdeckung` des Vorprodukts; auch Rohstoffe, die ihre Lagerstätten begrenzen,
   behalten so die Reserve aller Märkte.
6. **Firmen:** Anlagen werden je Branche und Land gebündelt. Sind es mehr Bündel als
   Firmen, geht das kleinste in das kleinste derselben Branche auf; sind es weniger,
   wird das größte geteilt (Anlagen oder Anzahl halbiert).
7. **Ausstattung je Firma:** ein Standort je Land und Standorttyp, je Konzession ein
   Förderstandort, eigenes Kraftwerk im Land für den Strom, den das Netz nicht liefert
   (Bedarf · (1 − Netzanteil)). Lager: `lager_ausgang_tage` der Erzeugung (zu
   Herstellkosten; gleich dem Lagerziel der Erzeugung, damit nicht alle Werke am ersten
   Tag zugleich Lager aufbauen) und `lager_eingang_tage` der Vorprodukte (zum Marktpreis). Kasse:
   `kasse_monate` der laufenden Kosten. Angebot im Marktpreis-Modus mit Untergrenze aus
   den variablen Kosten, Einkaufsaufträge für alle Vorprodukte. Eigenkapital = Summe
   aller Vermögenswerte.

### Verhalten

Alle Entscheidungen sind Befehle mit derselben Prüfung wie beim Spieler; abgelehnte
Befehle unterbleiben. Sie werden am Tagesanfang getroffen und nicht ins Journal
geschrieben, weil sie sich bei der Wiederholung aus dem Zustand ergeben.

- **Betrieb** alle `betrieb_alle_tage` (*k*) Tage, erster Termin zufällig gestreut:
  - Auslastung nach Absatz und Lagerziel (M16): Abgang a je Tag = Verkäufe des Vormonats
    und des laufenden Monats / (30 + Tage des laufenden Monats) + Verbrauch eigener
    Anlagen am Standort. Ohne Verkäufe im Vormonat (neues Angebot, erster Monat der
    Partie, ausverkauft) zählt nur der laufende Monat: Verkäufe / Tage. Dann

        Ziel = lager_ziel_tage · a
        Auslastung = (a + (Ziel − (Lager − Rückhalt)) / lager_ausgleich_tage) / Vollleistung,
                     begrenzt auf [auslastung_min, 1] und auf ± auslastung_aenderung_max
                     gegenüber der bisherigen Auslastung (Nacharbeit zu M16)

    Höher wird sie nur, wenn die Anlage nicht durch fehlende Vorprodukte oder
    Arbeitskräfte gebremst war (dann hülfe es nichts). Ist der Standort ausverkauft (Lager
    höchstens ein Tag Abgang) und verkauft er über seiner Preisuntergrenze ·
    (1 + `auslastung_schritt`), steigt die Auslastung um mindestens `auslastung_schritt`:
    Der Markt will mehr, als die Anlage bisher absetzen konnte. Ohne diese Regel blieb
    eine Anlage, die einmal Käufer verloren hatte, neben einem knappen Markt klein (so
    standen Baumwollfelder in den USA bei 12 % Auslastung, während Baumwolle das
    Dreifache kostete). Ohne jeden Abgang sinkt sie um
    `auslastung_schritt`, sobald das Lager `lager_hoch_tage` der Erzeugung übersteigt.
    (Bis M16: Schritte von ±`auslastung_schritt` nach der Lagerreichweite; sie brauchten
    Monate, um der Nachfrage zu folgen. Im ersten Monat einer Partie wurde der Absatz
    durch 30 + Tage geteilt, obwohl es keinen Vormonat gab: Nach drei Tagen galt er als
    ein Zehntel des wirklichen, und die Werke fuhren auf das Minimum herunter.)
  - Neue Anlagen bekommen das günstigste bekannte Rezept; bekannte bessere Rezepte auf
    derselben Anlage ersetzen alte (z. B. nach Forschung). Strom zählt dabei mit
    Strompreis / Netzanteil des Landes (eigenes Kraftwerk im Land: Netzanteil 1), damit
    elektrische Verfahren nur gewählt werden, wo genug Strom da ist (M15).
  - Preisuntergrenze = Vollkosten je Stück · `preisuntergrenze` (*a*): variable Kosten
    (Vorprodukte zu ihren Einstandskosten im Lager, ohne Lager zum Marktpreis; Arbeit nach
    Arbeitsproduktivität, Strom, Gemeinkosten) bei der laufenden Auslastung plus
    Fixkosten (Abschreibung, Instandhaltung, bei Förderung die Erschließung über
    `erschliessung_lebensdauer_jahre`) bei der Normalauslastung `start.auslastung`
    (M15; vorher nur variable Kosten). Darüber sucht der Marktpreis-Modus den Preis. Ein
    neuer `SetSale` mit gleichem Aufschlag setzt die Preissuche nicht zurück. Die
    Einstandskosten statt des Tagespreises (Nacharbeit zu M16) dämpfen Preisspitzen: Bis
    dahin trug jede kurze Knappheit beim Getreide die Mehlpreise am selben Tag mit, auch
    wenn die Mühlen ihr Getreide billig eingekauft hatten. Mit `preisuntergrenze`
    {1,2; 1,1} statt {1,15; 1,05} liegen die Preise im Wettbewerb näher am Richtpreis:
    An der Untergrenze gilt Preis = Richtpreis, wenn die Marge zum Richtpreis
    1 − 1/Faktor beträgt.
  - Erzeugnisse ohne Angebot, die der Standort nicht selbst braucht (z. B. Benzin als
    Nebenprodukt der Raffinerie), werden zum Marktpreis ohne Untergrenze angeboten.
  - Einkauf: Ziel = `lager_eingang_tage` · Tagesbedarf, Höchstpreis = Marktpreis ·
    (1 + `einkauf_aufschlag`); fehlt Ware (unter `lager_niedrig_tage`), steigt das Gebot
    je Durchgang um `auslastung_schritt` bis zur Zahlungsbereitschaft (M16), höchstens
    `hoechstfaktor` · Richtpreis im Land. Zahlungsbereitschaft für ein Vorprodukt *i*
    (die beste Verwendung am Standort):

        ZB_i = (Erlös je Tag − (variable Kosten je Tag − Menge_i · Marktpreis_i)) / Menge_i
        Erlös = Durchläufe · (Menge · Angebotspreis + Σ Nebenprodukte · Angebotspreis)

    (ohne eigenes Angebot zählt der Marktpreis). Die Werke zahlen also, was ihr Erzeugnis
    nach allen übrigen Kosten trägt; steigt ein Vorprodukt darüber, steht das Werk, bis
    sein eigener Preis nachzieht. Bis M16 endete das Gebot beim Dreifachen des
    Marktindex – der aber steht still, solange nichts verkauft wird.
  - Bremst nur das Stromnetz eine Anlage, darf die Auslastung trotzdem steigen: Das Netz
    liefert einen Anteil des Geplanten, mehr Planung bringt also mehr Erzeugung (bis M16
    sank sie dann nur noch, bis zum Minimum).
  - Eigene Ware: Eine Firma kann auf dem Markt nicht bei sich selbst kaufen. Fehlende
    Vorprodukte holt sie per `TransferGoods` von eigenen Standorten, die sie anbieten
    (zuerst im selben Land, sonst per Fracht).
- **Kasse** am Monatsanfang: unter `kasse_min_monate` laufender Kosten ein Kredit über
  `kredit_jahre` bis zur Mitte zwischen Minimum und Maximum (höchstens der
  Kreditrahmen), über `kasse_max_monate` Tilgung.
- **Ausbau** am letzten Tag jedes Quartals:
  - Zuerst **Eigenstrom** (Nacharbeit zu M16): Bremst das Netz Anlagen der Firma in einem
    Land (Grenze Strom), baut sie dort ein Kraftwerk mit dem günstigsten Rezept für den
    fehlenden Strom (Anzahl = fehlende MWh je Tag / Leistung bei `start.auslastung`,
    aufgerundet), im vorhandenen Kraftwerksstandort oder einem neuen – wie die
    Startbesetzung. Eigene Kraftwerke im Bau oder im Wiederanlauf zählen mit ihrer
    Leistung bei `start.auslastung` gegen den fehlenden Strom (M30: sonst käme während
    der Bauzeit jedes Quartal ein weiteres hinzu). Sonst:
  - das Produkt mit der höchsten Marge (Angebotspreis, ohne Angebot Marktpreis /
    Stückkosten − 1), dessen Auslastung mindestens `ausbau_auslastung` (*a*) und Marge
    mindestens `ausbau_marge` (*a*) ist und von dessen Erzeugung im Monat mindestens 90 %
    verkauft oder am selben Standort weiterverarbeitet wurden (M15: integrierte Werke),
    erhält 25 % mehr Anlagen (mindestens 1) mit dem günstigsten bekannten Rezept, bezahlt
    bis zu `ausbau_anteil_kasse_max` von Kasse + Kreditrahmen. Ausgenommen sind Produkte,
    deren Anlagen am Standort auf Arbeitskräfte oder ein Vorprodukt warten oder bei denen
    ein Vorprodukt unter `lager_niedrig_tage` des Verbrauchs liegt oder im Land mehr als
    `ausbau_vorprodukt_preis_max` · Richtpreis kostet (knapp): Weitere Anlagen würden nur
    um dieselbe knappe Ware bieten (bis M16 wuchsen so die Spinnereien auf das Fünffache,
    während die Baumwolle fehlte). Förderanlagen wachsen nur, solange die
    Konzession mehr erlaubt, als sie fördern (Höchstförderung · Maßstab · Anteil / 365 −
    Vollleistung, in ganzen Anlagen).
  - Sonst erschließen Förderfirmen eine freie Konzession, wenn ihr Rohstoff weltweit offen
    nachgefragt ist und sich nirgends stapelt (Lagerstätte und Zahl der Anlagen wie
    bei der Neugründung, M33).
- **Forschung** zum Jahresbeginn bei *k* ≥ `forschung_mindestkompetenz` und
  Vorjahresumsatz ≥ `forschung_mindestumsatz_usd`: die Technologie mit dem geringsten
  Aufwand unter denen, die ein Rezept oder eine Anlage der eigenen Branchen betreffen
  und bis Jahr + `forschung_vorgriff_jahre` (*k*) erfunden sind; ohne Forschungszentrum
  wird eines mit einem Labor am Sitz gebaut. Seit M32 zählen auch **Marktlücken**:
  Produkte, nach denen Endkunden oder Staat fragen, und die Vorprodukte ihrer nutzbaren
  Rezepte, die keine aktive Firma herstellen kann (jedes Rezept braucht eine Technologie
  des Rezepts oder der Anlage, die keine aktive Firma kennt). Ihre Technologien und alle
  Voraussetzungen erforscht eine Firma, die in den eigenen Branchen nichts zu
  erforschen hat (wieder die billigste), höchstens `forschung_luecke_firmen` Firmen
  zugleich je Technologie; eine Firma behält ihre Lücke. Sonst erforschte niemand
  Produkte neuer Branchen (Luftfahrt). Seit M33 zählen auch die Technologien der
  Rezepte teurer Märkte (Einstieg in teure Märkte, M33), solange weniger als
  `einstieg_firmen_max` Firmen das Produkt herstellen: Technologien werden erst
  `gemeingut_nach_jahren` nach ihrer Erfindung Allgemeingut, und ohne eigene Forschung
  käme bis dahin kein Wettbewerber hinzu. Die Lücken werden einmal je Jahr vor den
  Forschungsplänen bestimmt.
- **Pleite:** Eine zahlungsunfähige KI-Firma scheidet aus; ihre Belegschaft wird frei,
  Angebote und Aufträge enden, ihre Konzessionen werden frei.
- **Neugründung:** Sind weniger als *N* KI-Firmen aktiv, entstehen je Monat bis zu
  `gruendungen_je_monat` neue. Gesucht wird das Produkt mit der größten offenen
  Nachfrage (Wert); fehlt einem neuen Werk ein Vorprodukt (offene Nachfrage über die
  Hälfte seines Bedarfs), wird stattdessen dieser Engpass gegründet (bis zu sechs
  Stufen). Der Kette folgt die KI mit dem eigenen Rezept oder, wenn sie das Produkt
  selbst nicht herstellen kann, mit einem Rezept, das eine Firma nutzt oder kennt (M32);
  bauen muss sie das Glied, das sie selbst herstellen kann – eine Bauxitgrube braucht
  kein Wissen über Kochtöpfe. Ausgelassen werden Produkte, deren Lager weltweit mehr als
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

Die Startbesetzung plant je Produkt Anlagen, die bei Vollleistung `marktdeckung` (je
Produktart, derzeit überall 1,15) mal den Bedarf herstellen könnten, statt für den
Bedarf allein. Mehr freie
Kapazität löste über die Regel „freie Anlagen werben um Kunden“ (M7) einen dauerhaften
Preiskampf aus. Etablierte Firmen beginnen mit
Markenbekanntheit B_start in jedem Land und jeder Warengruppe, in denen sie zum Start
Endprodukte anbieten: `bekanntheit_start` für generierte, `bekanntheit_start_real` für
historische Firmen. Der Spieler beginnt überall mit B = 0.

Der Handel läuft vom ersten Tag an: Fehlt einem Land ein Gut (Bedarf der Anlagen,
Verbraucher und des Staates über der eigenen Erzeugung), halten die Händler dort diese
Lücke für `vorrat_tage` plus die Transporttage vom günstigsten Land auf Lager. Eingekauft ist die Ware zum Richtpreis des
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

## Nacharbeit zu M16 – Plausibilität und Versorgung

Parameter: `marktmodell.preisniveau_anteil`, `marktmodell.preisanpassung` (`hoch`,
`runter`, `auslastung_normal`), `produktionsmodell.gemeinkosten_anteil`,
`produktionsmodell.richtpreis_marge`, `laendermodell.produktivitaet`,
`kimodell.verhalten.lager_ziel_tage`, `lager_ausgleich_tage` und
`auslastung_aenderung_max`; am Rohstoff `pacht_anteil` und `foerderindex`.

Grundsatz: Jede Regel gilt für alle Produkte und Länder gleich und hängt nur von der
Produktart, den Daten und Marktsignalen ab (Absatz, Lager, Auslastung, offene Nachfrage).
Ein neues Produkt braucht keine eigene Regel; die KI reagiert auf jeden Spielverlauf mit
denselben Signalen.

### Preise der Waren je Land

    Preisfaktor(Land, Produkt) = Preisniveau(Land) ^ preisniveau_anteil(Produktart)
    Richtpreis im Land        = Richtpreis · Preisfaktor

Waren werden gehandelt; ihre Preise unterscheiden sich zwischen den Ländern viel weniger
als das Preisniveau, das vor allem Löhne und Dienstleistungen erfasst (Balassa-Samuelson).
Dem Preisniveau folgt nur der Anteil von Löhnen, Handel und Vertrieb im Land: bei
Vorprodukten kaum (0,1), bei Endprodukten im Laden zu 0,4; Strom kommt aus dem Netz im
eigenen Land (1). Der Richtpreis im Land gilt überall, wo bisher Richtpreis · Preisniveau
stand: Startpreise, Preisobergrenzen, Staatsmarkt. Auch ein Markt ohne Verkäufe beginnt
beim Richtpreis im Land (bis zur Nacharbeit begannen Märkte mit Verbrauchernachfrage beim
Richtpreis · Preisniveau, die übrigen beim Richtpreis im Land; Nägel kosteten zum Start
weniger als der Draht daraus). Die Kaufneigung der Verbraucher vergleicht weiter mit
Richtpreis · Preisniveau, denn die Einkommen sind in Marktpreisen gerechnet.

Bis M16 galt der volle Faktor für alle Waren. In einem Land mit Preisniveau 0,5 kostete
eingeführter Draht dann das Doppelte des Richtpreises im Land; Nägel daraus fanden zum
Richtpreis keinen Käufer, und die Preisuntergrenzen lagen beim 1,5- bis 2-Fachen. Folge
der Änderung: Verbraucher ärmerer Länder kaufen weniger Industriewaren, als ihr
Einkommen in Kaufkraft erwarten ließe – Industriewaren sind dort relativ teuer.

### Herstellkosten

- **Arbeitsproduktivität** Π = clamp((y / `bezug_usd`)^`elastizitaet`, `minimum`,
  `maximum`) mit y = BIP je Kopf (Kaufkraft). Stunden je Durchlauf = Stunden laut Rezept ·
  Arbeitsfaktor · Förderkostenfaktor / Π. Mit Elastizität 1 kostet die Arbeit je Stück in
  Kaufkraft überall gleich viel; in Marktpreisen folgt sie dem Preisniveau.
- **Gemeinkosten** (Verwaltung, Vertrieb, Logistik) je Durchlauf, gebucht als Kostenart
  „Verwaltung und Vertrieb“ aus der Kasse und in den Herstellkosten der Ware – als
  Zuschlag auf die Umwandlungskosten wie in der Zuschlagskalkulation:

      Umwandlung   = Arbeit + Strom + Anlage je Durchlauf
                     (Anlage: Investition · (1/Lebensdauer + Wartung) /
                      (365 · Durchläufe je Tag · auslastung_normal))
      Gemeinkosten = gemeinkosten_anteil(Produktart) · Umwandlung

  Die Rezepte enthalten nur direkte Kosten; ohne Gemeinkosten lagen die Kosten so weit
  unter den Richtpreisen, dass der Wettbewerb die Preise auf 20–50 % des Richtpreises
  drückte. Der Zuschlag auf die Umwandlung belastet Durchlaufgeschäfte wie Drahtziehen
  oder Mahlen wenig und Fertigwaren mit viel Arbeit stark, wie in der Wirklichkeit.
  Fließbandfertigung senkt ihn mit der Arbeit. Er hängt nicht an den Preisen; sie können
  daher nicht unter die echten Kosten sinken. (Erprobt und verworfen: ein Anteil der
  Wertschöpfung zu Richtpreisen hielt das Auto trotz Fließband teuer; ein Anteil der
  Wertschöpfung zu Marktpreisen fiel mit den Preisen und drückte Grundstoffe bis 1929
  auf ein Drittel des Richtpreises.) Liegt der Preis an der Untergrenze f · Vollkosten,
  gilt im Gleichgewicht P = f · (Eingänge + (1 + s) · Umwandlung).
- **Pacht und Förderabgaben** je geförderter Einheit eines Rohstoffs, gebucht als
  Kostenart „Pacht und Förderabgaben“ aus der Kasse und in den Herstellkosten:

      Pacht = pacht_anteil(Rohstoff) · Richtpreis im Land · Menge

  Bodenrente, Förderzins und Konzessionsabgaben (Pächter im Baumwollgürtel gaben ein
  Drittel bis die Hälfte der Ernte ab, Ölförderer ein Achtel als Förderabgabe). Ohne sie
  kostete die Förderung von Getreide, Baumwolle, Rohöl, Kautschuk und Kupfererz nur ein
  Drittel bis die Hälfte ihres Richtpreises; wo keine Lagerstätte knapp war, fielen die
  Preise im Wettbewerb dorthin und zogen die ganzen Ketten mit (1929: Getreide 0,37,
  Mehl 0,44, Kautschuk 0,38, Gummi 0,25, Rohöl 0,43 des Richtpreises). Mit Pacht 1929:
  0,86, 0,90, 0,81, 0,58, 0,94. Werte: Baumwolle 0,5, Getreide und Rohöl 0,45, Kautschuk
  0,4, Kupfererz 0,35, Holz 0,2, Eisenerz 0,15, Kohle 0.
- **Vollkosten je Stück** (Kern: `health::unit_cost`): Eingänge zu Preisen, Arbeit
  (Stunden · Lohn / Π), Strom, Abschreibung und Instandhaltung bei der Auslastung,
  Gemeinkosten, Pacht, abzüglich der Nebenprodukte; alles geteilt durch die Menge je
  Durchlauf.

### Plausibilitätsprüfung der Daten

Beim Laden (`wsim validate`) gilt für jedes Produkt außer Energie: Das beste seiner
Rezepte im ersten Jahr, in dem es hergestellt werden kann (Technik von Rezept und Anlage,
frühestens 1900), erzielt im Referenzland des Preisniveaus (Jahresmitte) eine Marge in
`richtpreis_marge`:

    Marge = 1 − Vollkosten (Eingänge zu Richtpreisen, Normalauslastung) / Richtpreis

Förderrezepte zählen mit ihrer Pacht und werden wie alle anderen in beide Richtungen
geprüft (bis zur Einführung der Pacht nur nach unten). Sonst gibt es eine Warnung mit
Datei und Zeile des Rezepts. `wsim rezepte --jahr J --laender …`
zeigt dieselbe Rechnung für beliebige Länder und Jahre.

Die erste Prüfung der Daten (1900, USA) fand: Nägel −7 %, Handwerkzeug −8 %, Möbel −3 %,
Glühlampe −13 %, Nähmaschine 4 % (zu viele Arbeitsstunden oder zu wenig Spanne zum
Vorprodukt), Schnittholz 53 % und Benzin 47 % (zu hoch). Korrigiert: Nägel aus 1,02 t
Draht zum Richtpreis 1.900 USD; Stunden für Werkzeug, Möbel, Glühlampen und Nähmaschinen
nach historischen Stückzahlen; Schnittholz zum Richtpreis 300 USD aus 1,8 t Holz;
Cracken mit 0,15 t Petroleum als Nebenprodukt. Alle Verarbeitungsrezepte liegen nun
zwischen 8 und 42 %.

### Preisbildung im Wettbewerb

- **Freie Anlagen werben um Kunden:** Liegt die geplante Auslastung der Anlagen eines
  Anbieters für das Produkt unter `auslastung_normal`, sinkt sein automatischer Preis
  täglich um `runter`, solange er nicht ausverkauft ist (M7). Mit einer Untergrenze aus
  Vollkosten · `preisuntergrenze` (M10) pendelt der Preis in gesättigten Märkten knapp
  über den Vollkosten: Produkte mit hoher Marge zum Richtpreis werden dann billiger als
  ihr Richtpreis, solche mit geringer bleiben nahe daran.
- **Knappheit über Grenzen:** Finden die Händler für den Bedarf der Einfuhrländer nicht
  genug Angebote, gilt jeder ausverkaufte Anbieter als knapp und erhöht seinen Preis –
  auch wenn im eigenen Land niemand mehr wartet.
- **Knapp ist nur, wofür jemand zahlt:** Bei der Räumung eines Markts merkt sich der Kern
  den höchsten Preis, den ein Käufer ohne Ware gezahlt hätte: das Gebot eines Werks, die
  Preisgrenze des Staats (`staat_hoechstpreis`), bei Verbrauchern und Händlern
  `hoechstfaktor` · Richtpreis im Land. Ein ausverkaufter Anbieter erhöht seinen Preis
  nur, solange er darunter liegt. Bis zur Nacharbeit zählte jede offene Nachfrage: Kaufte
  der Staat oberhalb seiner Grenze nicht, hoben die Anbieter ihre Preise weiter, und der
  Staat kaufte erst recht nicht – Nägel (Bauwesen) und Mehl (Heer) stiegen so im ersten
  Jahr auf das Drei- bis Vierfache.
- **Preisschritte:** `hoch` 0,25 % und `runter` 0,125 % je Tag, also etwa +8 % und −4 % im
  Monat (vorher 2 % und 1 %). Die Erzeugung folgt den Preisen über Auslastung und Ausbau
  erst in Wochen und Monaten; schnellere Preise schaukelten sich auf (Getreide 1900
  zwischen 0,55 und 3,9 des Richtpreises). Im Weltlauf 1900–1930 mit 100 KI-Firmen gab
  es mit den langsamen Schritten in allen Prüfungen weniger Verstöße (Varianten mit 1 %
  und 0,5 % je Tag lagen dazwischen).

### Förderung im Lauf der Zeit

    Höchstförderung(Lagerstätte, Jahr) = foerderung_max_je_jahr · foerderindex_Rohstoff(Jahr)

Der Förderindex (Jahreswerte am Rohstoff, ohne Angabe 1) bildet mehr Anbaufläche und
höhere Erträge ab: Getreide 1930 das 1,45-Fache von 1900, Baumwolle (für alle Fasern)
das 1,65-Fache (Baumwollernte 3,6 → 6,0 Mio. t), Holz das 1,3-Fache. Kleidung braucht
seit der Nacharbeit 5 statt 6 Stück je Kopf und Jahr bei voller Kaufneigung: Der
Faserbedarf lag 1900 bei 6,6 Mio. t, die Welt erzeugte etwa 5 Mio. t, und die Baumwolle
blieb bis 1929 beim Drei- bis Vierfachen des Richtpreises. Ohne ihn blieb die Förderung auf dem Stand von 1900, während
die Nachfrage mit der Bevölkerung wuchs; Getreide und Baumwolle kosteten dann über
Jahrzehnte das Vierfache des Richtpreises. Die Konzessionen behalten ihre Anteile; ihre
Förderanlagen wachsen über den Ausbau der KI (M10) mit.

### Startbesetzung und Handel

- Die Anlagen der Startbesetzung sind bei Vollleistung für `marktdeckung` mal den Bedarf
  bemessen (M10, Schritt 4).
- Händler bemessen ihren Bedarf mit den Transporttagen (M8) und halten zum Start den
  Vorrat dafür bereit (M16).
- **Start im Gleichgewicht:** Jedes Angebot der Startbesetzung beginnt mit einem Absatz im
  Vormonat von 30 Tagen seiner geplanten Erzeugung (ohne Eigenverbrauch). Bis zur
  Nacharbeit schätzte die KI den Absatz im ersten Monat aus den ersten Tagen, während
  Märkte und Händler erst anliefen, und drosselte ganze Ketten auf einmal (die größte
  US-Mühle am 1. Februar 1900 von 90 % auf 5 %); das löste den Getreidezyklus aus.
- **Gedämpfte Auslastung:** Die geplante Auslastung einer Anlage ändert sich je
  Entscheidung höchstens um `auslastung_aenderung_max` (0,3). Ein schwacher Monat legt
  kein Werk still, ein starker füllt nicht alle Lager zugleich (Peitscheneffekt über die
  Lagerstufen Farm – Mühle – Händler).

### Marktgesundheit und Prüfungen im Weltlauf

`wsim run --ki 100 --bis 1930-01-01 --protokoll <Ordner>` schreibt in `auswertung.md` die
Prüfungen je Jahr (Kern: `health::last_month`). Die Grenzen gelten für alle Produkte und
Länder gleich und stehen im Protokoll-Code (sie sind keine Spielregeln):

| Prüfung | Grenze |
| --- | --- |
| Versorgung von Verbrauchern und Staaten weltweit | ≥ 90 % |
| Versorgung je Land (ab 0,5 % der Weltnachfrage) | ≥ 75 % |
| Erzeugung, die auf fehlende Vorprodukte wartet | ≤ 10 % |
| Erzeugung, die auf fehlende Arbeitskräfte wartet | ≤ 10 % |
| Preis gegen Richtpreis im Land | 0,5 bis 2 × |
| Marge über Vollkosten | −20 % bis 50 % |
| Jahresergebnis des passiven Spielers gegen sein Startkapital | ≤ 50 % |
| Pleiten je Jahr; aktive KI-Firmen | ≤ 5 %; ≥ 80 % der Startzahl |
| Förderung von Rohstoffen gegen den Bedarf der Anlagen | ≥ 90 % |

## M18 – Personal und Lohnaufschlag (Entscheidung vom 05.10.2026)

Parameter: `produktionsmodell.lohnaufschlag_max`; KI:
`kimodell.verhalten.lohnaufschlag_schritt` und `lohnaufschlag_max`.

- **Lohnaufschlag** *a* je Standort, 0 ≤ *a* ≤ `lohnaufschlag_max` (Befehl
  `SetWagePremium`). Lohn je Stunde am Standort = Lohn(Land, Gruppe) · (1 + *a*). Er gilt
  für die täglichen Lohnkosten, für den Arbeitsanteil im Wert der Erzeugnisse und in den
  Kostenschätzungen der KI.
- **Besetzung** (zu Monatsbeginn und nach Änderungen, wie M5): Bedarf je Gruppe =
  Σ geplante Durchläufe · Stunden je Durchlauf / Arbeitsstunden je Tag, dazu die Forscher.
  Die fälligen Standorte werden nach Lohnaufschlag besetzt (höchster zuerst), bei gleichem
  Aufschlag in der Reihenfolge ihrer Gründung. Reichen die freien Kräfte des Landes nicht,
  **wirbt** ein Standort die fehlenden von Standorten desselben Landes **ab**, die einen
  niedrigeren Aufschlag zahlen: zuerst vom niedrigsten Aufschlag, bei gleichem vom jüngsten
  Standort (Lastenheft §5.3). Wer Leute verliert, wird erneut besetzt und produziert bis
  dahin mit weniger Personal. Bei gleichem Aufschlag wirbt niemand ab; ohne Aufschläge
  bleibt alles wie bisher.
- **KI:** Bei jeder Betriebsentscheidung hebt eine KI-Firma den Aufschlag eines Standorts
  um `lohnaufschlag_schritt` (höchstens ihr `lohnaufschlag_max`), wenn dort am letzten Tag
  eine Anlage auf Arbeitskräfte wartete; sonst senkt sie ihn um den Schritt bis 0.
  Knappe Arbeitsmärkte heben so die Löhne, entspannte senken sie wieder.

## M18 – Stückkosten, Ergebnis je Standort und Preis setzen

- **Stückkosten** eines Erzeugnisses am Standort (Anzeige der Werksansicht, keine
  Wirkung auf die Simulation), je Anlage mit dem Rezept bei geplanter Auslastung *u*
  (ruhende Anlagen: *u* = 1), Durchläufe je Tag *n* = Durchläufe je Anlage · Anzahl · *u*:
  - Material = Σ Menge · Marktpreis im Land; Vorprodukte, die der Standort selbst
    herstellt, mit ihren eigenen Stückkosten (wiederholt gerechnet, bis die Kette
    durchlaufen ist).
  - Personal = Σ Stunden je Durchlauf (mit Automatisierung, Lagerstätte und
    Produktivität wie in der Erzeugung) · Lohn · (1 + *a*).
  - Energie = MWh je Durchlauf · Industriestrompreis; Gemeinkosten und Pacht wie in der
    Erzeugung (M16).
  - Anlage = Anschaffungskosten der Anlagen · (1 / Lebensdauer + Instandhaltungsanteil)
    / 365 je Tag, verteilt auf die Tagesmenge.
  - Je Stück = Summe je Tag / Erzeugung je Tag; mehrere Anlagen desselben Erzeugnisses
    werden mengengewichtet zusammengefasst. Die variablen Stückkosten sind alles ohne
    Anlage. Nebenprodukte werden nicht gutgeschrieben; Gebäude und Erschließung stehen
    nur im Ergebnis des Standorts.
- **Interne Verrechnung der Löhne:** Die Löhne der in einem Durchlauf genutzten Stunden
  (die schon im Wert der Erzeugnisse stecken) werden von der Kostenstelle des Standorts
  auf die Kostenstelle des Erzeugnisses umgebucht. Summen je Kostenart und Bilanz bleiben
  gleich. Das Ergebnis je Erzeugnis ist damit Umsatz − Herstellkosten des Absatzes
  (Rohertrag); auf dem Standort bleiben die Löhne ungenutzter Stunden, Instandhaltung
  und Abschreibungen.
- **Ergebnis je Kostenstelle und Kostenart** wird für den laufenden und den letzten
  abgeschlossenen Monat (und das Jahr) geführt; ältere Monate behalten nur die Summen.
- **Preis setzen** (Befehl `SetPrice`): Ein fester Preis wird zum neuen festen Preis. Ein
  automatischer Preis springt auf den eingegebenen Wert (nicht unter die Untergrenze) und
  folgt von dort wieder Angebot und Nachfrage.

## M19 – Technologiebaum (Anzeige)

Die Werte sind Anzeigen der Forschungsansicht (`views::research_overview`) und wirken
nicht auf die Simulation.

- **Stand** einer Technologie: *bekannt* (die Firma kennt sie, auch als Gemeingut),
  *in Arbeit* (ein eigenes Forschungszentrum forscht daran), *erforschbar* (alle
  Voraussetzungen bekannt), sonst *Voraussetzungen fehlen*.
- **Noch nötige Punkte** R = Aufwand heute (M9, mit Faktor für Vorgriff oder Nachzügler)
  − gesammelte Punkte der Firma. Der Faktor wird mit angezeigt; vor dem historischen
  Jahr sinkt er mit der Zeit.
- **Mit deinen Zentren:** Punkte je Tag P = Σ über die eigenen Zentren an dieser
  Technologie: Forscher · Forschungseffizienz des Landes im Fachgebiet; Dauer = R / P.
- **Mit einem Labor:** ein voll besetztes Labor (F = `kapazitaet_je_tag` Forscherplätze)
  im Land des ersten eigenen Forschungszentrums, sonst am Sitz der Firma:

      P₁     = F · Forschungseffizienz(Land, Fachgebiet)
      Dauer  = R / P₁
      Kosten = Dauer · F · (Lohn(Land, Forschergruppe) · Arbeitsstunden je Tag
                            + sachkosten · Preisniveau(Land))

  ohne den Bau des Labors.
- **Schaltet frei:** Anlagen, die die Technologie verlangen; Verfahren (Rezepte), die sie
  selbst verlangen oder deren Anlage sie verlangt – fehlt der Anlage noch eine andere
  Technologie, wird diese genannt; Produkte, die eines dieser Verfahren herstellt.
- **Darstellung:** Spalten nach Zeitabschnitt der historischen Erfindung (bis 1850,
  1851–1875, 1876–1899, 1900–1909, 1910–1919, ab 1920), Zeilen nach Fachgebiet, Linien zu
  den Voraussetzungen.

## M20 – Rundenbericht und Einführung (Anzeige)

- Rundenbericht: Umsatz − Kosten + Lagerveränderung = Ergebnis. Die Lagerveränderung
  ist die Kostenart Bestandsveränderung: hergestellte, noch nicht verkaufte Ware zu
  Herstellkosten (positiv, wenn das Lager wächst).
- Die Einführung gilt als „erster Verkauf“ erreicht, wenn der Rundenbericht einen
  Umsatz über 0 ausweist; sonst führt sie zur nächsten Runde.

## M21 – Landeswährungen (Anzeige)

Lastenheft §3.6 und §18.2. Das Spiel rechnet weiter in US-Dollar mit der Kaufkraft des
Basisjahrs B = 2026 (`data/waehrungen/preisindex.yaml`). Währungen ändern nur die
Anzeige (`wsim_core::currency`, Sicht `Overview.money`); sie wirken nicht auf die
Simulation und stehen nicht im Spielstand.

- **Zeitpunkt** t als Jahr mit Bruchteil: 1. Januar 1900 = 1900,0. Ein Jahreswert der
  Daten steht für das Jahresmittel (t = Jahr + 0,5), ein Monatswert ("1923-11") für die
  Monatsmitte (t = Jahr + (Monat − 0,5)/12). Ein Zeitraum `ab: 1999` beginnt am
  1. Januar, `ab: "1948-06"` am 1. des Monats.
- **Kurs** e(W, t) = Einheiten der Währung W je US-Dollar der Zeit t. Zwischen zwei Werten
  (a, eₐ) und (b, e_b) auf logarithmischer Skala:

      e(t) = exp( ln eₐ + (ln e_b − ln eₐ) · (t − a)/(b − a) )

  Vor dem ersten und nach dem letzten Wert bleibt der Kurs gleich. Eine **gebundene**
  Währung hat e(W, t) = k · e(A, t) mit dem festen Faktor k je Einheit der Ankerwährung A.
- **Preisindex** P(t) der USA: linear zwischen den Jahresmitten; vor dem ersten Wert
  der erste Wert, nach dem letzten P(t) = P_letzt · (1 + π)^(t − t_letzt) mit
  π = `teuerung_danach`. Teuerung seit dem Basisjahr: I(t) = P(t) / P(B + 0,5).
- **Währung eines Landes** zum Zeitpunkt t: die des letzten Zeitraums mit Beginn ≤ t.
- **Umrechnungsfaktor** f (gezeigter Betrag = Spieldollar · f) für die vier Wahlmöglichkeiten:

  | Anzeige | Währung | Faktor |
  |---|---|---|
  | Landeswährung, Kaufkraft 2026 | Währung des Firmensitzes in B | e(W_B, B + 0,5) |
  | Landeswährung, Preise der Zeit | Währung des Firmensitzes zum Spieldatum | e(W_t, t) · I(t) |
  | US-Dollar, Kaufkraft 2026 | US-Dollar | 1 |
  | US-Dollar, Preise der Zeit | US-Dollar | I(t) |

  Beispiel Deutschland am 1.1.1914: e(Mark) = 4,198 M/$, I = 9,95/330 = 0,0302, also
  f = 0,1266 M je Spieldollar; mit der Kaufkraft 2026 f = 0,87 € je Spieldollar.
- Eingaben in Formularen gelten in der gezeigten Währung: Spieldollar = Eingabe / f.
- Alle Beträge eines Bildschirms nutzen den Faktor des aktuellen Spieldatums, auch
  Verläufe und Vormonatswerte; so bleiben sie untereinander vergleichbar.
- „Kaufkraft 2026“ zeigt die Landeswährung des Basisjahrs (in Deutschland den Euro):
  Eine Mark oder Reichsmark „mit der Kaufkraft von 2026“ gibt es nicht. Die damaligen
  Währungen mit ihren Umstellungen erscheinen mit „Preise der Zeit“.

## M22 – Anlagen stilllegen und verkaufen

Lastenheft §18.3. Parameter: `data/parameter/produktionsmodell.yaml` (`stilllegung`,
`verkauf`), für die KI `data/parameter/kimodell.yaml` (`stilllegen_auslastung`,
`stilllegen_zielauslastung`, `verkaufen_nach_monaten`). Spieler und KI nutzen dieselben
Befehle.

- **Stilllegen** (`MothballFacility`, *k* der *n* Einheiten einer Anlage): Sind es nicht
  alle, werden die *k* Einheiten ein eigener Platz mit dem Anteil *k*/*n* der Investition,
  gleichem Zustand, Rezept und gleicher Automatisierung; laufende Chargen bleiben beim
  weiterlaufenden Teil. Eine stillgelegte Anlage produziert nicht, braucht kein Personal
  (es wird bei der nächsten Besetzung frei) und verschleißt nicht. Sie kostet
  `stilllegung.instandhaltung_anteil` · Wartung; die Abschreibung läuft weiter.
- **Wieder anfahren** (`RestartFacility`): kostet einmalig
  `stilllegung.wiederanlauf_kosten` · Investition (Instandhaltung) und dauert
  `stilllegung.wiederanlauf_tage` Tage mit voller Wartung; danach produziert die Anlage
  mit Rezept und Auslastung von vorher.
- **Verkaufen** (`SellFacility`, *k* Einheiten). Mit dem Alter seit der Fertigstellung:

      Restbuchwert RBW = Investition · k/n · max(0, 1 − Alter / Lebensdauer)
      Erlös E         = max(verkauf.erloes_anteil · RBW, verkauf.schrottwert · Investition · k/n)

  Buchung: Kasse + E, Anlagen − RBW, die Differenz E − RBW als sonstiger Ertrag bzw.
  Aufwand. Laufende Chargen einer ganz verkauften Anlage werden sofort fertig (die Ware
  kommt ins Lager).
- Anlagen im Bau lassen sich weder stilllegen noch verkaufen.
- **KI** am letzten Tag jedes Quartals, vor dem Ausbau, je Standort und angebotenem
  Produkt (ohne Strom, Forschung und Förderung – Förderanlagen hängen an ihrer
  Konzession; Waren, die die Firma nur selbst verwendet oder zu eigenen Standorten
  schickt, misst kein Absatz):
  - Abgang *A* je Tag = Verkäufe des Vormonats und des laufenden Monats / (30 + Tage)
    + Verbrauch eigener Anlagen am Standort (wie beim Betrieb), *V* = Vollleistung der
    laufenden Einheiten, *n* ihre Zahl, Bedarf *d* = *A* / *V*.
  - **Wieder anfahren:** Ist *d* größer als `wiederanfahren_auslastung` und läuft keine
    Einheit des Produkts gerade wieder an, fährt die KI stillgelegte Einheiten wieder an,
    bis *A* höchstens `stilllegen_zielauslastung` · (laufende + anlaufende Vollleistung)
    ist.
  - **Stilllegen:** Ist *d* kleiner als `stilllegen_auslastung` und der Marktpreis im
    Land unter `stilllegen_preis_max` · Richtpreis, legt sie die Einheiten still, die
    bei `stilllegen_zielauslastung` (*Z*) nicht gebraucht werden: *n* − ⌈*n* · *d* / *Z*⌉,
    mindestens eine bleibt. Die teuersten Anlagen (Stückkosten bei voller Auslastung)
    zuerst. Die übrigen fahren mit der bisherigen geplanten Auslastung · *n* /
    (verbleibende Einheiten) weiter, damit die Erzeugung nicht einbricht.
  - Warum so: Der erste Weltlauf maß die geplante Auslastung statt des Abgangs und
    kannte keine Preisbedingung – alle Anbieter knapper Märkte legten zugleich still,
    Baumwolle, Garn, Stoff, Kleidung, Getreide und Mehl stiegen auf das Drei- bis
    Vierfache. Ein Wiederanfahren bei jeder Betriebsentscheidung ließ Anlagen im
    Tagesrhythmus pendeln (1 730 Stilllegungen und 1 286 Wiederanläufe in zwölf Jahren);
    vierteljährlich mit Abstand zwischen 0,5 und 0,95 sind es rund 850 und 300.
  - Wählt der Ausbau ein Produkt, von dem am Standort Einheiten stillliegen, fährt sie
    zuerst diese wieder an; solange welche wieder anlaufen, baut sie dafür nicht aus.
  - Einheiten, die länger als `verkaufen_nach_monaten` stillliegen, verkauft sie.
  - Legt ein Wettbewerber Anlagen eines Produkts still oder verkauft er sie, das der
    Spieler herstellt oder anbietet, steht das im Rundenbericht.

### Preise holen nach einer Schwemme auf

Ein knapper Anbieter (ausverkauft, ein Käufer hätte mehr gezahlt, siehe M16) erhöht
seinen automatischen Preis je Tag um

    hoch · clamp(Richtpreis im Land / Preis, 1, aufholen_max)

Liegt der Preis über dem Richtpreis, bleibt es bei `hoch` (0,25 % je Tag). Darunter
steigt er schneller, höchstens um `hoch` · `aufholen_max` (5 % je Tag bei
`aufholen_max` 20). Anlass: Nach der Benzinschwemme um 1910 lag Benzin 1929 noch bei
einem Dreitausendstel des Richtpreises, obwohl es knapp war – mit 8 % im Monat braucht
ein Preis für den Faktor 3 000 rund neun Jahre, und die Anbieter legten ihre Anlagen
inzwischen still (M22). Parameter: `marktmodell.preisanpassung.aufholen_max`.

### Nebenprodukte, Benzin und neue Verfahren (Balance-Runde M22)

- **Untergrenze für Nebenprodukte (KI):** Nebenprodukte haben keine eigenen Kosten. Die
  KI verkauft sie nicht unter ihrem Wert als Brennstoff:

      Untergrenze = Heizwert · min über andere Brennstoffe b (Marktpreis_b im Land / Heizwert_b)

  Brennstoffe sind Produkte mit `heizwert_mwh`; ohne Heizwert ist die Untergrenze 0.
  Benzin (12,2 MWh/t) kostet so mindestens, was dieselbe Wärme aus Kohle kostet (60 USD
  je 8,1 MWh): rund 90 USD/t, ein Zehntel des Richtpreises. Vorher fiel es mit
  Untergrenze 0 auf 0,3 Cent je Tonne.
- **Übrige Verwendung von Benzin:** `staatsnachfrage` 0,15 t je Mio. USD BIP für
  Lösungsmittel, Gaskocher und Gasmaschinen, Stationär- und Bootsmotoren, Heer und
  Marine (wie bei Gummi). 1900 sind das weltweit rund 50 000 t (Maßstab 0,1), 1929 rund
  100 000 t; dazu kommt der Verbrauch der Autos.
- **KI erkennt Nebenprodukte als Erzeugung:** Ob eine Ware „auf Halde liegt“ (Lager über
  `lager_hoch_tage` der Erzeugung), zählt auch, was als Nebenprodukt entsteht. Benzin aus
  den Raffinerien galt sonst immer als Halde, und niemand baute Crackanlagen.
- **Vorprodukte auf Halde sind kein Engpass:** Bei der Suche nach einer Lücke folgt die
  KI einem knappen Vorprodukt nur, wenn es nicht irgendwo auf Halde liegt; dann fehlt
  Handel, keine Erzeugung. Rohöl lag in Lagern, galt aber wegen der Einfuhren als
  knapp – und lenkte jede Benzin-Gelegenheit auf Ölfelder ohne freie Konzession.
- **Neue Verfahren der eigenen Forschung:** Firmen, die in ein neues Produkt einsteigen,
  nutzen alle Verfahren, die sie kennen – auch selbst erforschte, die noch nicht
  gemeinfrei sind (`gemeingut_nach_jahren` 25). Vorher konnten nur Neugründungen mit
  gemeinfreien Verfahren einsteigen; das Burton-Cracken von 1913 kam so erst 1938.

Weltlauf 1900–1930 (100 KI-Firmen, Seed 1) gegen den Stand von M21:

| Prüfung bzw. Wert | M21 | M22 |
| --- | --- | --- |
| Produkte mit zehn oder mehr Jahren Überkapazität | 30 | 23 |
| Verstöße „Preis gegen Richtpreis“ | 146 | 137 |
| Verstöße „Versorgung je Land“ | 334 | 328 |
| Benzin 1929: Preis / Richtpreis | 0,00 | 1,18 |
| Autoreifen 1929: Marge über Vollkosten | 49 % | 14 % |
| Pleiten bis 1929 | 13 | 4 |
| KI-Firmen mit Gewinn 1929 | 81 | 86 |
| Verstöße „Förderung gegen Bedarf der Anlagen“ | 26 | 90 |

Benzin fällt bis 1904 auf 0,16–0,2 × Richtpreis (Untergrenze Brennwert) und bleibt dort
bis 1922; 1923–1926 wird es knapp (bis 2,1 ×) und ist ab 1927 voll versorgt, nachdem ab 1925 Crackanlagen laufen –
wie die Benzinknappheit der frühen 1920er-Jahre, die das Cracken durchsetzte. Mehr
Verstöße bei der Förderung (schlechtester Wert 74 %, Kupfererz): Mit weniger
Überkapazität laufen die Anlagen näher an ihrer Grenze, und ihr Einkauf schwankt stärker
als die Förderung.

## M23 – Etappenziele

Lastenheft §18.3, Vorschlag 2. Daten: `data/etappen.yaml`. Etappen geben nach der
Einführung eine Richtung; sie wirken nicht auf die Simulation und gelten nur für den
Spieler.

- **Bewertung:** am Ende jedes Spieltags (nach dem Monatsabschluss), in der Reihenfolge der
  Daten. Eine erreichte Etappe bleibt erreicht und behält ihr Datum, auch wenn die
  Bedingung später nicht mehr gilt; im Rundenbericht steht sie als Erfolg.
- **Bedingungen** (`art`, Wert `wert`):

  | Art | Erreicht, wenn | Fortschritt |
  | --- | --- | --- |
  | `erster_verkauf` | Umsatz der Firma (laufendes oder ein abgeschlossenes Jahr) > 0 | – |
  | `gewinnmonat` | ein abgeschlossener Monat mit Ergebnis > 0 | – |
  | `anlagen` | fertige Einheiten aller Anlagen (auch stillgelegte) ≥ `wert` | Einheiten |
  | `eigenes_vorprodukt` | eine laufende eigene Anlage stellt her, was eine andere laufende eigene Anlage als Vorprodukt verbraucht | – |
  | `laender` | Standorte in ≥ `wert` Ländern | Länder |
  | `forschung` | eine Technologie selbst erforscht | – |
  | `marktfuehrer` | in einem Land im Vormonat mehr von einem Produkt verkauft als jede andere Firma, mit einem Anteil ≥ `wert` am Absatz dort | bester eigener Anteil |
  | `eigenkapital` | Eigenkapital ≥ `wert` · Startkapital | Eigenkapital |

- Ältere Spielstände ohne Etappen holen die erfüllten am ersten Tag nach dem Laden nach.

## M24 – Wettbewerb und Preise im Verlauf

Lastenheft §13.2, §14.2; Vorschlag 3. Parameter: `marktmodell.verlauf_monate`,
`marktmodell.meldung_preissenkung`. Beides wirkt nicht auf die Simulation.

- **Monatsreihe je Markt:** Am Monatsende bekommt jeder Markt, der in den letzten
  `verlauf_monate` Monaten etwas verkauft hat, einen Eintrag: bezahlter Durchschnittspreis
  (Umsatz / Menge; 0 ohne Verkauf), verkaufte Menge und die Menge des Spielers (diese erst,
  sobald er dort verkauft; die Monate davor zählen 0). Ältere Einträge fallen weg; ein
  Markt ohne Verkauf in allen gehaltenen Monaten verliert seine Reihe. Der Anteil des
  Spielers ist seine Menge / verkaufte Menge.
- **Wettbewerbsmeldungen:** Am Monatsende für jeden Markt (Produkt und Land), in dem der
  Spieler ein Angebot hat. Je Wettbewerber zählt sein niedrigster Angebotspreis dort.
  - Neuer Anbieter: Er hat jetzt ein Angebot, am letzten Monatsende nicht.
  - Anbieter weg: Er hatte ein Angebot und hat keines mehr (auch nach einer Pleite).
  - Preissenkung: Sein Preis liegt mindestens `meldung_preissenkung` unter seinem
    Bezugspreis. Der Bezugspreis ist der Preis bei der letzten Meldung, danach der höchste
    Preis seither; so werden auch schrittweise Senkungen gemeldet (automatische Preise
    sinken höchstens etwa 4 % im Monat), kleine Schwankungen aber nicht.
  - Der erste Monat in einem neuen Markt des Spielers merkt sich nur die Anbieter.

## M25 – Produktionsketten (Anzeige)

Lastenheft §14.1; Vorschlag 4. Nur Anzeige, keine Wirkung auf die Simulation.

- **Spitzen der Ketten:** alle Produkte mit Rezept, aus denen kein Rezept etwas anderes
  herstellt (Endprodukte, aber auch Nägel für das Bauwesen oder Petroleum und Benzin für
  Lampen und Autos); Strom nicht. Von dort geht es über das gezeigte Rezept zu den
  Vorprodukten bis zu den Rohstoffen.
- **Gezeigtes Rezept je Produkt:** unter den bis heute erfundenen Rezepten zuerst die,
  die der Spieler nutzen darf (Technologien von Rezept und Anlage bekannt), darunter das
  billigste. Ohne nutzbares Rezept das billigste erfundene mit den fehlenden
  Technologien.
- **Stückkosten** im Land des Firmensitzes wie in der Plausibilitätsprüfung (M16,
  `health::unit_cost`): Vorprodukte und Nebenprodukte zu Marktpreisen dort, Löhne und
  Arbeitsproduktivität des Landes, Strompreis, Gemeinkosten, Abschreibung und Wartung der
  Anlage bei `auslastung_normal`, Pacht der Rohstoffe. **Marge** = (Marktpreis −
  Stückkosten) / Marktpreis.
- **Eigene Abdeckung:** „stellst du her“ (eine laufende eigene Anlage macht das Produkt),
  „kaufst du ein“ (Einkaufsauftrag), „verkaufst du“ (Angebot); „hakt“ mit der Ursache, die
  eigene Anlagen des Produkts am letzten Tag bremste (Vorprodukt, Arbeitskräfte, Strom,
  Lagerstätte).

## M26 – Weiterlaufen bis … (Sitzung)

Vorschlag 5. Die Sitzung reiht Runden der gewählten Länge aneinander; der Kern rechnet
jede wie eine einzelne Runde (gleiches Journal, gleiche Ergebnisse wie Runde für Runde).

- `runde`: eine Runde (wie bisher).
- `jahresende`: Monatsrunden, bis das Datum im nächsten Jahr liegt.
- `meldung`: Monatsrunden bis nach der ersten Runde mit einer Warnung, einer Krise oder
  einem Weltereignis, höchstens ein Jahr (bis zum selben Monat des Folgejahrs).
- Immer: Halt beim Spielende.
- Der Bericht umfasst alle Runden: Zeitraum, Ergebnis und „Was lief“ gegen den Stand
  vor der ersten Runde, alle Meldungen, die Zahl der Runden und den Grund des Halts. Der
  automatische Spielstand wird einmal am Ende geschrieben.

## M27 – Ursachen erklären (Anzeige)

Lastenheft §14.3; Vorschlag 6. Die Erklärungen zeigen die Teile, aus denen der Kern einen
Wert berechnet; ein Test prüft je Wert, dass die Teile das Ganze ergeben.

- **Marktpreis** (Produktmarkt): Richtpreis × Preisniveau^Anteil (`preisniveau_anteil` der
  Produktart) = Richtpreis im Land; × Marktlage (Marktpreis / Richtpreis im Land) =
  Marktpreis. Dazu die Lage im Vormonat (Versorgung der Verbraucher und des Staats,
  offene Nachfrage, Anteil der Einfuhren, Zahl der Anbieter) und die Regel der
  automatischen Preise (M7, M16, M22).
- **Nachfrage der Verbraucher** (Produktmarkt): je Einkommensfünftel Einkommen je Kopf,
  Kaufneigung beim heutigen Marktpreis gegen den Vergleichspreis (Richtpreis ×
  Preisniveau, wie in M7), dazu
  - Verbrauchsgut: gesättigter Bedarf je Kopf; Nachfrage = Bedarf × Kaufneigung ×
    Netzanteil × Saison.
  - Gebrauchsgut: Zielbesitz (höchster Besitz × Kaufneigung × Rest nach Verdrängung ×
    Netzanteil) und Besitz je Kopf.
  - Ergänzungsgut: besessene Stücke des Gebrauchsguts je Kopf.
  Die Nachfrage je Kopf und Jahr ist die des laufenden Monats (zu Monatsbeginn gesetzt);
  zu Monatsbeginn ergeben die Teile sie genau.
- **Stückkosten** (Werk, Verkauf): Material, Personal, Energie, Gemeinkosten, Pacht und
  Anlage (Abschreibung und Wartung) wie unter „Kosten“ (M18), dazu die Kosten ohne
  Anlage für ein weiteres Stück.


## M28 – Geschichte erzählen (Meldungen)

Lastenheft §4.1, §3.6; Vorschlag 7. Nichts davon wirkt auf die Simulation zurück.

- **Weltereignisse 1940–2026** wie die von 1900–1939 (M11): am Tag des Ereignisses eine
  Meldung mit Titel, Beschreibung, Art und betroffenen Ländern. Neue Art `reform`
  (Wirtschaftspolitik). Ihre Folgen stecken in den Länderwerten; eigene Wirkungen folgen
  mit Stufe 4.
- **Währungsumstellung:** Am ersten Tag eines Monats prüft der Kern für das Land des
  Firmensitzes und jedes Land mit eigenem Standort, ob dort ein neuer Zeitraum der
  Landeswährungen (M21) beginnt (Beginn = Jahr + (Monat − 1)/12). Wechselt dabei die
  Währung, meldet er „ab Monat Jahr rechnet Land in neu statt in alt: 1 neu = Faktor alt“.
  - Faktor = gesetzlicher Umstellungskurs (`umrechnung` am Zeitraum), sonst
    Kurs_alt(Beginn) / Kurs_neu(Beginn) mit den Kursen je US-Dollar (M21), auf drei
    gültige Stellen gerundet (die Kurse sind Schätzungen).
  - Beispiele: Dezember 1923, 1 RM = 1 Billion M (gesetzlich); Januar 1999,
    1 € = 1,95583 DM (amtlicher Kurs).
  - Das Vermögen im Spiel ändert sich nicht (es zählt in US-Dollar mit der Kaufkraft
    von 2026); bei „Preise der Zeit“ erscheinen Beträge ab dann in der neuen Währung.

## M29 – Rang in der Übersicht

Vorschlag 8. Nichts davon wirkt auf die Simulation zurück.

- Gezählt werden alle Firmen, die nicht insolvent sind, der Spieler eingeschlossen.
- Eigenkapital = Summe der Aktiva − Kredite (wie in der Übersicht).
- Umsatz = Umsatzerlöse der letzten zwölf abgeschlossenen Monate.
- Platz = 1 + Zahl der Firmen mit einem höheren Wert; gleiche Werte teilen sich einen
  Platz. Ohne Umsatz in den zwölf Monaten gibt es keinen Platz nach Umsatz (zu
  Spielbeginn stünden sonst alle auf Platz 1).
- Die Plätze des Spielers werden zu Spielbeginn und am ersten Tag nach jedem Monatsende
  festgehalten, die letzten 24. Der Vergleich gilt dem Stand zu Beginn desselben Monats
  ein Jahr zuvor.
- Zum Jahresende meldet der Rundenbericht den Platz nach Eigenkapital und, mit Umsatz,
  nach Umsatz, jeweils mit dem Platz ein Jahr zuvor (nur mit Wettbewerbern).

## M30 – Kaufangebote I: Standorte, Labore und Lizenzen

Lastenheft §18.4. Jede Firma kann jeder anderen ein Angebot machen; Spieler und KI
nutzen dieselben Befehle und Prüfungen. Parameter: `data/parameter/kaufmodell.yaml`.

### Ablauf

- **Angebot** (Befehl `MakeOffer`): Käufer K bietet Verkäufer V einen Preis P für einen
  Gegenstand: einen Standort von V (jede Art, auch Forschungszentren) oder eine
  **Lizenz** auf eine Technologie, die V kennt und K nicht. Geprüft wird: V ≠ K, V nicht
  insolvent, der Gegenstand gehört V, der Standort besteht seit `mindestalter_monate`,
  P > 0, P ≤ Kasse von K, kein offenes Angebot von K für denselben Gegenstand, keine
  Sperre.
- **Antwort** (Befehl `AnswerOffer`): Wer am Zug ist – V auf ein Angebot, K auf ein
  Gegenangebot –, nimmt an oder lehnt ab. V kann statt dessen einmal einen höheren Preis
  nennen (**Gegenangebot**); dann ist K am Zug. Die Annahme vollzieht den Kauf sofort,
  wenn der Gegenstand noch V gehört und K den Preis zahlen kann; sonst wird sie
  abgewiesen.
- **Rücknahme** (Befehl `WithdrawOffer`): Wer den geltenden Preis genannt hat, kann ihn
  zurückziehen.
- **Frist:** `gueltig_monate` ab dem letzten Preis, danach verfällt das Angebot. Nach
  Ablehnung oder Verfall bietet derselbe Käufer frühestens nach `sperre_monate` wieder
  für denselben Gegenstand. Abgeschlossene Angebote bleiben so lange sichtbar.
- KI-Firmen antworten am Tag nach Eingang.

### Grundwert eines Standorts (für alle gleich)

- Buchwert B = Restbuchwert der fertigen Anlagen (M22) + Kosten der Anlagen im Bau +
  Restbuchwert von Gebäude, Erschließung und Firmenwert.
- Lager L = Wert der Vorräte am Standort und der Lieferungen, die dorthin unterwegs sind.
- Ergebnis R = Ergebnis des Standorts (alle seine Kostenstellen) in den letzten zwölf
  abgeschlossenen Monaten des Eigentümers; bei n < 12 Monaten R · 12/n, ab
  `ertrag_mindestmonate`, sonst 0.
- Ertragswert E = max(0, R) · `ertragsfaktor` (Jahre).
- Restwert Q = Erlös beim Verkauf aller fertigen Anlagen (M22).
- U = Kosten der Anlagen im Bau.
- **Grundwert G = max(E, Q) + U + L.**

### Höchstpreis eines Käufers

Aufschläge auf G, als Anteile:

- **Wettbewerb** a_w: m = Summe über die Produkte, die s im Vormonat in seinem Land
  verkauft hat und die K dort ebenfalls anbietet, von (Absatz von s / Absatz im Markt).
  a_w = `wettbewerb_aufschlag`(Aggressivität von K) · min(1, m).
- **Belegschaft** a_f = `fachkraefte_aufschlag` · q · k. q = Anteil der Beschäftigten ab
  Qualifikationsstufe `qualifiziert_ab_stufe` (Fachkräfte, Akademiker, Forscher); k =
  Knappheit dieser Gruppen im Land, 1 − frei/Bestand, gewichtet mit den Beschäftigten.
- **Eigenes Geschäft** a_b = `bauzeit_aufschlag`, wenn K eines der Produkte von s selbst
  herstellt: Kauf spart Bauzeit und Anlauf. Ein Forschungszentrum zählt, wenn K keines
  hat, aber forschen will (Kompetenz und Umsatz wie die Forschung der KI, M10) – ein
  zweites nutzt die KI nicht. Strom lässt sich nicht handeln: Ein Kraftwerk zählt nur,
  wenn K im Land des Kraftwerks Strom fehlt (Strombedarf S der eigenen Anlagen dort
  über der Leistung W seiner Kraftwerke dort, siehe unten).
- **Neubau erspart** a_n (nur mit a_b) = `neubau_anteil` · max(0, N − G) / G mit N =
  heutige Investition der Anlagen (Katalog, je Einheit) + Kosten des Standorts.
- **Höchstpreis H = G · (1 + a_w + a_f + a_b + a_n).**

### Lizenz

- Kosten je Forschungspunkt für K: (Tageslohn der Akademiker des Fachgebiets im Land des
  Firmensitzes + Material je Forscher und Tag · Preisniveau) / Forschungseffizienz des
  Landes im Fachgebiet (M9).
- Fehlende Punkte = Aufwand der Technologie (M9) − von K gesammelte Punkte.
- **Lizenzwert F = fehlende Punkte · Kosten je Punkt.** Höchstpreis von K:
  `lizenz_hoechst` · F.
- Abschluss: K erhält die Technologie (seine Punkte dafür entfallen), V behält sie.
  Buchung bei K als Aufwand, bei V als Ertrag der Kostenart „Lizenzen“.

### KI als Käufer

Am Monatsersten prüft jede KI-Firma mit der Wahrscheinlichkeit
`angebot_chance`(Aggressivität) ein Geschäft, höchstens eines je Monat und höchstens
`offene_angebote_max` offene Angebote:

- **Standorte:** Standorte anderer aktiver Firmen in Ländern, in denen K einen Standort
  oder den Sitz hat, ohne offenes Angebot und ohne Sperre. Vorteil v = H/G − 1; nur ab
  `mindestvorteil`. Gebot P = G · (1 + `gebotsaufschlag`(Aggressivität)), höchstens H.
  Forschungszentren und Kraftwerke nur, wenn sie zum eigenen Geschäft von K zählen
  (a_b). Braucht der Eigentümer den Standort selbst (siehe „KI als Verkäufer“), bietet
  K mindestens N und nur, wenn H ≥ N.
- **Lizenzen:** Technologien, an denen K forscht (gesammelte Punkte oder ein Labor
  daran), von einer Firma, die sie kennt. Gebot P = F · `lizenz_gebot`(Aggressivität).
- Gewählt wird das Geschäft mit dem größten Spielraum je Dollar, (H − P) / P: Kleine,
  aber lohnende Gegenstände zählen so viel wie große. Nur wenn P ≥ `mindestpreis_usd`
  und P ≤ `kasse_anteil_max` · Kasse.
- An den Spieler gehen höchstens `spieler_angebote_je_monat` neue Angebote je Monat.

### KI als Verkäufer und auf ein Gegenangebot

- Mindestpreis Standort M = G · (1 + `verkaufsaufschlag`(Aggressivität)); für einen
  Kernstandort (mindestens `kern_anteil` des Umsatzes der letzten zwölf Monate oder der
  einzige Standort mit Anlagen) M · (1 + `kern_aufschlag`).
- **Selbst gebraucht:** Ohne den Standort müsste V neu bauen, wenn er
  - ein Kraftwerk ist und S > W ohne dieses Kraftwerk, oder
  - das einzige Forschungszentrum von V ist und V dort forscht oder als KI-Firma
    forschen will (Kompetenz und Umsatz wie M10; sonst verkaufte sie es in der Pause
    zwischen zwei Forschungszielen und baute zum Quartalsende ein neues).

  Dann gilt M = max(M, N) mit N wie oben (Neubau). S = Summe über die nicht
  stillgelegten Anlagen von V im Land: Strom je Lauf · Läufe je Tag · Anzahl ·
  Auslastung; W = Summe über die nicht stillgelegten Kraftwerke dort: Läufe je Tag ·
  Anzahl · Menge je Lauf.
- Mindestpreis Lizenz M = F · `lizenz_mindest` (F für den Käufer); bietet K Produkte, die
  V herstellt, im selben Land an (Wettbewerber): M · (1 + `wettbewerb_lizenz`).
- P ≥ M: annehmen. P ≥ M · `gegen_schwelle`: Gegenangebot zu M. Sonst ablehnen.
- Auf ein Gegenangebot nimmt die KI an, wenn der Preis höchstens H ist und
  `kasse_anteil_max` · Kasse nicht übersteigt; sonst lehnt sie ab.

### Übergabe eines Standorts (Buchungen)

- **Verkäufer:** Sachanlagen −= Restbuchwert der fertigen Anlagen, des Gebäudes und der
  Erschließung; Anlagen im Bau −= U; Firmenwert −= sein Restbuchwert; Vorräte −= L;
  Kasse += P. Unterschied P − (B + L): Ertrag oder Aufwand „Sonstiges“ (wie M22).
- **Käufer:** Kasse −= P; Vorräte += L; Sachanlagen und Anlagen im Bau wie beim
  Verkäufer abgegangen (die Anlagen behalten Alter und Abschreibung). Firmenwert =
  P − (B ohne alten Firmenwert + L): positiv als Firmenwert des Standorts, linear über
  `firmenwert_jahre` abgeschrieben; negativ sofort Ertrag „Sonstiges“.
- Mit dem Standort wechseln Anlagen samt laufender Aufträge, Gebäude, Konzession, Lager,
  Belegschaft, Lohnaufschlag, Verkaufsangebote, Einkäufe und das Forschungsprojekt.
  Forschungspunkte, Marke und Kredite bleiben bei der Firma.

## M31 – Kaufangebote II: ganze Bereiche

Lastenheft §18.4: ganze Bereiche, d. h. alle Standorte einer Warengruppe mit der
Markenbekanntheit. Es gelten Ablauf, Fristen, Sperren, Gegenangebot und Buchungen aus
M30; hier steht nur, was für Bereiche dazukommt.

### Was zu einem Bereich gehört

- **Bereich (V, g)** für eine Warengruppe g: alle Standorte von V – Förderstandorte,
  Werke, Lager und Verkaufsbüros –, an denen eine Anlage ein Produkt von g herstellt
  oder die ein Produkt von g anbieten. Ein Standort geht immer ganz mit, auch mit
  Anlagen für andere Warengruppen. Kraftwerke und Forschungszentren gehören zu keinem
  Bereich; Warengruppen aus Energie (Strom) bilden keinen Bereich.
- Dazu die **Marke**: die Bekanntheit B_c von V für g in jedem Land c.
- Angebot möglich, wenn der Bereich mindestens einen Standort hat und der älteste davon
  seit `mindestalter_monate` besteht. Sperren und offene Angebote gelten je Käufer,
  Verkäufer und Gegenstand (für Standorte und Lizenzen ebenso).

### Grundwert

- **Markenwert** W = Σ_c K_c · (−ln(1 − min(B_c, 0,99))) / w: die Werbung, die dieselbe
  Bekanntheit aufbauen würde (M16: Ausgaben ΣA schließen 1 − exp(−w · ΣA / K) der
  Lücke; K_c wie M16, w = Wirkung des Werbemittels des Jahres; ohne Werbemittel W = 0).
- **Grundwert G_B = Σ G(s) + W** über die Standorte s des Bereichs (G wie M30).
- Neubau N_B = Σ N(s) + W (Anlagen und Standorte wie M30, die Marke über Werbung).

### Höchstpreis und KI als Käufer

- Aufschläge wie M30, über den ganzen Bereich: a_w mit der Summe der Marktanteile aller
  Standorte (höchstens 1), a_f über die Beschäftigten aller Standorte, a_b, wenn K ein
  Produkt eines der Standorte herstellt, a_n = `neubau_anteil` · max(0, N_B − G_B) / G_B.
  H_B = G_B · (1 + a_w + a_f + a_b + a_n).
- Die KI prüft Bereiche zusammen mit Standorten und Lizenzen (größter Spielraum je
  Dollar). Nur Bereiche mit mindestens zwei Standorten oder einer Marke (sonst wäre es
  das Angebot für den einzelnen Standort), mit mindestens einem Standort in einem Land,
  in dem K einen Standort oder den Sitz hat. Gebot P = G_B · (1 + `gebotsaufschlag`),
  höchstens H_B.

### KI als Verkäufer

- Mindestpreis M_B = G_B · (1 + `verkaufsaufschlag`); · (1 + `kern_aufschlag`), wenn
  der Bereich mindestens `kern_anteil` des Umsatzes im Vormonat bringt oder alle Werke
  und Förderstandorte mit Anlagen von V umfasst (wie der einzige Standort mit Anlagen
  in M30).

### Übergabe

- Der Preis wird auf die Standorte nach ihrem Grundwert aufgeteilt (P_s = P · G(s) /
  Σ G(s); bei Σ G(s) = 0 zu gleichen Teilen, Rundungsrest beim letzten). Jeder
  Standort wird wie in M30 übergeben; der Anteil der Marke geht so in den Preis der
  Standorte ein (als Firmenwert, soweit der Preis die Buchwerte übersteigt).
- Marke: K erhält je Land B_K = max(B_K, B_V); V verliert seine Bekanntheit und seine
  Werbebudgets für g.
- Offene Angebote für die übergebenen Standorte und für Bereiche von V sind danach
  hinfällig (ihr Inhalt hat sich geändert).

## M32 – Produkte 1915–1939

Lastenheft §18.4. Neue Produkte, Rohstoffe, Anlagen, Rezepte und Technologien stehen
in `data/ketten/14_*.yaml` bis `19_*.yaml`, `data/lagerstaetten/bauxit.yaml` und
`chilesalpeter.yaml`
(Annäherungen in den Kommentaren der Daten). An Formeln kommt hinzu:

- **Nachfrage erst ab Verfügbarkeit:** Endkunden und Staat fragen ein Produkt erst
  nach, wenn es sich herstellen lässt – ein Rezept, das es (auch als Nebenprodukt)
  erzeugt, braucht nur Technologien, deren Erfindungsjahr erreicht ist oder die eine
  aktive Firma kennt (vorzeitig erforscht) – oder wenn der Staatsmarkt es anbietet.
  Waren ganz ohne Rezept gelten als verfügbar. Vorher sind Nachfrage und Staatsbedarf
  null, und der Besitz von Gebrauchsgütern beginnt bei null (M7: Aneignung).

Damit neue Produkte überhaupt hergestellt werden, gelten für die KI (alle Firmen gleich,
M10) außerdem:

- **Marktlücken erforschen:** siehe M10, Forschung (`forschung_luecke_firmen` Firmen je
  Technologie zugleich, nur ohne Aufgabe in den eigenen Branchen).
- **Pioniere:** Am Quartalsende, vor der Diversifizierung, geht jede KI-Firma (die
  reichsten zuerst) die Produkte mit offener Nachfrage durch, die noch kein Standort
  herstellt (auch nicht als Nebenprodukt); sie baut für das wertvollste, das sie
  selbst herstellen kann, oder für den Engpass darunter (Suche wie bei der Neugründung)
  einen Standort, wenn ihr Budget wenigstens eine Anlage trägt. Je Firma und Quartal
  höchstens einer, jedes Produkt nur einmal. Ohne diese Regel wartete eine neue
  Technik, bis eine der reichsten Firmen sie kennt, und kleine neue Märkte (Traktor,
  Bauxit) kamen gegen die großen nie an die Reihe.
- **Anlagen im Bau zählen:** Für Neugründung, Diversifizierung und Pioniere sinkt die
  offene Nachfrage eines Produkts um die Erzeugung der Anlagen, die dafür im Bau sind
  (Kapazität × Startauslastung); ihr Wert sinkt im selben Verhältnis. Vorher entstand
  derselbe Engpass jedes Quartal neu, bis das erste Werk lief.
- **Engpass mit Ausweichen:** Sind mehrere Vorprodukte knapp, folgt die Suche zuerst
  dem knappsten; führt dessen Kette zu nichts (keine freie Konzession, keine bekannte
  Herstellung), versucht sie das nächste (vorher wurde die ganze Kette verworfen:
  knappe Kohle verdeckte die fehlende Bauxitgrube).
- **Standortwahl:** Ein Werk entsteht im Land mit der größten offenen Nachfrage *L*,
  außer der Transport wäre anderswo billiger: Kandidaten sind die Länder, in denen ein
  Vorprodukt hergestellt wird oder eine entdeckte Lagerstätte dafür liegt. Kosten je
  Lauf in Land *c* = Σ Menge des Vorprodukts × Fracht vom nächsten Herkunftsland nach
  *c* + Ausbringung × Fracht von *c* nach *L* (Fracht je Einheit wie im Handel, M8;
  Vorprodukte mit Staatsmarkt zählen nicht). Gewählt wird das billigste Land, bei
  Gleichstand *L*. Eine Tonerdefabrik (2,5 t Bauxit je t) geht so zum Bauxit, eine
  Kochtopffabrik bleibt beim Markt. Gruben wählen die freie Lagerstätte mit der
  billigsten Fracht nach *L* (vorher die erste im Land, sonst irgendeine). Das gilt nur
  für Vorprodukte, die allein Werke kaufen: Güter für Endkunden und Staat entstehen in
  *L* – Händler bringen sie kaum in Länder mit niedrigem Preisniveau, und Mühlen fern
  der Nachfrage wurden immer wieder neu gebaut.
- **Lieferungen an eigene Standorte:** Für Auslastung und Stilllegung zählt bei einem
  Angebot neben dem Verkauf auch, was die anderen Standorte der Firma davon verbrauchen
  (Läufe × Auslastung × Menge je Lauf), geteilt durch die Zahl ihrer Standorte, die die
  Ware herstellen. Vorher sah ein Werk, das nur die eigenen Anlagen belieferte, keine
  Abnehmer und fuhr auf die Mindestauslastung herunter (Flugmotoren für das eigene
  Flugzeugwerk).
- **Markt-Ansicht:** Waren, die noch niemand herstellen kann, stehen nicht in der Liste
  eines Landesmarkts.

## M33 – Produkte 1940–1964

Lastenheft §18.4. Neue Produkte, Rohstoffe, Anlagen, Rezepte und Technologien stehen
in `data/ketten/20_*.yaml` bis `27_*.yaml` und `data/lagerstaetten/germanium.yaml`
(Annäherungen in den Kommentaren der Daten). An Formeln kommt hinzu:

- **Verdrängung beim Staatsbedarf:** Ersetzt ein Produkt mit Staatsnachfrage ein anderes
  (`ersetzt`), sinkt der Staatsbedarf des alten ab dem Zeitpunkt *v*, zu dem sich das
  neue erstmals herstellen ließ, gleichmäßig über *J* = `verdraengung_staat_jahre`
  (Marktmodell) auf null: Faktor = min(1, max(0, 1 − (heute − *v*) / *J*)), bei
  mehreren Nachfolgern das Produkt der Faktoren. *v* ist das früheste Jahr unter den
  Rezepten des Nachfolgers, in dem alle Technologien von Rezept und Anlage erfunden
  waren – historisch oder früher im Spiel (erste Erfindung durch eine Firma). Bei
  Gebrauchsgütern bleibt es bei der Verdrängung über den Besitz (M9).

Für die KI (alle Firmen gleich, M10):

- **Verfahrenswahl mit Knappheit:** Wählt eine Firma für einen Standort zwischen
  mehreren Rezepten desselben Produkts, zählt zu den Stückkosten zum Richtpreis (M10)
  je Vorprodukt Menge × max(0, Marktpreis − Richtpreis im Land) / Ausbringung. Ein
  Verfahren, dessen Vorprodukt knapp und teuer ist, verdrängt so kein laufendes
  (vorher stellten alle Strumpfwerke auf Polyamid um, während Polyamid das Dreifache seines
  Richtpreises kostete). Ohne Land (Planung) bleibt es beim Richtpreis.
- **Einstieg in teure Märkte:** Zahlen die Käufer im Vormonat weltweit zusammen
  wenigstens `einstieg_preisfaktor` × den Wert zum Richtpreis (Σ Umsatz ≥ Faktor ·
  Σ Menge · Richtpreis im Land) und stellen weniger als `einstieg_firmen_max` Firmen das
  Produkt her (die bauende Firma nicht darunter), zählt für Diversifizierung und
  Neugründung zur offenen Nachfrage eine Menge *q* = `einstieg_anteil` · Absatz des
  Vormonats je Tag hinzu (abzüglich der Anlagen im Bau), im Wert von *q* zum gezahlten
  Durchschnittspreis. Gebaut wird – wenn es keine offene Nachfrage gibt – im Land mit dem
  größten Aufpreis (Umsatz − Menge · Richtpreis). Die Technologien solcher Produkte
  erforschen Firmen wie Marktlücken (M10, Forschung). Vorher blieb Penicillin zwanzig
  Jahre bei einem Hersteller, der das Vierfache des Richtpreises nahm: Wo alle Käufer
  bedient waren, sah niemand eine Lücke.
- **Was sich stapelt (M22) – nur bei Verkäufern:** Für die Frage, ob eine Ware irgendwo
  auf Händler wartet, zählen nur die Lager der Standorte, die sie herstellen oder
  anbieten. Die Eingangslager der Verarbeiter gehören nicht dazu; das Rohöl in den
  Raffinerien verdeckte sonst den Ölmangel der 1960er.
- **Gruben passend zur Konzession:** Ein neuer Förderstandort (Neugründung,
  Diversifizierung, weitere Konzession einer Förderfirma) bekommt höchstens
  ⌈Höchstförderung · *s* · Anteil / 365 / (Ausbringung je Anlage und Tag ·
  Startauslastung)⌉ Anlagen, mindestens eine. Vorher standen zwanzig Bohrtürme auf einem
  Ölfeld, das zwei auslastete.
- **Lagerstätte für eine neue Konzession:** entdeckt, mit freier Konzession, deren
  Restvorrat (Vorrat · Förderindex · *s* − Gefördertes, M41) wenigstens `vorrat_jahre_min` Jahre ihrer
  Höchstförderung trägt; unter diesen die mit der billigsten Fracht in das
  Land mit der größten offenen Nachfrage (für Förderfirmen vorher die erste in den Daten).
- **Ausbau nach zwei Monaten:** Ein Standort gilt beim Ausbau (M10) als ausverkauft,
  wenn Verkauf seit Beginn des Vormonats plus eigener Verbrauch wenigstens neun
  Zehntel der Erzeugung dieser Tage erreichen (vorher nur der laufende Monat: Waren,
  die in wenigen Stücken verkauft werden wie Verkehrsflugzeuge, erschienen in manchen
  Monaten unverkauft).

Bei den Daten kommt hinzu: Kautschuk lässt sich auch synthetisch aus Ethylen und Benzin
herstellen (1937, `27_synthesekautschuk.yaml`), Polyamid auch aus Kohle statt aus Benzin.
Für die Ebenen des Produktbaums (Lastenheft §17.2) zählt bei mehreren Rezepten eines
Vorprodukts der einfachste Weg; ein Rohstoff mit Abbau-Rezept ist die erste Ebene.

## M34 – Regionen

Offene Punkte, Abschnitt I. Aus 197 Ländern werden 111 Länder und Regionen
(`tools/daten/regionen.py`): Kleinstaaten gehen im Nachbarland auf (Luxemburg in Belgien,
Monaco in Frankreich), dünn besiedelte Nachbarn bilden Regionen (Baltikum, Mittelafrika,
Ozeanien). Eine Region behält den ISO-Code ihres namensgebenden Landes oder bekommt einen
frei verfügbaren Code (`X..`). Ihre Werte entstehen aus denen ihrer Länder *i*:

- **Bevölkerung, Fläche:** Summen Σ *Pᵢ*, Σ *Aᵢ*.
- **BIP je Kopf:** Σ *Pᵢ* · *yᵢ* / Σ *Pᵢ* je Jahr (Gesamt-BIP durch Gesamtbevölkerung).
- **Gini:** bevölkerungsgewichtetes Mittel Σ *Pᵢ* · *Gᵢ* / Σ *Pᵢ* (Annäherung: Die
  Ungleichheit zwischen den Ländern fehlt).
- **Hauptstadt, Kontinent:** die des namensgebenden Landes, sonst die des Landes mit den
  meisten Einwohnern im Jahr 2000. Dieses Land steht in `umfasst` vorn.
- **Binnenland** nur, wenn alle Länder Binnenländer sind; **Nachbarn** sind die
  Nachbarn aller Länder (als Regionen) ohne die Region selbst.
- **Steuern, Stabilität, Prägung:** Schätzungen gab es nur für einzelne große Länder;
  Regionen nehmen die Standardwerte des Ländermodells.
- **Währung:** eine Region mit eigenem Code behält ihre Währungen, eine `X..`-Region nimmt
  die des Landes, das in `umfasst` vorn steht. Währungen, die danach kein Land mehr
  verwendet, entfallen.
- **Verweise:** Lagerstätten, reale Firmen, Ereignisse und Namensgruppen nennen die
  Region statt des Landes.

**Alte Spielstände:** Die Codes der aufgegangenen Länder lesen sich als ihre Region. Steht
für eine Region in einem Spielstand mehr als ein Eintrag (etwa Märkte von Belgien und
Luxemburg), gilt der unter dem eigenen Code, sonst der des Landes, das in `umfasst` am
weitesten vorn steht. Die übrigen entfallen. Abgeleitete Länderwerte werden nach dem Laden
ohnehin neu berechnet.

## Mehr KI-Firmen (06.10.2026)

Wunsch: „Mehr KI-Gegner. Mache einen Simulationslauf mit 500 Gegnern.“ Mit *N* KI-Firmen
wächst der Marktmaßstab (M10: *s* = *N* / `firmen_bei_realer_groesse`) und mit ihm jeder
Markt. Firmenzahlen in den Regeln der KI galten bisher absolut; bei 500 Firmen blieben
neue Märkte (Radio, Kühlschrank) jahrelang bei einem Hersteller, der das Doppelte des
Richtpreises nahm, während etablierte Märkte fünfmal so viele Anbieter hatten.
Seitdem gilt für `forschung_luecke_firmen`, `gruendungen_je_monat`,
`diversifikationen_je_quartal` und `einstieg_firmen_max`:

  wirksame Zahl = runden(Zahl · max(1, *N* / `firmen_standard`))

Unter der Standardzahl bleibt es bei den Werten der Daten; 0 schaltet eine Regel weiter ab.

**Tempo:** Die tägliche Markträumung sucht die Anbieter und Käufer eines Produkts in einer
Liste je Produkt und Land statt unter allen Standorten des Landes, und die Händler
bestimmen den Weg je Herkunftsland einmal statt je Anbieter. Die Ergebnisse bleiben
gleich (gleicher Zustands-Hash).

## M35 – Grundstücke

Offene Punkte, Abschnitt H. Parameter: `data/parameter/grundstuecksmodell.yaml`; Fläche
einzelner Anlagen: `anlagen[].flaeche_ha`.

### Fläche eines Standorts

Jeder Standort außer Förderstätten (sie stehen auf ihrer Konzession) liegt auf genau einem
Grundstück. Eine Anlage braucht je Einheit

    a = flaeche_ha der Anlage, sonst Investition / investition_je_ha_usd

Ein Standort braucht max(`mindestflaeche_ha`, (1 + `zuschlag`) · Σ a · Einheiten) – auch
für stillgelegte Einheiten und Anlagen im Bau. Ein Bauauftrag, nach dem der Bedarf die
Fläche des Grundstücks überstiege, wird abgelehnt („Grundstück zu klein“); dann hilft
nur ein weiterer Standort auf einem neuen Grundstück im selben Land.

### Angebot je Land

Die Gewerbefläche, die ein Land anbietet, folgt seiner Wirtschaft:

    Ziel(t) = flaeche_ha_je_mrd_bip · BIP(t) in Mrd. USD (Kaufkraft 2026) · Marktmaßstab

Zu Spielbeginn und an jedem 1. Januar kommen Grundstücke hinzu, bis die Fläche aller
Grundstücke des Landes (belegt und frei) das Ziel erreicht. Jedes neue Grundstück wird
mit einem eigenen Zufallsstrom je Land und Jahr gezogen:

1. **Lage** nach den Anteilen von `lagen` (Hafen nur in Ländern mit Küste; sein Anteil
   geht dann an die anderen Lagen im Verhältnis ihrer Anteile).
2. **Größenklasse** nach den Anteilen der Klasse für den Wohlstand des Landes: reich ab
   `reich_ab_usd` BIP je Kopf, arm unter `arm_unter_usd`, sonst mittel.
3. **Fläche** gleichverteilt zwischen den Grenzen der Klasse, mal
   `1 + (Jahr − wachstum_ab_jahr) / wachstum_jahre` (Industriegebiete werden größer)
   und mal dem Flächenfaktor der Lage, auf 0,01 ha gerundet.

Grundstücke werden nie kleiner oder knapper als nötig, um zu starten: Für die
Startstandorte (KI und Spieler) entsteht, wo kein freies Grundstück passt, eines in der
nötigen Größe (Lage Stadt, Klasse nach der Fläche). Aufgegebene Standorte (Pleite)
geben ihr Grundstück frei.

### Preis, Kauf und Pacht

    Bodenpreis je ha = bodenpreis_usd_je_ha · Preisniveau des Landes
                       · (1 + knappheit · belegter Anteil der Gewerbefläche des Landes)
                       · Preisfaktor der Lage
    Wert eines Grundstücks = Bodenpreis je ha · Fläche

- **Kauf:** Der Wert wird bei der Gründung bezahlt und steht als „Grundstücke“ im
  Anlagevermögen, ohne Abschreibung. Wird der Standort verkauft (M30), geht das
  Grundstück mit; sein Grundwert enthält den heutigen Wert des Grundstücks.
- **Pacht:** keine Zahlung bei der Gründung; jeden Monatsersten `pacht_anteil` / 12 des
  heutigen Werts als Pacht (Kostenart Pacht, Kostenstelle Standort). Ein gepachtetes
  Grundstück geht bei einem Verkauf des Standorts als Pacht mit.
- Die bisherigen Standortkosten (`standortkosten_usd`) bleiben als Gebäude und
  Erschließung.
- Wer gründet, wählt ein freies Grundstück und Kauf oder Pacht. Ohne Wahl (Befehl
  `FoundSite`) nimmt der Standort das größte freie Grundstück des Landes, gekauft, wenn
  die Kasse Gebäude und Boden trägt, sonst gepachtet.

### Lage

| Lage | Wirkung |
| --- | --- |
| Stadt | teurer Boden, kleinere Grundstücke; wirbt Arbeitskräfte leichter an |
| Hafen | Fracht über See für Lieferungen von und zu diesem Standort günstiger |
| Land | billiger Boden, große Grundstücke; wirbt schwerer an; Lieferkosten auf den Absatz im Land |

- **Anwerben:** Bei der Reihenfolge der Einstellung und beim Abwerben (M18) zählt ein
  Standort mit Lohnaufschlag + `anwerben` der Lage (Stadt mehr, Land weniger). Gezahlt
  wird der eigene Lohnaufschlag.
- **Seefracht:** Händler, die bei einem Standort in Hafenlage kaufen, und Lieferungen
  einer Firma an oder aus einem solchen Standort zahlen auf Wegen über See
  `fracht_see` · Fracht.
- **Lieferkosten:** Was ein Standort auf dem Markt seines Landes verkauft, kostet ihn
  `lieferkosten` · Umsatz (Kostenart Transport).

### KI

- **Grundstückswahl:** gebraucht wird die Fläche des Vorhabens mal (1 + `ki_reserve`);
  unter den freien Grundstücken des Landes, die das fassen, das mit den geringsten
  jährlichen Kosten

      pacht_anteil · Wert des Grundstücks + lieferkosten der Lage · erwarteter Umsatz

  (erwarteter Umsatz: geplante Erzeugung der Waren, die der Standort anbietet, zum
  Marktpreis im Land; Kraftwerke und Labore verkaufen nichts), bei Gleichstand das
  kleinere; sonst das größte freie. Ohne Grundstück für wenigstens eine Anlage fällt das
  Vorhaben in diesem Land aus (Neugründung und Diversifizierung suchen das nächste).
  Startstandorte und Standorte aus alten Spielständen wählen genauso.
- **Kauf oder Pacht:** gekauft, wenn danach noch `kasse_min_monate` laufende Kosten in
  der Kasse bleiben, sonst gepachtet.
- **Ausbau (M10):** Fasst das Grundstück nicht alle geplanten Einheiten, baut die KI so
  viele, wie passen. Passt keine mehr, baut sie auf einem anderen eigenen Standort
  derselben Art im Land mit freier Fläche weiter (zuerst einem, der das Produkt schon
  herstellt, sonst dem mit dem meisten Platz für die Anlage) und bietet das Produkt dort
  an; erst ohne einen solchen gründet sie einen Standort auf einem neuen Grundstück.
  (Erster Weltlauf: Mit `ki_reserve` 0,5 und ohne diese Regel wurde fast jeder Ausbau ein
  eigener Standort – 6 507 statt 2 143 Standorte 1965, eine Firma mit 185 Mühlen.)

### Spielstände

Standorte aus Spielständen vor M35 bekommen beim Laden ein gekauftes Grundstück in ihrem
Land, das ihre Anlagen mit `reserve` fasst (Lage Stadt). Es steht ohne Buchung im
Anlagevermögen: der Kaufpreis steckte bisher in den Standortkosten.

## M36 – Anlagen in fünf Größen

Offene Punkte, Abschnitt H, Punkt 9. Parameter: `data/parameter/produktionsmodell.yaml`,
Abschnitt `anlagengroessen`.

Jede Anlage gibt es in fünf Größen. Die Datenwerte einer Anlage (Investition, Durchläufe
je Tag, Fläche, Bauzeit) und ihrer Rezepte (Arbeitsstunden je Durchlauf) gelten für die
Größe **mittel**. Eine Größe mit dem Kapazitätsfaktor *k* ändert sie so:

| Wert | Formel | sehr klein | klein | mittel | groß | sehr groß |
| --- | --- | --- | --- | --- | --- | --- |
| Kapazität (Durchläufe je Tag) | · k | 0,25 | 0,5 | 1 | 2 | 4 |
| Investition | · k^0,7 | 0,38 | 0,62 | 1 | 1,62 | 2,64 |
| Investition je Kapazität | k^−0,3 | 1,52 | 1,23 | 1 | 0,81 | 0,66 |
| Arbeitsstunden je Durchlauf | · k^−0,15 | 1,23 | 1,11 | 1 | 0,90 | 0,81 |
| Fläche je Einheit (M35) | · k^0,7 | 0,38 | 0,62 | 1 | 1,62 | 2,64 |
| Bauzeit (gerundet, mind. 1 Tag) | · k^0,3 | 0,66 | 0,81 | 1 | 1,23 | 1,52 |

(Exponenten: `investition_exponent`, `arbeit_exponent`, `flaeche_exponent`,
`bauzeit_exponent`.) Vorprodukte, Strom und Nebenprodukte je Durchlauf bleiben gleich,
ebenso die Qualität. Was von der Investition abhängt, folgt ihr: Abschreibung, Wartung,
Kosten der Automatisierung, Wiederanlauf und Verkaufserlös (M22). Ein Labor im
Forschungszentrum beschäftigt `Durchläufe je Tag · k · Einheiten · Auslastung` Forscher.

- Eine Anlage am Standort hat eine Größe; ihre Einheiten sind gleich groß. Wer dieselbe
  Anlage in einer anderen Größe baut, bekommt eine weitere Anlage am Standort.
- Große Anlagen sparen Investition und Arbeit je Stück, brauchen aber Kapital, Fläche und
  Absatz; kleine passen in kleine Märkte und auf kleine Grundstücke und laufen dort voll
  statt halb.
- Spielstände und Befehle von vor M36 meinen die Größe mittel; die Startbesetzung (M10)
  und die Startformen bauen weiter mittelgroße Anlagen (die Daten sind darauf abgestimmt),
  außer für Vorprodukte, deren Bedarf keine mittlere Anlage füllt (Startbesetzung,
  Schritt 4).

### KI

Die KI plant wie bisher eine gewünschte Kapazität *U*, gemessen in Anlagen der Größe
mittel (bisher die Anlagenzahl, jetzt ungerundet):

- Neugründung, Diversifizierung und Engpass: U = offene Nachfrage / Erzeugung einer
  mittleren Anlage bei Normalauslastung, begrenzt auf die kleinste Größe bis 20 und bei
  Förderung auf die Konzession.
- Ausbau: U = max(25 % der Kapazität des Produkts am Standort, min(1, Kapazität)) – kleine
  Werke verdoppeln sich, große wachsen um ein Viertel, mindestens um eine mittlere Anlage
  (bis M36: 25 % der Anlagenzahl, mindestens 1).
- Eigenstrom: U = fehlender Strom / Leistung einer mittleren Anlage bei Normalauslastung.
- Neue Konzession: U = `anlagen_je_konzession`, begrenzt auf die Konzession.

Daraus Größe und Zahl: die größte Größe mit k ≤ U (sonst die kleinste) und
Zahl = max(1, round(U / k)). Beispiele: U = 0,3 → 1 × sehr klein; U = 1 → 1 × mittel;
U = 3 → 2 × groß; U = 5 → 1 × sehr groß; U = 12 → 3 × sehr groß. Reichen Grundstück
(M35) oder Budget nicht, baut sie so viele Einheiten, wie passen; passt nicht einmal eine,
versucht sie die nächstkleinere Größe mit ihrer Zahl.

## M37 – Weiterentwicklung erforschter Produkte

Offene Punkte, Abschnitt J. Parameter: `data/parameter/forschungsmodell.yaml`, Abschnitt
`weiterentwicklung`.

Jede Firma kann jedes Produkt, das sie herstellen darf, in ihren Forschungszentren
weiterentwickeln – Endprodukte ebenso wie Rohstoffe, Halbzeuge und Bauteile (besseres
Garn, reinerer Stahl, sparsamerer Motor). Das geschieht Stufe für Stufe bis `stufen`.

### Stufe einer Firma

    L(f, p, t) = max(L_eigen(f, p), L_gemein(p, t))

- L_eigen: die Stufen, die die Firma selbst erforscht hat.
- L_gemein: Gemeingut – die Zahl der Stufen, die irgendeine Firma vor mindestens
  `gemeingut_nach_jahren` Jahren als erste erreicht hat (Wissen verbreitet sich). Wer
  vorn liegt, hat so viele Jahre Vorsprung.

### Wirkung

In allen Rezepten der Firma für das Produkt p, mit L = L(f, p, t):

    Qualität              + L · je_stufe.qualitaet          (vor der Begrenzung auf 0–100)
    Arbeitsstunden        · (1 − L · je_stufe.arbeit)        je Durchlauf
    Vorprodukte           · (1 − L · je_stufe.vorprodukte)   je Durchlauf

Erzeugung, Nebenprodukte und Strom je Durchlauf bleiben gleich. Mit den Datenwerten bringt
Stufe 5 +20 Qualitätspunkte, −15 % Arbeit und −10 % Vorprodukte. Bessere Vorprodukte
heben die Qualität der daraus gebauten Waren (bestehende Regel: `produktionsmodell.qualitaet.vorprodukte`
je Punkt über 50), bessere Qualität bringt Marktanteile (Kaufentscheidung, M16).

### Voraussetzung und Fachgebiet

- Weiterentwickeln darf eine Firma ein Produkt, wenn sie ein Rezept dafür nutzen darf
  (dessen Technologie kennt oder es braucht keine) und L < `stufen`.
- Fachgebiet der Forscher: das Fachgebiet der Technologie des ersten Rezepts mit
  Technologie (Reihenfolge der Daten); für Produkte ohne Technologie (Ernte, einfacher
  Abbau) das Fachgebiet der Branche aus `fachgebiete`.

### Aufwand

Für die nächste Stufe n = L + 1:

    Punkte(n) = B(p) · aufwand.anteil · aufwand.wachstum^(n − 1) · N
    B(p)      = max(aufwand.grundaufwand, größter Forschungsaufwand der Technologien
                    der Rezepte von p)
    N         = max(nachzuegler.minimum, (1 − nachzuegler.rabatt_je_jahr)^(t − E_n))
                wenn eine Firma die Stufe n schon zum Zeitpunkt E_n erreicht hat, sonst 1

Beispiel B = 10 000 (Grundaufwand; Median der Technologien 12 000), anteil 1, wachstum 1,7:
Stufe 1 10 000, Stufe 2 17 000, Stufe 3 28 900, Stufe 4 49 130, Stufe 5 83 521 Punkte,
zusammen 188 551 – mit einem Labor (20 Forscher, Effizienz 1) 500, 850, 1 445, 2 457 und
4 176 Tage, rund 26 Jahre. (Im ersten Weltlauf mit anteil 0,2 und wachstum 1,6 erreichten
die großen Firmen alle Stufen ihrer Produkte in etwa fünf Jahren, bis 1930 war fast alles
Gemeingut; Preise unter dem halben Richtpreis kamen doppelt so oft vor.)

### Forschen

- Ein Forschungszentrum arbeitet entweder an einer Technologie (M9) oder an der
  Weiterentwicklung eines Produkts. Forscher, Punkte je Tag und Kosten wie in M9, mit
  dem Fachgebiet des Produkts. Die Punkte gehören der Firma, je Produkt.
- Erreichen die Punkte den Aufwand der nächsten Stufe, ist sie erreicht; die Punkte
  beginnen wieder bei 0 und das Zentrum arbeitet an der folgenden Stufe weiter, bis
  `stufen` erreicht ist.
- Lizenzen (M30) und Kaufangebote übertragen keine Stufen: Nachzügler forschen billiger,
  und nach `gemeingut_nach_jahren` Jahren hat jeder die Stufe.

### KI

Eine Firma, die forscht (M10: Kompetenz und Umsatz), aber keine Technologie für ihre
Branchen oder eine Marktlücke findet, entwickelt ihr umsatzstärkstes Produkt des
Vorjahres weiter, das noch nicht ausgereizt ist, dessen Umsatz U
`forschung_mindestumsatz_usd` erreicht und dessen nächste Stufe sich bezahlt macht:

    U · entwicklung_nutzen_je_stufe · entwicklung_amortisation_jahre ≥ Punkte(n) · K
    K = (Stundenlohn der Forscher · Stunden je Tag + sachkosten · Preisniveau) / Effizienz

(K: Kosten eines Forschungspunkts im Land ihres Forschungszentrums bzw. ihres Sitzes,
Parameter in `data/parameter/kimodell.yaml`.) Sie bleibt dabei, solange das gilt und keine
Technologie ansteht. Mit den Datenwerten (3 % des Umsatzes je Stufe und Jahr, 5 Jahre)
lohnt Stufe 1 eines Produkts mit B = 10 000 in Deutschland 1914 (etwa 1,4 Mio. USD) ab
rund 9,5 Mio. USD Jahresumsatz, Stufe 5 (etwa 12 Mio. USD) erst ab rund 80 Mio. USD.

### Spielstände

Neue Felder (Stufen und Punkte je Firma und Produkt, Projekt eines Zentrums, Tag der
ersten Erreichung je Stufe) sind optional; ältere Spielstände beginnen mit Stufe 0.

## M38 – Pleiten: Versteigerung der Standorte

Offene Punkte, Abschnitt K. Parameter: `data/parameter/kaufmodell.yaml`, Abschnitt
`insolvenz`.

Wird eine KI-Firma zahlungsunfähig (M6), stehen ihre Werke nicht mehr einfach still und
verschwinden: Der Insolvenzverwalter versteigert jeden Standort `insolvenz.tage` Tage lang.

- **Während der Versteigerung** ruht der Standort: keine Erzeugung, die Belegschaft ist
  entlassen, Angebote und Einkäufe enden. Anlagen, Lager, die erschlossene Konzession und
  das Grundstück bleiben.
- **Mindestgebot** = `insolvenz.mindestpreis` · Grundwert des Standorts (M30) am Tag des
  Gebots bzw. am Ende.
- **Gebote:** Der Spieler bietet wie bei Kaufangeboten (Wettbewerb → Firma → Standort),
  mindestens das Mindestgebot, höchstens seine Kasse. Eine KI-Firma bietet nur, wenn sie
  den Standort auch sonst kaufen würde (M30: im Land vertreten, Vorteil mindestens
  `mindestvorteil`, Labore und Kraftwerke nur im eigenen Geschäft), und zwar wie bei einem
  Kaufangebot Grundwert · (1 + `gebotsaufschlag`), höchstens ihren Höchstpreis und
  `kasse_anteil_max` ihrer Kasse, mindestens das Mindestgebot. (Im ersten Weltlauf bot
  jede KI-Firma ihren vollen Höchstpreis; Käufer zahlten so oft weit über Grundwert, und
  bis 1958 gingen 14 statt 9 bzw. 7 Firmen pleite.)
- **Zuschlag** am letzten Tag, Standort für Standort in ihrer Reihenfolge: das höchste
  Gebot; bezahlt wird das zweithöchste Gebot, mindestens das Mindestgebot (wer am meisten
  bietet, zahlt nicht mehr als nötig). Die Übergabe bucht wie ein Kauf (M30): Anlagen,
  Lager, Grundstück und Konzession gehen an den Käufer; ein Erlös über dem Buchwert ist
  ein Ertrag der insolventen Firma.
- **Ohne Gebot** wird der Standort aufgegeben wie bisher: Konzession und Grundstück
  werden frei.

Der Käufer übernimmt den Standort mit erschlossener Konzession und fertigen Anlagen und
stellt nur Personal ein; so fällt die Erzeugung höchstens für die Dauer der Versteigerung
aus statt für Erschließung und Bau (1955 kostete Erz nach dem Ausfall eines Förderers
das 3,7-Fache des Richtpreises).

**Während der Versteigerung** zählt die insolvente Firma bei Neugründungen von KI-Firmen
(M10) noch mit, und ihre Anlagen gelten für die Suche nach unversorgter Nachfrage wie
Anlagen im Bau als kommende Versorgung. Sonst füllt eine neue Firma die vorübergehende
Lücke, und nach dem Zuschlag stehen doppelt so viele Anlagen im Markt: In den ersten
Weltläufen gingen dadurch bis 1958 doppelt bis dreimal so viele Firmen pleite wie ohne
Versteigerung (Seed 2: 20 statt 7; nur 3 der 20 hatten selbst Standorte ersteigert).

## M39 – Produkte 1965–1989

Lastenheft §18.4. Neue Produkte, Rohstoffe, Anlagen, Rezepte und Technologien stehen in
`data/ketten/28_*.yaml` bis `34_*.yaml` und `data/lagerstaetten/quarz.yaml` (Annäherungen
und Quellen in den Kommentaren der Daten):

| Kette | Neue Produkte | Technologien (Jahr) |
| --- | --- | --- |
| 28 Halbleiter | Quarz, Reinstsilizium, Mikrochip; Transistoren auch aus Silizium | Reinstsilizium (1959), Planartechnik (1960), integrierte Schaltung (1961), Mikroprozessor (1971) |
| 29 Taschenrechner | Taschenrechner | elektronischer Taschenrechner (1971) |
| 30 Farbfernsehen | Farbbildröhre, Farbfernseher (verdrängt den Fernseher) | Lochmasken-Farbbildröhre (1954), Farbfernsehen (1965) |
| 31 Mikrowelle | Magnetron, Mikrowelle | Magnetron (1964), Mikrowellenherd (1967) |
| 32 Personal Computer | Personal Computer (mit Monitor) | Personal Computer (1977) |
| 33 Videorekorder | Videorekorder | Videorekorder (1976) |
| 34 CD-Spieler | CD-Spieler | Optische Speicherplatte (1982) |

Alle neuen Endprodukte sind Gebrauchsgüter mit den Nachfrageformeln aus M7 und M9; der
Mikrochip ist das gemeinsame Vorprodukt (Taschenrechner 2, CD-Spieler 10, Videorekorder
15, Personal Computer 150 Stück).

- **Staatsbedarf im Zeitverlauf:** Der staatliche Bedarf eines Produkts je Tag ist

      je_mio_usd_bip · BIP / 10⁶ / 365 · Verdrängungsfaktor (M33) · v(t)

  mit v(t) aus `staatsnachfrage.verlauf` (Jahreswerte, dazwischen linear, vor dem ersten
  und nach dem letzten Jahr deren Wert; ohne Angabe 1), t das Datum als Jahr mit Bruchteil.
  So wächst ein Bedarf schneller oder langsamer als das BIP: Stickstoffdünger 1913 mit 1,
  danach mit dem Verbrauch je BIP (Welt: 1913 etwa 0,26 kg Stickstoff je 1 000 USD BIP,
  1970 2,3, 1990 2,9, 2020 1,3).
- **Vorlauf für kommende Produkte (KI):** Für die Marktlücken der Forschung (M32) zählen
  auch Produkte mit Verbraucher- oder Staatsnachfrage, die es noch nicht gibt, die sich
  aber historisch in höchstens *L* = `forschung_vorlauf_jahre` Jahren herstellen lassen
  (ein Rezept, dessen Technologien samt Anlage bis dahin erfunden sind), samt den
  Vorprodukten ihrer Rezepte. Ihre Technologien mit allen Voraussetzungen sind Lücken;
  erforscht wird wie bisher nur, was erfunden ist oder im Vorgriff der Firma liegt (M10).
  Vorher begann die Forschung erst mit der Nachfrage: Für den Taschenrechner (1971)
  mussten Reinstsilizium, Planartechnik und integrierte Schaltung erst danach erforscht
  werden, und bis 1976 baute niemand Mikrochips.
- **Später erschlossene Lagerstätten:** Eisenerz, Kupfererz und Erdöl bekommen die großen
  Funde und Tagebaue nach 1950 (Pilbara, Carajás, Labrador, Escondida, Ghawar, Samotlor,
  Nordsee, Prudhoe Bay …) mit ihrem Entdeckungsjahr, dazu einen Förderindex für Eisen- und
  Kupfererz (Tagebaue, Pellets, Laugung). Bis M38 war 1964 jede Eisenerzgrube ausgelastet
  (Kapazität 1950 und 1964 gleich), und die Ölfelder waren zu 85 % belegt.

## M40 – Produkte 1990–2009

Lastenheft §18.4. Neue Produkte, Rohstoffe, Anlagen, Rezepte und Technologien stehen in
`data/ketten/35_*.yaml` bis `41_*.yaml` und `data/lagerstaetten/lithium.yaml`,
`kobalt.yaml` (Annäherungen und Quellen in den Kommentaren der Daten):

| Kette | Neue Produkte | Technologien (Jahr) |
| --- | --- | --- |
| 35 Lithium-Ionen-Akku | Lithium, Kobalt, Lithium-Ionen-Akku (Zelle) | Lithium-Ionen-Akku (1991) |
| 36 Flüssigkristallbildschirm | LCD-Panel (in m² Bildfläche, neue Einheit `m2`) | TFT-LCD (1990) |
| 37 Mobiltelefon | Mobiltelefon | Mobilfunk (1992) |
| 38 Laptop | Laptop | Laptop (1992) |
| 39 Digitalkamera | Digitalkamera | Digitalkamera (1995) |
| 40 Flachbildfernseher | Flachbildfernseher (verdrängt den Farbfernseher) | Flachbildfernseher (2001) |
| 41 DVD-Spieler | DVD-Spieler (verdrängt den Videorekorder) | DVD (1996) |

Neue Formeln braucht die Epoche nicht. Geändert ist das Rezept des Mikrochips (M39): Sein
Gehäuse ist Glas und Keramik statt Phenolharz. Im ersten Weltlauf bis 2010 standen alle
Chipwerke in den USA, wo Phenolharz das Vierfache kostete; nach der Ausbauregel (kein Ausbau,
solange ein Vorprodukt mehr als `ausbau_vorprodukt_preis_max` × Richtpreis kostet) baute
keine Firma mehr aus, und Chips kosteten 1999–2009 das Vierfache. Das Chipwerk bekommt
eine eigene Fläche (10 ha statt 25 ha nach der Regel „Investition je ha“).

## M41 – Produkte 2010–2026

Lastenheft §18.4. Neue Produkte, Anlagen, Rezepte und Technologien stehen in
`data/ketten/42_*.yaml` bis `48_*.yaml` (Entwurf eines Hilfsagenten, geprüft und
übernommen; Annäherungen und Quellen in den Kommentaren der Daten):

| Kette | Neue Produkte | Technologien (Jahr) |
| --- | --- | --- |
| 42 Smartphone | Smartphone (verdrängt Mobiltelefon und Digitalkamera) | Smartphone (2007) |
| 43 Tablet | Tablet | Tablet (2010) |
| 44 Elektroauto | Traktionsbatterie (in kWh, neue Einheit `kwh`), Elektroantrieb, Elektroauto (verdrängt das Automobil) | Traktionsbatterie (2009), Elektroauto (2010), Zellfertigung in Großserie (2017) |
| 45 Solarmodul | Rohsilizium (Staatsmarkt), Solarsilizium, Solarmodul (in kWp, neue Einheit `kwp`, Staatsbedarf) | Solarsilizium, Solarmodul (2006), PERC-Solarzelle (2016) |
| 46 Windkraftanlage | Windkraftanlage (2,5 MW, Staatsbedarf) | Windkraftanlage (2000) |
| 47 Wärmepumpe | Wärmepumpe | Luft-Wasser-Wärmepumpe (2008) |
| 48 LED-Lampe | LED-Lampe (verdrängt die Glühlampe) | LED-Lampe (2009) |

Solarmodule und Windkraftanlagen kauft der Staat; `staatsnachfrage.verlauf` (M39) folgt
dem Zubau der Welt (Solar: 2010 = 0,15, 2020 = 0,95, 2024 = 2,6 je Mio. USD BIP; Wind:
2010 = 0,55, 2020 = 1,0 als Faktor auf den Grundwert). Die zweiten Rezepte (Batterie ab
2017, Modul ab 2016) senken die Kosten um etwa 40 %.

### Engpass ohne Ausweg (KI)

Die Suche nach einem Engpass (M32, „Engpass mit Ausweichen“) steigt bei knappen
Vorprodukten in deren Ketten ab. Führt keine davon zu einem Werk (alle Konzessionen
vergeben, kein bekanntes Verfahren), gilt neu:

- Stellt schon eine Firma das Produkt her, bleibt es wie bisher: kein Werk (es würde nur
  um dieselben knappen Vorprodukte konkurrieren).
- Stellt es noch niemand her, wird es trotzdem gebaut; das Werk bietet wie jeder Käufer
  um seine Vorprodukte (M16).

Im ersten Weltlauf bis 2026 entstand sonst nie ein Batteriewerk: Kobalt und Kohle waren
knapp, alle ihre Konzessionen vergeben, und die zwölf Elektroautowerke standen zehn Jahre
still.

### Rohstoffe bis 2026

Der Förderindex folgt der Welterzeugung, wo die Lagerstätten des Spiels sie sonst nicht
abbilden: Kohle neu ab 1980 (2010 = 2,2, 2026 = 2,6; vorher ab 2000 das 1,8- bis 3,9-Fache
des Richtpreises), Eisenerz 2010 = 3,0 und 2026 = 3,4 (vorher 2,2), Kupfererz 2010 = 3,6
und 2026 = 4,6 (vorher 3,0), Kobalt 2015 = 1,5, 2020 = 1,7, 2024 = 3,5. Die Kobaltgewinnung
braucht 230 statt 910 Stunden je t (Anteil neben Kupfer und Nickel; in den Ländern mit
geringer Produktivität standen die Gruben zu drei Vierteln still).

**Vorräte wachsen mit dem Förderindex** (Nacharbeit zu M41): Der abbauwürdige Vorrat einer
Lagerstätte im Jahr *t* ist Vorrat (Daten) · Förderindex ihres Rohstoffs im Jahr *t* · *s*;
die Förderung zählt dagegen wie bisher. So wachsen die Reserven wie in der Wirklichkeit
mit Erkundung und Technik, statt dass ein höherer Index die Lagerstätten nur schneller
leert. Vorher waren in den Weltläufen bis 2026 zwölf der 26 Eisenerz-Lagerstätten
(Kiruna, Mesabi, Kursk …) und die größten Ölfelder leer, Eisenerz kostete ab 2018 das 2,7-
bis 3,8-Fache des Richtpreises. Das gilt für die Grenze der Förderung (M33) und für die
Wahl einer Konzession durch die KI (`vorrat_jahre_min`). Nachwachsende Rohstoffe haben
keinen Vorrat; für Baumwolle (2020 = 8,0, 2026 = 8,5; vorher 6,9) und Getreide (2010 = 7,5,
2020 = 9,0, 2026 = 9,5; vorher 8,5 im Jahr 2026) steigt der Index nach 2010 weiter
(Polyester und Ertragssteigerung, die das Spiel nicht eigens abbildet; vorher kosteten sie
ab 2020 das 3,4- bis 3,9-Fache bzw. das Doppelte).

## M42 – Produktnamen je Firma

Offene Punkte, Abschnitt M. Bausteine: `data/ki/produktnamen.yaml` (Format:
`docs/DATENFORMAT.md`, Abschnitt `produktnamen`).

Jede Firma gibt ihren Endprodukten eigene, erfundene Namen; echte Produkte, Modelle und
Marken kommen nicht vor. Rohstoffe, Halbzeuge, Bauteile und Strom bleiben Gattungsware
ohne Namen.

### Welche Produkte einen Namen tragen

Ein Produkt p trägt Namen, wenn es ein Endprodukt ist und seine Warengruppe einem
Namensstil zugeordnet ist (`stile[].warengruppen`). Mit den Daten:

| Stil | Warengruppen | Beispiele |
| --- | --- | --- |
| `technik` | Elektro, Maschinen, Fahrzeuge | „Kelvor M80“, „Marvik Typ 12“, „Irvok 500“ |
| `marke` | Metallwaren, Möbel und Hausrat, Bekleidung, Lebensmittel, Chemie | „Aurela“, „Nerola Classic“ |

### Name bilden

Für Firma f, Produkt p mit Stil S am Tag t:

1. Muster: die Muster von S, deren Zeitraum das Jahr von t enthält (`ab` ≤ Jahr ≤ `bis`,
   fehlende Grenze = offen).
2. Stamm: Mit Wahrscheinlichkeit `hausmarke` nimmt f einen Stamm, den sie schon für ein
   anderes Produkt desselben Stils verwendet (eine Hausmarke), sonst einen Stamm aus S,
   den noch keine andere Firma verwendet (sind alle vergeben: irgendeinen). Der Stamm
   eines Namens ist sein erstes Wort.
3. Ein Muster zufällig, die Platzhalter `{stamm}`, `{zahl}`, `{buchstabe}`, `{zusatz}`
   zufällig aus den Listen von S.
4. Der Name ist frei, wenn keine andere Firma p schon so nennt (ohne Unterschied von
   Groß- und Kleinschreibung) und keines seiner Wörter in `ausgeschlossen` steht.
   Bis zu 30 Versuche; danach bekommt der letzte Versuch die kleinste freie Nummer
   angehängt („Corvel 300 2“).

Die Zufallszahlen kommen aus einem eigenen Strom je Firma und Produkt: ein Name
verschiebt keine anderen Zufallszahlen, und derselbe Spielstand ergibt denselben Namen.

### Wann Firmen Namen vergeben

- **Neue Firmen** (Startbesetzung und spätere Gründungen) benennen bei ihrer Gründung
  jedes Endprodukt ihrer Anlagen.
- **KI-Firmen** benennen bei ihren Betriebsentscheidungen (alle 7 bis 14 Tage, M10)
  jedes Endprodukt, das sie herstellen (eine Anlage mit Rezept dafür, auch im Bau) oder
  anbieten und das noch keinen Namen hat – mit dem Befehl `NameProduct`, wie jede
  Handlung.
- **Der Spieler** benennt seine Produkte selbst (`NameProduct`), jederzeit und für jedes
  Endprodukt mit Stil, auch vor der ersten Herstellung. Die Ansicht schlägt drei Namen
  aus denselben Bausteinen vor. Ohne Namen erscheint sein Angebot unter dem
  Gattungsnamen („Automobil“); ein Hinweis erinnert daran.

### Befehl `NameProduct { product, name }`

- `name = None` löscht den Namen.
- Geprüft wird: p trägt Namen (sonst Fehler „trägt keinen eigenen Namen“); der Name ist
  nach dem Entfernen äußerer Leerzeichen nicht leer, höchstens 40 Zeichen lang, bei
  keiner anderen Firma für p vergeben und enthält kein Wort aus `ausgeschlossen`.
- Namen gehören der Firma: Verkauf von Standorten (M30) und Versteigerung (M38)
  übertragen keine Namen; eine insolvente Firma behält ihre Namen.

### Anzeige

- Markt: Anbieterliste eines Produkts mit dem Produktnamen jedes Anbieters; eigenes
  Angebot mit Namen, Vorschlägen und Ändern.
- Wettbewerb: Firmenansicht mit den Namen ihrer Produkte.
- Rundenbericht: Die Meldungen über Wettbewerber in den eigenen Märkten (neuer Anbieter,
  Preissenkung, Rückzug, M24) nennen den Produktnamen: „Becker AG bietet jetzt
  „Corvel 300“ (Automobil) in Deutschland an, für 5.200 USD.“
- Werksansicht: Name des Produkts beim Verkauf.

### Spielstände

Neues optionales Feld je Firma (Name je Produkt). Ältere Spielstände laden ohne Namen;
KI-Firmen benennen ihre Produkte bei ihrer nächsten Betriebsentscheidung (ohne Meldung,
weil sie schon verkaufen), der Spieler bei Bedarf selbst.

## MA0 – Entscheidungsbausteine

`docs/MANAGER.md` Abschnitte 5.1, 6.2 und 11. Bevor eine Regel der KI handelt, legt sie
ihr Vorhaben als **Entscheidung** mit Optionen einem Entscheider vor. Der Entscheider der
KI lässt die Regel immer handeln wie bisher: Ihr Verhalten bleibt bitgleich. Ab MA2 prüfen
Manager an denselben Stellen gegen Budget und Befugnis (Modul `decision`).

### Entscheidung, Option, Schritt

- Eine **Entscheidung** (`Decision`) hat ein Thema (`Topic`), die Firma, wo nötig Standort
  und Produkt, zwei bis vier **Optionen** (`Choice`) und die Option, die die Regeln wählen.
- Eine Option hat eine Art (Textschlüssel `option.<art>`) und **Schritte** (`Step`): je
  einen Befehl und ob er Pflicht ist. Wird eine Option als Befehlsliste ausgeführt, dann
  der Reihe nach; schlägt ein Pflichtschritt fehl, entfallen die folgenden.
- Jede Entscheidung bietet **Beibehalten** (keine Schritte).
- **Ablauf:** Die Regel bildet die Entscheidung und legt sie einem **Entscheider** vor
  (`Decider`). Er antwortet mit „Regel“ (die Regel führt ihre Option aus wie bisher), mit
  einer anderen Option (sie wird sofort als Befehlsliste ausgeführt) oder mit „nichts
  jetzt“ (beibehalten oder nachgefragt, ab MA2). Der Entscheider der KI antwortet immer
  „Regel“ und lässt die Entscheidung gar nicht erst bilden; ihr Verhalten und ihr Tempo
  bleiben gleich.
- Warum die Regel ihre eigene Option selbst ausführt: Manche Regeln planen einen Schritt
  erst, wenn der vorige getan ist (das Grundstück wird nach dem Kredit gekauft oder
  gepachtet, je nach der Kasse danach; die Bauzahl folgt aus der Größe des gefundenen
  Grundstücks). Die Schritte der Regel-Option beschreiben den Plan zum Zeitpunkt der
  Entscheidung; Manager, die sie später ausführen, führen diesen Plan aus.
- Die **Bewertung** (`Assessment`) je Option wird erst auf Nachfrage gerechnet.

### Themen

| Thema (`thema.<id>`) | Regel der KI | Termin | Optionen (die Wahl der Regeln zuerst) |
| --- | --- | --- | --- |
| `produktion` | Betrieb: Rezept und Auslastung je Anlage | alle paar Tage | neue Werte; beibehalten |
| `verkauf` | Betrieb: Preisuntergrenze und Vorrat je Angebot, Verkauf von Nebenprodukten | alle paar Tage | neue Werte; beibehalten |
| `einkauf` | Betrieb: Lagerziel und Höchstpreis je Vorprodukt | alle paar Tage | neue Werte; beibehalten |
| `lohn` | Betrieb: Lohnaufschlag je Standort | alle paar Tage | neuer Aufschlag; beibehalten |
| `eigenversorgung` | Lieferungen an eigene Standorte | alle paar Tage | liefern; beibehalten |
| `produktname` | Namen für Endprodukte (M42) | alle paar Tage | Name; ohne Namen |
| `kasse` | Kredit aufnehmen oder tilgen | monatlich | Kredit bzw. Tilgung; beibehalten |
| `werbung` | Werbebudget je Land und Warengruppe | monatlich | neues Budget; beibehalten |
| `kaufangebot` | Angebote für Standorte anderer Firmen (M30) | monatlich | anbieten; nicht anbieten |
| `ueberkapazitaet` | Einheiten stilllegen (M22), je Produkt und Standort | Quartal | stilllegen, die übrigen übernehmen die Erzeugung; dieselben Einheiten verkaufen; beibehalten |
| `stillgelegt` | lange stillgelegte Einheiten verkaufen (M22) | Quartal | verkaufen; stillgelegt lassen |
| `wiederanfahren` | stillgelegte Einheiten wieder anfahren | Quartal | anfahren; stillgelegt lassen |
| `ausbau` | Ausbau am Standort oder auf weiterem Grundstück | Quartal | bauen (Größe, Anzahl, nötiger Kredit); beibehalten |
| `kraftwerk` | eigenes Kraftwerk bei Strommangel | Quartal | bauen; beibehalten |
| `lagerstaette` | Konzession erschließen | Quartal | gründen, erschließen, bauen; beibehalten |
| `engpass` | Bauen in einem Engpass oder als Pionier (Diversifizierung) | Quartal | bauen; beibehalten |
| `forschung` | Forschungsziel des Labors (ohne Labor: erst Standort und Labor gründen) | jährlich | Ziel nach Regel, bis zu zwei weitere erforschbare Technologien der eigenen Branchen (die billigsten); beibehalten |
| `weiterentwicklung` | Entwicklungsziel (M37) | jährlich | Ziel nach Regel; beibehalten |

Nicht dazu gehören die Gründung neuer KI-Firmen und die Startbesetzung: Sie sind Sache
der Welt, nicht einer Firma. Die Antworten der KI auf Kaufangebote und ihre Gebote bei
Versteigerungen folgen mit der Landes- und Kontinentebene (MA3).

### Angerechneter Betrag je Option

Summe über die Schritte (Abschnitt 5.1 der Vorgabe):

| Befehl | Betrag |
| --- | --- |
| `BuildFacility` | Investition · Größenfaktor · Anzahl |
| `DevelopDeposit` | Erschließungskosten der Konzession |
| `FoundSite`, `FoundSiteOnPlot` | Gebäude; dazu Kaufpreis des Grundstücks bzw. eine Jahrespacht |
| `BuyPlot` | Kaufpreis |
| `MakeOffer` | gebotener Preis |
| `SetWagePremium` (höher) | Mehrkosten der Arbeit für 365 Tage bei geplanter Auslastung |
| `SetPurchase` (höherer Höchstpreis) | (neuer − alter Höchstpreis) · Lagerziel / Lagertage · 365; als alter Höchstpreis gilt mindestens der übliche der Regeln (Marktpreis im Land mit `einkauf_aufschlag`), ein neuer Auftrag zu diesem Preis zählt nichts |
| `SetAdvertising` (höher) | (neues − altes Monatsbudget) · 12 |
| `MothballFacility`, `SellFacility` | Restbuchwert der Einheiten |
| `RestartFacility` | Wiederanlaufkosten |
| `TakeLoan`, `RepayLoan` | Betrag (nur Ressort Finanzen oder CEO, ab MA5) |
| alle übrigen (Preis, Verkauf, Produktion, Forschung, Lieferung, Name) | 0 |

Ein Höchstpreis, ein Aufschlag oder ein Budget, das sinkt, zählt 0.

### Geschätzte Wirkung je Option

Ergebniswirkung in einem Jahr gegenüber „Beibehalten“, zu heutigen Preisen; einmalige
Beträge stehen getrennt.

- **Produkt an einem Standort** (Themen `produktion`, `lohn` – dort alle Produkte des
  Standorts –, `ueberkapazitaet`, `stillgelegt`, `wiederanfahren`):

      E = 365 · (min(A, Q) · p − Q · v) − F

  *A* Abgang je Tag wie beim Stilllegen (Verkäufe des Vormonats und des laufenden Monats
  je Tag plus Verbrauch eigener Anlagen am Standort), *Q* Erzeugung je Tag der laufenden
  Einheiten bei geplanter Auslastung, *p* Angebotspreis (ohne Angebot der Marktpreis im
  Land), *v* variable Stückkosten (Vorprodukte zum Marktpreis, Arbeit mit Lohnaufschlag,
  Strom), *F* feste Kosten je Jahr (Abschreibung und Wartung; stillgelegt nur
  `instandhaltung_anteil` der Wartung und die Abschreibung; verkauft keine). Wirkung =
  E(Option) − E(beibehalten).
  - Ausbau, Kraftwerk, Lagerstätte, Engpass: Die neue Leistung *ΔQ* bei
    `start.auslastung` gilt als verkauft (die Regeln bauen nur, wo der Markt mehr
    abnimmt): Wirkung = 365 · ΔQ · (p − v) − ΔF − Zinsen eines nötigen Kredits.
  - Einmalig (Ergebnis): Verkaufserlös − Restbuchwert (`verkaufen`),
    Wiederanlaufkosten (`wiederanfahren`). Investition und Erschließung sind kein
    Aufwand: Sie stehen im angerechneten Betrag und wirken über Abschreibung und Wartung,
    die Erschließung über `erschliessung_lebensdauer_jahre`.
- **Kasse:** − Zinsen des Kredits für ein Jahr; Tilgung: + ersparte Zinsen.
- **Werbung:** − (neues − altes Budget) · 12; den Mehrabsatz schätzt erst MA2.
- **Einkauf, Verkauf (Preisuntergrenze), Lieferungen, Namen, Forschung,
  Weiterentwicklung, Kaufangebote:** ohne Schätzung („–“). Sie halten den Betrieb in Gang
  oder wirken erst nach Jahren.

### Nachweis „bitgleich“

- Referenzläufe mit dem Stand vor MA0 und danach (100 KI-Firmen; 1900–1912, Seed 5;
  1985–1992, Seed 6) enden mit demselben Zustands-Hash und denselben Protokolldateien.
- Test: Ein Entscheider, der jede Entscheidung bilden lässt, aufzeichnet und „Regel“
  antwortet, ergibt denselben Zustand wie die KI ohne ihn (das Bilden ändert nichts).

## MA1 – Stellen und Manager

`docs/MANAGER.md` Abschnitte 2–4 und 11. Daten: `data/parameter/management.yaml`
(Format: `docs/DATENFORMAT.md`, Abschnitt `management`), Vornamen je Namensgruppe in
`data/ki/namen.yaml`. Kern: Modul `management`.

### Stellen

- Jeder Standort hat eine **Leitung** und die **Fachstellen** seines Standorttyps aus den
  Daten (Werk: Produktion, Einkauf und Lager, Vertrieb und Marketing, Personal, Logistik;
  Förderstätte: Produktion, Vertrieb und Marketing, Personal; Kraftwerk: Produktion,
  Einkauf und Lager; Lager: Logistik; Niederlassung: Vertrieb und Marketing;
  Forschungszentrum: keine). Die Stellen entstehen mit dem Standort und vergehen mit ihm;
  wer auf einer Stelle eines verkauften oder versteigerten Standorts sitzt, kehrt in den
  Bewerberpool zurück.
- Stellen der Länder, Kontinente und des Vorstands folgen mit MA3 und MA5. Das Organigramm
  zeigt die Ebenen schon (Konzern → Kontinent → Land → Standort).
- Jeder **Bereich** deckt Themen der Entscheidungen (MA0) ab, in den Daten änderbar:
  Produktion – `produktion`; Einkauf und Lager – `einkauf`, `eigenversorgung`; Vertrieb
  und Marketing – `verkauf`; Personal, Logistik, Forschung, Finanzen – in MA1 keine (mit
  Budget ab MA2: `lohn`, `werbung`, `kasse` …).
- Zuständig für ein Thema an einem Standort ist die Fachstelle seines Bereichs, wenn der
  Standorttyp sie hat und sie besetzt ist, sonst die Leitung (mit Abschlag, unten). Ist
  beides leer, entscheidet der Spieler selbst.

### Manager

- Fähigkeiten 0–100: Fachkompetenz je Bereich, Erkennen, Urteilsvermögen, Führung,
  Risikoneigung, Fragefreude (Wirkung von Urteilsvermögen, Führung und Fragefreude ab
  MA2). Jeder Manager hat einen **Schwerpunkt** (ein Bereich). Ziehung aus dem Strom des
  Managermarkts: Fachkompetenz im Schwerpunkt ~ N(`schwerpunkt.mittel`,
  `schwerpunkt.streuung`), in den übrigen Bereichen ~ N(`sonst.mittel`, `sonst.streuung`),
  Erkennen, Urteilsvermögen und Führung ~ N(`allgemein.mittel`, `allgemein.streuung`),
  Risikoneigung und Fragefreude gleichverteilt; alles auf 0–100 begrenzt und gerundet.
- Name: Vorname und Familienname der Namensgruppe des Heimatlands (sonst der
  Standardgruppe; `familienname_zuerst` dreht die Folge), frei unter den Managern: bis zu
  20 Versuche, danach mit Initiale („Erik B. Holm“). Heimatland nach der Zahl der
  Akademiker der Länder des Kontinents gewichtet. Schwerpunkt gleichverteilt über die
  Bereiche.
- **Anzeige:** fünf Stufen („schwach“ unter 20, „mäßig“ unter 40, „solide“ unter 60,
  „stark“ unter 80, sonst „herausragend“), aus Wert + Eindruck (auf 0–100 begrenzt). Der Eindruck je Fähigkeit
  wird bei der Ziehung gleichverteilt aus ±`eindruck_unschaerfe` gezogen und bleibt fest
  (ein starkes Personalressort macht ihn ab MA5 kleiner). Der Kern liefert nur die Stufe.

### Bewerberpool

- Je Kontinent *N* = clamp(round(Akademiker des Kontinents in Mio. ·
  `pool.je_mio_akademiker`), `pool.min`, `pool.max`) Kandidaten, die Akademiker aller
  Fachrichtungen der Länder des Kontinents nach dem Ländermodell des Jahres.
- Zum Spielbeginn gezogen. Am Monatsanfang verlässt jeder freie Kandidat den Markt mit
  `pool.abgang_monat`, dann wird jeder Kontinent auf *N* aufgefüllt (alte Spielstände:
  am ersten Monatsanfang nach dem Laden).
- Entlassene Manager kehren in den Pool ihres Heimatkontinents zurück.

### Gehalt

    Gehalt je Jahr = Gehaltsfaktor der Stelle · (0,5 + Stärke / 100)
                     · Stundenlohn akademiker.kaufmaennisch im Land des Standorts
                     · Jahresarbeitsstunden (Ländermodell)
    Stärke = Mittel aus Fachkompetenz im Schwerpunkt, Erkennen und Urteilsvermögen

- Gehaltsfaktoren (× Akademikerlohn): Fachstelle Standort 1,5, Leitung Standort 2,5 (Land
  3/4, Kontinent 5/7, Vorstand 10/15 ab MA3/MA5; Vorgabe 3).
- Festgelegt bei der Einstellung; beim Versetzen gilt das höhere aus bisherigem Gehalt und
  Forderung für die neue Stelle.
- Gebucht am Monatsende: ein Zwölftel als Personalkosten (Kostenart Personal) der
  Kostenstelle Standort, im Monat des Eintritts nach Tagen (Zwölftel · Tage seit dem
  Eintritt / Tage des Monats). Wer im Lauf des Monats geht oder versetzt wird, bekommt
  die Tage bis dahin sofort, gebucht auf den bisherigen Standort.
- **Abfindung** beim Entlassen: `abfindung_monate` · Jahresgehalt / 12, sofort, ebenso
  gebucht. Wer mit einem verkauften oder versteigerten Standort seine Stelle verliert,
  bekommt keine Abfindung.

### Befehle

- `HireManager { manager, position }`: Der Kandidat ist frei, die Stelle gehört zu einem
  eigenen Standort, passt zu seinem Typ und ist frei.
- `MoveManager { manager, position }`: eigener Manager auf eine freie eigene Stelle.
- `DismissManager { manager }`: eigener Manager; Abfindung, zurück in den Pool.
- Stelle = Standort + Rolle (Leitung oder Fachstelle eines Bereichs).

### Routine der Standortstellen

- Prüftermin je Standort alle `pruefung_tage` (Standort: 7) Tage, gestaffelt nach der
  Standortnummer: fällig, wenn (Tage seit dem 1.1.1900 + Standortnummer) durch
  `pruefung_tage` teilbar ist. In Forschungszentren läuft keine Routine (ihre Themen
  kommen mit dem Budget, MA2).
- Am Prüftermin laufen für den Standort die Betriebsregeln der KI (MA0: Produktion,
  Verkauf, Einkauf, Eigenversorgung) über einen Entscheider der Firma: Ein Thema läuft,
  wenn eine besetzte Stelle es abdeckt (Fachstelle des Bereichs, sonst die Leitung) und
  die Stelle die Lage bemerkt. Sonst bleibt es, wie es ist (der Spieler entscheidet
  selbst).
- **Bemerken** je Bereich und Prüftermin mit der Wahrscheinlichkeit
  `bemerken_grund` + (1 − `bemerken_grund`) · Sorgfalt / 100 (Vorgabe 0,5: zwischen 50 %
  und 100 %), Sorgfalt = (Fachkompetenz im Bereich + Erkennen) / 2; die Leitung anstelle
  der Fachstelle mit Fachkompetenz · (1 − `leitung_ohne_fach_abschlag`). Zufall aus dem
  eigenen Strom des Managers und Tages (eine Ziehung je Bereich, Bereiche in der
  Reihenfolge der Daten).
- Die Regeln nehmen für die Firma des Spielers mittlere Kompetenz und Aggressivität
  (0,5). Feste Preise des Spielers bleiben (die Regeln setzen nur Untergrenzen im
  Marktpreis-Modus); Auslastung und Einkauf führt die Stelle selbst. Was der Standort
  herstellt und weder anbietet noch selbst braucht, bietet der Vertrieb zum Marktpreis
  an (wie die KI bei Nebenprodukten); Lieferungen zwischen eigenen Standorten
  (`eigenversorgung`) bestellt der Einkauf des empfangenden Standorts bei Standorten, die
  die Ware anbieten.
- Bis zum nächsten Prüftermin gelten Änderungen des Spielers; die Stelle passt sie danach
  nach den Regeln an.
- KI-Firmen haben in MA1 keine Manager (ab MA6).

### Spielstände

Neue Felder mit Vorgabewerten: alte Stände laden ohne Manager; der Pool entsteht am
ersten Monatsanfang.

## MA2 – Budget und Anliegen auf Standortebene

`docs/MANAGER.md` Abschnitte 5, 6 und 12. Daten: `parameter/management.yaml` (Budget je
Ebene, Anliegen), Themen der Bereiche erweitert. Kern: Modul `management`.

### Themen der Stellen

| Bereich | Themen (MA0) | Neu in MA2 |
| --- | --- | --- |
| Produktion | `produktion` | `ueberkapazitaet`, `stillgelegt`, `wiederanfahren`, `ausbau` |
| Einkauf und Lager | `einkauf`, `eigenversorgung` | – |
| Vertrieb und Marketing | `verkauf` | – |
| Personal | – | `lohn` |
| Forschung | – | `forschung`, `weiterentwicklung` (Labore: Laborleitung) |
| Logistik, Finanzen | – | (Stufe 2 bzw. MA5) |

Forschung und Weiterentwicklung kommen nur in Laboren vor, alle anderen Themen nur
außerhalb davon; eine Stelle zeigt und übernimmt nur die Themen, die an ihrem Standorttyp
vorkommen.

### Wann welche Regel läuft

Für Standorte mit besetzter Stelle laufen die Regeln der KI wie bei KI-Firmen, über den
Entscheider der Stellen:

- an jedem Prüftermin des Standorts (MA1): Betrieb (`produktion`, `verkauf`, `einkauf`,
  `lohn`) und Lieferungen (`eigenversorgung`);
- am letzten Tag eines Quartals für alle besetzten Standorte: Stilllegen, Verkaufen und
  Wiederanfahren (`ueberkapazitaet`, `stillgelegt`, `wiederanfahren`) und Ausbau am
  Standort (`ausbau`, je Standort geprüft); Kraftwerke und Lagerstätten sind Sache von Land
  und Kontinent (MA3);
- im Labor an seinem Prüftermin, wenn es kein Ziel hat, und am 1. Januar: das nächste
  Forschungs- oder Entwicklungsziel (`forschung`, `weiterentwicklung`, nur Technologien der
  eigenen Branchen).

Am Quartalstag zieht jede Stelle ihr Bemerken (MA1) für die Themen dieses Tages.

### Budget je Stelle

    Bezug B = Umsatz des Standorts in den letzten 12 abgeschlossenen Monaten;
              ohne Umsatz (Labor, Kraftwerk, Lager) seine Kosten in dieser Zeit
    je Entscheidung = max(a · B, s_E · Jahresgehalt)
    je Jahr         = max(b · B, s_J · Jahresgehalt)

*a*, *b* je Ebene und Rolle aus den Daten (Standort: Fachstelle 2 % / 5 %, Leitung
5 % / 10 %), *s_E* = 1, *s_J* = 3 (`budget_sockel_gehaelter`). Der Spieler setzt *a* und
*b* je Stelle (`SetBudget`); 0 heißt „immer fragen“ (dann gilt auch kein Sockel). Das
Jahresbudget beginnt am 1. Januar neu. Was nach einer Freigabe des Spielers ausgeführt
wird, zählt nicht. Umsatz und Ergebnis je Standort führt das Hauptbuch dafür je Monat.

### Entscheiden

Für jede Entscheidung, deren Thema eine Stelle am Standort abdeckt und bemerkt hat:

1. **Bewerten** (MA0): angerechneter Betrag *A_i*, Wirkung *W_i* (Ergebnis je Jahr plus
   einmalige Wirkung) je Option; ohne Schätzung *W_i* = 0 für „Beibehalten“, sonst gilt die
   Option der Regeln als beste.
2. **Empfehlen:** Mit der Wahrscheinlichkeit *g* + (1 − *g*) · Urteilsvermögen / 100
   (*g* = `empfehlung_grund`, Vorgabe 0,5) die Option
   mit der höchsten wahren Wirkung, sonst die mit der höchsten geschätzten Wirkung
   *W_i* · (1 + ε_i), ε_i gleichverteilt in ±*f*, *f* = `schaetzfehler` · (1 −
   Fachkompetenz / 100). Zufall aus dem Strom des Managers.
3. **Im Rahmen:** *A* der Empfehlung ≤ Rest je Entscheidung und ≤ Rest im Jahr, keine
   Kredite in den Schritten (dürfen erst Finanzen und CEO, MA5): Die Stelle führt sie aus
   und bucht *A* auf ihr Jahresbudget; Entscheidungen mit Betrag oder Wirkung kommen ins
   Protokoll der Stelle.
4. **Sonst Anliegen** an den Spieler (MA3: an die nächste Stelle), außer das Thema ist für
   die Stelle stummgeschaltet oder gesperrt, die Stelle hat schon `offen_je_stelle` offene
   Anliegen oder dasselbe Anliegen (Thema, Standort, Produkt) ist offen. Bis zur Antwort
   bleibt alles, wie es ist. Der **Grund** wird festgehalten: Kredit in den Schritten
   (`kredit`), sonst Budget 0 (`immer`), sonst *A* > Budget je Entscheidung
   (`entscheidung`), sonst reicht das Jahresbudget nicht (`jahr`).

### Anliegen

- Inhalt: Stelle und Manager, Thema, Standort und Produkt, die Optionen mit Befehlen,
  Betrag und Prognose als Spanne *W_i* · (1 ± *f*), die Empfehlung, Frist =
  Tag + `frist_tage`.
- Antworten (`AnswerConcern`): eine Option wählen (sofort als Befehlsliste ausgeführt);
  „Entscheide selbst“ (die Empfehlung wird ausgeführt); „Zu diesem Thema nicht mehr
  fragen“ (die Stelle entscheidet das Thema weiter im Budget, darüber bleibt es ohne
  Rückfrage, wie es ist); „Ablehnen“ (das Thema ruht an dieser Stelle `sperre_tage`).
- „Wieder fragen“ (`AskAgain`) hebt Stummschaltung und Sperre eines Themas an einer
  Stelle auf.
- Nach der Frist: Das Anliegen verfällt, nichts ändert sich.
- **Erledigt:** Jeder ausgeführte Befehl der Firma schließt ihre offenen Anliegen, deren
  Optionen dasselbe Objekt betreffen – dieselbe Anlage (Auslastung, Automatisierung,
  Stilllegen, Wiederanfahren, Verkaufen), dasselbe Produkt am Standort im Verkauf (Preis,
  Angebot) oder im Einkauf, dieselbe Lieferung (Ziel, Produkt), Lohnaufschlag, Ausbau,
  Labor oder Lagerstätte des Standorts. Das gilt für eine Entscheidung des Spielers in einer
  Ansicht wie für eine spätere Entscheidung der Stelle im Budget.
- **Wichtig** ist ein Anliegen, dessen Thema nicht zur Routine (`routine_themen`) gehört.
- **Anhalten:** Mehrere Runden am Stück (M26) halten nach der Runde an, in der ein neues
  Anliegen entstand – je nach Einstellung bei allen, nur bei wichtigen (Vorgabe) oder nie.
  Die Meldung eines neuen Anliegens zählt dafür nicht als Warnung.
- **Bündelung:** Offene Anliegen mit gleichem Thema und gleicher Art der Empfehlung stehen
  in einer Gruppe; „für alle übernehmen“ beantwortet jedes mit „Entscheide selbst“.

### Rückmeldung zu Folgen

Für ausgeführte Optionen mit Wirkung an einem Standort: Basis = mittleres
Monatsergebnis des Standorts in den drei abgeschlossenen Monaten davor. Nach
`wirkzeit_tage` meldet die Stelle (oder für den Spieler der Standort)
(Mittel der drei letzten abgeschlossenen Monate − Basis) · 12 gegen die Prognose.

### Hinweise

Hinweise aus „Zu erledigen“ zu einem Standort entfallen, wenn die Stelle des Bereichs
besetzt ist oder die Leitung des Standorts ihn mangels Fachstelle übernimmt: fehlende
Vorprodukte und kein Einkauf → Einkauf und Lager; kein Verkauf, Preis unter den
Stückkosten, nichts verkauft → Vertrieb und Marketing; fehlende Arbeitskräfte und
Personal → Personal; Anlage ohne Verfahren oder ruhend → Produktion; Labor ohne Ziel →
Forschung. Strom und Lagerstätten bleiben (Sache von Land und Kontinent, MA3). Offene
Anliegen stehen als eigener Hinweis mit der frühesten Frist in der Übersicht.

## MA3 – Land und Kontinent

`docs/MANAGER.md` Abschnitte 3, 5.3, 6.5 und 11. Daten: `parameter/management.yaml`
(Fachstellen und Themen je Ebene, `anliegen.buendel_ab`). Kern: Modul `management`.

### Einheiten und Stellen

- **Einheiten:** Standort, Land, Kontinent. Eine Firma hat die Einheit eines Landes
  (Kontinents), solange ihr dort mindestens ein Standort gehört.
- **Stellen:** je Einheit eine Leitung (Landesleitung, Kontinentvorstand) und die
  Fachstellen ihrer Ebene (`ebenen[].fachstellen`; Vorgabe: Produktion, Einkauf und Lager,
  Vertrieb und Marketing, Personal, Logistik, Forschung). Stellen ohne Themen ihrer Ebene
  lassen sich nicht besetzen (wie MA1).
- **Gehalt:** Faktor der Ebene × (0,5 + Stärke / 100) × Jahreslohn der Akademiker im
  **Sitzland** der Einheit (MA1). Sitzland eines Landes ist das Land selbst, eines
  Kontinents das Land des Firmensitzes, wenn es dort liegt, sonst das Land des Kontinents
  mit den meisten eigenen Standorten (bei Gleichstand das erste in den Daten). Gehälter
  und Abfindungen der höheren Stellen sind Gemeinkosten der Firma (Kostenstelle ohne
  Standort).
- Verliert die Firma ihren letzten Standort einer Einheit, enden deren Stellen zum
  nächsten Monatsanfang (wie MA1).

### Themen je Ebene

Jede Ebene nennt die Themen, die ihre Stellen selbst aufgreifen (`ebenen[].themen`):

| Ebene | Themen |
| --- | --- |
| Standort | Auslastung, Verkauf, Einkauf, Lieferungen, Lohn, Überkapazität, stillgelegte Anlagen, Wiederanfahren, Ausbau am Standort; im Labor nur Forschung und Weiterentwicklung |
| Land, Kontinent | Überkapazität, stillgelegte Anlagen, Wiederanfahren, Ausbau (für Standorte ohne eigene zuständige Stelle und neue Standorte), Kraftwerk, Lagerstätte, Werbung |

Eine Stelle erledigt die Themen ihres Bereichs, die Leitung die Themen der Bereiche ohne
besetzte Fachstelle (MA1). Weitergeleitete Entscheidungen (unten) übernimmt eine höhere
Stelle für jedes Thema ihres Bereichs.

### Wann die höheren Stellen handeln

- **Am letzten Tag eines Quartals** ziehen die Stellen jedes Landes ihr Bemerken (MA1).
  Für die Standorte des Landes ohne eigene zuständige Stelle (weder Fachstelle Produktion
  noch Leitung besetzt) laufen dann die Regeln für Stilllegen, Verkaufen und Wiederanfahren
  und für den Ausbau; mit ihnen die Regeln für ein eigenes Kraftwerk, eine neue
  Lagerstätte und einen neuen Standort, wenn der alte keinen Platz mehr hat. Der Kontinent
  tut dasselbe für die Standorte seiner Länder ohne besetzte Landesstelle des Bereichs.
- **An ihrem Prüftermin** (alle `pruefung_tage` der Ebene, gestaffelt nach der Nummer der
  Einheit) die Werbung je Warengruppe in den Ländern der Einheit nach der Regel der KI
  (M16).

### Zuständigkeitskette und Weiterleitung

Für eine Entscheidung zum Thema *T* (Bereich *B*) an einem Standort oder in einem Land
gilt die Kette

    Fachstelle B des Standorts → Leitung des Standorts → Fachstelle B des Landes →
    Landesleitung → Fachstelle B des Kontinents → Kontinentvorstand → (MA5) → Spieler

Nur besetzte Stellen zählen; Stellen am Standort nur, wenn das Thema dort vorkommt.

1. Es beginnt die erste Stelle der Kette, die das Thema heute bemerkt hat.
2. Jede Stelle ab dort empfiehlt mit ihrem Urteilsvermögen und ihrer Fachkompetenz (MA2)
   und führt ihre Empfehlung aus, wenn deren Betrag in ihr Budget passt (je Entscheidung
   und Rest im Jahr) und sie keinen Kredit braucht (Kredite: MA5). Der Betrag zählt dann
   nur auf ihr eigenes Budget.
3. Sonst geht die Entscheidung an die nächste Stelle. Kann keine entscheiden, wird sie ein
   **Anliegen der obersten Stelle** an den Spieler, mit dem Weg: jede Stelle mit Manager und
   Empfehlung. Empfehlung und Prognose des Anliegens sind die der obersten Stelle.
4. Kein Anliegen entsteht, wenn das Thema an einer Stelle des Wegs stummgeschaltet oder
   gesperrt ist, wenn die erste Stelle schon `offen_je_stelle` offene Anliegen hat oder
   dasselbe Anliegen (Thema, Standort bzw. Land, Produkt) offen ist. „Nicht mehr fragen“,
   „Ablehnen“ und „Wieder fragen“ gelten für die oberste Stelle.

### Budget der höheren Stellen

- **Bezug** *B*: Umsatz der Firma in den letzten 12 abgeschlossenen Monaten an den
  Standorten des Landes (Kontinents); ohne Umsatz deren Kosten (wie MA2).
- **Anteile** *a*, *b* je Ebene und Rolle aus den Daten (Land: Fachstelle 3 % / 6 %,
  Leitung 5 % / 12 %; Kontinent: 3 % / 8 % und 6 % / 15 %); Sockel wie MA2.
- **Vorgaben je Stellentyp** (`SetBudgetRule`): Anteile für einen Stellentyp – Ebene, bei
  Standorten der Standorttyp, Leitung oder Bereich – für die ganze Firma, einen Kontinent
  oder ein Land. Es gilt die eigene Einstellung der Stelle, sonst die Vorgabe ihres
  Landes, ihres Kontinents, der Firma, sonst der Standardwert.
- **Deckel:** Budget je Entscheidung und je Jahr einer Stelle sind höchstens die ihrer
  nächsten besetzten Leitung: für Fachstellen die Leitung der eigenen Einheit, für
  Leitungen die der nächsthöheren Einheit (Standort → Land → Kontinent). Ausgaben zählen nur
  bei der Stelle, die entscheidet.

### Strategische Anliegen

Entstehen an einem Tag bei derselben obersten Stelle mindestens `buendel_ab` Anliegen mit
gleichem Thema und gleicher Art der Empfehlung aus verschiedenen Standorten, legt sie diese
als **ein** Anliegen vor: Optionen „An allen umsetzen“ (die Empfehlung je Standort) und „So
lassen“; Betrag, Prognose und einmalige Wirkung sind die Summen der Teile. Die Antwort gilt
für alle Teile; jeder Teil wird für sich ausgeführt (scheitert einer, laufen die übrigen).

### Hinweise und Spielstände

- Die Hinweise „Strom fehlt“ und „Lagerstätte“ eines Standorts entfallen, wenn eine
  besetzte Stelle der Kette das Thema Kraftwerk bzw. Lagerstätte übernimmt.
- Stellen alter Spielstände (Standort und Rolle) werden als Stellen ihres Standorts
  gelesen; das Format bleibt lesbar ohne neue Version.

## MA4 – Strategievorgaben

`docs/MANAGER.md` Abschnitt 7, Lastenheft §5.6. Daten: `parameter/management.yaml`
(`strategie`). Kern: Modul `strategy`.

### Geltungsbereiche und Vererbung

- Eine Vorgabe gilt für die ganze Firma (in Stufe 1 zugleich „weltweit“ und
  „Gesamtkonzern“; Tochterfirmen mit Stufe 2), einen Kontinent, ein Land oder einen
  Standort. Länder und Kontinente dürfen auch solche sein, in denen die Firma noch keinen
  Standort hat: Die Vorgabe gilt dann für neue Standorte dort.
- An einem Standort gilt je Feld die Vorgabe des Standorts, sonst die seines Landes,
  seines Kontinents, der Firma, sonst der **Standardwert**. Für ein Land und einen
  Kontinent gilt dasselbe ab ihrer Ebene. Eine untere Ebene überschreibt so gezielt eine
  geerbte Vorgabe; entfernt sie ihre eigene, gilt wieder die geerbte.
- Vorgaben richten sich an die Manager (MA1–MA3): Sie wirken, wo eine besetzte Stelle
  entscheidet. Was der Spieler selbst entscheidet, gilt sofort und hat Vorrang – ein
  fester Preis bleibt fest; Auslastung, Einkauf, Lohn und Lieferungen passt die
  zuständige Stelle an ihrem nächsten Prüftermin wieder nach der Vorgabe an. Wer an einem
  Ort dauerhaft anders will, setzt dort eine eigene Vorgabe.
- **Wie gut** eine Vorgabe umgesetzt wird, hängt vom Manager ab: Eine Stelle handelt am
  Prüftermin nur, wenn sie die Lage bemerkt (MA1) – eine geänderte Vorgabe setzt ein
  schwacher Manager also später um –, und bei Ermessensentscheidungen empfiehlt sie nach
  ihrem Urteilsvermögen (MA2).
- KI-Firmen haben keine Vorgaben (bis MA6). Ihre Regeln laufen mit den Standardwerten,
  die genau ihr bisheriges Verhalten sind.

### Felder

| Feld | Einstellungen | Standard | Wirkung |
| --- | --- | --- | --- |
| Preis | Marktpreis, Premium, Kampfpreis, Mindestmarge *m* | Marktpreis | Preisuntergrenze und Startpreis der Angebote im Marktpreis-Modus |
| Lager | Reichweite der Vorprodukte min/max, Lagerziel der Fertigwaren (Tage) | 7 / 20 / 14 | Einkauf und Erzeugung |
| Personal | Lohnaufschlag min/max | 0 % / 30 % | Lohnaufschlag je Standort |
| Eigenfertigung oder Zukauf | eigene Ware zuerst, nach Preis, nur Zukauf | eigene Ware zuerst | Lieferungen zwischen eigenen Standorten |
| Investitionsbudget | Betrag je Kalenderjahr | ohne | Investitionen der Stellen im Geltungsbereich |
| Liquiditätsreserve | Monate laufender Kosten | 0 | Investitionen nur über der Reserve |

Die Standardwerte von Lager und Personal sind die Werte der KI
(`verhalten.lager_niedrig_tage`, `start.lager_eingang_tage`, `verhalten.lager_ziel_tage`;
min(`verhalten.lohnaufschlag_max`, `produktionsmodell.lohnaufschlag_max`)).

**Preis.** Die Stelle setzt für jedes Angebot im Marktpreis-Modus

    Preisuntergrenze = Vollkosten je Stück · u        Startpreis = Marktpreis · (1 + s)

| Strategie | *u* | *s* |
| --- | --- | --- |
| Marktpreis | `verhalten.preisuntergrenze` der Regeln (bei Aggressivität 0,5: 1,15) | 0 |
| Premium | `strategie.premium.untergrenze` (1,35) | `strategie.premium.aufschlag` (+15 %) |
| Kampfpreis | `strategie.kampfpreis.untergrenze` (0,95) | `strategie.kampfpreis.aufschlag` (−10 %) |
| Mindestmarge *m* | 1 + *m* (0 ≤ *m* ≤ `strategie.mindestmarge_max`) | 0 |

Vollkosten je Stück wie bisher (M16, M22). Über der Untergrenze bestimmt der Markt den
Preis (M16); der Startpreis gilt, wenn das Angebot neu ist oder seine Strategie wechselt.
Die Stelle setzt das Angebot neu, wenn sich die Untergrenze um mehr als 5 %, der
zurückgehaltene Vorrat um mehr als 10 % oder der Startaufschlag ändert.

**Lager.** Mit Reichweite *min* und *max* der Vorprodukte und Lagerziel *z* der
Fertigwaren:

- Einkauf: Lagerziel = Verbrauch je Tag · *max*; liegt der Vorrat unter Verbrauch · *min*,
  steigt das Gebot (M16); liegt er über Verbrauch · `lager_hoch_tage` · *max* /
  `start.lager_eingang_tage`, sinkt es.
- Ein Angebot hält vom eigenen Erzeugnis zurück, was der Standort selbst braucht:
  Verbrauch · *max*.
- Erzeugung: steuert das Fertigwarenlager auf Abgang je Tag · *z* (M16).
- Grenzen: 0 < *min* ≤ *max* ≤ `strategie.lager_tage_max`, 0 ≤ *z* ≤
  `strategie.lager_tage_max`.

**Personal.** Mit Grenzen *a* ≤ *b* (0 ≤ *a*, *b* ≤ `produktionsmodell.lohnaufschlag_max`):
Warten Anlagen auf Arbeitskräfte, steigt der Aufschlag um `lohnaufschlag_schritt` bis *b*
(mindestens auf *a*), sonst sinkt er um den Schritt bis *a*.

**Eigenfertigung oder Zukauf** (Lieferungen, MA1):

- *eigene Ware zuerst*: Fehlt einem Standort ein Vorprodukt bis zum Lagerziel, liefern
  eigene Standorte mit freiem Vorrat, gleiches Land zuerst (wie bisher).
- *nach Preis*: nur Standorte, deren Angebotspreis plus Fracht je Stück zum Empfänger
  den Marktpreis im Land des Empfängers nicht übersteigt.
- *nur Zukauf*: keine Lieferungen; der Einkauf kauft am Markt, die eigenen Standorte
  verkaufen ihre Ware.

**Investitionsbudget.**

- Investition einer Option = angerechneter Betrag (MA0) ihrer Schritte Bauen,
  Erschließen, Standort gründen, Grundstück kaufen und Wiederanfahren.
- Jede Vorgabe begrenzt die Investitionen, die Stellen im Kalenderjahr für Orte ihres
  Geltungsbereichs beschließen (Ort = Standort der Entscheidung; ein neuer Standort zählt
  für sein Land). Es gelten **alle** Budgets, deren Bereich den Ort enthält (z. B. Firma
  und Europa). Rest = Budget − beschlossene Investitionen des Jahres; was der Spieler
  freigibt, zählt nicht (wie MA2). Ändert der Spieler den Betrag, bleibt das Verbrauchte.
- Passt die Investition einer Empfehlung nicht in den kleinsten Rest, entscheidet keine
  Stelle der Kette: Anliegen an den Spieler mit dem Grund `investition` (Weg wie MA3).

**Liquiditätsreserve.** Reserve = Monate · laufende Kosten eines Monats (alle Standorte
bei geplanter Erzeugung, wie die Kassenregel der KI). Eine Stelle beschließt eine
Investition nur, wenn Kasse − Investition ≥ Reserve am Ort; sonst Anliegen mit dem Grund
`reserve`. 0 ≤ Monate ≤ `strategie.liquiditaet_monate_max`.

Gründe eines Anliegens in dieser Reihenfolge: Kredit, Reserve, Investitionsbudget, Budget
0, je Entscheidung, Jahr.

### Befehl

`SetStrategy { scope, field, value }` setzt die Vorgabe eines Felds für einen
Geltungsbereich, `value: null` entfernt sie. Geprüft werden: ein Standort gehört der
Firma, der Wert passt zum Feld und liegt in seinen Grenzen, ein Budget ist nicht negativ.

### Anzeige

Die Sicht `strategy` zeigt die Einheiten wie das Organigramm (Firma, Kontinente, Länder,
Standorte; dazu Länder und Kontinente mit eigener Vorgabe ohne Standort) und je Einheit
und Feld den geltenden Wert, seine **Herkunft** (diese Ebene, Land, Kontinent, Firma oder
Standard) und ob die Einheit eine eigene Vorgabe hat. Beim Investitionsbudget steht der
kleinste Rest mit seinem Geltungsbereich, bei der Reserve der Betrag. An Standorten nennt
sie je Feld die Stelle, die es umsetzt: die erste besetzte Stelle der Kette (MA3) für das
Thema des Felds (Preis – `verkauf`, Lager – `einkauf`, Personal – `lohn`, Eigenfertigung –
`eigenversorgung`, Investition und Reserve – `ausbau`); ohne sie entscheidet der Spieler
selbst.

Darunter stehen die **Verkaufswege** (M8) mit ihren eigenen Geltungsbereichen (Firma,
Land, Produkt, Produkt im Land): je Käufergruppe erlaubt oder gesperrt, Mindestpreis und
Höchstmenge; die allgemeinen Regeln zuerst.

### Spielstände

Neue Felder mit Vorgabewerten: Alte Stände laden ohne Vorgaben, das Format bleibt
lesbar ohne neue Version. Vorschläge der Manager zu Vorgaben folgen mit der
Strategierücksprache (MA5).

## MA5 – Vorstand, CEO, Strategieauftrag und Strategierücksprache

`docs/MANAGER.md` Abschnitte 3, 5, 8, 11 und 12. Daten: `parameter/management.yaml`
(Ebene `vorstand`, Themen der Bereiche, `regel_themen`, `strategieauftrag`). Kern: Module
`management`, `mandate` (Strategieauftrag), `deals` (Angebote des Vorstands) und `review`
(Rücksprache).

### Einheit Vorstand

- Jede Firma mit mindestens einem Standort hat die Einheit **Vorstand**: CEO (Leitung)
  und Ressorts (Fachstellen der Ebene `vorstand`; Vorgabe: Produktion und Technik,
  Einkauf und Logistik, Vertrieb und Marketing, Personal, Forschung und Entwicklung,
  Finanzen – die Bereiche aus MA1).
- **Gehalt:** Faktor der Ebene (Ressort 10, CEO 15) × (0,5 + Stärke / 100) × Jahreslohn
  der Akademiker im Land des Firmensitzes; Gemeinkosten der Firma (wie MA3).
- **Budget:** Bezug = Umsatz der Firma in zwölf Monaten (ohne Umsatz ihre Kosten);
  Anteile Ressort 3 % / 8 %, CEO 8 % / 20 %; Sockel wie MA2. Deckel: Ressorts und
  Kontinentvorstände höchstens das Budget des CEO, sobald er besetzt ist (MA3); für alle
  gilt das Investitionsbudget der Firma (MA4).
- **Prüfrhythmus** des Vorstands: alle `pruefung_tage` (Vorgabe 30) Tage; die
  Rücksprache mit dem Spieler hat ihren eigenen Abstand (unten).
- **Kette:** … → Kontinentvorstand → Ressort des Bereichs → CEO → Spieler. Ohne CEO und
  Ressort entscheidet der Spieler wie bisher.

### Themen des Vorstands

| Thema | Bereich | Regel | Wann |
| --- | --- | --- | --- |
| Überkapazität, stillgelegt, Wiederanfahren, Ausbau, Kraftwerk, Lagerstätte, Werbung | wie MA3 | wie MA3, für Standorte und Länder ohne zuständige Stelle darunter | Quartalsende bzw. Prüftermin |
| `kasse` | Finanzen | Kassenregel der KI: Kredit unter `kasse_min_monate`, Tilgung über `kasse_max_monate` laufender Kosten | Prüftermin |
| `kaufangebot` | Finanzen | während einer Versteigerung (M38) Gebote für Standorte in eigenen Ländern mit Vorteil (Gebot wie die KI am Schluss, höchstens `kasse_anteil_max` der Kasse, mindestens das Mindestgebot), dann das beste Geschäft nach der Regel der KI (M30) – ohne ihren Zufall –; jeweils solange die Firma weniger als `offen_max` offene Angebote hat | Prüftermin |
| `antwort` (neu) | Finanzen | Antwort auf Kaufangebote an die Firma nach der Regel der KI: annehmen ab der Mindestforderung, Gegenangebot ab `gegen_schwelle`, sonst ablehnen | Prüftermin (Angebote gelten `gueltig_monate`) |
| `engpass` | Produktion | neue Kette oder neues Land: Gelegenheit der KI-Regel (M32/M33) für die eigene Firma | nur bei der Rücksprache, als Antrag |

- **Optionen einer Antwort:** die Antwort der Regel zuerst, dann die übrigen (Annehmen;
  Gegenangebot zur Mindestforderung, wenn sie über dem Preis liegt und noch keines
  gemacht ist; Ablehnen), zuletzt „So lassen“ (das Angebot bleibt offen). Ein Anliegen zu
  einem Angebot, das inzwischen geschlossen ist, gilt als erledigt; ebenso, wenn der
  Spieler das Angebot selbst beantwortet.
- **Meldungen:** Was der Vorstand selbst entscheidet (Gebot, Verkauf, Kauf, Ablehnung,
  Gegenangebot), meldet er im Rundenbericht. Solange eine Stelle des Vorstands Antworten
  übernimmt, hält ein eingehendes Angebot mehrere Runden am Stück nicht mehr an; was
  über ihr Budget geht, kommt als Anliegen (das hält wie jedes wichtige Anliegen).
- **Regelthemen** (`regel_themen`, Vorgabe `kasse`, `werbung`): Ihre Wirkung zählt nur
  Kosten (Zins, Mehrbudget) – den Nutzen (Zahlungsfähigkeit, Mehrabsatz) schätzt die
  Bewertung nicht. Wie bei der Routine folgt die Stelle deshalb der Option der Regeln;
  Anliegen dazu gelten aber als wichtig. (Bis MA4 hätte eine urteilsstarke Stelle so nie
  Werbung erhöht und nie einen Kredit aufgenommen.)
- **Angerechnete Beträge** (MA0) neu: Annahme eines Kaufangebots für eigene Standorte –
  deren Buchwert (Abgabe von Vermögen, bei einem Bereich aller seiner Standorte, bei einer
  Lizenz nichts); Annahme eines Gegenangebots als Käufer – der Preis; ein Gebot
  (`MakeOffer`) – der Preis; Gegenangebot und Ablehnung – nichts.
- **Kredite** (`TakeLoan`, `RepayLoan`) entscheiden nur das Ressort des Bereichs, der die
  Kasse führt (Finanzen), und der CEO; für alle anderen Stellen bleibt der Grund `kredit`
  (MA2). Kredite allein gehören zum Vorstand (Ort der Entscheidung). Nach dem Kredit
  dürfen die Kredite höchstens die Grenze des Strategieauftrags betragen:
  (K + m) / (A + m) ≤ v (K Kredite, A Bilanzsumme, m neuer Kredit, v Grenze), sonst
  Anliegen mit dem Grund `verschuldung`; das Anliegen zeigt den Spielraum
  m_max = (v · A − K) / (1 − v).
- **Sperren** des Strategieauftrags: Keine Stelle gründet Standorte in gesperrten Ländern,
  baut (Ausbau, Kraftwerk, Lagerstätte, Engpass) für Produkte gesperrter Warengruppen
  oder bietet für Standorte in gesperrten Ländern oder mit Produkten gesperrter
  Warengruppen und für Bereiche gesperrter Warengruppen (oder mit einem Standort in einem
  gesperrten Land). Solche Entscheidungen hält die Stelle zurück, ohne zu fragen;
  Gelegenheiten dort entfallen. Der Spieler selbst darf weiter alles.

### Strategieauftrag

Je Firma (`SetMandate`), Standard „Ertrag“, ohne Ziele und Sperren, Rücksprache
quartalsweise:

- **Leitlinie:** Wachstum, Ertrag, Sicherheit oder Marktführerschaft in einer
  Warengruppe. Sie gibt den Regeln, nach denen die Stellen der Firma entscheiden, ihre
  **Aggressivität** (`strategieauftrag.leitlinien`; Vorgabe 0,8 / 0,5 / 0,2 / 0,8) – wie
  bei KI-Firmen: Preisuntergrenze „Marktpreis“, Schwellen für Ausbau, Werbeanteil,
  Gebotsaufschlag. Bei Marktführerschaft wählt der CEO Gelegenheiten und Kaufangebote in
  der Warengruppe zuerst.
- **Ziele** (je optional): Umsatzwachstum in % je Jahr, Umsatzrendite in %,
  Eigenkapitalquote in %, Rang nach Umsatz.
- **Grenzen:** höchste Verschuldung (Kredite / Bilanzsumme, optional), gesperrte Länder,
  gesperrte Warengruppen. Investitionsbudget und Liquiditätsreserve sind die Vorgaben der
  Firma aus MA4.
- **Rücksprache:** monatlich, quartalsweise, halbjährlich oder jährlich.

### Strategierücksprache

- **Fällig**, wenn der CEO besetzt ist, nach dem Monatsabschluss am Ende jedes Abschnitts
  (Monat, Quartal, Halbjahr, Jahr – Kalenderabschnitte). Mehrere Runden am Stück halten
  danach immer an (Halt „ruecksprache“).
- **Bericht** über den abgeschlossenen Abschnitt, aus dem Hauptbuch wie das Controlling:
  - Umsatz und Ergebnis der Firma, je Kontinent (Kostenstellen der Standorte des
    Kontinents) und je Warengruppe (Kostenstellen der Produkte der Gruppe: Umsatz und
    Marge); Gemeinkosten der Firma getrennt.
  - Ziele gegen Ist: Umsatzwachstum = Umsatz der letzten 12 abgeschlossenen Monate / der
    12 davor − 1 (ohne Vorjahr kein Wert); Umsatzrendite = Ergebnis / Umsatz der letzten
    12 Monate; Eigenkapitalquote = Eigenkapital / Bilanzsumme heute; Rang nach Umsatz
    (M29). Je Ziel „erreicht“ oder „verfehlt“.
  - **Chancen:** die Produkte mit der höchsten Marge je Umsatz im Abschnitt (bis
    `chancen_risiken`) und die Anträge. **Risiken:** Standorte mit Verlust im Abschnitt
    (größter zuerst), verfehlte Ziele, Kasse unter der Liquiditätsreserve, Verschuldung
    über der Grenze.
- **Anträge:** Der CEO prüft die strategischen Themen (`engpass`, dazu `kaufangebot`) und
  legt bis zu `antraege_max` als Anliegen der Rücksprache vor (Grund `antrag`, wichtig),
  mit seiner Empfehlung (MA2), unabhängig vom Budget. Der Spieler genehmigt (Option
  wählen), überlässt dem CEO die Entscheidung, lehnt ab oder ändert den Auftrag.
- Zwischen den Rücksprachen meldet der CEO nur, was über sein Budget geht (Anliegen, wie
  MA3), und Kaufangebote an die Firma, die er nicht selbst beantworten darf; drohende
  Zahlungsunfähigkeit meldet die Kasse wie bisher (M5).
- Gespeichert bleiben die letzten `ruecksprachen_behalten` Rücksprachen.
- **Umsetzung:** Die Rücksprache folgt dem Monatsabschluss und dem Rang (M29), wenn der
  abgeschlossene Monat einen Abschnitt beendet (Monat m mit m mod Länge = 0). Der Bericht
  nimmt die letzten Länge abgeschlossenen Monate des Hauptbuchs (weniger, wenn die Firma
  jünger ist; „von“ ist der Anfang des ersten): Umsatz und Ergebnis der Firma, je Kontinent
  Umsatz der Standorte und Ergebnis ihrer Kostenstellen, Gemeinkosten = Ergebnis − Summe der
  Standorte, je Warengruppe Umsatz und Marge der Produkt-Kostenstellen. Dafür führt jeder
  Monat neben dem Umsatz je Standort (MA2) den **Umsatz je Produkt** (nur Aufzeichnung, kein
  Einfluss auf die Simulation). Umsatz und Marge je Warengruppe stimmen mit dem Rundenbericht
  desselben Zeitraums überein.
- **Chancen** nur mit Umsatz und Marge > 0, nach Marge / Umsatz, bei Gleichstand das
  Produkt mit der kleineren Nummer; **Risiken** zuerst die größten Verluste, dann
  verfehlte Ziele, Reserve (Liquiditätsreserve der Firma aus MA4 · laufende Kosten),
  Verschuldung.
- **Anträge:** das beste Kaufangebot nach `best_deal` mit den Sperren und der
  Marktführerschaft des Auftrags (ein Platz, wenn es eines gibt), die übrigen Plätze die
  Gelegenheiten der Diversifizierung (M32/M33) für die eigene Firma: je Produkt die
  Gelegenheit samt Engpass darunter, finanzierbar mit (Kasse + Kreditrahmen) ·
  `ausbau_anteil_kasse_max`, gesperrte Länder und Warengruppen übersprungen; bei
  Marktführerschaft zuerst die Ketten der Warengruppe. Kein Antrag, wenn zu Thema, Ort und
  Produkt schon ein Anliegen offen ist. Empfehlung wie MA2 mit dem Urteilsvermögen und der
  Fachkompetenz des CEO (Leitungsabschlag), Zufall aus dem Strom der Rücksprache (Firma,
  Monat).
- **Neue Standorte in späteren Antworten:** Gründet eine Option einen Standort, bekommt er
  beim Ausführen die nächste freie Nummer; die folgenden Schritte (Lagerstätte, Anlage,
  Produktion, Verkauf) werden auf ihn umgestellt. (Vorher konnte ein inzwischen von einer
  anderen Firma gegründeter Standort die geplante Nummer tragen – dann schlug der Bau fehl.)

### Ressort Personal

Die angezeigte Stufe einer Fähigkeit (MA1) nimmt den Eindruck nur zum Teil: Eindruck ·
(1 − `personal_schaerfe` · Fachkompetenz Personal des Personalressorts / 100); ohne
Ressort ganz.

### Spielstände und KI

Neue Felder mit Vorgabewerten (Auftrag, Rücksprachen, Bezug der Anliegen); alte Stände
laden ohne neue Version. KI-Firmen haben keinen Vorstand (bis MA6); ihr Verhalten bleibt
bitgleich.

## MA6 – Lebendiger Managermarkt

`docs/MANAGER.md` Abschnitte 0 (Punkt 3), 3 („Leitung stellt ein“), 4.1–4.3 und 11. Daten:
`parameter/management.yaml`, Block `markt`. Kern: Modul `staffing`. Alles am
**Monatsanfang** nach dem Auffüllen des Bewerberpools, in fester Reihenfolge: Erfahrung,
Zufriedenheit, Kündigungen, Einstellungen der Leitungen, dann je KI-Firma Gehälter,
Einstellungen und Abwerbungen, zuletzt die Kompetenz der KI-Firmen. Zufall der Manager aus
ihrem Monatsstrom (`Stream::ManagerMonth`, je Manager und Monat), Entscheidungen der KI aus
dem Monatsstrom der Firma (`Stream::Staffing`).

### Erfahrung

- Jeder Manager hat eine persönliche **Obergrenze** *P* der Fachkompetenz: seine höchste
  Fachkompetenz bei der Ziehung + gleichverteilt 0 … `erfahrung.spielraum`, höchstens 100.
  Sie stammt aus einem eigenen Strom je Manager-Nummer (`Stream::ManagerPotential`,
  verschiebt keine Ziehung des Markts) und wird am Monatsanfang nach dem Auffüllen
  gesetzt; Manager älterer Spielstände bekommen sie dort ebenso.
- Wer eine Stelle hat, gewinnt jeden Monat mit der Wahrscheinlichkeit
  `erfahrung.chance_monat` einen Punkt Fachkompetenz im **Bereich der Stelle** (Leitung:
  ihr Schwerpunkt), solange er unter *P* liegt. Mit 0,3 sind das im Mittel 3,6 Punkte im
  Jahr; Gehaltsforderung und Marktwert steigen mit.

### Zufriedenheit

- 0–100, beim Einstellen und nach einem Gegenangebot `zufriedenheit.start`; Manager
  älterer Spielstände beginnen dort. Am Monatsanfang für jeden Manager mit Stelle:

      Ziel Z = basis + gehalt_gewicht · 100 · (Gehalt / Marktwert − 1)
               − verlust_abzug   (wenn die Einheit in zwölf Monaten Verlust machte)
               − uebergangen_abzug · Ü
      Zufriedenheit ← round(Z_alt + anpassung · (Z − Z_alt)), auf 0–100 begrenzt

  Marktwert = Gehaltsforderung für seine Stelle heute (MA1, mit den heutigen Fähigkeiten
  und Löhnen; ohne Marktwert entfällt der Gehaltsteil); Ergebnis der Einheit = Summe der
  Monatsergebnisse ihrer Standorte in den letzten zwölf abgeschlossenen Monaten (MA2,
  Vorstand: alle Standorte); Ü = Anliegen dieses Managers, die in den letzten zwölf
  Monaten geschlossen wurden und bei denen der Spieler eine andere als die empfohlene
  Option gewählt oder das Thema abgelehnt hat.
- Weil die Löhne mit der Zeit steigen, fällt ein festes Gehalt hinter den Marktwert
  zurück. **Gehalt anpassen** (`RaiseSalary`): Die Firma setzt das Gehalt eines ihrer
  Manager herauf (nur erhöhen); die Oberfläche schlägt den Marktwert vor.
- Anzeige als Stufe („unzufrieden“ unter `zufriedenheit.stufen[0]`, „gemischt“ unter
  `stufen[1]`, sonst „zufrieden“), ohne Unschärfe, mit dem Marktwert.

### Kündigung

- Liegt die Zufriedenheit unter `kuendigung.schwelle` *S*, kündigt der Manager mit der
  Wahrscheinlichkeit `kuendigung.chance_max` · (*S* − Zufriedenheit) / *S*. Gehalt bis
  zum Tag, keine Abfindung; er kehrt in den Bewerberpool seines Kontinents zurück.
  Meldung an den Spieler, wenn es einer seiner Manager ist.

### Leitung stellt ein

- Schalter je Leitung (Befehl `SetHiringByHead`). Eine besetzte Leitung mit Schalter
  besetzt am Monatsanfang höchstens eine freie Fachstelle ihrer Einheit, die Themen oder
  eine Wirkung hat (MA5), in der Reihenfolge der Fachstellen.
- Wahl: freie Bewerber aus dem Kontinent des Sitzlands der Einheit mit dem Schwerpunkt
  des Bereichs, sonst alle des Kontinents; mit der Wahrscheinlichkeit `empfehlung_grund`
  + (1 − `empfehlung_grund`) · Urteilsvermögen / 100 der mit der höchsten Fachkompetenz
  im Bereich, sonst der mit der höchsten angezeigten Stufe darin (Eindruck der Firma).
  Nur wenn das Jahresgehalt in den Rest ihres Jahresbudgets passt; es zählt darauf.
  Meldung an den Spieler.

### KI-Firmen

KI-Firmen nutzen denselben Pool und dieselben Befehle (`HireManager`, `RaiseSalary`,
`PoachManager`, `MatchOffer`, `LetGo`). Ihre Entscheidungen bleiben die Regeln der KI;
Manager wirken über ihre **Kompetenz** (MANAGER.md §0.3, keine Sonderregeln). Sie haben
keine Budgets, Anliegen und Strategien: Ihre Manager entscheiden nicht über die
Stellen-Kette des Spielers.

- **Gehälter:** Liegt die Zufriedenheit eines ihrer Manager unter `zufriedenheit.stufen[1]`
  und sein Gehalt unter dem Marktwert, hebt sie es auf den Marktwert, wenn ihre Kasse das
  Jahresgehalt deckt.
- **Wen:** je Monat höchstens `ki.einstellungen_monat` Stellen: zuerst den CEO, wenn der
  Umsatz der Firma in zwölf Monaten ≥ `ki.umsatz_ceo_usd`, dann die Leitungen ihrer
  Standorte mit dem größten Umsatz in zwölf Monaten ≥ `ki.umsatz_standort_usd`. Nur wenn
  die Gehaltsforderung ≤ `ki.gehalt_anteil` · Umsatz der Einheit in zwölf Monaten und die
  Kasse das Jahresgehalt deckt.
- **Wahl:** Stärke wie MA1. Der stärkste freie Bewerber aus dem Kontinent des Sitzlands
  mit der Wahrscheinlichkeit der Kompetenz, sonst einer der drei stärksten
  (gleichverteilt).
- **Wirkung:** Kompetenz = Grundkompetenz + `ki.kompetenz_ceo` · (Stärke CEO − 50) / 50
  + `ki.kompetenz_leitung` · Anteil der Standorte mit Leitung · (mittlere Stärke der
  Leitungen − 50) / 50, auf 0–1 begrenzt; neu berechnet am Monatsanfang. Sie bestimmt wie
  bisher, wie oft die KI ihren Betrieb prüft und wie weit sie in der Forschung vorausplant.
  Ein starker CEO hebt sie, ein schwacher senkt sie. Die Gehälter sind Personalkosten wie
  beim Spieler.

### Abwerbung

- Sucht eine KI-Firma eine Stelle zu besetzen, prüft sie neben dem Pool die Manager
  anderer Firmen aus demselben Kontinent mit Stärke ≥ `abwerbung.staerke_min`, die in
  diesem Monat noch kein Angebot haben. Ist der stärkste davon um mindestens
  `abwerbung.vorsprung` stärker als der Bewerber, den sie nehmen würde, bietet sie ihm
  (`PoachManager`) das Höhere aus seiner Forderung für ihre Stelle und seinem Gehalt ·
  (1 + `abwerbung.aufschlag`) – nur wenn dieser Betrag die Grenzen fürs Einstellen
  einhält. Die Stelle bleibt frei, solange das Angebot gilt.
- **Manager des Spielers:** ein Anliegen (Thema `abwerbung`, wichtig) mit Frist
  `anliegen.frist_tage`. Es stellt die Personal-Stelle seiner Einheit (der Bereich mit dem
  Thema `abwerbung`), sonst die nächste besetzte Personal-Stelle darüber (Land, Kontinent,
  Vorstand), sonst der Manager selbst. Optionen: **Gegenangebot** (`MatchOffer`: Gehalt
  auf das Angebot, Zufriedenheit auf `zufriedenheit.start`; zählt die Erhöhung im Jahr)
  oder **Gehen lassen** (`LetGo`: Er wechselt zur KI-Firma, Gehalt bis zum Tag, keine
  Abfindung). Empfohlen wird, was eine KI-Firma täte (unten). Unbeantwortet bis zur Frist,
  abgelehnt oder bei einem Thema, das der Spieler ruhen lässt, bleibt alles, wie es ist
  (MANAGER.md §0.4): Das Angebot verfällt, die Zufriedenheit sinkt um
  `abwerbung.ignoriert_abzug`. Bei „Entscheide selbst“ setzt die Stelle ihre Empfehlung
  um, wenn die Erhöhung in ihr Budget passt.
- **Manager einer KI-Firma:** Sie hält ihn sofort, wenn das Angebot ≤
  `abwerbung.ki_gegen_max` · sein Gehalt ist und ihre Kasse das Jahresgehalt deckt
  (Gegenangebot), sonst lässt sie ihn gehen.
- Eine Firma macht je Monat höchstens ein Angebot; ein Manager hat höchstens ein offenes
  Angebot und bekommt nach einem Angebot `abwerbung.sperre_monate` Monate lang keines
  (sonst stiege sein Gehalt mit jedem gehaltenen Angebot Monat für Monat).

### Spielstände und Leistung

- Neue Felder mit Vorgaben (Obergrenze, Zufriedenheit, Schalter, offene Angebote,
  Kompetenz durch Manager); alte Stände laden ohne neue Version.
- Die KI ändert sich gewollt (Gehälter, Kompetenz). Mit `ki.einstellungen_monat: 0`
  bleibt sie bitgleich zu MA5. Benchmark: Weltlauf mit 1 000 KI-Firmen ohne deutlichen
  Leistungsverlust gegenüber MA5.

## ZA1 – Hauptsitz

`docs/BETEILIGUNGEN.md` Abschnitt 3. Daten: `parameter/zentrale.yaml`, Block `hauptsitz`.
Kern: Modul `central`.

- Der Firmensitz (`Company::headquarters`) ist der **Hauptsitz**: beim neuen Spiel das
  Startland. Er bestimmt schon heute die Gewinnsteuer (M6), das Lohnniveau der Gehälter
  des Vorstands (MA5) und das Land, in dem die Firma für Kaufangebote präsent ist (M30).
  Mit ZA2 kommen die Angestellten der Zentralabteilungen dazu (Lohn des Landes).
- **Verlegen** (`SetHeadquarters { country }`): in ein anderes Land, nicht während eines
  Umzugs. Kosten sofort, als sonstiger Aufwand der Firma (Gemeinkosten):

      Kosten = kosten_grund_usd + kosten_je_angestelltem_usd · Angestellte der Zentrale

  Die Kasse muss sie decken. Der Umzug dauert `verlegung_monate`; am ersten
  Monatsanfang ab diesem Datum gilt der neue Sitz. Von den Angestellten der
  Zentralabteilungen ziehen `mitziehen` mit (abgerundet je Abteilung); die übrigen
  scheiden aus, ohne Abfindung. Meldung an den Spieler.
- Kriegsrisiken des Sitzlands (Ausfall, Beschlagnahme) kommen mit Stufe 4; die Daten
  sehen sie noch nicht vor.
- KI-Firmen verlegen ihren Sitz seit ZA4 nach Steuern und Löhnen (Abschnitt ZA4).

## ZA2 – Zentralabteilungen

`docs/BETEILIGUNGEN.md` Abschnitte 4.1, 4.2 und 6. Daten: `parameter/zentrale.yaml`, Liste
`abteilungen` und `genauigkeit`; in `parameter/management.yaml` die neuen Bereiche
`strategie` (Themen `kaufangebot`, `antwort`) und `recht` (Thema `lizenz`: Lizenzen kaufen
und verkaufen) als Ressorts des Vorstands (gleiche Gehaltsfaktoren). Kern: Modul `central`.

### Aufbau und Kosten

- Abteilungen mit fester Aufgabe im Kern: `strategie`, `finanzen`, `personal`, `recht`,
  `marketing`; die Liste in den Daten wählt aus, welche es gibt. Je Abteilung: `bereich`
  (das Ressort des Vorstands, dessen Manager die **Leitung** ist), `lohngruppe` der
  Angestellten, `buero_usd` (Bürokosten je Angestelltem und Jahr), `faelle` (was ein
  Angestellter im Monat bearbeitet) und `wirkung`.
- **Angestellte** (`StaffDepartment { department, staff }`): eine Zahl je Abteilung, keine
  Einzelpersonen, im Land des Hauptsitzes, sofort eingestellt und entlassen (ohne
  Abfindung). Kosten je Monat, am Monatsende als Kosten des Hauptsitzes (ohne Standort)
  gebucht:

      Personal (Personalkosten) = Angestellte · Stundenlohn(lohngruppe, Sitzland) · Jahresstunden / 12
      Büro (Gemeinkosten)       = Angestellte · buero_usd / 12

  Eine kleine Abteilung kostet samt Ressortleitung rund eine halbe Million USD im Jahr –
  eine Werkstatt trägt das nicht: Mit je einem Angestellten in Finanzen und Marketing
  (1900, Deutschland, rund 8 000 USD im Monat) ist sie nach einem Jahr zahlungsunfähig.
- Ohne Leitung arbeitet eine Abteilung nicht (die Angestellten kosten trotzdem). Beim
  Umzug des Hauptsitzes bleiben je Abteilung ⌊Angestellte · mitziehen⌋; Meldung, wie viele
  nicht mitgezogen sind.
- KI-Firmen richten seit ZA4 Zentralabteilungen nach ihrem Umsatz ein (Abschnitt ZA4);
  ihre Manager wirken außerdem über die Kompetenz (MA6).

### Leistung

Für eine Abteilung mit Leitung *L* und *n* Angestellten:

    Kapazität K = n · faelle                          (Fälle je Monat)
    Güte      G = Fachkompetenz(L, bereich) / 100
    Abdeckung A = min(1, K / Arbeitslast)
    Stärke    W = wirkung · G · A

| Abteilung | Arbeitslast | Wirkung |
| --- | --- | --- |
| Strategie | 1 | beobachtet ⌊K⌋ Länder ohne eigenen Standort und außer dem Sitzland, die größten Volkswirtschaften (Einwohner · BIP je Kopf): Dort sucht der Vorstand Übernahmeziele mit (MA5) |
| Finanzen | Kredite + 1 | Risikoaufschlag neuer Kredite × (1 − W); Umschuldung (ZA3) |
| Personal | Manager der Firma | Schulung: Chance auf Erfahrung je Monat × (1 + W), höchstens 1 (MA6); Gehaltsrunde (ZA3) |
| Recht | 1 | prüft Lizenzen auch für Technologien, die Rezepte der eigenen Warengruppen brauchen (was die Firma herstellt oder anbietet), die sie weder kennt noch erforscht und die eine andere Firma lizenzieren könnte: die ⌊K⌋ mit dem höchsten Lizenzwert (M30) |
| Marketing | Werbebudgets + 1 | Werbewirkung × (1 + W) (M16) |

- **Fälle je Prüfung des Vorstands:** Ohne Abteilung bietet der Vorstand wie in MA5 je
  Prüfung das beste Übernahmeziel (Ressort Strategie) und die beste Lizenz (Ressort Recht)
  an; mit arbeitender Abteilung bis zu ⌊K⌋ der besten (mindestens eins), solange die Firma
  weniger offene Angebote hat, als die KI darf.
- **Genauigkeit** der Leitung bei Entscheidungen ihres Ressorts (die Vorstandsfachstelle
  der Abteilung):

      Schätzfehler    ← Schätzfehler · (1 − genauigkeit · A)
      Urteilsvermögen ← U + (100 − U) · genauigkeit · A

### Vorgabe „Beteiligungen“

Für die ganze Firma (Befehl `SetParticipations { budget, risk, limits }`), in der
Strategieansicht:

- **Budget je Jahr** für Übernahmen und Lizenzen. Es zählen die Käufe des Jahres (auch die
  des Spielers selbst und Zuschläge in Versteigerungen) und die offenen Gebote der Firma.
  Übersteigt ein Gebot einer Stelle den Rest, entscheidet keine Stelle selbst: Anliegen
  mit dem Grund „Beteiligungsbudget“. Ohne Angabe: keine Grenze. (Ab SU1 auch Start-ups.)
- **Risikobereitschaft** 0–1 (Vorgabe 0,5): ab SU1 die Mindestchance der Start-ups, die
  die Strategie empfiehlt.
- **Freigabegrenze je Abteilung:** Bis zu diesem Betrag entscheidet die Leitung (die
  Vorstandsfachstelle des Ressorts) selbst; die Grenze senkt ihr Budget je Entscheidung
  (MA2), darüber geht die Entscheidung den Weg der Stellen (CEO) und sonst als Anliegen mit
  dem Grund „Freigabegrenze“ an den Spieler. Ohne Angabe gilt das Budget allein.

## ZA3 – Empfehlungen und Trefferquote

`docs/BETEILIGUNGEN.md` Abschnitte 4.3 und 4.4. Daten: `parameter/zentrale.yaml`, Blöcke
`umschuldung` und `trefferquote`; Themen `umschuldung` (Finanzen) und `gehaltsrunde`
(Personal) auf der Ebene Vorstand. Kern: Module `central`, `management` (Vorstand),
`deals` (Angebote des Vorstands).

### Empfehlungen

Am Prüftermin des Vorstands bearbeiten die arbeitenden Abteilungen (mit Leitung und
Angestellten) bis zu ⌊K⌋ Fälle. Jeder Fall ist eine Entscheidung (MA0) mit „umsetzen“ und
„so lassen“: im Budget und unter der Freigabegrenze setzt die Stelle ihn um, sonst wird er
eine **Empfehlung** (ein Anliegen mit Begründung).

| Abteilung | Fall | Option | Begründung |
| --- | --- | --- | --- |
| Finanzen | Kredit, dessen Zins mindestens `umschuldung.mindestvorteil` über dem Zins eines neuen Kredits liegt (gleiche Schulden, Risikoaufschlag mit der Ersparnis von ZA2); die teuersten zuerst | `RefinanceLoan`: der neue Zins für Restschuld und Restlaufzeit; Gebühr `umschuldung.gebuehr` der Restschuld als Zinsaufwand. Wirkung: Restschuld · Zinsvorteil im Jahr, einmalig die Gebühr | ersparte Zinsen im Jahr und die Gebühr |
| Personal | Manager der Firma mit Zufriedenheit unter `zufriedenheit.stufen[1]` und Gehalt unter dem Marktwert, die unzufriedensten zuerst | `RaiseSalary` auf den Marktwert (Regelthema: die Stelle folgt der Regel) | Abstand zum Marktwert, drohende Kündigung |
| Strategie | das beste Übernahmeziel (MA5) in den eigenen und den beobachteten Ländern | Kaufangebot zum geschätzten Preis | Passung (eigene Warengruppe) oder Streuung (neue Warengruppe) |
| Recht | die beste Lizenz für Forschungsziele und geprüfte Technologien (ZA2) | Kaufangebot zum geschätzten Preis | Abkürzung: ersparte Forschung (Lizenzwert, M30) |

- Ohne arbeitende Abteilung gibt es keine Umschuldungen und Gehaltsrunden; Übernahmen und
  Lizenzen bietet der Vorstand wie in MA5 an (je eine je Prüfung).
- Werden Kredite getilgt oder laufen aus, verschieben sich ihre Nummern: offene
  Umschuldungs-Anliegen gelten dann als erledigt und kommen bei der nächsten Prüfung neu.

### Schätzung von Strategie und Recht

Statt des Preises der Regeln (MA5) schätzt die Stelle, die das Thema bearbeitet – die
Vorstandsfachstelle des Ressorts, sonst der CEO mit dem Abschlag der Leitung –, den Preis
*P* eines Ziels:

    Angebot = P · f,  f = 1 + (2u − 1) · e
    e = Schätzfehler · (1 − Fachkompetenz/100) · (1 − genauigkeit · A)

(*u* gleichverteilt aus dem Tagesstrom der Stelle, A die Abdeckung ihrer Abteilung, ohne
Abteilung 0). Schwache Leitungen überzahlen öfter (Fehlgriff) und bieten öfter zu wenig
(übersehene Chance). Mit Schätzfehler 0,5: Fachkompetenz 20 streut ± 40 %, 90 nur ± 5 %.

### Trefferquote

- Jede Schätzung eines Ziels wird nach `bewertung_monate` bewertet (am ersten
  Monatsanfang danach): **Treffer**, wenn das Angebot den Wert *V* nach den Regeln am Tag
  der Schätzung (was die Regeln höchstens zahlen würden) nicht übersteigt. Seit ZA4
  zählt bei Übernahmen der Wert am Tag der Bewertung, und Start-up-Empfehlungen werden
  ebenfalls bewertet (Abschnitt ZA4). Ob die Stelle
  es abgibt, der Spieler annimmt oder ablehnt, zählt nicht. Umschuldung und Gehaltsrunde
  können nicht danebenliegen und zählen nicht.
- Trefferquote mit Vorgewicht (wenige Fälle täuschen nicht):

      q = (Treffer + mittelwert · vorgewicht) / (bewertet + vorgewicht)

  Sichtbar bei den Fähigkeiten jedes Managers (Managermarkt, Organisation) und bei den
  Leitungen der Zentralabteilungen, mit der Zahl der bewerteten Fälle.
- **Gehaltsforderung** (MA1) mal `e^(k · (q − mittelwert))`, vor der ersten Bewertung 1;
  mit k = 2 fordert eine Leitung mit q = 0,9 das 2,2-Fache, mit q = 0,2 das 0,55-Fache.
  Der Marktwert (MA6) folgt der Forderung: Eine erfolgreiche Leitung wird unzufrieden,
  wenn ihr Gehalt nicht mitwächst.
- **Abwerbung und Einstellung** (MA6): Die KI vergleicht Stärke mal demselben Faktor;
  Leitungen mit hoher Trefferquote werden häufiger umworben. KI-Firmen schätzen nach ihren
  Regeln; ihre Manager werden nicht bewertet.


## SU1 – Start-ups

`docs/BETEILIGUNGEN.md` Abschnitte 5.1–5.3 und 5.6. Daten: `parameter/startups.yaml`
(Abschnitt `startups`) und `startups/erfinder.yaml` (Abschnitt `erfinder`). Kern: Modul
`ventures`. Zufall aus eigenen Strömen: `Stream::Ventures { month }` für die Entstehung,
`Stream::Venture { id, month }` je Start-up; alles am **Monatsanfang**, nach den Managern
und vor den Märkten: erst die laufenden Start-ups, dann neue, dann wird die Liste
gekürzt.

### Bezeichnung je Epoche

`bezeichnungen`: Liste von `{id, ab}` (Jahr); es gilt die letzte, deren `ab` erreicht ist:
1900 „Erfinder und Gründungen“, ab 1970 „Wagniskapital“, ab 1990 „Start-ups“ (Texte
`startup.bezeichnung.<id>`). Im Kern heißt alles `Venture`.

### Entstehung

- Je Monat entstehen im Mittel `je_jahr · Faktor / 12` Start-ups: ⌊Mittel⌋ sicher, eines
  mehr mit der Wahrscheinlichkeit des Rests. Den Faktor wählt der Spieler beim neuen
  Spiel (`haeufigkeiten`: keine 0, wenige 0,5, normal 1, viele 2; Vorgabe
  `haeufigkeit_standard`); er steht in `GameSettings::ventures`.
- **Art:** mit der Wahrscheinlichkeit `anteil_neu` eine **neue Technologie**, sonst eine
  **Verbesserung**; gibt es keine passende neue Technologie, eine Verbesserung (und
  umgekehrt); gibt es keins von beiden, entsteht keins.
  - Neue Technologie: weder historisch (Jahr der Erfindung ≤ laufendes Jahr) noch im
    Spiel erfunden, mit Forschungsaufwand, alle Voraussetzungen erfunden, historisches
    Jahr höchstens `vorlauf_jahre_max` voraus und noch nicht Ziel eines laufenden
    Start-ups; gleichverteilt. Vorlauf *v* = historisches Jahr − Jahr (≥ 1). Ab 2026
    gibt es keine mehr (die Daten enden dort).
  - Verbesserung: ein Produkt, das sich weiterentwickeln lässt (M37) und das eine
    Anlage einer Firma, die nicht pleite ist, heute herstellt, dessen nächste Stufe noch
    niemand erreicht hat und an dem kein laufendes Start-up arbeitet; Ziel ist die Stufe
    nach der weltweit höchsten (`GameState::developments`); v = 0.
- **Land:** gewichtet mit Einwohnern · BIP je Kopf · Entwicklungsstand (0–1): Erfinder
  sitzen dort, wo Wirtschaft und Bildung sind.
- **Name:** Gibt es für die Technologie einen historischen Erfinder (`erfinder`), der noch
  kein Start-up hatte, heißt es nach ihm und sitzt in seinem Land; sonst ein Gründer aus
  der Namensliste des Landes (Vor- und Familienname wie bei den Managern).
- Eigner am Anfang: die Gründer (`Holder::Private`) mit 100 %.

### Phasen und Finanzierung

- Drei Phasen (`phasen`: Idee 12 Monate, Prototyp 18, Marktreife 24) mit Kapitalbedarf,
  Chance und Bewertungsfaktor. Der Bedarf folgt dem Einkommen im Land; neue Technologien
  sind mit dem Vorlauf teurer und riskanter:

      Kapital = kapital_usd · clamp(BIP je Kopf / bezug_bip_je_kopf_usd,
                                    kapital_faktor_min, kapital_faktor_max)
                · (1 + vorlauf_kapital · v)
      Chance  = max(chance_min, chance · (1 − vorlauf_chance · v))

  Kapital und Chance werden zu Beginn der Phase festgehalten.
- **Finanzierungsrunde:** Zu Beginn jeder Phase ist eine Runde über den Kapitalbedarf
  offen. Investoren außerhalb des Spiels (`Holder::Investors`) finanzieren sie an jedem
  folgenden Monatsanfang mit der Wahrscheinlichkeit `investoren_chance_monat` ganz (ab
  SU2 auch Spieler und Firmen). Ist sie nach `frist_monate` nicht gedeckt, geht das
  Start-up ein („kein Geld“).
- Die Runde gibt neue Anteile aus: Bewertung vor der Runde B = Kapital · `bewertung`; die
  Geber erhalten Betrag / (B + Kapital), alle bisherigen Eigner behalten B / (B + Kapital)
  ihres Anteils. Mit den Daten (10, 5, 2,5) halten die Gründer nach drei Runden 54 %; die
  Bewertungen sind so gewählt, dass ein Einstieg (SU2) im Mittel früh rund das 2,8-Fache,
  spät rund das 1,2-Fache bringt (siehe SU2).
- Ist die Phase finanziert, entscheidet nach ihrer Dauer ein Zug mit der Chance über die
  nächste Phase oder das Scheitern. Alle drei bestanden: **Erfolg**.
- **Überholt:** Ist das Ziel inzwischen in der Welt – die Technologie historisch oder im
  Spiel erfunden, die Stufe von einer Firma erreicht –, geht das Start-up am nächsten
  Monatsanfang ein („überholt“). Neue Technologien mit wenig Vorlauf schaffen es daher
  selten: Alle Phasen dauern mit den Runden gut fünf Jahre.
- Richtwert: insgesamt scheitern 60–70 %. Mit 0,65 · 0,8 · 0,9 ≈ 0,47 und einer
  Finanzierung je Phase von 1 − 0,6⁶ ≈ 95 % bestehen Verbesserungen zu rund 40 %; neue
  Technologien seltener (Vorlauf, Überholen). Weltlauf: siehe `docs/FORTSCHRITT.md`.

### Erfolg und Scheitern

- **Neue Technologie:** Sie gilt ab diesem Tag als erfunden (`GameState::inventions`),
  wenn das früher ist als bisher: Nachforschen wird für alle günstiger (M9, Abschlag für
  Nachzügler ab der Erfindung), und ihre Produkte sind ab dann verfügbar (M33). Meldung
  an den Spieler. Das Patent liegt beim Start-up.
- **Verbesserung:** Die Stufe gilt weltweit als erreicht (`GameState::developments`); nach
  der Frist der Weiterentwicklung (M37, `gemeingut_nach_jahren`) kennt sie jeder Hersteller.
  Meldung an den Spieler, wenn er das Produkt verkauft.
- **Scheitern** (am Ende einer Phase, ohne Geld oder überholt): Die Anteile sind wertlos.
- Was Eigner davon haben (Lizenz, Eingliedern, neue KI-Firma, Forschungsbonus), regelt SU2.
- Beendete Start-ups bleiben `aufbewahren_jahre` in der Liste, die historischer Erfinder
  immer (so gründet jeder Erfinder nur einmal).

### Einschätzung

Die Erfolgschance (Chance der laufenden Phase, festgehalten bei ihrem Beginn, mal die
Chancen der folgenden Phasen mit demselben Vorlauf) zeigt das Spiel nur als Schätzung:
Bei der Entstehung wird *u* gleichverteilt in −1 … 1 gezogen;

    gezeigt = clamp(Chance · (1 + u · b), 0, 1),  b = unschaerfe · (1 − G · A)

mit Güte G und Abdeckung A der Strategieabteilung des Spielers (ZA2). Ohne arbeitende
Abteilung (b = unschaerfe) zeigt das Spiel nur die Stufe: gering unter `stufe_mittel_ab`
(20 %), mittel unter `stufe_hoch_ab` (40 %), sonst hoch.

## SU2 – Beteiligungen an Start-ups

`docs/BETEILIGUNGEN.md` Abschnitte 5.4–5.6. Daten: `parameter/startups.yaml`, Block
`beteiligung`. Kern: Modul `ventures` mit den Befehlen `InvestInVenture`, `GrantVenture`,
`SellVentureStake`, `SteerVenture` und `IntegrateVenture`. Eigner sind Firmen
(`Holder::Company`, auch die des Spielers: Die Beteiligung steht in ihrer Bilanz), die
Gründer (`Holder::Private`) und Investoren außerhalb des Spiels (`Holder::Investors`).
KI-Firmen nutzen dieselben Befehle ab SU3.

### Wert

- **Laufend:** bei offener Runde W = Kapital · `bewertung` der Phase (Wert vor der Runde),
  nach der Runde W = Kapital · (`bewertung` + 1).
- **Bei Erfolg:** W_E = Kapital der letzten Phase · (`bewertung` + 1) · `erfolg_faktor`.
- Mit den Daten (Bewertungen 10, 5, 2,5; Faktor 1,3) bringt ein Dollar der ersten Runde im
  Mittel rund das 2,8-Fache, der zweiten das 1,9-Fache, der dritten das 1,2-Fache – bei
  einer Verbesserung ohne Vorlauf; die meisten Einsätze gehen ganz verloren.

### Anteile kaufen (`InvestInVenture { venture, amount }`)

- Laufendes Start-up, Betrag > 0 und durch die Kasse gedeckt, nicht die Tochterfirma einer
  anderen Firma.
- **Offene Runde:** höchstens der offene Rest (Kapital − gesammelt). Der Betrag ist eine
  **Zusage**: Er geht sofort aus der Kasse in die Finanzanlagen. Decken die Zusagen die
  Runde, schließt sie sofort und die Phase beginnt heute; sonst decken Investoren den
  Rest wie in SU1 (`investoren_chance_monat`). Beim Schließen erhält jeder Geber
  Betrag / (B + Kapital) mit B = Kapital · `bewertung`; alle bisherigen Eigner behalten
  B / (B + Kapital) ihres Anteils. Geht die Runde ohne Geld ein, fließen die Zusagen zurück.
- **Zwischen den Runden (aufstocken):** Anteile von Gründern und Investoren, anteilig nach
  ihrem Bestand, zum Preis W · (1 + `kauf_aufschlag`) je ganzem Start-up; höchstens so
  viel, wie beide halten.
- Käufe zählen zum Budget „Beteiligungen“ des Jahres (ZA2) wie Übernahmen und Lizenzen.

### Fördergeld (`GrantVenture { venture, amount }`)

- Aufwand (Kostenart Forschung), keine Anteile und keine Rechte. Die Chance der laufenden
  Phase steigt:

      Chance ← Chance + (1 − Chance) · foerderung_wirkung · min(1, Betrag / Kapital)

- Nutzen: Gelingt das Start-up, ist die Technologie früher in der Welt (SU1).

### Rechte nach Anteil

| Anteil | Rechte |
| --- | --- |
| jeder | Erlös bei Verkauf und Börsengang |
| ab `sperrminoritaet` (25 %) einer Firma | keine andere Firma kann eingliedern |
| über `mehrheit` (50 %) | lenken, eingliedern; bei Erfolg wird es Tochterfirma |

Eine Minderheitsbeteiligung bringt nur Geld, keinen Zugang zur Technologie.

### Anteile verkaufen (`SellVentureStake { venture, share }`)

- Laufendes Start-up; an Investoren außerhalb des Spiels zum Preis Anteil · W ·
  (1 − `verkauf_abschlag`); höchstens der eigene Anteil (Zusagen zählen nicht).

### Lenken (`SteerVenture { venture, pace }`, über 50 %)

- Tempo `zuegig`, `normal` oder `gruendlich` (`lenkung`): Faktoren auf die Dauer und die
  Chance der Phasen, die danach beginnen, und der laufenden, solange ihre Runde offen ist.
  Die Chance bleibt höchstens `chance_max`.

### Eingliedern (`IntegrateVenture { venture }`, über 50 %)

- Keine andere Firma hält eine Sperrminorität. Die übrigen Eigner erhalten
  W · (1 + `kauf_aufschlag`) für ihren Anteil, Zusagen anderer Firmen fließen zurück; die
  Firma hält 100 %, das Start-up ist ihre **Tochterfirma**.
- Die Runden einer Tochterfirma deckt die Mutter am nächsten Monatsanfang ganz (Zusage,
  zählt zum Budget „Beteiligungen“), solange ihre Kasse reicht. Sonst decken Investoren die
  Runde wie in SU1 und verwässern die Mutter; fällt ihr Anteil auf `mehrheit` oder
  darunter, ist das Start-up keine Tochterfirma mehr.

### Erfolg

1. **Tochterfirma:** Die Mutter kennt die Technologie sofort (bzw. hat die Stufe der
   Weiterentwicklung). Ihr Buchwert der Beteiligung wird Forschungsaufwand.
2. **Eine Firma hält über 50 %, ohne Tochter:** Sie zahlt die übrigen Eigner zu W_E aus und
   wird Mutter wie unter 1. Reicht ihre Kasse nicht, folgt 3.
3. **Sonst – Börsengang:** Jede beteiligte Firma erhält Anteil · W_E. Spielen KI-Firmen mit,
   entsteht daraus eine **neue KI-Firma** im Land des Start-ups, benannt nach dem Gründer,
   mit der Technologie (bzw. der Stufe) und einem ersten Standort für ein Produkt, das
   sie nutzt (wie eine Neugründung der KI, M10).

Die Technologie bzw. Stufe ist in jedem Fall weltweit wie in SU1 in der Welt.

### Scheitern

- Die Anteile sind wertlos: Die Buchwerte werden abgeschrieben, Zusagen einer Runde, die
  nicht zustande kam, fließen zurück.
- **Forschungsbonus:** Die Firma mit über 50 % (oder die Mutter) erhält `forschungsbonus`
  des heutigen Forschungsaufwands der Technologie (bzw. der nächsten Stufe des Produkts)
  als Forschungspunkte.

### Buchungen

- Neues Aktivkonto **Finanzanlagen** (`Account::Participations`, Cashflow aus
  Investitionen) und neue Kostenart **Beteiligungen** (`CostType::Investments`).
- Kauf und Zusage: Kasse → Finanzanlagen; Rückzahlung umgekehrt. Verkauf, Auszahlung und
  Börsengang: Erlös in die Kasse, Buchwert (anteilig) aus den Finanzanlagen, der
  Unterschied als Ertrag oder Aufwand „Beteiligungen“. Scheitern: Abschreibung
  „Beteiligungen“. Fördergeld: Aufwand „Forschung“. Mutter bei Erfolg: Buchwert als
  Aufwand „Forschung“. Jede Buchung hält die Bilanz ausgeglichen.

### Empfehlung der Strategieabteilung

- Nur mit arbeitender Strategieabteilung (ZA2): Am Prüftermin des Vorstands bewertet die
  Stelle des Ressorts Strategie bis zu ⌊K⌋ offene Runden (Thema `startup`). Für eine
  Zusage ist der erwartete Ertrag je Dollar

      E = gezeigte Chance · W_E' / (B + Kapital) · Π (b_j / (b_j + 1))

  mit W_E' wie W_E aus dem heutigen Kapitalbedarf der letzten Phase und dem Produkt über
  die späteren Runden (b_j: ihre Bewertungen).
- Sie empfiehlt, wenn E ≥ 1 + (1 − Risikobereitschaft) · `rendite_mindest`
  (Risikobereitschaft aus der Vorgabe „Beteiligungen“, ZA2). Betrag: der offene Rest,
  höchstens der Rest des Budgets „Beteiligungen“ und `einsatz_kasse` der Kasse. Die
  besten E zuerst.
- Wie bei Umschuldung und Gehaltsrunde (ZA3): Im Budget und in der Freigabegrenze der
  Abteilung sagt die Stelle selbst zu, sonst wird es ein Anliegen mit Begründung
  (Chance, Wert bei Erfolg, Anteil danach, E).

## SU3 – Ausgründungen, KI-Firmen und KI-Investoren beteiligen sich

`docs/BETEILIGUNGEN.md` Abschnitte 5.3 und 5.7. Daten: `parameter/startups.yaml`, Blöcke
`ausgruendung` und `ki` im Abschnitt `beteiligung`. Kern: Modul `ventures` (Befehl
`SpinOff`, Regeln der KI-Firmen). KI-Firmen nutzen dieselben Befehle wie der Spieler
(`InvestInVenture`, `IntegrateVenture`, `SpinOff`); „KI-Investoren“ sind die Investoren
außerhalb des Spiels (`Holder::Investors`, SU1), die Runden decken und Anteile kaufen.

### Ausgründung (`SpinOff { site, sell }`)

- Ein Forschungszentrum der Firma arbeitet an einem Ziel, das ein Start-up haben kann:
  einer Technologie, die noch nicht erfunden ist (weder in der Geschichte noch im Spiel),
  oder der nächsten Stufe eines Produkts, die noch keine Firma hat (M37).
- Fortschritt F = gesammelte Punkte / heutiger Aufwand (Technologie: M9; Stufe: M37).
  Ausgründen geht ab F ≥ `ausgruendung.fortschritt_min`.
- Das neue Start-up sitzt im Land des Forschungszentrums, der Gründer ist ein Name aus
  diesem Land (eigener Zufallsstrom je Start-up), Vorlauf wie in SU1 (Jahre bis zum
  historischen Jahr, bei einer Stufe 0). Es beginnt in Phase ⌊F · Anzahl Phasen⌋
  (höchstens die letzte) mit offener Runde (SU1).
- Die Punkte der Firma für das Ziel gehen an das Start-up über (die Firma verliert sie),
  das Forschungszentrum ist frei.
- Die Firma hält 100 % (Buchwert 0: die Forschung war Aufwand). Mit `sell` (0 ≤ sell < 1)
  verkauft sie sofort diesen Anteil an Investoren wie beim Verkauf (SU2):
  Erlös = sell · W · (1 − `verkauf_abschlag`), ganz als Beteiligungsergebnis.
- Behält sie über `mehrheit`, ist das Start-up ihre **Tochterfirma**; sonst ist sie
  Minderheitseigner wie jeder andere.

### Runden einer Tochterfirma (ersetzt die Regel aus SU2)

- Am ersten Monatsanfang einer offenen Runde sagt die Mutter ihren Teil zu:
  Anteil der Mutter · Kapital (pro rata), sofern ihre Kasse reicht. Den Rest decken
  Investoren wie in SU1; so bleibt der Anteil der Mutter gleich. Bei 100 % (eingegliedert)
  schließt die Runde sofort wie bisher.
- Reicht die Kasse nicht, verwässern die Investoren die Mutter; unter `mehrheit` ist es
  keine Tochter mehr.

### KI-Firmen beteiligen sich (`ki`)

- **Prüfung:** Am Monatsanfang prüft jede KI-Firma mit Kasse ≥ `ki.kasse_min_usd` mit der
  Wahrscheinlichkeit `ki.pruefen_chance` die Start-ups (eigener Zufallsstrom je Firma und
  Monat, damit eine Welt ohne Start-ups bitgleich bleibt).
- **Einschätzung:** Die gezeigte Chance wie in SU1 mit der Unschärfe
  `unschaerfe` · (1 − Kompetenz) – die Kompetenz ersetzt die Strategieabteilung.
- **Zusage in einer offenen Runde:** erwarteter Ertrag je Dollar E wie in SU2. Sie sagt zu,
  wenn E ≥ 1 + (1 − Aggressivität) · `ki.rendite_mindest`, die beste Runde zuerst,
  höchstens der offene Rest und `ki.einsatz_kasse` der Kasse im Monat.
- **Wegkaufen:** Arbeitet ein Start-up an einer Technologie, die einem Produkt der Firma
  dient (Rezept eines Produkts ihrer Branchen braucht sie), oder an der nächsten Stufe eines
  Produkts, das sie herstellt, hält sie schon mindestens `ki.uebernahme_anteil_min` (aus
  Zusagen) und ist die gezeigte Chance ≥ `ki.uebernahme_chance_min`, kauft sie zwischen
  den Runden Anteile von Gründern und Investoren bis über die Mehrheit
  und gliedert es ein – wenn Kauf und Eingliederung zusammen höchstens
  `ki.uebernahme_kasse` der Kasse kosten und keine andere Firma eine Sperrminorität hält.
  Ein Spieler mit mindestens 25 % verhindert so die Übernahme.
- **Ausgründen:** Zum Jahresbeginn gründet eine KI-Firma mit der Wahrscheinlichkeit
  `ki.ausgruenden_chance` ein Forschungsprojekt aus, das die Bedingungen erfüllt und vor
  der Geschichte fertig werden kann (bei einer Technologie: Vorlauf in Monaten ≥ Summe der
  Monate der restlichen Phasen) und an dem kein anderes Forschungszentrum arbeitet, und
  verkauft `ki.ausgruenden_verkauf` an Investoren. Dem Spieler zeigt die Ansicht Vorlauf,
  Restdauer und die Zahl der Firmen am selben Ziel und warnt, wenn die Geschichte oder die
  Konkurrenz schneller sein dürfte.
- Jede Handlung läuft als Befehl durch dieselbe Prüfung wie beim Spieler (MA0: Entscheidung
  mit Thema `startup`); der Spieler erfährt, wenn eine Firma ein Start-up übernimmt, an dem
  er beteiligt ist (er erhält W · (1 + `kauf_aufschlag`) für seinen Anteil), und wenn eine
  Firma ausgründet.

## ZA4 – KI-Zentralen, Sitzverlegung der KI, Anteile an Firmen, Trefferquote nach Erfolg

Auftrag vom 07.10.2026 („Paket A“): offene Punkte O 5, 11, 14, 29 und 36
(`docs/OFFENE_PUNKTE.md`, Abschnitt O); `docs/BETEILIGUNGEN.md` Abschnitte 3, 4.2, 4.4, 5.4
und 5.5. Daten: `parameter/zentrale.yaml`, Block `ki`; die Gebote nutzen
`parameter/startups.yaml` (Block `ki`) und `parameter/kaufmodell.yaml`
(`gebotsaufschlag`). Kern: Module `central` (Zentrale und Sitz der KI, Trefferquote) und
`ventures` (Verkauf an Firmen). KI-Firmen nutzen dieselben Befehle wie der Spieler.

### KI-Firmen richten Zentralabteilungen ein

Zum Jahresbeginn (Monatsanfang im Januar) legt jede KI-Firma ihre Zentrale fest, mit
`StaffDepartment` wie der Spieler:

- **Budget im Jahr:** Z = `ki.anteil_umsatz`(Kompetenz) · Umsatz der letzten zwölf Monate;
  Z = 0, wenn das Ergebnis der letzten zwölf Monate nicht positiv ist (eine Firma mit
  Verlust spart an der Zentrale).
- **Bedarf:** Die Abteilungen in der Reihenfolge `ki.reihenfolge`, jede nur ab ihrer
  Arbeitslast `ki.mindestlast` (Arbeitslast wie ZA2: Finanzen Kredite + 1, Personal
  Manager der Firma, Marketing Werbebudgets + 1, Strategie und Recht 1; ohne Angabe 1).
  Angestellte n = ⌈Arbeitslast / `faelle`⌉, also volle Abdeckung.
- **Kosten im Jahr:**

      K = n · (Jahreslohn der Lohngruppe im Sitzland + buero_usd) + Gehalt der Leitung

  Gehalt der Leitung: das heutige, sonst die Forderung eines mittleren Managers
  (`gehalt_fach` des Vorstands · Jahreslohn nach `gehalt_lohngruppe` im Sitzland, MA1).
- Die Abteilungen kommen in dieser Reihenfolge dazu, solange ihre Kosten zusammen Z nicht
  übersteigen; die übrigen schließt die Firma (0 Angestellte) und entlässt deren Leitung
  (`DismissManager`, Abfindung wie MA1).
- **Leitung:** Die KI stellt sie ein wie den CEO (MA6): nach dem CEO, vor den
  Standortleitungen, Gehalt höchstens `gehalt_anteil` des Umsatzes der Firma.

Die Abteilungen wirken wie beim Spieler (ZA2): Finanzen senken den Risikoaufschlag neuer
Kredite, Personal schult die Manager, Marketing verstärkt die Werbung. Dazu:

- **Finanzen schulden um:** Am Monatsanfang ersetzt die KI bis zu ⌊K⌋ Kredite der
  Umschuldungsliste (ZA3) mit `RefinanceLoan`.
- **Strategie:** Die KI sucht Übernahmen auch in den beobachteten Ländern (ZA2) und sieht
  Start-ups schärfer: Unschärfe · (1 − Kompetenz) · (1 − Einblick), Einblick = Güte ·
  Abdeckung der Abteilung (SU1, SU3).
- **Recht:** Die KI bietet auch für Lizenzen der Technologien, die ihre Rechtsabteilung
  prüft (ZA2).

Ohne Abteilungen ändert sich nichts an den Regeln der KI.

### KI-Firmen verlegen ihren Sitz

Zum Jahresbeginn prüft jede KI-Firma ohne laufenden Umzug, deren letzter Umzug
mindestens `ki.sitz.sperre_jahre` zurückliegt:

- **Kandidaten:** Länder mit eigenem Standort, deren Standorte zusammen mindestens
  `ki.sitz.anteil_umsatz_min` des Umsatzes der letzten zwölf Monate bringen – eine Firma
  zieht dorthin, wo ihr Geschäft ist, nicht in jedes Land mit einer Niederlassung – und
  deren BIP je Kopf mindestens `ki.sitz.bip_anteil_min` des heutigen Sitzlands beträgt:
  Kapital, Manager und Verwaltung bleiben in reichen Ländern (ohne diese Bedingung zog im
  ersten Lauf eine deutsche Weberei 1902 nach Indonesien, um Löhne für einige Angestellte
  zu sparen).
- **Ersparnis im Jahr:**

      S(L) = max(0, Ergebnis vor Steuern) · (Steuersatz heute − Steuersatz L)
           + Σ Angestellte · (Jahreslohn heute − Jahreslohn in L)

  Ergebnis vor Steuern über die letzten zwölf Monate; Steuersatz = Gewinnsteuer des
  Landes heute (M6); die Summe über die Zentralabteilungen, Jahreslohn ihrer Lohngruppe.
  Die Gehälter der Manager ändern sich mit dem Umzug nicht.
- Sie verlegt den Sitz (`SetHeadquarters`) in das Land mit der größten Ersparnis, wenn
  S · `ki.sitz.amortisation_jahre` ≥ Kosten der Verlegung (ZA1). Der Spieler erfährt es
  als Nachricht über die Konkurrenz.

### Anteile an Firmen verkaufen (Bieterverfahren)

Neben dem Sofortverkauf an Investoren (SU2, mit Abschlag) bietet eine Firma ihren ganzen
Anteil an einem laufenden Start-up allen Firmen an:

- `OfferVentureStake { venture, minimum }` mit Mindestpreis; ein neues Angebot ersetzt das
  alte, ohne Mindestpreis ist es zurückgezogen. Das Bieterverfahren endet am nächsten
  Monatsanfang, bevor die Runden der Start-ups laufen.
- **Gebot** einer KI-Firma F für den Anteil *s* des Verkäufers:
  - Wert des Anteils A = s · W (W wie SU2).
  - Erwarteter Wert E = Chance(F) · W_E · s · Π b_j / (b_j + 1) – W_E der Wert bei Erfolg,
    wie er heute aussieht (SU2), das Produkt über die offene und die künftigen Runden
    (Verwässerung, wenn F nicht nachschießt), Chance(F) wie F sie sieht (SU3, mit
    Strategieabteilung schärfer).
  - Höchstpreis H = E / (1 + (1 − Aggressivität) · `ki.rendite_mindest`) wie bei Zusagen.
  - Gebot = min(H, A · (1 + `gebotsaufschlag`(Aggressivität)), `ki.uebernahme_kasse` ·
    Kasse); kein Gebot, das nicht über dem Preis der Investoren A · (1 −
    `verkauf_abschlag`) liegt.
  - Es bieten KI-Firmen mit Kasse ≥ `ki.kasse_min_usd`, nicht der Verkäufer, keine Firma,
    die damit über die Mehrheit käme, wenn ein Dritter eine Sperrminorität hält („Verkauf
    an Konkurrenten blockieren“, BETEILIGUNGEN 5.5), und keine Firma in die Tochterfirma
    eines anderen, außer die Mutter verkauft.
- **Zuschlag:** Das höchste Gebot ab dem Mindestpreis (bei Gleichstand die Firma mit der
  kleineren Nummer) kauft mit `BuyVentureStake { venture, seller, price }` zum Preis
  seines Gebots. Sonst bleibt der Anteil, und das Angebot endet. Der Spieler erfährt den
  Käufer und den Preis, sonst das beste Gebot (falls eines vorlag).
- **Buchungen:** Verkäufer wie beim Verkauf (SU2: Erlös in die Kasse, Buchwert aus den
  Finanzanlagen, der Unterschied als Beteiligungsergebnis); Käufer Kasse → Finanzanlagen,
  zählt zum Budget „Beteiligungen“ (ZA2). Verkauft die Mutter, ist es keine Tochter mehr.
- Endet das Start-up oder hat der Verkäufer keinen Anteil mehr, verfällt das Angebot.

### Trefferquote nach dem Erfolg (ersetzt die Bewertung aus ZA3 für Übernahmen)

- **Übernahmen** (Standorte und Bereiche) werden nach `bewertung_monate` am Wert des
  Gegenstands dann bewertet: Treffer, wenn

      Angebot ≤ V · G' / G

  V Wert nach den Regeln am Tag der Schätzung (was die Regeln höchstens zahlen), G der
  Grundwert der Standorte am Tag der Schätzung, G' ihr Grundwert am Tag der Bewertung
  (M30; bei einem Bereich die Summe seiner Standorte, dazu jeweils der Markenwert vom Tag
  der Schätzung). Ob die Übernahme zustande kam und bei wem die Standorte heute sind,
  zählt nicht: Es zählt, ob der Gegenstand den Preis wert blieb. Mit G = 0 gilt die Regel
  aus ZA3.
- **Lizenzen** wie bisher am Tag der Schätzung: Eine gekaufte Lizenz hat für den Käufer
  später keinen eigenen Wert mehr.
- **Start-ups** (neu): Jede Zusage, die die Strategieabteilung empfiehlt oder selbst gibt
  (SU2), wird für ihre Leitung bewertet – je Start-up und Leitung eine, solange sie
  aussteht. Nach `bewertung_monate`: Treffer, wenn das Start-up Erfolg hatte, oder wenn es
  noch läuft und mit der wahren Chance Chance · W_E · Anteil am Ende ≥ Betrag
  (Anteil am Ende = Betrag · Anteil je Dollar am Tag der Empfehlung, SU2); gescheitert ist
  kein Treffer.
- Ausstehende Bewertungen alter Spielstände behalten ihr Ergebnis vom Tag der Schätzung.

### Ausfallquote der Start-ups (O 36)

- Weltlauf 1900–1940 (100 KI-Firmen, „normal“, vor ZA4): 491 gegründet, 64 % der in den
  letzten zehn Jahren beendeten gescheitert – im Richtwert 60–70 %. Die 77–83 % der Läufe
  bis 1915 kamen vom Anlauf: Fehlschläge enden früher als Erfolge (ein Erfolg braucht alle
  drei Phasen, mindestens 54 Monate). Die Chancen der Phasen bleiben.

## B1 – Manager-Restpunkte: Gehalt über der Kasse, Abwerben, Nachfolgemodelle

Auftrag vom 07.10.2026 („alles außer Stufe 6“), Paket B: offene Punkte N 37 und N 46,
M 5 (`docs/OFFENE_PUNKTE.md`). Daten: `ki/produktnamen.yaml` (`nachfolger` je Stil).

### Warnung bei Gehältern über der Kasse (N 37)

Im Managermarkt trägt jeder Bewerber und jeder eigene Manager das Kennzeichen „über der
Kasse“, wenn seine Jahresforderung F größer ist als die Kasse K der Firma (F > K). Gibt
es solche Bewerber, steht über der Liste eine Warnung mit K. Dieselbe Prüfung gilt für
das Angebot beim Abwerben (unten). Die Regel verbietet nichts; sie zeigt nur, dass ein
Jahr Gehalt die Kasse übersteigt (ein Manager kostet F/12 im Monat).

### Abwerben aus dem Reiter „Wettbewerb“ (N 46)

Die Seite einer anderen Firma zeigt unter „Führung“ alle ihre Manager (Vorstand zuerst,
dann Kontinent, Land, Standort; Leitungen vor Fachstellen) mit Stelle und Fähigkeiten,
so wie der Spieler sie einschätzt (MA1, MA5). Für jede freie Stelle s des Spielers –
ohne Inhaber und ohne eigenes offenes Angebot – steht das Angebot

  A(m, s) = max(Forderung von m für s, Gehalt von m · (1 + `abwerbung.aufschlag`))

(MA6, `poach_salary`). „Abwerben“ sendet `PoachManager { manager, position }`; die Regeln
aus MA6 gelten unverändert (Sperre nach einem Angebot, nur ein Angebot je Manager, die
Stelle bleibt für das Angebot reserviert). Eine KI-Firma antwortet am nächsten Tag: Sie
hält den Manager zu A, wenn A ≤ Gehalt · `ki_gegenangebot_max` und ihre Kasse A trägt,
sonst wechselt er. Der Spieler bekommt in beiden Fällen eine Meldung; beim Halten mit dem
Tag, ab dem er erneut umwerben darf.

### Nachfolgemodelle (M 5)

Erreicht eine Firma für ein Produkt p mit Namen n eine neue Entwicklungsstufe (M37) und
hat der Stil von p eine Liste `nachfolger` G = (g₁, g₂, …), heißt das Nachfolgemodell:

1. Endet n auf ein gᵢ, wird es durch gᵢ₊₁ ersetzt; gibt es kein gᵢ₊₁, gibt es kein
   Nachfolgemodell („Kelvor 900 III“ → „Kelvor 900 IV“).
2. Sonst bekommt das letzte Wort mit einer Zahl z die kleinste Zahl der Liste `zahlen`
   des Stils, die größer als z ist; Buchstaben davor und danach bleiben („Kelvor M80“ →
   „Kelvor M90“ ist nur möglich, wenn 90 in der Liste steht; mit den Daten „Kelvor M80“ →
   „Kelvor M100“, „Marvik Typ 12“ → „Marvik Typ 16“).
3. Gibt es kein solches Wort oder keine größere Zahl, wird g₁ angehängt („Kelvor Super“
   → „Kelvor Super II“, „Irvok 900“ → „Irvok 900 II“).

Ist der Name schon bei einer anderen Firma für p vergeben, zu lang (über 40 Zeichen)
oder ausgeschlossen, folgt der nächste Schritt (bis zu 30). Dann gilt:

- **KI-Firmen** benennen ihr Produkt sofort um (`NameProduct`, wie jede Handlung).
- **Der Spieler** bekommt eine Meldung mit dem Vorschlag; in der Marktansicht steht das
  Nachfolgemodell als erster der drei Namensvorschläge. Umbenennen bleibt seine
  Entscheidung.
- Stile ohne `nachfolger` (Markennamen für Waren des täglichen Bedarfs) behalten den
  Namen. Marke und Bekanntheit hängen an Firma und Warengruppe, nicht am Namen: Ein
  Nachfolgemodell verliert keine Bekanntheit.

## B2 – Können der KI-Manager wirkt auf Preise und Ausbau

Auftrag vom 07.10.2026, Paket B: offener Punkt N 45. Daten: `parameter/kimodell.yaml`,
`verhalten.schaetzfehler`. Bisher wirkte die Kompetenz k einer KI-Firma (ihr Charakter
mit dem, was ihre Manager beitragen, MA6) nur auf die Häufigkeit der Betriebsprüfung und
die Vorausplanung der Forschung. Jetzt schätzt eine Firma auch ihre Kosten und Margen
nur so genau, wie ihre Leute es können.

### Schätzfehler

Je Firma f, Produkt p und Kalenderjahr j zieht der Zufallsstrom `Estimate { f, p, j }`
zwei Zahlen u_K, u_M gleichverteilt in [−1, 1]. Mit der Spanne

  F(k) = `schaetzfehler`.at(k)   (bei k = 0: 15 %, bei k = 1: 0 %)

sieht die Firma

- **Kosten:** Stückkosten K' = K · (1 + F(k) · u_K). Die Preisuntergrenze ist
  K' · `preisuntergrenze` statt K · `preisuntergrenze`. Eine schwache Firma setzt ihre
  Untergrenze also mal zu tief (sie verkauft mit Verlust und drückt den Markt), mal zu
  hoch (sie verliert Absatz).
- **Marge beim Ausbau:** (1 + m') = (1 + m) · (1 + F(k) · u_M). Ausgebaut wird, wenn m'
  die Schwelle `ausbau_marge` erreicht (Auslastung und Absatz wie bisher). Eine schwache
  Firma baut deshalb manchmal Anlagen für eine Marge, die es nicht gibt, und übersieht
  manchmal eine gute.

Die Zahlen gelten ein Kalenderjahr; im nächsten Jahr irrt die Firma anders. Der eigene
Strom je Firma, Produkt und Jahr verschiebt keine anderen Zufallszahlen. Der Spieler ist
nicht betroffen: Seine Stellen schätzen mit dem Prognosefehler der Manager (MA2).

## C1 – Händler gleichen Preisinseln aus (Arbitrage)

Auftrag vom 07.10.2026, Paket C: offener Punkt G (M40) 4 und Teil von G 9. Daten:
`parameter/marktmodell.yaml`, `haendler.arbitrage`. Ergänzt die Händlerregeln aus M8.

Bisher brachten Händler Ware nur für die **offene** Nachfrage ō eines Landes, also für
Käufer, die die Firmen im Land nicht bedienten. War ein Land zu hohen Preisen versorgt,
kam keine billigere Ware herein (2009: LCD-Panel in den USA beim Dreifachen des
chinesischen Preises).

### Regel

Für ein Zielland B mit Marktpreis P (wie in M8: Index oder höchstes Gebot) und dem
günstigsten Einstandspreis E aus dem Ausland (Angebotspreis + Transport):

- **Preisinsel:** P > E · (1 + `marge`) · (1 + `arbitrage.abstand`).
- Dann planen die Händler zusätzlich zur offenen Nachfrage einen Teil des Absatzes in B:

      a = `arbitrage.anteil` · v,   v = Absatz in B im Vormonat / 30

  und der Bedarf wird (vorrat_tage + Transporttage) · (ō + a) − Importlager − unterwegs.
- Den Anteil a decken nur Angebote, die selbst die Bedingung der Preisinsel erfüllen
  (E_i · (1 + marge) · (1 + abstand) < P), die günstigsten zuerst; teurere Angebote
  decken wie bisher nur die offene Nachfrage.
- Die Ware landet im Importlager und wird dort wie jedes Angebot verkauft: Käufer nehmen
  die billigsten Angebote zuerst. Die Firmen im Land verlieren Absatz, ihr Lager wächst,
  ihre Preise sinken (M16, „langsam“), bis P den Abstand zum Ausland nicht mehr
  übersteigt. Im Gleichgewicht gilt also P ≤ (Preis(A) + Transport) · (1 + marge) ·
  (1 + abstand).

Mit den Daten: Abstand 15 %, Anteil 25 % des Absatzes. Der Abstand verhindert, dass
Händler wegen kleiner Unterschiede Ware hin- und herschieben.

## C2 – Rohstoffe nach 2015: Förderkurve und Ausbau an allen Standorten

Auftrag vom 07.10.2026, Paket C: offener Punkt G (M41) 9. Daten:
`parameter/produktionsmodell.yaml` (`foerderkurve_ab`), `parameter/kimodell.yaml`
(`verhalten.ausbau_je_pruefung_max`). Rohöl und Baumwolle kosteten nach 2015 das Zwei- bis
Vierfache: Ölfelder förderten bis zuletzt voll und waren dann schlagartig leer, und eine
Firma baute je Prüfung nur ihren besten Standort aus.

### Förderkurve

Für eine Lagerstätte mit endlichem Vorrat R (im Jahr j, mit dem Förderindex, M41) und der
bisherigen Förderung F ist der Rest r = max(0, R − F/Marktskala) / R. Die zulässige
Förderung im Jahr ist

  Q = Q_max · min(1, r / `foerderkurve_ab`).

Mit 0,5 fördert ein Feld voll, bis die Hälfte des Vorrats gefördert ist; danach sinkt die
Förderung im selben Verhältnis wie der Rest (bei einem Viertel Rest die Hälfte). Erneuerbare
Lagerstätten (Holz, Baumwolle, Kautschuk, Getreide) fördern immer voll. Die Kurve gilt
für die Förderung, für die Größe neuer Konzessionen und für den Ausbau einer Konzession;
eine neue Konzession verlangt wie bisher, dass der Rest `vorrat_jahre_min` Jahre voller
Förderung reicht. Die sinkende Förderung zeigt den Firmen früh an, dass ein Feld
ausläuft: Die Preise steigen allmählich, und neue Felder lohnen sich, bevor das alte leer
ist.

### Ausbau an allen Standorten

Bei jeder Ausbauprüfung (KI vierteljährlich, Stellen nach ihrem Prüftermin) sammelt eine
Firma alle Standorte und Produkte, die die Bedingungen aus M16 erfüllen (Auslastung,
Absatz, Marge, keine knappen Vorprodukte), sortiert sie nach der Marge (die höchste
zuerst) und baut die ersten `ausbau_je_pruefung_max` (4) aus – jeden wie bisher um ein
Viertel, kleine Werke ums Doppelte. Vorher baute sie nur den besten aus; eine Firma mit
zwölf Plantagen brauchte so Jahrzehnte, bis alle wuchsen. Die Kasse begrenzt jeden
Ausbau wie bisher (`ausbau_anteil_kasse_max`, Kredit).

## W1 – Schulung

Lastenheft §5.4, Stufe 2. Daten: `parameter/produktionsmodell.yaml` (`schulung`),
`parameter/kimodell.yaml` (`verhalten.schulung`). Kern: Modul `training`.

### Schulungsniveau je Standort

Jeder Standort hat ein Schulungsniveau s ∈ [0, 1] und ein Ziel z ∈ [0, 1]. Das Ziel ist

1. das eigene Ziel des Standorts (Befehl `SetTraining { site, target }`), sonst
2. die Strategievorgabe „Schulung“ der Firma für den Standort (MA4: Standort, Land,
   Kontinent, weltweit; der genaueste Geltungsbereich gilt) – so hält die Personalstelle
   ein Zielniveau auf Landes-, Kontinent- oder Weltebene –, sonst
3. 0 (keine Schulung).

Zum Monatsanfang nähert sich s dem Ziel:

  s ← min(z, s + `aufbau_je_monat`)   wenn s < z,
  s ← max(z, s − `verlust_je_monat`)  wenn s > z.

Ohne Schulung verliert ein Standort also langsam, was er gelernt hat (Fluktuation).

### Kosten

Jeden Tag: Lohnsumme des Standorts · `kosten_anteil_lohn` · z, gebucht als
Personalkosten des Standorts. Die Kosten folgen dem Ziel, die Wirkung dem erreichten
Niveau: Wer schult, zahlt sofort und gewinnt erst nach und nach.

### Wirkung

- **Arbeit:** Die Arbeitsstunden je Durchlauf sinken um den Faktor
  (1 − `arbeitsersparnis` · s), zusätzlich zu Automatisierung, Größe und Entwicklung.
- **Qualität:** + `qualitaet_punkte` · s Punkte auf die Qualität der Erzeugnisse.

Mit den Daten (Ziel 1 nach etwa einem Jahr erreicht): 10 % weniger Arbeit und 8 Punkte
Qualität für 5 % der Lohnsumme. Lohnt sich bei arbeitsintensiven Werken, kaum bei
Förderstätten mit wenig Personal.

### KI-Firmen

Zum Monatsanfang setzen KI-Firmen für jeden Standort (außer Forschungszentren) das Ziel
`verhalten.schulung` nach ihrer Kompetenz (0 bei Kompetenz 0, 0,8 bei 1), auf 0,05
gerundet, mit demselben Befehl. Forschungszentren schulen nicht (ihre Forscher sind
Akademiker; die Forschung hat eigene Regeln).

## W2 – Zentrale in der Stadt

`docs/BETEILIGUNGEN.md` Abschnitt 3 („Land und Stadt“), offene Punkte O 1 und O 6. Daten:
`laender/*.yaml` (`staedte`), `parameter/zentrale.yaml` (Block `stadt`). Kern: Modul
`central`.

### Städte

Jedes Land hat bis zu fünf Städte: die Hauptstadt und die größten Orte nach Natural Earth
(Einwohner der Agglomeration heute), erzeugt von `tools/daten/laender.py`; Regionen haben
die Städte ihrer Mitglieder. Eine Stadt behält ihren Anteil an der heutigen Bevölkerung
des Landes über den ganzen Zeitraum:

    a = Einwohner der Stadt / Bevölkerung des Landes (2026)
    Einwohner(Jahr) = a · Bevölkerung des Landes (Jahr)

Der Hauptsitz liegt in einer Stadt des Sitzlands (`Company::hq_city`, ohne Angabe die
Hauptstadt, sonst die größte). Ein Land ohne Städte in den Daten zählt als eine Stadt mit
a = 1.

### Akademiker begrenzen die Zentrale

Die Angestellten der Zentralabteilungen sind Akademiker der Lohngruppe ihrer Abteilung. In
einer Stadt stehen allen Zentralen zusammen offen (Pool in realer Größe, ohne
Marktmaßstab):

    Pool_Stadt(g) = Pool(g, Land) · min(1, akademiker_konzentration · a) · anteil_zentralen

Die Sollzahl einer Abteilung setzt der Befehl `StaffDepartment`; besetzt wird:

    S = Summe der Sollzahlen aller Abteilungen der Lohngruppe g mit Sitz in der Stadt
    besetzt = Soll                                 wenn S ≤ Pool_Stadt(g)
    besetzt = ⌊Soll · Pool_Stadt(g) / S⌋           sonst

Neu berechnet an jedem Monatsanfang und nach jeder Änderung einer Sollzahl oder eines
Sitzes. Leistung (Kapazität K = besetzt · faelle), Gehälter, Büro und die Kosten eines
Umzugs folgen den besetzten Stellen. Die Zentrale nimmt den Werken keine Arbeitskräfte weg:
Die Werke rechnen im Marktmaßstab, die Zentrale in realer Größe; dafür steht ihr nur der
Anteil `anteil_zentralen` offen.

### Bürokosten

    Büro je Angestelltem und Jahr = buero_usd · Preisniveau(Land)
                                    · (Einwohner(Jahr) / buero_bezug_einwohner)^buero_elastizitaet

Eine Metropole kostet mehr Miete, bietet aber mehr Akademiker; eine kleine Stadt ist
günstig, aber schnell ausgeschöpft.

### Verlegen

`SetHeadquarters { country, city }`: in ein anderes Land (ohne Stadt: die Hauptstadt) oder
in eine andere Stadt desselben Landes. Kosten und Dauer wie in ZA1; im selben Land kostet
der Umzug nur den Anteil `umzug_im_land` und dauert halb so lange (aufgerundet). Unbekannte
Städte und der heutige Sitz werden abgelehnt.

### KI-Firmen

Zum Jahresbeginn (ZA4), nach dem Einrichten der Abteilungen: Ist eine Abteilung nicht voll
besetzt, zieht eine KI-Firma in die Stadt ihres Landes mit dem größten Pool_Stadt ihrer
Lohngruppe, wenn er größer ist als der heutige und die Kasse die Kosten trägt. Ein Umzug
in ein anderes Land (ZA4) geht in dessen Hauptstadt.

## C3 – Neue Märkte mit mehr Herstellern

Paket C, offener Punkt G (M41). Daten: `parameter/kimodell.yaml`
(`verhalten.einstiege_je_quartal`). Kern: `ai::newcomers`.

Bisher sah ein teurer Markt mit wenigen Herstellern (M33) nur die Diversifizierung der
reichsten Firmen: Jede nahm ihre größte Lücke – fast immer Stahl, Öl oder Getreide. Penicillin
blieb so bis 2015 bei einem Hersteller, obwohl 104 Firmen das Verfahren kannten; Solarmodule,
Tablets und Windkraftanlagen ebenso.

Jedes Quartal, nach den Pionieren (M32) und vor der Diversifizierung, bekommen bis zu
`einstiege_je_quartal` Märkte (wächst mit der Zahl der KI-Firmen wie die übrigen
Firmenzahlen) je einen neuen Hersteller:

- **Märkte:** Produkte mit 1 bis unter `einstieg_firmen_max` Herstellern, die
  - teuer sind (M33: Käufer zahlten im Vormonat wenigstens `einstieg_preisfaktor` × den
    Wert zum Richtpreis) oder
  - knapp sind: Die offene Nachfrage (abzüglich der Anlagen im Bau) ist mindestens
    `einstieg_anteil` · Absatz des Vormonats je Tag.
- **Reihenfolge:** nach dem Wert der Lücke (Menge · gezahlter Preis), der größte zuerst.
- **Wer einsteigt:** die reichste KI-Firma (Investitionsbudget wie bei der
  Diversifizierung), die das Produkt nicht herstellt, ein Verfahren dafür nutzen darf und
  die Anlage bezahlen kann; gebaut wird wie bei der Diversifizierung (Engpass unter dem
  Produkt, Standort, Größe). Je Markt und Quartal höchstens ein Einstieg.

## C4 – Plausibilität 1900–1930

Paket C, Weltlauf 1900–1930 (100 KI-Firmen) nach W1: Stahlwaren kosteten 0,44–0,65 × den
Richtpreis bei 40–50 % Auslastung, die Startwerkstatt verdiente 1900–1902 je 65 % ihres
Startkapitals, Kautschuk lief 1910–1920 bei 14–17 %, 1929 gab es umgerechnet 2,3 Mio.
Autos im Jahr (real 5,3 Mio.).

- **Startbesetzung der realen Firmen** (M10, Punkt 2): Leistung Anzahl · *s* in der
  passenden Größe statt mindestens einer mittelgroßen Anlage. Bei *s* = 0,1 verdoppelte
  die alte Regel 1900 Erz, Kohle und Stahl (Auslastung 31, 38 und 46 %).
- **Ausbau nur, wo der Markt Anlagen braucht:** Eine Firma baut ein Produkt nur aus
  (M10, C2), wenn alle laufenden Anlagen dafür weltweit im Schnitt wenigstens
  `ausbau_markt_auslastung` (0,85, wie `auslastung_normal`: darunter senken die Anbieter ihre Preise) ihrer Leistung planen (geplante Auslastung, gewichtet mit der
  Leistung, ohne stillgelegte und unfertige; ohne Anlagen 1) oder der Markt teuer ist
  (Käufer zahlten im Vormonat wenigstens `einstieg_preisfaktor` · Richtpreis, wie beim
  Einstieg, M33). Vorher baute der billigste Hersteller weiter aus, solange er selbst
  ausverkauft war – die anderen standen still, und die Preise fielen auf die Vollkosten.
  Ohne die Ausnahme blieb Kautschuk 1990–2026 stecken: Synthesewerke ohne Benzin planten
  wenig, die vollen Plantagen durften nicht wachsen (Preis bis 2,5 × Richtpreis).
- **Auto:** `kaufschwelle` 16 statt 80 – bei einem Viertel des Richtpreises kauft die
  Hälfte einer Schicht, wenn ihr Einkommen je Kopf rund das Anderthalbfache des Autos
  beträgt (mit 10 baute die Welt 1929 umgerechnet 9,3 Mio. Autos, real 5,3 Mio.).
- **Nägel der Startwerkstatt:** Richtpreis 1.800 statt 1.900 USD/t, Arbeit 14 + 3 statt
  8 + 2 Stunden je t (Zuführen, Sortieren, Packen).
- **Protokoll:** Die Leistung im Balance-Protokoll zählt stillgelegte Anlagen nicht mehr.

## W3 – Zölle

Lastenheft §8.2, offener Punkt 2. Daten: `parameter/zoelle.yaml`. Kern: Modul `tariffs`.

### Zollsatz

Für eine Ware der Warengruppe *g* aus Land *a* nach Land *b* im Jahr *t*:

    z(a, b, g, t) = 0                                   wenn a = b
                  = gesperrt                            bei einer Handelssperre zwischen a und b
                  = Z_b(t) · f_g · min(1, Faktoren der Zonen, in denen a und b zugleich sind)

- Z_b(t): Durchschnittszoll des Einfuhrlands (`laender`, sonst `standard`), linear
  zwischen den Jahren; ab 2027 zusätzlich die Dynamik (unten), höchstens `maximum`.
- f_g: Faktor der Warengruppe (`warengruppen`, ohne Angabe 1); einzelne Produkte können
  einen eigenen Faktor haben (`produkte`): Rohkautschuk, Rohbaumwolle, Chilesalpeter,
  Stickstoffdünger und die Batterierohstoffe sind zollfrei wie fast überall in der
  Geschichte – mit Zoll bekam die Welt 1900 nur 42 % des Düngers, weil der Salpeter aus
  Chile mit Fracht und Zoll über der Preisgrenze lag.
- Zonen (`zonen`): Mitglied von Beitritt bis vor Austritt; zwischen zwei Mitgliedern
  derselben Zone gilt der Zoll mal ihren `faktor` (EU, EFTA, NAFTA, Mercosur: 0; Empire-
  Präferenz 1932–1973: 0,5; RGW: 0,3).
- Handelssperren (`sperren`): kein Handel zwischen den beiden Ländern in den Jahren.

### Wirkung

- **Händler (M8):** Einstandspreis = (Preis am Ursprung + Fracht) · (1 + z). Bei einer
  Sperre kaufen Händler dort nicht ein. Der höhere Einstand hebt den Preis der Einfuhren und
  schützt die Hersteller im Land.
- **Eigene Lieferungen** zwischen Standorten in verschiedenen Ländern (`ShipGoods`): Zoll =
  Wert der Ware (Herstellkosten im Lager) · z, gebucht als Kostenart Zölle beim
  Empfänger-Standort; bei einer Sperre lehnt der Befehl ab.
- **KI-Planung:** Wo die KI Fracht zwischen Ländern vergleicht (Standortwahl, Engpässe),
  zählt der Zoll auf den Richtpreis im Zielland dazu.

### Dynamik nach 2026

Jedes Jahr am 1. Januar ab 2027 ändert sich der Zoll jedes Landes um eine gleichverteilte
Zufallszahl mit Mittelwert 0 und Standardabweichung `standardabweichung` · Faktor der
gewählten Stufe (Zufallsstrom „Zölle“), der
Zoll bleibt zwischen `minimum` und `maximum`. Die Stufe wählt der Spieler beim neuen Spiel
(`keine`, `normal`, `stark`); ohne Wahl gilt `standard`.

## W4 – Lieferverträge

Lastenheft §9.3. Daten: `parameter/vertraege.yaml`. Kern: Modul `contracts`.

### Inhalt

Ein Vertrag verbindet einen Standort des Verkäufers mit einem Standort des Käufers für ein
Produkt: Monatsmenge *q*, Preis *P* je Einheit frei Standort des Käufers (Fracht und Zoll
trägt der Verkäufer), Laufzeit *n* Monate ab dem nächsten Monatsersten, Mindestqualität
*Q* und Vertragsstrafe *s* (Anteil am Wert der fehlenden Menge, höchstens `strafe_max`).
Verträge gibt es zwischen dem Spieler und KI-Firmen (Stufe 2; Verträge zwischen KI-Firmen
später).

### Lieferung

Jeden Tag nach der Produktion und vor dem Markt liefert der Verkäufer, was bis heute fällig
ist:

    fällig = q · Tag / Tage des Monats − geliefert im Monat
    Menge  = min(fällig, Lager des Verkäufers, Kasse des Käufers / P)

Ware unter der Mindestqualität wird nicht geliefert. Die Ware geht im selben Land sofort
ins Lager des Käufers, sonst als Sendung (Fracht und Zoll wie bei eigenen Lieferungen,
gebucht beim Verkäufer). Der Käufer zahlt P · Menge (Lagerwert P), der Verkäufer bucht
Umsatz und den Abgang zum Lagerwert. Vertragsware ist für den Markt reserviert: Sie geht
vor Verkäufen am Markt.

### Monatsende

Fehlende Menge f = q − geliefert im Monat. Fehlte Ware (Lager oder Qualität), zahlt der
Verkäufer s · P · f an den Käufer; fehlte Geld beim Käufer, zahlt der Käufer s · P · f an
den Verkäufer (Kostenart Sonstiges). Nach *n* Monaten endet der Vertrag.

**Kündigung:** Wer kündigt, zahlt s · P · q · min(`kuendigung_monate`, Restmonate) an die
andere Seite.

### Abschluss

Der Spieler bietet einen Vertrag an (Befehl `ProposeContract`); die KI-Firma antwortet
sofort:

- **als Verkäuferin** nimmt sie an, wenn P − Fracht − Zoll ≥ (1 − `ki.abschlag_verkauf`)
  · ihr Angebotspreis am Standort (ohne Angebot: Marktpreis im Land), ihr Lager die
  Mindestqualität hält und q höchstens `ki.anteil` ihrer Monatsleistung des Produkts am
  Standort abzüglich laufender Verträge ist;
- **als Käuferin** nimmt sie an, wenn P ≤ (1 + `ki.aufschlag_kauf`) · Marktpreis im Land des
  Käufers und q höchstens `ki.anteil` ihres Monatsbedarfs am Standort (Rezepte bei
  geplanter Auslastung) abzüglich laufender Verträge ist;
- die Strafe höchstens `ki.strafe_max` ist.

**Angebote der KI:** Zu jedem Monatsbeginn prüft je Produkt, das der Spieler anbietet
(Verkaufsangebot eines Standorts), die KI-Firma mit dem größten Bedarf daran, und je
Produkt, das der Spieler einkauft (Einkaufsauftrag), die KI-Firma mit dem größten freien
Angebot. Mit Wahrscheinlichkeit `ki.angebot_chance` (Zufallsstrom „Verträge“) schlägt sie
einen Vertrag vor: q = `ki.anteil` · ihr freier Bedarf bzw. ihre freie Leistung, höchstens
die Monatsmenge des Spielers, P = Marktpreis im Land des Käufers, Laufzeit
`laufzeit_standard`, Strafe `strafe_standard`. Der Spieler antwortet binnen `angebot_tage`
Tagen (Befehl `AnswerContract`), sonst verfällt das Angebot. Dasselbe Paar aus Standorten
bekommt für ein Produkt erst wieder ein Angebot, wenn der letzte Vertrag aus der Liste
gefallen ist (`aufbewahren_monate` nach seinem Ende).

## W5 – Logistik

Lastenheft §8.1. Daten: `parameter/logistik.yaml`, `verkehrsmittel.yaml` (Nutzlast,
Kaufpreis). Kern: Modul `logistics`.

### Drei Wege

Für eigene Lieferungen (`TransferGoods`) und Lieferungen aus Verträgen (W4, beim Verkäufer)
wählt eine Firma einen Weg (Befehl `SetLogistics`, Vorgabe Frachtmarkt):

- **Frachtmarkt** (KI-Logistikfirmen): die Kosten der Route wie bisher (M8).
- **Staatlicher Transport:** Kosten · (1 + `staat.aufschlag`), Risiko · `staat.risiko_faktor`.
- **Eigene Flotte:** je Fahrzeug Kosten · Betriebsanteil (unten), solange die Flotte im
  Monat noch Platz hat; der Rest fährt über den Frachtmarkt.

Lieferungen innerhalb eines Landes kosten nichts und haben kein Risiko. Einkäufe am Markt
liefern die Händler (versichert, ohne Risiko für die Firma).

### Flotte

Fahrzeuge mit `nutzlast_t` und `kaufpreis_usd` (Land: Gelände, Straße, Schiene; See) kauft
die Firma zum Kaufpreis des Jahres (Befehl `BuyVehicles`). Kapazität im Monat
(Tonnenkilometer):

    C = Anzahl · Nutzlast(Jahr) · km je Tag(Jahr) · Tage des Monats · flotte.auslastung

Nutzlast und Geschwindigkeit gelten im laufenden Jahr (Erneuerung ist im Unterhalt
enthalten). Eine Ladung belegt t · d · Umweg (Tonnen, Luftlinie der Hauptstädte, Umweg zur
See, wenn die Route ein Seestück hat, sonst zu Land) und braucht ein Fahrzeug, das ihre
Transportklasse befördert.

Die Marktfracht deckt Betrieb, Kapital und die Marge der Logistikfirmen. Je tkm verlangt
der Markt auf einem Weg (Land oder See) für eine Transportklasse den Satz des günstigsten
verfügbaren Verkehrsmittels:

    M = min Kosten je tkm(Jahr) · Klassenfaktor

Die Kapitalkosten je tkm eines eigenen Fahrzeugs bei üblicher Auslastung sind

    K = Kaufpreis · (flotte.unterhalt_anteil + 1 / flotte.nutzungsdauer_jahre)
        / (Nutzlast · km je Tag · Tage des Jahres · flotte.auslastung)

und eine Fahrt kostet den **Betriebsanteil** der Marktfracht

    b = max(0, Kosten je tkm · Klassenfaktor · (1 − flotte.marge_frachtmarkt) − K) / M

Für das günstigste Verkehrsmittel spart eine voll genutzte Flotte also die Marge, eine
halb leere kostet mehr als der Markt. Fahrzeuge mit b ≥ 1 (etwa Fuhrwerke, wo es Bahnen
gibt) bleiben stehen; die Ladung fährt über den Markt. Die Infrastruktur der Länder und der
Umschlag in Häfen sind im Anteil enthalten, weil er sich auf die Marktfracht der Route
bezieht.
Monatlich: Unterhalt `flotte.unterhalt_anteil` · Kaufwert / 12 (Instandhaltung) und
Abschreibung Kaufwert / `flotte.nutzungsdauer_jahre` / 12 bis zum Buchwert 0. Verkauf
(`SellVehicles`): Buchwert · `flotte.verkauf_anteil`, der Rest ist ein Verlust (Sonstiges).

**Fracht für andere:** Ist sie erlaubt, findet am Monatsende `flotte.vermietung_anteil` des
freien Platzes Ladung zum Marktsatz M (für die erste Transportklasse des Fahrzeugs); die
Fahrten kosten den Betriebsanteil davon. Fahrzeuge mit b ≥ 1 mietet niemand.

### Risiko

Jede Ladung zwischen Ländern geht mit der Wahrscheinlichkeit `risiko.<land|see>(Jahr)` ·
Faktor des Wegs (Staat: `staat.risiko_faktor`, sonst 1) verloren; gezogen aus dem
Zufallsstrom „Fracht“ je Firma und laufender Nummer der Ladung. Eine verlorene Umlagerung
kommt nicht an; ihr Lagerwert wird bei Ankunft als Sonstiges abgeschrieben. Eine verlorene
Vertragslieferung trägt der Verkäufer: Ware und Fracht sind weg, der Käufer zahlt nichts,
die Menge fehlt im Vertrag.

### KI

Zu jedem Monatsbeginn vergleicht eine KI-Firma ihre Ladungen des Vormonats (tkm je Weg und
Transportklasse) mit ihrer Flotte: Trägt diese weniger als `ki.anteil` davon, kauft sie vom
Fahrzeug mit dem niedrigsten Betriebsanteil, das sie ganz füllt und bei voller Nutzung mehr
spart (Kapazität · M · (1 − b)), als Unterhalt und Abschreibung im Monat kosten, so viele,
wie in die Lücke passen (höchstens `ki.kasse_anteil` ihrer Kasse), und fährt fortan mit
eigener Flotte und Fracht für andere.

## W6 – Tochterfirmen und Konzern

Lastenheft §5.1, §17.3. Daten: `parameter/tochterfirmen.yaml`. Kern: Modul `group`.

### Gründung und Führung

Der Spieler gründet eine Tochter (Befehl `FoundSubsidiary`: Name, Land des Sitzes,
Startkapital ≥ `mindestkapital_usd`, Schwerpunkt). Die Mutter zahlt Kapital und
`gruendungskosten_usd` (Sonstiges); das Kapital steht bei ihr als Beteiligung (zu
Anschaffungskosten), bei der Tochter als gezeichnetes Kapital. Die Tochter gehört zu 100 %
der Mutter und ist eine eigene Firma mit eigener Bilanz, eigenen Steuern im Land ihres
Sitzes und **eigener Geschäftsführung**: Sie handelt nach den Regeln der KI mit
`geschaeftsfuehrung.kompetenz` und `.aggressivitaet` – sie betreibt, baut aus, forscht,
wirbt und besetzt Stellen wie eine KI-Firma, aber nie gegen ihren Konzern (keine
Kaufangebote, Lieferverträge oder Gebote innerhalb des Konzerns).

Schwerpunkte:

- **Produktion und Handel:** die Regeln der KI für ihre Standorte.
- **Logistik:** zusätzlich kauft sie jeden Monat mit `logistik.kasse_anteil` ihrer Kasse das
  Fahrzeug mit der höchsten Jahresrendite aus Fracht für andere (W5), wenn diese
  `logistik.rendite_min` erreicht:

      Rendite = 12 · (Kapazität · flotte.vermietung_anteil · M · (1 − b) − Unterhalt und
                Abschreibung im Monat) / Kaufpreis

### Kapital und Standorte im Konzern

- **Einlage** (`MoveCapital` mit Betrag > 0): Mutter Beteiligung an Kasse, Tochter Kasse an
  gezeichnetes Kapital.
- **Ausschüttung** (Betrag < 0, höchstens die Kasse der Tochter): zuerst aus dem Gewinn der
  Tochter (Gewinnrücklagen und Jahresergebnis, soweit positiv) – bei der Mutter steuerfrei
  direkt in die Gewinnrücklagen –, der Rest als Rückzahlung von Kapital (Tochter
  gezeichnetes Kapital, Mutter Beteiligung, höchstens deren Buchwert).
- **Standort übertragen** (`TransferSite`): zwischen Mutter und Tochter zu Buchwerten
  (Anlagen, Anlagen im Bau, Firmenwert, Lager, Grundstück); der Empfänger zahlt den
  Buchwert aus seiner Kasse, Gewinn oder Verlust entstehen nicht. Stellen am Standort werden
  frei, laufende Lieferverträge enden (W4).

### Konzernsicht

Konzern = die Spielerfirma und alle Töchter (auch Enkel). Konzernbilanz: Summe der Konten
aller Konzernfirmen; die Beteiligungen an Konzernfirmen werden gegen deren gezeichnetes
Kapital aufgerechnet:

    Beteiligungen(Konzern) = Σ Beteiligungen − Σ gezeichnetes Kapital der Töchter
    Eigenkapital(Konzern)  = Σ Eigenkapital − Σ gezeichnetes Kapital der Töchter

Konzern-GuV des laufenden Jahres: Summe je Kostenart (Ausschüttungen im Konzern sind kein
Ertrag, siehe oben). Geht eine Tochter pleite, schreibt die Mutter ihre Beteiligung ab
(Beteiligungen, Kostenart Beteiligungen) und die Tochter verlässt den Konzern.

### Frachtmarkt für fremde Flotten (Nachtrag zu W5)

Fracht für andere ist begrenzt: Je Monat und Weg (Land, See) zählt das Spiel die
Tonnenkilometer der Händler und der Firmenladungen über den Markt (F). Firmenflotten
übernehmen davon höchstens `flotte.vermietung_markt_anteil` · F; bieten sie mehr freien
Platz an (`vermietung_anteil` · frei), wird jede Firma im selben Verhältnis gekürzt.

## W7 – Controlling

Lastenheft §14.2. Kern: Sicht `views::controlling` (rechnet nur aus dem Hauptbuch, ändert
nichts).

### Deckungsbeiträge

Jede Buchung trägt Kostenart und Kostenstelle (Standort, Produkt). Je Knoten:

    Umsatz              = Umsatzerlöse
    variable Kosten     = Material, Bestandsveränderung, Energie, Transport, Zölle, Lizenzen
    Deckungsbeitrag I   = Umsatz + variable Kosten          (Kosten negativ)
    Fixkosten           = Personal, Instandhaltung, Abschreibungen, Miete und Pacht,
                          Verwaltung und Vertrieb
    Deckungsbeitrag II  = Deckungsbeitrag I + Fixkosten
    Ergebnis            = Deckungsbeitrag II + übrige Kostenarten (Marketing, Forschung,
                          Zinsen, Steuern, Beteiligungen, Sonstiges)

### Ebenen

Konzern (mit Töchtern) → Firma → Kontinent → Land → Standort → Produkt. Ein Standort
umfasst seine Produkte und, was keinem Produkt zugeordnet ist (z. B. Personal des Werks).
Was keinem Standort zugeordnet ist (Zinsen, Steuern, Werbung, Zentrale), steht je Firma
unter „Zentrale und Firma“. Jede Ebene ist die Summe ihrer Kinder.

### Zeiträume und Vergleich

- **Vormonat** (letzter abgeschlossener Monat), verglichen mit dem Monat davor,
- **laufendes Jahr** (bis heute, ohne Vergleich),
- **Vorjahr**, verglichen mit dem Jahr davor.

Der Vergleich nennt das Ergebnis der Vorperiode und die Abweichung; Firmen, Kontinente,
Länder und Standorte haben dazu die Ergebnisse der letzten zwölf abgeschlossenen Monate
(Zeitreihe), Produkte die Ergebnisse je Produkt über alle Standorte.

## K1 – Börse

Lastenheft §11.1, §11.2. Daten: `parameter/boerse.yaml`. Kern: Modul `stock`.

### Wert und Kurs

Je börsennotierter Firma (monatlich zum Monatsbeginn):

    n = bewertung.gewinn_monate, k = Zahl der abgeschlossenen Monate (höchstens n)
    E = 12/n · (Ergebnis der letzten k Monate + (n − k)/12 · rendite_annahme · max(B, 0))
    B = Eigenkapital
    F = gewicht_buchwert · B + (1 − gewicht_buchwert) · max(E, 0) · kgv
    Ziel T = max(F · S, boden_buchwert · B)

S ist die **Marktstimmung** (für alle Firmen gleich): ln S folgt einem Zufallspfad mit
Rückkehr zu 0 (`stimmung.schwankung`, `stimmung.rueckkehr`, Zufallsstrom „Börse“ je
Monat). Historische **Krisen** (`krisen`: Jahr, Monat, Einbruch) senken S im Monat der
Krise um den Einbruch; danach erholt sich S mit der Rückkehr. Der Börsenwert M nähert sich
dem Ziel:

    M' = M · (T / M)^traegheit · exp(rauschen · ε)      (ε je Firma und Monat)

Der Gewinn ist ein Durchschnitt über zwei Jahre, damit ein einzelner starker Monat (etwa
der erste nach dem Spielstart) den Wert nicht vervielfacht; fehlende Monate zählen mit der
angenommenen Rendite. Kurs = M / Aktienzahl (eine Million Aktien je Firma). Der **Index** beginnt bei 100 und
wird jeden Monat verkettet: I' = I · Σ M' / Σ M über die Firmen, die in beiden Monaten
notiert sind (Börsengänge und Pleiten verschieben ihn nicht).

### Börsengang und Kapitalerhöhung

Eine nicht notierte Firma mit Eigenkapital ≥ `boersengang.eigenkapital_min_usd` geht an die
Börse (`GoPublic`, Anteil s ≤ `boersengang.anteil_max` neuer Aktien):

    V = T · (1 − boersengang.abschlag)              (Wert vor dem Gang)
    Erlös = V · s / (1 − s)                       (neue Aktien an Investoren)
    Kosten = Erlös · boersengang.kosten_anteil     (Sonstiges)
    M = V + Erlös

Der Erlös ist Eigenkapital; alle bisherigen Anteile schrumpfen um (1 − s), die Investoren
halten s. Eine notierte Firma erhöht ihr Kapital (`IssueShares`) ebenso mit V = M ·
(1 − abschlag). Der Spieler muss die Mehrheit seiner Firma behalten (bis K2; seit K3 darf
er sie verlieren, siehe dort).

### Dividende

Jedes Jahr im Monat `dividende.monat` zahlt eine notierte Firma `Ausschüttungsquote` ·
Vorjahresergebnis (nur Gewinn, höchstens `dividende.kasse_max` ihrer Kasse) an alle Eigner.
Der Spieler setzt die Quote seiner Firma (`SetDividend`), KI-Firmen nehmen
`dividende.ki_quote`. Firmen als Eigner verbuchen sie als Ertrag (Beteiligungen), der
Anteil des Spielers und der Privatanleger verlässt die Firma (Gewinnrücklagen an Kasse).

### Aktienhandel

Der **Streubesitz** sind die Aktien bei den Anlegern außerhalb des Spiels; Gründer und
Familien (Privatanleger) verkaufen nicht über die Börse. Eine Firma kauft Anteil q einer
anderen notierten Firma aus dem Streubesitz zum Preis

    q · M · (1 + handel.aufschlag + handel.preiswirkung · q)

und hebt M um `preiswirkung · q`; Verkauf an Investoren zu q · M · (1 − handel.abschlag −
handel.preiswirkung · q), M sinkt ebenso. Der Kaufpreis steht in den Beteiligungen; beim
Verkauf geht der anteilige Einstand ab, Gewinn oder Verlust ist Ertrag bzw. Aufwand
(Beteiligungen). Bis K3 höchstens `handel.anteil_max` einer Firma.

### Start und KI

Beim Spielstart sind die KI-Firmen mit Eigenkapital ≥ `start.eigenkapital_min_usd`
notiert, mit `start.streubesitz` bei den Investoren. Eine KI-Firma ohne Notiz mit genug
Eigenkapital geht je Monat mit `ki.boersengang_chance` an die Börse (Anteil
`ki.boersengang_anteil`).

## K2 – Anleihen

Lastenheft §11.1. Daten: `parameter/anleihen.yaml`. Kern: Modul `bonds`.

### Bonität

Aus dem Hauptbuch der Firma, mit einer geplanten neuen Anleihe A:

    B = Bilanzsumme, D = Kredite + Anleihen
    Verschuldung v = (D + A) / (B + A)
    EBIT = 12/k · Σ (Ergebnis − Zinsen − Steuern) der letzten k ≤ gewinn_monate
           abgeschlossenen Monate
    Zinslast Z = 12/k · Σ Zinsaufwand der Monate + A · Kupon
    Zinsdeckung z = EBIT / Z   (ohne Zinslast unbegrenzt)

Die Bonität ist die beste Stufe aus `bonitaet` (geordnet von der besten), deren
`verschuldung_max` ≥ v und `zinsdeckung_min` ≤ z ist; der Kupon in der Zinslast ist der der
geprüften Stufe. Erfüllt keine Stufe die Bedingungen oder hat die Firma noch keinen
abgeschlossenen Monat, findet sie keine Anleger.

### Ausgabe, Kupon, Tilgung

Eine Firma mit Eigenkapital ≥ `eigenkapital_min_usd` gibt eine Anleihe über A ≥
`volumen_min_usd` mit einer Laufzeit von `laufzeit_jahre.min` bis `.max` Jahren aus:

    Kupon = Realzins(Jahr) + Aufschlag(Stufe) · (1 − Ersparnis der Finanzabteilung)
    Kasse += A,  Anleihen += A;  Kosten = A · kosten_anteil (Sonstiges)

Der Kupon bleibt für die ganze Laufzeit fest. Am Monatsende zahlt die Firma A · Kupon / 12
Zinsen; am Ende der Laufzeit zahlt sie A zurück (aus der Kasse, notfalls ins Minus mit den
Folgen der Kontoüberziehung). Vorzeitig kauft sie eine Anleihe ganz zurück zu A · (1 +
`rueckkauf_aufschlag`); der Aufschlag ist Zinsaufwand.

Anleihen sind unbesichert: Sie mindern den Kreditrahmen der Bank nicht, zählen aber zur
Verschuldung, nach der die Bank den Zins eines neuen Kredits bemisst.

### KI

Braucht eine KI-Firma einen Kredit über mindestens `volumen_min_usd`, gibt sie stattdessen
eine Anleihe mit `ki.laufzeit_jahre` aus, wenn deren Kupon mindestens `ki.vorteil_min`
unter dem Zins des Kredits liegt. Sie hält ihre Anleihen bis zum Ende.

## K3 – Investoren und Übernahmen

Lastenheft §11.1 (Aktienrückkauf), §11.2. Daten: `parameter/boerse.yaml` (Abschnitte
`uebernahme`, `rueckkauf`, `ki`). Kern: Modul `stock`.

### Fairer Wert

Ohne die Stimmung der Anleger (KI-Anleger urteilen danach):

    fair = max(F, boden_buchwert · B)      (F aus K1)

### Übernahmeangebot

Eine Firma bietet für alle Aktien einer notierten Firma, die sie noch nicht hält
(`TakeOver`), zu

    Preis je Anteil = M · (1 + uebernahme.aufschlag)
    Kosten = Σ Kaufpreise · uebernahme.kosten_anteil   (Banken, Berater; Sonstiges)

Alle Eigner verkaufen zu diesem Preis – Anleger, Gründer und Firmen (deren Gewinn oder
Verlust gegenüber dem Einstand ist Ertrag bzw. Aufwand der Beteiligungen) –, nur der
Spieler als Eigner seiner Firma nicht. Die Firma verlässt die Börse und wird Tochter des
Käufers (W6), mit eigener Geschäftsführung nach den Regeln der KI. Der Kaufpreis steht
in den Beteiligungen des Käufers; in der Konzernbilanz wird gegen das gezeichnete Kapital
der Tochter aufgerechnet, der Unterschied bleibt in den Finanzanlagen.

Bietet eine KI-Firma für die Firma des Spielers, gelingt die Übernahme nur, wenn die
übrigen Eigner zusammen mehr als die Hälfte halten; dann verliert der Spieler die
Kontrolle und das Spiel endet. Der Spieler kann seine Mehrheit seit K3 durch neue Aktien
verlieren (die Oberfläche warnt) und sie durch Aktienrückkäufe zurückgewinnen.

### Aktienrückkauf

Eine notierte Firma kauft Anteil q ihrer Aktien aus dem Streubesitz zurück (q ≤
`rueckkauf.anteil_max`), zum Kaufpreis des Aktienhandels (K1):

    Preis = q · M · (1 + handel.aufschlag + handel.preiswirkung · q)
    Eigenkapital −= Preis (zuerst Gewinnrücklagen, dann gezeichnetes Kapital), Kasse −= Preis
    Anteile: Anleger − q, danach alle Anteile · 1 / (1 − q)
    M' = (M − Preis) · (1 + preiswirkung · q)

### KI-Anleger

Am Monatsanfang legt eine KI-Firma, deren Kasse über `kasse_max_monate` laufender Kosten
liegt, `ki.depot_anteil_kasse` des Überschusses in Aktien an: in der notierten Firma mit
dem kleinsten M / fair, wenn das unter 1 − `ki.unterbewertung` liegt, bis
`ki.depot_anteil_max` dieser Firma. Sie verkauft eine Beteiligung ganz, sobald M / fair
über 1 + `ki.unterbewertung` steigt. Mit `ki.uebernahme_chance` · Aggressivität je Monat
bietet sie für die günstigste notierte Firma (kleinstes M / fair unter 1), deren Preis
höchstens `ki.uebernahme_kasse_anteil` ihrer Kasse ist.

## K4 – Investor und Bank

Lastenheft §17.3. Daten: `parameter/bank.yaml`. Kern: Modul `bank`.

### Spielweisen

Neben Werkstatt und Handelsniederlassung beginnt der Spieler als **Investmentfirma**
(ohne Standort; das Startkapital bleibt in der Kasse für Börse, Anleihen und Start-ups)
oder als **Bank** (ebenso ohne Standort, dazu die Bankgeschäfte unten). Töchter haben
neben „Produktion und Handel“ und „Logistik“ die Schwerpunkte **Investment** (sie legt
ihre Kasse wie ein KI-Anleger an, K3) und **Bank**.

### Einlagen

Eine Bank zahlt auf Einlagen den Zins r = Realzins(Jahr) + Einlagenaufschlag (vom
Spieler gesetzt). Die Anleger bringen ihr

    Kapazität = einlagen.hebel_max · max(Eigenkapital, 0)
    Ziel = Kapazität · clamp(0,5 + einlagen.elastizitaet · (Aufschlag − aufschlag_neutral), 0, 1)
    Einlagen' = Einlagen + einlagen.anpassung · (Ziel − Einlagen)     (Monatsende)

Zuflüsse erhöhen Kasse und Einlagen (Verbindlichkeit), Abflüsse senken beide – auch wenn
die Kasse dafür ins Minus geht (Bankrun). Zinsen: Einlagen · r / 12 je Monat
(Zinsaufwand). Ohne Bankgeschäft (Schwerpunkt gewechselt) fließen die Einlagen ab.

### Kredite an Firmen

Nimmt eine Firma einen Kredit auf, prüft sie die Banken des Spielers (nicht im eigenen
Konzern): Eine Bank gibt ihn, wenn

    Verschuldung des Kreditnehmers nach dem Kredit ≤ Verschuldung_max der Bank
    Kasse der Bank − mindestreserve · Einlagen ≥ Kreditbetrag

zum Zins i_Bank = i_Markt · (1 − Kreditnachlass), wobei i_Markt der Zins der Banken des
Marktes für diesen Kredit ist (M6). Unter mehreren Banken nimmt die Firma die günstigste.
Die Bank bucht den Kredit als Ausleihung (Anlagevermögen); Zinsen und Tilgungen des
Kreditnehmers fließen ihr zu (Zinsertrag bzw. Rückzahlung). Geht der Kreditnehmer pleite,
schreibt die Bank die Restschuld ab (Aufwand der Finanzanlagen).

## H1 – Ereignisfolgen

Lastenheft §4.1, §4.3, §17.4. Daten: Feld `wirkungen` der Ereignisse in `ereignisse/*.yaml`,
Parameter in `parameter/ereignisse.yaml`. Kern: Modul `events`.

Jedes historische Ereignis kann Wirkungen haben; sie gelten für Spieler und KI-Firmen
gleich. Länder einer Wirkung sind ihre `laender`, ohne Angabe die des Ereignisses.
Firmen „aus“ einem Land sind die, deren Konzernspitze dort ihren Sitz hat. Die Folgen
lassen sich beim neuen Spiel abschalten („Folgen historischer Ereignisse“); die Ereignisse
erscheinen dann nur als Meldung.

### Wirkungen mit Dauer

Sie gelten vom ersten Monatsersten am oder nach dem Ereignistag bis vor den ersten
Monatsersten am oder nach `bis` (ohne `bis` bis zum Spielende). Wie die Länderwerte gilt
der Wert des Monatsersten für den ganzen Monat. Gleichzeitige Faktoren derselben Art
werden multipliziert, Zollaufschläge addiert.

| Art | Felder | Wirkung |
|---|---|---|
| `nachfrage` | `warengruppen` (leer: alle), `konsum`, `staat` (je Standard 1) | Verbrauchernachfrage aller Einkommensschichten · konsum, Staatsnachfrage · staat |
| `handelssperre` | `gegen` | kein Handel zwischen jedem Land der Wirkung und jedem aus `gegen` (wie die Sperren in `zoelle.yaml`) |
| `zoll` | `gegen` (leer: alle anderen), `aufschlag` | Einfuhren in die Länder aus `gegen`: Zoll = Basis · Warengruppe · Zone + aufschlag |
| `arbeitskraefte` | `faktor` | verfügbare Arbeitskräfte jeder Gruppe · faktor (Einberufung); der Lohnaufschlag knapper Gruppen folgt M18b |
| `produktion` | `warengruppen` (leer: alle), `faktor` | Läufe je Anlage ≤ faktor · geplante Läufe an Standorten in den Ländern (Förderkürzung, Bürgerkrieg, Kriegsschäden); Engpass „Ereignis“ |
| `abschottung` | `alle` (Standard nein) | Firmen aus anderen Ländern gründen und kaufen dort keine Standorte und verlegen ihren Sitz nicht dorthin; mit `alle` gilt das auch für die eigenen Firmen des Landes außer dem Staatsbetrieb |

In einem Land, das nur Staatsbetriebe zulässt, sind neue KI-Firmen und – bei einem
Spielstart in dieser Zeit – die KI-Firmen der Startbesetzung Staatsbetriebe; die Firma des
Spielers bleibt privat (sie behält ihren Startstandort, wächst dort aber nicht).

Durchlaufende Verbrauchsgüter mit Faktor < 1 werden weniger gekauft; bei Gebrauchsgütern
sinkt der Bestand der Haushalte, und nach dem Ende holen sie den Rückstand mit der
üblichen Anschaffungsrate (M7) nach.

### Einmalige Wirkungen

Sie treten am Ereignistag ein (`boersenkrach` am folgenden Monatsersten, wenn die Börse
ihre Kurse bildet).

**Zerstörung** (`zerstoerung`, Feld `anteil`): An jedem Standort in den Ländern verliert
jede fertige Anlage mit n Einheiten

    zerstört = ⌊n · anteil⌋ + (1, wenn u < n · anteil − ⌊n · anteil⌋)

Einheiten (u gleichverteilt aus dem Zufallsstrom des Ereignisses); die Lagerbestände
sinken um den Anteil. Der Buchwert der zerstörten Einheiten (M22) und der Wert der
verlorenen Ware sind sonstiger Aufwand. Anlagen im Bau bleiben.

**Enteignung** (`enteignung`, Felder `nur_auslaendische` (Standard ja) und
`entschaedigung` (Anteil am Buchwert, Standard 0)): Betroffen sind alle Standorte in den
Ländern, deren Firma aus einem anderen Land ist – ohne `nur_auslaendische` alle außer
denen des Staatsbetriebs. Der bisherige Eigentümer bucht Anlagen, Anlagen im Bau,
Firmenwert, Lager und eigenen Grund aus und erhält

    Entschädigung = entschaedigung · Buchwert            (Kasse)
    sonstiger Aufwand = Buchwert − Entschädigung

Die Standorte gehen an den **Staatsbetrieb** des Landes: eine KI-Firma mit Sitz dort, die
beim ersten Mal gegründet wird (Name wie andere KI-Firmen des Landes, Fähigkeiten nach den
KI-Einstellungen). Er übernimmt die Buchwerte als Einlage (Eigenkapital), dazu vom Staat

    Betriebskapital = staatsbetrieb.betriebskapital_anteil · Buchwert   (Kasse, Einlage)

und die Technologien der übernommenen Rezepte und Anlagen. Staatsbetriebe gehen nicht an
die Börse (K1) und lassen sich daher nicht übernehmen (K3). Pachtgrundstücke bleiben
gepachtet, Konzessionen gehen mit dem Standort über, Lieferverträge der Standorte enden
(W4), Manager kehren in ihre Firma zurück (wie beim Verkauf, M30).

**Börsenkrach** (`boersenkrach`, Feld `einbruch`): Die Stimmung der Börse sinkt um
ln(1 − einbruch) (K1). Die bisherigen `krisen` in `parameter/boerse.yaml` sind in die
Ereignisse gewandert.

### Meldungen

Die Meldung eines Weltereignisses nennt seine Wirkungen je eine Zeile (Folge). Verliert
der Spieler durch Zerstörung oder Enteignung Anlagen oder Standorte, erhält er eine
Warnung mit dem Verlust.

## PE1 – Alter der Manager

Vorgabe `docs/PERSON.md` §3, §4. Daten: `parameter/lebenslauf.yaml`. Kern: Modul `aging`.

### Geburtsdatum

Jeder Manager hat ein Geburtsdatum; das Alter (volle Jahre) wird am Spieltag berechnet.
Ein neuer Kandidat bekommt eine **Ebene** nach seiner Stärke S (Mittel aus Fachwissen im
Schwerpunkt, Erkennen und Urteilsvermögen, MA1):

    Ebene = Standort, wenn S < grenze_land; Land, wenn S < grenze_kontinent;
            Kontinent, wenn S < grenze_vorstand; sonst Vorstand

und ein Eintrittsalter aus der Normalverteilung der Ebene (Mittel, Streuung), abgeschnitten
auf deren Spanne (Standort 26–50, Mittel 35; Land 32–55, 42; Kontinent 38–58, 47; Vorstand
42–62, 52). Der Tag im Jahr ist gleichverteilt. Alte Spielstände: beim Laden aus einem
eigenen Zufallsstrom des Managers, mit der Ebene seiner Stelle (ohne Stelle nach Stärke).
Dazu eine persönliche Abweichung vom Ruhestandsalter, gleichverteilt −5 bis +5 Jahre.

### Wirkung des Alters

- **Erfahrung** (MA6): Chance je Monat × 1,5 unter 35 Jahren, × 1,0 von 35 bis 54, × 0,5 ab 55.
- **Risikofreude** sinkt ab 40 Jahren um 0,3 Punkte je Jahr, höchstens um 10: am Geburtstag
  um ⌊0,3 · (Alter − 40)⌋ − ⌊0,3 · (Alter − 41)⌋ Punkte, bis 10 erreicht sind.
- **Abbau ab 65:** Am Geburtstag sinken Erkennen und Führung mit je 10 % Chance um einen
  Punkt. Urteilsvermögen und Fachwissen bleiben.
- Die Gehaltsforderung folgt weiter nur den Fähigkeiten.

### Ruhestand

    Ruhestandsalter = ruhestandsalter(Heimatland, Geburtsjahr + bezugsalter)
                      + persönliche Abweichung + vereinbarte Verlängerung
    Ruhestand = erster Monatserster an oder nach dem Tag, an dem er es erreicht

`ruhestandsalter` und `lebenserwartung` sind Länder-Zeitreihen (Standard für alle Länder,
eigene Reihen für einzelne Länder, linear zwischen den Jahren). Das Ruhestandsalter gilt
aus dem Jahr, in dem der Manager das Bezugsalter (65) erreicht; so verschiebt sich sein
Termin nicht mit jedem Jahr. Zwölf Monate vorher (`vorwarnung_monate`) stellt bei der
Spielerfirma die Personalstelle seiner Einheit oder einer Einheit darüber, sonst die
nächste besetzte Leitung darüber – ohne beide er selbst – das Anliegen **„Nachfolge
regeln“** (Thema `nachfolge`, Grund `ruhestand`, Frist bis zum Vortag des Ruhestands)
mit bis zu vier Optionen:

1. **Nachfolger suchen:** der stärkste freie Kandidat des Kontinents der Stelle (bei
   Fachstellen mit dem Schwerpunkt der Stelle, sonst beliebig) wird Nachfolger zu seiner
   Gehaltsforderung.
2. **Befördern:** der stärkste Manager der Firma auf einer Stelle darunter wird Nachfolger
   zum höheren aus bisherigem Gehalt und Forderung; seine bisherige Stelle wird sofort
   frei. Darunter liegen bei Fachstellen die Fachstellen derselben Fachrichtung in
   Einheiten darunter, bei Leitungen die Fachstellen der eigenen Einheit und die
   Leitungen der Einheiten darunter.
   Für 1 und 2 zählen nur Kandidaten, deren eigener Ruhestand später liegt als der
   Amtsantritt plus Vorwarnzeit.
3. **Verlängerung anbieten:** um die noch offenen Jahre bis `verlaengerung_jahre_max` (3)
   gegen 20 % mehr Gehalt (`verlaengerung_aufschlag`). Er sagt zu mit der Chance

       Zusage = Zufriedenheit / 100 · clamp(1 − (Alter − zusage_von) / (zusage_bis − zusage_von), 0, 1)

   mit `zusage_alter` = [60, 80]. Sagt er zu, verschiebt sich der Ruhestand; zwölf Monate
   vor dem neuen Termin wird wieder gefragt. Lehnt er ab, kommt zum nächsten Monatsersten
   ein neues Anliegen ohne diese Option.
4. **Unbesetzt lassen:** Er geht zum Termin, die Stelle bleibt leer.

Empfohlen wird die stärkere der Optionen 1 und 2, ohne Kandidaten die Verlängerung, sonst
„Unbesetzt lassen“. Hat der Spieler das Thema der fragenden Stelle abgegeben („Nicht mehr
fragen“), entscheidet sie selbst nach der Empfehlung und meldet es. Ein **Nachfolger**
sitzt bis zum Termin neben dem Amtsinhaber (beide Gehälter laufen), zählt aber nicht als
Inhaber der Stelle. Zum Ruhestand übernimmt er die Stelle mit den offenen Anliegen; ohne
Nachfolger bleibt sie leer, und die offenen Anliegen des Gegangenen gehen an die nächste
besetzte Stelle darüber (sonst verfallen sie). Fällt ein vorgesehener Nachfolger vorher
aus, wird erneut gefragt. Einen Nachfolger kann nur eine Stelle bekommen, deren Inhaber
innerhalb der Vorwarnzeit in den Ruhestand geht; eine freie Stelle besetzt der Befehl
sofort. KI-Firmen bekommen keine Anliegen: Ihre freien Stellen besetzt die KI wie bisher
(MA6).

### Tod

Die Sterbechance je Monat steigt ab 50 Jahren exponentiell:

    p(Alter) = 0,005 · 2^((Alter − Lebenserwartung) / verdopplung_jahre)    (Alter ≥ 50)

mit `lebenserwartung` des Heimatlandes (das Alter, in dem die Chance 0,5 % je Monat
erreicht) und `verdopplung_jahre` = 8. Ein Todesfall leert die Stelle sofort; ein
Nachfolger übernimmt, sonst kommt bei Spielerfirmen das Anliegen **„Stelle neu besetzen“**
(Thema `nachfolge`, Grund `unbesetzt`, Frist `frist_tage`; Optionen 1, 2 und 4, die
Besetzung gilt sofort), sofern eine besetzte Stelle darüber fragen kann.

### Pool und Ausgeschiedene

Freie Kandidaten altern mit; sie verlassen den Pool mit dem Ruhestand und sterben nach
derselben Tafel. Ausgeschiedene Manager verlassen den Bestand; die Firma behält sie im
Lebenslauf (Name, Geburtsdatum, letzte Stelle, von–bis, Trefferquote, Grund). Die
Zufallszahlen kommen aus eigenen Strömen je Manager: Geburt (einmal), Monat (Geburtstag,
Tod) und Tag der Verlängerungsfrage.

Reihenfolge am Monatsersten: Stellen ohne Firma enden, Anliegen werden aufgeräumt, freie
Kandidaten verlassen den Markt (MA6), dann Geburtstage, Tod und Ruhestand aller Manager,
erledigte Nachfolge-Anliegen schließen, neue Vorwarnungen; danach füllt sich der Pool, und
neue Kandidaten bekommen Potenzial und Geburtsdatum.

## PE2 – Spielerfigur als Person

Vorgabe `docs/PERSON.md` §5. Daten: `parameter/person.yaml`. Kern: Modul `person`.

### Person beim neuen Spiel

Name (leer: aus der Namensgruppe des Startlandes gezogen), Geburtsjahr (Vorgabe Startjahr −
`alter_start.standard` (30), erlaubt `von` 18 bis `bis` 60 Jahre alt; Tag im Jahr
gleichverteilt), Wohnsitz = Startland, verheiratet ja/nein (Vorgabe ja) und 0 bis 4 Kinder
(Vorgabe keine). Die Geburtstage vorhandener Kinder sind gleichverteilt zwischen dem Tag,
an dem die Person `kinder_ab` (22) wurde, und dem früheren von Spielbeginn und ihrem Tag
`kinder_bis` (45); ohne diese Spanne (zu junge Person) ist das Kind beim Start null Jahre
alt. Die Person ist zunächst CEO ihrer Firma.

Alte Spielstände: eine Standardperson mit gezogenem Namen, 30 Jahre am Ladetag (1. Januar
des Jahres), verheiratet, ohne Kinder, CEO der Firma.

### Kinder

Solange die Person verheiratet und `kinder_ab` bis unter `kinder_bis` Jahre alt ist und
weniger als `kinder_hoechstens` (4) Kinder hat, kommt am Monatsersten ein Kind mit der Chance

    p_Monat = 1 − (1 − kinder_chance_jahr)^(1/12)        (kinder_chance_jahr = 0,15)

Vorname aus der Namensgruppe des Wohnsitzlandes (bis zu fünf weitere Ziehungen, solange
der Vorname in der Familie schon vorkommt), Familienname der Person. Mit
`managerkarte_ab` (25) Jahren bekommt das Kind eine Managerkarte wie ein Bewerber (Ziehung
MA1, Geburtsdatum = das des Kindes, Heimat = Wohnsitz). Es steht nur der Person zur
Verfügung: Andere Firmen stellen es nicht ein und werben es nicht ab, Leitungen besetzen
mit ihm keine Stellen, es kündigt nicht und verlässt den Markt nur mit dem Ruhestand. Es
verlangt die übliche Gehaltsforderung. Stirbt es als Manager, gilt das Kind als verstorben.

### Rollen

- **Eigentümer:** jede Firma mit einem Anteil `Holder::Player`; kontrolliert ab über 50 %.
- **CEO:** Die Person führt ihre Firma, solange dort kein Manager CEO ist. Am Monatsersten
  wird abgeglichen: Ist ein Manager CEO geworden, gibt die Person den Vorsitz ab; ist die
  Stelle frei geworden, übernimmt sie ihn wieder. Beides steht im Lebenslauf.

### Lebenslauf

Chronik mit Datum: Beginn (Firma), Geburt eines Kindes, Berufseintritt eines Kindes
(Managerkarte), Tod eines Kindes, Vorsitz abgegeben (an wen), Vorsitz übernommen.
