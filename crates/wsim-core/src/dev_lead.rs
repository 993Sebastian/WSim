//! Development leadership (PD11, docs/FORMELN.md, PD11): the heads of the laboratories and
//! the research specialists of country, continent and board plan the staff of the
//! laboratories ahead of the needs of the projects, within a budget handed down the levels.
//! The plan acts through the same hiring and dismissing as the player (`lab_staff`).

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::{Catalog, LabRole, LeadershipModel, SiteType};
use crate::ids::{BranchId, Id};
use crate::management;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::state::{CompanyId, GameState, ManagerId, Position, Role, SiteId, Unit};

/// What a leader of a unit may do (the frame he works in).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DevFrame {
    pub company: CompanyId,
    pub unit: Unit,
    /// Hires on his own; otherwise only recommends.
    pub own: bool,
    /// May dismiss people.
    pub dismiss: bool,
    /// Reserve on the need; `None`: the data's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reserve: Option<f64>,
    /// Branches whose projects he plans for; empty: all.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub branches: Vec<BranchId>,
}

impl DevFrame {
    fn standard(company: CompanyId, unit: Unit) -> Self {
        DevFrame {
            company,
            unit,
            own: true,
            dismiss: true,
            reserve: None,
            branches: Vec::new(),
        }
    }
}

/// People ordered or moved who have not arrived yet.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DevOrder {
    pub site: SiteId,
    pub role: LabRole,
    pub count: u32,
    pub arrive: Date,
    /// Moved from another laboratory (else hired).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<SiteId>,
}

/// What the plan keeps for a laboratory.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DevSite {
    pub site: Option<SiteId>,
    /// Months the surplus of a role has lasted.
    pub surplus: [u32; 4],
    /// Project months lost to understaffing, summed.
    pub delay: f64,
    /// Month of the last message about the laboratory.
    pub noted: u32,
    /// Severance and the like paid by the plan, in total.
    pub severance: Money,
}

/// The leader responsible for a laboratory: the first filled position of laboratory head,
/// research specialist of the country, of the continent and of the board.
pub fn lead_of(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    site: SiteId,
) -> Option<(Position, ManagerId)> {
    management::chain(
        catalog,
        state,
        company,
        Unit::Site(site),
        crate::decision::Topic::Research,
    )
    .into_iter()
    .filter(|p| {
        matches!(
            (p.unit, &p.role),
            (Unit::Site(_), Role::Head)
                | (
                    Unit::Country(_) | Unit::Continent(_) | Unit::Board,
                    Role::Specialist(_)
                )
        )
    })
    .find_map(|p| management::holder(state, company, &p).map(|m| (p, m)))
}

/// Whether a laboratory is staffed by the plan: the data has the leadership and a leader is
/// in place.
pub fn planned(catalog: &Catalog, state: &GameState, company: CompanyId, site: SiteId) -> bool {
    catalog
        .research_model
        .projects
        .as_ref()
        .is_some_and(|m| m.leadership.is_some())
        && lead_of(catalog, state, company, site).is_some()
}

/// The frame of a unit of a company.
pub fn frame_of(state: &GameState, company: CompanyId, unit: Unit) -> DevFrame {
    state
        .dev_frames
        .iter()
        .find(|f| f.company == company && f.unit == unit)
        .cloned()
        .unwrap_or_else(|| DevFrame::standard(company, unit))
}

/// Monthly wage of a person in a laboratory (the researchers' group of the country).
pub fn lab_wage(catalog: &Catalog, state: &GameState, site: SiteId) -> Money {
    let country = state.sites[site.index()].country;
    let group = catalog
        .research_model
        .researchers
        .iter()
        .copied()
        .flatten()
        .next();
    let yearly = group.map_or_else(
        || management::yearly_wage(catalog, state, country),
        |g| management::group_yearly_wage(catalog, state, country, g),
    );
    Money::from_usd(yearly / 12.0).unwrap_or(Money::ZERO)
}

/// Posts of a laboratory today.
fn posts(catalog: &Catalog, state: &GameState, site: SiteId) -> u32 {
    let s = &state.sites[site.index()];
    // Posts are few; the cast is exact.
    crate::lab_staff::capacity(catalog, s, state.date)
        .floor()
        .max(0.0) as u32
}

