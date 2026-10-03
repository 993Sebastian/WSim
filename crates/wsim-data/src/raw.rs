//! Schema of the data files as written by hand (German field names).
//!
//! These structs mirror the files one to one; `build` checks them and turns them
//! into the catalog of `wsim-core`. Field reference: `docs/DATENFORMAT.md`.

use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawMeta {
    #[serde(rename = "datenversion")]
    pub data_version: u32,
}

/// Entries that only consist of an ID (continents, branches, goods groups, …).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSimple {
    pub id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawUnit {
    pub id: String,
    #[serde(rename = "gewicht_kg", default)]
    pub weight_kg: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawQualification {
    pub id: String,
    #[serde(rename = "stufe")]
    pub rank: u8,
    #[serde(rename = "mit_fachrichtung")]
    pub has_specialization: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawCountry {
    pub id: String,
    #[serde(rename = "kontinent")]
    pub continent: String,
    #[serde(rename = "werte")]
    pub values: RawCountryValues,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawCountryValues {
    #[serde(rename = "bevoelkerung")]
    pub population: BTreeMap<i32, f64>,
    #[serde(rename = "bip_je_kopf_usd")]
    pub gdp_per_capita_usd: BTreeMap<i32, f64>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub enum RawProductKind {
    #[serde(rename = "rohstoff")]
    RawMaterial,
    #[serde(rename = "halbzeug")]
    SemiFinished,
    #[serde(rename = "komponente")]
    Component,
    #[serde(rename = "endprodukt")]
    EndProduct,
    #[serde(rename = "energie")]
    Energy,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum RawUsage {
    #[serde(rename = "industrie")]
    Industry,
    #[serde(rename = "konsum")]
    Consumer,
    #[serde(rename = "beides")]
    Both,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProduct {
    pub id: String,
    #[serde(rename = "art")]
    pub kind: RawProductKind,
    #[serde(rename = "branche")]
    pub branch: String,
    #[serde(rename = "einheit")]
    pub unit: String,
    #[serde(rename = "verwendung")]
    pub usage: RawUsage,
    #[serde(rename = "warengruppe")]
    pub goods_group: String,
    #[serde(rename = "transportklasse")]
    pub transport_class: String,
    #[serde(rename = "gewicht_kg", default)]
    pub weight_kg: Option<f64>,
    #[serde(rename = "heizwert_mwh", default)]
    pub heating_value_mwh: Option<f64>,
    #[serde(rename = "nachfrage", default)]
    pub consumer_demand: Option<RawConsumerDemand>,
    #[serde(rename = "staatsnachfrage", default)]
    pub state_demand: Option<RawStateDemand>,
    #[serde(rename = "staatsmarkt", default)]
    pub state_market: Option<RawStateMarket>,
    #[serde(rename = "ersetzt", default)]
    pub replaces: Vec<String>,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum RawNeedClass {
    #[serde(rename = "grundbedarf")]
    Basic,
    #[serde(rename = "gebrauchsgut")]
    Durable,
    #[serde(rename = "luxus")]
    Luxury,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawConsumerDemand {
    #[serde(rename = "bedarfsklasse")]
    pub need_class: RawNeedClass,
    #[serde(rename = "verbrauch", default)]
    pub consumable: Option<RawConsumable>,
    #[serde(rename = "gebrauch", default)]
    pub durable: Option<RawDurable>,
    #[serde(rename = "einkommensschwelle_usd")]
    pub income_threshold_usd: f64,
    #[serde(rename = "preisempfindlichkeit")]
    pub price_sensitivity: f64,
    #[serde(rename = "einkommensempfindlichkeit")]
    pub income_sensitivity: f64,
    #[serde(rename = "saison", default)]
    pub seasonality: Option<Vec<f64>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawConsumable {
    #[serde(rename = "je_kopf_und_jahr")]
    pub per_capita_per_year: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawDurable {
    #[serde(rename = "nutzungsdauer_jahre")]
    pub service_life_years: f64,
    #[serde(rename = "max_besitzquote")]
    pub max_ownership: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawStateDemand {
    #[serde(rename = "je_mio_usd_bip")]
    pub per_million_gdp: f64,
    #[serde(rename = "kriegsfaktor", default = "one")]
    pub war_factor: f64,
}

fn one() -> f64 {
    1.0
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawStateMarket {
    #[serde(rename = "preis_usd")]
    pub price_usd: f64,
    #[serde(rename = "verfuegbar_ab", default)]
    pub available_from: Option<i32>,
    #[serde(rename = "verfuegbar_bis", default)]
    pub available_until: Option<i32>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum RawSiteType {
    #[serde(rename = "foerderstaette")]
    Extraction,
    #[serde(rename = "werk")]
    Factory,
    #[serde(rename = "kraftwerk")]
    PowerPlant,
    #[serde(rename = "lager")]
    Warehouse,
    #[serde(rename = "niederlassung")]
    SalesOffice,
    #[serde(rename = "forschungszentrum")]
    ResearchCenter,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFacility {
    pub id: String,
    #[serde(rename = "standorttyp")]
    pub site_type: RawSiteType,
    #[serde(rename = "investition_usd")]
    pub investment_usd: f64,
    #[serde(rename = "bauzeit_tage")]
    pub build_days: u32,
    #[serde(rename = "kapazitaet_je_tag")]
    pub runs_per_day: f64,
    #[serde(rename = "lebensdauer_jahre")]
    pub lifetime_years: u32,
    #[serde(rename = "wartung_je_jahr")]
    pub maintenance_share: f64,
    #[serde(rename = "automatisierung_max")]
    pub automation_max: f64,
    #[serde(rename = "technologie", default)]
    pub technology: Option<String>,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRecipe {
    pub id: String,
    #[serde(rename = "produkt")]
    pub product: String,
    #[serde(rename = "menge")]
    pub output: f64,
    #[serde(rename = "nebenprodukte", default)]
    pub by_products: BTreeMap<String, f64>,
    #[serde(rename = "dauer_tage")]
    pub duration_days: u32,
    #[serde(rename = "anlage")]
    pub facility: String,
    #[serde(rename = "technologie", default)]
    pub technology: Option<String>,
    #[serde(rename = "abbau", default)]
    pub extraction: bool,
    #[serde(rename = "eingang", default)]
    pub inputs: BTreeMap<String, f64>,
    #[serde(rename = "arbeit_stunden")]
    pub labor_hours: BTreeMap<String, f64>,
    #[serde(rename = "energie_mwh", default)]
    pub energy_mwh: f64,
    #[serde(rename = "qualitaet_basis")]
    pub base_quality: f64,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTechnology {
    pub id: String,
    #[serde(rename = "fachgebiet")]
    pub field: String,
    #[serde(rename = "erfindungsjahr")]
    pub invention_year: i32,
    #[serde(rename = "voraussetzungen", default)]
    pub prerequisites: Vec<String>,
    #[serde(rename = "forschungsaufwand", default)]
    pub research_effort: Option<f64>,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawDeposit {
    pub id: String,
    #[serde(rename = "land")]
    pub country: String,
    #[serde(rename = "rohstoff")]
    pub resource: String,
    #[serde(rename = "vorrat", default)]
    pub reserve: Option<f64>,
    #[serde(rename = "erneuerbar", default)]
    pub renewable: bool,
    #[serde(rename = "entdeckt", default)]
    pub discovered: Option<i32>,
    #[serde(rename = "erschliessung")]
    pub development: RawDevelopment,
    #[serde(rename = "foerderung_max_je_jahr")]
    pub max_output_per_year: f64,
    #[serde(rename = "foerderkosten_faktor", default = "one")]
    pub cost_factor: f64,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawDevelopment {
    #[serde(rename = "investition_usd")]
    pub investment_usd: f64,
    #[serde(rename = "dauer_tage")]
    pub days: u32,
}
