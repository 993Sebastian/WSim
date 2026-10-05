//! Every kind of data error produces an understandable message at the right place.

use std::collections::BTreeMap;

use wsim_data::{Finding, LoadOutcome, Severity, Source, load_sources};

const META: &str = "meta:\n  datenversion: 1\n";

const GRUNDLAGEN: &str = "\
einheiten:
  - id: t
    gewicht_kg: 1000
  - id: stueck
kontinente:
  - id: europa
branchen:
  - id: bergbau
  - id: metallurgie
warengruppen:
  - id: erze
transportklassen:
  - id: schuettgut
qualifikationen:
  - id: ungelernt
    stufe: 1
    mit_fachrichtung: false
  - id: fachkraft
    stufe: 3
    mit_fachrichtung: true
fachrichtungen:
  - id: metall
  - id: bergbau
";

const LAND: &str = "\
laender:
  - id: SWE
    kontinent: europa
    flaeche_km2: 450_000
    hauptstadt: {breite: 59.33, laenge: 18.07}
    binnenland: false
    werte:
      bevoelkerung: {1900: 5_000_000, 1930: 6_000_000}
      bip_je_kopf_usd: {1900: 6_000}
      gini: {1900: 0.45}
";

const MODELL: &str = "\
laendermodell:
  preisniveau: {referenzland: SWE, elastizitaet: 0.35, minimum: 0.2, maximum: 1.5}
  erwerbsquote: 0.42
  lohnquote: 0.6
  jahresarbeitsstunden: {1900: 2900}
  qualifikationsanteile:
    - {bip_je_kopf_usd: 1_000, anteile: {ungelernt: 0.9, fachkraft: 0.1}}
    - {bip_je_kopf_usd: 50_000, anteile: {ungelernt: 0.4, fachkraft: 0.6}}
  lohnabstand:
    - {bip_je_kopf_usd: 5_000, faktoren: {ungelernt: 0.8, fachkraft: 1.5}}
  fachrichtungsanteile:
    fachkraft: {metall: 0.6, bergbau: 0.4}
  strompreis_usd_je_mwh: {1900: 2000}
  stromnetz: {1900: 0.1}
  stromnetz_bezug_usd: 13_000
  steuer_unternehmen: {1900: 0.04}
  steuer_dividenden: {1900: 0.03}
  entwicklung: {von_usd: 1_300, bis_usd: 65_000}
  verkehrstraeger:
    schiene: {1900: 1}
    strasse: {1900: 0.05}
    luft: {1900: 0}
    hafen: {1900: 1}
  stabilitaet: 0.8
  forschung: {bezug_usd: 26_000, elastizitaet: 0.3, minimum: 0.3, maximum: 1.5}
  produktivitaet: {bezug_usd: 10_000, elastizitaet: 1.0, minimum: 0.1, maximum: 4.0}
  automatisierung: {basis: 0.3, je_verdopplung: 0.12, bezug_usd: 13_000}
";

const PRODUKTION: &str = "\
produktionsmodell:
  standortkosten_usd: {foerderstaette: 1, werk: 1, kraftwerk: 1, lager: 1, niederlassung: 1, forschungszentrum: 1}
  gebaeude_lebensdauer_jahre: 50
  erschliessung_lebensdauer_jahre: 30
  automatisierung: {arbeitsersparnis: 0.8, kostenanteil: 0.5}
  qualitaet: {vorprodukte: 0.3, automatisierung: 10, zustand: 20}
  zustand_minimum: 0.2
  lohnaufschlag_max: 1.0
  einspeiseverguetung: 0.5
  gemeinkosten_anteil: {rohstoff: 0.25, halbzeug: 0.5, komponente: 0.5, endprodukt: 1.0, energie: 0}
  richtpreis_marge: {minimum: 0.05, maximum: 0.45}
  nebenprodukte_lager_tage: 90
  startformen:
    werkstatt:
      standorttyp: werk
      gebaeude_usd: 20_000
      anlagen:
        - {anlage: ofen}
      einkauf:
        - {produkt: erz, ziel: 5, hoechstpreis_usd: 20}
      verkauf: [eisen]
    handel:
      standorttyp: niederlassung
      gebaeude_usd: 15_000
";

const FINANZEN: &str = "\
finanzmodell:
  realzins: {1900: 0.03}
  risikoaufschlag: {minimum: 0.01, je_verschuldung: 0.08}
  beleihung: 0.6
  dispo: {anteil: 0.1, aufschlag: 0.06}
  laufzeit_max_jahre: 30
";

const MARKT: &str = "\
marktmodell:
  preisgewicht: [2.0, 1.6, 1.2, 0.9, 0.6]
  qualitaetsgewicht: [0.3, 0.5, 0.8, 1.1, 1.5]
  aneignung_je_jahr: 0.25
  preisanpassung: {hoch: 0.02, runter: 0.01, lagertage: 30, auslastung_normal: 0.85, hoechstfaktor: 20}
  staat_hoechstpreis: 1.5
  preisniveau_anteil: {rohstoff: 0.2, halbzeug: 0.1, komponente: 0.1, endprodukt: 0.4, energie: 1}
  index_glaettung: 0.1
  haendler: {marge: 0.05, vorrat_tage: 30, glaettung_tage: 30}
  marke:
    markengewicht: [0.4, 0.6, 0.9, 1.2, 1.5]
    vergessen_je_monat: 0.03
    mundpropaganda: 0.08
    kosten_je_einwohner_usd: 0.05
    bekanntheit_start: 0.5
    bekanntheit_start_real: 0.7
    bekanntheit_handel: 0.2
    bekanntheit_staatsmarkt: 0.3
    werbemittel:
      - {id: zeitung, ab: 1800, wirkung: 1.0}
";

const TRANSPORT: &str = "\
transportmodell:
  umweg: {land: 1.3, see: 1.4, luft: 1.05}
  umschlag: {kosten_usd_je_t: 5, tage: 2}
  mindestinfrastruktur: 0.05
";

const FORSCHUNG: &str = "\
forschungsmodell:
  vorgriff_faktor: 1.25
  nachzuegler: {rabatt_je_jahr: 0.1, minimum: 0.2}
  gemeingut_nach_jahren: 25
  forscher: fachkraft
  sachkosten_usd_je_forschertag: 40
";

const VERKEHR: &str = "\
verkehrsmittel:
  - id: fuhrwerk
    weg: gelaende
    verfuegbar_ab: 1800
    transportklassen: [schuettgut]
    kosten_usd_je_tkm: {1900: 3.0}
    km_je_tag: {1900: 25}
";

const KETTE: &str = "\
produkte:
  - id: erz
    art: rohstoff
    branche: bergbau
    einheit: t
    verwendung: industrie
    warengruppe: erze
    transportklasse: schuettgut
    richtpreis_usd: 60
  - id: eisen
    art: halbzeug
    branche: metallurgie
    einheit: t
    verwendung: industrie
    warengruppe: erze
    transportklasse: schuettgut
    richtpreis_usd: 400
anlagen:
  - id: mine
    standorttyp: foerderstaette
    investition_usd: 1_000_000
    bauzeit_tage: 100
    kapazitaet_je_tag: 10
    lebensdauer_jahre: 20
    wartung_je_jahr: 0.05
    automatisierung_max: 0.1
  - id: ofen
    standorttyp: werk
    investition_usd: 2_000_000
    bauzeit_tage: 100
    kapazitaet_je_tag: 10
    lebensdauer_jahre: 20
    wartung_je_jahr: 0.05
    automatisierung_max: 0.1
rezepte:
  - id: erz_abbau
    produkt: erz
    menge: 1
    dauer_tage: 1
    anlage: mine
    abbau: true
    arbeit_stunden:
      ungelernt: 2
    qualitaet_basis: 50
  - id: eisen_schmelzen
    produkt: eisen
    menge: 1
    dauer_tage: 1
    anlage: ofen
    technologie: schmelzen
    eingang:
      erz: 2
    arbeit_stunden:
      fachkraft.metall: 1.5
    qualitaet_basis: 60
