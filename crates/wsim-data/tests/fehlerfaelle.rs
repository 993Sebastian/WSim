//! Every kind of data error produces an understandable message at the right place.

use std::collections::BTreeMap;

use wsim_core::catalog::StorageClass;
use wsim_core::ids::Id as _;
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
  foerderkurve_ab: 0.5
  rohstoffzyklus: {schwankung: 0.05, beharrung: 0.9, grenze: 0.4}
  schulung: {kosten_anteil_lohn: 0.05, aufbau_je_monat: 0.08, verlust_je_monat: 0.03, arbeitsersparnis: 0.1, qualitaet_punkte: 8}
  automatisierung: {arbeitsersparnis: 0.8, kostenanteil: 0.5}
  qualitaet: {vorprodukte: 0.3, automatisierung: 10, zustand: 20}
  zustand_minimum: 0.2
  lohnaufschlag_max: 1.0
  stilllegung: {instandhaltung_anteil: 0.25, wiederanlauf_tage: 30, wiederanlauf_kosten: 0.02}
  verkauf: {erloes_anteil: 0.5, schrottwert: 0.03}
  schliessung: {lager_erloes_anteil: 0.5, abfindung_monate: 3}
  einspeiseverguetung: 0.5
  gemeinkosten_anteil: {rohstoff: 0.25, halbzeug: 0.5, komponente: 0.5, endprodukt: 1.0, energie: 0}
  richtpreis_marge: {minimum: 0.05, maximum: 0.45}
  nebenprodukte_lager_tage: 90
  anlagengroessen:
    kapazitaet: {sehr_klein: 0.25, klein: 0.5, mittel: 1, gross: 2, sehr_gross: 4}
    investition_exponent: 0.7
    arbeit_exponent: -0.15
    flaeche_exponent: 0.7
    bauzeit_exponent: 0.3
    umstellung: {kosten_anteil: 0.5, dauer_anteil: 0.5}
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
  dividende: {reserve_monate_min: 3, konzern_anteil: 0.5, quote_min: 0.2, quote_max: 0.6, kasse_monate: 4}
";

const MARKT: &str = "\
marktmodell:
  preisgewicht: [2.0, 1.6, 1.2, 0.9, 0.6]
  qualitaetsgewicht: [0.3, 0.5, 0.8, 1.1, 1.5]
  aneignung_je_jahr: 0.25
  preisanpassung: {hoch: 0.02, runter: 0.01, lagertage: 30, auslastung_normal: 0.85, hoechstfaktor: 20, aufholen_max: 20}
  staat_hoechstpreis: 1.5
  verdraengung_staat_jahre: 15
  verlauf_monate: 24
  meldung_preissenkung: 0.1
  preisniveau_anteil: {rohstoff: 0.2, halbzeug: 0.1, komponente: 0.1, endprodukt: 0.4, energie: 1}
  index_glaettung: 0.1
  haendler: {marge: 0.05, vorrat_tage: 30, glaettung_tage: 30, arbitrage: {abstand: 0.15, anteil: 0.25}}
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
  weiterentwicklung:
    stufen: 5
    wirkung_max: {qualitaet: 30, arbeit: 0.2, vorprodukte: 0.15}
    fertigung: {ausstoss: 0.25, vorprodukte: 0.15}
    aufwand: {anteil: 0.2, wachstum: 1.6, grundaufwand: 10_000}
    fachgebiete:
      bergbau: bergbau
      metallurgie: metall
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
const KAUF: &str = include_str!("../../../data/parameter/kaufmodell.yaml");
const GRUNDSTUECKE: &str = include_str!("../../../data/parameter/grundstuecksmodell.yaml");
const GRUNDSTUECKE_TEXTE: &str = include_str!("../../../data/texte/de/grundstuecke.yaml");

const KI_NAMEN: &str = "\
namensgruppen:
  - id: schwedisch
    laender: [SWE]
    standard: true
    familiennamen: [Berg]
    vornamen: [Lars]
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
anlagengroesse.sehr_klein: sehr klein
anlagengroesse.klein: klein
anlagengroesse.mittel: mittel
anlagengroesse.gross: groß
anlagengroesse.sehr_gross: sehr groß
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
            ("parameter/kaufmodell.yaml", KAUF),
            ("parameter/grundstuecksmodell.yaml", GRUNDSTUECKE),
            ("ki/a.yaml", KI_NAMEN),
            ("texte/de/a.yaml", TEXTE),
            ("texte/de/grundstuecke.yaml", GRUNDSTUECKE_TEXTE),
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
fn umrechnung_nur_beim_wechsel_und_positiv() {
    let d = Daten::neu().ersetze(
        "waehrungen/a.yaml",
        "{ab: \"1999-01\", waehrung: krone_neu}",
        "{ab: \"1999-01\", waehrung: krone_neu, umrechnung: 2}",
    );
    let outcome = d.laden();
    assert!(outcome.report.findings().is_empty(), "{}", alle(&outcome));
    let catalog = outcome.data.unwrap().catalog;
    let swe = catalog.countries.id("SWE").unwrap();
    assert_eq!(
        catalog.currencies.reform_at(swe, 1999.0).unwrap().factor,
        2.0
    );

    let d = Daten::neu()
        .ersetze(
            "waehrungen/a.yaml",
            "{ab: 1900, waehrung: krone}",
            "{ab: 1900, waehrung: krone, umrechnung: 2}",
        )
        .ersetze(
            "waehrungen/a.yaml",
            "{ab: \"1999-01\", waehrung: krone_neu}",
            "{ab: \"1999-01\", waehrung: krone_neu, umrechnung: -1}",
        );
    let outcome = d.laden();
    let f = befund(&outcome, "„umrechnung“ gilt nur für einen Zeitraum");
    assert_ort(
        f,
        "waehrungen/a.yaml",
        d.zeile("waehrungen/a.yaml", "umrechnung: 2"),
        "landeswaehrungen[0].perioden[0].umrechnung",
    );
    befund(&outcome, "Wert -1 muss größer als 0 sein");
    nur_fehler(&outcome, 2);
}

#[test]
fn kaufmodell_wird_geprueft() {
    let d = Daten::neu()
        .ersetze(
            "parameter/kaufmodell.yaml",
            "qualifiziert_ab_stufe: 3",
            "qualifiziert_ab_stufe: 9",
        )
        .ersetze(
            "parameter/kaufmodell.yaml",
            "angebot_chance: {bei_0: 0.02,",
            "angebot_chance: {bei_0: 1.5,",
        )
        .ersetze(
            "parameter/kaufmodell.yaml",
            "insolvenz: {tage: 30, mindestpreis: 0.5}",
            "insolvenz: {tage: 400, mindestpreis: 1.2}",
        );
    let outcome = d.laden();
    let f = befund(&outcome, "Qualifikationsstufe 9 gibt es nicht");
    assert_ort(
        f,
        "parameter/kaufmodell.yaml",
        d.zeile("parameter/kaufmodell.yaml", "qualifiziert_ab_stufe"),
        "kaufmodell.qualifiziert_ab_stufe",
    );
    befund(&outcome, "1.5");
    // The auction of insolvent companies (M38): at most a year, a share of the base value.
    let f = befund(&outcome, "400");
    assert_ort(
        f,
        "parameter/kaufmodell.yaml",
        d.zeile("parameter/kaufmodell.yaml", "insolvenz:"),
        "kaufmodell.insolvenz.tage",
    );
    let f = befund(&outcome, "1.2");
    assert_ort(
        f,
        "parameter/kaufmodell.yaml",
        d.zeile("parameter/kaufmodell.yaml", "insolvenz:"),
        "kaufmodell.insolvenz.mindestpreis",
    );
    nur_fehler(&outcome, 4);

    let ohne = Daten::neu().ohne("parameter/kaufmodell.yaml");
    let outcome = ohne.laden();
    befund(&outcome, "Abschnitt „kaufmodell“ fehlt");
}

#[test]
fn anteile_werden_geprueft() {
    let datei = "parameter/kaufmodell.yaml";
    let gut = Daten::neu().laden();
    let m = &gut.data.as_ref().unwrap().catalog.deal_model.stakes;
    assert_eq!(m.premium_max, 0.5);
    assert_eq!(m.discount_min, 0.05);
    for (alt, neu, meldung, pfad) in [
        (
            "praemie_min: 0.2",
            "praemie_min: 0.8",
            "„praemie_min“ muss kleiner als „praemie_max“ sein.",
            "kaufmodell.anteile",
        ),
        (
            "abschlag_max: 0.2",
            "abschlag_max: 0.95",
            "Wert 0.95 liegt außerhalb des erlaubten Bereichs 0 bis 0.9.",
            "kaufmodell.anteile.abschlag_max",
        ),
        (
            "rendite_gut: 0.15",
            "rendite_gut: 0",
            "Wert 0 liegt außerhalb des erlaubten Bereichs 0.01 bis 1.",
            "kaufmodell.anteile.rendite_gut",
        ),
    ] {
        let outcome = Daten::neu().ersetze(datei, alt, neu).laden();
        let f = befund(&outcome, meldung);
        assert_eq!(f.path.to_string(), pfad);
    }
}

#[test]
fn grundstuecksmodell_wird_geprueft() {
    let datei = "parameter/grundstuecksmodell.yaml";
    let d = Daten::neu()
        .ersetze(datei, "anteile: {reich: 0.40,", "anteile: {reich: 0.50,")
        .ersetze(
            datei,
            "flaeche_ha: {von: 2, bis: 6}",
            "flaeche_ha: {von: 7, bis: 6}",
        )
        .ersetze(datei, "stadt: {anteil: 0.4,", "stadt: {anteil: 0.5,")
        .ersetze(datei, "pacht_anteil: 0.05", "pacht_anteil: 1.5")
        .ersetze(datei, "frei_min_anteil: 0.25", "frei_min_anteil: 0.95");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 0.95 liegt außerhalb des erlaubten Bereichs 0 bis 0.9.",
    );
    assert_eq!(f.path.to_string(), "grundstuecksmodell.frei_min_anteil");
    befund(
        &outcome,
        "Die Anteile „klassen.anteile.reich“ ergeben zusammen 1.1 statt 1.",
    );
    let f = befund(&outcome, "„von“ muss kleiner als „bis“ sein.");
    assert_eq!(
        f.path.to_string(),
        "grundstuecksmodell.klassen[1].flaeche_ha.von"
    );
    let f = befund(&outcome, "„lagen.anteil“ ergeben zusammen 1.1 statt 1.");
    assert_eq!(f.path.to_string(), "grundstuecksmodell.lagen");
    let f = befund(
        &outcome,
        "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_ort(
        f,
        datei,
        d.zeile(datei, "pacht_anteil"),
        "grundstuecksmodell.pacht_anteil",
    );
    nur_fehler(&outcome, 5);

    // Every size class and location needs a name.
    let outcome = Daten::neu()
        .ersetze(
            "texte/de/grundstuecke.yaml",
            "grundstuecksklasse.mittel: mittel\n",
            "",
        )
        .ersetze("texte/de/grundstuecke.yaml", "lage.hafen: Hafen\n", "")
        .laden();
    let f = befund(&outcome, "Text „grundstuecksklasse.mittel“ fehlt");
    assert_eq!(f.path.to_string(), "grundstuecksmodell.klassen[1].id");
    let f = befund(&outcome, "Text „lage.hafen“ fehlt");
    assert_eq!(f.path.to_string(), "grundstuecksmodell.lagen.hafen");

    // The area of a facility is positive.
    let outcome = Daten::neu()
        .ersetze(
            "ketten/a.yaml",
            "    standorttyp: werk\n",
            "    standorttyp: werk\n    flaeche_ha: -1\n",
        )
        .laden();
    let f = befund(&outcome, "Wert -1 muss größer als 0 sein");
    assert!(f.path.to_string().ends_with(".flaeche_ha"), "{}", f.path);

    let outcome = Daten::neu().ohne(datei).laden();
    befund(&outcome, "Abschnitt „grundstuecksmodell“ fehlt");
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
fn staatsnachfrage_verlauf_wird_geprueft() {
    let d = Daten::neu().ersetze(
        "ketten/a.yaml",
        "    richtpreis_usd: 400\n",
        "    richtpreis_usd: 400\n    staatsnachfrage:\n      je_mio_usd_bip: 0.5\n      verlauf: {1900: 1.0, 1950: -2.0, 2200: 3.0}\n",
    );
    let outcome = d.laden();
    let f = befund(&outcome, "Wert -2 darf nicht negativ sein");
    assert_eq!(
        f.path.to_string(),
        "produkte[1].staatsnachfrage.verlauf.1950"
    );
    let f = befund(&outcome, "Jahr 2200 liegt außerhalb");
    assert_eq!(
        f.path.to_string(),
        "produkte[1].staatsnachfrage.verlauf.2200"
    );

    // A valid profile reaches the catalog.
    let d = Daten::neu().ersetze(
        "ketten/a.yaml",
        "    richtpreis_usd: 400\n",
        "    richtpreis_usd: 400\n    staatsnachfrage:\n      je_mio_usd_bip: 0.5\n      verlauf: {1900: 1.0, 1950: 3.0}\n",
    );
    let outcome = d.laden();
    assert!(outcome.report.findings().is_empty(), "{}", alle(&outcome));
    let catalog = outcome.data.unwrap().catalog;
    let eisen = catalog.products.id("eisen").unwrap();
    let demand = catalog.products.get(eisen).state_demand.clone().unwrap();
    let date = |y| wsim_core::calendar::Date::new(y, 1, 1).unwrap();
    assert_eq!(demand.per_million_gdp_at(date(1900)), 0.5);
    assert_eq!(demand.per_million_gdp_at(date(1950)), 1.5);
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
fn foerderkurve_wird_geprueft() {
    // Deposits yield less towards their end (C2).
    let datei = "parameter/produktionsmodell.yaml";
    let d = Daten::neu().ersetze(datei, "foerderkurve_ab: 0.5", "foerderkurve_ab: 1.5");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_ort(
        f,
        datei,
        d.zeile(datei, "foerderkurve_ab"),
        "produktionsmodell.foerderkurve_ab",
    );
    // Training of the sites (W1).
    let d = Daten::neu().ersetze(datei, "arbeitsersparnis: 0.1,", "arbeitsersparnis: 0.95,");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 0.95 liegt außerhalb des erlaubten Bereichs 0 bis 0.9.",
    );
    assert_eq!(
        f.path.to_string(),
        "produktionsmodell.schulung.arbeitsersparnis"
    );
    // Closing a site.
    let d = Daten::neu().ersetze(datei, "abfindung_monate: 3", "abfindung_monate: 40");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 40 liegt außerhalb des erlaubten Bereichs 0 bis 36.",
    );
    assert_eq!(
        f.path.to_string(),
        "produktionsmodell.schliessung.abfindung_monate"
    );
    // Expansions per check (C2).
    let d = Daten::neu().ersetze(
        "parameter/kimodell.yaml",
        "ausbau_je_pruefung_max: 4",
        "ausbau_je_pruefung_max: 0",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 0 liegt außerhalb des erlaubten Bereichs 1 bis 50.",
    );
    assert_eq!(
        f.path.to_string(),
        "kimodell.verhalten.ausbau_je_pruefung_max"
    );
    // Entries into dear or short markets (C3) and the brake on expansions (C4).
    for (alt, neu, meldung, pfad) in [
        (
            "einstiege_je_quartal: 6",
            "einstiege_je_quartal: 500",
            "Wert 500 liegt außerhalb des erlaubten Bereichs 0 bis 100.",
            "kimodell.verhalten.einstiege_je_quartal",
        ),
        (
            "ausbau_markt_auslastung: 0.85",
            "ausbau_markt_auslastung: 1.5",
            "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
            "kimodell.verhalten.ausbau_markt_auslastung",
        ),
        // L1: priority of large market gaps over the own branches.
        (
            "forschung_luecke_vorrang_umsatz: 1.0",
            "forschung_luecke_vorrang_umsatz: -1.0",
            "Wert -1 darf nicht negativ sein.",
            "kimodell.verhalten.forschung_luecke_vorrang_umsatz",
        ),
    ] {
        let outcome = Daten::neu()
            .ersetze("parameter/kimodell.yaml", alt, neu)
            .laden();
        let f = befund(&outcome, meldung);
        assert_eq!(f.path.to_string(), pfad);
    }
}

#[test]
fn anlagengroessen_werden_geprueft() {
    let datei = "parameter/produktionsmodell.yaml";
    let d = Daten::neu()
        .ersetze(datei, "mittel: 1, gross: 2", "mittel: 1.5, gross: 1.2")
        .ersetze(datei, "sehr_gross: 4}", "riesig: 8}")
        .ersetze(datei, "arbeit_exponent: -0.15", "arbeit_exponent: -2");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Die Größe „mittel“ muss die Kapazität 1 haben (hier 1.5)",
    );
    assert_eq!(
        f.path.to_string(),
        "produktionsmodell.anlagengroessen.kapazitaet.mittel"
    );
    let f = befund(&outcome, "Unbekannter Wert „riesig“");
    assert_eq!(
        f.path.to_string(),
        "produktionsmodell.anlagengroessen.kapazitaet.riesig"
    );
    befund(&outcome, "Eintrag für Anlagengröße „sehr_gross“ fehlt.");
    let f = befund(
        &outcome,
        "Wert -2 liegt außerhalb des erlaubten Bereichs -1 bis 1.",
    );
    assert_ort(
        f,
        datei,
        d.zeile(datei, "arbeit_exponent"),
        "produktionsmodell.anlagengroessen.arbeit_exponent",
    );
    nur_fehler(&outcome, 4);

    // Capacities grow with the size.
    let outcome = Daten::neu()
        .ersetze(datei, "klein: 0.5, mittel", "klein: 0.2, mittel")
        .laden();
    let f = befund(
        &outcome,
        "Die Kapazität muss mit der Größe wachsen: „klein“ hat nicht mehr Kapazität als \
         „sehr_klein“.",
    );
    assert_eq!(
        f.path.to_string(),
        "produktionsmodell.anlagengroessen.kapazitaet.klein"
    );

    // Every size needs a name.
    let outcome = Daten::neu()
        .ersetze("texte/de/a.yaml", "anlagengroesse.gross: groß\n", "")
        .laden();
    let f = befund(&outcome, "Text „anlagengroesse.gross“ fehlt");
    assert_eq!(
        f.path.to_string(),
        "produktionsmodell.anlagengroessen.kapazitaet.gross"
    );
}

