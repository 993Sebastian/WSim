//! The player's banks (K4; docs/BEDIENUNG.md, "Finanzen → Bank").

use serde::{Deserialize, Serialize};

use super::{iso, usd};
use crate::bank;
use crate::game::Game;
use crate::ledger::{Account, CostType};
use crate::money::Money;
use crate::state::CompanyId;

/// A loan a bank gave.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BankLoanView {
    pub borrower: String,
    pub balance_usd: f64,
    pub rate: f64,
    pub start: String,
    pub months: u32,
}

/// A bank of the player's group.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BankRowView {
    pub company: u32,
    pub name: String,
    /// The player's company itself, else a subsidiary.
    pub own: bool,
    pub deposit_spread: f64,
    pub deposit_rate: f64,
    pub loan_discount: f64,
    pub max_debt_ratio: f64,
    pub deposits_usd: f64,
    /// Deposits the bank draws in the long run at its rate.
    pub deposit_target_usd: f64,
    /// Deposits at most (leverage times equity).
    pub capacity_usd: f64,
    pub equity_usd: f64,
    pub cash_usd: f64,
    pub reserve_usd: f64,
    /// Cash it may lend.
    pub room_usd: f64,
    pub loans_given_usd: f64,
    pub loans: Vec<BankLoanView>,
    /// The running year: interest (received less paid), write-offs, result.
    pub interest_year_usd: f64,
    pub write_offs_year_usd: f64,
    pub result_year_usd: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BankView {
    /// Without banks in the data the view is empty.
    pub enabled: bool,
    pub base_rate: f64,
    pub banks: Vec<BankRowView>,
    /// Whether the player may found a bank as a subsidiary.
    pub subsidiaries: bool,
}

pub fn bank_view(game: &Game) -> BankView {
    let (state, catalog) = (game.state(), game.catalog());
    let me = state.player;
    let m = &catalog.bank;
    let banks = crate::group::members(state, me)
        .into_iter()
        .filter_map(|id: CompanyId| {
            let c = &state.companies[id.index()];
            let s = c.bank.as_ref()?;
            let l = &c.ledger;
            let deposits = l.balance(Account::Deposits);
            let year = |t: CostType| l.year.by_type.get(&t).copied().unwrap_or_default();
            let loans = state
                .companies
                .iter()
                .flat_map(|b| b.loans.iter().map(move |loan| (b, loan)))
                .filter(|(_, loan)| loan.lender == Some(id))
                .map(|(b, loan)| BankLoanView {
                    borrower: b.name.clone(),
                    balance_usd: usd(loan.balance),
                    rate: loan.rate,
                    start: iso(loan.start),
                    months: loan.months,
                })
                .collect();
            Some(BankRowView {
                company: id.0,
                name: c.name.clone(),
                own: id == me,
                deposit_spread: s.deposit_spread,
                deposit_rate: bank::deposit_rate(catalog, s, state.date),
                loan_discount: s.loan_discount,
                max_debt_ratio: s.max_debt_ratio,
                deposits_usd: usd(deposits),
                deposit_target_usd: usd(bank::deposit_target(catalog, c)),
                capacity_usd: usd(crate::ranking::equity(c)
                    .max(Money::ZERO)
                    .scale(m.leverage_max)),
                equity_usd: usd(crate::ranking::equity(c)),
                cash_usd: usd(l.cash()),
                reserve_usd: usd(deposits.scale(m.reserve)),
                room_usd: usd(bank::lending_room(catalog, c)),
                loans_given_usd: usd(l.balance(Account::LoansGiven)),
                loans,
                interest_year_usd: usd(year(CostType::Interest)),
                write_offs_year_usd: usd(year(CostType::Investments)),
                result_year_usd: usd(l.year.total()),
            })
        })
        .collect();
    BankView {
        enabled: m.enabled,
        base_rate: crate::finance::base_rate(catalog, state.date),
        banks,
        subsidiaries: catalog.subsidiaries.enabled,
    }
}
