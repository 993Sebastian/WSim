//! All validation messages in one place (German, aimed at people editing data files).

use std::fmt::Display;

fn suggestion(suggestion: Option<&str>) -> String {
    suggestion
        .map(|s| format!(" Meinten Sie „{s}“?"))
        .unwrap_or_default()
}

fn list(items: &[&str]) -> String {
    items
        .iter()
        .map(|i| format!("„{i}“"))
        .collect::<Vec<_>>()
        .join(", ")
}

// --- YAML syntax ---

pub fn yaml_syntax(detail: &str) -> String {
    format!("Die Datei ist kein gültiges YAML (Parser meldet: {detail}).")
}

pub fn yaml_multiple_documents() -> String {
    "Eine Datei darf nur ein YAML-Dokument enthalten (kein zweites „---“).".into()
}

pub fn yaml_alias() -> String {
    "Anker und Verweise (&name, *name) werden in Datendateien nicht unterstützt.".into()
}

pub fn yaml_complex_key() -> String {
    "Schlüssel müssen einfache Werte sein, keine Listen oder Zuordnungen.".into()
}

pub fn yaml_duplicate_key(key: &str) -> String {
    format!("Schlüssel „{key}“ kommt hier doppelt vor.")
}

// --- Structure (raised while reading entries) ---

pub fn unknown_field(field: &str, expected: &[&str], suggested: Option<&str>) -> String {
    format!(
        "Unbekanntes Feld „{field}“.{} Erlaubt sind: {}.",
        suggestion(suggested),
        list(expected)
    )
}

pub fn missing_field(field: &str) -> String {
    format!("Pflichtfeld „{field}“ fehlt.")
}

pub fn duplicate_field(field: &str) -> String {
    format!("Feld „{field}“ ist doppelt angegeben.")
}

pub fn unknown_value(value: &str, expected: &[&str], suggested: Option<&str>) -> String {
    format!(
        "Unbekannter Wert „{value}“.{} Erlaubt sind: {}.",
        suggestion(suggested),
        list(expected)
    )
}

pub fn wrong_type(found: &str, expected: &str) -> String {
    format!("Erwartet wird {expected}, gefunden wurde {found}.")
}

pub fn decimal_comma_hint() -> &'static str {
    " Dezimalzahlen werden mit Punkt geschrieben, z. B. 1.5."
}

pub fn invalid_value(found: &str, expected: &str) -> String {
    format!("Ungültiger Wert: {found}; erwartet wird {expected}.")
}

pub fn wrong_length(found: usize, expected: &str) -> String {
    format!("Falsche Anzahl an Einträgen ({found}), erwartet wird {expected}.")
}

// --- Files and sections ---

pub fn read_error(error: impl Display) -> String {
    format!("Datei kann nicht gelesen werden: {error}")
}

pub fn not_a_directory() -> String {
    "Datenverzeichnis nicht gefunden.".into()
}

pub fn file_not_a_map() -> String {
    "Eine Datendatei besteht aus Abschnitten (z. B. „produkte:“), jeder mit einer Liste von Einträgen.".into()
}

pub fn empty_file() -> String {
    "Die Datei ist leer.".into()
}

pub fn yml_extension() -> String {
    "Dateien mit der Endung .yml werden nicht gelesen; bitte .yaml verwenden.".into()
}

pub fn unknown_section(section: &str, expected: &[&str], suggested: Option<&str>) -> String {
    format!(
        "Unbekannter Abschnitt „{section}“.{} Erlaubt sind: {}.",
        suggestion(suggested),
        list(expected)
    )
}

pub fn section_not_a_list(section: &str) -> String {
    format!("Abschnitt „{section}“ muss eine Liste sein (Einträge beginnen mit „- “).")
}

pub fn meta_missing() -> String {
    "Abschnitt „meta“ fehlt (erwartet in meta.yaml).".into()
}

pub fn meta_duplicate(other: &str) -> String {
    format!("Abschnitt „meta“ darf es nur einmal geben; er steht bereits in {other}.")
}

// --- Entries and references ---

