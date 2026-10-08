//! Every kind of data error produces an understandable message at the right place.

use std::collections::BTreeMap;

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
  schulung: {kosten_anteil_lohn: 0.05, aufbau_je_monat: 0.08, verlust_je_monat: 0.03, arbeitsersparnis: 0.1, qualitaet_punkte: 8}
  automatisierung: {arbeitsersparnis: 0.8, kostenanteil: 0.5}
  qualitaet: {vorprodukte: 0.3, automatisierung: 10, zustand: 20}
  zustand_minimum: 0.2
  lohnaufschlag_max: 1.0
  stilllegung: {instandhaltung_anteil: 0.25, wiederanlauf_tage: 30, wiederanlauf_kosten: 0.02}
  verkauf: {erloes_anteil: 0.5, schrottwert: 0.03}
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
    je_stufe: {qualitaet: 4, arbeit: 0.03, vorprodukte: 0.02}
    aufwand: {anteil: 0.2, wachstum: 1.6, grundaufwand: 10_000}
    gemeingut_nach_jahren: 15
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
        .ersetze(datei, "pacht_anteil: 0.05", "pacht_anteil: 1.5");
    let outcome = d.laden();
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
    nur_fehler(&outcome, 4);

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
        .ersetze(datei, "arbeit: 0.03", "arbeit: 0.2");
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
        "Wert 0.2 liegt außerhalb des erlaubten Bereichs 0 bis 0.09.",
    );
    assert_eq!(
        f.path.to_string(),
        "forschungsmodell.weiterentwicklung.je_stufe.arbeit"
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
        "    - {id: vorstand, pruefung_tage: 30, gehalt_fach: 10, gehalt_leitung: 15,\n       budget_fach: [0.03, 0.08], budget_leitung: [0.08, 0.20],\n       fachstellen: [produktion, einkauf_lager, vertrieb_marketing, personal, forschung,\n                     finanzen, strategie, recht],\n       themen: [ueberkapazitaet, stillgelegt, wiederanfahren, ausbau, kraftwerk,\n                lagerstaette, werbung, kasse, kaufangebot, antwort, engpass, lizenz,\n                umschuldung, gehaltsrunde, startup]}\n",
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
  ki:
    abschlag_verkauf: 0.05
    aufschlag_kauf: 0.05
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

    for (alt, neu, meldung, pfad) in [
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
  dividende:
    monat: 5
    ki_quote: 0.4
    kasse_max: 0.5
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
    assert_eq!(m.dividend_month, 5);
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
