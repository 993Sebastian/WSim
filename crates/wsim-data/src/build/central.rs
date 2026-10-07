//! Headquarters and central departments: the parameters of `zentrale` (docs/FORMELN.md,
//! ZA1–ZA3).

use wsim_core::catalog::{
    Catalog, CentralAiModel, CentralModel, Department, DepartmentKind, HeadquartersModel,
    HitRateModel, HqCityModel, RefinanceModel, Span,
};
use wsim_core::money::Money;

use super::{in_range, non_negative, positive, provenance};
use crate::messages;
use crate::raw::RawCentralAi;
use crate::read::{Ctx, Loc, RawData};

/// An amount in USD, at least zero.
fn usd(ctx: &mut Ctx, value: f64, loc: &Loc) -> Money {
    let value = non_negative(ctx, value, loc);
    Money::from_usd(value).unwrap_or(Money::ZERO)
}

pub(super) fn central_model(ctx: &mut Ctx, catalog: &Catalog, raw: &RawData) -> CentralModel {
    // Optional: without the section a move costs nothing and the company has no central
    // departments.
    let Some((entry, rest)) = raw.central.split_first() else {
        return CentralModel::default();
    };
    let first = ctx.describe(&entry.loc);
    for other in rest {
        ctx.error(&other.loc, messages::section_duplicate("zentrale", &first));
    }
    let c = &entry.value;
    let l = &entry.loc;
    let hl = l.field("hauptsitz");
    let h = &c.headquarters;
    in_range(
        ctx,
        f64::from(h.months),
        0.0,
        60.0,
        &hl.field("verlegung_monate"),
    );
    let m = &catalog.management;
    let board = m.levels.get(3).map(|level| level.specialists.as_slice());
    let mut departments: Vec<Department> = Vec::new();
    for (i, d) in c.departments.iter().enumerate() {
        let dl = l.field("abteilungen").index(i);
        let Some(kind) = DepartmentKind::from_key(&d.id) else {
            let keys = DepartmentKind::ALL.map(DepartmentKind::key);
            let suggested = crate::suggest::closest(&d.id, keys.iter().copied());
            ctx.error(
                &dl.field("id"),
                messages::unknown_reference("Abteilung", &d.id, suggested),
            );
            continue;
        };
        if departments.iter().any(|x| x.kind == kind) {
            ctx.error(
                &dl.field("id"),
                messages::duplicate_key("Abteilung", &d.id, "abteilungen"),
            );
            continue;
        }
        let Some(function) = m.function(&d.function) else {
            let suggested =
                crate::suggest::closest(&d.function, m.functions.iter().map(|f| f.key.as_str()));
            ctx.error(
                &dl.field("bereich"),
                messages::unknown_reference("Bereich", &d.function, suggested),
            );
            continue;
        };
        if !board.is_some_and(|b| b.contains(&function)) {
            ctx.error(
                &dl.field("bereich"),
                messages::department_not_on_board(&d.id, &d.function),
            );
        }
        let Some(labor_group) = catalog.labor_groups.id(&d.labor_group) else {
            ctx.error(
                &dl.field("lohngruppe"),
                messages::unknown_reference("Arbeitskräftegruppe", &d.labor_group, None),
            );
            continue;
        };
        departments.push(Department {
            kind,
            function,
            labor_group,
            office: usd(ctx, d.office_usd, &dl.field("buero_usd")),
            cases: non_negative(ctx, d.cases, &dl.field("faelle")),
            effect: in_range(ctx, d.effect, 0.0, 1.0, &dl.field("wirkung")),
        });
    }
    let refinance = c
        .refinance
        .as_ref()
        .map_or_else(RefinanceModel::default, |r| {
            let rl = l.field("umschuldung");
            RefinanceModel {
                min_advantage: in_range(
                    ctx,
                    r.min_advantage,
                    0.0,
                    1.0,
                    &rl.field("mindestvorteil"),
                ),
                fee: in_range(ctx, r.fee, 0.0, 1.0, &rl.field("gebuehr")),
            }
        });
    let hit_rate = c.hit_rate.as_ref().map_or_else(HitRateModel::default, |t| {
        let tl = l.field("trefferquote");
        in_range(
            ctx,
            f64::from(t.months),
            1.0,
            120.0,
            &tl.field("bewertung_monate"),
        );
        HitRateModel {
            months: t.months,
            mean: in_range(ctx, t.mean, 0.0, 1.0, &tl.field("mittelwert")),
            prior: non_negative(ctx, t.prior, &tl.field("vorgewicht")),
            k: in_range(ctx, t.k, 0.0, 10.0, &tl.field("k")),
        }
    });
    let ai = c.ai.as_ref().map_or_else(CentralAiModel::default, |a| {
        ai_model(ctx, a, &departments, &l.field("ki"))
    });
    let city = c.city.as_ref().map(|s| {
        let sl = l.field("stadt");
        HqCityModel {
            academics_concentration: in_range(
                ctx,
                s.academics_concentration,
                0.0,
                100.0,
                &sl.field("akademiker_konzentration"),
            ),
            hq_share: in_range(ctx, s.hq_share, 0.0, 1.0, &sl.field("anteil_zentralen")),
            office_reference_population: positive(
                ctx,
                s.office_reference_population,
                &sl.field("buero_bezug_einwohner"),
            ),
            office_elasticity: in_range(
                ctx,
                s.office_elasticity,
                0.0,
                1.0,
                &sl.field("buero_elastizitaet"),
            ),
            move_within_country: in_range(
                ctx,
                s.move_within_country,
                0.0,
                1.0,
                &sl.field("umzug_im_land"),
            ),
            population_year: {
                in_range(
                    ctx,
                    f64::from(s.population_year),
                    1900.0,
                    2100.0,
                    &sl.field("einwohner_jahr"),
                );
                s.population_year
            },
        }
    });
    CentralModel {
        headquarters: HeadquartersModel {
            months: h.months,
            cost_base: usd(ctx, h.cost_base_usd, &hl.field("kosten_grund_usd")),
            cost_per_employee: usd(
                ctx,
                h.cost_per_employee_usd,
                &hl.field("kosten_je_angestelltem_usd"),
            ),
            moving_share: in_range(ctx, h.moving_share, 0.0, 1.0, &hl.field("mitziehen")),
        },
        departments,
        accuracy: in_range(ctx, c.accuracy, 0.0, 1.0, &l.field("genauigkeit")),
        refinance,
        hit_rate,
        ai,
        city,
        provenance: provenance(c.approximation, c.source.as_ref()),
    }
}

