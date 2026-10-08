//! Checks the raw entries and compiles them into the catalog.
//!
//! All entries are checked even after the first error so that one run reports as
//! many problems as possible. Invalid references resolve to a placeholder ID; the
//! catalog is discarded as soon as any error was reported.

use std::collections::{BTreeMap, BTreeSet};

use wsim_core::EARLIEST_START_YEAR;
use wsim_core::catalog::{
    Branch, Catalog, ConsumerDemand, ConsumptionType, Continent, Deposit, Facility, GoodsGroup,
    LaborGroup, NeedClass, Product, ProductKind, Provenance, Qualification, Recipe, SiteType,
    Specialization, StateDemand, StateMarketOffer, Technology, TransportClass, Unit, Usage,
};
use wsim_core::health;
use wsim_core::ids::{Id, LaborGroupId, ProductId, RecipeId, TechnologyId};
use wsim_core::money::Money;
use wsim_core::time_series::TimeSeries;

use crate::messages;

mod ai;
mod central;
mod contracts;
mod countries;
mod currencies;
mod deals;
mod logistics;
mod management;
mod milestones;
mod names;
mod plots;
mod production;
mod tariffs;
mod ventures;
use crate::raw::{
    RawConsumerDemand, RawNeedClass, RawPerKind, RawProduct, RawProductKind, RawSiteType,
    RawStateMarket, RawUsage,
};
use crate::read::{Ctx, Entry, Loc, RawData};
use crate::suggest;
use crate::texts::TextIndex;

/// Years allowed for country time series and market availability.
const GAME_YEARS: (i32, i32) = (1800, 2100);
/// Product trees (Lastenheft §17.2): inputs per recipe and levels from raw material to
/// product; more than the typical levels only for products marked as very complex.
/// Design rules of the data, not parameters of the simulation.
const MAX_INPUTS: usize = 4;
const MAX_LEVELS: usize = 6;
const TYPICAL_LEVELS: usize = 4;
/// Years allowed for historical facts such as inventions and discoveries.
const HISTORY_YEARS: (i32, i32) = (-10_000, 2100);

#[derive(Clone, Copy)]
enum KeyFormat {
    /// `roheisen`, `fachkraft`
    Snake,
    /// ISO 3166 alpha-3, `DEU`
    CountryCode,
}

/// Accepted keys of one kind of entry.
struct Keys {
    /// German name of the kind for messages, e.g. "Produkt".
    kind: &'static str,
    /// Prefix of the display texts, e.g. "produkt".
    text_prefix: Option<&'static str>,
    index: BTreeMap<String, usize>,
    locations: Vec<Loc>,
    /// Keys of entries that failed to read; references to them are not reported.
    broken: BTreeSet<String>,
}

impl Keys {
    fn suggest(&self, key: &str) -> Option<&str> {
        suggest::closest(key, self.index.keys().map(String::as_str))
    }

    fn keys_in_order(&self) -> Vec<&str> {
        let mut keys: Vec<(&str, usize)> =
            self.index.iter().map(|(k, &i)| (k.as_str(), i)).collect();
        keys.sort_by_key(|&(_, i)| i);
        keys.into_iter().map(|(k, _)| k).collect()
    }
}

/// Registers the keys of `entries`; duplicates are reported and left out.
fn register<'r, T>(
    ctx: &mut Ctx,
    raw: &RawData,
    (section, kind, text_prefix): (&str, &'static str, Option<&'static str>),
    format: KeyFormat,
    entries: &'r [Entry<T>],
    key_of: impl Fn(&T) -> &str,
) -> (Keys, Vec<&'r Entry<T>>) {
    let broken = raw.broken.get(section).cloned().unwrap_or_default();
    let mut keys = Keys {
        kind,
        text_prefix,
        index: BTreeMap::new(),
        locations: Vec::new(),
        broken,
    };
    let mut accepted = Vec::new();
    for entry in entries {
        let key = key_of(&entry.value);
        let id_loc = entry.loc.field("id");
        match format {
            KeyFormat::Snake if !is_snake_key(key) => {
                ctx.error(&id_loc, messages::invalid_key(key))
            }
            KeyFormat::CountryCode if !is_country_code(key) => {
                ctx.error(&id_loc, messages::invalid_country_code(key));
            }
            _ => {}
        }
        if let Some(&first) = keys.index.get(key) {
            let first = ctx.describe(&keys.locations[first]);
            ctx.error(&id_loc, messages::duplicate_key(kind, key, &first));
            continue;
        }
        keys.index.insert(key.to_owned(), accepted.len());
        keys.locations.push(entry.loc.clone());
        accepted.push(entry);
    }
    (keys, accepted)
}

pub(crate) fn site_type(raw: RawSiteType) -> SiteType {
    match raw {
        RawSiteType::Extraction => SiteType::Extraction,
        RawSiteType::Factory => SiteType::Factory,
        RawSiteType::PowerPlant => SiteType::PowerPlant,
        RawSiteType::Warehouse => SiteType::Warehouse,
        RawSiteType::SalesOffice => SiteType::SalesOffice,
        RawSiteType::ResearchCenter => SiteType::ResearchCenter,
    }
}

