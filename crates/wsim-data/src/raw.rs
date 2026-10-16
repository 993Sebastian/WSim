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

/// A branch with its basic product (FW1).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawBranch {
    pub id: String,
    #[serde(rename = "grundprodukt", default)]
    pub basic_product: Option<String>,
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
    /// Codes of former regions that now lie in this entry (old saves read them as it).
    #[serde(rename = "frueher", default)]
    pub former: Vec<String>,
    #[serde(rename = "staedte", default)]
    pub cities: Vec<RawCity>,
    #[serde(rename = "werte")]
    pub values: RawCountryValues,
    #[serde(rename = "praegung", default)]
    pub profile: Option<RawProfile>,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

/// A city of a country (W2).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawCity {
    pub id: String,
    #[serde(rename = "einwohner")]
    pub population: f64,
    #[serde(rename = "breite")]
    pub lat: f64,
    #[serde(rename = "laenge")]
    pub lon: f64,
    #[serde(rename = "hauptstadt", default)]
    pub capital: bool,
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
    /// Consumables (MN1): utility of this product against the replaced one, and the
    /// highest share of its demand it may take over.
    #[serde(rename = "ersetzt_nutzen", default)]
    pub replaces_utility: Option<f64>,
    #[serde(rename = "ersetzt_anteil_max", default)]
    pub replaces_max_share: Option<f64>,
    /// Durables (MN4): used units change hands.
    #[serde(rename = "gebraucht", default)]
    pub used_market: bool,
    #[serde(rename = "foerderindex", default)]
    pub output_index: Option<RawSeries>,
    #[serde(rename = "pacht_anteil", default)]
    pub rent_share: Option<f64>,
    /// May use five or six levels in its product tree (Lastenheft §17.2).
    #[serde(rename = "sehr_komplex", default)]
    pub very_complex: bool,
    /// Storage class (BP3); without it the rules of the storage model decide.
    #[serde(rename = "lagerklasse", default)]
    pub storage_class: Option<String>,
    /// Shelf life in days (BP3); without it the product keeps.
    #[serde(rename = "haltbarkeit_tage", default)]
    pub shelf_life_days: Option<u32>,
    /// Storage volume per unit in m³; without it from the weight and the class density.
    #[serde(rename = "lagervolumen_m3", default)]
    pub storage_volume_m3: Option<f64>,
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
    /// Armament good (H3).
    #[serde(rename = "ruestung", default)]
    pub armament: bool,
}

/// `parameter/ruestung.yaml` (H3).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawArmament {
    #[serde(rename = "militaerausgaben")]
    pub military_share: RawCountrySeries,
    #[serde(rename = "frieden")]
    pub peace: f64,
    #[serde(rename = "krieg")]
    pub war: f64,
    #[serde(rename = "bezug")]
    pub reference: f64,
    #[serde(default, rename = "annaeherung")]
    pub approximation: bool,
    #[serde(default, rename = "quelle")]
    pub source: Option<String>,
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
    /// Emissions per run (H2): t CO2 and kg of pollutants.
    #[serde(rename = "co2_t", default)]
    pub co2_t: f64,
    #[serde(rename = "schadstoff_kg", default)]
    pub pollutant_kg: f64,
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
pub struct RawCommodityCycle {
    pub schwankung: f64,
    pub beharrung: f64,
    pub grenze: f64,
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
    #[serde(rename = "foerderkurve_ab")]
    pub decline_from: f64,
    #[serde(rename = "rohstoffzyklus", default)]
    pub commodity_cycle: Option<RawCommodityCycle>,
    #[serde(rename = "schulung")]
    pub training: RawTraining,
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
    #[serde(rename = "schliessung")]
    pub closing: RawClosing,
    /// Storage capacity, costs and spoilage; without it stocks are unlimited.
    #[serde(rename = "lager", default)]
    pub storage: Option<RawStorage>,
    /// Learning curve; without it experience has no effect.
    #[serde(rename = "lernkurve", default)]
    pub learning: Option<RawLearning>,
    /// Trim levels of end products (MN2); without it all make the standard.
    #[serde(rename = "ausstattung", default)]
    pub trims: Option<RawTrims>,
    /// Maintenance, breakdowns, overhauls; without it wear as before.
    #[serde(rename = "instandhaltung", default)]
    pub maintenance: Option<RawMaintenance>,
    /// Scrap, accidents, corruption, rule of law (BP4).
    #[serde(rename = "betrieb", default)]
    pub operations: Option<RawOperations>,
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
    #[serde(rename = "umstellung")]
    pub changeover: RawChangeover,
}

/// Changing a unit to another product of its branch (FW1).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawChangeover {
    pub kosten_anteil: f64,
    pub dauer_anteil: f64,
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

