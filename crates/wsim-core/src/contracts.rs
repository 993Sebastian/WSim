//! Supply contracts between companies (W4; formulas in docs/FORMELN.md).
//!
//! A contract ties a seller's site to a buyer's site for one product: a quantity a month at
//! a fixed price delivered to the buyer, for some months, with a minimum quality and a
//! penalty on missing quantities. Deliveries run every day after production and before the
//! markets, so contracted goods come first.

use serde::{Deserialize, Serialize};

use crate::calendar::{Date, days_in_month};
use crate::catalog::Catalog;
use crate::command::CommandError;
use crate::ids::ProductId;
use crate::ledger::{Account, CostCenter, CostType};
use crate::market;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{CompanyId, Consignee, GameState, Shipment, SiteId};

/// Quantities below this count as nothing.
const EPS: f64 = 1e-9;

fn site_id(index: usize) -> SiteId {
    SiteId(u32::try_from(index).expect("site count fits u32"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractStatus {
    /// Waiting for the answer of the side that did not propose.
    Proposed,
    Active,
    /// Ran its months.
    Ended,
    /// Ended early by one side (`Contract::cancelled_by`) or withdrawn before the answer.
    Cancelled,
    Declined,
    /// Not answered in time.
    Expired,
    /// A site changed hands or its company failed.
    Void,
}

/// Why the month's quantity fell short: the seller lacked goods or the buyer cash.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Shortfall {
    Goods,
    Cash,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Contract {
    pub id: u32,
    pub seller: SiteId,
    pub buyer: SiteId,
    /// The owners when proposed; the contract is void when a site changes hands.
    pub seller_company: CompanyId,
    pub buyer_company: CompanyId,
    pub product: ProductId,
    pub per_month: f64,
    /// Per unit, delivered to the buyer's site.
    pub price: Money,
    pub months: u32,
    pub min_quality: f64,
    /// Share of the value of a missing quantity.
    pub penalty: f64,
    pub proposer: CompanyId,
    pub status: ContractStatus,
    pub proposed: Date,
    /// First day of deliveries, a first of a month.
    #[serde(default)]
    pub start: Option<Date>,
    #[serde(default)]
    pub closed: Option<Date>,
    #[serde(default)]
    pub cancelled_by: Option<CompanyId>,
    #[serde(default)]
    pub delivered_month: f64,
    #[serde(default)]
    pub short_of: Option<Shortfall>,
    #[serde(default)]
    pub delivered_total: f64,
    /// Penalties paid by the seller and by the buyer so far.
    #[serde(default)]
    pub paid_by_seller: Money,
    #[serde(default)]
    pub paid_by_buyer: Money,
}

impl Contract {
    /// The other side of `company`.
    pub fn partner(&self, company: CompanyId) -> CompanyId {
        if company == self.seller_company {
            self.buyer_company
        } else {
            self.seller_company
        }
    }

    /// The side that answers a proposal.
    pub fn answering(&self) -> CompanyId {
        self.partner(self.proposer)
    }

    pub fn open(&self) -> bool {
        matches!(
            self.status,
            ContractStatus::Proposed | ContractStatus::Active
        )
    }

    /// Last day of deliveries is the day before this.
    pub fn end(&self) -> Option<Date> {
        self.start.map(|s| s.add_months(self.months))
    }

    /// Months left from `today`, the current one counted.
    pub fn months_left(&self, today: Date) -> u32 {
        let Some(end) = self.end() else {
            return self.months;
        };
        let mut months = 0;
        let mut month = today.first_of_month().max(self.start.unwrap_or(end));
        while month < end {
            months += 1;
            month = month.add_months(1);
        }
        months
    }
}

/// The terms of a contract as proposed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Terms {
    pub seller: SiteId,
    pub buyer: SiteId,
    pub product: ProductId,
    pub per_month: f64,
    pub price: Money,
    pub months: u32,
    pub min_quality: f64,
    pub penalty: f64,
}

/// Why an AI company declines a proposal (text `vertrag.abgelehnt.<key>`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decline {
    Price,
    Quantity,
    Quality,
    Penalty,
}

impl Decline {
    pub fn key(self) -> &'static str {
        match self {
            Decline::Price => "preis",
            Decline::Quantity => "menge",
            Decline::Quality => "qualitaet",
            Decline::Penalty => "strafe",
        }
    }
}

