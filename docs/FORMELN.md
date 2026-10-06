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
  Höchstförderung je Kalenderjahr und bis der Vorrat erschöpft ist. Die Jahresmenge
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
   nicht vom Spieler belegt) erhalten ihre Anlagen mit Anzahl max(1, round(Anzahl ·
   *s*)); ihre Leistung deckt Nachfrage, ihr Bedarf erhöht die der Vorprodukte.
3. **Bedarf:** Produkte werden so geordnet, dass jedes nach allen Produkten kommt, die
   es verbrauchen. Je Produkt und Land gilt Bedarf = Verbraucher- + Staatsnachfrage pro
   Tag + Vorproduktbedarf der schon geplanten Anlagen − Leistung realer Anlagen.
4. **Anlagen:** *F* = Bedarf · `marktdeckung` / Tagesleistung bei Vollleistung (M16;
   vorher Bedarf / (Tagesleistung · `start.auslastung`)); unter `anlage_mindestanteil`
   keine. Die Anlagen beginnen mit `start.auslastung`. Verteilung auf Länder mit Gewicht Bedarf ·
   Entwicklung^*w* (*w* je Produktart aus `gewicht_entwicklung`), bei Rohstoffen auf
   Lagerstätten mit freien Konzessionen nach Förderung / Kostenfaktor (höchstens die
   Förderung). Ganze Zahlen nach dem größten Rest, Summe round(*F*), mindestens 1.
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
  (vorher stellten alle Strumpfwerke auf Nylon um, während Nylon das Dreifache seines
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
  Restvorrat (Vorrat · *s* − Gefördertes) wenigstens `vorrat_jahre_min` Jahre ihrer
  Höchstförderung trägt; unter diesen die mit der billigsten Fracht in das
  Land mit der größten offenen Nachfrage (für Förderfirmen vorher die erste in den Daten).
- **Ausbau nach zwei Monaten:** Ein Standort gilt beim Ausbau (M10) als ausverkauft,
  wenn Verkauf seit Beginn des Vormonats plus eigener Verbrauch wenigstens neun
  Zehntel der Erzeugung dieser Tage erreichen (vorher nur der laufende Monat: Waren,
  die in wenigen Stücken verkauft werden wie Verkehrsflugzeuge, erschienen in manchen
  Monaten unverkauft).

Bei den Daten kommt hinzu: Kautschuk lässt sich auch synthetisch aus Ethylen und Benzin
herstellen (1937, `27_synthesekautschuk.yaml`), Nylon auch aus Kohle statt aus Benzin.
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
  und die Startformen bauen weiter mittelgroße Anlagen (die Daten sind darauf abgestimmt).

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
