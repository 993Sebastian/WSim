//! The production model.

use std::collections::BTreeMap;

use wsim_core::catalog::{
    Catalog, ProductionModel, ResearchModel, SiteType, StartSetup, TransportModel, Vehicle, Way,
};
use wsim_core::ids::QualificationId;
use wsim_core::state::StartForm;

use super::{
    HISTORY_YEARS, Keys, in_range, money, non_negative, per_kind, positive, provenance, resolve,
    time_series, year,
};
use crate::messages;
use crate::raw::{RawLimits, RawProductionModel, RawVehicle, RawWay};
use crate::read::{Ctx, Entry, Loc, RawData};
use crate::suggest;

/// The one entry of a parameter section; reports a missing or repeated section.
pub(super) fn single<'r, T>(
    ctx: &mut Ctx,
    entries: &'r [Entry<T>],
    section: &str,
    file: &str,
) -> Option<&'r Entry<T>> {
    match entries {
        [] => {
            ctx.general_error(messages::section_missing(section, file));
            None
        }
        [first, rest @ ..] => {
            let first_loc = ctx.describe(&first.loc);
            for other in rest {
                ctx.error(&other.loc, messages::section_duplicate(section, &first_loc));
            }
            Some(first)
        }
    }
}

const SITE_TYPES: &[(&str, SiteType)] = &[
    ("foerderstaette", SiteType::Extraction),
    ("werk", SiteType::Factory),
    ("kraftwerk", SiteType::PowerPlant),
    ("lager", SiteType::Warehouse),
    ("niederlassung", SiteType::SalesOffice),
    ("forschungszentrum", SiteType::ResearchCenter),
];

pub(super) fn production_model(
    ctx: &mut Ctx,
    raw: &RawData,
    (products, facilities, recipes): (&Keys, &Keys, &Keys),
) -> ProductionModel {
    let Some(entry) = single(
        ctx,
        &raw.production_model,
        "produktionsmodell",
        "parameter/produktionsmodell.yaml",
    ) else {
        return ProductionModel::default();
    };
    let m = &entry.value;
    let l = &entry.loc;
    let costs = l.field("standortkosten_usd");
    let names: Vec<&str> = SITE_TYPES.iter().map(|(n, _)| *n).collect();
    let mut site_cost = Vec::new();
    for (key, &usd) in &m.site_cost {
        match SITE_TYPES.iter().find(|(n, _)| n == key) {
            Some(&(_, kind)) => site_cost.push((kind, money(ctx, usd, &costs.field(key)))),
            None => ctx.error(
                &costs.key(key),
                messages::unknown_value(key, &names, suggest::closest(key, names.iter().copied())),
            ),
        }
    }
    for (name, _) in SITE_TYPES {
        if !m.site_cost.contains_key(*name) {
            ctx.error(&costs, messages::entry_missing("Standorttyp", name));
        }
    }
    let automation = l.field("automatisierung");
    let quality = l.field("qualitaet");
    ProductionModel {
        site_cost,
        building_lifetime_years: positive(
            ctx,
            m.building_lifetime_years,
            &l.field("gebaeude_lebensdauer_jahre"),
        ),
        development_lifetime_years: positive(
            ctx,
            m.development_lifetime_years,
            &l.field("erschliessung_lebensdauer_jahre"),
        ),
        automation_labor_saving: in_range(
            ctx,
            m.automation.labor_saving,
            0.0,
            1.0,
            &automation.field("arbeitsersparnis"),
        ),
        automation_cost_share: in_range(
            ctx,
            m.automation.cost_share,
            0.0,
            10.0,
            &automation.field("kostenanteil"),
        ),
        quality_inputs: in_range(
            ctx,
            m.quality.inputs,
            0.0,
            1.0,
            &quality.field("vorprodukte"),
        ),
        quality_automation: in_range(
            ctx,
            m.quality.automation,
            0.0,
            100.0,
            &quality.field("automatisierung"),
        ),
        quality_condition: in_range(
            ctx,
            m.quality.condition,
            0.0,
            100.0,
            &quality.field("zustand"),
        ),
        condition_min: in_range(ctx, m.condition_min, 0.0, 1.0, &l.field("zustand_minimum")),
        electricity: m
            .electricity
            .as_ref()
            .map(|key| resolve(ctx, products, key, &l.field("strom"))),
        feed_in_share: in_range(
            ctx,
            m.feed_in_share,
            0.0,
            1.0,
            &l.field("einspeiseverguetung"),
        ),
        overhead_share: per_kind(
            ctx,
            &m.overhead_share,
            &l.field("gemeinkosten_anteil"),
            |ctx, v, loc| in_range(ctx, v, 0.0, 5.0, loc),
        ),
        reference_margin: reference_margin(ctx, &m.reference_margin, &l.field("richtpreis_marge")),
        by_product_stock_days: positive(
            ctx,
            m.by_product_stock_days,
            &l.field("nebenprodukte_lager_tage"),
        ),
        start_setups: start_setups(ctx, m, l, (products, facilities, recipes)),
    }
}

