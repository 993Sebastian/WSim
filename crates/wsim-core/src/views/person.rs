//! The player as a person (PE2): profile, family, roles and chronicle.

use serde::{Deserialize, Serialize};

use super::{MessageView, iso, message_view};
use crate::aging;
use crate::game::Game;
use crate::management;
use crate::message::{Message, MessageKind, Param};
use crate::person;
use crate::state::{LifeEventKind, Position, Role, Unit};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChildView {
    pub name: String,
    pub born: String,
    /// Age today, or at death.
    pub age: u32,
    pub died: Option<String>,
    /// The manager card: its id and, while employed, the position in words.
    pub manager: Option<u32>,
    pub position: Option<MessageView>,
    /// Years until the manager card.
    pub card_in: Option<u32>,
}

/// A company the person holds shares of.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HoldingView {
    pub company: String,
    pub share: f64,
    pub controlled: bool,
    /// The person leads it as CEO; otherwise the manager who does, if any.
    pub person_ceo: bool,
    pub ceo: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LifeEventView {
    pub date: String,
    pub text: MessageView,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersonView {
    pub name: String,
    pub born: String,
    pub age: u32,
    /// Country of residence (ISO).
    pub home: String,
    pub married: bool,
    pub children: Vec<ChildView>,
    /// Children still possible: the person is married and of age for them.
    pub children_possible: bool,
    pub holdings: Vec<HoldingView>,
    /// The chronicle, the latest first.
    pub history: Vec<LifeEventView>,
}

pub fn person(game: &Game) -> PersonView {
    let state = game.state();
    let catalog = game.catalog();
    let p = &state.person;
    let today = state.date;
    let children = p
        .children
        .iter()
        .map(|c| {
            let manager = c
                .manager
                .and_then(|id| state.managers.get(&id).map(|m| (id, m)));
            let position = manager.and_then(|(_, m)| {
                let job = m.job.as_ref()?;
                Some(message_view(
                    &Message::new(MessageKind::Info, "person.stelle")
                        .with("stelle", management::position_param(state, &job.position))
                        .with(
                            "ort",
                            crate::staffing::place_param(catalog, state, job.position.unit),
                        )
                        .with(
                            "firma",
                            Param::Text(state.companies[job.company.index()].name.clone()),
                        ),
                ))
            });
            let age = aging::age(c.born, c.died.unwrap_or(today));
            let card = catalog.person.card_age;
            ChildView {
                name: c.name.clone(),
                born: iso(c.born),
                age,
                died: c.died.map(iso),
                manager: manager.map(|(id, _)| id.0),
                position,
                card_in: (c.died.is_none() && age < card).then(|| card - age),
            }
        })
        .collect();
    let model = &catalog.person;
    let age = person::age(state, today);
    let holdings = state
        .companies
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            // Few companies; the cast is exact.
            let id = crate::state::CompanyId(i as u32);
            let share = person::share(state, id);
            (share > 0.0).then(|| HoldingView {
                company: c.name.clone(),
                share,
                controlled: share > 0.5,
                person_ceo: p.ceo == Some(id),
                ceo: management::holder(state, id, &Position::new(Unit::Board, Role::Head))
                    .map(|m| state.managers[&m].name.clone()),
            })
        })
        .collect();
    let company = |c: crate::state::CompanyId| {
        Param::Text(
            state
                .companies
                .get(c.index())
                .map_or_else(String::new, |x| x.name.clone()),
        )
    };
    let history = p
        .history
        .iter()
        .rev()
        .map(|e| {
            let m = |key: &str| Message::new(MessageKind::Info, key);
            let text = match &e.kind {
                LifeEventKind::Start { company: c } => {
                    m("person.ereignis.start").with("firma", company(*c))
                }
                LifeEventKind::Takeover { company: c } => {
                    m("person.ereignis.uebernahme").with("firma", company(*c))
                }
                LifeEventKind::ChildBorn { name } => {
                    m("person.ereignis.geburt").with("name", Param::Text(name.clone()))
                }
                LifeEventKind::Career { name } => {
                    m("person.ereignis.beruf").with("name", Param::Text(name.clone()))
                }
                LifeEventKind::ChildDied { name } => {
                    m("person.ereignis.kind_tot").with("name", Param::Text(name.clone()))
                }
                LifeEventKind::CeoHandedOver { company: c, to } => m("person.ereignis.vorsitz_ab")
                    .with("firma", company(*c))
                    .with("name", Param::Text(to.clone())),
                LifeEventKind::CeoTakenBack { company: c } => {
                    m("person.ereignis.vorsitz_an").with("firma", company(*c))
                }
            };
            LifeEventView {
                date: iso(e.date),
                text: message_view(&text),
            }
        })
        .collect();
    PersonView {
        name: p.name.clone(),
        born: iso(p.born),
        age,
        home: catalog.countries.key(p.home).to_owned(),
        married: p.married,
        children,
        children_possible: model.enabled
            && p.married
            && age < model.children_ages[1]
            && p.children.len() < usize::try_from(model.children_max).unwrap_or(0),
        holdings,
        history,
    }
}
