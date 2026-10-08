//! Subsidiaries and the group (W6; docs/BEDIENUNG.md, "Organisation → Tochterfirmen").

use serde::{Deserialize, Serialize};

use super::usd;
use crate::command::site_type_key;
use crate::game::Game;
use crate::group::{self, SubsidiaryFocus};
use crate::ledger::{Account, CostType};
use crate::money::Money;
use crate::ranking::{equity, revenue_of_year};
use crate::state::{CompanyId, SiteId};

/// A subsidiary of the player's group.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SubsidiaryView {
    pub company: u32,
    pub name: String,
    pub country: String,
    /// Name of its parent.
    pub parent: String,
    /// The player may steer it (a direct subsidiary).
    pub direct: bool,
    /// `produktion` or `logistik`.
    pub focus: String,
    pub cash_usd: f64,
    pub equity_usd: f64,
    /// Capital the parent paid in.
    pub paid_in_usd: f64,
    pub revenue_year_usd: f64,
    pub result_year_usd: f64,
    pub sites: u32,
    pub vehicles: u32,
    /// The most the parent can take out now.
    pub payout_max_usd: f64,
}

/// A site of the player or of a direct subsidiary that can move within the group.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GroupSiteView {
    pub site: u32,
    pub company: u32,
    pub company_name: String,
    pub country: String,
    pub kind_text: String,
    /// What the receiver pays: the book values of the site and its stocks.
    pub book_usd: f64,
}

/// One line of the group balance sheet or income statement.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GroupLineView {
    pub key: String,
    pub usd: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GroupView {
    /// Without subsidiaries in the data the view is empty.
    pub enabled: bool,
    pub min_capital_usd: f64,
    pub founding_cost_usd: f64,
    /// The player's company itself.
    pub company: u32,
    pub company_name: String,
    pub own_equity_usd: f64,
    pub subsidiaries: Vec<SubsidiaryView>,
    pub sites: Vec<GroupSiteView>,
    /// Group balance: assets, then loans and equity.
    pub assets: Vec<GroupLineView>,
    pub liabilities: Vec<GroupLineView>,
    pub total_assets_usd: f64,
    pub group_equity_usd: f64,
    /// The running year by cost type.
    pub income: Vec<GroupLineView>,
    pub result_year_usd: f64,
}

fn focus_key(focus: SubsidiaryFocus) -> &'static str {
    match focus {
        SubsidiaryFocus::Production => "produktion",
        SubsidiaryFocus::Logistics => "logistik",
    }
}

fn line(account: Account, amount: Money) -> GroupLineView {
    GroupLineView {
        key: account.text_key().to_owned(),
        usd: usd(amount),
    }
}

pub fn group(game: &Game) -> GroupView {
    let (state, catalog) = (game.state(), game.catalog());
    let me = state.player;
    let m = &catalog.subsidiaries;
    let members = group::members(state, me);
    let subsidiaries = members
        .iter()
        .skip(1)
        .map(|&c| {
            let company = &state.companies[c.index()];
            let of = company.subsidiary_of.expect("member of a group");
            let l = &company.ledger;
            let profit = (l.balance(Account::RetainedEarnings) + l.balance(Account::Result))
                .max(Money::ZERO);
            let stake = state.companies[of.parent.index()]
                .ledger
                .balance(Account::Participations);
            let capital = l.balance(Account::Equity).min(stake).max(Money::ZERO);
            SubsidiaryView {
                company: c.0,
                name: company.name.clone(),
                country: catalog.countries.key(company.headquarters).to_owned(),
                parent: state.companies[of.parent.index()].name.clone(),
                direct: of.parent == me,
                focus: focus_key(of.focus).to_owned(),
                cash_usd: usd(l.cash()),
                equity_usd: usd(equity(company)),
                paid_in_usd: usd(l.balance(Account::Equity)),
                revenue_year_usd: usd(revenue_of_year(company)),
                result_year_usd: usd(l.balance(Account::Result)),
                sites: u32::try_from(state.sites.iter().filter(|s| s.owner == c).count())
                    .unwrap_or(u32::MAX),
                vehicles: company.logistics.fleet.iter().map(|h| h.count).sum(),
                payout_max_usd: usd((profit + capital).min(l.cash().max(Money::ZERO))),
            }
        })
        .collect();
    let movable = |c: CompanyId| {
        c == me
            || state.companies[c.index()]
                .subsidiary_of
                .is_some_and(|s| s.parent == me && !state.companies[c.index()].bankrupt)
    };
    let sites = if m.enabled && members.len() > 1 {
        state
            .sites
            .iter()
            .enumerate()
            .filter(|(_, s)| movable(s.owner))
            .map(|(i, s)| {
                let site = SiteId(u32::try_from(i).expect("site count fits u32"));
                let v = crate::deals::site_value(state, catalog, site);
                GroupSiteView {
                    site: site.0,
                    company: s.owner.0,
                    company_name: state.companies[s.owner.index()].name.clone(),
                    country: catalog.countries.key(s.country).to_owned(),
                    kind_text: site_type_key(s.kind),
                    book_usd: usd(v.fixed_assets
                        + v.under_construction
                        + v.goodwill
                        + v.inventory
                        + v.land_book),
                }
            })
            .collect()
    } else {
        Vec::new()
    };
    let g = group::consolidated(state, me);
    let assets = Account::ALL
        .iter()
        .filter(|a| a.is_asset())
        .map(|&a| line(a, g.balance(a)))
        .filter(|l| l.usd != 0.0)
        .collect();
    let liabilities = [
        Account::Loans,
        Account::Bonds,
        Account::Equity,
        Account::RetainedEarnings,
        Account::Result,
    ]
    .iter()
    .map(|&a| line(a, g.balance(a)))
    .collect();
    let income = CostType::ALL
        .iter()
        .filter_map(|&t| {
            let amount = g.year.get(&t).copied()?;
            (amount != Money::ZERO).then(|| GroupLineView {
                key: t.text_key().to_owned(),
                usd: usd(amount),
            })
        })
        .collect();
    let own = &state.companies[me.index()];
    GroupView {
        enabled: m.enabled,
        min_capital_usd: usd(m.min_capital),
        founding_cost_usd: usd(m.founding_cost),
        company: me.0,
        company_name: own.name.clone(),
        own_equity_usd: usd(equity(own)),
        subsidiaries,
        sites,
        assets,
        liabilities,
        total_assets_usd: usd(g.total_assets()),
        group_equity_usd: usd(g.equity()),
        income,
        result_year_usd: usd(g.balance(Account::Result)),
    }
}
