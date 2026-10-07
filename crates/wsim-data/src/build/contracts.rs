//! Supply contracts of `vertraege` (docs/FORMELN.md, W4).

use wsim_core::catalog::{ContractAiModel, ContractModel};

use super::{in_range, provenance};
use crate::messages;
use crate::read::{Ctx, RawData};

pub(super) fn contract_model(ctx: &mut Ctx, raw: &RawData) -> ContractModel {
    // Optional: without the section there are no contracts.
    let Some((entry, rest)) = raw.contracts.split_first() else {
        return ContractModel::default();
    };
    let first = ctx.describe(&entry.loc);
    for other in rest {
        ctx.error(&other.loc, messages::section_duplicate("vertraege", &first));
    }
    let c = &entry.value;
    let l = &entry.loc;
    let months = |ctx: &mut Ctx, value: u32, min: f64, max: f64, field: &str| {
        in_range(ctx, f64::from(value), min, max, &l.field(field));
        value
    };
    let months_max = months(ctx, c.months_max, 1.0, 600.0, "laufzeit_monate_max");
    let months_default = months(ctx, c.months_default, 1.0, 600.0, "laufzeit_standard");
    if months_default > months_max {
        ctx.error(
            &l.field("laufzeit_standard"),
            messages::range_inverted("laufzeit_standard", "laufzeit_monate_max"),
        );
    }
    let penalty_max = in_range(ctx, c.penalty_max, 0.0, 10.0, &l.field("strafe_max"));
    let penalty_default = in_range(
        ctx,
        c.penalty_default,
        0.0,
        penalty_max.max(0.0),
        &l.field("strafe_standard"),
    );
    let a = &c.ki;
    let al = l.field("ki");
    ContractModel {
        months_max,
        months_default,
        penalty_max,
        penalty_default,
        cancel_months: months(ctx, c.cancel_months, 0.0, 600.0, "kuendigung_monate"),
        proposal_days: months(ctx, c.proposal_days, 1.0, 365.0, "angebot_tage"),
        keep_months: months(ctx, c.keep_months, 0.0, 600.0, "aufbewahren_monate"),
        ai: ContractAiModel {
            sale_discount: in_range(
                ctx,
                a.sale_discount,
                0.0,
                1.0,
                &al.field("abschlag_verkauf"),
            ),
            purchase_premium: in_range(
                ctx,
                a.purchase_premium,
                0.0,
                1.0,
                &al.field("aufschlag_kauf"),
            ),
            share: in_range(ctx, a.share, 0.0, 1.0, &al.field("anteil")),
            penalty_max: in_range(ctx, a.penalty_max, 0.0, 10.0, &al.field("strafe_max")),
            proposal_chance: in_range(
                ctx,
                a.proposal_chance,
                0.0,
                1.0,
                &al.field("angebot_chance"),
            ),
        },
        provenance: provenance(c.approximation, c.source.as_ref()),
    }
}