technologien:
  - id: schmelzen
    fachgebiet: metall
    erfindungsjahr: 1800
lagerstaetten:
  - id: grube
    land: SWE
    rohstoff: erz
    vorrat: 1_000_000
    erschliessung: {investition_usd: 500_000, dauer_tage: 100}
    foerderung_max_je_jahr: 10_000
";

const KI: &str = include_str!("../../../data/parameter/kimodell.yaml");

const KI_NAMEN: &str = "\
namensgruppen:
  - id: schwedisch
    laender: [SWE]
    standard: true
    familiennamen: [Berg]
    orte: [Kiruna]
    rechtsformen: [AB]
    muster: [\"{familienname} {branche} {rechtsform}\"]
    branchen: {bergbau: Gruv}
";

const KI_FIRMA: &str = "\
reale_firmen:
  - id: lkab
    name: LKAB
    sitz: SWE
    gegruendet: 1890
    standorte:
      - land: SWE
        lagerstaette: grube
        anlagen: [{anlage: mine, anzahl: 2}]
      - land: SWE
        anlagen: [{anlage: ofen, anzahl: 1, rezept: eisen_schmelzen}]
";

const WAEHRUNG: &str = "\
preisindex:
  leitwaehrung: dollar
  basisjahr: 2026
  teuerung_danach: 0.02
  werte: {1900: 8.4, 2026: 330}
waehrungen:
  - id: dollar
    zeichen: USD
    kurse: {1900: 1}
  - id: krone
    zeichen: kr
    kurse: {1900: 3.73, \"1923-11\": 4.0}
  - id: krone_neu
    zeichen: kr
    bindung: {an: krone, faktor: 2}
landeswaehrungen:
  - land: SWE
    perioden:
      - {ab: 1900, waehrung: krone}
      - {ab: \"1999-01\", waehrung: krone_neu}
";

const TEXTE: &str = "\
waehrung.dollar: Dollar
waehrung.krone: Krone
waehrung.krone_neu: Neue Krone
schwierigkeit.leicht: Leicht
schwierigkeit.mittel: Mittel
schwierigkeit.schwer: Schwer
einheit.t: t
einheit.stueck: Stück
kontinent.europa: Europa
branche.bergbau: Bergbau
branche.metallurgie: Metallurgie
warengruppe.erze: Erze
werbemittel.zeitung: Zeitung
transportklasse.schuettgut: Schüttgut
qualifikation.ungelernt: Ungelernte
qualifikation.fachkraft: Fachkräfte
fachrichtung.metall: Metall
fachrichtung.bergbau: Bergbau
land.SWE: Schweden
produkt.erz: Erz
produkt.eisen: Eisen
anlage.mine: Mine
anlage.ofen: Ofen
rezept.erz_abbau: Erz fördern
rezept.eisen_schmelzen: Eisen schmelzen
technologie.schmelzen: Schmelzen
lagerstaette.grube: Grube
verkehrsmittel.fuhrwerk: Fuhrwerk
";

struct Daten {
    files: BTreeMap<String, String>,
}

impl Daten {
    fn neu() -> Self {
        let files = [
            ("meta.yaml", META),
            ("grundlagen.yaml", GRUNDLAGEN),
            ("laender/SWE.yaml", LAND),
            ("parameter/laendermodell.yaml", MODELL),
            ("parameter/produktionsmodell.yaml", PRODUKTION),
            ("parameter/finanzmodell.yaml", FINANZEN),
            ("parameter/marktmodell.yaml", MARKT),
            ("parameter/transportmodell.yaml", TRANSPORT),
            ("parameter/forschungsmodell.yaml", FORSCHUNG),
            ("verkehrsmittel.yaml", VERKEHR),
            ("waehrungen/a.yaml", WAEHRUNG),
            ("ketten/a.yaml", KETTE),
            ("parameter/kimodell.yaml", KI),
            ("ki/a.yaml", KI_NAMEN),
            ("texte/de/a.yaml", TEXTE),
        ];
        Self {
            files: files
                .iter()
                .map(|(p, t)| ((*p).to_owned(), (*t).to_owned()))
                .collect(),
        }
    }

    fn ersetze(mut self, file: &str, alt: &str, neu: &str) -> Self {
        let text = self.files.get_mut(file).expect("Datei existiert");
        assert!(text.contains(alt), "„{alt}“ nicht in {file}");
        *text = text.replacen(alt, neu, 1);
        self
    }

    fn datei(mut self, file: &str, text: &str) -> Self {
        self.files.insert(file.to_owned(), text.to_owned());
        self
    }

    fn ohne(mut self, file: &str) -> Self {
        self.files.remove(file);
        self
    }

    fn laden(&self) -> LoadOutcome {
        load_sources(self.files.iter().map(|(p, t)| Source::new(p, t)).collect())
    }

    /// Line number (1-based) of the first occurrence of `needle` in `file`.
    fn zeile(&self, file: &str, needle: &str) -> u32 {
        let text = &self.files[file];
        let offset = text
            .find(needle)
            .unwrap_or_else(|| panic!("„{needle}“ nicht in {file}"));
        u32::try_from(text[..offset].matches('\n').count() + 1).unwrap()
    }
}

fn alle(outcome: &LoadOutcome) -> String {
    outcome
        .report
        .findings()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

/// The only finding containing `text`; fails with the full report otherwise.
fn befund<'a>(outcome: &'a LoadOutcome, text: &str) -> &'a Finding {
    let matches: Vec<_> = outcome
        .report
        .findings()
        .iter()
        .filter(|f| f.message.contains(text))
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "„{text}“ nicht genau einmal gefunden in:\n{}",
        alle(outcome)
    );
    matches[0]
}

fn assert_ort(finding: &Finding, file: &str, line: u32, path: &str) {
    assert_eq!(finding.file.as_deref(), Some(file), "{finding}");
    assert_eq!(finding.position.map(|p| p.line), Some(line), "{finding}");
    assert_eq!(finding.path.to_string(), path, "{finding}");
}

fn nur_fehler(outcome: &LoadOutcome, anzahl: usize) {
    assert_eq!(outcome.report.errors().count(), anzahl, "{}", alle(outcome));
    assert!(
        outcome.data.is_none(),
        "Bei Fehlern darf es keinen Katalog geben"
    );
}

#[test]
fn grunddaten_laden_ohne_befunde() {
    let outcome = Daten::neu().laden();
    assert!(outcome.report.findings().is_empty(), "{}", alle(&outcome));
    let data = outcome.data.unwrap();
    assert_eq!(data.catalog.products.len(), 2);
    assert_eq!(data.catalog.labor_groups.len(), 3);
}

#[test]
fn waehrungen_werden_umgerechnet() {
    let data = Daten::neu().laden().data.unwrap();
    let m = &data.catalog.currencies;
    let swe = data.catalog.countries.id("SWE").unwrap();
    // 2026: die neue Krone, fest an die alte gebunden (2 je Krone).
    let heute = m.at_base(swe).unwrap();
    assert_eq!(heute.currency, "krone_neu");
    assert!((heute.factor - 8.0).abs() < 1e-9);
    // 1900: die alte Krone, mit der Teuerung seit 1900.
    let damals = m.at_time(swe, 1900.5).unwrap();
    assert_eq!(damals.currency, "krone");
    assert!((damals.factor - 3.73 * 8.4 / 330.0).abs() < 1e-9);
}

#[test]
fn waehrung_mit_ungueltigem_zeitpunkt() {
    let d = Daten::neu().ersetze("waehrungen/a.yaml", "\"1923-11\"", "\"1923-13\"");
    let outcome = d.laden();
    let f = befund(&outcome, "„1923-13“ ist kein gültiger Zeitpunkt");
    assert_ort(
        f,
        "waehrungen/a.yaml",
        d.zeile("waehrungen/a.yaml", "\"1923-13\""),
        "waehrungen[1].kurse.1923-13",
    );
    nur_fehler(&outcome, 1);
}

#[test]
fn waehrung_mit_kursen_und_bindung() {
    let d = Daten::neu().ersetze(
        "waehrungen/a.yaml",
        "    bindung: {an: krone, faktor: 2}",
        "    bindung: {an: krone, faktor: 2}\n    kurse: {1900: 1}",
    );
    let outcome = d.laden();
    befund(&outcome, "Genau eines von „kurse“");
    nur_fehler(&outcome, 1);
}

#[test]
fn waehrung_an_gebundene_waehrung_gebunden() {
    let d = Daten::neu().datei(
        "waehrungen/b.yaml",
        "waehrungen:\n  - id: krone_neuer\n    zeichen: kr\n    bindung: {an: krone_neu, faktor: 1}\n",
    );
    let outcome = d.laden();
    befund(&outcome, "„krone_neu“ ist selbst gebunden");
}

#[test]
fn waehrung_negativer_kurs_und_unbekannte_waehrung() {
    let d = Daten::neu()
        .ersetze("waehrungen/a.yaml", "{1900: 3.73,", "{1900: -3.73,")
        .ersetze(
            "waehrungen/a.yaml",
            "waehrung: krone_neu}",
            "waehrung: krona_neu}",
        );
    let outcome = d.laden();
    befund(&outcome, "Wert -3.73 muss größer als 0 sein");
    let f = befund(&outcome, "krona_neu");
    assert!(f.message.contains("krone_neu"), "Vorschlag fehlt: {f}");
    nur_fehler(&outcome, 2);
}

#[test]
fn land_ohne_waehrung_und_spaeter_beginn() {
    let ohne = Daten::neu().ersetze("waehrungen/a.yaml", "  - land: SWE", "  - land: XXX");
    let outcome = ohne.laden();
    befund(&outcome, "Für das Land „SWE“ fehlt ein Eintrag");
    let spaet = Daten::neu().ersetze(
        "waehrungen/a.yaml",
        "      - {ab: 1900, waehrung: krone}",
        "      - {ab: 1920, waehrung: krone}",
    );
    let outcome = spaet.laden();
    befund(&outcome, "Der erste Zeitraum beginnt erst 1920");
    nur_fehler(&outcome, 1);
}

#[test]
fn zeitraeume_absteigend_und_basisjahr_ohne_preis() {
    let d = Daten::neu()
        .ersetze("waehrungen/a.yaml", "{ab: \"1999-01\"", "{ab: \"1899-01\"")
        .ersetze("waehrungen/a.yaml", "2026: 330}", "2025: 330}");
    let outcome = d.laden();
    befund(&outcome, "nach „ab“ aufsteigend sortiert");
    befund(&outcome, "Für das Basisjahr 2026 fehlt ein Wert");
}

#[test]
fn leitwaehrung_braucht_kurs_eins() {
    let d = Daten::neu().ersetze(
        "waehrungen/a.yaml",
        "kurse: {1900: 1}",
        "kurse: {1900: 1.2}",
    );
    let outcome = d.laden();
    befund(&outcome, "Leitwährung „dollar“");
    nur_fehler(&outcome, 1);
}

#[test]
fn unbenutzte_waehrung_ist_eine_warnung() {
    let d = Daten::neu()
        .datei(
            "waehrungen/b.yaml",
            "waehrungen:\n  - id: taler\n    zeichen: Tlr\n    kurse: {1900: 4}\n",
        )
        .ersetze(
            "texte/de/a.yaml",
            "waehrung.dollar: Dollar",
            "waehrung.dollar: Dollar\nwaehrung.taler: Taler",
        );
    let outcome = d.laden();
    let f = befund(&outcome, "„taler“ wird von keinem Land verwendet");
    assert_eq!(f.severity, Severity::Warning);
    assert!(outcome.data.is_some());
}

#[test]
fn yaml_syntaxfehler() {
    let d = Daten::neu().ersetze("ketten/a.yaml", "      erz: 2\n", "      erz: [2\n");
    let outcome = d.laden();
    let f = befund(&outcome, "kein gültiges YAML");
    assert_eq!(f.file.as_deref(), Some("ketten/a.yaml"));
    assert!(f.position.is_some());
}

#[test]
fn doppelter_schluessel() {
    let d = Daten::neu().ersetze(
        "ketten/a.yaml",
        "    menge: 1\n    dauer_tage: 1\n    anlage: mine",
        "    menge: 1\n    menge: 2\n    dauer_tage: 1\n    anlage: mine",
    );
    let outcome = d.laden();
    let f = befund(&outcome, "Schlüssel „menge“ kommt hier doppelt vor");
    assert_eq!(
        f.position.unwrap().line,
        d.zeile("ketten/a.yaml", "    menge: 2")
    );
    assert_eq!(outcome.report.warnings().count(), 0, "{}", alle(&outcome));
}

#[test]
fn unbekannter_abschnitt_mit_vorschlag() {
    let d = Daten::neu().ersetze("ketten/a.yaml", "rezepte:", "rezepe:");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Unbekannter Abschnitt „rezepe“. Meinten Sie „rezepte“?",
    );
    assert_ort(
        f,
        "ketten/a.yaml",
        d.zeile("ketten/a.yaml", "rezepe:"),
        "rezepe",
    );
}