/// The need and the target per role of a laboratory (docs/FORMELN.md, PD11).
fn targets(
    catalog: &Catalog,
    state: &GameState,
    model: &LeadershipModel,
    site: SiteId,
    (frame, risk): (&DevFrame, u8),
) -> [u32; 4] {
    let Some(m) = catalog.research_model.projects.as_ref() else {
        return [0; 4];
    };
    let horizon_days = i32::try_from(model.horizon_months * 30).unwrap_or(i32::MAX);
    let mut need = [0u32; 4];
    for p in state
        .dev_projects
        .iter()
        .filter(|p| p.running() && p.site == site)
    {
        if !frame.branches.is_empty()
            && !frame
                .branches
                .contains(&catalog.products.get(p.product).branch)
        {
            continue;
        }
        let phase = usize::from(p.phase);
        let soon = crate::projects::days_left_in_phase(m, p) <= f64::from(horizon_days);
        for role in LabRole::ALL {
            let now = crate::projects::need(m, phase, role, p.effort);
            let next = if soon && phase + 1 < m.phases.len() {
                crate::projects::need(m, phase + 1, role, p.effort)
            } else {
                0
            };
            need[role.index()] += now.max(next);
        }
    }
    let character = lerp(
        model.reserve_generous,
        model.reserve_thrifty,
        f64::from(risk) / 100.0,
    );
    let reserve = frame.reserve.unwrap_or(model.reserve) * character;
    let mut out = [0u32; 4];
    for role in LabRole::ALL {
        let b = f64::from(need[role.index()]);
        // Counts are small; the cast is exact.
        out[role.index()] = (b * (1.0 + reserve)).ceil() as u32;
    }
    out
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// What a laboratory plans in a month.
struct Plan {
    site: SiteId,
    lead: Position,
    frame: DevFrame,
    wage: f64,
    posts: u32,
    target: [u32; 4],
    have: [u32; 4],
    coming: [u32; 4],
    free: [u32; 4],
    /// Budget in USD a year after the cascade.
    budget: f64,
}

impl Plan {
    fn total(&self) -> u32 {
        self.have.iter().sum::<u32>() + self.coming.iter().sum::<u32>()
    }

    fn gap(&self, role: LabRole) -> u32 {
        let i = role.index();
        self.target[i].saturating_sub(self.have[i] + self.coming[i])
    }

    fn cost_year(&self, people: u32) -> f64 {
        f64::from(people) * self.wage * 12.0
    }
}

/// Spreads the budget of a company over its planned laboratories: by posts, then what a
/// laboratory does not need goes to those short of it, within the country first, then
/// the continent, then the company (docs/FORMELN.md, PD11).
fn cascade(catalog: &Catalog, state: &GameState, model: &LeadershipModel, plans: &mut [Plan]) {
    if plans.is_empty() {
        return;
    }
    let company = state.sites[plans[0].site.index()].owner;
    let all: Vec<SiteId> = (0..state.sites.len())
        .map(|i| SiteId(u32::try_from(i).expect("site count fits u32")))
        .filter(|&s| {
            let x = &state.sites[s.index()];
            x.held_by(company) && x.kind == SiteType::ResearchCenter
        })
        .collect();
    let post_sum: f64 = all
        .iter()
        .map(|&s| f64::from(posts(catalog, state, s)))
        .sum();
    let base: f64 = all
        .iter()
        .map(|&s| f64::from(posts(catalog, state, s)) * lab_wage(catalog, state, s).to_usd() * 12.0)
        .sum::<f64>()
        * model.base_share;
    let revenue = management::budget_base(catalog, state, company, Unit::Board).to_usd();
    let total = base.max(revenue * model.budget_share);
    for p in plans.iter_mut() {
        p.budget = if post_sum > 0.0 {
            total * f64::from(p.posts) / post_sum
        } else {
            0.0
        };
    }
    let need = |p: &Plan| p.cost_year(p.target.iter().sum::<u32>().max(p.have.iter().sum()));
    for scope in 0..3 {
        let key = |p: &Plan| -> u32 {
            let s = &state.sites[p.site.index()];
            match scope {
                0 => s.country.index() as u32,
                1 => catalog.countries.get(s.country).continent.index() as u32,
                _ => 0,
            }
        };
        let mut keys: Vec<u32> = plans.iter().map(key).collect();
        keys.sort_unstable();
        keys.dedup();
        for k in keys {
            let spare: f64 = plans
                .iter()
                .filter(|p| key(p) == k)
                .map(|p| (p.budget - need(p)).max(0.0))
                .sum();
            let deficit: f64 = plans
                .iter()
                .filter(|p| key(p) == k)
                .map(|p| (need(p) - p.budget).max(0.0))
                .sum();
            if spare <= 0.0 || deficit <= 0.0 {
                continue;
            }
            let moved = spare.min(deficit);
            for p in plans.iter_mut().filter(|p| key(p) == k) {
                let n = need(p);
                if p.budget > n {
                    p.budget -= (p.budget - n) * moved / spare;
                } else if n > p.budget {
                    p.budget += (n - p.budget) * moved / deficit;
                }
            }
        }
    }
}

/// `SetDevFrame`: the frame of a unit of the actor's company.
pub(crate) fn set_frame(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    frame: DevFrame,
) -> Result<(), crate::command::CommandError> {
    use crate::command::CommandError;
    if frame.company != actor || !management::has_unit(catalog, state, actor, frame.unit) {
        return Err(CommandError::NotOwner);
    }
    if frame.reserve.is_some_and(|r| !(0.0..=1.0).contains(&r)) {
        return Err(CommandError::InvalidQuantity);
    }
    match state
        .dev_frames
        .iter_mut()
        .find(|f| f.company == frame.company && f.unit == frame.unit)
    {
        Some(f) => *f = frame,
        None => {
            state.dev_frames.push(frame);
            state
                .dev_frames
                .sort_by(|a, b| (a.company, a.unit).cmp(&(b.company, b.unit)));
        }
    }
    Ok(())
}

fn site_state(state: &mut GameState, site: SiteId) -> &mut DevSite {
    let at = match state.dev_sites.iter().position(|d| d.site == Some(site)) {
        Some(i) => i,
        None => {
            state.dev_sites.push(DevSite {
                site: Some(site),
                ..DevSite::default()
            });
            state.dev_sites.sort_by_key(|d| d.site);
            state
                .dev_sites
                .iter()
                .position(|d| d.site == Some(site))
                .expect("just inserted")
        }
    };
    &mut state.dev_sites[at]
}

/// Orders that are due arrive: hired people join, moved ones start working.
fn arrivals(state: &mut GameState, catalog: &Catalog, date: Date) {
    let due: Vec<DevOrder> = state
        .dev_orders
        .iter()
        .filter(|o| o.arrive <= date)
        .cloned()
        .collect();
    state.dev_orders.retain(|o| o.arrive > date);
    for o in due {
        let Some(s) = state.sites.get(o.site.index()) else {
            continue;
        };
        if s.closed.is_some() {
            continue;
        }
        let owner = s.owner;
        let room = i64::from(posts(catalog, state, o.site))
            - i64::from(state.sites[o.site.index()].lab_staff.total());
        let count = o.count.min(u32::try_from(room.max(0)).unwrap_or(0));
        if count > 0 {
            // The people were paid for in the plan; a failing hire only drops the order.
            let _ =
                crate::lab_staff::change(state, catalog, owner, (o.site, o.role), (count, true));
        }
    }
}

/// Month start: the plan of every company with a leader for its laboratories.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    let Some(model) = catalog
        .research_model
        .projects
        .as_ref()
        .and_then(|m| m.leadership.as_ref())
    else {
        return news;
    };
    arrivals(state, catalog, date);
    let month = management::month_number(date);
    for ci in 0..state.companies.len() {
        let company = CompanyId(u32::try_from(ci).expect("company count fits u32"));
        if state.companies[ci].bankrupt {
            continue;
        }
        let mut plans = plan_company(state, catalog, model, company);
        if plans.is_empty() {
            continue;
        }
        cascade(catalog, state, model, &mut plans);
        for plan in &plans {
            news.extend(run_plan(
                state,
                catalog,
                model,
                (plan, &plans),
                (date, month),
            ));
        }
        // Months of delay by understaffing, for the balance of the levels.
        for plan in &plans {
            let late: f64 = state
                .dev_projects
                .iter()
                .filter(|p| p.running() && p.site == plan.site)
                .map(|p| shortfall_of(catalog, p))
                .sum();
            site_state(state, plan.site).delay += late;
        }
    }
    news
}

