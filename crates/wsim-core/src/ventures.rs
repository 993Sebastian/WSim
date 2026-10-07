//! Start-ups (SU1–SU3; formulas in docs/FORMELN.md).
//!
//! Inventors and young companies outside the game work on a technology not yet invented
//! or on the next development level of a product. Every phase needs a funding round of
//! new shares; most of them fail. A success brings the technology or the level into the
//! world earlier. Companies take stakes, give grants, steer and integrate them (SU2); they
//! spin off research projects, and AI companies take part by the same commands (SU3).

use std::collections::BTreeSet;

use crate::calendar::Date;
use crate::catalog::{Catalog, DepartmentKind, SiteType, VentureModel};
use crate::command::{Command, CommandError};
use crate::decision::{self, Choice, ChoiceKind, Decider, Decision, Topic};
use crate::ids::{CountryId, ProductId, TechnologyId};
use crate::ledger::{Account, CostCenter, CostType};
use crate::management;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{
    CompanyId, GameState, Holder, SiteId, Stake, Venture, VentureExit, VentureFailure, VenturePace,
    VentureStatus, VentureTarget,
};

/// Start-ups at the start of a month: rounds and phases of the active ones, then the new
/// ones, then closed ones leave the list. Returns messages for the player.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let m = &catalog.ventures;
    if m.phases.is_empty() {
        return Vec::new();
    }
    let month = management::month_number(date);
    let mut messages = Vec::new();
    for i in 0..state.ventures.len() {
        if state.ventures[i].status != VentureStatus::Active {
            continue;
        }
        let stream = Stream::Venture {
            id: state.ventures[i].id,
            month,
        };
        let mut rng = SimRng::for_stream(state.settings.seed, stream);
        messages.extend(advance(state, catalog, i, date, &mut rng));
    }
    found(state, catalog, date, month);
    prune(state, m, date);
    messages
}

/// Chance of a phase for a start-up with a lead of `lead` years.
pub fn phase_chance(m: &VentureModel, phase: usize, lead: f64) -> f64 {
    m.phases.get(phase).map_or(0.0, |p| {
        (p.chance * (1.0 - m.lead_chance * lead)).max(m.chance_min)
    })
}

/// Chance of success from the current phase on: the chance of the current phase, fixed
/// at its start, times those of the phases still to come.
pub fn success_chance(m: &VentureModel, v: &Venture) -> f64 {
    if v.status != VentureStatus::Active {
        return 0.0;
    }
    (v.phase + 1..m.phases.len())
        .map(|p| phase_chance(m, p, v.lead))
        .fold(v.chance, |a, c| a * c)
}

/// How well a company's strategy department sees start-ups: quality times coverage
/// (ZA2); `None` where it does not work.
pub fn insight(catalog: &Catalog, state: &GameState, company: CompanyId) -> Option<f64> {
    if state.companies[company.index()].departments.is_empty() {
        return None;
    }
    crate::central::performance(catalog, state, company, DepartmentKind::Strategy)
        .map(|p| p.quality * p.coverage)
}

/// What a company sees of the chance of success, and whether its strategy department
/// works on it: blurred by the draw of the start-up, the less the better the department
/// works. Without a working department the company sees only the level.
pub fn shown_chance(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    v: &Venture,
) -> (f64, bool) {
    let m = &catalog.ventures;
    let work = insight(catalog, state, company);
    let blur = m.blur * (1.0 - work.unwrap_or(0.0));
    let shown = (success_chance(m, v) * (1.0 + v.blur * blur)).clamp(0.0, 1.0);
    (shown, work.is_some())
}

/// Whether the world got to the target of a start-up first: the technology is invented,
/// in history or in the game, or someone reached the level.
fn overtaken(state: &GameState, catalog: &Catalog, target: VentureTarget, date: Date) -> bool {
    match target {
        VentureTarget::Technology(t) => {
            state.inventions.get(t).is_some()
                || catalog.technologies.get(t).invention_year <= date.year()
        }
        VentureTarget::Development { product, level } => {
            state.developments.get(product).len() >= usize::from(level)
        }
    }
}

/// One month of an active start-up: it is overtaken, a parent or investors fund its open
/// round or it runs out of time, or its funded phase is decided.
fn advance(
    state: &mut GameState,
    catalog: &Catalog,
    i: usize,
    date: Date,
    rng: &mut SimRng,
) -> Vec<Message> {
    let m = &catalog.ventures;
    if overtaken(state, catalog, state.ventures[i].target, date) {
        return fail(state, catalog, i, date, VentureFailure::Overtaken);
    }
    if let Some(until) = state.ventures[i].round_until {
        // A subsidiary's parent pledges its share of the round, as far as its cash goes;
        // investors cover the rest (SU3).
        if let Some(parent) = state.ventures[i].parent {
            let v = &state.ventures[i];
            let rest = v.capital - v.raised;
            let share = share_of(v, parent);
            let part = if share >= 1.0 - 1e-9 {
                rest
            } else {
                (v.capital.scale(share) - amount_of(&v.pledges, parent)).min(rest)
            };
            if part > Money::ZERO && state.companies[parent.index()].ledger.cash() >= part {
                pledge(state, i, parent, part);
                crate::central::count_purchase(state, parent, part);
            }
            if state.ventures[i].raised >= state.ventures[i].capital {
                close_round(m, &mut state.ventures[i], date);
                return Vec::new();
            }
        }
        if rng.chance(m.investor_chance) {
            let v = &mut state.ventures[i];
            let rest = v.capital - v.raised;
            v.pledges.retain(|&(_, a)| a > Money::ZERO);
            close_round_with(m, v, date, rest);
            keep_parent(m, v);
        } else if date >= until {
            return fail(state, catalog, i, date, VentureFailure::Funding);
        }
        return Vec::new();
    }
    let v = &state.ventures[i];
    if v.phase_until.is_none_or(|until| date < until) {
        return Vec::new();
    }
    if !rng.chance(v.chance) {
        return fail(state, catalog, i, date, VentureFailure::Phase);
    }
    let next = v.phase + 1;
    if next < m.phases.len() {
        let gdp = state
            .countries
            .get(state.ventures[i].country)
            .gdp_per_capita_usd;
        start_phase(m, &mut state.ventures[i], next, gdp, date);
        return Vec::new();
    }
    succeed(state, catalog, i, date)
}

/// Months and chance factors of a pace (SU2).
fn pace_factors(m: &VentureModel, pace: VenturePace) -> (f64, f64) {
    match pace {
        VenturePace::Normal => (1.0, 1.0),
        VenturePace::Fast => m.stakes.fast,
        VenturePace::Thorough => m.stakes.thorough,
    }
}

/// A round covered by the pledges, and investors for the rest: shares are issued and the
/// phase begins today (SU1, SU2).
fn close_round(m: &VentureModel, v: &mut Venture, date: Date) {
    let rest = (v.capital - v.raised).max(Money::ZERO);
    close_round_with(m, v, date, rest);
}

