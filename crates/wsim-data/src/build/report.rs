//! Limits of the round report: the parameters of `rundenbericht` (docs/FORMELN.md, RB2/RB3).

use wsim_core::catalog::ReportModel;

use super::production::single;
use super::{in_range, non_negative};
use crate::messages;
use crate::read::{Ctx, RawData};

pub(super) fn report_model(ctx: &mut Ctx, raw: &RawData) -> ReportModel {
    let Some(entry) = single(
        ctx,
        &raw.report_model,
        "rundenbericht",
        "parameter/rundenbericht.yaml",
    ) else {
        return ReportModel::default();
    };
    let (m, l) = (&entry.value, &entry.loc);
    let u = l.field("auslastung");
    let red = in_range(
        ctx,
        m.utilization.red_below,
        0.0,
        1.0,
        &u.field("rot_unter"),
    );
    let green = in_range(
        ctx,
        m.utilization.green_from,
        0.0,
        1.0,
        &u.field("gruen_ab"),
    );
    if red >= green {
        ctx.error(&u, messages::range_inverted("rot_unter", "gruen_ab"));
    }
    let drop_loc = l.field("anteil_verlust");
    let drop = non_negative(ctx, m.share_drop, &drop_loc);
    let share_drop = in_range(ctx, drop, 0.0, 1.0, &drop_loc);
    ReportModel {
        visible_products: m.visible_products.clamp(1, 50),
        attention_max: m.attention_max.clamp(1, 20),
        utilization_red: red,
        utilization_green: green,
        share_drop,
    }
}
