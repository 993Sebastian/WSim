//! Plots of land: the parameters of `grundstuecksmodell` (docs/FORMELN.md, M35).

use std::collections::BTreeMap;

use wsim_core::catalog::{Location, LocationModel, PlotClass, PlotModel};

use super::production::single;
use super::{in_range, non_negative, positive};
use crate::messages;
use crate::raw::{RawPlotLocation, RawWealthShares};
use crate::read::{Ctx, Loc, RawData};
use crate::texts::TextIndex;

/// Text prefix of the size classes.
const CLASS_TEXT: &str = "grundstuecksklasse";
/// Text prefix of the locations.
const LOCATION_TEXT: &str = "lage";

/// Shares (each 0–1) that must add up to one.
fn shares_sum(ctx: &mut Ctx, what: &str, shares: &[f64], loc: &Loc) {
    let sum: f64 = shares.iter().sum();
    if (sum - 1.0).abs() > 1e-6 {
        ctx.error(loc, messages::shares_not_one(what, sum));
    }
}

fn location(ctx: &mut Ctx, l: &RawPlotLocation, loc: &Loc) -> LocationModel {
    LocationModel {
        share: in_range(ctx, l.share, 0.0, 1.0, &loc.field("anteil")),
        area_factor: in_range(ctx, l.area_factor, 0.1, 10.0, &loc.field("flaeche")),
        price_factor: in_range(ctx, l.price_factor, 0.1, 10.0, &loc.field("bodenpreis")),
        hiring: in_range(ctx, l.hiring, -1.0, 1.0, &loc.field("anwerben")),
        sea_freight: in_range(ctx, l.sea_freight, 0.1, 2.0, &loc.field("fracht_see")),
        delivery_cost: in_range(ctx, l.delivery_cost, 0.0, 0.5, &loc.field("lieferkosten")),
    }
}

fn wealth_shares(ctx: &mut Ctx, s: &RawWealthShares, loc: &Loc) -> [f64; 3] {
    [
        in_range(ctx, s.rich, 0.0, 1.0, &loc.field("reich")),
        in_range(ctx, s.middle, 0.0, 1.0, &loc.field("mittel")),
        in_range(ctx, s.poor, 0.0, 1.0, &loc.field("arm")),
    ]
}