#[test]
fn unbekanntes_feld_mit_vorschlag() {
    let d = Daten::neu().ersetze(
        "ketten/a.yaml",
        "    wartung_je_jahr: 0.05\n    automatisierung_max: 0.1\n  - id: ofen",
        "    wartung_je_jahr: 0.05\n    automatsierung_max: 0.1\n  - id: ofen",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Unbekanntes Feld „automatsierung_max“. Meinten Sie „automatisierung_max“?",
    );
    assert_ort(
        f,
        "ketten/a.yaml",
        d.zeile("ketten/a.yaml", "automatsierung_max"),
        "anlagen[0].automatsierung_max",
    );
}

#[test]
fn pflichtfeld_fehlt() {
    let d = Daten::neu().ersetze("ketten/a.yaml", "    qualitaet_basis: 60\n", "");
    let outcome = d.laden();
    let f = befund(&outcome, "Pflichtfeld „qualitaet_basis“ fehlt.");
    assert_ort(
        f,
        "ketten/a.yaml",
        d.zeile("ketten/a.yaml", "eisen_schmelzen"),
        "rezepte[1]",
    );
}

#[test]
fn falscher_typ_mit_hinweis_auf_dezimalpunkt() {
    let d = Daten::neu().ersetze("ketten/a.yaml", "      erz: 2\n", "      erz: 2,5\n");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Erwartet wird eine Zahl, gefunden wurde der Text „2,5“.",
    );
    assert!(f.message.contains("mit Punkt"));
    assert_ort(
        f,
        "ketten/a.yaml",
        d.zeile("ketten/a.yaml", "erz: 2,5"),
        "rezepte[1].eingang.erz",
    );
}

#[test]
fn unbekannter_auswahlwert() {
    let d = Daten::neu().ersetze("ketten/a.yaml", "art: halbzeug", "art: halbzeg");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Unbekannter Wert „halbzeg“. Meinten Sie „halbzeug“?",
    );
    assert_ort(
        f,
        "ketten/a.yaml",
        d.zeile("ketten/a.yaml", "halbzeg"),
        "produkte[1].art",
    );
}

#[test]
fn unbekannter_verweis_mit_vorschlag() {
    let d = Daten::neu().ersetze("ketten/a.yaml", "      erz: 2\n", "      erzz: 2\n");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Produkt „erzz“ ist nicht definiert. Meinten Sie „erz“?",
    );
    assert_ort(
        f,
        "ketten/a.yaml",
        d.zeile("ketten/a.yaml", "erzz"),
        "rezepte[1].eingang.erzz",
    );
    nur_fehler(&outcome, 1);
}

