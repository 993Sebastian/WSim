//! Safe investments, gold and funds of the person (PV2, docs/FORMELN.md).

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::{AssetModel, Catalog};
use crate::command::CommandError;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{GameState, PrivateFlow};

/// Offset of the stream of the gold price among the streams of the person.
const GOLD_STREAM: u32 = 3 << 24;

/// The two kinds of fund (PV2): managed, and the cheap index fund of a later epoch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FundKind {
    Managed,
    Index,
}

impl FundKind {
    pub const ALL: [FundKind; 2] = [FundKind::Managed, FundKind::Index];

    pub fn key(self) -> &'static str {
        match self {
            FundKind::Managed => "gemanagt",
            FundKind::Index => "index",
        }
    }
}

/// A fixed-term deposit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Deposit {
    pub principal: Money,
    /// Yearly rate, fixed at the opening.
    pub rate: f64,
    pub until: Date,
    /// Interest paid out so far.
    pub paid: Money,
}

/// A government bond held.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GovBond {
    pub face: Money,
    pub coupon: f64,
    pub until: Date,
    /// What it cost.
    pub cost: Money,
}

/// Units of a fund and what they cost.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FundPosition {
    pub units: f64,
    pub cost: Money,
}

/// A monthly purchase of fund units.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavingsPlan {
    pub fund: FundKind,
    pub amount: Money,
}

/// What the person holds besides companies (PV2).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Portfolio {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deposits: Vec<Deposit>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bonds: Vec<GovBond>,
    #[serde(default)]
    pub gold_ounces: f64,
    #[serde(default)]
    pub gold_cost: Money,
    #[serde(default)]
    pub funds: [FundPosition; 2],
    #[serde(default)]
    pub plan: Option<SavingsPlan>,
    /// Real estate (PV3).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<crate::property::Property>,
}

impl Portfolio {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// The prices of gold and funds (PV2).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AssetMarket {
    /// The noise of the gold price (ln).
    pub gold_noise: f64,
    /// Gold per ounce now.
    pub gold_price: Money,
    /// Unit value of each fund; 0 until the first month.
    pub nav: [f64; 2],
    /// The stock index the funds last followed.
    pub last_index: f64,
    /// Properties for sale (PV3).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub offers: Vec<crate::property::PropertyOffer>,
    #[serde(default)]
    pub next_offer: u32,
}

impl AssetMarket {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// An action on the investments, one command (PV2).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Investment {
    OpenDeposit {
        amount: Money,
        months: u32,
    },
    BreakDeposit {
        index: u32,
    },
    BuyBond {
        face: Money,
        months: u32,
    },
    SellBond {
        index: u32,
    },
    BuyGold {
        amount: Money,
    },
    SellGold {
        share: f64,
    },
    BuyFund {
        fund: FundKind,
        amount: Money,
    },
    SellFund {
        fund: FundKind,
        share: f64,
    },
    /// An amount of zero ends the plan.
    SetPlan {
        fund: FundKind,
        amount: Money,
    },
    /// Buys an offer with a mortgage of the given share of the price (PV3).
    BuyProperty {
        offer: u32,
        mortgage: f64,
    },
    SellProperty {
        index: u32,
    },
    /// The rent as a multiple of the market rent.
    SetRent {
        index: u32,
        factor: f64,
    },
    Renovate {
        index: u32,
    },
    /// Lets a property to a site of the person's firm (flats: company housing, office and
    /// hall: lease), or ends it with none (PV5c).
    AssignProperty {
        index: u32,
        site: Option<u32>,
    },
    /// The level of the property management, 0 to manage oneself (PV5b).
    SetHousingManagement {
        level: u32,
    },
    /// The level of the tax advisor, 0 for none.
    SetTaxAdvisor {
        level: u32,
    },
    /// An asset manager with a strategy (0 cautious, 1 balanced, 2 risky) and the share of
    /// the free money he may invest each month.
    HireAssetManager {
        strategy: u32,
        budget: f64,
    },
    DismissAssetManager,
}

fn model(catalog: &Catalog) -> Option<&AssetModel> {
    catalog.person.assets.as_ref()
}

/// The yearly rate of a deposit opened today.
pub fn deposit_rate(catalog: &Catalog, state: &GameState, m: &AssetModel) -> f64 {
    catalog
        .person
        .savings_rate
        .value(state.person.home, state.date.year_fraction())
        + m.deposit_premium
}

/// The coupon of a bond bought today.
pub fn bond_coupon(catalog: &Catalog, state: &GameState, m: &AssetModel) -> f64 {
    deposit_rate(catalog, state, m) + m.bond_premium
}

