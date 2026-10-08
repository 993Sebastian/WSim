//! The player's banks (K4, Lastenheft §17.3; formulas in docs/FORMELN.md, K4).
//!
//! A bank takes deposits at its rate and lends to companies outside its group at a
//! discount on the rate of the market's banks, within its reserve and its credit
//! standard. Borrowers pay interest and instalments to it; their failures are its losses.

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::command::CommandError;
use crate::ledger::{Account, CostCenter, CostType};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::state::{Company, CompanyId, GameState};

/// What a bank offers; set by the player.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BankSettings {
    /// Deposit rate over the real base rate.
    pub deposit_spread: f64,
    /// Discount on the rate of the market's banks for its loans.
    pub loan_discount: f64,
    /// Debt over total assets a borrower may have after the loan.
    pub max_debt_ratio: f64,
}

impl BankSettings {
    pub fn start(catalog: &Catalog) -> Self {
        let m = &catalog.bank;
        BankSettings {
            deposit_spread: m.start_deposit_spread,
            loan_discount: m.start_loan_discount,
            max_debt_ratio: m.start_max_debt_ratio,
        }
    }
}

fn company_id(index: usize) -> CompanyId {
    CompanyId(u32::try_from(index).expect("company count fits u32"))
}

/// The deposit rate of a bank today.
pub fn deposit_rate(catalog: &Catalog, settings: &BankSettings, date: Date) -> f64 {
    crate::finance::base_rate(catalog, date) + settings.deposit_spread
}

/// Deposits a bank draws in the long run (docs/FORMELN.md, K4); without banking none.
pub fn deposit_target(catalog: &Catalog, company: &Company) -> Money {
    let m = &catalog.bank;
    let Some(s) = &company.bank else {
        return Money::ZERO;
    };
    let capacity = crate::ranking::equity(company)
        .max(Money::ZERO)
        .scale(m.leverage_max);
    let share = (0.5 + m.elasticity * (s.deposit_spread - m.neutral_spread)).clamp(0.0, 1.0);
    capacity.scale(share)
}

/// Cash a bank may lend: what is above the reserve on its deposits.
pub fn lending_room(catalog: &Catalog, company: &Company) -> Money {
    let reserve = company
        .ledger
        .balance(Account::Deposits)
        .scale(catalog.bank.reserve);
    (company.ledger.cash() - reserve).max(Money::ZERO)
}

/// Debt over total assets of a borrower after a new loan of `amount`.
fn debt_ratio_after(company: &Company, amount: Money) -> f64 {
    let l = &company.ledger;
    let assets = l.total_assets() + amount;
    if assets <= Money::ZERO {
        return f64::INFINITY;
    }
    (l.balance(Account::Loans) + l.balance(Account::Bonds) + amount).to_usd() / assets.to_usd()
}

/// The bank that lends `amount` to `borrower` cheapest, with its rate, if any bank takes
/// it at a rate below `market`.
pub fn lender_for(
    state: &GameState,
    catalog: &Catalog,
    borrower: CompanyId,
    amount: Money,
    market: f64,
) -> Option<(CompanyId, f64)> {
    if !catalog.bank.enabled {
        return None;
    }
    let b = &state.companies[borrower.index()];
    let ratio = debt_ratio_after(b, amount);
    state
        .companies
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            let s = c.bank.as_ref()?;
            let id = company_id(i);
            let lends = !c.bankrupt
                && s.loan_discount > 0.0
                && ratio <= s.max_debt_ratio
                && lending_room(catalog, c) >= amount
                && !crate::group::same_group(state, borrower, id);
            lends.then_some((market * (1.0 - s.loan_discount), i))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)))
        .map(|(rate, i)| (company_id(i), rate))
}

/// The bank pays out a loan of `amount`.
pub(crate) fn lend(state: &mut GameState, bank: CompanyId, amount: Money) {
    state.companies[bank.index()]
        .ledger
        .transfer(Account::LoansGiven, Account::Cash, amount);
}

/// The bank receives repayment and interest of a loan.
pub(crate) fn receive(state: &mut GameState, bank: CompanyId, repayment: Money, interest: Money) {
    let l = &mut state.companies[bank.index()].ledger;
    if repayment > Money::ZERO {
        l.transfer(Account::Cash, Account::LoansGiven, repayment);
    }
    if interest > Money::ZERO {
        l.income(
            CostType::Interest,
            CostCenter::default(),
            Account::Cash,
            interest,
        );
    }
}

/// Sets what a bank of the actor's group offers.
pub(crate) fn set(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    company: CompanyId,
    settings: BankSettings,
) -> Result<(), CommandError> {
    if !catalog.bank.enabled {
        return Err(CommandError::NoBanks);
    }
    if company != actor {
        crate::group::own_subsidiary(state, actor, company)?;
    }
    let s = &settings;
    let valid = (-0.2..=0.2).contains(&s.deposit_spread)
        && (0.0..=0.9).contains(&s.loan_discount)
        && (0.0..=1.0).contains(&s.max_debt_ratio);
    if !valid {
        return Err(CommandError::InvalidBankSettings);
    }
    let c = &mut state.companies[company.index()];
    let Some(bank) = c.bank.as_mut() else {
        return Err(CommandError::NotABank);
    };
    *bank = settings;
    Ok(())
}

/// End of a month (`last_day`): interest on deposits, and deposits flowing in or out
/// towards their target.
pub(crate) fn month_end(state: &mut GameState, catalog: &Catalog, last_day: Date) {
    let m = &catalog.bank;
    if !m.enabled {
        return;
    }
    for c in &mut state.companies {
        let deposits = c.ledger.balance(Account::Deposits);
        if c.bankrupt || (c.bank.is_none() && deposits == Money::ZERO) {
            continue;
        }
        if let Some(s) = &c.bank {
            let interest = deposits.scale(deposit_rate(catalog, s, last_day).max(0.0) / 12.0);
            if interest > Money::ZERO {
                c.ledger.expense(
                    CostType::Interest,
                    CostCenter::default(),
                    Account::Cash,
                    interest,
                );
            }
        }
        let change = (deposit_target(catalog, c) - deposits).scale(m.adjustment);
        if change > Money::ZERO {
            c.ledger.transfer(Account::Cash, Account::Deposits, change);
        } else if change < Money::ZERO {
            // Withdrawals are paid even when the cash runs short (a run on the bank).
            c.ledger.transfer(Account::Deposits, Account::Cash, -change);
        }
    }
}

/// After the insolvency check: banks write off what failed borrowers still owe them.
/// Returns the messages for the player.
pub(crate) fn write_off_failures(state: &mut GameState) -> Vec<Message> {
    let mut news = Vec::new();
    let player = state.main_company;
    for i in 0..state.companies.len() {
        if !state.companies[i].bankrupt {
            continue;
        }
        let name = state.companies[i].name.clone();
        let mut lost: Vec<(CompanyId, Money)> = Vec::new();
        for loan in &mut state.companies[i].loans {
            if let Some(bank) = loan.lender.take() {
                lost.push((bank, loan.balance));
            }
        }
        for (bank, amount) in lost {
            if amount > Money::ZERO {
                state.companies[bank.index()].ledger.expense(
                    CostType::Investments,
                    CostCenter::default(),
                    Account::LoansGiven,
                    amount,
                );
            }
            if player.is_some_and(|p| crate::group::same_group(state, p, bank)) {
                news.push(
                    Message::new(MessageKind::Warning, keys::BANK_LOAN_LOST)
                        .with("firma", Param::Text(name.clone()))
                        .with("betrag", Param::Money(amount)),
                );
            }
        }
    }
    news
}
