# Vorgabe Welt und Geschichte, Markt und Nachfrage, Betrieb und Lernkurve

Vorgabe des Auftraggebers vom 11.10.2026 (Anfrage vom 10.10.2026: „kreativ und in die
Breite, was für das Spiel noch relevant sein könnte“; am 11.10.2026 wurden die drei
Themenfelder Welt und Geschichte, Markt und Nachfrage, Betrieb und Lernkurve zur
Umsetzung freigegeben). Meilensteine WG1–WG5, MN1–MN6, BP1–BP5 (Abschnitt 14).
Entscheidungen in Abschnitt 15; sie gelten vorläufig (🟡) nach dem Muster vom
08.10.2026: pauschal angenommen, Feedback nach Testpartien. Gilt bei Widerspruch vor
Lastenheft §4, §6.3, §6.4, §8.2, §9.1, §9.4 und §12; Entscheidungen im Lastenheft §18.9.

Diese Vorgabe **ersetzt** die Meilensteine W3 (Zölle) und H1 (Ereignisfolgen) aus
Architektur §4.1 und **ergänzt** H2 (Regulierung), H3 (Rüstung) und W4
(Lieferverträge); was sie dort vorwegnimmt, ist je Abschnitt genannt.

## 1 Zusammenfassung

Drei Felder machen die Welt lebendiger, ohne das Fundament zu ändern:

- **Welt und Geschichte (Teil A):** Die historischen Ereignisse bekommen **Wirkungen**
  aus den Datendateien (Handelssperren, Zerstörung, Zwangsproduktion, Enteignung,
  Nachfrageverschiebung, Arbeitskräftemangel, Rohstoffschocks). Dazu kommen
  **Konjunktur** als Stimmung der Käufer und Banken, **Handelsblöcke mit Zöllen und
  Embargos**, **Infrastruktursprünge** (Kanäle, Container, Internet), **Rohstofffunde**
  nach Jahr und eine **Zukunft 2027–2100** mit eigenen Treibern (Klima, Erschöpfung,
  Demografie), deren Ereignisse ein Zufallsgenerator aus denselben Datentypen erzeugt.
- **Markt und Nachfrage (Teil B):** **Substitution** auch bei Verbrauchsgütern,
  **Ausstattungsstufen** (einfach, Standard, Premium) je Produkt und Standort,
  **Ausschreibungen des Staates**, **Ruf** als zweite Markendimension mit Rückrufen,
  **Vertriebswege nach Epoche**, **Gebrauchtmarkt und Veralterung** sowie **Trends**
  als Nachfragewellen.
- **Betrieb und Lernkurve (Teil C):** **Lernkurve** je Standort und Produkt,
  **Instandhaltung** als Stellhebel mit Ausfällen und Großüberholung, **Lager mit
  Kosten, Kapazität und Verderblichkeit**, **Ausschuss und Arbeitsunfälle**,
  **Korruption und Rechtssicherheit** als Standortfaktoren, **Zulieferverträge mit
  Ausfallrisiko** und eine **Make-or-buy-Ansicht**.

Alles gilt für Spieler und KI-Firmen gleichermaßen (Lastenheft §10). Alle Zahlen sind
Parameter in `data/`; Ereignisse, Blöcke, Trends und Vertriebswege sind Daten, keine
Programmlogik. Nichts in dieser Vorgabe verändert die Länderjahreswerte: Die Einbrüche
durch Kriege und Krisen stecken schon dort (Lastenheft §18.1) und werden nicht ein
zweites Mal erzeugt.

## 2 Ausgangslage im Code (Stand 11.10.2026, nach PE6)

| Bereich | Heute | Lücke |
| --- | --- | --- |
| Ereignisse | `data/ereignisse/*.yaml` mit `id`, `datum`, `art`, `laender`; nur Meldung im Rundenbericht (`game.rs::world_events`, M28) | Keine Wirkung auf Märkte, Handel, Standorte |
| Konjunktur | Einstellung „Stärke der Konjunkturschwankungen“ (§15) ohne Wirkung; Einkommen folgen den Jahreswerten | Keine Stimmung, keine Kreditklemme, kein Aufschieben von Käufen |
| Zölle | `warengruppe` je Produkt, Kostenart `Customs` in `reports.rs`, W3 offen | Keine Zölle, keine Blöcke, keine Embargos |
| Infrastruktur | `verkehrsmittel.yaml` mit `verfuegbar_ab`, Seewege als Luftlinie · Umwegfaktor (M8) | Keine Kanäle, keine Kostensprünge |
| Lagerstätten | `entdeckt` (Jahr) vorhanden | Daten nutzen es kaum (Nordseeöl, Seltene Erden) |
| Zukunft | Projektionen der Jahreswerte bis 2100 in Fünfjahresschritten | Keine Ereignisse, keine Treiber nach 2026 |
| Verdrängung | `ersetzt` wirkt über die Besitzquote (Gebrauchsgüter) und den Staatsbedarf (M33) | Verbrauchsgüter und Industriegüter werden nicht verdrängt |
| Segmente | Qualität 0–100 wirkt je Schicht über `qualitaetsgewicht` | Kein bewusster Hebel für einfache oder Premium-Ausführung |
| Staat | Täglicher Bedarf je BIP, billigster Anbieter | Keine Lose, keine Ausschreibung, kein Heimatbonus |
| Marke | Bekanntheit 0–1 je Firma, Land, Warengruppe (M16) | Kein Ruf, keine Rückrufe, Werbemittel nur „das beste“ |
| Vertrieb | Präsenz über Niederlassung oder Händler (M16) | Keine Vertriebswege mit Reichweite und Kosten |
| Lebenszyklus | Weiterentwicklung in Stufen (M37), Nachfolgemodelle (M42/B1) | Keine Veralterung, kein Gebrauchtmarkt |
| Trends | Saison je Produkt | Keine Nachfragewellen |
| Lernkurve | Arbeitsstunden fest je Rezept; Größen­effekt über `anlagengroessen` (M36) | Keine Erfahrung je Standort und Produkt |
| Instandhaltung | Zustand sinkt linear, Wartung fest (`wartung_je_jahr`) | Kein Hebel, keine Ausfälle, keine Überholung |
| Lager | Unbegrenzt, kostenlos, ewig haltbar | Keine Kosten, Kapazität, Verderblichkeit |
| Qualität | Formel M5; Schulung (W1) | Kein Ausschuss, keine Unfälle |
| Standortfaktoren | Lohn, Steuern, Infrastruktur, `stabilitaet` (ungenutzt außer Anzeige) | Keine Korruption, keine Rechtssicherheit, kein Enteignungsrisiko |
| Lieferverträge | W4 offen | Keine Verträge, kein Ausfallrisiko |
| Make or buy | Strategiefeld „Eigenfertigung oder Zukauf“ (MA4) | Keine Vergleichsansicht |