#[test]
fn meta_und_texte_muessen_vorhanden_sein() {
    let outcome = Daten::neu()
        .ohne("meta.yaml")
        .ohne("texte/de/a.yaml")
        .ohne("texte/de/grundstuecke.yaml")
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
fn regionen_werden_geprueft() {
    let region = |umfasst: &str, texte: &str| {
        Daten::neu()
            .ersetze(
                "laender/SWE.yaml",
                "    binnenland: false",
                &format!("    binnenland: false\n    umfasst: {umfasst}"),
            )
            .ersetze(
                "texte/de/a.yaml",
                "land.SWE: Schweden\n",
                &format!("land.SWE: Schweden\n{texte}"),
            )
    };
    let gut = region(
        "[SWE, ALA]",
        "teilland.SWE: Schweden\nteilland.ALA: Åland\n",
    )
    .laden();
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let swe = gut
        .data
        .as_ref()
        .expect("Daten laden")
        .catalog
        .countries
        .id("SWE");
    assert_eq!(
        gut.data
            .as_ref()
            .unwrap()
            .catalog
            .countries
            .get(swe.unwrap())
            .members,
        ["SWE", "ALA"]
    );

    let outcome = region("[SWE]", "teilland.SWE: Schweden\n").laden();
    let f = befund(
        &outcome,
        "Region „SWE“ muss mindestens zwei Länder umfassen.",
    );
    assert_eq!(f.path.to_string(), "laender[0].umfasst");

    let outcome = region(
        "[SWE, ala, ALA, ALA]",
        "teilland.SWE: Schweden\nteilland.ALA: Åland\nteilland.FIN: Finnland\n",
    )
    .laden();
    let f = befund(&outcome, "Ungültiger Ländercode „ala“");
    assert_eq!(f.path.to_string(), "laender[0].umfasst[1]");
    let f = befund(
        &outcome,
        "Land „ALA“ ist mehr als einmal einer Region zugeordnet",
    );
    assert_eq!(f.path.to_string(), "laender[0].umfasst[3]");
    let w = befund(&outcome, "Text „teilland.FIN“ gehört zu keinem Eintrag.");
    assert_eq!(w.severity, Severity::Warning);

    let outcome = region("[SWE, ALA]", "teilland.SWE: Schweden\n").laden();
    let f = befund(&outcome, "Text „teilland.ALA“ fehlt in texte/de/.");
    assert_eq!(f.path.to_string(), "laender[0].umfasst[1]");

    // A country of its own cannot be part of a region.
    let outcome = region(
        "[SWE, NOR]",
        "teilland.SWE: Schweden\nteilland.NOR: Norwegen\n",
    )
    .datei(
        "laender/NOR.yaml",
        &LAND.replace("SWE", "NOR").replace("59.33", "59.91"),
    )
    .ersetze(
        "texte/de/a.yaml",
        "land.SWE: Schweden\n",
        "land.SWE: Schweden\nland.NOR: Norwegen\n",
    )
    .laden();
    let f = befund(
        &outcome,
        "„NOR“ ist ein eigenes Land und kann nicht zu „SWE“ gehören.",
    );
    assert_eq!(f.path.to_string(), "laender[0].umfasst[1]");
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

    // Shut down and sold facilities (M22).
    let d = Daten::neu().ersetze(
        "parameter/produktionsmodell.yaml",
        "instandhaltung_anteil: 0.25",
        "instandhaltung_anteil: 1.5",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_eq!(
        f.path.to_string(),
        "produktionsmodell.stilllegung.instandhaltung_anteil"
    );
    let d = Daten::neu().ersetze(
        "parameter/produktionsmodell.yaml",
        "erloes_anteil: 0.5",
        "erloes_anteil: -0.1",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert -0.1 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_eq!(
        f.path.to_string(),
        "produktionsmodell.verkauf.erloes_anteil"
    );
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
fn dividenden_werden_geprueft() {
    let datei = "parameter/finanzmodell.yaml";
    let gut = Daten::neu().laden();
    let m = &gut.data.as_ref().unwrap().catalog.finance_model.dividends;
    assert_eq!(m.group_share, 0.5);
    assert_eq!(m.payout_max, 0.6);
    for (alt, neu, meldung, pfad) in [
        (
            "konzern_anteil: 0.5",
            "konzern_anteil: 1.5",
            "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
            "finanzmodell.dividende.konzern_anteil",
        ),
        (
            "quote_min: 0.2",
            "quote_min: 0.8",
            "„quote_min“ muss kleiner als „quote_max“ sein.",
            "finanzmodell.dividende",
        ),
        (
            "kasse_monate: 4",
            "kasse_monate: -1",
            "Wert -1 liegt außerhalb des erlaubten Bereichs 0 bis 60.",
            "finanzmodell.dividende.kasse_monate",
        ),
    ] {
        let outcome = Daten::neu().ersetze(datei, alt, neu).laden();
        let f = befund(&outcome, meldung);
        assert_eq!(f.path.to_string(), pfad);
    }
    let ohne = Daten::neu()
        .ersetze(datei, "  dividende: {reserve_monate_min: 3, konzern_anteil: 0.5, quote_min: 0.2, quote_max: 0.6, kasse_monate: 4}\n", "")
        .laden();
    befund(&ohne, "Pflichtfeld „dividende“ fehlt.");
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
    // Price islands (C1).
    let d = Daten::neu().ersetze("parameter/marktmodell.yaml", "anteil: 0.25}", "anteil: 2}");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 2 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_eq!(f.path.to_string(), "marktmodell.haendler.arbitrage.anteil");
}

#[test]
fn nachfragewellen_und_gebrauchtmarkt_werden_geprueft() {
    // Generated demand waves (MN4): the lower factor must not exceed the upper one.
    let d = Daten::neu().ersetze(
        "parameter/marktmodell.yaml",
        "marktmodell:\n",
        "marktmodell:\n  trendgenerator: {ab: 2027, chance: 0.3, faktor_min: 1.9, faktor_max: 1.2, jahre_min: 3, jahre_max: 8}\n",
    );
    let outcome = d.laden();
    let f = befund(&outcome, "faktor_min");
    assert_eq!(f.path.to_string(), "marktmodell.trendgenerator.faktor_max");
    // The share of the used market is a share.
    let d = Daten::neu().ersetze(
        "parameter/marktmodell.yaml",
        "marktmodell:\n",
        "marktmodell:\n  gebraucht: {weitergabe: 1.5, lebensdauer_anteil: 0.5, anteil_erstkauf: 0.5}\n",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_eq!(f.path.to_string(), "marktmodell.gebraucht.weitergabe");
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

    let d = Daten::neu().ersetze(
        "verkehrsmittel.yaml",
        "km_je_tag: {1900: 25}",
        "km_je_tag: {1900: 25}\n    nutzlast_t: {1900: -2}\n    kaufpreis_usd: {1900: 9000}",
    );
    let outcome = d.laden();
    let f = befund(&outcome, "Wert -2 muss größer als 0 sein.");
    assert_eq!(f.path.to_string(), "verkehrsmittel[0].nutzlast_t.1900");
    let d = Daten::neu().ersetze(
        "verkehrsmittel.yaml",
        "km_je_tag: {1900: 25}",
        "km_je_tag: {1900: 25}\n    nutzlast_t: {1900: 2}",
    );
    let outcome = d.laden();
    let f = befund(&outcome, "Zu „nutzlast_t“ gehört auch „kaufpreis_usd“");
    assert_eq!(f.path.to_string(), "verkehrsmittel[0].nutzlast_t");
    nur_fehler(&outcome, 1);

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
fn weiterentwicklung_wird_geprueft() {
    let datei = "parameter/forschungsmodell.yaml";
    let data = Daten::neu().laden().data.unwrap();
    let m = &data.catalog.research_model.development;
    assert_eq!(m.levels, 5);
    let bergbau = data.catalog.branches.id("bergbau").unwrap();
    assert_eq!(
        m.fields[bergbau.index()].map(|f| data.catalog.specializations.key(f).to_owned()),
        Some("bergbau".to_owned())
    );

    let d = Daten::neu()
        .ersetze(datei, "stufen: 5", "stufen: 0")
        .ersetze(
            datei,
            "arbeit: 0.2, vorprodukte",
            "arbeit: 0.95, vorprodukte",
        );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 0 liegt außerhalb des erlaubten Bereichs 1 bis 10.",
    );
    assert_ort(
        f,
        datei,
        d.zeile(datei, "stufen:"),
        "forschungsmodell.weiterentwicklung.stufen",
    );
    let f = befund(
        &outcome,
        "Wert 0.95 liegt außerhalb des erlaubten Bereichs 0 bis 0.9.",
    );
    assert_eq!(
        f.path.to_string(),
        "forschungsmodell.weiterentwicklung.wirkung_max.arbeit"
    );
    nur_fehler(&outcome, 2);

    // The ore is mined without a technology: its branch needs a research field.
    let d = Daten::neu().ersetze(datei, "      bergbau: bergbau\n", "");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Produkt „erz“ wird ohne Technologie hergestellt, und seine Branche „bergbau“ hat kein Fachgebiet",
    );
    assert_eq!(
        f.path.to_string(),
        "forschungsmodell.weiterentwicklung.fachgebiete"
    );
    nur_fehler(&outcome, 1);

    let d = Daten::neu().ersetze(datei, "bergbau: bergbau", "bergbau: holz");
    befund(&d.laden(), "Fachrichtung „holz“ ist nicht definiert.");

    // Manufacturing research (Q6): output gained and inputs saved per run.
    assert_eq!(m.process_output_max, 0.25);
    assert_eq!(m.process_inputs_max, 0.15);
    let d = Daten::neu().ersetze(
        datei,
        "fertigung: {ausstoss: 0.25, vorprodukte: 0.15}",
        "fertigung: {ausstoss: 2.5, vorprodukte: 0.95}",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 2.5 liegt außerhalb des erlaubten Bereichs 0 bis 2.",
    );
    assert_ort(
        f,
        datei,
        d.zeile(datei, "fertigung:"),
        "forschungsmodell.weiterentwicklung.fertigung.ausstoss",
    );
    let f = befund(
        &outcome,
        "Wert 0.95 liegt außerhalb des erlaubten Bereichs 0 bis 0.9.",
    );
    assert_eq!(
        f.path.to_string(),
        "forschungsmodell.weiterentwicklung.fertigung.vorprodukte"
    );
    nur_fehler(&outcome, 2);
    // Without the block there is no manufacturing research.
    let d = Daten::neu().ersetze(
        datei,
        "    fertigung: {ausstoss: 0.25, vorprodukte: 0.15}\n",
        "",
    );
    let ohne = d.laden().data.unwrap();
    assert_eq!(
        ohne.catalog.research_model.development.process_output_max,
        0.0
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

    // The AI shuts down below a utilization that must lie under its target (M22).
    let d = Daten::neu().ersetze(
        datei,
        "stilllegen_auslastung: 0.5",
        "stilllegen_auslastung: 0.9",
    );
    befund(
        &d.laden(),
        "„stilllegen_auslastung“ muss kleiner als „stilllegen_zielauslastung“ sein.",
    );
    let d = Daten::neu().ersetze(
        datei,
        "wiederanfahren_auslastung: 0.95",
        "wiederanfahren_auslastung: 0.7",
    );
    befund(
        &d.laden(),
        "„stilllegen_zielauslastung“ muss kleiner als „wiederanfahren_auslastung“ sein.",
    );

    // Newcomers enter markets paying at least the reference price (M33).
    let d = Daten::neu().ersetze(
        datei,
        "einstieg_preisfaktor: 1.3",
        "einstieg_preisfaktor: 0.8",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 0.8 liegt außerhalb des erlaubten Bereichs 1 bis 10.",
    );
    assert_eq!(
        f.path.to_string(),
        "kimodell.verhalten.einstieg_preisfaktor"
    );
    let d = Daten::neu().ersetze(datei, "einstieg_anteil: 0.25", "einstieg_anteil: 0");
    befund(
        &d.laden(),
        "Wert 0 liegt außerhalb des erlaubten Bereichs 0.01 bis 1.",
    );
    // Research prepares coming products at most 30 years ahead (M39).
    let d = Daten::neu().ersetze(
        datei,
        "forschung_vorlauf_jahre: 5",
        "forschung_vorlauf_jahre: 40",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 40 liegt außerhalb des erlaubten Bereichs 0 bis 30.",
    );
    assert_eq!(
        f.path.to_string(),
        "kimodell.verhalten.forschung_vorlauf_jahre"
    );
    // New concessions only where the reserve lasts (M33).
    let d = Daten::neu().ersetze(datei, "vorrat_jahre_min: 10", "vorrat_jahre_min: -1");
    let outcome = d.laden();
    let f = befund(&outcome, "Wert -1 darf nicht negativ sein.");
    assert_eq!(f.path.to_string(), "kimodell.verhalten.vorrat_jahre_min");
    // Estimates are off by a share (B2).
    let d = Daten::neu().ersetze(
        datei,
        "schaetzfehler: {bei_0: 0.15, bei_1: 0.0}",
        "schaetzfehler: {bei_0: 1.5, bei_1: 0.0}",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_eq!(f.path.to_string(), "kimodell.verhalten.schaetzfehler.bei_0");

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

    // Managers need first names (MA1).
    let d = Daten::neu().ersetze(datei, "vornamen: [Lars]", "vornamen: []");
    let outcome = d.laden();
    let f = befund(&outcome, "Die Liste darf nicht leer sein.");
    assert_eq!(f.path.to_string(), "namensgruppen[0].vornamen");

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

/// An event in Sweden with the given effects (H1).
fn mit_wirkungen(wirkungen: &str) -> Daten {
    let ereignis = format!(
        "\
ereignisse:
  - id: grosser_streik
    datum: \"1905-03-15\"
    art: krise
    laender: [SWE]
    wirkungen:
{wirkungen}"
    );
    Daten::neu().datei("ereignisse/a.yaml", &ereignis).ersetze(
        "texte/de/a.yaml",
        "land.SWE: Schweden\n",
        "land.SWE: Schweden\nereignis.grosser_streik: Großer Streik\n\
             ereignis.grosser_streik.text: Lange Arbeitskämpfe.\n",
    )
}

#[test]
fn ereigniswirkungen_werden_geprueft() {
    use wsim_core::catalog::EffectKind;
    let gut = mit_wirkungen(
        "      - {art: arbeitskraefte, faktor: 0.8, bis: \"1906-01-01\"}
      - {art: nachfrage, warengruppen: [erze], konsum: 0.5}
      - {art: zerstoerung, anteil: 0.1}
      - {art: enteignung, nur_auslaendische: false, entschaedigung: 0.3}
      - {art: abschottung, alle: true}
      - {art: boersenkrach, einbruch: 0.3}
",
    )
    .laden();
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let e = &gut.data.as_ref().unwrap().catalog.events[0];
    assert_eq!(e.effects.len(), 6);
    assert_eq!(e.effects[0].kind, EffectKind::Labor { factor: 0.8 });
    assert_eq!(e.effects[0].countries.len(), 1);
    assert!(e.effects[0].until.is_some());
    assert!(e.effects[5].countries.is_empty());
    assert_eq!(
        e.effects[3].kind,
        EffectKind::Expropriation {
            foreign_only: false,
            compensation: 0.3
        }
    );

    for (wirkung, meldung, pfad) in [
        (
            "{art: arbeitskraefte, faktor: 0.8, anteil: 0.1}",
            "Das Feld „anteil“ passt nicht zur Wirkung „arbeitskraefte“.",
            "ereignisse[0].wirkungen[0].anteil",
        ),
        (
            "{art: produktion}",
            "Die Wirkung „produktion“ braucht das Feld „faktor“.",
            "ereignisse[0].wirkungen[0]",
        ),
        (
            "{art: handelssperre}",
            "Die Wirkung „handelssperre“ braucht das Feld „gegen“.",
            "ereignisse[0].wirkungen[0]",
        ),
        (
            "{art: handelssperre, gegen: [SWE]}",
            "„SWE“ steht auf beiden Seiten der Wirkung (laender und gegen).",
            "ereignisse[0].wirkungen[0].gegen[0]",
        ),
        (
            "{art: nachfrage, konsum: 1}",
            "Die Wirkung „nachfrage“ ändert nichts; mindestens ein Faktor muss von 1 abweichen.",
            "ereignisse[0].wirkungen[0]",
        ),
        (
            "{art: zerstoerung, anteil: 1.5}",
            "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
            "ereignisse[0].wirkungen[0].anteil",
        ),
        (
            "{art: zerstoerung, anteil: 0.1, bis: \"1906-01-01\"}",
            "Das Feld „bis“ passt nicht zur Wirkung „zerstoerung“.",
            "ereignisse[0].wirkungen[0].bis",
        ),
        (
            "{art: arbeitskraefte, faktor: 0.8, bis: \"1905-01-01\"}",
            "Das Ende „1905-01-01“ liegt nicht nach dem Ereignistag 1905-03-15.",
            "ereignisse[0].wirkungen[0].bis",
        ),
        (
            "{art: boersenkrach, einbruch: 0.3, laender: [SWE]}",
            "Das Feld „laender“ passt nicht zur Wirkung „boersenkrach“.",
            "ereignisse[0].wirkungen[0].laender",
        ),
        (
            "{art: nachfrage, warengruppen: [moebel], staat: 2}",
            "Warengruppe „moebel“ ist nicht definiert.",
            "ereignisse[0].wirkungen[0].warengruppen[0]",
        ),
        (
            "{art: zoll, aufschlag: 0.2, gegen: [XXX]}",
            "Land „XXX“ ist nicht definiert.",
            "ereignisse[0].wirkungen[0].gegen[0]",
        ),
    ] {
        let outcome = mit_wirkungen(&format!("      - {wirkung}\n")).laden();
        let f = befund(&outcome, meldung);
        assert_eq!(f.path.to_string(), pfad, "{wirkung}");
    }
    // An unknown kind is a reading error of the file.
    let falsch = mit_wirkungen("      - {art: pest}\n").laden();
    befund(&falsch, "pest");
}

#[test]
fn ereignisfolgen_werden_geprueft() {
    let gut = Daten::neu()
        .datei(
            "parameter/ereignisse.yaml",
            "ereignisfolgen:\n  staatsbetrieb:\n    betriebskapital_anteil: 0.1\n",
        )
        .laden();
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let m = &gut.data.as_ref().unwrap().catalog.event_model;
    assert_eq!(m.working_capital_share, 0.1);
    let falsch = Daten::neu()
        .datei(
            "parameter/ereignisse.yaml",
            "ereignisfolgen:\n  staatsbetrieb:\n    betriebskapital_anteil: 2\n",
        )
        .laden();
    let f = befund(
        &falsch,
        "Wert 2 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_eq!(
        f.path.to_string(),
        "ereignisfolgen.staatsbetrieb.betriebskapital_anteil"
    );
}

#[test]
fn verlauf_und_preismeldung_im_marktmodell() {
    let d = Daten::neu()
        .ersetze(
            "parameter/marktmodell.yaml",
            "verlauf_monate: 24",
            "verlauf_monate: 0",
        )
        .ersetze(
            "parameter/marktmodell.yaml",
            "meldung_preissenkung: 0.1",
            "meldung_preissenkung: 1.5",
        );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 0 liegt außerhalb des erlaubten Bereichs 1 bis 120.",
    );
    assert_eq!(f.path.to_string(), "marktmodell.verlauf_monate");
    let f = befund(
        &outcome,
        "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0.01 bis 0.9.",
    );
    assert_eq!(f.path.to_string(), "marktmodell.meldung_preissenkung");
    nur_fehler(&outcome, 2);
}

/// M33: the years over which a successor displaces the state demand must be positive.
#[test]
fn verdraengung_des_staatsbedarfs_braucht_jahre() {
    let outcome = Daten::neu()
        .ersetze(
            "parameter/marktmodell.yaml",
            "verdraengung_staat_jahre: 15",
            "verdraengung_staat_jahre: 0",
        )
        .laden();
    let f = befund(&outcome, "Wert 0 muss größer als 0 sein.");
    assert_eq!(f.path.to_string(), "marktmodell.verdraengung_staat_jahre");
    nur_fehler(&outcome, 1);
}

#[test]
fn etappen_werden_geprueft() {
    let etappen = "\
etappen:
  - id: gewinn
    art: gewinnmonat
    wert: 3
  - id: werke
    art: anlagen
  - id: laender
    art: laender
    wert: 1.5
  - id: fuehrung
    art: marktfuehrer
    wert: 2
  - id: reich
    art: eigenkapital
    wert: 0.5
  - id: falsch
    art: gewinn_monat
";
    let texte = "\
etappe.gewinn: Gewinn
etappe.gewinn.hinweis: So geht's
etappe.werke: Werke
etappe.werke.hinweis: So geht's
etappe.laender: Länder
etappe.fuehrung: Führung
etappe.fuehrung.hinweis: So geht's
etappe.reich: Reich
etappe.reich.hinweis: So geht's
";
    let d = Daten::neu()
        .datei("etappen.yaml", etappen)
        .datei("texte/de/etappen.yaml", texte);
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Die Etappe der Art „gewinnmonat“ hat keinen „wert“; bitte entfernen.",
    );
    assert_ort(
        f,
        "etappen.yaml",
        d.zeile("etappen.yaml", "wert: 3"),
        "etappen[0].wert",
    );
    befund(
        &outcome,
        "Die Etappe der Art „anlagen“ braucht einen „wert“.",
    );
    befund(
        &outcome,
        "„wert“ muss eine ganze Zahl von 1 bis 1000 sein, nicht 1.5.",
    );
    befund(
        &outcome,
        "Wert 2 liegt außerhalb des erlaubten Bereichs 0.01 bis 1.",
    );
    befund(
        &outcome,
        "„wert“ ist das Vielfache des Startkapitals und muss größer als 1 sein, nicht 0.5.",
    );
    befund(&outcome, "Text „etappe.laender.hinweis“ fehlt");
    let f = befund(&outcome, "Unbekannter Wert „gewinn_monat“");
    assert!(f.message.contains("gewinnmonat"), "{f}");
    nur_fehler(&outcome, 7);
}

#[test]
fn etappe_ohne_namen() {
    let d = Daten::neu().datei(
        "etappen.yaml",
        "etappen:\n  - id: forschen\n    art: forschung\n",
    );
    let outcome = d.laden();
    befund(&outcome, "Text „etappe.forschen“ fehlt");
    befund(&outcome, "Text „etappe.forschen.hinweis“ fehlt");
    nur_fehler(&outcome, 2);
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
fn produktbaum_zaehlt_den_einfachsten_weg() {
    // A second, short recipe for stufe4 shortens everything built from it.
    let (kette, texte) = tiefe_kette(5, false);
    let kette = kette.replace(
        "rezepte:\n",
        "rezepte:\n  - id: stufe4_direkt\n    produkt: stufe4\n    menge: 1\n    dauer_tage: 1\n    anlage: ofen\n    eingang:\n      erz: 3\n    arbeit_stunden:\n      ungelernt: 1\n    qualitaet_basis: 50\n",
    );
    let texte = texte + "rezept.stufe4_direkt: Stufe 4 direkt\n";
    let outcome = Daten::neu()
        .datei("ketten/b.yaml", &kette)
        .datei("texte/de/a.yaml", &texte)
        .laden();
    assert!(
        outcome
            .report
            .findings()
            .iter()
            .all(|f| !f.message.contains("Ebenen")),
        "{:?}",
        outcome.report.findings()
    );
    let levels = wsim_data::product_levels(&outcome.data.unwrap().catalog);
    assert_eq!(levels.iter().max(), Some(&3));

    // A raw material that can also be synthesised counts as a leaf.
    let (kette, texte) = tiefe_kette(6, false);
    let kette = kette
        .replace(
            "  - id: stufe3\n    art: komponente\n",
            "  - id: stufe3\n    art: rohstoff\n",
        )
        .replace(
            "rezepte:\n",
            "rezepte:\n  - id: stufe3_abbau\n    produkt: stufe3\n    menge: 1\n    dauer_tage: 1\n    anlage: mine\n    abbau: true\n    arbeit_stunden:\n      ungelernt: 1\n    qualitaet_basis: 50\n",
        )
        + "lagerstaetten:\n  - id: stufe3_grube\n    land: SWE\n    rohstoff: stufe3\n    vorrat: 1_000_000\n    erschliessung: {investition_usd: 500_000, dauer_tage: 100}\n    foerderung_max_je_jahr: 10_000\n";
    let texte =
        texte + "rezept.stufe3_abbau: Stufe 3 fördern\nlagerstaette.stufe3_grube: Grube 3\n";
    let outcome = Daten::neu()
        .datei("ketten/b.yaml", &kette)
        .datei("texte/de/a.yaml", &texte)
        .laden();
    assert!(
        outcome
            .report
            .findings()
            .iter()
            .all(|f| !f.message.contains("Ebenen")),
        "{:?}",
        outcome.report.findings()
    );
    let levels = wsim_data::product_levels(&outcome.data.unwrap().catalog);
    assert_eq!(levels.iter().max(), Some(&4));
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

const PRODUKTNAMEN: &str = "\
produktnamen:
  hausmarke: 0.5
  ausgeschlossen: [Tesla]
  stile:
    - id: technik
      warengruppen: [erze]
      staemme: [Arvon, Belkor]
      muster:
        - {text: \"{stamm} Typ {zahl}\", bis: 1939}
        - {text: \"{stamm} {zahl}\"}
      zahlen: [2, 300]
      nachfolger: [II, III]
";

#[test]
fn produktnamen_werden_geprueft() {
    let datei = "ki/produktnamen.yaml";
    let basis = || Daten::neu().datei(datei, PRODUKTNAMEN);
    let outcome = basis().laden();
    assert!(outcome.report.findings().is_empty(), "{}", alle(&outcome));
    let naming = outcome.data.unwrap().catalog.product_naming;
    assert_eq!(naming.styles.len(), 1);
    assert_eq!(naming.styles[0].patterns[0].until, Some(1939));
    assert_eq!(naming.styles[0].successors, ["II", "III"]);
    assert!(naming.is_excluded("Neuer tesla 3"));
    // Without the section no product has a name.
    let ohne = Daten::neu().laden().data.unwrap().catalog.product_naming;
    assert!(ohne.styles.is_empty());

    let d = basis().ersetze(datei, "{stamm} {zahl}\"}", "{stamm} {nummer}\"}");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Platzhalter „{nummer}“ ist unbekannt; erlaubt sind {stamm}, {zahl}, {buchstabe} und {zusatz}.",
    );
    assert_ort(
        f,
        datei,
        d.zeile(datei, "{stamm} {nummer}"),
        "produktnamen.stile[0].muster[1].text",
    );

    let d = basis().ersetze(datei, "\"{stamm} {zahl}\"}", "\"Modell {zahl}\"}");
    befund(&d.laden(), "Das Muster braucht den Platzhalter {stamm}.");

    let d = basis().ersetze(
        datei,
        "\"{stamm} {zahl}\"}",
        "\"{stamm} {buchstabe}{zahl}\"}",
    );
    befund(
        &d.laden(),
        "Das Muster nutzt {buchstabe}; dafür darf „buchstaben“ nicht leer sein.",
    );

    let d = basis().ersetze(datei, "[Arvon, Belkor]", "[Arvon, Tesla]");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "„Tesla“ steht unter „ausgeschlossen“ (echte Produkt- und Markennamen).",
    );
    assert_eq!(f.path.to_string(), "produktnamen.stile[0].staemme[1]");

    let d = basis().ersetze(datei, "[Arvon, Belkor]", "[Arvon, arvon]");
    befund(&d.laden(), "„arvon“ steht doppelt in der Liste.");

    let d = basis().ersetze(datei, "[erze]", "[erzee]");
    befund(
        &d.laden(),
        "Warengruppe „erzee“ ist nicht definiert. Meinten Sie „erze“?",
    );

    let zweiter = "\n    - id: marke\n      warengruppen: [erze]\n      staemme: [Aurela]\n      muster:\n        - {text: \"{stamm}\"}\n";
    let d = basis().ersetze(
        datei,
        "      zahlen: [2, 300]\n",
        &format!("      zahlen: [2, 300]{zweiter}"),
    );
    befund(
        &d.laden(),
        "Warengruppe „erze“ gehört schon zum Stil „technik“.",
    );

    let d = basis().ersetze(datei, "bis: 1939}", "ab: 1950, bis: 1939}");
    befund(&d.laden(), "„ab“ (1950) liegt nach „bis“ (1939).");

    let d = basis().ersetze(
        datei,
        "\"{stamm} {zahl}\"}",
        "\"{stamm} {zahl}\", ab: 1950}",
    );
    befund(
        &d.laden(),
        "Mindestens ein Muster braucht weder „ab“ noch „bis“, damit es in jedem Jahr einen Namen gibt.",
    );

    let d = basis().ersetze(datei, "hausmarke: 0.5", "hausmarke: 1.5");
    befund(
        &d.laden(),
        "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );

    let d = basis().ersetze(datei, "zahlen: [2, 300]", "zahlen: [0, 300]");
    befund(&d.laden(), "Zahlen müssen größer als 0 sein.");

    let d = basis().ersetze(datei, "nachfolger: [II, III]", "nachfolger: [II, Mark 3]");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "„Mark 3“ muss ein einzelnes Wort ohne Ziffern sein (Kennzeichen eines Nachfolgemodells wie „II“).",
    );
    assert_eq!(f.path.to_string(), "produktnamen.stile[0].nachfolger[1]");
    let d = basis().ersetze(datei, "nachfolger: [II, III]", "nachfolger: [II, ii]");
    befund(&d.laden(), "„ii“ steht doppelt in der Liste.");

    let d = basis().datei("ki/zweite.yaml", PRODUKTNAMEN);
    befund(
        &d.laden(),
        "Abschnitt „produktnamen“ darf es nur einmal geben",
    );
}

const MANAGEMENT: &str = include_str!("../../../data/parameter/management.yaml");
/// Texts of the functions (the real ones are part of the interface texts).
const BEREICH_TEXTE: &str = "bereich.produktion: Produktion
bereich.einkauf_lager: Einkauf und Lager
bereich.vertrieb_marketing: Vertrieb und Marketing
bereich.personal: Personal
bereich.logistik: Logistik
bereich.forschung: Forschung
bereich.finanzen: Finanzen
bereich.strategie: Strategie
bereich.recht: Recht
";

/// The concern model of the base data.
fn outcome_ok_months<F: Fn() -> Daten>(basis: &F) -> wsim_core::catalog::ConcernModel {
    basis()
        .laden()
        .data
        .expect("loads")
        .catalog
        .management
        .concerns
}

/// MA1: Positions, salaries and the market for managers.
#[test]
fn management_wird_geprueft() {
    use wsim_core::catalog::SiteType;
    let datei = "parameter/management.yaml";
    // The test data have no academics; the salaries follow skilled metal workers.
    let daten = MANAGEMENT.replace("akademiker.kaufmaennisch", "fachkraft.metall");
    let basis = || {
        Daten::neu()
            .datei(datei, &daten)
            .datei("texte/de/bereiche.yaml", BEREICH_TEXTE)
    };
    let outcome = basis().laden();
    assert!(outcome.report.findings().is_empty(), "{}", alle(&outcome));
    let m = outcome.data.unwrap().catalog.management;
    assert!(m.enabled());
    assert_eq!(m.levels.len(), 4);
    assert_eq!(m.specialists_of(SiteType::Factory).len(), 5);
    assert!(m.specialists_of(SiteType::ResearchCenter).is_empty());
    // Without the section there are no managers.
    assert!(
        !Daten::neu()
            .laden()
            .data
            .unwrap()
            .catalog
            .management
            .enabled()
    );

    let d = basis().ersetze(
        datei,
        "themen: [produktion, ueberkapazitaet",
        "themen: [produktio, ueberkapazitaet",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Thema „produktio“ ist nicht definiert. Meinten Sie „produktion“?",
    );
    assert_ort(
        f,
        datei,
        d.zeile(datei, "themen: [produktio,"),
        "management.bereiche[0].themen[0]",
    );

    let d = basis().ersetze(
        datei,
        "themen: [verkauf, werbung]",
        "themen: [verkauf, werbung, produktion]",
    );
    befund(
        &d.laden(),
        "Das Thema „produktion“ gehört schon zum Bereich „produktion“; jedes Thema gehört zu höchstens einem Bereich.",
    );

    let d = basis().ersetze(
        datei,
        "    - {id: vorstand, pruefung_tage: 30, gehalt_fach: 10, gehalt_leitung: 15,\n       budget_fach: [0.03, 0.08], budget_leitung: [0.08, 0.20],\n       fachstellen: [produktion, einkauf_lager, vertrieb_marketing, personal, forschung,\n                     finanzen, strategie, recht],\n       themen: [ueberkapazitaet, stillgelegt, wiederanfahren, ausbau, kraftwerk,\n                lagerstaette, werbung, kasse, kaufangebot, antwort, engpass, lizenz,\n                umschuldung, gehaltsrunde, startup, dividende]}\n",
        "",
    );
    befund(&d.laden(), "Eintrag für Ebene „vorstand“ fehlt.");

    let d = basis().ersetze(datei, "pruefung_tage: 7,", "pruefung_tage: 0,");
    befund(&d.laden(), "Wert 0 muss größer als 0 sein.");

    let d = basis().ersetze(datei, "lager: [logistik]", "lager: [logistk]");
    befund(
        &d.laden(),
        "Bereich „logistk“ ist nicht definiert. Meinten Sie „logistik“?",
    );

    let d = basis().ersetze(
        datei,
        "    lager: [logistik]\n",
        "    lagerhaus: [logistik]\n",
    );
    let outcome = d.laden();
    let f = outcome
        .report
        .findings()
        .iter()
        .find(|f| f.message.starts_with("Unbekannter Wert „lagerhaus“."))
        .unwrap_or_else(|| panic!("{}", alle(&outcome)));
    assert_eq!(f.path.to_string(), "management.standorttypen.lagerhaus");

    let d = basis().ersetze(datei, "max: 60", "max: 6");
    befund(
        &d.laden(),
        "Der Pool braucht `max` (6) mindestens so groß wie `min` (12).",
    );

    let d = basis().ersetze(datei, "abgang_monat: 0.15", "abgang_monat: 1.5");
    befund(
        &d.laden(),
        "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );

    let d = basis().ersetze(datei, "bemerken_grund: 0.5", "bemerken_grund: 1.2");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 1.2 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_ort(
        f,
        datei,
        d.zeile(datei, "bemerken_grund: 1.2"),
        "management.bemerken_grund",
    );

    let d = basis().ersetze(
        datei,
        "budget_fach: [0.02, 0.05]",
        "budget_fach: [0.06, 0.05]",
    );
    befund(
        &d.laden(),
        "Das Budget je Entscheidung (0.06) darf nicht größer sein als das je Jahr (0.05).",
    );
    let d = basis().ersetze(
        datei,
        "routine_themen: [produktion,",
        "routine_themen: [produktio,",
    );
    befund(
        &d.laden(),
        "Thema „produktio“ ist nicht definiert. Meinten Sie „produktion“?",
    );
    // MA6: the living market of managers.
    let d = basis().ersetze(datei, "stufen: [30, 60]", "stufen: [60, 30]");
    befund(
        &d.laden(),
        "Die Stufen der Zufriedenheit steigen: „unzufrieden“ bis 60 darf nicht über „gemischt“ bis 30 liegen.",
    );
    let d = basis().ersetze(datei, "chance_monat: 0.3", "chance_monat: 1.5");
    befund(
        &d.laden(),
        "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    let d = basis().ersetze(datei, "ki_gegen_max: 1.3", "ki_gegen_max: 0.5");
    befund(
        &d.laden(),
        "Wert 0.5 liegt außerhalb des erlaubten Bereichs 1 bis 10.",
    );
    // MA5: a topic follows the rules either as routine or as a topic of costs only.
    let d = basis().ersetze(
        datei,
        "regel_themen: [kasse, werbung, gehaltsrunde]",
        "regel_themen: [kasse, werbung, gehaltsrunde, lohn]",
    );
    befund(
        &d.laden(),
        "Thema „lohn“ ist doppelt definiert; erste Definition in routine_themen.",
    );
    let d = basis().ersetze(
        datei,
        "regel_themen: [kasse, werbung, gehaltsrunde]",
        "regel_themen: [kasse, werbng, gehaltsrunde]",
    );
    befund(
        &d.laden(),
        "Thema „werbng“ ist nicht definiert. Meinten Sie „werbung“?",
    );
    // MA3: specialists and topics of the levels, bundling.
    let d = basis().ersetze(
        datei,
        "budget_fach: [0.02, 0.05], budget_leitung: [0.05, 0.10],",
        "budget_fach: [0.02, 0.05], budget_leitung: [0.05, 0.10], fachstellen: [produktion],",
    );
    befund(
        &d.laden(),
        "Die Fachstellen der Standorte stehen je Standorttyp unter `standorttypen`, nicht bei der Ebene.",
    );
    let d = basis().ersetze(
        datei,
        "fachstellen: [produktion, einkauf_lager,",
        "fachstellen: [produktio, einkauf_lager,",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Bereich „produktio“ ist nicht definiert. Meinten Sie „produktion“?",
    );
    assert_eq!(f.path.to_string(), "management.ebenen[1].fachstellen[0]");
    let d = basis().ersetze(
        datei,
        "lagerstaette, werbung]}",
        "lagerstaette, produktname]}",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Das Thema „produktname“ gehört zu keinem Bereich; eine Ebene kann nur Themen ihrer Bereiche aufgreifen.",
    );
    assert_eq!(f.path.to_string(), "management.ebenen[1].themen[6]");
    let d = basis().ersetze(datei, "buendel_ab: 3", "buendel_ab: 0");
    let outcome = d.laden();
    let f = befund(&outcome, "Wert 0 muss größer als 0 sein.");
    assert_eq!(f.path.to_string(), "management.anliegen.buendel_ab");

    // MG2: review months per topic and tolerance.
    let m = outcome_ok_months(&basis);
    assert_eq!(m.review_months(wsim_core::decision::Topic::Expansion), 12);
    assert_eq!(m.review_months(wsim_core::decision::Topic::Production), 3);
    assert_eq!(
        m.review_months(wsim_core::decision::Topic::Dividend),
        6,
        "standard for topics not named"
    );
    let d = basis().ersetze(datei, "ausbau: 12,", "ausbau: 99,");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 99 liegt außerhalb des erlaubten Bereichs 1 bis 60.",
    );
    assert_eq!(
        f.path.to_string(),
        "management.anliegen.wirkzeit_monate.ausbau"
    );
    let d = basis().ersetze(datei, "ausbau: 12,", "ausbao: 12,");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Thema „ausbao“ ist nicht definiert. Meinten Sie „ausbau“?",
    );
    assert_eq!(
        f.path.to_string(),
        "management.anliegen.wirkzeit_monate.ausbao"
    );
    let d = basis().ersetze(
        datei,
        "nichtstun_risiko: {standard: 0.1,",
        "nichtstun_risiko: {standard: 3,",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 3 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_eq!(
        f.path.to_string(),
        "management.anliegen.nichtstun_risiko.standard"
    );
    let d = basis().ersetze(datei, "- [verkauf, produktion]", "- [verkauf, verkauf]");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Ein Widerspruchspaar braucht zwei verschiedene Themen.",
    );
    assert_eq!(f.path.to_string(), "management.widerspruch_themen[0]");
    let d = basis().ersetze(datei, "- [werbung, verkauf]", "- [werbunk, verkauf]");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Thema „werbunk“ ist nicht definiert. Meinten Sie „werbung“?",
    );
    assert_eq!(f.path.to_string(), "management.widerspruch_themen[1]");
    let d = basis().ersetze(datei, "treffer_toleranz: 0.1", "treffer_toleranz: 2");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 2 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_eq!(f.path.to_string(), "management.anliegen.treffer_toleranz");

    let d = basis().ersetze(datei, "frist_tage: 30", "frist_tage: 0");
    let outcome = d.laden();
    let f = befund(&outcome, "Wert 0 muss größer als 0 sein.");
    assert_ort(
        f,
        datei,
        d.zeile(datei, "frist_tage: 0"),
        "management.anliegen.frist_tage",
    );

    // MA4: strategies.
    let d = basis().ersetze(
        datei,
        "premium: {untergrenze: 1.35,",
        "premium: {untergrenze: 0,",
    );
    let outcome = d.laden();
    let f = befund(&outcome, "Wert 0 muss größer als 0 sein.");
    assert_ort(
        f,
        datei,
        d.zeile(datei, "premium: {untergrenze: 0,"),
        "management.strategie.premium.untergrenze",
    );
    let d = basis().ersetze(datei, "aufschlag: -0.1}", "aufschlag: -1.5}");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert -1.5 liegt außerhalb des erlaubten Bereichs -0.9 bis 2.",
    );
    assert_eq!(
        f.path.to_string(),
        "management.strategie.kampfpreis.aufschlag"
    );
    let d = basis().ersetze(datei, "lager_tage_max: 180", "lager_tage_max: 10");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "`lager_tage_max` (10) liegt unter den Lagertagen der KI (20); ihre Werte sind die Standardwerte der Vorgabe „Lager“ und müssen sich einstellen lassen.",
    );
    assert_ort(
        f,
        datei,
        d.zeile(datei, "lager_tage_max: 10"),
        "management.strategie.lager_tage_max",
    );
    let d = basis().ersetze(
        datei,
        "liquiditaet_monate_max: 24",
        "liquiditaet_monate_max: -1",
    );
    befund(&d.laden(), "Wert -1 muss größer als 0 sein.");

    // MA5: the board and the mandate.
    let m = basis().laden().data.unwrap().catalog.management;
    assert_eq!(m.levels[3].specialists.len(), 8);
    assert!(
        m.levels[3]
            .topics
            .contains(&wsim_core::decision::Topic::OfferAnswer)
    );
    let d = basis().ersetze(datei, "sicherheit: 0.2,", "sicherheit: 1.2,");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 1.2 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_eq!(
        f.path.to_string(),
        "management.strategieauftrag.leitlinien.sicherheit"
    );
    let d = basis().ersetze(datei, "antraege_max: 3", "antraege_max: 0");
    let outcome = d.laden();
    let f = befund(&outcome, "Wert 0 muss größer als 0 sein.");
    assert_ort(
        f,
        datei,
        d.zeile(datei, "antraege_max: 0"),
        "management.strategieauftrag.antraege_max",
    );
    let d = basis().ersetze(datei, "{id: vorsichtig,", "{id: offensiv,");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Stil „offensiv“ ist doppelt definiert; erste Definition in Eintrag 1.",
    );
    assert_eq!(f.path.to_string(), "management.stil.arten[1].id");
    let d = basis().ersetze(
        datei,
        "vorschlag_aufschub_tage: 30",
        "vorschlag_aufschub_tage: 0",
    );
    let outcome = d.laden();
    let f = befund(&outcome, "Wert 0 muss größer als 0 sein.");
    assert_ort(
        f,
        datei,
        d.zeile(datei, "vorschlag_aufschub_tage: 0"),
        "management.strategieauftrag.vorschlag_aufschub_tage",
    );
    let d = basis().ersetze(
        datei,
        "kaufangebot, antwort, startup]",
        "kaufangebot, antwrt, startup]",
    );
    befund(
        &d.laden(),
        "Thema „antwrt“ ist nicht definiert. Meinten Sie „antwort“?",
    );

    let d = basis().ersetze("texte/de/bereiche.yaml", "bereich.logistik: Logistik\n", "");
    let outcome = d.laden();
    let f = befund(&outcome, "Text „bereich.logistik“ fehlt in texte/de/.");
    assert_ort(
        f,
        datei,
        d.zeile(datei, "- id: logistik"),
        "management.bereiche[4].id",
    );
    let d = basis().ersetze(
        "texte/de/bereiche.yaml",
        "bereich.finanzen: Finanzen\n",
        "bereich.finanzen: Finanzen\nbereich.vertrieb: Vertrieb\n",
    );
    befund(
        &d.laden(),
        "Text „bereich.vertrieb“ gehört zu keinem Eintrag.",
    );

    let d = Daten::neu()
        .datei(datei, MANAGEMENT)
        .datei("texte/de/bereiche.yaml", BEREICH_TEXTE);
    befund(
        &d.laden(),
        "Arbeitskräftegruppe „akademiker.kaufmaennisch“ ist nicht definiert.",
    );
}

const ZENTRALE: &str = "zentrale:
  hauptsitz: {verlegung_monate: 6, kosten_grund_usd: 250000, kosten_je_angestelltem_usd: 25000,
              mitziehen: 0.6}
  annaeherung: true
";

/// Headquarters and central departments (ZA1–ZA3).
#[test]
fn zentrale_wird_geprueft() {
    let datei = "parameter/zentrale.yaml";
    let basis = || Daten::neu().datei(datei, ZENTRALE);
    let outcome = basis().laden();
    assert!(outcome.report.findings().is_empty(), "{}", alle(&outcome));
    let c = outcome.data.unwrap().catalog.central;
    assert_eq!(c.headquarters.months, 6);
    assert!((c.headquarters.moving_share - 0.6).abs() < 1e-9);
    // Optional: without it a move is free.
    let outcome = Daten::neu().laden();
    assert!(outcome.report.findings().is_empty(), "{}", alle(&outcome));
    let d = basis().ersetze(datei, "mitziehen: 0.6", "mitziehen: 1.5");
    befund(
        &d.laden(),
        "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    let d = basis().ersetze(datei, "verlegung_monate: 6", "verlegung_monate: 100");
    befund(
        &d.laden(),
        "Wert 100 liegt außerhalb des erlaubten Bereichs 0 bis 60.",
    );
    let d = basis().ersetze(datei, "kosten_grund_usd: 250000", "kosten_grund_usd: -5");
    befund(&d.laden(), "-5");
    let d = basis().datei("parameter/zentrale2.yaml", ZENTRALE);
    befund(&d.laden(), "Abschnitt „zentrale“ darf es nur einmal geben");

    // ZA2, ZA3: departments headed by the board's members, recommendations, hit rate.
    let management = MANAGEMENT.replace("akademiker.kaufmaennisch", "fachkraft.metall");
    let mit_abteilungen = ZENTRALE.replace(
        "  annaeherung: true\n",
        "  abteilungen:
    - {id: strategie, bereich: strategie, lohngruppe: fachkraft.metall, buero_usd: 15000,
       faelle: 2, wirkung: 0}
    - {id: marketing, bereich: vertrieb_marketing, lohngruppe: fachkraft.metall,
       buero_usd: 15000, faelle: 5, wirkung: 0.3}
  genauigkeit: 0.5
  umschuldung: {mindestvorteil: 0.01, gebuehr: 0.01}
  trefferquote: {bewertung_monate: 12, mittelwert: 0.5, vorgewicht: 4, k: 2}
  annaeherung: true
",
    );
    let basis = || {
        Daten::neu()
            .datei(datei, &mit_abteilungen)
            .datei("parameter/management.yaml", &management)
            .datei("texte/de/bereiche.yaml", BEREICH_TEXTE)
    };
    let outcome = basis().laden();
    assert!(outcome.report.findings().is_empty(), "{}", alle(&outcome));
    let c = outcome.data.unwrap().catalog.central;
    assert_eq!(c.departments.len(), 2);
    assert_eq!(
        c.departments[1].kind,
        wsim_core::catalog::DepartmentKind::Marketing
    );
    assert!((c.accuracy - 0.5).abs() < 1e-9);
    assert_eq!(c.hit_rate.months, 12);
    let d = basis().ersetze(datei, "{id: strategie, bereich", "{id: strategy, bereich");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Abteilung „strategy“ ist nicht definiert. Meinten Sie „strategie“?",
    );
    assert_ort(
        f,
        datei,
        d.zeile(datei, "{id: strategy,"),
        "zentrale.abteilungen[0].id",
    );
    let d = basis().ersetze(datei, "{id: marketing, bereich", "{id: strategie, bereich");
    befund(
        &d.laden(),
        "Abteilung „strategie“ ist doppelt definiert; erste Definition in abteilungen.",
    );
    let d = basis().ersetze(datei, "bereich: vertrieb_marketing", "bereich: logistik");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Abteilung „marketing“: Bereich „logistik“ ist kein Ressort des Vorstands",
    );
    assert_eq!(f.path.to_string(), "zentrale.abteilungen[1].bereich");
    let d = basis().ersetze(
        "parameter/management.yaml",
        "finanzen, strategie, recht],",
        "finanzen, recht],",
    );
    befund(
        &d.laden(),
        "Abteilung „strategie“: Bereich „strategie“ ist kein Ressort des Vorstands",
    );
    let d = basis().ersetze(datei, "bereich: strategie,", "bereich: strategi,");
    befund(
        &d.laden(),
        "Bereich „strategi“ ist nicht definiert. Meinten Sie „strategie“?",
    );
    let d = basis().ersetze(
        datei,
        "lohngruppe: fachkraft.metall, buero_usd: 15000,\n       faelle: 2",
        "lohngruppe: akademiker, buero_usd: 15000,\n       faelle: 2",
    );
    befund(
        &d.laden(),
        "Arbeitskräftegruppe „akademiker“ ist nicht definiert.",
    );
    let d = basis().ersetze(datei, "faelle: 5, wirkung: 0.3", "faelle: 5, wirkung: 1.3");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 1.3 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_eq!(f.path.to_string(), "zentrale.abteilungen[1].wirkung");
    let d = basis().ersetze(datei, "faelle: 2, wirkung: 0}", "faelle: -2, wirkung: 0}");
    befund(&d.laden(), "-2");
    let d = basis().ersetze(datei, "genauigkeit: 0.5", "genauigkeit: 2");
    befund(
        &d.laden(),
        "Wert 2 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    let d = basis().ersetze(datei, "gebuehr: 0.01", "gebuehr: 1.5");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_eq!(f.path.to_string(), "zentrale.umschuldung.gebuehr");
    let d = basis().ersetze(datei, "bewertung_monate: 12", "bewertung_monate: 0");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 0 liegt außerhalb des erlaubten Bereichs 1 bis 120.",
    );
    assert_eq!(f.path.to_string(), "zentrale.trefferquote.bewertung_monate");
    let d = basis().ersetze(datei, "k: 2}", "k: 20}");
    befund(
        &d.laden(),
        "Wert 20 liegt außerhalb des erlaubten Bereichs 0 bis 10.",
    );

    // ZA4: central departments and headquarters of the AI companies.
    let mit_ki = mit_abteilungen.replace(
        "  annaeherung: true\n",
        "  ki:
    anteil_umsatz: {bei_0: 0, bei_1: 0.02}
    reihenfolge: [marketing, strategie]
    mindestlast: {marketing: 2}
    sitz: {anteil_umsatz_min: 0.25, bip_anteil_min: 0.75, amortisation_jahre: 3,
           sperre_jahre: 10}
  annaeherung: true
",
    );
    let basis = || {
        Daten::neu()
            .datei(datei, &mit_ki)
            .datei("parameter/management.yaml", &management)
            .datei("texte/de/bereiche.yaml", BEREICH_TEXTE)
    };
    let outcome = basis().laden();
    assert!(outcome.report.findings().is_empty(), "{}", alle(&outcome));
    let ai = outcome.data.unwrap().catalog.central.ai;
    use wsim_core::catalog::DepartmentKind::{Marketing, Strategy};
    assert_eq!(ai.order, vec![(Marketing, 2.0), (Strategy, 1.0)]);
    assert!((ai.revenue_share.at(0.5) - 0.01).abs() < 1e-12);
    assert_eq!(ai.lock_years, 10);
    // Without the block AI companies set up nothing and never move.
    let outcome = Daten::neu()
        .datei(datei, &mit_abteilungen)
        .datei("parameter/management.yaml", &management)
        .datei("texte/de/bereiche.yaml", BEREICH_TEXTE)
        .laden();
    let ai = outcome.data.unwrap().catalog.central.ai;
    assert!(ai.order.is_empty() && ai.payback_years == 0.0);
    let d = basis().ersetze(
        datei,
        "reihenfolge: [marketing, strategie]",
        "reihenfolge: [marketing, finanzen]",
    );
    let outcome = d.laden();
    let f = befund(&outcome, "Abteilung „finanzen“ ist nicht definiert.");
    assert_ort(
        f,
        datei,
        d.zeile(datei, "reihenfolge:"),
        "zentrale.ki.reihenfolge[1]",
    );
    let d = basis().ersetze(
        datei,
        "reihenfolge: [marketing, strategie]",
        "reihenfolge: [marketing, marketing]",
    );
    befund(
        &d.laden(),
        "Abteilung „marketing“ ist doppelt definiert; erste Definition in reihenfolge.",
    );
    let d = basis().ersetze(
        datei,
        "mindestlast: {marketing: 2}",
        "mindestlast: {marketng: 2}",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Abteilung „marketng“ ist nicht definiert. Meinten Sie „marketing“?",
    );
    assert_eq!(f.path.to_string(), "zentrale.ki.mindestlast.marketng");
    let d = basis().ersetze(
        datei,
        "mindestlast: {marketing: 2}",
        "mindestlast: {marketing: -2}",
    );
    befund(&d.laden(), "-2");
    let d = basis().ersetze(datei, "bei_1: 0.02}", "bei_1: 1.5}");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_eq!(f.path.to_string(), "zentrale.ki.anteil_umsatz.bei_1");
    let d = basis().ersetze(datei, "anteil_umsatz_min: 0.25", "anteil_umsatz_min: 2");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 2 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_eq!(f.path.to_string(), "zentrale.ki.sitz.anteil_umsatz_min");
    let d = basis().ersetze(datei, "bip_anteil_min: 0.75", "bip_anteil_min: -1");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert -1 liegt außerhalb des erlaubten Bereichs 0 bis 10.",
    );
    assert_eq!(f.path.to_string(), "zentrale.ki.sitz.bip_anteil_min");
    let d = basis().ersetze(datei, "sperre_jahre: 10", "sperre_jahre: 500");
    befund(
        &d.laden(),
        "Wert 500 liegt außerhalb des erlaubten Bereichs 0 bis 100.",
    );
}

const STARTUPS: &str = include_str!("../../../data/parameter/startups.yaml");
const STARTUP_TEXTE: &str = include_str!("../../../data/texte/de/startups.yaml");
const ERFINDER: &str = "erfinder:
  - {technologie: schmelzen, name: Ada Muster, land: SWE, quelle: Test}
";

#[test]
fn startups_werden_geprueft() {
    let datei = "parameter/startups.yaml";
    let erfinder = "startups/erfinder.yaml";
    let basis = || {
        Daten::neu()
            .datei(datei, STARTUPS)
            .datei(erfinder, ERFINDER)
            .datei("texte/de/startups.yaml", STARTUP_TEXTE)
    };
    let outcome = basis().laden();
    assert!(outcome.report.findings().is_empty(), "{}", alle(&outcome));
    let v = outcome.data.unwrap().catalog.ventures;
    assert!(v.enabled());
    assert_eq!(v.phases.len(), 3);
    assert_eq!(v.frequencies.len(), 4);
    assert_eq!(v.frequencies[v.default_frequency].0, "normal");
    assert!((v.default_factor() - 1.0).abs() < 1e-9);
    assert_eq!(v.label(1950), Some("erfinder"));
    assert_eq!(v.label(1995), Some("startups"));
    assert_eq!(v.inventors.len(), 1);
    assert_eq!(v.inventors[0].name, "Ada Muster");
    // Optional: without the section there are none; inventors alone are a warning.
    let outcome = Daten::neu().laden();
    assert!(outcome.report.findings().is_empty(), "{}", alle(&outcome));
    let outcome = Daten::neu().datei(erfinder, ERFINDER).laden();
    befund(
        &outcome,
        "Ohne Abschnitt „startups“ gibt es keine Start-ups; die Erfinder bleiben ungenutzt.",
    );
    assert_eq!(outcome.report.errors().count(), 0, "{}", alle(&outcome));

    // The inventors: a known technology once, a known country, a name.
    let d = basis().ersetze(erfinder, "technologie: schmelzen", "technologie: schmelzn");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Technologie „schmelzn“ ist nicht definiert. Meinten Sie „schmelzen“?",
    );
    assert_ort(
        f,
        erfinder,
        d.zeile(erfinder, "technologie: schmelzn"),
        "erfinder[0].technologie",
    );
    let d = basis().datei(
        erfinder,
        &format!("{ERFINDER}  - {{technologie: schmelzen, name: Bo Beispiel, land: SWE}}\n"),
    );
    befund(
        &d.laden(),
        "Erfinder der Technologie „schmelzen“ ist doppelt definiert",
    );
    let d = basis().ersetze(erfinder, "land: SWE", "land: XYZ");
    befund(&d.laden(), "Land „XYZ“ ist nicht definiert.");
    let d = basis().ersetze(erfinder, "name: Ada Muster", "name: \" \"");
    befund(&d.laden(), "„name“ darf nicht leer sein.");

    // The parameters.
    let d = basis().ersetze(datei, "chance: 0.65", "chance: 1.5");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_eq!(f.path.to_string(), "startups.phasen[0].chance");
    let d = basis().ersetze(datei, "monate: 12,", "monate: 0,");
    let outcome = d.laden();
    let f = befund(&outcome, "Wert 0 muss größer als 0 sein.");
    assert_eq!(f.path.to_string(), "startups.phasen[0].monate");
    let d = basis().ersetze(datei, "{id: prototyp,", "{id: idee,");
    befund(
        &d.laden(),
        "Phase „idee“ ist doppelt definiert; erste Definition in phasen.",
    );
    let d = basis().ersetze(
        datei,
        "haeufigkeit_standard: normal",
        "haeufigkeit_standard: mittel",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Häufigkeit „mittel“ ist unter „haeufigkeiten“ nicht aufgeführt.",
    );
    assert_eq!(f.path.to_string(), "startups.haeufigkeit_standard");
    let d = basis().ersetze(datei, "{id: viele, faktor: 2}", "{id: viele, faktor: -2}");
    befund(&d.laden(), "-2");
    let d = basis().ersetze(datei, "kapital_faktor_max: 2", "kapital_faktor_max: 0.01");
    befund(
        &d.laden(),
        "„kapital_faktor_min“ muss kleiner als „kapital_faktor_max“ sein.",
    );
    let d = basis().ersetze(datei, "stufe_hoch_ab: 0.4", "stufe_hoch_ab: 0.1");
    befund(
        &d.laden(),
        "„stufe_mittel_ab“ muss kleiner als „stufe_hoch_ab“ sein.",
    );
    let d = basis().ersetze(
        datei,
        "{id: wagniskapital, ab: 1970}",
        "{id: wagniskapital, ab: 1890}",
    );
    befund(
        &d.laden(),
        "Die Zeiträume müssen nach „ab“ aufsteigend sortiert sein.",
    );
    let d = basis().ersetze(datei, "frist_monate: 6", "frist_monate: 0");
    let outcome = d.laden();
    let f = befund(&outcome, "Wert 0 muss größer als 0 sein.");
    assert_eq!(f.path.to_string(), "startups.frist_monate");
    let d = basis().datei("parameter/startups2.yaml", STARTUPS);
    befund(&d.laden(), "Abschnitt „startups“ darf es nur einmal geben");

    // The stakes (SU2): shares between 0 and 1, the majority not below the blocking
    // minority, pace factors in their range.
    let stakes = basis().laden().data.unwrap().catalog.ventures.stakes;
    assert!((stakes.majority - 0.5).abs() < 1e-9);
    assert!((stakes.blocking - 0.25).abs() < 1e-9);
    assert!(stakes.fast.0 < 1.0 && stakes.thorough.0 > 1.0);
    let d = basis().ersetze(datei, "verkauf_abschlag: 0.2", "verkauf_abschlag: 1.5");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_ort(
        f,
        datei,
        d.zeile(datei, "verkauf_abschlag: 1.5"),
        "startups.beteiligung.verkauf_abschlag",
    );
    let d = basis().ersetze(datei, "mehrheit: 0.5", "mehrheit: 0.2");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "„sperrminoritaet“ muss kleiner als „mehrheit“ sein.",
    );
    assert_eq!(f.path.to_string(), "startups.beteiligung.mehrheit");
    let d = basis().ersetze(
        datei,
        "zuegig: {monate: 0.75, chance: 0.9}",
        "zuegig: {monate: 0, chance: 0.9}",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 0 liegt außerhalb des erlaubten Bereichs 0.1 bis 10.",
    );
    assert_eq!(
        f.path.to_string(),
        "startups.beteiligung.lenkung.zuegig.monate"
    );
    let d = basis().ersetze(datei, "    einsatz_kasse: 0.1\n", "");
    befund(&d.laden(), "einsatz_kasse");

    // Spin-offs and AI companies (SU3).
    let ai = basis().laden().data.unwrap().catalog.ventures.stakes.ai;
    assert!(ai.check_chance > 0.0 && ai.spin_off_sale < 1.0);
    let d = basis().ersetze(datei, "pruefen_chance: 0.25", "pruefen_chance: 1.5");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_ort(
        f,
        datei,
        d.zeile(datei, "pruefen_chance: 1.5"),
        "startups.beteiligung.ki.pruefen_chance",
    );
    let d = basis().ersetze(datei, "ausgruenden_verkauf: 0.4", "ausgruenden_verkauf: 1");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 1 liegt außerhalb des erlaubten Bereichs 0 bis 0.99.",
    );
    assert_eq!(
        f.path.to_string(),
        "startups.beteiligung.ki.ausgruenden_verkauf"
    );
    let d = basis().ersetze(
        datei,
        "ausgruendung_fortschritt_min: 0.1",
        "ausgruendung_fortschritt_min: -0.1",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert -0.1 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_eq!(
        f.path.to_string(),
        "startups.beteiligung.ausgruendung_fortschritt_min"
    );

    // Every phase, label and choice needs its text.
    let d = basis().ersetze(
        "texte/de/startups.yaml",
        "startup.phase.prototyp: Prototyp\n",
        "",
    );
    befund(
        &d.laden(),
        "Text „startup.phase.prototyp“ fehlt in texte/de/.",
    );
    let d = basis().ersetze(
        "texte/de/startups.yaml",
        "startup.haeufigkeit.viele: Viele\n",
        "",
    );
    befund(
        &d.laden(),
        "Text „startup.haeufigkeit.viele“ fehlt in texte/de/.",
    );
    let d = basis().ersetze(
        "texte/de/startups.yaml",
        "startup.lenkung.zuegig: Zügig\n",
        "",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Text „startup.lenkung.zuegig“ fehlt in texte/de/.",
    );
    assert_eq!(f.path.to_string(), "startups.beteiligung.lenkung");
}

