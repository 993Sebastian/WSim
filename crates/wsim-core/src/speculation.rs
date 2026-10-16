//! Securities loan and collectors' market of the person (PV7a, docs/FORMELN.md).

use crate::calendar::Date;
use crate::catalog::{Catalog, SpeculationModel};
use crate::command::CommandError;
use crate::message::{Message, MessageKind, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::savings::{FundKind, Investment};
use crate::state::{GameState, PrivateFlow};

const COLLECT_STREAM: u32 = 8 << 24;

pub fn model(catalog: &Catalog) -> Option<&SpeculationModel> {
    catalog.person.speculation.as_ref()
}

/// The price level of collectibles now.
pub fn collector_factor(state: &GameState) -> f64 {
    libm::exp(state.assets.collect_noise)
}

/// What the collectibles of the person fetch in a sale.
pub fn collectibles_value(catalog: &Catalog, state: &GameState) -> Money {
    let (Some(m), Some(s)) = (model(catalog), catalog.person.standing.as_ref()) else {
        return Money::ZERO;
    };
    state
        .person
        .circle
        .goods
        .iter()
        .filter(|g| s.goods.get(g.kind as usize).is_some_and(|t| t.collectible))
        .filter_map(|g| crate::standing::good_price(catalog, state, g.kind as usize))
        .map(|p| p.scale(1.0 - m.collectors.spread))
        .sum()
}

/// The lending value of funds, gold and collectibles.
pub fn capacity(catalog: &Catalog, state: &GameState) -> Money {
    let Some(m) = model(catalog) else {
        return Money::ZERO;
    };
    let l = &m.lombard.lending;
    crate::savings::funds_value(state).scale(l[0])
        + crate::savings::gold_value(state).scale(l[1])
        + collectibles_value(catalog, state).scale(l[2])
}

/// The yearly interest of the loan.
pub fn rate(catalog: &Catalog, state: &GameState) -> f64 {
    model(catalog).map_or(0.0, |m| {
        catalog
            .person
            .savings_rate
            .value(state.person.home, state.date.year_fraction())
            + m.lombard.rate_premium
    })
}

pub(crate) fn apply(
    state: &mut GameState,
    catalog: &Catalog,
    action: Investment,
) -> Result<(), CommandError> {
    if model(catalog).is_none() {
        return Err(CommandError::NotPossible);
    }
    match action {
        Investment::BorrowSecurities { amount } => {
            let room = capacity(catalog, state) - state.person.portfolio.lombard;
            if amount <= Money::ZERO || amount > room {
                return Err(CommandError::NotPossible);
            }
            state.person.portfolio.lombard += amount;
            crate::private::book_flow(state, PrivateFlow::Lombard, amount);
        }
        Investment::RepaySecurities { amount } => {
            let debt = state.person.portfolio.lombard;
            if amount <= Money::ZERO || amount > debt {
                return Err(CommandError::NotPossible);
            }
            if state.person.account.balance < amount {
                return Err(CommandError::NotEnoughPrivateMoney {
                    needed: amount - state.person.account.balance,
                });
            }
            state.person.portfolio.lombard -= amount;
            crate::private::book_flow(state, PrivateFlow::Lombard, -amount);
            if state.person.portfolio.lombard <= capacity(catalog, state) {
                state.person.portfolio.call_since = None;
            }
        }
        _ => return Err(CommandError::NotPossible),
    }
    Ok(())
}

/// At a month start: the collectors' market moves, the loan costs interest, a margin call
/// begins or ends in a forced sale.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    let Some(m) = model(catalog).cloned() else {
        return news;
    };
    let mut rng = SimRng::for_stream(
        state.settings.seed,
        Stream::Person {
            month: crate::management::month_number(date) + COLLECT_STREAM,
        },
    );
    let eps = crate::stock::normal(&mut rng);
    state.assets.collect_noise =
        state.assets.collect_noise * (1.0 - m.collectors.reversion) + m.collectors.volatility * eps;
    let debt = state.person.portfolio.lombard;
    if debt <= Money::ZERO {
        state.person.portfolio.call_since = None;
        return news;
    }
    let interest = debt.scale(rate(catalog, state) / 12.0);
    if state.person.account.balance >= interest {
        crate::private::book_flow(state, PrivateFlow::LombardInterest, -interest);
    } else {
        // What the account cannot pay is added to the loan.
        state.person.portfolio.lombard += interest;
    }
    let cap = capacity(catalog, state);
    if state.person.portfolio.lombard <= cap {
        state.person.portfolio.call_since = None;
        return news;
    }
    match state.person.portfolio.call_since {
        None => {
            state.person.portfolio.call_since = Some(date);
            news.push(
                Message::new(MessageKind::Warning, keys::MARGIN_CALL)
                    .with(
                        "schuld",
                        crate::message::Param::Money(state.person.portfolio.lombard),
                    )
                    .with("deckung", crate::message::Param::Money(cap)),
            );
        }
        Some(since) if date >= since.add_months(m.lombard.call_months) => {
            force_sale(state, catalog, &m);
            state.person.portfolio.call_since = None;
            news.push(
                Message::new(MessageKind::Warning, keys::MARGIN_FORCED).with(
                    "schuld",
                    crate::message::Param::Money(state.person.portfolio.lombard),
                ),
            );
        }
        Some(_) => {}
    }
    news
}

