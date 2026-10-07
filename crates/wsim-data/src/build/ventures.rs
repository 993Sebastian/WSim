//! Start-ups: the parameters of `startups` and the historical inventors of `erfinder`
//! (docs/FORMELN.md, SU1–SU3).

use wsim_core::catalog::{Catalog, Inventor, VentureModel, VenturePhase};
use wsim_core::money::Money;

use super::{in_range, non_negative, positive, provenance};
use crate::messages;
use crate::read::{Ctx, Loc, RawData};
use crate::texts::TextIndex;

/// Prefixes of the texts of the labels and the phases.
const LABEL_TEXT: &str = "startup.bezeichnung";
const PHASE_TEXT: &str = "startup.phase";
const FREQUENCY_TEXT: &str = "startup.haeufigkeit";

pub(super) fn venture_model(ctx: &mut Ctx, catalog: &Catalog, raw: &RawData) -> VentureModel {
    // Optional: without the section there are no start-ups.
    let Some((entry, rest)) = raw.ventures.split_first() else {
        if let Some(first) = raw.inventors.first() {
            ctx.warning(&first.loc, messages::inventors_without_ventures());
        }
        return VentureModel::default();
    };
    let first = ctx.describe(&entry.loc);
    for other in rest {
        ctx.error(&other.loc, messages::section_duplicate("startups", &first));
    }
    let v = &entry.value;
    let l = &entry.loc;
    let ll = l.field("bezeichnungen");
    if v.labels.is_empty() {
        ctx.error(&ll, messages::list_empty());
    }
    let mut labels: Vec<(String, i32)> = Vec::new();
    for (i, label) in v.labels.iter().enumerate() {
        if labels.iter().any(|(k, _)| *k == label.id) {
            ctx.error(
                &ll.index(i).field("id"),
                messages::duplicate_key("Bezeichnung", &label.id, "bezeichnungen"),
            );
            continue;
        }
        if labels.last().is_some_and(|&(_, from)| from >= label.from) {
            ctx.error(&ll.index(i).field("ab"), messages::periods_not_ascending());
        }
        labels.push((label.id.clone(), label.from));
    }
    let fl = l.field("haeufigkeiten");
    if v.frequencies.is_empty() {
        ctx.error(&fl, messages::list_empty());
    }
    let mut frequencies: Vec<(String, f64)> = Vec::new();
    for (i, f) in v.frequencies.iter().enumerate() {
        if frequencies.iter().any(|(k, _)| *k == f.id) {
            ctx.error(
                &fl.index(i).field("id"),
                messages::duplicate_key("Häufigkeit", &f.id, "haeufigkeiten"),
            );
            continue;
        }
        let factor = in_range(ctx, f.factor, 0.0, 10.0, &fl.index(i).field("faktor"));
        frequencies.push((f.id.clone(), factor));
    }
    let default_frequency = match frequencies
        .iter()
        .position(|(k, _)| *k == v.default_frequency)
    {
        Some(i) => i,
        None => {
            if !frequencies.is_empty() {
                ctx.error(
                    &l.field("haeufigkeit_standard"),
                    messages::default_frequency_unknown(&v.default_frequency),
                );
            }
            0
        }
    };
    let pl = l.field("phasen");
    if v.phases.is_empty() {
        ctx.error(&pl, messages::list_empty());
    }
    let mut phases: Vec<VenturePhase> = Vec::new();
    for (i, p) in v.phases.iter().enumerate() {
        let at = pl.index(i);
        if phases.iter().any(|x| x.key == p.id) {
            ctx.error(
                &at.field("id"),
                messages::duplicate_key("Phase", &p.id, "phasen"),
            );
            continue;
        }
        if p.months == 0 {
            ctx.error(&at.field("monate"), messages::not_positive(0.0));
        }
        let capital = non_negative(ctx, p.capital_usd, &at.field("kapital_usd"));
        phases.push(VenturePhase {
            key: p.id.clone(),
            months: p.months,
            capital: Money::from_usd(capital).unwrap_or(Money::ZERO),
            chance: in_range(ctx, p.chance, 0.0, 1.0, &at.field("chance")),
            valuation: positive(ctx, p.valuation, &at.field("bewertung")),
        });
    }
    if v.deadline_months == 0 {
        ctx.error(&l.field("frist_monate"), messages::not_positive(0.0));
    }
    in_range(
        ctx,
        f64::from(v.lead_years_max),
        0.0,
        100.0,
        &l.field("vorlauf_jahre_max"),
    );
    in_range(
        ctx,
        f64::from(v.keep_years),
        0.0,
        100.0,
        &l.field("aufbewahren_jahre"),
    );
    let medium = in_range(
        ctx,
        v.level_medium_from,
        0.0,
        1.0,
        &l.field("stufe_mittel_ab"),
    );
    let high = in_range(ctx, v.level_high_from, 0.0, 1.0, &l.field("stufe_hoch_ab"));
    if high < medium {
        ctx.error(
            &l.field("stufe_hoch_ab"),
            messages::range_inverted("stufe_mittel_ab", "stufe_hoch_ab"),
        );
    }
    let factor_min = positive(ctx, v.capital_factor_min, &l.field("kapital_faktor_min"));
    let factor_max = positive(ctx, v.capital_factor_max, &l.field("kapital_faktor_max"));
    if factor_max < factor_min {
        ctx.error(
            &l.field("kapital_faktor_max"),
            messages::range_inverted("kapital_faktor_min", "kapital_faktor_max"),
        );
    }
    VentureModel {
        labels,
        per_year: in_range(ctx, v.per_year, 0.0, 1000.0, &l.field("je_jahr")),
        frequencies,
        default_frequency,
        new_share: in_range(ctx, v.new_share, 0.0, 1.0, &l.field("anteil_neu")),
        lead_years_max: v.lead_years_max,
        phases,
        reference_gdp_usd: positive(ctx, v.reference_gdp_usd, &l.field("bezug_bip_je_kopf_usd")),
        capital_factor: (factor_min, factor_max.max(factor_min)),
        lead_capital: non_negative(ctx, v.lead_capital, &l.field("vorlauf_kapital")),
        lead_chance: in_range(ctx, v.lead_chance, 0.0, 1.0, &l.field("vorlauf_chance")),
        chance_min: in_range(ctx, v.chance_min, 0.0, 1.0, &l.field("chance_min")),
        investor_chance: in_range(
            ctx,
            v.investor_chance,
            0.0,
            1.0,
            &l.field("investoren_chance_monat"),
        ),
        deadline_months: v.deadline_months,
        blur: in_range(ctx, v.blur, 0.0, 1.0, &l.field("unschaerfe")),
        chance_levels: (medium, high.max(medium)),
        keep_years: v.keep_years,
        inventors: inventors(ctx, catalog, raw),
        provenance: provenance(v.approximation, v.source.as_ref()),
    }
}