fn close_round_with(m: &VentureModel, v: &mut Venture, date: Date, investors: Money) {
    let valuation = m.phases.get(v.phase).map_or(1.0, |p| p.valuation);
    let mut givers: Vec<(Holder, Money)> = v
        .pledges
        .iter()
        .map(|&(c, a)| (Holder::Company(c), a))
        .collect();
    if investors > Money::ZERO {
        givers.push((Holder::Investors, investors));
    }
    issue(v, &givers, valuation);
    v.pledges.clear();
    v.raised = v.capital;
    v.round_until = None;
    let (months_factor, _) = pace_factors(m, v.pace);
    let months = m.phases.get(v.phase).map_or(0, |p| p.months);
    // Phases last a few months; the cast is exact.
    let months = (f64::from(months) * months_factor).round().max(1.0) as u32;
    v.phase_until = Some(date.add_months(months));
}

/// A parent diluted to the majority or below no longer holds a subsidiary.
fn keep_parent(m: &VentureModel, v: &mut Venture) {
    if let Some(p) = v.parent
        && share_of(v, p) <= m.stakes.majority
    {
        v.parent = None;
    }
}

/// A company pledges to the open round: its cash goes into its financial assets.
fn pledge(state: &mut GameState, i: usize, company: CompanyId, amount: Money) {
    state.companies[company.index()].ledger.transfer(
        Account::Participations,
        Account::Cash,
        amount,
    );
    let v = &mut state.ventures[i];
    add_to(&mut v.pledges, company, amount);
    add_to(&mut v.book, company, amount);
    v.raised += amount;
}

/// The start-up fails: pledges flow back, stakes are written off, and the majority owner
/// keeps part of the research (SU2).
fn fail(
    state: &mut GameState,
    catalog: &Catalog,
    i: usize,
    date: Date,
    why: VentureFailure,
) -> Vec<Message> {
    let m = &catalog.ventures;
    let player = state.player;
    let majority = state.ventures[i]
        .parent
        .or_else(|| majority_company(m, &state.ventures[i]));
    let v = &mut state.ventures[i];
    v.status = VentureStatus::Failed(date, why);
    v.round_until = None;
    v.phase_until = None;
    let (name, target) = (v.name.clone(), v.target);
    let pledges = std::mem::take(&mut v.pledges);
    let mut messages = Vec::new();
    for (c, amount) in pledges {
        state.companies[c.index()]
            .ledger
            .transfer(Account::Cash, Account::Participations, amount);
        take_from(&mut state.ventures[i].book, c, amount);
        if c == player {
            messages.push(
                Message::new(MessageKind::Info, keys::VENTURE_REFUND)
                    .with("name", Param::Text(name.clone()))
                    .with("betrag", Param::Money(amount)),
            );
        }
    }
    let book = std::mem::take(&mut state.ventures[i].book);
    for (c, value) in book {
        if value <= Money::ZERO {
            continue;
        }
        state.companies[c.index()].ledger.expense(
            CostType::Investments,
            CostCenter::default(),
            Account::Participations,
            value,
        );
        if c == player {
            messages.push(
                Message::new(MessageKind::Warning, keys::VENTURE_LOST)
                    .with("name", Param::Text(name.clone()))
                    .with("betrag", Param::Money(value)),
            );
        }
    }
    if let Some(c) = majority
        && research_bonus(state, catalog, c, target, date)
        && c == player
    {
        messages.push(
            Message::new(MessageKind::Info, keys::VENTURE_BONUS)
                .with("name", Param::Text(name))
                .with("ziel", target_param(catalog, target)),
        );
    }
    messages
}

/// Part of the research effort of the target for the majority owner of a failed
/// start-up; false where it has nothing to gain.
fn research_bonus(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    target: VentureTarget,
    date: Date,
) -> bool {
    let bonus = catalog.ventures.stakes.research_bonus;
    if bonus <= 0.0 {
        return false;
    }
    match target {
        VentureTarget::Technology(t) => {
            if state.knows(catalog, company, t) {
                return false;
            }
            let Some(e) = crate::research::effort(catalog, state, t, date) else {
                return false;
            };
            *state.companies[company.index()]
                .research
                .entry(t)
                .or_default() += e.points * bonus;
            true
        }
        VentureTarget::Development { product, level } => {
            match crate::development::next_effort(catalog, state, company, product, date) {
                Some((next, points)) if next == level => {
                    *state.companies[company.index()]
                        .development
                        .points
                        .entry(product)
                        .or_default() += points * bonus;
                    true
                }
                _ => false,
            }
        }
    }
}

/// The text of a start-up's target: its technology or product.
fn target_param(catalog: &Catalog, target: VentureTarget) -> Param {
    match target {
        VentureTarget::Technology(t) => {
            Param::TextKey(format!("technologie.{}", catalog.technologies.key(t)))
        }
        VentureTarget::Development { product, .. } => {
            Param::TextKey(format!("produkt.{}", catalog.products.key(product)))
        }
    }
}

/// A phase begins: its capital and chance are fixed and its round opens.
fn start_phase(m: &VentureModel, v: &mut Venture, phase: usize, gdp_per_capita: f64, date: Date) {
    let Some(p) = m.phases.get(phase) else {
        return;
    };
    let (low, high) = m.capital_factor;
    let income = (gdp_per_capita / m.reference_gdp_usd).clamp(low, high);
    v.phase = phase;
    v.capital = p.capital.scale(income * (1.0 + m.lead_capital * v.lead));
    let (_, chance_factor) = pace_factors(m, v.pace);
    v.chance = (phase_chance(m, phase, v.lead) * chance_factor).min(m.stakes.chance_max);
    v.raised = Money::ZERO;
    v.round_until = Some(date.add_months(m.deadline_months));
    v.phase_until = None;
}

/// A funded round: the givers get shares for their amounts at the value after the round,
/// all earlier owners keep the value before it.
fn issue(v: &mut Venture, givers: &[(Holder, Money)], valuation: f64) {
    let before = v.capital.to_usd() * valuation;
    let raised: f64 = givers.iter().map(|(_, a)| a.to_usd()).sum();
    let after = before + raised;
    if after <= 0.0 || raised <= 0.0 {
        return;
    }
    let keep = before / after;
    for s in &mut v.owners {
        s.share *= keep;
    }
    for &(holder, amount) in givers {
        let share = amount.to_usd() / after;
        match v.owners.iter_mut().find(|s| s.holder == holder) {
            Some(s) => s.share += share,
            None => v.owners.push(Stake { holder, share }),
        }
    }
}

/// All phases passed: the technology counts as invented from today if that is earlier
/// than so far, or the level as reached; then the owners settle (SU2).
fn succeed(state: &mut GameState, catalog: &Catalog, i: usize, date: Date) -> Vec<Message> {
    let v = &mut state.ventures[i];
    v.status = VentureStatus::Succeeded(date);
    v.phase = catalog.ventures.phases.len();
    v.round_until = None;
    v.phase_until = None;
    let (name, country, target) = (v.name.clone(), v.country, v.target);
    let named = |key: &str| {
        Message::new(MessageKind::Info, key)
            .with("name", Param::Text(name.clone()))
            .with(
                "land",
                Param::Country(catalog.countries.key(country).to_owned()),
            )
    };
    let mut messages = Vec::new();
    match target {
        VentureTarget::Technology(t) => {
            let historical = f64::from(catalog.technologies.get(t).invention_year);
            let first = state.inventions.get_mut(t);
            if first.is_none() && date.year_fraction() < historical {
                *first = Some(date);
                messages.push(named(keys::VENTURE_INVENTION).with(
                    "technologie",
                    Param::TextKey(format!("technologie.{}", catalog.technologies.key(t))),
                ));
            }
        }
        VentureTarget::Development { product, level } => {
            let firsts = state.developments.get_mut(product);
            if firsts.len() + 1 == usize::from(level) {
                firsts.push(date);
                if crate::research::player_offers(state, product) {
                    messages.push(
                        named(keys::VENTURE_DEVELOPMENT)
                            .with(
                                "produkt",
                                Param::TextKey(format!(
                                    "produkt.{}",
                                    catalog.products.key(product)
                                )),
                            )
                            .with("stufe", Param::Integer(i64::from(level)))
                            .with(
                                "jahre",
                                Param::Number(
                                    catalog.research_model.development.public_domain_years,
                                ),
                            ),
                    );
                }
            }
        }
    }
    messages.extend(settle_success(state, catalog, i, date));
    messages
}

