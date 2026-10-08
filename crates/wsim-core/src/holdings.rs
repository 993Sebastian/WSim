//! Shares of companies bought and sold by the person and its companies, several companies
//! with a chosen main company, and the end of the game without shares or money (PE5,
//! docs/FORMELN.md).

use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::command::CommandError;
use crate::ledger::Account;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::state::{AiState, CompanyId, GameState, Holder, LifeEvent, LifeEventKind};

/// Who buys: the person or one of its companies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Buyer {
    Person,
    Company(CompanyId),
}

impl Buyer {
    fn holder(self) -> Holder {
        match self {
            Buyer::Person => Holder::Player,
            Buyer::Company(c) => Holder::Company(c),
        }
    }
}

/// How good the earnings are, 0–1: last year's net profit per equity against `rendite_gut`.
pub fn earnings_grade(catalog: &Catalog, state: &GameState, company: CompanyId) -> f64 {
    let c = &state.companies[company.index()];
    let equity = crate::ranking::equity(c);
    if equity <= Money::ZERO {
        return 0.0;
    }
    let r = crate::dividends::year_profit(c).to_usd() / equity.to_usd();
    (r / catalog.deal_model.stakes.good_return).clamp(0.0, 1.0)
}

/// The price a holder asks for `share` of a company.
pub fn ask(catalog: &Catalog, state: &GameState, company: CompanyId, share: f64) -> Money {
    let m = &catalog.deal_model.stakes;
    let g = earnings_grade(catalog, state, company);
    let premium = m.premium_min + (m.premium_max - m.premium_min) * g;
    crate::private::company_value(catalog, state, company).scale(share * (1.0 + premium))
}

/// What investors pay the person for `share` of a company at once.
pub fn bid_of_investors(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    share: f64,
) -> Money {
    let m = &catalog.deal_model.stakes;
    let g = earnings_grade(catalog, state, company);
    let discount = m.discount_max - (m.discount_max - m.discount_min) * g;
    crate::private::company_value(catalog, state, company).scale(share * (1.0 - discount))
}

/// Whether a holder sells shares of a company to the person or its companies: founders,
/// investors and AI companies the person does not control.
pub fn sells(state: &GameState, holder: Holder) -> bool {
    match holder {
        Holder::Private | Holder::Investors => true,
        Holder::Company(h) => state.company(h).is_some_and(|c| {
            c.ai.is_some() && !c.bankrupt && !crate::private::is_controlled(state, h)
        }),
        Holder::Player => false,
    }
}

/// `BuyStake`: `share` of a company from one holder at `price`; at least the holder's ask.
pub(crate) fn buy(
    state: &mut GameState,
    catalog: &Catalog,
    buyer: Buyer,
    (company, holder): (CompanyId, Holder),
    (share, price): (f64, Money),
) -> Result<(), CommandError> {
    let c = state
        .company(company)
        .ok_or(CommandError::UnknownCompany(company))?;
    if c.bankrupt {
        return Err(CommandError::CompanyBankrupt);
    }
    if Buyer::Company(company) == buyer || !sells(state, holder) {
        return Err(CommandError::NotSeller);
    }
    if let Buyer::Company(b) = buyer
        && Holder::Company(b) == holder
    {
        return Err(CommandError::NotSeller);
    }
    if !(share.is_finite() && share > 0.0) {
        return Err(CommandError::InvalidQuantity);
    }
    let held = crate::stock::stake(c, holder);
    if share > held + 1e-9 {
        return Err(CommandError::NotEnoughStock { held });
    }
    let min = ask(catalog, state, company, share);
    if price < min {
        return Err(CommandError::PriceTooLow { min });
    }
    match buyer {
        Buyer::Person => {
            if state.person.account.balance < price {
                return Err(CommandError::NotEnoughPrivateMoney { needed: price });
            }
            crate::private::pay_stake(state, company, price);
        }
        Buyer::Company(b) => {
            let bc = &mut state.companies[b.index()];
            if bc.ledger.cash() < price {
                return Err(CommandError::NotEnoughCash { needed: price });
            }
            bc.ledger
                .transfer(Account::Participations, Account::Cash, price);
            *bc.stock_cost.entry(company).or_default() += price;
        }
    }
    if let Holder::Company(h) = holder {
        crate::stock::realize(
            &mut state.companies[h.index()],
            company,
            (held, share),
            price,
        );
    }
    let owners = &mut state.companies[company.index()].owners;
    crate::stock::remove_share(owners, holder, share.min(held));
    crate::stock::add_share(owners, buyer.holder(), share.min(held));
    take_control(state, catalog, company);
    Ok(())
}

/// A company the person now controls becomes its company: no AI any more, no subsidiary
/// of a company outside.
fn take_control(state: &mut GameState, _catalog: &Catalog, company: CompanyId) -> Vec<Message> {
    let controlled = crate::private::is_controlled(state, company);
    let c = &state.companies[company.index()];
    if !controlled || c.ai.is_none() {
        return Vec::new();
    }
    let outside = c
        .subsidiary_of
        .is_some_and(|s| !crate::private::is_controlled(state, s.parent));
    let name = c.name.clone();
    let today = state.date;
    let c = &mut state.companies[company.index()];
    c.ai = None;
    if outside {
        c.subsidiary_of = None;
    }
    state.person.history.push(LifeEvent {
        date: today,
        kind: LifeEventKind::TookControl { company },
    });
    if state.main_company.is_none() {
        state.main_company = Some(company);
    }
    vec![Message::new(MessageKind::Success, keys::HOLDING_CONTROL).with("firma", Param::Text(name))]
}

