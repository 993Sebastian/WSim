//! Dividends of all companies (PE4, docs/FORMELN.md): a policy per company, paid at the
//! end of January from the closed year within the retained earnings and the liquidity
//! reserve, shared by the owners with the withholding tax of the headquarters' country.
//! The CEO of a company of the person proposes a policy at the year's start; AI
//! companies pay by their character.

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::command::{Command, CommandError};
use crate::decision::{Choice, ChoiceKind, Decision, Topic};
use crate::ledger::{Account, CostCenter, CostType};
use crate::management;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::state::{
    Company, CompanyId, Concern, ConcernOption, ConcernReason, ConcernStatus, GameState, Holder,
    ManagerId, Position, Role, Unit,
};

/// How much of the closed year a company pays out.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum DividendPolicy {
    /// This share (0–1) of the net profit after tax.
    Share(f64),
    /// This amount per year.
    Amount(Money),
}

/// The dividend state of a company.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Dividends {
    /// The policy the company decided; `None`: the default of its kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy: Option<DividendPolicy>,
    /// The last payout: year of the profit and amount, special dividends included.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last: Option<(i32, Money)>,
}

impl Dividends {
    pub fn is_empty(&self) -> bool {
        self.policy.is_none() && self.last.is_none()
    }
}

/// The policy of a company: its own (or the share of K1), `None` for the default.
pub fn policy(company: &Company) -> Option<DividendPolicy> {
    company
        .dividend
        .policy
        .or(company.dividend_payout.map(DividendPolicy::Share))
}

/// Net profit after tax of the last closed year.
pub fn year_profit(company: &Company) -> Money {
    company
        .ledger
        .years
        .last()
        .map(|y| y.by_type.values().copied().sum())
        .unwrap_or_default()
}

/// What the company may distribute: retained earnings and the result of the running year.
pub fn distributable(company: &Company) -> Money {
    let l = &company.ledger;
    (l.balance(Account::RetainedEarnings) + l.balance(Account::Result)).max(Money::ZERO)
}

/// The cash that stays: the strategy's reserve, at least `reserve_monate_min` months of
/// running costs.
pub fn reserve(catalog: &Catalog, state: &GameState, company: CompanyId) -> Money {
    let months = crate::strategy::reserve_months(catalog, state, company, Unit::Board)
        .max(catalog.finance_model.dividends.reserve_months_min);
    crate::strategy::monthly_cost(catalog, state, company).scale(months)
}

/// The most the company can pay out now.
pub fn payable(catalog: &Catalog, state: &GameState, company: CompanyId) -> Money {
    let c = &state.companies[company.index()];
    let free = (c.ledger.cash() - reserve(catalog, state, company)).max(Money::ZERO);
    distributable(c).min(free)
}

/// The share a cautious (0) to aggressive (1) payer proposes.
fn character_share(catalog: &Catalog, aggressiveness: f64) -> f64 {
    let m = &catalog.finance_model.dividends;
    m.payout_min + (m.payout_max - m.payout_min) * (1.0 - aggressiveness.clamp(0.0, 1.0))
}

/// Whether the cash is too short for AI and CEO to propose a dividend.
fn short(catalog: &Catalog, state: &GameState, company: CompanyId) -> bool {
    let c = &state.companies[company.index()];
    let months = catalog.finance_model.dividends.cash_months;
    c.ledger.cash() < crate::strategy::monthly_cost(catalog, state, company).scale(months)
}

/// The share AI and CEO propose for a payer of this aggressiveness: none after a loss or
/// with short cash.
fn proposed_share(catalog: &Catalog, state: &GameState, company: CompanyId, aggr: f64) -> f64 {
    let c = &state.companies[company.index()];
    if year_profit(c) <= Money::ZERO || short(catalog, state, company) {
        0.0
    } else {
        character_share(catalog, aggr)
    }
}

/// The dividend a policy wants for the closed year.
pub fn wanted_by(company: &Company, policy: DividendPolicy) -> Money {
    match policy {
        DividendPolicy::Share(q) => year_profit(company).max(Money::ZERO).scale(q),
        DividendPolicy::Amount(a) => a,
    }
}

/// The dividend the company wants for the closed year: its policy, an AI company without
/// one by its character, others nothing.
pub fn wanted(catalog: &Catalog, state: &GameState, company: CompanyId) -> Money {
    let c = &state.companies[company.index()];
    match (policy(c), &c.ai) {
        (Some(p), _) => wanted_by(c, p),
        (None, Some(ai)) => {
            let q = proposed_share(catalog, state, company, ai.aggressiveness);
            wanted_by(c, DividendPolicy::Share(q))
        }
        (None, None) => Money::ZERO,
    }
}

