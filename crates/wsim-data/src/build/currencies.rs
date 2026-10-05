//! Currencies for display (M21): rates or pegs, the periods of each country, US prices.

use std::collections::BTreeSet;

use wsim_core::EARLIEST_START_YEAR;
use wsim_core::currency::{Currency, CurrencyModel, Rate};

use super::production::single;
use super::{GAME_YEARS, KeyFormat, Keys, in_range, positive, provenance, register, resolve_index};
use crate::messages;
use crate::raw::RawPointInTime;
use crate::read::{Ctx, Loc, RawData};

/// Year and month of a point in time (month 0: only the year).
fn parse(text: &str) -> Option<(i32, u32)> {
    let (year, month) = match text.split_once('-') {
        Some((y, m)) if m.len() == 2 => {
            (y, m.parse::<u32>().ok().filter(|m| (1..=12).contains(m))?)
        }
        Some(_) => return None,
        None => (text, 0),
    };
    if year.len() != 4 {
        return None;
    }
    Some((year.parse().ok()?, month))
}

fn year_in_range(ctx: &mut Ctx, year: i32, loc: &Loc) -> bool {
    let (min, max) = GAME_YEARS;
    if (min..=max).contains(&year) {
        return true;
    }
    ctx.error(loc, messages::year_out_of_range(year, min, max));
    false
}

/// A rate's point in time: a year stands for its average (the middle of the year), a
/// month for its middle.
fn rate_time(ctx: &mut Ctx, p: &RawPointInTime, loc: &Loc) -> Option<f64> {
    let Some((year, month)) = parse(&p.0) else {
        ctx.error(loc, messages::point_in_time_invalid(&p.0));
        return None;
    };
    year_in_range(ctx, year, loc).then(|| {
        f64::from(year)
            + if month == 0 {
                0.5
            } else {
                (f64::from(month) - 0.5) / 12.0
            }
    })
}

/// Start of a period: the first day of the year or month.
fn period_start(ctx: &mut Ctx, p: &RawPointInTime, loc: &Loc) -> Option<f64> {
    let Some((year, month)) = parse(&p.0) else {
        ctx.error(loc, messages::point_in_time_invalid(&p.0));
        return None;
    };
    year_in_range(ctx, year, loc)
        .then(|| f64::from(year) + f64::from(month.saturating_sub(1)) / 12.0)
}

