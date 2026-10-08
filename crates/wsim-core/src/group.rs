//! Subsidiaries and the group (W6, Lastenheft §5.1; formulas in docs/FORMELN.md, W6).
//!
//! A subsidiary is a company of its own with its own ledger, taxes and management: it
//! acts by the AI rules with the character of the data, never against its group. Its
//! parent founds it, puts capital in or takes it out, and moves sites to and from it at
//! book values. The group view adds the ledgers up and offsets the parents' stakes
//! against the subsidiaries' subscribed capital.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::command::{CommandError, check_company_name};
use crate::ids::{CountryId, Id};
use crate::ledger::{Account, CostCenter, CostType, Ledger};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{AiState, Company, CompanyId, CompanyKind, GameState, Holder, SiteId, Stake};

/// What a subsidiary is for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubsidiaryFocus {
    /// The AI rules for its sites.
    #[default]
    Production,
    /// Also a fleet that carries the freight of others (W5).
    Logistics,
}

/// The parent of a subsidiary and its focus.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SubsidiaryOf {
    pub parent: CompanyId,
    #[serde(default)]
    pub focus: SubsidiaryFocus,
}

/// The top company of a company's group.
pub fn top(state: &GameState, company: CompanyId) -> CompanyId {
    let mut c = company;
    // A chain is never longer than the list of companies; the bound guards against loops.
    for _ in 0..state.companies.len() {
        match state.companies[c.index()].subsidiary_of {
            Some(s) => c = s.parent,
            None => break,
        }
    }
    c
}

/// Whether two companies belong to the same group.
pub fn same_group(state: &GameState, a: CompanyId, b: CompanyId) -> bool {
    a == b || top(state, a) == top(state, b)
}

/// The companies of a group: the top first, then its subsidiaries by number.
pub fn members(state: &GameState, top_company: CompanyId) -> Vec<CompanyId> {
    let mut out = vec![top_company];
    out.extend(
        (0..state.companies.len())
            .map(company_id)
            .filter(|&c| c != top_company && state.companies[c.index()].subsidiary_of.is_some())
            .filter(|&c| top(state, c) == top_company),
    );
    out
}

/// Whether `company` is a subsidiary (of any depth) of a group's top company other than
/// itself.
pub fn is_subsidiary(state: &GameState, company: CompanyId) -> bool {
    state.companies[company.index()].subsidiary_of.is_some()
}

fn company_id(index: usize) -> CompanyId {
    CompanyId(u32::try_from(index).expect("company count fits u32"))
}

/// A direct subsidiary of `parent`, or an error.
fn own_subsidiary(
    state: &GameState,
    parent: CompanyId,
    company: CompanyId,
) -> Result<&Company, CommandError> {
    let c = state
        .companies
        .get(company.index())
        .ok_or(CommandError::UnknownCompany(company))?;
    if c.subsidiary_of.map(|s| s.parent) != Some(parent) || c.bankrupt {
        return Err(CommandError::NotOwnSubsidiary);
    }
    Ok(c)
}

