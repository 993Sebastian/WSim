//! Product suggestions of the strategy, the promise frozen at the project and the
//! after-calculation (PD6, docs/PRODUKTENTWICKLUNG.md §5; formulas in docs/FORMELN.md, PD6).
//!
//! Once a month a company with a laboratory gets a suggestion for a product that fits it.
//! The forecast comes from the unbiased figures of a project not yet started, blurred by
//! the spread the strategy department cannot remove. Starting from a suggestion freezes the
//! forecast at the project; two years after the release the actual figures are compared
//! with it, and the result goes into the balance of the project leader.

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::{Catalog, DepartmentKind, SiteType, SuggestionModel};
use crate::command::CommandError;
use crate::ids::{Id, ProductId};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::project_reports::{self, Kpis};
use crate::projects::{self, DevProject, ProjectStatus};
use crate::rng::{SimRng, Stream};
use crate::state::{CompanyId, GameState, SiteId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestionStatus {
    Open,
    /// Started as the project with this number.
    Started(u32),
    Declined,
    Expired,
}

/// The result of the after-calculation.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Review {
    pub date: Date,
    /// Total value 0–1.
    pub value: f64,
    pub hit: bool,
}

/// A product suggestion with its forecast.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Suggestion {
    pub company: CompanyId,
    pub site: SiteId,
    pub product: ProductId,
    pub created: Date,
    pub expires: Date,
    pub forecast: Kpis,
    /// `branche` (the company makes goods of the branch) or `luecke` (open demand).
    pub reason: String,
    pub status: SuggestionStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review: Option<Review>,
}

/// What the forecast promised, frozen at the project.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Promise {
    pub suggestion: u32,
    pub forecast: Kpis,
}

fn model(catalog: &Catalog) -> Option<&SuggestionModel> {
    catalog
        .research_model
        .projects
        .as_ref()?
        .suggestions
        .as_ref()
}

fn id32(index: usize) -> u32 {
    u32::try_from(index).unwrap_or(u32::MAX)
}

/// A project that has not started, with the choices of the standard (profile and path in
/// the middle), for the unbiased figures it would have today.
fn hypothetical(
    state: &GameState,
    (company, site, product): (CompanyId, SiteId, ProductId),
    effort: f64,
    choices: (u8, u8, u8, u8),
) -> DevProject {
    DevProject {
        company,
        site,
        product,
        lead: u32::MAX,
        tempo: choices.0,
        trim: choices.1,
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
        profile: choices.2,
        path: choices.3,
        conditions: 0,
        loops: 0,
        paused: false,
        delegation: None,
        pending: None,
        queued: Vec::new(),
        baseline: None,
        reports: Vec::new(),
        event: None,
        event_phase: None,
        defect: 0.0,
        bonus_quality: 0.0,
        head_start: 0.0,
        subsidy: None,
        first_mover: false,
        platform: 0.0,
        license: None,
        partner: None,
        origin: None,
        fast_track: false,
        pilot: 0,
        shortfall_sum: 0.0,
        promise: None,
    }
}

/// A laboratory of the company with at least one working place.
fn lab_of(state: &GameState, catalog: &Catalog, company: CompanyId, date: Date) -> Option<SiteId> {
    state.sites.iter().enumerate().find_map(|(i, s)| {
        (s.kind == SiteType::ResearchCenter
            && s.held_by(company)
            && crate::lab_staff::capacity(catalog, s, date) >= 1.0)
            .then(|| SiteId(id32(i)))
    })
}

/// Branches the company makes goods in.
fn branches(state: &GameState, catalog: &Catalog, company: CompanyId) -> Vec<crate::ids::BranchId> {
    let mut out = Vec::new();
    for s in state.sites.iter().filter(|s| s.held_by(company)) {
        for sl in &s.slots {
            if let Some(r) = sl.recipe {
                let b = catalog.products.get(catalog.recipes.get(r).product).branch;
                if !out.contains(&b) {
                    out.push(b);
                }
            }
        }
    }
    out
}