fn is_snake_key(key: &str) -> bool {
    key.starts_with(|c: char| c.is_ascii_lowercase())
        && key
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

fn is_country_code(key: &str) -> bool {
    key.len() == 3 && key.chars().all(|c| c.is_ascii_uppercase())
}

/// Resolves a reference; reports unknown keys and returns a placeholder then.
fn resolve<I: Id>(ctx: &mut Ctx, keys: &Keys, key: &str, loc: &Loc) -> I {
    I::from_index(resolve_index(ctx, keys, key, loc))
}

/// Index of a key; unknown keys are reported and resolve to 0 (the catalog is then
/// discarded anyway).
fn resolve_index(ctx: &mut Ctx, keys: &Keys, key: &str, loc: &Loc) -> usize {
    match keys.index.get(key) {
        Some(&i) => i,
        None if keys.broken.contains(key) => 0,
        None => {
            ctx.error(
                loc,
                messages::unknown_reference(keys.kind, key, keys.suggest(key)),
            );
            0
        }
    }
}

fn positive(ctx: &mut Ctx, value: f64, loc: &Loc) -> f64 {
    if !(value > 0.0 && value.is_finite()) {
        ctx.error(loc, messages::not_positive(value));
    }
    value
}

fn non_negative(ctx: &mut Ctx, value: f64, loc: &Loc) -> f64 {
    if !(value >= 0.0 && value.is_finite()) {
        ctx.error(loc, messages::negative(value));
    }
    value
}

fn in_range(ctx: &mut Ctx, value: f64, min: f64, max: f64, loc: &Loc) -> f64 {
    if !(min..=max).contains(&value) {
        ctx.error(loc, messages::out_of_range(value, min, max));
    }
    value
}

/// One value per product kind, in the order of `ProductKind::index`, each checked by
/// `check`.
fn per_kind(
    ctx: &mut Ctx,
    values: &RawPerKind,
    loc: &Loc,
    check: impl Fn(&mut Ctx, f64, &Loc) -> f64,
) -> [f64; 5] {
    [
        check(ctx, values.raw_material, &loc.field("rohstoff")),
        check(ctx, values.semi_finished, &loc.field("halbzeug")),
        check(ctx, values.component, &loc.field("komponente")),
        check(ctx, values.end_product, &loc.field("endprodukt")),
        check(ctx, values.energy, &loc.field("energie")),
    ]
}

fn year(ctx: &mut Ctx, year: i32, (min, max): (i32, i32), loc: &Loc) -> i32 {
    if !(min..=max).contains(&year) {
        ctx.error(loc, messages::year_out_of_range(year, min, max));
    }
    year
}

fn money(ctx: &mut Ctx, usd: f64, loc: &Loc) -> Money {
    match Money::from_usd(usd).filter(|_| usd >= 0.0) {
        Some(money) => money,
        None => {
            ctx.error(loc, messages::money_out_of_range(usd));
            Money::ZERO
        }
    }
}

fn provenance(approximation: bool, source: Option<&String>) -> Provenance {
    Provenance {
        approximation,
        source: source.cloned(),
    }
}

fn time_series(ctx: &mut Ctx, values: &BTreeMap<i32, f64>, loc: &Loc) -> TimeSeries {
    if values.is_empty() {
        ctx.error(loc, messages::time_series_empty());
    }
    for (&y, &v) in values {
        let point = loc.field(&y.to_string());
        year(ctx, y, GAME_YEARS, &point);
        non_negative(ctx, v, &point);
    }
    let points = values.iter().map(|(&y, &v)| (y, v)).collect();
    TimeSeries::new(points)
        .unwrap_or_else(|_| TimeSeries::new(vec![(EARLIEST_START_YEAR, 0.0)]).expect("valid"))
}

struct Builder<'c, 'f> {
    ctx: &'c mut Ctx<'f>,
    catalog: Catalog,
}

/// Checks `raw` and builds the catalog. Returns `None` if any error was found.
pub(crate) fn build(
    ctx: &mut Ctx,
    raw: &RawData,
    texts: &TextIndex,
    all_files_read: bool,
) -> Option<Catalog> {
    let errors_before = ctx.report.errors().count();
    let mut b = Builder {
        ctx,
        catalog: Catalog::default(),
    };

    b.meta(raw);

    let (unit_keys, units) = register(
        b.ctx,
        raw,
        ("einheiten", "Einheit", Some("einheit")),
        KeyFormat::Snake,
        &raw.units,
        |e| &e.id,
    );
    let (continent_keys, continents) = register(
        b.ctx,
        raw,
        ("kontinente", "Kontinent", Some("kontinent")),
        KeyFormat::Snake,
        &raw.continents,
        |e| &e.id,
    );
    let (branch_keys, branches) = register(
        b.ctx,
        raw,
        ("branchen", "Branche", Some("branche")),
        KeyFormat::Snake,
        &raw.branches,
        |e| &e.id,
    );
    let (group_keys, groups) = register(
        b.ctx,
        raw,
        ("warengruppen", "Warengruppe", Some("warengruppe")),
        KeyFormat::Snake,
        &raw.goods_groups,
        |e| &e.id,
    );
    let (transport_keys, transport_classes) = register(
        b.ctx,
        raw,
        (
            "transportklassen",
            "Transportklasse",
            Some("transportklasse"),
        ),
        KeyFormat::Snake,
        &raw.transport_classes,
        |e| &e.id,
    );
    let (qualification_keys, qualifications) = register(
        b.ctx,
        raw,
        ("qualifikationen", "Qualifikation", Some("qualifikation")),
        KeyFormat::Snake,
        &raw.qualifications,
        |e| &e.id,
    );
    let (specialization_keys, specializations) = register(
        b.ctx,
        raw,
        ("fachrichtungen", "Fachrichtung", Some("fachrichtung")),
        KeyFormat::Snake,
        &raw.specializations,
        |e| &e.id,
    );
    let (country_keys, countries) = register(
        b.ctx,
        raw,
        ("laender", "Land", Some("land")),
        KeyFormat::CountryCode,
        &raw.countries,
        |e| &e.id,
    );
    let (product_keys, products) = register(
        b.ctx,
        raw,
        ("produkte", "Produkt", Some("produkt")),
        KeyFormat::Snake,
        &raw.products,
        |e| &e.id,
    );
    let (facility_keys, facilities) = register(
        b.ctx,
        raw,
        ("anlagen", "Anlage", Some("anlage")),
        KeyFormat::Snake,
        &raw.facilities,
        |e| &e.id,
    );
    let (recipe_keys, recipes) = register(
        b.ctx,
        raw,
        ("rezepte", "Rezept", Some("rezept")),
        KeyFormat::Snake,
        &raw.recipes,
        |e| &e.id,
    );
    let (technology_keys, technologies) = register(
        b.ctx,
        raw,
        ("technologien", "Technologie", Some("technologie")),
        KeyFormat::Snake,
        &raw.technologies,
        |e| &e.id,
    );
    let (deposit_keys, deposits) = register(
        b.ctx,
        raw,
        ("lagerstaetten", "Lagerstätte", Some("lagerstaette")),
        KeyFormat::Snake,
        &raw.deposits,
        |e| &e.id,
    );
    let (vehicle_keys, vehicles) = register(
        b.ctx,
        raw,
        ("verkehrsmittel", "Verkehrsmittel", Some("verkehrsmittel")),
        KeyFormat::Snake,
        &raw.vehicles,
        |e| &e.id,
    );

    for e in &units {
        let weight_kg = e
            .value
            .weight_kg
            .map(|w| non_negative(b.ctx, w, &e.loc.field("gewicht_kg")));
        b.catalog.units.insert(&e.value.id, Unit { weight_kg });
    }
    for e in &continents {
        b.catalog.continents.insert(&e.value.id, Continent);
    }
    for e in &branches {
        b.catalog.branches.insert(&e.value.id, Branch);
    }
    for e in &groups {
        b.catalog.goods_groups.insert(&e.value.id, GoodsGroup);
    }
    for e in &transport_classes {
        let cost_factor = positive(b.ctx, e.value.cost_factor, &e.loc.field("kostenfaktor"));
        b.catalog
            .transport_classes
            .insert(&e.value.id, TransportClass { cost_factor });
    }
    for e in &qualifications {
        let q = Qualification {
            rank: e.value.rank,
            has_specialization: e.value.has_specialization,
        };
        b.catalog.qualifications.insert(&e.value.id, q);
    }
    for e in &specializations {
        b.catalog
            .specializations
            .insert(&e.value.id, Specialization);
    }
    b.labor_groups();

    for e in &countries {
        let country = countries::country(
            b.ctx,
            &b.catalog,
            e,
            &continent_keys,
            &country_keys,
            &specialization_keys,
        );
        b.catalog.countries.insert(&e.value.id, country);
    }
    countries::check_neighbors(b.ctx, &b.catalog, &countries);
    b.catalog.production_model =
        production::production_model(b.ctx, raw, (&product_keys, &facility_keys, &recipe_keys));
    b.catalog.finance_model = production::finance_model(b.ctx, raw);
    let (market_model, media_keys) = production::market_model(b.ctx, raw);
    b.catalog.market_model = market_model;
    b.catalog.transport_model = production::transport_model(b.ctx, raw);
    b.catalog.research_model = production::research_model(
        b.ctx,
        &b.catalog,
        raw,
        (&qualification_keys, &branch_keys, &specialization_keys),
    );
    for e in &vehicles {
        let vehicle = production::vehicle(b.ctx, e, &transport_keys);
        b.catalog.vehicles.insert(&e.value.id, vehicle);
    }
    b.catalog.country_model = countries::country_model(
        b.ctx,
        &b.catalog,
        raw,
        (&country_keys, &qualification_keys, &specialization_keys),
    );
    let (currency_model, currency_keys) = currencies::currency_model(b.ctx, raw, &country_keys);
    b.catalog.currencies = currency_model;

    for e in &products {
        let refs = ProductRefs {
            units: &unit_keys,
            branches: &branch_keys,
            groups: &group_keys,
            transport: &transport_keys,
            products: &product_keys,
        };
        let product = b.product(e, &refs);
        b.catalog.products.insert(&e.value.id, product);
    }

    for e in &technologies {
        let v = &e.value;
        let year_loc = e.loc.field("erfindungsjahr");
        let invention_year = year(b.ctx, v.invention_year, HISTORY_YEARS, &year_loc);
        let prerequisites = v
            .prerequisites
            .iter()
            .enumerate()
            .map(|(i, p)| {
                resolve(
                    b.ctx,
                    &technology_keys,
                    p,
                    &e.loc.field("voraussetzungen").index(i),
                )
            })
            .collect();
        let effort_loc = e.loc.field("forschungsaufwand");
        let research_effort = v.research_effort.map(|r| positive(b.ctx, r, &effort_loc));
        if invention_year > EARLIEST_START_YEAR && research_effort.is_none() {
            b.ctx.error(
                &year_loc,
                messages::research_effort_missing(invention_year, EARLIEST_START_YEAR),
            );
        }
        if invention_year <= EARLIEST_START_YEAR && research_effort.is_some() {
            b.ctx.warning(
                &effort_loc,
                messages::research_effort_unused(invention_year, EARLIEST_START_YEAR),
            );
        }
        let technology = Technology {
            field: resolve(
                b.ctx,
                &specialization_keys,
                &v.field,
                &e.loc.field("fachgebiet"),
            ),
            invention_year,
            prerequisites,
            research_effort,
            provenance: provenance(v.approximation, v.source.as_ref()),
        };
        b.catalog.technologies.insert(&v.id, technology);
    }
    b.check_technology_graph(&technologies);

    for e in &facilities {
        let v = &e.value;
        let l = &e.loc;
        let facility = Facility {
            site_type: site_type(v.site_type),
            investment: money(b.ctx, v.investment_usd, &l.field("investition_usd")),
            build_days: v.build_days,
            runs_per_day: positive(b.ctx, v.runs_per_day, &l.field("kapazitaet_je_tag")),
            lifetime_years: {
                if v.lifetime_years == 0 {
                    b.ctx
                        .error(&l.field("lebensdauer_jahre"), messages::not_positive(0.0));
                }
                v.lifetime_years
            },
            maintenance_share: in_range(
                b.ctx,
                v.maintenance_share,
                0.0,
                1.0,
                &l.field("wartung_je_jahr"),
            ),
            automation_max: in_range(
                b.ctx,
                v.automation_max,
                0.0,
                1.0,
                &l.field("automatisierung_max"),
            ),
            technology: v
                .technology
                .as_ref()
                .map(|t| resolve(b.ctx, &technology_keys, t, &l.field("technologie"))),
            area_ha: v
                .area_ha
                .map(|a| positive(b.ctx, a, &l.field("flaeche_ha"))),
            provenance: provenance(v.approximation, v.source.as_ref()),
        };
        b.catalog.facilities.insert(&v.id, facility);
    }

    for e in &recipes {
        let recipe = b.recipe(e, &product_keys, &facility_keys, &technology_keys);
        b.catalog.recipes.insert(&e.value.id, recipe);
    }

    for e in &deposits {
        let v = &e.value;
        let l = &e.loc;
        let resource: ProductId = resolve(b.ctx, &product_keys, &v.resource, &l.field("rohstoff"));
        if product_keys.index.contains_key(&v.resource)
            && b.catalog.products.get(resource).kind != ProductKind::RawMaterial
        {
            b.ctx.error(
                &l.field("rohstoff"),
                messages::deposit_needs_raw_material(&v.resource),
            );
        }
        let reserve = match (v.renewable, v.reserve) {
            (false, None) => {
                b.ctx
                    .error(&l.field("id"), messages::deposit_reserve_missing());
                None
            }
            (true, Some(_)) => {
                b.ctx
                    .error(&l.field("vorrat"), messages::deposit_reserve_renewable());
                None
            }
            (_, reserve) => reserve.map(|r| non_negative(b.ctx, r, &l.field("vorrat"))),
        };
        let deposit = Deposit {
            country: resolve(b.ctx, &country_keys, &v.country, &l.field("land")),
            resource,
            reserve,
            discovered: v
                .discovered
                .map(|y| year(b.ctx, y, HISTORY_YEARS, &l.field("entdeckt"))),
            development_cost: money(
                b.ctx,
                v.development.investment_usd,
                &l.field("erschliessung").field("investition_usd"),
            ),
            development_days: v.development.days,
            max_output_per_year: positive(
                b.ctx,
                v.max_output_per_year,
                &l.field("foerderung_max_je_jahr"),
            ),
            cost_factor: positive(b.ctx, v.cost_factor, &l.field("foerderkosten_faktor")),
            provenance: provenance(v.approximation, v.source.as_ref()),
        };
        b.catalog.deposits.insert(&v.id, deposit);
    }

    let (ai_model, difficulty_keys) = ai::ai_model(b.ctx, raw);
    b.catalog.ai_model = ai_model;
    b.catalog.deal_model = deals::deal_model(b.ctx, &b.catalog, raw);
    b.catalog.plot_model = plots::plot_model(b.ctx, raw);
    b.catalog.name_groups = ai::name_groups(b.ctx, &b.catalog, raw, (&country_keys, &branch_keys));
    b.catalog.product_naming = names::product_naming(b.ctx, &b.catalog, raw, &group_keys);
    b.catalog.management = management::management(b.ctx, &b.catalog, raw);
    b.catalog.central = central::central_model(b.ctx, &b.catalog, raw);
    b.catalog.ventures = ventures::venture_model(b.ctx, &b.catalog, raw);
    b.catalog.tariffs = tariffs::tariff_model(b.ctx, &b.catalog, raw);
    b.catalog.contracts = contracts::contract_model(b.ctx, raw);
    b.catalog.logistics = logistics::logistics_model(b.ctx, raw);
    let (_, real_companies) = register(
        b.ctx,
        raw,
        ("reale_firmen", "reale Firma", None),
        KeyFormat::Snake,
        &raw.real_companies,
        |e| &e.id,
    );
    b.catalog.real_companies = ai::real_companies(
        b.ctx,
        &b.catalog,
        &real_companies,
        (&country_keys, &deposit_keys, &facility_keys, &recipe_keys),
    );

    let (event_keys, events) = register(
        b.ctx,
        raw,
        ("ereignisse", "Ereignis", Some("ereignis")),
        KeyFormat::Snake,
        &raw.events,
        |e| &e.id,
    );
    b.catalog.events = ai::events(b.ctx, &events, &country_keys, texts);

    let (milestone_keys, milestone_entries) = register(
        b.ctx,
        raw,
        ("etappen", "Etappe", Some("etappe")),
        KeyFormat::Snake,
        &raw.milestones,
        |e| &e.id,
    );
    milestones::milestones(b.ctx, &mut b.catalog, &milestone_entries, texts);

    b.check_product_sources(&products);
    // Only on a consistent catalog: unresolved references would point anywhere.
    if !b.ctx.report.has_errors() {
        b.check_product_tree(&recipes, &products);
        b.check_reference_margins(&recipes);
    }
    b.check_electricity(raw);
    production::check_start_setups(b.ctx, &b.catalog, raw);
    production::check_development_fields(b.ctx, &b.catalog, raw);
    b.check_complements(&products);

    let all_keys = [
        &unit_keys,
        &continent_keys,
        &branch_keys,
        &group_keys,
        &transport_keys,
        &qualification_keys,
        &specialization_keys,
        &country_keys,
        &product_keys,
        &facility_keys,
        &recipe_keys,
        &technology_keys,
        &deposit_keys,
        &vehicle_keys,
        &difficulty_keys,
        &event_keys,
        &milestone_keys,
        &media_keys,
        &currency_keys,
    ];
    check_texts(b.ctx, &all_keys, texts, all_files_read);
    countries::check_regions(b.ctx, &countries, &country_keys, texts, all_files_read);
    countries::check_city_texts(b.ctx, &countries, texts, all_files_read);
    plots::check_texts(b.ctx, &b.catalog.plot_model, raw, texts, all_files_read);
    production::check_size_texts(b.ctx, raw, texts, all_files_read);
    management::check_texts(b.ctx, &b.catalog.management, raw, texts, all_files_read);
    ventures::check_texts(b.ctx, &b.catalog.ventures, raw, texts, all_files_read);
    tariffs::check_texts(b.ctx, &b.catalog.tariffs, raw, texts, all_files_read);

    let catalog = b.catalog;
    (ctx.report.errors().count() == errors_before).then_some(catalog)
}

struct ProductRefs<'k> {
    units: &'k Keys,
    branches: &'k Keys,
    groups: &'k Keys,
    transport: &'k Keys,
    products: &'k Keys,
}