fn reference_margin(ctx: &mut Ctx, m: &RawLimits, loc: &Loc) -> (f64, f64) {
    let min = in_range(ctx, m.min, -1.0, 1.0, &loc.field("minimum"));
    let max = in_range(ctx, m.max, -1.0, 1.0, &loc.field("maximum"));
    if min > max {
        ctx.error(loc, messages::range_inverted("minimum", "maximum"));
    }
    (min, max)
}

pub(super) fn finance_model(ctx: &mut Ctx, raw: &RawData) -> wsim_core::catalog::FinanceModel {
    use wsim_core::time_series::TimeSeries;
    let Some(entry) = single(
        ctx,
        &raw.finance_model,
        "finanzmodell",
        "parameter/finanzmodell.yaml",
    ) else {
        return wsim_core::catalog::FinanceModel::default();
    };
    let m = &entry.value;
    let l = &entry.loc;
    let rates = l.field("realzins");
    if m.real_rate.is_empty() {
        ctx.error(&rates, messages::time_series_empty());
    }
    for (&year, &rate) in &m.real_rate {
        let point = rates.field(&year.to_string());
        super::year(ctx, year, super::GAME_YEARS, &point);
        in_range(ctx, rate, -0.2, 0.5, &point);
    }
    let real_rate = TimeSeries::new(m.real_rate.iter().map(|(&y, &r)| (y, r)).collect())
        .unwrap_or_else(|_| TimeSeries::new(vec![(1900, 0.0)]).expect("valid"));
    let premium = l.field("risikoaufschlag");
    let overdraft = l.field("dispo");
    if m.max_term_years == 0 {
        ctx.error(&l.field("laufzeit_max_jahre"), messages::not_positive(0.0));
    }
    wsim_core::catalog::FinanceModel {
        real_rate,
        premium_min: in_range(ctx, m.premium.min, 0.0, 1.0, &premium.field("minimum")),
        premium_per_debt_ratio: in_range(
            ctx,
            m.premium.per_debt_ratio,
            0.0,
            1.0,
            &premium.field("je_verschuldung"),
        ),
        loan_to_value: in_range(ctx, m.loan_to_value, 0.0, 1.0, &l.field("beleihung")),
        overdraft_share: in_range(ctx, m.overdraft.share, 0.0, 1.0, &overdraft.field("anteil")),
        overdraft_premium: in_range(
            ctx,
            m.overdraft.premium,
            0.0,
            1.0,
            &overdraft.field("aufschlag"),
        ),
        max_term_years: m.max_term_years,
    }
}

