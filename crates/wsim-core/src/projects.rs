//! Development projects PM0–PM7 (PD2, docs/PRODUKTENTWICKLUNG.md, formulas in
//! docs/FORMELN.md, PD2).
//!
//! A project turns the missing technologies of a product into a staged development: six
//! phases between the milestones, each driven by the people assigned from the laboratory's
//! pools and by a project leader. At PM7 the company learns what the product needs.
//! Decisions at the milestones, events and reports follow with PD3 and PD4; until then the
//! project passes the milestones by itself.

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::{Catalog, LabRole, ProjectModel, SiteType};
use crate::command::CommandError;
use crate::ids::{CountryId, Id, ProductId};
use crate::ledger::{Account, CostCenter, CostType};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{CompanyId, GameState, SiteId};

/// The milestone a phase ends at: PM1, PM2, PM3, PM5, PM6, PM7 (PM4 is the gate).
pub const PHASE_END_MILESTONE: [u8; 6] = [1, 2, 3, 5, 6, 7];

/// A person who leads development projects.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectLead {
    pub name: String,
    pub home: CountryId,
    pub born: Date,
    /// Index into the characters of the data.
    pub character: u8,
    /// Experience from 0 to 1.
    pub experience: f64,
    #[serde(default)]
    pub finished: u32,
    #[serde(default)]
    pub aborted: u32,
    /// The company that employs the leader; none on the market.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub employer: Option<CompanyId>,
    /// The day the leader came on the market.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entered: Option<Date>,
}

/// Where a project stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectStatus {
    Running,
    /// Released at PM7.
    Released,
    Aborted,
}

/// One development project.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DevProject {
    pub company: CompanyId,
    pub site: SiteId,
    pub product: ProductId,
    /// Number of the leader in `project_leads`.
    pub lead: u32,
    /// Index into the model's tempos and trims.
    pub tempo: u8,
    pub trim: u8,
    pub started: Date,
    /// Research points the project covers (W), fixed at the start.
    pub effort: f64,
    /// Current phase 0–5; 6 once released.
    pub phase: u8,
    /// Share of the current phase done, 0–1.
    pub progress: f64,
    /// People per role, as the owner set them (when `auto` is off).
    pub wanted: [u32; 4],
    /// People assigned today (set every day from the lab's free people).
    #[serde(default)]
    pub assigned: [u32; 4],
    /// The project takes the people the phase needs from the lab's free people.
    pub auto: bool,
    pub status: ProjectStatus,
    pub spent: Money,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished: Option<Date>,
}

impl DevProject {
    pub fn running(&self) -> bool {
        self.status == ProjectStatus::Running
    }
}

/// Whether the data have the development projects.
pub fn enabled(catalog: &Catalog) -> bool {
    catalog.research_model.projects.is_some()
}

fn scale(m: &ProjectModel, effort: f64) -> f64 {
    crate::math::pow(effort / m.reference_effort, m.need_exponent)
}

/// People a role needs in a phase for a project of `effort` points.
pub fn need(m: &ProjectModel, phase: usize, role: LabRole, effort: f64) -> u32 {
    let Some(pr) = m.phases[phase].roles.iter().find(|r| r.role == role) else {
        return 0;
    };
    // Counts are small; the cast is exact.
    (pr.need * scale(m, effort)).ceil() as u32
}

/// Gain in speed of a role by its staffing q (assigned / need).
fn gain(m: &ProjectModel, q: f64) -> f64 {
    if q >= 1.0 {
        1.0 + m.over_max * (1.0 - crate::math::exp(-(q - 1.0) / m.over_tau))
    } else {
        q.max(m.rest_share)
    }
}

fn staffing(m: &ProjectModel, phase: usize, effort: f64, assigned: [u32; 4]) -> (f64, f64) {
    // Speed of the phase and the effect-weighted shortfall (1 − min(1, q)).
    let mut speed = 0.0;
    let mut shortfall = 0.0;
    for pr in &m.phases[phase].roles {
        let n = need(m, phase, pr.role, effort);
        let q = if n == 0 {
            1.0
        } else {
            f64::from(assigned[pr.role.index()]) / f64::from(n)
        };
        speed += pr.effect * gain(m, q);
        shortfall += pr.effect * (1.0 - q.min(1.0));
    }
    (speed, shortfall)
}