/// Cities of a country (W2): IDs, once each, one capital, a text each.
#[test]
fn staedte_werden_geprueft() {
    let staedte = |liste: &str, texte: &str| {
        Daten::neu()
            .ersetze(
                "laender/SWE.yaml",
                "    binnenland: false",
                &format!("    binnenland: false\n    staedte:\n{liste}"),
            )
            .ersetze(
                "texte/de/a.yaml",
                "land.SWE: Schweden\n",
                &format!("land.SWE: Schweden\n{texte}"),
            )
    };
    let zwei = "      - {id: malmoe, einwohner: 269_349, breite: 55.6, laenge: 13.0}
      - {id: stockholm, einwohner: 1_264_000, breite: 59.33, laenge: 18.07, hauptstadt: true}
";
    let texte = "stadt.SWE.stockholm: Stockholm\nstadt.SWE.malmoe: Malmö\n";
    let gut = staedte(zwei, texte).laden();
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let catalog = &gut.data.as_ref().unwrap().catalog;
    let swe = catalog.countries.get(catalog.countries.id("SWE").unwrap());
    // The most populous first.
    let keys: Vec<&str> = swe.cities.iter().map(|c| c.key.as_str()).collect();
    assert_eq!(keys, ["stockholm", "malmoe"]);
    assert!(swe.cities[0].capital && !swe.cities[1].capital);

    let ohne_text = staedte(zwei, "stadt.SWE.stockholm: Stockholm\n").laden();
    let f = befund(&ohne_text, "Text „stadt.SWE.malmoe“ fehlt");
    assert_eq!(f.path.to_string(), "laender[0].staedte[0].id");
    befund(
        &staedte(&zwei.replace("malmoe", "stockholm"), texte).laden(),
        "Die Stadt „stockholm“ steht zweimal in diesem Land.",
    );
    befund(
        &staedte(
            &zwei.replace("laenge: 13.0}", "laenge: 13.0, hauptstadt: true}"),
            texte,
        )
        .laden(),
        "höchstens eine Hauptstadt",
    );
    befund(
        &staedte(&zwei.replace("id: malmoe", "id: Malmö"), texte).laden(),
        "Ungültige ID „Malmö“",
    );
    let ungenutzt = staedte(zwei, &format!("{texte}stadt.SWE.atlantis: Atlantis\n")).laden();
    let f = befund(
        &ungenutzt,
        "Text „stadt.SWE.atlantis“ gehört zu keinem Eintrag.",
    );
    assert_eq!(f.severity, Severity::Warning);
    let leer = staedte(&zwei.replace("einwohner: 269_349", "einwohner: 0"), texte).laden();
    let f = befund(&leer, "Wert 0 muss größer als 0 sein.");
    assert_eq!(f.path.to_string(), "laender[0].staedte[0].einwohner");

    // The city model of the headquarters.
    let datei = "parameter/zentrale.yaml";
    let mit_stadt = ZENTRALE.replace(
        "  annaeherung: true\n",
        "  stadt: {akademiker_konzentration: 4, anteil_zentralen: 0.05, buero_bezug_einwohner: 1_000_000,
          buero_elastizitaet: 0.15, umzug_im_land: 0.5, einwohner_jahr: 2020}
  annaeherung: true
",
    );
    let outcome = Daten::neu().datei(datei, &mit_stadt).laden();
    assert!(outcome.report.findings().is_empty(), "{}", alle(&outcome));
    let m = outcome
        .data
        .unwrap()
        .catalog
        .central
        .city
        .expect("Stadtmodell");
    assert!((m.hq_share - 0.05).abs() < 1e-12);
    assert_eq!(m.population_year, 2020);
    let zu_gross = Daten::neu()
        .datei(
            datei,
            &mit_stadt.replace("anteil_zentralen: 0.05", "anteil_zentralen: 1.5"),
        )
        .laden();
    let f = befund(
        &zu_gross,
        "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_eq!(f.path.to_string(), "zentrale.stadt.anteil_zentralen");

    // The workload of the strategy department (MG9).
    let mit_last = ZENTRALE.replace(
        "  annaeherung: true\n",
        "  strategie_last: {je_branche: 0.5, je_land: 0.25, je_beteiligung: 0.5}\n  annaeherung: true\n",
    );
    let outcome = Daten::neu().datei(datei, &mit_last).laden();
    assert!(outcome.report.findings().is_empty(), "{}", alle(&outcome));
    let m = outcome.data.unwrap().catalog.central.strategy_load;
    assert!((m.per_branch - 0.5).abs() < 1e-12 && (m.per_participation - 0.5).abs() < 1e-12);
    let falsch = Daten::neu()
        .datei(datei, &mit_last.replace("je_land: 0.25", "je_land: 11"))
        .laden();
    let f = befund(
        &falsch,
        "Wert 11 liegt außerhalb des erlaubten Bereichs 0 bis 10.",
    );
    assert_eq!(f.path.to_string(), "zentrale.strategie_last.je_land");
}

