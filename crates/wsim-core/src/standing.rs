//! Private standing, network, clubs and possessions of the person (PV6a, docs/FORMELN.md).

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::{Catalog, ContactRole, StandingModel};
use crate::command::CommandError;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::savings::Investment;
use crate::state::{GameState, PrivateFlow};

/// Offset of the stream of the circle among the streams of the person.
const CIRCLE_STREAM: u32 = 6 << 24;

/// A person of the network.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Contact {
    pub role: ContactRole,
    pub name: String,
    pub since: Date,
    /// The last day: contacts age and die like managers.
    pub until: Date,
    /// 0–100; falls without time for the network.
    pub strength: f64,
}

/// A possession bought.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnedGood {
    /// Index in the data.
    pub kind: u32,
    pub paid: Money,
}

/// Standing, clubs, contacts and possessions of the person.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Circle {
    /// The standing 0–100; `None` until the first month (the background gives the start).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standing: Option<f64>,
    /// Clubs by index in the data.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clubs: Vec<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contacts: Vec<Contact>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub goods: Vec<OwnedGood>,
}

impl Circle {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

pub fn model(catalog: &Catalog) -> Option<&StandingModel> {
    catalog.person.standing.as_ref()
}

/// The standing now; the background gives the start.
pub fn standing(catalog: &Catalog, state: &GameState) -> f64 {
    state
        .person
        .circle
        .standing
        .unwrap_or_else(|| crate::wellbeing::effect(catalog, state).standing)
}

/// The price of a possession now: academic yearly wages of the home country.
pub fn good_price(catalog: &Catalog, state: &GameState, kind: usize) -> Option<Money> {
    let m = model(catalog)?;
    let g = m.goods.get(kind)?;
    let wage = crate::private::academic_monthly_wage(catalog, state, state.person.home);
    Some(wage.scale(12.0 * g.price))
}

/// The standing the person drifts to.
pub fn target(catalog: &Catalog, state: &GameState) -> f64 {
    let Some(m) = model(catalog) else {
        return 0.0;
    };
    let level = crate::private::lifestyle_at(catalog, &state.person, state.date) as usize;
    let companies = (0..state.companies.len())
        .filter(|&i| {
            crate::private::is_controlled(
                state,
                crate::state::CompanyId(u32::try_from(i).unwrap_or(u32::MAX)),
            )
        })
        .count();
    let goods: f64 = state
        .person
        .circle
        .goods
        .iter()
        .filter_map(|g| m.goods.get(g.kind as usize))
        .map(|g| g.representation)
        .sum();
    // Few companies; the cast is exact.
    #[allow(clippy::cast_precision_loss)]
    let firms = m.target_per_company * companies as f64;
    let foundation = crate::trust::standing_points(catalog, state);
    (m.target_base + m.target_lifestyle[level.min(3)] + firms + goods + foundation)
        .clamp(0.0, 100.0)
}

/// The strength 0–1 of the best living contact of a role.
fn best(state: &GameState, role: ContactRole) -> f64 {
    state
        .person
        .circle
        .contacts
        .iter()
        .filter(|c| c.role == role)
        .map(|c| c.strength / 100.0)
        .fold(0.0, f64::max)
}

/// Added to the rate of a new bank loan: the standing and the best banker.
pub fn loan_rate_offset(catalog: &Catalog, state: &GameState) -> f64 {
    model(catalog).map_or(0.0, |m| {
        m.interest_per_point * standing(catalog, state)
            + m.banker_interest * best(state, ContactRole::Banker)
    })
}

/// The share off the prices of properties the best broker gets (PV6a).
pub fn broker_discount(catalog: &Catalog, state: &GameState) -> f64 {
    model(catalog).map_or(0.0, |m| {
        m.broker_discount * best(state, ContactRole::Broker)
    })
}

/// An action of the circle.
pub(crate) fn apply(
    state: &mut GameState,
    catalog: &Catalog,
    action: Investment,
) -> Result<(), CommandError> {
    let Some(m) = model(catalog).cloned() else {
        return Err(CommandError::NotPossible);
    };
    match action {
        Investment::Donate { amount } => {
            if amount <= Money::ZERO {
                return Err(CommandError::NotPossible);
            }
            pay(state, amount, PrivateFlow::Donation)?;
            let year = crate::private::academic_monthly_wage(catalog, state, state.person.home)
                .scale(12.0)
                .to_usd()
                .max(1.0);
            let points = (m.donation_points * amount.to_usd() / year).min(m.donation_cap);
            let now = standing(catalog, state);
            state.person.circle.standing = Some((now + points).min(100.0));
        }
        Investment::JoinClub { club } => {
            let c = m
                .clubs
                .get(club as usize)
                .ok_or(CommandError::NotPossible)?;
            let level = crate::private::lifestyle_at(catalog, &state.person, state.date) as usize;
            if state.person.circle.clubs.contains(&club)
                || standing(catalog, state) < c.standing
                || level < usize::from(c.lifestyle)
            {
                return Err(CommandError::NotPossible);
            }
            state.person.circle.clubs.push(club);
            state.person.circle.clubs.sort_unstable();
        }
        Investment::LeaveClub { club } => {
            let before = state.person.circle.clubs.len();
            state.person.circle.clubs.retain(|&c| c != club);
            if state.person.circle.clubs.len() == before {
                return Err(CommandError::NotPossible);
            }
        }
        Investment::BuyGood { good } => {
            let g = m
                .goods
                .get(good as usize)
                .ok_or(CommandError::NotPossible)?;
            if state.date.year() < g.from {
                return Err(CommandError::NotPossible);
            }
            let price =
                good_price(catalog, state, good as usize).ok_or(CommandError::NotPossible)?;
            pay(state, price, PrivateFlow::GoodBought)?;
            state.person.circle.goods.push(OwnedGood {
                kind: good,
                paid: price,
            });
        }
        Investment::SellGood { index } => {
            let i = index as usize;
            let g = *state
                .person
                .circle
                .goods
                .get(i)
                .ok_or(CommandError::NotPossible)?;
            state.person.circle.goods.remove(i);
            // The sale brings the price of today less a resale loss.
            let price = good_price(catalog, state, g.kind as usize).unwrap_or(g.paid);
            crate::private::book_flow(state, PrivateFlow::GoodSold, price.scale(0.8));
        }
        _ => return Err(CommandError::NotPossible),
    }
    Ok(())
}

fn pay(state: &mut GameState, amount: Money, flow: PrivateFlow) -> Result<(), CommandError> {
    if state.person.account.balance < amount {
        return Err(CommandError::NotEnoughPrivateMoney {
            needed: amount - state.person.account.balance,
        });
    }
    crate::private::book_flow(state, flow, -amount);
    Ok(())
}

/// At a month start: fees and upkeep, the drift of the standing, the care of the contacts,
/// new and ending contacts, hints.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    let Some(m) = model(catalog).cloned() else {
        return news;
    };
    if state.person.name.is_empty() {
        return news;
    }
    let month = crate::management::month_number(date);
    let mut rng = SimRng::for_stream(
        state.settings.seed,
        Stream::Person {
            month: month + CIRCLE_STREAM,
        },
    );
    let wage = crate::private::academic_monthly_wage(catalog, state, state.person.home);
    // Upkeep of the possessions and the fees of the clubs, paid as far as the account
    // reaches.
    let mut cost = Money::ZERO;
    for g in &state.person.circle.goods {
        if let Some(t) = m.goods.get(g.kind as usize) {
            cost += g.paid.scale(t.upkeep / 12.0);
        }
    }
    let level = crate::private::lifestyle_at(catalog, &state.person, date) as usize;
    let have = state
        .person
        .circle
        .standing
        .unwrap_or_else(|| standing(catalog, state));
    let mut dropped = Vec::new();
    for &club in &state.person.circle.clubs {
        let Some(c) = m.clubs.get(club as usize) else {
            continue;
        };
        if have < c.standing - 5.0 || level < usize::from(c.lifestyle) {
            dropped.push(club);
        } else {
            cost += wage.scale(c.fee / 12.0);
        }
    }
    for club in dropped {
        state.person.circle.clubs.retain(|&c| c != club);
        news.push(Message::new(MessageKind::Warning, keys::CLUB_DROPPED).with(
            "club",
            Param::TextKey(format!("club.{}", m.clubs[club as usize].key)),
        ));
    }
    let cost = cost.min(state.person.account.balance);
    if cost > Money::ZERO {
        crate::private::book_flow(state, PrivateFlow::CircleCost, -cost);
    }
    // The standing drifts to its target.
    let goal = target(catalog, state);
    state.person.circle.standing = Some((have + (goal - have) * m.rate).clamp(0.0, 100.0));
    // The contacts: care, ending, new ones.
    let share = crate::wellbeing::time(catalog, state).map_or(10.0, |t| f64::from(t.network));
    let n = state.person.circle.contacts.len();
    // Few contacts; the cast is exact.
    #[allow(clippy::cast_precision_loss)]
    let load = (5.0 / n.max(1) as f64).min(1.0);
    let (gain, decay) = (m.care_gain * share / 10.0 * load, m.care_decay);
    for c in &mut state.person.circle.contacts {
        c.strength = (c.strength + gain - decay).clamp(0.0, 100.0);
    }
    let (gone, kept): (Vec<Contact>, Vec<Contact>) =
        std::mem::take(&mut state.person.circle.contacts)
            .into_iter()
            .partition(|c| c.until <= date);
    state.person.circle.contacts = kept;
    for c in gone {
        news.push(
            Message::new(MessageKind::Info, keys::CONTACT_DIED)
                .with("name", Param::Text(c.name))
                .with("rolle", Param::TextKey(format!("rolle.{}", c.role.key()))),
        );
    }
    let mut taken: BTreeSet<String> = state
        .person
        .circle
        .contacts
        .iter()
        .map(|c| c.name.clone())
        .collect();
    for &club in &state.person.circle.clone().clubs {
        let Some(c) = m.clubs.get(club as usize) else {
            continue;
        };
        if rng.chance(c.contacts_year / 12.0) && !c.roles.is_empty() {
            let role = c.roles[usize::try_from(rng.below(c.roles.len() as u64)).unwrap_or(0)];
            let name =
                crate::management::manager_name(catalog, &mut rng, state.person.home, &mut taken);
            let years =
                m.contact_years[0] + (m.contact_years[1] - m.contact_years[0]) * rng.next_f64();
            let until = date.add_months((years * 12.0).round() as u32);
            news.push(
                Message::new(MessageKind::Info, keys::CONTACT_MET)
                    .with("name", Param::Text(name.clone()))
                    .with("rolle", Param::TextKey(format!("rolle.{}", role.key())))
                    .with("club", Param::TextKey(format!("club.{}", c.key))),
            );
            state.person.circle.contacts.push(Contact {
                role,
                name,
                since: date,
                until,
                strength: 20.0,
            });
        }
    }
    news.extend(hints(state, catalog, &m, date, &mut rng));
    news
}