/// The owners of a successful start-up (SU2): a parent takes it in; a company with the
/// majority buys out the others and becomes its parent if its cash allows; else it goes
/// public, every owner gets the value of its share and AI companies get a new rival.
fn settle_success(state: &mut GameState, catalog: &Catalog, i: usize, date: Date) -> Vec<Message> {
    let m = &catalog.ventures;
    let player = state.player;
    let v = &state.ventures[i];
    let worth = success_value(m, v);
    // The parent or the majority, if it can pay the others.
    let parent = v.parent.or_else(|| majority_company(m, v)).filter(|&c| {
        let others = worth.scale((1.0 - share_of(v, c)).max(0.0));
        state.companies[c.index()].ledger.cash() >= others
    });
    let mut messages = Vec::new();
    if let Some(p) = parent {
        // The others are paid out at the value of the success.
        let paid = buy_out(state, i, p, worth, &mut messages);
        if paid > Money::ZERO && p == player {
            messages.push(
                Message::new(MessageKind::Info, keys::VENTURE_PAID_OUT)
                    .with("name", Param::Text(state.ventures[i].name.clone()))
                    .with("betrag", Param::Money(paid)),
            );
        }
        messages.extend(to_parent(state, catalog, i, p));
        return messages;
    }
    // On the stock market: every company gets the value of its share.
    let owners: Vec<(CompanyId, f64)> = companies_of(&state.ventures[i]);
    for (c, share) in owners {
        let proceeds = worth.scale(share);
        exit_stake(state, i, c, share, proceeds);
        if c == player {
            messages.push(
                Message::new(MessageKind::Success, keys::VENTURE_LISTED)
                    .with("name", Param::Text(state.ventures[i].name.clone()))
                    .with("betrag", Param::Money(proceeds)),
            );
        }
    }
    let v = &state.ventures[i];
    let (target, country, name) = (v.target, v.country, v.name.clone());
    let company = crate::ai::found_from_venture(state, catalog, date, target, country, &name);
    state.ventures[i].exit = Some(VentureExit::Listed(company));
    if let Some(c) = company {
        messages.push(
            Message::new(MessageKind::Info, keys::VENTURE_NEW_COMPANY)
                .with("name", Param::Text(name))
                .with(
                    "land",
                    Param::Country(catalog.countries.key(country).to_owned()),
                )
                .with(
                    "firma",
                    Param::Text(state.companies[c.index()].name.clone()),
                )
                .with("ziel", target_param(catalog, target)),
        );
    }
    messages
}

/// A company buys all other owners out at `worth` for the whole start-up: companies get
/// their part as proceeds, founders and investors leave. Returns what it paid.
fn buy_out(
    state: &mut GameState,
    i: usize,
    buyer: CompanyId,
    worth: Money,
    messages: &mut Vec<Message>,
) -> Money {
    let player = state.player;
    let others: Vec<Stake> = state.ventures[i]
        .owners
        .iter()
        .copied()
        .filter(|s| s.holder != Holder::Company(buyer))
        .collect();
    let mut paid = Money::ZERO;
    for s in others {
        let price = worth.scale(s.share);
        paid += price;
        if let Holder::Company(c) = s.holder {
            exit_stake(state, i, c, s.share, price);
            if c == player {
                messages.push(
                    Message::new(MessageKind::Info, keys::VENTURE_BOUGHT_OUT)
                        .with("name", Param::Text(state.ventures[i].name.clone()))
                        .with(
                            "firma",
                            Param::Text(state.companies[buyer.index()].name.clone()),
                        )
                        .with("betrag", Param::Money(price)),
                );
            }
        }
    }
    if paid > Money::ZERO {
        state.companies[buyer.index()].ledger.transfer(
            Account::Participations,
            Account::Cash,
            paid,
        );
    }
    let v = &mut state.ventures[i];
    add_to(&mut v.book, buyer, paid);
    v.owners = Stake::sole(Holder::Company(buyer));
    // Pledges of others to an open round flow back.
    let pledges: Vec<(CompanyId, Money)> = v
        .pledges
        .iter()
        .copied()
        .filter(|&(c, _)| c != buyer)
        .collect();
    for (c, amount) in pledges {
        let v = &mut state.ventures[i];
        v.pledges.retain(|&(x, _)| x != c);
        v.raised -= amount;
        take_from(&mut v.book, c, amount);
        state.companies[c.index()]
            .ledger
            .transfer(Account::Cash, Account::Participations, amount);
    }
    paid
}

/// The parent of a successful start-up uses its technology or level from today; what it
/// paid for it was research.
fn to_parent(
    state: &mut GameState,
    catalog: &Catalog,
    i: usize,
    parent: CompanyId,
) -> Vec<Message> {
    let v = &mut state.ventures[i];
    v.parent = Some(parent);
    v.exit = Some(VentureExit::Parent(parent));
    let (target, name) = (v.target, v.name.clone());
    let book = take_all(&mut v.book, parent);
    let company = &mut state.companies[parent.index()];
    if book > Money::ZERO {
        company.ledger.expense(
            CostType::Research,
            CostCenter::default(),
            Account::Participations,
            book,
        );
    }
    match target {
        VentureTarget::Technology(t) => {
            company.research.remove(&t);
            company.technologies.insert(t);
        }
        VentureTarget::Development { product, level } => {
            let own = company.development.levels.entry(product).or_default();
            *own = (*own).max(level);
            company.development.points.remove(&product);
        }
    }
    if parent != state.player {
        return Vec::new();
    }
    vec![
        Message::new(MessageKind::Success, keys::VENTURE_PARENT)
            .with("name", Param::Text(name))
            .with("ziel", target_param(catalog, target)),
    ]
}

/// A company leaves a start-up with `share` of it for `proceeds`: its book value of that
/// part leaves the financial assets, the difference is its gain or loss.
fn exit_stake(state: &mut GameState, i: usize, company: CompanyId, share: f64, proceeds: Money) {
    let v = &mut state.ventures[i];
    let held = share_of(v, company);
    if held <= 0.0 {
        return;
    }
    let part = (share / held).min(1.0);
    let book = amount_of(&v.book, company).scale(part);
    take_from(&mut v.book, company, book);
    remove_share(v, Holder::Company(company), share);
    let ledger = &mut state.companies[company.index()].ledger;
    ledger.transfer(Account::Cash, Account::Participations, book);
    let center = CostCenter::default();
    if proceeds > book {
        ledger.income(
            CostType::Investments,
            center,
            Account::Cash,
            proceeds - book,
        );
    } else if proceeds < book {
        ledger.expense(
            CostType::Investments,
            center,
            Account::Cash,
            book - proceeds,
        );
    }
}

