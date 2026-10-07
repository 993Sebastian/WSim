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
    #[serde(rename = "umfasst", default)]
    pub members: Vec<String>,
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
    #[serde(rename = "produktivitaet")]
    pub productivity: RawResearch,
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
    #[serde(rename = "foerderindex", default)]
    pub output_index: Option<RawSeries>,
    #[serde(rename = "pacht_anteil", default)]
    pub rent_share: Option<f64>,
    /// May use five or six levels in its product tree (Lastenheft §17.2).
    #[serde(rename = "sehr_komplex", default)]
    pub very_complex: bool,
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
    #[serde(rename = "verlauf", default)]
    pub index: Option<RawSeries>,
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
    #[serde(rename = "flaeche_ha", default)]
    pub area_ha: Option<f64>,
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
    #[serde(rename = "gemeinkosten_anteil")]
    pub overhead_share: RawPerKind,
    #[serde(rename = "richtpreis_marge")]
    pub reference_margin: RawLimits,
    #[serde(rename = "nebenprodukte_lager_tage")]
    pub by_product_stock_days: f64,
    #[serde(rename = "lohnaufschlag_max")]
    pub wage_premium_max: f64,
    #[serde(rename = "stilllegung")]
    pub mothballing: RawMothballing,
    #[serde(rename = "verkauf")]
    pub facility_sale: RawFacilitySale,
    #[serde(rename = "startformen")]
    pub start_setups: BTreeMap<String, RawStartSetup>,
    #[serde(rename = "anlagengroessen")]
    pub sizes: RawFacilitySizes,
}

/// Facility sizes (M36): capacity per size and the powers of the capacity.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFacilitySizes {
    #[serde(rename = "kapazitaet")]
    pub capacity: BTreeMap<String, f64>,
    #[serde(rename = "investition_exponent")]
    pub investment_exponent: f64,
    #[serde(rename = "arbeit_exponent")]
    pub labor_exponent: f64,
    #[serde(rename = "flaeche_exponent")]
    pub area_exponent: f64,
    #[serde(rename = "bauzeit_exponent")]
    pub build_exponent: f64,
}

/// Shut down facilities (M22).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawMothballing {
    #[serde(rename = "instandhaltung_anteil")]
    pub maintenance_share: f64,
    #[serde(rename = "wiederanlauf_tage")]
    pub restart_days: u32,
    #[serde(rename = "wiederanlauf_kosten")]
    pub restart_cost_share: f64,
}

