//! Environment and regulations (`umwelt`, `regulierungen`; Lastenheft §12, docs/FORMELN.md
//! H2).

use wsim_core::calendar::Date;
use wsim_core::catalog::{Catalog, EnvironmentModel, Regulation, RegulationKind, RetrofitLevel};
use wsim_core::ids::{CountryId, ProductId};

use super::events::parse_date;
use super::life::country_series;
use super::{Keys, in_range, provenance, resolve};
use crate::messages;
use crate::raw::RawRegulation;
use crate::read::{Ctx, Entry, Loc, RawData};

/// Kinds of regulations and the fields each takes besides `id`, `datum`, `laender`, `art`.
const KINDS: &[(&str, &[&str])] = &[
    ("auflage", &["stufe", "frist_monate"]),
    ("arbeitsschutz", &["lohnaufschlag"]),
    ("verbot", &["produkte", "herstellung", "verkauf"]),
    ("kartellaufsicht", &[]),
];

pub(super) fn environment_model(
    ctx: &mut Ctx,
    catalog: &Catalog,
    raw: &RawData,
) -> EnvironmentModel {
    let Some((entry, rest)) = raw.environment.split_first() else {
        return EnvironmentModel::default();
    };
    let first = ctx.describe(&entry.loc);
    for other in rest {
        ctx.error(&other.loc, messages::section_duplicate("umwelt", &first));
    }
    let v = &entry.value;
    let l = &entry.loc;
    let rl = l.field("nachruestung");
    let retrofit: Vec<RetrofitLevel> = v
        .retrofit
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let at = rl.index(i);
            RetrofitLevel {
                from_year: r.from_year,
                reduction: in_range(ctx, r.reduction, 0.0, 1.0, &at.field("minderung")),
                cost_share: in_range(ctx, r.cost_share, 0.0, 10.0, &at.field("kosten_anteil")),
            }
        })
        .collect();
    if retrofit.windows(2).any(|w| w[0].from_year > w[1].from_year) {
        ctx.error(&rl, messages::retrofit_years_descending());
    }
    EnvironmentModel {
        retrofit,
        co2_price: country_series(
            ctx,
            catalog,
            &v.co2_price,
            &l.field("co2_preis"),
            (0.0, 10_000.0),
        ),
        antitrust_share_max: in_range(
            ctx,
            v.antitrust.share_max,
            0.0,
            1.0,
            &l.field("kartell").field("marktanteil_max"),
        ),
        image_weight: in_range(
            ctx,
            v.image.weight,
            0.0,
            1.0,
            &l.field("markenbild").field("gewicht"),
        ),
        provenance: provenance(v.approximation, v.source.as_ref()),
    }
}

/// The regulations, sorted by date; each needs a text `regulierung.<id>`.
pub(super) fn regulations(
    ctx: &mut Ctx,
    catalog: &Catalog,
    entries: &[&Entry<RawRegulation>],
    (countries, products): (&Keys, &Keys),
) -> Vec<Regulation> {
    let mut out = Vec::new();
    for e in entries {
        let v = &e.value;
        let l = &e.loc;
        let date = parse_date(&v.date)
            .filter(|d| (super::GAME_YEARS.0..=super::GAME_YEARS.1).contains(&d.year()));
        if date.is_none() {
            ctx.error(&l.field("datum"), messages::event_date_invalid(&v.date));
        }
        if v.countries.is_empty() {
            ctx.error(
                &l.field("laender"),
                messages::regulation_without_countries(),
            );
        }
        let ids: Vec<CountryId> = v
            .countries
            .iter()
            .enumerate()
            .map(|(i, c)| resolve(ctx, countries, c, &l.field("laender").index(i)))
            .collect();
        let Some(kind) = kind(ctx, catalog, v, l, products) else {
            continue;
        };
        out.push(Regulation {
            key: v.id.clone(),
            date: date.unwrap_or(Date::first_of_year(super::GAME_YEARS.0)),
            countries: ids,
            kind,
            provenance: provenance(v.approximation, v.source.as_ref()),
        });
    }
    out.sort_by(|a, b| a.date.cmp(&b.date).then(a.key.cmp(&b.key)));
    out
}

fn kind(
    ctx: &mut Ctx,
    catalog: &Catalog,
    v: &RawRegulation,
    l: &Loc,
    products: &Keys,
) -> Option<RegulationKind> {
    let Some((name, allowed)) = KINDS.iter().find(|(k, _)| *k == v.kind) else {
        let all: Vec<&str> = KINDS.iter().map(|(k, _)| *k).collect();
        ctx.error(
            &l.field("art"),
            messages::regulation_kind_unknown(&v.kind, &all.join(", ")),
        );
        return None;
    };
    let given = [
        ("stufe", v.level.is_some()),
        ("frist_monate", v.months.is_some()),
        ("lohnaufschlag", v.wage_surcharge.is_some()),
        ("produkte", !v.products.is_empty()),
        ("herstellung", v.production),
        ("verkauf", v.sales),
    ];
    for (field, set) in given {
        if set && !allowed.contains(&field) {
            ctx.error(
                &l.field(field),
                messages::regulation_field_not_allowed(field, name),
            );
        }
    }
    let missing = |ctx: &mut Ctx, field: &str| {
        ctx.error(
            &l.field("art"),
            messages::regulation_field_missing(field, name),
        );
    };
    Some(match *name {
        "auflage" => {
            let levels = catalog.environment.retrofit.len();
            let level = v.level.unwrap_or_else(|| {
                missing(ctx, "stufe");
                1
            });
            if level == 0 || level as usize > levels {
                ctx.error(
                    &l.field("stufe"),
                    messages::retrofit_level_unknown(level, levels),
                );
            }
            let months = v.months.unwrap_or_else(|| {
                missing(ctx, "frist_monate");
                0
            });
            if months > 240 {
                ctx.error(
                    &l.field("frist_monate"),
                    messages::out_of_range(f64::from(months), 0.0, 240.0),
                );
            }
            RegulationKind::Retrofit { level, months }
        }
        "arbeitsschutz" => {
            let surcharge = v.wage_surcharge.unwrap_or_else(|| {
                missing(ctx, "lohnaufschlag");
                0.0
            });
            RegulationKind::Safety {
                wage_surcharge: in_range(ctx, surcharge, 0.0, 1.0, &l.field("lohnaufschlag")),
            }
        }
        "verbot" => {
            if v.products.is_empty() {
                ctx.error(&l.field("produkte"), messages::ban_without_products());
            }
            if !v.production && !v.sales {
                ctx.error(&l.field("art"), messages::ban_without_effect());
            }
            let ids: Vec<ProductId> = v
                .products
                .iter()
                .enumerate()
                .map(|(i, p)| resolve(ctx, products, p, &l.field("produkte").index(i)))
                .collect();
            RegulationKind::Ban {
                products: ids,
                production: v.production,
                sales: v.sales,
            }
        }
        _ => RegulationKind::Antitrust,
    })
}
