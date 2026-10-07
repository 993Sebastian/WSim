//! Headquarters and central departments: the parameters of `zentrale` (docs/FORMELN.md,
//! ZA1–ZA3).

use wsim_core::catalog::{Catalog, CentralModel, HeadquartersModel};
use wsim_core::money::Money;

use super::{in_range, non_negative, provenance};
use crate::messages;
use crate::read::{Ctx, Loc, RawData};

/// An amount in USD, at least zero.
fn usd(ctx: &mut Ctx, value: f64, loc: &Loc) -> Money {
    let value = non_negative(ctx, value, loc);
    Money::from_usd(value).unwrap_or(Money::ZERO)
}

pub(super) fn central_model(ctx: &mut Ctx, _catalog: &Catalog, raw: &RawData) -> CentralModel {
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
        provenance: provenance(c.approximation, c.source.as_ref()),
    }
}