/// Technologies a new start-up could work on, with their lead in years: neither invented
/// in history nor in the game, all prerequisites invented, at most the lead of the data
/// ahead and no target of an active start-up.
fn new_technologies(state: &GameState, catalog: &Catalog, date: Date) -> Vec<(TechnologyId, f64)> {
    let m = &catalog.ventures;
    let year = date.year();
    let invented = |t: TechnologyId| {
        state.inventions.get(t).is_some() || catalog.technologies.get(t).invention_year <= year
    };
    let targeted: BTreeSet<TechnologyId> = state
        .ventures
        .iter()
        .filter(|v| v.status == VentureStatus::Active)
        .filter_map(|v| match v.target {
            VentureTarget::Technology(t) => Some(t),
            VentureTarget::Development { .. } => None,
        })
        .collect();
    let max = i64::from(m.lead_years_max);
    catalog
        .technologies
        .iter()
        .filter(|(id, t)| {
            let lead = i64::from(t.invention_year) - i64::from(year);
            t.research_effort.is_some()
                && lead <= max
                && !invented(*id)
                && !targeted.contains(id)
                && t.prerequisites.iter().all(|&p| invented(p))
        })
        .map(|(id, t)| (id, f64::from(t.invention_year - year)))
        .collect()
}

/// Products a new start-up could improve, with the level: those that can be developed
/// and are made in the world, whose next level no one has reached and no active start-up
/// works on.
fn improvements(state: &GameState, catalog: &Catalog, date: Date) -> Vec<(ProductId, u8)> {
    let levels = catalog.research_model.development.levels;
    let made: BTreeSet<ProductId> = state
        .sites
        .iter()
        .filter(|s| !state.companies[s.owner.index()].bankrupt)
        .flat_map(|s| s.slots.iter())
        .filter(|sl| sl.operating(date))
        .filter_map(|sl| sl.recipe)
        .map(|r| catalog.recipes.get(r).product)
        .collect();
    let targeted: BTreeSet<ProductId> = state
        .ventures
        .iter()
        .filter(|v| v.status == VentureStatus::Active)
        .filter_map(|v| match v.target {
            VentureTarget::Development { product, .. } => Some(product),
            VentureTarget::Technology(_) => None,
        })
        .collect();
    made.into_iter()
        .filter(|p| !targeted.contains(p))
        .filter_map(|p| {
            let next = u8::try_from(state.developments.get(p).len() + 1).ok()?;
            (next <= levels && crate::development::basis(catalog, p).is_some()).then_some((p, next))
        })
        .collect()
}

/// New start-ups of a month (docs/FORMELN.md, SU1).
fn found(state: &mut GameState, catalog: &Catalog, date: Date, month: u32) {
    let m = &catalog.ventures;
    let mean = m.per_year * state.settings.ventures / 12.0;
    if mean.is_nan() || mean <= 0.0 {
        return;
    }
    let mut rng = SimRng::for_stream(state.settings.seed, Stream::Ventures { month });
    let whole = mean.floor();
    // A handful a month; the cast is exact.
    let count = whole as usize + usize::from(rng.chance(mean - whole));
    if count == 0 {
        return;
    }
    let mut technologies = new_technologies(state, catalog, date);
    let mut products = improvements(state, catalog, date);
    let countries: Vec<(CountryId, f64)> = catalog
        .countries
        .ids()
        .map(|c| {
            let v = state.countries.get(c);
            (
                c,
                (v.population * v.gdp_per_capita_usd * v.development).max(0.0),
            )
        })
        .collect();
    if countries.is_empty() {
        return;
    }
    let total: f64 = countries.iter().map(|c| c.1).sum();
    let mut names: BTreeSet<String> = state.ventures.iter().map(|v| v.name.clone()).collect();
    for _ in 0..count {
        let new = rng.chance(m.new_share);
        let pick = |rng: &mut SimRng, n: usize| {
            usize::try_from(rng.below(u64::try_from(n).unwrap_or(1))).unwrap_or(0)
        };
        let (target, lead) = if (new || products.is_empty()) && !technologies.is_empty() {
            let (t, lead) = technologies.remove(pick(&mut rng, technologies.len()));
            (VentureTarget::Technology(t), lead)
        } else if !products.is_empty() {
            let (product, level) = products.remove(pick(&mut rng, products.len()));
            (VentureTarget::Development { product, level }, 0.0)
        } else {
            return;
        };
        let inventor = match target {
            VentureTarget::Technology(t) => m
                .inventors
                .iter()
                .find(|x| x.technology == t && !names.contains(&x.name)),
            VentureTarget::Development { .. } => None,
        };
        let (name, country) = match inventor {
            Some(x) => {
                names.insert(x.name.clone());
                (x.name.clone(), x.country)
            }
            None => {
                let country = management::pick_country(&mut rng, &countries, total);
                let name = management::manager_name(catalog, &mut rng, country, &mut names);
                (name, country)
            }
        };
        let blur = 2.0 * rng.next_f64() - 1.0;
        let mut v = Venture {
            id: state.next_venture,
            name,
            inventor: inventor.is_some(),
            country,
            target,
            lead,
            founded: date,
            phase: 0,
            capital: Money::ZERO,
            chance: 0.0,
            raised: Money::ZERO,
            round_until: None,
            phase_until: None,
            owners: Stake::sole(Holder::Private),
            status: VentureStatus::Active,
            blur,
            pledges: Vec::new(),
            book: Vec::new(),
            grants: Vec::new(),
            parent: None,
            pace: VenturePace::Normal,
            exit: None,
            origin: None,
        };
        let gdp = state.countries.get(country).gdp_per_capita_usd;
        start_phase(m, &mut v, 0, gdp, date);
        state.next_venture += 1;
        state.ventures.push(v);
    }
}

/// Closed start-ups leave the list after the years of the data; those of historical
/// inventors stay, so that each founds only once.
fn prune(state: &mut GameState, m: &VentureModel, date: Date) {
    let months = m.keep_years.saturating_mul(12);
    state.ventures.retain(|v| match v.status {
        VentureStatus::Active => true,
        VentureStatus::Succeeded(d) | VentureStatus::Failed(d, _) => {
            v.inventor || d.add_months(months) > date
        }
    });
}

/// Value of a whole start-up (SU2): before its open round, else after the last one; at
/// success the value after the last round times the success factor; nothing once failed.
pub fn value(m: &VentureModel, v: &Venture) -> Money {
    let valuation = m.phases.get(v.phase).map_or(1.0, |p| p.valuation);
    match v.status {
        VentureStatus::Active if v.round_until.is_some() => v.capital.scale(valuation),
        VentureStatus::Active => v.capital.scale(valuation + 1.0),
        VentureStatus::Succeeded(_) => success_value(m, v),
        VentureStatus::Failed(..) => Money::ZERO,
    }
}

/// Value at success: the capital of the last phase after its round times the success
/// factor (the capital of the current phase until then).
pub fn success_value(m: &VentureModel, v: &Venture) -> Money {
    let last = m.phases.len().saturating_sub(1);
    let valuation = m.phases.get(last).map_or(1.0, |p| p.valuation);
    v.capital.scale((valuation + 1.0) * m.stakes.success_factor)
}

