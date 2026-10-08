//! Stock market (K1, Lastenheft §11.1–11.2; formulas in docs/FORMELN.md, K1).
//!
//! Listed companies have a market value that follows their book value and earnings,
//! the market's sentiment (with the historical crises of the data) and some noise of
//! their own. Companies go public or raise capital by selling new shares to investors,
//! pay dividends once a year and trade shares of other listed companies from the free
//! float.

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::command::CommandError;
use crate::ledger::{Account, CostCenter, CostType, Ledger};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{Company, CompanyId, GameState, Holder, Stake};

/// Shares of every listed company.
pub const SHARES: u64 = 1_000_000;

/// Months of history kept per company and for the index.
const HISTORY_MONTHS: usize = 240;

/// A company's listing.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Listing {
    /// Market value of all shares.
    pub value: Money,
    pub since: Date,
    /// Market value at the start of each month, oldest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub history: Vec<(Date, Money)>,
    /// The dividend paid last.
    #[serde(default)]
    pub last_dividend: Money,
    /// Shares outstanding: `SHARES` at the listing, more after new shares, fewer after
    /// buybacks (K3).
    #[serde(default = "shares_at_listing")]
    pub shares: u64,
}

fn shares_at_listing() -> u64 {
    SHARES
}

impl Listing {
    pub fn new(value: Money, since: Date) -> Self {
        Listing {
            value,
            since,
            history: vec![(since, value)],
            last_dividend: Money::ZERO,
            shares: SHARES,
        }
    }

    pub fn price(&self) -> Money {
        // Shares are far below the f64 range of exact integers.
        #[allow(clippy::cast_precision_loss)]
        self.value.scale(1.0 / self.shares.max(1) as f64)
    }

    /// Shares outstanding scaled by `factor`.
    fn scale_shares(&mut self, factor: f64) {
        // Shares stay far below the range where the casts lose anything that matters.
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        {
            self.shares = ((self.shares as f64) * factor).round().max(1.0) as u64;
        }
    }
}

/// The market as a whole.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StockMarket {
    /// ln S, the sentiment of all investors.
    pub sentiment: f64,
    /// Chained index, 100 at the start.
    pub index: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub index_history: Vec<(Date, f64)>,
    /// Dividends paid to the player as an owner.
    #[serde(default)]
    pub player_dividends: Money,
}

