//! Countries and the country model.

use std::collections::BTreeMap;

use wsim_core::catalog::{Catalog, Country, CountryModel, CountryProfile, CountryValues, GeoPoint};
use wsim_core::ids::{CountryId, Id};
use wsim_core::time_series::TimeSeries;

use super::{Keys, in_range, positive, provenance, resolve, time_series};
use crate::messages;
use crate::raw::{RawCountry, RawCountryModel, RawSeries};
use crate::read::{Ctx, Entry, Loc, RawData};
use crate::texts::TextIndex;

/// Time series with values between `min` and `max`.
fn bounded(ctx: &mut Ctx, values: &RawSeries, min: f64, max: f64, loc: &Loc) -> TimeSeries {
    for (&y, &v) in values {
        in_range(ctx, v, min, max, &loc.field(&y.to_string()));
    }
    time_series(ctx, values, loc)
}

/// Weights per specialization from a map `{fachrichtung: gewicht}`; missing ones are 1.
fn weights(
    ctx: &mut Ctx,
    catalog: &Catalog,
    specializations: &Keys,
    map: &BTreeMap<String, f64>,
    loc: &Loc,
) -> Vec<f64> {
    let mut out = vec![1.0; catalog.specializations.len()];
    for (key, &weight) in map {
        let id: wsim_core::ids::SpecializationId =
            resolve(ctx, specializations, key, &loc.key(key));
        let weight = positive(ctx, weight, &loc.field(key));
        if specializations.index.contains_key(key) {
            out[id.index()] = weight;
        }
    }
    out
}

pub(super) fn country(
    ctx: &mut Ctx,
    catalog: &Catalog,
    e: &Entry<RawCountry>,
    continents: &Keys,
    countries: &Keys,
    specializations: &Keys,
) -> Country {
    let v = &e.value;
    let l = &e.loc;
    let values = l.field("werte");
    let neighbors = v
        .neighbors
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let loc = l.field("nachbarn").index(i);
            if n == &v.id {
                ctx.error(&loc, messages::neighbor_self(n));
            }
            resolve::<CountryId>(ctx, countries, n, &loc)
        })
        .collect();
    let profile = match &v.profile {
        None => CountryProfile::default(),
        Some(p) => {
            let p_loc = l.field("praegung");
            CountryProfile {
                automation_bonus: in_range(
                    ctx,
                    p.automation,
                    -1.0,
                    1.0,
                    &p_loc.field("automatisierung"),
                ),
                specialization_weights: weights(
                    ctx,
                    catalog,
                    specializations,
                    &p.specializations,
                    &p_loc.field("fachrichtungen"),
                ),
                research_weights: weights(
                    ctx,
                    catalog,
                    specializations,
                    &p.research,
                    &p_loc.field("forschung"),
                ),
            }
        }
    };
    let capital = l.field("hauptstadt");
    Country {
        continent: resolve(ctx, continents, &v.continent, &l.field("kontinent")),
        area_km2: positive(ctx, v.area_km2, &l.field("flaeche_km2")),
        capital: GeoPoint {
            lat: in_range(ctx, v.capital.lat, -90.0, 90.0, &capital.field("breite")),
            lon: in_range(ctx, v.capital.lon, -180.0, 180.0, &capital.field("laenge")),
        },
        landlocked: v.landlocked,
        neighbors,
        members: v.members.clone(),
        values: CountryValues {
            population: time_series(ctx, &v.values.population, &values.field("bevoelkerung")),
            gdp_per_capita_usd: time_series(
                ctx,
                &v.values.gdp_per_capita_usd,
                &values.field("bip_je_kopf_usd"),
            ),
            gini: bounded(ctx, &v.values.gini, 0.0, 0.95, &values.field("gini")),
            stability: v
                .values
                .stability
                .as_ref()
                .map(|s| bounded(ctx, s, 0.0, 1.0, &values.field("stabilitaet"))),
            corporate_tax: v
                .values
                .corporate_tax
                .as_ref()
                .map(|s| bounded(ctx, s, 0.0, 1.0, &values.field("steuer_unternehmen"))),
            dividend_tax: v
                .values
                .dividend_tax
                .as_ref()
                .map(|s| bounded(ctx, s, 0.0, 1.0, &values.field("steuer_dividenden"))),
        },
        profile,
        provenance: provenance(v.approximation, v.source.as_ref()),
    }
}