---

# Teil A – Welt und Geschichte

## 3 Ereigniswirkungen (ersetzt H1)

Jedes Ereignis in `data/ereignisse/` darf eine Liste `wirkungen` tragen. Ohne `wirkungen`
bleibt es eine Meldung wie bisher. Eine Wirkung hat Art, Dauer und Zielmenge; mehrere
Wirkungen desselben Ereignisses laufen parallel.

Gemeinsame Felder jeder Wirkung:

| Feld | Bedeutung |
| --- | --- |
| **art** | eine der Arten unten |
| ab | Beginn, `"JJJJ-MM-TT"`; ohne Angabe das Datum des Ereignisses |
| **bis** | Ende (einschließlich) oder `dauer_tage` (eins von beiden) |
| laender | Betroffene Länder; ohne Angabe die Länder des Ereignisses |
| anlauf_tage, auslauf_tage | Die Stärke steigt linear an und klingt linear ab (Standard 0 / 0) |
| annaeherung, quelle | Herkunft |

Arten und ihre eigenen Felder:

| art | Felder | Wirkung |
| --- | --- | --- |
| `handelssperre` | `gegen` (Länder oder Block), `warengruppen` (leer = alle), `richtung` (`beide`, `ausfuhr`, `einfuhr`) | Händler und Firmen bewegen keine Ware dieser Gruppen zwischen den Seiten; laufende Lieferverträge zwischen ihnen ruhen ohne Strafe |
| `nachfrage` | `warengruppen`, `faktor` (0–10), optional `schichten` (Fünftel 1–5) | Faktor auf die Endkunden-Kaufneigung `S_q` dieser Gruppen (Rüstung statt Konsum: Konsumgüter 0,6) |
| `staatsbedarf` | `warengruppen`, `faktor` | Faktor auf den Staatsbedarf; ersetzt den bisherigen festen `kriegsfaktor` (der bleibt als Standard, wenn ein Ereignis der Art `krieg` keine eigene Wirkung nennt) |
| `zerstoerung` | `anteil` (0–1), optional `standorttypen` | Zu Beginn verliert jeder Standort der Länder zufällig (Zufallsstrom „ereignisse“) `anteil` seiner Einheiten; Zustand der übrigen −0,2; Lager −`anteil`. Buchung als außerordentlicher Aufwand. Danach **Wiederaufbau**: Ersatzbau kostet in den Ländern für `dauer_tage` nur `wiederaufbau_kostenfaktor` (Daten, 0,7) |
| `zwangsproduktion` | `warengruppen_ziel` (Rüstung), `anteil` (0–1) | Werke der Länder, die ein Produkt einer Zielgruppe fertigen könnten (bekanntes Rezept auf vorhandener Anlage), müssen `anteil` ihrer Kapazität dafür einsetzen; der Staat kauft zum Richtpreis · `zwangspreis` (Daten, 0,9). Ohne passendes Rezept: `anteil` der Kapazität ruht (Material und Arbeit werden abgezogen) |
| `enteignung` | `betroffen` (`auslaendisch` oder `alle`), `entschaedigung` (0–1) | Standorte fremder (Sitz in einem anderen Land) bzw. aller Firmen gehen an den Staat (Versteigerung wie M38 nach `dauer_tage`, Erlös an den Staat); die Firma erhält `entschaedigung` · Buchwert. Kuba 1960, Iran 1951, Russland 1918 |
| `arbeitskraefte` | `faktor` (0–1), optional `qualifikationen` | Pool der Gruppen × Faktor (Einberufung) |
| `rohstoffpreis` | `produkte`, `faktor`, `weltweit: true` | Staatsmarkt- und Richtpreise × Faktor (Ölkrise 1973: Rohöl 3,5, weltweit); Märkte folgen über die Preisanpassung |
| `produktionsausfall` | `anteil`, optional `branchen` | Pandemie, Erdbeben: Durchläufe aller Anlagen × (1 − `anteil`) |
| `boersenstimmung` | `faktor` | Vormerkung für K1 (Börse): Bewertungsfaktor; bis K1 ohne Wirkung, die Prüfung akzeptiert das Feld |
| `kredit` | `faktor` | Faktor auf Kreditrahmen und Zins­aufschlag (Bankenpanik 1907, 2008): Rahmen × `faktor`, Zins + (1 − `faktor`) · `kreditklemme_zins` (Daten) |
| `infrastruktur` | siehe Abschnitt 6 | Kanal, Kostensprung, Werbemittel |
| `regulierung` | Vormerkung für H2 | Wird mit H2 ausgefüllt; die Prüfung akzeptiert das Feld |

Regeln:

- Historische Jahreswerte bleiben unverändert (Lastenheft §18.1). Die Wirkungen betreffen
  nur, was nicht in den Jahreswerten steckt: Handel, Standorte, Nachfragestruktur,
  Arbeitskräfte, Preise einzelner Güter, Kredite.
- Wirkungen auf Spieler und KI-Firmen sind gleich. Die KI reagiert über ihre
  bestehenden Regeln (Preise, Stilllegung, Ausbau); neu nur: Sie bietet in
  Zwangsproduktion nicht aus und baut in Ländern mit laufender `zerstoerung` nicht neu.
- Jede beginnende und endende Wirkung ist eine Meldung mit eigener Art („Weltereignis“)
  und Verweis auf die betroffenen Standorte; die Weltkarte bekommt die Ebene
  „Konflikte und Sperren“ (Lastenheft §3.3).
- Ereignisse 1900–2026 werden mit Wirkungen nachgepflegt (WG1, Abschnitt 14;
  mindestens Weltkriege, 1929, 1907, Ölkrisen, Suezkrise, Kalter-Krieg-Embargo,
  Enteignungen Russland/Kuba/Iran, 2008, 2020, zwei Erdbeben, Spanische Grippe).
- Determinismus: Zufallsstrom `ereignisse`, Reihenfolge nach `id`.

## 4 Konjunktur

Die Jahreswerte tragen den realen Verlauf des Einkommens; die Konjunktur ergänzt die
**Stimmung**, die Kaufentscheidungen und Kredite kurzfristig verschiebt, und gibt der
Einstellung „Stärke der Konjunkturschwankungen“ (§15) endlich Sinn.

    Wachstum g(Land, Monat) = BIP je Kopf (Jahreswert, interpoliert) / Wert vor 12 Monaten − 1
    Stimmung k(Land) = begrenzt(g / g_bezug, −1, +1),   g_bezug = `konjunktur.bezug_wachstum` (0,04)
    Konjunkturfaktor K = 1 + Stärke · k · gewicht_klasse

