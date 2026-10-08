//! Fleets, state transport and freight risk of `logistik` (docs/FORMELN.md, W5).

use wsim_core::catalog::LogisticsModel;

use super::{in_range, positive, provenance, time_series};
use crate::messages;
use crate::raw::RawSeries;
use crate::read::{Ctx, Loc, RawData};

/// A series of chances between 0 and 1.
fn chances(ctx: &mut Ctx, values: &RawSeries, loc: &Loc) -> wsim_core::time_series::TimeSeries {
    for (&y, &v) in values {
        in_range(ctx, v, 0.0, 1.0, &loc.field(&y.to_string()));
    }
    time_series(ctx, values, loc)
}

pub(super) fn logistics_model(ctx: &mut Ctx, raw: &RawData) -> LogisticsModel {
    // Optional: without the section only the freight market of stage 1.
    let Some((entry, rest)) = raw.logistics.split_first() else {
        return LogisticsModel::default();
    };
    let first = ctx.describe(&entry.loc);
    for other in rest {
        ctx.error(&other.loc, messages::section_duplicate("logistik", &first));
    }
    let v = &entry.value;
    let l = &entry.loc;
    let (sl, fl, rl, al) = (
        l.field("staat"),
        l.field("flotte"),
        l.field("risiko"),
        l.field("ki"),
    );
    let f = &v.fleet;
    LogisticsModel {
        enabled: true,
        state_surcharge: in_range(ctx, v.state.surcharge, 0.0, 10.0, &sl.field("aufschlag")),
        state_risk_factor: in_range(
            ctx,
            v.state.risk_factor,
            0.0,
            10.0,
            &sl.field("risiko_faktor"),
        ),
        market_margin: in_range(
            ctx,
            f.market_margin,
            0.0,
            0.9,
            &fl.field("marge_frachtmarkt"),
        ),
        load: in_range(ctx, f.load, 0.01, 1.0, &fl.field("auslastung")),
        upkeep_share: in_range(ctx, f.upkeep_share, 0.0, 1.0, &fl.field("unterhalt_anteil")),
        life_years: positive(ctx, f.life_years, &fl.field("nutzungsdauer_jahre")),
        sale_share: in_range(ctx, f.sale_share, 0.0, 1.0, &fl.field("verkauf_anteil")),
        rental_share: in_range(
            ctx,
            f.rental_share,
            0.0,
            1.0,
            &fl.field("vermietung_anteil"),
        ),
        risk_land: Some(chances(ctx, &v.risk.land, &rl.field("land"))),
        risk_sea: Some(chances(ctx, &v.risk.sea, &rl.field("see"))),
        ai_share: in_range(ctx, v.ki.share, 0.0, 1.0, &al.field("anteil")),
        ai_cash_share: in_range(ctx, v.ki.cash_share, 0.0, 1.0, &al.field("kasse_anteil")),
        provenance: provenance(v.approximation, v.source.as_ref()),
    }
}