/// The price of a bond in shares of its face value.
pub fn bond_quote(catalog: &Catalog, state: &GameState, m: &AssetModel, bond: &GovBond) -> f64 {
    let years = f64::from(state.date.days_until(bond.until).max(0)) / 365.25;
    let duration = m.bond_duration_factor * years;
    (1.0 - duration * (bond_coupon(catalog, state, m) - bond.coupon)).clamp(0.5, 1.5)
}

/// Whether a fund can be bought in the year.
pub fn fund_open(catalog: &Catalog, state: &GameState, fund: FundKind) -> bool {
    model(catalog).is_some_and(|m| state.date.year() >= m.funds[fund as usize].from)
}

/// The order fee: the model's, less what the background gives.
pub fn order_fee(catalog: &Catalog, state: &GameState, base: f64) -> f64 {
    (base * (1.0 + crate::wellbeing::effect(catalog, state).order_fee)).max(0.0)
}

/// The unit value of a fund now.
pub fn nav(state: &GameState, fund: FundKind) -> f64 {
    let v = state.assets.nav[fund as usize];
    if v > 0.0 { v } else { 100.0 }
}

/// What the safe investments are worth: deposits at face, bonds at their price.
pub fn safe_value(catalog: &Catalog, state: &GameState) -> Money {
    let Some(m) = model(catalog) else {
        return Money::ZERO;
    };
    let p = &state.person.portfolio;
    let deposits: Money = p.deposits.iter().map(|d| d.principal).sum();
    let bonds: Money = p
        .bonds
        .iter()
        .map(|b| b.face.scale(bond_quote(catalog, state, m, b)))
        .sum();
    deposits + bonds
}

pub fn gold_value(state: &GameState) -> Money {
    state
        .assets
        .gold_price
        .scale(state.person.portfolio.gold_ounces)
}

pub fn funds_value(state: &GameState) -> Money {
    FundKind::ALL
        .iter()
        .map(|&k| {
            Money::from_usd(state.person.portfolio.funds[k as usize].units * nav(state, k))
                .unwrap_or(Money::ZERO)
        })
        .sum()
}

fn pay(state: &mut GameState, flow: PrivateFlow, amount: Money) -> Result<(), CommandError> {
    if state.person.account.balance < amount {
        return Err(CommandError::NotEnoughPrivateMoney {
            needed: amount - state.person.account.balance,
        });
    }
    crate::private::book_flow(state, flow, -amount);
    Ok(())
}

/// Receives sale proceeds and pays the tax on the gain.
fn receive(state: &mut GameState, catalog: &Catalog, proceeds: Money, basis: Money) {
    crate::private::book_flow(state, PrivateFlow::AssetSold, proceeds);
    let tax = (proceeds - basis)
        .max(Money::ZERO)
        .scale(crate::private::capital_tax_rate(catalog, state));
    if tax > Money::ZERO {
        crate::private::book_flow(state, PrivateFlow::GainTax, -tax.min(proceeds));
    }
}