/// The value a success would have, judged today: the last phase's capital as the country's
/// income and the lead set it now.
pub fn expected_success_value(state: &GameState, m: &VentureModel, v: &Venture) -> Money {
    let last = m.phases.len().saturating_sub(1);
    if v.phase >= last {
        return success_value(m, v);
    }
    let Some(p) = m.phases.get(last) else {
        return Money::ZERO;
    };
    let (low, high) = m.capital_factor;
    let gdp = state.countries.get(v.country).gdp_per_capita_usd;
    let income = (gdp / m.reference_gdp_usd).clamp(low, high);
    p.capital
        .scale(income * (1.0 + m.lead_capital * v.lead) * (p.valuation + 1.0))
        .scale(m.stakes.success_factor)
}

/// Share a dollar pledged to the open round buys at the end, after the rounds still to
/// come with the valuations of the data.
pub fn share_per_dollar(m: &VentureModel, v: &Venture) -> f64 {
    let valuation = m.phases.get(v.phase).map_or(1.0, |p| p.valuation);
    let after = v.capital.to_usd() * (valuation + 1.0);
    if after <= 0.0 {
        return 0.0;
    }
    (v.phase + 1..m.phases.len())
        .map(|j| m.phases[j].valuation / (m.phases[j].valuation + 1.0))
        .fold(1.0 / after, |a, keep| a * keep)
}

/// Share of a start-up a company holds.
pub fn share_of(v: &Venture, company: CompanyId) -> f64 {
    v.owners
        .iter()
        .filter(|s| s.holder == Holder::Company(company))
        // fold from +0.0: an empty f64 sum is -0.0, which views would show.
        .fold(0.0, |sum, s| sum + s.share)
}

/// The company holding more than the majority share.
pub fn majority_company(m: &VentureModel, v: &Venture) -> Option<CompanyId> {
    companies_of(v)
        .into_iter()
        .find(|&(_, share)| share > m.stakes.majority)
        .map(|(c, _)| c)
}

/// The companies among the owners with their shares, in the order they came in.
fn companies_of(v: &Venture) -> Vec<(CompanyId, f64)> {
    let mut out: Vec<(CompanyId, f64)> = Vec::new();
    for s in &v.owners {
        if let Holder::Company(c) = s.holder {
            match out.iter_mut().find(|(x, _)| *x == c) {
                Some((_, share)) => *share += s.share,
                None => out.push((c, s.share)),
            }
        }
    }
    out
}

/// Whether another company holds a blocking minority against `company`.
pub fn blocked(m: &VentureModel, v: &Venture, company: CompanyId) -> bool {
    companies_of(v)
        .into_iter()
        .any(|(c, share)| c != company && share >= m.stakes.blocking)
}

fn remove_share(v: &mut Venture, holder: Holder, share: f64) {
    if let Some(s) = v.owners.iter_mut().find(|s| s.holder == holder) {
        s.share -= share;
    }
    v.owners.retain(|s| s.share > 1e-12);
}

fn add_share(v: &mut Venture, holder: Holder, share: f64) {
    match v.owners.iter_mut().find(|s| s.holder == holder) {
        Some(s) => s.share += share,
        None => v.owners.push(Stake { holder, share }),
    }
}

fn add_to(list: &mut Vec<(CompanyId, Money)>, company: CompanyId, amount: Money) {
    match list.iter_mut().find(|(c, _)| *c == company) {
        Some((_, a)) => *a += amount,
        None => list.push((company, amount)),
    }
}

/// Takes up to `amount` of a company's entry; returns what it took.
fn take_from(list: &mut Vec<(CompanyId, Money)>, company: CompanyId, amount: Money) -> Money {
    let Some(entry) = list.iter_mut().find(|(c, _)| *c == company) else {
        return Money::ZERO;
    };
    let taken = amount.min(entry.1);
    entry.1 -= taken;
    list.retain(|(_, a)| *a > Money::ZERO);
    taken
}

fn take_all(list: &mut Vec<(CompanyId, Money)>, company: CompanyId) -> Money {
    let all = amount_of(list, company);
    take_from(list, company, all)
}

/// A company's entry in a list of amounts.
pub fn amount_of(list: &[(CompanyId, Money)], company: CompanyId) -> Money {
    list.iter()
        .filter(|(c, _)| *c == company)
        .map(|&(_, a)| a)
        .sum()
}

/// The position of a start-up in the list.
fn position(state: &GameState, venture: u32) -> Result<usize, CommandError> {
    state
        .ventures
        .iter()
        .position(|v| v.id == venture)
        .ok_or(CommandError::UnknownVenture)
}

/// A start-up a company may act on: known and active.
fn active(state: &GameState, venture: u32) -> Result<usize, CommandError> {
    let i = position(state, venture)?;
    if state.ventures[i].status != VentureStatus::Active {
        return Err(CommandError::VentureClosed);
    }
    Ok(i)
}

fn check_amount(state: &GameState, actor: CompanyId, amount: Money) -> Result<(), CommandError> {
    if amount <= Money::ZERO {
        return Err(CommandError::InvalidAmount);
    }
    if state.companies[actor.index()].ledger.cash() < amount {
        return Err(CommandError::NotEnoughCash { needed: amount });
    }
    Ok(())
}

/// `InvestInVenture` (SU2): a pledge to the open round, or shares of founders and
/// investors between rounds.
pub(crate) fn invest(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    venture: u32,
    amount: Money,
) -> Result<(), CommandError> {
    let m = &catalog.ventures;
    let i = active(state, venture)?;
    check_amount(state, actor, amount)?;
    let v = &state.ventures[i];
    if v.parent.is_some_and(|p| p != actor) {
        return Err(CommandError::VentureOfOther);
    }
    let date = state.date;
    if v.round_until.is_some() {
        let open = v.capital - v.raised;
        if amount > open {
            return Err(CommandError::AmountTooHigh { max: open });
        }
        pledge(state, i, actor, amount);
        let v = &mut state.ventures[i];
        if v.raised >= v.capital {
            close_round(m, v, date);
            keep_parent(m, v);
        }
    } else {
        let price = value(m, v).scale(1.0 + m.stakes.buy_premium);
        let outside: f64 = v
            .owners
            .iter()
            .filter(|s| matches!(s.holder, Holder::Private | Holder::Investors))
            .map(|s| s.share)
            .sum();
        if price <= Money::ZERO || outside <= 0.0 {
            return Err(CommandError::AmountTooHigh { max: Money::ZERO });
        }
        let share = amount.to_usd() / price.to_usd();
        if share > outside + 1e-12 {
            return Err(CommandError::AmountTooHigh {
                max: price.scale(outside),
            });
        }
        state.companies[actor.index()].ledger.transfer(
            Account::Participations,
            Account::Cash,
            amount,
        );
        let v = &mut state.ventures[i];
        // Founders and investors sell in proportion to what they hold.
        let sellers: Vec<Stake> = v
            .owners
            .iter()
            .copied()
            .filter(|s| matches!(s.holder, Holder::Private | Holder::Investors))
            .collect();
        for s in sellers {
            remove_share(v, s.holder, share * s.share / outside);
        }
        add_share(v, Holder::Company(actor), share);
        add_to(&mut v.book, actor, amount);
    }
    crate::central::count_purchase(state, actor, amount);
    Ok(())
}