impl Builder<'_, '_> {
    fn meta(&mut self, raw: &RawData) {
        match raw.meta.as_slice() {
            [] => self.ctx.general_error(messages::meta_missing()),
            [first, rest @ ..] => {
                self.catalog.data_version = first.value.data_version;
                let first = self.ctx.describe(&first.loc);
                for other in rest {
                    self.ctx.error(&other.loc, messages::meta_duplicate(&first));
                }
            }
        }
    }

    /// Complements belong to a durable (petrol to the car).
    fn check_complements(&mut self, products: &[&Entry<RawProduct>]) {
        for e in products {
            let Some(c) = e
                .value
                .consumer_demand
                .as_ref()
                .and_then(|d| d.complement.as_ref())
            else {
                continue;
            };
            let Some(of) = self.catalog.products.id(&c.of) else {
                continue;
            };
            let durable = matches!(
                self.catalog
                    .products
                    .get(of)
                    .consumer_demand
                    .as_ref()
                    .map(|d| d.consumption),
                Some(ConsumptionType::Durable { .. })
            );
            if !durable {
                self.ctx.error(
                    &e.loc.field("nachfrage").field("ergaenzung").field("zu"),
                    messages::complement_needs_durable(&c.of),
                );
            }
        }
    }

    /// The electricity product of the production model must be of kind `energie`.
    fn check_electricity(&mut self, raw: &RawData) {
        let (Some(entry), Some(id)) = (
            raw.production_model.first(),
            self.catalog.production_model.electricity,
        ) else {
            return;
        };
        let Some(key) = entry.value.electricity.as_deref() else {
            return;
        };
        if self.catalog.products.id(key) == Some(id)
            && self.catalog.products.get(id).kind != ProductKind::Energy
        {
            self.ctx.error(
                &entry.loc.field("strom"),
                messages::electricity_not_energy(key),
            );
        }
    }