- `gewicht_klasse` (Daten): Grundbedarf 0,05, Gebrauchsgut 0,4, Luxus 0,7. Im
  Abschwung schieben Haushalte Gebrauchs- und Luxusgüter auf; Grundbedarf bleibt.
- Wirkung: `S_q` × K (Endkunden), Staatsbedarf × (1 + 0,5 · Stärke · k), Kreditrahmen ×
  (1 + 0,5 · Stärke · k), Zinsaufschlag + max(0, −k) · Stärke · `kreditklemme_zins`.
- Stärke = Einstellung 0–2 (Standard 1); 0 schaltet die Konjunktur ab.
- Übersicht und Markt zeigen je Land ein Konjunkturbarometer (−1 bis +1) und
  „Aufschwung/Abschwung seit …“; der Rundenbericht meldet den Vorzeichenwechsel.
- Nach 2026 kommt zum Verlauf der Projektion ein **erzeugter Zyklus**: Sinus mit Periode
  `zyklus_jahre` (7, ±2 Jahre zufällig je Zyklus, Zufallsstrom `konjunktur`) und
  Amplitude `zyklus_amplitude` (0,03) auf g, je Kontinent um bis zu zwei Jahre versetzt.

## 5 Handelsblöcke, Zölle, Embargos (ersetzt W3)

Daten in `data/handel/bloecke.yaml` und `data/handel/zoelle.yaml`.

**Blöcke** (`bloecke`): `id`, `mitglieder` als Zeitreihe von Länderlisten (`{1958: [BEL,
DEU, FRA, ITA, LUX, NLD], 1973: [...]}`), `binnenzoll` (Faktor auf den Zoll zwischen
Mitgliedern, 0 = Zollunion), `aussenzoll` (Aufschlag der Mitglieder nach außen, optional).
Vorgesehen: Britisches Empire/Commonwealth-Präferenz, Französisches Kolonialreich,
Comecon, EWG/EU, EFTA, NAFTA/USMCA, Mercosur, ASEAN; ab 2027 keine neuen.

**Zölle** (`zoelle`): Liste von Regeln `{ab, bis, von, nach, warengruppen, satz}`:

- `von`, `nach`: Land, Block oder `alle`; `warengruppen` leer = alle.
- `satz` 0–3 als Anteil am Warenwert (Richtpreis im Zielland), Jahreswerte erlaubt.
- Auflösung je Einfuhr: speziellste Regel gewinnt (Land vor Block vor `alle`,
  Warengruppe vor leer, jüngeres `ab` vor älterem).
- Grundverlauf aus den Daten: Weltdurchschnitt 1900 ≈ 0,15, Spitze um 1932 (Smoot-Hawley
  USA 0,45, Antwortzölle), Senkung über GATT-Runden bis 2000 ≈ 0,04; alles mit
  `annaeherung: true`, wo nicht recherchiert.

**Embargos** sind Ereigniswirkungen `handelssperre` (Abschnitt 3); dauerhafte Sperren
wie CoCom (1949–1994, Warengruppen Hochtechnologie gegen Comecon) stehen als
Ereignis mit langer Dauer.

Wirkung im Spiel:

- Zoll = Satz · Richtpreis im Zielland · Menge, gezahlt vom Importeur (Firma oder
  Händler) bei Grenzübertritt; Kostenart `Customs`. Händler schlagen ihn auf ihren
  Preis auf.
- Einfuhr ins eigene Werk zahlt den Zoll des Ziellands; Lieferung an ein Konzernwerk
  ebenfalls (keine Konzernbefreiung).
- **Dynamik nach 2026** (Einstellung „Dynamik von Zöllen“, 0–2): jährlich mit Chance
  `zolldynamik.chance` · Dynamik wählt der Generator ein Länderpaar oder einen Block und
  verschiebt den Satz um ±`zolldynamik.schritt` (Zufallsstrom `handel`); Meldung an alle
  Firmen mit Lieferungen auf dieser Strecke. Dynamik 0 = Sätze bleiben.
- Weltkarte: Ebene „Zölle und Handelsbeschränkungen“ für ein gewähltes Produkt vom
  gewählten Land aus (Lastenheft §3.3). Controlling: Zölle je Route (§14.2).
- KI-Firmen rechnen Zoll in ihre Einstandsschätzung (Händler C1, Ausbau in neuen Märkten
  C3) ein; sie bauen eher Werke hinter hohen Zollmauern (Zoll­faktor im Ausbauvergleich).

## 6 Infrastruktursprünge

Wirkungsart `infrastruktur` (Abschnitt 3) mit `typ`:

| typ | Felder | Wirkung |
| --- | --- | --- |
| `kanal` | `id` (`panama`, `suez`, `kiel`), `gesperrt: true/false` | Seewege: `data/transport/kanaele.yaml` listet je Kanal die Länderpaare bzw. Kontinentpaare, deren Seeweg um `verkuerzung` (Anteil, Panama: Ostküste USA–Westküste/Asien 0,45) kürzer wird; `gesperrt` nimmt den Kanal zeitweise heraus (Suez 1956–57, 1967–75) |
| `kostensprung` | `verkehrsmittel`, `faktor` | Kosten je tkm × Faktor ab dem Datum (Container 1966: `seeschiff` Stückgut 0,5 bis 1980 schrittweise; ersetzt bzw. ergänzt die Jahreswerte in `verkehrsmittel.yaml`, die bleiben die Grundlage) |
| `werbemittel` | `id` | Schaltet ein Werbemittel frei (Internet 1995) – die Liste `marktmodell.marke.werbemittel` bleibt, das Ereignis macht den Moment sichtbar |
| `vertriebsweg` | `id` | Schaltet einen Vertriebsweg frei (Abschnitt 11), zusätzlich zu `verfuegbar_ab` |

Kanäle sind Daten, keine Sonderlogik: M8 rechnet Seewege nach Luftlinie · Umwegfaktor;
ein offener Kanal setzt für die betroffenen Paare einen eigenen Umwegfaktor.

## 7 Rohstofffunde und Erschöpfung

- `entdeckt` wird in `data/lagerstaetten/` konsequent gepflegt: Nordseeöl 1969, Seltene
  Erden China ab 1985 im großen Maßstab, Ghawar 1948, Ölsande Kanada 1967, Kupfer Chile
  (Chuquicamata 1910), Bauxit Guinea 1950er, Lithium Chile 1980er, Fracking USA 2008
  (als zweite Lagerstätte `usa_schiefer` mit `entdeckt: 2008` und eigenem
  `foerderkosten_faktor`).
- Eine Entdeckung ist eine Meldung „Neue Lagerstätte“ an alle Firmen mit Bergbau
  (Rundenbericht, Weltkarte).