impl StockMarket {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

fn company_id(index: usize) -> CompanyId {
    CompanyId(u32::try_from(index).expect("company count fits u32"))
}

fn equity(ledger: &Ledger) -> Money {
    [Account::Equity, Account::RetainedEarnings, Account::Result]
        .iter()
        .map(|&a| ledger.balance(a))
        .sum()
}

/// Earnings of a year: the average of the last closed months (`gewinn_monate`); months
/// missing at the assumed return on the equity.
pub fn earnings(catalog: &Catalog, company: &Company) -> Money {
    let m = &catalog.stock;
    let window = m.earnings_months.max(1) as usize;
    let months: Vec<Money> = company
        .ledger
        .months
        .iter()
        .rev()
        .take(window)
        .map(|m| m.by_type.values().copied().sum())
        .collect();
    // At most 24 months; the casts are exact.
    #[allow(clippy::cast_precision_loss)]
    let (n, w) = (months.len() as f64, window as f64);
    let missing = equity(&company.ledger)
        .max(Money::ZERO)
        .scale(m.assumed_return * (w - n) / 12.0);
    (months.iter().copied().sum::<Money>() + missing).scale(12.0 / w)
}

/// F, the value of book and earnings, and the floor of the book value.
fn value_parts(catalog: &Catalog, company: &Company) -> (Money, Money) {
    let m = &catalog.stock;
    let book = equity(&company.ledger).max(Money::ZERO);
    let e = earnings(catalog, company).max(Money::ZERO);
    (
        book.scale(m.book_weight) + e.scale((1.0 - m.book_weight) * m.pe),
        book.scale(m.book_floor),
    )
}

/// The value the market aims at for a company (T).
pub fn target(catalog: &Catalog, market: &StockMarket, company: &Company) -> Money {
    let (f, floor) = value_parts(catalog, company);
    f.scale(libm::exp(market.sentiment)).max(floor)
}

/// The value without the investors' mood (K3): what AI investors judge by.
pub fn fair(catalog: &Catalog, company: &Company) -> Money {
    let (f, floor) = value_parts(catalog, company);
    f.max(floor)
}

/// The free float: the shares with investors outside the game, traded on the market.
pub fn free_float(company: &Company) -> f64 {
    stake(company, Holder::Investors)
}

/// The share a holder owns of a company.
pub fn stake(company: &Company, holder: Holder) -> f64 {
    company
        .owners
        .iter()
        .filter(|s| s.holder == holder)
        .fold(0.0, |sum, s| sum + s.share)
}

/// What the buyer pays for share q (premium and price effect included).
pub fn buy_price(catalog: &Catalog, listing: &Listing, share: f64) -> Money {
    let m = &catalog.stock;
    listing
        .value
        .scale(share * (1.0 + m.trade_premium + m.trade_impact * share))
}

/// What the seller gets for share q (discount and price effect deducted).
pub fn sell_proceeds(catalog: &Catalog, listing: &Listing, share: f64) -> Money {
    let m = &catalog.stock;
    listing
        .value
        .scale(share * (1.0 - m.trade_discount - m.trade_impact * share).max(0.0))
}

/// The value new shares are sold at: for a listed company its market value, else the
/// market's target; less the discount.
pub fn issue_value(catalog: &Catalog, market: &StockMarket, company: &Company) -> Money {
    let value = match &company.listing {
        Some(l) => l.value,
        None => target(catalog, market, company),
    };
    value.scale(1.0 - catalog.stock.ipo_discount)
}

/// Proceeds of new shares s at a value V before: V·s/(1−s).
pub fn issue_proceeds(before: Money, share: f64) -> Money {
    before.scale(share / (1.0 - share))
}

/// The dividend a company would pay now on last year's profit.
pub fn dividend(catalog: &Catalog, company: &Company) -> Money {
    let m = &catalog.stock;
    let quote = company.dividend_payout.unwrap_or(if company.ai.is_some() {
        m.ai_payout
    } else {
        0.0
    });
    let profit = company
        .ledger
        .years
        .last()
        .map(|y| y.by_type.values().copied().sum::<Money>())
        .unwrap_or_default();
    profit.max(Money::ZERO).scale(quote).min(
        company
            .ledger
            .cash()
            .max(Money::ZERO)
            .scale(m.dividend_cash_max),
    )
}

fn add_share(owners: &mut Vec<Stake>, holder: Holder, share: f64) {
    match owners.iter_mut().find(|s| s.holder == holder) {
        Some(s) => s.share += share,
        None => owners.push(Stake { holder, share }),
    }
}

fn remove_share(owners: &mut Vec<Stake>, holder: Holder, share: f64) {
    if let Some(s) = owners.iter_mut().find(|s| s.holder == holder) {
        s.share -= share;
    }
    owners.retain(|s| s.share > 1e-12);
}

/// New shares: every owner keeps (1 − s), the investors buy s.
fn dilute(owners: &mut Vec<Stake>, share: f64) {
    for s in owners.iter_mut() {
        s.share *= 1.0 - share;
    }
    add_share(owners, Holder::Investors, share);
}

/// The listings at the start of a game: AI companies with enough equity, part of their
/// shares with investors.
pub(crate) fn list_at_start(state: &mut GameState, catalog: &Catalog) {
    let m = &catalog.stock;
    if !m.enabled {
        return;
    }
    state.stock.index = 100.0;
    let date = state.date;
    for i in 0..state.companies.len() {
        let c = &state.companies[i];
        // State companies are not listed (H1).
        if c.ai.is_none()
            || c.listing.is_some()
            || c.state_owned.is_some()
            || equity(&c.ledger) < m.start_equity_min
        {
            continue;
        }
        let value = target(catalog, &state.stock, c);
        let c = &mut state.companies[i];
        c.owners = vec![
            Stake {
                holder: Holder::Private,
                share: 1.0 - m.start_free_float,
            },
            Stake {
                holder: Holder::Investors,
                share: m.start_free_float,
            },
        ];
        c.owners.retain(|s| s.share > 1e-12);
        c.listing = Some(Listing::new(value, date));
    }
}

/// Goes public with new shares s (docs/FORMELN.md, K1).
pub(crate) fn go_public(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    share: f64,
) -> Result<(), CommandError> {
    let m = &catalog.stock;
    if !m.enabled {
        return Err(CommandError::NoStockMarket);
    }
    let c = &state.companies[actor.index()];
    if c.listing.is_some() {
        return Err(CommandError::AlreadyListed);
    }
    if c.subsidiary_of.is_some() {
        return Err(CommandError::WithinGroup);
    }
    if equity(&c.ledger) < m.ipo_equity_min {
        return Err(CommandError::EquityTooLow {
            min: m.ipo_equity_min,
        });
    }
    check_share(share, m.ipo_share_max)?;
    let before = issue_value(catalog, &state.stock, c);
    let proceeds = issue_proceeds(before, share);
    let date = state.date;
    let c = &mut state.companies[actor.index()];
    sell_new_shares(c, proceeds, m.ipo_cost_share);
    dilute(&mut c.owners, share);
    c.listing = Some(Listing::new(before + proceeds, date));
    Ok(())
}

/// New shares s of a listed company at its market value less the discount.
pub(crate) fn issue_shares(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    share: f64,
) -> Result<(), CommandError> {
    let m = &catalog.stock;
    if !m.enabled {
        return Err(CommandError::NoStockMarket);
    }
    let c = &state.companies[actor.index()];
    if c.listing.is_none() {
        return Err(CommandError::NotListed);
    }
    check_share(share, m.ipo_share_max)?;
    let proceeds = issue_proceeds(issue_value(catalog, &state.stock, c), share);
    let c = &mut state.companies[actor.index()];
    sell_new_shares(c, proceeds, m.ipo_cost_share);
    dilute(&mut c.owners, share);
    if let Some(l) = c.listing.as_mut() {
        l.value += proceeds;
        l.scale_shares(1.0 / (1.0 - share));
    }
    Ok(())
}

fn check_share(share: f64, max: f64) -> Result<(), CommandError> {
    if !(share.is_finite() && share > 0.0 && share <= max + 1e-12) {
        return Err(CommandError::ShareOutOfRange { max });
    }
    Ok(())
}

/// Whether new shares `share` cost the player the majority of its company (K3: allowed,
/// but then a takeover can end the game).
pub fn loses_majority(c: &Company, share: f64) -> bool {
    let player = stake(c, Holder::Player);
    player > 0.5 && player * (1.0 - share) <= 0.5
}

fn sell_new_shares(c: &mut Company, proceeds: Money, cost_share: f64) {
    c.ledger.transfer(Account::Cash, Account::Equity, proceeds);
    let cost = proceeds.scale(cost_share);
    if cost > Money::ZERO {
        c.ledger
            .expense(CostType::Other, CostCenter::default(), Account::Cash, cost);
    }
}

/// The share of last year's profit the company pays out.
pub(crate) fn set_dividend(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    payout: f64,
) -> Result<(), CommandError> {
    if !catalog.stock.enabled {
        return Err(CommandError::NoStockMarket);
    }
    if !(payout.is_finite() && (0.0..=1.0).contains(&payout)) {
        return Err(CommandError::ShareOutOfRange { max: 1.0 });
    }
    state.companies[actor.index()].dividend_payout = Some(payout);
    Ok(())
}

/// Buys share q of a listed company from its free float.
pub(crate) fn buy(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    target_company: CompanyId,
    share: f64,
) -> Result<(), CommandError> {
    let m = &catalog.stock;
    if !m.enabled {
        return Err(CommandError::NoStockMarket);
    }
    let t = state
        .companies
        .get(target_company.index())
        .ok_or(CommandError::UnknownCompany(target_company))?;
    if crate::group::same_group(state, actor, target_company) {
        return Err(CommandError::WithinGroup);
    }
    let Some(listing) = &t.listing else {
        return Err(CommandError::NotListed);
    };
    if t.bankrupt {
        return Err(CommandError::SellerBankrupt);
    }
    if !(share.is_finite() && share > 0.0) {
        return Err(CommandError::InvalidQuantity);
    }
    let float = free_float(t);
    if share > float + 1e-12 {
        return Err(CommandError::NotEnoughFreeFloat { available: float });
    }
    let held = stake(t, Holder::Company(actor));
    if held + share > m.trade_share_max + 1e-12 {
        return Err(CommandError::ShareOutOfRange {
            max: (m.trade_share_max - held).max(0.0),
        });
    }
    let price = buy_price(catalog, listing, share);
    let buyer = &mut state.companies[actor.index()];
    if buyer.ledger.cash() < price {
        return Err(CommandError::NotEnoughCash { needed: price });
    }
    buyer
        .ledger
        .transfer(Account::Participations, Account::Cash, price);
    *buyer.stock_cost.entry(target_company).or_default() += price;
    let t = &mut state.companies[target_company.index()];
    remove_share(&mut t.owners, Holder::Investors, share);
    add_share(&mut t.owners, Holder::Company(actor), share);
    if let Some(l) = t.listing.as_mut() {
        l.value = l.value.scale(1.0 + m.trade_impact * share);
    }
    Ok(())
}

/// Sells share q of a listed company to investors.
pub(crate) fn sell(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    target_company: CompanyId,
    share: f64,
) -> Result<(), CommandError> {
    let m = &catalog.stock;
    if !m.enabled {
        return Err(CommandError::NoStockMarket);
    }
    let t = state
        .companies
        .get(target_company.index())
        .ok_or(CommandError::UnknownCompany(target_company))?;
    let Some(listing) = &t.listing else {
        return Err(CommandError::NotListed);
    };
    if !(share.is_finite() && share > 0.0) {
        return Err(CommandError::InvalidQuantity);
    }
    let held = stake(t, Holder::Company(actor));
    if share > held + 1e-12 {
        return Err(CommandError::NotEnoughStock { held });
    }
    let proceeds = sell_proceeds(catalog, listing, share);
    realize(
        &mut state.companies[actor.index()],
        target_company,
        (held, share),
        proceeds,
    );
    let t = &mut state.companies[target_company.index()];
    remove_share(&mut t.owners, Holder::Company(actor), share);
    add_share(&mut t.owners, Holder::Investors, share);
    if let Some(l) = t.listing.as_mut() {
        l.value = l.value.scale((1.0 - m.trade_impact * share).max(0.01));
    }
    Ok(())
}

/// Books the sale of `share` of a holding `held` in `target` for `proceeds`: the cost of
/// the shares sold leaves the participations, the difference is a gain or a loss.
fn realize(seller: &mut Company, target: CompanyId, (held, share): (f64, f64), proceeds: Money) {
    let cost_all = seller.stock_cost.get(&target).copied().unwrap_or_default();
    let cost = if share >= held - 1e-12 || held <= 0.0 {
        cost_all
    } else {
        cost_all.scale(share / held)
    };
    let ledger = &mut seller.ledger;
    let center = CostCenter::default();
    if proceeds >= cost {
        ledger.income(
            CostType::Investments,
            center,
            Account::Participations,
            proceeds - cost,
        );
    } else {
        ledger.expense(
            CostType::Investments,
            center,
            Account::Participations,
            cost - proceeds,
        );
    }
    ledger.transfer(Account::Cash, Account::Participations, proceeds);
    if cost >= cost_all {
        seller.stock_cost.remove(&target);
    } else if let Some(c) = seller.stock_cost.get_mut(&target) {
        *c -= cost;
    }
}

/// What a takeover bid for all shares a buyer does not hold costs: the price paid to the
/// owners and the costs of banks and advisers. The player never sells its own stake.
pub fn takeover_price(catalog: &Catalog, target: &Company, buyer: CompanyId) -> (Money, Money) {
    let m = &catalog.stock;
    let Some(listing) = &target.listing else {
        return (Money::ZERO, Money::ZERO);
    };
    let bought: f64 = target
        .owners
        .iter()
        .filter(|s| s.holder != Holder::Company(buyer) && s.holder != Holder::Player)
        .fold(0.0, |sum, s| sum + s.share);
    let price = listing.value.scale(bought * (1.0 + m.takeover_premium));
    (price, price.scale(m.takeover_cost_share))
}

/// A takeover bid (K3, docs/FORMELN.md): the buyer pays every owner but the player the
/// market value with the premium; the target leaves the market and becomes the buyer's
/// subsidiary. Taking over the player's company ends the game.
pub(crate) fn take_over(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    target_company: CompanyId,
) -> Result<(), CommandError> {
    let m = &catalog.stock;
    if !m.enabled {
        return Err(CommandError::NoStockMarket);
    }
    let t = state
        .companies
        .get(target_company.index())
        .ok_or(CommandError::UnknownCompany(target_company))?;
    if crate::group::same_group(state, actor, target_company) {
        return Err(CommandError::WithinGroup);
    }
    let Some(listing) = &t.listing else {
        return Err(CommandError::NotListed);
    };
    if t.bankrupt {
        return Err(CommandError::SellerBankrupt);
    }
    // The player keeps its stake; without the majority of the rest no control.
    let kept = stake(t, Holder::Player);
    if kept >= 0.5 {
        return Err(CommandError::TakeoverNoMajority { held: kept });
    }
    let (price, cost) = takeover_price(catalog, t, actor);
    if state.companies[actor.index()].ledger.cash() < price + cost {
        return Err(CommandError::NotEnoughCash {
            needed: price + cost,
        });
    }
    let per_share = listing.value.scale(1.0 + m.takeover_premium);
    let owners = t.owners.clone();
    // The sellers: companies book the sale, everyone else leaves the game with the money.
    for s in &owners {
        if let Holder::Company(h) = s.holder
            && h != actor
        {
            let held = s.share;
            realize(
                &mut state.companies[h.index()],
                target_company,
                (held, held),
                per_share.scale(held),
            );
        }
    }
    let buyer = &mut state.companies[actor.index()];
    buyer
        .ledger
        .transfer(Account::Participations, Account::Cash, price);
    if cost > Money::ZERO {
        buyer
            .ledger
            .expense(CostType::Other, CostCenter::default(), Account::Cash, cost);
    }
    // The cost of shares bought before stays in the participations of the subsidiary.
    buyer.stock_cost.remove(&target_company);
    let t = &mut state.companies[target_company.index()];
    t.owners = vec![Stake {
        holder: Holder::Company(actor),
        share: 1.0 - kept,
    }];
    if kept > 1e-12 {
        t.owners.push(Stake {
            holder: Holder::Player,
            share: kept,
        });
    }
    t.listing = None;
    t.subsidiary_of = Some(crate::group::SubsidiaryOf {
        parent: actor,
        focus: crate::group::SubsidiaryFocus::Production,
    });
    if state.is_main(target_company) {
        state.game_over = true;
    }
    Ok(())
}

/// The company buys back share q of its own stock from the free float (K3).
pub(crate) fn buy_back(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    share: f64,
) -> Result<(), CommandError> {
    let m = &catalog.stock;
    if !m.enabled {
        return Err(CommandError::NoStockMarket);
    }
    let c = &state.companies[actor.index()];
    let Some(listing) = &c.listing else {
        return Err(CommandError::NotListed);
    };
    check_share(share, m.buyback_share_max)?;
    let float = free_float(c);
    if share > float + 1e-12 {
        return Err(CommandError::NotEnoughFreeFloat { available: float });
    }
    let price = buy_price(catalog, listing, share);
    if c.ledger.cash() < price {
        return Err(CommandError::NotEnoughCash { needed: price });
    }
    let c = &mut state.companies[actor.index()];
    // Out of the retained earnings first, then out of the subscribed capital.
    let reserves = c
        .ledger
        .balance(Account::RetainedEarnings)
        .max(Money::ZERO)
        .min(price);
    c.ledger
        .transfer(Account::RetainedEarnings, Account::Cash, reserves);
    c.ledger
        .transfer(Account::Equity, Account::Cash, price - reserves);
    remove_share(&mut c.owners, Holder::Investors, share);
    for s in &mut c.owners {
        s.share /= 1.0 - share;
    }
    if let Some(l) = c.listing.as_mut() {
        l.value = (l.value - price)
            .max(Money::ZERO)
            .scale(1.0 + m.trade_impact * share);
        l.scale_shares(1.0 - share);
    }
    Ok(())
}

/// What an AI investor trades this month (K3): its holdings far above their fair value
/// sold, and a part of its `surplus` cash in the most undervalued listed company.
pub(crate) fn ai_trades(
    state: &GameState,
    catalog: &Catalog,
    id: CompanyId,
    surplus: Money,
) -> Vec<crate::command::Command> {
    use crate::command::Command;
    let m = &catalog.stock;
    let mut out = Vec::new();
    if !m.enabled {
        return out;
    }
    let me = &state.companies[id.index()];
    let ratio = |c: &Company| {
        let f = fair(catalog, c);
        c.listing
            .as_ref()
            .filter(|_| f > Money::ZERO)
            .map(|l| l.value.to_usd() / f.to_usd())
    };
    for &held_id in me.stock_cost.keys() {
        let t = &state.companies[held_id.index()];
        let held = stake(t, Holder::Company(id));
        if held > 0.0 && ratio(t).is_some_and(|r| r > 1.0 + m.ai_undervaluation) {
            out.push(Command::SellShares {
                company: held_id,
                share: held,
            });
        }
    }
    if surplus <= Money::ZERO || m.ai_portfolio_cash_share <= 0.0 {
        return out;
    }
    let best = state
        .companies
        .iter()
        .enumerate()
        .filter(|(i, c)| {
            !c.bankrupt
                && c.listing.is_some()
                && !crate::group::same_group(state, id, company_id(*i))
        })
        .filter_map(|(i, c)| ratio(c).map(|r| (r, i)))
        .filter(|(r, _)| *r < 1.0 - m.ai_undervaluation)
        .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    if let Some((_, i)) = best {
        let t = &state.companies[i];
        let l = t.listing.as_ref().expect("listed");
        let budget = surplus.scale(m.ai_portfolio_cash_share);
        let room = (m.ai_portfolio_stake_max - stake(t, Holder::Company(id)))
            .min(free_float(t))
            .min(m.trade_share_max - stake(t, Holder::Company(id)));
        let affordable = budget.to_usd() / (l.value.to_usd() * (1.0 + m.trade_premium)).max(1.0);
        let share = affordable.min(room);
        if share >= 0.001 {
            out.push(Command::BuyShares {
                company: company_id(i),
                share,
            });
        }
    }
    out
}

/// The company an AI company bids for this month, if any (K3): with a chance by its
/// aggressiveness, the listed company most below its fair value that its cash affords.
pub(crate) fn ai_takeover(
    state: &mut GameState,
    catalog: &Catalog,
    id: CompanyId,
    aggressiveness: f64,
) -> Option<CompanyId> {
    let m = &catalog.stock;
    if !m.enabled || m.ai_takeover_chance <= 0.0 {
        return None;
    }
    let draw = state.companies[id.index()].rng.next_f64();
    if draw >= m.ai_takeover_chance * aggressiveness {
        return None;
    }
    let budget = state.companies[id.index()]
        .ledger
        .cash()
        .scale(m.ai_takeover_cash_share);
    state
        .companies
        .iter()
        .enumerate()
        .filter(|(i, c)| {
            !c.bankrupt
                && c.listing.is_some()
                && stake(c, Holder::Player) < 0.5
                && !crate::group::same_group(state, id, company_id(*i))
        })
        .filter_map(|(i, c)| {
            let f = fair(catalog, c);
            let l = c.listing.as_ref()?;
            let (price, cost) = takeover_price(catalog, c, id);
            (f > Money::ZERO && l.value < f && price + cost <= budget)
                .then(|| (l.value.to_usd() / f.to_usd(), i))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)))
        .map(|(_, i)| company_id(i))
}