/// Founds a subsidiary of `actor` with its capital (docs/FORMELN.md, W6).
pub(crate) fn found(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    (name, country): (&str, CountryId),
    capital: Money,
    focus: SubsidiaryFocus,
) -> Result<CompanyId, CommandError> {
    let m = &catalog.subsidiaries;
    if !m.enabled {
        return Err(CommandError::NoSubsidiaries);
    }
    if country.index() >= catalog.countries.len() {
        return Err(CommandError::UnknownCountry);
    }
    let name = check_company_name(Some(state), name, None).map_err(CommandError::Name)?;
    if capital < m.min_capital {
        return Err(CommandError::CapitalTooLow { min: m.min_capital });
    }
    let needed = capital + m.founding_cost;
    let date = state.date;
    let parent = &mut state.companies[actor.index()];
    if parent.ledger.cash() < needed {
        return Err(CommandError::NotEnoughCash { needed });
    }
    parent
        .ledger
        .transfer(Account::Participations, Account::Cash, capital);
    if m.founding_cost > Money::ZERO {
        parent.ledger.expense(
            CostType::Other,
            CostCenter::default(),
            Account::Cash,
            m.founding_cost,
        );
    }
    let index = u32::try_from(state.companies.len()).unwrap_or(u32::MAX);
    let id = CompanyId(index);
    state.companies.push(Company {
        brands: Vec::new(),
        advertising: Vec::new(),
        auction_until: None,
        development: Default::default(),
        product_names: Default::default(),
        positions: Vec::new(),
        budget_rules: Vec::new(),
        strategies: Vec::new(),
        mandate: crate::mandate::Mandate::default(),
        reviews: Vec::new(),
        relocation: None,
        relocated: None,
        departments: Default::default(),
        departments_staffed: Default::default(),
        hq_city: None,
        participations: Default::default(),
        logistics: Default::default(),
        subsidiary_of: Some(SubsidiaryOf {
            parent: actor,
            focus,
        }),
        listing: None,
        dividend_payout: None,
        stock_cost: Default::default(),
        bonds: Vec::new(),
        owners: Stake::sole(Holder::Company(actor)),
        name,
        kind: CompanyKind::Ai,
        headquarters: country,
        founded: date,
        rng: SimRng::for_stream(state.settings.seed, Stream::Company(index)),
        ledger: Ledger::new(date, capital),
        technologies: Default::default(),
        bankrupt: false,
        loans: Vec::new(),
        loss_carryforward: Money::ZERO,
        sales_policies: Vec::new(),
        research: Default::default(),
        ai: Some(AiState {
            competence: m.competence,
            aggressiveness: m.aggressiveness,
            real: None,
            next_operations: date,
            staff: 0.0,
        }),
    });
    // The subsidiary knows what its parent knows.
    let known = state.companies[actor.index()].technologies.clone();
    state.companies[id.index()].technologies = known;
    Ok(id)
}

/// Capital into a subsidiary (`amount` > 0) or back to the parent (< 0): first from the
/// subsidiary's profit, then as a return of capital.
pub(crate) fn move_capital(
    state: &mut GameState,
    actor: CompanyId,
    company: CompanyId,
    amount: Money,
) -> Result<(), CommandError> {
    let sub = own_subsidiary(state, actor, company)?;
    if amount == Money::ZERO {
        return Err(CommandError::InvalidQuantity);
    }
    if amount > Money::ZERO {
        let parent = &mut state.companies[actor.index()].ledger;
        if parent.cash() < amount {
            return Err(CommandError::NotEnoughCash { needed: amount });
        }
        parent.transfer(Account::Participations, Account::Cash, amount);
        state.companies[company.index()]
            .ledger
            .transfer(Account::Cash, Account::Equity, amount);
        return Ok(());
    }
    let wanted = Money::ZERO - amount;
    let l = &sub.ledger;
    let profit =
        (l.balance(Account::RetainedEarnings) + l.balance(Account::Result)).max(Money::ZERO);
    let stake = state.companies[actor.index()]
        .ledger
        .balance(Account::Participations);
    let capital = l.balance(Account::Equity).min(stake).max(Money::ZERO);
    let max = (profit + capital).min(l.cash().max(Money::ZERO));
    if wanted > max {
        return Err(CommandError::AmountTooHigh { max });
    }
    let dividend = wanted.min(profit);
    let back = wanted - dividend;
    let l = &mut state.companies[company.index()].ledger;
    l.transfer(Account::RetainedEarnings, Account::Cash, dividend);
    l.transfer(Account::Equity, Account::Cash, back);
    // Dividends within the group are no income of the parent: they go to its reserves.
    let p = &mut state.companies[actor.index()].ledger;
    p.transfer(Account::Cash, Account::RetainedEarnings, dividend);
    p.transfer(Account::Cash, Account::Participations, back);
    Ok(())
}

