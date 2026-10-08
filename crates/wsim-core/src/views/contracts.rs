//! Supply contracts (W4; docs/BEDIENUNG.md, "Markt → Lieferverträge").

use serde::{Deserialize, Serialize};

use super::{iso, usd};
use crate::command::site_type_key;
use crate::contracts::{self, Contract, ContractStatus};
use crate::game::Game;
use crate::ids::ProductId;
use crate::market;
use crate::money::Money;
use crate::state::SiteId;

/// A site of the player and what it could contract.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContractSiteView {
    pub site: u32,
    pub country: String,
    /// Text key of the site type.
    pub kind_text: String,
    /// Products the site makes or offers.
    pub sells: Vec<String>,
    /// Products the site uses or orders.
    pub buys: Vec<String>,
}

/// A contract of the player.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContractView {
    pub id: u32,
    /// The player's side: `verkauf` or `einkauf`.
    pub role: String,
    /// `angeboten`, `laufend`, `beendet`, `gekuendigt`, `abgelehnt`, `verfallen`, `nichtig`.
    pub status: String,
    pub product: String,
    /// Key of the product's unit.
    pub unit: String,
    pub own_site: u32,
    pub own_country: String,
    pub own_kind_text: String,
    pub partner: String,
    pub partner_country: String,
    pub per_month: f64,
    pub price_usd: f64,
    pub months: u32,
    pub min_quality: f64,
    pub penalty: f64,
    pub proposed: String,
    pub start: Option<String>,
    pub end: Option<String>,
    pub closed: Option<String>,
    pub delivered_month: f64,
    pub delivered_total: f64,
    /// Penalties the player paid and received.
    pub penalties_paid_usd: f64,
    pub penalties_received_usd: f64,
    /// The market price in the buyer's country, for comparison.
    pub market_price_usd: f64,
    /// The player must answer this proposal.
    pub answer: bool,
    /// The player may cancel (with `cancel_fee_usd`) or withdraw it.
    pub can_cancel: bool,
    pub cancel_fee_usd: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContractsView {
    /// Without contracts in the data the view is empty.
    pub enabled: bool,
    pub months_max: u32,
    pub months_default: u32,
    pub penalty_max: f64,
    pub penalty_default: f64,
    pub cancel_months: u32,
    /// Open contracts first (proposals to answer at the top), then closed ones.
    pub contracts: Vec<ContractView>,
    pub sites: Vec<ContractSiteView>,
}

/// A possible partner for a contract of an own site.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContractPartnerView {
    pub site: u32,
    pub company: String,
    pub country: String,
    pub kind_text: String,
    /// How much it could take or deliver a month at most (its share for contracts).
    pub free_per_month: f64,
    /// The price it would propose, delivered to the buyer.
    pub suggested_price_usd: f64,
    /// Freight and customs per unit the seller pays.
    pub delivery_cost_usd: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContractPartnersView {
    pub site: u32,
    pub product: String,
    pub unit: String,
    /// The own site sells (`verkauf`) or buys (`einkauf`).
    pub role: String,
    /// What the own site makes (selling) or uses (buying) a month.
    pub own_per_month: f64,
    pub partners: Vec<ContractPartnerView>,
}

fn status_key(status: ContractStatus) -> &'static str {
    match status {
        ContractStatus::Proposed => "angeboten",
        ContractStatus::Active => "laufend",
        ContractStatus::Ended => "beendet",
        ContractStatus::Cancelled => "gekuendigt",
        ContractStatus::Declined => "abgelehnt",
        ContractStatus::Expired => "verfallen",
        ContractStatus::Void => "nichtig",
    }
}

fn unit_key(game: &Game, product: ProductId) -> String {
    let c = game.catalog();
    c.units.key(c.products.get(product).unit).to_owned()
}