pub fn invalid_key(key: &str) -> String {
    format!(
        "Ungültige ID „{key}“: erlaubt sind Kleinbuchstaben a–z, Ziffern und „_“, beginnend mit einem Buchstaben (keine Umlaute)."
    )
}

pub fn invalid_country_code(key: &str) -> String {
    format!(
        "Ungültiger Ländercode „{key}“: erwartet werden drei Großbuchstaben nach ISO 3166 (z. B. „DEU“)."
    )
}

pub fn duplicate_key(kind: &str, key: &str, other: &str) -> String {
    format!("{kind} „{key}“ ist doppelt definiert; erste Definition in {other}.")
}

pub fn unknown_reference(kind: &str, key: &str, suggested: Option<&str>) -> String {
    format!(
        "{kind} „{key}“ ist nicht definiert.{}",
        suggestion(suggested)
    )
}

pub fn qualification_needs_specialization(qualification: &str, example: &str) -> String {
    format!(
        "Qualifikation „{qualification}“ braucht eine Fachrichtung, z. B. „{qualification}.{example}“."
    )
}

pub fn qualification_without_specialization(qualification: &str) -> String {
    format!("Qualifikation „{qualification}“ wird ohne Fachrichtung angegeben.")
}

pub fn out_of_range(value: f64, min: f64, max: f64) -> String {
    format!("Wert {value} liegt außerhalb des erlaubten Bereichs {min} bis {max}.")
}

pub fn not_positive(value: f64) -> String {
    format!("Wert {value} muss größer als 0 sein.")
}

pub fn money_out_of_range(value: f64) -> String {
    format!("Betrag {value} ist ungültig (zu groß oder negativ).")
}

pub fn year_out_of_range(year: i32, min: i32, max: i32) -> String {
    format!("Jahr {year} liegt außerhalb von {min} bis {max}.")
}

pub fn time_series_empty() -> String {
    "Mindestens ein Jahreswert ist nötig.".into()
}

pub fn year_range_inverted(from: i32, until: i32) -> String {
    format!("„verfuegbar_ab“ ({from}) liegt nach „verfuegbar_bis“ ({until}).")
}

pub fn weight_missing(unit: &str) -> String {
    format!("Feld „gewicht_kg“ fehlt: Die Einheit „{unit}“ legt kein Gewicht fest.")
}

pub fn demand_type_ambiguous() -> String {
    "Genau eines von „verbrauch“ (Verbrauchsgut), „gebrauch“ (Gebrauchsgut) oder \
     „ergaenzung“ (Verbrauch je besessenem Gebrauchsgut) angeben."
        .into()
}

pub fn season_length(found: usize) -> String {
    format!("„saison“ braucht 12 Monatswerte, angegeben sind {found}.")
}

pub fn product_replaces_itself(key: &str) -> String {
    format!("Produkt „{key}“ kann sich nicht selbst ersetzen.")
}

pub fn technology_cycle(chain: &[&str]) -> String {
    format!("Zirkuläre Voraussetzungen: {}.", chain.join(" → "))
}

pub fn research_effort_missing(year: i32, start: i32) -> String {
    format!(
        "„forschungsaufwand“ fehlt: Die Technologie wird {year} erfunden, also nach dem frühesten Startjahr {start}."
    )
}

pub fn research_effort_unused(year: i32, start: i32) -> String {
    format!(
        "„forschungsaufwand“ wird nicht verwendet: Die Technologie ({year}) ist ab dem Startjahr {start} bekannt."
    )
}

pub fn prerequisite_younger(prerequisite: &str, year: i32) -> String {
    format!(
        "Voraussetzung „{prerequisite}“ wird erst {year} erfunden, also später als diese Technologie."
    )
}

pub fn output_index_needs_raw_material(product: &str) -> String {
    format!(
        "Einen Förderindex („foerderindex“) haben nur Rohstoffe; „{product}“ ist kein Rohstoff."
    )
}

pub fn rent_needs_raw_material(product: &str) -> String {
    format!(
        "Pacht und Förderabgaben („pacht_anteil“) zahlen nur Rohstoffe; „{product}“ ist kein Rohstoff."
    )
}

