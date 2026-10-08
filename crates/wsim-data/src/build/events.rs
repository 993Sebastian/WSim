//! Historical events of `ereignisse` and their effects (Lastenheft §4.1; docs/FORMELN.md, H1).

use wsim_core::EARLIEST_START_YEAR;
use wsim_core::calendar::Date;
use wsim_core::catalog::{EffectKind, EventEffect, EventModel, HistoricalEvent};
use wsim_core::ids::{CountryId, GoodsGroupId};

use super::{Keys, in_range, provenance, resolve};
use crate::TextIndex;
use crate::messages;
use crate::raw::{RawEffectKind, RawEvent, RawEventEffect};
use crate::read::{Ctx, Entry, Loc, RawData};

/// Kinds of historical events; each has a text `ereignisart.<kind>`.
pub const EVENT_KINDS: &[&str] = &[
    "krieg",
    "kriegsende",
    "krise",
    "revolution",
    "staatsgruendung",
    "abkommen",
    "katastrophe",
    "technik",
    "reform",
];

fn parse_date(text: &str) -> Option<Date> {
    let mut parts = text.split('-');
    let year = parts.next()?.parse().ok()?;
    let month = parts.next()?.parse().ok()?;
    let day = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Date::new(year, month, day)
}

/// Historical events, sorted by date; each needs a title and a `.text` description.
pub(super) fn events(
    ctx: &mut Ctx,
    entries: &[&Entry<RawEvent>],
    (countries, groups): (&Keys, &Keys),
    texts: &TextIndex,
) -> Vec<HistoricalEvent> {
    let mut events = Vec::new();
    for e in entries {
        let v = &e.value;
        let l = &e.loc;
        let date = parse_date(&v.date)
            .filter(|d| (super::GAME_YEARS.0..=super::GAME_YEARS.1).contains(&d.year()));
        if date.is_none() {
            ctx.error(&l.field("datum"), messages::event_date_invalid(&v.date));
        }
        if !EVENT_KINDS.contains(&v.kind.as_str()) {
            ctx.error(
                &l.field("art"),
                messages::event_kind_unknown(&v.kind, &EVENT_KINDS.join(", ")),
            );
        }
        let description = format!("ereignis.{}.text", v.id);
        if texts.texts.get(&description).is_none() {
            ctx.error(
                &l.field("id"),
                messages::text_missing(&description, crate::LANGUAGE),
            );
        }
        let event_countries: Vec<CountryId> = v
            .countries
            .iter()
            .enumerate()
            .map(|(i, c)| resolve(ctx, countries, c, &l.field("laender").index(i)))
            .collect();
        let effects = v
            .effects
            .iter()
            .enumerate()
            .map(|(i, w)| {
                let at = l.field("wirkungen").index(i);
                effect(ctx, w, &at, (date, &event_countries), (countries, groups))
            })
            .collect();
        events.push(HistoricalEvent {
            key: v.id.clone(),
            date: date.unwrap_or(Date::first_of_year(EARLIEST_START_YEAR)),
            kind: v.kind.clone(),
            countries: event_countries,
            effects,
            provenance: provenance(v.approximation, v.source.as_ref()),
        });
    }
    events.sort_by(|a, b| a.date.cmp(&b.date).then(a.key.cmp(&b.key)));
    events
}

/// The fields a kind of effect takes besides `art` and `laender`.
fn allowed(kind: RawEffectKind) -> &'static [&'static str] {
    match kind {
        RawEffectKind::Demand => &["warengruppen", "konsum", "staat", "bis"],
        RawEffectKind::Embargo => &["gegen", "bis"],
        RawEffectKind::Tariff => &["gegen", "aufschlag", "bis"],
        RawEffectKind::Labor => &["faktor", "bis"],
        RawEffectKind::Production => &["warengruppen", "faktor", "bis"],
        RawEffectKind::Closure => &["alle", "bis"],
        RawEffectKind::Destruction => &["anteil"],
        RawEffectKind::Expropriation => &["nur_auslaendische", "entschaedigung"],
        RawEffectKind::StockCrash => &["einbruch"],
    }
}

