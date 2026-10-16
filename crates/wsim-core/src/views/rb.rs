//! Data basis of the round report (RB1, docs/RUNDENBERICHT_2.md, docs/FORMELN.md RB1): per
//! product made, sold, revenue, contribution margin and market shares; utilization of the
//! facilities with the main cause of a shortfall; the split of the result.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{Snapshot, main_ledger, usd};
use crate::command::site_type_key;
use crate::game::Game;
use crate::ids::{CountryId, ProductId};
use crate::ledger::{CostType, PeriodResult};
use crate::money::Money;
use crate::state::{GameState, Operation, Shortfall, SlotUse};

/// Where a booking belongs in the report's account of the result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Class {
    /// Contribution margin of its product.
    Contribution,
    Fixed,
    Other,
}

fn class(cost: CostType, has_product: bool) -> Class {
    use CostType::{
        Customs, Depreciation, Energy, Environment, InventoryChange, Investments, Licenses,
        Maintenance, Marketing, Material, Other, Overhead, Personnel, Rent, Research, Revenue,
        Storage, Taxes, Transport,
    };
    match cost {
        Revenue | Material | InventoryChange | Energy | Transport | Customs | Licenses
        | Environment | Storage | Personnel | Rent
            if has_product =>
        {
            Class::Contribution
        }
        Revenue => Class::Other,
        Material | InventoryChange | Energy | Transport | Customs | Licenses | Environment
        | Storage | Personnel | Rent | Overhead | Maintenance | Depreciation | Research
        | Marketing => Class::Fixed,
        Taxes | Investments | Other | CostType::Interest => Class::Other,
    }
}

/// The result of a period split into the contribution margins by product, the fixed costs
/// and the rest.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct Split {
    db: BTreeMap<ProductId, Money>,
    fixed: Money,
    other: Money,
}

fn split_of(period: &PeriodResult) -> Split {
    let mut s = Split::default();
    for (center, by_type) in &period.by_center {
        for (&t, &m) in by_type {
            match class(t, center.product.is_some()) {
                Class::Contribution => {
                    if let Some(p) = center.product {
                        *s.db.entry(p).or_insert(Money::ZERO) += m;
                    }
                }
                Class::Fixed => s.fixed += m,
                Class::Other => s.other += m,
            }
        }
    }
    s
}

fn add(into: &mut Split, from: &Split, sign: i64) {
    let f = |m: Money| if sign > 0 { m } else { -m };
    for (&p, &m) in &from.db {
        *into.db.entry(p).or_insert(Money::ZERO) += f(m);
    }
    into.fixed += f(from.fixed);
    into.other += f(from.other);
}

/// What the report needs from the books before the round.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct Base {
    split: Split,
    slot_use: BTreeMap<(usize, usize), SlotUse>,
    market_sold: BTreeMap<(ProductId, CountryId), f64>,
    person_wealth: Money,
}

pub(super) fn base(game: &Game) -> Base {
    let state = game.state();
    let empty = crate::ledger::Ledger::new(state.date, Money::ZERO);
    let ledger = main_ledger(state).unwrap_or(&empty);
    Base {
        split: split_of(&ledger.year),
        slot_use: own_slot_use(state),
        market_sold: market_sold(state),
        person_wealth: crate::private::wealth(game.catalog(), state).total(),
    }
}

fn own_slot_use(state: &GameState) -> BTreeMap<(usize, usize), SlotUse> {
    state
        .sites
        .iter()
        .enumerate()
        .filter(|(_, s)| state.is_main(s.owner))
        .flat_map(|(i, s)| {
            s.slots
                .iter()
                .enumerate()
                .map(move |(j, sl)| ((i, j), sl.usage))
        })
        .collect()
}

fn market_sold(state: &GameState) -> BTreeMap<(ProductId, CountryId), f64> {
    let mut out = BTreeMap::new();
    for (p, per_country) in state.markets.iter() {
        for (c, m) in per_country.iter() {
            if m.sold_total > 0.0 {
                out.insert((p, c), m.sold_total);
            }
        }
    }
    out
}

