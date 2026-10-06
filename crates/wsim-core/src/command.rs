//! Commands: the only way to change a company (Lastenheft §10).
//!
//! The player, AI companies and later managers all act through the same commands and
//! the same checks.

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::{Catalog, FacilitySize, SiteType};
use crate::deals::{DealObject, OfferAnswer};
use crate::ids::{
    CountryId, DepositId, FacilityId, GoodsGroupId, Id, ProductId, RecipeId, TechnologyId,
};
use crate::ledger::{Account, CostCenter, CostType, Ledger};
use crate::message::{Message, Param, keys};
use crate::money::Money;
use crate::plots;
use crate::policy::{self, BuyerGroup, SalesRule, Scope};
use crate::state::{
    CompanyId, Consignee, GameState, Operation, PerId, PlotId, PriceMode, PurchaseOrder, SaleOffer,
    Shipment, Site, SiteId, Slot, Tenure,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Command {
    /// Renames the acting company.
    RenameCompany { name: String },
    /// Founds a site (land and buildings) in a country, on the largest free plot (M35),
    /// bought when the cash pays building and land, else leased.
    FoundSite { country: CountryId, kind: SiteType },
    /// Founds a site on a free plot, bought or leased (M35).
    FoundSiteOnPlot {
        plot: PlotId,
        kind: SiteType,
        lease: bool,
    },
    /// Buys the leased plot of a site at today's value (M35).
    BuyPlot { site: SiteId },
    /// Builds `count` identical units of a facility in a size (M36; default medium) at
    /// a site, working together in one slot; they produce after the construction time.
    BuildFacility {
        site: SiteId,
        facility: FacilityId,
        #[serde(default = "one_unit")]
        count: u32,
        #[serde(default)]
        size: FacilitySize,
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
    /// A research center develops a product to its next levels (M37) instead of
    /// researching a technology; `None` stops it.
    SetDevelopment {
        site: SiteId,
        product: Option<ProductId>,
    },
    /// Sets who besides consumers and governments may buy the company's goods
    /// (`None` removes the policy of this scope, the more general one applies).
    SetSalesPolicy {
        buyer: BuyerGroup,
        scope: Scope,
        rule: Option<SalesRule>,
    },
    /// Monthly advertising budget for a goods group in a country (0 stops it).
    SetAdvertising {
        country: CountryId,
        group: GoodsGroupId,
        budget: Money,
    },
    /// Wage premium of a site over the country's wages (M18, 0.1 = 10 %): when workers
    /// are scarce, better payers get them first and hire them away from others.
    SetWagePremium { site: SiteId, premium: f64 },
    /// Sets the asking price of an existing offer (M18): a fixed price becomes this
    /// price; an automatic price goes on from here (never below its floor).
    SetPrice {
        site: SiteId,
        product: ProductId,
        price: Money,
    },
    /// Shuts down `count` units of a finished facility (M22): no production and staff,
    /// less maintenance, no wear. Units of a larger facility become a facility of their
    /// own (at the end of the site's list).
    MothballFacility {
        site: SiteId,
        slot: usize,
        count: u32,
    },
    /// Starts a shut down facility up again (M22); it produces after the restart time.
    RestartFacility { site: SiteId, slot: usize },
    /// Sells `count` units of a finished facility for part of their book value (M22).
    /// Selling all units removes the facility; later facilities move up one place.
    SellFacility {
        site: SiteId,
        slot: usize,
        count: u32,
    },
    /// Offers another company a price for one of its sites or for a licence (M30).
    MakeOffer {
        seller: CompanyId,
        object: DealObject,
        price: Money,
    },
    /// Answers an offer: accept, decline, or (the seller, once) name a higher price.
    AnswerOffer { offer: u32, answer: OfferAnswer },
    /// Withdraws the price the acting company named last.
    WithdrawOffer { offer: u32 },
    /// Gives an end product of the company its own name (M42); `None` removes it.
    NameProduct {
        product: ProductId,
        name: Option<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NameError {
    Empty,
    TooLong {
        max: usize,
    },
    Taken {
        name: String,
    },
    /// A word of the name is a real product or brand name (M42).
    Excluded {
        name: String,
    },
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
            NameError::Excluded { name } => {
                Message::error(keys::NAME_EXCLUDED).with("name", Param::Text(name.clone()))
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
    InvalidWagePremium {
        max: f64,
    },
    /// The site offers no such product (key of the product).
    NoOffer(String),
    /// Already known, or prerequisites missing (key of the technology).
    NotResearchable(String),
    /// The company cannot make the product or it is fully developed (M37).
    NotDevelopable(String),
    /// Raw materials, semi-finished goods, components and power have no names (M42).
    NotNameable(String),
    /// No transport route for the product between the two countries.
    NoRoute {
        product: String,
        from: String,
        to: String,
    },
    /// The facility is not finished yet (M22).
    UnderConstruction,
    AlreadyMothballed,
    NotMothballed,
    /// No free plot in the country (M35).
    NoFreePlot,
    UnknownPlot,
    PlotTaken,
    /// Extraction sites stand on their concession.
    PlotNotNeeded,
    /// The site would need more land than its plot has (ha).
    PlotTooSmall {
        needed_ha: f64,
        area_ha: f64,
    },
    PlotOwned,
    /// More units than the facility has (M22).
    TooManyUnits {
        count: u32,
    },
    /// An offer to the acting company itself (M30).
    OwnObject,
    SellerBankrupt,
    /// A bid in an auction below its minimum (M38).
    BelowMinimumBid {
        minimum: Money,
    },
    /// The site or technology does not (or no longer) belong to the seller.
    NotSellersObject,
    SiteTooYoung {
        months: u32,
    },
    /// The seller does not know the technology, or the buyer already does.
    LicenseNotPossible,
    OfferExists,
    OfferBlocked {
        until: Date,
    },
    UnknownOffer,
    NotYourTurn,
    NoCounter,
    BuyerCannotPay,
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
            CommandError::InvalidWagePremium { max } => {
                e(keys::COMMAND_INVALID_WAGE_PREMIUM).with("max", Param::Number(*max * 100.0))
            }
            CommandError::NoOffer(product) => e(keys::COMMAND_NO_OFFER)
                .with("produkt", Param::TextKey(format!("produkt.{product}"))),
            CommandError::NotResearchable(t) => e(keys::COMMAND_NOT_RESEARCHABLE)
                .with("technologie", Param::TextKey(format!("technologie.{t}"))),
            CommandError::NotDevelopable(p) => e(keys::COMMAND_NOT_DEVELOPABLE)
                .with("produkt", Param::TextKey(format!("produkt.{p}"))),
            CommandError::NotNameable(p) => e(keys::COMMAND_NOT_NAMEABLE)
                .with("produkt", Param::TextKey(format!("produkt.{p}"))),
            CommandError::NoRoute { product, from, to } => e(keys::COMMAND_NO_ROUTE)
                .with("produkt", Param::TextKey(format!("produkt.{product}")))
                .with("von", Param::TextKey(format!("land.{from}")))
                .with("nach", Param::TextKey(format!("land.{to}"))),
            CommandError::UnderConstruction => e(keys::COMMAND_UNDER_CONSTRUCTION),
            CommandError::AlreadyMothballed => e(keys::COMMAND_ALREADY_MOTHBALLED),
            CommandError::NotMothballed => e(keys::COMMAND_NOT_MOTHBALLED),
            CommandError::TooManyUnits { count } => {
                e(keys::COMMAND_TOO_MANY_UNITS).with("anzahl", Param::Integer(i64::from(*count)))
            }
            CommandError::NoFreePlot => e(keys::COMMAND_NO_FREE_PLOT),
            CommandError::UnknownPlot => e(keys::COMMAND_UNKNOWN_PLOT),
            CommandError::PlotTaken => e(keys::COMMAND_PLOT_TAKEN),
            CommandError::PlotNotNeeded => e(keys::COMMAND_PLOT_NOT_NEEDED),
            CommandError::PlotTooSmall { needed_ha, area_ha } => e(keys::COMMAND_PLOT_TOO_SMALL)
                .with("benoetigt", Param::Number(*needed_ha))
                .with("flaeche", Param::Number(*area_ha)),
            CommandError::PlotOwned => e(keys::COMMAND_PLOT_OWNED),
            CommandError::OwnObject => e(keys::COMMAND_OWN_OBJECT),
            CommandError::SellerBankrupt => e(keys::COMMAND_SELLER_BANKRUPT),
            CommandError::BelowMinimumBid { minimum } => {
                e(keys::COMMAND_BELOW_MINIMUM_BID).with("mindestgebot", Param::Money(*minimum))
            }
            CommandError::NotSellersObject => e(keys::COMMAND_NOT_SELLERS_OBJECT),
            CommandError::SiteTooYoung { months } => {
                e(keys::COMMAND_SITE_TOO_YOUNG).with("monate", Param::Integer(i64::from(*months)))
            }
            CommandError::LicenseNotPossible => e(keys::COMMAND_LICENSE_NOT_POSSIBLE),
            CommandError::OfferExists => e(keys::COMMAND_OFFER_EXISTS),
            CommandError::OfferBlocked { until } => {
                e(keys::COMMAND_OFFER_BLOCKED).with("datum", Param::Date(*until))
            }
            CommandError::UnknownOffer => e(keys::COMMAND_UNKNOWN_OFFER),
            CommandError::NotYourTurn => e(keys::COMMAND_NOT_YOUR_TURN),
            CommandError::NoCounter => e(keys::COMMAND_NO_COUNTER),
            CommandError::BuyerCannotPay => e(keys::COMMAND_BUYER_CANNOT_PAY),
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

/// Checks a product name (M42) and returns it trimmed: not empty, not too long, no real
/// product or brand name and not used by another company for the product.
pub fn check_product_name(
    catalog: &Catalog,
    state: &GameState,
    product: ProductId,
    name: &str,
    own: CompanyId,
) -> Result<String, NameError> {
    let name = name.split_whitespace().collect::<Vec<_>>().join(" ");
    if name.is_empty() {
        return Err(NameError::Empty);
    }
    let max = crate::product_names::MAX_LENGTH;
    if name.chars().count() > max {
        return Err(NameError::TooLong { max });
    }
    if catalog.product_naming.is_excluded(&name) {
        return Err(NameError::Excluded { name });
    }
    if crate::product_names::taken(state, product, &name, own) {
        return Err(NameError::Taken { name });
    }
    Ok(name)
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

/// A new site with its building and, where it needs one, its plot (M35): the land is
/// paid with the building unless leased.
fn found_site(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    today: Date,
    (country, kind): (CountryId, SiteType),
    (plot, lease): (Option<PlotId>, bool),
) -> Result<(), CommandError> {
    let cost = catalog.production_model.site_cost(kind);
    let land = plot
        .filter(|_| !lease)
        .map_or(Money::ZERO, |p| plots::value(catalog, state, p));
    let company = state.company_mut(actor).expect("checked above");
    if company.ledger.cash() < cost + land {
        return Err(CommandError::NotEnoughCash {
            needed: cost + land,
        });
    }
    pay(&mut company.ledger, Account::FixedAssets, cost)?;
    if land > Money::ZERO {
        pay(&mut company.ledger, Account::Land, land)?;
    }
    state.sites.push(Site {
        owner: actor,
        country,
        kind,
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
        development: None,
        wage_premium: 0.0,
        acquired: None,
        goodwill: None,
        plot: None,
    });
    if let Some(p) = plot {
        let site = SiteId(u32::try_from(state.sites.len() - 1).expect("site count fits u32"));
        let tenure = if lease {
            Tenure::Leased
        } else {
            Tenure::Owned(land)
        };
        plots::occupy(state, p, site, tenure);
    }
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
        Command::NameProduct { product, name } => {
            if catalog.product_naming.style(catalog, *product).is_none() {
                return Err(CommandError::NotNameable(
                    catalog.products.key(*product).to_owned(),
                ));
            }
            let names = match name {
                Some(name) => Some(
                    check_product_name(catalog, state, *product, name, actor)
                        .map_err(CommandError::Name)?,
                ),
                None => None,
            };
            let company = state.company_mut(actor).expect("checked above");
            match names {
                Some(name) => company.product_names.insert(*product, name),
                None => company.product_names.remove(product),
            };
        }
        Command::FoundSite { country, kind } => {
            if country.index() >= catalog.countries.len() {
                return Err(CommandError::DifferentCountries);
            }
            // Without a choice the largest free plot: the most room to grow.
            let plot = if plots::needs_plot(catalog, *kind) {
                Some(
                    plots::choose(catalog, state, *country, f64::INFINITY, 0.0)
                        .ok_or(CommandError::NoFreePlot)?,
                )
            } else {
                None
            };
            // Bought when the cash pays building and land, else leased.
            let cost = catalog.production_model.site_cost(*kind);
            let cash = state.company(actor).expect("checked above").ledger.cash();
            let lease = plot.is_some_and(|p| cash < cost + plots::value(catalog, state, p));
            found_site(
                state,
                catalog,
                actor,
                today,
                (*country, *kind),
                (plot, lease),
            )?;
        }
        Command::FoundSiteOnPlot { plot, kind, lease } => {
            let p = state
                .plots
                .get(plot.index())
                .ok_or(CommandError::UnknownPlot)?;
            if p.site.is_some() {
                return Err(CommandError::PlotTaken);
            }
            if !plots::needs_plot(catalog, *kind) {
                return Err(CommandError::PlotNotNeeded);
            }
            let country = p.country;
            found_site(
                state,
                catalog,
                actor,
                today,
                (country, *kind),
                (Some(*plot), *lease),
            )?;
        }
        Command::BuyPlot { site } => {
            let plot = own_site(state, actor, *site)?
                .plot
                .ok_or(CommandError::UnknownPlot)?;
            if matches!(state.plots[plot.index()].tenure, Tenure::Owned(_)) {
                return Err(CommandError::PlotOwned);
            }
            let price = plots::value(catalog, state, plot);
            let company = state.company_mut(actor).expect("checked above");
            pay(&mut company.ledger, Account::Land, price)?;
            state.plots[plot.index()].tenure = Tenure::Owned(price);
        }
        Command::BuildFacility {
            site,
            facility,
            count,
            size,
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
            if let Some(plot) = s.plot {
                let needed_ha = plots::site_area(catalog, s, Some((*facility, *size, *count)));
                let area_ha = state.plots[plot.index()].area_ha;
                if needed_ha > area_ha + 1e-9 {
                    return Err(CommandError::PlotTooSmall { needed_ha, area_ha });
                }
            }
            if let Some(t) = f.technology.filter(|&t| !state.knows(catalog, actor, t)) {
                return Err(unknown_technology(catalog, t));
            }
            let company = state.company_mut(actor).expect("checked above");
            let sizes = &catalog.production_model.sizes;
            let investment = f
                .investment
                .scale(sizes.investment(*size) * f64::from(*count));
            pay(
                &mut company.ledger,
                Account::AssetsUnderConstruction,
                investment,
            )?;
            let days = sizes.build_days(*size, f.build_days);
            let ready = today.add_days(i32::try_from(days).unwrap_or(i32::MAX));
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
                    // Laboratories have no recipe: they work as soon as they are ready.
                    utilization: if f.site_type == SiteType::ResearchCenter {
                        1.0
                    } else {
                        0.0
                    },
                    automation: 0.0,
                    condition: 1.0,
                    batches: Vec::new(),
                    last_runs: 0.0,
                    limit: None,
                    operation: crate::state::Operation::Running,
                    size: *size,
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
                    * catalog.production_model.sizes.investment(sl.size)
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
            // Sea freight is cheaper where either end lies at a port (M35).
            let sea = plots::sea_freight(catalog, state, source)
                .min(plots::sea_freight(catalog, state, target));
            let route = if from_country == to_country {
                None
            } else {
                Some(
                    state
                        .routes
                        .for_product_via(catalog, *product, (from_country, to_country), sea)
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
                        sold_last_month: 0.0,
                        to_traders_month: 0.0,
                        to_companies_month: 0.0,
                    })
                }
                Some(PriceMode::Market { markup, floor }) => {
                    if !(-0.9..=2.0).contains(markup) || floor.is_negative() {
                        return Err(CommandError::InvalidPrice);
                    }
                    // Same markup as before: the price search goes on from the current
                    // price, only the new floor applies.
                    let current = state.sites[site.index()]
                        .offers
                        .get(product)
                        .filter(|o| {
                            matches!(o.mode, PriceMode::Market { markup: m, .. } if m == *markup)
                        })
                        .map(|o| o.price);
                    let start = current
                        .unwrap_or_else(|| {
                            crate::market::market_price(catalog, state, country, *product)
                                .scale(1.0 + markup)
                        })
                        .max(*floor);
                    Some(SaleOffer {
                        mode: *mode.as_ref().expect("some"),
                        price: start,
                        keep: *keep,
                        sold_today: 0.0,
                        sold_month: 0.0,
                        sold_last_month: 0.0,
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
                        offer.sold_last_month = old.sold_last_month;
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
                let (bought_month, bought_last_month) = s
                    .orders
                    .get(product)
                    .map_or((0.0, 0.0), |o| (o.bought_month, o.bought_last_month));
                s.orders.insert(
                    *product,
                    PurchaseOrder {
                        target: *target,
                        max_price: *max_price,
                        min_quality: *min_quality,
                        bought_month,
                        bought_last_month,
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
            if technology.is_some() {
                s.development = None;
            }
            s.staffing_due = true;
        }
        Command::SetDevelopment { site, product } => {
            let s = own_site(state, actor, *site)?;
            if s.kind != SiteType::ResearchCenter {
                return Err(CommandError::WrongSiteType {
                    required: SiteType::ResearchCenter,
                });
            }
            if let Some(p) = product
                && !crate::development::can_develop(catalog, state, actor, *p)
            {
                return Err(CommandError::NotDevelopable(
                    catalog.products.key(*p).to_owned(),
                ));
            }
            let s = state.site_mut(*site).expect("checked above");
            s.development = *product;
            if product.is_some() {
                s.research = None;
            }
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
        Command::SetAdvertising {
            country,
            group,
            budget,
        } => {
            if budget.is_negative() {
                return Err(CommandError::InvalidAmount);
            }
            let company = state.company_mut(actor).expect("checked above");
            company
                .advertising
                .retain(|a| !(a.country == *country && a.group == *group));
            if *budget > Money::ZERO {
                company.advertising.push(crate::state::Advertising {
                    country: *country,
                    group: *group,
                    budget: *budget,
                });
                company.advertising.sort_by_key(|a| (a.country, a.group));
            }
        }
        Command::SetWagePremium { site, premium } => {
            let max = catalog.production_model.wage_premium_max;
            if !(premium.is_finite() && (0.0..=max + 1e-9).contains(premium)) {
                return Err(CommandError::InvalidWagePremium { max });
            }
            own_site(state, actor, *site)?;
            let s = state.site_mut(*site).expect("checked above");
            s.wage_premium = *premium;
            // A higher premium may hire workers away from others at once.
            s.staffing_due = true;
        }
        Command::SetPrice {
            site,
            product,
            price,
        } => {
            own_site(state, actor, *site)?;
            if *price <= Money::ZERO {
                return Err(CommandError::InvalidPrice);
            }
            let s = state.site_mut(*site).expect("checked above");
            let Some(offer) = s.offers.get_mut(product) else {
                return Err(CommandError::NoOffer(
                    catalog.products.key(*product).to_owned(),
                ));
            };
            match &mut offer.mode {
                PriceMode::Fixed(fixed) => {
                    *fixed = *price;
                    offer.price = *price;
                }
                PriceMode::Market { floor, .. } => offer.price = (*price).max(*floor),
            }
        }
        Command::MothballFacility { site, slot, count } => {
            let sl = own_slot(state, actor, *site, *slot)?;
            check_units(sl, *count, today)?;
            if sl.mothballed() {
                return Err(CommandError::AlreadyMothballed);
            }
            let s = state.site_mut(*site).expect("checked above");
            let shut = Operation::Mothballed { since: today };
            let sl = &mut s.slots[*slot];
            if *count == sl.count {
                sl.operation = shut;
                sl.last_runs = 0.0;
                sl.limit = None;
            } else {
                // The shut units become a facility of their own; running batches stay.
                let cost = sl.share_of_cost(*count);
                let mut part = sl.clone();
                sl.cost -= cost;
                sl.count -= count;
                part.count = *count;
                part.cost = cost;
                part.batches = Vec::new();
                part.operation = shut;
                part.last_runs = 0.0;
                part.limit = None;
                s.slots.push(part);
            }
            s.staffing_due = true;
        }
        Command::RestartFacility { site, slot } => {
            let sl = own_slot(state, actor, *site, *slot)?;
            if !sl.mothballed() {
                return Err(CommandError::NotMothballed);
            }
            let model = &catalog.production_model;
            let cost = sl.cost.scale(model.restart_cost_share);
            let ledger = &mut state.company_mut(actor).expect("checked above").ledger;
            if ledger.cash() < cost {
                return Err(CommandError::NotEnoughCash { needed: cost });
            }
            ledger.expense(
                CostType::Maintenance,
                CostCenter::site(*site),
                Account::Cash,
                cost,
            );
            let until = today.add_days(i32::try_from(model.restart_days).unwrap_or(i32::MAX));
            let s = state.site_mut(*site).expect("checked above");
            s.slots[*slot].operation = Operation::Restarting { until };
            s.staffing_due = true;
        }
        Command::SellFacility { site, slot, count } => {
            let sl = own_slot(state, actor, *site, *slot)?;
            check_units(sl, *count, today)?;
            let (book, proceeds) = crate::production::sale_value(catalog, sl, *count, today);
            let s = state.site_mut(*site).expect("checked above");
            let open = if *count == s.slots[*slot].count {
                s.slots.remove(*slot).batches
            } else {
                let sl = &mut s.slots[*slot];
                let cost = sl.share_of_cost(*count);
                sl.cost -= cost;
                sl.count -= count;
                Vec::new()
            };
            // Goods still in production of a facility that goes are finished at once.
            crate::production::deliver(catalog, s, open);
            s.staffing_due = true;
            let ledger = &mut state.company_mut(actor).expect("checked above").ledger;
            let center = CostCenter::site(*site);
            // Loss (or gain) against the book value, then the proceeds for the rest.
            if book > proceeds {
                ledger.expense(
                    CostType::Other,
                    center,
                    Account::FixedAssets,
                    book - proceeds,
                );
            } else {
                ledger.income(
                    CostType::Other,
                    center,
                    Account::FixedAssets,
                    proceeds - book,
                );
            }
            ledger.transfer(Account::Cash, Account::FixedAssets, proceeds);
        }
        Command::MakeOffer {
            seller,
            object,
            price,
        } => crate::deals::make_offer(state, catalog, actor, *seller, *object, *price)?,
        Command::AnswerOffer { offer, answer } => {
            crate::deals::answer_offer(state, catalog, actor, *offer, *answer)?;
        }
        Command::WithdrawOffer { offer } => crate::deals::withdraw_offer(state, actor, *offer)?,
    }
    Ok(())
}

/// A shut down or sold part of a facility: finished, and not more units than it has.
fn check_units(sl: &Slot, count: u32, today: Date) -> Result<(), CommandError> {
    if sl.ready > today {
        return Err(CommandError::UnderConstruction);
    }
    if count == 0 {
        return Err(CommandError::InvalidQuantity);
    }
    if count > sl.count {
        return Err(CommandError::TooManyUnits { count: sl.count });
    }
    Ok(())
}
