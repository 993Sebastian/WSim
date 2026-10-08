//! Patents and licences (Lastenheft §7.2; formulas in docs/FORMELN.md, P7).
//!
//! The first company to research a technology may patent it country by country within
//! a filing period. Where it is patented, other companies may build its facilities and
//! run its recipes only with a licence of the holder; research, trade and imports stay
//! free.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::{Catalog, PatentModel};
use crate::command::CommandError;
use crate::ids::{CountryId, Id, RecipeId, TechnologyId};
use crate::ledger::{Account, CostCenter, CostType};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::state::{CompanyId, GameState};

/// A patent claim of the first inventor, filed in some countries or not yet.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Patent {
    pub holder: CompanyId,
    /// Day of the invention; filing is open for the filing period from here.
    pub invented: Date,
    /// First filing: the term runs from here. `None`: a claim not filed yet.
    #[serde(default)]
    pub filed: Option<Date>,
    #[serde(default)]
    pub countries: BTreeSet<CountryId>,
    /// Companies besides the holder that may use it: licensees and prior users.
    #[serde(default)]
    pub licensees: BTreeSet<CompanyId>,
}

impl Patent {
    /// Last day to file further countries.
    pub fn deadline(&self, model: &PatentModel) -> Date {
        self.invented.add_days(model.filing_days)
    }

    /// Day the patent ends, once filed.
    pub fn until(&self, model: &PatentModel) -> Option<Date> {
        let months = u32::try_from(model.term_years.max(0) * 12).unwrap_or(0);
        self.filed.map(|d| d.add_months(months))
    }
}

fn model(catalog: &Catalog) -> Option<&PatentModel> {
    catalog.research_model.patents.as_ref()
}

/// The patent on a technology that is in force today.
pub fn in_force<'a>(
    state: &'a GameState,
    catalog: &Catalog,
    technology: TechnologyId,
) -> Option<&'a Patent> {
    let model = model(catalog)?;
    let p = state.patents.get(technology).as_ref()?;
    let until = p.until(model)?;
    let holder_alive = state.company(p.holder).is_some_and(|c| !c.bankrupt);
    (state.date < until && !p.countries.is_empty() && holder_alive).then_some(p)
}

/// Whether a patent keeps a company from using a technology in a country.
pub fn blocks(
    state: &GameState,
    catalog: &Catalog,
    company: CompanyId,
    technology: TechnologyId,
    country: CountryId,
) -> bool {
    in_force(state, catalog, technology).is_some_and(|p| {
        p.holder != company && !p.licensees.contains(&company) && p.countries.contains(&country)
    })
}

/// Whether a company may use a technology in a country: it knows it and no patent of
/// another company stands in the way.
pub fn may_use(
    state: &GameState,
    catalog: &Catalog,
    company: CompanyId,
    technology: TechnologyId,
    country: CountryId,
) -> bool {
    state.knows(catalog, company, technology)
        && !blocks(state, catalog, company, technology, country)
}

/// The technology of a recipe or of its facility that a patent blocks for the company
/// in the country, if any.
pub fn blocked_recipe(
    state: &GameState,
    catalog: &Catalog,
    company: CompanyId,
    recipe: RecipeId,
    country: CountryId,
) -> Option<TechnologyId> {
    let r = catalog.recipes.get(recipe);
    let facility = catalog.facilities.get(r.facility).technology;
    [r.technology, facility]
        .into_iter()
        .flatten()
        .find(|&t| blocks(state, catalog, company, t, country))
}

/// What filing in a country costs.
pub fn filing_cost(state: &GameState, catalog: &Catalog, country: CountryId) -> Money {
    model(catalog).map_or(Money::ZERO, |m| {
        Money::from_usd(m.cost_per_country_usd * state.countries.get(country).price_level)
            .unwrap_or(Money::ZERO)
    })
}

/// Whether a company may still file countries for a technology.
pub fn can_file(
    state: &GameState,
    catalog: &Catalog,
    company: CompanyId,
    technology: TechnologyId,
) -> bool {
    let Some(model) = model(catalog) else {
        return false;
    };
    state
        .patents
        .get(technology)
        .as_ref()
        .is_some_and(|p| p.holder == company && state.date <= p.deadline(model))
}

/// The first invention of a technology by a company: a claim to a patent, and a note to
/// the player when it is theirs.
pub(crate) fn claim(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    technology: TechnologyId,
    date: Date,
) -> Option<Message> {
    let model = model(catalog)?;
    let patent = Patent {
        holder: company,
        invented: date,
        filed: None,
        countries: BTreeSet::new(),
        licensees: BTreeSet::new(),
    };
    let deadline = patent.deadline(model);
    *state.patents.get_mut(technology) = Some(patent);
    state.is_main(company).then(|| {
        Message::new(MessageKind::Info, keys::PATENT_CLAIM)
            .with("technologie", technology_param(catalog, technology))
            .with("bis", Param::Date(deadline))
    })
}

fn technology_param(catalog: &Catalog, technology: TechnologyId) -> Param {
    Param::TextKey(format!(
        "technologie.{}",
        catalog.technologies.key(technology)
    ))
}