/// At the start of a month: sentiment and crises, market values and the index,
/// dividends in the dividend month, failed listed companies written off. Returns the
/// messages for the player.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let m = &catalog.stock;
    let mut news = Vec::new();
    if !m.enabled {
        return news;
    }
    let month = u32::try_from(date.year() * 12).unwrap_or(0) + date.month();
    let mut rng = SimRng::for_stream(state.settings.seed, Stream::Stock { month });
    // Sentiment: back towards 0, a random step, the crashes of the events (H1).
    let mut s = state.stock.sentiment * (1.0 - m.sentiment_reversion)
        + m.sentiment_volatility * normal(&mut rng);
    for drop in crate::events::crashes(state, catalog, date) {
        s += libm::log(1.0 - drop);
        news.push(
            Message::new(MessageKind::Warning, keys::STOCK_CRISIS)
                .with("einbruch", Param::Number((drop * 100.0).round())),
        );
    }
    state.stock.sentiment = s;

    // Market values, chained into the index.
    let (mut before, mut after) = (0.0_f64, 0.0_f64);
    for i in 0..state.companies.len() {
        if state.companies[i].bankrupt || state.companies[i].listing.is_none() {
            continue;
        }
        let aim = target(catalog, &state.stock, &state.companies[i]);
        let eps = normal(&mut rng);
        let l = state.companies[i].listing.as_mut().expect("listed");
        let old = l.value;
        let ratio = if old > Money::ZERO {
            (aim.to_usd() / old.to_usd()).max(1e-6)
        } else {
            1.0
        };
        let new = if old > Money::ZERO {
            old.scale(libm::pow(ratio, m.inertia) * libm::exp(m.noise * eps))
        } else {
            aim
        };
        l.value = new;
        l.history.push((date, new));
        if l.history.len() > HISTORY_MONTHS {
            l.history.remove(0);
        }
        before += old.to_usd();
        after += new.to_usd();
    }
    if before > 0.0 {
        state.stock.index *= after / before;
    }
    let index = state.stock.index;
    state.stock.index_history.push((date, index));
    if state.stock.index_history.len() > HISTORY_MONTHS {
        state.stock.index_history.remove(0);
    }
    news.extend(delist_failures(state));
    if date.month() == m.dividend_month {
        news.extend(dividends(state, catalog));
    }
    news
}