#[test]
fn doppelte_id_nennt_erste_definition() {
    let d = Daten::neu().datei(
        "ketten/b.yaml",
        "anlagen:\n  - id: ofen\n    standorttyp: werk\n    investition_usd: 1\n    bauzeit_tage: 1\n    kapazitaet_je_tag: 1\n    lebensdauer_jahre: 1\n    wartung_je_jahr: 0\n    automatisierung_max: 0\n",
    );
    let outcome = d.laden();
    let erste = d.zeile("ketten/a.yaml", "  - id: ofen");
    let f = befund(
        &outcome,
        &format!(
            "Anlage „ofen“ ist doppelt definiert; erste Definition in ketten/a.yaml, Zeile {erste}."
        ),
    );
    assert_ort(f, "ketten/b.yaml", 2, "anlagen[0].id");
    nur_fehler(&outcome, 1);
}

#[test]
fn ungueltige_ids() {
    let d = Daten::neu()
        .ersetze("ketten/a.yaml", "id: grube", "id: Grube")
        .ersetze(
            "texte/de/a.yaml",
            "lagerstaette.grube",
            "lagerstaette.Grube",
        )
        .ersetze("laender/SWE.yaml", "id: SWE", "id: swe")
        .ersetze("texte/de/a.yaml", "land.SWE", "land.swe")
        .ersetze("ketten/a.yaml", "land: SWE", "land: swe")
        .ersetze(
            "parameter/laendermodell.yaml",
            "referenzland: SWE",
            "referenzland: swe",
        )
        .ersetze("ki/a.yaml", "laender: [SWE]", "laender: [swe]")
        .ersetze("waehrungen/a.yaml", "land: SWE", "land: swe");
    let outcome = d.laden();
    befund(&outcome, "Ungültige ID „Grube“");
    befund(&outcome, "Ungültiger Ländercode „swe“");
    nur_fehler(&outcome, 2);
}

#[test]
fn zirkulaere_technologien() {
    let d = Daten::neu().ersetze(
        "ketten/a.yaml",
        "    erfindungsjahr: 1800\n",
        "    erfindungsjahr: 1800\n    voraussetzungen: [giessen]\n  - id: giessen\n    fachgebiet: metall\n    erfindungsjahr: 1700\n    voraussetzungen: [schmelzen]\n",
    )
    .ersetze("texte/de/a.yaml", "technologie.schmelzen: Schmelzen\n", "technologie.schmelzen: Schmelzen\ntechnologie.giessen: Gießen\n");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Zirkuläre Voraussetzungen: schmelzen → giessen → schmelzen.",
    );
    assert_eq!(f.severity, Severity::Error);
    nur_fehler(&outcome, 1);
    // schmelzen (1800) needs giessen (1700): fine; giessen needs the younger schmelzen.
    befund(
        &outcome,
        "Voraussetzung „schmelzen“ wird erst 1800 erfunden",
    );
}

#[test]
fn forschungsaufwand_nach_1900_noetig() {
    let d = Daten::neu().ersetze(
        "ketten/a.yaml",
        "erfindungsjahr: 1800",
        "erfindungsjahr: 1910",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "„forschungsaufwand“ fehlt: Die Technologie wird 1910 erfunden",
    );
    assert_ort(
        f,
        "ketten/a.yaml",
        d.zeile("ketten/a.yaml", "erfindungsjahr: 1910"),
        "technologien[0].erfindungsjahr",
    );

    let ok = Daten::neu().ersetze(
        "ketten/a.yaml",
        "erfindungsjahr: 1800",
        "erfindungsjahr: 1910\n    forschungsaufwand: 500",
    );
    assert!(
        ok.laden().report.findings().is_empty(),
        "{}",
        alle(&ok.laden())
    );

    let unused = Daten::neu().ersetze(
        "ketten/a.yaml",
        "erfindungsjahr: 1800",
        "erfindungsjahr: 1800\n    forschungsaufwand: 500",
    );
    let outcome = unused.laden();
    assert_eq!(
        befund(&outcome, "wird nicht verwendet").severity,
        Severity::Warning
    );
    assert!(
        outcome.data.is_some(),
        "Warnungen verhindern das Laden nicht"
    );
}

