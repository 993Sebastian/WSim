//! The headquarters and the central departments (ZA1–ZA3; docs/BEDIENUNG.md,
//! "Organisation").

use serde::{Deserialize, Serialize};

use super::{iso, usd};
use crate::catalog::DepartmentKind;
use crate::central;
use crate::game::Game;
use crate::management;
use crate::money::Money;

/// A country the headquarters could move to: what it would mean for taxes and salaries.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeatCountryView {
    pub country: String,
    /// Profit tax (share).
    pub tax: f64,
    /// Yearly wage of the salary group of managers (USD): what the board's salaries
    /// follow.
    pub wage_usd: f64,
}

/// A move under way.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RelocationView {
    pub country: String,
    /// The new seat holds from the first month start on or after this day.
    pub until: String,
}

/// A central department (ZA2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DepartmentView {
    /// As the command `StaffDepartment` names it, e.g. `Finance`.
    pub kind: String,
    /// Key of the texts (`abteilung.<key>`).
    pub key: String,
    /// The board's function whose member heads it (`bereich.<function>`).
    pub function: String,
    pub staff: u32,
    /// The head's name; none while the position is vacant.
    pub head: Option<String>,
    /// The head's expertise in the function as the player sees it (1–5).
    pub head_level: Option<u8>,
    /// The head's hit rate and judged estimates (ZA3); the rate none before the first.
    pub head_hit_rate: Option<f64>,
    pub head_judged: u32,
    /// Cases per month: employees · cases each.
    pub capacity: f64,
    pub cases_each: f64,
    /// What there is to do in a month.
    pub workload: f64,
    /// Share of the workload covered (0–1).
    pub coverage: f64,
    /// With head and employees.
    pub working: bool,
    /// Its effect at full quality and coverage.
    pub effect: f64,
    /// Countries the strategy department observes, technologies the legal department
    /// checks; 0 for the others.
    pub reach: u32,
    pub monthly_cost_usd: f64,
    /// What one more employee costs a month.
    pub cost_per_employee_usd: f64,
    /// Up to which amount its head decides alone; none without a limit.
    pub release_limit_usd: Option<f64>,
}

/// The policy „Beteiligungen“ (ZA2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParticipationsView {
    /// Per year; none without a limit.
    pub budget_usd: Option<f64>,
    pub risk: f64,
    /// Bought this year: takeovers and licences.
    pub spent_usd: f64,
    /// Bids still open.
    pub open_bids_usd: f64,
    /// What is left of the budget; none without one.
    pub left_usd: Option<f64>,
    /// The release limit of each department of the data.
    pub limits: Vec<ReleaseLimitView>,
}

/// Up to which amount a department's head decides alone.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReleaseLimitView {
    /// As `DepartmentView::kind`.
    pub kind: String,
    pub key: String,
    /// None: only the budget of the position binds.
    pub limit_usd: Option<f64>,
}

