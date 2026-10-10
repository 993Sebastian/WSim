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
    let share =
        |ctx: &mut Ctx, value: f64, field: &str| in_range(ctx, value, 0.0, 1.0, &al.field(field));
    let sale_discount = share(ctx, a.sale_discount, "abschlag_verkauf");
    let sale_discount_max = share(ctx, a.sale_discount_max, "abschlag_max");
    if sale_discount_max < sale_discount {
        ctx.error(
            &al.field("abschlag_max"),
            messages::range_inverted("abschlag_verkauf", "abschlag_max"),
        );
    }
    let purchase_premium = share(ctx, a.purchase_premium, "aufschlag_kauf");
    let purchase_premium_max = share(ctx, a.purchase_premium_max, "aufschlag_max");
    if purchase_premium_max < purchase_premium {
        ctx.error(
            &al.field("aufschlag_max"),
            messages::range_inverted("aufschlag_kauf", "aufschlag_max"),
        );
    }
    ContractModel {
        months_max,
        months_default,
        penalty_max,
        penalty_default,
        cancel_months: months(ctx, c.cancel_months, 0.0, 600.0, "kuendigung_monate"),
        proposal_days: months(ctx, c.proposal_days, 1.0, 365.0, "angebot_tage"),
        keep_months: months(ctx, c.keep_months, 0.0, 600.0, "aufbewahren_monate"),
        ai: ContractAiModel {
            sale_discount,
            sale_discount_per_month: share(ctx, a.sale_discount_per_month, "abschlag_je_monat"),
            sale_discount_max,
            purchase_premium,
            purchase_premium_per_month: share(
                ctx,
                a.purchase_premium_per_month,
                "aufschlag_je_monat",
            ),
            purchase_premium_max,
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