const ZOELLE: &str = "\
zoelle:
  standard: {1900: 0.15, 2026: 0.06}
  laender:
    SWE: {1900: 0.1, 2026: 0.03}
  warengruppen: {erze: 0.1}
  zonen:
    - id: nordisch
      faktor: 0
      mitglieder: {SWE: [1960]}
  sperren: []
  dynamik:
    standardabweichung: 0.01
    minimum: 0
    maximum: 0.6
    stufen: [{id: keine, faktor: 0}, {id: normal, faktor: 1}]
    standard: normal
  annaeherung: true
";

const ZOLL_TEXTE: &str = "\
zoll.zone.nordisch: Nordisch
zoll.dynamik.keine: Fest
zoll.dynamik.normal: Normal
";

#[test]
fn zoelle_werden_geprueft() {
    let datei = "parameter/zoelle.yaml";
    let zoelle = |alt: &str, neu: &str| {
        Daten::neu()
            .datei(datei, &ZOELLE.replacen(alt, neu, 1))
            .datei("texte/de/zoelle.yaml", ZOLL_TEXTE)
            .laden()
    };
    let gut = zoelle("", "");
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let catalog = &gut.data.as_ref().unwrap().catalog;
    let t = &catalog.tariffs;
    assert!(t.enabled());
    assert_eq!(t.last_year(), 2026);
    assert!((t.groups[0] - 0.1).abs() < 1e-12);
    assert!(t.countries[0].is_some());
    assert!(t.products.iter().all(Option::is_none));
    assert_eq!(t.zones[0].members[0].1, 1960);
    assert!((t.default_factor() - 1.0).abs() < 1e-12);

    for (alt, neu, meldung) in [
        (
            "    SWE: {1900",
            "    NOR: {1900",
            "Land „NOR“ ist nicht definiert.",
        ),
        (
            "{erze: 0.1}",
            "{holz: 0.1}",
            "Warengruppe „holz“ ist nicht definiert.",
        ),
        (
            "  zonen:",
            "  produkte: {kupfer: 0}\n  zonen:",
            "Produkt „kupfer“ ist nicht definiert.",
        ),
        (
            "SWE: [1960]",
            "SWE: [1960, 1950]",
            "„Beitritt“ muss kleiner als „Austritt“ sein.",
        ),
        (
            "SWE: [1960]",
            "SWE: [1960, 1970, 1980]",
            "angegeben sind 3 Jahre",
        ),
        (
            "sperren: []",
            "sperren: [{laender: [SWE, SWE], von: 1950}]",
            "Eine Handelssperre braucht genau zwei verschiedene Länder.",
        ),
        (
            "standard: normal",
            "standard: stark",
            "Stufe „stark“ ist unter „stufen“ nicht aufgeführt.",
        ),
        (
            "{1900: 0.15, 2026",
            "{1900: 7, 2026",
            "Wert 7 liegt außerhalb des erlaubten Bereichs 0 bis 5.",
        ),
        (
            "maximum: 0.6",
            "maximum: 0.6\n    unbekannt: 1",
            "unbekannt",
        ),
    ] {
        befund(&zoelle(alt, neu), meldung);
    }

    // Every zone and level needs its text.
    let ohne_text = Daten::neu().datei(datei, ZOELLE).laden();
    let f = befund(&ohne_text, "Text „zoll.zone.nordisch“ fehlt");
    assert_eq!(f.path.to_string(), "zoelle.zonen[0].id");
    // Twice the section.
    let doppelt = Daten::neu()
        .datei(datei, ZOELLE)
        .datei("parameter/zoelle2.yaml", ZOELLE)
        .datei("texte/de/zoelle.yaml", ZOLL_TEXTE)
        .laden();
    befund(&doppelt, "Abschnitt „zoelle“ darf es nur einmal geben");
}

const VERTRAEGE: &str = "\
vertraege:
  laufzeit_monate_max: 60
  laufzeit_standard: 12
  strafe_max: 1.0
  strafe_standard: 0.2
  kuendigung_monate: 3
  angebot_tage: 30
  aufbewahren_monate: 12
  partnerschaft:
    aufbau_monate: 60
    verfall_monate: 120
    abzug_fehlmenge: 0.25
    rabatt_max: 0.1
    umsatz_rabatt_max: 0.1
    umsatz_ab: 0.2
    gesamt_max: 0.2
  ki:
    abschlag_verkauf: 0.08
    abschlag_je_monat: 0.005
    abschlag_max: 0.2
    aufschlag_kauf: 0.05
    aufschlag_je_monat: 0.0025
    aufschlag_max: 0.12
    anteil: 0.5
    strafe_max: 0.5
    angebot_chance: 0.25
";

#[test]
fn vertraege_werden_geprueft() {
    let datei = "parameter/vertraege.yaml";
    let vertraege = |alt: &str, neu: &str| {
        Daten::neu()
            .datei(datei, &VERTRAEGE.replacen(alt, neu, 1))
            .laden()
    };
    let gut = vertraege("", "");
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let m = &gut.data.as_ref().unwrap().catalog.contracts;
    assert!(m.enabled());
    assert_eq!(m.months_default, 12);
    assert!((m.ai.share - 0.5).abs() < 1e-12);
    assert_eq!(m.partnership.build_months, 60);
    assert!((m.partnership.discount(1.0, 0.2) - 0.2).abs() < 1e-12);

    for (alt, neu, meldung, pfad) in [
        (
            "rabatt_max: 0.1",
            "rabatt_max: 1.5",
            "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
            "vertraege.partnerschaft.rabatt_max",
        ),
        (
            "aufbau_monate: 60",
            "aufbau_monate: 0",
            "Wert 0 liegt außerhalb des erlaubten Bereichs 1 bis 1200.",
            "vertraege.partnerschaft.aufbau_monate",
        ),
        (
            "umsatz_ab: 0.2",
            "umsatz_ab: 0",
            "Wert 0 liegt außerhalb des erlaubten Bereichs 0.001 bis 1.",
            "vertraege.partnerschaft.umsatz_ab",
        ),
        (
            "aufbewahren_monate: 12",
            "aufbewahren_monate: 12\n  nachverhandlung_menge_min: 1.5",
            "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
            "vertraege.nachverhandlung_menge_min",
        ),
        (
            "aufbewahren_monate: 12",
            "aufbewahren_monate: 12\n  nachverhandlung_verlaengerung_monate: 500",
            "Wert 500 liegt außerhalb des erlaubten Bereichs 0 bis 120.",
            "vertraege.nachverhandlung_verlaengerung_monate",
        ),
        (
            "laufzeit_standard: 12",
            "laufzeit_standard: 72",
            "„laufzeit_standard“ muss kleiner als „laufzeit_monate_max“ sein.",
            "vertraege.laufzeit_standard",
        ),
        (
            "strafe_standard: 0.2",
            "strafe_standard: 2",
            "Wert 2 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
            "vertraege.strafe_standard",
        ),
        (
            "abschlag_max: 0.2",
            "abschlag_max: 0.01",
            "„abschlag_verkauf“ muss kleiner als „abschlag_max“ sein.",
            "vertraege.ki.abschlag_max",
        ),
        (
            "aufschlag_je_monat: 0.0025",
            "aufschlag_je_monat: 2",
            "Wert 2 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
            "vertraege.ki.aufschlag_je_monat",
        ),
        (
            "anteil: 0.5",
            "anteil: 1.5",
            "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
            "vertraege.ki.anteil",
        ),
        (
            "angebot_tage: 30",
            "angebot_tage: 0",
            "Wert 0 liegt außerhalb des erlaubten Bereichs 1 bis 365.",
            "vertraege.angebot_tage",
        ),
    ] {
        let outcome = vertraege(alt, neu);
        let f = befund(&outcome, meldung);
        assert_eq!(f.path.to_string(), pfad);
    }
    befund(&vertraege("  ki:", "  unbekannt: 1\n  ki:"), "unbekannt");
    // Without the section there are no contracts.
    let ohne = Daten::neu().laden();
    assert!(!ohne.data.unwrap().catalog.contracts.enabled());
}