/// `FilePatent`: files the claim in further countries and pays for them. Companies
/// that know the technology already keep using it (prior use).
pub(crate) fn file(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    technology: TechnologyId,
    countries: &[CountryId],
) -> Result<(), CommandError> {
    if technology.index() >= catalog.technologies.len()
        || !can_file(state, catalog, actor, technology)
    {
        return Err(CommandError::NoPatentClaim);
    }
    if countries
        .iter()
        .any(|c| c.index() >= catalog.countries.len())
    {
        return Err(CommandError::UnknownCountry);
    }
    let covered = &state
        .patents
        .get(technology)
        .as_ref()
        .expect("checked above")
        .countries;
    let new: BTreeSet<CountryId> = countries
        .iter()
        .copied()
        .filter(|c| !covered.contains(c))
        .collect();
    if new.is_empty() {
        return Err(CommandError::NoPatentClaim);
    }
    let cost = new
        .iter()
        .map(|&c| filing_cost(state, catalog, c))
        .fold(Money::ZERO, |a, b| a + b);
    let ledger = &mut state.companies[actor.index()].ledger;
    if ledger.cash() < cost {
        return Err(CommandError::NotEnoughCash { needed: cost });
    }
    ledger.expense(
        CostType::Licenses,
        CostCenter::default(),
        Account::Cash,
        cost,
    );
    let users: Vec<CompanyId> = (0..state.companies.len())
        .map(company_id)
        .filter(|&c| c != actor && state.knows(catalog, c, technology))
        .collect();
    let date = state.date;
    let p = state
        .patents
        .get_mut(technology)
        .as_mut()
        .expect("checked above");
    p.filed.get_or_insert(date);
    p.countries.extend(new);
    p.licensees.extend(users);
    Ok(())
}

fn company_id(index: usize) -> CompanyId {
    CompanyId(u32::try_from(index).expect("company count fits u32"))
}

/// A licence sold by the holder of a patent: the buyer may use it everywhere.
pub(crate) fn licensed(
    state: &mut GameState,
    technology: TechnologyId,
    buyer: CompanyId,
    seller: CompanyId,
) {
    if let Some(p) = state.patents.get_mut(technology).as_mut()
        && p.holder == seller
    {
        p.licensees.insert(buyer);
    }
}

/// Whether `seller` may license a technology to `buyer`: under a patent only the holder,
/// to anyone it still blocks; otherwise anyone who knows it to anyone who does not.
pub fn may_license(
    state: &GameState,
    catalog: &Catalog,
    seller: CompanyId,
    buyer: CompanyId,
    technology: TechnologyId,
) -> bool {
    match in_force(state, catalog, technology) {
        Some(p) => p.holder == seller && buyer != seller && !p.licensees.contains(&buyer),
        None => {
            state.knows(catalog, seller, technology) && !state.knows(catalog, buyer, technology)
        }
    }
}

/// Whether a company knows a technology but a patent keeps it from using it somewhere.
pub fn needs_license(
    state: &GameState,
    catalog: &Catalog,
    company: CompanyId,
    technology: TechnologyId,
) -> bool {
    state.knows(catalog, company, technology)
        && in_force(state, catalog, technology)
            .is_some_and(|p| p.holder != company && !p.licensees.contains(&company))
}

/// The countries an AI company files in: its site countries and the largest markets
/// (GDP), each once, in country order.
pub fn ai_countries(
    state: &GameState,
    catalog: &Catalog,
    company: CompanyId,
) -> (Vec<CountryId>, Vec<CountryId>) {
    let own: BTreeSet<CountryId> = state
        .sites
        .iter()
        .filter(|s| s.owner == company)
        .map(|s| s.country)
        .collect();
    let largest = model(catalog).map_or(0, |m| m.ai_largest_markets);
    let mut by_gdp: Vec<(f64, CountryId)> = catalog
        .countries
        .ids()
        .map(|c| {
            let v = state.countries.get(c);
            (v.population * v.gdp_per_capita_usd, c)
        })
        .collect();
    by_gdp.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut all = own.clone();
    all.extend(by_gdp.into_iter().take(largest).map(|(_, c)| c));
    (own.into_iter().collect(), all.into_iter().collect())
}

/// Start of a month: patents that ended or whose holder went bankrupt, and claims not
/// filed in time, lapse; the player hears of their own.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog) -> Vec<Message> {
    let Some(model) = model(catalog) else {
        return Vec::new();
    };
    let date = state.date;
    let mut news = Vec::new();
    for t in catalog.technologies.ids() {
        let Some(p) = state.patents.get(t).as_ref() else {
            continue;
        };
        let bankrupt = state.company(p.holder).is_none_or(|c| c.bankrupt);
        let (ended, key) = match p.until(model) {
            Some(until) => (until <= date || bankrupt, keys::PATENT_ENDED),
            None => (
                date > p.deadline(model) || bankrupt,
                keys::PATENT_CLAIM_LAPSED,
            ),
        };
        if !ended {
            continue;
        }
        if state.is_main(p.holder) {
            news.push(
                Message::new(MessageKind::Info, key)
                    .with("technologie", technology_param(catalog, t)),
            );
        }
        *state.patents.get_mut(t) = None;
    }
    // The player's facilities a patent stops, once a month per technology and country.
    if let Some(main) = state.main_company {
        let mut seen: BTreeSet<(TechnologyId, CountryId)> = BTreeSet::new();
        for s in state.sites.iter().filter(|s| s.owner == main) {
            for sl in &s.slots {
                let Some(recipe) = sl.recipe else {
                    continue;
                };
                if let Some(t) = blocked_recipe(state, catalog, main, recipe, s.country)
                    && seen.insert((t, s.country))
                {
                    let holder = in_force(state, catalog, t)
                        .and_then(|p| state.company(p.holder))
                        .map_or_else(String::new, |c| c.name.clone());
                    news.push(
                        Message::new(MessageKind::Warning, keys::PATENT_BLOCKED)
                            .with("technologie", technology_param(catalog, t))
                            .with(
                                "land",
                                Param::Country(catalog.countries.key(s.country).to_owned()),
                            )
                            .with("firma", Param::Text(holder)),
                    );
                }
            }
        }
    }
    news
}
