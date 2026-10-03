//! Double-entry bookkeeping of a company (Lastenheft §11.4, §14.2).
//!
//! Every movement of money or value is a booking between two accounts. Income and
//! expenses go to the result account with a cost type (Gesamtkostenverfahren) and a
//! cost center (site, product). Reports (income statement, balance sheet, cash flow)
//! are evaluations of these bookings.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::ids::ProductId;
use crate::money::Money;
use crate::state::SiteId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Account {
    // Assets
    Cash,
    Inventory,
    FixedAssets,
    AssetsUnderConstruction,
    // Liabilities and equity
    Loans,
    Equity,
    RetainedEarnings,
    /// Result of the current year; closed into retained earnings at year end.
    Result,
}

impl Account {
    pub const ALL: [Account; 8] = [
        Account::Cash,
        Account::Inventory,
        Account::FixedAssets,
        Account::AssetsUnderConstruction,
        Account::Loans,
        Account::Equity,
        Account::RetainedEarnings,
        Account::Result,
    ];

    pub fn is_asset(self) -> bool {
        matches!(
            self,
            Account::Cash
                | Account::Inventory
                | Account::FixedAssets
                | Account::AssetsUnderConstruction
        )
    }

    fn slot(self) -> usize {
        self as usize
    }
}

/// Cost types (Kostenarten). Income types are positive in the result, expenses negative.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CostType {
    Revenue,
    InventoryChange,
    Material,
    Personnel,
    Energy,
    Transport,
    Customs,
    Taxes,
    Marketing,
    Research,
    Interest,
    Depreciation,
    Maintenance,
    Other,
}

/// Where a cost arises; both parts optional.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct CostCenter {
    pub site: Option<SiteId>,
    pub product: Option<ProductId>,
}

impl CostCenter {
    pub fn site(site: SiteId) -> Self {
        Self {
            site: Some(site),
            product: None,
        }
    }

    pub fn product(site: SiteId, product: ProductId) -> Self {
        Self {
            site: Some(site),
            product: Some(product),
        }
    }
}

/// Result of a period by cost type, site and product.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PeriodResult {
    pub start: Option<Date>,
    pub by_type: BTreeMap<CostType, Money>,
    pub by_site: BTreeMap<SiteId, Money>,
    pub by_product: BTreeMap<ProductId, Money>,
}

impl PeriodResult {
    fn starting(start: Date) -> Self {
        Self {
            start: Some(start),
            ..Self::default()
        }
    }

    fn add(&mut self, cost: CostType, center: CostCenter, amount: Money) {
        *self.by_type.entry(cost).or_default() += amount;
        if let Some(site) = center.site {
            *self.by_site.entry(site).or_default() += amount;
        }
        if let Some(product) = center.product {
            *self.by_product.entry(product).or_default() += amount;
        }
    }

    pub fn total(&self) -> Money {
        self.by_type.values().copied().sum()
    }
}

/// How many closed months are kept.
const MONTHS_KEPT: usize = 24;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ledger {
    /// Balances, debit positive (assets positive, liabilities and equity negative).
    balances: Vec<Money>,
    pub month: PeriodResult,
    pub year: PeriodResult,
    /// Closed months, oldest first.
    pub months: Vec<PeriodResult>,
    /// Closed years, oldest first.
    pub years: Vec<PeriodResult>,
}

impl Ledger {
    /// A new company with its start capital in cash.
    pub fn new(start: Date, capital: Money) -> Self {
        let mut ledger = Self {
            balances: vec![Money::ZERO; Account::ALL.len()],
            month: PeriodResult::starting(start.first_of_month()),
            year: PeriodResult::starting(Date::first_of_year(start.year())),
            months: Vec::new(),
            years: Vec::new(),
        };
        ledger.transfer(Account::Cash, Account::Equity, capital);
        ledger
    }

    /// Booking between balance sheet accounts: `debit` increases by `amount` (assets),
    /// `credit` decreases (assets) or increases (liabilities, equity).
    pub fn transfer(&mut self, debit: Account, credit: Account, amount: Money) {
        self.balances[debit.slot()] += amount;
        self.balances[credit.slot()] -= amount;
    }