    fn labor_groups(&mut self) {
        let c = &mut self.catalog;
        let qualifications: Vec<_> = c
            .qualifications
            .iter()
            .map(|(id, q)| (id, q.clone()))
            .collect();
        for (qualification, q) in qualifications {
            let key = c.qualifications.key(qualification).to_owned();
            if q.has_specialization {
                let specializations: Vec<_> = c.specializations.ids().collect();
                for specialization in specializations {
                    let group_key = format!("{key}.{}", c.specializations.key(specialization));
                    let group = LaborGroup {
                        qualification,
                        specialization: Some(specialization),
                    };
                    c.labor_groups.insert(&group_key, group);
                }
            } else {
                c.labor_groups.insert(
                    &key,
                    LaborGroup {
                        qualification,
                        specialization: None,
                    },
                );
            }
        }
    }

    fn labor_group(&mut self, key: &str, loc: &Loc) -> LaborGroupId {
        if let Some(id) = self.catalog.labor_groups.id(key) {
            return id;
        }
        let c = &self.catalog;
        let (qualification, specialization) = match key.split_once('.') {
            Some((q, s)) => (q, Some(s)),
            None => (key, None),
        };
        let message = match c
            .qualifications
            .id(qualification)
            .map(|q| c.qualifications.get(q))
        {
            None => {
                let candidates = c.qualifications.ids().map(|q| c.qualifications.key(q));
                messages::unknown_reference(
                    "Qualifikation",
                    qualification,
                    suggest::closest(qualification, candidates),
                )
            }
            Some(q) if q.has_specialization && specialization.is_none() => {
                let example = c
                    .specializations
                    .ids()
                    .next()
                    .map_or("", |s| c.specializations.key(s));
                messages::qualification_needs_specialization(qualification, example)
            }
            Some(q) if !q.has_specialization => {
                messages::qualification_without_specialization(qualification)
            }
            Some(_) => {
                let s = specialization.unwrap_or_default();
                let candidates = c.specializations.ids().map(|s| c.specializations.key(s));
                messages::unknown_reference("Fachrichtung", s, suggest::closest(s, candidates))
            }
        };
        self.ctx.error(loc, message);
        LaborGroupId::from_index(0)
    }