/// The best product to suggest to the company, with the reason.
fn best_product(
    state: &GameState,
    catalog: &Catalog,
    m: &SuggestionModel,
    company: CompanyId,
    date: Date,
) -> Option<(ProductId, f64, &'static str)> {
    let mine = branches(state, catalog, company);
    let mut best: Option<(ProductId, f64, &'static str, f64)> = None;
    for index in 0..catalog.products.len() {
        let product = ProductId::from_index(index);
        if state
            .dev_projects
            .iter()
            .any(|p| p.running() && p.company == company && p.product == product)
            || state.product_suggestions.iter().any(|s| {
                s.company == company
                    && s.product == product
                    && (s.status == SuggestionStatus::Open
                        || (s.status == SuggestionStatus::Declined
                            && s.created.days_until(date) < 365))
            })
        {
            continue;
        }
        let Some(effort) = projects::project_effort(catalog, state, company, product, date) else {
            continue;
        };
        let open: f64 = state
            .markets
            .get(product)
            .iter()
            .map(|(_, mk)| crate::market::open_demand(mk, &catalog.market_model, date))
            .sum();
        let p = catalog.products.get(product);
        let mut value = open * p.reference_price.to_usd();
        let own = mine.contains(&p.branch);
        if own {
            value *= 1.0 + m.branch_bonus;
        }
        if value <= 0.0 {
            continue;
        }
        let score = value / effort.max(1.0);
        if best.is_none_or(|b| score > b.3) {
            best = Some((
                product,
                effort,
                if own { "branche" } else { "luecke" },
                score,
            ));
        }
    }
    best.map(|(p, e, r, _)| (p, e, r))
}

/// Once a month: offers expire, products are suggested, after-calculations come due.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let Some(m) = model(catalog) else {
        return Vec::new();
    };
    let mut news = Vec::new();
    for s in &mut state.product_suggestions {
        if s.status == SuggestionStatus::Open && s.expires <= date {
            s.status = SuggestionStatus::Expired;
        }
    }
    news.extend(reviews(state, catalog, m, date));
    for index in 0..state.companies.len() {
        let company = CompanyId(id32(index));
        if state.companies[index].bankrupt {
            continue;
        }
        let open = state
            .product_suggestions
            .iter()
            .filter(|s| s.company == company && s.status == SuggestionStatus::Open)
            .count();
        if open >= m.open_max as usize {
            continue;
        }
        let Some(site) = lab_of(state, catalog, company, date) else {
            continue;
        };
        let Some((product, effort, reason)) = best_product(state, catalog, m, company, date) else {
            continue;
        };
        let truth = project_reports::truth_kpis(
            state,
            catalog,
            &hypothetical(state, (company, site, product), effort, (1, 1, 1, 1)),
        );
        let strength = crate::central::strength(catalog, state, company, DepartmentKind::Strategy);
        let sigma = m.uncertainty * (1.0 - m.accuracy * strength);
        let mut rng = SimRng::for_stream(
            state.settings.seed,
            Stream::ProductSuggestion {
                company: id32(index),
                month: u32::try_from(date.year() * 12).unwrap_or(0) + date.month(),
            },
        );
        let bias = (1.0 + sigma * (2.0 * rng.next_f64() - 1.0)).max(0.1);
        let forecast = project_reports::forecast(&truth, bias);
        state.product_suggestions.push(Suggestion {
            company,
            site,
            product,
            created: date,
            expires: date.add_days(i32::try_from(m.valid_days).unwrap_or(180)),
            forecast,
            reason: reason.to_owned(),
            status: SuggestionStatus::Open,
            review: None,
        });
        if state.is_main(company) {
            news.push(
                Message::new(MessageKind::Info, keys::PRODUCT_SUGGESTION).with(
                    "produkt",
                    Param::TextKey(format!("produkt.{}", catalog.products.key(product))),
                ),
            );
        }
    }
    news
}

fn value(actual: f64, promised: f64, tolerance: f64) -> f64 {
    let d = (actual - promised).abs() / promised.abs().max(1e-9);
    (1.0 - d / tolerance).max(0.0)
}

/// After-calculation of released projects started from a suggestion.
fn reviews(
    state: &mut GameState,
    catalog: &Catalog,
    m: &SuggestionModel,
    date: Date,
) -> Vec<Message> {
    let mut news = Vec::new();
    for si in 0..state.product_suggestions.len() {
        let s = &state.product_suggestions[si];
        let SuggestionStatus::Started(pi) = s.status else {
            continue;
        };
        if s.review.is_some() {
            continue;
        }
        let Some(p) = state.dev_projects.get(pi as usize) else {
            continue;
        };
        let (ProjectStatus::Released, Some(done)) = (p.status, p.finished) else {
            continue;
        };
        if done.add_days(365 * i32::try_from(m.review_years).unwrap_or(2)) > date {
            continue;
        }
        let forecast = s.forecast;
        let (company, site, product, lead) = (p.company, p.site, p.product, p.lead);
        let choices = (p.tempo, p.trim, p.profile, p.path);
        let days = f64::from(p.started.days_until(done));
        let cost = p.spent.to_usd() / projects::price_level(state, site).max(1e-9);
        let mut today = hypothetical(state, (company, site, product), p.effort, choices);
        today.started = date;
        let actual = project_reports::truth_kpis(state, catalog, &today);
        let figures = (value(actual.unit_cost, forecast.unit_cost, m.tolerance)
            + value(
                actual.contribution_year,
                forecast.contribution_year,
                m.tolerance,
            )
            + value(actual.quality, forecast.quality, m.tolerance))
            / 3.0;
        let total = m.weights[0] * value(days, forecast.project_days, m.tolerance)
            + m.weights[1] * value(cost, forecast.project_cost, m.tolerance)
            + m.weights[2] * figures;
        let hit = total >= m.hit_from;
        state.product_suggestions[si].review = Some(Review {
            date,
            value: total,
            hit,
        });
        if let Some(l) = state.project_leads.get_mut(lead as usize) {
            l.balance_sum += total;
            l.balance_count += 1;
        }
        if state.is_main(company) {
            news.push(
                Message::new(
                    if hit {
                        MessageKind::Success
                    } else {
                        MessageKind::Warning
                    },
                    keys::PRODUCT_REVIEW,
                )
                .with(
                    "produkt",
                    Param::TextKey(format!("produkt.{}", catalog.products.key(product))),
                )
                .with("wert", Param::Number(total)),
            );
        }
    }
    news
}