/// Days a phase takes at the reference speed.
pub fn phase_days(m: &ProjectModel, tempo: usize, phase: usize, effort: f64) -> f64 {
    effort / m.reference_rate * m.phases[phase].time * m.tempos[tempo].time
}

/// Cost of a phase in USD at price level 1.
pub fn phase_cost_usd(
    m: &ProjectModel,
    (tempo, trim): (usize, usize),
    phase: usize,
    effort: f64,
) -> f64 {
    effort * m.cost_per_point_usd * m.phases[phase].cost * m.tempos[tempo].cost * m.trims[trim].cost
}

/// Cost of the gate PM3→PM4 in USD at price level 1.
pub fn gate_cost_usd(m: &ProjectModel, (tempo, trim): (usize, usize), effort: f64) -> f64 {
    effort * m.cost_per_point_usd * m.gate_cost * m.tempos[tempo].cost * m.trims[trim].cost
}

/// Risk of a phase (the counter for the events of PD4): base risk by tempo, understaffing
/// and the leader's experience.
pub fn phase_risk(m: &ProjectModel, tempo: usize, shortfall: f64, experience: f64) -> f64 {
    let r = &m.risk;
    (r.base
        * m.tempos[tempo].risk
        * (1.0 + r.shortfall * shortfall)
        * (1.0 - r.experience * experience))
        .max(0.0)
}

/// One staffing scenario of the leader: what the rest of the project takes if each role
/// has `delta` people more or fewer than it needs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scenario {
    pub delta: i32,
    /// Days until PM7.
    pub days: f64,
    /// Chance of a problem over the rest of the project.
    pub risk: f64,
    /// Loss on the figures from understaffing, 0 – cap.
    pub penalty: f64,
}

/// The scenario `delta` for the rest of the project from `phase` with `progress`.
pub fn scenario(m: &ProjectModel, p: &DevProject, experience: f64, delta: i32) -> Scenario {
    let tempo = usize::from(p.tempo);
    let (mut days, mut risk_free, mut short_sum, mut weight) = (0.0, 1.0, 0.0, 0.0);
    for phase in usize::from(p.phase).min(5)..6 {
        let mut assigned = [0u32; 4];
        for role in LabRole::ALL {
            let n = need(m, phase, role, p.effort);
            assigned[role.index()] = (i64::from(n) + i64::from(delta)).max(0) as u32;
        }
        let (speed, shortfall) = staffing(m, phase, p.effort, assigned);
        let left = if phase == usize::from(p.phase) {
            1.0 - p.progress
        } else {
            1.0
        };
        let d = phase_days(m, tempo, phase, p.effort) * left / speed.max(1e-9);
        days += d;
        risk_free *= 1.0 - phase_risk(m, tempo, shortfall, experience) * left;
        short_sum += shortfall * d;
        weight += d;
    }
    let shortfall = if weight > 0.0 {
        short_sum / weight
    } else {
        0.0
    };
    Scenario {
        delta,
        days,
        risk: 1.0 - risk_free,
        penalty: (m.shortfall_penalty * shortfall).min(m.shortfall_cap),
    }
}

/// The research points a project for `product` covers: what the company still needs to
/// learn, less what it has collected. `None` if there is nothing to develop.
pub fn project_effort(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    product: ProductId,
    date: Date,
) -> Option<f64> {
    if !crate::product_research::can_research(catalog, state, company, product) {
        return None;
    }
    let (needed, collected) =
        crate::product_research::effort(catalog, state, company, product, date);
    Some((needed - collected).max(1.0))
}

fn lead_busy(state: &GameState, lead: u32) -> usize {
    state
        .dev_projects
        .iter()
        .filter(|p| p.running() && p.lead == lead)
        .count()
}

fn price_level(state: &GameState, site: SiteId) -> f64 {
    state
        .countries
        .get(state.sites[site.index()].country)
        .price_level
}