pub fn extraction_needs_raw_material(product: &str) -> String {
    format!("Abbau-Rezepte („abbau: true“) erzeugen Rohstoffe; „{product}“ ist kein Rohstoff.")
}

pub fn too_many_inputs(count: usize, max: usize) -> String {
    format!(
        "Das Rezept hat {count} Vorprodukte; erlaubt sind höchstens {max} (Lastenheft §17.2). Lieber ein Eingangsmaterial weglassen."
    )
}

pub fn product_tree_too_deep(levels: usize, max: usize, path: &[&str]) -> String {
    format!(
        "Der Produktbaum hat hier {levels} Ebenen ({}); erlaubt sind höchstens {max} vom Rohstoff bis zum Zielprodukt (Lastenheft §17.2).",
        path.join(" → ")
    )
}

pub fn product_tree_deep(levels: usize, typical: usize, path: &[&str]) -> String {
    format!(
        "Der Produktbaum hat hier {levels} Ebenen ({}); mehr als {typical} sind nur für sehr komplexe Produkte gedacht. Ebenen sparen oder beim Produkt „sehr_komplex: true“ angeben (Lastenheft §17.2).",
        path.join(" → ")
    )
}

pub fn recipe_consumes_own_product(product: &str) -> String {
    format!("Das Rezept verbraucht sein eigenes Produkt „{product}“.")
}

pub fn product_without_source(key: &str) -> String {
    format!(
        "Produkt „{key}“ kann weder hergestellt noch vom Staatsmarkt bezogen werden: Es fehlt ein Rezept oder ein Eintrag „staatsmarkt“."
    )
}

pub fn reference_margin(
    product: &str,
    year: i32,
    (cost, price): (f64, f64),
    margin: f64,
    (min, max): (f64, f64),
) -> String {
    let percent = |share: f64| format!("{:.0}", share * 100.0);
    format!(
        "Das Rezept stellt „{product}“ {year} zu Richtpreisen für {cost:.2} USD je Einheit \
         her, der Richtpreis ist {price:.2} USD: Marge {} %, erwartet {}–{} %. \
         Richtpreis, Vorprodukte, Arbeitsstunden, Anlage oder bei Rohstoffen die Pacht \
         prüfen (Rechnung: docs/FORMELN.md, Plausibilität).",
        percent(margin),
        percent(min),
        percent(max),
    )
}

pub fn raw_material_without_deposit(key: &str) -> String {
    format!("Für den Rohstoff „{key}“ gibt es ein Abbau-Rezept, aber keine Lagerstätte.")
}

pub fn deposit_needs_raw_material(product: &str) -> String {
    format!("Lagerstätten enthalten Rohstoffe; „{product}“ ist kein Rohstoff.")
}

pub fn deposit_reserve_missing() -> String {
    "Feld „vorrat“ fehlt: Nicht erneuerbare Lagerstätten haben einen endlichen Vorrat.".into()
}

pub fn deposit_reserve_renewable() -> String {
    "Erneuerbare Lagerstätten („erneuerbar: true“) haben keinen „vorrat“, nur eine Höchstförderung."
        .into()
}

// --- Texts ---

pub fn text_not_scalar() -> String {
    "Ein Text muss ein einfacher Wert sein (ggf. in Anführungszeichen).".into()
}

pub fn text_empty() -> String {
    "Der Text ist leer.".into()
}

pub fn text_duplicate(key: &str, other: &str) -> String {
    format!("Text „{key}“ ist doppelt definiert; erste Definition in {other}.")
}

pub fn text_missing(key: &str, language: &str) -> String {
    format!("Text „{key}“ fehlt in texte/{language}/.")
}

pub fn text_unused(key: &str) -> String {
    format!("Text „{key}“ gehört zu keinem Eintrag.")
}

pub fn texts_missing(language: &str) -> String {
    format!("Ordner texte/{language}/ fehlt.")
}

pub fn negative(value: f64) -> String {
    format!("Wert {value} darf nicht negativ sein.")
}

// --- Countries and country model ---

pub fn rank_unknown(rank: u8, max: u8) -> String {
    format!("Eine Qualifikationsstufe {rank} gibt es nicht (Stufen 1 bis {max}).")
}