const LOGISTIK: &str = "\
logistik:
  staat:
    aufschlag: 0.5
    risiko_faktor: 0.5
  flotte:
    marge_frachtmarkt: 0.15
    auslastung: 0.6
    unterhalt_anteil: 0.06
    nutzungsdauer_jahre: 20
    verkauf_anteil: 0.6
    vermietung_anteil: 0.5
    vermietung_markt_anteil: 0.25
  risiko:
    land: {1900: 0.002, 2000: 0.0003}
    see: {1900: 0.006, 2000: 0.001}
  ki:
    anteil: 0.5
    kasse_anteil: 0.2
  nachschub:
    tage_standard: 20
    tage_max: 365
    glaettung: 0.1
    ausverkauft_aufschlag: 0.25
    start_anteil: 0.05
";

#[test]
fn logistik_wird_geprueft() {
    let datei = "parameter/logistik.yaml";
    let logistik = |alt: &str, neu: &str| {
        Daten::neu()
            .datei(datei, &LOGISTIK.replacen(alt, neu, 1))
            .laden()
    };
    let gut = logistik("", "");
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let m = &gut.data.as_ref().unwrap().catalog.logistics;
    assert!(m.enabled);
    assert!((m.state_surcharge - 0.5).abs() < 1e-12);
    assert!((m.load - 0.6).abs() < 1e-12);
    assert!((m.supply_days_default - 20.0).abs() < 1e-12);

    for (alt, neu, meldung, pfad) in [
        (
            "auslastung: 0.6",
            "auslastung: 0",
            "Wert 0 liegt außerhalb des erlaubten Bereichs 0.01 bis 1.",
            "logistik.flotte.auslastung",
        ),
        (
            "marge_frachtmarkt: 0.15",
            "marge_frachtmarkt: 0.95",
            "Wert 0.95 liegt außerhalb des erlaubten Bereichs 0 bis 0.9.",
            "logistik.flotte.marge_frachtmarkt",
        ),
        (
            "nutzungsdauer_jahre: 20",
            "nutzungsdauer_jahre: 0",
            "Wert 0 muss größer als 0 sein.",
            "logistik.flotte.nutzungsdauer_jahre",
        ),
        (
            "see: {1900: 0.006",
            "see: {1900: 1.5",
            "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
            "logistik.risiko.see.1900",
        ),
        (
            "tage_standard: 20",
            "tage_standard: 400",
            "Wert 400 liegt außerhalb des erlaubten Bereichs 1 bis 365.",
            "logistik.nachschub.tage_standard",
        ),
        (
            "glaettung: 0.1",
            "glaettung: 0",
            "Wert 0 liegt außerhalb des erlaubten Bereichs 0.01 bis 1.",
            "logistik.nachschub.glaettung",
        ),
        (
            "start_anteil: 0.05",
            "start_anteil: 2",
            "Wert 2 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
            "logistik.nachschub.start_anteil",
        ),
        (
            "kasse_anteil: 0.2",
            "kasse_anteil: -0.2",
            "Wert -0.2 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
            "logistik.ki.kasse_anteil",
        ),
    ] {
        let outcome = logistik(alt, neu);
        let f = befund(&outcome, meldung);
        assert_eq!(f.path.to_string(), pfad);
    }
    befund(&logistik("  ki:", "  unbekannt: 1\n  ki:"), "unbekannt");
    // Without the section only the freight market.
    let ohne = Daten::neu().laden();
    assert!(!ohne.data.unwrap().catalog.logistics.enabled);
}

const TOCHTERFIRMEN: &str = "\
tochterfirmen:
  mindestkapital_usd: 25000
  gruendungskosten_usd: 3000
  geschaeftsfuehrung:
    kompetenz: 0.6
    aggressivitaet: 0.4
  logistik:
    kasse_anteil: 0.3
    rendite_min: 0.06
";

#[test]
fn tochterfirmen_werden_geprueft() {
    let datei = "parameter/tochterfirmen.yaml";
    let toechter = |alt: &str, neu: &str| {
        Daten::neu()
            .datei(datei, &TOCHTERFIRMEN.replacen(alt, neu, 1))
            .laden()
    };
    let gut = toechter("", "");
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let m = &gut.data.as_ref().unwrap().catalog.subsidiaries;
    assert!(m.enabled);
    assert!((m.competence - 0.6).abs() < 1e-12);
    assert_eq!(m.min_capital.to_usd(), 25_000.0);

    for (alt, neu, meldung, pfad) in [
        (
            "kompetenz: 0.6",
            "kompetenz: 1.6",
            "Wert 1.6 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
            "tochterfirmen.geschaeftsfuehrung.kompetenz",
        ),
        (
            "mindestkapital_usd: 25000",
            "mindestkapital_usd: -5",
            "-5",
            "tochterfirmen.mindestkapital_usd",
        ),
        (
            "kasse_anteil: 0.3",
            "kasse_anteil: 2",
            "Wert 2 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
            "tochterfirmen.logistik.kasse_anteil",
        ),
    ] {
        let outcome = toechter(alt, neu);
        let f = befund(&outcome, meldung);
        assert_eq!(f.path.to_string(), pfad);
    }
    befund(
        &toechter("  logistik:", "  unbekannt: 1\n  logistik:"),
        "unbekannt",
    );
    // Without the section there are no subsidiaries.
    let ohne = Daten::neu().laden();
    assert!(!ohne.data.unwrap().catalog.subsidiaries.enabled);
}

const BOERSE: &str = "\
boerse:
  bewertung:
    gewicht_buchwert: 0.4
    kgv: 12
    boden_buchwert: 0.3
    rendite_annahme: 0.08
    gewinn_monate: 24
  stimmung:
    schwankung: 0.04
    rueckkehr: 0.06
  traegheit: 0.3
  rauschen: 0.03
  boersengang:
    eigenkapital_min_usd: 2000000
    anteil_max: 0.4
    abschlag: 0.15
    kosten_anteil: 0.05
  handel:
    aufschlag: 0.02
    abschlag: 0.02
    preiswirkung: 0.5
    anteil_max: 0.5
  uebernahme:
    aufschlag: 0.3
    kosten_anteil: 0.02
  rueckkauf:
    anteil_max: 0.1
  start:
    eigenkapital_min_usd: 20000000
    streubesitz: 0.6
  ki:
    boersengang_chance: 0.02
    boersengang_anteil: 0.3
    depot_anteil_kasse: 0.2
    depot_anteil_max: 0.1
    unterbewertung: 0.25
    uebernahme_chance: 0.005
    uebernahme_kasse_anteil: 0.5
";

#[test]
fn boerse_wird_geprueft() {
    let datei = "parameter/boerse.yaml";
    let boerse = |alt: &str, neu: &str| {
        Daten::neu()
            .datei(datei, &BOERSE.replacen(alt, neu, 1))
            .laden()
    };
    let gut = boerse("", "");
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let m = &gut.data.as_ref().unwrap().catalog.stock;
    assert!(m.enabled);
    assert_eq!(m.earnings_months, 24);

    for (alt, neu, meldung, pfad) in [
        (
            "gewinn_monate: 24",
            "gewinn_monate: 36",
            "Wert 36 liegt außerhalb des erlaubten Bereichs 1 bis 24.",
            "boerse.bewertung.gewinn_monate",
        ),
        (
            "kgv: 12",
            "kgv: 0",
            "Wert 0 muss größer als 0 sein.",
            "boerse.bewertung.kgv",
        ),
        (
            "anteil_max: 0.4",
            "anteil_max: 0.95",
            "Wert 0.95 liegt außerhalb des erlaubten Bereichs 0.01 bis 0.9.",
            "boerse.boersengang.anteil_max",
        ),
        (
            "unterbewertung: 0.25",
            "unterbewertung: 1.5",
            "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 0.9.",
            "boerse.ki.unterbewertung",
        ),
        (
            "aufschlag: 0.3\n    kosten_anteil: 0.02",
            "aufschlag: -0.1\n    kosten_anteil: 0.02",
            "Wert -0.1 liegt außerhalb des erlaubten Bereichs 0 bis 5.",
            "boerse.uebernahme.aufschlag",
        ),
    ] {
        let outcome = boerse(alt, neu);
        let f = befund(&outcome, meldung);
        assert_eq!(f.path.to_string(), pfad);
    }
    befund(&boerse("  ki:", "  unbekannt: 1\n  ki:"), "unbekannt");
    // Without the section no company is listed.
    let ohne = Daten::neu().laden();
    assert!(!ohne.data.unwrap().catalog.stock.enabled);
}

const ANLEIHEN: &str = "\
anleihen:
  eigenkapital_min_usd: 10000000
  volumen_min_usd: 1000000
  laufzeit_jahre: {min: 3, max: 30}
  kosten_anteil: 0.015
  rueckkauf_aufschlag: 0.02
  gewinn_monate: 12
  bonitaet:
    - {stufe: aaa, verschuldung_max: 0.15, zinsdeckung_min: 10, aufschlag: 0.004}
    - {stufe: bbb, verschuldung_max: 0.45, zinsdeckung_min: 3, aufschlag: 0.018}
  ki:
    laufzeit_jahre: 10
    vorteil_min: 0.005
";

#[test]
fn anleihen_werden_geprueft() {
    let datei = "parameter/anleihen.yaml";
    let anleihen = |alt: &str, neu: &str| {
        Daten::neu()
            .datei(datei, &ANLEIHEN.replacen(alt, neu, 1))
            .laden()
    };
    let gut = anleihen("", "");
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let m = &gut.data.as_ref().unwrap().catalog.bonds;
    assert!(m.enabled);
    assert_eq!(m.grades.len(), 2);
    assert_eq!(m.grades[1].key, "bbb");
    assert_eq!((m.term_min_years, m.term_max_years), (3, 30));

    for (alt, neu, meldung, pfad) in [
        (
            "stufe: bbb",
            "stufe: xyz",
            "Unbekannter Wert „xyz“.",
            "anleihen.bonitaet[1].stufe",
        ),
        (
            "stufe: bbb",
            "stufe: aaa",
            "Bonitätsstufe „aaa“ ist doppelt definiert",
            "anleihen.bonitaet[1].stufe",
        ),
        (
            "aufschlag: 0.018",
            "aufschlag: 0.003",
            "Die Bonitätsstufen stehen von der besten zur schlechtesten: „bbb“ braucht nach „aaa“",
            "anleihen.bonitaet[1].stufe",
        ),
        (
            "{min: 3, max: 30}",
            "{min: 12, max: 5}",
            "Die kürzeste Laufzeit (12) ist länger als die längste (5).",
            "anleihen.laufzeit_jahre",
        ),
        (
            "gewinn_monate: 12",
            "gewinn_monate: 30",
            "Wert 30 liegt außerhalb des erlaubten Bereichs 1 bis 24.",
            "anleihen.gewinn_monate",
        ),
        (
            "laufzeit_jahre: 10",
            "laufzeit_jahre: 40",
            "Wert 40 liegt außerhalb des erlaubten Bereichs 3 bis 30.",
            "anleihen.ki.laufzeit_jahre",
        ),
    ] {
        let outcome = anleihen(alt, neu);
        let f = befund(&outcome, meldung);
        assert_eq!(f.path.to_string(), pfad);
    }
    befund(&anleihen("  ki:", "  unbekannt: 1\n  ki:"), "unbekannt");
    // Without the section no company issues bonds.
    let ohne = Daten::neu().laden();
    assert!(!ohne.data.unwrap().catalog.bonds.enabled);
}

const BANK: &str = "\
bank:
  einlagen:
    hebel_max: 10
    aufschlag_neutral: -0.01
    elastizitaet: 25
    anpassung: 0.15
  mindestreserve: 0.1
  start:
    einlagen_aufschlag: -0.01
    kreditnachlass: 0.1
    verschuldung_max: 0.5
";

#[test]
fn bank_wird_geprueft() {
    let datei = "parameter/bank.yaml";
    let bank = |alt: &str, neu: &str| {
        Daten::neu()
            .datei(datei, &BANK.replacen(alt, neu, 1))
            .laden()
    };
    let gut = bank("", "");
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let m = &gut.data.as_ref().unwrap().catalog.bank;
    assert!(m.enabled);
    assert_eq!(m.leverage_max, 10.0);
    assert_eq!(m.start_loan_discount, 0.1);

    for (alt, neu, meldung, pfad) in [
        (
            "hebel_max: 10",
            "hebel_max: 0",
            "Wert 0 muss größer als 0 sein.",
            "bank.einlagen.hebel_max",
        ),
        (
            "mindestreserve: 0.1",
            "mindestreserve: 1.5",
            "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
            "bank.mindestreserve",
        ),
        (
            "einlagen_aufschlag: -0.01",
            "einlagen_aufschlag: 0.5",
            "Wert 0.5 liegt außerhalb des erlaubten Bereichs -0.2 bis 0.2.",
            "bank.start.einlagen_aufschlag",
        ),
    ] {
        let outcome = bank(alt, neu);
        let f = befund(&outcome, meldung);
        assert_eq!(f.path.to_string(), pfad);
    }
    befund(&bank("  start:", "  unbekannt: 1\n  start:"), "unbekannt");
    // Without the section there are no banks.
    let ohne = Daten::neu().laden();
    assert!(!ohne.data.unwrap().catalog.bank.enabled);
}

const LEBENSLAUF: &str = "\
lebenslauf:
  eintrittsalter:
    standort: {von: 26, bis: 50, mittel: 35, streuung: 6}
    land: {von: 32, bis: 55, mittel: 42, streuung: 6}
    kontinent: {von: 38, bis: 58, mittel: 47, streuung: 5}
    vorstand: {von: 42, bis: 62, mittel: 52, streuung: 5}
  ebene_nach_staerke: {land: 50, kontinent: 57, vorstand: 63}
  erfahrung: {jung_bis: 35, jung: 1.5, alt_ab: 55, alt: 0.5}
  risiko: {ab: 40, je_jahr: 0.3, hoechstens: 10}
  abbau: {ab: 65, chance: 0.1}
  ruhestand: {abweichung: 5, vorwarnung_monate: 12, verlaengerung_jahre_max: 3,
              verlaengerung_aufschlag: 0.2, bezugsalter: 65, zusage_alter: [60, 80]}
  sterbetafel: {ab: 50, chance: 0.005, verdopplung_jahre: 8}
  ruhestandsalter:
    standard: {1900: 70, 1960: 65}
    laender:
      SWE: {1900: 67, 2000: 65}
  lebenserwartung:
    standard: {1900: 60, 2100: 79}
  annaeherung: true
";

#[test]
fn lebenslauf_wird_geprueft() {
    let datei = "parameter/lebenslauf.yaml";
    let lauf = |alt: &str, neu: &str| {
        Daten::neu()
            .datei(datei, &LEBENSLAUF.replacen(alt, neu, 1))
            .laden()
    };
    let gut = lauf("", "");
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let data = gut.data.as_ref().unwrap();
    let m = &data.catalog.life;
    assert!(m.enabled);
    assert_eq!(m.warning_months, 12);
    assert_eq!(m.entry_age[3].mean, 52.0);
    let swe = data.catalog.countries.id("SWE").unwrap();
    assert_eq!(m.retirement_age.value(swe, 1900.0), 67.0);
    assert_eq!(m.life_expectancy.value(swe, 1900.0), 60.0);

    for (alt, neu, meldung, pfad) in [
        (
            "mittel: 35",
            "mittel: 55",
            "Das Mittel 55 liegt nicht zwischen „von“ (26) und „bis“ (50).",
            "lebenslauf.eintrittsalter.standort.mittel",
        ),
        (
            "kontinent: 57",
            "kontinent: 70",
            "Die Grenzen müssen von Land über Kontinent bis Vorstand steigen.",
            "lebenslauf.ebene_nach_staerke",
        ),
        (
            "chance: 0.005",
            "chance: 2",
            "Wert 2 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
            "lebenslauf.sterbetafel.chance",
        ),
        (
            "1960: 65",
            "1960: 95",
            "Wert 95 liegt außerhalb des erlaubten Bereichs 40 bis 90.",
            "lebenslauf.ruhestandsalter.standard.1960",
        ),
        (
            "zusage_alter: [60, 80]",
            "zusage_alter: [80, 60]",
            "„zusage_alter[0]“ muss kleiner als „zusage_alter[1]“ sein.",
            "lebenslauf.ruhestand.zusage_alter",
        ),
    ] {
        let outcome = lauf(alt, neu);
        let f = befund(&outcome, meldung);
        assert_eq!(f.path.to_string(), pfad);
    }
    befund(&lauf("      SWE:", "      XYZ:"), "XYZ");
    befund(&lauf("  abbau:", "  unbekannt: 1\n  abbau:"), "unbekannt");
    // Without the section managers do not age.
    let ohne = Daten::neu().laden();
    assert!(!ohne.data.unwrap().catalog.life.enabled);
}

const PERSON: &str = "\
person:
  alter_start: {standard: 30, von: 18, bis: 60}
  familie: {kinder_ab: 22, kinder_bis: 45, kinder_hoechstens: 4, kinder_chance_jahr: 0.15,
            managerkarte_ab: 25}
  gruendung: {kosten_anteil: 0.005, kosten_mindestens_monatsloehne: 1, einlage_vorschlag: 0.9}
  gehalt: {hoechstens_mitgesellschafter: 2}
  darlehen: {zins_hoechstens: 0.15, jahre_hoechstens: 30}
  lebensstil:
    standard: buergerlich
    wechsel_monate: 12
    stufen:
      bescheiden: {kosten: 0.8, zins: 0.005, gehaltsforderung: 0.05, ausbildung: 0,
                   sterblichkeit: 1.1}
      buergerlich: {kosten: 1.5, zins: 0, gehaltsforderung: 0, ausbildung: 5, sterblichkeit: 1}
      gehoben: {kosten: 4, zins: -0.0025, gehaltsforderung: -0.05, ausbildung: 10,
                sterblichkeit: 0.95}
      luxurioes: {kosten: 10, zins: -0.005, gehaltsforderung: -0.1, ausbildung: 15,
                  sterblichkeit: 0.95}
  einkommensteuer:
    standard: {1900: 0.04, 2026: 0.3}
    laender:
      SWE: {1900: 0.03, 2026: 0.4}
  einkommensteuer_stufen:
    - {ab: 0, faktor: 0.2}
    - {ab: 1, faktor: 1}
  kapitalertragsteuer:
    standard: {1900: 0, 2026: 0.25}
  sparzins:
    standard: {1900: 0.02}
  anlagen:
    festgeld:
      aufschlag: 0.005
      laufzeit_monate: {von: 3, bis: 120}
      kuendigung_abschlag: 0.5
    staatsanleihen:
      aufschlag: 0.005
      laufzeit_monate: {von: 12, bis: 360}
      dauer_faktor: 0.8
      spanne: 0.01
    gold:
      preis_usd: {1900: 800, 2026: 3000}
      krisen_beta: 0.8
      rauschen: 0.03
      rueckkehr: 0.05
      spanne: 0.01
      gebuehr: 0.005
    fonds:
      gemanagt: {ab: 1900, kosten: 0.015, aufschlag: 0.05, gebuehr: 0.005}
      index: {ab: 1990, kosten: 0.002, aufschlag: 0, gebuehr: 0.005}
    immobilien:
      arten:
        wohnung: {preis_jahreslohn: 5, rendite: 0.045, leerstand: 0.04, instandhaltung: 0.012, verfall: 2}
        mehrfamilienhaus: {preis_jahreslohn: 25, rendite: 0.05, leerstand: 0.05, instandhaltung: 0.014, verfall: 2}
        buero: {preis_jahreslohn: 30, rendite: 0.055, leerstand: 0.08, instandhaltung: 0.015, verfall: 2.5}
        halle: {preis_jahreslohn: 20, rendite: 0.06, leerstand: 0.06, instandhaltung: 0.012, verfall: 2}
        grundstueck: {preis_jahreslohn: 8, rendite: 0.01, leerstand: 0.0, instandhaltung: 0.002, verfall: 0}
      nebenkosten: 0.09
      eigenkapital_min: 0.2
      hypothek_aufschlag: 0.015
      tilgung_jahr: 0.03
      mietband: {von: 0.8, bis: 1.2}
      leerstand_steigung: 1.5
      schaden: {chance: 0.01, kosten: 0.02, punkte: 8}
      sanierung: {kosten: 0.12, punkte: 40, monate: 3}
      werk_effekt: {je_standort: 0.02, hoechstens: 0.08}
      verkauf_abschlag: 0.03
      notverkauf_abschlag: 0.15
      zustandsgewicht: 0.3
      angebot:
        monate: 3
        spanne: {von: -0.1, bis: 0.15}
        guete: {von: 0.7, bis: 1.4}
        zustand: {von: 30, bis: 95}
        laender_max: 5
  wohlbefinden:
    hintergruende:
      tueftler: {forschung: 0.05}
      kaufmann: {kreditzins: -0.005, ordergebuehr: -0.2}
      charismatiker: {loyalitaet: 0.1}
      erbe: {ansehen: 20, netzwerk: 10}
    deckel: 0.1
    erfahrung_je_jahr: 0.1
    zeit:
      standard: {firma: 60, familie: 15, netzwerk: 10, erholung: 15}
      mindest_firma: {basis: 20, je_firma: 10, hoechstens: 80}
      verzoegerung_tage: 30
    gesundheit:
      start: 85
      altern_ab: 40
      altern_je_jahr: 0.01
      ueberlastung_ab: 70
      ueberlastung_verlust: 2
      erholung_gewinn: 2
      lebensstil_gewinn: [-0.2, 0, 0.2, 0.3]
      vorsorge_gewinn: 0.2
      gesund_ab: 80
      sterblichkeit_null: 3
      krank_unter: 35
      krank_chance: 0.3
      krank_monate: 3
  erbe: {neffe_alter: {von: 25, bis: 40}, hinweis_ab: 70}
  erbschaftsteuer:
    standard: {1900: 0.03, 2026: 0.15}
    laender:
      SWE: {1900: 0.03, 2003: 0.3, 2004: 0}
  annaeherung: true
";

#[test]
fn erbe_und_erbschaftsteuer_werden_geprueft() {
    let datei = "parameter/person.yaml";
    let lauf = |alt: &str, neu: &str| {
        Daten::neu()
            .datei(datei, &PERSON.replacen(alt, neu, 1))
            .laden()
    };
    let gut = lauf("", "");
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let data = gut.data.as_ref().unwrap();
    let m = &data.catalog.person;
    assert_eq!(m.nephew_age, [25, 40]);
    assert_eq!(m.succession_hint_from, 70);
    let swe = data.catalog.countries.id("SWE").unwrap();
    assert!(m.inheritance_tax.value(swe, 2010.0).abs() < 1e-12);

    for (alt, neu, meldung, pfad) in [
        (
            "neffe_alter: {von: 25, bis: 40}",
            "neffe_alter: {von: 45, bis: 40}",
            "„von“ muss kleiner als „bis“ sein.",
            "person.erbe.neffe_alter",
        ),
        (
            "hinweis_ab: 70",
            "hinweis_ab: 10",
            "Wert 10 liegt außerhalb des erlaubten Bereichs 18 bis 120.",
            "person.erbe.hinweis_ab",
        ),
        (
            "SWE: {1900: 0.03, 2003",
            "SWE: {1900: 1.5, 2003",
            "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
            "person.erbschaftsteuer.laender.SWE.1900",
        ),
    ] {
        let outcome = lauf(alt, neu);
        let f = befund(&outcome, meldung);
        assert_eq!(f.path.to_string(), pfad);
    }
    befund(&lauf("  erbe: {", "  erbe: {unbekannt: 1, "), "unbekannt");
}

