//! Loans, overdraft, interest, taxes and insolvency (Lastenheft §11.1, §11.3; formulas in
//! docs/FORMELN.md, section M6).

use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::ledger::{Account, CostCenter, CostType, Ledger};
use crate::math;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::state::{Company, CompanyId, GameState, Loan};

/// Real base rate in a year.
pub fn base_rate(catalog: &Catalog, date: Date) -> f64 {
    catalog
        .finance_model
        .real_rate
        .value_at(date.year_fraction())
}

fn outstanding(company: &Company) -> Money {
    company.loans.iter().map(|l| l.balance).sum()
}

/// Largest new loan the bank grants: a share of fixed assets and inventory as
/// collateral, minus loans already outstanding.
pub fn credit_limit(catalog: &Catalog, company: &Company) -> Money {
    let l = &company.ledger;
    let collateral = l.balance(Account::FixedAssets)
        + l.balance(Account::AssetsUnderConstruction)
        + l.balance(Account::Inventory);
    (collateral.scale(catalog.finance_model.loan_to_value) - outstanding(company)).max(Money::ZERO)
}

/// Overdraft the bank tolerates on the cash account.
pub fn overdraft_limit(catalog: &Catalog, ledger: &Ledger) -> Money {
    ledger
        .total_assets()
        .max(Money::ZERO)
        .scale(catalog.finance_model.overdraft_share)
}

/// Interest rate for a new loan of `amount`; the finance department saves `cut` of the
/// risk premium (ZA2).
pub fn loan_rate(catalog: &Catalog, company: &Company, amount: Money, date: Date, cut: f64) -> f64 {
    let assets = company.ledger.total_assets() + amount;
    rate_for_debt(catalog, (outstanding(company) + amount, assets), date, cut)
}

/// Interest rate at a debt and total assets.
pub fn rate_for_debt(
    catalog: &Catalog,
    (debt, assets): (Money, Money),
    date: Date,
    cut: f64,
) -> f64 {
    let m = &catalog.finance_model;
    let debt_ratio = if assets > Money::ZERO {
        debt.to_usd() / assets.to_usd()
    } else {
        1.0
    };
    let keep = 1.0 - cut;
    base_rate(catalog, date)
        + m.premium_min * keep
        + m.premium_per_debt_ratio * debt_ratio.clamp(0.0, 2.0) * keep
}

/// Monthly instalment of an annuity loan.
pub fn instalment(principal: Money, rate: f64, months: u32) -> Money {
    let r = rate / 12.0;
    let n = f64::from(months.max(1));
    if r.abs() < 1e-12 {
        return principal.scale(1.0 / n);
    }
    principal.scale(r / (1.0 - math::pow(1.0 + r, -n)))
}

/// Takes up a loan; the caller has checked the limit.
pub(crate) fn grant_loan(
    catalog: &Catalog,
    company: &mut Company,
    (amount, years): (Money, u32),
    date: Date,
    cut: f64,
) {
    let rate = loan_rate(catalog, company, amount, date, cut);
    let months = years * 12;
    company.loans.push(Loan {
        principal: amount,
        balance: amount,
        rate,
        start: date,
        months,
        instalment: instalment(amount, rate, months),
    });
    company
        .ledger
        .transfer(Account::Cash, Account::Loans, amount);
}

/// Repays part of a loan early.
pub(crate) fn repay(company: &mut Company, loan: usize, amount: Money) {
    let amount = amount.min(company.loans[loan].balance);
    company
        .ledger
        .transfer(Account::Loans, Account::Cash, amount);
    company.loans[loan].balance -= amount;
    if company.loans[loan].balance == Money::ZERO {
        company.loans.remove(loan);
    }
}