#[test]
fn werte_ausserhalb_des_bereichs() {
    let d = Daten::neu()
        .ersetze(
            "ketten/a.yaml",
            "qualitaet_basis: 60",
            "qualitaet_basis: 120",
        )
        .ersetze("ketten/a.yaml", "      erz: 2\n", "      erz: -1\n")
        .ersetze(
            "ketten/a.yaml",
            "wartung_je_jahr: 0.05",
            "wartung_je_jahr: 5",
        );
    let outcome = d.laden();
    befund(
        &outcome,
        "Wert 120 liegt außerhalb des erlaubten Bereichs 0 bis 100.",
    );
    befund(&outcome, "Wert -1 muss größer als 0 sein.");
    befund(
        &outcome,
        "Wert 5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    nur_fehler(&outcome, 3);
}

#[test]
fn ganze_zahl_erwartet() {
    let d = Daten::neu().ersetze("ketten/a.yaml", "bauzeit_tage: 100", "bauzeit_tage: 100.5");
    let outcome = d.laden();
    befund(
        &outcome,
        "Erwartet wird eine ganze Zahl ab 0, gefunden wurde die Zahl 100.5.",
    );
}

#[test]
fn fehlender_und_ueberzaehliger_text() {
    let d = Daten::neu().ersetze(
        "texte/de/a.yaml",
        "produkt.eisen: Eisen\n",
        "produkt.eisn: Eisen\n",
    );
    let outcome = d.laden();
    let fehlt = befund(&outcome, "Text „produkt.eisen“ fehlt in texte/de/.");
    assert_ort(
        fehlt,
        "ketten/a.yaml",
        d.zeile("ketten/a.yaml", "id: eisen"),
        "produkte[1].id",
    );
    let ungenutzt = befund(&outcome, "Text „produkt.eisn“ gehört zu keinem Eintrag.");
    assert_eq!(ungenutzt.severity, Severity::Warning);
    assert_ort(
        ungenutzt,
        "texte/de/a.yaml",
        d.zeile("texte/de/a.yaml", "produkt.eisn"),
        "produkt.eisn",
    );
}

#[test]
fn produkt_ohne_bezugsquelle() {
    let d = Daten::neu()
        .ersetze(
            "ketten/a.yaml",
            "    produkt: eisen\n",
            "    produkt: erz\n    abbau: true\n",
        )
        .ersetze("ketten/a.yaml", "      erz: 2\n", "      eisen: 2\n");
    let outcome = d.laden();
    befund(
        &outcome,
        "Produkt „eisen“ kann weder hergestellt noch vom Staatsmarkt bezogen werden",
    );
}

#[test]
fn staatsmarkt_ist_bezugsquelle() {
    let d = Daten::neu()
        .ersetze("ketten/a.yaml", "anlagen:", "  - id: glas\n    art: halbzeug\n    branche: metallurgie\n    einheit: t\n    verwendung: industrie\n    warengruppe: erze\n    transportklasse: schuettgut\n    richtpreis_usd: 1_000\n    staatsmarkt:\n      preis_usd: 1_200\n      verfuegbar_ab: 1900\nanlagen:")
        .ersetze("texte/de/a.yaml", "produkt.eisen: Eisen\n", "produkt.eisen: Eisen\nprodukt.glas: Glas\n");
    let outcome = d.laden();
    assert!(outcome.report.findings().is_empty(), "{}", alle(&outcome));
    let data = outcome.data.unwrap();
    let glas = data
        .catalog
        .products
        .get(data.catalog.products.id("glas").unwrap());
    assert_eq!(glas.state_market.unwrap().price.to_usd(), 1200.0);
}

#[test]
fn abbau_nur_fuer_rohstoffe() {
    let d = Daten::neu().ersetze(
        "ketten/a.yaml",
        "    anlage: ofen\n",
        "    anlage: ofen\n    abbau: true\n",
    );
    let outcome = d.laden();
    let f = befund(&outcome, "„eisen“ ist kein Rohstoff");
    assert_eq!(f.path.to_string(), "rezepte[1].abbau");
}

#[test]
fn foerderindex_nur_fuer_rohstoffe() {
    let d = Daten::neu().ersetze(
        "ketten/a.yaml",
        "    richtpreis_usd: 400\n",
        "    richtpreis_usd: 400\n    foerderindex: {1900: 1.0, 1930: 1.5}\n",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Einen Förderindex („foerderindex“) haben nur Rohstoffe",
    );
    assert_eq!(f.path.to_string(), "produkte[1].foerderindex");

    // A raw material may have one; the deposit's output follows it.
    let d = Daten::neu().ersetze(
        "ketten/a.yaml",
        "    richtpreis_usd: 60\n",
        "    richtpreis_usd: 60\n    foerderindex: {1900: 1.0, 1930: 1.5}\n",
    );
    let outcome = d.laden();
    assert!(outcome.report.findings().is_empty(), "{}", alle(&outcome));
    let catalog = outcome.data.unwrap().catalog;
    let grube = catalog.deposits.id("grube").unwrap();
    assert_eq!(catalog.max_output(grube, 1900), 10_000.0);
    assert_eq!(catalog.max_output(grube, 1930), 15_000.0);
}

#[test]
fn pacht_nur_fuer_rohstoffe() {
    let d = Daten::neu().ersetze(
        "ketten/a.yaml",
        "    richtpreis_usd: 400\n",
        "    richtpreis_usd: 400\n    pacht_anteil: 0.1\n",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Pacht und Förderabgaben („pacht_anteil“) zahlen nur Rohstoffe",
    );
    assert_eq!(f.path.to_string(), "produkte[1].pacht_anteil");

    // A raw material pays it on every unit extracted.
    let d = Daten::neu().ersetze(
        "ketten/a.yaml",
        "    richtpreis_usd: 60\n",
        "    richtpreis_usd: 60\n    pacht_anteil: 0.1\n",
    );
    let outcome = d.laden();
    assert!(outcome.report.findings().is_empty(), "{}", alle(&outcome));
    let catalog = outcome.data.unwrap().catalog;
    let erz = catalog.products.id("erz").unwrap();
    assert_eq!(catalog.products.get(erz).rent_share, 0.1);
}

#[test]
fn fachrichtung_bei_arbeitskraeften() {
    let d = Daten::neu()
        .ersetze("ketten/a.yaml", "fachkraft.metall: 1.5", "fachkraft: 1.5")
        .ersetze(
            "ketten/a.yaml",
            "      ungelernt: 2\n",
            "      ungelernt.metall: 2\n",
        );
    let outcome = d.laden();
    befund(
        &outcome,
        "Qualifikation „fachkraft“ braucht eine Fachrichtung, z. B. „fachkraft.metall“.",
    );
    befund(
        &outcome,
        "Qualifikation „ungelernt“ wird ohne Fachrichtung angegeben.",
    );
    nur_fehler(&outcome, 2);

    let tippfehler = Daten::neu().ersetze(
        "ketten/a.yaml",
        "fachkraft.metall: 1.5",
        "fachkraft.metal: 1.5",
    );
    befund(
        &tippfehler.laden(),
        "Fachrichtung „metal“ ist nicht definiert. Meinten Sie „metall“?",
    );
}

#[test]
fn lagerstaetten_vorrat() {
    let ohne_vorrat = Daten::neu().ersetze("ketten/a.yaml", "    vorrat: 1_000_000\n", "");
    befund(&ohne_vorrat.laden(), "Feld „vorrat“ fehlt");

    let erneuerbar = Daten::neu().ersetze(
        "ketten/a.yaml",
        "    vorrat: 1_000_000\n",
        "    vorrat: 1_000_000\n    erneuerbar: true\n",
    );
    befund(&erneuerbar.laden(), "Erneuerbare Lagerstätten");

    let kein_rohstoff = Daten::neu().ersetze("ketten/a.yaml", "rohstoff: erz", "rohstoff: eisen");
    befund(&kein_rohstoff.laden(), "„eisen“ ist kein Rohstoff");
}

#[test]
fn rohstoff_ohne_lagerstaette_ist_warnung() {
    let kette = &KETTE[..KETTE.find("lagerstaetten:").unwrap()];
    let texte = TEXTE.replace("lagerstaette.grube: Grube\n", "");
    let outcome = Daten::neu()
        .datei("ketten/a.yaml", kette)
        .datei("texte/de/a.yaml", &texte)
        .laden();
    let w = befund(&outcome, "gibt es ein Abbau-Rezept, aber keine Lagerstätte");
    assert_eq!(w.severity, Severity::Warning);
    assert!(outcome.data.is_some());
}

#[test]
fn richtpreis_passt_nicht_zu_den_herstellkosten() {
    // Too high: the reference price leaves far more than the plausible margin.
    let d = Daten::neu().ersetze(
        "ketten/a.yaml",
        "    richtpreis_usd: 400\n",
        "    richtpreis_usd: 4000\n",
    );
    let outcome = d.laden();
    let w = befund(&outcome, "Das Rezept stellt „eisen“ 1900 zu Richtpreisen");
    assert_eq!(w.severity, Severity::Warning);
    assert!(w.message.contains("erwartet 5–45 %"), "{w}");
    let line = d.zeile("ketten/a.yaml", "  - id: eisen_schmelzen");
    assert_ort(w, "ketten/a.yaml", line, "rezepte[1].id");
    assert!(outcome.data.is_some());

    // Too low.
    let d = Daten::neu().ersetze(
        "ketten/a.yaml",
        "    richtpreis_usd: 60\n",
        "    richtpreis_usd: 10\n",
    );
    let outcome = d.laden();
    let w = befund(&outcome, "Das Rezept stellt „erz“ 1900 zu Richtpreisen");
    assert!(w.message.contains("erwartet 5–45 %"), "{w}");
    let line = d.zeile("ketten/a.yaml", "  - id: erz_abbau");
    assert_ort(w, "ketten/a.yaml", line, "rezepte[0].id");

    // Extraction far below its price is checked too: the rent of the land belongs into
    // the data (pacht_anteil), or the raw material falls to its bare cost in the game.
    let teuer = |extra: &str| {
        Daten::neu()
            .ersetze(
                "ketten/a.yaml",
                "    richtpreis_usd: 60\n",
                &format!("    richtpreis_usd: 6000\n{extra}"),
            )
            .laden()
    };
    assert!(alle(&teuer("")).contains("„erz“"), "{}", alle(&teuer("")));
    let outcome = teuer("    pacht_anteil: 0.8\n");
    assert!(!alle(&outcome).contains("„erz“"), "{}", alle(&outcome));
}

#[test]
fn yml_dateien_werden_gemeldet() {
    let dir = tempfile::tempdir().unwrap();
    for (path, text) in &Daten::neu().files {
        let target = dir.path().join(path);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(target, text).unwrap();
    }
    std::fs::write(dir.path().join("ketten/extra.yml"), "produkte: []\n").unwrap();
    let outcome = wsim_data::load_dir(dir.path());
    let w = befund(&outcome, "Endung .yml");
    assert_eq!(w.severity, Severity::Warning);
    assert!(w.file.as_deref().unwrap().ends_with("/ketten/extra.yml"));
    assert!(outcome.data.is_some());
}

#[test]
fn zeitreihe_ausserhalb_des_spielzeitraums() {
    let d = Daten::neu().ersetze("laender/SWE.yaml", "1930: 6_000_000", "2150: 6_000_000");
    let outcome = d.laden();
    let f = befund(&outcome, "Jahr 2150 liegt außerhalb von 1800 bis 2100.");
    assert_ort(
        f,
        "laender/SWE.yaml",
        8,
        "laender[0].werte.bevoelkerung.2150",
    );
}

#[test]
fn meta_und_texte_muessen_vorhanden_sein() {
    let outcome = Daten::neu()
        .ohne("meta.yaml")
        .ohne("texte/de/a.yaml")
        .laden();
    befund(&outcome, "Abschnitt „meta“ fehlt");
    befund(&outcome, "Ordner texte/de/ fehlt.");
}

#[test]
fn fehlerhafter_eintrag_erzeugt_keine_folgefehler() {
    // The furnace cannot be read; the recipe that uses it must not report it as unknown.
    let d = Daten::neu().ersetze(
        "ketten/a.yaml",
        "investition_usd: 2_000_000",
        "investition_usd: viel",
    );
    let outcome = d.laden();
    befund(&outcome, "gefunden wurde der Text „viel“");
    nur_fehler(&outcome, 1);
}

#[test]
fn bericht_ist_reproduzierbar() {
    let d = Daten::neu()
        .ersetze("ketten/a.yaml", "      erz: 2\n", "      erzz: 2\n")
        .ersetze("ketten/a.yaml", "rezepte:", "rezepe:");
    assert_eq!(d.laden().report, d.laden().report);
}

#[test]
fn laendermodell_muss_vorhanden_sein() {
    let outcome = Daten::neu().ohne("parameter/laendermodell.yaml").laden();
    befund(&outcome, "Abschnitt „laendermodell“ fehlt");
}

#[test]
fn anteile_muessen_eins_ergeben() {
    let d = Daten::neu().ersetze(
        "parameter/laendermodell.yaml",
        "ungelernt: 0.9, fachkraft: 0.1",
        "ungelernt: 0.9, fachkraft: 0.2",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Die Anteile ergeben zusammen 1.1000, müssen aber 1 ergeben.",
    );
    assert_eq!(
        f.path.to_string(),
        "laendermodell.qualifikationsanteile[0].anteile"
    );
    nur_fehler(&outcome, 1);
}

#[test]
fn tabellen_muessen_aufsteigend_sein() {
    let d = Daten::neu().ersetze(
        "parameter/laendermodell.yaml",
        "bip_je_kopf_usd: 50_000",
        "bip_je_kopf_usd: 500",
    );
    befund(&d.laden(), "aufsteigend sortiert");
}

#[test]
fn jede_qualifikation_braucht_einen_anteil() {
    let d = Daten::neu().ersetze(
        "parameter/laendermodell.yaml",
        "{ungelernt: 0.9, fachkraft: 0.1}",
        "{ungelernt: 1.0}",
    );
    befund(&d.laden(), "Eintrag für Qualifikation „fachkraft“ fehlt.");
    let ohne_fachrichtung = Daten::neu().ersetze(
        "parameter/laendermodell.yaml",
        "    fachkraft: {metall: 0.6, bergbau: 0.4}",
        "    fachkraft: {metall: 0.6, bergbau: 0.4}\n    ungelernt: {metall: 1.0}",
    );
    befund(
        &ohne_fachrichtung.laden(),
        "Qualifikation „ungelernt“ hat keine Fachrichtungen.",
    );
}

#[test]
fn nachbarn_werden_geprueft() {
    let selbst = Daten::neu().ersetze(
        "laender/SWE.yaml",
        "    binnenland: false",
        "    binnenland: false\n    nachbarn: [SWE]",
    );
    befund(
        &selbst.laden(),
        "Land „SWE“ kann nicht sein eigener Nachbar sein.",
    );

    let einseitig = Daten::neu()
        .ersetze(
            "laender/SWE.yaml",
            "    binnenland: false",
            "    binnenland: false\n    nachbarn: [NOR]",
        )
        .datei(
            "laender/NOR.yaml",
            &LAND.replace("SWE", "NOR").replace("59.33", "59.91"),
        )
        .ersetze(
            "texte/de/a.yaml",
            "land.SWE: Schweden\n",
            "land.SWE: Schweden\nland.NOR: Norwegen\n",
        );
    let outcome = einseitig.laden();
    let w = befund(&outcome, "„NOR“ führt „SWE“ nicht als Nachbarn.");
    assert_eq!(w.severity, Severity::Warning);
}

#[test]
fn gini_und_stabilitaet_im_bereich() {
    let d = Daten::neu().ersetze(
        "laender/SWE.yaml",
        "gini: {1900: 0.45}",
        "gini: {1900: 45}\n      stabilitaet: {1900: 2}",
    );
    let outcome = d.laden();
    befund(
        &outcome,
        "Wert 45 liegt außerhalb des erlaubten Bereichs 0 bis 0.95.",
    );
    befund(
        &outcome,
        "Wert 2 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
}

#[test]
fn produktionsmodell_wird_geprueft() {
    let outcome = Daten::neu()
        .ohne("parameter/produktionsmodell.yaml")
        .laden();
    befund(&outcome, "Abschnitt „produktionsmodell“ fehlt");

    let d = Daten::neu().ersetze("parameter/produktionsmodell.yaml", "werk: 1,", "werkk: 1,");
    let outcome = d.laden();
    befund(&outcome, "Unbekannter Wert „werkk“. Meinten Sie „werk“?");
    befund(&outcome, "Eintrag für Standorttyp „werk“ fehlt.");

    let d = Daten::neu().ersetze(
        "parameter/produktionsmodell.yaml",
        "zustand_minimum: 0.2",
        "zustand_minimum: 2",
    );
    befund(
        &d.laden(),
        "Wert 2 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );

    let d = Daten::neu().ersetze(
        "parameter/produktionsmodell.yaml",
        "lohnaufschlag_max: 1.0",
        "lohnaufschlag_max: 7",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 7 liegt außerhalb des erlaubten Bereichs 0 bis 5.",
    );
    assert_eq!(f.path.to_string(), "produktionsmodell.lohnaufschlag_max");
}

#[test]
fn finanzmodell_wird_geprueft() {
    let outcome = Daten::neu().ohne("parameter/finanzmodell.yaml").laden();
    befund(&outcome, "Abschnitt „finanzmodell“ fehlt");
    let d = Daten::neu().ersetze(
        "parameter/finanzmodell.yaml",
        "beleihung: 0.6",
        "beleihung: 60",
    );
    befund(
        &d.laden(),
        "Wert 60 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    let d = Daten::neu().ersetze(
        "parameter/finanzmodell.yaml",
        "laufzeit_max_jahre: 30",
        "laufzeit_max_jahre: 0",
    );
    befund(&d.laden(), "Wert 0 muss größer als 0 sein.");
}

#[test]
fn marktmodell_wird_geprueft() {
    let outcome = Daten::neu().ohne("parameter/marktmodell.yaml").laden();
    befund(&outcome, "Abschnitt „marktmodell“ fehlt");
    let d = Daten::neu().ersetze(
        "parameter/marktmodell.yaml",
        "[2.0, 1.6, 1.2, 0.9, 0.6]",
        "[2.0, 1.6]",
    );
    befund(
        &d.laden(),
        "Falsche Anzahl an Einträgen (2), erwartet wird 5 Werte",
    );
}

#[test]
fn richtpreis_ist_pflicht() {
    let d = Daten::neu().ersetze("ketten/a.yaml", "    richtpreis_usd: 60\n", "");
    befund(&d.laden(), "Pflichtfeld „richtpreis_usd“ fehlt.");
}

#[test]
fn transportmodell_wird_geprueft() {
    let outcome = Daten::neu().ohne("parameter/transportmodell.yaml").laden();
    befund(&outcome, "Abschnitt „transportmodell“ fehlt");
    let d = Daten::neu().ersetze("parameter/transportmodell.yaml", "land: 1.3", "land: 0.5");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 0.5 liegt außerhalb des erlaubten Bereichs 1 bis 5.",
    );
    assert_ort(
        f,
        "parameter/transportmodell.yaml",
        2,
        "transportmodell.umweg.land",
    );
    let d = Daten::neu().ersetze(
        "parameter/marktmodell.yaml",
        "glaettung_tage: 30",
        "glaettung_tage: 0",
    );
    befund(
        &d.laden(),
        "Wert 0 liegt außerhalb des erlaubten Bereichs 1 bis 365.",
    );
}

#[test]
fn verkehrsmittel_werden_geprueft() {
    let d = Daten::neu().ersetze("verkehrsmittel.yaml", "[schuettgut]", "[schuettgud]");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Transportklasse „schuettgud“ ist nicht definiert. Meinten Sie „schuettgut“?",
    );
    assert_ort(
        f,
        "verkehrsmittel.yaml",
        d.zeile("verkehrsmittel.yaml", "transportklassen"),
        "verkehrsmittel[0].transportklassen[0]",
    );
    nur_fehler(&outcome, 1);

    let d = Daten::neu().ersetze("verkehrsmittel.yaml", "[schuettgut]", "[]");
    befund(&d.laden(), "Mindestens eine Transportklasse ist nötig");

    let d = Daten::neu().ersetze(
        "verkehrsmittel.yaml",
        "verfuegbar_ab: 1800",
        "verfuegbar_ab: 1800\n    verfuegbar_bis: 1700",
    );
    befund(
        &d.laden(),
        "„verfuegbar_ab“ (1800) liegt nach „verfuegbar_bis“ (1700).",
    );

    let d = Daten::neu().ersetze("verkehrsmittel.yaml", "{1900: 25}", "{1900: 0}");
    let outcome = d.laden();
    let f = befund(&outcome, "Wert 0 muss größer als 0 sein.");
    assert_eq!(f.path.to_string(), "verkehrsmittel[0].km_je_tag.1900");

    let d = Daten::neu().ersetze("verkehrsmittel.yaml", "weg: gelaende", "weg: gelende");
    befund(
        &d.laden(),
        "Unbekannter Wert „gelende“. Meinten Sie „gelaende“?",
    );

    let d = Daten::neu().ersetze("texte/de/a.yaml", "verkehrsmittel.fuhrwerk: Fuhrwerk\n", "");
    befund(&d.laden(), "verkehrsmittel.fuhrwerk");
}

#[test]
fn kostenfaktor_der_transportklasse_ist_positiv() {
    let d = Daten::neu().ersetze(
        "grundlagen.yaml",
        "  - id: schuettgut\n",
        "  - id: schuettgut\n    kostenfaktor: 0\n",
    );
    let outcome = d.laden();
    let f = befund(&outcome, "Wert 0 muss größer als 0 sein.");
    assert_eq!(f.path.to_string(), "transportklassen[0].kostenfaktor");
}

#[test]
fn forschungsmodell_wird_geprueft() {
    let outcome = Daten::neu().ohne("parameter/forschungsmodell.yaml").laden();
    befund(&outcome, "Abschnitt „forschungsmodell“ fehlt");
    let data = Daten::neu().laden().data.unwrap();
    let groups = &data.catalog.research_model.researchers;
    assert_eq!(groups.len(), 2, "eine Gruppe je Fachrichtung");
    assert_eq!(
        groups[0].map(|g| data.catalog.labor_groups.key(g).to_owned()),
        Some("fachkraft.metall".to_owned())
    );

    let d = Daten::neu().ersetze(
        "parameter/forschungsmodell.yaml",
        "forscher: fachkraft",
        "forscher: ungelernt",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Forscher brauchen eine Qualifikation mit Fachrichtungen",
    );
    assert_ort(
        f,
        "parameter/forschungsmodell.yaml",
        5,
        "forschungsmodell.forscher",
    );
    let d = Daten::neu().ersetze(
        "parameter/forschungsmodell.yaml",
        "forscher: fachkraft",
        "forscher: fachkraf",
    );
    befund(
        &d.laden(),
        "Qualifikation „fachkraf“ ist nicht definiert. Meinten Sie „fachkraft“?",
    );
    let d = Daten::neu().ersetze(
        "parameter/forschungsmodell.yaml",
        "vorgriff_faktor: 1.25",
        "vorgriff_faktor: 0.5",
    );
    befund(
        &d.laden(),
        "Wert 0.5 liegt außerhalb des erlaubten Bereichs 1 bis 10.",
    );
}

#[test]
fn startformen_werden_geprueft() {
    let datei = "parameter/produktionsmodell.yaml";
    let d = Daten::neu().ersetze(datei, "    handel:\n", "    hendel:\n");
    let outcome = d.laden();
    befund(&outcome, "Unbekannter Wert „hendel“. Meinten Sie „handel“?");
    befund(&outcome, "Eintrag für Startform „handel“ fehlt.");

    let d = Daten::neu().ersetze(
        datei,
        "{anlage: ofen}",
        "{anlage: mine, rezept: eisen_schmelzen}",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Anlage „mine“ passt nicht zum Standorttyp der Startform.",
    );
    assert_eq!(
        f.path.to_string(),
        "produktionsmodell.startformen.werkstatt.anlagen[0].anlage"
    );
    befund(
        &outcome,
        "Rezept „eisen_schmelzen“ läuft nicht auf der Anlage „mine“.",
    );

    let d = Daten::neu().ersetze(
        datei,
        "{anlage: ofen}",
        "{anlage: ofen, rezept: eisen_schmelzn}",
    );
    befund(
        &d.laden(),
        "Rezept „eisen_schmelzn“ ist nicht definiert. Meinten Sie „eisen_schmelzen“?",
    );
}

#[test]
fn kimodell_wird_geprueft() {
    let datei = "parameter/kimodell.yaml";
    let d = Daten::neu().ersetze(
        datei,
        "schwierigkeit_standard: mittel",
        "schwierigkeit_standard: mitel",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Schwierigkeit „mitel“ ist unter „schwierigkeiten“ nicht aufgeführt.",
    );
    assert_eq!(f.path.to_string(), "kimodell.schwierigkeit_standard");

    let d = Daten::neu().ersetze(datei, "lager_niedrig_tage: 7", "lager_niedrig_tage: 30");
    befund(
        &d.laden(),
        "„lager_niedrig_tage“ muss kleiner als „lager_hoch_tage“ sein.",
    );

    let d = Daten::neu().ersetze(datei, "streuung: 0.15", "streuung: 0.9");
    befund(
        &d.laden(),
        "Wert 0.9 liegt außerhalb des erlaubten Bereichs 0 bis 0.5.",
    );

    let d = Daten::neu().ersetze(
        datei,
        "lohnaufschlag_schritt: 0.05",
        "lohnaufschlag_schritt: 2",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 2 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_eq!(
        f.path.to_string(),
        "kimodell.verhalten.lohnaufschlag_schritt"
    );

    let d = Daten::neu().ohne(datei);
    befund(
        &d.laden(),
        "Abschnitt „kimodell“ fehlt (erwartet in parameter/kimodell.yaml).",
    );
}

#[test]
fn namensgruppen_werden_geprueft() {
    let datei = "ki/a.yaml";
    let d = Daten::neu().ersetze(datei, "{branche} {rechtsform}", "{branch} {rechtsform}");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Platzhalter „{branch}“ ist unbekannt; erlaubt sind {familienname}, {ort}, {rechtsform} und {branche}.",
    );
    assert_eq!(f.path.to_string(), "namensgruppen[0].muster[0]");

    let d = Daten::neu().ersetze(datei, "    standard: true\n", "");
    befund(
        &d.laden(),
        "Genau eine Namensgruppe muss „standard: true“ haben, gefunden: 0.",
    );

    let d = Daten::neu().ersetze(datei, "orte: [Kiruna]", "orte: []");
    befund(&d.laden(), "Die Liste darf nicht leer sein.");

    let d = Daten::neu().ersetze(datei, "{bergbau: Gruv}", "{bergbaau: Gruv}");
    befund(
        &d.laden(),
        "Branche „bergbaau“ ist nicht definiert. Meinten Sie „bergbau“?",
    );
}

#[test]
fn reale_firmen_werden_geprueft() {
    let datei = "ki/firmen.yaml";
    let neu = || Daten::neu().datei(datei, KI_FIRMA);
    let d = neu().ersetze(datei, "        lagerstaette: grube\n", "");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Anlage „mine“ fördert einen Rohstoff; der Standort braucht eine „lagerstaette“.",
    );
    assert_eq!(
        f.path.to_string(),
        "reale_firmen[0].standorte[0].anlagen[0].anlage"
    );

    let d = neu().ersetze(datei, "gegruendet: 1890", "gegruendet: 1907");
    befund(
        &d.laden(),
        "Gegründet 1907: Reale Firmen beschreiben den Stand 1900.",
    );

    let d = neu().ersetze(
        datei,
        "{anlage: ofen, anzahl: 1,",
        "{anlage: mine, anzahl: 1,",
    );
    befund(
        &d.laden(),
        "Rezept „eisen_schmelzen“ läuft nicht auf der Anlage „mine“.",
    );

    let d = neu().ersetze(
        datei,
        "anlagen: [{anlage: mine, anzahl: 2}]",
        "anlagen: [{anlage: mine, anzahl: 2}, {anlage: ofen, anzahl: 1}]",
    );
    befund(
        &d.laden(),
        "Alle Anlagen eines Standorts brauchen denselben Standorttyp.",
    );

    let d = neu().ersetze(datei, "anzahl: 2", "anzahl: 0");
    befund(&d.laden(), "0 muss größer als 0 sein.");
}

