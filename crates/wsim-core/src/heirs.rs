//! Death, heir and hand-over of the person (PE6, docs/FORMELN.md): the mortality of the
//! managers with the lifestyle, the heir by rule, inheritance tax with forced sales.

use crate::aging;
use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::command::CommandError;
use crate::management;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::person;
use crate::rng::{SimRng, Stream};
use crate::state::{Ancestor, Child, CompanyId, Departure, GameState, LifeEvent, LifeEventKind};

/// The person's chance to die within a month: the managers' table times the lifestyle.
pub fn death_chance(catalog: &Catalog, state: &GameState, today: Date) -> f64 {
    let p = &state.person;
    let age = f64::from(person::age(state, today));
    let level = crate::private::lifestyle_at(catalog, p, today);
    let care = if p.health_care {
        catalog.person.private_life.care_mortality
    } else {
        1.0
    };
    (aging::death_chance_at(catalog, age, p.home, today)
        * catalog.person.lifestyle(level).mortality
        * crate::wellbeing::mortality_factor(catalog, state)
        * care)
        .clamp(0.0, 1.0)
}

/// Who inherits: the chosen child if alive, else the eldest living child; `None` means a
/// nephew or niece.
pub fn heir(state: &GameState) -> Option<usize> {
    let children = &state.person.children;
    let alive = |i: usize| children.get(i).is_some_and(|c| c.died.is_none());
    state
        .person
        .heir
        .map(|h| h as usize)
        .filter(|&i| alive(i))
        .or_else(|| {
            (0..children.len())
                .filter(|&i| alive(i))
                .min_by_key(|&i| (children[i].born, i))
        })
}

/// What the person leaves: account, shares at value, start-ups, open loans.
pub fn estate(catalog: &Catalog, state: &GameState) -> Money {
    crate::private::wealth(catalog, state).total()
}

/// The inheritance tax of the estate now: the rate on the estate less the relief for the
/// shares of companies that go on (PV4).
pub fn tax_due(catalog: &Catalog, state: &GameState) -> Money {
    let relief = catalog
        .person
        .legacy
        .as_ref()
        .map_or(0.0, |l| l.firm_relief);
    let firms = crate::private::wealth(catalog, state)
        .shares
        .max(Money::ZERO);
    (estate(catalog, state) - firms.scale(relief))
        .max(Money::ZERO)
        .scale(tax_rate(catalog, state))
}

/// The part of the estate that stays with the successor, as a fraction (PV4).
pub fn successor_share(catalog: &Catalog, state: &GameState) -> f64 {
    let Some(i) = heir(state) else {
        return 1.0;
    };
    let me = u32::try_from(i).unwrap_or(u32::MAX);
    let all = shares(catalog, state);
    if all.is_empty() {
        return 1.0;
    }
    all.iter()
        .find(|(h, _)| *h == crate::state::Heir::Child(me))
        .map_or(0.0, |(_, f)| *f)
}

/// The co-heirs take their parts: the shares of companies in kind (they become private
/// holders), the rest as money from the account, if need be after sales. Returns what the
/// sales raised.
fn pay_co_heirs(state: &mut GameState, catalog: &Catalog, keep: f64) -> Money {
    if catalog.person.legacy.is_none() || keep >= 1.0 {
        return Money::ZERO;
    }
    let gone = 1.0 - keep;
    let w = crate::private::wealth(catalog, state);
    let claim = (w.total() - w.shares).max(Money::ZERO).scale(gone);
    // Shares of companies in kind.
    for i in 0..state.companies.len() {
        // Few companies; the cast is exact.
        let id = CompanyId(i as u32);
        let held = person::share(state, id);
        if held <= 0.0 || state.companies[i].bankrupt {
            continue;
        }
        let moved = held * gone;
        let owners = &mut state.companies[i].owners;
        crate::stock::remove_share(owners, crate::state::Holder::Player, moved);
        crate::stock::add_share(owners, crate::state::Holder::Private, moved);
        *state.person.heir_stakes.entry(id).or_insert(0.0) += moved;
    }
    crate::savings::liquidate(state, catalog, claim);
    let mut raised = Money::ZERO;
    if state.person.account.balance < claim {
        raised = raise(state, catalog, claim);
    }
    let pay = claim.min(state.person.account.balance);
    if pay > Money::ZERO {
        crate::private::book_flow(state, crate::state::PrivateFlow::InheritancePayout, -pay);
    }
    raised
}