- Nach 2026 erzeugt der Generator (Abschnitt 8) seltene Funde (`fund`): neue
  Lagerstätte eines Rohstoffs in einem Land, das ihn schon fördert, Vorrat 10–50 % der
  größten bestehenden, `foerderkosten_faktor` 1,2–2,0.

## 8 Zukunft 2027–2100 und Zufallsgenerator

Ab 2027 gibt es keine Jahreswerte mehr, nur Projektionen und einen Generator. Der
Generator (Stufe 6 im Lastenheft; die **Datenstruktur und die Wirkungen** entstehen
jetzt, der Generator selbst als WG5) erzeugt Ereignisse derselben Datentypen wie
Abschnitt 3. Vorlagen stehen in `data/ereignisse/vorlagen.yaml`:

| Vorlage | Häufigkeit je Jahr weltweit (Standard) | Wirkungen |
| --- | --- | --- |
| `rezession` | 0,15 (Konjunktur übernimmt den Rest) | `kredit` 0,7 für 18 Monate, `nachfrage` Luxus 0,8 |
| `konflikt` | 0,2 · (1 − Stabilität der Beteiligten) | `handelssperre`, `zerstoerung` 0,05–0,2, `staatsbedarf` Rüstung 3, `arbeitskraefte` 0,9 |
| `katastrophe` | 0,5, gewichtet mit `klima.katastrophen_faktor(Jahr)` | `produktionsausfall` 0,3 für 2–6 Monate in 1–3 Ländern |
| `pandemie` | 0,03 | `produktionsausfall` 0,15 weltweit, `nachfrage` Gebrauchsgüter 0,8 |
| `umbruch` | 0,1 · (1 − Stabilität) | `enteignung` oder Wechsel der Stabilität um ±0,2 für 10 Jahre |
| `fund` | 0,3 | neue Lagerstätte (Abschnitt 7) |
| `zollrunde` | 0,2 | Zollsatz eines Blocks ±0,05 (Abschnitt 5) |

Klima und Erschöpfung als Treiber (Daten `data/parameter/zukunftsmodell.yaml`):

- `klima.katastrophen_faktor`: Jahreswerte 2026: 1,0 → 2100: 2,5 (annaeherung).
- `klima.ertragsfaktor` je Kontinent: Jahreswerte auf `foerderung_max_je_jahr`
  erneuerbarer Lagerstätten (Getreide, Baumwolle): Afrika/Südasien bis 0,8, Nordeuropa
  bis 1,1.
- `klima.kueste`: ab 2060 zählen Standorte in Küstenstädten (Stadt mit `kueste: true`,
  aus Natural Earth) bei `katastrophe` doppelt.
- Erschöpfung: braucht nichts Neues – endliche Vorräte (C2) und `foerderindex`, der nach
  2026 nicht mehr steigt. Die Datenprüfung warnt, wenn ein Rohstoff bis 2100 bei
  historischem Verbrauch weltweit ausgeht, als Hinweis für die Balance.
- Demografie: liegt in den Projektionen der Jahreswerte (Bevölkerung, BIP je Kopf).

Einstellung „Häufigkeit und Stärke zufälliger Ereignisse ab 2027“ (§15) skaliert
Häufigkeit (0–3) und die Stärke der Wirkungen (0,5–2). Zufallsstrom `zukunft`;
erzeugte Ereignisse stehen im Spielstand mit allen Feldern wie historische und
erscheinen in der Chronik.

---

# Teil B – Markt und Nachfrage

## 9 Substitution und Ausstattungsstufen

### 9.1 Substitution bei Verbrauchs- und Industriegütern

`ersetzt` wirkt heute nur über Besitzquoten. Neu für Verbrauchsgüter und Ergänzungsgüter
(Petroleum → Strom, Kohle → Heizöl, Butter → Margarine):

    Verfügbarkeit v_B(Land) = Anteil der Nachfrage nach B im Land, der im letzten Jahr bedient wurde (0–1)
    Preisvorteil π = ln(r_A · Nutzen_A / (p_B / Nutzen_B)),  Nutzen aus den Daten (Standard 1)
    Zielanteil z = v_B · σ(`substitution.steilheit` · π)       (logistisch)
    Anteil s_B ← s_B + (z − s_B) · `substitution.je_jahr` / 12    (monatlich)
    Nachfrage A × (1 − s_B), Nachfrage B × (1 + s_B · Bedarf_A / Bedarf_B)

- `ersetzt` darf auf Verbrauchs- und Ergänzungsgüter zeigen; je Eintrag optional
  `nutzen` (Vorteil des Nachfolgers, z. B. Glühlampe 2,0 gegen Petroleumlampe).
- Industriegüter: keine eigene Nachfragelogik – Verfahren (mehrere Rezepte je Produkt,
  M22) bilden den Wechsel der Eingangsstoffe ab. Neu nur: Die KI wechselt das Rezept
  einer Anlage, wenn ein anderes bekanntes Rezept auf derselben Anlage dauerhaft (zwei
  Quartale) günstiger ist (Thema `produktion`, Regel „günstigstes Verfahren“).

### 9.2 Ausstattungsstufen

Je Endprodukt und Standort wählt die Firma eine Ausstattung (Befehl
`SetTrim { site, product, trim }`), Standard `standard`:

| Stufe | Material | Arbeit | Qualität | Zielgruppe |
| --- | --- | --- | --- | --- |
| `einfach` | × 0,85 | × 0,9 | −15 Punkte | Fünftel 1–2 |
| `standard` | × 1 | × 1 | ±0 | alle |
| `premium` | × 1,25 | × 1,3 | +15 Punkte | Fünftel 4–5 |

(Faktoren in `produktionsmodell.ausstattung`.) Ein Standort fertigt ein Produkt in einer
Ausstattung; die Ware trägt die Stufe als Merkmal, der Markt führt je Stufe eine Qualität
und einen Preis, die Anbieterwahl (M7/M16) bleibt dieselbe. Das Strategiefeld
„Qualität“ (MA4) bekommt die Einstellung „Ausstattung je Warengruppe“; KI-Firmen wählen
nach ihrer Marktausrichtung (Kompetenz, `kimodell.ausstattung`).

## 10 Kundentypen: Ausschreibungen und Ruf

### 10.1 Ausschreibungen des Staates

Der tägliche Staatsbedarf (M7) läuft weiter für Kleinbedarf. Für Warengruppen mit
`ausschreibung: true` (Schienen, Rüstung, Fahrzeuge, Baustoffe, Flugzeuge) sammelt der
Staat seinen Bedarf und schreibt **Lose** aus:

- Jedes Quartal je Land und Warengruppe ein Los über den Bedarf des nächsten Jahres,
  Frist 30 Tage; Anliegen „Ausschreibung“ an alle Firmen mit Produkt im Land oder mit
  Lieferung dorthin.
