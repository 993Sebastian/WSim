//! AI companies: model, name parts and historical companies (docs/FORMELN.md, M10).

use std::collections::BTreeMap;

use wsim_core::EARLIEST_START_YEAR;
use wsim_core::catalog::{
    AiBehavior, AiModel, AiStart, Catalog, Difficulty, NameGroup, RealCompany, RealSite, SiteType,
    Span,
};
use wsim_core::ids::{BranchId, CountryId, DepositId, FacilityId, Id, RecipeId};

use super::production::single;
use super::{HISTORY_YEARS, Keys, in_range, non_negative, positive, provenance, resolve, year};
use crate::messages;
use crate::raw::{RawRealCompany, RawSpan};
use crate::read::{Ctx, Entry, Loc, RawData};

/// Placeholders allowed in name patterns.
const PLACEHOLDERS: &[&str] = &["familienname", "ort", "rechtsform", "branche"];

fn span(ctx: &mut Ctx, s: &RawSpan, loc: &Loc) -> Span {
    Span {
        at_0: positive(ctx, s.at_0, &loc.field("bei_0")),
        at_1: positive(ctx, s.at_1, &loc.field("bei_1")),
    }
}

fn share(ctx: &mut Ctx, value: f64, loc: &Loc) -> f64 {
    in_range(ctx, value, 0.0, 1.0, loc)
}

