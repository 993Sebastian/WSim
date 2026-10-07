//! Import tariffs, trade zones and embargoes of `zoelle` (docs/FORMELN.md, W3).

use wsim_core::catalog::{Catalog, Embargo, TariffDynamics, TariffModel, TariffZone};
use wsim_core::ids::{CountryId, Id};

use super::{GAME_YEARS, in_range, non_negative, provenance, time_series, year};
use crate::messages;
use crate::raw::RawSeries;
use crate::read::{Ctx, Loc, RawData};
use crate::texts::TextIndex;

/// Prefixes of the texts of the zones and the dynamics levels.
const ZONE_TEXT: &str = "zoll.zone";
const LEVEL_TEXT: &str = "zoll.dynamik";
/// Highest average tariff the data may give (share of the value).
const MAX_TARIFF: f64 = 5.0;

fn country(ctx: &mut Ctx, catalog: &Catalog, key: &str, loc: &Loc) -> Option<CountryId> {
    let id = catalog.countries.id(key);
    if id.is_none() {
        let suggested =
            crate::suggest::closest(key, catalog.countries.keys().iter().map(String::as_str));
        ctx.error(loc, messages::unknown_reference("Land", key, suggested));
    }
    id
}

fn series(ctx: &mut Ctx, values: &RawSeries, loc: &Loc) -> wsim_core::time_series::TimeSeries {
    for (&y, &v) in values {
        in_range(ctx, v, 0.0, MAX_TARIFF, &loc.field(&y.to_string()));
    }
    time_series(ctx, values, loc)
}

