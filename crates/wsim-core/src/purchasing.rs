//! Purchasing log of a company (EC1, docs/FORMELN.md): what was bought per site, product
//! and source – quantity, quality and spending per month – as the basis of purchasing
//! controlling and of the question whether making a part itself pays.

use serde::{Deserialize, Serialize};

use crate::ids::{Id, ProductId};
use crate::money::Money;
use crate::state::{CompanyId, GameState, SiteId};

/// Months kept per record.
pub const MONTHS_KEPT: usize = 12;

/// Where goods were bought: on the market (also imports and the state) or under a
/// supply contract. Deliveries between a company's own sites are not purchases.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PurchaseSource {
    Market,
    Contract,
}

/// Goods bought in a month.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PurchaseMonth {
    pub quantity: f64,
    pub spent: Money,
    /// Quantity times quality, for the weighted average.
    pub quality_sum: f64,
}

impl PurchaseMonth {
    pub fn add(&mut self, other: &PurchaseMonth) {
        self.quantity += other.quantity;
        self.spent += other.spent;
        self.quality_sum += other.quality_sum;
    }

    pub fn is_empty(&self) -> bool {
        self.quantity <= 0.0 && self.spent == Money::ZERO
    }
}

/// The purchases of one site of one product from one source.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PurchaseRecord {
    pub site: SiteId,
    pub product: ProductId,
    pub source: PurchaseSource,
    /// The running month.
    pub month: PurchaseMonth,
    /// The last closed months, oldest first (at most `MONTHS_KEPT`).
    pub months: Vec<PurchaseMonth>,
}

fn key(site: SiteId, product: ProductId, source: PurchaseSource) -> (usize, usize, u8) {
    (site.index(), product.index(), source as u8)
}

/// Notes goods a site bought.
pub(crate) fn note(
    state: &mut GameState,
    company: CompanyId,
    (site, product, source): (SiteId, ProductId, PurchaseSource),
    (quantity, spent, quality): (f64, Money, f64),
) {
    if quantity <= 0.0 {
        return;
    }
    let records = &mut state.companies[company.index()].purchases;
    let at = records.binary_search_by_key(&key(site, product, source), |r| {
        key(r.site, r.product, r.source)
    });
    let i = at.unwrap_or_else(|i| {
        records.insert(
            i,
            PurchaseRecord {
                site,
                product,
                source,
                month: PurchaseMonth::default(),
                months: Vec::new(),
            },
        );
        i
    });
    let m = &mut records[i].month;
    m.quantity += quantity;
    m.spent += spent;
    m.quality_sum += quantity * quality;
}

/// At the start of a month: the running month closes; records without purchases for a
/// year are dropped.
pub(crate) fn month_start(state: &mut GameState) {
    for company in &mut state.companies {
        if company.purchases.is_empty() {
            continue;
        }
        for r in &mut company.purchases {
            r.months.push(std::mem::take(&mut r.month));
            if r.months.len() > MONTHS_KEPT {
                r.months.remove(0);
            }
        }
        company
            .purchases
            .retain(|r| r.months.iter().any(|m| !m.is_empty()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::money::Money;

    fn usd(v: f64) -> Money {
        Money::from_usd(v).unwrap()
    }

    #[test]
    fn purchases_sum_per_month_and_close_with_the_month() {
        let mut game = crate::trade_tests::new_game(crate::catalog::test_support::trading());
        let me = game.player();
        let state = game.state_mut();
        let (site, product) = (SiteId(0), ProductId::from_index(0));
        let src = PurchaseSource::Market;
        note(state, me, (site, product, src), (10.0, usd(100.0), 40.0));
        note(state, me, (site, product, src), (30.0, usd(450.0), 60.0));
        note(
            state,
            me,
            (site, product, PurchaseSource::Contract),
            (5.0, usd(40.0), 50.0),
        );
        // A purchase without quantity is not noted.
        note(state, me, (site, product, src), (0.0, usd(1.0), 50.0));
        let records = &state.companies[me.index()].purchases;
        assert_eq!(records.len(), 2, "one per source");
        let market = &records[0];
        assert_eq!(market.source, PurchaseSource::Market);
        assert_eq!(market.month.quantity, 40.0);
        assert_eq!(market.month.spent, usd(550.0));
        assert!((market.month.quality_sum / market.month.quantity - 55.0).abs() < 1e-12);
        month_start(state);
        let market = &state.companies[me.index()].purchases[0];
        assert!(market.month.is_empty());
        assert_eq!(market.months.len(), 1);
        assert_eq!(market.months[0].spent, usd(550.0));
        // After a year without purchases the record is dropped.
        for _ in 0..=MONTHS_KEPT {
            month_start(state);
        }
        assert!(state.companies[me.index()].purchases.is_empty());
    }
}