/// A share of the market of a product in a region of the round; `None` without sales there.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Shares {
    pub country: Option<f64>,
    pub continent: Option<f64>,
    pub world: Option<f64>,
}

/// One country a product was made in, with the shares there.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CountryShareView {
    pub country: String,
    pub made: f64,
    pub country_share: Option<f64>,
    pub continent_share: Option<f64>,
}

/// A product in the round: quantities, revenue, contribution margin and shares.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProductRow {
    pub product: String,
    pub unit: String,
    pub made: f64,
    pub sold: f64,
    pub revenue_usd: f64,
    /// Revenue less material, variable costs and the direct wages of the production.
    pub contribution_usd: f64,
    /// The country with the largest production, for the shares of the row.
    pub country: Option<String>,
    pub shares: Shares,
    /// Every country it was made in, the largest first (when more than one).
    pub countries: Vec<CountryShareView>,
    /// The values of the round before, set by the session.
    #[serde(default)]
    pub previous: Option<PreviousRow>,
}

/// What the trend arrows compare with.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PreviousRow {
    pub contribution_usd: f64,
    pub shares: Shares,
}

/// The result of the round as the sum of the contribution margins less the fixed costs and
/// the rest.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResultSplit {
    pub contribution_usd: f64,
    pub fixed_usd: f64,
    pub other_usd: f64,
    pub result_usd: f64,
}

/// The state of a facility in the round.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SlotRow {
    /// Text `anlage.<key>`.
    pub facility: String,
    pub product: Option<String>,
    pub count: u32,
    /// `laeuft`, `im_bau`, `stillgelegt`, `anlauf`.
    pub state: String,
    /// Days until it is built or started up.
    pub days_left: Option<i32>,
    /// Share of the possible runs; `None` while it did not work.
    pub utilization: Option<f64>,
    /// Main reason for a shortfall (`gedrosselt`, `material`, `personal`, …).
    pub loss: Option<String>,
}

/// A works with its facilities.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SiteRow {
    pub site: u32,
    pub name: Option<String>,
    pub kind_text: String,
    pub country: String,
    pub utilization: Option<f64>,
    pub loss: Option<String>,
    pub slots: Vec<SlotRow>,
}

/// The data of the new round report.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReportDetail {
    /// By revenue, the largest first.
    pub products: Vec<ProductRow>,
    pub result: ResultSplit,
    pub sites: Vec<SiteRow>,
    /// Change of the person's wealth in the round.
    pub person_change_usd: f64,
    /// Limits for display (visible rows, colors of the utilization).
    #[serde(default)]
    pub limits: crate::catalog::ReportModel,
}

/// Gives the rows of this round the values of the round before for the trend arrows.
pub fn attach_previous(now: &mut ReportDetail, before: &ReportDetail) {
    for row in &mut now.products {
        row.previous = before
            .products
            .iter()
            .find(|r| r.product == row.product)
            .map(|r| PreviousRow {
                contribution_usd: r.contribution_usd,
                shares: r.shares,
            });
    }
}