/// Daily output of a product at a site at full load of its running units.
pub fn daily_output(state: &GameState, catalog: &Catalog, site: SiteId, product: ProductId) -> f64 {
    let date = state.date;
    state.sites[site.index()]
        .slots
        .iter()
        .filter(|sl| sl.operating(date))
        .filter_map(|sl| sl.recipe.map(|r| (sl, catalog.recipes.get(r))))
        .filter(|(_, r)| r.product == product)
        .map(|(sl, r)| sl.full_runs(catalog) * r.output)
        .sum()
}

/// Daily use of a product at a site at the planned load of its running units.
pub fn daily_need(state: &GameState, catalog: &Catalog, site: SiteId, product: ProductId) -> f64 {
    let date = state.date;
    state.sites[site.index()]
        .slots
        .iter()
        .filter(|sl| sl.operating(date))
        .filter_map(|sl| sl.recipe.map(|r| (sl, catalog.recipes.get(r))))
        .map(|(sl, r)| {
            let runs = sl.full_runs(catalog) * sl.utilization;
            r.inputs
                .iter()
                .filter(|&&(i, _)| i == product)
                .map(|&(_, q)| runs * q)
                .sum::<f64>()
        })
        .sum()
}

/// Monthly quantity of the open contracts of a site for a product, as seller or buyer.
pub fn contracted(state: &GameState, site: SiteId, product: ProductId, selling: bool) -> f64 {
    state
        .contracts
        .iter()
        .filter(|c| c.open() && c.product == product)
        .filter(|c| if selling { c.seller } else { c.buyer } == site)
        .map(|c| c.per_month)
        .sum()
}

/// Freight and customs per unit from the seller's site to the buyer's: `None` without a
/// route or under an embargo.
pub fn delivery_cost(
    state: &GameState,
    catalog: &Catalog,
    (seller, buyer): (SiteId, SiteId),
    product: ProductId,
    price: Money,
) -> Option<(Money, u32)> {
    let (a, b) = (&state.sites[seller.index()], &state.sites[buyer.index()]);
    let tariff = state
        .tariffs
        .for_product(catalog, a.country, b.country, product)?;
    if a.country == b.country {
        return Some((Money::ZERO, 0));
    }
    let sea = crate::plots::sea_freight(catalog, state, a)
        .min(crate::plots::sea_freight(catalog, state, b));
    let (freight, days) =
        state
            .routes
            .for_product_via(catalog, product, (a.country, b.country), sea)?;
    Some((freight + price.scale(tariff), days))
}

/// The delivered price at which a seller gets its own price after freight and customs.
pub fn ask_price(
    state: &GameState,
    catalog: &Catalog,
    (seller, buyer): (SiteId, SiteId),
    product: ProductId,
) -> Option<Money> {
    let (freight, _) = delivery_cost(state, catalog, (seller, buyer), product, Money::ZERO)?;
    let (a, b) = (&state.sites[seller.index()], &state.sites[buyer.index()]);
    let tariff = state
        .tariffs
        .for_product(catalog, a.country, b.country, product)?;
    let net = seller_price(state, catalog, seller, product) + freight;
    Some(net.scale(1.0 / (1.0 - tariff).max(0.01)))
}

/// The price an AI seller asks at its site: its offer, else the market price there.
fn seller_price(state: &GameState, catalog: &Catalog, site: SiteId, product: ProductId) -> Money {
    let s = &state.sites[site.index()];
    s.offers.get(&product).map_or_else(
        || market::market_price(catalog, state, s.country, product),
        |o| o.price,
    )
}

/// Quality of the goods a site has or makes: its stock, else the middle.
fn quality_at(state: &GameState, site: SiteId, product: ProductId) -> f64 {
    state.sites[site.index()]
        .inventory
        .get(&product)
        .filter(|s| s.quantity > EPS)
        .map_or(50.0, |s| s.quality)
}

