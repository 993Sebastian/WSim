//! The player's banks of `bank` (docs/FORMELN.md, K4).

use wsim_core::catalog::BankModel;

use super::{in_range, positive, provenance};
use crate::messages;
use crate::read::{Ctx, RawData};

pub(super) fn bank_model(ctx: &mut Ctx, raw: &RawData) -> BankModel {
    // Optional: without the section there are no banks.
    let Some((entry, rest)) = raw.bank.split_first() else {
        return BankModel::default();
    };
    let first = ctx.describe(&entry.loc);
    for other in rest {
        ctx.error(&other.loc, messages::section_duplicate("bank", &first));
    }
    let v = &entry.value;
    let l = &entry.loc;
    let (dl, sl) = (l.field("einlagen"), l.field("start"));
    let unit = |ctx: &mut Ctx, x: f64, at: &crate::read::Loc| in_range(ctx, x, 0.0, 1.0, at);
    let spread = |ctx: &mut Ctx, x: f64, at: &crate::read::Loc| in_range(ctx, x, -0.2, 0.2, at);
    BankModel {
        enabled: true,
        leverage_max: positive(ctx, v.deposits.leverage_max, &dl.field("hebel_max")),
        neutral_spread: spread(
            ctx,
            v.deposits.neutral_spread,
            &dl.field("aufschlag_neutral"),
        ),
        elasticity: in_range(
            ctx,
            v.deposits.elasticity,
            0.0,
            1000.0,
            &dl.field("elastizitaet"),
        ),
        adjustment: unit(ctx, v.deposits.adjustment, &dl.field("anpassung")),
        reserve: unit(ctx, v.reserve, &l.field("mindestreserve")),
        start_deposit_spread: spread(ctx, v.start.deposit_spread, &sl.field("einlagen_aufschlag")),
        start_loan_discount: in_range(
            ctx,
            v.start.loan_discount,
            0.0,
            0.9,
            &sl.field("kreditnachlass"),
        ),
        start_max_debt_ratio: unit(ctx, v.start.max_debt_ratio, &sl.field("verschuldung_max")),
        provenance: provenance(v.approximation, v.source.as_ref()),
    }
}
