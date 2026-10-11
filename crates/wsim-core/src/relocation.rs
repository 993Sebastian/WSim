//! Moving facilities between sites (docs/FORMELN.md „Anlagen verlegen“): within a company
//! the facility keeps its value and wear; to another company of the group it is sold.

use crate::catalog::Catalog;
use crate::command::CommandError;
use crate::ledger::{Account, CostCenter, CostType};
use crate::money::Money;
use crate::state::{CompanyId, Downtime, GameState, SiteId, Slot};

/// Moves `count` units of a finished facility from `site` to the site `to` of the actor or
/// of a company of its group. Within the company it keeps cost, age and condition; to
/// another company it goes for `price` (the book value when `None`) and starts there like a
/// new facility. Either way it stands for the restart days of the data.
pub(crate) fn move_facility(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    (site, slot, count): (SiteId, usize, u32),
    to: SiteId,
    price: Option<Money>,
) -> Result<(), CommandError> {
    let today = state.date;
    let source = state.site(site).ok_or(CommandError::UnknownSite)?;
    if !source.held_by(actor) {
        return Err(CommandError::NotOwner);
    }
    let sl = source.slots.get(slot).ok_or(CommandError::UnknownSlot)?;
    if sl.ready > today {
        return Err(CommandError::UnderConstruction);
    }
    if count == 0 {
        return Err(CommandError::InvalidQuantity);
    }
    if count > sl.count {
        return Err(CommandError::TooManyUnits { count: sl.count });
    }
    let target = state.site(to).ok_or(CommandError::UnknownSite)?;
    if to == site || target.closed.is_some() {
        return Err(CommandError::NotPossible);
    }
    let seller = source.owner;
    let buyer = target.owner;
    if buyer != seller && !crate::group::same_group(state, seller, buyer) {
        return Err(CommandError::NotOwner);
    }
    let f = catalog.facilities.get(sl.facility);
    if target.kind != f.site_type {
        return Err(CommandError::WrongSiteType {
            required: f.site_type,
        });
    }
    if let Some(plot) = target.plot {
        let needed_ha =
            crate::plots::site_area(catalog, target, Some((sl.facility, sl.size, count)));
        let area_ha = state.plots[plot.index()].area_ha;
        if needed_ha > area_ha + 1e-9 {
            return Err(CommandError::PlotTooSmall { needed_ha, area_ha });
        }
    }
    let model = &catalog.production_model;
    let restart = today.add_days(i32::try_from(model.restart_days).unwrap_or(i32::MAX));
    let (book, _) = crate::production::sale_value(catalog, sl, count, today);
    let sale = buyer != seller;
    let price = price.unwrap_or(book);
    if price.is_negative() {
        return Err(CommandError::InvalidAmount);
    }
    if sale && state.companies[buyer.index()].ledger.cash() < price {
        return Err(CommandError::NotEnoughCash { needed: price });
    }

    // Take the units off the source site.
    let s = &mut state.sites[site.index()];
    let mut moved: Slot = s.slots[slot].clone();
    if count == s.slots[slot].count {
        let open = s.slots.remove(slot).batches;
        crate::production::deliver(catalog, s, open);
    } else {
        let sl = &mut s.slots[slot];
        let cost = sl.share_of_cost(count);
        sl.cost -= cost;
        sl.count -= count;
        moved.cost = cost;
        moved.count = count;
    }
    s.staffing_due = true;
    moved.batches = Vec::new();
    moved.recipe = None;
    moved.utilization = 0.0;
    moved.last_runs = 0.0;
    moved.limit = None;
    moved.operation = crate::state::Operation::Running;
    moved.down = Some(Downtime {
        until: restart,
        overhaul: false,
    });

    if sale {
        // The seller gives up the book value for the price; the buyer builds anew.
        let ledger = &mut state.companies[seller.index()].ledger;
        let center = CostCenter::site(site);
        if book > price {
            ledger.expense(CostType::Other, center, Account::FixedAssets, book - price);
        } else if price > book {
            ledger.income(CostType::Other, center, Account::FixedAssets, price - book);
        }
        ledger.transfer(Account::Cash, Account::FixedAssets, price);
        state.companies[buyer.index()].ledger.transfer(
            Account::AssetsUnderConstruction,
            Account::Cash,
            price,
        );
        moved.cost = price;
        moved.ready = restart;
        moved.down = None;
    }
    let t = &mut state.sites[to.index()];
    t.slots.push(moved);
    t.staffing_due = true;
    Ok(())
}