/// The inheritance or gift tax rate at home today.
pub fn tax_rate(catalog: &Catalog, state: &GameState) -> f64 {
    catalog
        .person
        .inheritance_tax
        .value(state.person.home, state.date.year_fraction())
        .clamp(0.0, 1.0)
}

/// At a month start: perhaps the person dies; from the hint age each January the hint to
/// settle the succession.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    if !catalog.person.enabled || state.person.name.is_empty() || state.game_over {
        return news;
    }
    let mut rng = SimRng::for_stream(
        state.settings.seed,
        Stream::Succession {
            month: management::month_number(date),
        },
    );
    if rng.chance(death_chance(catalog, state, date)) {
        news.push(succeed(state, catalog, &mut rng, date, true));
        return news;
    }
    if date.month() == 1 {
        news.extend(dispute(state, catalog, &mut rng));
    }
    let age = person::age(state, date);
    if date.month() == 1
        && age >= catalog.person.succession_hint_from
        && state.person.heir.is_none()
    {
        news.push(
            Message::new(MessageKind::Warning, keys::PERSON_SUCCESSION_HINT)
                .with("alter", Param::Integer(i64::from(age))),
        );
    }
    news
}

/// Shares of the estate as fractions: the will, else the statutory succession (PV4).
pub fn shares(catalog: &Catalog, state: &GameState) -> Vec<(crate::state::Heir, f64)> {
    use crate::state::Heir;
    let p = &state.person;
    if !p.will.is_empty() {
        let total: f64 = p.will.iter().map(|w| f64::from(w.percent)).sum();
        if total > 0.0 {
            return p
                .will
                .iter()
                .map(|w| (w.heir, f64::from(w.percent) / total))
                .collect();
        }
    }
    let kids: Vec<u32> = p
        .children
        .iter()
        .enumerate()
        .filter(|(_, c)| c.died.is_none())
        .map(|(i, _)| u32::try_from(i).unwrap_or(u32::MAX))
        .collect();
    let partner = if p.married {
        catalog
            .person
            .legacy
            .as_ref()
            .map_or(0.0, |l| l.partner_share)
    } else {
        0.0
    };
    if kids.is_empty() {
        return if p.married {
            vec![(Heir::Partner, 1.0)]
        } else {
            Vec::new()
        };
    }
    // Few children; the cast is exact.
    #[allow(clippy::cast_precision_loss)]
    let each = (1.0 - partner) / kids.len() as f64;
    let mut out = Vec::new();
    if partner > 0.0 {
        out.push((Heir::Partner, partner));
    }
    out.extend(kids.into_iter().map(|i| (Heir::Child(i), each)));
    out
}

/// A quarrel among the co-heirs (PV4): each January, with the chance of the data, they sell
/// part of their shares of a company to investors.
fn dispute(state: &mut GameState, catalog: &Catalog, rng: &mut SimRng) -> Vec<Message> {
    let Some(l) = &catalog.person.legacy else {
        return Vec::new();
    };
    let mut news = Vec::new();
    let held: Vec<(CompanyId, f64)> = state
        .person
        .heir_stakes
        .iter()
        .map(|(&c, &q)| (c, q))
        .collect();
    for (id, q) in held {
        if !rng.chance(l.dispute_chance) {
            continue;
        }
        let sold = q * l.dispute_sale;
        let owners = &mut state.companies[id.index()].owners;
        crate::stock::remove_share(owners, crate::state::Holder::Private, sold);
        crate::stock::add_share(owners, crate::state::Holder::Investors, sold);
        let left = q - sold;
        if left < 1e-6 {
            state.person.heir_stakes.remove(&id);
        } else {
            state.person.heir_stakes.insert(id, left);
        }
        news.push(
            Message::new(MessageKind::Warning, keys::HEIRS_DISPUTE)
                .with("anteil", Param::Number((sold * 1000.0).round() / 10.0))
                .with(
                    "firma",
                    Param::Text(state.companies[id.index()].name.clone()),
                ),
        );
    }
    news
}