    fn product(&mut self, e: &Entry<RawProduct>, refs: &ProductRefs) -> Product {
        let v = &e.value;
        let l = &e.loc;
        let unit = resolve(self.ctx, refs.units, &v.unit, &l.field("einheit"));
        let unit_weight = refs
            .units
            .index
            .contains_key(&v.unit)
            .then(|| self.catalog.units.get(unit).weight_kg)
            .flatten();
        let weight_kg = match (v.weight_kg, unit_weight) {
            (Some(w), _) => non_negative(self.ctx, w, &l.field("gewicht_kg")),
            (None, Some(w)) => w,
            (None, None) => {
                if refs.units.index.contains_key(&v.unit) {
                    self.ctx
                        .error(&l.field("einheit"), messages::weight_missing(&v.unit));
                }
                0.0
            }
        };
        let replaces = v
            .replaces
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let loc = l.field("ersetzt").index(i);
                if r == &v.id {
                    self.ctx.error(&loc, messages::product_replaces_itself(r));
                }
                resolve(self.ctx, refs.products, r, &loc)
            })
            .collect();
        Product {
            kind: match v.kind {
                RawProductKind::RawMaterial => ProductKind::RawMaterial,
                RawProductKind::SemiFinished => ProductKind::SemiFinished,
                RawProductKind::Component => ProductKind::Component,
                RawProductKind::EndProduct => ProductKind::EndProduct,
                RawProductKind::Energy => ProductKind::Energy,
            },
            branch: resolve(self.ctx, refs.branches, &v.branch, &l.field("branche")),
            unit,
            usage: match v.usage {
                RawUsage::Industry => Usage::Industry,
                RawUsage::Consumer => Usage::Consumer,
                RawUsage::Both => Usage::Both,
            },
            goods_group: resolve(
                self.ctx,
                refs.groups,
                &v.goods_group,
                &l.field("warengruppe"),
            ),
            transport_class: resolve(
                self.ctx,
                refs.transport,
                &v.transport_class,
                &l.field("transportklasse"),
            ),
            reference_price: {
                let loc = l.field("richtpreis_usd");
                positive(self.ctx, v.reference_price_usd, &loc);
                money(self.ctx, v.reference_price_usd, &loc)
            },
            weight_kg,
            heating_value_mwh: v
                .heating_value_mwh
                .map(|h| positive(self.ctx, h, &l.field("heizwert_mwh"))),
            consumer_demand: v
                .consumer_demand
                .as_ref()
                .map(|d| self.consumer_demand(d, &l.field("nachfrage"), refs.products)),
            state_demand: v.state_demand.as_ref().map(|d| {
                let loc = l.field("staatsnachfrage");
                StateDemand {
                    per_million_gdp: non_negative(
                        self.ctx,
                        d.per_million_gdp,
                        &loc.field("je_mio_usd_bip"),
                    ),
                    war_factor: non_negative(self.ctx, d.war_factor, &loc.field("kriegsfaktor")),
                    index: d
                        .index
                        .as_ref()
                        .map(|values| time_series(self.ctx, values, &loc.field("verlauf"))),
                }
            }),
            state_market: v
                .state_market
                .as_ref()
                .map(|m| self.state_market(m, &l.field("staatsmarkt"))),
            replaces,
            output_index: v.output_index.as_ref().map(|values| {
                let loc = l.field("foerderindex");
                if !matches!(v.kind, RawProductKind::RawMaterial) {
                    self.ctx
                        .error(&loc, messages::output_index_needs_raw_material(&v.id));
                }
                time_series(self.ctx, values, &loc)
            }),
            rent_share: v.rent_share.map_or(0.0, |share| {
                let loc = l.field("pacht_anteil");
                if !matches!(v.kind, RawProductKind::RawMaterial) {
                    self.ctx
                        .error(&loc, messages::rent_needs_raw_material(&v.id));
                }
                in_range(self.ctx, share, 0.0, 0.9, &loc)
            }),
            provenance: provenance(v.approximation, v.source.as_ref()),
        }
    }

    fn consumer_demand(
        &mut self,
        d: &RawConsumerDemand,
        l: &Loc,
        products: &Keys,
    ) -> ConsumerDemand {
        let consumption = match (&d.consumable, &d.durable, &d.complement) {
            (None, None, Some(c)) => {
                let c_loc = l.field("ergaenzung");
                ConsumptionType::Complement {
                    of: resolve(self.ctx, products, &c.of, &c_loc.field("zu")),
                    per_unit_per_year: positive(
                        self.ctx,
                        c.per_unit_per_year,
                        &c_loc.field("je_besitz_und_jahr"),
                    ),
                }
            }
            (Some(c), None, None) => ConsumptionType::Consumable {
                per_capita_per_year: positive(
                    self.ctx,
                    c.per_capita_per_year,
                    &l.field("verbrauch").field("je_kopf_und_jahr"),
                ),
            },
            (None, Some(g), None) => {
                let g_loc = l.field("gebrauch");
                ConsumptionType::Durable {
                    service_life_years: positive(
                        self.ctx,
                        g.service_life_years,
                        &g_loc.field("nutzungsdauer_jahre"),
                    ),
                    max_ownership: in_range(
                        self.ctx,
                        g.max_ownership,
                        0.0,
                        10.0,
                        &g_loc.field("max_besitzquote"),
                    ),
                }
            }
            _ => {
                self.ctx.error(l, messages::demand_type_ambiguous());
                ConsumptionType::Consumable {
                    per_capita_per_year: 0.0,
                }
            }
        };
        let seasonality = d.seasonality.as_ref().map(|s| {
            let loc = l.field("saison");
            for (i, &f) in s.iter().enumerate() {
                non_negative(self.ctx, f, &loc.index(i));
            }
            <[f64; 12]>::try_from(s.as_slice()).unwrap_or_else(|_| {
                self.ctx.error(&loc, messages::season_length(s.len()));
                [1.0; 12]
            })
        });
        ConsumerDemand {
            need_class: match d.need_class {
                RawNeedClass::Basic => NeedClass::Basic,
                RawNeedClass::Durable => NeedClass::Durable,
                RawNeedClass::Luxury => NeedClass::Luxury,
            },
            consumption,
            purchase_threshold: positive(self.ctx, d.purchase_threshold, &l.field("kaufschwelle")),
            price_sensitivity: non_negative(
                self.ctx,
                d.price_sensitivity,
                &l.field("preisempfindlichkeit"),
            ),
            income_sensitivity: non_negative(
                self.ctx,
                d.income_sensitivity,
                &l.field("einkommensempfindlichkeit"),
            ),
            seasonality,
            needs_grid: d.needs_grid,
        }
    }

    fn state_market(&mut self, m: &RawStateMarket, l: &Loc) -> StateMarketOffer {
        let price_loc = l.field("preis_usd");
        positive(self.ctx, m.price_usd, &price_loc);
        let available_from = m
            .available_from
            .map(|y| year(self.ctx, y, GAME_YEARS, &l.field("verfuegbar_ab")));
        let available_until = m
            .available_until
            .map(|y| year(self.ctx, y, GAME_YEARS, &l.field("verfuegbar_bis")));
        if let (Some(from), Some(until)) = (available_from, available_until)
            && from > until
        {
            self.ctx.error(
                &l.field("verfuegbar_bis"),
                messages::year_range_inverted(from, until),
            );
        }
        StateMarketOffer {
            price: money(self.ctx, m.price_usd, &price_loc),
            available_from,
            available_until,
        }
    }

    fn recipe(
        &mut self,
        e: &Entry<crate::raw::RawRecipe>,
        products: &Keys,
        facilities: &Keys,
        technologies: &Keys,
    ) -> Recipe {
        let v = &e.value;
        let l = &e.loc;
        let product: ProductId = resolve(self.ctx, products, &v.product, &l.field("produkt"));
        let product_known = products.index.contains_key(&v.product);
        if v.extraction
            && product_known
            && self.catalog.products.get(product).kind != ProductKind::RawMaterial
        {
            self.ctx.error(
                &l.field("abbau"),
                messages::extraction_needs_raw_material(&v.product),
            );
        }
        if v.inputs.contains_key(&v.product) {
            self.ctx.error(
                &l.field("eingang").key(&v.product),
                messages::recipe_consumes_own_product(&v.product),
            );
        }
        let mut quantities = |field: &str, map: &BTreeMap<String, f64>| -> Vec<(ProductId, f64)> {
            map.iter()
                .map(|(key, &qty)| {
                    let id = resolve(self.ctx, products, key, &l.field(field).key(key));
                    (id, positive(self.ctx, qty, &l.field(field).field(key)))
                })
                .collect()
        };
        let inputs = quantities("eingang", &v.inputs);
        let by_products = quantities("nebenprodukte", &v.by_products);
        let material = inputs
            .iter()
            .filter(|&&(p, _)| {
                p.index() >= self.catalog.products.len()
                    || self.catalog.products.get(p).kind != ProductKind::Energy
            })
            .count();
        if material > MAX_INPUTS {
            self.ctx.error(
                &l.field("eingang"),
                messages::too_many_inputs(material, MAX_INPUTS),
            );
        }
        let labor_hours = v
            .labor_hours
            .iter()
            .map(|(key, &hours)| {
                let group = self.labor_group(key, &l.field("arbeit_stunden").key(key));
                (
                    group,
                    non_negative(self.ctx, hours, &l.field("arbeit_stunden").field(key)),
                )
            })
            .collect();
        if v.duration_days == 0 {
            self.ctx
                .error(&l.field("dauer_tage"), messages::not_positive(0.0));
        }
        Recipe {
            product,
            output: positive(self.ctx, v.output, &l.field("menge")),
            by_products,
            duration_days: v.duration_days,
            facility: resolve(self.ctx, facilities, &v.facility, &l.field("anlage")),
            technology: v
                .technology
                .as_ref()
                .map(|t| resolve(self.ctx, technologies, t, &l.field("technologie"))),
            extraction: v.extraction,
            inputs,
            labor_hours,
            energy_mwh: non_negative(self.ctx, v.energy_mwh, &l.field("energie_mwh")),
            base_quality: in_range(
                self.ctx,
                v.base_quality,
                0.0,
                100.0,
                &l.field("qualitaet_basis"),
            ),
            provenance: provenance(v.approximation, v.source.as_ref()),
        }
    }

    /// Reports cycles and prerequisites invented after the technology that needs them.
    fn check_technology_graph(&mut self, entries: &[&Entry<crate::raw::RawTechnology>]) {
        let techs = &self.catalog.technologies;
        for (id, tech) in techs.iter() {
            for (i, &p) in tech.prerequisites.iter().enumerate() {
                let p_year = techs.get(p).invention_year;
                if p_year > tech.invention_year && p != id {
                    let loc = entries[id.index()].loc.field("voraussetzungen").index(i);
                    self.ctx
                        .warning(&loc, messages::prerequisite_younger(techs.key(p), p_year));
                }
            }
        }
        // Depth-first search; each cycle is reported once at its first member.
        #[derive(Clone, Copy, PartialEq)]
        enum State {
            New,
            Active,
            Done,
        }
        fn visit(
            id: TechnologyId,
            catalog: &Catalog,
            state: &mut [State],
            stack: &mut Vec<TechnologyId>,
            cycles: &mut Vec<Vec<TechnologyId>>,
        ) {
            state[id.index()] = State::Active;
            stack.push(id);
            for &p in &catalog.technologies.get(id).prerequisites {
                match state[p.index()] {
                    State::New => visit(p, catalog, state, stack, cycles),
                    State::Active => {
                        let start = stack.iter().position(|&s| s == p).unwrap_or(0);
                        let mut cycle = stack[start..].to_vec();
                        cycle.push(p);
                        cycles.push(cycle);
                    }
                    State::Done => {}
                }
            }
            stack.pop();
            state[id.index()] = State::Done;
        }
        let mut state = vec![State::New; techs.len()];
        let mut cycles = Vec::new();
        for id in techs.ids() {
            if state[id.index()] == State::New {
                visit(id, &self.catalog, &mut state, &mut Vec::new(), &mut cycles);
            }
        }
        for cycle in cycles {
            let names: Vec<&str> = cycle
                .iter()
                .map(|&t| self.catalog.technologies.key(t))
                .collect();
            let loc = entries[cycle[0].index()].loc.field("voraussetzungen");
            self.ctx.error(&loc, messages::technology_cycle(&names));
        }
    }

    /// Lastenheft §17.2: at most `MAX_LEVELS` levels from the raw material to the product
    /// of every recipe, more than `TYPICAL_LEVELS` only for very complex products.
    fn check_product_tree(
        &mut self,
        recipes: &[&Entry<crate::raw::RawRecipe>],
        products: &[&Entry<RawProduct>],
    ) {
        let c = &self.catalog;
        let chains = product_chains(c);
        let mut findings = Vec::new();
        for (id, r) in c.recipes.iter() {
            let path = recipe_chain(c, &chains, r);
            if path.len() <= TYPICAL_LEVELS {
                continue;
            }
            let names: Vec<&str> = path.iter().map(|&p| c.products.key(p)).collect();
            let loc = recipes[id.index()].loc.field("eingang");
            if path.len() > MAX_LEVELS {
                let message = messages::product_tree_too_deep(path.len(), MAX_LEVELS, &names);
                findings.push((true, loc, message));
            } else if !products[r.product.index()].value.very_complex {
                let message = messages::product_tree_deep(path.len(), TYPICAL_LEVELS, &names);
                findings.push((false, loc, message));
            }
        }
        for (error, loc, message) in findings {
            if error {
                self.ctx.error(&loc, message);
            } else {
                self.ctx.warning(&loc, message);
            }
        }
    }

    /// Plausibility of the reference prices: at reference prices, the best recipe of a
    /// product in the first year it can be made earns a margin within the band of the
    /// production model (extraction only from below: scarce deposits earn a rent).
    fn check_reference_margins(&mut self, recipes: &[&Entry<crate::raw::RawRecipe>]) {
        let c = &self.catalog;
        let (min, max) = c.production_model.reference_margin;
        let mut findings = Vec::new();
        for (product, p) in c.products.iter() {
            if p.kind == ProductKind::Energy {
                continue;
            }
            let made: Vec<(RecipeId, i32)> = c
                .recipes
                .iter()
                .filter(|(_, r)| r.product == product)
                .map(|(id, _)| (id, health::first_year(c, id)))
                .collect();
            let Some(first) = made.iter().map(|&(_, y)| y).min() else {
                continue;
            };
            let best = made
                .iter()
                .filter(|&&(_, y)| y == first)
                .filter_map(|&(r, _)| health::reference_margin(c, r).map(|m| (r, m)))
                .max_by(|a, b| a.1.total_cmp(&b.1));
            let Some((recipe, margin)) = best else {
                continue;
            };
            if margin < min || margin > max {
                let price = p.reference_price.to_usd();
                let message = messages::reference_margin(
                    c.products.key(product),
                    first,
                    ((1.0 - margin) * price, price),
                    margin,
                    (min, max),
                );
                findings.push((recipes[recipe.index()].loc.field("id"), message));
            }
        }
        for (loc, message) in findings {
            self.ctx.warning(&loc, message);
        }
    }

    /// Every product must be producible or available on the state market.
    fn check_product_sources(&mut self, entries: &[&Entry<RawProduct>]) {
        let c = &self.catalog;
        let mut produced = BTreeSet::new();
        let mut extracted = BTreeSet::new();
        for (_, recipe) in c.recipes.iter() {
            produced.insert(recipe.product);
            produced.extend(recipe.by_products.iter().map(|&(p, _)| p));
            if recipe.extraction {
                extracted.insert(recipe.product);
            }
        }
        let with_deposit: BTreeSet<ProductId> =
            c.deposits.iter().map(|(_, d)| d.resource).collect();
        for (id, product) in c.products.iter() {
            let loc = entries[id.index()].loc.field("id");
            let key = c.products.key(id);
            if !produced.contains(&id) && product.state_market.is_none() {
                self.ctx.error(&loc, messages::product_without_source(key));
            }
            if extracted.contains(&id) && !with_deposit.contains(&id) {
                self.ctx
                    .warning(&loc, messages::raw_material_without_deposit(key));
            }
        }
    }
}

