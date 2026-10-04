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

pub fn extraction_needs_raw_material(product: &str) -> String {
    format!("Abbau-Rezepte („abbau: true“) erzeugen Rohstoffe; „{product}“ ist kein Rohstoff.")
}

pub fn recipe_consumes_own_product(product: &str) -> String {
    format!("Das Rezept verbraucht sein eigenes Produkt „{product}“.")
}

pub fn product_without_source(key: &str) -> String {
    format!(
        "Produkt „{key}“ kann weder hergestellt noch vom Staatsmarkt bezogen werden: Es fehlt ein Rezept oder ein Eintrag „staatsmarkt“."
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