fn contract_view(game: &Game, c: &Contract) -> ContractView {
    let (state, catalog) = (game.state(), game.catalog());
    let me = game.player();
    let selling = c.seller_company == me;
    let (own, other) = if selling {
        (c.seller, c.buyer)
    } else {
        (c.buyer, c.seller)
    };
    let (own_site, other_site) = (&state.sites[own.index()], &state.sites[other.index()]);
    let buyer_country = state.sites[c.buyer.index()].country;
    let fee = if c.status == ContractStatus::Active {
        let months = c
            .months_left(state.date)
            .min(catalog.contracts.cancel_months);
        Money::times(c.price, c.per_month * f64::from(months)).scale(c.penalty)
    } else {
        Money::ZERO
    };
    let (paid, received) = if selling {
        (c.paid_by_seller, c.paid_by_buyer)
    } else {
        (c.paid_by_buyer, c.paid_by_seller)
    };
    ContractView {
        id: c.id,
        role: if selling { "verkauf" } else { "einkauf" }.to_owned(),
        status: status_key(c.status).to_owned(),
        product: catalog.products.key(c.product).to_owned(),
        unit: unit_key(game, c.product),
        own_site: own.0,
        own_country: catalog.countries.key(own_site.country).to_owned(),
        own_kind_text: site_type_key(own_site.kind),
        partner: state.companies[c.partner(me).index()].name.clone(),
        partner_country: catalog.countries.key(other_site.country).to_owned(),
        per_month: c.per_month,
        price_usd: usd(c.price),
        months: c.months,
        min_quality: c.min_quality,
        penalty: c.penalty,
        proposed: iso(c.proposed),
        start: c.start.map(iso),
        end: c.end().map(iso),
        closed: c.closed.map(iso),
        delivered_month: c.delivered_month,
        delivered_total: c.delivered_total,
        penalties_paid_usd: usd(paid),
        penalties_received_usd: usd(received),
        market_price_usd: usd(market::market_price(
            catalog,
            state,
            buyer_country,
            c.product,
        )),
        answer: c.status == ContractStatus::Proposed && c.answering() == me,
        can_cancel: c.status == ContractStatus::Active
            || (c.status == ContractStatus::Proposed && c.proposer == me),
        cancel_fee_usd: usd(fee),
    }
}

/// What a site of the player makes or offers, and what it uses or orders.
fn site_products(game: &Game, site: SiteId) -> (Vec<ProductId>, Vec<ProductId>) {
    let (state, catalog) = (game.state(), game.catalog());
    let s = &state.sites[site.index()];
    let mut sells: Vec<ProductId> = s
        .offers
        .keys()
        .copied()
        .chain(
            catalog
                .products
                .ids()
                .filter(|&p| contracts::daily_output(state, catalog, site, p) > 0.0),
        )
        .filter(|&p| catalog.products.get(p).state_market.is_none())
        .collect();
    sells.sort();
    sells.dedup();
    let mut buys: Vec<ProductId> = s
        .orders
        .iter()
        .filter(|(_, o)| o.target > 0.0)
        .map(|(&p, _)| p)
        .chain(
            catalog
                .products
                .ids()
                .filter(|&p| contracts::daily_need(state, catalog, site, p) > 0.0),
        )
        .filter(|&p| catalog.products.get(p).state_market.is_none())
        .collect();
    buys.sort();
    buys.dedup();
    (sells, buys)
}