/// Countries merged into regions (M34): ISO codes, none a country of its own or part of
/// two regions. Every member needs a display text `teilland.<ISO>`.
pub(super) fn check_regions(
    ctx: &mut Ctx,
    entries: &[&Entry<RawCountry>],
    countries: &Keys,
    texts: &TextIndex,
    report_unused: bool,
) {
    let mut members: BTreeMap<&str, Loc> = BTreeMap::new();
    for e in entries {
        let v = &e.value;
        let list = e.loc.field("umfasst");
        if v.members.len() == 1 {
            ctx.error(&list, messages::region_too_small(&v.id));
        }
        for (i, member) in v.members.iter().enumerate() {
            let loc = list.index(i);
            if !super::is_country_code(member) {
                ctx.error(&loc, messages::invalid_country_code(member));
            } else if member != &v.id && countries.index.contains_key(member) {
                ctx.error(&loc, messages::region_member_is_country(member, &v.id));
            } else if let Some(first) = members.get(member.as_str()) {
                let first = ctx.describe(first);
                ctx.error(&loc, messages::region_member_twice(member, &first));
            } else {
                let text_key = format!("{MEMBER_TEXT}.{member}");
                if texts.texts.get(&text_key).is_none() {
                    ctx.error(&loc, messages::text_missing(&text_key, crate::LANGUAGE));
                }
                members.insert(member, loc);
            }
        }
    }
    if report_unused {
        for (text_key, loc) in &texts.locations {
            if let Some(member) = text_key
                .strip_prefix(MEMBER_TEXT)
                .and_then(|r| r.strip_prefix('.'))
                && !members.contains_key(member)
            {
                ctx.warning(loc, messages::text_unused(text_key));
            }
        }
    }
}

/// Text prefix for the names of countries within a region.
const MEMBER_TEXT: &str = "teilland";

/// Neighbourhood must be mutual; reported as warning.
pub(super) fn check_neighbors(ctx: &mut Ctx, catalog: &Catalog, entries: &[&Entry<RawCountry>]) {
    for (id, country) in catalog.countries.iter() {
        for (i, &n) in country.neighbors.iter().enumerate() {
            if n != id && !catalog.countries.get(n).neighbors.contains(&id) {
                let loc = entries[id.index()].loc.field("nachbarn").index(i);
                ctx.warning(
                    &loc,
                    messages::neighbor_asymmetric(
                        catalog.countries.key(n),
                        catalog.countries.key(id),
                    ),
                );
            }
        }
    }
}

/// Values per qualification from a map; every qualification must be present.
fn per_qualification(
    ctx: &mut Ctx,
    catalog: &Catalog,
    qualifications: &Keys,
    map: &BTreeMap<String, f64>,
    loc: &Loc,
) -> Vec<f64> {
    let mut out = vec![0.0; catalog.qualifications.len()];
    for (key, &value) in map {
        let id: wsim_core::ids::QualificationId = resolve(ctx, qualifications, key, &loc.key(key));
        if qualifications.index.contains_key(key) {
            out[id.index()] = value;
        }
    }
    for q in catalog.qualifications.ids() {
        let key = catalog.qualifications.key(q);
        if !map.contains_key(key) {
            ctx.error(loc, messages::entry_missing("Qualifikation", key));
        }
    }
    out
}

fn check_sum(ctx: &mut Ctx, values: &[f64], loc: &Loc) {
    let sum: f64 = values.iter().sum();
    if (sum - 1.0).abs() > 0.001 {
        ctx.error(loc, messages::shares_sum(sum));
    }
}

/// How to read one kind of table row.
struct RowSpec<R> {
    gdp: fn(&R) -> f64,
    values: fn(&R) -> &BTreeMap<String, f64>,
    field: &'static str,
    /// Values are shares that must add up to 1 (otherwise positive factors).
    shares: bool,
}

