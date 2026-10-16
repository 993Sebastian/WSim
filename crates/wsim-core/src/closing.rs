//! Closing a site: its facilities, building, plot and stocks are sold, its people go with
//! a severance pay (docs/FORMELN.md, Standort schließen).

use crate::catalog::Catalog;
use crate::command::CommandError;
use crate::contracts::ContractStatus;
use crate::deals::{DealObject, OfferStatus};
use crate::ledger::{Account, CostCenter, CostType};
use crate::money::Money;
use crate::state::{CompanyId, Consignee, GameState, SiteId};

/// What closing a site brings and costs today.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Closing {
    /// Proceeds of the finished facilities (M22).
    pub facilities: Money,
    pub building: Money,
    /// A bought plot at today's value.
    pub land: Money,
    /// The stocks and goods in production, sold off.
    pub stock: Money,
    /// Book values written off: fixed assets, goodwill, land, stocks.
    pub book: Money,
    pub severance: Money,
}

impl Closing {
    pub fn proceeds(&self) -> Money {
        self.facilities + self.building + self.land + self.stock
    }

    /// Proceeds less book values and severance: the effect on the result.
    pub fn result(&self) -> Money {
        self.proceeds() - self.book - self.severance
    }
}

/// The values of closing a site today.
pub fn value(state: &GameState, catalog: &Catalog, site: SiteId) -> Closing {
    let pm = &catalog.production_model;
    let date = state.date;
    let v = crate::deals::site_value_with(state, catalog, site, Money::ZERO);
    let s = &state.sites[site.index()];
    let in_production: Money = s
        .slots
        .iter()
        .flat_map(|sl| sl.batches.iter())
        .map(|b| b.value)
        .sum();
    let stock = v.inventory + in_production;
    let building_days = pm.building_lifetime_years * 365.0;
    let age = f64::from(s.founded.days_until(date).max(0));
    let building = s
        .building_cost
        .scale((1.0 - age / building_days.max(1.0)).max(0.0));
    let wages = crate::production::daily_wage_bill(state, catalog, site, date);
    Closing {
        facilities: v.liquidation,
        building: building.scale(pm.sale_proceeds_share),
        land: v.land,
        stock: stock.scale(pm.close_stock_share),
        book: v.fixed_assets + v.goodwill + v.land_book + stock,
        severance: Money::from_usd(wages * pm.severance_months * 30.0).unwrap_or(Money::ZERO),
    }
}

/// Why a site cannot be closed now, if it cannot.
pub fn blocker(state: &GameState, catalog: &Catalog, site: SiteId) -> Option<CommandError> {
    let date = state.date;
    let s = &state.sites[site.index()];
    let v = crate::deals::site_value(state, catalog, site);
    if s.slots.iter().any(|sl| sl.ready > date) || v.under_construction > Money::ZERO {
        return Some(CommandError::UnderConstruction);
    }
    if state
        .contracts
        .iter()
        .any(|c| c.status == ContractStatus::Active && (c.seller == site || c.buyer == site))
    {
        return Some(CommandError::SiteHasContracts);
    }
    if state
        .shipments
        .iter()
        .any(|sh| !sh.lost && sh.to == Consignee::Site(site))
    {
        return Some(CommandError::GoodsOnTheWay);
    }
    let c = value(state, catalog, site);
    let cash = state.companies[s.owner.index()].ledger.cash();
    if cash + c.proceeds() < c.severance {
        return Some(CommandError::NotEnoughCash {
            needed: c.severance - c.proceeds(),
        });
    }
    None
}

/// Closes a site of the acting company (checked by the caller).
pub(crate) fn close(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    site: SiteId,
) -> Result<(), CommandError> {
    if let Some(e) = blocker(state, catalog, site) {
        return Err(e);
    }
    let today = state.date;
    let c = value(state, catalog, site);
    let v = crate::deals::site_value_with(state, catalog, site, Money::ZERO);

    let center = CostCenter::site(site);
    let l = &mut state.companies[actor.index()].ledger;
    l.transfer(Account::Cash, Account::FixedAssets, v.fixed_assets);
    l.transfer(Account::Cash, Account::Goodwill, v.goodwill);
    l.transfer(Account::Cash, Account::Land, v.land_book);
    l.transfer(Account::Cash, Account::Inventory, c.book - v.book());
    let proceeds = c.proceeds();
    if proceeds >= c.book {
        l.income(CostType::Other, center, Account::Cash, proceeds - c.book);
    } else {
        l.expense(CostType::Other, center, Account::Cash, c.book - proceeds);
    }
    l.expense(CostType::Personnel, center, Account::Cash, c.severance);

    // Proposed contracts lapse; offers for the site are withdrawn.
    for k in &mut state.contracts {
        if k.status == ContractStatus::Proposed && (k.seller == site || k.buyer == site) {
            k.status = ContractStatus::Void;
            k.closed = Some(today);
        }
    }
    for o in &mut state.offers {
        let concerned = match o.object {
            DealObject::Site(x) => x == site,
            DealObject::Area(_) => o.seller == actor,
            DealObject::License(_) => false,
        };
        if o.status == OfferStatus::Open && concerned {
            o.status = OfferStatus::Withdrawn;
            o.closed = Some(today);
        }
    }
    for other in &mut state.sites {
        other.supplies.retain(|_, x| x.source != site);
    }
    crate::management::release_site(state, site);
    // The plot goes back to the market, the concession to the state.
    crate::ai::give_up_site(state, site);

    let s = &mut state.sites[site.index()];
    s.slots.clear();
    s.inventory.clear();
    for (_, w) in s.workforce.iter_mut() {
        *w = 0.0;
    }
    s.offers.clear();
    s.orders.clear();
    s.supplies.clear();
    s.research = None;
    s.development = None;
    s.development_shares.clear();
    s.staffing_due = false;
    s.building_cost = Money::ZERO;
    s.hall_rent = Money::ZERO;
    s.goodwill = None;
    s.deposit = None;
    s.closed = Some(today);
    Ok(())
}