#[test]
fn person_wird_geprueft() {
    let datei = "parameter/person.yaml";
    let lauf = |alt: &str, neu: &str| {
        Daten::neu()
            .datei(datei, &PERSON.replacen(alt, neu, 1))
            .laden()
    };
    let gut = lauf("", "");
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let m = &gut.data.as_ref().unwrap().catalog.person;
    assert!(m.enabled);
    assert_eq!(m.start_age, [30, 18, 60]);
    assert_eq!(m.children_max, 4);

    for (alt, neu, meldung, pfad) in [
        (
            "standard: 30",
            "standard: 70",
            "Das Standardalter muss zwischen „von“ und „bis“ liegen.",
            "person.alter_start",
        ),
        (
            "kinder_bis: 45",
            "kinder_bis: 20",
            "„kinder_ab“ muss kleiner als „kinder_bis“ sein.",
            "person.familie",
        ),
        (
            "kinder_chance_jahr: 0.15",
            "kinder_chance_jahr: 1.5",
            "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
            "person.familie.kinder_chance_jahr",
        ),
    ] {
        let outcome = lauf(alt, neu);
        let f = befund(&outcome, meldung);
        assert_eq!(f.path.to_string(), pfad);
    }
    befund(
        &lauf("  familie:", "  unbekannt: 1\n  familie:"),
        "unbekannt",
    );
    // Without the section the person has no children.
    let ohne = Daten::neu().laden();
    assert!(!ohne.data.unwrap().catalog.person.enabled);
}

#[test]
fn privatleben_wird_geprueft() {
    let datei = "parameter/person.yaml";
    let mit = |text: &str| {
        let person = format!(
            "{}  privatleben:\n    partnersuche: {{kosten_monat: 0.3, chance_monat: 0.12}}\n    \
             privatschule: {{kosten_monat: 0.8, ausbildung: 12}}\n    vorsorge: {{kosten_monat: 0.5, \
             sterblichkeit: {text}}}\n",
            PERSON
        );
        Daten::neu().datei(datei, &person).laden()
    };
    let gut = mit("0.8");
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let pl = gut.data.as_ref().unwrap().catalog.person.private_life;
    assert!(pl.enabled && (pl.care_mortality - 0.8).abs() < 1e-12);
    let schlecht = mit("2");
    let f = befund(
        &schlecht,
        "Wert 2 liegt außerhalb des erlaubten Bereichs 0.1 bis 1.",
    );
    assert_eq!(
        f.path.to_string(),
        "person.privatleben.vorsorge.sterblichkeit"
    );
}

#[test]
fn privatkonto_und_lebensstil_werden_geprueft() {
    let datei = "parameter/person.yaml";
    let lauf = |alt: &str, neu: &str| {
        Daten::neu()
            .datei(datei, &PERSON.replacen(alt, neu, 1))
            .laden()
    };
    let gut = lauf("", "");
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let m = &gut.data.as_ref().unwrap().catalog.person;
    assert_eq!(m.default_lifestyle, wsim_core::state::Lifestyle::Middle);
    assert!((m.lifestyles[3].cost - 10.0).abs() < 1e-12);
    assert!((m.founding.cost_share - 0.005).abs() < 1e-12);
    assert_eq!(m.loan_max_years, 30);
    let swe = gut
        .data
        .as_ref()
        .unwrap()
        .catalog
        .countries
        .id("SWE")
        .unwrap();
    assert!((m.income_tax.value(swe, 2026.0) - 0.4).abs() < 1e-12);

    for (alt, neu, meldung, pfad) in [
        (
            "gehoben: {kosten: 4",
            "gehoben: {kosten: 1",
            "Die Kosten der Lebensstile müssen von „bescheiden“ über „buergerlich“ und \
             „gehoben“ bis „luxurioes“ steigen.",
            "person.lebensstil.stufen",
        ),
        (
            "standard: buergerlich",
            "standard: buergerlch",
            "Lebensstil „buergerlch“ ist nicht definiert. Meinten Sie „buergerlich“?",
            "person.lebensstil.standard",
        ),
        (
            "SWE: {1900: 0.03",
            "SWE: {1900: 1.03",
            "Wert 1.03 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
            "person.einkommensteuer.laender.SWE.1900",
        ),
        (
            "- {ab: 1, faktor: 1}",
            "- {ab: 0, faktor: 1}",
            "Die Stufen müssen nach „ab“ aufsteigend sortiert sein.",
            "person.einkommensteuer_stufen[1].ab",
        ),
        (
            "standard: {1900: 0, 2026: 0.25}",
            "standard: {1900: 0, 2026: 2.5}",
            "Wert 2.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
            "person.kapitalertragsteuer.standard.2026",
        ),
        (
            "standard: {firma: 60, familie: 15, netzwerk: 10, erholung: 15}",
            "standard: {firma: 60, familie: 15, netzwerk: 10, erholung: 20}",
            "Die Anteile der Zeit (firma, familie, netzwerk, erholung) müssen zusammen 100 ergeben.",
            "person.wohlbefinden.zeit.standard",
        ),
        (
            "      erbe: {ansehen: 20, netzwerk: 10}",
            "      erbe: {ansehen: 20, netzwerk: 10}\n      zauberer: {}",
            "Hintergrund „zauberer“ ist nicht bekannt (erlaubt: tueftler, kaufmann, charismatiker, erbe).",
            "person.wohlbefinden.hintergruende.zauberer",
        ),
        (
            "      kaufmann: {kreditzins: -0.005, ordergebuehr: -0.2}\n",
            "",
            "Der Hintergrund „kaufmann“ fehlt.",
            "person.wohlbefinden.hintergruende",
        ),
        (
            "laufzeit_monate: {von: 12, bis: 360}",
            "laufzeit_monate: {von: 400, bis: 360}",
            "Die Laufzeit braucht „von“ ≥ 1, „von“ ≤ „bis“ und höchstens 600 Monate.",
            "person.anlagen.staatsanleihen.laufzeit_monate",
        ),
        (
            "preis_usd: {1900: 800, 2026: 3000}",
            "preis_usd: {1900: 0, 2026: 3000}",
            "Wert 0 muss größer als 0 sein.",
            "person.anlagen.gold.preis_usd.1900",
        ),
        (
            "gemanagt: {ab: 1900, kosten: 0.015",
            "gemanagt: {ab: 1900, kosten: 0.5",
            "Wert 0.5 liegt außerhalb des erlaubten Bereichs 0 bis 0.1.",
            "person.anlagen.fonds.gemanagt.kosten",
        ),
        (
            "mietband: {von: 0.8, bis: 1.2}",
            "mietband: {von: 1.3, bis: 1.2}",
            "„von“ muss kleiner als „bis“ sein.",
            "person.anlagen.immobilien.mietband",
        ),
        (
            "eigenkapital_min: 0.2",
            "eigenkapital_min: 1.5",
            "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
            "person.anlagen.immobilien.eigenkapital_min",
        ),
        (
            "wohnung: {preis_jahreslohn: 5,",
            "wohnung: {preis_jahreslohn: 0,",
            "Wert 0 liegt außerhalb des erlaubten Bereichs 0.1 bis 500.",
            "person.anlagen.immobilien.arten.wohnung.preis_jahreslohn",
        ),
        (
            "krank_chance: 0.3",
            "krank_chance: 3",
            "Wert 3 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
            "person.wohlbefinden.gesundheit.krank_chance",
        ),
        (
            "kosten_anteil: 0.005",
            "kosten_anteil: -0.005",
            "Wert -0.005 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
            "person.gruendung.kosten_anteil",
        ),
    ] {
        let outcome = lauf(alt, neu);
        let f = befund(&outcome, meldung);
        assert_eq!(f.path.to_string(), pfad);
    }
    befund(&lauf("  sparzins:", "  sparzinz:"), "sparzinz");
}

const UMWELT: &str = "\
umwelt:
  nachruestung:
    - {ab: 1910, minderung: 0.5, kosten_anteil: 0.03}
    - {ab: 1950, minderung: 0.6, kosten_anteil: 0.05}
  co2_preis:
    standard: {1900: 0}
    laender:
      SWE: {2005: 20, 2026: 80}
  kartell: {marktanteil_max: 0.4}
  markenbild: {gewicht: 0.3}
  annaeherung: true
";

const REGELN: &str = "\
regulierungen:
  - id: auflage_swe
    datum: \"1969-05-29\"
    laender: [SWE]
    art: auflage
    stufe: 2
    frist_monate: 60
  - id: schutz_swe
    datum: \"1970-01-01\"
    laender: [SWE]
    art: arbeitsschutz
    lohnaufschlag: 0.02
  - id: verbot_swe
    datum: \"2012-09-01\"
    laender: [SWE]
    art: verbot
    produkte: [eisen]
    verkauf: true
  - id: kartell_swe
    datum: \"1990-01-01\"
    laender: [SWE]
    art: kartellaufsicht
";

const REGEL_TEXTE: &str = "\
regulierung.auflage_swe: Auflage
regulierung.schutz_swe: Schutz
regulierung.verbot_swe: Verbot
regulierung.kartell_swe: Kartell
";

#[test]
fn umwelt_und_regulierungen_werden_geprueft() {
    let lauf = |datei: &str, alt: &str, neu: &str| {
        let mut d = Daten::neu()
            .datei("parameter/umwelt.yaml", UMWELT)
            .datei("regulierungen.yaml", REGELN)
            .datei("texte/de/regulierungen.yaml", REGEL_TEXTE);
        if !alt.is_empty() {
            d = d.ersetze(datei, alt, neu);
        }
        d.laden()
    };
    let gut = lauf("", "", "");
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let data = gut.data.as_ref().unwrap();
    assert_eq!(data.catalog.environment.retrofit.len(), 2);
    assert_eq!(data.catalog.regulations.len(), 4);
    let swe = data.catalog.countries.id("SWE").unwrap();
    assert!((data.catalog.environment.co2_price.value(swe, 2026.0) - 80.0).abs() < 1e-9);

    let u = "parameter/umwelt.yaml";
    let r = "regulierungen.yaml";
    for (datei, alt, neu, meldung, pfad) in [
        (
            u,
            "minderung: 0.5",
            "minderung: 1.5",
            "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
            "umwelt.nachruestung[0].minderung",
        ),
        (
            u,
            "ab: 1950",
            "ab: 1905",
            "Die Nachrüststufen müssen in der Reihenfolge ihrer Jahre („ab“) stehen.",
            "umwelt.nachruestung",
        ),
        (
            r,
            "art: kartellaufsicht",
            "art: kartel",
            "Unbekannte Art der Regulierung „kartel“; erlaubt sind: auflage, arbeitsschutz, verbot, kartellaufsicht.",
            "regulierungen[3].art",
        ),
        (
            r,
            "stufe: 2",
            "stufe: 3",
            "Nachrüststufe 3 gibt es nicht; „umwelt.nachruestung“ hat 2 Stufen (1 bis 2).",
            "regulierungen[0].stufe",
        ),
        (
            r,
            "    verkauf: true\n",
            "",
            "Ein Verbot muss „herstellung: true“, „verkauf: true“ oder beides setzen.",
            "regulierungen[2].art",
        ),
        (
            r,
            "    lohnaufschlag: 0.02\n",
            "    lohnaufschlag: 0.02\n    stufe: 1\n",
            "Das Feld „stufe“ passt nicht zu einer Regulierung der Art „arbeitsschutz“.",
            "regulierungen[1].stufe",
        ),
        (
            r,
            "    frist_monate: 60\n",
            "",
            "Eine Regulierung der Art „auflage“ braucht das Feld „frist_monate“.",
            "regulierungen[0].art",
        ),
    ] {
        let outcome = lauf(datei, alt, neu);
        let f = befund(&outcome, meldung);
        assert_eq!(f.path.to_string(), pfad);
    }
    befund(&lauf(r, "produkte: [eisen]", "produkte: [eisn]"), "eisn");
    befund(
        &lauf(
            "texte/de/regulierungen.yaml",
            "regulierung.kartell_swe: Kartell\n",
            "",
        ),
        "regulierung.kartell_swe",
    );
    // Without the sections nothing is regulated.
    let ohne = Daten::neu().laden();
    let c = &ohne.data.unwrap().catalog;
    assert!(c.regulations.is_empty() && c.environment.retrofit.is_empty());
}

#[test]
fn rezepte_mit_emissionen_werden_geprueft() {
    let datei = "ketten/a.yaml";
    let gut = Daten::neu()
        .ersetze(
            datei,
            "    qualitaet_basis: 60",
            "    qualitaet_basis: 60\n    co2_t: 2.0\n    schadstoff_kg: 20",
        )
        .laden();
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let falsch = Daten::neu()
        .ersetze(
            datei,
            "    qualitaet_basis: 60",
            "    qualitaet_basis: 60\n    co2_t: -1",
        )
        .laden();
    befund(
        &falsch,
        "Wert -1 liegt außerhalb des erlaubten Bereichs 0 bis 1000.",
    );
}

#[test]
fn patente_werden_geprueft() {
    let datei = "parameter/forschungsmodell.yaml";
    // Without the block there are no patents.
    let data = Daten::neu().laden().data.unwrap();
    assert!(data.catalog.research_model.patents.is_none());
    let mit = |neu: &str| {
        Daten::neu().ersetze(
            datei,
            "  sachkosten_usd_je_forschertag: 40\n",
            &format!("  sachkosten_usd_je_forschertag: 40\n{neu}"),
        )
    };
    let block = "  patente:\n    laufzeit_jahre: 20\n    anmeldefrist_tage: 365\n    \
                 kosten_je_land_usd: 30_000\n    ki_groesste_maerkte: 5\n";
    let gut = mit(block).laden();
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let p = gut
        .data
        .unwrap()
        .catalog
        .research_model
        .patents
        .clone()
        .unwrap();
    assert_eq!(
        (p.term_years, p.filing_days, p.ai_largest_markets),
        (20, 365, 5)
    );
    assert!((p.cost_per_country_usd - 30_000.0).abs() < 1e-9);

    let falsch = mit(&block.replace("laufzeit_jahre: 20", "laufzeit_jahre: 80")).laden();
    let f = befund(
        &falsch,
        "Wert 80 liegt außerhalb des erlaubten Bereichs 1 bis 50.",
    );
    assert_ort(f, datei, 8, "forschungsmodell.patente.laufzeit_jahre");
    let falsch =
        mit(&block.replace("kosten_je_land_usd: 30_000", "kosten_je_land_usd: -1")).laden();
    befund(&falsch, "-1");
    let falsch = mit(&format!("{block}    gebuehr: 3\n")).laden();
    befund(&falsch, "gebuehr");
}

#[test]
fn ruestung_wird_geprueft() {
    const RUESTUNG: &str = "\
ruestung:
  frieden: 0.06
  krieg: 0.30
  bezug: 0.03
  militaerausgaben:
    standard: {1900: 0.02, 2026: 0.02}
    laender:
      SWE: {1900: 0.02, 1915: 0.4, 1919: 0.03}
";
    let ruestungsgut = "    richtpreis_usd: 400\n    staatsnachfrage:\n      je_mio_usd_bip: 0.5\n      ruestung: true\n";
    let d = Daten::neu()
        .datei("parameter/ruestung.yaml", RUESTUNG)
        .ersetze("ketten/a.yaml", "    richtpreis_usd: 400\n", ruestungsgut);
    let gut = d.laden();
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let data = gut.data.unwrap();
    let m = &data.catalog.armament;
    assert!(m.enabled);
    let aaa = data.catalog.countries.id("SWE").unwrap();
    let (share, war) = m.at(aaa, 1915.0);
    assert!((share - 0.4).abs() < 1e-9 && war == 1.0, "{share} {war}");

    // An armament good needs the section and takes no war factor.
    let ohne = Daten::neu()
        .ersetze("ketten/a.yaml", "    richtpreis_usd: 400\n", ruestungsgut)
        .laden();
    let f = befund(&ohne, "Rüstungsgüter brauchen den Abschnitt „ruestung“");
    assert_eq!(f.path.to_string(), "produkte[1].staatsnachfrage.ruestung");
    let mit_faktor = Daten::neu()
        .datei("parameter/ruestung.yaml", RUESTUNG)
        .ersetze(
            "ketten/a.yaml",
            "    richtpreis_usd: 400\n",
            &format!("{ruestungsgut}      kriegsfaktor: 3\n"),
        )
        .laden();
    befund(&mit_faktor, "Bei Rüstungsgütern wirkt kein Kriegsfaktor");
    // War above peace.
    let falsch = Daten::neu()
        .datei(
            "parameter/ruestung.yaml",
            &RUESTUNG.replace("krieg: 0.30", "krieg: 0.05"),
        )
        .laden();
    let f = befund(&falsch, "„krieg“ (0.05) muss über „frieden“ (0.06) liegen");
    assert_ort(f, "parameter/ruestung.yaml", 3, "ruestung.krieg");
    let falsch = Daten::neu()
        .datei(
            "parameter/ruestung.yaml",
            &RUESTUNG.replace("bezug: 0.03", "bezug: 0"),
        )
        .laden();
    befund(&falsch, "außerhalb des erlaubten Bereichs");
}

#[test]
fn rohstoffzyklus_wird_geprueft() {
    // Swings of the raw materials' output (Rohstoffzyklus).
    let datei = "parameter/produktionsmodell.yaml";
    let d = Daten::neu().ersetze(datei, "beharrung: 0.9", "beharrung: 1.2");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 1.2 liegt außerhalb des erlaubten Bereichs 0 bis 0.99.",
    );
    assert_ort(
        f,
        datei,
        d.zeile(datei, "beharrung"),
        "produktionsmodell.rohstoffzyklus.beharrung",
    );
}