/// `StartSuggestedProject`: starts the project of an open suggestion and freezes its forecast.
pub(crate) fn start(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    (suggestion, lead): (u32, u32),
    (tempo, trim): (u8, u8),
) -> Result<(), CommandError> {
    if model(catalog).is_none() {
        return Err(CommandError::ProjectsOff);
    }
    let Some(s) = state.product_suggestions.get(suggestion as usize) else {
        return Err(CommandError::UnknownSuggestion);
    };
    if s.company != actor || s.status != SuggestionStatus::Open {
        return Err(CommandError::UnknownSuggestion);
    }
    let (site, product, forecast) = (s.site, s.product, s.forecast);
    projects::start(state, catalog, actor, (site, product, lead), (tempo, trim))?;
    let index = state.dev_projects.len() - 1;
    state.dev_projects[index].promise = Some(Promise {
        suggestion,
        forecast,
    });
    state.product_suggestions[suggestion as usize].status = SuggestionStatus::Started(id32(index));
    Ok(())
}

/// `DeclineSuggestion`.
pub(crate) fn decline(
    state: &mut GameState,
    actor: CompanyId,
    suggestion: u32,
) -> Result<(), CommandError> {
    let Some(s) = state.product_suggestions.get_mut(suggestion as usize) else {
        return Err(CommandError::UnknownSuggestion);
    };
    if s.company != actor || s.status != SuggestionStatus::Open {
        return Err(CommandError::UnknownSuggestion);
    }
    s.status = SuggestionStatus::Declined;
    Ok(())
}

/// Share of the reviewed suggestions of a company that were hits.
pub fn hit_rate(state: &GameState, company: CompanyId) -> Option<f64> {
    let reviewed: Vec<bool> = state
        .product_suggestions
        .iter()
        .filter(|s| s.company == company)
        .filter_map(|s| s.review.map(|r| r.hit))
        .collect();
    (!reviewed.is_empty())
        .then(|| reviewed.iter().filter(|&&h| h).count() as f64 / reviewed.len() as f64)
}

/// The AI takes the open suggestion with the best payback when a leader and a laboratory
/// are free and it pays off in time.
pub(crate) fn ai_accept(
    state: &GameState,
    catalog: &Catalog,
    company: CompanyId,
) -> Option<crate::command::Command> {
    let pm = catalog.research_model.projects.as_ref()?;
    let limit = pm
        .reports
        .as_ref()
        .map_or(f64::MAX, |r| r.payback_max_years);
    let (index, s) = state
        .product_suggestions
        .iter()
        .enumerate()
        .filter(|(_, s)| s.company == company && s.status == SuggestionStatus::Open)
        .filter(|(_, s)| {
            let k = &s.forecast;
            k.contribution_year > 0.0
                && (k.project_cost + k.investment) / k.contribution_year <= limit
        })
        .min_by(|a, b| {
            let pay = |k: &Kpis| (k.project_cost + k.investment) / k.contribution_year.max(1e-9);
            pay(&a.1.forecast).total_cmp(&pay(&b.1.forecast))
        })?;
    let lead = (0..state.project_leads.len()).find(|&l| {
        state.project_leads[l].employer == Some(company)
            && !state
                .dev_projects
                .iter()
                .any(|p| p.running() && p.lead as usize == l)
    })?;
    if let Some(buy) = crate::project_deals::ai_license(state, catalog, company, index, id32(lead))
    {
        return Some(buy);
    }
    let _ = s;
    Some(crate::command::Command::StartSuggestedProject {
        suggestion: id32(index),
        lead: id32(lead),
        tempo: 1,
        trim: 1,
    })
}
