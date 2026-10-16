//! Staff of the central departments as a decision of their heads (MG8, docs/FORMELN.md,
//! MG8): each head estimates the workload, recommends the staff and either sets it (a CEO
//! with the switch, within the limit) or asks the player; a file reviews the decision.

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::{Catalog, DepartmentKind};
use crate::command::{self, Command};
use crate::decision::{Choice, ChoiceKind, Decision, Topic};
use crate::management;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{
    CompanyId, Concern, ConcernOption, ConcernReason, ConcernStatus, GameState, ManagerId, Role,
    Unit,
};

/// A staff decision of a department head waiting for its review.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StaffFile {
    pub company: CompanyId,
    pub kind: DepartmentKind,
    pub head: ManagerId,
    pub due: Date,
}

/// The staff a head recommends: his estimate of the load (error by judgment) over the
/// cases of an employee, times the factor of his style, within what the city's academics
/// allow. `None` without the department in the data or a head.
pub fn recommended(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    kind: DepartmentKind,
) -> Option<u32> {
    let d = catalog.central.department(kind)?;
    let head = crate::central::head(catalog, state, company, kind)?;
    let m = state.managers.get(&head)?;
    let mut rng = SimRng::for_stream(
        state.settings.seed,
        Stream::Manager {
            id: head.0,
            day: state.date.ordinal(),
        },
    );
    let error = (rng.next_f64() * 2.0 - 1.0)
        * catalog.management.concerns.estimate_error
        * (1.0 - f64::from(m.judgment) / 100.0);
    let load = crate::central::workload(catalog, state, company, kind) * (1.0 + error);
    let style = management::style_of(catalog, m).map_or(1.0, |k| k.staff);
    if d.cases <= 0.0 {
        return None;
    }
    // Staff counts are small; the cast saturates.
    let wanted = ((load / d.cases).ceil() * style).round().max(1.0) as u32;
    Some(wanted.min(crate::central::max_staff(catalog, state, company, kind).max(1)))
}

/// The month start: the quarterly review of every department of the player's company, and
/// outside it where the department is under- or overstaffed.
pub(crate) fn month_start(
    state: &mut GameState,
    catalog: &Catalog,
    date: Date,
    news: &mut Vec<Message>,
) {
    review(state, catalog, date, news);
    let Some(company) = state.main_company else {
        return;
    };
    if !catalog.management.enabled() || state.companies[company.index()].bankrupt {
        return;
    }
    let model = catalog.management.staffing;
    let quarter = management::month_number(date).is_multiple_of(3);
    for kind in catalog
        .central
        .departments
        .iter()
        .map(|d| d.kind)
        .collect::<Vec<_>>()
    {
        let Some(head) = crate::central::head(catalog, state, company, kind) else {
            continue;
        };
        let Some(d) = catalog.central.department(kind) else {
            continue;
        };
        // Only a department the company has set up asks for a different staff.
        if crate::central::staff(state, company, kind) == 0 {
            continue;
        }
        let perf = crate::central::performance(catalog, state, company, kind);
        let load = crate::central::workload(catalog, state, company, kind);
        let off_course = perf.is_none_or(|p| {
            p.coverage < model.under || (p.capacity > 0.0 && load < model.over * p.capacity)
        });
        if !quarter && !off_course {
            continue;
        }
        let Some(wanted) = recommended(catalog, state, company, kind) else {
            continue;
        };
        let now = crate::central::staff(state, company, kind);
        if wanted == now || asked(state, company, kind) {
            continue;
        }
        let extra = crate::central::yearly_cost(catalog, state, company, kind, wanted)
            - crate::central::yearly_cost(catalog, state, company, kind, now);
        let ceo_switch = management::position_state(
            state,
            company,
            &crate::state::Position::new(Unit::Board, Role::Head),
        )
        .is_some_and(|p| p.hires);
        let limit = state.companies[company.index()]
            .participations
            .limits
            .get(&kind)
            .copied()
            .unwrap_or_else(|| Money::from_usd(model.limit_usd).unwrap_or(Money::ZERO));
        let command = Command::StaffDepartment {
            department: kind,
            staff: wanted,
        };
        let _ = d;
        if ceo_switch && extra <= limit {
            if command::execute(state, catalog, company, &command).is_ok() {
                register(state, catalog, company, kind, head);
                news.push(
                    Message::new(MessageKind::Info, keys::STAFF_NEED_SET)
                        .with(
                            "abteilung",
                            Param::TextKey(format!("abteilung.{}", kind.key())),
                        )
                        .with("von", Param::Integer(i64::from(now)))
                        .with("auf", Param::Integer(i64::from(wanted)))
                        .with("kosten", Param::Money(extra)),
                );
            }
        } else {
            ask(
                state,
                catalog,
                (company, kind, head),
                (command, extra),
                date,
            );
        }
    }
}