/// `GrantVenture` (SU2): money without shares raises the chance of the phase.
pub(crate) fn grant(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    venture: u32,
    amount: Money,
) -> Result<(), CommandError> {
    let m = &catalog.ventures;
    let i = active(state, venture)?;
    check_amount(state, actor, amount)?;
    state.companies[actor.index()].ledger.expense(
        CostType::Research,
        CostCenter::default(),
        Account::Cash,
        amount,
    );
    let v = &mut state.ventures[i];
    let reach = if v.capital > Money::ZERO {
        (amount.to_usd() / v.capital.to_usd()).min(1.0)
    } else {
        1.0
    };
    v.chance = (v.chance + (1.0 - v.chance) * m.stakes.grant_effect * reach)
        .min(m.stakes.chance_max.max(v.chance));
    add_to(&mut v.grants, actor, amount);
    Ok(())
}

/// `SellVentureStake` (SU2): a share to investors below the value.
pub(crate) fn sell(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    venture: u32,
    share: f64,
) -> Result<(), CommandError> {
    let m = &catalog.ventures;
    let i = active(state, venture)?;
    let v = &state.ventures[i];
    let held = share_of(v, actor);
    if !share.is_finite() || share <= 0.0 {
        return Err(CommandError::InvalidShare);
    }
    if share > held + 1e-9 {
        return Err(CommandError::NotEnoughShares);
    }
    let share = share.min(held);
    let proceeds = value(m, v).scale(share * (1.0 - m.stakes.sale_discount));
    exit_stake(state, i, actor, share, proceeds);
    let v = &mut state.ventures[i];
    add_share(v, Holder::Investors, share);
    keep_parent(m, v);
    Ok(())
}

/// `SteerVenture` (SU2): the majority sets the pace of the phases to come and of the
/// current one while its round is open.
pub(crate) fn steer(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    venture: u32,
    pace: VenturePace,
) -> Result<(), CommandError> {
    let m = &catalog.ventures;
    let i = active(state, venture)?;
    let v = &mut state.ventures[i];
    if v.parent != Some(actor) && share_of(v, actor) <= m.stakes.majority {
        return Err(CommandError::NoMajority);
    }
    if v.round_until.is_some() {
        let (_, old) = pace_factors(m, v.pace);
        let (_, new) = pace_factors(m, pace);
        if old > 0.0 {
            v.chance = (v.chance / old * new).min(m.stakes.chance_max.max(v.chance.min(1.0)));
        }
    }
    v.pace = pace;
    Ok(())
}

/// What a company pays to buy out the other owners of a start-up: their share of the
/// value with the premium.
pub fn integration_price(m: &VentureModel, v: &Venture, company: CompanyId) -> Money {
    value(m, v).scale((1.0 + m.stakes.buy_premium) * (1.0 - share_of(v, company)))
}

/// `IntegrateVenture` (SU2): the majority buys out the others; the start-up becomes its
/// subsidiary.
pub(crate) fn integrate(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    venture: u32,
) -> Result<(), CommandError> {
    let m = &catalog.ventures;
    let i = active(state, venture)?;
    let v = &state.ventures[i];
    if v.parent == Some(actor) {
        return Err(CommandError::VentureClosed);
    }
    if share_of(v, actor) <= m.stakes.majority {
        return Err(CommandError::NoMajority);
    }
    if blocked(m, v, actor) {
        return Err(CommandError::VentureBlocked);
    }
    let price = integration_price(m, v, actor);
    if state.companies[actor.index()].ledger.cash() < price {
        return Err(CommandError::NotEnoughCash { needed: price });
    }
    let worth = value(m, v).scale(1.0 + m.stakes.buy_premium);
    let mut news = Vec::new();
    buy_out(state, i, actor, worth, &mut news);
    state.ventures[i].parent = Some(actor);
    crate::central::count_purchase(state, actor, price);
    Ok(())
}

/// What a dollar pledged to the open round of a start-up brings on average as a company
/// sees it (SU2): its shown chance times the value of a success times the share a dollar
/// buys at the end.
pub fn expected_return(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    v: &Venture,
) -> f64 {
    let (chance, _) = shown_chance(catalog, state, company, v);
    return_at(state, &catalog.ventures, v, chance)
}

/// What a dollar pledged now brings on average at a chance of success.
fn return_at(state: &GameState, m: &VentureModel, v: &Venture, chance: f64) -> f64 {
    chance * expected_success_value(state, m, v).to_usd() * share_per_dollar(m, v)
}

/// The pledges the strategy department of a company recommends (SU2): open rounds it may
/// join whose expected return per dollar reaches the minimum for the readiness for risks,
/// best first, each for the open rest within the budget and the share of the cash. None
/// without a working department.
pub(crate) fn recommendations(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
) -> Vec<(u32, Money)> {
    let m = &catalog.ventures;
    if insight(catalog, state, company).is_none() {
        return Vec::new();
    }
    let c = &state.companies[company.index()];
    let wanted = 1.0 + (1.0 - c.participations.risk) * m.stakes.min_return;
    let mut left = c.ledger.cash().scale(m.stakes.cash_share).max(Money::ZERO);
    if let Some(budget) = crate::central::participations_left(state, company) {
        left = left.min(budget);
    }
    let mut found: Vec<(f64, u32, Money)> = state
        .ventures
        .iter()
        .filter(|v| {
            v.status == VentureStatus::Active
                && v.round_until.is_some()
                && v.parent.is_none_or(|p| p == company)
                && amount_of(&v.pledges, company) == Money::ZERO
        })
        .map(|v| {
            (
                expected_return(catalog, state, company, v),
                v.id,
                v.capital - v.raised,
            )
        })
        .filter(|&(e, _, open)| e >= wanted && open > Money::ZERO)
        .collect();
    found.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut out = Vec::new();
    for (_, id, open) in found {
        let amount = open.min(left);
        if amount <= Money::ZERO {
            break;
        }
        left -= amount;
        out.push((id, amount));
    }
    out
}

/// The project of a research center as a start-up (SU3): its target, how far it is and
/// the phase it would begin in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpinOffPlan {
    pub target: VentureTarget,
    pub progress: f64,
    pub phase: usize,
    pub lead: f64,
    pub country: CountryId,
}

/// What a company's research center could spin off today: a technology not yet invented
/// or the next level of a product no company has reached, from enough progress on.
pub fn spin_off_plan(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    site: SiteId,
    date: Date,
) -> Result<SpinOffPlan, CommandError> {
    let m = &catalog.ventures;
    let s = state
        .site(site)
        .filter(|s| s.owner == company && s.kind == SiteType::ResearchCenter)
        .ok_or(CommandError::NoSpinOff)?;
    if m.phases.is_empty() {
        return Err(CommandError::NoSpinOff);
    }
    let c = &state.companies[company.index()];
    let (target, points, effort, lead) = match (s.research, s.development) {
        (Some(t), _) => {
            let tech = catalog.technologies.get(t);
            if state.inventions.get(t).is_some() || tech.invention_year <= date.year() {
                return Err(CommandError::NoSpinOff);
            }
            let effort = crate::research::effort(catalog, state, t, date)
                .ok_or(CommandError::NoSpinOff)?
                .points;
            let points = c.research.get(&t).copied().unwrap_or(0.0);
            let lead = f64::from(tech.invention_year - date.year());
            (VentureTarget::Technology(t), points, effort, lead)
        }
        (None, Some(product)) => {
            let (level, effort) =
                crate::development::next_effort(catalog, state, company, product, date)
                    .ok_or(CommandError::NoSpinOff)?;
            // Only a level no company has reached yet.
            if state.developments.get(product).len() + 1 != usize::from(level) {
                return Err(CommandError::NoSpinOff);
            }
            let points = c.development.points.get(&product).copied().unwrap_or(0.0);
            (
                VentureTarget::Development { product, level },
                points,
                effort,
                0.0,
            )
        }
        (None, None) => return Err(CommandError::NoSpinOff),
    };
    if effort <= 0.0 {
        return Err(CommandError::NoSpinOff);
    }
    let progress = (points / effort).clamp(0.0, 1.0);
    let min = m.stakes.spin_off_progress_min;
    if progress < min {
        return Err(CommandError::SpinOffTooEarly { progress, min });
    }
    let phases = m.phases.len();
    // A handful of phases; the cast is exact.
    let phase = ((progress * phases as f64).floor() as usize).min(phases - 1);
    Ok(SpinOffPlan {
        target,
        progress,
        phase,
        lead,
        country: s.country,
    })
}