/// A department of the list named by its key, with an error where there is none.
fn listed(
    ctx: &mut Ctx,
    key: &str,
    departments: &[Department],
    loc: &Loc,
) -> Option<DepartmentKind> {
    let kind = DepartmentKind::from_key(key).filter(|&k| departments.iter().any(|d| d.kind == k));
    if kind.is_none() {
        let keys: Vec<&str> = departments.iter().map(|d| d.kind.key()).collect();
        let suggested = crate::suggest::closest(key, keys.iter().copied());
        ctx.error(
            loc,
            messages::unknown_reference("Abteilung", key, suggested),
        );
    }
    kind
}

/// The rules of the AI companies for their central departments and headquarters (ZA4).
fn ai_model(
    ctx: &mut Ctx,
    a: &RawCentralAi,
    departments: &[Department],
    l: &Loc,
) -> CentralAiModel {
    let sl = l.field("anteil_umsatz");
    let revenue_share = Span {
        at_0: in_range(ctx, a.revenue_share.at_0, 0.0, 1.0, &sl.field("bei_0")),
        at_1: in_range(ctx, a.revenue_share.at_1, 0.0, 1.0, &sl.field("bei_1")),
    };
    let mut loads = std::collections::BTreeMap::new();
    for (key, &load) in &a.min_load {
        let ml = l.field("mindestlast").field(key);
        if let Some(kind) = listed(ctx, key, departments, &ml) {
            loads.insert(kind, non_negative(ctx, load, &ml));
        }
    }
    let mut order: Vec<(DepartmentKind, f64)> = Vec::new();
    for (i, key) in a.order.iter().enumerate() {
        let ol = l.field("reihenfolge").index(i);
        let Some(kind) = listed(ctx, key, departments, &ol) else {
            continue;
        };
        if order.iter().any(|&(k, _)| k == kind) {
            ctx.error(
                &ol,
                messages::duplicate_key("Abteilung", key, "reihenfolge"),
            );
            continue;
        }
        order.push((kind, loads.get(&kind).copied().unwrap_or(1.0)));
    }
    let tl = l.field("sitz");
    let seat = &a.seat;
    in_range(
        ctx,
        f64::from(seat.lock_years),
        0.0,
        100.0,
        &tl.field("sperre_jahre"),
    );
    CentralAiModel {
        revenue_share,
        order,
        seat_revenue_share: in_range(
            ctx,
            seat.revenue_share_min,
            0.0,
            1.0,
            &tl.field("anteil_umsatz_min"),
        ),
        seat_gdp_share: in_range(
            ctx,
            seat.gdp_share_min,
            0.0,
            10.0,
            &tl.field("bip_anteil_min"),
        ),
        payback_years: in_range(
            ctx,
            seat.payback_years,
            0.0,
            100.0,
            &tl.field("amortisation_jahre"),
        ),
        lock_years: seat.lock_years,
    }
}