pub fn section_missing(section: &str, file: &str) -> String {
    format!("Abschnitt „{section}“ fehlt (erwartet in {file}).")
}

pub fn country_model_missing() -> String {
    "Abschnitt „laendermodell“ fehlt (erwartet in parameter/laendermodell.yaml).".into()
}

pub fn section_duplicate(section: &str, other: &str) -> String {
    format!("Abschnitt „{section}“ darf es nur einmal geben; er steht bereits in {other}.")
}

pub fn rows_not_ascending() -> String {
    "Die Zeilen müssen nach „bip_je_kopf_usd“ aufsteigend sortiert sein.".into()
}

pub fn table_empty() -> String {
    "Mindestens eine Zeile ist nötig.".into()
}

pub fn entry_missing(kind: &str, key: &str) -> String {
    format!("Eintrag für {kind} „{key}“ fehlt.")
}

pub fn shares_sum(sum: f64) -> String {
    format!("Die Anteile ergeben zusammen {sum:.4}, müssen aber 1 ergeben.")
}

pub fn specialization_not_allowed(qualification: &str) -> String {
    format!("Qualifikation „{qualification}“ hat keine Fachrichtungen.")
}

pub fn neighbor_self(key: &str) -> String {
    format!("Land „{key}“ kann nicht sein eigener Nachbar sein.")
}

pub fn neighbor_asymmetric(neighbor: &str, key: &str) -> String {
    format!("„{neighbor}“ führt „{key}“ nicht als Nachbarn.")
}

pub fn size_medium_not_one(capacity: f64) -> String {
    format!(
        "Die Größe „mittel“ muss die Kapazität 1 haben (hier {capacity}): Für sie gelten die \
         Werte der Anlagen in den Daten."
    )
}

pub fn sizes_not_increasing(smaller: &str, larger: &str) -> String {
    format!(
        "Die Kapazität muss mit der Größe wachsen: „{larger}“ hat nicht mehr Kapazität als \
         „{smaller}“."
    )
}

pub fn plot_classes_missing() -> String {
    "Mindestens eine Größenklasse für Grundstücke ist nötig.".to_owned()
}

pub fn shares_not_one(what: &str, sum: f64) -> String {
    let sum = (sum * 1000.0).round() / 1000.0;
    format!("Die Anteile „{what}“ ergeben zusammen {sum} statt 1.")
}

pub fn region_too_small(region: &str) -> String {
    format!("Region „{region}“ muss mindestens zwei Länder umfassen.")
}

pub fn region_member_is_country(member: &str, region: &str) -> String {
    format!("„{member}“ ist ein eigenes Land und kann nicht zu „{region}“ gehören.")
}

pub fn region_member_twice(member: &str, first: &str) -> String {
    format!("Land „{member}“ ist mehr als einmal einer Region zugeordnet; erste Angabe in {first}.")
}

pub fn range_inverted(min: &str, max: &str) -> String {
    format!("„{min}“ muss kleiner als „{max}“ sein.")
}

pub fn vehicle_without_classes() -> String {
    "Mindestens eine Transportklasse ist nötig: Was befördert dieses Verkehrsmittel?".into()
}

pub fn researchers_need_fields(qualification: &str) -> String {
    format!("Forscher brauchen eine Qualifikation mit Fachrichtungen; „{qualification}“ hat keine.")
}

pub fn electricity_not_energy(key: &str) -> String {
    format!("„{key}“ steht für Strom und muss deshalb ein Produkt der Art „energie“ sein.")
}

pub fn complement_needs_durable(key: &str) -> String {
    format!(
        "„{key}“ ist kein Gebrauchsgut; eine Ergänzung gehört zu einem Gebrauchsgut wie dem Auto."
    )
}

pub fn start_facility_wrong_site(facility: &str) -> String {
    format!("Anlage „{facility}“ passt nicht zum Standorttyp der Startform.")
}

pub fn start_recipe_wrong_facility(recipe: &str, facility: &str) -> String {
    format!("Rezept „{recipe}“ läuft nicht auf der Anlage „{facility}“.")
}

