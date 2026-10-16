//! Property management, tax advisor and asset manager of the person (PV5b,
//! docs/FORMELN.md).

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::{Catalog, ServiceModel};
use crate::command::CommandError;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::savings::{FundKind, Investment};
use crate::state::{GameState, PrivateFlow};

/// Offset of the stream of the asset manager among the streams of the person.
const MANAGER_STREAM: u32 = 5 << 24;

/// The asset manager hired.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetManager {
    /// 0 cautious, 1 balanced, 2 risky.
    pub strategy: u32,
    /// Share of the free money invested each month.
    pub budget: f64,
    /// Honesty 0–100, drawn at the hiring.
    pub honesty: u32,
}

/// The services the person has engaged (PV5b).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Services {
    #[serde(default)]
    pub housing: u32,
    #[serde(default)]
    pub tax: u32,
    #[serde(default)]
    pub manager: Option<AssetManager>,
}

impl Services {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

fn model(catalog: &Catalog) -> Option<&ServiceModel> {
    catalog
        .person
        .assets
        .as_ref()
        .and_then(|a| a.services.as_ref())
}

/// The property management now: fee on the rent, cut of vacancy and of damage.
pub fn housing(catalog: &Catalog, state: &GameState) -> (f64, f64, f64) {
    let level = state.person.services.housing as usize;
    match (model(catalog), level) {
        (Some(m), l) if l >= 1 && l <= m.housing.len() => {
            let x = m.housing[l - 1];
            (x[0], x[1], x[2])
        }
        _ => (0.0, 0.0, 0.0),
    }
}

/// The relief of the income and capital tax by the tax advisor.
pub fn tax_relief(catalog: &Catalog, state: &GameState) -> f64 {
    let level = state.person.services.tax as usize;
    match (model(catalog), level) {
        (Some(m), l) if l >= 1 && l <= m.tax.len() => m.tax[l - 1][1],
        _ => 0.0,
    }
}

/// An action on the services.
pub(crate) fn apply(
    state: &mut GameState,
    catalog: &Catalog,
    action: Investment,
) -> Result<(), CommandError> {
    let Some(m) = model(catalog) else {
        return Err(CommandError::NotPossible);
    };
    match action {
        Investment::SetHousingManagement { level } => {
            if level as usize > m.housing.len() {
                return Err(CommandError::NotPossible);
            }
            state.person.services.housing = level;
        }
        Investment::SetTaxAdvisor { level } => {
            if level as usize > m.tax.len() {
                return Err(CommandError::NotPossible);
            }
            state.person.services.tax = level;
        }
        Investment::HireAssetManager { strategy, budget } => {
            if strategy as usize >= m.strategies.len() || !(0.0..=1.0).contains(&budget) {
                return Err(CommandError::NotPossible);
            }
            let mut rng = SimRng::for_stream(
                state.settings.seed,
                Stream::Person {
                    month: crate::management::month_number(state.date) + MANAGER_STREAM,
                },
            );
            // Within 0–100; the cast is exact.
            let honesty = (rng.next_f64() * 100.0).round() as u32;
            state.person.services.manager = Some(AssetManager {
                strategy,
                budget,
                honesty,
            });
        }
        Investment::DismissAssetManager => state.person.services.manager = None,
        _ => return Err(CommandError::NotPossible),
    }
    Ok(())
}

/// At a month start: the fees of the advisors; the asset manager invests, charges, and may
/// embezzle.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    let Some(m) = model(catalog).cloned() else {
        return news;
    };
    if state.person.name.is_empty() {
        return news;
    }
    let wage = crate::private::academic_monthly_wage(catalog, state, state.person.home);
    let mut fee = Money::ZERO;
    let tax = state.person.services.tax as usize;
    if tax >= 1 && tax <= m.tax.len() {
        fee += wage.scale(m.tax[tax - 1][0]);
    }
    let Some(manager) = state.person.services.manager else {
        pay_fee(state, fee);
        return news;
    };
    let wealth = crate::private::wealth(catalog, state);
    let managed = wealth.safe + wealth.gold + wealth.funds;
    fee += managed.max(Money::ZERO).scale(m.manager_fee / 12.0);
    pay_fee(state, fee);
    let mut rng = SimRng::for_stream(
        state.settings.seed,
        Stream::Person {
            month: crate::management::month_number(date) + MANAGER_STREAM + 1,
        },
    );
    let honesty = f64::from(manager.honesty) / 100.0;
    if rng.chance(m.dishonest_chance * (1.0 - honesty)) {
        let stolen = wealth.funds.max(Money::ZERO).scale(m.embezzlement);
        let held = state.person.portfolio.funds;
        for (i, pos) in held.iter().enumerate() {
            let take = pos.units * m.embezzlement;
            state.person.portfolio.funds[i].units -= take;
            state.person.portfolio.funds[i].cost = state.person.portfolio.funds[i]
                .cost
                .scale(1.0 - m.embezzlement);
        }
        state.person.services.manager = None;
        news.push(
            Message::new(MessageKind::Warning, keys::MANAGER_EMBEZZLED)
                .with("betrag", Param::Money(stolen)),
        );
        return news;
    }
    invest(state, catalog, &m, manager);
    news
}

fn pay_fee(state: &mut GameState, fee: Money) {
    let paid = fee.min(state.person.account.balance);
    if paid > Money::ZERO {
        crate::private::book_flow(state, PrivateFlow::AdvisorFee, -paid);
    }
}

/// The manager's monthly investment: a share of what the account holds above the reserve,
/// by the weights of his strategy.
pub(crate) fn invest(
    state: &mut GameState,
    catalog: &Catalog,
    m: &ServiceModel,
    manager: AssetManager,
) {
    let level = crate::private::lifestyle_at(catalog, &state.person, state.date);
    let reserve =
        crate::private::lifestyle_cost(catalog, state, level).scale(f64::from(m.reserve_months));
    let free = (state.person.account.balance - reserve).max(Money::ZERO);
    let budget = free.scale(manager.budget);
    if budget <= Money::ZERO {
        return;
    }
    let w = m.strategies[manager.strategy as usize];
    let part = |x: f64| budget.scale(x);
    let months_deposit = catalog
        .person
        .assets
        .as_ref()
        .map_or(12, |a| 12.clamp(a.deposit_months[0], a.deposit_months[1]));
    let months_bond = catalog
        .person
        .assets
        .as_ref()
        .map_or(24, |a| 24.clamp(a.bond_months[0], a.bond_months[1]));
    let fund = if crate::savings::fund_open(catalog, state, FundKind::Index) {
        FundKind::Index
    } else {
        FundKind::Managed
    };
    let steps = [
        Investment::OpenDeposit {
            amount: part(w[0]),
            months: months_deposit,
        },
        Investment::BuyBond {
            face: part(w[1]),
            months: months_bond,
        },
        Investment::BuyFund {
            fund,
            amount: part(w[2]),
        },
        Investment::BuyGold { amount: part(w[3]) },
    ];
    for (i, step) in steps.into_iter().enumerate() {
        if w[i] > 0.0 {
            let _ = crate::savings::apply(state, catalog, step);
        }
    }
}