/// End of a month (`last_day`): interest and instalments, overdraft interest, and on
/// 31 December the profit tax.
pub(crate) fn month_end(state: &mut GameState, catalog: &Catalog, last_day: Date) {
    let base = base_rate(catalog, last_day);
    let overdraft_rate = base + catalog.finance_model.overdraft_premium;
    let year_end = last_day.month() == 12;
    for index in 0..state.companies.len() {
        let hq = state.companies[index].headquarters;
        let tax_rate = state.countries.get(hq).corporate_tax;
        let company = &mut state.companies[index];
        if company.bankrupt {
            continue;
        }
        let center = CostCenter::default();
        for loan in &mut company.loans {
            let interest = loan.balance.scale(loan.rate / 12.0);
            let repayment = (loan.instalment - interest)
                .max(Money::ZERO)
                .min(loan.balance);
            company
                .ledger
                .expense(CostType::Interest, center, Account::Cash, interest);
            company
                .ledger
                .transfer(Account::Loans, Account::Cash, repayment);
            loan.balance -= repayment;
        }
        let before = company.loans.len();
        company.loans.retain(|l| l.balance > Money::ZERO);
        let paid_off = company.loans.len() < before;
        let cash = company.ledger.cash();
        if cash.is_negative() {
            let interest = (-cash).scale(overdraft_rate / 12.0);
            company
                .ledger
                .expense(CostType::Interest, center, Account::Cash, interest);
        }
        if year_end {
            pay_profit_tax(company, tax_rate);
        }
        if paid_off {
            // Few companies; the cast is exact.
            let id = CompanyId(index as u32);
            crate::management::settle_topic(state, id, crate::decision::Topic::Refinance);
        }
    }
}

fn pay_profit_tax(company: &mut Company, rate: f64) {
    let profit: Money = company
        .ledger
        .year
        .by_type
        .iter()
        .filter(|(t, _)| **t != CostType::Taxes)
        .map(|(_, &m)| m)
        .sum();
    if profit > Money::ZERO {
        let taxable = (profit - company.loss_carryforward).max(Money::ZERO);
        company.loss_carryforward = (company.loss_carryforward - profit).max(Money::ZERO);
        let tax = taxable.scale(rate);
        company
            .ledger
            .expense(CostType::Taxes, CostCenter::default(), Account::Cash, tax);
    } else {
        company.loss_carryforward += -profit;
    }
}

/// A company is insolvent when its overdraft exceeds the limit and no new loan could
/// cover the gap (Lastenheft §11.3).
pub fn is_insolvent(catalog: &Catalog, company: &Company) -> bool {
    let cash = company.ledger.cash();
    let limit = overdraft_limit(catalog, &company.ledger);
    let gap = -cash - limit;
    gap > Money::ZERO && credit_limit(catalog, company) < gap
}

/// Monthly insolvency check after the month's payments. Marks insolvent companies;
/// for the player the game ends.
pub(crate) fn check_insolvency(state: &mut GameState, catalog: &Catalog) -> Vec<Message> {
    let mut messages = Vec::new();
    for index in 0..state.companies.len() {
        let company = &state.companies[index];
        if company.bankrupt || !is_insolvent(catalog, company) {
            continue;
        }
        state.companies[index].bankrupt = true;
        let id = CompanyId(u32::try_from(index).expect("company count fits u32"));
        if id == state.player {
            state.game_over = true;
            messages.push(Message::new(MessageKind::Crisis, keys::GAME_OVER_INSOLVENT));
        } else if catalog.deal_model.insolvency_days > 0 {
            // The sites are auctioned before they are given up (M38).
            crate::ai::stop_operations(state, id);
            let until = state
                .date
                .add_days(i32::try_from(catalog.deal_model.insolvency_days).unwrap_or(i32::MAX));
            state.companies[index].auction_until = Some(until);
            let sites = state.sites.iter().filter(|s| s.owner == id).count();
            messages.push(
                Message::new(MessageKind::Info, keys::COMPANY_INSOLVENT_AUCTION)
                    .with("firma", Param::Text(state.companies[index].name.clone()))
                    .with(
                        "anzahl",
                        Param::Integer(i64::try_from(sites).unwrap_or(i64::MAX)),
                    )
                    .with("datum", Param::Date(until)),
            );
        } else {
            crate::ai::release_assets(state, id);
            messages.push(
                Message::new(MessageKind::Info, keys::COMPANY_INSOLVENT)
                    .with("firma", Param::Text(state.companies[index].name.clone())),
            );
        }
    }
    messages
}

/// Warning for the player when the cash account is overdrawn.
pub(crate) fn overdraft_warning(state: &GameState, catalog: &Catalog) -> Option<Message> {
    let company = state.company(state.player)?;
    let cash = company.ledger.cash();
    cash.is_negative().then(|| {
        Message::new(MessageKind::Warning, keys::OVERDRAFT)
            .with("betrag", Param::Money(-cash))
            .with(
                "limit",
                Param::Money(overdraft_limit(catalog, &company.ledger)),
            )
    })
}