/// The historical inventors: a known technology, at most one each, and a country.
fn inventors(ctx: &mut Ctx, catalog: &Catalog, raw: &RawData) -> Vec<Inventor> {
    let mut out: Vec<Inventor> = Vec::new();
    for entry in &raw.inventors {
        let i = &entry.value;
        let l = &entry.loc;
        let Some(technology) = catalog.technologies.id(&i.technology) else {
            let suggested = crate::suggest::closest(
                &i.technology,
                catalog.technologies.keys().iter().map(String::as_str),
            );
            ctx.error(
                &l.field("technologie"),
                messages::unknown_reference("Technologie", &i.technology, suggested),
            );
            continue;
        };
        if out.iter().any(|x| x.technology == technology) {
            ctx.error(
                &l.field("technologie"),
                messages::duplicate_key("Erfinder der Technologie", &i.technology, "erfinder"),
            );
            continue;
        }
        let Some(country) = catalog.countries.id(&i.country) else {
            let suggested = crate::suggest::closest(
                &i.country,
                catalog.countries.keys().iter().map(String::as_str),
            );
            ctx.error(
                &l.field("land"),
                messages::unknown_reference("Land", &i.country, suggested),
            );
            continue;
        };
        if i.name.trim().is_empty() {
            ctx.error(&l.field("name"), messages::field_empty("name"));
            continue;
        }
        out.push(Inventor {
            technology,
            name: i.name.trim().to_owned(),
            country,
            provenance: provenance(i.approximation, i.source.as_ref()),
        });
    }
    out
}

/// Every label and phase needs its text; texts of unknown ones are unused.
pub(super) fn check_texts(
    ctx: &mut Ctx,
    model: &VentureModel,
    raw: &RawData,
    texts: &TextIndex,
    report_unused: bool,
) {
    let Some(entry) = raw.ventures.first() else {
        return;
    };
    let wanted: [(&str, &str, Vec<&str>); 3] = [
        (
            LABEL_TEXT,
            "bezeichnungen",
            model.labels.iter().map(|(k, _)| k.as_str()).collect(),
        ),
        (
            FREQUENCY_TEXT,
            "haeufigkeiten",
            model.frequencies.iter().map(|(k, _)| k.as_str()).collect(),
        ),
        (
            PHASE_TEXT,
            "phasen",
            model.phases.iter().map(|p| p.key.as_str()).collect(),
        ),
    ];
    for (prefix, field, keys) in &wanted {
        let at: Loc = entry.loc.field(field);
        for (i, key) in keys.iter().enumerate() {
            let text = format!("{prefix}.{key}");
            if texts.texts.get(&text).is_none() {
                ctx.error(
                    &at.index(i).field("id"),
                    messages::text_missing(&text, crate::LANGUAGE),
                );
            }
        }
    }
    if !report_unused {
        return;
    }
    for (text_key, loc) in &texts.locations {
        for (prefix, _, keys) in &wanted {
            if let Some(rest) = text_key
                .strip_prefix(prefix)
                .and_then(|r| r.strip_prefix('.'))
                && !keys.contains(&rest)
            {
                ctx.warning(loc, messages::text_unused(text_key));
            }
        }
    }
}