/// `StartProject`: PM0, a new project in a laboratory with a leader of the company.
pub(crate) fn start(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    (site, product, lead): (SiteId, ProductId, u32),
    (tempo, trim): (u8, u8),
) -> Result<(), CommandError> {
    let Some(m) = catalog.research_model.projects.as_ref() else {
        return Err(CommandError::ProjectsOff);
    };
    let Some(s) = state.sites.get(site.index()) else {
        return Err(CommandError::UnknownSite);
    };
    if !s.held_by(actor) {
        return Err(CommandError::NotOwner);
    }
    if s.kind != SiteType::ResearchCenter {
        return Err(CommandError::WrongSiteType {
            required: SiteType::ResearchCenter,
        });
    }
    if usize::from(tempo) >= m.tempos.len() || usize::from(trim) >= m.trims.len() {
        return Err(CommandError::InvalidQuantity);
    }
    let Some(leader) = state.project_leads.get(lead as usize) else {
        return Err(CommandError::LeadUnavailable);
    };
    if leader.employer != Some(actor) || lead_busy(state, lead) >= m.lead.capacity_new as usize {
        return Err(CommandError::LeadUnavailable);
    }
    if state
        .dev_projects
        .iter()
        .any(|p| p.running() && p.company == actor && p.product == product)
    {
        return Err(CommandError::ProjectExists);
    }
    let Some(effort) = project_effort(catalog, state, actor, product, state.date) else {
        return Err(CommandError::NotResearchableProduct(
            catalog.products.key(product).to_owned(),
        ));
    };
    state.dev_projects.push(DevProject {
        company: actor,
        site,
        product,
        lead,
        tempo,
        trim,
        started: state.date,
        effort,
        phase: 0,
        progress: 0.0,
        wanted: [0; 4],
        assigned: [0; 4],
        auto: true,
        status: ProjectStatus::Running,
        spent: Money::ZERO,
        finished: None,
    });
    state.sites[site.index()].staffing_due = true;
    Ok(())
}

/// `AssignProjectStaff`: the owner sets how many people of a role work on a project.
pub(crate) fn assign(
    state: &mut GameState,
    actor: CompanyId,
    project: u32,
    (role, count): (LabRole, u32),
) -> Result<(), CommandError> {
    let Some(p) = state.dev_projects.get_mut(project as usize) else {
        return Err(CommandError::UnknownProject);
    };
    if p.company != actor {
        return Err(CommandError::NotOwner);
    }
    if !p.running() {
        return Err(CommandError::UnknownProject);
    }
    if !p.auto {
        p.wanted[role.index()] = count;
    } else {
        // Switching to manual keeps what works today.
        p.wanted = p.assigned;
        p.wanted[role.index()] = count;
        p.auto = false;
    }
    Ok(())
}

/// `AutoAssignProject`: the project takes the people each phase needs again.
pub(crate) fn auto_assign(
    state: &mut GameState,
    actor: CompanyId,
    project: u32,
) -> Result<(), CommandError> {
    let Some(p) = state.dev_projects.get_mut(project as usize) else {
        return Err(CommandError::UnknownProject);
    };
    if p.company != actor || !p.running() {
        return Err(CommandError::UnknownProject);
    }
    p.auto = true;
    Ok(())
}

/// `AbortProject`: the project stops; the leader is free and counts it as aborted.
pub(crate) fn abort(
    state: &mut GameState,
    actor: CompanyId,
    project: u32,
) -> Result<(), CommandError> {
    let Some(p) = state.dev_projects.get_mut(project as usize) else {
        return Err(CommandError::UnknownProject);
    };
    if p.company != actor || !p.running() {
        return Err(CommandError::UnknownProject);
    }
    p.status = ProjectStatus::Aborted;
    p.finished = Some(state.date);
    let lead = p.lead as usize;
    if let Some(l) = state.project_leads.get_mut(lead) {
        l.aborted += 1;
    }
    Ok(())
}

/// `HireProjectLead`: a leader of the market joins the company.
pub(crate) fn hire_lead(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    lead: u32,
) -> Result<(), CommandError> {
    if catalog.research_model.projects.is_none() {
        return Err(CommandError::ProjectsOff);
    }
    let date = state.date;
    let Some(l) = state.project_leads.get_mut(lead as usize) else {
        return Err(CommandError::LeadUnavailable);
    };
    if l.employer.is_some() || age(l.born, date) >= 66.0 {
        return Err(CommandError::LeadUnavailable);
    }
    l.employer = Some(actor);
    l.entered = None;
    Ok(())
}

/// `DismissProjectLead`: an idle leader of the company leaves for the market.
pub(crate) fn dismiss_lead(
    state: &mut GameState,
    actor: CompanyId,
    lead: u32,
) -> Result<(), CommandError> {
    if lead_busy(state, lead) > 0 {
        return Err(CommandError::LeadUnavailable);
    }
    let date = state.date;
    let Some(l) = state.project_leads.get_mut(lead as usize) else {
        return Err(CommandError::LeadUnavailable);
    };
    if l.employer != Some(actor) {
        return Err(CommandError::NotOwner);
    }
    l.employer = None;
    l.entered = Some(date);
    Ok(())
}