/// Storage of the sites (capacity, costs, spoilage).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawStorage {
    #[serde(rename = "mindestens_m3")]
    pub minimum_m3: BTreeMap<String, f64>,
    #[serde(rename = "tage")]
    pub days: f64,
    #[serde(rename = "ausbau")]
    pub step: RawStorageStep,
    #[serde(rename = "kosten_jahr_anteil")]
    pub cost_share_year: f64,
    #[serde(rename = "verderb", default)]
    pub spoilage: BTreeMap<String, f64>,
    #[serde(rename = "ki_ausbau_ab")]
    pub ai_expand_at: f64,
    /// Yearly cost share per storage class (BP3).
    #[serde(rename = "klassen", default)]
    pub classes: BTreeMap<String, f64>,
    #[serde(rename = "nach_transportklasse", default)]
    pub by_transport: BTreeMap<String, String>,
    #[serde(rename = "nach_warengruppe", default)]
    pub by_group: BTreeMap<String, String>,
    #[serde(rename = "gekuehlt_bis_tage", default)]
    pub chilled_up_to_days: u32,
    #[serde(rename = "fremdlager", default)]
    pub external: Option<RawExternalStorage>,
    #[serde(rename = "verderb_meldung_ab", default)]
    pub spoil_message_share: Option<f64>,
    /// Stacking density per storage class in t per m³: weight per unit → storage volume.
    #[serde(rename = "dichte_t_je_m3", default)]
    pub density: BTreeMap<String, f64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawExternalStorage {
    #[serde(rename = "kapazitaet_faktor")]
    pub capacity: f64,
    #[serde(rename = "kosten_faktor")]
    pub rate: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawOperations {
    #[serde(rename = "ausschuss")]
    pub scrap: RawScrap,
    #[serde(rename = "unfaelle_je_100000_stunden")]
    pub accidents: RawSeries,
    #[serde(rename = "unfall_kosten_stunden")]
    pub accident_hours: f64,
    #[serde(rename = "unfall_stillstand_anteil")]
    pub accident_stop_share: f64,
    #[serde(rename = "korruption")]
    pub corruption: RawCorruption,
    #[serde(rename = "rechtssicherheit")]
    pub rule_of_law: RawRuleOfLaw,
    #[serde(rename = "enteignung_basis")]
    pub expropriation: f64,
    #[serde(rename = "make_or_buy_toleranz")]
    pub make_or_buy_tolerance: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawScrap {
    #[serde(rename = "basis")]
    pub base: RawPerKind,
    #[serde(rename = "min")]
    pub min: f64,
    #[serde(rename = "max")]
    pub max: f64,
    #[serde(rename = "schulung")]
    pub training: f64,
    #[serde(rename = "weiterentwicklung")]
    pub development: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawCorruption {
    #[serde(rename = "grund")]
    pub base: f64,
    #[serde(rename = "entwicklung")]
    pub development: f64,
    #[serde(rename = "instabilitaet")]
    pub instability: f64,
    #[serde(rename = "bauzeit")]
    pub build_time: f64,
    #[serde(rename = "gebuehr")]
    pub fee: f64,
    #[serde(rename = "forderung_chance")]
    pub demand_chance: f64,
    #[serde(rename = "forderung_anteil")]
    pub demand_share: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRuleOfLaw {
    #[serde(rename = "grund")]
    pub base: f64,
    #[serde(rename = "entwicklung")]
    pub development: f64,
    #[serde(rename = "stabilitaet")]
    pub stability: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawMaintenance {
    #[serde(rename = "stufe_min")]
    pub level_min: f64,
    #[serde(rename = "stufe_max")]
    pub level_max: f64,
    #[serde(rename = "exponent")]
    pub exponent: f64,
    #[serde(rename = "kapazitaet_zustand")]
    pub capacity_weight: f64,
    #[serde(rename = "ausfall")]
    pub failure: RawFailure,
    #[serde(rename = "ueberholung")]
    pub overhaul: RawOverhaul,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFailure {
    #[serde(rename = "basis")]
    pub chance: f64,
    #[serde(rename = "tage_min")]
    pub days_min: u32,
    #[serde(rename = "tage_max")]
    pub days_max: u32,
    #[serde(rename = "kosten_anteil")]
    pub cost_share: f64,
    #[serde(rename = "warnung_ab")]
    pub warning: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawOverhaul {
    #[serde(rename = "kosten_anteil")]
    pub cost_share: f64,
    #[serde(rename = "tage")]
    pub days: u32,
    #[serde(rename = "ki_ab")]
    pub ai_below: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLearning {
    #[serde(rename = "rate")]
    pub rates: BTreeMap<String, f64>,
    #[serde(rename = "rate_sonst")]
    pub default_rate: f64,
    #[serde(rename = "untergrenze")]
    pub minimum: f64,
    #[serde(rename = "vergessen_je_monat")]
    pub forget_per_month: f64,
    #[serde(rename = "entlassung")]
    pub layoff_loss: f64,
    #[serde(rename = "uebertrag")]
    pub transfer: f64,
    #[serde(rename = "start_etabliert_jahre")]
    pub established_years: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawStorageStep {
    #[serde(rename = "kapazitaet_m3")]
    pub capacity_m3: f64,
    #[serde(rename = "kosten_usd")]
    pub cost_usd: f64,
}

/// Closed sites.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawClosing {
    #[serde(rename = "lager_erloes_anteil")]
    pub stock_proceeds_share: f64,
    #[serde(rename = "abfindung_monate")]
    pub severance_months: f64,
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
    #[serde(rename = "dividende")]
    pub dividends: RawDividends,
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
    /// Substitution of consumables (MN1); without it nothing is displaced.
    #[serde(rename = "substitution", default)]
    pub substitution: Option<RawSubstitution>,
    /// Demand trends (MN1).
    #[serde(rename = "trends", default)]
    pub trends: Vec<RawTrend>,
    /// Reputation and recalls (MN2).
    #[serde(rename = "ruf", default)]
    pub reputation: Option<RawReputation>,
    /// Tenders of governments (MN2).
    #[serde(rename = "ausschreibung", default)]
    pub tenders: Option<RawTenders>,
    /// Sales channels (MN3).
    #[serde(rename = "vertrieb", default)]
    pub channels: Option<RawChannelModel>,
    /// Obsolescence, used market and generated trends (MN4).
    #[serde(rename = "veralterung", default)]
    pub obsolescence: Option<RawObsolescence>,
    #[serde(rename = "gebraucht", default)]
    pub used: Option<RawUsed>,
    #[serde(rename = "trendgenerator", default)]
    pub trend_generator: Option<RawTrendGenerator>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawObsolescence {
    #[serde(rename = "punkte_je_stufe")]
    pub points_per_level: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawUsed {
    #[serde(rename = "weitergabe")]
    pub pass_on: f64,
    #[serde(rename = "lebensdauer_anteil")]
    pub life_share: f64,
    #[serde(rename = "anteil_erstkauf")]
    pub first_share: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTrendGenerator {
    #[serde(rename = "ab")]
    pub from_year: i32,
    pub chance: f64,
    pub faktor_min: f64,
    pub faktor_max: f64,
    pub jahre_min: u32,
    pub jahre_max: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawChannelModel {
    #[serde(rename = "grundreichweite")]
    pub base_reach: f64,
    #[serde(rename = "ki_verzoegerung_jahre")]
    pub ai_delay_years: f64,
    #[serde(rename = "ki_marge")]
    pub ai_margin: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTenders {
    #[serde(rename = "warengruppen")]
    pub groups: Vec<String>,
    #[serde(rename = "los_anteil")]
    pub lot_share: f64,
    #[serde(rename = "frist_tage")]
    pub deadline_days: u32,
    #[serde(rename = "laufzeit_monate")]
    pub months: u32,
    #[serde(rename = "preisgewicht")]
    pub price_weight: f64,
    #[serde(rename = "qualitaetsgewicht")]
    pub quality_weight: f64,
    #[serde(rename = "heimatbonus")]
    pub home_bonus: f64,
    #[serde(rename = "kriegsfaktor")]
    pub war_factor: f64,
    #[serde(rename = "rufgewicht")]
    pub reputation_weight: f64,
    #[serde(rename = "ki_nachlass")]
    pub ai_discount: f64,
    #[serde(rename = "strafe")]
    pub penalty: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawReputation {
    #[serde(rename = "gewinn_je_monat")]
    pub gain_per_month: f64,
    #[serde(rename = "rueckkehr_je_monat")]
    pub return_per_month: f64,
    #[serde(rename = "unfall_abzug")]
    pub accident_loss: f64,
    #[serde(rename = "rueckruf_schwelle")]
    pub recall_below: f64,
    #[serde(rename = "rueckruf_chance")]
    pub recall_chance: f64,
    #[serde(rename = "rueckruf_abzug")]
    pub recall_loss: f64,
    #[serde(rename = "rufgewicht")]
    pub weight: Vec<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTrims {
    pub einfach: RawTrim,
    pub premium: RawTrim,
    /// Competence of AI companies from which they make premium, and up to which simple.
    #[serde(rename = "ki_premium_ab")]
    pub ai_premium_from: f64,
    #[serde(rename = "ki_einfach_bis")]
    pub ai_simple_up_to: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTrim {
    pub material: f64,
    #[serde(rename = "arbeit")]
    pub labor: f64,
    #[serde(rename = "qualitaet")]
    pub quality: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSubstitution {
    #[serde(rename = "steilheit")]
    pub steepness: f64,
    #[serde(rename = "je_jahr")]
    pub per_year: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTrend {
    pub id: String,
    #[serde(rename = "warengruppen", default)]
    pub groups: Vec<String>,
    #[serde(rename = "produkte", default)]
    pub products: Vec<String>,
    #[serde(rename = "laender", default)]
    pub countries: Vec<String>,
    #[serde(rename = "verlauf")]
    pub curve: RawSeries,
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
pub struct RawChannel {
    pub id: String,
    #[serde(rename = "verfuegbar_ab")]
    pub available_from: i32,
    #[serde(rename = "warengruppen", default)]
    pub groups: Vec<String>,
    #[serde(rename = "reichweite")]
    pub reach: RawSeries,
    #[serde(rename = "schichtfaktor")]
    pub layer_factor: Vec<f64>,
    #[serde(rename = "fixkosten_je_land_usd")]
    pub fixed_cost_usd: RawSeries,
    #[serde(rename = "kosten_je_verkauf")]
    pub cost_per_sale: f64,
    #[serde(rename = "ruf_faktor")]
    pub reputation_factor: f64,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
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
    /// Payload of one vehicle in tonnes and its price in USD, by year (W5); without
    /// them no fleet buys one.
    #[serde(rename = "nutzlast_t", default)]
    pub payload_t: Option<RawSeries>,
    #[serde(rename = "kaufpreis_usd", default)]
    pub price_usd: Option<RawSeries>,
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
    /// Price islands (C1).
    #[serde(rename = "arbitrage")]
    pub arbitrage: RawArbitrage,
}

/// Training of the sites (W1).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTraining {
    #[serde(rename = "kosten_anteil_lohn")]
    pub cost_share: f64,
    #[serde(rename = "aufbau_je_monat")]
    pub gain_per_month: f64,
    #[serde(rename = "verlust_je_monat")]
    pub loss_per_month: f64,
    #[serde(rename = "arbeitsersparnis")]
    pub labor_saving: f64,
    #[serde(rename = "qualitaet_punkte")]
    pub quality_points: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawArbitrage {
    #[serde(rename = "abstand")]
    pub gap: f64,
    #[serde(rename = "anteil")]
    pub share: f64,
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
    /// Patents (P7); without the block there are none.
    #[serde(rename = "patente", default)]
    pub patents: Option<RawPatents>,
    /// Staff roles of a laboratory (PD1); without the block the posts fill themselves.
    #[serde(rename = "labor", default)]
    pub lab_staff: Option<RawLabStaff>,
    /// Development projects PM0–PM7 (PD2); without the block products are researched as before.
    #[serde(rename = "projekte", default)]
    pub projects: Option<RawProjectModel>,
}

/// Development projects (PD2, docs/FORMELN.md).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectModel {
    #[serde(rename = "punktekosten_usd")]
    pub cost_per_point_usd: f64,
    #[serde(rename = "referenzrate")]
    pub reference_rate: f64,
    #[serde(rename = "referenzaufwand")]
    pub reference_effort: f64,
    #[serde(rename = "bedarf_exponent")]
    pub need_exponent: f64,
    #[serde(rename = "ueberbesetzung")]
    pub over: RawOverstaffing,
    /// Share of a phase's speed that runs without any person in a role.
    #[serde(rename = "restanteil")]
    pub rest_share: f64,
    #[serde(rename = "unterbesetzung")]
    pub shortfall: RawShortfall,
    #[serde(rename = "streuung")]
    pub spread: f64,
    /// Cost share of the gate PM3→PM4 (preparation of the release, no duration).
    #[serde(rename = "gate_kosten")]
    pub gate_cost: f64,
    #[serde(rename = "phasen")]
    pub phases: Vec<RawProjectPhase>,
    #[serde(rename = "tempo")]
    pub tempos: Vec<RawProjectTempo>,
    #[serde(rename = "ausstattung")]
    pub trims: Vec<RawProjectTrim>,
    #[serde(rename = "projektleiter")]
    pub lead: RawProjectLead,
    #[serde(rename = "risiko")]
    pub risk: RawProjectRisk,
    /// Milestone reports and decisions (PD3); without them the milestones pass by themselves.
    #[serde(rename = "berichte", default)]
    pub reports: Option<RawProjectReports>,
    #[serde(rename = "entscheidung", default)]
    pub decision: Option<RawProjectDecision>,
    #[serde(rename = "profile", default)]
    pub profiles: Vec<RawProjectChoice>,
    #[serde(rename = "loesungswege", default)]
    pub paths: Vec<RawProjectChoice>,
    /// Events during the phases, hidden defects and recalls (PD4).
    #[serde(rename = "ereignisse", default)]
    pub events: Option<RawProjectEvents>,
    /// Improvement projects WM0–WM3 (PD5).
    #[serde(rename = "weiterentwicklung", default)]
    pub improvement: Option<RawProjectImprovement>,
    /// Product suggestions and the after-calculation (PD6).
    #[serde(rename = "vorschlaege", default)]
    pub suggestions: Option<RawProjectSuggestions>,
    /// Gate council (PD7).
    #[serde(rename = "gremium", default)]
    pub council: Option<RawProjectCouncil>,
    /// Fast-track (PD7).
    #[serde(rename = "fast_track", default)]
    pub fast_track: Option<RawProjectFastTrack>,
    /// Pilot series (PD7).
    #[serde(rename = "pilot", default)]
    pub pilot: Option<RawProjectPilot>,
    /// Development leadership: levels, budget cascade, lead times (PD11).
    #[serde(rename = "leitung", default)]
    pub leadership: Option<RawProjectLeadership>,
    /// Partial knowledge of aborted projects (PD8).
    #[serde(rename = "teilwissen", default)]
    pub knowledge: Option<RawProjectKnowledge>,
    /// Platform strategy (PD8).
    #[serde(rename = "plattform", default)]
    pub platform: Option<RawProjectPlatform>,
    /// Project licence (PD8).
    #[serde(rename = "lizenz", default)]
    pub license: Option<RawProjectLicense>,
    /// Development partnership (PD8).
    #[serde(rename = "partnerschaft", default)]
    pub partnership: Option<RawProjectPartnership>,
    /// First-mover bonus (PD9).
    #[serde(rename = "erstankunft", default)]
    pub first_mover: Option<RawProjectFirstMover>,
    /// State programs (PD9).
    #[serde(rename = "foerderung", default)]
    pub subsidies: Vec<RawProjectSubsidy>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectFirstMover {
    #[serde(rename = "bekanntheit")]
    pub awareness: f64,
    #[serde(rename = "ruf")]
    pub reputation: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectSubsidy {
    pub id: String,
    #[serde(rename = "von")]
    pub from_year: i32,
    #[serde(rename = "bis")]
    pub to_year: i32,
    #[serde(rename = "anteil")]
    pub share: f64,
    #[serde(rename = "deckel_usd")]
    pub cap_usd: f64,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectKnowledge {
    #[serde(rename = "anteil")]
    pub share: f64,
    #[serde(rename = "wirkung")]
    pub effect: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectPlatform {
    #[serde(rename = "wirkung")]
    pub effect: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectLicense {
    #[serde(rename = "preis_anteil")]
    pub price_share: f64,
    #[serde(rename = "ab_phase")]
    pub from_phase: u8,
    #[serde(rename = "gebuehr_anteil")]
    pub fee_share: f64,
    #[serde(rename = "gebuehr_jahre")]
    pub fee_years: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectPartnership {
    #[serde(rename = "kosten_anteil")]
    pub cost_share: f64,
    #[serde(rename = "ki_annahme")]
    pub ai_accept: f64,
}

/// The gate council at PM3, PM4 and PM7 (PD7).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectCouncil {
    #[serde(rename = "unschaerfe")]
    pub uncertainty: f64,
    #[serde(rename = "genauigkeit")]
    pub accuracy: f64,
    #[serde(rename = "schwelle")]
    pub threshold: f64,
    #[serde(rename = "pause_mehrheit")]
    pub pause_majority: u32,
    #[serde(rename = "schleifen_max")]
    pub max_loops: u32,
}

/// Fast-track: overlapping phases on the leader's proposal (PD7).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectFastTrack {
    #[serde(rename = "erfahrung_min")]
    pub experience_min: f64,
    #[serde(rename = "ueberlappung")]
    pub overlap: f64,
    #[serde(rename = "kosten_faktor")]
    pub cost_factor: f64,
    #[serde(rename = "mangel")]
    pub defect: f64,
}

/// Development leadership (PD11).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectLeadership {
    #[serde(rename = "vorlauf_monate")]
    pub lead_months: u32,
    #[serde(rename = "einarbeitung_monate")]
    pub induction_months: u32,
    #[serde(rename = "abfindung_monate")]
    pub severance_months: u32,
    #[serde(rename = "vorausschau_monate")]
    pub horizon_months: u32,
    #[serde(rename = "geduld_monate")]
    pub patience_months: u32,
    pub reserve: f64,
    #[serde(rename = "reserve_sparsam")]
    pub reserve_thrifty: f64,
    #[serde(rename = "reserve_grosszuegig")]
    pub reserve_generous: f64,
    #[serde(rename = "budget_anteil")]
    pub budget_share: f64,
    #[serde(rename = "grundausstattung")]
    pub base_share: f64,
    /// Marks estimated values; not used by the build.
    #[serde(rename = "annaeherung", default)]
    #[allow(dead_code)]
    pub approximation: bool,
}

/// Pilot series between PM6 and PM7 (PD7).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectPilot {
    #[serde(rename = "risiko_ab")]
    pub risk_from: f64,
    #[serde(rename = "dauer_anteil")]
    pub duration_share: f64,
    #[serde(rename = "kosten_anteil")]
    pub cost_share: f64,
    #[serde(rename = "aufdeckung_faktor")]
    pub detection: f64,
    #[serde(rename = "behebung")]
    pub repair: f64,
    #[serde(rename = "nacharbeit")]
    pub rework: f64,
    #[serde(rename = "qualitaet_bonus")]
    pub quality_bonus: f64,
    #[serde(rename = "beruhigung")]
    pub calm: f64,
}

/// Product suggestions of the strategy and their after-calculation (PD6).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectSuggestions {
    #[serde(rename = "offen_max")]
    pub open_max: u32,
    #[serde(rename = "gueltig_tage")]
    pub valid_days: u32,
    #[serde(rename = "branchenbonus")]
    pub branch_bonus: f64,
    #[serde(rename = "unschaerfe")]
    pub uncertainty: f64,
    #[serde(rename = "genauigkeit")]
    pub accuracy: f64,
    #[serde(rename = "nachkalkulation_jahre")]
    pub review_years: u32,
    #[serde(rename = "toleranz")]
    pub tolerance: f64,
    #[serde(rename = "treffer_ab")]
    pub hit_from: f64,
    #[serde(rename = "gewichte")]
    pub weights: RawSuggestionWeights,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSuggestionWeights {
    #[serde(rename = "termin")]
    pub schedule: f64,
    #[serde(rename = "budget")]
    pub budget: f64,
    #[serde(rename = "kennzahlen")]
    pub figures: f64,
}

/// Improvement projects (PD5): effort, approaches, outcome, phases and targets.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectImprovement {
    #[serde(rename = "stufen")]
    pub stages: u8,
    #[serde(rename = "restanteil")]
    pub rest_share: f64,
    #[serde(rename = "aufwand")]
    pub effort: RawImprovementEffort,
    #[serde(rename = "umruestung_tage")]
    pub retrofit_days: u32,
    #[serde(rename = "sofort_kosten_anteil")]
    pub immediate_cost_share: f64,
    #[serde(rename = "frist_tage")]
    pub deadline_days: u32,
    #[serde(rename = "ansaetze")]
    pub approaches: RawImprovementApproaches,
    #[serde(rename = "ergebnis")]
    pub outcome: RawImprovementOutcome,
    #[serde(rename = "phasen")]
    pub phases: Vec<RawProjectPhase>,
    #[serde(rename = "ziele")]
    pub targets: Vec<RawImprovementTarget>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawImprovementEffort {
    #[serde(rename = "anteil")]
    pub share: f64,
    #[serde(rename = "wachstum")]
    pub growth: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawImprovementApproaches {
    pub min: u8,
    pub max: u8,
    #[serde(rename = "streuung")]
    pub spread: f64,
    #[serde(rename = "kosten_steigung")]
    pub cost_slope: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawImprovementOutcome {
    #[serde(rename = "streuung")]
    pub spread: f64,
    #[serde(rename = "erfahrung")]
    pub experience: f64,
    #[serde(rename = "unterbesetzung")]
    pub shortfall: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawImprovementTarget {
    pub id: String,
    #[serde(rename = "je_stufe")]
    pub per_stage: f64,
    #[serde(rename = "obergrenze")]
    pub cap: f64,
    #[serde(rename = "wirkung")]
    pub effect: RawImprovementEffect,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawImprovementEffect {
    #[serde(rename = "arbeit", default)]
    pub labor: f64,
    #[serde(rename = "vorprodukte", default)]
    pub inputs: f64,
    #[serde(rename = "qualitaet", default)]
    pub quality: f64,
    #[serde(rename = "ausschuss", default)]
    pub scrap: f64,
    #[serde(rename = "menge", default)]
    pub output: f64,
}

/// Event catalogue, hidden defects and recalls (PD4).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectEvents {
    #[serde(rename = "frist_tage")]
    pub deadline_days: u32,
    #[serde(rename = "mangel")]
    pub defect: RawProjectDefect,
    #[serde(rename = "rueckruf")]
    pub recall: RawProjectRecall,
    #[serde(rename = "katalog")]
    pub kinds: Vec<RawProjectEventKind>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectDefect {
    #[serde(rename = "obergrenze")]
    pub cap: f64,
    #[serde(rename = "je_auflage")]
    pub per_condition: f64,
    #[serde(rename = "je_unterbesetzung")]
    pub per_shortfall: f64,
    #[serde(rename = "je_tempo")]
    pub per_tempo: Vec<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectRecall {
    #[serde(rename = "verzug_tage")]
    pub days: [u32; 2],
    #[serde(rename = "kosten_anteil")]
    pub cost_share: f64,
    #[serde(rename = "bekanntheit_verlust")]
    pub awareness_loss: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectEventKind {
    pub id: String,
    #[serde(rename = "phasen")]
    pub phases: Vec<u8>,
    #[serde(rename = "gewicht")]
    pub weight: f64,
    #[serde(rename = "empfehlung")]
    pub recommended: usize,
    #[serde(rename = "optionen")]
    pub options: Vec<RawProjectEventOption>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectEventOption {
    pub id: String,
    #[serde(rename = "kosten")]
    pub cost: f64,
    #[serde(rename = "verzug")]
    pub delay: f64,
    #[serde(rename = "qualitaet")]
    pub quality: f64,
    #[serde(rename = "mangel")]
    pub defect: f64,
}

/// Report figures, traffic lights and the leader's traits (PD3).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectReports {
    #[serde(rename = "lebenszyklus_jahre")]
    pub lifecycle_years: f64,
    #[serde(rename = "absatzanteil")]
    pub capture_share: f64,
    #[serde(rename = "anlagen_ohne_nachfrage")]
    pub plants_without_demand: f64,
    #[serde(rename = "auslastung")]
    pub utilization: f64,
    #[serde(rename = "unschaerfe")]
    pub uncertainty: f64,
    #[serde(rename = "genauigkeit_je_pm")]
    pub accuracy_per_pm: f64,
    #[serde(rename = "charaktere")]
    pub characters: std::collections::BTreeMap<String, RawLeadTrait>,
    #[serde(rename = "ampel")]
    pub lights: RawLights,
    #[serde(rename = "amortisation_max_jahre")]
    pub payback_max_years: f64,
    #[serde(rename = "auflagen_risiko")]
    pub conditions_risk: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLeadTrait {
    #[serde(rename = "optimismus")]
    pub optimism: f64,
    #[serde(rename = "weg")]
    pub path: String,
    #[serde(rename = "gegenmeinung", default)]
    pub dissent: f64,
    #[serde(rename = "fast_track", default)]
    pub fast_track: f64,
    #[serde(rename = "pilot", default)]
    pub pilot: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLights {
    #[serde(rename = "termin")]
    pub schedule: RawLight,
    #[serde(rename = "budget")]
    pub budget: RawLight,
    #[serde(rename = "qualitaet")]
    pub quality: RawLight,
    #[serde(rename = "risiko")]
    pub risk: RawLight,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLight {
    #[serde(rename = "gelb")]
    pub yellow: f64,
    #[serde(rename = "rot")]
    pub red: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectDecision {
    #[serde(rename = "spieler_bei")]
    pub player_at: Vec<u8>,
    #[serde(rename = "frist_tage")]
    pub deadline_days: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectChoice {
    pub id: String,
    #[serde(rename = "kosten")]
    pub cost: f64,
    #[serde(rename = "dauer")]
    pub time: f64,
    #[serde(rename = "risiko")]
    pub risk: f64,
    #[serde(rename = "qualitaet")]
    pub quality: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawOverstaffing {
    #[serde(rename = "maximum")]
    pub max: f64,
    #[serde(rename = "tau")]
    pub tau: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawShortfall {
    #[serde(rename = "abschlag")]
    pub penalty: f64,
    #[serde(rename = "hoechstens")]
    pub cap: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectPhase {
    pub id: String,
    #[serde(rename = "kosten")]
    pub cost: f64,
    #[serde(rename = "dauer")]
    pub time: f64,
    #[serde(rename = "rollen")]
    pub roles: Vec<RawPhaseRole>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPhaseRole {
    pub id: String,
    /// People needed at the reference effort.
    #[serde(rename = "bedarf")]
    pub need: f64,
    /// Share of the phase's speed this role carries.
    #[serde(rename = "wirkung")]
    pub effect: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectTempo {
    pub id: String,
    #[serde(rename = "dauer")]
    pub time: f64,
    #[serde(rename = "kosten")]
    pub cost: f64,
    #[serde(rename = "risiko")]
    pub risk: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectTrim {
    pub id: String,
    #[serde(rename = "kosten")]
    pub cost: f64,
    #[serde(rename = "qualitaet")]
    pub quality: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectLead {
    #[serde(rename = "gehalt_usd_jahr")]
    pub salary_usd_year: f64,
    #[serde(rename = "kapazitaet_neu")]
    pub capacity_new: u32,
    #[serde(rename = "kapazitaet_weiter")]
    pub capacity_further: u32,
    /// Candidates on the market per laboratory country.
    #[serde(rename = "kandidaten")]
    pub candidates: u32,
    /// Experience (0–1) a lead gains for each finished project.
    #[serde(rename = "erfahrung_je_projekt")]
    pub experience_gain: f64,
    #[serde(rename = "alter_min")]
    pub age_min: u32,
    #[serde(rename = "alter_max")]
    pub age_max: u32,
    #[serde(rename = "charaktere")]
    pub characters: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProjectRisk {
    #[serde(rename = "basis")]
    pub base: f64,
    #[serde(rename = "unterbesetzung")]
    pub shortfall: f64,
    #[serde(rename = "erfahrung")]
    pub experience: f64,
    #[serde(rename = "branche")]
    pub branch: f64,
    #[serde(rename = "puffer")]
    pub buffer: f64,
    #[serde(rename = "fast_track")]
    pub fast_track: f64,
    #[serde(rename = "branche_anlagen_ref", default)]
    pub branch_ref: f64,
    #[serde(rename = "puffer_ref", default)]
    pub buffer_ref: f64,
}

/// Staff roles of a laboratory (PD1).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLabStaff {
    #[serde(rename = "rollen")]
    pub roles: Vec<RawLabRole>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLabRole {
    pub id: String,
    #[serde(rename = "forschung")]
    pub research: f64,
}

/// Patents (P7).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPatents {
    #[serde(rename = "laufzeit_jahre")]
    pub term_years: u32,
    #[serde(rename = "anmeldefrist_tage")]
    pub filing_days: u32,
    #[serde(rename = "kosten_je_land_usd")]
    pub cost_per_country_usd: f64,
    #[serde(rename = "ki_groesste_maerkte")]
    pub ai_largest_markets: u32,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

/// Development of researched products (M37).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProductDevelopment {
    #[serde(rename = "stufen")]
    pub levels: u32,
    /// Effect at the leader's level (Q1).
    #[serde(rename = "wirkung_max")]
    pub per_level: RawDevelopmentPerLevel,
    #[serde(rename = "aufwand")]
    pub effort: RawDevelopmentEffort,
    /// Research field by branch, for products without a technology.
    #[serde(rename = "fachgebiete")]
    pub fields: BTreeMap<String, String>,
    /// Manufacturing research (Q6): effect at the leader's level.
    #[serde(rename = "fertigung", default)]
    pub process: Option<RawProcessResearch>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProcessResearch {
    #[serde(rename = "ausstoss")]
    pub output: f64,
    #[serde(rename = "vorprodukte")]
    pub inputs: f64,
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
    /// Product-specific effort: depth of the input chain and number of distinct parts.
    #[serde(rename = "komplexitaet", default)]
    pub complexity: Option<RawComplexity>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawComplexity {
    #[serde(rename = "zuschlag_je_stufe")]
    pub per_level: f64,
    #[serde(rename = "zuschlag_je_bauteil")]
    pub per_part: f64,
    #[serde(rename = "faktor_min")]
    pub min: f64,
    #[serde(rename = "faktor_max")]
    pub max: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLatecomer {
    #[serde(rename = "rabatt_je_jahr")]
    pub discount: f64,
    #[serde(rename = "minimum")]
    pub min: f64,
}

/// Build-up of the markets from a start year (`markt_aufbau`).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawMarketRamp {
    #[serde(rename = "jahr_start")]
    pub from_year: i32,
    #[serde(rename = "faktor_start")]
    pub start_factor: f64,
    #[serde(rename = "jahre")]
    pub years: f64,
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
    #[serde(rename = "markt_aufbau")]
    pub market_ramp: RawMarketRamp,
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
    #[serde(rename = "frei_min_anteil", default)]
    pub free_share_min: f64,
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
    #[serde(rename = "anteile")]
    pub stakes: RawStakes,
}

/// Shares of companies bought and sold by the person and its companies (PE5).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawStakes {
    #[serde(rename = "praemie_min")]
    pub premium_min: f64,
    #[serde(rename = "praemie_max")]
    pub premium_max: f64,
    #[serde(rename = "rendite_gut")]
    pub good_return: f64,
    #[serde(rename = "abschlag_min")]
    pub discount_min: f64,
    #[serde(rename = "abschlag_max")]
    pub discount_max: f64,
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
    #[serde(rename = "schaetzfehler")]
    pub estimate_error: RawSpan,
    #[serde(rename = "schulung")]
    pub training: RawSpan,
    #[serde(rename = "werbeanteil")]
    pub advertising_share: RawSpan,
    #[serde(rename = "einkauf_aufschlag")]
    pub purchase_markup: f64,
    #[serde(rename = "ausbau_auslastung")]
    pub expand_utilization: RawSpan,
    #[serde(rename = "ausbau_marge")]
    pub expand_margin: RawSpan,
    #[serde(rename = "ausbau_markt_auslastung", default)]
    pub expand_market_load: f64,
    #[serde(rename = "ausbau_vorprodukt_preis_max")]
    pub expand_input_price_max: f64,
    #[serde(rename = "ausbau_anteil_kasse_max")]
    pub invest_share_max: f64,
    #[serde(rename = "ausbau_je_pruefung_max")]
    pub expansions_max: u32,
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
    #[serde(rename = "forschung_luecke_vorrang_umsatz", default)]
    pub research_gap_priority_revenue: f64,
    #[serde(rename = "lernen_entwicklung_anteil", default)]
    pub learning_development_share: f64,
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
    #[serde(rename = "einstiege_je_quartal", default)]
    pub entries_per_quarter: u32,
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
    #[serde(rename = "regel_themen", default)]
    pub rule_topics: Vec<String>,
    /// Pairs of topics whose recommendations can contradict each other (MG4).
    #[serde(rename = "widerspruch_themen", default)]
    pub contradictions: Vec<[String; 2]>,
    #[serde(rename = "budget_sockel_gehaelter")]
    pub budget_floor: RawBudgetFloor,
    #[serde(rename = "anliegen")]
    pub concerns: RawConcerns,
    #[serde(rename = "strategie")]
    pub strategy: RawStrategy,
    #[serde(rename = "strategieauftrag")]
    pub mandate: RawMandateModel,
    #[serde(rename = "markt")]
    pub market: RawManagerMarket,
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
    #[serde(rename = "branchenstaerke")]
    pub branch: RawBranchSkill,
    #[serde(rename = "stil")]
    pub style: RawStyleModel,
    #[serde(rename = "personal")]
    pub staffing: RawStaffing,
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
    /// Months to the review of a decision per topic key; `standard` for the rest (MG2).
    #[serde(rename = "wirkzeit_monate", default)]
    pub effect_months: std::collections::BTreeMap<String, u32>,
    #[serde(rename = "treffer_toleranz", default)]
    pub hit_tolerance: f64,
    #[serde(rename = "lernchance", default)]
    pub learn_chance: f64,
    #[serde(rename = "akten_behalten", default)]
    pub files_kept: u32,
    /// Risk of an option per topic key; `standard` for the rest (MG4).
    #[serde(rename = "option_risiko", default)]
    pub option_risk: std::collections::BTreeMap<String, f64>,
    #[serde(rename = "nichtstun_risiko", default)]
    pub keep_risk: std::collections::BTreeMap<String, f64>,
    /// Running cost per year as a share of the amount, per topic key (MG4).
    #[serde(rename = "laufend_anteil", default)]
    pub running_share: std::collections::BTreeMap<String, f64>,
}

/// Strategies of the managers (MA4).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawStrategy {
    pub premium: RawPriceStrategy,
    #[serde(rename = "kampfpreis")]
    pub fight: RawPriceStrategy,
    #[serde(rename = "mindestmarge_max")]
    pub min_margin_max: f64,
    #[serde(rename = "lager_tage_max")]
    pub stock_days_max: f64,
    #[serde(rename = "liquiditaet_monate_max")]
    pub reserve_months_max: f64,
}

/// Mandate and strategy review (MA5).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawMandateModel {
    #[serde(rename = "leitlinien")]
    pub guidelines: RawGuidelines,
    #[serde(rename = "antraege_max")]
    pub proposals_max: u32,
    #[serde(rename = "chancen_risiken")]
    pub chances_risks: u32,
    #[serde(rename = "ruecksprachen_behalten")]
    pub reviews_kept: u32,
    #[serde(rename = "personal_schaerfe")]
    pub personnel_sharpness: f64,
    #[serde(rename = "vorschlaege_je_quartal")]
    pub proposals_per_quarter: u32,
    #[serde(rename = "vorschlag_aufschub_tage")]
    pub proposal_postpone_days: u32,
    #[serde(rename = "vorschlag_branche_gewicht")]
    pub proposal_branch_weight: f64,
}

/// The living market of managers (MA6).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawManagerMarket {
    #[serde(rename = "erfahrung")]
    pub experience: RawExperience,
    #[serde(rename = "zufriedenheit")]
    pub satisfaction: RawSatisfaction,
    #[serde(rename = "kuendigung")]
    pub resignation: RawResignation,
    #[serde(rename = "abwerbung")]
    pub poaching: RawPoaching,
    #[serde(rename = "ki")]
    pub ai: RawAiHiring,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawExperience {
    #[serde(rename = "chance_monat")]
    pub chance_per_month: f64,
    #[serde(rename = "spielraum")]
    pub room: u8,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSatisfaction {
    pub start: u8,
    #[serde(rename = "basis")]
    pub base: f64,
    #[serde(rename = "gehalt_gewicht")]
    pub salary_weight: f64,
    #[serde(rename = "verlust_abzug")]
    pub loss_penalty: f64,
    #[serde(rename = "uebergangen_abzug")]
    pub overruled_penalty: f64,
    #[serde(rename = "anpassung")]
    pub adjust: f64,
    /// Upper bounds of "unzufrieden" and "gemischt".
    #[serde(rename = "stufen")]
    pub bands: [u8; 2],
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawResignation {
    #[serde(rename = "schwelle")]
    pub threshold: u8,
    pub chance_max: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPoaching {
    #[serde(rename = "staerke_min")]
    pub min_strength: f64,
    #[serde(rename = "vorsprung")]
    pub lead: f64,
    #[serde(rename = "aufschlag")]
    pub markup: f64,
    #[serde(rename = "ignoriert_abzug")]
    pub ignored_penalty: u8,
    pub ki_gegen_max: f64,
    #[serde(rename = "sperre_monate")]
    pub pause_months: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAiHiring {
    #[serde(rename = "einstellungen_monat")]
    pub per_month: u32,
    #[serde(rename = "umsatz_ceo_usd")]
    pub ceo_revenue_usd: f64,
    #[serde(rename = "umsatz_standort_usd")]
    pub site_revenue_usd: f64,
    #[serde(rename = "gehalt_anteil")]
    pub salary_share: f64,
    #[serde(rename = "kompetenz_ceo")]
    pub competence_ceo: f64,
    #[serde(rename = "kompetenz_leitung")]
    pub competence_heads: f64,
}

/// Aggressiveness of the rules per guideline (0–1).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawGuidelines {
    #[serde(rename = "wachstum")]
    pub growth: f64,
    #[serde(rename = "ertrag")]
    pub profit: f64,
    #[serde(rename = "sicherheit")]
    pub safety: f64,
    #[serde(rename = "marktfuehrung")]
    pub leadership: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPriceStrategy {
    /// Price floor as a multiple of the full unit cost.
    #[serde(rename = "untergrenze")]
    pub floor: f64,
    /// Markup on the market price an offer starts at.
    #[serde(rename = "aufschlag")]
    pub markup: f64,
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
    #[serde(rename = "ki_wartetage", default)]
    pub ai_wait_days: u32,
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

/// Hiring by the heads (MG8).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawStaffing {
    #[serde(rename = "einstellungen_monat")]
    pub per_month: u32,
    #[serde(rename = "kandidaten", default = "RawStaffing::candidates_default")]
    pub candidates: u32,
    #[serde(rename = "je_personal", default = "RawStaffing::per_personnel_default")]
    pub per_personnel: u32,
    #[serde(rename = "optionen", default = "RawStaffing::options_default")]
    pub options: u32,
    #[serde(rename = "wirkzeit_monate", default = "RawStaffing::review_default")]
    pub review_months: u32,
    #[serde(rename = "treffer_zufriedenheit", default = "RawStaffing::hit_default")]
    pub hit_satisfaction: u32,
    #[serde(rename = "grenze_usd", default = "RawStaffing::limit_default")]
    pub limit_usd: f64,
    #[serde(rename = "unterdeckt_ab", default = "RawStaffing::under_default")]
    pub under: f64,
    #[serde(rename = "ueberbesetzt_ab", default = "RawStaffing::over_default")]
    pub over: f64,
    #[serde(
        rename = "abteilung_wirkzeit_monate",
        default = "RawStaffing::review_default"
    )]
    pub dept_review_months: u32,
}

impl RawStaffing {
    fn candidates_default() -> u32 {
        3
    }
    fn per_personnel_default() -> u32 {
        2
    }
    fn options_default() -> u32 {
        3
    }
    fn review_default() -> u32 {
        12
    }
    fn hit_default() -> u32 {
        60
    }
    fn limit_default() -> f64 {
        250_000.0
    }
    fn under_default() -> f64 {
        0.8
    }
    fn over_default() -> f64 {
        0.6
    }
}

/// Style, favourite reasons and objections of managers (MG7).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawStyleModel {
    #[serde(rename = "arten")]
    pub kinds: Vec<RawStyleKind>,
    #[serde(rename = "begruendungen")]
    pub reasons: Vec<String>,
    #[serde(rename = "zweite_begruendung_chance")]
    pub second_reason_chance: f64,
    #[serde(rename = "einwand")]
    pub objection: RawObjection,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawStyleKind {
    pub id: String,
    #[serde(rename = "anteil")]
    pub share: f64,
    #[serde(rename = "abwarten")]
    pub keep_bias: f64,
    #[serde(rename = "spanne")]
    pub spread: f64,
    #[serde(rename = "optimismus")]
    pub optimism: f64,
    #[serde(rename = "abweichung")]
    pub deviation: f64,
    #[serde(rename = "einwand")]
    pub objection: f64,
    /// Factor on the recommended staff of a department (MG8).
    #[serde(rename = "personal", default = "RawStyleKind::staff_default")]
    pub staff: f64,
}

impl RawStyleKind {
    fn staff_default() -> f64 {
        1.0
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawObjection {
    #[serde(rename = "schwelle_usd")]
    pub threshold_usd: f64,
    #[serde(rename = "chance")]
    pub chance: f64,
}

/// Industry experience of managers (MG5).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawBranchSkill {
    #[serde(rename = "heimat")]
    pub home: RawSpread,
    #[serde(rename = "zweite_chance")]
    pub second_chance: f64,
    #[serde(rename = "zweite")]
    pub second: RawSpread,
    #[serde(rename = "uebrige_max")]
    pub other_max: u8,
    #[serde(rename = "wachstum_chance")]
    pub growth_chance: f64,
    #[serde(rename = "schwelle")]
    pub threshold: f64,
    #[serde(rename = "leitung_anteil")]
    pub head_share: f64,
    #[serde(rename = "wirkung")]
    pub effect: RawBranchEffect,
    #[serde(rename = "untergrenzen")]
    pub floors: RawProductionFloors,
    #[serde(rename = "ki_gewicht")]
    pub ai_weight: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawBranchEffect {
    #[serde(rename = "stunden")]
    pub hours: f64,
    #[serde(rename = "ausschuss")]
    pub scrap: f64,
    #[serde(rename = "material")]
    pub material: f64,
}

/// Least values of the product of all production factors (MG5).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProductionFloors {
    #[serde(rename = "stunden")]
    pub hours: f64,
    #[serde(rename = "material")]
    pub material: f64,
    #[serde(rename = "ausschuss")]
    pub scrap: f64,
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
    /// Generation marks of successor models (B1), e.g. `[II, III]`; empty: none.
    #[serde(rename = "nachfolger", default)]
    pub successors: Vec<String>,
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
    #[serde(rename = "wirkungen", default)]
    pub effects: Vec<RawEventEffect>,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

/// An effect of a historical event (H1). Which fields a kind takes is checked when the
/// catalog is built.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawEventEffect {
    #[serde(rename = "art")]
    pub kind: RawEffectKind,
    #[serde(rename = "laender", default)]
    pub countries: Option<Vec<String>>,
    #[serde(rename = "gegen", default)]
    pub against: Option<Vec<String>>,
    #[serde(rename = "warengruppen", default)]
    pub goods_groups: Option<Vec<String>>,
    #[serde(rename = "konsum", default)]
    pub consumer: Option<f64>,
    #[serde(rename = "staat", default)]
    pub state: Option<f64>,
    #[serde(rename = "faktor", default)]
    pub factor: Option<f64>,
    #[serde(rename = "aufschlag", default)]
    pub surcharge: Option<f64>,
    #[serde(rename = "alle", default)]
    pub all: Option<bool>,
    #[serde(rename = "anteil", default)]
    pub share: Option<f64>,
    #[serde(rename = "nur_auslaendische", default)]
    pub foreign_only: Option<bool>,
    #[serde(rename = "entschaedigung", default)]
    pub compensation: Option<f64>,
    #[serde(rename = "einbruch", default)]
    pub drop: Option<f64>,
    /// `JJJJ-MM-TT`
    #[serde(rename = "bis", default)]
    pub until: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub enum RawEffectKind {
    #[serde(rename = "nachfrage")]
    Demand,
    #[serde(rename = "handelssperre")]
    Embargo,
    #[serde(rename = "zoll")]
    Tariff,
    #[serde(rename = "arbeitskraefte")]
    Labor,
    #[serde(rename = "produktion")]
    Production,
    #[serde(rename = "abschottung")]
    Closure,
    #[serde(rename = "zerstoerung")]
    Destruction,
    #[serde(rename = "enteignung")]
    Expropriation,
    #[serde(rename = "boersenkrach")]
    StockCrash,
}

impl RawEffectKind {
    /// The key in the data.
    pub fn key(self) -> &'static str {
        match self {
            Self::Demand => "nachfrage",
            Self::Embargo => "handelssperre",
            Self::Tariff => "zoll",
            Self::Labor => "arbeitskraefte",
            Self::Production => "produktion",
            Self::Closure => "abschottung",
            Self::Destruction => "zerstoerung",
            Self::Expropriation => "enteignung",
            Self::StockCrash => "boersenkrach",
        }
    }
}

/// The player as a person (`parameter/person.yaml`, PE2).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPerson {
    #[serde(rename = "alter_start")]
    pub start_age: RawStartAge,
    #[serde(rename = "familie")]
    pub family: RawFamily,
    #[serde(rename = "gruendung")]
    pub founding: RawFounding,
    #[serde(rename = "gehalt")]
    pub salary: RawPersonSalary,
    #[serde(rename = "darlehen")]
    pub loans: RawShareholderLoans,
    #[serde(rename = "lebensstil")]
    pub lifestyle: RawLifestyles,
    #[serde(rename = "einkommensteuer")]
    pub income_tax: RawCountrySeries,
    #[serde(default, rename = "einkommensteuer_stufen")]
    pub income_brackets: Vec<RawTaxBracket>,
    #[serde(default, rename = "kapitalertragsteuer")]
    pub capital_tax: Option<RawCountrySeries>,
    #[serde(rename = "sparzins")]
    pub savings_rate: RawCountrySeries,
    #[serde(rename = "erbe")]
    pub heirs: RawHeirs,
    #[serde(rename = "erbschaftsteuer")]
    pub inheritance_tax: RawCountrySeries,
    #[serde(default, rename = "privatleben")]
    pub private_life: Option<RawPrivateLife>,
    #[serde(default, rename = "wohlbefinden")]
    pub wellbeing: Option<RawWellbeing>,
    #[serde(default, rename = "anlagen")]
    pub assets: Option<RawAssets>,
    #[serde(default, rename = "annaeherung")]
    pub approximation: bool,
    #[serde(default, rename = "quelle")]
    pub source: Option<String>,
}

/// Safe investments, gold and funds of the person (PV2).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAssets {
    #[serde(rename = "festgeld")]
    pub deposit: RawDepositTerms,
    #[serde(rename = "staatsanleihen")]
    pub bonds: RawGovBonds,
    #[serde(rename = "gold")]
    pub gold: RawGold,
    #[serde(rename = "fonds")]
    pub funds: RawFunds,
    #[serde(default, rename = "immobilien")]
    pub property: Option<RawProperty>,
}

/// Private real estate (PV3).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProperty {
    #[serde(rename = "arten")]
    pub kinds: RawPropertyKinds,
    #[serde(rename = "nebenkosten")]
    pub costs: f64,
    #[serde(rename = "eigenkapital_min")]
    pub equity_min: f64,
    #[serde(rename = "hypothek_aufschlag")]
    pub mortgage_premium: f64,
    #[serde(rename = "tilgung_jahr")]
    pub repay_year: f64,
    #[serde(rename = "mietband")]
    pub rent_band: RawFloatSpan,
    #[serde(rename = "leerstand_steigung")]
    pub vacancy_slope: f64,
    #[serde(rename = "schaden")]
    pub damage: RawDamage,
    #[serde(rename = "sanierung")]
    pub renovation: RawRenovation,
    #[serde(rename = "werk_effekt")]
    pub site_effect: RawSiteEffect,
    #[serde(rename = "verkauf_abschlag")]
    pub sale_discount: f64,
    #[serde(rename = "notverkauf_abschlag")]
    pub distress_discount: f64,
    #[serde(rename = "zustandsgewicht")]
    pub condition_weight: f64,
    #[serde(rename = "angebot")]
    pub offers: RawPropertyOffers,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPropertyKinds {
    #[serde(rename = "wohnung")]
    pub apartment: RawPropertyKind,
    #[serde(rename = "mehrfamilienhaus")]
    pub multi: RawPropertyKind,
    #[serde(rename = "buero")]
    pub office: RawPropertyKind,
    #[serde(rename = "halle")]
    pub hall: RawPropertyKind,
    #[serde(rename = "grundstueck")]
    pub land: RawPropertyKind,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPropertyKind {
    #[serde(rename = "preis_jahreslohn")]
    pub price_years: f64,
    #[serde(rename = "rendite")]
    pub yield_: f64,
    #[serde(rename = "leerstand")]
    pub vacancy: f64,
    #[serde(rename = "instandhaltung")]
    pub upkeep: f64,
    #[serde(rename = "verfall")]
    pub decay: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFloatSpan {
    #[serde(rename = "von")]
    pub min: f64,
    #[serde(rename = "bis")]
    pub max: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawDamage {
    #[serde(rename = "chance")]
    pub chance: f64,
    #[serde(rename = "kosten")]
    pub cost: f64,
    #[serde(rename = "punkte")]
    pub points: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRenovation {
    #[serde(rename = "kosten")]
    pub cost: f64,
    #[serde(rename = "punkte")]
    pub points: f64,
    #[serde(rename = "monate")]
    pub months: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSiteEffect {
    #[serde(rename = "je_standort")]
    pub per_site: f64,
    #[serde(rename = "hoechstens")]
    pub most: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPropertyOffers {
    #[serde(rename = "monate")]
    pub months: u32,
    #[serde(rename = "spanne")]
    pub spread: RawFloatSpan,
    #[serde(rename = "guete")]
    pub quality: RawFloatSpan,
    #[serde(rename = "zustand")]
    pub condition: RawFloatSpan,
    #[serde(rename = "laender_max")]
    pub countries_max: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawMonthsSpan {
    #[serde(rename = "von")]
    pub min: u32,
    #[serde(rename = "bis")]
    pub max: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawDepositTerms {
    #[serde(rename = "aufschlag")]
    pub premium: f64,
    #[serde(rename = "laufzeit_monate")]
    pub months: RawMonthsSpan,
    #[serde(rename = "kuendigung_abschlag")]
    pub penalty: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawGovBonds {
    #[serde(rename = "aufschlag")]
    pub premium: f64,
    #[serde(rename = "laufzeit_monate")]
    pub months: RawMonthsSpan,
    #[serde(rename = "dauer_faktor")]
    pub duration_factor: f64,
    #[serde(rename = "spanne")]
    pub spread: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawGold {
    #[serde(rename = "preis_usd")]
    pub price: RawSeries,
    #[serde(rename = "krisen_beta")]
    pub crisis_beta: f64,
    #[serde(rename = "rauschen")]
    pub noise: f64,
    #[serde(rename = "rueckkehr")]
    pub reversion: f64,
    #[serde(rename = "spanne")]
    pub spread: f64,
    #[serde(rename = "gebuehr")]
    pub fee: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFunds {
    #[serde(rename = "gemanagt")]
    pub managed: RawFundTerms,
    #[serde(rename = "index")]
    pub index: RawFundTerms,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFundTerms {
    #[serde(rename = "ab")]
    pub from: i32,
    #[serde(rename = "kosten")]
    pub cost: f64,
    #[serde(rename = "aufschlag")]
    pub load: f64,
    #[serde(rename = "gebuehr")]
    pub fee: f64,
}

/// Background, time budget and health of the person (PV1b).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawWellbeing {
    #[serde(rename = "hintergruende")]
    pub backgrounds: std::collections::BTreeMap<String, RawBackground>,
    #[serde(rename = "deckel")]
    pub cap: f64,
    #[serde(rename = "erfahrung_je_jahr")]
    pub experience_per_year: f64,
    #[serde(rename = "zeit")]
    pub time: RawTimeModel,
    #[serde(rename = "gesundheit")]
    pub health: RawHealthModel,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawBackground {
    #[serde(default, rename = "forschung")]
    pub research: f64,
    #[serde(default, rename = "kreditzins")]
    pub loan_rate: f64,
    #[serde(default, rename = "ordergebuehr")]
    pub order_fee: f64,
    #[serde(default, rename = "loyalitaet")]
    pub loyalty: f64,
    #[serde(default, rename = "ansehen")]
    pub standing: f64,
    #[serde(default, rename = "netzwerk")]
    pub network: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTimeModel {
    #[serde(rename = "standard")]
    pub default: RawTimeBudget,
    #[serde(rename = "mindest_firma")]
    pub min_firm: RawMinFirm,
    #[serde(rename = "verzoegerung_tage")]
    pub delay_days: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTimeBudget {
    #[serde(rename = "firma")]
    pub firm: u32,
    #[serde(rename = "familie")]
    pub family: u32,
    #[serde(rename = "netzwerk")]
    pub network: u32,
    #[serde(rename = "erholung")]
    pub rest: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawMinFirm {
    #[serde(rename = "basis")]
    pub base: f64,
    #[serde(rename = "je_firma")]
    pub per_company: f64,
    #[serde(rename = "hoechstens")]
    pub most: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawHealthModel {
    #[serde(rename = "start")]
    pub start: f64,
    #[serde(rename = "altern_ab")]
    pub aging_from: u32,
    #[serde(rename = "altern_je_jahr")]
    pub aging_per_year: f64,
    #[serde(rename = "ueberlastung_ab")]
    pub overload_from: f64,
    #[serde(rename = "ueberlastung_verlust")]
    pub overload_loss: f64,
    #[serde(rename = "erholung_gewinn")]
    pub rest_gain: f64,
    /// Gain per month by lifestyle, from modest to luxurious.
    #[serde(rename = "lebensstil_gewinn")]
    pub lifestyle_gain: [f64; 4],
    #[serde(rename = "vorsorge_gewinn")]
    pub care_gain: f64,
    #[serde(rename = "gesund_ab")]
    pub healthy_from: f64,
    #[serde(rename = "sterblichkeit_null")]
    pub mortality_at_zero: f64,
    #[serde(rename = "krank_unter")]
    pub ill_below: f64,
    #[serde(rename = "krank_chance")]
    pub ill_chance: f64,
    #[serde(rename = "krank_monate")]
    pub ill_months: u32,
}

/// A bracket of the income tax (PV1).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTaxBracket {
    #[serde(rename = "ab")]
    pub from: f64,
    #[serde(rename = "faktor")]
    pub factor: f64,
}

/// What the person can do privately: look for a partner, private schools, health care.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPrivateLife {
    #[serde(rename = "partnersuche")]
    pub partner: RawPartnerSearch,
    #[serde(rename = "privatschule")]
    pub school: RawPrivateSchool,
    #[serde(rename = "vorsorge")]
    pub care: RawHealthCare,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPartnerSearch {
    #[serde(rename = "kosten_monat")]
    pub cost: f64,
    #[serde(rename = "chance_monat")]
    pub chance: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPrivateSchool {
    #[serde(rename = "kosten_monat")]
    pub cost: f64,
    #[serde(rename = "ausbildung")]
    pub education: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawHealthCare {
    #[serde(rename = "kosten_monat")]
    pub cost: f64,
    #[serde(rename = "sterblichkeit")]
    pub mortality: f64,
}

/// Heirs of the person (PE6).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawHeirs {
    #[serde(rename = "neffe_alter")]
    pub nephew_age: RawAgeRange,
    #[serde(rename = "hinweis_ab")]
    pub hint_from: f64,
}

/// An age from–to in years.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAgeRange {
    #[serde(rename = "von")]
    pub from: f64,
    #[serde(rename = "bis")]
    pub to: f64,
}

/// Founding a company (PE3).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFounding {
    #[serde(rename = "kosten_anteil")]
    pub cost_share: f64,
    #[serde(rename = "kosten_mindestens_monatsloehne")]
    pub cost_min_months: f64,
    #[serde(rename = "einlage_vorschlag")]
    pub capital_suggestion: f64,
}

/// The person's salary as CEO (PE3).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPersonSalary {
    #[serde(rename = "hoechstens_mitgesellschafter")]
    pub co_owner_max: f64,
}

/// Loans of the person to its companies (PE3).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawShareholderLoans {
    #[serde(rename = "zins_hoechstens")]
    pub max_rate: f64,
    #[serde(rename = "jahre_hoechstens")]
    pub max_years: f64,
}

/// The four levels of lifestyle (PE3).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLifestyles {
    pub standard: String,
    #[serde(rename = "wechsel_monate")]
    pub change_months: f64,
    #[serde(rename = "stufen")]
    pub levels: RawLifestyleLevels,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLifestyleLevels {
    #[serde(rename = "bescheiden")]
    pub modest: RawLifestyle,
    #[serde(rename = "buergerlich")]
    pub middle: RawLifestyle,
    #[serde(rename = "gehoben")]
    pub upscale: RawLifestyle,
    #[serde(rename = "luxurioes")]
    pub luxury: RawLifestyle,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLifestyle {
    #[serde(rename = "kosten")]
    pub cost: f64,
    #[serde(rename = "zins")]
    pub interest: f64,
    #[serde(rename = "gehaltsforderung")]
    pub salary_demand: f64,
    #[serde(rename = "ausbildung")]
    pub education: f64,
    #[serde(rename = "sterblichkeit")]
    pub mortality: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawStartAge {
    pub standard: f64,
    #[serde(rename = "von")]
    pub min: f64,
    #[serde(rename = "bis")]
    pub max: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFamily {
    #[serde(rename = "kinder_ab")]
    pub children_from: f64,
    #[serde(rename = "kinder_bis")]
    pub children_until: f64,
    #[serde(rename = "kinder_hoechstens")]
    pub children_max: f64,
    #[serde(rename = "kinder_chance_jahr")]
    pub child_chance: f64,
    #[serde(rename = "managerkarte_ab")]
    pub card_age: f64,
}

/// Age, retirement and death of the managers (`parameter/lebenslauf.yaml`, PE1).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLife {
    #[serde(rename = "eintrittsalter")]
    pub entry_age: RawEntryAges,
    #[serde(rename = "ebene_nach_staerke")]
    pub level_strength: RawLevelStrength,
    #[serde(rename = "erfahrung")]
    pub experience: RawAgeExperience,
    #[serde(rename = "risiko")]
    pub risk: RawAgeRisk,
    #[serde(rename = "abbau")]
    pub decline: RawAgeDecline,
    #[serde(rename = "ruhestand")]
    pub retirement: RawRetirement,
    #[serde(rename = "sterbetafel")]
    pub mortality: RawMortality,
    #[serde(rename = "ruhestandsalter")]
    pub retirement_age: RawCountrySeries,
    #[serde(rename = "lebenserwartung")]
    pub life_expectancy: RawCountrySeries,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawEntryAges {
    #[serde(rename = "standort")]
    pub site: RawAgeSpan,
    #[serde(rename = "land")]
    pub country: RawAgeSpan,
    #[serde(rename = "kontinent")]
    pub continent: RawAgeSpan,
    #[serde(rename = "vorstand")]
    pub board: RawAgeSpan,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAgeSpan {
    #[serde(rename = "von")]
    pub min: f64,
    #[serde(rename = "bis")]
    pub max: f64,
    #[serde(rename = "mittel")]
    pub mean: f64,
    #[serde(rename = "streuung")]
    pub spread: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLevelStrength {
    #[serde(rename = "land")]
    pub country: f64,
    #[serde(rename = "kontinent")]
    pub continent: f64,
    #[serde(rename = "vorstand")]
    pub board: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAgeExperience {
    pub jung_bis: f64,
    pub jung: f64,
    pub alt_ab: f64,
    pub alt: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAgeRisk {
    #[serde(rename = "ab")]
    pub from: f64,
    #[serde(rename = "je_jahr")]
    pub per_year: f64,
    #[serde(rename = "hoechstens")]
    pub max: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAgeDecline {
    #[serde(rename = "ab")]
    pub from: f64,
    pub chance: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRetirement {
    #[serde(rename = "abweichung")]
    pub spread: f64,
    #[serde(rename = "vorwarnung_monate")]
    pub warning_months: f64,
    #[serde(rename = "verlaengerung_jahre_max")]
    pub extension_years_max: f64,
    #[serde(rename = "verlaengerung_aufschlag")]
    pub extension_raise: f64,
    /// The retirement age of the year in which a manager reaches this age applies to him.
    #[serde(rename = "bezugsalter")]
    pub reference_age: f64,
    /// Ages from which the chance to agree falls, and at which it reaches nothing.
    #[serde(rename = "zusage_alter")]
    pub acceptance_ages: [f64; 2],
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawMortality {
    #[serde(rename = "ab")]
    pub from: f64,
    pub chance: f64,
    #[serde(rename = "verdopplung_jahre")]
    pub doubling_years: f64,
}

/// A value per country and year: a series for all countries and own series for some.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawCountrySeries {
    #[serde(rename = "standard")]
    pub default: RawSeries,
    #[serde(rename = "laender", default)]
    pub countries: BTreeMap<String, RawSeries>,
}

/// Parameters of the events' effects (`parameter/ereignisse.yaml`, H1).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawEventModel {
    #[serde(rename = "staatsbetrieb")]
    pub state_company: RawStateCompany,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawStateCompany {
    #[serde(rename = "betriebskapital_anteil")]
    pub working_capital_share: f64,
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

/// Headquarters and central departments (`parameter/zentrale.yaml`, ZA1–ZA3).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawCentral {
    #[serde(rename = "hauptsitz")]
    pub headquarters: RawHeadquarters,
    #[serde(rename = "abteilungen", default)]
    pub departments: Vec<RawDepartment>,
    #[serde(rename = "genauigkeit", default)]
    pub accuracy: f64,
    #[serde(rename = "umschuldung", default)]
    pub refinance: Option<RawRefinance>,
    #[serde(rename = "trefferquote", default)]
    pub hit_rate: Option<RawHitRate>,
    #[serde(rename = "ki", default)]
    pub ai: Option<RawCentralAi>,
    #[serde(rename = "stadt", default)]
    pub city: Option<RawHqCity>,
    #[serde(rename = "strategie_last", default)]
    pub strategy_load: Option<RawStrategyLoad>,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

/// The workload of the strategy department (MG9).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawStrategyLoad {
    #[serde(rename = "je_branche")]
    pub per_branch: f64,
    #[serde(rename = "je_land")]
    pub per_country: f64,
    #[serde(rename = "je_beteiligung")]
    pub per_participation: f64,
}

/// The city of the headquarters (W2).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawHqCity {
    #[serde(rename = "akademiker_konzentration")]
    pub academics_concentration: f64,
    #[serde(rename = "anteil_zentralen")]
    pub hq_share: f64,
    #[serde(rename = "buero_bezug_einwohner")]
    pub office_reference_population: f64,
    #[serde(rename = "buero_elastizitaet")]
    pub office_elasticity: f64,
    #[serde(rename = "umzug_im_land")]
    pub move_within_country: f64,
    #[serde(rename = "einwohner_jahr")]
    pub population_year: i32,
}

/// Central departments and headquarters of the AI companies (ZA4).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawCentralAi {
    #[serde(rename = "anteil_umsatz")]
    pub revenue_share: RawSpan,
    #[serde(rename = "reihenfolge", default)]
    pub order: Vec<String>,
    #[serde(rename = "mindestlast", default)]
    pub min_load: BTreeMap<String, f64>,
    #[serde(rename = "sitz")]
    pub seat: RawCentralAiSeat,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawCentralAiSeat {
    #[serde(rename = "anteil_umsatz_min")]
    pub revenue_share_min: f64,
    #[serde(rename = "bip_anteil_min")]
    pub gdp_share_min: f64,
    #[serde(rename = "amortisation_jahre")]
    pub payback_years: f64,
    #[serde(rename = "sperre_jahre")]
    pub lock_years: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawHeadquarters {
    #[serde(rename = "verlegung_monate")]
    pub months: u32,
    #[serde(rename = "kosten_grund_usd")]
    pub cost_base_usd: f64,
    #[serde(rename = "kosten_je_angestelltem_usd")]
    pub cost_per_employee_usd: f64,
    #[serde(rename = "mitziehen")]
    pub moving_share: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawDepartment {
    pub id: String,
    #[serde(rename = "bereich")]
    pub function: String,
    #[serde(rename = "lohngruppe")]
    pub labor_group: String,
    #[serde(rename = "buero_usd")]
    pub office_usd: f64,
    #[serde(rename = "faelle")]
    pub cases: f64,
    #[serde(rename = "wirkung")]
    pub effect: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRefinance {
    #[serde(rename = "mindestvorteil")]
    pub min_advantage: f64,
    #[serde(rename = "gebuehr")]
    pub fee: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawHitRate {
    #[serde(rename = "bewertung_monate")]
    pub months: u32,
    #[serde(rename = "mittelwert")]
    pub mean: f64,
    #[serde(rename = "vorgewicht")]
    pub prior: f64,
    pub k: f64,
}

/// Start-ups (`parameter/startups.yaml`, SU1–SU3).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawVentures {
    #[serde(rename = "bezeichnungen")]
    pub labels: Vec<RawVentureLabel>,
    #[serde(rename = "je_jahr")]
    pub per_year: f64,
    #[serde(rename = "anteil_neu")]
    pub new_share: f64,
    #[serde(rename = "vorlauf_jahre_max")]
    pub lead_years_max: u32,
    #[serde(rename = "haeufigkeiten")]
    pub frequencies: Vec<RawVentureFrequency>,
    #[serde(rename = "haeufigkeit_standard")]
    pub default_frequency: String,
    #[serde(rename = "phasen")]
    pub phases: Vec<RawVenturePhase>,
    #[serde(rename = "bezug_bip_je_kopf_usd")]
    pub reference_gdp_usd: f64,
    #[serde(rename = "kapital_faktor_min")]
    pub capital_factor_min: f64,
    #[serde(rename = "kapital_faktor_max")]
    pub capital_factor_max: f64,
    #[serde(rename = "vorlauf_kapital")]
    pub lead_capital: f64,
    #[serde(rename = "vorlauf_chance")]
    pub lead_chance: f64,
    pub chance_min: f64,
    #[serde(rename = "investoren_chance_monat")]
    pub investor_chance: f64,
    #[serde(rename = "frist_monate")]
    pub deadline_months: u32,
    #[serde(rename = "beteiligung")]
    pub stakes: RawVentureStakes,
    #[serde(rename = "unschaerfe")]
    pub blur: f64,
    #[serde(rename = "stufe_mittel_ab")]
    pub level_medium_from: f64,
    #[serde(rename = "stufe_hoch_ab")]
    pub level_high_from: f64,
    #[serde(rename = "aufbewahren_jahre")]
    pub keep_years: u32,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

/// Stakes in start-ups (SU2).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawVentureStakes {
    #[serde(rename = "erfolg_faktor")]
    pub success_factor: f64,
    #[serde(rename = "kauf_aufschlag")]
    pub buy_premium: f64,
    #[serde(rename = "verkauf_abschlag")]
    pub sale_discount: f64,
    #[serde(rename = "foerderung_wirkung")]
    pub grant_effect: f64,
    #[serde(rename = "sperrminoritaet")]
    pub blocking: f64,
    #[serde(rename = "mehrheit")]
    pub majority: f64,
    #[serde(rename = "forschungsbonus")]
    pub research_bonus: f64,
    pub chance_max: f64,
    #[serde(rename = "lenkung")]
    pub pace: RawVenturePace,
    #[serde(rename = "rendite_mindest")]
    pub min_return: f64,
    #[serde(rename = "einsatz_kasse")]
    pub cash_share: f64,
    #[serde(rename = "ausgruendung_fortschritt_min")]
    pub spin_off_progress_min: f64,
    #[serde(rename = "ki")]
    pub ai: RawVentureAi,
}

/// How AI companies take part in start-ups (SU3).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawVentureAi {
    #[serde(rename = "pruefen_chance")]
    pub check_chance: f64,
    #[serde(rename = "kasse_min_usd")]
    pub cash_min_usd: f64,
    #[serde(rename = "einsatz_kasse")]
    pub cash_share: f64,
    #[serde(rename = "rendite_mindest")]
    pub min_return: f64,
    #[serde(rename = "uebernahme_chance_min")]
    pub takeover_chance_min: f64,
    #[serde(rename = "uebernahme_anteil_min")]
    pub takeover_share_min: f64,
    #[serde(rename = "uebernahme_kasse")]
    pub takeover_cash: f64,
    #[serde(rename = "ausgruenden_chance")]
    pub spin_off_chance: f64,
    #[serde(rename = "ausgruenden_verkauf")]
    pub spin_off_sale: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawVenturePace {
    #[serde(rename = "zuegig")]
    pub fast: RawPaceFactors,
    #[serde(rename = "gruendlich")]
    pub thorough: RawPaceFactors,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPaceFactors {
    #[serde(rename = "monate")]
    pub months: f64,
    pub chance: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawVentureFrequency {
    pub id: String,
    #[serde(rename = "faktor")]
    pub factor: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawVentureLabel {
    pub id: String,
    #[serde(rename = "ab")]
    pub from: i32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawVenturePhase {
    pub id: String,
    #[serde(rename = "monate")]
    pub months: u32,
    #[serde(rename = "kapital_usd")]
    pub capital_usd: f64,
    pub chance: f64,
    #[serde(rename = "bewertung")]
    pub valuation: f64,
}

/// A historical inventor of a technology (`startups/erfinder.yaml`, SU1).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawInventor {
    #[serde(rename = "technologie")]
    pub technology: String,
    pub name: String,
    #[serde(rename = "land")]
    pub country: String,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

/// Tariffs and trade embargoes (`parameter/zoelle.yaml`, W3).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTariffs {
    #[serde(rename = "standard")]
    pub default: RawSeries,
    #[serde(rename = "laender", default)]
    pub countries: BTreeMap<String, RawSeries>,
    #[serde(rename = "warengruppen", default)]
    pub groups: BTreeMap<String, f64>,
    /// Product → factor instead of its goods group's.
    #[serde(rename = "produkte", default)]
    pub products: BTreeMap<String, f64>,
    #[serde(rename = "zonen", default)]
    pub zones: Vec<RawTariffZone>,
    #[serde(rename = "sperren", default)]
    pub embargoes: Vec<RawEmbargo>,
    #[serde(rename = "dynamik")]
    pub dynamics: RawTariffDynamics,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTariffZone {
    pub id: String,
    #[serde(rename = "faktor")]
    pub factor: f64,
    /// Member → [entry year] or [entry year, exit year].
    #[serde(rename = "mitglieder")]
    pub members: BTreeMap<String, Vec<i32>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawEmbargo {
    #[serde(rename = "laender")]
    pub countries: Vec<String>,
    #[serde(rename = "von")]
    pub from: i32,
    #[serde(rename = "bis", default)]
    pub until: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTariffDynamics {
    #[serde(rename = "standardabweichung")]
    pub deviation: f64,
    #[serde(rename = "minimum")]
    pub min: f64,
    #[serde(rename = "maximum")]
    pub max: f64,
    #[serde(rename = "stufen")]
    pub levels: Vec<RawTariffLevel>,
    #[serde(rename = "standard")]
    pub default: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTariffLevel {
    pub id: String,
    #[serde(rename = "faktor")]
    pub factor: f64,
}

/// Supply contracts (`parameter/vertraege.yaml`, W4).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawContracts {
    #[serde(rename = "laufzeit_monate_max")]
    pub months_max: u32,
    #[serde(rename = "laufzeit_standard")]
    pub months_default: u32,
    #[serde(rename = "strafe_max")]
    pub penalty_max: f64,
    #[serde(rename = "strafe_standard")]
    pub penalty_default: f64,
    #[serde(rename = "kuendigung_monate")]
    pub cancel_months: u32,
    #[serde(rename = "angebot_tage")]
    pub proposal_days: u32,
    #[serde(rename = "aufbewahren_monate")]
    pub keep_months: u32,
    #[serde(rename = "ausfall_kuendigung_monate", default)]
    pub free_cancel_after: u32,
    #[serde(rename = "exklusiv_aufschlag", default)]
    pub exclusive_premium: f64,
    #[serde(rename = "nachverhandlung_menge_min", default)]
    pub renegotiate_quantity_min: f64,
    #[serde(rename = "nachverhandlung_verlaengerung_monate", default)]
    pub renegotiate_extension_min: u32,
    #[serde(rename = "partnerschaft", default)]
    pub partnership: Option<RawPartnership>,
    pub ki: RawContractAi,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

/// Partnership of supplier and buyer (`vertraege.partnerschaft`, LV2).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPartnership {
    #[serde(rename = "aufbau_monate")]
    pub build_months: u32,
    #[serde(rename = "verfall_monate")]
    pub decay_months: u32,
    #[serde(rename = "abzug_fehlmenge")]
    pub shortfall_drop: f64,
    #[serde(rename = "rabatt_max")]
    pub discount_max: f64,
    #[serde(rename = "umsatz_rabatt_max")]
    pub turnover_discount_max: f64,
    #[serde(rename = "umsatz_ab")]
    pub turnover_from: f64,
    #[serde(rename = "gesamt_max")]
    pub total_max: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawContractAi {
    #[serde(rename = "abschlag_verkauf")]
    pub sale_discount: f64,
    #[serde(rename = "abschlag_je_monat")]
    pub sale_discount_per_month: f64,
    #[serde(rename = "abschlag_max")]
    pub sale_discount_max: f64,
    #[serde(rename = "aufschlag_kauf")]
    pub purchase_premium: f64,
    #[serde(rename = "aufschlag_je_monat")]
    pub purchase_premium_per_month: f64,
    #[serde(rename = "aufschlag_max")]
    pub purchase_premium_max: f64,
    #[serde(rename = "anteil")]
    pub share: f64,
    #[serde(rename = "strafe_max")]
    pub penalty_max: f64,
    #[serde(rename = "angebot_chance")]
    pub proposal_chance: f64,
}

/// Logistics (`parameter/logistik.yaml`, W5).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLogistics {
    #[serde(rename = "staat")]
    pub state: RawStateFreight,
    #[serde(rename = "flotte")]
    pub fleet: RawFleet,
    #[serde(rename = "risiko")]
    pub risk: RawFreightRisk,
    pub ki: RawLogisticsAi,
    /// Internal supply by need: default and highest reach in days.
    #[serde(rename = "nachschub")]
    pub supply: RawSupplyDays,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawStateFreight {
    #[serde(rename = "aufschlag")]
    pub surcharge: f64,
    #[serde(rename = "risiko_faktor")]
    pub risk_factor: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFleet {
    #[serde(rename = "marge_frachtmarkt")]
    pub market_margin: f64,
    #[serde(rename = "auslastung")]
    pub load: f64,
    #[serde(rename = "unterhalt_anteil")]
    pub upkeep_share: f64,
    #[serde(rename = "nutzungsdauer_jahre")]
    pub life_years: f64,
    #[serde(rename = "verkauf_anteil")]
    pub sale_share: f64,
    #[serde(rename = "vermietung_anteil")]
    pub rental_share: f64,
    #[serde(rename = "vermietung_markt_anteil")]
    pub rental_market_share: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFreightRisk {
    pub land: RawSeries,
    #[serde(rename = "see")]
    pub sea: RawSeries,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSupplyDays {
    #[serde(rename = "tage_standard")]
    pub default: f64,
    #[serde(rename = "tage_max")]
    pub max: f64,
    /// How fast the learned demand of an offer follows the sales of a day with goods.
    #[serde(rename = "glaettung")]
    pub learn_rate: f64,
    /// A sold-out day counts this much more (the true demand was higher).
    #[serde(rename = "ausverkauft_aufschlag")]
    pub sold_out_premium: f64,
    /// Without any sales yet: this share of the country's demand.
    #[serde(rename = "start_anteil")]
    pub start_share: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLogisticsAi {
    #[serde(rename = "anteil")]
    pub share: f64,
    #[serde(rename = "kasse_anteil")]
    pub cash_share: f64,
}

/// Subsidiaries (`parameter/tochterfirmen.yaml`, W6).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSubsidiaries {
    #[serde(rename = "mindestkapital_usd")]
    pub min_capital_usd: f64,
    #[serde(rename = "gruendungskosten_usd")]
    pub founding_cost_usd: f64,
    #[serde(rename = "geschaeftsfuehrung")]
    pub management: RawSubsidiaryManagement,
    #[serde(rename = "logistik")]
    pub logistics: RawSubsidiaryLogistics,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSubsidiaryManagement {
    #[serde(rename = "kompetenz")]
    pub competence: f64,
    #[serde(rename = "aggressivitaet")]
    pub aggressiveness: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSubsidiaryLogistics {
    #[serde(rename = "kasse_anteil")]
    pub cash_share: f64,
    #[serde(rename = "rendite_min")]
    pub min_return: f64,
}

/// Stock market (`parameter/boerse.yaml`, K1).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawStockMarket {
    #[serde(rename = "bewertung")]
    pub valuation: RawValuation,
    #[serde(rename = "stimmung")]
    pub sentiment: RawSentiment,
    #[serde(rename = "traegheit")]
    pub inertia: f64,
    #[serde(rename = "rauschen")]
    pub noise: f64,
    #[serde(rename = "boersengang")]
    pub ipo: RawIpo,
    #[serde(rename = "handel")]
    pub trading: RawTrading,
    #[serde(rename = "uebernahme")]
    pub takeover: RawTakeover,
    #[serde(rename = "rueckkauf")]
    pub buyback: RawBuyback,
    pub start: RawStockStart,
    pub ki: RawStockAi,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawValuation {
    #[serde(rename = "gewicht_buchwert")]
    pub book_weight: f64,
    #[serde(rename = "kgv")]
    pub pe: f64,
    #[serde(rename = "boden_buchwert")]
    pub book_floor: f64,
    #[serde(rename = "rendite_annahme")]
    pub assumed_return: f64,
    #[serde(rename = "gewinn_monate")]
    pub earnings_months: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSentiment {
    #[serde(rename = "schwankung")]
    pub volatility: f64,
    #[serde(rename = "rueckkehr")]
    pub reversion: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawIpo {
    #[serde(rename = "eigenkapital_min_usd")]
    pub equity_min_usd: f64,
    #[serde(rename = "anteil_max")]
    pub share_max: f64,
    #[serde(rename = "abschlag")]
    pub discount: f64,
    #[serde(rename = "kosten_anteil")]
    pub cost_share: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawDividends {
    #[serde(rename = "reserve_monate_min")]
    pub reserve_months_min: f64,
    #[serde(rename = "konzern_anteil")]
    pub group_share: f64,
    #[serde(rename = "quote_min")]
    pub payout_min: f64,
    #[serde(rename = "quote_max")]
    pub payout_max: f64,
    #[serde(rename = "kasse_monate")]
    pub cash_months: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTrading {
    #[serde(rename = "aufschlag")]
    pub premium: f64,
    #[serde(rename = "abschlag")]
    pub discount: f64,
    #[serde(rename = "preiswirkung")]
    pub impact: f64,
    #[serde(rename = "anteil_max")]
    pub share_max: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawStockStart {
    #[serde(rename = "eigenkapital_min_usd")]
    pub equity_min_usd: f64,
    #[serde(rename = "streubesitz")]
    pub free_float: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawStockAi {
    #[serde(rename = "boersengang_chance")]
    pub ipo_chance: f64,
    #[serde(rename = "boersengang_anteil")]
    pub ipo_share: f64,
    #[serde(rename = "depot_anteil_kasse")]
    pub portfolio_cash_share: f64,
    #[serde(rename = "depot_anteil_max")]
    pub portfolio_stake_max: f64,
    #[serde(rename = "unterbewertung")]
    pub undervaluation: f64,
    #[serde(rename = "uebernahme_chance")]
    pub takeover_chance: f64,
    #[serde(rename = "uebernahme_kasse_anteil")]
    pub takeover_cash_share: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTakeover {
    #[serde(rename = "aufschlag")]
    pub premium: f64,
    #[serde(rename = "kosten_anteil")]
    pub cost_share: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawBuyback {
    #[serde(rename = "anteil_max")]
    pub share_max: f64,
}

/// Corporate bonds (`parameter/anleihen.yaml`, K2).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawBonds {
    #[serde(rename = "eigenkapital_min_usd")]
    pub equity_min_usd: f64,
    #[serde(rename = "volumen_min_usd")]
    pub volume_min_usd: f64,
    #[serde(rename = "laufzeit_jahre")]
    pub term_years: RawBondTerm,
    #[serde(rename = "kosten_anteil")]
    pub cost_share: f64,
    #[serde(rename = "rueckkauf_aufschlag")]
    pub redeem_premium: f64,
    #[serde(rename = "gewinn_monate")]
    pub earnings_months: u32,
    #[serde(rename = "bonitaet")]
    pub grades: Vec<RawBondGrade>,
    pub ki: RawBondAi,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawBondTerm {
    pub min: u32,
    pub max: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawBondGrade {
    #[serde(rename = "stufe")]
    pub id: String,
    #[serde(rename = "verschuldung_max")]
    pub debt_ratio_max: f64,
    #[serde(rename = "zinsdeckung_min")]
    pub coverage_min: f64,
    #[serde(rename = "aufschlag")]
    pub spread: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawBondAi {
    #[serde(rename = "laufzeit_jahre")]
    pub term_years: u32,
    #[serde(rename = "vorteil_min")]
    pub advantage_min: f64,
}

/// The player's banks (`parameter/bank.yaml`, K4).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawBank {
    #[serde(rename = "einlagen")]
    pub deposits: RawBankDeposits,
    #[serde(rename = "mindestreserve")]
    pub reserve: f64,
    pub start: RawBankStart,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawBankDeposits {
    #[serde(rename = "hebel_max")]
    pub leverage_max: f64,
    #[serde(rename = "aufschlag_neutral")]
    pub neutral_spread: f64,
    #[serde(rename = "elastizitaet")]
    pub elasticity: f64,
    #[serde(rename = "anpassung")]
    pub adjustment: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawBankStart {
    #[serde(rename = "einlagen_aufschlag")]
    pub deposit_spread: f64,
    #[serde(rename = "kreditnachlass")]
    pub loan_discount: f64,
    #[serde(rename = "verschuldung_max")]
    pub max_debt_ratio: f64,
}

/// Environment and regulation (`parameter/umwelt.yaml`, H2).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawEnvironment {
    /// Levels of retrofitting, the first first.
    #[serde(rename = "nachruestung")]
    pub retrofit: Vec<RawRetrofitLevel>,
    /// CO2 price in USD per t by country and year.
    #[serde(rename = "co2_preis")]
    pub co2_price: RawCountrySeries,
    #[serde(rename = "kartell")]
    pub antitrust: RawAntitrust,
    #[serde(rename = "markenbild")]
    pub image: RawImage,
    #[serde(default, rename = "annaeherung")]
    pub approximation: bool,
    #[serde(default, rename = "quelle")]
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRetrofitLevel {
    #[serde(rename = "ab")]
    pub from_year: i32,
    #[serde(rename = "minderung")]
    pub reduction: f64,
    #[serde(rename = "kosten_anteil")]
    pub cost_share: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAntitrust {
    #[serde(rename = "marktanteil_max")]
    pub share_max: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawImage {
    #[serde(rename = "gewicht")]
    pub weight: f64,
}

/// A regulation of `regulierungen` (H2).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRegulation {
    pub id: String,
    #[serde(rename = "datum")]
    pub date: String,
    #[serde(rename = "laender")]
    pub countries: Vec<String>,
    #[serde(rename = "art")]
    pub kind: String,
    /// `auflage`: the level required and the months to reach it.
    #[serde(default, rename = "stufe")]
    pub level: Option<u32>,
    #[serde(default, rename = "frist_monate")]
    pub months: Option<u32>,
    /// `arbeitsschutz`: share more on the hourly wages.
    #[serde(default, rename = "lohnaufschlag")]
    pub wage_surcharge: Option<f64>,
    /// `verbot`: the products and what is banned.
    #[serde(default, rename = "produkte")]
    pub products: Vec<String>,
    #[serde(default, rename = "herstellung")]
    pub production: bool,
    #[serde(default, rename = "verkauf")]
    pub sales: bool,
    #[serde(default, rename = "annaeherung")]
    pub approximation: bool,
    #[serde(default, rename = "quelle")]
    pub source: Option<String>,
}

/// Tenders for machines (`parameter/maschinenangebote.yaml`, MAA).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawMachineOffers {
    #[serde(rename = "laufzeit_tage")]
    pub tender_days: u32,
    #[serde(rename = "gueltig_tage")]
    pub valid_days: u32,
    #[serde(rename = "angebote_je_woche")]
    pub offers_per_week: Vec<u32>,
    #[serde(rename = "verteilung")]
    pub distribution: RawOfferDistribution,
    #[serde(rename = "wirkung")]
    pub effect: RawOfferEffect,
    #[serde(rename = "mengenrabatt_anteil")]
    pub volume_discount: f64,
    #[serde(rename = "ki_schwelle")]
    pub ai_threshold: f64,
    #[serde(rename = "annaeherung", default)]
    pub approximation: bool,
    #[serde(rename = "quelle", default)]
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawOfferDistribution {
    #[serde(rename = "normal_anteil")]
    pub normal_share: f64,
    #[serde(rename = "abweichung")]
    pub deviation: f64,
    #[serde(rename = "ausreisser")]
    pub outlier: Vec<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawOfferEffect {
    #[serde(rename = "ausstoss")]
    pub output: f64,
    #[serde(rename = "personal")]
    pub labor: f64,
    #[serde(rename = "vorprodukte")]
    pub inputs: f64,
    #[serde(rename = "ausschuss")]
    pub scrap: f64,
    #[serde(rename = "preis_leistung")]
    pub price_performance: f64,
    #[serde(rename = "preis_lage")]
    pub price_position: f64,
    #[serde(rename = "flaeche_streuung")]
    pub area_spread: f64,
    #[serde(rename = "bauzeit")]
    pub build_time: f64,
    #[serde(rename = "rauschen")]
    pub noise: f64,
    #[serde(rename = "grenze")]
    pub limit: f64,
}

/// A fictitious machine builder (`data/maschinenbauer.yaml`, MAA).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawMachineBuilder {
    pub id: String,
    #[serde(rename = "land")]
    pub country: String,
    #[serde(rename = "tendenz")]
    pub tendency: f64,
    #[serde(rename = "ab")]
    pub from: i32,
    #[serde(rename = "bis", default)]
    pub until: Option<i32>,
}