/// Sold facilities (M22).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFacilitySale {
    #[serde(rename = "erloes_anteil")]
    pub proceeds_share: f64,
    #[serde(rename = "schrottwert")]
    pub scrap_share: f64,
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
    #[serde(rename = "verdraengung_staat_jahre")]
    pub state_displacement_years: f64,
    #[serde(rename = "verlauf_monate")]
    pub history_months: u32,
    #[serde(rename = "meldung_preissenkung")]
    pub price_cut_report: f64,
    #[serde(rename = "preisniveau_anteil")]
    pub price_level_share: RawPerKind,
    #[serde(rename = "index_glaettung")]
    pub index_smoothing: f64,
    #[serde(rename = "haendler")]
    pub traders: RawTraders,
    #[serde(rename = "marke")]
    pub brand: RawBrandModel,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawBrandModel {
    #[serde(rename = "markengewicht")]
    pub weight: Vec<f64>,
    #[serde(rename = "vergessen_je_monat")]
    pub forgetting_per_month: f64,
    #[serde(rename = "mundpropaganda")]
    pub word_of_mouth: f64,
    #[serde(rename = "kosten_je_einwohner_usd")]
    pub cost_per_inhabitant_usd: f64,
    #[serde(rename = "bekanntheit_start")]
    pub start_awareness: f64,
    #[serde(rename = "bekanntheit_start_real")]
    pub start_awareness_real: f64,
    #[serde(rename = "bekanntheit_handel")]
    pub trade_awareness: f64,
    #[serde(rename = "bekanntheit_staatsmarkt")]
    pub state_market_awareness: f64,
    #[serde(rename = "werbemittel")]
    pub media: Vec<RawAdvertisingMedium>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAdvertisingMedium {
    pub id: String,
    #[serde(rename = "ab")]
    pub from_year: i32,
    #[serde(rename = "wirkung")]
    pub effect: f64,
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
    #[serde(rename = "auslastung_normal")]
    pub normal_utilization: f64,
    #[serde(rename = "hoechstfaktor")]
    pub max_factor: f64,
    #[serde(rename = "aufholen_max")]
    pub catch_up_max: f64,
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
    #[serde(rename = "weiterentwicklung")]
    pub development: RawProductDevelopment,
}

/// Development of researched products (M37).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProductDevelopment {
    #[serde(rename = "stufen")]
    pub levels: u32,
    #[serde(rename = "je_stufe")]
    pub per_level: RawDevelopmentPerLevel,
    #[serde(rename = "aufwand")]
    pub effort: RawDevelopmentEffort,
    #[serde(rename = "gemeingut_nach_jahren")]
    pub public_domain_years: u32,
    /// Research field by branch, for products without a technology.
    #[serde(rename = "fachgebiete")]
    pub fields: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawDevelopmentPerLevel {
    #[serde(rename = "qualitaet")]
    pub quality: f64,
    #[serde(rename = "arbeit")]
    pub labor: f64,
    #[serde(rename = "vorprodukte")]
    pub inputs: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawDevelopmentEffort {
    #[serde(rename = "anteil")]
    pub share: f64,
    #[serde(rename = "wachstum")]
    pub growth: f64,
    #[serde(rename = "grundaufwand")]
    pub base: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLatecomer {
    #[serde(rename = "rabatt_je_jahr")]
    pub discount: f64,
    #[serde(rename = "minimum")]
    pub min: f64,
}

/// `data/parameter/kimodell.yaml`
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAiModel {
    #[serde(rename = "firmen_standard")]
    pub default_companies: u32,
    #[serde(rename = "firmen_max")]
    pub max_companies: u32,
    #[serde(rename = "firmen_bei_realer_groesse")]
    pub companies_for_real_size: f64,
    #[serde(rename = "massstab")]
    pub scale: RawLimits,
    #[serde(rename = "arbeitskraefte_min")]
    pub min_labor_pool: f64,
    #[serde(rename = "anlagen_je_konzession")]
    pub plants_per_concession: f64,
    #[serde(rename = "konzessionen_max")]
    pub max_concessions: u32,
    #[serde(rename = "schwierigkeiten")]
    pub difficulties: Vec<RawDifficulty>,
    #[serde(rename = "schwierigkeit_standard")]
    pub default_difficulty: String,
    #[serde(rename = "streuung")]
    pub trait_spread: f64,
    pub start: RawAiStart,
    #[serde(rename = "verhalten")]
    pub behavior: RawAiBehavior,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLimits {
    #[serde(rename = "minimum")]
    pub min: f64,
    #[serde(rename = "maximum")]
    pub max: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawDifficulty {
    pub id: String,
    #[serde(rename = "kompetenz")]
    pub competence: f64,
    #[serde(rename = "aggressivitaet")]
    pub aggressiveness: f64,
}

/// Plots of land (`parameter/grundstuecksmodell.yaml`, M35).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPlotModel {
    #[serde(rename = "flaeche_ha_je_mrd_bip")]
    pub area_per_gdp_bn_ha: f64,
    #[serde(rename = "wachstum")]
    pub growth: RawPlotGrowth,
    #[serde(rename = "wohlstand")]
    pub wealth: RawWealth,
    #[serde(rename = "klassen")]
    pub classes: Vec<RawPlotClass>,
    #[serde(rename = "lagen")]
    pub locations: RawPlotLocations,
    #[serde(rename = "bodenpreis_usd_je_ha")]
    pub land_price_usd_per_ha: f64,
    #[serde(rename = "knappheit")]
    pub scarcity: f64,
    #[serde(rename = "pacht_anteil")]
    pub rent_share: f64,
    #[serde(rename = "anlagenflaeche")]
    pub facility_area: RawFacilityArea,
    #[serde(rename = "ki_reserve")]
    pub ai_reserve: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPlotGrowth {
    #[serde(rename = "ab_jahr")]
    pub from_year: i32,
    #[serde(rename = "jahre")]
    pub years: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawWealth {
    #[serde(rename = "reich_ab_usd")]
    pub rich_from_usd: f64,
    #[serde(rename = "arm_unter_usd")]
    pub poor_below_usd: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPlotClass {
    pub id: String,
    #[serde(rename = "flaeche_ha")]
    pub area_ha: RawAreaRange,
    #[serde(rename = "anteile")]
    pub shares: RawWealthShares,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAreaRange {
    #[serde(rename = "von")]
    pub from: f64,
    #[serde(rename = "bis")]
    pub to: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawWealthShares {
    #[serde(rename = "reich")]
    pub rich: f64,
    #[serde(rename = "mittel")]
    pub middle: f64,
    #[serde(rename = "arm")]
    pub poor: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPlotLocations {
    pub stadt: RawPlotLocation,
    pub hafen: RawPlotLocation,
    pub land: RawPlotLocation,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPlotLocation {
    #[serde(rename = "anteil")]
    pub share: f64,
    #[serde(rename = "flaeche")]
    pub area_factor: f64,
    #[serde(rename = "bodenpreis")]
    pub price_factor: f64,
    #[serde(rename = "anwerben")]
    pub hiring: f64,
    #[serde(rename = "fracht_see")]
    pub sea_freight: f64,
    #[serde(rename = "lieferkosten")]
    pub delivery_cost: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFacilityArea {
    #[serde(rename = "investition_je_ha_usd")]
    pub investment_per_ha_usd: f64,
    #[serde(rename = "zuschlag")]
    pub overhead: f64,
    #[serde(rename = "mindestflaeche_ha")]
    pub min_site_area_ha: f64,
}

/// Offers between companies (`parameter/kaufmodell.yaml`, M30).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawDealModel {
    #[serde(rename = "gueltig_monate")]
    pub valid_months: u32,
    #[serde(rename = "sperre_monate")]
    pub block_months: u32,
    #[serde(rename = "mindestalter_monate")]
    pub min_age_months: u32,
    #[serde(rename = "ertragsfaktor")]
    pub earnings_years: f64,
    #[serde(rename = "ertrag_mindestmonate")]
    pub earnings_min_months: u32,
    #[serde(rename = "firmenwert_jahre")]
    pub goodwill_years: f64,
    #[serde(rename = "qualifiziert_ab_stufe")]
    pub qualified_rank: u8,
    pub ki: RawDealAi,
    #[serde(rename = "insolvenz")]
    pub insolvency: RawInsolvency,
}

/// Auction of the sites of an insolvent company (M38).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawInsolvency {
    #[serde(rename = "tage")]
    pub days: u32,
    #[serde(rename = "mindestpreis")]
    pub min_share: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawDealAi {
    #[serde(rename = "angebot_chance")]
    pub chance: RawSpan,
    #[serde(rename = "offene_angebote_max")]
    pub open_max: u32,
    #[serde(rename = "spieler_angebote_je_monat")]
    pub player_offers_per_month: u32,
    #[serde(rename = "wettbewerb_aufschlag")]
    pub competition_markup: RawSpan,
    #[serde(rename = "fachkraefte_aufschlag")]
    pub staff_markup: f64,
    #[serde(rename = "bauzeit_aufschlag")]
    pub build_time_markup: f64,
    #[serde(rename = "neubau_anteil")]
    pub new_build_share: f64,
    #[serde(rename = "mindestvorteil")]
    pub min_advantage: f64,
    #[serde(rename = "gebotsaufschlag")]
    pub bid_markup: RawSpan,
    #[serde(rename = "mindestpreis_usd")]
    pub min_price_usd: f64,
    #[serde(rename = "kasse_anteil_max")]
    pub cash_share_max: f64,
    #[serde(rename = "lizenz_gebot")]
    pub license_bid: RawSpan,
    #[serde(rename = "lizenz_hoechst")]
    pub license_max: f64,
    #[serde(rename = "verkaufsaufschlag")]
    pub sale_markup: RawSpan,
    #[serde(rename = "kern_anteil")]
    pub core_share: f64,
    #[serde(rename = "kern_aufschlag")]
    pub core_markup: f64,
    #[serde(rename = "lizenz_mindest")]
    pub license_min: f64,
    #[serde(rename = "wettbewerb_lizenz")]
    pub license_competition: f64,
    #[serde(rename = "gegen_schwelle")]
    pub counter_threshold: f64,
}

/// A value depending on a company trait: at 0 and at 1.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSpan {
    #[serde(rename = "bei_0")]
    pub at_0: f64,
    #[serde(rename = "bei_1")]
    pub at_1: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAiStart {
    #[serde(rename = "auslastung")]
    pub utilization: f64,
    #[serde(rename = "anlage_mindestanteil")]
    pub min_plant_share: f64,
    #[serde(rename = "lager_eingang_tage")]
    pub input_stock_days: f64,
    #[serde(rename = "lager_ausgang_tage")]
    pub output_stock_days: f64,
    #[serde(rename = "kasse_monate")]
    pub cash_months: f64,
    #[serde(rename = "gewicht_entwicklung")]
    pub development_weight: RawPerKind,
    #[serde(rename = "referenzlohn_usd")]
    pub reference_wage_usd: f64,
    #[serde(rename = "marktdeckung")]
    pub market_cover: RawPerKind,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPerKind {
    #[serde(rename = "rohstoff")]
    pub raw_material: f64,
    #[serde(rename = "halbzeug")]
    pub semi_finished: f64,
    #[serde(rename = "komponente")]
    pub component: f64,
    #[serde(rename = "endprodukt")]
    pub end_product: f64,
    #[serde(rename = "energie")]
    pub energy: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAiBehavior {
    #[serde(rename = "betrieb_alle_tage")]
    pub operations_days: RawSpan,
    #[serde(rename = "lager_hoch_tage")]
    pub stock_high_days: f64,
    #[serde(rename = "lager_niedrig_tage")]
    pub stock_low_days: f64,
    #[serde(rename = "auslastung_schritt")]
    pub utilization_step: f64,
    #[serde(rename = "auslastung_min")]
    pub utilization_min: f64,
    #[serde(rename = "auslastung_aenderung_max")]
    pub utilization_change_max: f64,
    #[serde(rename = "lager_ziel_tage")]
    pub stock_target_days: f64,
    #[serde(rename = "lager_ausgleich_tage")]
    pub stock_adjust_days: f64,
    #[serde(rename = "preisuntergrenze")]
    pub floor_factor: RawSpan,
    #[serde(rename = "werbeanteil")]
    pub advertising_share: RawSpan,
    #[serde(rename = "einkauf_aufschlag")]
    pub purchase_markup: f64,
    #[serde(rename = "ausbau_auslastung")]
    pub expand_utilization: RawSpan,
    #[serde(rename = "ausbau_marge")]
    pub expand_margin: RawSpan,
    #[serde(rename = "ausbau_vorprodukt_preis_max")]
    pub expand_input_price_max: f64,
    #[serde(rename = "ausbau_anteil_kasse_max")]
    pub invest_share_max: f64,
    #[serde(rename = "lohnaufschlag_schritt")]
    pub wage_premium_step: f64,
    #[serde(rename = "lohnaufschlag_max")]
    pub wage_premium_max: f64,
    #[serde(rename = "forschung_vorgriff_jahre")]
    pub research_lookahead_years: RawSpan,
    #[serde(rename = "forschung_mindestumsatz_usd")]
    pub research_min_revenue_usd: f64,
    #[serde(rename = "forschung_mindestkompetenz")]
    pub research_competence_min: f64,
    #[serde(rename = "entwicklung_nutzen_je_stufe")]
    pub development_benefit_per_level: f64,
    #[serde(rename = "entwicklung_amortisation_jahre")]
    pub development_payback_years: f64,
    #[serde(rename = "forschung_luecke_firmen")]
    pub research_gap_companies: u32,
    #[serde(rename = "forschung_vorlauf_jahre", default)]
    pub research_lead_years: u32,
    #[serde(rename = "kasse_min_monate")]
    pub cash_min_months: f64,
    #[serde(rename = "kasse_max_monate")]
    pub cash_max_months: f64,
    #[serde(rename = "kredit_jahre")]
    pub loan_years: u32,
    #[serde(rename = "gruendungen_je_monat")]
    pub foundings_per_month: u32,
    #[serde(rename = "diversifikationen_je_quartal")]
    pub diversifications_per_quarter: u32,
    #[serde(rename = "einstieg_preisfaktor")]
    pub entry_price_factor: f64,
    #[serde(rename = "einstieg_firmen_max")]
    pub entry_companies_max: u32,
    #[serde(rename = "einstieg_anteil")]
    pub entry_share: f64,
    #[serde(rename = "vorrat_jahre_min")]
    pub reserve_years_min: f64,
    #[serde(rename = "gruendung_kapitalfaktor")]
    pub founding_capital_factor: f64,
    #[serde(rename = "stilllegen_auslastung")]
    pub mothball_utilization: f64,
    #[serde(rename = "stilllegen_zielauslastung")]
    pub mothball_target_utilization: f64,
    #[serde(rename = "stilllegen_preis_max")]
    pub mothball_price_max: f64,
    #[serde(rename = "wiederanfahren_auslastung")]
    pub restart_utilization: f64,
    #[serde(rename = "verkaufen_nach_monaten")]
    pub sell_after_months: u32,
}

/// Name parts for generated companies (`data/ki/namen.yaml`).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawNameGroup {
    pub id: String,
    #[serde(rename = "laender", default)]
    pub countries: Vec<String>,
    #[serde(rename = "standard", default)]
    pub is_default: bool,
    #[serde(rename = "familiennamen")]
    pub surnames: Vec<String>,
    #[serde(rename = "vornamen")]
    pub first_names: Vec<String>,
    /// Managers' names with the family name first (East Asia).
    #[serde(rename = "familienname_zuerst", default)]
    pub surname_first: bool,
    #[serde(rename = "orte")]
    pub places: Vec<String>,
    #[serde(rename = "rechtsformen")]
    pub legal_forms: Vec<String>,
    #[serde(rename = "muster")]
    pub patterns: Vec<String>,
    #[serde(rename = "branchen")]
    pub branch_words: BTreeMap<String, String>,
}

/// Positions, managers and their market (`data/parameter/management.yaml`, MA1).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawManagement {
    #[serde(rename = "bereiche")]
    pub functions: Vec<RawManagementFunction>,
    #[serde(rename = "ebenen")]
    pub levels: Vec<RawManagementLevel>,
    #[serde(rename = "standorttypen")]
    pub site_types: BTreeMap<String, Vec<String>>,
    #[serde(rename = "routine_themen", default)]
    pub routine_topics: Vec<String>,
    #[serde(rename = "budget_sockel_gehaelter")]
    pub budget_floor: RawBudgetFloor,
    #[serde(rename = "anliegen")]
    pub concerns: RawConcerns,
    #[serde(rename = "leitung_ohne_fach_abschlag")]
    pub head_discount: f64,
    #[serde(rename = "bemerken_grund")]
    pub notice_base: f64,
    #[serde(rename = "gehalt_lohngruppe")]
    pub salary_group: String,
    #[serde(rename = "abfindung_monate")]
    pub severance_months: f64,
    pub pool: RawManagerPool,
    #[serde(rename = "faehigkeiten")]
    pub skills: RawSkills,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawManagementFunction {
    pub id: String,
    #[serde(rename = "themen")]
    pub topics: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawManagementLevel {
    pub id: String,
    #[serde(rename = "pruefung_tage")]
    pub check_days: u32,
    #[serde(rename = "gehalt_fach")]
    pub salary_specialist: f64,
    #[serde(rename = "gehalt_leitung")]
    pub salary_head: f64,
    /// Share of the unit's revenue per decision and per year.
    #[serde(rename = "budget_fach")]
    pub budget_specialist: [f64; 2],
    #[serde(rename = "budget_leitung")]
    pub budget_head: [f64; 2],
    /// Specialist functions of a country or continent (MA3).
    #[serde(rename = "fachstellen", default)]
    pub specialists: Vec<String>,
    /// Topics the level's positions take up themselves (MA3).
    #[serde(rename = "themen", default)]
    pub topics: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawBudgetFloor {
    #[serde(rename = "entscheidung")]
    pub decision: f64,
    #[serde(rename = "jahr")]
    pub year: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawConcerns {
    #[serde(rename = "frist_tage")]
    pub deadline_days: u32,
    #[serde(rename = "sperre_tage")]
    pub block_days: u32,
    #[serde(rename = "offen_je_stelle")]
    pub open_per_position: u32,
    #[serde(rename = "wirkzeit_tage")]
    pub followup_days: u32,
    #[serde(rename = "schaetzfehler")]
    pub estimate_error: f64,
    #[serde(rename = "empfehlung_grund")]
    pub recommend_base: f64,
    #[serde(rename = "buendel_ab")]
    pub bundle_from: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawManagerPool {
    #[serde(rename = "je_mio_akademiker")]
    pub per_million_academics: f64,
    pub min: u32,
    pub max: u32,
    #[serde(rename = "abgang_monat")]
    pub leave_per_month: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSkills {
    #[serde(rename = "schwerpunkt")]
    pub focus: RawSpread,
    #[serde(rename = "sonst")]
    pub other: RawSpread,
    #[serde(rename = "allgemein")]
    pub general: RawSpread,
    #[serde(rename = "eindruck_unschaerfe")]
    pub impression_blur: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSpread {
    #[serde(rename = "mittel")]
    pub mean: f64,
    #[serde(rename = "streuung")]
    pub spread: f64,
}

/// Name parts for the product names of companies (`data/ki/produktnamen.yaml`, M42).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProductNaming {
    #[serde(rename = "hausmarke")]
    pub house_brand: f64,
    #[serde(rename = "ausgeschlossen", default)]
    pub excluded: Vec<String>,
    #[serde(rename = "stile")]
    pub styles: Vec<RawNamingStyle>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawNamingStyle {
    pub id: String,
    #[serde(rename = "warengruppen")]
    pub goods_groups: Vec<String>,
    #[serde(rename = "staemme")]
    pub stems: Vec<String>,
    #[serde(rename = "muster")]
    pub patterns: Vec<RawNamePattern>,
    #[serde(rename = "zahlen", default)]
    pub numbers: Vec<u32>,
    #[serde(rename = "buchstaben", default)]
    pub letters: Vec<String>,
    #[serde(rename = "zusaetze", default)]
    pub additions: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawNamePattern {
    pub text: String,
    #[serde(default)]
    pub ab: Option<i32>,
    #[serde(default)]
    pub bis: Option<i32>,
}

/// A historical company (`data/ki/reale_firmen.yaml`).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRealCompany {
    pub id: String,
    pub name: String,
    #[serde(rename = "sitz")]
    pub headquarters: String,
    #[serde(rename = "gegruendet")]
    pub founded: i32,
    #[serde(rename = "standorte")]
    pub sites: Vec<RawRealSite>,
    #[serde(rename = "kompetenz", default)]
    pub competence: Option<f64>,
    #[serde(rename = "aggressivitaet", default)]
    pub aggressiveness: Option<f64>,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRealSite {
    #[serde(rename = "land")]
    pub country: String,
    #[serde(rename = "lagerstaette", default)]
    pub deposit: Option<String>,
    #[serde(rename = "anlagen")]
    pub facilities: Vec<RawRealFacility>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRealFacility {
    #[serde(rename = "anlage")]
    pub facility: String,
    #[serde(rename = "anzahl")]
    pub count: f64,
    #[serde(rename = "rezept", default)]
    pub recipe: Option<String>,
}

/// A historical event (`data/ereignisse/`).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawEvent {
    pub id: String,
    /// `JJJJ-MM-TT`
    #[serde(rename = "datum")]
    pub date: String,
    #[serde(rename = "art")]
    pub kind: String,
    #[serde(rename = "laender", default)]
    pub countries: Vec<String>,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

/// A goal for the player after the introduction (M23).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawMilestone {
    pub id: String,
    #[serde(rename = "art")]
    pub kind: RawMilestoneKind,
    /// Count, share or factor, depending on the kind.
    #[serde(rename = "wert", default)]
    pub value: Option<f64>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub enum RawMilestoneKind {
    #[serde(rename = "erster_verkauf")]
    FirstSale,
    #[serde(rename = "gewinnmonat")]
    ProfitMonth,
    #[serde(rename = "anlagen")]
    Facilities,
    #[serde(rename = "eigenes_vorprodukt")]
    OwnInput,
    #[serde(rename = "laender")]
    Countries,
    #[serde(rename = "forschung")]
    Research,
    #[serde(rename = "marktfuehrer")]
    MarketLeader,
    #[serde(rename = "eigenkapital")]
    Equity,
}

/// A point in time of the currency data: a year (`1924`) or a year and month
/// (`"1923-11"`). Checked and converted in `build`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RawPointInTime(pub String);

impl<'de> Deserialize<'de> for RawPointInTime {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl serde::de::Visitor<'_> for Visitor {
            type Value = RawPointInTime;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("ein Jahr (1924) oder Jahr und Monat (\"1923-11\")")
            }

            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(RawPointInTime(v.to_string()))
            }

            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(RawPointInTime(v.to_string()))
            }

            fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Self::Value, E> {
                Ok(RawPointInTime(v.to_string()))
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(RawPointInTime(v.to_owned()))
            }
        }
        d.deserialize_any(Visitor)
    }
}

/// US consumer prices (M21): turn the game's dollars into money of the time.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPriceIndex {
    /// The currency whose prices these are; the unit of the simulation.
    #[serde(rename = "leitwaehrung")]
    pub lead_currency: String,
    /// Year whose purchasing power the game's dollars have.
    #[serde(rename = "basisjahr")]
    pub base_year: i32,
    /// Annual averages.
    #[serde(rename = "werte")]
    pub values: BTreeMap<i32, f64>,
    #[serde(rename = "teuerung_danach")]
    pub inflation_after: f64,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawCurrency {
    pub id: String,
    #[serde(rename = "zeichen")]
    pub symbol: String,
    /// Units per US dollar of the time: at a year (annual average) or a month.
    #[serde(rename = "kurse", default)]
    pub rates: Option<BTreeMap<RawPointInTime, f64>>,
    /// Instead of rates: fixed to another currency.
    #[serde(rename = "bindung", default)]
    pub peg: Option<RawPeg>,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPeg {
    #[serde(rename = "an")]
    pub currency: String,
    /// Units of this currency per unit of the other.
    #[serde(rename = "faktor")]
    pub factor: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawCountryCurrencies {
    #[serde(rename = "land")]
    pub country: String,
    #[serde(rename = "perioden")]
    pub periods: Vec<RawCurrencyPeriod>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawCurrencyPeriod {
    #[serde(rename = "ab")]
    pub from: RawPointInTime,
    #[serde(rename = "waehrung")]
    pub currency: String,
    /// Units of the previous currency per unit of this one, fixed by law (M28).
    #[serde(rename = "umrechnung", default)]
    pub conversion: Option<f64>,
}