/// An action of the person (`Command::Invest`).
pub(crate) fn apply(
    state: &mut GameState,
    catalog: &Catalog,
    action: Investment,
) -> Result<(), CommandError> {
    let Some(m) = model(catalog).cloned() else {
        return Err(CommandError::NotPossible);
    };
    if state.person.name.is_empty() {
        return Err(CommandError::PersonOnly);
    }
    let today = state.date;
    match action {
        Investment::OpenDeposit { amount, months } => {
            if amount <= Money::ZERO || months < m.deposit_months[0] || months > m.deposit_months[1]
            {
                return Err(CommandError::NotPossible);
            }
            pay(state, PrivateFlow::AssetBought, amount)?;
            let rate = deposit_rate(catalog, state, &m);
            state.person.portfolio.deposits.push(Deposit {
                principal: amount,
                rate,
                until: today.add_months(months),
                paid: Money::ZERO,
            });
        }
        Investment::BreakDeposit { index } => {
            let i = index as usize;
            if i >= state.person.portfolio.deposits.len() {
                return Err(CommandError::NotPossible);
            }
            let d = state.person.portfolio.deposits.remove(i);
            let back = (d.principal - d.paid.scale(m.deposit_penalty)).max(Money::ZERO);
            crate::private::book_flow(state, PrivateFlow::AssetSold, back);
        }
        Investment::BuyBond { face, months } => {
            if face <= Money::ZERO || months < m.bond_months[0] || months > m.bond_months[1] {
                return Err(CommandError::NotPossible);
            }
            let cost = face.scale(1.0 + m.bond_spread / 2.0);
            pay(state, PrivateFlow::AssetBought, cost)?;
            let coupon = bond_coupon(catalog, state, &m);
            state.person.portfolio.bonds.push(GovBond {
                face,
                coupon,
                until: today.add_months(months),
                cost,
            });
        }
        Investment::SellBond { index } => {
            let i = index as usize;
            if i >= state.person.portfolio.bonds.len() {
                return Err(CommandError::NotPossible);
            }
            let b = state.person.portfolio.bonds.remove(i);
            let proceeds = b
                .face
                .scale(bond_quote(catalog, state, &m, &b) * (1.0 - m.bond_spread / 2.0));
            receive(state, catalog, proceeds, b.cost);
        }
        Investment::BuyGold { amount } => {
            let price = state.assets.gold_price;
            if amount <= Money::ZERO || price <= Money::ZERO {
                return Err(CommandError::NotPossible);
            }
            pay(state, PrivateFlow::AssetBought, amount)?;
            let f = order_fee(catalog, state, m.gold_fee);
            let fee_paid = amount.scale(f / (1.0 + f));
            let net = amount - fee_paid;
            // Ounces: the net money over the price with the half spread.
            let ounces = net.to_usd() / (price.to_usd() * (1.0 + m.gold_spread / 2.0));
            let p = &mut state.person.portfolio;
            p.gold_ounces += ounces;
            p.gold_cost += net;
            // The fee is a cost of its own, not part of the wealth.
            crate::private::note_fee(state, fee_paid);
        }
        Investment::SellGold { share } => {
            let p = &state.person.portfolio;
            if !(share > 0.0 && share <= 1.0) || p.gold_ounces <= 0.0 {
                return Err(CommandError::NotPossible);
            }
            let ounces = p.gold_ounces * share;
            let basis = p.gold_cost.scale(share);
            let f = order_fee(catalog, state, m.gold_fee);
            let gross = state
                .assets
                .gold_price
                .scale(ounces * (1.0 - m.gold_spread / 2.0));
            let proceeds = gross.scale(1.0 - f);
            let p = &mut state.person.portfolio;
            p.gold_ounces -= ounces;
            p.gold_cost -= basis;
            receive(state, catalog, proceeds, basis);
        }
        Investment::BuyFund { fund, amount } => buy_fund(state, catalog, &m, fund, amount)?,
        Investment::SellFund { fund, share } => {
            let pos = state.person.portfolio.funds[fund as usize];
            if !(share > 0.0 && share <= 1.0) || pos.units <= 0.0 {
                return Err(CommandError::NotPossible);
            }
            let units = pos.units * share;
            let basis = pos.cost.scale(share);
            let f = order_fee(catalog, state, m.funds[fund as usize].fee);
            let proceeds =
                Money::from_usd(units * nav(state, fund) * (1.0 - f)).unwrap_or(Money::ZERO);
            let p = &mut state.person.portfolio.funds[fund as usize];
            p.units -= units;
            p.cost -= basis;
            receive(state, catalog, proceeds, basis);
        }
        Investment::SetHousingManagement { .. }
        | Investment::SetTaxAdvisor { .. }
        | Investment::HireAssetManager { .. }
        | Investment::DismissAssetManager => crate::services::apply(state, catalog, action)?,
        Investment::BuyProperty { .. }
        | Investment::SellProperty { .. }
        | Investment::SetRent { .. }
        | Investment::Renovate { .. }
        | Investment::AssignProperty { .. } => crate::property::apply(state, catalog, action)?,
        Investment::SetPlan { fund, amount } => {
            state.person.portfolio.plan =
                (amount > Money::ZERO).then_some(SavingsPlan { fund, amount });
        }
    }
    Ok(())
}

fn buy_fund(
    state: &mut GameState,
    catalog: &Catalog,
    m: &AssetModel,
    fund: FundKind,
    amount: Money,
) -> Result<(), CommandError> {
    if amount <= Money::ZERO || !fund_open(catalog, state, fund) {
        return Err(CommandError::NotPossible);
    }
    pay(state, PrivateFlow::AssetBought, amount)?;
    let t = m.funds[fund as usize];
    let charge = (order_fee(catalog, state, t.fee) + t.load).min(0.9);
    let fee_paid = amount.scale(charge);
    let net = amount - fee_paid;
    let units = net.to_usd() / nav(state, fund);
    let pos = &mut state.person.portfolio.funds[fund as usize];
    pos.units += units;
    pos.cost += net;
    crate::private::note_fee(state, fee_paid);
    Ok(())
}