/// Moves a site between a company and one of its direct subsidiaries at book values.
pub(crate) fn transfer_site(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    site: SiteId,
    to: CompanyId,
) -> Result<(), CommandError> {
    let s = state.site(site).ok_or(CommandError::UnknownSite)?;
    let from = s.owner;
    let allowed = (from == actor && own_subsidiary(state, actor, to).is_ok())
        || (to == actor && own_subsidiary(state, actor, from).is_ok());
    if !allowed {
        return Err(CommandError::NotOwnSubsidiary);
    }
    let v = crate::deals::site_value(state, catalog, site);
    let book = v.fixed_assets + v.under_construction + v.goodwill + v.inventory + v.land_book;
    if state.companies[to.index()].ledger.cash() < book {
        return Err(CommandError::NotEnoughCash { needed: book });
    }
    let parts = [
        (Account::FixedAssets, v.fixed_assets),
        (Account::AssetsUnderConstruction, v.under_construction),
        (Account::Goodwill, v.goodwill),
        (Account::Inventory, v.inventory),
        (Account::Land, v.land_book),
    ];
    let seller = &mut state.companies[from.index()].ledger;
    for (account, amount) in parts {
        seller.transfer(Account::Cash, account, amount);
    }
    let buyer = &mut state.companies[to.index()].ledger;
    for (account, amount) in parts {
        buyer.transfer(account, Account::Cash, amount);
    }
    state.sites[site.index()].owner = to;
    crate::management::release_site(state, site);
    state.sites[site.index()].staffing_due = true;
    Ok(())
}

/// Changes the focus of a direct subsidiary.
pub(crate) fn set_focus(
    state: &mut GameState,
    actor: CompanyId,
    company: CompanyId,
    focus: SubsidiaryFocus,
) -> Result<(), CommandError> {
    own_subsidiary(state, actor, company)?;
    if let Some(s) = state.companies[company.index()].subsidiary_of.as_mut() {
        s.focus = focus;
    }
    Ok(())
}

/// After the insolvency check: a parent writes its stake in a bankrupt subsidiary off,
/// and the subsidiary leaves the group. Returns the messages for the player.
pub(crate) fn settle_failures(state: &mut GameState) -> Vec<Message> {
    let mut news = Vec::new();
    for i in 0..state.companies.len() {
        let c = &state.companies[i];
        let (true, Some(of)) = (c.bankrupt, c.subsidiary_of) else {
            continue;
        };
        let paid_in = c.ledger.balance(Account::Equity).max(Money::ZERO);
        let name = c.name.clone();
        let parent = &mut state.companies[of.parent.index()].ledger;
        let loss = paid_in.min(parent.balance(Account::Participations).max(Money::ZERO));
        if loss > Money::ZERO {
            parent.expense(
                CostType::Investments,
                CostCenter::default(),
                Account::Participations,
                loss,
            );
        }
        let c = &mut state.companies[i];
        c.subsidiary_of = None;
        c.owners = Stake::sole(Holder::Private);
        if top(state, of.parent) == state.player {
            news.push(
                Message::new(MessageKind::Warning, keys::SUBSIDIARY_FAILED)
                    .with("firma", Param::Text(name))
                    .with("verlust", Param::Money(loss)),
            );
        }
    }
    news
}

/// A group's ledgers added up, the stakes in its own companies offset (docs/FORMELN.md,
/// W6).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Consolidated {
    /// Balance per account in natural sign.
    pub balances: BTreeMap<Account, Money>,
    /// The running year per cost type.
    pub year: BTreeMap<CostType, Money>,
}

impl Consolidated {
    pub fn balance(&self, account: Account) -> Money {
        self.balances.get(&account).copied().unwrap_or_default()
    }

    pub fn equity(&self) -> Money {
        [Account::Equity, Account::RetainedEarnings, Account::Result]
            .iter()
            .map(|&a| self.balance(a))
            .sum()
    }

    pub fn total_assets(&self) -> Money {
        Account::ALL
            .iter()
            .filter(|a| a.is_asset())
            .map(|&a| self.balance(a))
            .sum()
    }
}

/// The group of `top_company` consolidated.
pub fn consolidated(state: &GameState, top_company: CompanyId) -> Consolidated {
    let mut out = Consolidated::default();
    for c in members(state, top_company) {
        let company = &state.companies[c.index()];
        for &a in &Account::ALL {
            *out.balances.entry(a).or_default() += company.ledger.balance(a);
        }
        for (&t, &amount) in &company.ledger.year.by_type {
            *out.year.entry(t).or_default() += amount;
        }
        if company.subsidiary_of.is_some() {
            // The parent's stake against the subsidiary's subscribed capital.
            let paid_in = company.ledger.balance(Account::Equity);
            *out.balances.entry(Account::Participations).or_default() -= paid_in;
            *out.balances.entry(Account::Equity).or_default() -= paid_in;
        }
    }
    out
}