/// Whether an AI company takes a proposal (docs/FORMELN.md, W4).
pub fn ai_answer(
    state: &GameState,
    catalog: &Catalog,
    t: &Terms,
    ai: CompanyId,
) -> Result<(), Decline> {
    let m = &catalog.contracts.ai;
    if t.penalty > m.penalty_max + EPS {
        return Err(Decline::Penalty);
    }
    let selling = state.sites[t.seller.index()].owner == ai;
    if selling {
        let (cost, _) = delivery_cost(state, catalog, (t.seller, t.buyer), t.product, t.price)
            .ok_or(Decline::Price)?;
        let own = seller_price(state, catalog, t.seller, t.product);
        if (t.price - cost) < own.scale(1.0 - m.sale_discount) {
            return Err(Decline::Price);
        }
        let free = m.share * 30.0 * daily_output(state, catalog, t.seller, t.product)
            - contracted(state, t.seller, t.product, true);
        if t.per_month > free + EPS {
            return Err(Decline::Quantity);
        }
        if quality_at(state, t.seller, t.product) + EPS < t.min_quality {
            return Err(Decline::Quality);
        }
    } else {
        let country = state.sites[t.buyer.index()].country;
        let market = market::market_price(catalog, state, country, t.product);
        if t.price > market.scale(1.0 + m.purchase_premium) {
            return Err(Decline::Price);
        }
        let free = m.share * 30.0 * daily_need(state, catalog, t.buyer, t.product)
            - contracted(state, t.buyer, t.product, false);
        if t.per_month > free + EPS {
            return Err(Decline::Quantity);
        }
    }
    Ok(())
}

/// Checks and records a proposal of `actor`; an AI partner answers at once.
pub(crate) fn propose(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    t: &Terms,
) -> Result<(), CommandError> {
    let m = &catalog.contracts;
    if !m.enabled() {
        return Err(CommandError::NoContracts);
    }
    let (Some(seller), Some(buyer)) = (state.site(t.seller), state.site(t.buyer)) else {
        return Err(CommandError::UnknownSite);
    };
    let (sc, bc) = (seller.owner, buyer.owner);
    if sc != actor && bc != actor {
        return Err(CommandError::NotOwner);
    }
    if sc == bc {
        return Err(CommandError::ContractWithItself);
    }
    if crate::group::same_group(state, sc, bc) {
        return Err(CommandError::WithinGroup);
    }
    if !(t.per_month.is_finite() && t.per_month > 0.0) {
        return Err(CommandError::InvalidQuantity);
    }
    if t.price <= Money::ZERO {
        return Err(CommandError::InvalidAmount);
    }
    if t.months == 0 || t.months > m.months_max {
        return Err(CommandError::InvalidTerm { max: m.months_max });
    }
    if !(0.0..=m.penalty_max).contains(&t.penalty) || !(0.0..=100.0).contains(&t.min_quality) {
        return Err(CommandError::InvalidShare);
    }
    let partner = if sc == actor { bc } else { sc };
    if state.companies[partner.index()].bankrupt {
        return Err(CommandError::UnknownCompany(partner));
    }
    if delivery_cost(state, catalog, (t.seller, t.buyer), t.product, t.price).is_none() {
        let (a, b) = (seller.country, buyer.country);
        return Err(
            if state
                .tariffs
                .for_product(catalog, a, b, t.product)
                .is_none()
            {
                CommandError::Embargo {
                    from: catalog.countries.key(a).to_owned(),
                    to: catalog.countries.key(b).to_owned(),
                }
            } else {
                CommandError::NoRoute {
                    product: catalog.products.key(t.product).to_owned(),
                    from: catalog.countries.key(a).to_owned(),
                    to: catalog.countries.key(b).to_owned(),
                }
            },
        );
    }
    let today = state.date;
    // Every company but the player answers by its rules.
    let ai_partner = !state.is_main(partner);
    if ai_partner {
        ai_answer(state, catalog, t, partner).map_err(|d| CommandError::ContractDeclined {
            reason: d.key().to_owned(),
        })?;
    }
    let id = state.next_contract;
    state.next_contract += 1;
    state.contracts.push(Contract {
        id,
        seller: t.seller,
        buyer: t.buyer,
        seller_company: sc,
        buyer_company: bc,
        product: t.product,
        per_month: t.per_month,
        price: t.price,
        months: t.months,
        min_quality: t.min_quality,
        penalty: t.penalty,
        proposer: actor,
        status: if ai_partner {
            ContractStatus::Active
        } else {
            ContractStatus::Proposed
        },
        proposed: today,
        start: ai_partner.then(|| today.first_of_next_month()),
        closed: None,
        cancelled_by: None,
        delivered_month: 0.0,
        short_of: None,
        delivered_total: 0.0,
        paid_by_seller: Money::ZERO,
        paid_by_buyer: Money::ZERO,
    });
    Ok(())
}