/// A company founded from a start-up the person holds the majority of is its own (PE5).
pub(crate) fn take_over_new(state: &mut GameState, company: CompanyId) {
    let today = state.date;
    state.companies[company.index()].ai = None;
    state.person.history.push(LifeEvent {
        date: today,
        kind: LifeEventKind::TookControl { company },
    });
    if state.main_company.is_none() {
        state.main_company = Some(company);
    }
}

/// A company the person no longer controls becomes an AI company with a character drawn
/// from its own stream (as a new AI company, SU2); the person leaves its chair, and the
/// main company passes on.
pub(crate) fn lose_control(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
) -> Vec<Message> {
    let mut news = Vec::new();
    if crate::private::is_controlled(state, company) {
        return news;
    }
    let date = state.date;
    let base = state.settings.ai;
    let spread = catalog.ai_model.trait_spread;
    let c = &mut state.companies[company.index()];
    if c.ai.is_none() && !c.bankrupt {
        let mut vary = |b: f64| (b + spread * (2.0 * c.rng.next_f64() - 1.0)).clamp(0.0, 1.0);
        let competence = vary(base.competence);
        let aggressiveness = vary(base.aggressiveness);
        c.ai = Some(AiState {
            competence,
            aggressiveness,
            real: None,
            next_operations: date.next_day(),
            staff: 0.0,
        });
        news.push(
            Message::new(MessageKind::Info, keys::HOLDING_LOST)
                .with("firma", Param::Text(c.name.clone())),
        );
    }
    if state.person.ceo == Some(company) {
        state.person.ceo = None;
    }
    if state.main_company == Some(company) {
        news.extend(next_main(state));
    }
    news
}

/// The companies the person may choose as main company: controlled, its own (no AI).
pub fn selectable(state: &GameState) -> Vec<CompanyId> {
    crate::private::controlled(state)
        .into_iter()
        .filter(|&c| state.companies[c.index()].ai.is_none())
        .collect()
}

/// The next company the person controls becomes the main company, or none.
pub(crate) fn next_main(state: &mut GameState) -> Vec<Message> {
    let next = selectable(state).into_iter().next();
    state.main_company = next;
    vec![match next {
        Some(c) => Message::new(MessageKind::Warning, keys::MAIN_COMPANY_CHANGED).with(
            "firma",
            Param::Text(state.companies[c.index()].name.clone()),
        ),
        None => Message::new(MessageKind::Warning, keys::MAIN_COMPANY_NONE),
    }]
}

/// `SelectCompany`: another company of the person becomes the main company.
pub(crate) fn select(state: &mut GameState, company: CompanyId) -> Result<(), CommandError> {
    state
        .company(company)
        .ok_or(CommandError::UnknownCompany(company))?;
    if !selectable(state).contains(&company) {
        return Err(CommandError::NotControlled);
    }
    state.main_company = Some(company);
    Ok(())
}

/// `SellStake`: the person sells `share` of a company to investors at once; the gain over
/// the cost basis is taxed at home.
pub(crate) fn sell(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    share: f64,
) -> Result<(), CommandError> {
    let c = state
        .company(company)
        .ok_or(CommandError::UnknownCompany(company))?;
    if c.bankrupt {
        return Err(CommandError::CompanyBankrupt);
    }
    if !(share.is_finite() && share > 0.0) {
        return Err(CommandError::InvalidQuantity);
    }
    let held = crate::stock::stake(c, Holder::Player);
    if share > held + 1e-9 {
        return Err(CommandError::NotEnoughStock { held });
    }
    let share = share.min(held);
    let proceeds = bid_of_investors(catalog, state, company, share);
    let basis = state
        .person
        .cost_basis
        .get(&company)
        .copied()
        .unwrap_or_default()
        .scale(share / held);
    let rate = state.countries.get(state.person.home).dividend_tax;
    let tax = (proceeds - basis).max(Money::ZERO).scale(rate);
    crate::private::receive_sale(state, company, (proceeds, basis, tax));
    let today = state.date;
    state.person.history.push(LifeEvent {
        date: today,
        kind: LifeEventKind::Sold {
            company,
            share,
            proceeds,
        },
    });
    let owners = &mut state.companies[company.index()].owners;
    crate::stock::remove_share(owners, Holder::Player, share);
    crate::stock::add_share(owners, Holder::Investors, share);
    // Companies it controlled through this one may be lost too.
    for i in 0..state.companies.len() {
        // Few companies; the cast is exact.
        let id = CompanyId(i as u32);
        if state.companies[i].ai.is_none() && !state.companies[i].bankrupt {
            lose_control(state, catalog, id);
        }
    }
    Ok(())
}

/// Whether the person holds any share of a company or start-up.
fn holds_anything(state: &GameState) -> bool {
    state
        .companies
        .iter()
        .any(|c| !c.bankrupt && crate::stock::stake(c, Holder::Player) > 0.0)
        || state.ventures.iter().any(|v| {
            v.status == crate::state::VentureStatus::Active
                && (v.person_pledge > Money::ZERO
                    || v.owners.iter().any(|s| s.holder == Holder::Player))
        })
}

/// End of a month (docs/FORMELN.md, PE5): without shares and with less money than the
/// cheapest founding the game ends.
pub(crate) fn month_end(state: &mut GameState, catalog: &Catalog, _last: Date) -> Vec<Message> {
    if state.game_over || holds_anything(state) {
        return Vec::new();
    }
    let min = crate::private::start_minimum(catalog, state, state.person.home);
    let balance = state.person.account.balance;
    if balance >= min && balance > Money::ZERO {
        return Vec::new();
    }
    state.game_over = true;
    vec![Message::new(MessageKind::Crisis, keys::GAME_OVER_BROKE).with("betrag", Param::Money(min))]
}
