//! Sales channels to end customers (MN3, docs/FORMELN.md): beyond the general trade every
//! company reaches, a company can sell through channels per country and goods group. They
//! reach more households – some income fifths more than others – and cost a share of the
//! revenue and a fixed amount per month.

use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::command::CommandError;
use crate::ids::{ChannelId, CountryId, GoodsGroupId, Id};
use crate::ledger::{Account, CostCenter, CostType};
use crate::money::Money;
use crate::state::{Company, CompanyId, GameState};

/// A company's channels and consumer revenue in one country and goods group.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SalesArea {
    pub country: CountryId,
    pub group: GoodsGroupId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub channels: Vec<ChannelId>,
    /// Revenue from consumers this year and the year before.
    #[serde(default)]
    pub revenue: Money,
    #[serde(default)]
    pub revenue_last_year: Money,
}

/// What the channels do for one offer, per income fifth.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Effect {
    /// Factor on the attraction in the choice of seller.
    pub reach: [f64; 5],
    /// Share of the revenue paid to the channels.
    pub cost: [f64; 5],
}

impl Effect {
    pub const NONE: Effect = Effect {
        reach: [1.0; 5],
        cost: [0.0; 5],
    };
}

fn position(company: &Company, country: CountryId, group: GoodsGroupId) -> Result<usize, usize> {
    company
        .sales_areas
        .binary_search_by(|a| (a.country, a.group).cmp(&(country, group)))
}

fn area_mut(company: &mut Company, country: CountryId, group: GoodsGroupId) -> &mut SalesArea {
    let i = match position(company, country, group) {
        Ok(i) => i,
        Err(i) => {
            company.sales_areas.insert(
                i,
                SalesArea {
                    country,
                    group,
                    channels: Vec::new(),
                    revenue: Money::ZERO,
                    revenue_last_year: Money::ZERO,
                },
            );
            i
        }
    };
    &mut company.sales_areas[i]
}

/// The channels a company uses in a country for a goods group.
pub fn chosen(company: &Company, country: CountryId, group: GoodsGroupId) -> &[ChannelId] {
    position(company, country, group).map_or(&[], |i| &company.sales_areas[i].channels)
}

/// Reach of the channels per income fifth and the shares of the sales through them
/// (docs/FORMELN.md, MN3).
fn shares(catalog: &Catalog, channels: &[ChannelId], year: i32) -> ([f64; 5], Vec<[f64; 5]>) {
    let base = catalog.market_model.channels.base_reach;
    let t = f64::from(year);
    let mut total = [base; 5];
    let reaches: Vec<[f64; 5]> = channels
        .iter()
        .map(|&c| {
            let ch = catalog.channels.get(c);
            let r = if ch.available(year) {
                ch.reach.value_at(t)
            } else {
                0.0
            };
            let mut out = [0.0; 5];
            for q in 0..5 {
                out[q] = r * ch.layer_factor[q];
                total[q] += out[q];
            }
            out
        })
        .collect();
    let mut reach = [1.0; 5];
    for q in 0..5 {
        reach[q] = total[q].min(1.0) / base;
    }
    let shares = reaches
        .into_iter()
        .map(|r| {
            let mut s = [0.0; 5];
            for q in 0..5 {
                s[q] = r[q] / total[q];
            }
            s
        })
        .collect();
    (reach, shares)
}

/// The channels' effect for a company's offer to consumers.
pub fn effect(
    catalog: &Catalog,
    company: &Company,
    (country, group): (CountryId, GoodsGroupId),
    year: i32,
) -> Effect {
    if catalog.market_model.channels.base_reach <= 0.0 {
        return Effect::NONE;
    }
    let channels = chosen(company, country, group);
    if channels.is_empty() {
        return Effect::NONE;
    }
    let (reach, shares) = shares(catalog, channels, year);
    let mut cost = [0.0; 5];
    for (&c, s) in channels.iter().zip(&shares) {
        let per_sale = catalog.channels.get(c).cost_per_sale;
        for q in 0..5 {
            cost[q] += s[q] * per_sale;
        }
    }
    Effect { reach, cost }
}