fn find(state: &GameState, id: u32) -> Result<usize, CommandError> {
    state
        .contracts
        .iter()
        .position(|c| c.id == id)
        .ok_or(CommandError::UnknownContract)
}

/// The answer to a proposal by the side that did not propose.
pub(crate) fn answer(
    state: &mut GameState,
    actor: CompanyId,
    id: u32,
    accept: bool,
) -> Result<(), CommandError> {
    let i = find(state, id)?;
    let today = state.date;
    let c = &mut state.contracts[i];
    if c.status != ContractStatus::Proposed || c.answering() != actor {
        return Err(CommandError::UnknownContract);
    }
    if accept {
        c.status = ContractStatus::Active;
        c.start = Some(today.first_of_next_month());
    } else {
        c.status = ContractStatus::Declined;
        c.closed = Some(today);
    }
    Ok(())
}

/// Ends a contract early: the side that cancels pays the penalty on up to
/// `kuendigung_monate` monthly quantities; a proposal is withdrawn for free.
pub(crate) fn cancel(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    id: u32,
) -> Result<(), CommandError> {
    let i = find(state, id)?;
    let today = state.date;
    let c = &state.contracts[i];
    if !c.open() || (actor != c.seller_company && actor != c.buyer_company) {
        return Err(CommandError::UnknownContract);
    }
    if c.status == ContractStatus::Proposed && c.proposer != actor {
        return Err(CommandError::UnknownContract);
    }
    let fee = if c.status == ContractStatus::Active {
        let months = c.months_left(today).min(catalog.contracts.cancel_months);
        Money::times(c.price, c.per_month * f64::from(months)).scale(c.penalty)
    } else {
        Money::ZERO
    };
    let partner = c.partner(actor);
    let center = CostCenter::product(
        if actor == c.seller_company {
            c.seller
        } else {
            c.buyer
        },
        c.product,
    );
    if fee > Money::ZERO {
        let ledger = &mut state.companies[actor.index()].ledger;
        if ledger.cash() < fee {
            return Err(CommandError::NotEnoughCash { needed: fee });
        }
        ledger.expense(CostType::Other, center, Account::Cash, fee);
        state.companies[partner.index()].ledger.income(
            CostType::Other,
            CostCenter::default(),
            Account::Cash,
            fee,
        );
    }
    let c = &mut state.contracts[i];
    if actor == c.seller_company {
        c.paid_by_seller += fee;
    } else {
        c.paid_by_buyer += fee;
    }
    c.status = ContractStatus::Cancelled;
    c.cancelled_by = Some(actor);
    c.closed = Some(today);
    Ok(())
}