#[test]
fn lager_wird_geprueft() {
    let datei = "parameter/produktionsmodell.yaml";
    let mit = |verderb: &str, tage: &str| {
        Daten::neu()
            .ersetze(
                datei,
                "  schliessung: {lager_erloes_anteil: 0.5, abfindung_monate: 3}\n",
                &format!(
                    "  schliessung: {{lager_erloes_anteil: 0.5, abfindung_monate: 3}}\n  lager:\n    \
                     mindestens_m3: {{foerderstaette: 1, werk: 1, kraftwerk: 1, lager: 1, \
                     niederlassung: 1, forschungszentrum: 1}}\n    tage: {tage}\n    ausbau: \
                     {{kapazitaet_m3: 1000, kosten_usd: 100}}\n    kosten_jahr_anteil: 0.04\n    \
                     verderb: {{{verderb}}}\n    ki_ausbau_ab: 0.9\n"
                ),
            )
            .laden()
    };
    let gut = mit("erze: 0.03", "60");
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let m = &gut.data.as_ref().unwrap().catalog.production_model.storage;
    assert!(m.enabled && m.spoilage.len() == 1 && (m.days - 60.0).abs() < 1e-12);
    let zu_viel = mit("erze: 1.5", "60");
    let f = befund(
        &zu_viel,
        "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_eq!(f.path.to_string(), "produktionsmodell.lager.verderb.erze");
    let schlecht = mit("kaese: 0.1", "60");
    assert!(
        schlecht
            .report
            .findings()
            .iter()
            .any(|f| f.message.contains("kaese")),
        "{}",
        alle(&schlecht)
    );
    let schlecht = mit("erze: 0.1", "0");
    let f = schlecht
        .report
        .findings()
        .iter()
        .find(|f| f.path.to_string() == "produktionsmodell.lager.tage");
    assert!(f.is_some(), "{}", alle(&schlecht));
}

#[test]
fn fruehere_regionscodes_werden_geprueft() {
    let mit = |frueher: &str| {
        Daten::neu()
            .ersetze(
                "laender/SWE.yaml",
                "    binnenland: false",
                &format!("    binnenland: false\n    umfasst: [SWE, ALA]\n    frueher: {frueher}"),
            )
            .ersetze(
                "texte/de/a.yaml",
                "land.SWE: Schweden\n",
                "land.SWE: Schweden\nteilland.SWE: Schweden\nteilland.ALA: Åland\n",
            )
            .laden()
    };
    let gut = mit("[XNO]");
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let c = &gut.data.as_ref().unwrap().catalog;
    assert_eq!(
        c.countries.get(c.countries.id("SWE").unwrap()).former,
        ["XNO"]
    );
    // The former code names a country of its own, or a member of the region.
    let schlecht = mit("[SWE]");
    assert!(
        schlecht
            .report
            .findings()
            .iter()
            .any(|f| f.path.to_string() == "laender[0].frueher[0]"),
        "{}",
        alle(&schlecht)
    );
    let schlecht = mit("[ALA]");
    assert!(
        schlecht
            .report
            .findings()
            .iter()
            .any(|f| f.path.to_string() == "laender[0].frueher[0]"),
        "{}",
        alle(&schlecht)
    );
    let schlecht = mit("[x1]");
    assert!(
        !schlecht.report.findings().is_empty(),
        "{}",
        alle(&schlecht)
    );
}

#[test]
fn lernkurve_wird_geprueft() {
    let datei = "parameter/produktionsmodell.yaml";
    let mit = |rate: &str| {
        Daten::neu()
            .ersetze(
                datei,
                "  schliessung: {lager_erloes_anteil: 0.5, abfindung_monate: 3}\n",
                &format!(
                    "  schliessung: {{lager_erloes_anteil: 0.5, abfindung_monate: 3}}\n  \
                     lernkurve:\n    rate: {{{rate}}}\n    rate_sonst: 0.95\n    \
                     untergrenze: 0.5\n    vergessen_je_monat: 0.03\n    entlassung: 0.1\n    \
                     uebertrag: 0.5\n    start_etabliert_jahre: 10\n"
                ),
            )
            .laden()
    };
    let gut = mit("metallurgie: 0.92");
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let m = &gut.data.as_ref().unwrap().catalog.production_model.learning;
    assert!(m.enabled && m.rates.len() == 1 && (m.rates[0].1 - 0.92).abs() < 1e-12);
    let zu_hoch = mit("metallurgie: 1.2");
    let f = befund(
        &zu_hoch,
        "Wert 1.2 liegt außerhalb des erlaubten Bereichs 0.5 bis 1.",
    );
    assert_eq!(
        f.path.to_string(),
        "produktionsmodell.lernkurve.rate.metallurgie"
    );
    let schlecht = mit("schmiede: 0.9");
    assert!(
        schlecht
            .report
            .findings()
            .iter()
            .any(|f| f.message.contains("schmiede")),
        "{}",
        alle(&schlecht)
    );
}

#[test]
fn instandhaltung_wird_geprueft() {
    let datei = "parameter/produktionsmodell.yaml";
    let mit = |stufe_min: &str, tage: &str| {
        Daten::neu()
            .ersetze(
                datei,
                "  schliessung: {lager_erloes_anteil: 0.5, abfindung_monate: 3}\n",
                &format!(
                    "  schliessung: {{lager_erloes_anteil: 0.5, abfindung_monate: 3}}\n  \
                     instandhaltung:\n    stufe_min: {stufe_min}\n    stufe_max: 1.5\n    \
                     exponent: 1.5\n    kapazitaet_zustand: 0.3\n    ausfall: {{basis: 0.004, \
                     {tage}, kosten_anteil: 0.02, warnung_ab: 3}}\n    ueberholung: \
                     {{kosten_anteil: 0.3, tage: 30, ki_ab: 0.35}}\n"
                ),
            )
            .laden()
    };
    let gut = mit("0.5", "tage_min: 3, tage_max: 14");
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let m = &gut
        .data
        .as_ref()
        .unwrap()
        .catalog
        .production_model
        .maintenance;
    assert!(m.enabled && m.failure_days == (3, 14));
    let zu_hoch = mit("1.2", "tage_min: 3, tage_max: 14");
    let f = befund(
        &zu_hoch,
        "Wert 1.2 liegt außerhalb des erlaubten Bereichs 0.1 bis 1.",
    );
    assert_eq!(
        f.path.to_string(),
        "produktionsmodell.instandhaltung.stufe_min"
    );
    let verdreht = mit("0.5", "tage_min: 20, tage_max: 14");
    let f = befund(
        &verdreht,
        "„tage_min“ muss mindestens 1 sein und darf nicht über „tage_max“ liegen.",
    );
    assert_eq!(
        f.path.to_string(),
        "produktionsmodell.instandhaltung.ausfall.tage_min"
    );
}

#[test]
fn lagerklassen_werden_geprueft() {
    let datei = "parameter/produktionsmodell.yaml";
    let mit = |klassen: &str, transport: &str| {
        Daten::neu()
            .ersetze(
                datei,
                "  schliessung: {lager_erloes_anteil: 0.5, abfindung_monate: 3}\n",
                &format!(
                    "  schliessung: {{lager_erloes_anteil: 0.5, abfindung_monate: 3}}\n  lager:\n    \
                     mindestens_m3: {{foerderstaette: 1, werk: 1, kraftwerk: 1, lager: 1, \
                     niederlassung: 1, forschungszentrum: 1}}\n    tage: 60\n    ausbau: \
                     {{kapazitaet_m3: 1000, kosten_usd: 100}}\n    kosten_jahr_anteil: 0.04\n    \
                     ki_ausbau_ab: 0.9\n    klassen: {{{klassen}}}\n    nach_transportklasse: \
                     {{{transport}}}\n    fremdlager: {{kapazitaet_faktor: 2, kosten_faktor: 2}}\n    \
                     dichte_t_je_m3: {{schuettgut: 1.2, stueckgut: 0.25, fluessig: 0.8, gekuehlt: \
                     0.35, gefahrgut: 0.6}}\n"
                ),
            )
            .laden()
    };
    let alle_klassen =
        "schuettgut: 0.02, stueckgut: 0.04, fluessig: 0.03, gekuehlt: 0.12, gefahrgut: 0.06";
    let gut = mit(alle_klassen, "schuettgut: schuettgut");
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let m = &gut.data.as_ref().unwrap().catalog.production_model.storage;
    assert_eq!(m.classes.len(), 5);
    assert!(m.product_class.iter().all(|&c| c == StorageClass::Bulk));
    let falsch = mit(alle_klassen, "schuettgut: kuehlhaus");
    assert!(
        falsch
            .report
            .findings()
            .iter()
            .any(|f| f.message.contains("kuehlhaus")),
        "{}",
        alle(&falsch)
    );
    let ohne = mit(alle_klassen, "");
    assert!(
        ohne.report
            .findings()
            .iter()
            .any(|f| f.message.contains("Lagerklasse")),
        "jedes Produkt braucht eine Klasse: {}",
        alle(&ohne)
    );
    // Storage volume from the weight per unit and the class density (1.2 t per m³).
    let c = &gut.data.as_ref().unwrap().catalog;
    let p = c.products.ids().next().unwrap();
    let weight = c.products.get(p).weight_kg;
    assert!((m.volume(p) - weight / 1000.0 / 1.2).abs() < 1e-9);
    let falsche_dichte = Daten::neu()
        .ersetze(
            datei,
            "  schliessung: {lager_erloes_anteil: 0.5, abfindung_monate: 3}\n",
            &format!(
                "  schliessung: {{lager_erloes_anteil: 0.5, abfindung_monate: 3}}\n  lager:\n    \
                 mindestens_m3: {{foerderstaette: 1, werk: 1, kraftwerk: 1, lager: 1, \
                 niederlassung: 1, forschungszentrum: 1}}\n    tage: 60\n    ausbau: \
                 {{kapazitaet_m3: 1000, kosten_usd: 100}}\n    kosten_jahr_anteil: 0.04\n    \
                 ki_ausbau_ab: 0.9\n    klassen: {{{alle_klassen}}}\n    nach_transportklasse: \
                 {{schuettgut: schuettgut}}\n    dichte_t_je_m3: {{schuettgut: 0, stueckgut: \
                 0.25, fluessig: 0.8, gekuehlt: 0.35, gefahrgut: 0.6}}\n"
            ),
        )
        .laden();
    let f = befund(&falsche_dichte, "Wert 0 muss größer als 0 sein.");
    assert_eq!(
        f.path.to_string(),
        "produktionsmodell.lager.dichte_t_je_m3.schuettgut"
    );
    let luecke = mit("schuettgut: 0.02", "schuettgut: schuettgut");
    assert!(
        luecke
            .report
            .findings()
            .iter()
            .any(|f| f.message.contains("gekuehlt")),
        "{}",
        alle(&luecke)
    );
}

#[test]
fn betrieb_wird_geprueft() {
    let datei = "parameter/produktionsmodell.yaml";
    let mit = |ausschuss: &str| {
        Daten::neu()
            .ersetze(
                datei,
                "  schliessung: {lager_erloes_anteil: 0.5, abfindung_monate: 3}\n",
                &format!(
                    "  schliessung: {{lager_erloes_anteil: 0.5, abfindung_monate: 3}}\n  \
                     betrieb:\n    ausschuss: {{{ausschuss}}}\n    unfaelle_je_100000_stunden: \
                     {{1900: 8, 2026: 0.5}}\n    unfall_kosten_stunden: 100\n    unfall_stillstand_anteil: 0.05\n    korruption: \
                     {{grund: 0.6, entwicklung: 0.6, instabilitaet: 0.4, bauzeit: 0.5, gebuehr: \
                     0.05, forderung_chance: 0.3, forderung_anteil: 0.005}}\n    \
                     rechtssicherheit: {{grund: 0.1, entwicklung: 0.5, stabilitaet: 0.4}}\n    \
                     enteignung_basis: 0.02\n    make_or_buy_toleranz: 0.1\n"
                ),
            )
            .laden()
    };
    let gut = mit(
        "basis: {rohstoff: 0, halbzeug: 0.02, komponente: 0.04, endprodukt: 0.04, energie: 0}, min: 0.005, max: 0.3, schulung: 0.6, weiterentwicklung: 0.8",
    );
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let m = &gut
        .data
        .as_ref()
        .unwrap()
        .catalog
        .production_model
        .operations;
    assert!(m.enabled && (m.scrap_base[3] - 0.04).abs() < 1e-12);
    let verdreht = mit(
        "basis: {rohstoff: 0, halbzeug: 0.02, komponente: 0.04, endprodukt: 0.04, energie: 0}, min: 0.3, max: 0.1, schulung: 0.6, weiterentwicklung: 0.8",
    );
    let f = befund(
        &verdreht,
        "„min“ muss mindestens 1 sein und darf nicht über „max“ liegen.",
    );
    assert_eq!(
        f.path.to_string(),
        "produktionsmodell.betrieb.ausschuss.min"
    );
    let zu_hoch = mit(
        "basis: {rohstoff: 0, halbzeug: 0.02, komponente: 0.04, endprodukt: 1.5, energie: 0}, min: 0.005, max: 0.3, schulung: 0.6, weiterentwicklung: 0.8",
    );
    let f = befund(
        &zu_hoch,
        "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_eq!(
        f.path.to_string(),
        "produktionsmodell.betrieb.ausschuss.basis.endprodukt"
    );
}

#[test]
fn lagervolumen_eines_produkts_wird_geprueft() {
    let lager = "  schliessung: {lager_erloes_anteil: 0.5, abfindung_monate: 3}\n  lager:\n    \
                 mindestens_m3: {foerderstaette: 1, werk: 1, kraftwerk: 1, lager: 1, \
                 niederlassung: 1, forschungszentrum: 1}\n    tage: 60\n    ausbau: \
                 {kapazitaet_m3: 1000, kosten_usd: 100}\n    kosten_jahr_anteil: 0.04\n    \
                 ki_ausbau_ab: 0.9\n    klassen: {schuettgut: 0.02, stueckgut: 0.04, fluessig: \
                 0.03, gekuehlt: 0.12, gefahrgut: 0.06}\n    nach_transportklasse: {schuettgut: \
                 schuettgut}\n    dichte_t_je_m3: {schuettgut: 1.2, stueckgut: 0.25, fluessig: \
                 0.8, gekuehlt: 0.35, gefahrgut: 0.6}\n";
    let mit = |volumen: &str| {
        Daten::neu()
            .ersetze(
                "parameter/produktionsmodell.yaml",
                "  schliessung: {lager_erloes_anteil: 0.5, abfindung_monate: 3}\n",
                lager,
            )
            .ersetze(
                "ketten/a.yaml",
                "    richtpreis_usd: 60\n",
                &format!("    richtpreis_usd: 60\n    lagervolumen_m3: {volumen}\n"),
            )
            .laden()
    };
    let gut = mit("2");
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let c = &gut.data.as_ref().unwrap().catalog;
    let erz = c.products.id("erz").unwrap();
    assert!((c.production_model.storage.volume(erz) - 2.0).abs() < 1e-12);
    let schlecht = mit("0");
    let f = befund(&schlecht, "Wert 0 muss größer als 0 sein.");
    assert!(
        f.path.to_string().ends_with("lagervolumen_m3"),
        "{}",
        f.path
    );
}

#[test]
fn substitution_und_trends_werden_geprueft() {
    let markt = "parameter/marktmodell.yaml";
    let kette = "ketten/a.yaml";
    let mit = |d: Daten| {
        d.ersetze(
            markt,
            "      - {id: zeitung, ab: 1800, wirkung: 1.0}\n",
            "      - {id: zeitung, ab: 1800, wirkung: 1.0}\n  substitution: {steilheit: 2, je_jahr: 0.3}\n  trends:\n    - id: eisenwelle\n      produkte: [eisen]\n      laender: [SWE]\n      verlauf: {1900: 1.0, 1905: 1.5, 1910: 1.0}\n",
        )
    };
    let data = mit(Daten::neu()).laden().data.unwrap();
    let m = &data.catalog.market_model;
    assert_eq!(m.substitution_per_year, 0.3);
    assert_eq!(m.trends.len(), 1);
    let eisen = data.catalog.products.id("eisen").unwrap();
    assert_eq!(m.trends[0].products, vec![eisen]);
    // Without the block nothing is displaced.
    let ohne = Daten::neu().laden().data.unwrap();
    assert_eq!(ohne.catalog.market_model.substitution_per_year, 0.0);
    assert!(ohne.catalog.market_model.trends.is_empty());

    // A trend without goods, a duplicate, an unknown product, a factor out of range.
    let d = mit(Daten::neu())
        .ersetze(markt, "      produkte: [eisen]\n", "      produkte: [stahl]\n")
        .ersetze(markt, "1905: 1.5", "1905: 50")
        .ersetze(
            markt,
            "      verlauf: {1900: 1.0, 1905: 50, 1910: 1.0}\n",
            "      verlauf: {1900: 1.0, 1905: 50, 1910: 1.0}\n    - id: eisenwelle\n      verlauf: {1900: 1.0}\n",
        );
    let outcome = d.laden();
    befund(
        &outcome,
        "Wert 50 liegt außerhalb des erlaubten Bereichs 0.1 bis 10.",
    );
    befund(&outcome, "Trend „eisenwelle“ ist doppelt angegeben.");
    let f = befund(
        &outcome,
        "Trend „eisenwelle“ nennt weder Warengruppen noch Produkte.",
    );
    assert_eq!(f.path.to_string(), "marktmodell.trends[1]");
    befund(&outcome, "„stahl“");

    // Substitution fields of a product: range, and only with something to replace.
    let d = Daten::neu().ersetze(
        kette,
        "    richtpreis_usd: 400\n",
        "    richtpreis_usd: 400\n    ersetzt: [erz]\n    ersetzt_nutzen: 20\n    ersetzt_anteil_max: 0.5\n",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 20 liegt außerhalb des erlaubten Bereichs 0.1 bis 10.",
    );
    assert_eq!(f.path.to_string(), "produkte[1].ersetzt_nutzen");
    nur_fehler(&outcome, 1);
    let d = Daten::neu().ersetze(
        kette,
        "    richtpreis_usd: 400\n",
        "    richtpreis_usd: 400\n    ersetzt_anteil_max: 0.5\n",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Produkt „eisen“ gibt „ersetzt_nutzen“ oder „ersetzt_anteil_max“ an, ersetzt aber nichts",
    );
    assert_eq!(f.path.to_string(), "produkte[1].ersetzt");
    nur_fehler(&outcome, 1);
    let data = Daten::neu()
        .ersetze(
            kette,
            "    richtpreis_usd: 400\n",
            "    richtpreis_usd: 400\n    ersetzt: [erz]\n    ersetzt_anteil_max: 0.5\n",
        )
        .laden()
        .data
        .unwrap();
    let eisen = data
        .catalog
        .products
        .get(data.catalog.products.id("eisen").unwrap());
    assert_eq!(
        (eisen.replaces_utility, eisen.replaces_max_share),
        (1.0, 0.5)
    );
}

#[test]
fn ausstattung_wird_geprueft() {
    let datei = "parameter/produktionsmodell.yaml";
    let block = "  ausstattung:\n    einfach: {material: 0.85, arbeit: 0.9, qualitaet: -15}\n    premium: {material: 1.25, arbeit: 1.3, qualitaet: 15}\n    ki_premium_ab: 0.75\n    ki_einfach_bis: 0.25\n";
    let mit = |d: Daten| {
        d.ersetze(
            datei,
            "  zustand_minimum: 0.2\n",
            &format!("  zustand_minimum: 0.2\n{block}"),
        )
    };
    let data = mit(Daten::neu()).laden().data.unwrap();
    let t = &data.catalog.production_model.trims;
    assert!(t.enabled);
    assert_eq!(t.premium.quality, 15.0);
    assert_eq!(t.simple.material, 0.85);
    assert!(
        !Daten::neu()
            .laden()
            .data
            .unwrap()
            .catalog
            .production_model
            .trims
            .enabled
    );

    let d = mit(Daten::neu())
        .ersetze(datei, "material: 1.25", "material: 5")
        .ersetze(datei, "ki_einfach_bis: 0.25", "ki_einfach_bis: 0.9");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 5 liegt außerhalb des erlaubten Bereichs 0.3 bis 3.",
    );
    assert_ort(
        f,
        datei,
        d.zeile(datei, "premium:"),
        "produktionsmodell.ausstattung.premium.material",
    );
    let f = befund(
        &outcome,
        "„ki_einfach_bis“ (0.9) darf nicht über „ki_premium_ab“ (0.75) liegen.",
    );
    assert_eq!(
        f.path.to_string(),
        "produktionsmodell.ausstattung.ki_einfach_bis"
    );
    nur_fehler(&outcome, 2);
}

#[test]
fn ruf_wird_geprueft() {
    let markt = "parameter/marktmodell.yaml";
    let mit = |d: Daten| {
        d.ersetze(
            markt,
            "      - {id: zeitung, ab: 1800, wirkung: 1.0}\n",
            "      - {id: zeitung, ab: 1800, wirkung: 1.0}\n  ruf:\n    gewinn_je_monat: 0.05\n    rueckkehr_je_monat: 0.02\n    unfall_abzug: 0.005\n    rueckruf_schwelle: 25\n    rueckruf_chance: 0.3\n    rueckruf_abzug: 0.1\n    rufgewicht: [0.2, 0.3, 0.5, 0.7, 1.0]\n",
        )
    };
    let data = mit(Daten::neu()).laden().data.unwrap();
    let r = &data.catalog.market_model.reputation;
    assert!(r.enabled);
    assert_eq!(r.recall_below, 25.0);
    assert_eq!(r.weight[4], 1.0);
    assert!(
        !Daten::neu()
            .laden()
            .data
            .unwrap()
            .catalog
            .market_model
            .reputation
            .enabled
    );

    let d = mit(Daten::neu())
        .ersetze(markt, "rueckruf_chance: 0.3", "rueckruf_chance: 3")
        .ersetze(
            markt,
            "rufgewicht: [0.2, 0.3, 0.5, 0.7, 1.0]",
            "rufgewicht: [0.2, 0.3]",
        );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 3 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_ort(
        f,
        markt,
        d.zeile(markt, "rueckruf_chance:"),
        "marktmodell.ruf.rueckruf_chance",
    );
    let f = outcome
        .report
        .findings()
        .iter()
        .find(|f| f.path.to_string() == "marktmodell.ruf.rufgewicht")
        .expect("weight length reported");
    assert!(f.message.contains("5 Werte"), "{}", f.message);
    nur_fehler(&outcome, 2);
}

#[test]
fn ausschreibung_wird_geprueft() {
    let markt = "parameter/marktmodell.yaml";
    let mit = |d: Daten| {
        d.ersetze(
            markt,
            "      - {id: zeitung, ab: 1800, wirkung: 1.0}\n",
            "      - {id: zeitung, ab: 1800, wirkung: 1.0}\n  ausschreibung:\n    warengruppen: [erze]\n    los_anteil: 0.5\n    frist_tage: 30\n    laufzeit_monate: 12\n    preisgewicht: 2.0\n    qualitaetsgewicht: 0.5\n    heimatbonus: 0.3\n    kriegsfaktor: 3.3\n    rufgewicht: 1.0\n    ki_nachlass: 0.05\n    strafe: 0.2\n",
        )
    };
    let data = mit(Daten::neu()).laden().data.unwrap();
    let t = &data.catalog.market_model.tenders;
    assert!(t.enabled);
    assert_eq!(t.months, 12);
    assert_eq!(
        t.groups,
        vec![data.catalog.goods_groups.id("erze").unwrap()]
    );
    assert!(
        !Daten::neu()
            .laden()
            .data
            .unwrap()
            .catalog
            .market_model
            .tenders
            .enabled
    );

    let d = mit(Daten::neu())
        .ersetze(markt, "warengruppen: [erze]", "warengruppen: [erz]")
        .ersetze(markt, "frist_tage: 30", "frist_tage: 0")
        .ersetze(markt, "los_anteil: 0.5", "los_anteil: 2");
    let outcome = d.laden();
    let f = befund(&outcome, "„erz“");
    assert_eq!(
        f.path.to_string(),
        "marktmodell.ausschreibung.warengruppen[0]"
    );
    let f = befund(
        &outcome,
        "Wert 0 liegt außerhalb des erlaubten Bereichs 1 bis 90.",
    );
    assert_ort(
        f,
        markt,
        d.zeile(markt, "frist_tage:"),
        "marktmodell.ausschreibung.frist_tage",
    );
    befund(
        &outcome,
        "Wert 2 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    nur_fehler(&outcome, 3);
}

#[test]
fn vertriebswege_werden_geprueft() {
    let markt = "parameter/marktmodell.yaml";
    let wege = "vertriebswege.yaml";
    let mit = |d: Daten| {
        d.ersetze(
            markt,
            "      - {id: zeitung, ab: 1800, wirkung: 1.0}\n",
            "      - {id: zeitung, ab: 1800, wirkung: 1.0}\n  vertrieb: {grundreichweite: 0.6, ki_verzoegerung_jahre: 15, ki_marge: 0.25}\n",
        )
        .datei(
            wege,
            "vertriebswege:\n  - id: versand\n    verfuegbar_ab: 1890\n    warengruppen: [erze]\n    reichweite: {1890: 0.1, 1950: 0.3}\n    schichtfaktor: [1.3, 1.2, 1.0, 0.8, 0.6]\n    fixkosten_je_land_usd: {1890: 100000}\n    kosten_je_verkauf: 0.15\n    ruf_faktor: 0.9\n    annaeherung: true\n",
        )
        .ersetze(
            "texte/de/a.yaml",
            "verkehrsmittel.fuhrwerk: Fuhrwerk\n",
            "verkehrsmittel.fuhrwerk: Fuhrwerk\nvertriebsweg.versand: Versandhandel\n",
        )
    };
    let data = mit(Daten::neu()).laden().data.unwrap();
    let c = &data.catalog;
    assert_eq!(c.market_model.channels.base_reach, 0.6);
    let v = c.channels.get(c.channels.id("versand").unwrap());
    assert_eq!(v.layer_factor[0], 1.3);
    assert_eq!(v.groups, vec![c.goods_groups.id("erze").unwrap()]);
    // Without the block channels are off.
    assert_eq!(
        Daten::neu()
            .laden()
            .data
            .unwrap()
            .catalog
            .market_model
            .channels
            .base_reach,
        0.0
    );

    let d = mit(Daten::neu())
        .ersetze(wege, "1950: 0.3", "1950: 1.5")
        .ersetze(wege, "[1.3, 1.2, 1.0, 0.8, 0.6]", "[1.3, 1.2]")
        .ersetze(markt, "grundreichweite: 0.6", "grundreichweite: 0");
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Wert 1.5 liegt außerhalb des erlaubten Bereichs 0 bis 1.",
    );
    assert_ort(
        f,
        wege,
        d.zeile(wege, "reichweite:"),
        "vertriebswege[0].reichweite.1950",
    );
    befund(&outcome, "5 Werte");
    befund(
        &outcome,
        "Wert 0 liegt außerhalb des erlaubten Bereichs 0.05 bis 1.",
    );
    nur_fehler(&outcome, 3);
}

#[test]
fn grundprodukt_wird_geprueft() {
    let g = "grundlagen.yaml";
    let data = Daten::neu()
        .ersetze(
            g,
            "  - id: metallurgie\n",
            "  - id: metallurgie\n    grundprodukt: eisen\n",
        )
        .laden()
        .data
        .unwrap();
    let c = &data.catalog;
    assert_eq!(c.basic_products, vec![c.products.id("eisen").unwrap()]);

    // The basic product of the mining branch belongs to metallurgy.
    let d = Daten::neu().ersetze(
        g,
        "  - id: bergbau\n",
        "  - id: bergbau\n    grundprodukt: eisen\n",
    );
    let outcome = d.laden();
    let f = befund(
        &outcome,
        "Grundprodukt „eisen“ gehört zur Branche „metallurgie“, nicht zu „bergbau“",
    );
    assert_ort(
        f,
        g,
        d.zeile(g, "grundprodukt:"),
        "branchen[0].grundprodukt",
    );
    nur_fehler(&outcome, 1);
}

const ANGEBOTE: &str = "maschinenangebote:
  laufzeit_tage: 28
  gueltig_tage: 14
  angebote_je_woche: [1, 5]
  verteilung: {normal_anteil: 0.85, abweichung: 0.35, ausreisser: [0.7, 1.0]}
  wirkung:
    ausstoss: 0.2
    personal: 0.15
    vorprodukte: 0.1
    ausschuss: 0.2
    preis_leistung: 0.12
    preis_lage: 0.08
    flaeche_streuung: 0.1
    bauzeit: 0.2
    rauschen: 0.03
    grenze: 0.2
  mengenrabatt_anteil: 0.1
  ki_schwelle: 0.02
";

const BAUER: &str = "maschinenbauer:
  - {id: nordstahl, land: SWE, tendenz: 0.1, ab: 1900}
";

fn maschinen() -> Daten {
    Daten::neu()
        .datei("parameter/maschinenangebote.yaml", ANGEBOTE)
        .datei("maschinenbauer.yaml", BAUER)
        .datei(
            "texte/de/maschinenbauer.yaml",
            "maschinenbauer.nordstahl: Nordstahl\n",
        )
}