/// Share of the consumer sales running through each chosen channel (middle income
/// fifth).
pub fn through(catalog: &Catalog, channels: &[ChannelId], year: i32) -> Vec<f64> {
    if channels.is_empty() || catalog.market_model.channels.base_reach <= 0.0 {
        return Vec::new();
    }
    shares(catalog, channels, year)
        .1
        .iter()
        .map(|s| s[2])
        .collect()
}

/// Factor on the reputation gain from the channels' mix (middle income fifth).
pub fn reputation_factor(
    catalog: &Catalog,
    company: &Company,
    (country, group): (CountryId, GoodsGroupId),
    year: i32,
) -> f64 {
    let channels = chosen(company, country, group);
    if channels.is_empty() || catalog.market_model.channels.base_reach <= 0.0 {
        return 1.0;
    }
    let (_, shares) = shares(catalog, channels, year);
    let mut through = 0.0;
    let mut factor = 0.0;
    for (&c, s) in channels.iter().zip(&shares) {
        through += s[2];
        factor += s[2] * catalog.channels.get(c).reputation_factor;
    }
    factor + (1.0 - through)
}

/// A sale to consumers: the channels' share of the revenue, the area's revenue.
pub(crate) fn book_sale(
    state: &mut GameState,
    (company, center): (CompanyId, CostCenter),
    (country, group): (CountryId, GoodsGroupId),
    amount: Money,
    cost_share: f64,
) {
    let c = &mut state.companies[company.index()];
    area_mut(c, country, group).revenue += amount;
    if cost_share > 0.0 {
        let cost = amount.scale(cost_share);
        if cost > Money::ZERO {
            c.ledger
                .expense(CostType::Overhead, center, Account::Cash, cost);
        }
    }
}

/// Sets a company's channels in a country for a goods group (`SetChannels`).
pub(crate) fn set(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    (country, group): (CountryId, GoodsGroupId),
    channels: &[ChannelId],
) -> Result<(), CommandError> {
    let year = state.date.year();
    let mut list = Vec::new();
    for &c in channels {
        let ch = catalog.channels.get(c);
        if catalog.market_model.channels.base_reach <= 0.0
            || !ch.available(year)
            || !ch.serves(group)
        {
            return Err(CommandError::NoChannel(catalog.channels.key(c).to_owned()));
        }
        if !list.contains(&c) {
            list.push(c);
        }
    }
    list.sort();
    area_mut(&mut state.companies[company.index()], country, group).channels = list;
    Ok(())
}

/// Monthly fixed cost of a company's channels in a country (USD at the country's price
/// level).
pub fn fixed_cost(
    catalog: &Catalog,
    state: &GameState,
    channels: &[ChannelId],
    country: CountryId,
    year: i32,
) -> Money {
    let level = state.countries.get(country).price_level;
    let usd: f64 = channels
        .iter()
        .map(|&c| {
            let ch = catalog.channels.get(c);
            if ch.available(year) {
                ch.fixed_cost_usd.value_at(f64::from(year))
            } else {
                0.0
            }
        })
        .sum();
    Money::from_usd(usd * level / 12.0).unwrap_or(Money::ZERO)
}

/// Month start: the channels' fixed costs; at the year's start the revenue moves to the
/// year before.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: crate::calendar::Date) {
    if catalog.market_model.channels.base_reach <= 0.0 {
        return;
    }
    let year = date.year();
    let new_year = date.month() == 1;
    for index in 0..state.companies.len() {
        if state.companies[index].bankrupt {
            continue;
        }
        let costs: Vec<Money> = state.companies[index]
            .sales_areas
            .iter()
            .map(|a| fixed_cost(catalog, state, &a.channels, a.country, year))
            .collect();
        let c = &mut state.companies[index];
        for cost in costs {
            if cost > Money::ZERO {
                c.ledger.expense(
                    CostType::Overhead,
                    CostCenter::default(),
                    Account::Cash,
                    cost,
                );
            }
        }
        if new_year {
            for a in &mut c.sales_areas {
                a.revenue_last_year = a.revenue;
                a.revenue = Money::ZERO;
            }
            c.sales_areas
                .retain(|a| !a.channels.is_empty() || a.revenue_last_year > Money::ZERO);
        }
    }
}