fn effect(
    ctx: &mut Ctx,
    w: &RawEventEffect,
    l: &Loc,
    (date, event_countries): (Option<Date>, &[CountryId]),
    (countries, groups): (&Keys, &Keys),
) -> EventEffect {
    let kind = w.kind.key();
    let present = [
        ("laender", w.countries.is_some()),
        ("gegen", w.against.is_some()),
        ("warengruppen", w.goods_groups.is_some()),
        ("konsum", w.consumer.is_some()),
        ("staat", w.state.is_some()),
        ("faktor", w.factor.is_some()),
        ("aufschlag", w.surcharge.is_some()),
        ("alle", w.all.is_some()),
        ("anteil", w.share.is_some()),
        ("nur_auslaendische", w.foreign_only.is_some()),
        ("entschaedigung", w.compensation.is_some()),
        ("einbruch", w.drop.is_some()),
        ("bis", w.until.is_some()),
    ];
    // A crash acts on the one stock market; all other effects act in countries.
    let local = w.kind != RawEffectKind::StockCrash;
    for (field, given) in present {
        let fits = allowed(w.kind).contains(&field) || (field == "laender" && local);
        if given && !fits {
            ctx.error(
                &l.field(field),
                messages::effect_field_not_allowed(field, kind),
            );
        }
    }
    let country_list = |ctx: &mut Ctx, keys: &[String], field: &str| -> Vec<CountryId> {
        keys.iter()
            .enumerate()
            .map(|(i, c)| resolve(ctx, countries, c, &l.field(field).index(i)))
            .collect()
    };
    let own = match &w.countries {
        Some(list) => country_list(ctx, list, "laender"),
        None => event_countries.to_vec(),
    };
    if local && own.is_empty() {
        ctx.error(
            &l.field("laender"),
            messages::effect_without_countries(kind),
        );
    }
    let against = country_list(ctx, w.against.as_deref().unwrap_or_default(), "gegen");
    if let Some(i) = against.iter().position(|c| own.contains(c)) {
        let key = &w.against.as_deref().unwrap_or_default()[i];
        ctx.error(
            &l.field("gegen").index(i),
            messages::effect_country_on_both_sides(key),
        );
    }
    let group_list: Vec<GoodsGroupId> = w
        .goods_groups
        .as_deref()
        .unwrap_or_default()
        .iter()
        .enumerate()
        .map(|(i, g)| resolve(ctx, groups, g, &l.field("warengruppen").index(i)))
        .collect();
    let required = |ctx: &mut Ctx, value: Option<f64>, field: &str| -> f64 {
        if value.is_none() && allowed(w.kind).contains(&field) {
            ctx.error(l, messages::effect_field_missing(field, kind));
        }
        value.unwrap_or(1.0)
    };
    let kind_value = match w.kind {
        RawEffectKind::Demand => {
            let consumer = in_range(
                ctx,
                w.consumer.unwrap_or(1.0),
                0.0,
                10.0,
                &l.field("konsum"),
            );
            let state = in_range(ctx, w.state.unwrap_or(1.0), 0.0, 10.0, &l.field("staat"));
            if consumer == 1.0 && state == 1.0 {
                ctx.error(l, messages::effect_without_change(kind));
            }
            EffectKind::Demand {
                groups: group_list,
                consumer,
                state,
            }
        }
        RawEffectKind::Embargo => {
            if against.is_empty() {
                ctx.error(l, messages::effect_field_missing("gegen", kind));
            }
            EffectKind::Embargo { against }
        }
        RawEffectKind::Tariff => {
            let surcharge = required(ctx, w.surcharge, "aufschlag");
            EffectKind::Tariff {
                against,
                surcharge: in_range(ctx, surcharge, 0.0, 5.0, &l.field("aufschlag")),
            }
        }
        RawEffectKind::Labor => {
            let factor = required(ctx, w.factor, "faktor");
            EffectKind::Labor {
                factor: in_range(ctx, factor, 0.05, 2.0, &l.field("faktor")),
            }
        }
        RawEffectKind::Production => {
            let factor = required(ctx, w.factor, "faktor");
            EffectKind::Production {
                groups: group_list,
                factor: in_range(ctx, factor, 0.0, 2.0, &l.field("faktor")),
            }
        }
        RawEffectKind::Closure => EffectKind::Closure {
            all: w.all.unwrap_or(false),
        },
        RawEffectKind::Destruction => {
            let share = required(ctx, w.share, "anteil");
            EffectKind::Destruction {
                share: in_range(ctx, share, 0.0, 1.0, &l.field("anteil")),
            }
        }
        RawEffectKind::Expropriation => EffectKind::Expropriation {
            foreign_only: w.foreign_only.unwrap_or(true),
            compensation: in_range(
                ctx,
                w.compensation.unwrap_or(0.0),
                0.0,
                1.0,
                &l.field("entschaedigung"),
            ),
        },
        RawEffectKind::StockCrash => {
            let drop = required(ctx, w.drop, "einbruch");
            EffectKind::StockCrash {
                drop: in_range(ctx, drop, 0.0, 0.95, &l.field("einbruch")),
            }
        }
    };
    let until = w.until.as_ref().and_then(|text| {
        let parsed = parse_date(text);
        match (parsed, date) {
            (None, _) => {
                ctx.error(&l.field("bis"), messages::event_date_invalid(text));
                None
            }
            (Some(u), Some(d)) if u <= d => {
                ctx.error(
                    &l.field("bis"),
                    messages::effect_until_not_after(text, &d.to_string()),
                );
                None
            }
            (Some(u), _) => Some(u),
        }
    });
    EventEffect {
        countries: if local { own } else { Vec::new() },
        kind: kind_value,
        until,
    }
}

/// Parameters of the effects (`ereignisfolgen`); optional, without them the state company
/// gets no working capital.
pub(super) fn event_model(ctx: &mut Ctx, raw: &RawData) -> EventModel {
    let Some((entry, rest)) = raw.event_model.split_first() else {
        return EventModel::default();
    };
    let first = ctx.describe(&entry.loc);
    for other in rest {
        ctx.error(
            &other.loc,
            messages::section_duplicate("ereignisfolgen", &first),
        );
    }
    let v = &entry.value;
    let l = &entry.loc;
    EventModel {
        working_capital_share: in_range(
            ctx,
            v.state_company.working_capital_share,
            0.0,
            1.0,
            &l.field("staatsbetrieb").field("betriebskapital_anteil"),
        ),
        provenance: provenance(v.approximation, v.source.as_ref()),
    }
}