pub fn start_needs_new_technology(key: &str) -> String {
    format!("„{key}“ braucht eine Technologie, die im frühesten Startjahr noch nicht erfunden ist.")
}

pub fn default_difficulty_unknown(key: &str) -> String {
    format!("Schwierigkeit „{key}“ ist unter „schwierigkeiten“ nicht aufgeführt.")
}

pub fn management_topic_twice(topic: &str, first: &str) -> String {
    format!(
        "Das Thema „{topic}“ gehört schon zum Bereich „{first}“; jedes Thema gehört zu höchstens einem Bereich."
    )
}

pub fn management_pool_bounds(min: u32, max: u32) -> String {
    format!("Der Pool braucht `max` ({max}) mindestens so groß wie `min` ({min}).")
}

pub fn management_site_specialists() -> String {
    "Die Fachstellen der Standorte stehen je Standorttyp unter `standorttypen`, nicht bei der Ebene."
        .to_owned()
}

pub fn management_topic_without_function(topic: &str) -> String {
    format!(
        "Das Thema „{topic}“ gehört zu keinem Bereich; eine Ebene kann nur Themen ihrer Bereiche aufgreifen."
    )
}

pub fn management_budget_order(decision: f64, year: f64) -> String {
    format!(
        "Das Budget je Entscheidung ({decision}) darf nicht größer sein als das je Jahr ({year})."
    )
}

pub fn list_empty() -> String {
    "Die Liste darf nicht leer sein.".into()
}

pub fn name_placeholder_unknown(placeholder: &str) -> String {
    format!(
        "Platzhalter „{{{placeholder}}}“ ist unbekannt; erlaubt sind {{familienname}}, {{ort}}, \
         {{rechtsform}} und {{branche}}."
    )
}

pub fn name_default_count(count: usize) -> String {
    format!("Genau eine Namensgruppe muss „standard: true“ haben, gefunden: {count}.")
}

pub fn name_country_twice(country: &str, first: &str) -> String {
    format!("Land „{country}“ gehört schon zur Namensgruppe „{first}“.")
}

// --- Product names (M42) ---

pub fn naming_placeholder_unknown(placeholder: &str) -> String {
    format!(
        "Platzhalter „{{{placeholder}}}“ ist unbekannt; erlaubt sind {{stamm}}, {{zahl}}, \
         {{buchstabe}} und {{zusatz}}."
    )
}

pub fn naming_stem_missing() -> String {
    "Das Muster braucht den Platzhalter {stamm}.".into()
}

pub fn naming_list_needed(placeholder: &str, field: &str) -> String {
    format!("Das Muster nutzt {{{placeholder}}}; dafür darf „{field}“ nicht leer sein.")
}

pub fn naming_group_twice(group: &str, first: &str) -> String {
    format!("Warengruppe „{group}“ gehört schon zum Stil „{first}“.")
}

pub fn naming_excluded(word: &str) -> String {
    format!("„{word}“ steht unter „ausgeschlossen“ (echte Produkt- und Markennamen).")
}

pub fn naming_word_twice(word: &str) -> String {
    format!("„{word}“ steht doppelt in der Liste.")
}

pub fn naming_word_empty() -> String {
    "Leere Einträge sind nicht erlaubt.".into()
}

pub fn naming_years(from: i32, until: i32) -> String {
    format!("„ab“ ({from}) liegt nach „bis“ ({until}).")
}

pub fn naming_no_open_pattern() -> String {
    "Mindestens ein Muster braucht weder „ab“ noch „bis“, damit es in jedem Jahr einen \
     Namen gibt."
        .into()
}

pub fn naming_number_zero() -> String {
    "Zahlen müssen größer als 0 sein.".into()
}

pub fn real_deposit_other_country(deposit: &str, country: &str) -> String {
    format!("Lagerstätte „{deposit}“ liegt nicht in „{country}“.")
}

pub fn real_site_mixed_types() -> String {
    "Alle Anlagen eines Standorts brauchen denselben Standorttyp.".into()
}