/// Table rows ascending by GDP per capita.
fn rows<R>(
    ctx: &mut Ctx,
    catalog: &Catalog,
    qualifications: &Keys,
    rows: &[R],
    loc: &Loc,
    spec: &RowSpec<R>,
) -> Vec<(f64, Vec<f64>)> {
    if rows.is_empty() {
        ctx.error(loc, messages::table_empty());
    }
    let mut previous = 0.0;
    rows.iter()
        .enumerate()
        .map(|(i, row)| {
            let row_loc = loc.index(i);
            let gdp = positive(ctx, (spec.gdp)(row), &row_loc.field("bip_je_kopf_usd"));
            if gdp <= previous {
                ctx.error(
                    &row_loc.field("bip_je_kopf_usd"),
                    messages::rows_not_ascending(),
                );
            }
            previous = gdp;
            let values_loc = row_loc.field(spec.field);
            let values = per_qualification(
                ctx,
                catalog,
                qualifications,
                (spec.values)(row),
                &values_loc,
            );
            for (key, &v) in (spec.values)(row) {
                if spec.shares {
                    in_range(ctx, v, 0.0, 1.0, &values_loc.field(key));
                } else {
                    positive(ctx, v, &values_loc.field(key));
                }
            }
            if spec.shares {
                check_sum(ctx, &values, &values_loc);
            }
            (gdp, values)
        })
        .collect()
}

pub(super) fn country_model(
    ctx: &mut Ctx,
    catalog: &Catalog,
    raw: &RawData,
    keys: (&Keys, &Keys, &Keys),
) -> CountryModel {
    let (countries, qualifications, specializations) = keys;
    let entry = match raw.country_model.as_slice() {
        [] => {
            ctx.general_error(messages::country_model_missing());
            return CountryModel::default();
        }
        [first, rest @ ..] => {
            let first_loc = ctx.describe(&first.loc);
            for other in rest {
                ctx.error(
                    &other.loc,
                    messages::section_duplicate("laendermodell", &first_loc),
                );
            }
            first
        }
    };
    let m = &entry.value;
    let l = &entry.loc;
    let price = l.field("preisniveau");
    if m.price_level.min > m.price_level.max {
        ctx.error(
            &price.field("minimum"),
            messages::range_inverted("minimum", "maximum"),
        );
    }
    let development = l.field("entwicklung");
    if m.development.from >= m.development.to {
        ctx.error(
            &development.field("von_usd"),
            messages::range_inverted("von_usd", "bis_usd"),
        );
    }
    let productivity = l.field("produktivitaet");
    if m.productivity.min > m.productivity.max {
        ctx.error(
            &productivity.field("minimum"),
            messages::range_inverted("minimum", "maximum"),
        );
    }
    let research = l.field("forschung");
    if m.research.min > m.research.max {
        ctx.error(
            &research.field("minimum"),
            messages::range_inverted("minimum", "maximum"),
        );
    }
    let transport = l.field("verkehrstraeger");

    let specialization_shares = specialization_shares(
        ctx,
        catalog,
        m,
        &l.field("fachrichtungsanteile"),
        qualifications,
        specializations,
    );

    CountryModel {
        price_reference: Some(resolve(
            ctx,
            countries,
            &m.price_level.reference,
            &price.field("referenzland"),
        )),
        price_elasticity: in_range(
            ctx,
            m.price_level.elasticity,
            0.0,
            2.0,
            &price.field("elastizitaet"),
        ),
        price_min: positive(ctx, m.price_level.min, &price.field("minimum")),
        price_max: positive(ctx, m.price_level.max, &price.field("maximum")),
        participation_rate: in_range(
            ctx,
            m.participation_rate,
            0.01,
            1.0,
            &l.field("erwerbsquote"),
        ),
        labor_share: in_range(ctx, m.labor_share, 0.01, 1.0, &l.field("lohnquote")),
        annual_hours: time_series(ctx, &m.annual_hours, &l.field("jahresarbeitsstunden")),
        qualification_shares: rows(
            ctx,
            catalog,
            qualifications,
            &m.qualification_shares,
            &l.field("qualifikationsanteile"),
            &RowSpec {
                gdp: |r| r.gdp,
                values: |r| &r.shares,
                field: "anteile",
                shares: true,
            },
        ),
        wage_factors: rows(
            ctx,
            catalog,
            qualifications,
            &m.wage_factors,
            &l.field("lohnabstand"),
            &RowSpec {
                gdp: |r| r.gdp,
                values: |r| &r.factors,
                field: "faktoren",
                shares: false,
            },
        ),
        specialization_shares,
        electricity_price_usd_mwh: time_series(
            ctx,
            &m.electricity_price,
            &l.field("strompreis_usd_je_mwh"),
        ),
        grid_reach: bounded(ctx, &m.grid_reach, 0.0, 1.0, &l.field("stromnetz")),
        grid_reference_usd: positive(ctx, m.grid_reference_usd, &l.field("stromnetz_bezug_usd")),
        corporate_tax: bounded(
            ctx,
            &m.corporate_tax,
            0.0,
            1.0,
            &l.field("steuer_unternehmen"),
        ),
        dividend_tax: bounded(
            ctx,
            &m.dividend_tax,
            0.0,
            1.0,
            &l.field("steuer_dividenden"),
        ),
        development_from_usd: positive(ctx, m.development.from, &development.field("von_usd")),
        development_to_usd: positive(ctx, m.development.to, &development.field("bis_usd")),
        rail: bounded(
            ctx,
            &m.transport.rail,
            0.0,
            1.0,
            &transport.field("schiene"),
        ),
        road: bounded(
            ctx,
            &m.transport.road,
            0.0,
            1.0,
            &transport.field("strasse"),
        ),
        air: bounded(ctx, &m.transport.air, 0.0, 1.0, &transport.field("luft")),
        port: bounded(ctx, &m.transport.port, 0.0, 1.0, &transport.field("hafen")),
        stability: in_range(ctx, m.stability, 0.0, 1.0, &l.field("stabilitaet")),
        research_reference_usd: positive(ctx, m.research.reference, &research.field("bezug_usd")),
        research_elasticity: in_range(
            ctx,
            m.research.elasticity,
            0.0,
            2.0,
            &research.field("elastizitaet"),
        ),
        research_min: positive(ctx, m.research.min, &research.field("minimum")),
        research_max: positive(ctx, m.research.max, &research.field("maximum")),
        productivity_reference_usd: positive(
            ctx,
            m.productivity.reference,
            &productivity.field("bezug_usd"),
        ),
        productivity_elasticity: in_range(
            ctx,
            m.productivity.elasticity,
            0.0,
            2.0,
            &productivity.field("elastizitaet"),
        ),
        productivity_min: positive(ctx, m.productivity.min, &productivity.field("minimum")),
        productivity_max: positive(ctx, m.productivity.max, &productivity.field("maximum")),
        automation_base: in_range(
            ctx,
            m.automation.base,
            0.0,
            1.0,
            &l.field("automatisierung").field("basis"),
        ),
        automation_per_doubling: in_range(
            ctx,
            m.automation.per_doubling,
            0.0,
            1.0,
            &l.field("automatisierung").field("je_verdopplung"),
        ),
        automation_reference_usd: positive(
            ctx,
            m.automation.reference,
            &l.field("automatisierung").field("bezug_usd"),
        ),
    }
}