/// The share the CEO proposes (by his risk appetite).
pub fn ceo_share(catalog: &Catalog, state: &GameState, company: CompanyId, ceo: ManagerId) -> f64 {
    let risk = state.managers.get(&ceo).map_or(50, |m| m.risk);
    proposed_share(catalog, state, company, f64::from(risk) / 100.0)
}

/// What a company's owners get of `amount`.
fn shares(catalog: &Catalog, state: &GameState, company: CompanyId, amount: Money) -> Vec<Part> {
    let c = &state.companies[company.index()];
    let tax = state.countries.get(c.headquarters).dividend_tax;
    let group = catalog.finance_model.dividends.group_share;
    c.owners
        .iter()
        .filter(|s| s.share > 0.0)
        .map(|s| {
            let gross = amount.scale(s.share);
            let withheld = match s.holder {
                Holder::Company(_) if s.share > group => Money::ZERO,
                Holder::Player | Holder::Company(_) => gross.scale(tax),
                Holder::Private | Holder::Investors => Money::ZERO,
            };
            Part {
                holder: s.holder,
                gross,
                tax: withheld,
                group: matches!(s.holder, Holder::Company(_)) && s.share > group,
            }
        })
        .collect()
}

struct Part {
    holder: Holder,
    gross: Money,
    tax: Money,
    /// The holder controls the payer: free of tax into its reserves.
    group: bool,
}

/// What a payout brought the person and the main company.
#[derive(Default)]
struct Paid {
    person_gross: Money,
    person_tax: Money,
    to_main: Money,
}

/// Pays `amount` out of a company for `year` and shares it among its owners.
fn pay(
    catalog: &Catalog,
    state: &mut GameState,
    company: CompanyId,
    (amount, year): (Money, i32),
) -> Paid {
    let mut paid = Paid::default();
    if amount <= Money::ZERO {
        return paid;
    }
    let parts = shares(catalog, state, company, amount);
    let c = &mut state.companies[company.index()];
    c.ledger
        .transfer(Account::RetainedEarnings, Account::Cash, amount);
    c.dividend.last = Some((year, amount));
    if let Some(l) = c.listing.as_mut() {
        l.last_dividend = amount;
    }
    for p in parts {
        match p.holder {
            Holder::Player => {
                paid.person_gross += p.gross;
                paid.person_tax += p.tax;
            }
            Holder::Company(h) => {
                let Some(holder) = state.companies.get_mut(h.index()).filter(|x| !x.bankrupt)
                else {
                    continue;
                };
                let net = p.gross - p.tax;
                if p.group {
                    holder
                        .ledger
                        .transfer(Account::Cash, Account::RetainedEarnings, net);
                } else {
                    holder.ledger.income(
                        CostType::Investments,
                        CostCenter::default(),
                        Account::Cash,
                        net,
                    );
                }
                if state.main_company == Some(h) {
                    paid.to_main += net;
                }
            }
            Holder::Private | Holder::Investors => {}
        }
    }
    if paid.person_gross > Money::ZERO {
        state.stock.player_dividends += paid.person_gross - paid.person_tax;
        crate::private::receive_dividend(state, paid.person_gross, paid.person_tax);
    }
    paid
}

/// End of January (docs/FORMELN.md, PE4): every company pays the dividend for the closed
/// year. Returns the messages for the player.
pub(crate) fn pay_year(state: &mut GameState, catalog: &Catalog, last: Date) -> Vec<Message> {
    let mut news = Vec::new();
    if last.month() != 1 {
        return news;
    }
    let mut to_main = Money::ZERO;
    for i in 0..state.companies.len() {
        let id = company_id(i);
        let c = &state.companies[i];
        if c.bankrupt || c.ledger.years.is_empty() {
            continue;
        }
        let want = wanted(catalog, state, id);
        if want <= Money::ZERO {
            continue;
        }
        let amount = want.min(payable(catalog, state, id));
        let name = state.companies[i].name.clone();
        let paid = pay(catalog, state, id, (amount, last.year() - 1));
        to_main += paid.to_main;
        let main = state.is_main(id);
        if amount < want && main {
            news.push(
                Message::new(MessageKind::Warning, keys::DIVIDEND_CUT)
                    .with("firma", Param::Text(name.clone()))
                    .with("gewollt", Param::Money(want))
                    .with("betrag", Param::Money(amount)),
            );
        }
        if amount <= Money::ZERO {
            continue;
        }
        if main {
            news.push(
                Message::new(MessageKind::Info, keys::DIVIDEND_PAID)
                    .with("firma", Param::Text(name))
                    .with("betrag", Param::Money(amount))
                    .with("privat", Param::Money(paid.person_gross - paid.person_tax))
                    .with("steuer", Param::Money(paid.person_tax)),
            );
        } else if paid.person_gross > Money::ZERO {
            news.push(
                Message::new(MessageKind::Info, keys::DIVIDEND_PERSON)
                    .with("firma", Param::Text(name))
                    .with("privat", Param::Money(paid.person_gross - paid.person_tax))
                    .with("steuer", Param::Money(paid.person_tax)),
            );
        }
    }
    if to_main > Money::ZERO {
        news.push(
            Message::new(MessageKind::Info, keys::DIVIDEND_RECEIVED)
                .with("betrag", Param::Money(to_main)),
        );
    }
    news
}