/// The AI model and the keys of the difficulties (for the text check).
pub(super) fn ai_model(ctx: &mut Ctx, raw: &RawData) -> (AiModel, Keys) {
    let mut difficulty_keys = Keys {
        kind: "Schwierigkeit",
        text_prefix: Some("schwierigkeit"),
        index: BTreeMap::new(),
        locations: Vec::new(),
        broken: Default::default(),
    };
    let Some(entry) = single(ctx, &raw.ai_model, "kimodell", "parameter/kimodell.yaml") else {
        return (AiModel::default(), difficulty_keys);
    };
    let m = &entry.value;
    let l = &entry.loc;
    let mut difficulties = Vec::new();
    let list_loc = l.field("schwierigkeiten");
    if m.difficulties.is_empty() {
        ctx.error(&list_loc, messages::list_empty());
    }
    for (i, d) in m.difficulties.iter().enumerate() {
        let loc = list_loc.index(i);
        if difficulty_keys.index.contains_key(&d.id) {
            let first = ctx.describe(&difficulty_keys.locations[difficulty_keys.index[&d.id]]);
            ctx.error(
                &loc.field("id"),
                messages::duplicate_key("Schwierigkeit", &d.id, &first),
            );
            continue;
        }
        difficulty_keys
            .index
            .insert(d.id.clone(), difficulties.len());
        difficulty_keys.locations.push(loc.clone());
        difficulties.push(Difficulty {
            key: d.id.clone(),
            competence: share(ctx, d.competence, &loc.field("kompetenz")),
            aggressiveness: share(ctx, d.aggressiveness, &loc.field("aggressivitaet")),
        });
    }
    let default_difficulty = match difficulty_keys.index.get(&m.default_difficulty) {
        Some(&i) => i,
        None => {
            ctx.error(
                &l.field("schwierigkeit_standard"),
                messages::default_difficulty_unknown(&m.default_difficulty),
            );
            0
        }
    };
    if m.default_companies > m.max_companies {
        ctx.error(
            &l.field("firmen_standard"),
            messages::range_inverted("firmen_standard", "firmen_max"),
        );
    }
    let scale_loc = l.field("massstab");
    let scale_min = in_range(ctx, m.scale.min, 1e-4, 1.0, &scale_loc.field("minimum"));
    let scale_max = in_range(ctx, m.scale.max, 1e-4, 1.0, &scale_loc.field("maximum"));
    if scale_min > scale_max {
        ctx.error(&scale_loc, messages::range_inverted("minimum", "maximum"));
    }
    let s = &m.start;
    let sl = l.field("start");
    let w = &s.development_weight;
    let wl = sl.field("gewicht_entwicklung");
    let start = AiStart {
        utilization: in_range(ctx, s.utilization, 0.05, 1.0, &sl.field("auslastung")),
        min_plant_share: share(ctx, s.min_plant_share, &sl.field("anlage_mindestanteil")),
        input_stock_days: non_negative(ctx, s.input_stock_days, &sl.field("lager_eingang_tage")),
        output_stock_days: non_negative(ctx, s.output_stock_days, &sl.field("lager_ausgang_tage")),
        cash_months: non_negative(ctx, s.cash_months, &sl.field("kasse_monate")),
        development_weight: [
            non_negative(ctx, w.raw_material, &wl.field("rohstoff")),
            non_negative(ctx, w.semi_finished, &wl.field("halbzeug")),
            non_negative(ctx, w.component, &wl.field("bauteil")),
            non_negative(ctx, w.end_product, &wl.field("endprodukt")),
            non_negative(ctx, w.energy, &wl.field("energie")),
        ],
        reference_wage_usd: positive(ctx, s.reference_wage_usd, &sl.field("referenzlohn_usd")),
    };
    let b = &m.behavior;
    let bl = l.field("verhalten");
    if b.stock_low_days >= b.stock_high_days {
        ctx.error(
            &bl.field("lager_niedrig_tage"),
            messages::range_inverted("lager_niedrig_tage", "lager_hoch_tage"),
        );
    }
    if b.cash_min_months >= b.cash_max_months {
        ctx.error(
            &bl.field("kasse_min_monate"),
            messages::range_inverted("kasse_min_monate", "kasse_max_monate"),
        );
    }
    let behavior = AiBehavior {
        operations_days: span(ctx, &b.operations_days, &bl.field("betrieb_alle_tage")),
        stock_high_days: positive(ctx, b.stock_high_days, &bl.field("lager_hoch_tage")),
        stock_low_days: non_negative(ctx, b.stock_low_days, &bl.field("lager_niedrig_tage")),
        utilization_step: in_range(
            ctx,
            b.utilization_step,
            0.01,
            1.0,
            &bl.field("auslastung_schritt"),
        ),
        utilization_min: share(ctx, b.utilization_min, &bl.field("auslastung_min")),
        floor_factor: span(ctx, &b.floor_factor, &bl.field("preisuntergrenze")),
        purchase_markup: non_negative(ctx, b.purchase_markup, &bl.field("einkauf_aufschlag")),
        expand_utilization: span(ctx, &b.expand_utilization, &bl.field("ausbau_auslastung")),
        expand_margin: span(ctx, &b.expand_margin, &bl.field("ausbau_marge")),
        invest_share_max: share(
            ctx,
            b.invest_share_max,
            &bl.field("ausbau_anteil_kasse_max"),
        ),
        research_lookahead_years: Span {
            at_0: non_negative(
                ctx,
                b.research_lookahead_years.at_0,
                &bl.field("forschung_vorgriff_jahre").field("bei_0"),
            ),
            at_1: non_negative(
                ctx,
                b.research_lookahead_years.at_1,
                &bl.field("forschung_vorgriff_jahre").field("bei_1"),
            ),
        },
        research_min_revenue_usd: non_negative(
            ctx,
            b.research_min_revenue_usd,
            &bl.field("forschung_mindestumsatz_usd"),
        ),
        research_competence_min: share(
            ctx,
            b.research_competence_min,
            &bl.field("forschung_mindestkompetenz"),
        ),
        cash_min_months: non_negative(ctx, b.cash_min_months, &bl.field("kasse_min_monate")),
        cash_max_months: positive(ctx, b.cash_max_months, &bl.field("kasse_max_monate")),
        loan_years: b.loan_years.max(1),
        foundings_per_month: b.foundings_per_month,
        founding_capital_factor: positive(
            ctx,
            b.founding_capital_factor,
            &bl.field("gruendung_kapitalfaktor"),
        ),
    };
    let model = AiModel {
        default_companies: m.default_companies,
        max_companies: m.max_companies,
        companies_for_real_size: positive(
            ctx,
            m.companies_for_real_size,
            &l.field("firmen_bei_realer_groesse"),
        ),
        scale_min,
        scale_max,
        min_labor_pool: non_negative(ctx, m.min_labor_pool, &l.field("arbeitskraefte_min")),
        plants_per_concession: positive(
            ctx,
            m.plants_per_concession,
            &l.field("anlagen_je_konzession"),
        ),
        max_concessions: m.max_concessions.max(1),
        difficulties,
        default_difficulty,
        trait_spread: in_range(ctx, m.trait_spread, 0.0, 0.5, &l.field("streuung")),
        start,
        behavior,
    };
    (model, difficulty_keys)
}