pub(super) fn currency_model(
    ctx: &mut Ctx,
    raw: &RawData,
    countries: &Keys,
) -> (CurrencyModel, Keys) {
    let (keys, entries) = register(
        ctx,
        raw,
        ("waehrungen", "Währung", Some("waehrung")),
        KeyFormat::Snake,
        &raw.currencies,
        |e| &e.id,
    );
    let mut model = CurrencyModel::default();
    let pegged: BTreeSet<&str> = entries
        .iter()
        .filter(|e| e.value.peg.is_some())
        .map(|e| e.value.id.as_str())
        .collect();
    let mut anchors: BTreeSet<usize> = BTreeSet::new();
    for e in &entries {
        let v = &e.value;
        if v.symbol.trim().is_empty() {
            ctx.error(&e.loc.field("zeichen"), messages::field_empty("zeichen"));
        }
        let rate = match (&v.rates, &v.peg) {
            (Some(rates), None) => {
                let l = e.loc.field("kurse");
                let mut points: Vec<(f64, f64)> = Vec::new();
                for (p, &value) in rates {
                    let loc = l.key(&p.0);
                    if let Some(t) = rate_time(ctx, p, &loc) {
                        points.push((t, positive(ctx, value, &loc)));
                    }
                }
                if rates.is_empty() {
                    ctx.error(&l, messages::time_series_empty());
                }
                points.sort_by(|a, b| a.0.total_cmp(&b.0));
                Rate::Points(points)
            }
            (None, Some(peg)) => {
                let l = e.loc.field("bindung");
                let target = resolve_index(ctx, &keys, &peg.currency, &l.field("an"));
                if pegged.contains(peg.currency.as_str()) {
                    ctx.error(&l.field("an"), messages::peg_to_pegged(&peg.currency));
                }
                anchors.insert(target);
                Rate::Peg {
                    currency: target,
                    factor: positive(ctx, peg.factor, &l.field("faktor")),
                }
            }
            _ => {
                ctx.error(&e.loc.field("id"), messages::rates_or_peg());
                Rate::Points(vec![(f64::from(EARLIEST_START_YEAR), 1.0)])
            }
        };
        model.currencies.push(Currency {
            key: v.id.clone(),
            symbol: v.symbol.clone(),
            rate,
            provenance: provenance(v.approximation, v.source.as_ref()),
        });
    }

    if let Some(e) = single(
        ctx,
        &raw.price_index,
        "preisindex",
        "waehrungen/preisindex.yaml",
    ) {
        let v = &e.value;
        model.lead = resolve_index(ctx, &keys, &v.lead_currency, &e.loc.field("leitwaehrung"));
        anchors.insert(model.lead);
        let unit = matches!(
            model.currencies.get(model.lead).map(|c| &c.rate),
            Some(Rate::Points(points)) if points.iter().all(|&(_, r)| (r - 1.0).abs() < 1e-12)
        );
        if keys.index.contains_key(&v.lead_currency) && !unit {
            ctx.error(
                &e.loc.field("leitwaehrung"),
                messages::lead_currency_not_one(&v.lead_currency),
            );
        }
        let l = e.loc.field("werte");
        for (&year, &value) in &v.values {
            let loc = l.key(&year.to_string());
            if year_in_range(ctx, year, &loc) {
                model
                    .us_prices
                    .push((f64::from(year) + 0.5, positive(ctx, value, &loc)));
            }
        }
        if v.values.is_empty() {
            ctx.error(&l, messages::time_series_empty());
        }
        model.base_year = v.base_year;
        model.prices_provenance = provenance(v.approximation, v.source.as_ref());
        if !v.values.contains_key(&v.base_year) {
            ctx.error(
                &e.loc.field("basisjahr"),
                messages::base_year_without_price(v.base_year),
            );
        }
        model.inflation_after = in_range(
            ctx,
            v.inflation_after,
            -0.5,
            1.0,
            &e.loc.field("teuerung_danach"),
        );
    }

    model.periods = vec![Vec::new(); countries.index.len()];
    let mut seen: Vec<Option<Loc>> = vec![None; countries.index.len()];
    for e in &raw.country_currencies {
        let v = &e.value;
        let country_loc = e.loc.field("land");
        let index = resolve_index(ctx, countries, &v.country, &country_loc);
        // An unknown country is reported; its periods are still checked.
        let known = countries.index.contains_key(&v.country);
        if known {
            if let Some(first) = &seen[index] {
                let first = ctx.describe(first);
                ctx.error(
                    &country_loc,
                    messages::country_currencies_duplicate(&v.country, &first),
                );
                continue;
            }
            seen[index] = Some(e.loc.clone());
        }
        let l = e.loc.field("perioden");
        if v.periods.is_empty() {
            ctx.error(&l, messages::list_empty());
        }
        let mut periods: Vec<(f64, usize)> = Vec::new();
        for (i, p) in v.periods.iter().enumerate() {
            let pl = l.index(i);
            let currency = resolve_index(ctx, &keys, &p.currency, &pl.field("waehrung"));
            anchors.insert(currency);
            let Some(from) = period_start(ctx, &p.from, &pl.field("ab")) else {
                continue;
            };
            if periods.last().is_some_and(|&(last, _)| from <= last) {
                ctx.error(&pl.field("ab"), messages::periods_not_ascending());
            }
            if i == 0 && from > f64::from(EARLIEST_START_YEAR) {
                ctx.error(
                    &pl.field("ab"),
                    messages::first_period_late(&p.from.0, EARLIEST_START_YEAR),
                );
            }
            periods.push((from, currency));
        }
        if known {
            model.periods[index] = periods;
        }
    }
    for country in countries.keys_in_order() {
        if seen[countries.index[country]].is_none() && !countries.broken.contains(country) {
            ctx.general_error(messages::country_without_currency(country));
        }
    }
    for (i, e) in entries.iter().enumerate() {
        if !anchors.contains(&i) {
            ctx.warning(&e.loc.field("id"), messages::currency_unused(&e.value.id));
        }
    }
    (model, keys)
}