pub fn real_site_needs_deposit(facility: &str) -> String {
    format!("Anlage „{facility}“ fördert einen Rohstoff; der Standort braucht eine „lagerstaette“.")
}

pub fn real_company_too_young(founded: i32, year: i32) -> String {
    format!("Gegründet {founded}: Reale Firmen beschreiben den Stand {year}.")
}

pub fn event_date_invalid(text: &str) -> String {
    format!("„{text}“ ist kein gültiges Datum; erwartet wird JJJJ-MM-TT, z. B. 1914-07-28.")
}

pub fn event_kind_unknown(kind: &str, known: &str) -> String {
    format!("Unbekannte Ereignisart „{kind}“; erlaubt sind {known}.")
}

// --- Milestones (M23) ---

pub fn milestone_value_missing(kind: &str) -> String {
    format!("Die Etappe der Art „{kind}“ braucht einen „wert“.")
}

pub fn milestone_value_not_allowed(kind: &str) -> String {
    format!("Die Etappe der Art „{kind}“ hat keinen „wert“; bitte entfernen.")
}

pub fn milestone_count_invalid(value: f64) -> String {
    format!("„wert“ muss eine ganze Zahl von 1 bis 1000 sein, nicht {value}.")
}

pub fn milestone_factor_invalid(value: f64) -> String {
    format!("„wert“ ist das Vielfache des Startkapitals und muss größer als 1 sein, nicht {value}.")
}

// --- Currencies (M21) ---

pub fn point_in_time_invalid(text: &str) -> String {
    format!(
        "„{text}“ ist kein gültiger Zeitpunkt: erlaubt sind ein Jahr (1924) oder Jahr und \
         Monat („1923-11“)."
    )
}

pub fn field_empty(field: &str) -> String {
    format!("„{field}“ darf nicht leer sein.")
}

pub fn rates_or_peg() -> String {
    "Genau eines von „kurse“ (Einheiten je US-Dollar) oder „bindung“ (fest an eine andere \
     Währung) angeben."
        .into()
}

pub fn peg_to_pegged(other: &str) -> String {
    format!(
        "Die Währung „{other}“ ist selbst gebunden; binden Sie an eine Währung mit eigenen \
         Kursen."
    )
}

pub fn lead_currency_not_one(currency: &str) -> String {
    format!(
        "Die Leitwährung „{currency}“ ist die Einheit des Spiels: Sie braucht eigene Kurse, \
         alle gleich 1."
    )
}

pub fn base_year_without_price(year: i32) -> String {
    format!("Für das Basisjahr {year} fehlt ein Wert in „werte“.")
}

pub fn periods_not_ascending() -> String {
    "Die Zeiträume müssen nach „ab“ aufsteigend sortiert sein.".into()
}

pub fn conversion_without_change() -> String {
    "„umrechnung“ gilt nur für einen Zeitraum, in dem das Land die Währung wechselt.".into()
}

pub fn first_period_late(from: &str, start: i32) -> String {
    format!(
        "Der erste Zeitraum beginnt erst {from}; er muss spätestens im frühesten Startjahr \
         {start} beginnen."
    )
}

pub fn country_without_currency(country: &str) -> String {
    format!("Für das Land „{country}“ fehlt ein Eintrag im Abschnitt „landeswaehrungen“.")
}

pub fn country_currencies_duplicate(country: &str, first: &str) -> String {
    format!("Für das Land „{country}“ gibt es schon einen Eintrag ({first}).")
}

pub fn currency_unused(currency: &str) -> String {
    format!("Die Währung „{currency}“ wird von keinem Land verwendet und ist an nichts gebunden.")
}

pub fn development_field_without_researchers(field: &str, qualification: &str) -> String {
    format!(
        "Für das Fachgebiet „{field}“ gibt es keine Forscher (Arbeitskräftegruppe „{qualification}.{field}“ fehlt)."
    )
}

pub fn development_field_missing(product: &str, branch: &str) -> String {
    format!(
        "Produkt „{product}“ wird ohne Technologie hergestellt, und seine Branche „{branch}“ hat kein Fachgebiet für die Weiterentwicklung (forschungsmodell.weiterentwicklung.fachgebiete)."
    )
}