#[test]
fn ereignisse_werden_geprueft() {
    let ereignis = "\
ereignisse:
  - id: grosser_streik
    datum: \"1905-13-01\"
    art: streik
    laender: [SWE, XXX]
";
    let d = Daten::neu().datei("ereignisse/a.yaml", ereignis).ersetze(
        "texte/de/a.yaml",
        "land.SWE: Schweden\n",
        "land.SWE: Schweden\nereignis.grosser_streik: Großer Streik\n",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "„1905-13-01“ ist kein gültiges Datum; erwartet wird JJJJ-MM-TT, z. B. 1914-07-28.",
    );
    assert_eq!(f.path.to_string(), "ereignisse[0].datum");
    befund(&outcome, "Unbekannte Ereignisart „streik“");
    befund(&outcome, "Land „XXX“ ist nicht definiert.");
    befund(&outcome, "Text „ereignis.grosser_streik.text“ fehlt");
}

/// A chain `eisen → stufe3 → … → stufe<n>` on top of the test chain (erz → eisen).
fn tiefe_kette(ebenen: usize, sehr_komplex: bool) -> (String, String) {
    let mut kette = String::from("produkte:\n");
    let mut rezepte = String::from("rezepte:\n");
    let mut texte = TEXTE.to_owned();
    for stufe in 3..=ebenen {
        let vorher = if stufe == 3 {
            "eisen".to_owned()
        } else {
            format!("stufe{}", stufe - 1)
        };
        let komplex = if sehr_komplex && stufe == ebenen {
            "    sehr_komplex: true\n"
        } else {
            ""
        };
        kette.push_str(&format!(
            "  - id: stufe{stufe}\n    art: komponente\n    branche: metallurgie\n    einheit: t\n    verwendung: industrie\n    warengruppe: erze\n    transportklasse: schuettgut\n    richtpreis_usd: 1000\n{komplex}"
        ));
        rezepte.push_str(&format!(
            "  - id: stufe{stufe}_bauen\n    produkt: stufe{stufe}\n    menge: 1\n    dauer_tage: 1\n    anlage: ofen\n    eingang:\n      {vorher}: 1\n    arbeit_stunden:\n      ungelernt: 1\n    qualitaet_basis: 50\n"
        ));
        texte.push_str(&format!(
            "produkt.stufe{stufe}: Stufe {stufe}\nrezept.stufe{stufe}_bauen: Stufe {stufe} bauen\n"
        ));
    }
    (kette + &rezepte, texte)
}

