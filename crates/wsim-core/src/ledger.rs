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
    /// Goodwill of bought sites (M30); an asset, last so that older saves keep their
    /// balances in place.
    Goodwill,
    /// Bought plots of land (M35); not written off.
    Land,
    /// Stakes in start-ups and pledges to their rounds (SU2), at cost.
    Participations,
}

impl Account {
    pub const ALL: [Account; 11] = [
        Account::Cash,
        Account::Inventory,
        Account::FixedAssets,
        Account::AssetsUnderConstruction,
        Account::Loans,
        Account::Equity,
        Account::RetainedEarnings,
        Account::Result,
        Account::Goodwill,
        Account::Land,
        Account::Participations,
    ];

    pub fn is_asset(self) -> bool {
        matches!(
            self,
            Account::Cash
                | Account::Inventory
                | Account::FixedAssets
                | Account::AssetsUnderConstruction
                | Account::Goodwill
                | Account::Land
                | Account::Participations
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
    /// Administration, sales and logistics of the production (M16).
    Overhead,
    /// Land rent and royalties of extraction (M16).
    Rent,
    /// Licence fees paid and received (M30).
    Licenses,
    Other,
    /// Gains and losses of stakes in start-ups (SU2).
    Investments,
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
    #[serde(default)]
    pub cash_flow: CashFlow,
    /// Result by cost center and cost type (M18: results of a site and of its products).
    /// Kept for the running and the last closed periods only, to keep saves small.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub by_center: BTreeMap<CostCenter, BTreeMap<CostType, Money>>,
    /// Revenue per site (MA2: the budgets of its positions).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub site_revenue: BTreeMap<SiteId, Money>,
    /// Revenue per product (MA5: the strategy review).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub product_revenue: BTreeMap<ProductId, Money>,
}

/// Change of cash in a period by activity (Kapitalflussrechnung).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CashFlow {
    pub operating: Money,
    pub investing: Money,
    pub financing: Money,
}

impl CashFlow {
    pub fn total(&self) -> Money {
        self.operating + self.investing + self.financing
    }

    fn add(&mut self, counter_account: Account, amount: Money) {
        match counter_account {
            Account::FixedAssets
            | Account::AssetsUnderConstruction
            | Account::Goodwill
            | Account::Land
            | Account::Participations => self.investing += amount,
            Account::Loans | Account::Equity | Account::RetainedEarnings => {
                self.financing += amount
            }
            Account::Cash | Account::Inventory | Account::Result => self.operating += amount,
        }
    }
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
            if cost == CostType::Revenue {
                *self.site_revenue.entry(site).or_default() += amount;
            }
        }
        if let Some(product) = center.product {
            *self.by_product.entry(product).or_default() += amount;
            if cost == CostType::Revenue {
                *self.product_revenue.entry(product).or_default() += amount;
            }
        }
        *self
            .by_center
            .entry(center)
            .or_default()
            .entry(cost)
            .or_default() += amount;
    }

    /// Sum of a cost type over the centers of a site (with its products).
    pub fn site_type(&self, site: SiteId, cost: CostType) -> Money {
        self.by_center
            .iter()
            .filter(|(c, _)| c.site == Some(site))
            .filter_map(|(_, t)| t.get(&cost))
            .copied()
            .sum()
    }

    /// Sum of a cost type over the centers of a product (all sites).
    pub fn product_type(&self, product: ProductId, cost: CostType) -> Money {
        self.by_center
            .iter()
            .filter(|(c, _)| c.product == Some(product))
            .filter_map(|(_, t)| t.get(&cost))
            .copied()
            .sum()
    }

    pub fn total(&self) -> Money {
        self.by_type.values().copied().sum()
    }
}

/// How many closed months are kept.
pub const MONTHS_KEPT: usize = 24;

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
        let cash_change = match (debit, credit) {
            (Account::Cash, Account::Cash) => None,
            (Account::Cash, other) => Some((other, amount)),
            (other, Account::Cash) => Some((other, -amount)),
            _ => None,
        };
        if let Some((other, change)) = cash_change {
            self.month.cash_flow.add(other, change);
            self.year.cash_flow.add(other, change);
        }
    }

    /// Sum of all assets (= sum of liabilities and equity).
    pub fn total_assets(&self) -> Money {
        Account::ALL
            .iter()
            .filter(|a| a.is_asset())
            .map(|&a| self.balance(a))
            .sum()
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

    /// Moves a cost booked on `from` to `to` (internal allocation, e.g. the wages of the
    /// hours a product used from the site's wage bill). Balances and totals by cost type
    /// stay as they are.
    pub fn allocate(&mut self, cost: CostType, from: CostCenter, to: CostCenter, amount: Money) {
        self.record(cost, from, amount);
        self.record(cost, to, -amount);
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
    /// Accounts added in later versions start at zero in older saves.
    pub(crate) fn fit_accounts(&mut self) {
        self.balances.resize(Account::ALL.len(), Money::ZERO);
    }

    pub fn close_month(&mut self, next: Date) {
        let closed = std::mem::replace(&mut self.month, PeriodResult::starting(next));
        if let Some(last) = self.months.last_mut() {
            last.by_center.clear();
        }
        self.months.push(closed);
        if self.months.len() > MONTHS_KEPT {
            self.months.remove(0);
        }
        if next.month() == 1 {
            let closed = std::mem::replace(&mut self.year, PeriodResult::starting(next));
            if let Some(last) = self.years.last_mut() {
                last.by_center.clear();
            }
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
    use crate::ids::Id;

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

    #[test]
    fn allocation_moves_costs_between_centers_only() {
        let start = Date::first_of_year(1900);
        let mut l = Ledger::new(start, usd(100.0));
        let site = SiteId(2);
        let product = ProductId::from_index(0);
        l.expense(
            CostType::Personnel,
            CostCenter::site(site),
            Account::Cash,
            usd(30.0),
        );
        l.allocate(
            CostType::Personnel,
            CostCenter::site(site),
            CostCenter::product(site, product),
            usd(20.0),
        );
        assert!(l.is_balanced());
        assert_eq!(l.month.by_type[&CostType::Personnel], usd(-30.0));
        assert_eq!(l.month.by_site[&site], usd(-30.0));
        assert_eq!(l.month.by_product[&product], usd(-20.0));
        assert_eq!(l.month.site_type(site, CostType::Personnel), usd(-30.0));
        let at = |c: CostCenter| l.month.by_center[&c][&CostType::Personnel];
        assert_eq!(at(CostCenter::site(site)), usd(-10.0));
        assert_eq!(at(CostCenter::product(site, product)), usd(-20.0));
        // Only the last closed months keep the details.
        l.close_month(Date::new(1900, 2, 1).unwrap());
        l.close_month(Date::new(1900, 3, 1).unwrap());
        assert!(l.months[0].by_center.is_empty());
        assert_eq!(l.months[0].by_type[&CostType::Personnel], usd(-30.0));
    }
}