/// Sells funds, gold, deposits and bonds, in that order, until the account holds `need`
/// (the estate's payout to the co-heirs, PV4).
pub(crate) fn liquidate(state: &mut GameState, catalog: &Catalog, need: Money) {
    let enough = |s: &GameState| s.person.account.balance >= need;
    for fund in FundKind::ALL {
        if enough(state) {
            return;
        }
        let _ = apply(state, catalog, Investment::SellFund { fund, share: 1.0 });
    }
    if !enough(state) {
        let _ = apply(state, catalog, Investment::SellGold { share: 1.0 });
    }
    while !enough(state) && !state.person.portfolio.deposits.is_empty() {
        let _ = apply(state, catalog, Investment::BreakDeposit { index: 0 });
    }
    while !enough(state) && !state.person.portfolio.bonds.is_empty() {
        let _ = apply(state, catalog, Investment::SellBond { index: 0 });
    }
}

/// At a month start: interest and maturities, the gold price, the fund values, the plan.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    let Some(m) = model(catalog).cloned() else {
        return news;
    };
    // Gold: the series, the mood of the market and a noise of its own.
    let mut rng = SimRng::for_stream(
        state.settings.seed,
        Stream::Person {
            month: crate::management::month_number(date) + GOLD_STREAM,
        },
    );
    let eps = crate::stock::normal(&mut rng);
    state.assets.gold_noise =
        state.assets.gold_noise * (1.0 - m.gold_reversion) + m.gold_noise * eps;
    let base = m.gold_price.value_at(date.year_fraction());
    state.assets.gold_price = Money::from_usd(
        base * libm::exp(state.assets.gold_noise - m.gold_crisis_beta * state.stock.sentiment),
    )
    .unwrap_or(Money::ZERO);
    // Funds follow the index and the dividends.
    let index = state.stock.index;
    if state.assets.last_index > 0.0 && index > 0.0 {
        let ratio = index / state.assets.last_index;
        let (dividends, value) = state
            .companies
            .iter()
            .filter(|c| !c.bankrupt)
            .filter_map(|c| c.listing.as_ref())
            .fold((0.0, 0.0), |(d, v), l| {
                (d + l.last_dividend.to_usd(), v + l.value.to_usd())
            });
        let yield_month = if value > 0.0 {
            dividends / value / 12.0
        } else {
            0.0
        };
        for k in FundKind::ALL {
            let cost = m.funds[k as usize].cost / 12.0;
            state.assets.nav[k as usize] = nav(state, k) * (ratio + yield_month) * (1.0 - cost);
        }
    }
    state.assets.last_index = index;
    if state.person.name.is_empty() {
        return news;
    }
    // Interest and maturities. The interest counts like the account's (taxed at month end).
    let mut interest = Money::ZERO;
    let mut back = Money::ZERO;
    let mut gain_basis = Money::ZERO;
    let mut due = Vec::new();
    for d in &mut state.person.portfolio.deposits {
        let pay = d.principal.scale(d.rate / 12.0);
        d.paid += pay;
        interest += pay;
        if d.until <= date {
            due.push(d.principal);
        }
    }
    state.person.portfolio.deposits.retain(|d| d.until > date);
    back += due.iter().copied().sum::<Money>();
    let mut matured = Vec::new();
    for b in &state.person.portfolio.bonds {
        interest += b.face.scale(b.coupon / 12.0);
        if b.until <= date {
            matured.push((b.face, b.cost));
        }
    }
    state.person.portfolio.bonds.retain(|b| b.until > date);
    for (face, cost) in matured {
        back += face;
        gain_basis += cost;
    }
    if interest > Money::ZERO {
        crate::private::book_flow(state, PrivateFlow::Interest, interest);
    }
    if back > Money::ZERO {
        // Deposits come back whole; a bond's loss or gain is settled like a sale.
        let face_bonds = back - due.iter().copied().sum::<Money>();
        crate::private::book_flow(state, PrivateFlow::AssetSold, back);
        let tax = (face_bonds - gain_basis)
            .max(Money::ZERO)
            .scale(crate::private::capital_tax_rate(catalog, state));
        if tax > Money::ZERO {
            crate::private::book_flow(state, PrivateFlow::GainTax, -tax);
        }
    }
    news.extend(crate::property::month_start(state, catalog, date));
    news.extend(crate::services::month_start(state, catalog, date));
    // The plan.
    if let Some(plan) = state.person.portfolio.plan
        && buy_fund(state, catalog, &m, plan.fund, plan.amount).is_err()
    {
        state.person.portfolio.plan = None;
        news.push(
            Message::new(MessageKind::Warning, keys::SAVINGS_PLAN_ENDED)
                .with("betrag", Param::Money(plan.amount)),
        );
    }
    news
}