/// Share of the people a project's phase needs that are missing today.
pub fn shortfall_of(catalog: &Catalog, p: &crate::projects::DevProject) -> f64 {
    let Some(m) = catalog.research_model.projects.as_ref() else {
        return 0.0;
    };
    let phase = usize::from(p.phase);
    let (mut need, mut miss) = (0.0, 0.0);
    for role in LabRole::ALL {
        let n = f64::from(crate::projects::need(m, phase, role, p.effort));
        need += n;
        miss += (n - f64::from(p.assigned[role.index()])).max(0.0);
    }
    if need > 0.0 { miss / need } else { 0.0 }
}

fn plan_company(
    state: &GameState,
    catalog: &Catalog,
    model: &LeadershipModel,
    company: CompanyId,
) -> Vec<Plan> {
    let mut plans = Vec::new();
    for i in 0..state.sites.len() {
        let site = SiteId(u32::try_from(i).expect("site count fits u32"));
        let s = &state.sites[i];
        if !s.held_by(company) || s.kind != SiteType::ResearchCenter {
            continue;
        }
        let Some((lead, manager)) = lead_of(catalog, state, company, site) else {
            continue;
        };
        let frame = frame_of(state, company, lead.unit);
        let risk = state.managers.get(&manager).map_or(50, |m| m.risk);
        let mut coming = [0u32; 4];
        for o in state.dev_orders.iter().filter(|o| o.site == site) {
            coming[o.role.index()] += o.count;
        }
        let mut have = [0u32; 4];
        let mut free = [0u32; 4];
        for role in LabRole::ALL {
            have[role.index()] = s.lab_staff.get(role);
            free[role.index()] = s.lab_staff.free(role);
        }
        plans.push(Plan {
            site,
            target: targets(catalog, state, model, site, (&frame, risk)),
            wage: lab_wage(catalog, state, site).to_usd(),
            posts: posts(catalog, state, site),
            lead,
            frame,
            have,
            coming,
            free,
            budget: 0.0,
        });
    }
    plans
}