/// Placeholders of a pattern such as `{familienname} & {familienname}`.
fn placeholders(pattern: &str) -> impl Iterator<Item = &str> {
    pattern
        .split('{')
        .skip(1)
        .filter_map(|part| part.split_once('}').map(|(name, _)| name))
}

pub(super) fn name_groups(
    ctx: &mut Ctx,
    catalog: &Catalog,
    raw: &RawData,
    (countries, branches): (&Keys, &Keys),
) -> Vec<NameGroup> {
    let mut groups = Vec::new();
    let mut country_owner: BTreeMap<CountryId, String> = BTreeMap::new();
    for e in &raw.name_groups {
        let v = &e.value;
        let l = &e.loc;
        for (field, list) in [
            ("familiennamen", &v.surnames),
            ("orte", &v.places),
            ("rechtsformen", &v.legal_forms),
            ("muster", &v.patterns),
        ] {
            if list.is_empty() {
                ctx.error(&l.field(field), messages::list_empty());
            }
        }
        for (i, pattern) in v.patterns.iter().enumerate() {
            for p in placeholders(pattern) {
                if !PLACEHOLDERS.contains(&p) {
                    ctx.error(
                        &l.field("muster").index(i),
                        messages::name_placeholder_unknown(p),
                    );
                }
            }
        }
        let mut group_countries = Vec::new();
        for (i, key) in v.countries.iter().enumerate() {
            let loc = l.field("laender").index(i);
            let country: CountryId = resolve(ctx, countries, key, &loc);
            if !countries.index.contains_key(key) {
                continue;
            }
            if let Some(first) = country_owner.get(&country) {
                ctx.error(&loc, messages::name_country_twice(key, first));
                continue;
            }
            country_owner.insert(country, v.id.clone());
            group_countries.push(country);
        }
        let mut branch_words = vec![None; catalog.branches.len()];
        for (key, word) in &v.branch_words {
            let branch: BranchId = resolve(ctx, branches, key, &l.field("branchen").field(key));
            if branches.index.contains_key(key) {
                branch_words[branch.index()] = Some(word.clone());
            }
        }
        groups.push(NameGroup {
            key: v.id.clone(),
            countries: group_countries,
            is_default: v.is_default,
            surnames: v.surnames.clone(),
            places: v.places.clone(),
            legal_forms: v.legal_forms.clone(),
            patterns: v.patterns.clone(),
            branch_words,
        });
    }
    let defaults = groups.iter().filter(|g| g.is_default).count();
    if defaults != 1 && !raw.name_groups.is_empty() {
        ctx.general_error(messages::name_default_count(defaults));
    }
    groups
}

