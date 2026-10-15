//! PD8 – partial knowledge of aborted projects, platform components, licences of released
//! projects and development partnerships (docs/FORMELN.md).

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::command::{Command, CommandError};
use crate::ids::ProductId;
use crate::ledger::{Account, CostCenter, CostType};
use crate::money::Money;
use crate::project_suggestions::{Promise, SuggestionStatus};
use crate::projects::{self, DevProject, ProjectStatus};
use crate::rng::{SimRng, Stream};
use crate::state::{CompanyId, GameState, SiteId};

/// What a company keeps of an aborted project of a product (0–1 of the progress).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Knowledge {
    pub company: CompanyId,
    pub product: ProductId,
    pub share: f64,
}

/// The licence a project was started with.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectLicense {
    pub seller: CompanyId,
    /// Fees over all the fee years.
    pub fee_total: Money,
}

/// A running licence fee between two companies.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Royalty {
    pub payer: CompanyId,
    pub payee: CompanyId,
    pub product: ProductId,
    pub monthly: Money,
    pub until: Date,
}

fn project_id(index: usize) -> u32 {
    u32::try_from(index).unwrap_or(u32::MAX)
}

/// What the company keeps of earlier projects for the product.
pub fn knowledge_of(state: &GameState, company: CompanyId, product: ProductId) -> f64 {
    state
        .project_knowledge
        .iter()
        .find(|k| k.company == company && k.product == product)
        .map_or(0.0, |k| k.share)
}

/// An aborted project leaves partial knowledge; a higher value stays.
pub(crate) fn keep_knowledge(state: &mut GameState, catalog: &Catalog, index: usize) {
    let Some(m) = catalog.research_model.projects.as_ref() else {
        return;
    };
    let Some(k) = &m.knowledge else {
        return;
    };
    let p = &state.dev_projects[index];
    let share = (k.share * projects::overall_progress(m, p)).clamp(0.0, 1.0);
    if share <= 0.0 {
        return;
    }
    let (company, product) = (p.company, p.product);
    match state
        .project_knowledge
        .iter_mut()
        .find(|e| e.company == company && e.product == product)
    {
        Some(e) => e.share = e.share.max(share),
        None => {
            state.project_knowledge.push(Knowledge {
                company,
                product,
                share,
            });
            state
                .project_knowledge
                .sort_by_key(|e| (e.company, e.product));
        }
    }
}

/// The product is released: nothing is left of the partial knowledge.
pub(crate) fn drop_knowledge(state: &mut GameState, company: CompanyId, product: ProductId) {
    state
        .project_knowledge
        .retain(|e| !(e.company == company && e.product == product));
}

/// Share of the components of the product the company already knows.
fn known_inputs(
    state: &GameState,
    catalog: &Catalog,
    company: CompanyId,
    product: ProductId,
) -> f64 {
    let Some(r) = catalog
        .recipes
        .ids()
        .find(|&r| catalog.recipes.get(r).product == product)
    else {
        return 1.0;
    };
    let inputs = &catalog.recipes.get(r).inputs;
    if inputs.is_empty() {
        return 1.0;
    }
    let known = inputs
        .iter()
        .filter(|(i, _)| state.companies[company.index()].known_products.contains(i))
        .count();
    known as f64 / inputs.len() as f64
}

/// The head start of a new project: from partial knowledge and from known components.
pub(crate) fn start_bonus(
    state: &GameState,
    catalog: &Catalog,
    company: CompanyId,
    product: ProductId,
) -> (f64, f64) {
    let Some(m) = catalog.research_model.projects.as_ref() else {
        return (0.0, 0.0);
    };
    let head = m
        .knowledge
        .as_ref()
        .map_or(0.0, |k| k.effect * knowledge_of(state, company, product));
    let platform = m.platform.as_ref().map_or(0.0, |p| {
        p.effect * known_inputs(state, catalog, company, product)
    });
    (head, platform)
}

/// The price of the licence of a released project.
pub fn license_price(catalog: &Catalog, seller: &DevProject) -> Option<(Money, Money)> {
    let l = catalog
        .research_model
        .projects
        .as_ref()
        .and_then(|m| m.license.as_ref())?;
    let spent = seller.spent.to_usd();
    Some((
        Money::from_usd(spent * l.price_share)?,
        Money::from_usd(spent * l.fee_share)?,
    ))
}