/// The start-up a plan of a company becomes: the next number, a founder of its country,
/// its phase begun today with an open round, all of it the company's.
fn spin_off_venture(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    plan: &SpinOffPlan,
    date: Date,
) -> Venture {
    let id = state.next_venture;
    let mut rng = SimRng::for_stream(state.settings.seed, Stream::SpinOff { id });
    let mut names: BTreeSet<String> = state.ventures.iter().map(|v| v.name.clone()).collect();
    let name = management::manager_name(catalog, &mut rng, plan.country, &mut names);
    let mut v = Venture {
        id,
        name,
        inventor: false,
        country: plan.country,
        target: plan.target,
        lead: plan.lead,
        founded: date,
        phase: 0,
        capital: Money::ZERO,
        chance: 0.0,
        raised: Money::ZERO,
        round_until: None,
        phase_until: None,
        owners: Stake::sole(Holder::Company(company)),
        status: VentureStatus::Active,
        blur: 2.0 * rng.next_f64() - 1.0,
        pledges: Vec::new(),
        book: Vec::new(),
        grants: Vec::new(),
        parent: None,
        pace: VenturePace::Normal,
        exit: None,
        origin: Some(company),
    };
    let gdp = state.countries.get(plan.country).gdp_per_capita_usd;
    start_phase(&catalog.ventures, &mut v, plan.phase, gdp, date);
    v
}

/// Months the phases from `phase` on take at the normal pace (without waiting for rounds).
pub fn months_from(m: &VentureModel, phase: usize) -> u32 {
    m.phases.iter().skip(phase).map(|p| p.months).sum()
}

/// Whether a plan can end before history: a new technology needs at least as many years
/// ahead as its phases take; a level has no date in history.
pub fn ahead_of_history(m: &VentureModel, plan: &SpinOffPlan) -> bool {
    match plan.target {
        VentureTarget::Technology(_) => plan.lead * 12.0 >= f64::from(months_from(m, plan.phase)),
        VentureTarget::Development { .. } => true,
    }
}

/// Other companies whose research centers work on the same target: they may reach it
/// first and overtake the start-up.
pub fn rivals(state: &GameState, company: CompanyId, target: VentureTarget) -> u32 {
    let mut seen: BTreeSet<CompanyId> = BTreeSet::new();
    for s in &state.sites {
        if s.owner == company
            || s.kind != SiteType::ResearchCenter
            || state.companies[s.owner.index()].bankrupt
        {
            continue;
        }
        let same = match target {
            VentureTarget::Technology(t) => s.research == Some(t),
            VentureTarget::Development { product, .. } => {
                s.research.is_none() && s.development == Some(product)
            }
        };
        if same {
            seen.insert(s.owner);
        }
    }
    // Few companies; the cast is exact.
    seen.len() as u32
}

/// What the start-up of a plan would be worth today, before its first round.
pub fn spin_off_value(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    plan: &SpinOffPlan,
) -> Money {
    value(
        &catalog.ventures,
        &spin_off_venture(catalog, state, company, plan, state.date),
    )
}

/// `SpinOff` (SU3): the project of a research center becomes a start-up the company
/// holds; it sells `sell` of it to investors at once and keeps a subsidiary above the
/// majority.
pub(crate) fn spin_off(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    site: SiteId,
    sell_share: f64,
) -> Result<(), CommandError> {
    if !sell_share.is_finite() || !(0.0..1.0).contains(&sell_share) {
        return Err(CommandError::InvalidShare);
    }
    let m = &catalog.ventures;
    let date = state.date;
    let plan = spin_off_plan(catalog, state, actor, site, date)?;
    let v = spin_off_venture(catalog, state, actor, &plan, date);
    let id = v.id;
    state.next_venture += 1;
    state.ventures.push(v);
    // The project moves into the start-up; the research center is free.
    let c = &mut state.companies[actor.index()];
    match plan.target {
        VentureTarget::Technology(t) => {
            c.research.remove(&t);
        }
        VentureTarget::Development { product, .. } => {
            c.development.points.remove(&product);
        }
    }
    let s = &mut state.sites[site.index()];
    s.research = None;
    s.development = None;
    if sell_share > 0.0 {
        sell(state, catalog, actor, id, sell_share)?;
    }
    let i = state.ventures.len() - 1;
    if share_of(&state.ventures[i], actor) > m.stakes.majority {
        state.ventures[i].parent = Some(actor);
    }
    Ok(())
}

/// The chance of success an AI company sees (SU3): blurred by the start-up's draw, the
/// less the more competent the company is.
fn ai_chance(catalog: &Catalog, state: &GameState, company: CompanyId, v: &Venture) -> f64 {
    let m = &catalog.ventures;
    let (competence, _) = crate::ai::traits(catalog, state, company);
    let blur = m.blur * (1.0 - competence);
    (success_chance(m, v) * (1.0 + v.blur * blur)).clamp(0.0, 1.0)
}

/// What an AI company does with start-ups at a month start (SU3): at the start of a year
/// it may spin off research projects; it pledges to promising rounds and takes over
/// start-ups it can use. Every step is a command through the decider. Returns news for
/// the player.
pub(crate) fn ai_month(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    own: &[SiteId],
    date: Date,
    decider: &mut dyn Decider,
) -> Vec<Message> {
    let m = &catalog.ventures;
    let a = &m.stakes.ai;
    let mut news = Vec::new();
    // NaN counts as no start-ups.
    let wanted = state.settings.ventures > 0.0;
    if m.phases.is_empty() || !wanted {
        return news;
    }
    let month = management::month_number(date);
    let mut rng = SimRng::for_stream(
        state.settings.seed,
        Stream::VentureBids {
            company: company.0,
            month,
        },
    );
    if date.ordinal() == 1 {
        news.extend(ai_spin_offs(
            state, catalog, company, own, &mut rng, decider,
        ));
    }
    if !state
        .ventures
        .iter()
        .any(|v| v.status == VentureStatus::Active)
        || state.companies[company.index()].ledger.cash() < a.cash_min
        || !rng.chance(a.check_chance)
    {
        return news;
    }
    ai_pledges(state, catalog, company, decider);
    news.extend(ai_takeover(state, catalog, company, own, decider));
    news
}