/// Age in years on `date`.
pub fn age(born: Date, date: Date) -> f64 {
    f64::from(born.days_until(date)) / 365.25
}

/// Today's step: assigns the people, advances every running project and releases the
/// finished ones.
pub(crate) fn simulate_day(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    let Some(m) = catalog.research_model.projects.as_ref() else {
        return news;
    };
    for site in &mut state.sites {
        site.lab_staff.assigned = [0; 4];
    }
    for index in 0..state.dev_projects.len() {
        if !state.dev_projects[index].running() {
            continue;
        }
        let (company, site, phase, effort) = {
            let p = &state.dev_projects[index];
            (p.company, p.site, usize::from(p.phase), p.effort)
        };
        if state.companies[company.index()].bankrupt {
            state.dev_projects[index].status = ProjectStatus::Aborted;
            state.dev_projects[index].finished = Some(date);
            continue;
        }
        // The people: what the phase needs, or what the owner wants, of the free ones.
        let mut assigned = [0u32; 4];
        for role in LabRole::ALL {
            let p = &state.dev_projects[index];
            let wish = if p.auto {
                need(m, phase, role, effort)
            } else {
                p.wanted[role.index()]
            };
            let staff = &mut state.sites[site.index()].lab_staff;
            let give = wish.min(staff.free(role));
            staff.assigned[role.index()] += give;
            assigned[role.index()] = give;
        }
        let (speed, _) = staffing(m, phase, effort, assigned);
        let (tempo, trim, progress) = {
            let p = &mut state.dev_projects[index];
            p.assigned = assigned;
            (usize::from(p.tempo), usize::from(p.trim), p.progress)
        };
        let days = phase_days(m, tempo, phase, effort).max(1e-9);
        let step = (speed / days).min(1.0 - progress);
        let cost = Money::from_usd(
            phase_cost_usd(m, (tempo, trim), phase, effort) * step * price_level(state, site),
        )
        .unwrap_or(Money::ZERO);
        book(state, index, cost);
        state.dev_projects[index].progress += step;
        if state.dev_projects[index].progress < 1.0 - 1e-9 {
            continue;
        }
        // The phase is done: a milestone.
        let milestone = PHASE_END_MILESTONE[phase];
        {
            let p = &mut state.dev_projects[index];
            p.phase += 1;
            p.progress = 0.0;
        }
        news.extend(milestone_message(state, catalog, index, milestone));
        if phase == 2 {
            // Gate PM3→PM4: the release of the main budget costs its preparation.
            let gate =
                Money::from_usd(gate_cost_usd(m, (tempo, trim), effort) * price_level(state, site))
                    .unwrap_or(Money::ZERO);
            book(state, index, gate);
            news.extend(milestone_message(state, catalog, index, 4));
        }
        if phase == 5 {
            news.extend(release(state, catalog, index, date));
        }
    }
    news
}

fn book(state: &mut GameState, index: usize, amount: Money) {
    let (company, site) = {
        let p = &state.dev_projects[index];
        (p.company, p.site)
    };
    state.companies[company.index()].ledger.expense(
        CostType::Research,
        CostCenter::site(site),
        Account::Cash,
        amount,
    );
    state.dev_projects[index].spent += amount;
}

fn milestone_message(
    state: &GameState,
    catalog: &Catalog,
    index: usize,
    milestone: u8,
) -> Option<Message> {
    let p = &state.dev_projects[index];
    if !state.is_main(p.company) {
        return None;
    }
    Some(
        Message::new(MessageKind::Info, keys::PROJECT_MILESTONE)
            .with(
                "produkt",
                Param::TextKey(format!("produkt.{}", catalog.products.key(p.product))),
            )
            .with("meilenstein", Param::Number(f64::from(milestone))),
    )
}