/// Today's deliveries of all running contracts; also voids contracts whose sites changed
/// hands and lets proposals expire.
pub(crate) fn deliver(state: &mut GameState, catalog: &Catalog, today: Date) -> Vec<Message> {
    let mut news = Vec::new();
    if state.contracts.is_empty() {
        return news;
    }
    let player = state.main_company;
    let days = f64::from(days_in_month(today.year(), today.month()));
    let expire = i32::try_from(catalog.contracts.proposal_days).unwrap_or(i32::MAX);
    for i in 0..state.contracts.len() {
        let c = &state.contracts[i];
        if !c.open() {
            continue;
        }
        let (seller, buyer) = (
            &state.sites[c.seller.index()],
            &state.sites[c.buyer.index()],
        );
        let failed = |company: CompanyId| state.companies[company.index()].bankrupt;
        if seller.owner != c.seller_company
            || buyer.owner != c.buyer_company
            || failed(c.seller_company)
            || failed(c.buyer_company)
        {
            let c = &mut state.contracts[i];
            c.status = ContractStatus::Void;
            c.closed = Some(today);
            continue;
        }
        if c.status == ContractStatus::Proposed {
            if c.proposed.add_days(expire) <= today {
                let involves_player =
                    Some(c.seller_company) == player || Some(c.buyer_company) == player;
                let product = c.product;
                let c = &mut state.contracts[i];
                c.status = ContractStatus::Expired;
                c.closed = Some(today);
                if involves_player {
                    news.push(
                        Message::new(MessageKind::Info, keys::CONTRACT_EXPIRED).with(
                            "produkt",
                            Param::TextKey(format!("produkt.{}", catalog.products.key(product))),
                        ),
                    );
                }
            }
            continue;
        }
        if c.start.is_none_or(|s| s > today) {
            continue;
        }
        let due = c.per_month * f64::from(today.day()) / days - c.delivered_month;
        if due <= EPS {
            continue;
        }
        let (product, price) = (c.product, c.price);
        let stock = seller
            .inventory
            .get(&product)
            .filter(|s| s.quality + EPS >= c.min_quality)
            .map_or(0.0, |s| s.quantity);
        let cash = state.companies[c.buyer_company.index()].ledger.cash();
        let affordable = (cash.to_usd() / price.to_usd().max(EPS)).max(0.0);
        let quantity = due.min(stock).min(affordable);
        let short = if stock + EPS < due {
            Some(Shortfall::Goods)
        } else if affordable + EPS < due {
            Some(Shortfall::Cash)
        } else {
            None
        };
        if short.is_some() {
            state.contracts[i].short_of = short;
        }
        if quantity > EPS && ship(state, catalog, i, quantity, today) {
            let c = &state.contracts[i];
            if Some(c.seller_company) == player {
                news.push(crate::logistics::loss_message(
                    catalog,
                    (product, quantity),
                    (
                        state.sites[c.seller.index()].country,
                        state.sites[c.buyer.index()].country,
                    ),
                ));
            }
        }
    }
    news
}

/// Moves `quantity` of contract `i` to the buyer with payment, freight and customs;
/// `true` when the load is lost on the way (W5).
fn ship(state: &mut GameState, catalog: &Catalog, i: usize, quantity: f64, today: Date) -> bool {
    let c = state.contracts[i].clone();
    let amount = Money::times(c.price, quantity);
    let (cost, days) = delivery_cost(state, catalog, (c.seller, c.buyer), c.product, c.price)
        .unwrap_or((Money::ZERO, 0));
    let (from, to) = (
        state.sites[c.seller.index()].country,
        state.sites[c.buyer.index()].country,
    );
    let stock = state.sites[c.seller.index()]
        .inventory
        .entry(c.product)
        .or_default();
    let quality = stock.quality;
    let value = stock.take(quantity);
    let center = CostCenter::product(c.seller, c.product);
    let tariff = state
        .tariffs
        .for_product(catalog, from, to, c.product)
        .unwrap_or(0.0);
    let customs = amount.scale(tariff);
    let market = Money::times(cost, quantity) - customs;
    // The seller's way (W5); a lost load costs it the goods and the freight.
    let plan = (from != to).then(|| {
        crate::logistics::plan(
            state,
            catalog,
            c.seller_company,
            (c.product, quantity),
            (from, to),
            market,
        )
    });
    let freight = plan.as_ref().map_or(market, |p| p.cost);
    if plan.is_some_and(|p| crate::logistics::book(state, catalog, c.seller_company, &p)) {
        let ledger = &mut state.companies[c.seller_company.index()].ledger;
        ledger.expense(CostType::Other, center, Account::Inventory, value);
        if freight > Money::ZERO {
            ledger.expense(CostType::Transport, center, Account::Cash, freight);
        }
        crate::logistics::note_loss(state, c.seller_company, value);
        return true;
    }
    let ledger = &mut state.companies[c.seller_company.index()].ledger;
    ledger.income(CostType::Revenue, center, Account::Cash, amount);
    ledger.expense(CostType::InventoryChange, center, Account::Inventory, value);
    if freight > Money::ZERO {
        ledger.expense(CostType::Transport, center, Account::Cash, freight);
    }
    if customs > Money::ZERO {
        ledger.expense(CostType::Customs, center, Account::Cash, customs);
    }
    state.companies[c.buyer_company.index()].ledger.transfer(
        Account::Inventory,
        Account::Cash,
        amount,
    );
    if from == to {
        let s = &mut state.sites[c.buyer.index()];
        s.inventory
            .entry(c.product)
            .or_default()
            .add(quantity, amount, quality);
        if let Some(order) = s.orders.get_mut(&c.product) {
            order.bought_month += quantity;
        }
    } else {
        state.shipments.push(Shipment {
            product: c.product,
            quantity,
            quality,
            value: amount,
            from,
            to: Consignee::Site(c.buyer),
            arrival: today.add_days(i32::try_from(days.max(1)).unwrap_or(i32::MAX)),
            lost: false,
        });
    }
    let c = &mut state.contracts[i];
    c.delivered_month += quantity;
    c.delivered_total += quantity;
    false
}