fn scope_covers(catalog: &Catalog, state: &GameState, lead: Unit, other: SiteId) -> bool {
    let s = &state.sites[other.index()];
    match lead {
        Unit::Site(x) => x == other,
        Unit::Country(c) => s.country == c,
        Unit::Continent(k) => catalog.countries.get(s.country).continent == k,
        Unit::Board => true,
    }
}

/// One laboratory: moves, hires and dismissals, or the recommendation.
fn run_plan(
    state: &mut GameState,
    catalog: &Catalog,
    model: &LeadershipModel,
    (plan, all): (&Plan, &[Plan]),
    (date, month): (Date, u32),
) -> Vec<Message> {
    let mut news = Vec::new();
    let company = state.sites[plan.site.index()].owner;
    let mut gap = [0u32; 4];
    let mut surplus = [0u32; 4];
    for role in LabRole::ALL {
        let i = role.index();
        gap[i] = plan.gap(role);
        let needed = plan.target[i].max(state.sites[plan.site.index()].lab_staff.assigned[i]);
        surplus[i] = (plan.have[i] + plan.coming[i])
            .saturating_sub(needed)
            .min(plan.free[i]);
    }
    {
        let d = site_state(state, plan.site);
        for i in 0..4 {
            d.surplus[i] = if surplus[i] > 0 { d.surplus[i] + 1 } else { 0 };
        }
    }
    let patient: [bool; 4] = {
        let d = site_state(state, plan.site);
        std::array::from_fn(|i| d.surplus[i] > model.patience_months)
    };
    if !plan.frame.own {
        if (gap.iter().any(|&g| g > 0) || surplus.iter().any(|&s| s > 0))
            && site_state(state, plan.site).noted != month
        {
            site_state(state, plan.site).noted = month;
            news.push(note(state, plan.site, keys::DEV_LEAD_RECOMMEND, &gap));
        }
        return news;
    }
    let arrive = date.add_months(model.lead_months);
    let mut room = i64::from(plan.posts) - i64::from(plan.total());
    // Moves first: a laboratory with a lasting surplus gives to those of its leader's scope.
    for role in LabRole::ALL {
        let i = role.index();
        if !patient[i] || surplus[i] == 0 {
            continue;
        }
        for other in all.iter().filter(|o| o.site != plan.site) {
            if surplus[i] == 0 {
                break;
            }
            if !scope_covers(catalog, state, plan.lead.unit, other.site)
                || state.sites[other.site.index()].owner != company
            {
                continue;
            }
            let want = other.gap(role).min(surplus[i]);
            let target_room = i64::from(other.posts) - i64::from(other.total());
            let n = want.min(u32::try_from(target_room.max(0)).unwrap_or(0));
            if n == 0 {
                continue;
            }
            if crate::lab_staff::change(state, catalog, company, (plan.site, role), (n, false))
                .is_ok()
            {
                state.dev_orders.push(DevOrder {
                    site: other.site,
                    role,
                    count: n,
                    arrive: date.add_months(model.induction_months),
                    from: Some(plan.site),
                });
                surplus[i] -= n;
            }
        }
    }
    // Hires within the budget; what the budget does not carry is escalated.
    let mut people = plan.total();
    let mut short = [0u32; 4];
    for role in LabRole::ALL {
        let i = role.index();
        let mut n = gap[i].min(u32::try_from(room.max(0)).unwrap_or(0));
        while n > 0 && plan.cost_year(people + n) > plan.budget {
            n -= 1;
        }
        if n > 0 {
            state.dev_orders.push(DevOrder {
                site: plan.site,
                role,
                count: n,
                arrive,
                from: None,
            });
            people += n;
            room -= i64::from(n);
        }
        short[i] = gap[i].saturating_sub(n);
    }
    if short.iter().any(|&s| s > 0) && site_state(state, plan.site).noted != month {
        site_state(state, plan.site).noted = month;
        news.push(note(state, plan.site, keys::DEV_LEAD_ESCALATED, &short));
    }
    // Dismissals of a lasting surplus, with severance.
    if plan.frame.dismiss {
        for role in LabRole::ALL {
            let i = role.index();
            if !patient[i] || surplus[i] == 0 {
                continue;
            }
            let n = surplus[i];
            if crate::lab_staff::change(state, catalog, company, (plan.site, role), (n, false))
                .is_ok()
            {
                let pay =
                    Money::from_usd(f64::from(n) * plan.wage * f64::from(model.severance_months))
                        .unwrap_or(Money::ZERO);
                management::book_personnel(state, company, Unit::Site(plan.site), pay);
                site_state(state, plan.site).severance += pay;
            }
        }
    }
    news
}