pub(super) fn real_companies(
    ctx: &mut Ctx,
    catalog: &Catalog,
    entries: &[&Entry<RawRealCompany>],
    (countries, deposits, facilities, recipes): (&Keys, &Keys, &Keys, &Keys),
) -> Vec<RealCompany> {
    let too_new = |t: Option<wsim_core::ids::TechnologyId>| {
        t.is_some_and(|t| catalog.technologies.get(t).invention_year > EARLIEST_START_YEAR)
    };
    let mut companies = Vec::new();
    for e in entries {
        let v = &e.value;
        let l = &e.loc;
        let founded = year(ctx, v.founded, HISTORY_YEARS, &l.field("gegruendet"));
        if founded > EARLIEST_START_YEAR {
            ctx.error(
                &l.field("gegruendet"),
                messages::real_company_too_young(founded, EARLIEST_START_YEAR),
            );
        }
        let headquarters: CountryId = resolve(ctx, countries, &v.headquarters, &l.field("sitz"));
        let mut sites = Vec::new();
        for (si, s) in v.sites.iter().enumerate() {
            let sl = l.field("standorte").index(si);
            let country: CountryId = resolve(ctx, countries, &s.country, &sl.field("land"));
            let deposit: Option<DepositId> = s.deposit.as_ref().map(|d| {
                let id: DepositId = resolve(ctx, deposits, d, &sl.field("lagerstaette"));
                if deposits.index.contains_key(d)
                    && countries.index.contains_key(&s.country)
                    && catalog.deposits.get(id).country != country
                {
                    ctx.error(
                        &sl.field("lagerstaette"),
                        messages::real_deposit_other_country(d, &s.country),
                    );
                }
                id
            });
            if s.facilities.is_empty() {
                ctx.error(&sl.field("anlagen"), messages::list_empty());
            }
            let mut site_type: Option<SiteType> = None;
            let mut list = Vec::new();
            for (fi, f) in s.facilities.iter().enumerate() {
                let fl = sl.field("anlagen").index(fi);
                let count = positive(ctx, f.count, &fl.field("anzahl"));
                if !facilities.index.contains_key(&f.facility) {
                    let _: FacilityId = resolve(ctx, facilities, &f.facility, &fl.field("anlage"));
                    continue;
                }
                let facility: FacilityId =
                    resolve(ctx, facilities, &f.facility, &fl.field("anlage"));
                let fac = catalog.facilities.get(facility);
                match site_type {
                    Some(t) if t != fac.site_type => {
                        ctx.error(&fl.field("anlage"), messages::real_site_mixed_types());
                    }
                    _ => site_type = Some(fac.site_type),
                }
                if fac.site_type == SiteType::Extraction && deposit.is_none() {
                    ctx.error(
                        &fl.field("anlage"),
                        messages::real_site_needs_deposit(&f.facility),
                    );
                }
                if too_new(fac.technology) {
                    ctx.error(
                        &fl.field("anlage"),
                        messages::start_needs_new_technology(&f.facility),
                    );
                }
                let recipe: Option<RecipeId> = f.recipe.as_ref().and_then(|r| {
                    let id: RecipeId = resolve(ctx, recipes, r, &fl.field("rezept"));
                    if !recipes.index.contains_key(r) {
                        return None;
                    }
                    let rec = catalog.recipes.get(id);
                    if rec.facility != facility {
                        ctx.error(
                            &fl.field("rezept"),
                            messages::start_recipe_wrong_facility(r, &f.facility),
                        );
                    }
                    if too_new(rec.technology) {
                        ctx.error(&fl.field("rezept"), messages::start_needs_new_technology(r));
                    }
                    Some(id)
                });
                list.push((facility, count, recipe));
            }
            sites.push(RealSite {
                country,
                deposit,
                facilities: list,
            });
        }
        companies.push(RealCompany {
            key: v.id.clone(),
            name: v.name.clone(),
            headquarters,
            founded,
            sites,
            competence: v.competence.map(|c| share(ctx, c, &l.field("kompetenz"))),
            aggressiveness: v
                .aggressiveness
                .map(|a| share(ctx, a, &l.field("aggressivitaet"))),
            provenance: provenance(v.approximation, v.source.as_ref()),
        });
    }
    companies
}