/// At the start of a month: penalties for the month before, ends of contracts, the list
/// cleared of old ones.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, today: Date) -> Vec<Message> {
    let mut news = Vec::new();
    let player = state.main_company;
    for i in 0..state.contracts.len() {
        let c = state.contracts[i].clone();
        if c.status != ContractStatus::Active || c.start.is_none_or(|s| s >= today) {
            continue;
        }
        let missing = (c.per_month - c.delivered_month).max(0.0);
        let product = Param::TextKey(format!("produkt.{}", catalog.products.key(c.product)));
        if missing > c.per_month * 1e-6 {
            let fee = Money::times(c.price, missing).scale(c.penalty);
            let seller_fault = c.short_of != Some(Shortfall::Cash);
            let (payer, payee) = if seller_fault {
                (c.seller_company, c.buyer_company)
            } else {
                (c.buyer_company, c.seller_company)
            };
            if fee > Money::ZERO {
                let site = if seller_fault { c.seller } else { c.buyer };
                state.companies[payer.index()].ledger.expense(
                    CostType::Other,
                    CostCenter::product(site, c.product),
                    Account::Cash,
                    fee,
                );
                state.companies[payee.index()].ledger.income(
                    CostType::Other,
                    CostCenter::default(),
                    Account::Cash,
                    fee,
                );
            }
            let cs = &mut state.contracts[i];
            if seller_fault {
                cs.paid_by_seller += fee;
            } else {
                cs.paid_by_buyer += fee;
            }
            if Some(payer) == player || Some(payee) == player {
                let key = if Some(payer) == player {
                    keys::CONTRACT_SHORT_OWN
                } else {
                    keys::CONTRACT_SHORT_PARTNER
                };
                news.push(
                    Message::new(MessageKind::Warning, key)
                        .with("produkt", product.clone())
                        .with("menge", Param::Number(missing))
                        .with("strafe", Param::Money(fee)),
                );
            }
        }
        let cs = &mut state.contracts[i];
        cs.delivered_month = 0.0;
        cs.short_of = None;
        if cs.end().is_some_and(|end| end <= today) {
            cs.status = ContractStatus::Ended;
            cs.closed = Some(today);
            if Some(cs.seller_company) == player || Some(cs.buyer_company) == player {
                news.push(
                    Message::new(MessageKind::Info, keys::CONTRACT_ENDED).with("produkt", product),
                );
            }
        }
    }
    let keep = catalog.contracts.keep_months;
    state
        .contracts
        .retain(|c| c.open() || c.closed.is_none_or(|d| d.add_months(keep) > today));
    news.extend(ai_proposals(state, catalog, today));
    news
}