/// The market model and the keys of the advertising media (for the text check).
pub(super) fn market_model(
    ctx: &mut Ctx,
    raw: &RawData,
) -> (wsim_core::catalog::MarketModel, Keys) {
    let mut media_keys = Keys {
        kind: "Werbemittel",
        text_prefix: Some("werbemittel"),
        index: BTreeMap::new(),
        locations: Vec::new(),
        broken: Default::default(),
    };
    let Some(entry) = single(
        ctx,
        &raw.market_model,
        "marktmodell",
        "parameter/marktmodell.yaml",
    ) else {
        return (wsim_core::catalog::MarketModel::default(), media_keys);
    };
    let m = &entry.value;
    let l = &entry.loc;
    let mut five = |values: &[f64], field: &str| -> [f64; 5] {
        let loc = l.field(field);
        for (i, &v) in values.iter().enumerate() {
            positive(ctx, v, &loc.index(i));
        }
        <[f64; 5]>::try_from(values).unwrap_or_else(|_| {
            ctx.error(
                &loc,
                messages::wrong_length(values.len(), "5 Werte (ärmstes Fünftel zuerst)"),
            );
            [1.0; 5]
        })
    };
    let price_weight = five(&m.price_weight, "preisgewicht");
    let quality_weight = five(&m.quality_weight, "qualitaetsgewicht");
    let bl = l.field("marke");
    let brand_weight = {
        let loc = bl.field("markengewicht");
        for (i, &v) in m.brand.weight.iter().enumerate() {
            non_negative(ctx, v, &loc.index(i));
        }
        <[f64; 5]>::try_from(m.brand.weight.as_slice()).unwrap_or_else(|_| {
            ctx.error(
                &loc,
                messages::wrong_length(m.brand.weight.len(), "5 Werte (ärmstes Fünftel zuerst)"),
            );
            [0.0; 5]
        })
    };
    let b = &m.brand;
    let mut media = Vec::new();
    for (i, medium) in b.media.iter().enumerate() {
        let loc = bl.field("werbemittel").index(i);
        if let Some(&first) = media_keys.index.get(&medium.id) {
            let first = ctx.describe(&media_keys.locations[first]);
            ctx.error(
                &loc.field("id"),
                messages::duplicate_key("Werbemittel", &medium.id, &first),
            );
            continue;
        }
        media_keys.index.insert(medium.id.clone(), media.len());
        media_keys.locations.push(loc.clone());
        media.push(wsim_core::catalog::AdvertisingMedium {
            key: medium.id.clone(),
            from_year: medium.from_year,
            effect: positive(ctx, medium.effect, &loc.field("wirkung")),
        });
    }
    if media.is_empty() {
        ctx.error(&bl.field("werbemittel"), messages::list_empty());
    }
    let brand = wsim_core::catalog::BrandModel {
        weight: brand_weight,
        forgetting_per_month: in_range(
            ctx,
            b.forgetting_per_month,
            0.0,
            1.0,
            &bl.field("vergessen_je_monat"),
        ),
        word_of_mouth: in_range(ctx, b.word_of_mouth, 0.0, 1.0, &bl.field("mundpropaganda")),
        cost_per_inhabitant_usd: positive(
            ctx,
            b.cost_per_inhabitant_usd,
            &bl.field("kosten_je_einwohner_usd"),
        ),
        start_awareness: in_range(
            ctx,
            b.start_awareness,
            0.0,
            1.0,
            &bl.field("bekanntheit_start"),
        ),
        start_awareness_real: in_range(
            ctx,
            b.start_awareness_real,
            0.0,
            1.0,
            &bl.field("bekanntheit_start_real"),
        ),
        trade_awareness: in_range(
            ctx,
            b.trade_awareness,
            0.0,
            1.0,
            &bl.field("bekanntheit_handel"),
        ),
        state_market_awareness: in_range(
            ctx,
            b.state_market_awareness,
            0.0,
            1.0,
            &bl.field("bekanntheit_staatsmarkt"),
        ),
        media,
    };
    let adjust = l.field("preisanpassung");
    let traders = l.field("haendler");
    let model = wsim_core::catalog::MarketModel {
        price_weight,
        quality_weight,
        adoption_per_year: in_range(
            ctx,
            m.adoption_per_year,
            0.0,
            1.0,
            &l.field("aneignung_je_jahr"),
        ),
        price_step_up: in_range(ctx, m.price_adjustment.up, 0.0, 0.5, &adjust.field("hoch")),
        price_step_down: in_range(
            ctx,
            m.price_adjustment.down,
            0.0,
            0.5,
            &adjust.field("runter"),
        ),
        stock_days: positive(
            ctx,
            m.price_adjustment.stock_days,
            &adjust.field("lagertage"),
        ),
        normal_utilization: in_range(
            ctx,
            m.price_adjustment.normal_utilization,
            0.0,
            1.0,
            &adjust.field("auslastung_normal"),
        ),
        price_max_factor: in_range(
            ctx,
            m.price_adjustment.max_factor,
            1.0,
            1000.0,
            &adjust.field("hoechstfaktor"),
        ),
        state_price_cap: positive(ctx, m.state_price_cap, &l.field("staat_hoechstpreis")),
        price_level_share: per_kind(
            ctx,
            &m.price_level_share,
            &l.field("preisniveau_anteil"),
            |ctx, v, loc| in_range(ctx, v, 0.0, 1.0, loc),
        ),
        index_smoothing: in_range(
            ctx,
            m.index_smoothing,
            0.0,
            1.0,
            &l.field("index_glaettung"),
        ),
        trader_margin: in_range(ctx, m.traders.margin, 0.0, 1.0, &traders.field("marge")),
        trader_cover_days: positive(ctx, m.traders.cover_days, &traders.field("vorrat_tage")),
        demand_smoothing_days: in_range(
            ctx,
            m.traders.smoothing_days,
            1.0,
            365.0,
            &traders.field("glaettung_tage"),
        ),
        brand,
    };
    (model, media_keys)
}

