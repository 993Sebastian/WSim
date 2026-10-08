//! Stock market of `boerse` (docs/FORMELN.md, K1).

use wsim_core::catalog::StockModel;

use super::{in_range, money, positive, provenance};
use crate::messages;
use crate::read::{Ctx, RawData};

pub(super) fn stock_model(ctx: &mut Ctx, raw: &RawData) -> StockModel {
    // Optional: without the section no company is listed.
    let Some((entry, rest)) = raw.stock.split_first() else {
        return StockModel::default();
    };
    let first = ctx.describe(&entry.loc);
    for other in rest {
        ctx.error(&other.loc, messages::section_duplicate("boerse", &first));
    }
    let v = &entry.value;
    let l = &entry.loc;
    let unit = |ctx: &mut Ctx, x: f64, at: &crate::read::Loc| in_range(ctx, x, 0.0, 1.0, at);
    let (bl, sl, il, dl, hl, stl, al) = (
        l.field("bewertung"),
        l.field("stimmung"),
        l.field("boersengang"),
        l.field("dividende"),
        l.field("handel"),
        l.field("start"),
        l.field("ki"),
    );
    let crises = v
        .crises
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let at = l.field("krisen").index(i);
            in_range(ctx, f64::from(c.month), 1.0, 12.0, &at.field("monat"));
            let drop = in_range(ctx, c.drop, 0.0, 0.95, &at.field("einbruch"));
            (c.year, c.month, drop)
        })
        .collect();
    // The ledger keeps only so many closed months.
    let kept = u32::try_from(wsim_core::ledger::MONTHS_KEPT).unwrap_or(u32::MAX);
    in_range(
        ctx,
        f64::from(v.valuation.earnings_months),
        1.0,
        f64::from(kept),
        &bl.field("gewinn_monate"),
    );
    let dividend_month = in_range(
        ctx,
        f64::from(v.dividend.month),
        1.0,
        12.0,
        &dl.field("monat"),
    );
    StockModel {
        enabled: true,
        book_weight: unit(ctx, v.valuation.book_weight, &bl.field("gewicht_buchwert")),
        pe: positive(ctx, v.valuation.pe, &bl.field("kgv")),
        book_floor: unit(ctx, v.valuation.book_floor, &bl.field("boden_buchwert")),
        assumed_return: unit(
            ctx,
            v.valuation.assumed_return,
            &bl.field("rendite_annahme"),
        ),
        earnings_months: v.valuation.earnings_months,
        sentiment_volatility: in_range(
            ctx,
            v.sentiment.volatility,
            0.0,
            1.0,
            &sl.field("schwankung"),
        ),
        sentiment_reversion: unit(ctx, v.sentiment.reversion, &sl.field("rueckkehr")),
        inertia: unit(ctx, v.inertia, &l.field("traegheit")),
        noise: in_range(ctx, v.noise, 0.0, 1.0, &l.field("rauschen")),
        crises,
        ipo_equity_min: money(ctx, v.ipo.equity_min_usd, &il.field("eigenkapital_min_usd")),
        ipo_share_max: in_range(ctx, v.ipo.share_max, 0.01, 0.9, &il.field("anteil_max")),
        ipo_discount: unit(ctx, v.ipo.discount, &il.field("abschlag")),
        ipo_cost_share: unit(ctx, v.ipo.cost_share, &il.field("kosten_anteil")),
        // In range 1–12 after the check; the cast cannot overflow.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        dividend_month: dividend_month.round() as u32,
        ai_payout: unit(ctx, v.dividend.ai_payout, &dl.field("ki_quote")),
        dividend_cash_max: unit(ctx, v.dividend.cash_max, &dl.field("kasse_max")),
        trade_premium: unit(ctx, v.trading.premium, &hl.field("aufschlag")),
        trade_discount: unit(ctx, v.trading.discount, &hl.field("abschlag")),
        trade_impact: in_range(ctx, v.trading.impact, 0.0, 5.0, &hl.field("preiswirkung")),
        trade_share_max: unit(ctx, v.trading.share_max, &hl.field("anteil_max")),
        start_equity_min: money(
            ctx,
            v.start.equity_min_usd,
            &stl.field("eigenkapital_min_usd"),
        ),
        start_free_float: unit(ctx, v.start.free_float, &stl.field("streubesitz")),
        ai_ipo_chance: unit(ctx, v.ki.ipo_chance, &al.field("boersengang_chance")),
        ai_ipo_share: in_range(
            ctx,
            v.ki.ipo_share,
            0.0,
            0.9,
            &al.field("boersengang_anteil"),
        ),
        takeover_premium: in_range(
            ctx,
            v.takeover.premium,
            0.0,
            5.0,
            &l.field("uebernahme").field("aufschlag"),
        ),
        takeover_cost_share: unit(
            ctx,
            v.takeover.cost_share,
            &l.field("uebernahme").field("kosten_anteil"),
        ),
        buyback_share_max: in_range(
            ctx,
            v.buyback.share_max,
            0.0,
            0.9,
            &l.field("rueckkauf").field("anteil_max"),
        ),
        ai_portfolio_cash_share: unit(
            ctx,
            v.ki.portfolio_cash_share,
            &al.field("depot_anteil_kasse"),
        ),
        ai_portfolio_stake_max: unit(ctx, v.ki.portfolio_stake_max, &al.field("depot_anteil_max")),
        ai_undervaluation: in_range(
            ctx,
            v.ki.undervaluation,
            0.0,
            0.9,
            &al.field("unterbewertung"),
        ),
        ai_takeover_chance: unit(ctx, v.ki.takeover_chance, &al.field("uebernahme_chance")),
        ai_takeover_cash_share: unit(
            ctx,
            v.ki.takeover_cash_share,
            &al.field("uebernahme_kasse_anteil"),
        ),
        provenance: provenance(v.approximation, v.source.as_ref()),
    }
}