/// `SetWill`: shares of partner and living children adding up to 100 percent; an empty list
/// restores the statutory succession.
pub(crate) fn set_will(
    state: &mut GameState,
    shares: &[crate::state::WillShare],
) -> Result<(), CommandError> {
    use crate::state::Heir;
    if shares.is_empty() {
        state.person.will.clear();
        return Ok(());
    }
    let total: u32 = shares.iter().map(|w| w.percent).sum();
    let mut seen: Vec<Heir> = Vec::new();
    for w in shares {
        let valid = match w.heir {
            Heir::Partner => state.person.married,
            Heir::Child(i) => state
                .person
                .children
                .get(i as usize)
                .is_some_and(|c| c.died.is_none()),
        };
        if !valid || seen.contains(&w.heir) {
            return Err(CommandError::WillInvalid);
        }
        seen.push(w.heir);
    }
    if total != 100 {
        return Err(CommandError::WillInvalid);
    }
    state.person.will = shares.to_vec();
    Ok(())
}

/// `SetHeir`: a living child, or the rule.
pub(crate) fn set_heir(state: &mut GameState, child: Option<u32>) -> Result<(), CommandError> {
    if let Some(i) = child {
        let alive = state
            .person
            .children
            .get(i as usize)
            .is_some_and(|c| c.died.is_none());
        if !alive {
            return Err(CommandError::NoSuchChild);
        }
    }
    state.person.heir = child;
    Ok(())
}

/// `HandOver`: everything goes to the heir now, with the gift tax.
pub(crate) fn hand_over(state: &mut GameState, catalog: &Catalog) -> Result<Message, CommandError> {
    if state.person.name.is_empty() {
        return Err(CommandError::PersonOnly);
    }
    let date = state.date;
    let mut rng = SimRng::for_stream(
        state.settings.seed,
        Stream::Succession {
            month: management::month_number(date),
        },
    );
    Ok(succeed(state, catalog, &mut rng, date, false))
}

/// The heir becomes the person: the tax is paid, if need be from forced sales.
fn succeed(
    state: &mut GameState,
    catalog: &Catalog,
    rng: &mut SimRng,
    date: Date,
    died: bool,
) -> Message {
    let old = state.person.name.clone();
    let age = person::age(state, date);
    let tax = tax_due(catalog, state);
    revalue(catalog, state);
    let mut sold = raise(state, catalog, tax);
    let paid = tax.min(state.person.account.balance);
    if paid > Money::ZERO {
        crate::private::pay_inheritance_tax(state, paid);
    }
    // The co-heirs get their parts (PV4): the successor keeps only its share.
    let keep = successor_share(catalog, state);
    sold += pay_co_heirs(state, catalog, keep);
    revalue(catalog, state);
    let (heir, nephew) = take_heir(state, catalog, rng, date);
    let p = &mut state.person;
    p.ancestors.push(Ancestor {
        name: old.clone(),
        born: p.born,
        until: date,
        died,
    });
    p.name = heir.name.clone();
    p.born = heir.born;
    p.married = true;
    p.heir = None;
    let children = own_children(catalog, rng, &heir, date, p);
    let p = &mut state.person;
    p.children = children;
    p.history.push(LifeEvent {
        date,
        kind: LifeEventKind::Succession {
            from: old.clone(),
            to: heir.name.clone(),
            died,
            tax: paid,
        },
    });
    let key = if died {
        keys::PERSON_DIED
    } else {
        keys::PERSON_HANDED_OVER
    };
    Message::new(MessageKind::Warning, key)
        .with("name", Param::Text(old))
        .with("alter", Param::Integer(i64::from(age)))
        .with("erbe", Param::Text(heir.name))
        .with(
            "verwandt",
            Param::TextKey(if nephew {
                "person.verwandt.neffe".into()
            } else {
                "person.verwandt.kind".into()
            }),
        )
        .with("steuer", Param::Money(paid))
        .with("verkauf", Param::Money(sold))
}

