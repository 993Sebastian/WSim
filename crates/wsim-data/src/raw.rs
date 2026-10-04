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
    #[serde(rename = "flaeche_km2")]
    pub area_km2: f64,
    #[serde(rename = "hauptstadt")]
    pub capital: RawLocation,
    #[serde(rename = "binnenland")]
    pub landlocked: bool,
    #[serde(rename = "nachbarn", default)]
    pub neighbors: Vec<String>,
    #[serde(rename = "werte")]
    pub values: RawCountryValues,
    #[serde(rename = "praegung", default)]
    pub profile: Option<RawProfile>,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLocation {
    #[serde(rename = "breite")]
    pub lat: f64,
    #[serde(rename = "laenge")]
    pub lon: f64,
}

pub type RawSeries = BTreeMap<i32, f64>;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawCountryValues {
    #[serde(rename = "bevoelkerung")]
    pub population: RawSeries,
    #[serde(rename = "bip_je_kopf_usd")]
    pub gdp_per_capita_usd: RawSeries,
    pub gini: RawSeries,
    #[serde(rename = "stabilitaet", default)]
    pub stability: Option<RawSeries>,
    #[serde(rename = "steuer_unternehmen", default)]
    pub corporate_tax: Option<RawSeries>,
    #[serde(rename = "steuer_dividenden", default)]
    pub dividend_tax: Option<RawSeries>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProfile {
    #[serde(rename = "automatisierung", default)]
    pub automation: f64,
    #[serde(rename = "fachrichtungen", default)]
    pub specializations: BTreeMap<String, f64>,
    #[serde(rename = "forschung", default)]
    pub research: BTreeMap<String, f64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawCountryModel {
    #[serde(rename = "preisniveau")]
    pub price_level: RawPriceLevel,
    #[serde(rename = "erwerbsquote")]
    pub participation_rate: f64,
    #[serde(rename = "lohnquote")]
    pub labor_share: f64,
    #[serde(rename = "jahresarbeitsstunden")]
    pub annual_hours: RawSeries,
    #[serde(rename = "qualifikationsanteile")]
    pub qualification_shares: Vec<RawShareRow>,
    #[serde(rename = "lohnabstand")]
    pub wage_factors: Vec<RawFactorRow>,
    #[serde(rename = "fachrichtungsanteile")]
    pub specialization_shares: BTreeMap<String, BTreeMap<String, f64>>,
    #[serde(rename = "strompreis_usd_je_mwh")]
    pub electricity_price: RawSeries,
    #[serde(rename = "stromnetz")]
    pub grid_reach: RawSeries,
    #[serde(rename = "stromnetz_bezug_usd")]
    pub grid_reference_usd: f64,
    #[serde(rename = "steuer_unternehmen")]
    pub corporate_tax: RawSeries,
    #[serde(rename = "steuer_dividenden")]
    pub dividend_tax: RawSeries,
    #[serde(rename = "entwicklung")]
    pub development: RawRange,
    #[serde(rename = "verkehrstraeger")]
    pub transport: RawTransportAvailability,
    #[serde(rename = "stabilitaet")]
    pub stability: f64,
    #[serde(rename = "forschung")]
    pub research: RawResearch,
    #[serde(rename = "automatisierung")]
    pub automation: RawAutomation,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPriceLevel {
    #[serde(rename = "referenzland")]
    pub reference: String,
    #[serde(rename = "elastizitaet")]
    pub elasticity: f64,
    #[serde(rename = "minimum")]
    pub min: f64,
    #[serde(rename = "maximum")]
    pub max: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawShareRow {
    #[serde(rename = "bip_je_kopf_usd")]
    pub gdp: f64,
    #[serde(rename = "anteile")]
    pub shares: BTreeMap<String, f64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFactorRow {
    #[serde(rename = "bip_je_kopf_usd")]
    pub gdp: f64,
    #[serde(rename = "faktoren")]
    pub factors: BTreeMap<String, f64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRange {
    #[serde(rename = "von_usd")]
    pub from: f64,
    #[serde(rename = "bis_usd")]
    pub to: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTransportAvailability {
    #[serde(rename = "schiene")]
    pub rail: RawSeries,
    #[serde(rename = "strasse")]
    pub road: RawSeries,
    #[serde(rename = "luft")]
    pub air: RawSeries,
    #[serde(rename = "hafen")]
    pub port: RawSeries,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawResearch {
    #[serde(rename = "bezug_usd")]
    pub reference: f64,
    #[serde(rename = "elastizitaet")]
    pub elasticity: f64,
    #[serde(rename = "minimum")]
    pub min: f64,
    #[serde(rename = "maximum")]
    pub max: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAutomation {
    #[serde(rename = "basis")]
    pub base: f64,
    #[serde(rename = "je_verdopplung")]
    pub per_doubling: f64,
    #[serde(rename = "bezug_usd")]
    pub reference: f64,
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
    #[serde(rename = "richtpreis_usd")]
    pub reference_price_usd: f64,
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
    #[serde(rename = "ergaenzung", default)]
    pub complement: Option<RawComplement>,
    #[serde(rename = "kaufschwelle")]
    pub purchase_threshold: f64,
    #[serde(rename = "preisempfindlichkeit")]
    pub price_sensitivity: f64,
    #[serde(rename = "einkommensempfindlichkeit")]
    pub income_sensitivity: f64,
    #[serde(rename = "saison", default)]
    pub seasonality: Option<Vec<f64>>,
    #[serde(rename = "netzabhaengig", default)]
    pub needs_grid: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawComplement {
    #[serde(rename = "zu")]
    pub of: String,
    #[serde(rename = "je_besitz_und_jahr")]
    pub per_unit_per_year: f64,
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProductionModel {
    #[serde(rename = "standortkosten_usd")]
    pub site_cost: BTreeMap<String, f64>,
    #[serde(rename = "gebaeude_lebensdauer_jahre")]
    pub building_lifetime_years: f64,
    #[serde(rename = "erschliessung_lebensdauer_jahre")]
    pub development_lifetime_years: f64,
    #[serde(rename = "automatisierung")]
    pub automation: RawAutomationCost,
    #[serde(rename = "qualitaet")]
    pub quality: RawQuality,
    #[serde(rename = "zustand_minimum")]
    pub condition_min: f64,
    #[serde(rename = "strom", default)]
    pub electricity: Option<String>,
    #[serde(rename = "einspeiseverguetung")]
    pub feed_in_share: f64,
    #[serde(rename = "startformen")]
    pub start_setups: BTreeMap<String, RawStartSetup>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawStartSetup {
    #[serde(rename = "standorttyp")]
    pub site_type: RawSiteType,
    #[serde(rename = "gebaeude_usd")]
    pub building_usd: f64,
    #[serde(rename = "anlagen", default)]
    pub facilities: Vec<RawStartFacility>,
    #[serde(rename = "einkauf", default)]
    pub purchases: Vec<RawStartPurchase>,
    #[serde(rename = "verkauf", default)]
    pub sales: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawStartFacility {
    #[serde(rename = "anlage")]
    pub facility: String,
    #[serde(rename = "rezept", default)]
    pub recipe: Option<String>,
    #[serde(rename = "auslastung", default = "one")]
    pub utilization: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawStartPurchase {
    #[serde(rename = "produkt")]
    pub product: String,
    #[serde(rename = "ziel")]
    pub target: f64,
    #[serde(rename = "hoechstpreis_usd")]
    pub max_price_usd: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAutomationCost {
    #[serde(rename = "arbeitsersparnis")]
    pub labor_saving: f64,
    #[serde(rename = "kostenanteil")]
    pub cost_share: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawQuality {
    #[serde(rename = "vorprodukte")]
    pub inputs: f64,
    #[serde(rename = "automatisierung")]
    pub automation: f64,
    #[serde(rename = "zustand")]
    pub condition: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFinanceModel {
    #[serde(rename = "realzins")]
    pub real_rate: BTreeMap<i32, f64>,
    #[serde(rename = "risikoaufschlag")]
    pub premium: RawPremium,
    #[serde(rename = "beleihung")]
    pub loan_to_value: f64,
    #[serde(rename = "dispo")]
    pub overdraft: RawOverdraft,
    #[serde(rename = "laufzeit_max_jahre")]
    pub max_term_years: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPremium {
    #[serde(rename = "minimum")]
    pub min: f64,
    #[serde(rename = "je_verschuldung")]
    pub per_debt_ratio: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawOverdraft {
    #[serde(rename = "anteil")]
    pub share: f64,
    #[serde(rename = "aufschlag")]
    pub premium: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawMarketModel {
    #[serde(rename = "preisgewicht")]
    pub price_weight: Vec<f64>,
    #[serde(rename = "qualitaetsgewicht")]
    pub quality_weight: Vec<f64>,
    #[serde(rename = "aneignung_je_jahr")]
    pub adoption_per_year: f64,
    #[serde(rename = "preisanpassung")]
    pub price_adjustment: RawPriceAdjustment,
    #[serde(rename = "staat_hoechstpreis")]
    pub state_price_cap: f64,
    #[serde(rename = "index_glaettung")]
    pub index_smoothing: f64,
    #[serde(rename = "haendler")]
    pub traders: RawTraders,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPriceAdjustment {
    #[serde(rename = "hoch")]
    pub up: f64,
    #[serde(rename = "runter")]
    pub down: f64,
    #[serde(rename = "lagertage")]
    pub stock_days: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTransportClass {
    pub id: String,
    #[serde(rename = "kostenfaktor", default = "one")]
    pub cost_factor: f64,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum RawWay {
    #[serde(rename = "gelaende")]
    Terrain,
    #[serde(rename = "strasse")]
    Road,
    #[serde(rename = "schiene")]
    Rail,
    #[serde(rename = "see")]
    Sea,
    #[serde(rename = "luft")]
    Air,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawVehicle {
    pub id: String,
    #[serde(rename = "weg")]
    pub way: RawWay,
    #[serde(rename = "verfuegbar_ab")]
    pub available_from: i32,
    #[serde(rename = "verfuegbar_bis", default)]
    pub available_until: Option<i32>,
    #[serde(rename = "transportklassen")]
    pub classes: Vec<String>,
    #[serde(rename = "kosten_usd_je_tkm")]
    pub cost_per_tkm: RawSeries,
    #[serde(rename = "km_je_tag")]
    pub km_per_day: RawSeries,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTransportModel {
    #[serde(rename = "umweg")]
    pub detour: RawDetour,
    #[serde(rename = "umschlag")]
    pub handling: RawHandling,
    #[serde(rename = "mindestinfrastruktur")]
    pub min_infrastructure: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawDetour {
    pub land: f64,
    pub see: f64,
    pub luft: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawHandling {
    #[serde(rename = "kosten_usd_je_t")]
    pub cost_usd: f64,
    #[serde(rename = "tage")]
    pub days: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTraders {
    #[serde(rename = "marge")]
    pub margin: f64,
    #[serde(rename = "vorrat_tage")]
    pub cover_days: f64,
    #[serde(rename = "glaettung_tage")]
    pub smoothing_days: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawResearchModel {
    #[serde(rename = "vorgriff_faktor")]
    pub ahead_base: f64,
    #[serde(rename = "nachzuegler")]
    pub latecomer: RawLatecomer,
    #[serde(rename = "gemeingut_nach_jahren")]
    pub public_domain_years: u32,
    #[serde(rename = "forscher")]
    pub researchers: String,
    #[serde(rename = "sachkosten_usd_je_forschertag")]
    pub material_usd_per_day: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLatecomer {
    #[serde(rename = "rabatt_je_jahr")]
    pub discount: f64,
    #[serde(rename = "minimum")]
    pub min: f64,
}