/// `BuyProjectLicense`: buys the know-how of a released project of another company and
/// starts the own project in a later phase.
pub(crate) fn buy_license(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    seller_project: u32,
    (site, lead): (SiteId, u32),
    (tempo, trim): (u8, u8),
) -> Result<(), CommandError> {
    let Some(m) = catalog.research_model.projects.as_ref() else {
        return Err(CommandError::ProjectsOff);
    };
    let Some(l) = &m.license else {
        return Err(CommandError::ProjectsOff);
    };
    let Some(sp) = state.dev_projects.get(seller_project as usize) else {
        return Err(CommandError::LicenseUnavailable);
    };
    let (seller, product) = (sp.company, sp.product);
    if sp.status != ProjectStatus::Released
        || seller == actor
        || state.companies[seller.index()].bankrupt
        || state.companies[actor.index()]
            .known_products
            .contains(&product)
    {
        return Err(CommandError::LicenseUnavailable);
    }
    let Some((price, fee_total)) = license_price(catalog, sp) else {
        return Err(CommandError::LicenseUnavailable);
    };
    if state.companies[actor.index()].ledger.cash() < price {
        return Err(CommandError::NotEnoughCash { needed: price });
    }
    projects::start(state, catalog, actor, (site, product, lead), (tempo, trim))?;
    let index = state.dev_projects.len() - 1;
    let p = &mut state.dev_projects[index];
    p.phase = l.from_phase;
    p.license = Some(ProjectLicense { seller, fee_total });
    state.companies[actor.index()].ledger.expense(
        CostType::Licenses,
        CostCenter::default(),
        Account::Cash,
        price,
    );
    state.companies[seller.index()].ledger.income(
        CostType::Licenses,
        CostCenter::default(),
        Account::Cash,
        price,
    );
    // An open suggestion for the product is started with the licence.
    if let Some(i) = state.product_suggestions.iter().position(|s| {
        s.company == actor && s.product == product && s.status == SuggestionStatus::Open
    }) {
        let forecast = state.product_suggestions[i].forecast;
        state.dev_projects[index].promise = Some(Promise {
            suggestion: project_id(i),
            forecast,
        });
        state.product_suggestions[i].status = SuggestionStatus::Started(project_id(index));
    }
    Ok(())
}

/// The licensed project is released: the fees run from now on.
pub(crate) fn start_royalty(state: &mut GameState, catalog: &Catalog, index: usize, date: Date) {
    let Some(l) = catalog
        .research_model
        .projects
        .as_ref()
        .and_then(|m| m.license.as_ref())
    else {
        return;
    };
    let p = &state.dev_projects[index];
    let Some(license) = p.license else {
        return;
    };
    let months = f64::from(l.fee_years) * 12.0;
    let Some(monthly) = Money::from_usd(license.fee_total.to_usd() / months) else {
        return;
    };
    let until = date.add_days(i32::try_from(l.fee_years * 365).unwrap_or(i32::MAX));
    state.project_royalties.push(Royalty {
        payer: p.company,
        payee: license.seller,
        product: p.product,
        monthly,
        until,
    });
}

/// Month start: the licence fees are paid; the expired ones leave.
pub(crate) fn pay_royalties(state: &mut GameState, date: Date) {
    let due: Vec<Royalty> = state
        .project_royalties
        .iter()
        .filter(|r| date <= r.until)
        .cloned()
        .collect();
    for r in due {
        if state.companies[r.payer.index()].bankrupt {
            continue;
        }
        state.companies[r.payer.index()].ledger.expense(
            CostType::Licenses,
            CostCenter::default(),
            Account::Cash,
            r.monthly,
        );
        state.companies[r.payee.index()].ledger.income(
            CostType::Licenses,
            CostCenter::default(),
            Account::Cash,
            r.monthly,
        );
    }
    state.project_royalties.retain(|r| date <= r.until);
}

/// `ProposePartnership`: an AI company shares the costs and gets the product too.
pub(crate) fn propose(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    project: u32,
    partner: CompanyId,
) -> Result<(), CommandError> {
    let Some(m) = catalog.research_model.projects.as_ref() else {
        return Err(CommandError::ProjectsOff);
    };
    let Some(pm) = &m.partnership else {
        return Err(CommandError::ProjectsOff);
    };
    let index = project as usize;
    let Some(p) = state.dev_projects.get(index) else {
        return Err(CommandError::UnknownProject);
    };
    if p.company != actor || !p.running() || p.phase > 2 || p.partner.is_some() {
        return Err(CommandError::UnknownProject);
    }
    let Some(c) = state.companies.get(partner.index()) else {
        return Err(CommandError::PartnerUnavailable);
    };
    if partner == actor || c.bankrupt || c.ai.is_none() || c.known_products.contains(&p.product) {
        return Err(CommandError::PartnerUnavailable);
    }
    let level = projects::price_level(state, p.site);
    let share_cost = crate::project_reports::open_cost(m, p) * pm.cost_share * level;
    if c.ledger.cash().to_usd() < share_cost {
        return Err(CommandError::PartnerUnavailable);
    }
    let mut rng = SimRng::for_stream(
        state.settings.seed,
        Stream::ProjectPartner {
            project: project_id(index),
        },
    );
    if !rng.chance(pm.ai_accept) {
        return Err(CommandError::PartnershipDeclined);
    }
    state.dev_projects[index].partner = Some((partner, pm.cost_share));
    Ok(())
}

/// The command an AI company gives to buy a licence instead of developing a product, if
/// that is cheaper than the forecast development cost of its suggestion.
pub(crate) fn ai_license(
    state: &GameState,
    catalog: &Catalog,
    company: CompanyId,
    suggestion: usize,
    lead: u32,
) -> Option<Command> {
    let s = state.product_suggestions.get(suggestion)?;
    let (i, sp) = state.dev_projects.iter().enumerate().find(|(_, p)| {
        p.product == s.product && p.status == ProjectStatus::Released && p.company != company
    })?;
    let (price, fees) = license_price(catalog, sp)?;
    let total = (price.to_usd() + fees.to_usd()) / projects::price_level(state, s.site).max(1e-9);
    (total < s.forecast.project_cost).then_some(Command::BuyProjectLicense {
        seller_project: project_id(i),
        site: s.site,
        lead,
        tempo: 1,
        trim: 1,
    })
}
