//! Offers between companies: the parameters of `kaufmodell` (docs/FORMELN.md, M30).

use wsim_core::catalog::{Catalog, DealAi, DealModel, Span};

use super::production::single;
use super::{in_range, non_negative};
use crate::messages;
use crate::raw::RawSpan;
use crate::read::{Ctx, Loc, RawData};

/// A span whose values lie between `min` and `max`.
fn span(ctx: &mut Ctx, s: &RawSpan, min: f64, max: f64, loc: &Loc) -> Span {
    Span {
        at_0: in_range(ctx, s.at_0, min, max, &loc.field("bei_0")),
        at_1: in_range(ctx, s.at_1, min, max, &loc.field("bei_1")),
    }
}

fn months(ctx: &mut Ctx, value: u32, min: u32, max: u32, loc: &Loc) -> u32 {
    in_range(ctx, f64::from(value), f64::from(min), f64::from(max), loc);
    value
}

pub(super) fn deal_model(ctx: &mut Ctx, catalog: &Catalog, raw: &RawData) -> DealModel {
    let Some(entry) = single(
        ctx,
        &raw.deal_model,
        "kaufmodell",
        "parameter/kaufmodell.yaml",
    ) else {
        return DealModel::default();
    };
    let m = &entry.value;
    let l = &entry.loc;
    let max_rank = catalog
        .qualifications
        .values()
        .map(|q| q.rank)
        .max()
        .unwrap_or(0);
    if m.qualified_rank == 0 || m.qualified_rank > max_rank {
        ctx.error(
            &l.field("qualifiziert_ab_stufe"),
            messages::rank_unknown(m.qualified_rank, max_rank),
        );
    }
    let a = &m.ki;
    let k = l.field("ki");
    DealModel {
        valid_months: months(ctx, m.valid_months, 1, 24, &l.field("gueltig_monate")),
        block_months: months(ctx, m.block_months, 0, 120, &l.field("sperre_monate")),
        min_age_months: months(
            ctx,
            m.min_age_months,
            0,
            120,
            &l.field("mindestalter_monate"),
        ),
        earnings_years: in_range(ctx, m.earnings_years, 0.5, 30.0, &l.field("ertragsfaktor")),
        earnings_min_months: months(
            ctx,
            m.earnings_min_months,
            1,
            12,
            &l.field("ertrag_mindestmonate"),
        ),
        goodwill_years: in_range(
            ctx,
            m.goodwill_years,
            1.0,
            40.0,
            &l.field("firmenwert_jahre"),
        ),
        qualified_rank: m.qualified_rank,
        ai: DealAi {
            chance: span(ctx, &a.chance, 0.0, 1.0, &k.field("angebot_chance")),
            open_max: months(ctx, a.open_max, 0, 20, &k.field("offene_angebote_max")),
            player_offers_per_month: months(
                ctx,
                a.player_offers_per_month,
                0,
                10,
                &k.field("spieler_angebote_je_monat"),
            ),
            min_advantage: in_range(ctx, a.min_advantage, 0.0, 5.0, &k.field("mindestvorteil")),
            bid_markup: span(ctx, &a.bid_markup, 0.0, 5.0, &k.field("gebotsaufschlag")),
            min_price_usd: non_negative(ctx, a.min_price_usd, &k.field("mindestpreis_usd")),
            cash_share_max: in_range(
                ctx,
                a.cash_share_max,
                0.0,
                1.0,
                &k.field("kasse_anteil_max"),
            ),
            competition_markup: span(
                ctx,
                &a.competition_markup,
                0.0,
                5.0,
                &k.field("wettbewerb_aufschlag"),
            ),
            staff_markup: in_range(
                ctx,
                a.staff_markup,
                0.0,
                5.0,
                &k.field("fachkraefte_aufschlag"),
            ),
            build_time_markup: in_range(
                ctx,
                a.build_time_markup,
                0.0,
                5.0,
                &k.field("bauzeit_aufschlag"),
            ),
            new_build_share: in_range(ctx, a.new_build_share, 0.0, 1.0, &k.field("neubau_anteil")),
            license_bid: span(ctx, &a.license_bid, 0.0, 2.0, &k.field("lizenz_gebot")),
            license_max: in_range(ctx, a.license_max, 0.0, 2.0, &k.field("lizenz_hoechst")),
            sale_markup: span(ctx, &a.sale_markup, 0.0, 5.0, &k.field("verkaufsaufschlag")),
            core_share: in_range(ctx, a.core_share, 0.0, 1.0, &k.field("kern_anteil")),
            core_markup: in_range(ctx, a.core_markup, 0.0, 10.0, &k.field("kern_aufschlag")),
            license_min: in_range(ctx, a.license_min, 0.0, 2.0, &k.field("lizenz_mindest")),
            license_competition: in_range(
                ctx,
                a.license_competition,
                0.0,
                10.0,
                &k.field("wettbewerb_lizenz"),
            ),
            counter_threshold: in_range(
                ctx,
                a.counter_threshold,
                0.0,
                1.0,
                &k.field("gegen_schwelle"),
            ),
        },
    }
}