/// Every entry needs a display text; texts for unknown entries are reported as unused.
fn check_texts(ctx: &mut Ctx, all_keys: &[&Keys], texts: &TextIndex, report_unused: bool) {
    for keys in all_keys {
        let Some(prefix) = keys.text_prefix else {
            continue;
        };
        for key in keys.keys_in_order() {
            let text_key = format!("{prefix}.{key}");
            if texts.texts.get(&text_key).is_none() {
                let loc = keys.locations[keys.index[key]].field("id");
                ctx.error(&loc, messages::text_missing(&text_key, crate::LANGUAGE));
            }
        }
    }
    if !report_unused {
        return;
    }
    for (text_key, loc) in &texts.locations {
        let Some((prefix, rest)) = text_key.split_once('.') else {
            continue;
        };
        let Some(keys) = all_keys.iter().find(|k| k.text_prefix == Some(prefix)) else {
            continue;
        };
        let id = rest.split('.').next().unwrap_or_default();
        if !keys.index.contains_key(id) && !keys.broken.contains(id) {
            ctx.warning(loc, messages::text_unused(text_key));
        }
    }
}

/// Chain of inputs that ends with each product, raw material first: through each recipe
/// the longest chain of its inputs, of several recipes the shortest (M33: synthetic
/// rubber from crude oil does not make rubber goods more complex than tapped rubber).
/// Goods without a processing recipe or with an extraction recipe are raw materials;
/// electricity is a production factor.
pub(crate) fn product_chains(c: &Catalog) -> BTreeMap<ProductId, Vec<ProductId>> {
    fn visit(
        p: ProductId,
        c: &Catalog,
        memo: &mut BTreeMap<ProductId, Vec<ProductId>>,
        active: &mut BTreeSet<ProductId>,
    ) -> Vec<ProductId> {
        if let Some(found) = memo.get(&p) {
            return found.clone();
        }
        if !active.insert(p) {
            // A cycle: stop here, the chain is reported where it is long enough.
            return vec![p];
        }
        let extracted = c.recipes.values().any(|r| r.product == p && r.extraction);
        let mut best: Option<Vec<ProductId>> = extracted.then(Vec::new);
        for (_, r) in c
            .recipes
            .iter()
            .filter(|(_, r)| r.product == p && !r.extraction)
        {
            let mut route: Vec<ProductId> = Vec::new();
            for &(input, _) in &r.inputs {
                if c.products.get(input).kind == ProductKind::Energy {
                    continue;
                }
                let sub = visit(input, c, memo, active);
                if sub.len() > route.len() {
                    route = sub;
                }
            }
            if best.as_ref().is_none_or(|b| route.len() < b.len()) {
                best = Some(route);
            }
        }
        let mut best = best.unwrap_or_default();
        active.remove(&p);
        best.push(p);
        memo.insert(p, best.clone());
        best
    }
    let mut memo = BTreeMap::new();
    for (p, _) in c.products.iter() {
        visit(p, c, &mut memo, &mut BTreeSet::new());
    }
    memo
}

/// The longest chain through one recipe, ending with its product.
fn recipe_chain(
    c: &Catalog,
    chains: &BTreeMap<ProductId, Vec<ProductId>>,
    r: &wsim_core::catalog::Recipe,
) -> Vec<ProductId> {
    let mut path: Vec<ProductId> = Vec::new();
    if !r.extraction {
        for &(input, _) in &r.inputs {
            if c.products.get(input).kind == ProductKind::Energy {
                continue;
            }
            if let Some(sub) = chains.get(&input).filter(|sub| sub.len() > path.len()) {
                path = sub.clone();
            }
        }
    }
    path.push(r.product);
    path
}