/// PM7: the company learns what the product needs; the leader gains experience.
fn release(state: &mut GameState, catalog: &Catalog, index: usize, date: Date) -> Vec<Message> {
    let (company, product, lead) = {
        let p = &state.dev_projects[index];
        (p.company, p.product, p.lead)
    };
    let mut news = Vec::new();
    for t in crate::product_research::missing(catalog, state, company, product) {
        crate::research::acquire(state, catalog, company.index(), t, date, &mut news);
    }
    let c = &mut state.companies[company.index()];
    c.known_products.insert(product);
    c.product_points.remove(&product);
    let gain = catalog
        .research_model
        .projects
        .as_ref()
        .map_or(0.0, |m| m.lead.experience_gain);
    if let Some(l) = state.project_leads.get_mut(lead as usize) {
        l.finished += 1;
        l.experience = (l.experience + gain).min(1.0);
    }
    let p = &mut state.dev_projects[index];
    p.status = ProjectStatus::Released;
    p.finished = Some(date);
    if state.is_main(company) {
        news.push(
            Message::new(MessageKind::Success, keys::PRODUCT_RESEARCHED).with(
                "produkt",
                Param::TextKey(format!("produkt.{}", catalog.products.key(product))),
            ),
        );
    }
    news
}

/// Month start: leaders are paid, the market is topped up, old projects leave the list.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) {
    let Some(m) = catalog.research_model.projects.as_ref() else {
        return;
    };
    // Salaries of the employed leaders, at the laboratory they work for (else the first).
    for lead in 0..state.project_leads.len() {
        let Some(company) = state.project_leads[lead].employer else {
            continue;
        };
        let lead_id = u32::try_from(lead).expect("lead count fits u32");
        let site = state
            .dev_projects
            .iter()
            .find(|p| p.running() && p.lead == lead_id)
            .map(|p| p.site)
            .or_else(|| {
                state
                    .sites
                    .iter()
                    .position(|s| s.kind == SiteType::ResearchCenter && s.held_by(company))
                    .map(|i| SiteId(u32::try_from(i).expect("site count fits u32")))
            });
        let Some(site) = site else {
            continue;
        };
        let pay = Money::from_usd(m.lead.salary_usd_year / 12.0 * price_level(state, site))
            .unwrap_or(Money::ZERO);
        state.companies[company.index()].ledger.expense(
            CostType::Research,
            CostCenter::site(site),
            Account::Cash,
            pay,
        );
    }
    top_up_market(state, catalog, date);
}

/// Keeps `kandidaten` free leaders per continent on the market.
pub(crate) fn top_up_market(state: &mut GameState, catalog: &Catalog, date: Date) {
    let Some(m) = catalog.research_model.projects.as_ref() else {
        return;
    };
    let month = crate::management::month_number(date);
    let mut taken: std::collections::BTreeSet<String> = state
        .project_leads
        .iter()
        .map(|l| l.name.clone())
        .chain(state.managers.values().map(|x| x.name.clone()))
        .collect();
    for (continent, _) in catalog.continents.iter() {
        let countries: Vec<CountryId> = catalog
            .countries
            .iter()
            .filter(|&(_, c)| c.continent == continent)
            .map(|(id, _)| id)
            .collect();
        if countries.is_empty() {
            continue;
        }
        let free = state
            .project_leads
            .iter()
            .filter(|l| {
                l.employer.is_none() && catalog.countries.get(l.home).continent == continent
            })
            .count();
        let mut rng = SimRng::for_stream(
            state.settings.seed,
            Stream::ProjectLeads {
                continent: u16::try_from(continent.index()).unwrap_or(u16::MAX),
                month,
            },
        );
        for _ in free..m.lead.candidates as usize {
            let home = countries[usize::try_from(rng.below(countries.len() as u64)).unwrap_or(0)];
            let name = crate::management::manager_name(catalog, &mut rng, home, &mut taken);
            let span = u64::from(m.lead.age_max - m.lead.age_min).max(1);
            let years = f64::from(m.lead.age_min) + rng.below(span) as f64 + rng.next_f64();
            let born = date.add_days(-((years * 365.25) as i32));
            let character = u8::try_from(rng.below(m.lead.characters.len() as u64)).unwrap_or(0);
            // Experience grows with the years of work: about 0.4 at the youngest, 1 at the oldest.
            let experience = (0.2
                + 0.8 * (years - f64::from(m.lead.age_min)) / span as f64
                + (rng.next_f64() - 0.5) * 0.2)
                .clamp(0.0, 1.0);
            state.project_leads.push(ProjectLead {
                name,
                home,
                born,
                character,
                experience,
                finished: 0,
                aborted: 0,
                employer: None,
                entered: Some(date),
            });
        }
    }
}