pub(super) fn transport_model(ctx: &mut Ctx, raw: &RawData) -> TransportModel {
    let Some(entry) = single(
        ctx,
        &raw.transport_model,
        "transportmodell",
        "parameter/transportmodell.yaml",
    ) else {
        return TransportModel::default();
    };
    let m = &entry.value;
    let l = &entry.loc;
    let detour = l.field("umweg");
    let handling = l.field("umschlag");
    TransportModel {
        detour_land: in_range(ctx, m.detour.land, 1.0, 5.0, &detour.field("land")),
        detour_sea: in_range(ctx, m.detour.see, 1.0, 5.0, &detour.field("see")),
        detour_air: in_range(ctx, m.detour.luft, 1.0, 5.0, &detour.field("luft")),
        handling_cost_usd: non_negative(
            ctx,
            m.handling.cost_usd,
            &handling.field("kosten_usd_je_t"),
        ),
        handling_days: non_negative(ctx, m.handling.days, &handling.field("tage")),
        min_infrastructure: in_range(
            ctx,
            m.min_infrastructure,
            0.001,
            1.0,
            &l.field("mindestinfrastruktur"),
        ),
    }
}

pub(super) fn vehicle(ctx: &mut Ctx, e: &Entry<RawVehicle>, classes: &Keys) -> Vehicle {
    let v = &e.value;
    let l = &e.loc;
    let available_from = year(
        ctx,
        v.available_from,
        HISTORY_YEARS,
        &l.field("verfuegbar_ab"),
    );
    let available_until = v
        .available_until
        .map(|y| year(ctx, y, HISTORY_YEARS, &l.field("verfuegbar_bis")));
    if let Some(until) = available_until
        && available_from > until
    {
        ctx.error(
            &l.field("verfuegbar_bis"),
            messages::year_range_inverted(available_from, until),
        );
    }
    let classes_loc = l.field("transportklassen");
    if v.classes.is_empty() {
        ctx.error(&classes_loc, messages::vehicle_without_classes());
    }
    let classes = v
        .classes
        .iter()
        .enumerate()
        .map(|(i, c)| resolve(ctx, classes, c, &classes_loc.index(i)))
        .collect();
    let speed_loc = l.field("km_je_tag");
    for (&y, &speed) in &v.km_per_day {
        positive(ctx, speed, &speed_loc.field(&y.to_string()));
    }
    Vehicle {
        way: match v.way {
            RawWay::Terrain => Way::Terrain,
            RawWay::Road => Way::Road,
            RawWay::Rail => Way::Rail,
            RawWay::Sea => Way::Sea,
            RawWay::Air => Way::Air,
        },
        available_from,
        available_until,
        classes,
        cost_per_tkm: time_series(ctx, &v.cost_per_tkm, &l.field("kosten_usd_je_tkm")),
        km_per_day: time_series(ctx, &v.km_per_day, &speed_loc),
        provenance: provenance(v.approximation, v.source.as_ref()),
    }
}

pub(super) fn research_model(
    ctx: &mut Ctx,
    catalog: &Catalog,
    raw: &RawData,
    qualifications: &Keys,
) -> ResearchModel {
    let Some(entry) = single(
        ctx,
        &raw.research_model,
        "forschungsmodell",
        "parameter/forschungsmodell.yaml",
    ) else {
        return ResearchModel::default();
    };
    let m = &entry.value;
    let l = &entry.loc;
    let latecomer = l.field("nachzuegler");
    let researchers_loc = l.field("forscher");
    let qualification: QualificationId =
        resolve(ctx, qualifications, &m.researchers, &researchers_loc);
    let researchers = if catalog.qualifications.get(qualification).has_specialization {
        catalog
            .specializations
            .ids()
            .map(|s| {
                let key = format!("{}.{}", m.researchers, catalog.specializations.key(s));
                catalog.labor_groups.id(&key)
            })
            .collect()
    } else {
        if qualifications.index.contains_key(&m.researchers) {
            ctx.error(
                &researchers_loc,
                messages::researchers_need_fields(&m.researchers),
            );
        }
        Vec::new()
    };
    ResearchModel {
        ahead_base: in_range(ctx, m.ahead_base, 1.0, 10.0, &l.field("vorgriff_faktor")),
        latecomer_discount: in_range(
            ctx,
            m.latecomer.discount,
            0.0,
            1.0,
            &latecomer.field("rabatt_je_jahr"),
        ),
        latecomer_min: in_range(ctx, m.latecomer.min, 0.0, 1.0, &latecomer.field("minimum")),
        public_domain_years: i32::try_from(m.public_domain_years).unwrap_or(i32::MAX),
        material_usd_per_day: non_negative(
            ctx,
            m.material_usd_per_day,
            &l.field("sachkosten_usd_je_forschertag"),
        ),
        researchers,
    }
}