pub fn contracts(game: &Game) -> ContractsView {
    let (state, catalog) = (game.state(), game.catalog());
    let m = &catalog.contracts;
    let me = game.player();
    let mut list: Vec<&Contract> = state
        .contracts
        .iter()
        .filter(|c| c.seller_company == me || c.buyer_company == me)
        .collect();
    // Proposals to answer first, then running ones, then closed ones, newest first.
    let rank = |c: &Contract| match c.status {
        ContractStatus::Proposed if c.answering() == me => 0,
        ContractStatus::Proposed => 1,
        ContractStatus::Active => 2,
        _ => 3,
    };
    list.sort_by(|a, b| rank(a).cmp(&rank(b)).then(b.id.cmp(&a.id)));
    let sites = if m.enabled() {
        state
            .sites
            .iter()
            .enumerate()
            .filter(|(_, s)| s.owner == me)
            .map(|(i, s)| {
                let site = SiteId(u32::try_from(i).expect("site count fits u32"));
                let (sells, buys) = site_products(game, site);
                let keys = |v: Vec<ProductId>| {
                    v.into_iter()
                        .map(|p| catalog.products.key(p).to_owned())
                        .collect()
                };
                ContractSiteView {
                    site: site.0,
                    country: catalog.countries.key(s.country).to_owned(),
                    kind_text: site_type_key(s.kind),
                    sells: keys(sells),
                    buys: keys(buys),
                }
            })
            .filter(|v| !v.sells.is_empty() || !v.buys.is_empty())
            .collect()
    } else {
        Vec::new()
    };
    ContractsView {
        enabled: m.enabled(),
        months_max: m.months_max,
        months_default: m.months_default,
        penalty_max: m.penalty_max,
        penalty_default: m.penalty_default,
        cancel_months: m.cancel_months,
        contracts: list.into_iter().map(|c| contract_view(game, c)).collect(),
        sites,
    }
}

/// Partners for a contract of an own site: buyers when it sells the product, sellers
/// when it uses it; `None` for a site of another company or an unknown product.
pub fn contract_partners(game: &Game, site: u32, product: &str) -> Option<ContractPartnersView> {
    let (state, catalog) = (game.state(), game.catalog());
    let me = game.player();
    let own = SiteId(site);
    state.site(own).filter(|s| s.owner == me)?;
    let p = catalog.products.id(product)?;
    let (sells, _) = site_products(game, own);
    let selling = sells.contains(&p);
    let share = catalog.contracts.ai.share;
    let own_per_month = 30.0
        * if selling {
            contracts::daily_output(state, catalog, own, p)
        } else {
            contracts::daily_need(state, catalog, own, p)
        };
    let mut partners: Vec<ContractPartnerView> = Vec::new();
    for (i, other) in state.sites.iter().enumerate() {
        let site = SiteId(u32::try_from(i).expect("site count fits u32"));
        let company = &state.companies[other.owner.index()];
        if crate::group::same_group(state, other.owner, me) || company.bankrupt {
            continue;
        }
        let (free, pair) = if selling {
            (
                share * 30.0 * contracts::daily_need(state, catalog, site, p)
                    - contracts::contracted(state, site, p, false),
                (own, site),
            )
        } else {
            if !other.offers.contains_key(&p) {
                continue;
            }
            (
                share * 30.0 * contracts::daily_output(state, catalog, site, p)
                    - contracts::contracted(state, site, p, true),
                (site, own),
            )
        };
        if free <= 1e-9 {
            continue;
        }
        let buyer_country = state.sites[pair.1.index()].country;
        let market_price = market::market_price(catalog, state, buyer_country, p);
        let Some((cost, _)) = contracts::delivery_cost(state, catalog, pair, p, market_price)
        else {
            continue;
        };
        let suggested = if selling {
            market_price
        } else {
            let Some(ask) = contracts::ask_price(state, catalog, pair, p) else {
                continue;
            };
            market_price.max(ask)
        };
        partners.push(ContractPartnerView {
            site: site.0,
            company: company.name.clone(),
            country: catalog.countries.key(other.country).to_owned(),
            kind_text: site_type_key(other.kind),
            free_per_month: free,
            suggested_price_usd: usd(suggested),
            delivery_cost_usd: usd(cost),
        });
    }
    partners.sort_by(|a, b| {
        b.free_per_month
            .total_cmp(&a.free_per_month)
            .then(a.site.cmp(&b.site))
    });
    partners.truncate(20);
    Some(ContractPartnersView {
        site,
        product: product.to_owned(),
        unit: unit_key(game, p),
        role: if selling { "verkauf" } else { "einkauf" }.to_owned(),
        own_per_month,
        partners,
    })
}