#[test]
fn maschinenangebote_werden_geprueft() {
    let data = maschinen().laden().data.unwrap();
    let c = &data.catalog;
    assert!(c.machine_offers.enabled);
    assert_eq!(c.machine_offers.tender_days, 28);
    assert_eq!(c.machine_builders.len(), 1);

    // Tendency beyond +-0.3.
    let d = maschinen().ersetze("maschinenbauer.yaml", "tendenz: 0.1", "tendenz: 0.5");
    let outcome = d.laden();
    let f = befund(&outcome, "außerhalb des erlaubten Bereichs -0.3 bis 0.3");
    assert_ort(
        f,
        "maschinenbauer.yaml",
        d.zeile("maschinenbauer.yaml", "nordstahl"),
        "maschinenbauer[0].tendenz",
    );
    nur_fehler(&outcome, 1);

    // Weekly offers need exactly two values.
    let outcome = maschinen()
        .ersetze("parameter/maschinenangebote.yaml", "[1, 5]", "[1]")
        .laden();
    befund(&outcome, "2 Werte (von, bis)");
    nur_fehler(&outcome, 1);

    // Outlier magnitude outside 0..1.
    let outcome = maschinen()
        .ersetze(
            "parameter/maschinenangebote.yaml",
            "[0.7, 1.0]",
            "[0.7, 1.5]",
        )
        .laden();
    befund(&outcome, "außerhalb des erlaubten Bereichs 0 bis 1");
    nur_fehler(&outcome, 1);
}

#[test]
fn entwicklungsprojekte_werden_geprueft() {
    let datei = "parameter/forschungsmodell.yaml";
    // Without the block products are researched as before.
    let data = Daten::neu().laden().data.unwrap();
    assert!(data.catalog.research_model.projects.is_none());
    let block = projekt_block();
    let mit = |neu: &str| {
        Daten::neu().ersetze(
            datei,
            "  sachkosten_usd_je_forschertag: 40\n",
            &format!("  sachkosten_usd_je_forschertag: 40\n{neu}"),
        )
    };
    let gut = mit(&block).laden();
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let m = gut
        .data
        .unwrap()
        .catalog
        .research_model
        .projects
        .clone()
        .unwrap();
    assert_eq!(m.phases.len(), 6);
    assert_eq!(m.tempos.len(), 3);
    assert!((m.gate_cost - 0.1).abs() < 1e-9);

    let falsch = mit(&block.replace("gate_kosten: 0.1", "gate_kosten: 0.3")).laden();
    befund(&falsch, "„kosten“ aller Phasen müssen zusammen 1 ergeben");
    let falsch = mit(&block.replacen("wirkung: 0.3", "wirkung: 0.5", 1)).laden();
    befund(
        &falsch,
        "Wirkungen der Rollen einer Phase müssen zusammen 1 ergeben",
    );
    let falsch = mit(&block.replace("id: pm4_pm5", "id: pm3_pm4")).laden();
    befund(&falsch, "muss „pm4_pm5“ stehen");
    let falsch = mit(&block.replacen("id: entwickler", "id: ingenieur", 1)).laden();
    befund(&falsch, "„ingenieur“ ist keine Rolle im Labor");
    let falsch = mit(&block.replace("alter_min: 30", "alter_min: 70")).laden();
    befund(&falsch, "muss kleiner als");
    let falsch = mit(&block.replace("id: normal", "id: mittel")).laden();
    befund(&falsch, "muss „normal“ stehen");
}

/// The block `projekte` of the research model as the tests use it (PD2).
fn projekt_block() -> String {
    let phase = |id: &str, kosten: f64, dauer: f64| {
        format!(
            "      - id: {id}\n        kosten: {kosten}\n        dauer: {dauer}\n        rollen:\n          \
             - {{id: entwickler, bedarf: 2, wirkung: 0.7}}\n          \
             - {{id: laborpersonal, bedarf: 1, wirkung: 0.3}}\n"
        )
    };
    format!(
        "  projekte:\n    punktekosten_usd: 100\n    referenzrate: 10\n    referenzaufwand: 1000\n    \
         bedarf_exponent: 0.5\n    ueberbesetzung: {{maximum: 0.3, tau: 0.5}}\n    restanteil: 0.1\n    \
         unterbesetzung: {{abschlag: 0.5, hoechstens: 0.25}}\n    streuung: 0.1\n    gate_kosten: 0.1\n    \
         phasen:\n{}{}{}{}{}{}    tempo:\n      - {{id: gruendlich, dauer: 1.2, kosten: 1.1, risiko: 0.7}}\n      \
         - {{id: normal, dauer: 1, kosten: 1, risiko: 1}}\n      \
         - {{id: beschleunigt, dauer: 0.8, kosten: 1.2, risiko: 1.5}}\n    ausstattung:\n      \
         - {{id: basis, kosten: 0.8, qualitaet: -0.1}}\n      - {{id: standard, kosten: 1, qualitaet: 0}}\n      \
         - {{id: premium, kosten: 1.3, qualitaet: 0.1}}\n    projektleiter:\n      gehalt_usd_jahr: 90000\n      \
         kapazitaet_neu: 1\n      kapazitaet_weiter: 2\n      kandidaten: 4\n      erfahrung_je_projekt: 0.1\n      alter_min: 30\n      \
         alter_max: 60\n      charaktere: [optimistisch, vorsichtig]\n    risiko:\n      basis: 0.02\n      \
         unterbesetzung: 1.5\n      erfahrung: 0.4\n      branche: 0.3\n      puffer: 0.5\n      fast_track: 1\n",
        phase("pm0_pm1", 0.1, 0.1),
        phase("pm1_pm2", 0.1, 0.1),
        phase("pm2_pm3", 0.1, 0.1),
        phase("pm4_pm5", 0.3, 0.3),
        phase("pm5_pm6", 0.2, 0.2),
        phase("pm6_pm7", 0.1, 0.2),
    )
}

#[test]
fn projektberichte_werden_geprueft() {
    let datei = "parameter/forschungsmodell.yaml";
    let berichte = "    berichte:\n      lebenszyklus_jahre: 10\n      absatzanteil: 0.15\n      \
         anlagen_ohne_nachfrage: 1\n      auslastung: 0.85\n      unschaerfe: 0.3\n      \
         genauigkeit_je_pm: 0.6\n      charaktere:\n        optimistisch: {optimismus: 0.2, weg: innovativ}\n        \
         vorsichtig: {optimismus: -0.1, weg: bewaehrt}\n      ampel:\n        termin: {gelb: 0.1, rot: 0.25}\n        \
         budget: {gelb: 0.1, rot: 0.25}\n        qualitaet: {gelb: 0.05, rot: 0.15}\n        risiko: {gelb: 0.15, rot: 0.3}\n      \
         amortisation_max_jahre: 12\n      auflagen_risiko: 0.15\n    entscheidung:\n      spieler_bei: [2, 4, 6, 7]\n      \
         frist_tage: 7\n    profile:\n      - {id: kosten, kosten: 0.9, dauer: 1, risiko: 1, qualitaet: -0.05}\n      \
         - {id: ausgewogen, kosten: 1, dauer: 1, risiko: 1, qualitaet: 0}\n      \
         - {id: qualitaet, kosten: 1.15, dauer: 1.1, risiko: 1, qualitaet: 0.08}\n    loesungswege:\n      \
         - {id: bewaehrt, kosten: 0.95, dauer: 0.95, risiko: 0.7, qualitaet: -0.03}\n      \
         - {id: standard, kosten: 1, dauer: 1, risiko: 1, qualitaet: 0}\n      \
         - {id: innovativ, kosten: 1.2, dauer: 1.1, risiko: 1.5, qualitaet: 0.1}\n";
    let block = format!("{}{berichte}", projekt_block());
    let mit = |neu: &str| {
        Daten::neu().ersetze(
            datei,
            "  sachkosten_usd_je_forschertag: 40\n",
            &format!("  sachkosten_usd_je_forschertag: 40\n{neu}"),
        )
    };
    let gut = mit(&block).laden();
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let m = gut
        .data
        .unwrap()
        .catalog
        .research_model
        .projects
        .clone()
        .unwrap();
    let r = m.reports.expect("reports");
    assert_eq!((r.profiles.len(), r.paths.len(), r.traits.len()), (3, 3, 2));
    assert_eq!(r.traits[0].path, 2);
    assert_eq!(r.player_at, vec![2, 4, 6, 7]);

    // Without the report blocks the milestones pass by themselves.
    let alt = mit(&projekt_block()).laden();
    assert!(
        alt.data
            .unwrap()
            .catalog
            .research_model
            .projects
            .unwrap()
            .reports
            .is_none()
    );
    // Reports without a decision are an error.
    let falsch = mit(&block.replace(
        "    entscheidung:\n      spieler_bei: [2, 4, 6, 7]\n      frist_tage: 7\n",
        "",
    ))
    .laden();
    befund(&falsch, "Berichte und Entscheidung gehören zusammen");
    let falsch = mit(&block.replace(
        "        vorsichtig: {optimismus: -0.1, weg: bewaehrt}\n",
        "",
    ))
    .laden();
    befund(&falsch, "Für den Charakter „vorsichtig“");
    let falsch = mit(&block.replace("weg: bewaehrt", "weg: genial")).laden();
    befund(&falsch, "„genial“ ist kein Lösungsweg");
    let falsch = mit(&block.replace(
        "gelb: 0.1, rot: 0.25}\n        budget",
        "gelb: 0.3, rot: 0.25}\n        budget",
    ))
    .laden();
    befund(&falsch, "muss kleiner als");
    let falsch = mit(&block.replace("spieler_bei: [2, 4, 6, 7]", "spieler_bei: [2, 9]")).laden();
    befund(&falsch, "außerhalb des erlaubten Bereichs");
    let falsch = mit(&block.replace("id: ausgewogen", "id: mittel")).laden();
    befund(&falsch, "muss „ausgewogen“ stehen");
}

#[test]
fn projektereignisse_werden_geprueft() {
    let datei = "parameter/forschungsmodell.yaml";
    let ereignisse = "    ereignisse:\n      frist_tage: 7\n      mangel:\n        obergrenze: 0.6\n        \
         je_auflage: 0.08\n        je_unterbesetzung: 0.2\n        je_tempo: [0, 0, 0.06]\n      \
         rueckruf:\n        verzug_tage: [120, 400]\n        kosten_anteil: 0.6\n        \
         bekanntheit_verlust: 0.25\n      katalog:\n        - id: problem\n          phasen: [1, 2]\n          \
         gewicht: 2\n          empfehlung: 0\n          optionen:\n            \
         - {id: budget, kosten: 0.2, verzug: 0, qualitaet: 0, mangel: 0}\n            \
         - {id: verzug, kosten: 0, verzug: 0.25, qualitaet: 0, mangel: 0.05}\n";
    let block = format!("{}{ereignisse}", projekt_block());
    let mit = |neu: &str| {
        Daten::neu().ersetze(
            datei,
            "  sachkosten_usd_je_forschertag: 40\n",
            &format!("  sachkosten_usd_je_forschertag: 40\n{neu}"),
        )
    };
    let gut = mit(&block).laden();
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let e = gut
        .data
        .unwrap()
        .catalog
        .research_model
        .projects
        .clone()
        .unwrap()
        .events
        .expect("events");
    assert_eq!((e.kinds.len(), e.kinds[0].options.len()), (1, 2));
    assert_eq!(e.recall_days, (120, 400));

    let falsch = mit(&block.replace("[0, 0, 0.06]", "[0, 0.06]")).laden();
    befund(&falsch, "„je_tempo“ braucht einen Wert je Tempo");
    let falsch = mit(&block.replace("phasen: [1, 2]", "phasen: [1, 7]")).laden();
    befund(&falsch, "außerhalb des erlaubten Bereichs");
    let falsch = mit(&block.replace("empfehlung: 0", "empfehlung: 2")).laden();
    befund(&falsch, "außerhalb des erlaubten Bereichs");
    let falsch = mit(&block.replace("[120, 400]", "[400, 120]")).laden();
    befund(&falsch, "muss kleiner als");
    let falsch = mit(&block.replace(
        "            - {id: verzug, kosten: 0, verzug: 0.25, qualitaet: 0, mangel: 0.05}\n",
        "",
    ))
    .laden();
    befund(&falsch, "braucht zwei oder drei Optionen");
}

#[test]
fn projektweiterentwicklung_wird_geprueft() {
    let datei = "parameter/forschungsmodell.yaml";
    let rolle = |pm: f64, e: f64, fp: f64, lp: f64| {
        format!(
            "          rollen:\n            - {{id: produktmanager, bedarf: 1, wirkung: {pm}}}\n            \
             - {{id: entwickler, bedarf: 2, wirkung: {e}}}\n            \
             - {{id: fertigungsplaner, bedarf: 1, wirkung: {fp}}}\n            \
             - {{id: laborpersonal, bedarf: 1, wirkung: {lp}}}\n"
        )
    };
    let phase = |id: &str, k: f64, d: f64| {
        format!(
            "        - id: {id}\n          kosten: {k}\n          dauer: {d}\n{}",
            rolle(0.25, 0.25, 0.25, 0.25)
        )
    };
    let ziel = |id: &str| {
        format!(
            "        - id: {id}\n          je_stufe: 1\n          obergrenze: 0.5\n          \
             wirkung: {{arbeit: 0.05, qualitaet: 2}}\n"
        )
    };
    let wm = format!(
        "    weiterentwicklung:\n      stufen: 3\n      restanteil: 0.2\n      aufwand: {{anteil: 0.15, wachstum: 1.5}}\n      \
         umruestung_tage: 90\n      sofort_kosten_anteil: 0.2\n      frist_tage: 7\n      \
         ansaetze: {{min: 2, max: 4, streuung: 0.3, kosten_steigung: 0.8}}\n      \
         ergebnis: {{streuung: 0.3, erfahrung: 0.5, unterbesetzung: 1.5}}\n      phasen:\n{}{}{}      ziele:\n{}{}{}{}{}{}",
        phase("wm0_wm1", 0.2, 0.25),
        phase("wm1_wm2", 0.55, 0.5),
        phase("wm2_wm3", 0.25, 0.25),
        ziel("kosten"),
        ziel("qualitaet"),
        ziel("ausschuss"),
        ziel("rate"),
        ziel("vorprodukte"),
        ziel("ausstattung"),
    );
    let block = format!("{}{wm}", projekt_block());
    let mit = |neu: &str| {
        Daten::neu().ersetze(
            datei,
            "  sachkosten_usd_je_forschertag: 40\n",
            &format!("  sachkosten_usd_je_forschertag: 40\n{neu}"),
        )
    };
    let gut = mit(&block).laden();
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let w = gut
        .data
        .unwrap()
        .catalog
        .research_model
        .projects
        .clone()
        .unwrap()
        .improvement
        .expect("improvement");
    assert_eq!((w.phases.len(), w.targets.len(), w.stages), (3, 6, 3));

    let falsch = mit(&block.replace("id: wm1_wm2", "id: wm9")).laden();
    befund(&falsch, "wm9");
    let falsch = mit(&block.replace("id: ausschuss", "id: schrott")).laden();
    befund(&falsch, "schrott");
    let falsch = mit(&block.replace("kosten: 0.55", "kosten: 0.5")).laden();
    befund(&falsch, "ergeben");
    let falsch = mit(&block.replace("min: 2, max: 4", "min: 5, max: 4")).laden();
    befund(&falsch, "Zahl der Ansätze");
}

#[test]
fn projektvorschlaege_werden_geprueft() {
    let datei = "parameter/forschungsmodell.yaml";
    let vorschlaege = "    vorschlaege:\n      offen_max: 3\n      gueltig_tage: 180\n      branchenbonus: 0.5\n      \
         unschaerfe: 0.35\n      genauigkeit: 0.6\n      nachkalkulation_jahre: 2\n      toleranz: 0.5\n      \
         treffer_ab: 0.6\n      gewichte: {termin: 0.25, budget: 0.25, kennzahlen: 0.5}\n";
    let block = format!("{}{vorschlaege}", projekt_block());
    let mit = |neu: &str| {
        Daten::neu().ersetze(
            datei,
            "  sachkosten_usd_je_forschertag: 40\n",
            &format!("  sachkosten_usd_je_forschertag: 40\n{neu}"),
        )
    };
    let gut = mit(&block).laden();
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let v = gut
        .data
        .unwrap()
        .catalog
        .research_model
        .projects
        .clone()
        .unwrap()
        .suggestions
        .expect("suggestions");
    assert_eq!((v.open_max, v.review_years), (3, 2));

    let falsch = mit(&block.replace("kennzahlen: 0.5", "kennzahlen: 0.4")).laden();
    befund(&falsch, "ergeben");
    let falsch = mit(&block.replace("unschaerfe: 0.35", "unschaerfe: 1.5")).laden();
    befund(&falsch, "außerhalb des erlaubten Bereichs");
    let falsch = mit(&block.replace("toleranz: 0.5", "toleranz: 0")).laden();
    assert!(!falsch.report.findings().is_empty());
}

#[test]
fn entwicklungsleitung_wird_geprueft() {
    let datei = "parameter/forschungsmodell.yaml";
    let zusatz = "    leitung:\n      vorlauf_monate: 2\n      einarbeitung_monate: 1\n      abfindung_monate: 3\n      \
         vorausschau_monate: 3\n      geduld_monate: 2\n      reserve: 0.1\n      reserve_sparsam: 0.3\n      \
         reserve_grosszuegig: 1.8\n      budget_anteil: 0.04\n      grundausstattung: 0.5\n";
    let block = format!("{}{zusatz}", projekt_block());
    let mit = |neu: &str| {
        Daten::neu().ersetze(
            datei,
            "  sachkosten_usd_je_forschertag: 40\n",
            &format!("  sachkosten_usd_je_forschertag: 40\n{neu}"),
        )
    };
    let gut = mit(&block).laden();
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let m = gut
        .data
        .unwrap()
        .catalog
        .research_model
        .projects
        .clone()
        .unwrap();
    let l = m.leadership.expect("leadership");
    assert_eq!((l.lead_months, l.severance_months), (2, 3));
    assert!((l.reserve_generous - 1.8).abs() < 1e-9);

    let falsch = mit(&block.replace("reserve: 0.1", "reserve: 1.5")).laden();
    befund(&falsch, "außerhalb des erlaubten Bereichs");
    let falsch = mit(&block.replace("vorlauf_monate: 2", "vorlauf_monate: 40")).laden();
    befund(&falsch, "außerhalb des erlaubten Bereichs");
}

#[test]
fn projektgremium_fast_track_und_pilot_werden_geprueft() {
    let datei = "parameter/forschungsmodell.yaml";
    let zusatz = "    gremium:\n      unschaerfe: 0.5\n      genauigkeit: 0.6\n      schwelle: 0.25\n      pause_mehrheit: 3\n      schleifen_max: 2\n    \
         fast_track:\n      erfahrung_min: 0.3\n      ueberlappung: 0.25\n      kosten_faktor: 1.1\n      mangel: 0.12\n    \
         pilot:\n      risiko_ab: 0.15\n      dauer_anteil: 0.3\n      kosten_anteil: 0.25\n      aufdeckung_faktor: 2.5\n      \
         behebung: 0.7\n      nacharbeit: 0.15\n      qualitaet_bonus: 0.03\n      beruhigung: 0.4\n";
    let block = format!("{}{zusatz}", projekt_block());
    let mit = |neu: &str| {
        Daten::neu().ersetze(
            datei,
            "  sachkosten_usd_je_forschertag: 40\n",
            &format!("  sachkosten_usd_je_forschertag: 40\n{neu}"),
        )
    };
    let gut = mit(&block).laden();
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let m = gut
        .data
        .unwrap()
        .catalog
        .research_model
        .projects
        .clone()
        .unwrap();
    assert_eq!(m.council.expect("council").pause_majority, 3);
    assert!((m.fast_track.expect("fast_track").overlap - 0.25).abs() < 1e-9);
    assert!((m.pilot.expect("pilot").detection - 2.5).abs() < 1e-9);

    let falsch = mit(&block.replace("schwelle: 0.25", "schwelle: 1.5")).laden();
    befund(&falsch, "außerhalb des erlaubten Bereichs");
    let falsch = mit(&block.replace("kosten_faktor: 1.1", "kosten_faktor: 0.5")).laden();
    befund(&falsch, "außerhalb des erlaubten Bereichs");
    let falsch = mit(&block.replace("dauer_anteil: 0.3", "dauer_anteil: 0")).laden();
    assert!(!falsch.report.findings().is_empty());
    let falsch = mit(&block.replace("behebung: 0.7", "behebung: 2")).laden();
    befund(&falsch, "außerhalb des erlaubten Bereichs");
}

#[test]
fn projekt_teilwissen_plattform_lizenz_und_partnerschaft_werden_geprueft() {
    let datei = "parameter/forschungsmodell.yaml";
    let zusatz = "    teilwissen:\n      anteil: 0.6\n      wirkung: 0.5\n    plattform:\n      wirkung: 0.4\n    lizenz:\n      \
         preis_anteil: 0.5\n      ab_phase: 4\n      gebuehr_anteil: 0.1\n      gebuehr_jahre: 5\n    partnerschaft:\n      \
         kosten_anteil: 0.5\n      ki_annahme: 0.5\n";
    let block = format!("{}{zusatz}", projekt_block());
    let mit = |neu: &str| {
        Daten::neu().ersetze(
            datei,
            "  sachkosten_usd_je_forschertag: 40\n",
            &format!("  sachkosten_usd_je_forschertag: 40\n{neu}"),
        )
    };
    let gut = mit(&block).laden();
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let m = gut
        .data
        .unwrap()
        .catalog
        .research_model
        .projects
        .clone()
        .unwrap();
    assert!((m.knowledge.expect("knowledge").share - 0.6).abs() < 1e-9);
    assert_eq!(m.license.expect("license").from_phase, 4);
    assert!((m.partnership.expect("partnership").cost_share - 0.5).abs() < 1e-9);

    let falsch = mit(&block.replace("ab_phase: 4", "ab_phase: 9")).laden();
    befund(&falsch, "zwischen 1 und 5");
    let falsch = mit(&block.replace("wirkung: 0.5", "wirkung: 1.5")).laden();
    befund(&falsch, "außerhalb des erlaubten Bereichs");
    let falsch = mit(&block.replace("kosten_anteil: 0.5", "kosten_anteil: 0.99")).laden();
    befund(&falsch, "außerhalb des erlaubten Bereichs");
}

#[test]
fn projekt_erstankunft_und_foerderung_werden_geprueft() {
    let datei = "parameter/forschungsmodell.yaml";
    let zusatz = "    erstankunft:\n      bekanntheit: 0.25\n      ruf: 0.05\n    foerderung:\n      \
         - {id: a, von: 1900, bis: 1939, anteil: 0.05, deckel_usd: 150000, annaeherung: true}\n      \
         - {id: b, von: 1945, bis: 1964, anteil: 0.15, deckel_usd: 400000, annaeherung: true}\n";
    let block = format!("{}{zusatz}", projekt_block());
    let mit = |neu: &str| {
        Daten::neu().ersetze(
            datei,
            "  sachkosten_usd_je_forschertag: 40\n",
            &format!("  sachkosten_usd_je_forschertag: 40\n{neu}"),
        )
    };
    let gut = mit(&block).laden();
    assert!(gut.report.findings().is_empty(), "{}", alle(&gut));
    let m = gut
        .data
        .unwrap()
        .catalog
        .research_model
        .projects
        .clone()
        .unwrap();
    assert!((m.first_mover.expect("first mover").awareness - 0.25).abs() < 1e-9);
    assert_eq!(m.subsidies.len(), 2);

    let falsch = mit(&block.replace("von: 1900, bis: 1939", "von: 1950, bis: 1939")).laden();
    befund(&falsch, "endet vor seinem Beginn");
    let falsch = mit(&block.replace("{id: b,", "{id: a,")).laden();
    befund(&falsch, "mehrfach");
    let falsch = mit(&block.replace(
        "deckel_usd: 400000, annaeherung: true",
        "deckel_usd: 400000",
    ))
    .laden();
    befund(&falsch, "geschätzt");
    let falsch = mit(&block.replace("anteil: 0.15", "anteil: 1.5")).laden();
    befund(&falsch, "außerhalb des erlaubten Bereichs");
}
