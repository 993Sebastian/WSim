//! The production model.

use wsim_core::catalog::{ProductionModel, SiteType};

use super::{in_range, money, positive};
use crate::messages;
use crate::read::{Ctx, RawData};
use crate::suggest;

const SITE_TYPES: &[(&str, SiteType)] = &[
    ("foerderstaette", SiteType::Extraction),
    ("werk", SiteType::Factory),
    ("kraftwerk", SiteType::PowerPlant),
    ("lager", SiteType::Warehouse),
    ("niederlassung", SiteType::SalesOffice),
    ("forschungszentrum", SiteType::ResearchCenter),
];

pub(super) fn production_model(ctx: &mut Ctx, raw: &RawData) -> ProductionModel {
    let entry = match raw.production_model.as_slice() {
        [] => {
            ctx.general_error(messages::section_missing(
                "produktionsmodell",
                "parameter/produktionsmodell.yaml",
            ));
            return ProductionModel::default();
        }
        [first, rest @ ..] => {
            let first_loc = ctx.describe(&first.loc);
            for other in rest {
                ctx.error(
                    &other.loc,
                    messages::section_duplicate("produktionsmodell", &first_loc),
                );
            }
            first
        }
    };
    let m = &entry.value;
    let l = &entry.loc;
    let costs = l.field("standortkosten_usd");
    let names: Vec<&str> = SITE_TYPES.iter().map(|(n, _)| *n).collect();
    let mut site_cost = Vec::new();
    for (key, &usd) in &m.site_cost {
        match SITE_TYPES.iter().find(|(n, _)| n == key) {
            Some(&(_, kind)) => site_cost.push((kind, money(ctx, usd, &costs.field(key)))),
            None => ctx.error(
                &costs.key(key),
                messages::unknown_value(key, &names, suggest::closest(key, names.iter().copied())),
            ),
        }
    }
    for (name, _) in SITE_TYPES {
        if !m.site_cost.contains_key(*name) {
            ctx.error(&costs, messages::entry_missing("Standorttyp", name));
        }
    }
    let automation = l.field("automatisierung");
    let quality = l.field("qualitaet");
    ProductionModel {
        site_cost,
        building_lifetime_years: positive(
            ctx,
            m.building_lifetime_years,
            &l.field("gebaeude_lebensdauer_jahre"),
        ),
        development_lifetime_years: positive(
            ctx,
            m.development_lifetime_years,
            &l.field("erschliessung_lebensdauer_jahre"),
        ),
        automation_labor_saving: in_range(
            ctx,
            m.automation.labor_saving,
            0.0,
            1.0,
            &automation.field("arbeitsersparnis"),
        ),
        automation_cost_share: in_range(
            ctx,
            m.automation.cost_share,
            0.0,
            10.0,
            &automation.field("kostenanteil"),
        ),
        quality_inputs: in_range(
            ctx,
            m.quality.inputs,
            0.0,
            1.0,
            &quality.field("vorprodukte"),
        ),
        quality_automation: in_range(
            ctx,
            m.quality.automation,
            0.0,
            100.0,
            &quality.field("automatisierung"),
        ),
        quality_condition: in_range(
            ctx,
            m.quality.condition,
            0.0,
            100.0,
            &quality.field("zustand"),
        ),
        condition_min: in_range(ctx, m.condition_min, 0.0, 1.0, &l.field("zustand_minimum")),
    }
}

pub(super) fn finance_model(ctx: &mut Ctx, raw: &RawData) -> wsim_core::catalog::FinanceModel {
    use wsim_core::time_series::TimeSeries;
    let entry = match raw.finance_model.as_slice() {
        [] => {
            ctx.general_error(messages::section_missing(
                "finanzmodell",
                "parameter/finanzmodell.yaml",
            ));
            return wsim_core::catalog::FinanceModel::default();
        }
        [first, rest @ ..] => {
            let first_loc = ctx.describe(&first.loc);
            for other in rest {
                ctx.error(
                    &other.loc,
                    messages::section_duplicate("finanzmodell", &first_loc),
                );
            }
            first
        }
    };
    let m = &entry.value;
    let l = &entry.loc;
    let rates = l.field("realzins");
    if m.real_rate.is_empty() {
        ctx.error(&rates, messages::time_series_empty());
    }
    for (&year, &rate) in &m.real_rate {
        let point = rates.field(&year.to_string());
        super::year(ctx, year, super::GAME_YEARS, &point);
        in_range(ctx, rate, -0.2, 0.5, &point);
    }
    let real_rate = TimeSeries::new(m.real_rate.iter().map(|(&y, &r)| (y, r)).collect())
        .unwrap_or_else(|_| TimeSeries::new(vec![(1900, 0.0)]).expect("valid"));
    let premium = l.field("risikoaufschlag");
    let overdraft = l.field("dispo");
    if m.max_term_years == 0 {
        ctx.error(&l.field("laufzeit_max_jahre"), messages::not_positive(0.0));
    }
    wsim_core::catalog::FinanceModel {
        real_rate,
        premium_min: in_range(ctx, m.premium.min, 0.0, 1.0, &premium.field("minimum")),
        premium_per_debt_ratio: in_range(
            ctx,
            m.premium.per_debt_ratio,
            0.0,
            1.0,
            &premium.field("je_verschuldung"),
        ),
        loan_to_value: in_range(ctx, m.loan_to_value, 0.0, 1.0, &l.field("beleihung")),
        overdraft_share: in_range(ctx, m.overdraft.share, 0.0, 1.0, &overdraft.field("anteil")),
        overdraft_premium: in_range(
            ctx,
            m.overdraft.premium,
            0.0,
            1.0,
            &overdraft.field("aufschlag"),
        ),
        max_term_years: m.max_term_years,
    }
}