pub(super) fn detail(game: &Game, before: &Snapshot) -> ReportDetail {
    let (state, catalog) = (game.state(), game.catalog());
    let empty = crate::ledger::Ledger::new(state.date, Money::ZERO);
    let ledger = main_ledger(state).unwrap_or(&empty);
    // The split of the round: years closed since, the running year, less the start.
    let mut split = Split::default();
    for y in ledger.years.get(before.years_closed..).unwrap_or_default() {
        add(&mut split, &split_of(y), 1);
    }
    add(&mut split, &split_of(&ledger.year), 1);
    add(&mut split, &before.rb.split, -1);

    // Quantities of the main company's sites, by product and country.
    let mut made: BTreeMap<ProductId, BTreeMap<CountryId, f64>> = BTreeMap::new();
    let mut sold: BTreeMap<ProductId, BTreeMap<CountryId, f64>> = BTreeMap::new();
    for (i, s) in state.sites.iter().enumerate() {
        if !state.is_main(s.owner) {
            continue;
        }
        for (&p, t) in &s.totals {
            let b = before.site_totals.get(&(i, p)).copied().unwrap_or_default();
            let (m, v) = ((t.made - b.made).max(0.0), (t.sold - b.sold).max(0.0));
            if m > 1e-9 {
                *made.entry(p).or_default().entry(s.country).or_default() += m;
            }
            if v > 1e-9 {
                *sold.entry(p).or_default().entry(s.country).or_default() += v;
            }
        }
    }
    let market_now = market_sold(state);
    let market = |p: ProductId, c: CountryId| -> f64 {
        (market_now.get(&(p, c)).copied().unwrap_or(0.0)
            - before.rb.market_sold.get(&(p, c)).copied().unwrap_or(0.0))
        .max(0.0)
    };
    let share = |own: f64, total: f64| (total > 1e-9 && own > 1e-9).then(|| (own / total).min(1.0));
    let continent_of = |c: CountryId| catalog.countries.get(c).continent;
    let all_countries: Vec<CountryId> = catalog.countries.iter().map(|(id, _)| id).collect();

    let revenue_of = |p: ProductId| -> Money {
        // Revenue of the product, from the booked centers of the round.
        let mut m = Money::ZERO;
        for y in ledger.years.get(before.years_closed..).unwrap_or_default() {
            m += product_revenue(y, p);
        }
        m += product_revenue(&ledger.year, p);
        m -= before.products.get(&p).map_or(Money::ZERO, |x| x.0);
        m
    };

    let mut products: std::collections::BTreeSet<ProductId> = split.db.keys().copied().collect();
    products.extend(made.keys().copied());
    products.extend(sold.keys().copied());
    let mut rows: Vec<ProductRow> = Vec::new();
    for p in products {
        let mine_made = made.get(&p);
        let mine_sold = sold.get(&p);
        let revenue = revenue_of(p);
        let contribution = split.db.get(&p).copied().unwrap_or(Money::ZERO);
        let made_sum: f64 = mine_made.map_or(0.0, |m| m.values().sum());
        let sold_sum: f64 = mine_sold.map_or(0.0, |m| m.values().sum());
        if made_sum <= 1e-9
            && sold_sum <= 1e-9
            && revenue == Money::ZERO
            && contribution == Money::ZERO
        {
            continue;
        }
        let own_in = |c: CountryId| mine_sold.and_then(|m| m.get(&c)).copied().unwrap_or(0.0);
        let shares_for = |c: CountryId| -> (Option<f64>, Option<f64>) {
            let k = continent_of(c);
            let in_k: Vec<CountryId> = all_countries
                .iter()
                .copied()
                .filter(|&x| continent_of(x) == k)
                .collect();
            (
                share(own_in(c), market(p, c)),
                share(
                    in_k.iter().map(|&x| own_in(x)).sum(),
                    in_k.iter().map(|&x| market(p, x)).sum(),
                ),
            )
        };
        let mut countries: Vec<(CountryId, f64)> = mine_made
            .map(|m| m.iter().map(|(&c, &q)| (c, q)).collect())
            .unwrap_or_default();
        countries.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        let lead = countries.first().map(|&(c, _)| c);
        let (cs, ks) = lead.map_or((None, None), shares_for);
        let world = share(
            all_countries.iter().map(|&x| own_in(x)).sum(),
            all_countries.iter().map(|&x| market(p, x)).sum(),
        );
        rows.push(ProductRow {
            product: catalog.products.key(p).to_owned(),
            unit: catalog.units.key(catalog.products.get(p).unit).to_owned(),
            made: made_sum,
            sold: sold_sum,
            revenue_usd: usd(revenue),
            contribution_usd: usd(contribution),
            country: lead.map(|c| catalog.countries.key(c).to_owned()),
            shares: Shares {
                country: cs,
                continent: ks,
                world,
            },
            countries: if countries.len() > 1 {
                countries
                    .iter()
                    .map(|&(c, q)| {
                        let (a, b) = shares_for(c);
                        CountryShareView {
                            country: catalog.countries.key(c).to_owned(),
                            made: q,
                            country_share: a,
                            continent_share: b,
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            },
            previous: None,
        });
    }
    rows.sort_by(|a, b| {
        b.revenue_usd
            .total_cmp(&a.revenue_usd)
            .then(a.product.cmp(&b.product))
    });

    let contribution: Money = split.db.values().copied().sum();
    ReportDetail {
        products: rows,
        result: ResultSplit {
            contribution_usd: usd(contribution),
            fixed_usd: usd(split.fixed),
            other_usd: usd(split.other),
            result_usd: usd(contribution + split.fixed + split.other),
        },
        sites: site_rows(game, before),
        person_change_usd: usd(
            crate::private::wealth(catalog, state).total() - before.rb.person_wealth
        ),
        limits: catalog.report_model,
    }
}

fn product_revenue(period: &PeriodResult, p: ProductId) -> Money {
    period
        .by_center
        .iter()
        .filter(|(c, _)| c.product == Some(p))
        .map(|(_, t)| t.get(&CostType::Revenue).copied().unwrap_or(Money::ZERO))
        .sum()
}

fn site_rows(game: &Game, before: &Snapshot) -> Vec<SiteRow> {
    let (state, catalog) = (game.state(), game.catalog());
    let date = state.date;
    let mut out = Vec::new();
    for (i, s) in state.sites.iter().enumerate() {
        if !state.is_main(s.owner) || s.closed.is_some() || s.slots.is_empty() {
            continue;
        }
        let mut total = SlotUse::default();
        let mut slots = Vec::new();
        for (j, sl) in s.slots.iter().enumerate() {
            let used = sl
                .usage
                .since(&before.rb.slot_use.get(&(i, j)).copied().unwrap_or_default());
            let working = used.full > 1e-12;
            if working {
                total.full += used.full;
                total.runs += used.runs;
                for k in 0..Shortfall::COUNT {
                    total.lost[k] += used.lost[k];
                }
            }
            let (state_key, days_left) = if sl.ready > date {
                ("im_bau", Some(date.days_until(sl.ready)))
            } else {
                match sl.operation {
                    Operation::Mothballed { .. } => ("stillgelegt", None),
                    Operation::Restarting { until } if until > date => {
                        ("anlauf", Some(date.days_until(until)))
                    }
                    _ => ("laeuft", None),
                }
            };
            slots.push(SlotRow {
                facility: catalog.facilities.key(sl.facility).to_owned(),
                product: sl.recipe.map(|r| {
                    catalog
                        .products
                        .key(catalog.recipes.get(r).product)
                        .to_owned()
                }),
                count: sl.count,
                state: state_key.to_owned(),
                days_left,
                utilization: working.then(|| used.utilization()),
                loss: used
                    .main_loss()
                    .filter(|_| working)
                    .map(|x| x.key().to_owned()),
            });
        }
        out.push(SiteRow {
            site: u32::try_from(i).unwrap_or(u32::MAX),
            name: s.name.clone(),
            kind_text: site_type_key(s.kind),
            country: catalog.countries.key(s.country).to_owned(),
            utilization: (total.full > 1e-12).then(|| total.utilization()),
            loss: total.main_loss().map(|x| x.key().to_owned()),
            slots,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_use_of_a_facility_is_runs_over_possible_runs_with_its_main_loss() {
        let before = SlotUse {
            full: 10.0,
            runs: 10.0,
            ..SlotUse::default()
        };
        let mut now = before;
        // 20 more runs possible: 8 made, 9 lost to missing material, 3 to the plan.
        now.full += 20.0;
        now.runs += 8.0;
        now.lost[Shortfall::Input as usize] += 9.0;
        now.lost[Shortfall::Throttled as usize] += 3.0;
        let d = now.since(&before);
        assert!((d.utilization() - 0.4).abs() < 1e-12);
        assert_eq!(d.main_loss(), Some(Shortfall::Input));
        // No day of work: no loss, full use.
        assert_eq!(SlotUse::default().main_loss(), None);
        assert_eq!(SlotUse::default().utilization(), 1.0);
        // A facility without a bottleneck runs at full use.
        let full = SlotUse {
            full: 5.0,
            runs: 5.0,
            ..SlotUse::default()
        };
        assert_eq!(full.utilization(), 1.0);
        assert_eq!(full.main_loss(), None);
    }
}