const START_FORMS: &[(&str, StartForm)] = &[
    ("werkstatt", StartForm::Workshop),
    ("handel", StartForm::Trading),
];

fn start_setups(
    ctx: &mut Ctx,
    m: &RawProductionModel,
    l: &Loc,
    (products, facilities, recipes): (&Keys, &Keys, &Keys),
) -> Vec<(StartForm, StartSetup)> {
    let forms_loc = l.field("startformen");
    let names: Vec<&str> = START_FORMS.iter().map(|(n, _)| *n).collect();
    let mut setups = Vec::new();
    for (key, raw) in &m.start_setups {
        let Some(&(_, form)) = START_FORMS.iter().find(|(n, _)| n == key) else {
            ctx.error(
                &forms_loc.key(key),
                messages::unknown_value(key, &names, suggest::closest(key, names.iter().copied())),
            );
            continue;
        };
        let loc = forms_loc.field(key);
        let facilities_loc = loc.field("anlagen");
        let purchases_loc = loc.field("einkauf");
        let sales_loc = loc.field("verkauf");
        let setup = StartSetup {
            site_type: super::site_type(raw.site_type),
            building: money(ctx, raw.building_usd, &loc.field("gebaeude_usd")),
            facilities: raw
                .facilities
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let f_loc = facilities_loc.index(i);
                    (
                        resolve(ctx, facilities, &f.facility, &f_loc.field("anlage")),
                        f.recipe
                            .as_ref()
                            .map(|r| resolve(ctx, recipes, r, &f_loc.field("rezept"))),
                        in_range(ctx, f.utilization, 0.0, 1.0, &f_loc.field("auslastung")),
                    )
                })
                .collect(),
            purchases: raw
                .purchases
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let p_loc = purchases_loc.index(i);
                    (
                        resolve(ctx, products, &p.product, &p_loc.field("produkt")),
                        positive(ctx, p.target, &p_loc.field("ziel")),
                        money(ctx, p.max_price_usd, &p_loc.field("hoechstpreis_usd")),
                    )
                })
                .collect(),
            sales: raw
                .sales
                .iter()
                .enumerate()
                .map(|(i, p)| resolve(ctx, products, p, &sales_loc.index(i)))
                .collect(),
        };
        setups.push((form, setup));
    }
    for (name, _) in START_FORMS {
        if !m.start_setups.contains_key(*name) {
            ctx.error(&forms_loc, messages::entry_missing("Startform", name));
        }
    }
    setups
}

/// Facilities of a start form fit its site type, recipes fit their facility, and both
/// are known in the earliest start year.
pub(super) fn check_start_setups(ctx: &mut Ctx, catalog: &Catalog, raw: &RawData) {
    let Some(entry) = raw.production_model.first() else {
        return;
    };
    for (key, setup) in &entry.value.start_setups {
        let Some(&(_, form)) = START_FORMS.iter().find(|(n, _)| n == key) else {
            continue;
        };
        let Some(built) = catalog.production_model.start_setup(form) else {
            continue;
        };
        let loc = entry.loc.field("startformen").field(key).field("anlagen");
        for (i, (&(facility, recipe, _), raw_f)) in
            built.facilities.iter().zip(&setup.facilities).enumerate()
        {
            if catalog.facilities.id(&raw_f.facility) != Some(facility) {
                continue;
            }
            let f = catalog.facilities.get(facility);
            if f.site_type != built.site_type {
                ctx.error(
                    &loc.index(i).field("anlage"),
                    messages::start_facility_wrong_site(&raw_f.facility),
                );
            }
            let too_new = |t: Option<wsim_core::ids::TechnologyId>| {
                t.is_some_and(|t| {
                    catalog.technologies.get(t).invention_year > wsim_core::EARLIEST_START_YEAR
                })
            };
            if too_new(f.technology) {
                ctx.error(
                    &loc.index(i).field("anlage"),
                    messages::start_needs_new_technology(&raw_f.facility),
                );
            }
            if let (Some(recipe), Some(raw_r)) = (recipe, raw_f.recipe.as_deref())
                && catalog.recipes.id(raw_r) == Some(recipe)
            {
                let r = catalog.recipes.get(recipe);
                if r.facility != facility {
                    ctx.error(
                        &loc.index(i).field("rezept"),
                        messages::start_recipe_wrong_facility(raw_r, &raw_f.facility),
                    );
                }
                if too_new(r.technology) {
                    ctx.error(
                        &loc.index(i).field("rezept"),
                        messages::start_needs_new_technology(raw_r),
                    );
                }
            }
        }
    }
}