pub(super) fn plot_model(ctx: &mut Ctx, raw: &RawData) -> PlotModel {
    let Some(entry) = single(
        ctx,
        &raw.plot_model,
        "grundstuecksmodell",
        "parameter/grundstuecksmodell.yaml",
    ) else {
        return PlotModel::default();
    };
    let m = &entry.value;
    let l = &entry.loc;
    let wealth = l.field("wohlstand");
    if m.wealth.poor_below_usd > m.wealth.rich_from_usd {
        ctx.error(
            &wealth.field("arm_unter_usd"),
            messages::range_inverted("arm_unter_usd", "reich_ab_usd"),
        );
    }
    let classes_loc = l.field("klassen");
    if m.classes.is_empty() {
        ctx.error(&classes_loc, messages::plot_classes_missing());
    }
    let mut seen: BTreeMap<&str, Loc> = BTreeMap::new();
    let mut classes = Vec::new();
    for (i, c) in m.classes.iter().enumerate() {
        let loc = classes_loc.index(i);
        if let Some(first) = seen.get(c.id.as_str()) {
            let first = ctx.describe(first);
            ctx.error(
                &loc.field("id"),
                messages::duplicate_key("Größenklasse", &c.id, &first),
            );
            continue;
        }
        seen.insert(&c.id, loc.clone());
        let area = loc.field("flaeche_ha");
        let from = positive(ctx, c.area_ha.from, &area.field("von"));
        let to = positive(ctx, c.area_ha.to, &area.field("bis"));
        if from > to {
            ctx.error(&area.field("von"), messages::range_inverted("von", "bis"));
        }
        classes.push(PlotClass {
            key: c.id.clone(),
            area_ha: (from, to),
            shares: wealth_shares(ctx, &c.shares, &loc.field("anteile")),
        });
    }
    for (k, level) in ["reich", "mittel", "arm"].into_iter().enumerate() {
        let shares: Vec<f64> = classes.iter().map(|c| c.shares[k]).collect();
        if !shares.is_empty() {
            shares_sum(
                ctx,
                &format!("klassen.anteile.{level}"),
                &shares,
                &classes_loc,
            );
        }
    }
    let loc = l.field("lagen");
    let locations = [
        location(ctx, &m.locations.stadt, &loc.field("stadt")),
        location(ctx, &m.locations.hafen, &loc.field("hafen")),
        location(ctx, &m.locations.land, &loc.field("land")),
    ];
    let shares: Vec<f64> = locations.iter().map(|x| x.share).collect();
    shares_sum(ctx, "lagen.anteil", &shares, &loc);
    let growth = l.field("wachstum");
    let area = l.field("anlagenflaeche");
    PlotModel {
        area_per_gdp_bn_ha: positive(ctx, m.area_per_gdp_bn_ha, &l.field("flaeche_ha_je_mrd_bip")),
        growth_from_year: {
            in_range(
                ctx,
                f64::from(m.growth.from_year),
                1800.0,
                2100.0,
                &growth.field("ab_jahr"),
            );
            m.growth.from_year
        },
        growth_years: positive(ctx, m.growth.years, &growth.field("jahre")),
        rich_from_usd: positive(ctx, m.wealth.rich_from_usd, &wealth.field("reich_ab_usd")),
        poor_below_usd: positive(ctx, m.wealth.poor_below_usd, &wealth.field("arm_unter_usd")),
        classes,
        locations,
        land_price_usd_per_ha: non_negative(
            ctx,
            m.land_price_usd_per_ha,
            &l.field("bodenpreis_usd_je_ha"),
        ),
        scarcity: in_range(ctx, m.scarcity, 0.0, 20.0, &l.field("knappheit")),
        rent_share: in_range(ctx, m.rent_share, 0.0, 1.0, &l.field("pacht_anteil")),
        investment_per_ha_usd: positive(
            ctx,
            m.facility_area.investment_per_ha_usd,
            &area.field("investition_je_ha_usd"),
        ),
        overhead: in_range(
            ctx,
            m.facility_area.overhead,
            0.0,
            5.0,
            &area.field("zuschlag"),
        ),
        min_site_area_ha: non_negative(
            ctx,
            m.facility_area.min_site_area_ha,
            &area.field("mindestflaeche_ha"),
        ),
        ai_reserve: in_range(ctx, m.ai_reserve, 0.0, 5.0, &l.field("ki_reserve")),
    }
}

/// Size classes and locations need display texts (`grundstuecksklasse.<id>`,
/// `lage.<key>`); texts for unknown ones are reported as unused.
pub(super) fn check_texts(
    ctx: &mut Ctx,
    model: &PlotModel,
    raw: &RawData,
    texts: &TextIndex,
    report_unused: bool,
) {
    let Some(entry) = raw.plot_model.first() else {
        return;
    };
    let classes = entry.loc.field("klassen");
    for (i, c) in model.classes.iter().enumerate() {
        let key = format!("{CLASS_TEXT}.{}", c.key);
        if texts.texts.get(&key).is_none() {
            ctx.error(
                &classes.index(i).field("id"),
                messages::text_missing(&key, crate::LANGUAGE),
            );
        }
    }
    for l in Location::ALL {
        let key = format!("{LOCATION_TEXT}.{}", l.key());
        if texts.texts.get(&key).is_none() {
            ctx.error(
                &entry.loc.field("lagen").field(l.key()),
                messages::text_missing(&key, crate::LANGUAGE),
            );
        }
    }
    if !report_unused {
        return;
    }
    for (text_key, loc) in &texts.locations {
        let Some((prefix, rest)) = text_key.split_once('.') else {
            continue;
        };
        let known = match prefix {
            CLASS_TEXT => model.classes.iter().any(|c| c.key == rest),
            LOCATION_TEXT => Location::ALL.iter().any(|l| l.key() == rest),
            _ => true,
        };
        if !known {
            ctx.warning(loc, messages::text_unused(text_key));
        }
    }
}