/// Sells funds, gold and collectibles and repays the loan with the proceeds until the cover
/// holds again: just the share that is needed, collectibles whole.
fn force_sale(state: &mut GameState, catalog: &Catalog, m: &SpeculationModel) {
    let l = m.lombard.lending;
    let excess = |s: &GameState| s.person.portfolio.lombard - capacity(catalog, s);
    // The fees of a sale leave a rest; a few rounds close it.
    for _ in 0..4 {
        for fund in FundKind::ALL {
            let over = excess(state);
            if over <= Money::ZERO {
                return;
            }
            let value = Money::from_usd(
                state.person.portfolio.funds[fund as usize].units
                    * crate::savings::nav(state, fund),
            )
            .unwrap_or(Money::ZERO);
            if value <= Money::ZERO {
                continue;
            }
            let share = (over.to_usd() / (1.0 - l[0]) / value.to_usd()).clamp(0.0, 1.0);
            sell_and_repay(state, |s| {
                let _ = crate::savings::apply(s, catalog, Investment::SellFund { fund, share });
            });
        }
        let over = excess(state);
        let gold = crate::savings::gold_value(state);
        if over > Money::ZERO && gold > Money::ZERO {
            let share = (over.to_usd() / (1.0 - l[1]) / gold.to_usd()).clamp(0.0, 1.0);
            sell_and_repay(state, |s| {
                let _ = crate::savings::apply(s, catalog, Investment::SellGold { share });
            });
        }
    }
    while excess(state) > Money::ZERO {
        let Some(i) = state.person.circle.goods.iter().position(|g| {
            catalog
                .person
                .standing
                .as_ref()
                .is_some_and(|s| s.goods.get(g.kind as usize).is_some_and(|t| t.collectible))
        }) else {
            break;
        };
        sell_and_repay(state, |s| {
            let _ = crate::savings::apply(
                s,
                catalog,
                Investment::SellGood {
                    index: u32::try_from(i).unwrap_or(0),
                },
            );
        });
    }
}

/// Runs a sale and puts what it brought against the loan.
fn sell_and_repay(state: &mut GameState, sale: impl FnOnce(&mut GameState)) {
    let before = state.person.account.balance;
    sale(state);
    let proceeds = (state.person.account.balance - before).max(Money::ZERO);
    let pay = state.person.portfolio.lombard.min(proceeds);
    if pay > Money::ZERO {
        state.person.portfolio.lombard -= pay;
        crate::private::book_flow(state, PrivateFlow::Lombard, -pay);
    }
}