/// First of January: the CEO of each company of the person proposes the year's
/// dividend; with the topic muted he decides himself.
pub(crate) fn year_start(state: &mut GameState, catalog: &Catalog, next: Date) -> Vec<Message> {
    let mut news = Vec::new();
    if next.ordinal() != 1 || !catalog.management.enabled() {
        return news;
    }
    let ceo = Position::new(Unit::Board, Role::Head);
    for i in 0..state.companies.len() {
        let id = company_id(i);
        let c = &state.companies[i];
        if c.bankrupt || c.ai.is_some() || c.ledger.years.is_empty() {
            continue;
        }
        if !crate::private::is_controlled(state, id) {
            continue;
        }
        let Some(manager) = management::ceo_of(state, id) else {
            continue;
        };
        let open = state.concerns.iter().any(|x| {
            x.company == id
                && x.status == ConcernStatus::Open
                && x.decision.topic == Topic::Dividend
        });
        if open {
            continue;
        }
        let q = ceo_share(catalog, state, id, manager);
        let current = policy(c).unwrap_or(DividendPolicy::Share(0.0));
        let keep = wanted_by(c, current);
        let proposal = wanted_by(c, DividendPolicy::Share(q));
        if proposal == keep {
            continue;
        }
        let set = Command::SetDividendPolicy {
            policy: DividendPolicy::Share(q),
        };
        let muted = management::position_state(state, id, &ceo)
            .is_some_and(|p| p.muted.contains(&Topic::Dividend));
        if muted {
            state.companies[i].dividend.policy = Some(DividendPolicy::Share(q));
            state.companies[i].dividend_payout = None;
            if state.is_main(id) {
                news.push(
                    Message::new(MessageKind::Info, keys::DIVIDEND_CEO_SET)
                        .with("anteil", Param::Number((q * 100.0).round()))
                        .with("betrag", Param::Money(proposal)),
                );
            }
            continue;
        }
        let payable = payable(catalog, state, id);
        let option = |amount: Money| ConcernOption {
            amount: amount.min(payable),
            forecast: None,
            once: Money::ZERO,
        };
        let concern = Concern {
            id: state.next_concern,
            company: id,
            position: ceo.clone(),
            manager,
            decision: Decision {
                topic: Topic::Dividend,
                company: id,
                site: None,
                product: None,
                choices: vec![Choice::keep(), Choice::one(ChoiceKind::Adjust, set)],
                rule: 1,
            },
            recommended: 1,
            options: vec![option(keep), option(proposal)],
            reason: ConcernReason::Dividend,
            path: Vec::new(),
            parts: Vec::new(),
            created: next,
            // Decided before the payout on the last day of January.
            deadline: next.add_months(1).add_days(-2),
            status: ConcernStatus::Open,
            closed: None,
        };
        state.next_concern += 1;
        state.concerns.push(concern);
    }
    news
}

fn company_id(index: usize) -> CompanyId {
    // Few companies; the cast is exact.
    CompanyId(u32::try_from(index).unwrap_or(u32::MAX))
}

/// `SetDividendPolicy`: a share of 0–100 % or an amount of at least zero.
pub(crate) fn set_policy(
    state: &mut GameState,
    actor: CompanyId,
    policy: DividendPolicy,
) -> Result<(), CommandError> {
    match policy {
        DividendPolicy::Share(q) if !(q.is_finite() && (0.0..=1.0).contains(&q)) => {
            return Err(CommandError::ShareOutOfRange { max: 1.0 });
        }
        DividendPolicy::Amount(a) if a.is_negative() => return Err(CommandError::InvalidAmount),
        _ => {}
    }
    let c = &mut state.companies[actor.index()];
    c.dividend.policy = Some(policy);
    c.dividend_payout = None;
    Ok(())
}

/// `SpecialDividend`: paid at once within the retained earnings and the reserve.
pub(crate) fn special(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    amount: Money,
) -> Result<(), CommandError> {
    if amount <= Money::ZERO {
        return Err(CommandError::InvalidAmount);
    }
    let max = payable(catalog, state, actor);
    if amount > max {
        return Err(CommandError::DividendTooHigh { max });
    }
    let year = state.date.year();
    pay(catalog, state, actor, (amount, year));
    Ok(())
}
