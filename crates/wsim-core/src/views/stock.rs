//! Stock market (K1; docs/BEDIENUNG.md, "Finanzen → Börse").

use serde::{Deserialize, Serialize};

use super::{iso, usd};
use crate::game::Game;
use crate::group;
use crate::money::Money;
use crate::ranking::equity;
use crate::state::{Company, CompanyId, Holder};
use crate::stock::{self, Listing};

/// Months of history shown per company.
const COMPANY_MONTHS: usize = 24;
/// Shares offered to buy at once; the largest possible share is added.
const BUY_STEPS: [f64; 4] = [0.01, 0.05, 0.1, 0.25];
/// Steps of new shares, in percent.
const ISSUE_PERCENT: f64 = 5.0;
const STEPS_MAX: u32 = 20;

/// A share with what it costs or brings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StockQuoteView {
    pub share: f64,
    pub usd: f64,
}

/// New shares of the player's company: what they bring and what remains.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IssueQuoteView {
    pub share: f64,
    pub proceeds_usd: f64,
    pub cost_usd: f64,
    /// The player's share of its company afterwards.
    pub stake_after: f64,
}

/// A listed company.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ListedCompanyView {
    pub company: u32,
    pub name: String,
    pub country: String,
    /// `eigen` (the player's company), `konzern` (one of its subsidiaries), `fremd`.
    pub relation: String,
    pub since: String,
    pub value_usd: f64,
    pub price_usd: f64,
    /// Against the value a month and a year ago.
    pub change_month: Option<f64>,
    pub change_year: Option<f64>,
    pub equity_usd: f64,
    /// Earnings of a year (average of the last months, see `stock::earnings`).
    pub earnings_usd: f64,
    /// Price-earnings ratio, without earnings none.
    pub pe: Option<f64>,
    pub dividend_usd: f64,
    pub dividend_yield: Option<f64>,
    pub free_float: f64,
    /// The share the player's company holds.
    pub held: f64,
    pub held_cost_usd: f64,
    pub held_value_usd: f64,
    /// What buying costs and selling brings, by share.
    pub buy: Vec<StockQuoteView>,
    pub sell: Vec<StockQuoteView>,
    /// Market value at the start of the last months, oldest first.
    pub series_usd: Vec<f64>,
}

/// The player's own company on the stock market.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnListingView {
    pub listed: bool,
    /// Subsidiaries do not go public themselves.
    pub subsidiary: bool,
    pub equity_usd: f64,
    pub equity_min_usd: f64,
    /// The player's share of its company.
    pub player_stake: f64,
    pub free_float: f64,
    /// Value new shares are sold at (after the discount).
    pub issue_value_usd: f64,
    pub discount: f64,
    pub share_max: f64,
    /// Possible steps of new shares (keeping the majority).
    pub issue: Vec<IssueQuoteView>,
    /// Share of last year's profit paid out.
    pub payout: f64,
    pub dividend_month: u32,
    pub profit_last_year_usd: f64,
    /// The dividend at the current payout in the next dividend month.
    pub dividend_estimate_usd: f64,
    pub last_dividend_usd: f64,
    /// Dividends the player received privately as owner.
    pub player_dividends_usd: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StockMarketView {
    /// Without a stock market in the data the view is empty.
    pub enabled: bool,
    pub index: f64,
    /// The investors' mood: S − 1 (0 neutral).
    pub mood: f64,
    /// First days of the months of the index, oldest first; the series of the
    /// companies end with the same month.
    pub months: Vec<String>,
    pub index_series: Vec<f64>,
    pub own: OwnListingView,
    /// The player's company first, then by market value.
    pub companies: Vec<ListedCompanyView>,
    pub portfolio_cost_usd: f64,
    pub portfolio_value_usd: f64,
    pub trade_share_max: f64,
}

fn change(listing: &Listing, months: usize) -> Option<f64> {
    let n = listing.history.len();
    let (_, before) = listing.history.get(n.checked_sub(months + 1)?)?;
    (*before > Money::ZERO).then(|| listing.value.to_usd() / before.to_usd() - 1.0)
}