/// A standard normal draw (Box–Muller).
fn normal(rng: &mut SimRng) -> f64 {
    let u1 = rng.next_f64().max(1e-12);
    let u2 = rng.next_f64();
    libm::sqrt(-2.0 * libm::log(u1)) * libm::cos(2.0 * std::f64::consts::PI * u2)
}

/// Failed listed companies leave the market; companies holding their shares write them
/// off.
fn delist_failures(state: &mut GameState) -> Vec<Message> {
    let mut news = Vec::new();
    for i in 0..state.companies.len() {
        if !(state.companies[i].bankrupt && state.companies[i].listing.is_some()) {
            continue;
        }
        let failed = company_id(i);
        state.companies[i].listing = None;
        let name = state.companies[i].name.clone();
        for h in 0..state.companies.len() {
            let Some(cost) = state.companies[h].stock_cost.remove(&failed) else {
                continue;
            };
            if cost > Money::ZERO {
                state.companies[h].ledger.expense(
                    CostType::Investments,
                    CostCenter::default(),
                    Account::Participations,
                    cost,
                );
            }
            if state.is_main(company_id(h)) {
                news.push(
                    Message::new(MessageKind::Warning, keys::STOCK_WRITTEN_OFF)
                        .with("firma", Param::Text(name.clone()))
                        .with("betrag", Param::Money(cost)),
                );
            }
        }
    }
    news
}