/// AI companies propose contracts to the player: buyers of what the player offers,
/// sellers of what the player orders (docs/FORMELN.md, W4).
fn ai_proposals(state: &mut GameState, catalog: &Catalog, today: Date) -> Vec<Message> {
    let mut news = Vec::new();
    let m = &catalog.contracts;
    if !m.enabled() || m.ai.proposal_chance <= 0.0 {
        return news;
    }
    let Some(player) = state.main_company else {
        return news;
    };
    if state.companies[player.index()].bankrupt {
        return news;
    }
    let month = u32::try_from(today.year() * 12).unwrap_or(0) + today.month();
    let mut rng = SimRng::for_stream(state.settings.seed, Stream::Contracts { month });
    let own: Vec<SiteId> = (0..state.sites.len())
        .map(site_id)
        .filter(|&s| state.sites[s.index()].owner == player)
        .collect();
    let mut wanted: Vec<(SiteId, ProductId, bool)> = Vec::new();
    for &site in &own {
        let s = &state.sites[site.index()];
        for &p in s.offers.keys() {
            wanted.push((site, p, true));
        }
        for (&p, o) in &s.orders {
            if o.target > 0.0 {
                wanted.push((site, p, false));
            }
        }
    }
    for (site, product, player_sells) in wanted {
        if rng.next_f64() >= m.ai.proposal_chance {
            continue;
        }
        let Some(terms) = best_partner(state, catalog, site, product, player_sells) else {
            continue;
        };
        let ai = if player_sells {
            state.sites[terms.buyer.index()].owner
        } else {
            state.sites[terms.seller.index()].owner
        };
        if propose(state, catalog, ai, &terms).is_ok() {
            let name = state.companies[ai.index()].name.clone();
            news.push(
                Message::new(MessageKind::Info, keys::CONTRACT_PROPOSED)
                    .with("firma", Param::Text(name))
                    .with(
                        "produkt",
                        Param::TextKey(format!("produkt.{}", catalog.products.key(product))),
                    ),
            );
        }
    }
    news
}

/// The AI site with the most free need (when the player sells) or free output (when the
/// player buys) of a product that a route reaches, and the terms it proposes.
fn best_partner(
    state: &GameState,
    catalog: &Catalog,
    own: SiteId,
    product: ProductId,
    player_sells: bool,
) -> Option<Terms> {
    let m = &catalog.contracts;
    let player = state.main_company?;
    let mut best: Option<(f64, SiteId)> = None;
    for (i, s) in state.sites.iter().enumerate() {
        let site = site_id(i);
        let owner = &state.companies[s.owner.index()];
        if crate::group::same_group(state, s.owner, player) || owner.bankrupt {
            continue;
        }
        let pair = if player_sells {
            (own, site)
        } else {
            (site, own)
        };
        // One proposal per pair as long as the list keeps the last one: no new one while
        // a contract runs or soon after one was declined or lapsed.
        if state
            .contracts
            .iter()
            .any(|c| c.product == product && (c.seller, c.buyer) == pair)
        {
            continue;
        }
        let free = if player_sells {
            m.ai.share * 30.0 * daily_need(state, catalog, site, product)
                - contracted(state, site, product, false)
        } else {
            if !s.offers.contains_key(&product) {
                continue;
            }
            m.ai.share * 30.0 * daily_output(state, catalog, site, product)
                - contracted(state, site, product, true)
        };
        if free <= EPS || best.is_some_and(|(b, _)| b >= free) {
            continue;
        }
        if delivery_cost(state, catalog, pair, product, Money::ZERO).is_none() {
            continue;
        }
        best = Some((free, site));
    }
    let (free, partner) = best?;
    let (seller, buyer) = if player_sells {
        (own, partner)
    } else {
        (partner, own)
    };
    // As much as the player's site makes or uses, at most the partner's free part.
    let player_side = 30.0
        * if player_sells {
            daily_output(state, catalog, own, product)
        } else {
            daily_need(state, catalog, own, product)
        };
    let per_month = if player_side > EPS {
        free.min(m.ai.share * player_side)
    } else {
        free
    };
    let buyer_country = state.sites[buyer.index()].country;
    let market = market::market_price(catalog, state, buyer_country, product);
    let price = if player_sells {
        market
    } else {
        market.max(ask_price(state, catalog, (seller, buyer), product)?)
    };
    (per_month > EPS && price > Money::ZERO).then_some(Terms {
        seller,
        buyer,
        product,
        per_month,
        price,
        months: m.months_default,
        min_quality: 0.0,
        penalty: m.penalty_default.min(m.ai.penalty_max),
    })
}
