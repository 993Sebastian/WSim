//! Machine offers and builders (docs/MASCHINENANGEBOTE.md, MAA).

use wsim_core::catalog::{MachineBuilder, MachineOfferModel};

use super::{Keys, in_range, non_negative, provenance, resolve};
use crate::messages;
use crate::raw::RawMachineBuilder;
use crate::read::{Ctx, Entry, RawData};

/// The tender parameters. Optional: without the section facilities are bought directly.
pub(super) fn offer_model(ctx: &mut Ctx, raw: &RawData) -> MachineOfferModel {
    let Some((entry, rest)) = raw.machine_offers.split_first() else {
        return MachineOfferModel::default();
    };
    let first = ctx.describe(&entry.loc);
    for other in rest {
        ctx.error(
            &other.loc,
            messages::section_duplicate("maschinenangebote", &first),
        );
    }
    let v = &entry.value;
    let l = &entry.loc;
    let (dl, el) = (l.field("verteilung"), l.field("wirkung"));
    let pair = |ctx: &mut Ctx, found: &[f64], loc: &crate::read::Loc| -> (f64, f64) {
        if found.len() != 2 {
            ctx.error(
                loc,
                messages::wrong_length(found.len(), "2 Werte (von, bis)"),
            );
            return (0.0, 0.0);
        }
        if found[0] > found[1] {
            ctx.error(loc, messages::range_inverted("von", "bis"));
        }
        (found[0], found[1])
    };
    let weekly: Vec<f64> = v.offers_per_week.iter().map(|&n| f64::from(n)).collect();
    let (low, high) = pair(ctx, &weekly, &l.field("angebote_je_woche"));
    let (out_low, out_high) = pair(ctx, &v.distribution.outlier, &dl.field("ausreisser"));
    for (value, loc) in [(out_low, "ausreisser"), (out_high, "ausreisser")] {
        in_range(ctx, value, 0.0, 1.0, &dl.field(loc));
    }
    MachineOfferModel {
        enabled: true,
        tender_days: v.tender_days.clamp(1, 365),
        valid_days: v.valid_days.clamp(1, 365),
        offers_per_week: (low as u32, (high as u32).max(1)),
        normal_share: in_range(
            ctx,
            v.distribution.normal_share,
            0.0,
            1.0,
            &dl.field("normal_anteil"),
        ),
        deviation: in_range(
            ctx,
            v.distribution.deviation,
            0.01,
            1.0,
            &dl.field("abweichung"),
        ),
        outlier: (out_low, out_high),
        output: in_range(ctx, v.effect.output, 0.0, 1.0, &el.field("ausstoss")),
        labor: in_range(ctx, v.effect.labor, 0.0, 1.0, &el.field("personal")),
        inputs: in_range(ctx, v.effect.inputs, 0.0, 1.0, &el.field("vorprodukte")),
        scrap: in_range(ctx, v.effect.scrap, 0.0, 1.0, &el.field("ausschuss")),
        price_performance: in_range(
            ctx,
            v.effect.price_performance,
            0.0,
            1.0,
            &el.field("preis_leistung"),
        ),
        price_position: in_range(
            ctx,
            v.effect.price_position,
            0.0,
            1.0,
            &el.field("preis_lage"),
        ),
        area_spread: in_range(
            ctx,
            v.effect.area_spread,
            0.0,
            1.0,
            &el.field("flaeche_streuung"),
        ),
        build_time: in_range(ctx, v.effect.build_time, 0.0, 1.0, &el.field("bauzeit")),
        noise: in_range(ctx, v.effect.noise, 0.0, 0.5, &el.field("rauschen")),
        limit: in_range(ctx, v.effect.limit, 0.01, 1.0, &el.field("grenze")),
        volume_discount: in_range(
            ctx,
            v.volume_discount,
            0.0,
            1.0,
            &l.field("mengenrabatt_anteil"),
        ),
        ai_threshold: in_range(ctx, v.ai_threshold, -1.0, 1.0, &l.field("ki_schwelle")),
        provenance: provenance(v.approximation, v.source.as_ref()),
    }
}

/// A machine builder of a country.
pub(super) fn builder(
    ctx: &mut Ctx,
    e: &Entry<RawMachineBuilder>,
    countries: &Keys,
) -> MachineBuilder {
    let v = &e.value;
    let l = &e.loc;
    if let Some(until) = v.until
        && until < v.from
    {
        ctx.error(&l.field("bis"), messages::range_inverted("ab", "bis"));
    }
    let _ = non_negative;
    MachineBuilder {
        country: resolve(ctx, countries, &v.country, &l.field("land")),
        tendency: in_range(ctx, v.tendency, -0.3, 0.3, &l.field("tendenz")),
        from_year: v.from,
        until_year: v.until,
    }
}