pub(super) fn tariff_model(ctx: &mut Ctx, catalog: &Catalog, raw: &RawData) -> TariffModel {
    // Optional: without the section there are no tariffs.
    let Some((entry, rest)) = raw.tariffs.split_first() else {
        return TariffModel::default();
    };
    let first = ctx.describe(&entry.loc);
    for other in rest {
        ctx.error(&other.loc, messages::section_duplicate("zoelle", &first));
    }
    let t = &entry.value;
    let l = &entry.loc;
    let default = series(ctx, &t.default, &l.field("standard"));

    let mut countries = vec![None; catalog.countries.len()];
    let cl = l.field("laender");
    for (key, values) in &t.countries {
        if let Some(id) = country(ctx, catalog, key, &cl.key(key)) {
            countries[id.index()] = Some(series(ctx, values, &cl.field(key)));
        }
    }

    let mut groups = vec![1.0; catalog.goods_groups.len()];
    let gl = l.field("warengruppen");
    for (key, &factor) in &t.groups {
        match catalog.goods_groups.id(key) {
            Some(id) => groups[id.index()] = non_negative(ctx, factor, &gl.field(key)),
            None => {
                let suggested = crate::suggest::closest(
                    key,
                    catalog.goods_groups.keys().iter().map(String::as_str),
                );
                ctx.error(
                    &gl.key(key),
                    messages::unknown_reference("Warengruppe", key, suggested),
                );
            }
        }
    }

    let mut products = vec![None; catalog.products.len()];
    let pl = l.field("produkte");
    for (key, &factor) in &t.products {
        match catalog.products.id(key) {
            Some(id) => products[id.index()] = Some(non_negative(ctx, factor, &pl.field(key))),
            None => {
                let suggested = crate::suggest::closest(
                    key,
                    catalog.products.keys().iter().map(String::as_str),
                );
                ctx.error(
                    &pl.key(key),
                    messages::unknown_reference("Produkt", key, suggested),
                );
            }
        }
    }

    let zl = l.field("zonen");
    let mut zones: Vec<TariffZone> = Vec::new();
    for (i, z) in t.zones.iter().enumerate() {
        let at = zl.index(i);
        if zones.iter().any(|x| x.key == z.id) {
            ctx.error(
                &at.field("id"),
                messages::duplicate_key("Zone", &z.id, "zonen"),
            );
            continue;
        }
        let ml = at.field("mitglieder");
        if z.members.is_empty() {
            ctx.error(&ml, messages::list_empty());
        }
        let mut members = Vec::new();
        for (key, years) in &z.members {
            let Some(id) = country(ctx, catalog, key, &ml.key(key)) else {
                continue;
            };
            let place = ml.field(key);
            let (entry_year, exit) = match years.as_slice() {
                [from] => (*from, None),
                [from, until] => (*from, Some(*until)),
                _ => {
                    ctx.error(&place, messages::membership_years(years.len()));
                    continue;
                }
            };
            year(ctx, entry_year, GAME_YEARS, &place);
            if let Some(until) = exit {
                year(ctx, until, GAME_YEARS, &place);
                if until <= entry_year {
                    ctx.error(&place, messages::range_inverted("Beitritt", "Austritt"));
                }
            }
            members.push((id, entry_year, exit));
        }
        zones.push(TariffZone {
            key: z.id.clone(),
            factor: in_range(ctx, z.factor, 0.0, 1.0, &at.field("faktor")),
            members,
        });
    }

    let el = l.field("sperren");
    let mut embargoes = Vec::new();
    for (i, e) in t.embargoes.iter().enumerate() {
        let at = el.index(i);
        let ids: Vec<Option<CountryId>> = e
            .countries
            .iter()
            .enumerate()
            .map(|(j, key)| country(ctx, catalog, key, &at.field("laender").index(j)))
            .collect();
        year(ctx, e.from, GAME_YEARS, &at.field("von"));
        if let Some(until) = e.until {
            year(ctx, until, GAME_YEARS, &at.field("bis"));
            if until <= e.from {
                ctx.error(&at.field("bis"), messages::range_inverted("von", "bis"));
            }
        }
        match ids.as_slice() {
            [Some(a), Some(b)] if a != b => embargoes.push(Embargo {
                countries: (*a, *b),
                from: e.from,
                until: e.until,
            }),
            [Some(_), Some(_)] | [_] | [] | [_, _, _, ..] => {
                ctx.error(&at.field("laender"), messages::embargo_countries());
            }
            _ => {}
        }
    }

    let d = &t.dynamics;
    let dl = l.field("dynamik");
    let min = in_range(ctx, d.min, 0.0, MAX_TARIFF, &dl.field("minimum"));
    let max = in_range(ctx, d.max, 0.0, MAX_TARIFF, &dl.field("maximum"));
    if max < min {
        ctx.error(
            &dl.field("maximum"),
            messages::range_inverted("minimum", "maximum"),
        );
    }
    let sl = dl.field("stufen");
    if d.levels.is_empty() {
        ctx.error(&sl, messages::list_empty());
    }
    let mut levels: Vec<(String, f64)> = Vec::new();
    for (i, level) in d.levels.iter().enumerate() {
        if levels.iter().any(|(k, _)| *k == level.id) {
            ctx.error(
                &sl.index(i).field("id"),
                messages::duplicate_key("Stufe", &level.id, "stufen"),
            );
            continue;
        }
        let factor = in_range(ctx, level.factor, 0.0, 10.0, &sl.index(i).field("faktor"));
        levels.push((level.id.clone(), factor));
    }
    let default_level = match levels.iter().position(|(k, _)| *k == d.default) {
        Some(i) => i,
        None => {
            if !levels.is_empty() {
                ctx.error(
                    &dl.field("standard"),
                    messages::tariff_level_unknown(&d.default),
                );
            }
            0
        }
    };

    TariffModel {
        default: Some(default),
        countries,
        groups,
        products,
        zones,
        embargoes,
        dynamics: TariffDynamics {
            deviation: in_range(ctx, d.deviation, 0.0, 1.0, &dl.field("standardabweichung")),
            min,
            max: max.max(min),
            levels,
            default_level,
        },
        provenance: provenance(t.approximation, t.source.as_ref()),
    }
}

/// Every zone and dynamics level needs its text.
pub(super) fn check_texts(
    ctx: &mut Ctx,
    model: &TariffModel,
    raw: &RawData,
    texts: &TextIndex,
    report_unused: bool,
) {
    let Some(entry) = raw.tariffs.first() else {
        return;
    };
    let zones: Vec<&str> = model.zones.iter().map(|z| z.key.as_str()).collect();
    let levels: Vec<&str> = model.dynamics.levels.iter().map(|l| l.0.as_str()).collect();
    let wanted: [(&str, Loc, Vec<&str>); 2] = [
        (ZONE_TEXT, entry.loc.field("zonen"), zones),
        (
            LEVEL_TEXT,
            entry.loc.field("dynamik").field("stufen"),
            levels,
        ),
    ];
    for (prefix, at, keys) in &wanted {
        for (i, key) in keys.iter().enumerate() {
            let text = format!("{prefix}.{key}");
            if texts.texts.get(&text).is_none() {
                ctx.error(
                    &at.index(i).field("id"),
                    messages::text_missing(&text, crate::LANGUAGE),
                );
            }
        }
    }
    if !report_unused {
        return;
    }
    for (text_key, loc) in &texts.locations {
        for (prefix, _, keys) in &wanted {
            if let Some(rest) = text_key
                .strip_prefix(prefix)
                .and_then(|r| r.strip_prefix('.'))
                && !keys.contains(&rest)
            {
                ctx.warning(loc, messages::text_unused(text_key));
            }
        }
    }
}