/// The yearly dividends of the listed companies.
fn dividends(state: &mut GameState, catalog: &Catalog) -> Vec<Message> {
    let mut news = Vec::new();
    let player = state.main_company;
    let mut received = Money::ZERO;
    for i in 0..state.companies.len() {
        let c = &state.companies[i];
        if c.bankrupt || c.listing.is_none() {
            continue;
        }
        let amount = dividend(catalog, c);
        if amount <= Money::ZERO {
            continue;
        }
        let owners = c.owners.clone();
        let c = &mut state.companies[i];
        c.ledger
            .transfer(Account::RetainedEarnings, Account::Cash, amount);
        if let Some(l) = c.listing.as_mut() {
            l.last_dividend = amount;
        }
        if Some(company_id(i)) == player {
            news.push(
                Message::new(MessageKind::Info, keys::STOCK_DIVIDEND_PAID)
                    .with("betrag", Param::Money(amount)),
            );
        }
        for s in owners {
            let part = amount.scale(s.share);
            match s.holder {
                Holder::Company(h) => {
                    state.companies[h.index()].ledger.income(
                        CostType::Investments,
                        CostCenter::default(),
                        Account::Cash,
                        part,
                    );
                    if Some(h) == player {
                        received += part;
                    }
                }
                Holder::Player => state.stock.player_dividends += part,
                Holder::Private | Holder::Investors => {}
            }
        }
    }
    if received > Money::ZERO {
        news.push(
            Message::new(MessageKind::Info, keys::STOCK_DIVIDEND_RECEIVED)
                .with("betrag", Param::Money(received)),
        );
    }
    news
}

/// Whether an AI company goes public this month, and with what share.
pub(crate) fn ai_ipo(state: &mut GameState, catalog: &Catalog, id: CompanyId) -> Option<f64> {
    let m = &catalog.stock;
    let c = &state.companies[id.index()];
    if !m.enabled
        || c.listing.is_some()
        || c.subsidiary_of.is_some()
        || c.state_owned.is_some()
        || c.bankrupt
        || equity(&c.ledger) < m.start_equity_min.max(m.ipo_equity_min)
    {
        return None;
    }
    let draw = state.companies[id.index()].rng.next_f64();
    (draw < m.ai_ipo_chance).then_some(m.ai_ipo_share)
}