fn note(state: &GameState, site: SiteId, key: &str, gap: &[u32; 4]) -> Message {
    let mut m = Message::new(MessageKind::Warning, key).with(
        "standort",
        crate::command::site_label(&state.sites[site.index()]),
    );
    for role in LabRole::ALL {
        m = m.with(role.key(), Param::Integer(i64::from(gap[role.index()])));
    }
    m
}

/// A laboratory with a development leader as the plan sees it now (for the view).
pub struct LabRow {
    pub site: SiteId,
    pub lead: Position,
    pub manager: ManagerId,
    pub frame: DevFrame,
    pub have: [u32; 4],
    pub coming: [u32; 4],
    pub target: [u32; 4],
    pub free: [u32; 4],
    pub posts: u32,
    pub budget_usd: f64,
    pub cost_usd: f64,
    pub delay: f64,
    pub severance: Money,
}

/// The planned laboratories of a company.
pub fn rows(catalog: &Catalog, state: &GameState, company: CompanyId) -> Vec<LabRow> {
    let Some(model) = catalog
        .research_model
        .projects
        .as_ref()
        .and_then(|m| m.leadership.as_ref())
    else {
        return Vec::new();
    };
    let mut plans = plan_company(state, catalog, model, company);
    cascade(catalog, state, model, &mut plans);
    plans
        .into_iter()
        .filter_map(|p| {
            let manager = lead_of(catalog, state, company, p.site)?.1;
            let kept = state.dev_sites.iter().find(|d| d.site == Some(p.site));
            let cost = p.cost_year(p.have.iter().sum());
            Some(LabRow {
                site: p.site,
                lead: p.lead,
                manager,
                frame: p.frame,
                have: p.have,
                coming: p.coming,
                target: p.target,
                free: p.free,
                posts: p.posts,
                budget_usd: p.budget,
                cost_usd: cost,
                delay: kept.map_or(0.0, |d| d.delay),
                severance: kept.map_or(Money::ZERO, |d| d.severance),
            })
        })
        .collect()
}