fn listed_company(
    game: &Game,
    id: CompanyId,
    c: &Company,
    l: &Listing,
    relation: &str,
) -> ListedCompanyView {
    let (state, catalog) = (game.state(), game.catalog());
    let me = state.player;
    let m = &catalog.stock;
    let earnings = stock::earnings(catalog, c);
    let held = stock::stake(c, Holder::Company(me));
    let float = stock::free_float(c);
    let buy_max = float.min(m.trade_share_max - held).max(0.0);
    let mut buy_steps: Vec<f64> = BUY_STEPS
        .iter()
        .copied()
        .filter(|&q| q < buy_max - 1e-9)
        .collect();
    if buy_max > 1e-9 {
        buy_steps.push(buy_max);
    }
    let foreign = relation == "fremd";
    let buy = if foreign && !c.bankrupt {
        buy_steps
            .into_iter()
            .map(|q| StockQuoteView {
                share: q,
                usd: usd(stock::buy_price(catalog, l, q)),
            })
            .collect()
    } else {
        Vec::new()
    };
    let sell = if held > 1e-9 {
        let mut steps: Vec<f64> = BUY_STEPS
            .iter()
            .copied()
            .filter(|&q| q < held - 1e-9)
            .collect();
        steps.push(held);
        steps
            .into_iter()
            .map(|q| StockQuoteView {
                share: q,
                usd: usd(stock::sell_proceeds(catalog, l, q)),
            })
            .collect()
    } else {
        Vec::new()
    };
    let dividend = l.last_dividend;
    ListedCompanyView {
        company: id.0,
        name: c.name.clone(),
        country: catalog.countries.key(c.headquarters).to_owned(),
        relation: relation.to_owned(),
        since: iso(l.since),
        value_usd: usd(l.value),
        price_usd: usd(l.price()),
        change_month: change(l, 1),
        change_year: change(l, 12),
        equity_usd: usd(equity(c)),
        earnings_usd: usd(earnings),
        pe: (earnings > Money::ZERO).then(|| l.value.to_usd() / earnings.to_usd()),
        dividend_usd: usd(dividend),
        dividend_yield: (l.value > Money::ZERO).then(|| dividend.to_usd() / l.value.to_usd()),
        free_float: float,
        held,
        held_cost_usd: usd(state.companies[me.index()]
            .stock_cost
            .get(&id)
            .copied()
            .unwrap_or_default()),
        held_value_usd: usd(l.value.scale(held)),
        buy,
        sell,
        series_usd: l
            .history
            .iter()
            .rev()
            .take(COMPANY_MONTHS)
            .rev()
            .map(|(_, v)| usd(*v))
            .collect(),
    }
}

fn own_listing(game: &Game) -> OwnListingView {
    let (state, catalog) = (game.state(), game.catalog());
    let m = &catalog.stock;
    let c = &state.companies[state.player.index()];
    let before = stock::issue_value(catalog, &state.stock, c);
    let player_stake = stock::stake(c, Holder::Player);
    let mut issue = Vec::new();
    for k in 1..=STEPS_MAX {
        let s = f64::from(k) * ISSUE_PERCENT / 100.0;
        if s > m.ipo_share_max + 1e-9 {
            break;
        }
        if stock::keeps_majority(c, s).is_ok() {
            let proceeds = stock::issue_proceeds(before, s);
            issue.push(IssueQuoteView {
                share: s,
                proceeds_usd: usd(proceeds),
                cost_usd: usd(proceeds.scale(m.ipo_cost_share)),
                stake_after: player_stake * (1.0 - s),
            });
        }
    }
    let profit = c
        .ledger
        .years
        .last()
        .map(|y| y.by_type.values().copied().sum::<Money>())
        .unwrap_or_default();
    OwnListingView {
        listed: c.listing.is_some(),
        subsidiary: c.subsidiary_of.is_some(),
        equity_usd: usd(equity(c)),
        equity_min_usd: usd(m.ipo_equity_min),
        player_stake,
        free_float: stock::free_float(c),
        issue_value_usd: usd(before),
        discount: m.ipo_discount,
        share_max: m.ipo_share_max,
        issue,
        payout: c.dividend_payout.unwrap_or(0.0),
        dividend_month: m.dividend_month,
        profit_last_year_usd: usd(profit),
        dividend_estimate_usd: usd(stock::dividend(catalog, c)),
        last_dividend_usd: usd(c.listing.as_ref().map_or(Money::ZERO, |l| l.last_dividend)),
        player_dividends_usd: usd(state.stock.player_dividends),
    }
}

pub fn stock(game: &Game) -> StockMarketView {
    let (state, catalog) = (game.state(), game.catalog());
    let me = state.player;
    let mut companies: Vec<ListedCompanyView> = state
        .companies
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            let l = c.listing.as_ref()?;
            let id = CompanyId(u32::try_from(i).ok()?);
            let relation = if id == me {
                "eigen"
            } else if group::same_group(state, me, id) {
                "konzern"
            } else {
                "fremd"
            };
            Some(listed_company(game, id, c, l, relation))
        })
        .collect();
    companies.sort_by(|a, b| {
        (b.relation == "eigen")
            .cmp(&(a.relation == "eigen"))
            .then(b.value_usd.total_cmp(&a.value_usd))
            .then(a.company.cmp(&b.company))
    });
    let portfolio_cost_usd = companies.iter().map(|c| c.held_cost_usd).sum();
    let portfolio_value_usd = companies.iter().map(|c| c.held_value_usd).sum();
    StockMarketView {
        enabled: catalog.stock.enabled,
        index: state.stock.index,
        mood: libm::exp(state.stock.sentiment) - 1.0,
        months: state
            .stock
            .index_history
            .iter()
            .map(|(d, _)| iso(*d))
            .collect(),
        index_series: state.stock.index_history.iter().map(|(_, v)| *v).collect(),
        own: own_listing(game),
        companies,
        portfolio_cost_usd,
        portfolio_value_usd,
        trade_share_max: catalog.stock.trade_share_max,
    }
}
