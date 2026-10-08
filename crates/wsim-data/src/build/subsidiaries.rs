//! Subsidiaries of `tochterfirmen` (docs/FORMELN.md, W6).

use wsim_core::catalog::SubsidiaryModel;

use super::{in_range, money, provenance};
use crate::messages;
use crate::read::{Ctx, RawData};

pub(super) fn subsidiary_model(ctx: &mut Ctx, raw: &RawData) -> SubsidiaryModel {
    // Optional: without the section the player cannot found subsidiaries.
    let Some((entry, rest)) = raw.subsidiaries.split_first() else {
        return SubsidiaryModel::default();
    };
    let first = ctx.describe(&entry.loc);
    for other in rest {
        ctx.error(
            &other.loc,
            messages::section_duplicate("tochterfirmen", &first),
        );
    }
    let v = &entry.value;
    let l = &entry.loc;
    let (ml, ll) = (l.field("geschaeftsfuehrung"), l.field("logistik"));
    SubsidiaryModel {
        enabled: true,
        min_capital: money(ctx, v.min_capital_usd, &l.field("mindestkapital_usd")),
        founding_cost: money(ctx, v.founding_cost_usd, &l.field("gruendungskosten_usd")),
        competence: in_range(
            ctx,
            v.management.competence,
            0.0,
            1.0,
            &ml.field("kompetenz"),
        ),
        aggressiveness: in_range(
            ctx,
            v.management.aggressiveness,
            0.0,
            1.0,
            &ml.field("aggressivitaet"),
        ),
        logistics_cash_share: in_range(
            ctx,
            v.logistics.cash_share,
            0.0,
            1.0,
            &ll.field("kasse_anteil"),
        ),
        logistics_min_return: in_range(
            ctx,
            v.logistics.min_return,
            0.0,
            10.0,
            &ll.field("rendite_min"),
        ),
        provenance: provenance(v.approximation, v.source.as_ref()),
    }
}
