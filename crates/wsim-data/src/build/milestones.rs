//! Milestones of the player after the introduction (M23).

use wsim_core::catalog::{Catalog, Milestone, MilestoneCondition};

use super::in_range;
use crate::messages;
use crate::raw::{RawMilestone, RawMilestoneKind};
use crate::read::{Ctx, Entry};
use crate::texts::TextIndex;

/// Name of a kind in the data, for messages.
fn kind_name(kind: RawMilestoneKind) -> &'static str {
    match kind {
        RawMilestoneKind::FirstSale => "erster_verkauf",
        RawMilestoneKind::ProfitMonth => "gewinnmonat",
        RawMilestoneKind::Facilities => "anlagen",
        RawMilestoneKind::OwnInput => "eigenes_vorprodukt",
        RawMilestoneKind::Countries => "laender",
        RawMilestoneKind::Research => "forschung",
        RawMilestoneKind::MarketLeader => "marktfuehrer",
        RawMilestoneKind::Equity => "eigenkapital",
    }
}

pub(super) fn milestones(
    ctx: &mut Ctx,
    catalog: &mut Catalog,
    entries: &[&Entry<RawMilestone>],
    texts: &TextIndex,
) {
    for e in entries {
        let v = &e.value;
        let l = &e.loc;
        let hint = format!("etappe.{}.hinweis", v.id);
        if texts.texts.get(&hint).is_none() {
            ctx.error(
                &l.field("id"),
                messages::text_missing(&hint, crate::LANGUAGE),
            );
        }
        let name = kind_name(v.kind);
        let needs_value = matches!(
            v.kind,
            RawMilestoneKind::Facilities
                | RawMilestoneKind::Countries
                | RawMilestoneKind::MarketLeader
                | RawMilestoneKind::Equity
        );
        let value = match (needs_value, v.value) {
            (true, None) => {
                ctx.error(&l.field("art"), messages::milestone_value_missing(name));
                1.0
            }
            (false, Some(_)) => {
                ctx.error(
                    &l.field("wert"),
                    messages::milestone_value_not_allowed(name),
                );
                1.0
            }
            (_, value) => value.unwrap_or(1.0),
        };
        let loc = l.field("wert");
        let count = |ctx: &mut Ctx| -> u32 {
            if value.fract() != 0.0 || !(1.0..=1000.0).contains(&value) {
                ctx.error(&loc, messages::milestone_count_invalid(value));
                return 1;
            }
            // Checked above: a whole number from 1 to 1000.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let n = value as u32;
            n
        };
        let condition = match v.kind {
            RawMilestoneKind::FirstSale => MilestoneCondition::FirstSale,
            RawMilestoneKind::ProfitMonth => MilestoneCondition::ProfitMonth,
            RawMilestoneKind::Facilities => MilestoneCondition::Facilities(count(ctx)),
            RawMilestoneKind::OwnInput => MilestoneCondition::OwnInput,
            RawMilestoneKind::Countries => MilestoneCondition::Countries(count(ctx)),
            RawMilestoneKind::Research => MilestoneCondition::Research,
            RawMilestoneKind::MarketLeader => {
                MilestoneCondition::MarketLeader(in_range(ctx, value, 0.01, 1.0, &loc))
            }
            RawMilestoneKind::Equity => {
                if !(value > 1.0 && value <= 1000.0) {
                    ctx.error(&loc, messages::milestone_factor_invalid(value));
                }
                MilestoneCondition::Equity(value)
            }
        };
        catalog.milestones.insert(&v.id, Milestone { condition });
    }
}