#[test]
fn hoechstens_vier_vorprodukte() {
    let d = Daten::neu().ersetze(
        "ketten/a.yaml",
        "    eingang:\n      erz: 2\n",
        "    eingang:\n      erz: 2\n      e2: 1\n      e3: 1\n      e4: 1\n      e5: 1\n",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Das Rezept hat 5 Vorprodukte; erlaubt sind höchstens 4",
    );
    assert_eq!(f.severity, Severity::Error);
    assert_eq!(f.path.to_string(), "rezepte[1].eingang");
}

#[test]
fn produktbaum_hoechstens_sechs_ebenen() {
    // Five levels: allowed only for very complex products.
    let (kette, texte) = tiefe_kette(5, false);
    let outcome = Daten::neu()
        .datei("ketten/b.yaml", &kette)
        .datei("texte/de/a.yaml", &texte)
        .laden();
    let w = befund(
        &outcome,
        "5 Ebenen (erz → eisen → stufe3 → stufe4 → stufe5); mehr als 4 sind nur für sehr komplexe Produkte",
    );
    assert_eq!(w.severity, Severity::Warning);
    assert!(outcome.data.is_some());
    let levels = wsim_data::product_levels(&outcome.data.unwrap().catalog);
    assert_eq!(levels.iter().max(), Some(&5));

    let (kette, texte) = tiefe_kette(6, true);
    let outcome = Daten::neu()
        .datei("ketten/b.yaml", &kette)
        .datei("texte/de/a.yaml", &texte)
        .laden();
    assert!(
        outcome
            .report
            .findings()
            .iter()
            .all(|f| !f.message.contains("6 Ebenen")),
        "{:?}",
        outcome.report.findings()
    );

    let (kette, texte) = tiefe_kette(7, true);
    let outcome = Daten::neu()
        .datei("ketten/b.yaml", &kette)
        .datei("texte/de/a.yaml", &texte)
        .laden();
    let f = befund(&outcome, "Der Produktbaum hat hier 7 Ebenen");
    assert_eq!(f.severity, Severity::Error);
    assert!(f.message.contains("erlaubt sind höchstens 6"));
}