    /// An expense paid from (or reducing) `credit`, e.g. wages from cash.
    pub fn expense(&mut self, cost: CostType, center: CostCenter, credit: Account, amount: Money) {
        self.transfer(Account::Result, credit, amount);
        self.record(cost, center, -amount);
    }

    /// Income received into (or increasing) `debit`, e.g. revenue into cash.
    pub fn income(&mut self, cost: CostType, center: CostCenter, debit: Account, amount: Money) {
        self.transfer(debit, Account::Result, amount);
        self.record(cost, center, amount);
    }

    fn record(&mut self, cost: CostType, center: CostCenter, amount: Money) {
        self.month.add(cost, center, amount);
        self.year.add(cost, center, amount);
    }

    /// Balance in natural sign: assets positive, liabilities and equity positive.
    pub fn balance(&self, account: Account) -> Money {
        let raw = self.balances[account.slot()];
        if account.is_asset() { raw } else { -raw }
    }

    pub fn cash(&self) -> Money {
        self.balance(Account::Cash)
    }

    /// Debits equal credits; always true unless there is a bug.
    pub fn is_balanced(&self) -> bool {
        self.balances.iter().copied().sum::<Money>() == Money::ZERO
    }

    /// Closes the running month; called on the first day of the next month.
    pub fn close_month(&mut self, next: Date) {
        let closed = std::mem::replace(&mut self.month, PeriodResult::starting(next));
        self.months.push(closed);
        if self.months.len() > MONTHS_KEPT {
            self.months.remove(0);
        }
        if next.month() == 1 {
            let closed = std::mem::replace(&mut self.year, PeriodResult::starting(next));
            self.years.push(closed);
            // The year's result moves into retained earnings.
            let result = self.balances[Account::Result.slot()];
            self.balances[Account::RetainedEarnings.slot()] += result;
            self.balances[Account::Result.slot()] = Money::ZERO;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usd(v: f64) -> Money {
        Money::from_usd(v).unwrap()
    }

    #[test]
    fn bookings_stay_balanced() {
        let start = Date::first_of_year(1900);
        let mut l = Ledger::new(start, usd(1000.0));
        assert_eq!(l.cash(), usd(1000.0));
        assert_eq!(l.balance(Account::Equity), usd(1000.0));
        l.transfer(Account::AssetsUnderConstruction, Account::Cash, usd(300.0));
        l.expense(
            CostType::Personnel,
            CostCenter::default(),
            Account::Cash,
            usd(50.0),
        );
        l.income(
            CostType::Revenue,
            CostCenter::default(),
            Account::Cash,
            usd(80.0),
        );
        assert!(l.is_balanced());
        assert_eq!(l.cash(), usd(730.0));
        assert_eq!(l.balance(Account::Result), usd(30.0));
        assert_eq!(l.month.total(), usd(30.0));
        // Assets = liabilities + equity
        let assets: Money = Account::ALL
            .iter()
            .filter(|a| a.is_asset())
            .map(|&a| l.balance(a))
            .sum();
        let claims: Money = Account::ALL
            .iter()
            .filter(|a| !a.is_asset())
            .map(|&a| l.balance(a))
            .sum();
        assert_eq!(assets, claims);
    }

    #[test]
    fn closing_moves_the_result() {
        let start = Date::first_of_year(1900);
        let mut l = Ledger::new(start, usd(100.0));
        l.income(
            CostType::Revenue,
            CostCenter::default(),
            Account::Cash,
            usd(10.0),
        );
        l.close_month(Date::new(1900, 2, 1).unwrap());
        assert_eq!(l.months.len(), 1);
        assert_eq!(l.month.total(), Money::ZERO);
        assert_eq!(l.year.total(), usd(10.0));
        l.close_month(Date::first_of_year(1901));
        assert_eq!(l.years[0].total(), usd(10.0));
        assert_eq!(l.balance(Account::Result), Money::ZERO);
        assert_eq!(l.balance(Account::RetainedEarnings), usd(10.0));
        assert!(l.is_balanced());
    }
}
