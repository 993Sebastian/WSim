//! Corporate bonds of `anleihen` (docs/FORMELN.md, K2).

use wsim_core::catalog::{BondGrade, BondModel};

use super::{in_range, money, provenance};
use crate::messages;
use crate::read::{Ctx, RawData};

pub(super) fn bond_model(ctx: &mut Ctx, raw: &RawData) -> BondModel {
    // Optional: without the section no company issues bonds.
    let Some((entry, rest)) = raw.bonds.split_first() else {
        return BondModel::default();
    };
    let first = ctx.describe(&entry.loc);
    for other in rest {
        ctx.error(&other.loc, messages::section_duplicate("anleihen", &first));
    }
    let v = &entry.value;
    let l = &entry.loc;
    let unit = |ctx: &mut Ctx, x: f64, at: &crate::read::Loc| in_range(ctx, x, 0.0, 1.0, at);
    let tl = l.field("laufzeit_jahre");
    in_range(
        ctx,
        f64::from(v.term_years.min),
        1.0,
        100.0,
        &tl.field("min"),
    );
    in_range(
        ctx,
        f64::from(v.term_years.max),
        1.0,
        100.0,
        &tl.field("max"),
    );
    if v.term_years.min > v.term_years.max {
        ctx.error(
            &tl,
            messages::term_range_inverted(v.term_years.min, v.term_years.max),
        );
    }
    // The ledger keeps only so many closed months.
    let kept = u32::try_from(wsim_core::ledger::MONTHS_KEPT).unwrap_or(u32::MAX);
    in_range(
        ctx,
        f64::from(v.earnings_months),
        1.0,
        f64::from(kept),
        &l.field("gewinn_monate"),
    );
    if v.grades.is_empty() {
        ctx.error(&l.field("bonitaet"), messages::table_empty());
    }
    let mut grades: Vec<BondGrade> = Vec::new();
    for (i, g) in v.grades.iter().enumerate() {
        let gl = l.field("bonitaet").index(i);
        if !BondGrade::KEYS.contains(&g.id.as_str()) {
            let suggested = crate::suggest::closest(&g.id, BondGrade::KEYS.iter().copied());
            ctx.error(
                &gl.field("stufe"),
                messages::unknown_value(&g.id, &BondGrade::KEYS, suggested),
            );
        } else if grades.iter().any(|x| x.key == g.id) {
            ctx.error(
                &gl.field("stufe"),
                messages::duplicate_key("Bonitätsstufe", &g.id, "bonitaet"),
            );
        }
        let grade = BondGrade {
            key: g.id.clone(),
            debt_ratio_max: unit(ctx, g.debt_ratio_max, &gl.field("verschuldung_max")),
            coverage_min: in_range(
                ctx,
                g.coverage_min,
                0.0,
                100.0,
                &gl.field("zinsdeckung_min"),
            ),
            spread: in_range(ctx, g.spread, 0.0, 0.5, &gl.field("aufschlag")),
        };
        if let Some(better) = grades.last()
            && !(grade.spread > better.spread
                && grade.debt_ratio_max > better.debt_ratio_max
                && grade.coverage_min < better.coverage_min)
        {
            ctx.error(
                &gl.field("stufe"),
                messages::bond_grades_not_ordered(&better.key, &grade.key),
            );
        }
        grades.push(grade);
    }
    let al = l.field("ki");
    in_range(
        ctx,
        f64::from(v.ki.term_years),
        f64::from(v.term_years.min),
        f64::from(v.term_years.max),
        &al.field("laufzeit_jahre"),
    );
    BondModel {
        enabled: true,
        equity_min: money(ctx, v.equity_min_usd, &l.field("eigenkapital_min_usd")),
        volume_min: money(ctx, v.volume_min_usd, &l.field("volumen_min_usd")),
        term_min_years: v.term_years.min,
        term_max_years: v.term_years.max,
        cost_share: unit(ctx, v.cost_share, &l.field("kosten_anteil")),
        redeem_premium: unit(ctx, v.redeem_premium, &l.field("rueckkauf_aufschlag")),
        earnings_months: v.earnings_months,
        grades,
        ai_term_years: v.ki.term_years,
        ai_advantage_min: unit(ctx, v.ki.advantage_min, &al.field("vorteil_min")),
        provenance: provenance(v.approximation, v.source.as_ref()),
    }
}
