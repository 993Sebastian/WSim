//! Policies (Vorgaben) of a company, inherited from general to specific scopes
//! (docs/ARCHITEKTUR.md §2.2). Stage 1 has the sales channels of Lastenheft §9.2:
//! whether traders and other companies may buy, with minimum price and maximum
//! quantity, per product, per country or for the whole company.

use serde::{Deserialize, Serialize};

use crate::ids::{CountryId, ProductId};
use crate::money::Money;
use crate::state::Company;

/// Who buys from a sale offer, besides consumers and governments.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BuyerGroup {
    /// Traders who export the goods to other countries.
    Traders,
    /// Purchase orders of other companies.
    Companies,
}

/// Where a policy applies; more specific scopes override general ones.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Scope {
    Company,
    Country(CountryId),
    Product(ProductId),
    ProductInCountry(ProductId, CountryId),
}

impl Scope {
    fn matches(self, product: ProductId, country: CountryId) -> bool {
        match self {
            Scope::Company => true,
            Scope::Country(c) => c == country,
            Scope::Product(p) => p == product,
            Scope::ProductInCountry(p, c) => p == product && c == country,
        }
    }

    /// Higher is more specific.
    fn rank(self) -> u8 {
        match self {
            Scope::Company => 0,
            Scope::Country(_) => 1,
            Scope::Product(_) => 2,
            Scope::ProductInCountry(..) => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SalesRule {
    pub allowed: bool,
    /// Sells only at this price or higher.
    pub min_price: Option<Money>,
    /// At most this quantity per sale offer and month.
    pub max_per_month: Option<f64>,
}

impl Default for SalesRule {
    /// Without a policy everyone may buy.
    fn default() -> Self {
        Self {
            allowed: true,
            min_price: None,
            max_per_month: None,
        }
    }
}

impl SalesRule {
    /// How much of `available` a buyer group may take at `price`, given what it bought
    /// this month already.
    pub fn allowance(&self, price: Money, bought_this_month: f64) -> f64 {
        if !self.allowed || self.min_price.is_some_and(|min| price < min) {
            return 0.0;
        }
        self.max_per_month
            .map_or(f64::INFINITY, |max| (max - bought_this_month).max(0.0))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SalesPolicy {
    pub buyer: BuyerGroup,
    pub scope: Scope,
    pub rule: SalesRule,
}

/// The rule that applies to a buyer group for a product sold in a country, and the
/// scope it comes from (`None`: the default).
pub fn sales_rule(
    company: &Company,
    buyer: BuyerGroup,
    product: ProductId,
    country: CountryId,
) -> (SalesRule, Option<Scope>) {
    company
        .sales_policies
        .iter()
        .filter(|p| p.buyer == buyer && p.scope.matches(product, country))
        .max_by_key(|p| p.scope.rank())
        .map_or((SalesRule::default(), None), |p| (p.rule, Some(p.scope)))
}

/// Sets (or with `None` removes) the policy for one buyer group and scope.
pub(crate) fn set(company: &mut Company, buyer: BuyerGroup, scope: Scope, rule: Option<SalesRule>) {
    company
        .sales_policies
        .retain(|p| !(p.buyer == buyer && p.scope == scope));
    if let Some(rule) = rule {
        company
            .sales_policies
            .push(SalesPolicy { buyer, scope, rule });
        company
            .sales_policies
            .sort_by(|a, b| a.buyer.cmp(&b.buyer).then(a.scope.cmp(&b.scope)));
    }
}