- Gebot: Preis je Einheit, Menge (Teillose erlaubt), Lieferzeitraum.
- Zuschlag nach Punktwert = −Preisgewicht · ln(p / r) + Qualitätsgewicht · (Q − 50)/25 +
  `heimatbonus` (Firma mit Werk im Land, Daten 0,3; in Kriegszeiten 1,0) +
  `staatsbonus_ruf` · Ruf. Gewichte aus `marktmodell.ausschreibung`.
- Der Zuschlag ist ein Liefervertrag (W4) mit Vertragsstrafe; W4 liefert die Mechanik,
  die Ausschreibung ist ihr erster Nutzer.
- KI-Firmen bieten mit ihrer Preisregel + Zuschlag nach Kompetenz (Schätzfehler B2).

### 10.2 Ruf

Neben der Bekanntheit (M16) führt jede Firma je Land einen **Ruf** 0–1 (Start 0,5):

- Steigt monatlich mit Qualität über dem Marktdurchschnitt
  (`ruf.gewinn_je_monat` · (Q − Q̄)/50), sinkt bei Qualität darunter, bei Rückrufen, bei
  Arbeitsunfällen (Abschnitt 13.4) und später bei Umweltverstößen (H2).
- **Rückruf:** Liegt die Qualität einer Lieferung an Endkunden unter
  `ruf.rueckruf_schwelle` (Daten, 25), tritt mit Chance `ruf.rueckruf_chance` ein
  Rückruf ein: Kosten = Verkäufe des Monats zu Herstellkosten, Ruf −0,1, Meldung.
- Wirkung: Anbieterwahl + `rufgewicht_q` · (Ruf − 0,5) analog zur Marke; Staat und
  Ausschreibung (10.1); Gehaltsforderungen der Manager −/+ 5 % bei Ruf unter 0,3 / über
  0,8.
- Werbung wirkt nicht auf den Ruf. Ein Vertriebsweg `markenladen` (Abschnitt 11) hebt
  den Ruf­gewinn um 50 %.
- Ansicht Markt zeigt Bekanntheit und Ruf nebeneinander; Erklärungen (M27) nennen die
  Ursachen des letzten Jahres.

## 11 Vertriebswege nach Epoche

Daten `data/vertriebswege.yaml`:

| Feld | Bedeutung |
| --- | --- |
| **id** | `handelsvertreter`, `kaufhaus`, `versandhandel`, `fachhandel`, `franchise`, `markenladen`, `discounter`, `e_commerce` |
| **verfuegbar_ab** | Jahr (Kaufhaus 1870, Versandhandel 1890, Franchise 1950, Discounter 1960, E-Commerce 1995) |
| **warengruppen** | Welche Gruppen darüber verkauft werden |
| **reichweite** | Anteil der Haushalte des Landes, den der Weg erreicht, Jahreswerte (E-Commerce: 0,05 in 1995 → 0,9 in 2026); je Schicht × `schichtfaktor` (Versandhandel: Land stärker als Stadt → Fünftel 1–3 höher) |
| **fixkosten_je_land_usd** | Jahreswerte, × Preisniveau |
| **kosten_je_verkauf** | Anteil am Verkaufspreis (Kaufhaus 0,25, E-Commerce 0,12) |
| ruf_faktor | Faktor auf den Rufgewinn (Markenladen 1,5, Discounter 0,7) |

- Je Land und Warengruppe wählt eine Firma einen oder mehrere Wege (Befehl
  `SetChannels { country, group, channels }`); ohne Wahl gilt der günstigste verfügbare
  mit voller Reichweite (heutiges Verhalten).
- **Präsenz** (M16) wird zur Summe der Reichweiten der gewählten Wege (höchstens 1),
  statt 1 bei Niederlassung. Händler (KI) nutzen ihren eigenen Weg (`fachhandel`).
- Das Strategiefeld „Verkaufswege“ erhält die Einstellung je Warengruppe; die
  Vertriebsstelle (MA2) entscheidet unter dem Budget.
- KI-Firmen: Kompetenz entscheidet, ob sie neue Wege im Jahr ihrer Verfügbarkeit oder
  erst nach `kimodell.vertrieb_verzoegerung_jahre` nutzen.

## 12 Lebenszyklus, Gebrauchtmarkt, Trends

### 12.1 Veralterung

Für Gebrauchsgüter mit Entwicklungsstufen (M37): Die Qualität einer Ware sinkt in der
Anbieterwahl um `veralterung.punkte_je_stufe` (5) je Stufe, die der Marktführer im Land
voraus ist. Veraltete Lagerware verliert dadurch Käufer; die Firma senkt den Preis oder
verschrottet (M22-Befehl `Scrap`).

### 12.2 Gebrauchtmarkt

Für Gebrauchsgüter mit `gebraucht: true` (Autos, Fahrräder, Nähmaschinen, Möbel):

- Jeder Ersatzkauf einer Schicht q ≥ 2 gibt mit Anteil `gebraucht.weitergabe` (0,6) ein
  Gerät auf den Gebrauchtmarkt des Landes (Bestand je Produkt, altert `gebraucht.
  lebensdauer_anteil` · Nutzungsdauer, dann weg).
- Erstkäufer der Fünftel 1–3 decken zuerst `gebraucht.anteil_erstkauf` (0,5) ihres
  Bedarfs aus dem Bestand, solange er reicht – ohne Umsatz für Firmen. Die Besitzquote
  steigt dadurch schneller (Ersatzkäufe später).
- Ansicht Markt: „davon Gebrauchtmarkt“ je Land.

### 12.3 Ersatzteile und Service

Keine neue Mechanik: Ersatzteile sind Ergänzungsgüter (`ergaenzung.zu`) mit eigener Kette;
WG-Datenpflege ergänzt je Fahrzeug-/Maschinenkette ein Ersatzteil-Produkt
(`ersatzteile_<produkt>`), das vom Hersteller des Geräts mit Vorteil in der Anbieterwahl
verkauft wird (`ersatzteil_bonus`, 0,5 Punkte wenn dieselbe Firma das Gerät verkauft hat;
der Markt führt den Marktanteil des Vorjahres als Näherung).

### 12.4 Trends

Daten `data/trends.yaml`: `{id, warengruppen, laender (leer = alle), verlauf: {Jahr:
Faktor}}` auf die Kaufneigung `S_q` (Fahrrad-Boom 1895–1900: 1,6 → 1,0; Kunststoff
1950er; Heimcomputer 1983; Fitness 1980er; Elektroauto 2018–2026). Nach 2026 erzeugt der
Generator Trends: je Jahr mit Chance 0,3 eine Warengruppe, Faktor 1,2–1,8 über 3–8
Jahre, dann zurück (Zufallsstrom `trends`). Meldung „Nachfragewelle“ zu Beginn.

---

