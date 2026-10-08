//! Controlling (W7, Lastenheft §14.2; formulas in docs/FORMELN.md, W7): contribution
//! margins by group, company, continent, country, site and product, with the cost types
//! of each level, the period before and the results of the last months.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{iso, usd};
use crate::command::site_type_key;
use crate::game::Game;
use crate::ids::{ContinentId, CountryId, ProductId};
use crate::ledger::{CostCenter, CostType, PeriodResult};
use crate::money::Money;
use crate::state::{CompanyId, SiteId};

/// Months in the series of a node.
const SERIES_MONTHS: usize = 12;

/// Revenue, variable and fixed costs, and the rest of the result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Block {
    Revenue,
    Variable,
    Fixed,
    Other,
}

fn block(t: CostType) -> Block {
    match t {
        CostType::Revenue => Block::Revenue,
        CostType::Material
        | CostType::InventoryChange
        | CostType::Energy
        | CostType::Transport
        | CostType::Customs
        | CostType::Licenses => Block::Variable,
        CostType::Personnel
        | CostType::Maintenance
        | CostType::Depreciation
        | CostType::Rent
        | CostType::Overhead => Block::Fixed,
        CostType::Taxes
        | CostType::Marketing
        | CostType::Research
        | CostType::Interest
        | CostType::Investments
        | CostType::Other => Block::Other,
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CostLineView {
    /// Text key of the cost type.
    pub key: String,
    pub usd: f64,
}

/// One level of the drill-down.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ControllingNode {
    /// Unique within the view.
    pub key: String,
    /// `konzern`, `firma`, `kontinent`, `land`, `standort`, `produkt`, `allgemein`,
    /// `zentrale`.
    pub level: String,
    /// Name of a company, else a text key.
    pub name: String,
    /// Country of a site.
    pub country: Option<String>,
    pub revenue_usd: f64,
    pub variable_usd: f64,
    pub margin1_usd: f64,
    pub fixed_usd: f64,
    pub margin2_usd: f64,
    pub other_usd: f64,
    pub result_usd: f64,
    /// Result of the period before, where the ledger keeps it.
    pub previous_result_usd: Option<f64>,
    /// Results of the last closed months, oldest first.
    pub series_usd: Vec<f64>,
    pub costs: Vec<CostLineView>,
    pub children: Vec<ControllingNode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ControllingView {
    /// `monat`, `jahr` or `vorjahr`.
    pub period: String,
    /// Periods with data.
    pub periods: Vec<String>,
    pub start: Option<String>,
    /// First days of the months of the series, oldest first (the series of a company end
    /// with the latest of them).
    pub months: Vec<String>,
    /// `None` before the first closed period.
    pub root: Option<ControllingNode>,
}

type Sums = BTreeMap<CostType, Money>;

fn add_into(into: &mut Sums, from: &Sums) {
    for (&t, &m) in from {
        *into.entry(t).or_default() += m;
    }
}

fn node(
    (key, level, name): (String, &str, String),
    sums: &Sums,
    previous: Option<Money>,
    series: Vec<Money>,
    children: Vec<ControllingNode>,
) -> ControllingNode {
    let part = |b: Block| -> Money {
        sums.iter()
            .filter(|(t, _)| block(**t) == b)
            .map(|(_, &m)| m)
            .sum()
    };
    let (revenue, variable, fixed, other) = (
        part(Block::Revenue),
        part(Block::Variable),
        part(Block::Fixed),
        part(Block::Other),
    );
    ControllingNode {
        key,
        level: level.to_owned(),
        name,
        country: None,
        revenue_usd: usd(revenue),
        variable_usd: usd(variable),
        margin1_usd: usd(revenue + variable),
        fixed_usd: usd(fixed),
        margin2_usd: usd(revenue + variable + fixed),
        other_usd: usd(other),
        result_usd: usd(revenue + variable + fixed + other),
        previous_result_usd: previous.map(usd),
        series_usd: series.into_iter().map(usd).collect(),
        costs: CostType::ALL
            .iter()
            .filter_map(|&t| {
                let m = sums.get(&t).copied()?;
                (m != Money::ZERO).then(|| CostLineView {
                    key: t.text_key().to_owned(),
                    usd: usd(m),
                })
            })
            .collect(),
        children,
    }
}

/// The periods of a company's ledger for a choice: (period, period before).
fn periods<'a>(
    ledger: &'a crate::ledger::Ledger,
    period: &str,
) -> Option<(&'a PeriodResult, Option<&'a PeriodResult>)> {
    match period {
        "monat" => {
            let n = ledger.months.len();
            let last = ledger.months.last()?;
            Some((last, n.checked_sub(2).and_then(|i| ledger.months.get(i))))
        }
        "vorjahr" => {
            let n = ledger.years.len();
            let last = ledger.years.last()?;
            Some((last, n.checked_sub(2).and_then(|i| ledger.years.get(i))))
        }
        _ => Some((&ledger.year, None)),
    }
}

/// The drill-down of one company.
fn company_node(game: &Game, id: CompanyId, period: &str) -> Option<ControllingNode> {
    let (state, catalog) = (game.state(), game.catalog());
    let company = &state.companies[id.index()];
    let ledger = &company.ledger;
    let (now, before) = periods(ledger, period)?;
    let site_place = |site: SiteId| -> (CountryId, ContinentId) {
        let country = state.sites[site.index()].country;
        (country, catalog.countries.get(country).continent)
    };
    let recent: Vec<&PeriodResult> = ledger
        .months
        .iter()
        .rev()
        .take(SERIES_MONTHS)
        .rev()
        .collect();
    let series = |pick: &dyn Fn(&PeriodResult) -> Money| -> Vec<Money> {
        recent.iter().map(|p| pick(p)).collect()
    };
    let total = |p: &PeriodResult| -> Money { p.by_type.values().copied().sum() };
    let of_sites = |p: &PeriodResult, keep: &dyn Fn(SiteId) -> bool| -> Money {
        p.by_site
            .iter()
            .filter(|(s, _)| keep(**s))
            .map(|(_, &m)| m)
            .sum()
    };

    // Sums per site and product, per site without product, and for the company.
    let mut by_site: BTreeMap<SiteId, (Sums, BTreeMap<ProductId, Sums>)> = BTreeMap::new();
    let mut central = Sums::new();
    for (center, sums) in &now.by_center {
        match *center {
            CostCenter {
                site: Some(site),
                product,
            } => {
                let entry = by_site.entry(site).or_default();
                match product {
                    Some(p) => add_into(entry.1.entry(p).or_default(), sums),
                    None => add_into(&mut entry.0, sums),
                }
            }
            CostCenter { site: None, .. } => add_into(&mut central, sums),
        }
    }
    // Sites into countries and continents.
    let mut tree: BTreeMap<ContinentId, BTreeMap<CountryId, Vec<SiteId>>> = BTreeMap::new();
    for &site in by_site.keys() {
        let (country, continent) = site_place(site);
        tree.entry(continent)
            .or_default()
            .entry(country)
            .or_default()
            .push(site);
    }
    let mut company_sums = Sums::new();
    let mut continents = Vec::new();
    for (continent, countries) in &tree {
        let mut continent_sums = Sums::new();
        let mut country_nodes = Vec::new();
        for (&country, sites) in countries {
            let mut country_sums = Sums::new();
            let mut site_nodes = Vec::new();
            for &site in sites {
                let (general, products) = &by_site[&site];
                let mut site_sums = general.clone();
                let mut children: Vec<ControllingNode> = products
                    .iter()
                    .map(|(&p, sums)| {
                        add_into(&mut site_sums, sums);
                        node(
                            (
                                format!("produkt:{}:{}", site.0, catalog.products.key(p)),
                                "produkt",
                                format!("produkt.{}", catalog.products.key(p)),
                            ),
                            sums,
                            None,
                            Vec::new(),
                            Vec::new(),
                        )
                    })
                    .collect();
                if !general.is_empty() && !products.is_empty() {
                    children.push(node(
                        (
                            format!("allgemein:{}", site.0),
                            "allgemein",
                            "controlling.allgemein".to_owned(),
                        ),
                        general,
                        None,
                        Vec::new(),
                        Vec::new(),
                    ));
                }
                children.sort_by(|a, b| b.result_usd.total_cmp(&a.result_usd));
                let mut n = node(
                    (
                        format!("standort:{}", site.0),
                        "standort",
                        site_type_key(state.sites[site.index()].kind),
                    ),
                    &site_sums,
                    before.map(|p| p.by_site.get(&site).copied().unwrap_or_default()),
                    series(&|p| p.by_site.get(&site).copied().unwrap_or_default()),
                    children,
                );
                n.country = Some(catalog.countries.key(country).to_owned());
                site_nodes.push(n);
                add_into(&mut country_sums, &site_sums);
            }
            site_nodes.sort_by(|a, b| b.result_usd.total_cmp(&a.result_usd));
            let in_country = |s: SiteId| site_place(s).0 == country;
            country_nodes.push(node(
                (
                    format!("land:{}:{}", id.0, catalog.countries.key(country)),
                    "land",
                    format!("land.{}", catalog.countries.key(country)),
                ),
                &country_sums,
                before.map(|p| of_sites(p, &in_country)),
                series(&|p| of_sites(p, &in_country)),
                site_nodes,
            ));
            add_into(&mut continent_sums, &country_sums);
        }
        country_nodes.sort_by(|a, b| b.result_usd.total_cmp(&a.result_usd));
        let in_continent = |s: SiteId| site_place(s).1 == *continent;
        let key = catalog.continents.key(*continent);
        continents.push(node(
            (
                format!("kontinent:{}:{key}", id.0),
                "kontinent",
                format!("kontinent.{key}"),
            ),
            &continent_sums,
            before.map(|p| of_sites(p, &in_continent)),
            series(&|p| of_sites(p, &in_continent)),
            country_nodes,
        ));
        add_into(&mut company_sums, &continent_sums);
    }
    continents.sort_by(|a, b| b.result_usd.total_cmp(&a.result_usd));
    if !central.is_empty() {
        let without_site = |p: &PeriodResult| total(p) - of_sites(p, &|_| true);
        continents.push(node(
            (
                format!("zentrale:{}", id.0),
                "zentrale",
                "controlling.zentrale".to_owned(),
            ),
            &central,
            before.map(without_site),
            series(&without_site),
            Vec::new(),
        ));
        add_into(&mut company_sums, &central);
    }
    Some(node(
        (format!("firma:{}", id.0), "firma", company.name.clone()),
        &company_sums,
        before.map(total),
        series(&total),
        continents,
    ))
}

/// The player's controlling for `period` (`monat`, `jahr` or `vorjahr`): the group with
/// its companies when it has subsidiaries, else the player's company.
pub fn controlling(game: &Game, period: &str) -> ControllingView {
    let state = game.state();
    let period = match period {
        "monat" | "vorjahr" => period,
        _ => "jahr",
    };
    let ledger = &state.companies[state.player.index()].ledger;
    let mut periods = Vec::new();
    if !ledger.months.is_empty() {
        periods.push("monat".to_owned());
    }
    periods.push("jahr".to_owned());
    if !ledger.years.is_empty() {
        periods.push("vorjahr".to_owned());
    }
    let start = periods_start(ledger, period);
    let members = crate::group::members(state, state.player);
    let companies: Vec<ControllingNode> = members
        .iter()
        .filter_map(|&c| company_node(game, c, period))
        .collect();
    let root = if members.len() > 1 && !companies.is_empty() {
        // The group: the sum of its companies.
        let sum = |f: fn(&ControllingNode) -> f64| companies.iter().map(f).sum::<f64>();
        let previous = companies
            .iter()
            .map(|c| c.previous_result_usd)
            .sum::<Option<f64>>();
        let months = companies
            .iter()
            .map(|c| c.series_usd.len())
            .max()
            .unwrap_or(0);
        let series = (0..months)
            .map(|i| {
                companies
                    .iter()
                    .map(|c| {
                        // Aligned at the latest month; younger companies have fewer.
                        let offset = months - c.series_usd.len();
                        i.checked_sub(offset)
                            .and_then(|j| c.series_usd.get(j))
                            .copied()
                            .unwrap_or(0.0)
                    })
                    .sum()
            })
            .collect();
        let mut costs: BTreeMap<String, f64> = BTreeMap::new();
        for c in &companies {
            for l in &c.costs {
                *costs.entry(l.key.clone()).or_default() += l.usd;
            }
        }
        let costs = CostType::ALL
            .iter()
            .filter_map(|t| {
                let key = t.text_key();
                costs.get(key).map(|&usd| CostLineView {
                    key: key.to_owned(),
                    usd,
                })
            })
            .collect();
        Some(ControllingNode {
            key: "konzern".to_owned(),
            level: "konzern".to_owned(),
            name: "controlling.konzern".to_owned(),
            country: None,
            revenue_usd: sum(|c| c.revenue_usd),
            variable_usd: sum(|c| c.variable_usd),
            margin1_usd: sum(|c| c.margin1_usd),
            fixed_usd: sum(|c| c.fixed_usd),
            margin2_usd: sum(|c| c.margin2_usd),
            other_usd: sum(|c| c.other_usd),
            result_usd: sum(|c| c.result_usd),
            previous_result_usd: previous,
            series_usd: series,
            costs,
            children: companies,
        })
    } else {
        companies.into_iter().next()
    };
    let months = ledger
        .months
        .iter()
        .rev()
        .take(SERIES_MONTHS)
        .rev()
        .filter_map(|m| m.start.map(iso))
        .collect();
    ControllingView {
        period: period.to_owned(),
        periods,
        start,
        months,
        root,
    }
}

fn periods_start(ledger: &crate::ledger::Ledger, period: &str) -> Option<String> {
    periods(ledger, period).and_then(|(p, _)| p.start.map(iso))
}
