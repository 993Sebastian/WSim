//! Commands: the only way to change a company (Lastenheft §10).
//!
//! The player, AI companies and later managers all act through the same commands and
//! the same checks.

use serde::{Deserialize, Serialize};

use crate::catalog::{Catalog, SiteType};
use crate::ids::{CountryId, DepositId, FacilityId, Id, ProductId, RecipeId, TechnologyId};
use crate::ledger::{Account, CostCenter, CostType, Ledger};
use crate::message::{Message, Param, keys};
use crate::money::Money;
use crate::policy::{self, BuyerGroup, SalesRule, Scope};
use crate::state::{
    CompanyId, Consignee, GameState, PerId, PriceMode, PurchaseOrder, SaleOffer, Shipment, Site,
    SiteId, Slot,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Command {
    /// Renames the acting company.
    RenameCompany { name: String },
    /// Founds a site (land and buildings) in a country.
    FoundSite { country: CountryId, kind: SiteType },
    /// Builds `count` identical units of a facility at a site, working together in one
    /// slot; they produce after the construction time.
    BuildFacility {
        site: SiteId,
        facility: FacilityId,
        #[serde(default = "one_unit")]
        count: u32,
    },
    /// Develops a deposit for an extraction site in the same country.
    DevelopDeposit { site: SiteId, deposit: DepositId },
    /// Sets what a facility produces and how much of its capacity is planned.
    SetProduction {
        site: SiteId,
        slot: usize,
        recipe: Option<RecipeId>,
        utilization: f64,
    },
    /// Sets the degree of automation of a facility; raising it costs money.
    SetAutomation {
        site: SiteId,
        slot: usize,
        level: f64,
    },
    /// Moves goods between two own sites: at once within a country, otherwise by
    /// freight with transport costs and travel time.
    TransferGoods {
        from: SiteId,
        to: SiteId,
        product: ProductId,
        quantity: f64,
    },
    /// Takes up a bank loan, repaid monthly over `years`.
    TakeLoan { amount: Money, years: u32 },
    /// Repays (part of) a loan early.
    RepayLoan { loan: usize, amount: Money },
    /// Offers goods of a site on the country's market (`None` withdraws the offer).
    SetSale {
        site: SiteId,
        product: ProductId,
        mode: Option<PriceMode>,
        keep: f64,
    },
    /// Keeps a stock at a site by buying on the country's market (`target` 0 stops).
    SetPurchase {
        site: SiteId,
        product: ProductId,
        target: f64,
        max_price: Money,
        min_quality: f64,
    },
    /// Lets a research center work on a technology (`None` stops it; the points
    /// collected so far stay with the company).
    SetResearch {
        site: SiteId,
        technology: Option<TechnologyId>,
    },
    /// Sets who besides consumers and governments may buy the company's goods
    /// (`None` removes the policy of this scope, the more general one applies).
    SetSalesPolicy {
        buyer: BuyerGroup,
        scope: Scope,
        rule: Option<SalesRule>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NameError {
    Empty,
    TooLong { max: usize },
    Taken { name: String },
}

/// Longest allowed company name in characters.
pub const MAX_NAME_LENGTH: usize = 60;

impl NameError {
    pub fn message(&self) -> Message {
        match self {
            NameError::Empty => Message::error(keys::NAME_EMPTY),
            NameError::TooLong { max } => {
                Message::error(keys::NAME_TOO_LONG).with("max", integer(*max))
            }
            NameError::Taken { name } => {
                Message::error(keys::NAME_TAKEN).with("name", Param::Text(name.clone()))
            }
        }
    }
}

fn one_unit() -> u32 {
    1
}

fn integer(n: usize) -> Param {
    Param::Integer(i64::try_from(n).unwrap_or(i64::MAX))
}

#[derive(Clone, Debug, PartialEq)]
pub enum CommandError {
    GameOver,
    UnknownCompany(CompanyId),
    CompanyBankrupt,
    Name(NameError),
    UnknownSite,
    NotOwner,
    UnknownSlot,
    WrongSiteType {
        required: SiteType,
    },
    /// Key of the missing technology.
    TechnologyUnknown(String),
    NotEnoughCash {
        needed: Money,
    },
    DepositUnavailable,
    DepositOtherCountry,
    DepositNotDiscovered {
        year: i32,
    },
    SiteHasDeposit,
    RecipeNotForFacility,
    RecipeNeedsDeposit,
    InvalidShare,
    AutomationTooHigh {
        max: f64,
    },
    InvalidQuantity,
    NotEnoughGoods {
        available: f64,
    },
    DifferentCountries,
    InvalidAmount,
    LoanTooLarge {
        limit: Money,
    },
    InvalidTerm {
        max: u32,
    },
    UnknownLoan,
    InvalidPrice,
    /// Already known, or prerequisites missing (key of the technology).
    NotResearchable(String),
    /// No transport route for the product between the two countries.
    NoRoute {
        product: String,
        from: String,
        to: String,
    },
}

impl CommandError {
    pub fn message(&self) -> Message {
        let e = Message::error;
        match self {
            CommandError::GameOver => e(keys::COMMAND_GAME_OVER),
            CommandError::UnknownCompany(id) => {
                e(keys::COMMAND_UNKNOWN_COMPANY).with("firma", Param::Integer(i64::from(id.0)))
            }
            CommandError::CompanyBankrupt => e(keys::COMMAND_BANKRUPT),
            CommandError::Name(n) => n.message(),
            CommandError::UnknownSite => e(keys::COMMAND_UNKNOWN_SITE),
            CommandError::NotOwner => e(keys::COMMAND_NOT_OWNER),
            CommandError::UnknownSlot => e(keys::COMMAND_UNKNOWN_SLOT),
            CommandError::WrongSiteType { required } => e(keys::COMMAND_WRONG_SITE_TYPE)
                .with("typ", Param::TextKey(site_type_key(*required))),
            CommandError::TechnologyUnknown(t) => e(keys::COMMAND_TECHNOLOGY_UNKNOWN)
                .with("technologie", Param::TextKey(format!("technologie.{t}"))),
            CommandError::NotEnoughCash { needed } => {
                e(keys::COMMAND_NOT_ENOUGH_CASH).with("betrag", Param::Money(*needed))
            }
            CommandError::DepositUnavailable => e(keys::COMMAND_DEPOSIT_UNAVAILABLE),
            CommandError::DepositOtherCountry => e(keys::COMMAND_DEPOSIT_OTHER_COUNTRY),
            CommandError::DepositNotDiscovered { year } => e(keys::COMMAND_DEPOSIT_NOT_DISCOVERED)
                .with("jahr", Param::Integer(i64::from(*year))),
            CommandError::SiteHasDeposit => e(keys::COMMAND_SITE_HAS_DEPOSIT),
            CommandError::RecipeNotForFacility => e(keys::COMMAND_RECIPE_NOT_FOR_FACILITY),
            CommandError::RecipeNeedsDeposit => e(keys::COMMAND_RECIPE_NEEDS_DEPOSIT),
            CommandError::InvalidShare => e(keys::COMMAND_INVALID_SHARE),
            CommandError::AutomationTooHigh { max } => {
                e(keys::COMMAND_AUTOMATION_TOO_HIGH).with("max", Param::Number(*max * 100.0))
            }
            CommandError::InvalidQuantity => e(keys::COMMAND_INVALID_QUANTITY),
            CommandError::NotEnoughGoods { available } => {
                e(keys::COMMAND_NOT_ENOUGH_GOODS).with("menge", Param::Number(*available))
            }
            CommandError::DifferentCountries => e(keys::COMMAND_DIFFERENT_COUNTRIES),
            CommandError::InvalidAmount => e(keys::COMMAND_INVALID_AMOUNT),
            CommandError::LoanTooLarge { limit } => {
                e(keys::COMMAND_LOAN_TOO_LARGE).with("limit", Param::Money(*limit))
            }
            CommandError::InvalidTerm { max } => {
                e(keys::COMMAND_INVALID_TERM).with("max", Param::Integer(i64::from(*max)))
            }
            CommandError::UnknownLoan => e(keys::COMMAND_UNKNOWN_LOAN),
            CommandError::InvalidPrice => e(keys::COMMAND_INVALID_PRICE),
            CommandError::NotResearchable(t) => e(keys::COMMAND_NOT_RESEARCHABLE)
                .with("technologie", Param::TextKey(format!("technologie.{t}"))),
            CommandError::NoRoute { product, from, to } => e(keys::COMMAND_NO_ROUTE)
                .with("produkt", Param::TextKey(format!("produkt.{product}")))
                .with("von", Param::TextKey(format!("land.{from}")))
                .with("nach", Param::TextKey(format!("land.{to}"))),
        }
    }
}

/// Text key of a site type, e.g. `standorttyp.werk`.
pub fn site_type_key(kind: SiteType) -> String {
    let name = match kind {
        SiteType::Extraction => "foerderstaette",
        SiteType::Factory => "werk",
        SiteType::PowerPlant => "kraftwerk",
        SiteType::Warehouse => "lager",
        SiteType::SalesOffice => "niederlassung",
        SiteType::ResearchCenter => "forschungszentrum",
    };
    format!("standorttyp.{name}")
}

/// Checks a company name and returns it trimmed. `own` is excluded from the
/// uniqueness check (renaming to the same name is fine).
pub fn check_company_name(
    state: Option<&GameState>,
    name: &str,
    own: Option<CompanyId>,
) -> Result<String, NameError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(NameError::Empty);
    }
    if name.chars().count() > MAX_NAME_LENGTH {
        return Err(NameError::TooLong {
            max: MAX_NAME_LENGTH,
        });
    }
    if let Some(state) = state {
        let taken = state.companies.iter().enumerate().any(|(i, c)| {
            Some(CompanyId(u32::try_from(i).unwrap_or(u32::MAX))) != own
                && c.name.eq_ignore_ascii_case(name)
        });
        if taken {
            return Err(NameError::Taken {
                name: name.to_owned(),
            });
        }
    }
    Ok(name.to_owned())
}

fn unknown_technology(catalog: &Catalog, t: TechnologyId) -> CommandError {
    CommandError::TechnologyUnknown(catalog.technologies.key(t).to_owned())
}

fn pay(ledger: &mut Ledger, asset: Account, amount: Money) -> Result<(), CommandError> {
    if ledger.cash() < amount {
        return Err(CommandError::NotEnoughCash { needed: amount });
    }
    ledger.transfer(asset, Account::Cash, amount);
    Ok(())
}

fn own_site(state: &GameState, actor: CompanyId, site: SiteId) -> Result<&Site, CommandError> {
    let s = state.site(site).ok_or(CommandError::UnknownSite)?;
    if s.owner != actor {
        return Err(CommandError::NotOwner);
    }
    Ok(s)
}

fn own_slot(
    state: &GameState,
    actor: CompanyId,
    site: SiteId,
    slot: usize,
) -> Result<&Slot, CommandError> {
    own_site(state, actor, site)?
        .slots
        .get(slot)
        .ok_or(CommandError::UnknownSlot)
}

fn check_share(value: f64) -> Result<(), CommandError> {
    if (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(CommandError::InvalidShare)
    }
}

pub(crate) fn execute(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    command: &Command,
) -> Result<(), CommandError> {
    if state.game_over {
        return Err(CommandError::GameOver);
    }
    let company = state
        .company(actor)
        .ok_or(CommandError::UnknownCompany(actor))?;
    if company.bankrupt {
        return Err(CommandError::CompanyBankrupt);
    }
    let today = state.date;
    match command {
        Command::RenameCompany { name } => {
            let name =
                check_company_name(Some(state), name, Some(actor)).map_err(CommandError::Name)?;
            state.company_mut(actor).expect("checked above").name = name;
        }
        Command::FoundSite { country, kind } => {
            if country.index() >= catalog.countries.len() {
                return Err(CommandError::DifferentCountries);
            }
            let cost = catalog.production_model.site_cost(*kind);
            let company = state.company_mut(actor).expect("checked above");
            pay(&mut company.ledger, Account::FixedAssets, cost)?;
            state.sites.push(Site {
                owner: actor,
                country: *country,
                kind: *kind,
                founded: today,
                building_cost: cost,
                deposit: None,
                slots: Vec::new(),
                inventory: Default::default(),
                workforce: PerId::from_fn(catalog.labor_groups.len(), |_| 0.0),
                staffing_due: false,
                offers: Default::default(),
                orders: Default::default(),
                research: None,
            });
        }
        Command::BuildFacility {
            site,
            facility,
            count,
        } => {
            if *count == 0 {
                return Err(CommandError::InvalidQuantity);
            }
            let s = own_site(state, actor, *site)?;
            let f = catalog.facilities.get(*facility);
            if s.kind != f.site_type {
                return Err(CommandError::WrongSiteType {
                    required: f.site_type,
                });
            }
            if let Some(t) = f.technology.filter(|&t| !state.knows(catalog, actor, t)) {
                return Err(unknown_technology(catalog, t));
            }
            let company = state.company_mut(actor).expect("checked above");
            let investment = f.investment.scale(f64::from(*count));
            pay(
                &mut company.ledger,
                Account::AssetsUnderConstruction,
                investment,
            )?;
            let ready = today.add_days(i32::try_from(f.build_days).unwrap_or(i32::MAX));
            state
                .site_mut(*site)
                .expect("checked above")
                .slots
                .push(Slot {
                    facility: *facility,
                    ready,
                    count: *count,
                    cost: investment,
                    recipe: None,
                    utilization: 0.0,
                    automation: 0.0,
                    condition: 1.0,
                    batches: Vec::new(),
                    last_runs: 0.0,
                });
        }
        Command::DevelopDeposit { site, deposit } => {
            let s = own_site(state, actor, *site)?;
            if s.kind != SiteType::Extraction {
                return Err(CommandError::WrongSiteType {
                    required: SiteType::Extraction,
                });
            }
            if s.deposit.is_some() {
                return Err(CommandError::SiteHasDeposit);
            }
            let d = catalog.deposits.get(*deposit);
            if d.country != s.country {
                return Err(CommandError::DepositOtherCountry);
            }
            if let Some(year) = d.discovered.filter(|&y| y > today.year()) {
                return Err(CommandError::DepositNotDiscovered { year });
            }
            let Some(field) = state
                .deposits
                .get(*deposit)
                .concessions
                .iter()
                .position(|c| c.site.is_none())
            else {
                return Err(CommandError::DepositUnavailable);
            };
            let share = state.deposits.get(*deposit).concessions[field].share;
            let cost = d
                .development_cost
                .scale(state.settings.market_scale * share);
            let company = state.company_mut(actor).expect("checked above");
            pay(&mut company.ledger, Account::AssetsUnderConstruction, cost)?;
            let ready = today.add_days(i32::try_from(d.development_days).unwrap_or(i32::MAX));
            let c = &mut state.deposits.get_mut(*deposit).concessions[field];
            c.site = Some(*site);
            c.ready = Some(ready);
            c.development_cost = cost;
            state.site_mut(*site).expect("checked above").deposit = Some(*deposit);
        }
        Command::SetProduction {
            site,
            slot,
            recipe,
            utilization,
        } => {
            check_share(*utilization)?;
            let s = own_site(state, actor, *site)?;
            let sl = own_slot(state, actor, *site, *slot)?;
            if let Some(recipe) = recipe {
                let r = catalog.recipes.get(*recipe);
                if r.facility != sl.facility {
                    return Err(CommandError::RecipeNotForFacility);
                }
                if let Some(t) = r.technology.filter(|&t| !state.knows(catalog, actor, t)) {
                    return Err(unknown_technology(catalog, t));
                }
                if r.extraction {
                    let fits = s
                        .deposit
                        .is_some_and(|d| catalog.deposits.get(d).resource == r.product);
                    if !fits {
                        return Err(CommandError::RecipeNeedsDeposit);
                    }
                }
            }
            // Laboratories have no recipe; their utilization sets the researcher posts.
            let lab = catalog.facilities.get(sl.facility).site_type == SiteType::ResearchCenter;
            let s = state.site_mut(*site).expect("checked above");
            let sl = &mut s.slots[*slot];
            sl.recipe = *recipe;
            sl.utilization = if recipe.is_some() || lab {
                *utilization
            } else {
                0.0
            };
            s.staffing_due = true;
        }
        Command::SetAutomation { site, slot, level } => {
            check_share(*level)?;
            let sl = own_slot(state, actor, *site, *slot)?;
            let f = catalog.facilities.get(sl.facility);
            if *level > f.automation_max + 1e-9 {
                return Err(CommandError::AutomationTooHigh {
                    max: f.automation_max,
                });
            }
            let cost = f.investment.scale(
                (level - sl.automation).max(0.0)
                    * catalog.production_model.automation_cost_share
                    * f64::from(sl.count),
            );
            let company = state.company_mut(actor).expect("checked above");
            pay(&mut company.ledger, Account::FixedAssets, cost)?;
            let s = state.site_mut(*site).expect("checked above");
            let sl = &mut s.slots[*slot];
            sl.automation = *level;
            sl.cost += cost;
            s.staffing_due = true;
        }
        Command::TransferGoods {
            from,
            to,
            product,
            quantity,
        } => {
            if !(quantity.is_finite() && *quantity > 0.0) {
                return Err(CommandError::InvalidQuantity);
            }
            let source = own_site(state, actor, *from)?;
            let target = own_site(state, actor, *to)?;
            let (from_country, to_country) = (source.country, target.country);
            let route = if from_country == to_country {
                None
            } else {
                Some(
                    state
                        .routes
                        .for_product(catalog, *product, from_country, to_country)
                        .ok_or_else(|| CommandError::NoRoute {
                            product: catalog.products.key(*product).to_owned(),
                            from: catalog.countries.key(from_country).to_owned(),
                            to: catalog.countries.key(to_country).to_owned(),
                        })?,
                )
            };
            let available = source.inventory.get(product).map_or(0.0, |s| s.quantity);
            if *quantity > available + 1e-9 {
                return Err(CommandError::NotEnoughGoods { available });
            }
            let freight = if let Some((per_unit, days)) = route {
                let cost = Money::times(per_unit, *quantity);
                let company = state.company_mut(actor).expect("checked above");
                if company.ledger.cash() < cost {
                    return Err(CommandError::NotEnoughCash { needed: cost });
                }
                company.ledger.expense(
                    CostType::Transport,
                    CostCenter::product(*from, *product),
                    Account::Cash,
                    cost,
                );
                Some(days)
            } else {
                None
            };
            let stock = state
                .site_mut(*from)
                .expect("checked above")
                .inventory
                .entry(*product)
                .or_default();
            let quality = stock.quality;
            let value = stock.take(*quantity);
            match freight {
                None => state
                    .site_mut(*to)
                    .expect("checked above")
                    .inventory
                    .entry(*product)
                    .or_default()
                    .add(*quantity, value, quality),
                Some(days) => state.shipments.push(Shipment {
                    product: *product,
                    quantity: *quantity,
                    quality,
                    value,
                    from: from_country,
                    to: Consignee::Site(*to),
                    arrival: today.add_days(i32::try_from(days).expect("short route")),
                }),
            }
        }
        Command::TakeLoan { amount, years } => {
            if *amount <= Money::ZERO {
                return Err(CommandError::InvalidAmount);
            }
            let max = catalog.finance_model.max_term_years;
            if *years == 0 || *years > max {
                return Err(CommandError::InvalidTerm { max });
            }
            let company = state.company_mut(actor).expect("checked above");
            let limit = crate::finance::credit_limit(catalog, company);
            if *amount > limit {
                return Err(CommandError::LoanTooLarge { limit });
            }
            crate::finance::grant_loan(catalog, company, *amount, *years, today);
        }
        Command::RepayLoan { loan, amount } => {
            if *amount <= Money::ZERO {
                return Err(CommandError::InvalidAmount);
            }
            let company = state.company_mut(actor).expect("checked above");
            if *loan >= company.loans.len() {
                return Err(CommandError::UnknownLoan);
            }
            let amount = (*amount).min(company.loans[*loan].balance);
            if company.ledger.cash() < amount {
                return Err(CommandError::NotEnoughCash { needed: amount });
            }
            crate::finance::repay(company, *loan, amount);
        }
        Command::SetSale {
            site,
            product,
            mode,
            keep,
        } => {
            own_site(state, actor, *site)?;
            if !(keep.is_finite() && *keep >= 0.0) {
                return Err(CommandError::InvalidQuantity);
            }
            let country = state.sites[site.index()].country;
            let offer = match mode {
                None => None,
                Some(PriceMode::Fixed(price)) => {
                    if *price <= Money::ZERO {
                        return Err(CommandError::InvalidPrice);
                    }
                    Some(SaleOffer {
                        mode: *mode.as_ref().expect("some"),
                        price: *price,
                        keep: *keep,
                        sold_today: 0.0,
                        sold_month: 0.0,
                        to_traders_month: 0.0,
                        to_companies_month: 0.0,
                    })
                }
                Some(PriceMode::Market { markup, floor }) => {
                    if !(-0.9..=2.0).contains(markup) || floor.is_negative() {
                        return Err(CommandError::InvalidPrice);
                    }
                    let start = crate::market::market_price(catalog, state, country, *product)
                        .scale(1.0 + markup)
                        .max(*floor);
                    Some(SaleOffer {
                        mode: *mode.as_ref().expect("some"),
                        price: start,
                        keep: *keep,
                        sold_today: 0.0,
                        sold_month: 0.0,
                        to_traders_month: 0.0,
                        to_companies_month: 0.0,
                    })
                }
            };
            let s = state.site_mut(*site).expect("checked above");
            match offer {
                Some(mut offer) => {
                    // A new price does not reset the month's sales and policy limits.
                    if let Some(old) = s.offers.get(product) {
                        offer.sold_month = old.sold_month;
                        offer.to_traders_month = old.to_traders_month;
                        offer.to_companies_month = old.to_companies_month;
                    }
                    s.offers.insert(*product, offer);
                }
                None => {
                    s.offers.remove(product);
                }
            }
        }
        Command::SetPurchase {
            site,
            product,
            target,
            max_price,
            min_quality,
        } => {
            own_site(state, actor, *site)?;
            if !(target.is_finite() && *target >= 0.0) {
                return Err(CommandError::InvalidQuantity);
            }
            if *target > 0.0 && *max_price <= Money::ZERO {
                return Err(CommandError::InvalidPrice);
            }
            if !(0.0..=100.0).contains(min_quality) {
                return Err(CommandError::InvalidShare);
            }
            let s = state.site_mut(*site).expect("checked above");
            if *target > 0.0 {
                s.orders.insert(
                    *product,
                    PurchaseOrder {
                        target: *target,
                        max_price: *max_price,
                        min_quality: *min_quality,
                        bought_month: 0.0,
                    },
                );
            } else {
                s.orders.remove(product);
            }
        }
        Command::SetResearch { site, technology } => {
            let s = own_site(state, actor, *site)?;
            if s.kind != SiteType::ResearchCenter {
                return Err(CommandError::WrongSiteType {
                    required: SiteType::ResearchCenter,
                });
            }
            if let Some(t) = technology
                && !crate::research::can_research(catalog, state, actor, *t)
            {
                return Err(CommandError::NotResearchable(
                    catalog.technologies.key(*t).to_owned(),
                ));
            }
            let s = state.site_mut(*site).expect("checked above");
            s.research = *technology;
            s.staffing_due = true;
        }
        Command::SetSalesPolicy { buyer, scope, rule } => {
            if let Some(r) = rule {
                if r.min_price.is_some_and(|p| p.is_negative()) {
                    return Err(CommandError::InvalidPrice);
                }
                if r.max_per_month
                    .is_some_and(|m| !(m.is_finite() && m >= 0.0))
                {
                    return Err(CommandError::InvalidQuantity);
                }
            }
            let company = state.company_mut(actor).expect("checked above");
            policy::set(company, *buyer, *scope, *rule);
        }
    }
    Ok(())
}