# Teil C – Betrieb und Lernkurve

## 13 Produktion

### 13.1 Lernkurve

Je Standort und Produkt wird die **kumulierte Menge** `n` gezählt (Spielstand, `i64`
Einheiten). Arbeitsstunden je Durchlauf:

    Lernfaktor L = max(L_min, (max(n, n_0) / n_0) ^ (−b)),  b = −log₂(Lernrate)

- `Lernrate` je Branche in `data/parameter/produktionsmodell.yaml` (`lernkurve.rate`):
  Flugzeugbau 0,80, Fahrzeuge 0,85, Elektronik 0,85, Maschinenbau 0,88, Metallurgie und
  Chemie 0,92, Rohstoffe und Lebensmittel 0,97. `n_0` = Jahresausstoß einer mittleren
  Anlage (aus den Anlagendaten), `L_min` = 0,5.
- Der Lernfaktor wirkt auf die Arbeitsstunden (Rezept) und auf die Ausschussquote
  (13.4), nicht auf Material und Strom. Er multipliziert sich mit Schulung (W1) und
  Größe (M36).
- **Vergessen:** Je Monat ohne Produktion des Produkts am Standort sinkt n um
  `lernkurve.vergessen_je_monat` (3 %); bei Entlassung von mehr als einem Drittel der
  Belegschaft um 10 % einmalig. Ein neu gebautes Werk startet bei n = 0, ein gekauftes
  (M30) übernimmt n des Verkäufers.
- **Produktfamilien:** Wechselt ein Standort auf ein Nachfolgeprodukt (`ersetzt` oder
  nächste Entwicklungsstufe M37), startet n beim Anteil `lernkurve.uebertrag` (0,5) des
  Vorgängers.
- Anzeige: Werksansicht zeigt je Produkt „Erfahrung“ als Faktor und die Stückkosten vor
  und nach Erfahrung; Controlling trennt den Lernvorteil als eigene Zeile der
  Stückkosten (M18). Die Einstandsschätzung der KI (B2) rechnet L ein.
- Skaleneffekte bleiben, wie sie sind (Anlagengrößen M36, Gemeinkosten je Durchlauf).

### 13.2 Instandhaltung, Ausfälle, Großüberholung

Instandhaltung wird zum Hebel je Standort (Befehl `SetMaintenance { site, level }`,
Stufe 0,5–1,5 der Datenwartung, Standard 1,0; Strategiefeld „Investitionen“ bekommt
„Instandhaltung“):

    Zustandsverlust je Jahr = Grundverlust · (1 / Stufe)^`instandhaltung.exponent` (1,5)
    Ausfallchance je Anlage und Tag = `ausfall.basis` · ((1 − Zustand) / (1 − Zustand_min))^2

- Ein Ausfall stoppt die Anlage für `ausfall.dauer_tage` (3–14, Zufallsstrom
  `betrieb`) und kostet `ausfall.kosten_anteil` (0,02) der Investition; Meldung
  „Anlagenausfall“, bei mehr als drei im Jahr Anliegen „Instandhaltung erhöhen oder
  überholen“ der Produktionsstelle.
- **Großüberholung** (Befehl `Overhaul { site, facility }`): Kosten `ueberholung.
  kostenanteil` (0,3) der Investition, Dauer `ueberholung.tage` (30), Zustand danach 1,0
  und die Lebensdauer verlängert sich um `ueberholung.lebensdauer_anteil` (0,5). Lohnt
  sich für große, bezahlte Anlagen; neu kaufen bleibt die Alternative.
- Zustand wirkt neu auch auf die **Kapazität**: Durchläufe × (0,7 + 0,3 · Zustand).
- KI: Instandhaltung = 0,8 + 0,5 · Kompetenz; überholt, wenn Überholung billiger als
  Neubau über die Restlebensdauer.

### 13.3 Lager: Kosten, Kapazität, Verderblichkeit

- **Lagerklasse** je Produkt (`lagerklasse`: `schuettgut`, `stueckgut`, `fluessig`,
  `gekuehlt`, `gefahrgut`) mit Kosten je Einheit und Tag in
  `data/parameter/lagermodell.yaml` (× Preisniveau), gebucht als Kostenart „Lager“.
  Rohstoffe und Halbzeuge liegen billig, Gekühltes teuer.
- **Kapazität** je Standort: `lagerplatz` aus der Grundstücksfläche (M35) – je ha
  `lagermodell.einheiten_je_ha` nach Lagerklasse – plus Lagergebäude (neue Anlage
  `lagerhalle` in fünf Größen, Standorttyp `lager` wird dadurch sinnvoll). Über der
  Kapazität: Fremdlager zum doppelten Satz (kein Produktionsstopp, nur Kosten) und
  Warnung.
- **Verderblichkeit:** `haltbarkeit_tage` je Produkt (Getreide 365, Mehl 180, Konserven
  1 800, Frischware 14). Lager wird in Chargen nach Eingangsdatum geführt (FIFO);
  abgelaufene Chargen werden entsorgt (Aufwand), Meldung bei mehr als 5 % im Monat.
  Produkte ohne Angabe verderben nicht.
- KI: Lagerziel (`lager_ziel_tage`) wird bei verderblicher Ware auf die halbe
  Haltbarkeit begrenzt.

### 13.4 Ausschuss und Arbeitsunfälle

    Ausschussquote a = `ausschuss.basis` (0,04) · (1 − Schulung · 0,5) · (1,5 − Zustand · 0,5) · L^0,5 · (1 − Automatisierung · 0,3), begrenzt 0,005–0,3

- Ausschuss mindert den Ausstoß je Durchlauf; die Kosten bleiben (Herstellkosten
  steigen). Werksansicht und Controlling zeigen die Quote; Erklärung (M27) die Ursachen.
- **Arbeitsunfälle:** Chance je 100 000 Arbeitsstunden = `unfall.basis(Jahr)` (Jahreswerte,
  1900: 8 → 2026: 0,5) · (1 − Schulung · 0,4) · (1,5 − Zustand · 0,5) · Landesfaktor
  `arbeitsschutz` (Zeitreihe je Land, Standard 1; H2 füllt sie mit Arbeitsschutzgesetzen).
  Ein Unfall kostet `unfall.kosten` (ein Jahreslohn der Gruppe, × Preisniveau),
  Produktionsausfall 1 Tag an der Anlage, Ruf −0,02 (Abschnitt 10.2); Statistik im
  Controlling. Zufallsstrom `betrieb`.

### 13.5 Standortfaktoren: Korruption und Rechtssicherheit

Neue Länder-Zeitreihen (`laender.py`, Quelle TI/World Bank ab 1995, davor
`annaeherung` aus BIP und Stabilität): `korruption` 0–1, `rechtssicherheit` 0–1.