/// Whether a question about the staff of a department is open.
fn asked(state: &GameState, company: CompanyId, kind: DepartmentKind) -> bool {
    state.concerns.iter().any(|c| {
        c.company == company
            && c.status == ConcernStatus::Open
            && c.decision.topic == Topic::StaffNeed
            && c.decision.choices.iter().any(|ch| {
                ch.steps.iter().any(|s| {
                    matches!(&s.command, Command::StaffDepartment { department, .. } if *department == kind)
                })
            })
    })
}

fn ask(
    state: &mut GameState,
    catalog: &Catalog,
    (company, kind, head): (CompanyId, DepartmentKind, ManagerId),
    (command, extra): (Command, Money),
    today: Date,
) {
    let Some(position) = crate::central::head_position(catalog, kind) else {
        return;
    };
    let choices = vec![Choice::one(ChoiceKind::Adjust, command), Choice::keep()];
    let options = vec![
        ConcernOption {
            amount: extra.max(Money::ZERO),
            forecast: None,
            once: Money::ZERO,
        },
        ConcernOption {
            amount: Money::ZERO,
            forecast: None,
            once: Money::ZERO,
        },
    ];
    let id = state.next_concern;
    state.next_concern += 1;
    let deadline =
        today.add_days(i32::try_from(catalog.management.concerns.deadline_days).unwrap_or(30));
    state.concerns.push(Concern {
        id,
        company,
        position,
        manager: head,
        decision: Decision {
            topic: Topic::StaffNeed,
            company,
            site: None,
            product: None,
            choices,
            rule: 0,
        },
        recommended: 0,
        options,
        reason: ConcernReason::StaffNeed,
        path: Vec::new(),
        parts: Vec::new(),
        created: today,
        deadline,
        status: ConcernStatus::Open,
        closed: None,
        memory: None,
        dissent: None,
        statement: None,
    });
}

/// Keeps the file of a staff decision.
pub(crate) fn register(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    kind: DepartmentKind,
    head: ManagerId,
) {
    let due = state
        .date
        .add_months(catalog.management.staffing.dept_review_months);
    state.staff_files.push(StaffFile {
        company,
        kind,
        head,
        due,
    });
}

/// The reviews that are due: a hit where the department covers its workload (not under the
/// limit of the data) without being idle (not over the other limit); it counts in the
/// hit rate of its head.
fn review(state: &mut GameState, catalog: &Catalog, today: Date, news: &mut Vec<Message>) {
    let model = catalog.management.staffing;
    let (due, rest): (Vec<StaffFile>, Vec<StaffFile>) = std::mem::take(&mut state.staff_files)
        .into_iter()
        .partition(|f| f.due <= today);
    state.staff_files = rest;
    for f in due {
        let perf = crate::central::performance(catalog, state, f.company, f.kind);
        let load = crate::central::workload(catalog, state, f.company, f.kind);
        let hit =
            perf.is_some_and(|p| p.coverage >= model.under && load >= model.over * p.capacity);
        if let Some(m) = state.managers.get_mut(&f.head) {
            m.judged += 1;
            m.hits += u32::from(hit);
        }
        if state.is_main(f.company) {
            news.push(
                Message::new(
                    if hit {
                        MessageKind::Success
                    } else {
                        MessageKind::Warning
                    },
                    if hit {
                        keys::STAFF_NEED_HIT
                    } else {
                        keys::STAFF_NEED_MISS
                    },
                )
                .with(
                    "abteilung",
                    Param::TextKey(format!("abteilung.{}", f.kind.key())),
                ),
            );
        }
    }
}