#[test]
fn marke_und_werbung_werden_geprueft() {
    let d = Daten::neu().ersetze(
        "parameter/marktmodell.yaml",
        "markengewicht: [0.4, 0.6, 0.9, 1.2, 1.5]",
        "markengewicht: [0.4, 0.6, 0.9, 1.2]",
    );
    let outcome = d.laden();
    let f = befund(&outcome, "Falsche Anzahl an Einträgen (4)");
    assert_eq!(f.path.to_string(), "marktmodell.marke.markengewicht");

    let d = Daten::neu().ersetze(
        "parameter/marktmodell.yaml",
        "      - {id: zeitung, ab: 1800, wirkung: 1.0}",
        "      - {id: zeitung, ab: 1800, wirkung: 1.0}\n      - {id: zeitung, ab: 1923, wirkung: 1.4}",
    );
    befund(&d.laden(), "Werbemittel „zeitung“ ist doppelt definiert");

    let d = Daten::neu().ersetze(
        "parameter/marktmodell.yaml",
        "      - {id: zeitung, ab: 1800, wirkung: 1.0}",
        "      - {id: zeitung, ab: 1800, wirkung: 1.0}\n      - {id: plakat, ab: 1850, wirkung: 0.8}",
    );
    befund(&d.laden(), "werbemittel.plakat");

    let d = Daten::neu().ersetze(
        "parameter/marktmodell.yaml",
        "bekanntheit_start: 0.5",
        "bekanntheit_start: 1.5",
    );
    let outcome = d.laden();
    let f = befund(&outcome, "1.5");
    assert_eq!(f.path.to_string(), "marktmodell.marke.bekanntheit_start");
}
