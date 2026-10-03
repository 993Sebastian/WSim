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
    werte:
      bevoelkerung: {1900: 5_000_000, 1930: 6_000_000}
      bip_je_kopf_usd: {1900: 6_000}
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
  - id: eisen
    art: halbzeug
    branche: metallurgie
    einheit: t
    verwendung: industrie
    warengruppe: erze
    transportklasse: schuettgut
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

const TEXTE: &str = "\
einheit.t: t
einheit.stueck: Stück
kontinent.europa: Europa
branche.bergbau: Bergbau
branche.metallurgie: Metallurgie
warengruppe.erze: Erze
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
            ("ketten/a.yaml", KETTE),
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
        .ersetze("ketten/a.yaml", "land: SWE", "land: swe");
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
        .ersetze("ketten/a.yaml", "anlagen:", "  - id: glas\n    art: halbzeug\n    branche: metallurgie\n    einheit: t\n    verwendung: industrie\n    warengruppe: erze\n    transportklasse: schuettgut\n    staatsmarkt:\n      preis_usd: 1_200\n      verfuegbar_ab: 1900\nanlagen:")
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
        5,
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