- Bauzeit × (1 + `korruption` · `korruption.bauzeit`) (0,5); Genehmigungskosten je
  Neubau = `korruption` · `korruption.gebuehr` · Investition (0,05).
- **Forderung:** jährlich mit Chance `korruption` · 0,3 je Standort Anliegen „Zahlung
  verlangt“: zahlen (Kosten 0,5 % der Investition, Buchung als „sonstige Abgaben“) oder
  ablehnen (Standort ruht `korruption.verzoegerung_tage` (30)). Zahlen ist ohne
  Strafe; H2 kann später Compliance-Regeln nachrüsten.
- **Enteignungsrisiko** (ohne Ereignis): jährlich Chance (1 − `rechtssicherheit`) ·
  (1 − `stabilitaet`) · `enteignung.basis` (0,02) je Land, dass ausländische Standorte
  enteignet werden (Wirkung wie Abschnitt 3, Entschädigung `rechtssicherheit`).
- Weltkarte: Ebenen „Korruption“ und „Rechtssicherheit“; die KI gewichtet beide im
  Ausbauvergleich (Zuschlag auf die Investition wie oben, Risikoabschlag).

### 13.6 Zulieferverträge mit Ausfallrisiko (ergänzt W4)

W4 liefert Verträge mit Menge, Preis, Laufzeit, Mindestqualität, Vertragsstrafe. Dazu:

- `exklusiv: true`: Der Verkäufer darf das Produkt im Land während der Laufzeit an
  keinen anderen liefern; Preis­aufschlag erwartet (KI verlangt +10 %).
- **Lieferausfall:** Kann der Verkäufer nicht liefern (Lager leer, Ausfall 13.2,
  Ereignis), zahlt er die Vertragsstrafe; der Käufer deckt sich am Spotmarkt ein. Drei
  Ausfälle in Folge: Käufer darf kündigen. Anliegen „Lieferant fällt aus“ der
  Einkaufsstelle mit Optionen (abwarten, Spotmarkt, zweiter Lieferant).
- **Sicherheitsbestand** als Strategiefeld „Lager“ ist schon da (Mindestbestand); neu:
  Vorschlag der Einkaufsstelle, den Mindestbestand zu erhöhen, wenn ein Lieferant im
  Jahr zweimal ausfiel („just in time“ gegen Puffer wird so erlebbar).

### 13.7 Make or buy

Ansicht Controlling → „Eigenfertigung oder Zukauf“ je Vorprodukt und Standort:

| Spalte | Inhalt |
| --- | --- |
| Eigene Stückkosten | Herstellkosten heute, mit Lernkurve und Auslastung; bei fehlender Anlage: Schätzung mit Investition über die Lebensdauer |
| Zukauf | Marktpreis im Land + Transport + Zoll vom günstigsten Markt |
| Versorgung | Anteil der Monate des letzten Jahres mit Knappheit im Markt |
| Empfehlung | billiger und sicher → so lassen; Zukauf billiger, aber Versorgung < 80 % → „Puffer“; Eigenfertigung teurer um mehr als `make_or_buy.toleranz` (10 %) → „Zukauf prüfen“ |

Das Strategiefeld „Eigenfertigung oder Zukauf“ (MA4) mit Stellung „nach Kosten“ nutzt
dieselbe Rechnung; die Einkaufs- und Produktionsstelle setzen sie um (Anliegen bei
Investition über Budget).

---

# Gemeinsames

## 14 Meilensteine

Reihenfolge: nach PE6. Zuerst Betrieb (verändert die Kostenbasis, auf die alles andere
aufsetzt), dann Markt, dann Welt. W4 (Lieferverträge) kommt vor MN2 und BP4, weil beide
darauf bauen; W3 und H1 entfallen.

| Nr. | Inhalt | Abnahme |
| --- | --- | --- |
| BP1 | Lernkurve je Standort und Produkt, Vergessen, Übertrag; Anzeige und Controlling-Zeile; KI-Schätzung | Szenario: zwei gleiche Werke, eines mit 5 Jahren Vorsprung, Stückkosten unterscheiden sich im erwarteten Bereich (Lernrate 0,85: −30 % bei 8-facher Menge); Weltlauf 1900–1960 ohne Preisverfall unter Materialkosten; Determinismus grün; alte Spielstände laden (n = 0) |
| BP2 | Instandhaltungsstufe, Ausfälle, Großüberholung, Zustand wirkt auf Kapazität; Strategiefeld; KI | Szenario: Stufe 0,5 ergibt binnen 10 Jahren mehr Ausfälle als 1,5; Überholung stellt Zustand 1,0 her; Weltlauf: KI-Firmen überholen |
| BP3 | Lagerklassen, Lagerkosten, Kapazität mit Lagerhalle, Verderblichkeit mit Chargen; KI-Lagerziel | Datenprüfung: jedes Produkt hat eine Lagerklasse; Szenario: Getreide ohne Absatz verdirbt nach einem Jahr; Weltlauf: Lagerkosten unter 3 % des Umsatzes im Mittel |
| BP4 | Ausschuss, Arbeitsunfälle, Korruption und Rechtssicherheit mit Zeitreihen, Enteignungsrisiko, Zulieferverträge mit Ausfall und Exklusivität (nach W4), Make-or-buy-Ansicht | Zeitreihen für alle Länder vorhanden; Szenario: Lieferausfall löst Strafe und Anliegen aus; Make-or-buy zeigt für Stahl im Startland plausible Spalten |
| BP5 | Balance-Runde Betrieb: Richtpreise, Margen und KI-Ausbau mit Lernkurve, Lager und Ausschuss nachziehen | Balance-Protokoll 1900–1930 im bisherigen Zielkorridor (Nacharbeit M16) |
| MN1 | Substitution bei Verbrauchs- und Ergänzungsgütern, KI-Verfahrenswechsel, Trends aus Daten | Weltlauf: Petroleumlampe verliert bis 1930 über 80 % der Nachfrage in elektrifizierten Ländern; Trend-Daten geprüft |
| MN2 | Ausstattungsstufen, Ausschreibungen des Staates (nach W4), Ruf mit Rückrufen | Szenario: Premium gewinnt Fünftel 5, einfach Fünftel 1; Ausschreibung Schienen vergibt ein Los mit Vertrag; Rückruf senkt Ruf und Absatz |
| MN3 | Vertriebswege mit Reichweite und Kosten, Präsenz neu, Strategiefeld, KI | Datenprüfung der Wege; Weltlauf: E-Commerce nach 1995 übernimmt Anteile; Umsatz der KI-Händler bleibt plausibel |
| MN4 | Veralterung, Gebrauchtmarkt, Ersatzteile als Ergänzungsgüter (Datenpflege), Trend-Generator ab 2027 | Szenario: Autos – Gebrauchtmarkt deckt in Fünftel 1–3 einen Teil der Erstkäufe; Weltlauf 2027–2060 erzeugt Trends |
| MN5 | Anzeige: Markt mit Ruf, Segmenten, Vertriebswegen, Gebrauchtmarkt; Erklärungen; Einführung ergänzt | Oberflächentests; Erklärung nennt alle Faktoren der Anbieterwahl |
| MN6 | Balance-Runde Markt | Weltlauf 1900–2026 mit Marktanteilen, die historischen Größenordnungen nahekommen (Protokoll) |
| WG1 | Ereigniswirkungen: Datentyp, Prüfregeln, alle Arten außer `infrastruktur`; Daten für die Ereignisse 1900–2026 nachgepflegt; Karte „Konflikte und Sperren“ | Datenprüfung aller Wirkungen; Weltlauf 1900–1950: Zerstörung und Wiederaufbau in Europa sichtbar, Rüstungsbedarf 1914–18 und 1939–45 steigt; Spieler-Werk in Frankreich 1940 betroffen |
| WG2 | Konjunktur mit Barometer, Kreditwirkung, Einstellung; erzeugter Zyklus ab 2027 | Szenario 1929–1933 USA: Gebrauchsgüter-Nachfrage sinkt stärker als Grundbedarf; Stärke 0 reproduziert den Lauf vor WG2 |
| WG3 | Handelsblöcke, Zölle, Embargos; Kostenart, Karte, Controlling; KI-Standortwahl; Dynamik ab 2027 | Zölle 1900–2026 als Daten; Szenario: Zollmauer 1932 lässt KI-Werke hinter der Mauer entstehen; Comecon-Embargo stoppt Hochtechnologie-Lieferungen |
| WG4 | Infrastruktursprünge (Kanäle, Kostensprünge, Werbemittel, Vertriebswege als Ereignis), Rohstofffunde als Meldung, Datenpflege `entdeckt` | Route Hamburg–San Francisco wird 1914 kürzer; Containerisierung senkt Seefrachtkosten; Nordseeöl ab 1969 nutzbar |
| WG5 | Zufallsgenerator ab 2027 mit Vorlagen, Klima-Treiber, Küstenregel, Einstellungen; Chronik | Weltlauf 2027–2100 mit Ereignissen in plausibler Zahl (Protokoll: Häufigkeit je Vorlage); Determinismus grün; Häufigkeit 0 ergibt keinen Eintrag |