fn specialization_shares(
    ctx: &mut Ctx,
    catalog: &Catalog,
    m: &RawCountryModel,
    loc: &Loc,
    qualifications: &Keys,
    specializations: &Keys,
) -> Vec<Vec<f64>> {
    let mut out = vec![Vec::new(); catalog.qualifications.len()];
    for (key, map) in &m.specialization_shares {
        let q: wsim_core::ids::QualificationId = resolve(ctx, qualifications, key, &loc.key(key));
        if !qualifications.index.contains_key(key) {
            continue;
        }
        if !catalog.qualifications.get(q).has_specialization {
            ctx.error(&loc.key(key), messages::specialization_not_allowed(key));
            continue;
        }
        let q_loc = loc.field(key);
        let mut shares = vec![0.0; catalog.specializations.len()];
        for (s_key, &share) in map {
            let s: wsim_core::ids::SpecializationId =
                resolve(ctx, specializations, s_key, &q_loc.key(s_key));
            in_range(ctx, share, 0.0, 1.0, &q_loc.field(s_key));
            if specializations.index.contains_key(s_key) {
                shares[s.index()] = share;
            }
        }
        for s in catalog.specializations.ids() {
            let s_key = catalog.specializations.key(s);
            if !map.contains_key(s_key) {
                ctx.error(&q_loc, messages::entry_missing("Fachrichtung", s_key));
            }
        }
        check_sum(ctx, &shares, &q_loc);
        out[q.index()] = shares;
    }
    for (q, qualification) in catalog.qualifications.iter() {
        let key = catalog.qualifications.key(q);
        if qualification.has_specialization && !m.specialization_shares.contains_key(key) {
            ctx.error(loc, messages::entry_missing("Qualifikation", key));
        }
    }
    out
}