/// The channels an AI company of `skill` wants in an area (docs/FORMELN.md, MN3): among
/// the available ones serving the group that have been around long enough for its skill,
/// it adds one by one the channel that raises the expected margin most – more households
/// reached against the channel's share of the revenue and its fixed cost – while that
/// gain is positive.
pub fn ai_choice(
    catalog: &Catalog,
    state: &GameState,
    area: &SalesArea,
    skill: f64,
    year: i32,
) -> Vec<ChannelId> {
    let m = &catalog.market_model.channels;
    let delay = m.ai_delay_years * (1.0 - skill.clamp(0.0, 1.0));
    let t = f64::from(year);
    let reach = |c: ChannelId| catalog.channels.get(c).reach.value_at(t);
    // Revenue without channels: last year's scaled back by the reach it had.
    let had: f64 = area
        .channels
        .iter()
        .filter(|&&c| catalog.channels.get(c).available(year))
        .map(|&c| reach(c))
        .sum();
    let base = m.base_reach;
    let plain = area.revenue_last_year.to_usd() * base / (base + had).min(1.0);
    if plain <= 0.0 {
        return Vec::new();
    }
    let value = |set: &[ChannelId]| {
        let total: f64 = base + set.iter().map(|&c| reach(c)).sum::<f64>();
        let revenue = plain * total.min(1.0) / base;
        let cost: f64 = set
            .iter()
            .map(|&c| {
                let ch = catalog.channels.get(c);
                fixed_cost(catalog, state, &[c], area.country, year).to_usd() * 12.0
                    + ch.cost_per_sale * revenue * reach(c) / total
            })
            .sum();
        m.ai_margin * revenue - cost
    };
    let mut candidates: Vec<ChannelId> = (0..catalog.channels.len())
        .map(ChannelId::from_index)
        .filter(|&c| {
            let ch = catalog.channels.get(c);
            ch.available(year)
                && ch.serves(area.group)
                && f64::from(year - ch.available_from) >= delay
        })
        .collect();
    let mut chosen: Vec<ChannelId> = Vec::new();
    let mut best = value(&chosen);
    loop {
        let mut pick: Option<(usize, f64)> = None;
        for (i, &c) in candidates.iter().enumerate() {
            let mut trial = chosen.clone();
            trial.push(c);
            let v = value(&trial);
            if v > best + 1e-6 && pick.is_none_or(|(_, top)| v > top) {
                pick = Some((i, v));
            }
        }
        let Some((i, v)) = pick else { break };
        chosen.push(candidates.remove(i));
        best = v;
    }
    chosen.sort();
    chosen
}

/// AI companies at the start of a year choose their channels – through the player's
/// command.
pub(crate) fn ai_year(state: &mut GameState, catalog: &Catalog, company: CompanyId) {
    if catalog.market_model.channels.base_reach <= 0.0 {
        return;
    }
    let Some(skill) = state.companies[company.index()]
        .ai
        .as_ref()
        .map(|ai| ai.skill())
    else {
        return;
    };
    let year = state.date.year();
    let wanted: Vec<(CountryId, GoodsGroupId, Vec<ChannelId>)> = state.companies[company.index()]
        .sales_areas
        .iter()
        .filter_map(|a| {
            let w = ai_choice(catalog, state, a, skill, year);
            (w != a.channels).then_some((a.country, a.group, w))
        })
        .collect();
    for (country, group, channels) in wanted {
        let _ = crate::command::execute(
            state,
            catalog,
            company,
            &crate::command::Command::SetChannels {
                country,
                group,
                channels,
            },
        );
    }
}
