//! Headquarters and central departments: the parameters of `zentrale` (docs/FORMELN.md,
//! ZA1–ZA3).

use wsim_core::catalog::{
    Catalog, CentralModel, Department, DepartmentKind, HeadquartersModel, HitRateModel,
    RefinanceModel,
};
use wsim_core::money::Money;

use super::{in_range, non_negative, provenance};
use crate::messages;
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
        provenance: provenance(c.approximation, c.source.as_ref()),
    }
}