/// Research projects an AI company spins off at the start of a year, each with the
/// chance of the data.
fn ai_spin_offs(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    own: &[SiteId],
    rng: &mut SimRng,
    decider: &mut dyn Decider,
) -> Vec<Message> {
    let a = &catalog.ventures.stakes.ai;
    let mut news = Vec::new();
    if a.spin_off_chance <= 0.0 {
        return news;
    }
    for &site in own {
        let date = state.date;
        // Only projects that can end before history and no rival works on.
        let promising = spin_off_plan(catalog, state, company, site, date).is_ok_and(|plan| {
            ahead_of_history(&catalog.ventures, &plan) && rivals(state, company, plan.target) == 0
        });
        if !promising || !rng.chance(a.spin_off_chance) {
            continue;
        }
        let command = Command::SpinOff {
            site,
            sell: a.spin_off_sale,
        };
        let decided = decision::decided(decider, state, catalog, |_| {
            Decision::new(
                Topic::Venture,
                company,
                Choice::one(ChoiceKind::Sell, command.clone()),
            )
        });
        if !decided || crate::command::execute(state, catalog, company, &command).is_err() {
            continue;
        }
        let Some(v) = state.ventures.last() else {
            continue;
        };
        news.push(
            Message::new(MessageKind::Info, keys::VENTURE_SPIN_OFF)
                .with(
                    "firma",
                    Param::Text(state.companies[company.index()].name.clone()),
                )
                .with("name", Param::Text(v.name.clone()))
                .with(
                    "land",
                    Param::Country(catalog.countries.key(v.country).to_owned()),
                )
                .with("ziel", target_param(catalog, v.target)),
        );
    }
    news
}

/// Pledges of an AI company to the open rounds it expects most of, within its share of
/// the cash for a month.
fn ai_pledges(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    decider: &mut dyn Decider,
) {
    let m = &catalog.ventures;
    let a = &m.stakes.ai;
    let (_, aggressiveness) = crate::ai::traits(catalog, state, company);
    let wanted = 1.0 + (1.0 - aggressiveness) * a.min_return;
    let mut left = state.companies[company.index()]
        .ledger
        .cash()
        .scale(a.cash_share);
    let mut found: Vec<(f64, u32, Money)> = state
        .ventures
        .iter()
        .filter(|v| {
            v.status == VentureStatus::Active
                && v.round_until.is_some()
                && v.parent.is_none()
                && amount_of(&v.pledges, company) == Money::ZERO
        })
        .map(|v| {
            let chance = ai_chance(catalog, state, company, v);
            (return_at(state, m, v, chance), v.id, v.capital - v.raised)
        })
        .filter(|&(e, _, open)| e >= wanted && open > Money::ZERO)
        .collect();
    found.sort_by(|x, y| y.0.total_cmp(&x.0).then(x.1.cmp(&y.1)));
    for (_, venture, open) in found {
        let amount = open.min(left);
        if amount <= Money::ZERO {
            break;
        }
        let command = Command::InvestInVenture { venture, amount };
        let decided = decision::decided(decider, state, catalog, |_| {
            Decision::new(
                Topic::Venture,
                company,
                Choice::one(ChoiceKind::Invest, command.clone()),
            )
        });
        if decided && crate::command::execute(state, catalog, company, &command).is_ok() {
            left -= amount;
        }
    }
}

/// Whether a start-up's target serves an AI company: a technology for a product of its
/// branches, or the next level of a product it makes.
fn serves(
    catalog: &Catalog,
    target: VentureTarget,
    branches: &BTreeSet<crate::ids::BranchId>,
    products: &BTreeSet<ProductId>,
) -> bool {
    match target {
        VentureTarget::Technology(t) => catalog.recipes.values().any(|r| {
            (r.technology == Some(t) || catalog.facilities.get(r.facility).technology == Some(t))
                && branches.contains(&catalog.products.get(r.product).branch)
        }),
        VentureTarget::Development { product, .. } => products.contains(&product),
    }
}

/// An AI company buys the majority of the most promising start-up it can use and already
/// holds a share of, between its rounds, and integrates it, if no other company holds a
/// blocking minority and both cost at most its share of the cash for takeovers.
fn ai_takeover(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    own: &[SiteId],
    decider: &mut dyn Decider,
) -> Vec<Message> {
    let m = &catalog.ventures;
    let a = &m.stakes.ai;
    let mut products = BTreeSet::new();
    for &site in own {
        for sl in &state.sites[site.index()].slots {
            if let Some(r) = sl.recipe {
                products.insert(catalog.recipes.get(r).product);
            }
        }
    }
    let branches: BTreeSet<crate::ids::BranchId> = products
        .iter()
        .map(|&p| catalog.products.get(p).branch)
        .collect();
    if products.is_empty() {
        return Vec::new();
    }
    let limit = state.companies[company.index()]
        .ledger
        .cash()
        .scale(a.takeover_cash);
    let wanted = m.stakes.majority + 0.01;
    let best = state
        .ventures
        .iter()
        .filter(|v| {
            let own = share_of(v, company);
            v.status == VentureStatus::Active
                && v.round_until.is_none()
                && v.parent.is_none()
                && !blocked(m, v, company)
                && own >= a.takeover_share_min
                && own <= m.stakes.majority
        })
        .filter_map(|v| {
            let chance = ai_chance(catalog, state, company, v);
            let own_share = share_of(v, company);
            let outside: f64 = v
                .owners
                .iter()
                .filter(|s| matches!(s.holder, Holder::Private | Holder::Investors))
                .fold(0.0, |sum, s| sum + s.share);
            let price = value(m, v).scale(1.0 + m.stakes.buy_premium);
            let cost = price.scale(1.0 - own_share);
            (chance >= a.takeover_chance_min
                && outside + own_share >= wanted
                && price > Money::ZERO
                && cost <= limit
                && serves(catalog, v.target, &branches, &products))
            .then(|| (chance, v.id, price.scale(wanted - own_share)))
        })
        .max_by(|x, y| x.0.total_cmp(&y.0).then(y.1.cmp(&x.1)));
    let Some((_, venture, buy)) = best else {
        return Vec::new();
    };
    let commands = [
        Command::InvestInVenture {
            venture,
            amount: buy,
        },
        Command::IntegrateVenture { venture },
    ];
    let decided = decision::decided(decider, state, catalog, |_| {
        Decision::new(
            Topic::Venture,
            company,
            Choice::one(ChoiceKind::Invest, commands[0].clone()).then(commands[1].clone(), true),
        )
    });
    if !decided {
        return Vec::new();
    }
    let player = state.player;
    let Ok(i) = position(state, venture) else {
        return Vec::new();
    };
    let player_share = share_of(&state.ventures[i], player);
    let payment = value(m, &state.ventures[i]).scale((1.0 + m.stakes.buy_premium) * player_share);
    if crate::command::execute(state, catalog, company, &commands[0]).is_err()
        || crate::command::execute(state, catalog, company, &commands[1]).is_err()
    {
        return Vec::new();
    }
    let v = &state.ventures[i];
    let firm = state.companies[company.index()].name.clone();
    let message = if player_share > 0.0 {
        Message::new(MessageKind::Info, keys::VENTURE_BOUGHT_OUT)
            .with("firma", Param::Text(firm))
            .with("name", Param::Text(v.name.clone()))
            .with("betrag", Param::Money(payment))
    } else {
        Message::new(MessageKind::Info, keys::VENTURE_TAKEN_OVER)
            .with("firma", Param::Text(firm))
            .with("name", Param::Text(v.name.clone()))
            .with("ziel", target_param(catalog, v.target))
    };
    vec![message]
}