/// The cost basis of every company becomes the share at its value today.
fn revalue(catalog: &Catalog, state: &mut GameState) {
    for i in 0..state.companies.len() {
        // Few companies; the cast is exact.
        let id = CompanyId(i as u32);
        let held = person::share(state, id);
        if held > 0.0 && !state.companies[i].bankrupt {
            let value = crate::private::company_value(catalog, state, id).scale(held);
            state.person.cost_basis.insert(id, value);
        }
    }
}

/// Sells shares to investors until the account covers `need`: minority stakes first, then
/// shares of controlled companies, each in the order of the companies. Returns the proceeds.
fn raise(state: &mut GameState, catalog: &Catalog, need: Money) -> Money {
    let mut proceeds = Money::ZERO;
    for controlled in [false, true] {
        for i in 0..state.companies.len() {
            let missing = need - state.person.account.balance;
            if missing <= Money::ZERO {
                return proceeds;
            }
            // Few companies; the cast is exact.
            let id = CompanyId(i as u32);
            if state.companies[i].bankrupt || crate::private::is_controlled(state, id) != controlled
            {
                continue;
            }
            let held = person::share(state, id);
            if held <= 0.0 {
                continue;
            }
            let all = crate::holdings::bid_of_investors(catalog, state, id, held);
            if all <= Money::ZERO {
                continue;
            }
            let share = (held * missing.to_usd() / all.to_usd()).min(held);
            let before = state.person.account.balance;
            if crate::holdings::sell(state, catalog, id, share).is_ok() {
                proceeds += state.person.account.balance - before;
            }
        }
    }
    proceeds
}

/// The heir: the child by rule, whose manager card ends, else a new nephew or niece.
fn take_heir(
    state: &mut GameState,
    catalog: &Catalog,
    rng: &mut SimRng,
    date: Date,
) -> (Child, bool) {
    if let Some(i) = heir(state) {
        let child = state.person.children[i].clone();
        if let Some(id) = child.manager {
            aging::depart(
                state,
                catalog,
                id,
                Departure::Retired,
                date,
                &mut Vec::new(),
            );
        }
        return (child, false);
    }
    let [from, to] = catalog.person.nephew_age;
    let years =
        from + u32::try_from(rng.below(u64::from(to.saturating_sub(from)) + 1)).unwrap_or(0);
    // Exactly `years` old today: the birthday within the year before that age.
    let latest = Date::new(
        date.year() - i32::try_from(years).unwrap_or(30),
        date.month(),
        1,
    )
    .unwrap_or(date)
    .add_days(date.day() as i32 - 1);
    // Below 365; the cast is exact.
    let born = latest.add_days(-(rng.below(365) as i32));
    let name = person::child_name(catalog, rng, &state.person);
    (person::new_child(catalog, rng, name, born), true)
}

/// The heir's own children, as if born in the years already past (docs/FORMELN.md, PE6).
fn own_children(
    catalog: &Catalog,
    rng: &mut SimRng,
    heir: &Child,
    today: Date,
    family: &crate::state::Person,
) -> Vec<Child> {
    let model = &catalog.person;
    let [from, until] = model.children_ages;
    let mut kids: Vec<Child> = Vec::new();
    // The family name comes from the person, now the heir.
    let mut named = family.clone();
    named.name.clone_from(&heir.name);
    named.children.clear();
    for age in from..until {
        if kids.len() as u32 >= model.children_max {
            break;
        }
        let year = heir.born.year() + i32::try_from(age).unwrap_or(0);
        if year >= today.year() {
            break;
        }
        if !rng.chance(model.child_chance.clamp(0.0, 1.0)) {
            continue;
        }
        let born = person::day_in(rng, year).max(heir.born);
        let name = person::child_name(catalog, rng, &named);
        let child = person::new_child(catalog, rng, name, born);
        named.children.push(child.clone());
        kids.push(child);
    }
    kids
}