/// Hints of the contacts: an unhappy manager, a start-up, a change of the income tax.
fn hints(
    state: &GameState,
    catalog: &Catalog,
    m: &StandingModel,
    date: Date,
    rng: &mut SimRng,
) -> Vec<Message> {
    let mut news = Vec::new();
    for c in &state.person.circle.contacts {
        if !rng.chance(m.tip_chance * c.strength / 100.0) {
            continue;
        }
        match c.role {
            ContactRole::Entrepreneur => {
                // An unhappy manager of another company on the market for people.
                let tip = state.managers.iter().find(|(_, x)| {
                    x.job.as_ref().is_some_and(|j| {
                        !state.is_main(j.company)
                            && crate::staffing::satisfaction(catalog, j)
                                < catalog.management.staffing.poach_below
                    })
                });
                if let Some((_, x)) = tip {
                    news.push(
                        Message::new(MessageKind::Info, keys::TIP_MANAGER)
                            .with("kontakt", Param::Text(c.name.clone()))
                            .with("name", Param::Text(x.name.clone())),
                    );
                }
            }
            ContactRole::Scientist => {
                if let Some(v) = state
                    .ventures
                    .iter()
                    .filter(|v| matches!(v.status, crate::state::VentureStatus::Active))
                    .max_by(|a, b| {
                        a.chance
                            .partial_cmp(&b.chance)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                {
                    news.push(
                        Message::new(MessageKind::Info, keys::TIP_VENTURE)
                            .with("kontakt", Param::Text(c.name.clone()))
                            .with("name", Param::Text(v.name.clone())),
                    );
                }
            }
            ContactRole::Politician => {
                let now = catalog
                    .person
                    .income_tax
                    .value(state.person.home, date.year_fraction());
                let next = catalog
                    .person
                    .income_tax
                    .value(state.person.home, date.year_fraction() + 1.0);
                if (next - now).abs() >= 0.01 {
                    news.push(
                        Message::new(MessageKind::Info, keys::TIP_TAX)
                            .with("kontakt", Param::Text(c.name.clone()))
                            .with(
                                "richtung",
                                Param::TextKey(
                                    if next > now {
                                        "hinweis.steigt"
                                    } else {
                                        "hinweis.sinkt"
                                    }
                                    .to_owned(),
                                ),
                            ),
                    );
                }
            }
            ContactRole::Banker | ContactRole::Broker => {}
        }
    }
    news
}