/// The policy „Beteiligungen“ of the player's company (ZA2).
pub fn participations(game: &Game) -> ParticipationsView {
    let c = game.catalog();
    let state = game.state();
    let player = game.player();
    let p = &state.companies[player.index()].participations;
    let open_bids: Money = state
        .offers
        .iter()
        .filter(|o| o.status == crate::deals::OfferStatus::Open && o.buyer == player)
        .map(|o| o.price)
        .sum();
    ParticipationsView {
        budget_usd: p.budget.map(usd),
        risk: p.risk,
        spent_usd: usd(p.spent_in(state.date.year())),
        open_bids_usd: usd(open_bids),
        left_usd: central::participations_left(state, player).map(usd),
        limits: c
            .central
            .departments
            .iter()
            .map(|d| ReleaseLimitView {
                kind: format!("{:?}", d.kind),
                key: d.kind.key().to_owned(),
                limit_usd: p.limits.get(&d.kind).map(|&m| usd(m)),
            })
            .collect(),
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CentralView {
    /// The country of the headquarters.
    pub country: String,
    pub tax: f64,
    pub wage_usd: f64,
    pub relocation: Option<RelocationView>,
    /// What a move costs now, how long it takes and the share of employees moving along.
    pub move_cost_usd: f64,
    pub move_months: u32,
    pub moving_share: f64,
    /// All countries to choose from, in the order of the data.
    pub countries: Vec<SeatCountryView>,
    /// The departments of the data, in their order (ZA2).
    pub departments: Vec<DepartmentView>,
    pub employees: u32,
    /// Salaries and offices of all departments a month.
    pub monthly_cost_usd: f64,
    pub participations: ParticipationsView,
}

/// The player's headquarters.
pub fn central(game: &Game) -> CentralView {
    let c = game.catalog();
    let state = game.state();
    let player = game.player();
    let company = &state.companies[player.index()];
    let h = &c.central.headquarters;
    let seat = |country: crate::ids::CountryId| SeatCountryView {
        country: c.countries.key(country).to_owned(),
        tax: state.countries.get(country).corporate_tax,
        wage_usd: management::yearly_wage(c, state, country),
    };
    let here = seat(company.headquarters);
    let share = management::impression_share(c, state, player);
    let departments = c
        .central
        .departments
        .iter()
        .map(|d| {
            let kind = d.kind;
            let function = c.management.functions[d.function].key.clone();
            let staff = central::staff(state, player, kind);
            let head = central::head(c, state, player, kind).and_then(|id| state.managers.get(&id));
            let head_level = head.map(|m| {
                let key = management::expertise_key(&function);
                let value = management::skill(m, &key).unwrap_or(0);
                let impression = management::shown_impression(
                    m.impression.get(&key).copied().unwrap_or(0),
                    share,
                );
                management::shown_level(value, impression)
            });
            let capacity = f64::from(staff) * d.cases;
            let workload = central::workload(state, player, kind);
            let reach = match kind {
                DepartmentKind::Strategy => central::observed_countries(c, state, player).len(),
                DepartmentKind::Legal => central::legal_technologies(c, state, player).len(),
                _ => 0,
            };
            let wage = management::group_yearly_wage(c, state, company.headquarters, d.labor_group);
            let per_employee = wage / 12.0 + d.office.to_usd() / 12.0;
            DepartmentView {
                kind: format!("{kind:?}"),
                key: kind.key().to_owned(),
                function,
                staff,
                head: head.map(|m| m.name.clone()),
                head_level,
                head_hit_rate: head
                    .filter(|m| m.judged > 0)
                    .map(|m| central::hit_rate(c, m)),
                head_judged: head.map_or(0, |m| m.judged),
                capacity,
                cases_each: d.cases,
                workload,
                coverage: if workload > 0.0 {
                    (capacity / workload).min(1.0)
                } else {
                    1.0
                },
                working: central::performance(c, state, player, kind).is_some(),
                effect: d.effect,
                reach: u32::try_from(reach).unwrap_or(u32::MAX),
                monthly_cost_usd: per_employee * f64::from(staff),
                cost_per_employee_usd: per_employee,
                release_limit_usd: company.participations.limits.get(&kind).map(|&m| usd(m)),
            }
        })
        .collect();
    let (personnel, office) = central::monthly_cost(c, state, player);
    CentralView {
        country: here.country,
        tax: here.tax,
        wage_usd: here.wage_usd,
        relocation: company.relocation.map(|r| RelocationView {
            country: c.countries.key(r.country).to_owned(),
            until: iso(r.until),
        }),
        move_cost_usd: usd(central::relocation_cost(c, state, player)),
        move_months: h.months,
        moving_share: h.moving_share,
        countries: c.countries.ids().map(seat).collect(),
        departments,
        employees: central::employees(state, player),
        monthly_cost_usd: usd(personnel + office),
        participations: participations(game),
    }
}