Formeln je Meilenstein in `docs/FORMELN.md`, Datenformat in `docs/DATENFORMAT.md`,
Fortschritt in `docs/FORTSCHRITT.md`, eigenständige Entscheidungen in
`docs/OFFENE_PUNKTE.md`, Abschnitt R. Spielstände: Jede neue Größe bekommt beim Laden
alter Stände einen Standardwert (Lernmenge 0, Instandhaltung 1,0, Ruf 0,5, Ausstattung
`standard`, Lager ohne Chargen → eine Charge mit Ladedatum).

## 15 Entscheidungen vom 11.10.2026 (🟡 vorläufig, Feedback nach Testpartien)

1. Die Länderjahreswerte bleiben die alleinige Quelle für Einkommen und Bevölkerung;
   Ereignisse und Konjunktur wirken nur auf das, was dort nicht steckt.
2. Konjunktur ist Stimmung (Kaufneigung, Kredite), abgeleitet aus dem Wachstum der
   Jahreswerte; nach 2026 ein erzeugter Zyklus. Stärke 0 schaltet sie ab.
3. Zölle sind Daten je Regel mit Vorrang der speziellsten Regel; Händler und Firmen
   zahlen beim Grenzübertritt, keine Konzernbefreiung.
4. Embargos und Handelssperren sind Ereigniswirkungen, auch die langen (CoCom).
5. Enteignungen entschädigen anteilig; Standorte werden versteigert wie bei Pleiten.
6. Zwangsproduktion kauft der Staat zu 90 % des Richtpreises; wer kein Rüstungsrezept
   hat, verliert den Anteil der Kapazität.
7. Infrastruktur ist Daten: Kanäle als Umwegfaktor je Paar, Kostensprünge als Faktor.
8. Ab 2027 erzeugt ein Generator Ereignisse aus Vorlagen; Klima erhöht Katastrophen und
   senkt Erträge in warmen Regionen; Küstenstandorte sind ab 2060 stärker betroffen.
9. Substitution bei Verbrauchsgütern logistisch nach Preis und Verfügbarkeit; bei
   Industriegütern über den Verfahrenswechsel (kein neuer Datentyp).
10. Drei Ausstattungsstufen je Produkt und Standort; keine freie Produktgestaltung.
11. Ausschreibungen nur für Warengruppen mit `ausschreibung: true`; Zuschlag mit
    Heimatbonus, in Kriegen stark; Ergebnis ist ein Liefervertrag.
12. Ruf ist eine zweite Größe neben der Bekanntheit; Werbung wirkt nicht auf ihn.
13. Vertriebswege ersetzen die Präsenz 0/1 durch Reichweiten; ohne Wahl gilt das
    heutige Verhalten.
14. Gebrauchtmarkt nur für gekennzeichnete Gebrauchsgüter, ohne Umsatz für Firmen.
15. Ersatzteile sind Ergänzungsgüter (Datenpflege) mit Bonus für den Gerätehersteller.
16. Lernkurve mit Lernrate je Branche, Untergrenze 0,5, Vergessen bei Stillstand,
    Übertrag 50 % auf Nachfolger; sie wirkt auf Arbeit und Ausschuss, nicht auf Material.
17. Instandhaltung 0,5–1,5 als Stufe; Ausfälle quadratisch mit dem Zustand;
    Großüberholung zu 30 % der Investition.
18. Lagerkosten je Lagerklasse, Kapazität aus Fläche plus Lagerhalle, Fremdlager zum
    doppelten Satz; Verderblichkeit nur mit `haltbarkeit_tage`, FIFO-Chargen.
19. Ausschuss und Unfälle hängen an Schulung, Zustand, Erfahrung, Automatisierung und
    Arbeitsschutz; Unfälle treffen Kosten und Ruf.
20. Korruption: Bauzeit, Gebühren, gelegentliche Forderungen ohne Strafe für das Zahlen;
    Enteignungsrisiko aus Rechtssicherheit und Stabilität.
21. Make or buy ist zuerst eine Ansicht; die Strategie „nach Kosten“ nutzt dieselbe
    Rechnung.
22. Reihenfolge BP → MN → WG nach PE6; W3 und H1 entfallen zugunsten dieser Vorgabe,
    W4 wird vor MN2/BP4 eingeschoben.
23. Alle neuen Zahlen sind Näherungen (`annaeherung: true`) bis zur Balance-Runde.
