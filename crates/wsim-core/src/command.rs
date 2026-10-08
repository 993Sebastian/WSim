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
    VehicleId,
};
use crate::ledger::{Account, CostCenter, CostType, Ledger};
use crate::message::{Message, Param, keys};
use crate::money::Money;
use crate::plots;
use crate::policy::{self, BuyerGroup, SalesRule, Scope};
use crate::state::{
    CompanyId, Consignee, GameState, ManagerId, Operation, PerId, PlotId, Position, PriceMode,
    PurchaseOrder, SaleOffer, Shipment, Site, SiteId, Slot, Tenure,
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
    /// Proposes a supply contract between a site of the actor and one of another company
    /// (W4); an AI company answers at once.
    ProposeContract {
        seller: SiteId,
        buyer: SiteId,
        product: ProductId,
        per_month: f64,
        price: Money,
        months: u32,
        #[serde(default)]
        min_quality: f64,
        penalty: f64,
    },
    /// Accepts or declines a proposed contract.
    AnswerContract { contract: u32, accept: bool },
    /// Ends a contract early (with the penalty) or withdraws a proposal.
    CancelContract { contract: u32 },
    /// Buys vehicles for the own fleet (W5).
    BuyVehicles { vehicle: VehicleId, count: u32 },
    /// Sells vehicles of the own fleet at a share of their book value.
    SellVehicles { vehicle: VehicleId, count: u32 },
    /// How the company sends its own loads, and whether its fleet carries for others.
    SetLogistics {
        mode: crate::logistics::FreightMode,
        carry_for_others: bool,
    },
    /// Founds a subsidiary with its capital (W6).
    FoundSubsidiary {
        name: String,
        country: CountryId,
        capital: Money,
        focus: crate::group::SubsidiaryFocus,
    },
    /// Capital into a direct subsidiary (> 0) or back to the parent (< 0).
    MoveCapital { company: CompanyId, amount: Money },
    /// Moves a site between the company and a direct subsidiary at book values.
    TransferSite { site: SiteId, to: CompanyId },
    /// Changes the focus of a direct subsidiary.
    SetSubsidiaryFocus {
        company: CompanyId,
        focus: crate::group::SubsidiaryFocus,
    },
    /// Goes public with new shares (K1).
    GoPublic { share: f64 },
    /// New shares of a listed company.
    IssueShares { share: f64 },
    /// The share of last year's profit paid out as a dividend (K1; since PE4 a short
    /// form of `SetDividendPolicy`).
    SetDividend { payout: f64 },
    /// The dividend policy of the company (PE4).
    SetDividendPolicy {
        policy: crate::dividends::DividendPolicy,
    },
    /// A dividend paid at once (PE4).
    SpecialDividend { amount: Money },
    /// Buys a share of a listed company from its free float.
    BuyShares { company: CompanyId, share: f64 },
    /// Sells a share of a listed company to investors.
    SellShares { company: CompanyId, share: f64 },
    /// Bids for all shares of a listed company it does not hold (K3).
    TakeOver { company: CompanyId },
    /// Buys back a share of its own stock from the free float.
    BuyBackShares { share: f64 },
    /// Issues a bond of `amount`, repaid at once after `years` (K2).
    IssueBond { amount: Money, years: u32 },
    /// Buys a bond back before maturity.
    RedeemBond { bond: usize },
    /// What a bank of the player's group offers (K4).
    SetBank {
        company: CompanyId,
        settings: crate::bank::BankSettings,
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
    /// Sets a site's training target, 0–1; `None` follows the strategy (W1).
    SetTraining { site: SiteId, target: Option<f64> },
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
    /// Retrofits a facility against pollutants by one level (H2).
    Retrofit { site: SiteId, slot: usize },
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
    /// Hires a free manager of the market for a free position of the company (MA1).
    HireManager {
        manager: ManagerId,
        position: Position,
    },
    /// Moves a manager of the company to another free position of it (MA1).
    MoveManager {
        manager: ManagerId,
        position: Position,
    },
    /// Dismisses a manager of the company against a severance pay (MA1).
    DismissManager { manager: ManagerId },
    /// Answers a concern of one of the company's positions (MA2).
    AnswerConcern {
        concern: u32,
        answer: crate::management::ConcernAnswer,
    },
    /// Sets the budget of a position as shares of its reference per decision and per
    /// year; `None` restores the defaults (MA2).
    SetBudget {
        position: Position,
        shares: Option<(f64, f64)>,
    },
    /// A position asks about a muted or declined topic again (MA2).
    AskAgain {
        position: Position,
        topic: crate::decision::Topic,
    },
    /// Budget shares for all positions of a type in a scope; `None` removes the rule
    /// (MA3).
    SetBudgetRule {
        kind: crate::state::PositionKind,
        scope: crate::state::RuleScope,
        shares: Option<(f64, f64)>,
    },
    /// The strategy of a field for a scope; `None` removes it (MA4).
    SetStrategy {
        scope: crate::strategy::StrategyScope,
        field: crate::strategy::StrategyField,
        value: Option<crate::strategy::StrategyValue>,
    },
    /// The mandate to the board (MA5).
    SetMandate { mandate: crate::mandate::Mandate },
    /// A head fills the free specialist positions of its unit itself, or no longer (MA6).
    SetHiringByHead { position: Position, enabled: bool },
    /// Raises the yearly salary of a manager of the company (MA6).
    RaiseSalary { manager: ManagerId, salary: Money },
    /// Offers another company's manager a free position of the company (MA6).
    PoachManager {
        manager: ManagerId,
        position: Position,
    },
    /// Keeps a manager of the company at the salary another company offered him (MA6).
    MatchOffer { manager: ManagerId },
    /// Lets a manager of the company go to the company that made him an offer (MA6).
    LetGo { manager: ManagerId },
    /// A free candidate or a manager of the company from a position below becomes the
    /// successor of a holder who retires soon, or takes the free position at once (PE1).
    AppointSuccessor {
        manager: ManagerId,
        position: Position,
    },
    /// Offers a manager of the company who retires soon to stay longer at a higher salary
    /// (PE1); he agrees with a chance.
    ExtendContract { manager: ManagerId, years: u8 },
    /// The position stays free when its holder retires (PE1).
    LeaveVacant { position: Position },
    /// Moves the headquarters to another country (ZA1) or city (W2; none: the capital).
    SetHeadquarters {
        country: CountryId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        city: Option<String>,
    },
    /// Sets the number of employees of a central department (ZA2).
    StaffDepartment {
        department: crate::catalog::DepartmentKind,
        staff: u32,
    },
    /// The policy on takeovers, licences and start-ups (ZA2).
    SetParticipations {
        budget: Option<Money>,
        risk: f64,
        limits: std::collections::BTreeMap<crate::catalog::DepartmentKind, Money>,
    },
    /// Replaces a loan by a new one at today's rate for its balance and the months left
    /// (ZA3).
    RefinanceLoan { loan: usize },
    /// Pledges to the open round of a start-up, or buys shares of its founders and
    /// investors between rounds (SU2).
    InvestInVenture { venture: u32, amount: Money },
    /// Money for a start-up without shares: a higher chance for its phase (SU2).
    GrantVenture { venture: u32, amount: Money },
    /// Sells a share of a start-up to investors (SU2).
    SellVentureStake { venture: u32, share: f64 },
    /// Offers all of the company's shares of a start-up to the other companies from a
    /// minimum price until the next month start; without one the offer is withdrawn
    /// (ZA4).
    OfferVentureStake {
        venture: u32,
        minimum: Option<Money>,
    },
    /// Buys all shares another company offers of a start-up, at a price from its minimum
    /// (ZA4).
    BuyVentureStake {
        venture: u32,
        seller: CompanyId,
        price: Money,
    },
    /// The pace of a start-up the company holds the majority of (SU2).
    SteerVenture {
        venture: u32,
        pace: crate::state::VenturePace,
    },
    /// Buys out the other owners of a start-up: it becomes a subsidiary (SU2).
    IntegrateVenture { venture: u32 },
    /// Turns the project of a research center into a start-up and sells a share of it
    /// to investors (SU3).
    SpinOff { site: SiteId, sell: f64 },
    /// The person founds its company (PE3): name, start form, seat and capital paid in.
    FoundCompany {
        name: String,
        form: crate::state::StartForm,
        country: CountryId,
        capital: Money,
    },
    /// The person's yearly salary as CEO of the main company (PE3).
    SetPersonSalary { amount: Money },
    /// The person's lifestyle from the next month on (PE3).
    SetLifestyle { level: crate::state::Lifestyle },
    /// The person pays capital into a company (PE3).
    ContributeCapital { company: CompanyId, amount: Money },
    /// The person lends a company money (PE3).
    LendToCompany {
        company: CompanyId,
        amount: Money,
        rate: f64,
        years: u32,
    },
    /// A company pays capital back to the person (PE3).
    WithdrawCapital { company: CompanyId, amount: Money },
    /// The person buys `share` of a company from one of its holders (PE5).
    BuyStake {
        company: CompanyId,
        holder: crate::state::Holder,
        share: f64,
        price: Money,
    },
    /// The company buys `share` of another company from one of its holders (PE5).
    BidForStake {
        company: CompanyId,
        holder: crate::state::Holder,
        share: f64,
        price: Money,
    },
    /// The person sells `share` of a company to investors (PE5).
    SellStake { company: CompanyId, share: f64 },
    /// Another company of the person becomes the main company (PE5).
    SelectCompany { company: CompanyId },
    /// The person invests in a start-up from its private account (PE5).
    InvestPrivately { venture: u32, amount: Money },
    /// The child who inherits (index among the children); `None`: the rule decides (PE6).
    SetHeir { child: Option<u32> },
    /// The person hands everything to the heir while alive (PE6).
    HandOver {},
}

impl Command {
    /// Commands of the person rather than of a company (PE3).
    pub fn is_personal(&self) -> bool {
        matches!(
            self,
            Command::FoundCompany { .. }
                | Command::SetPersonSalary { .. }
                | Command::SetLifestyle { .. }
                | Command::ContributeCapital { .. }
                | Command::LendToCompany { .. }
                | Command::WithdrawCapital { .. }
                | Command::BuyStake { .. }
                | Command::SellStake { .. }
                | Command::SelectCompany { .. }
                | Command::InvestPrivately { .. }
                | Command::SetHeir { .. }
                | Command::HandOver {}
        )
    }
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
    /// A strategy outside its bounds or for another field (MA4).
    InvalidStrategy,
    /// A mandate outside its bounds (MA5).
    InvalidMandate,
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
    /// A training target outside 0–1 (W1).
    InvalidTraining,
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
    /// No such manager (any more) in the market or a company (MA1).
    UnknownManager,
    /// The manager works for a company already.
    ManagerEmployed,
    /// The manager does not work for the acting company.
    NotYourManager,
    /// The site has no such position.
    UnknownPosition,
    PositionTaken,
    /// No such concern of the acting company (MA2).
    UnknownConcern,
    /// The concern was answered or expired already.
    ConcernClosed,
    UnknownOption,
    /// Only a head fills positions itself (MA6).
    NotAHead,
    /// A salary is only raised.
    SalaryNotHigher,
    /// The manager works for no company: hire him instead.
    ManagerFree,
    /// The manager works for the acting company itself.
    OwnManager,
    /// The manager has an open offer already.
    ManagerHasOffer,
    /// The manager had an offer lately; the next one from the date.
    ManagerCourted {
        until: Date,
    },
    /// No open offer to the manager.
    NoPoachOffer,
    /// No such country (ZA1).
    UnknownCountry,
    /// The headquarters are there already.
    SameHeadquarters,
    /// No such city in the country (W2).
    UnknownCity,
    /// An embargo forbids trade between the two countries (W3).
    Embargo {
        from: String,
        to: String,
    },
    /// The data have no contracts (W4).
    NoContracts,
    /// Both sites belong to the same company.
    ContractWithItself,
    /// No such contract, or none the company may answer or cancel.
    UnknownContract,
    /// The other company declines (reason: text `vertrag.abgelehnt.<key>`).
    ContractDeclined {
        reason: String,
    },
    /// The data have no logistics (W5).
    NoLogistics,
    /// The vehicle cannot be bought for a fleet (now).
    VehicleNotForFleet,
    /// The fleet has fewer vehicles of the kind.
    TooManyVehicles {
        count: u32,
    },
    /// The data have no subsidiaries (W6).
    NoSubsidiaries,
    /// Less capital than a subsidiary needs.
    CapitalTooLow {
        min: Money,
    },
    /// Not a direct subsidiary of the company (or one that failed).
    NotOwnSubsidiary,
    /// Within the group sites move by `TransferSite`, not by offers or contracts.
    WithinGroup,
    /// The data have no stock market (K1).
    NoStockMarket,
    NotListed,
    AlreadyListed,
    /// Too little equity to go public.
    EquityTooLow {
        min: Money,
    },
    /// A share outside (0, max].
    ShareOutOfRange {
        max: f64,
    },
    /// The player keeps the majority of the company bid for (K3).
    TakeoverNoMajority {
        held: f64,
    },
    NotEnoughFreeFloat {
        available: f64,
    },
    NotEnoughStock {
        held: f64,
    },
    /// The country is closed to the company (H1).
    CountryClosed,
    /// The data have no banks (K4).
    NoBanks,
    NotABank,
    InvalidBankSettings,
    /// The data have no bonds (K2).
    NoBonds,
    BondTerm {
        min: u32,
        max: u32,
    },
    BondTooSmall {
        min: Money,
    },
    /// Too little equity for investors to buy bonds.
    BondCompanyTooSmall {
        min: Money,
    },
    /// No investor buys a bond this large; `max` is the largest that sells.
    NoBondInvestors {
        max: Money,
    },
    UnknownBond,
    /// A move is under way until the date.
    RelocationUnderWay {
        until: Date,
    },
    /// No such central department in the data (ZA2).
    UnknownDepartment,
    /// Budget or limits below zero, or the readiness for risks beyond 0–1 (ZA2).
    InvalidParticipations,
    /// A new loan would not be cheaper (ZA3).
    NoAdvantage,
    /// No such start-up (SU2).
    UnknownVenture,
    /// The start-up has ended.
    VentureClosed,
    /// The start-up is another company's subsidiary.
    VentureOfOther,
    /// More than the round or the founders and investors offer.
    AmountTooHigh {
        max: Money,
    },
    /// The company holds less than it wants to sell.
    NotEnoughShares,
    /// Only for the majority owner.
    NoMajority,
    /// Another company holds a blocking minority.
    VentureBlocked,
    /// The research center works on nothing a start-up could aim at (SU3).
    NoSpinOff,
    /// The project has not got far enough yet.
    SpinOffTooEarly {
        progress: f64,
        min: f64,
    },
    /// The company offers no shares of the start-up (ZA4).
    NoStakeOffer,
    /// The manager does not retire within the warning period (PE1).
    NotRetiring,
    /// He may stay at most so many years longer.
    ExtensionTooLong {
        max: u8,
    },
    /// The person's children work only for the person's companies (PE2).
    FamilyOnly,
    /// Only the person gives this command (PE3).
    PersonOnly,
    /// The person has no company yet (PE3).
    NoCompany,
    /// The person already founded its company.
    AlreadyFounded,
    /// The private account holds less.
    NotEnoughPrivateMoney {
        needed: Money,
    },
    /// The person does not control the company.
    NotControlled,
    /// The lifestyle changed recently; the next change from this day on.
    LifestyleChangedRecently {
        from: crate::calendar::Date,
    },
    /// With co-owners the salary is limited.
    SalaryTooHigh {
        max: Money,
    },
    /// Interest or term of a loan of the person outside the bounds of the data.
    LoanTerms {
        max_rate: f64,
        max_years: u32,
    },
    /// More than the person paid in, the subscribed capital or the cash above the reserve.
    WithdrawalTooHigh {
        max: Money,
    },
    /// Only while the person holds all shares.
    NotSoleOwner,
    /// More than the retained earnings or the cash above the reserve (PE4).
    DividendTooHigh {
        max: Money,
    },
    /// The holder does not sell: the person, the buyer itself or a company of the person
    /// (PE5).
    NotSeller,
    /// Below the price the holder asks (PE5).
    PriceTooLow {
        min: Money,
    },
    /// No living child of the person with this index (PE6).
    NoSuchChild,
    /// The unit has no pollutants or already the best level available (H2).
    NoRetrofit,
    /// Antitrust forbids the takeover: the combined share of a market (H2).
    Antitrust {
        product: String,
        country: String,
        share: f64,
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
            CommandError::InvalidStrategy => e(keys::COMMAND_INVALID_STRATEGY),
            CommandError::InvalidMandate => e(keys::COMMAND_INVALID_MANDATE),
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
            CommandError::InvalidTraining => e(keys::COMMAND_INVALID_TRAINING),
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
            CommandError::UnknownManager => e(keys::COMMAND_UNKNOWN_MANAGER),
            CommandError::ManagerEmployed => e(keys::COMMAND_MANAGER_EMPLOYED),
            CommandError::NotYourManager => e(keys::COMMAND_NOT_YOUR_MANAGER),
            CommandError::UnknownPosition => e(keys::COMMAND_UNKNOWN_POSITION),
            CommandError::PositionTaken => e(keys::COMMAND_POSITION_TAKEN),
            CommandError::UnknownConcern => e(keys::COMMAND_UNKNOWN_CONCERN),
            CommandError::ConcernClosed => e(keys::COMMAND_CONCERN_CLOSED),
            CommandError::UnknownOption => e(keys::COMMAND_UNKNOWN_OPTION),
            CommandError::NotAHead => e(keys::COMMAND_NOT_A_HEAD),
            CommandError::SalaryNotHigher => e(keys::COMMAND_SALARY_NOT_HIGHER),
            CommandError::ManagerFree => e(keys::COMMAND_MANAGER_FREE),
            CommandError::OwnManager => e(keys::COMMAND_OWN_MANAGER),
            CommandError::ManagerHasOffer => e(keys::COMMAND_MANAGER_HAS_OFFER),
            CommandError::ManagerCourted { until } => {
                e(keys::COMMAND_MANAGER_COURTED).with("datum", Param::Date(*until))
            }
            CommandError::NoPoachOffer => e(keys::COMMAND_NO_POACH_OFFER),
            CommandError::UnknownCountry => e(keys::COMMAND_UNKNOWN_COUNTRY),
            CommandError::SameHeadquarters => e(keys::COMMAND_SAME_HEADQUARTERS),
            CommandError::UnknownCity => e(keys::COMMAND_UNKNOWN_CITY),
            CommandError::Embargo { from, to } => e(keys::COMMAND_EMBARGO)
                .with("von", Param::TextKey(format!("land.{from}")))
                .with("nach", Param::TextKey(format!("land.{to}"))),
            CommandError::NoContracts => e(keys::COMMAND_NO_CONTRACTS),
            CommandError::ContractWithItself => e(keys::COMMAND_CONTRACT_ITSELF),
            CommandError::UnknownContract => e(keys::COMMAND_UNKNOWN_CONTRACT),
            CommandError::ContractDeclined { reason } => e(keys::COMMAND_CONTRACT_DECLINED).with(
                "grund",
                Param::TextKey(format!("vertrag.abgelehnt.{reason}")),
            ),
            CommandError::NoLogistics => e(keys::COMMAND_NO_LOGISTICS),
            CommandError::VehicleNotForFleet => e(keys::COMMAND_VEHICLE_NOT_FOR_FLEET),
            CommandError::NoSubsidiaries => e(keys::COMMAND_NO_SUBSIDIARIES),
            CommandError::CapitalTooLow { min } => {
                e(keys::COMMAND_CAPITAL_TOO_LOW).with("min", Param::Money(*min))
            }
            CommandError::NotOwnSubsidiary => e(keys::COMMAND_NOT_OWN_SUBSIDIARY),
            CommandError::WithinGroup => e(keys::COMMAND_WITHIN_GROUP),
            CommandError::NoStockMarket => e(keys::COMMAND_NO_STOCK_MARKET),
            CommandError::NotListed => e(keys::COMMAND_NOT_LISTED),
            CommandError::AlreadyListed => e(keys::COMMAND_ALREADY_LISTED),
            CommandError::EquityTooLow { min } => {
                e(keys::COMMAND_EQUITY_TOO_LOW).with("min", Param::Money(*min))
            }
            CommandError::ShareOutOfRange { max } => e(keys::COMMAND_SHARE_OUT_OF_RANGE)
                .with("max", Param::Number((max * 1000.0).round() / 10.0)),
            CommandError::TakeoverNoMajority { held } => e(keys::COMMAND_TAKEOVER_NO_MAJORITY)
                .with("anteil", Param::Number((held * 1000.0).round() / 10.0)),
            CommandError::NotEnoughFreeFloat { available } => e(keys::COMMAND_FREE_FLOAT)
                .with("anteil", Param::Number((available * 1000.0).round() / 10.0)),
            CommandError::NotEnoughStock { held } => e(keys::COMMAND_NOT_ENOUGH_STOCK)
                .with("anteil", Param::Number((held * 1000.0).round() / 10.0)),
            CommandError::NoBanks => e(keys::COMMAND_NO_BANKS),
            CommandError::NotABank => e(keys::COMMAND_NOT_A_BANK),
            CommandError::CountryClosed => e(keys::COMMAND_COUNTRY_CLOSED),
            CommandError::InvalidBankSettings => e(keys::COMMAND_INVALID_BANK_SETTINGS),
            CommandError::NoBonds => e(keys::COMMAND_NO_BONDS),
            CommandError::BondTerm { min, max } => e(keys::COMMAND_BOND_TERM)
                .with("min", Param::Integer(i64::from(*min)))
                .with("max", Param::Integer(i64::from(*max))),
            CommandError::BondTooSmall { min } => {
                e(keys::COMMAND_BOND_TOO_SMALL).with("min", Param::Money(*min))
            }
            CommandError::BondCompanyTooSmall { min } => {
                e(keys::COMMAND_BOND_COMPANY_TOO_SMALL).with("min", Param::Money(*min))
            }
            CommandError::NoBondInvestors { max } => {
                e(keys::COMMAND_NO_BOND_INVESTORS).with("max", Param::Money(*max))
            }
            CommandError::UnknownBond => e(keys::COMMAND_UNKNOWN_BOND),
            CommandError::TooManyVehicles { count } => {
                e(keys::COMMAND_TOO_MANY_VEHICLES).with("anzahl", Param::Integer(i64::from(*count)))
            }
            CommandError::RelocationUnderWay { until } => {
                e(keys::COMMAND_RELOCATION_UNDER_WAY).with("datum", Param::Date(*until))
            }
            CommandError::UnknownDepartment => e(keys::COMMAND_UNKNOWN_DEPARTMENT),
            CommandError::InvalidParticipations => e(keys::COMMAND_INVALID_PARTICIPATIONS),
            CommandError::NoAdvantage => e(keys::COMMAND_NO_ADVANTAGE),
            CommandError::UnknownVenture => e(keys::COMMAND_UNKNOWN_VENTURE),
            CommandError::VentureClosed => e(keys::COMMAND_VENTURE_CLOSED),
            CommandError::VentureOfOther => e(keys::COMMAND_VENTURE_OF_OTHER),
            CommandError::AmountTooHigh { max } => {
                e(keys::COMMAND_AMOUNT_TOO_HIGH).with("max", Param::Money(*max))
            }
            CommandError::NotEnoughShares => e(keys::COMMAND_NOT_ENOUGH_SHARES),
            CommandError::NoMajority => e(keys::COMMAND_NO_MAJORITY),
            CommandError::VentureBlocked => e(keys::COMMAND_VENTURE_BLOCKED),
            CommandError::NoSpinOff => e(keys::COMMAND_NO_SPIN_OFF),
            CommandError::SpinOffTooEarly { progress, min } => e(keys::COMMAND_SPIN_OFF_TOO_EARLY)
                .with("fortschritt", Param::Number((*progress * 100.0).floor()))
                .with("mindestens", Param::Number((*min * 100.0).round())),
            CommandError::NoStakeOffer => e(keys::COMMAND_NO_STAKE_OFFER),
            CommandError::NotRetiring => e(keys::COMMAND_NOT_RETIRING),
            CommandError::FamilyOnly => e(keys::COMMAND_FAMILY_ONLY),
            CommandError::PersonOnly => e(keys::COMMAND_PERSON_ONLY),
            CommandError::NoCompany => e(keys::COMMAND_NO_COMPANY),
            CommandError::AlreadyFounded => e(keys::COMMAND_ALREADY_FOUNDED),
            CommandError::NotEnoughPrivateMoney { needed } => {
                e(keys::COMMAND_PRIVATE_MONEY).with("betrag", Param::Money(*needed))
            }
            CommandError::NotControlled => e(keys::COMMAND_NOT_CONTROLLED),
            CommandError::LifestyleChangedRecently { from } => {
                e(keys::COMMAND_LIFESTYLE_RECENT).with("datum", Param::Date(*from))
            }
            CommandError::SalaryTooHigh { max } => {
                e(keys::COMMAND_SALARY_TOO_HIGH).with("max", Param::Money(*max))
            }
            CommandError::LoanTerms {
                max_rate,
                max_years,
            } => e(keys::COMMAND_LOAN_TERMS)
                .with("zins", Param::Number((max_rate * 1000.0).round() / 10.0))
                .with("jahre", Param::Integer(i64::from(*max_years))),
            CommandError::WithdrawalTooHigh { max } => {
                e(keys::COMMAND_WITHDRAWAL_TOO_HIGH).with("max", Param::Money(*max))
            }
            CommandError::NotSoleOwner => e(keys::COMMAND_NOT_SOLE_OWNER),
            CommandError::NotSeller => e(keys::COMMAND_NOT_SELLER),
            CommandError::NoSuchChild => e(keys::COMMAND_NO_SUCH_CHILD),
            CommandError::NoRetrofit => e(keys::COMMAND_NO_RETROFIT),
            CommandError::Antitrust {
                product,
                country,
                share,
            } => e(keys::COMMAND_ANTITRUST)
                .with("produkt", Param::TextKey(format!("produkt.{product}")))
                .with("land", Param::Country(country.clone()))
                .with("anteil", Param::Number((share * 1000.0).round() / 10.0)),
            CommandError::PriceTooLow { min } => {
                e(keys::COMMAND_PRICE_TOO_LOW).with("min", Param::Money(*min))
            }
            CommandError::DividendTooHigh { max } => {
                e(keys::COMMAND_DIVIDEND_TOO_HIGH).with("max", Param::Money(*max))
            }
            CommandError::ExtensionTooLong { max } => {
                e(keys::COMMAND_EXTENSION_TOO_LONG).with("jahre", integer(usize::from(*max)))
            }
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
    if crate::events::closed_to(state, actor, country) {
        return Err(CommandError::CountryClosed);
    }
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
        training: 0.0,
        training_target: None,
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
    run(state, catalog, actor, command)?;
    crate::management::settle(state, actor, command);
    Ok(())
}

fn run(
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
        Command::HireManager { manager, position } => {
            crate::management::hire(state, catalog, actor, *manager, position)?;
        }
        Command::MoveManager { manager, position } => {
            crate::management::move_to(state, catalog, actor, *manager, position)?;
        }
        Command::DismissManager { manager } => {
            crate::management::dismiss(state, catalog, actor, *manager)?;
        }
        Command::AnswerConcern { concern, answer } => {
            crate::management::answer(state, catalog, actor, *concern, *answer)?;
        }
        Command::SetBudget { position, shares } => {
            crate::management::set_budget(state, catalog, actor, position, *shares)?;
        }
        Command::AskAgain { position, topic } => {
            crate::management::ask_again(state, catalog, actor, position, *topic)?;
        }
        Command::SetBudgetRule {
            kind,
            scope,
            shares,
        } => {
            crate::management::set_budget_rule(state, catalog, actor, kind, *scope, *shares)?;
        }
        Command::SetStrategy {
            scope,
            field,
            value,
        } => {
            crate::strategy::set(state, catalog, actor, (*scope, *field), *value)?;
        }
        Command::SetMandate { mandate } => {
            crate::mandate::set(state, catalog, actor, mandate)?;
        }
        Command::SetHiringByHead { position, enabled } => {
            crate::staffing::set_hiring(state, catalog, actor, position, *enabled)?;
        }
        Command::RaiseSalary { manager, salary } => {
            crate::staffing::raise_salary(state, actor, *manager, *salary)?;
        }
        Command::PoachManager { manager, position } => {
            crate::staffing::poach(state, catalog, actor, *manager, position)?;
        }
        Command::MatchOffer { manager } => {
            crate::staffing::match_offer(state, catalog, actor, *manager)?;
        }
        Command::LetGo { manager } => {
            crate::staffing::let_go(state, catalog, actor, *manager)?;
        }
        Command::AppointSuccessor { manager, position } => {
            crate::aging::appoint(state, catalog, actor, *manager, position)?;
        }
        Command::ExtendContract { manager, years } => {
            crate::aging::extend(state, catalog, actor, *manager, *years)?;
        }
        Command::LeaveVacant { position } => {
            crate::aging::leave_vacant(state, catalog, actor, position)?;
        }
        Command::SetHeadquarters { country, city } => {
            crate::central::set_headquarters(state, catalog, actor, (*country, city.as_deref()))?;
        }
        Command::StaffDepartment { department, staff } => {
            crate::central::staff_department(state, catalog, actor, *department, *staff)?;
        }
        Command::SetParticipations {
            budget,
            risk,
            limits,
        } => {
            crate::central::set_participations(state, catalog, actor, (*budget, *risk), limits)?;
        }
        Command::RefinanceLoan { loan } => {
            crate::central::refinance(state, catalog, actor, *loan)?;
        }
        Command::InvestInVenture { venture, amount } => {
            crate::ventures::invest(state, catalog, actor, *venture, *amount)?;
        }
        Command::GrantVenture { venture, amount } => {
            crate::ventures::grant(state, catalog, actor, *venture, *amount)?;
        }
        Command::SellVentureStake { venture, share } => {
            crate::ventures::sell(state, catalog, actor, *venture, *share)?;
        }
        Command::OfferVentureStake { venture, minimum } => {
            crate::ventures::offer_stake(state, actor, *venture, *minimum)?;
        }
        Command::BuyVentureStake {
            venture,
            seller,
            price,
        } => {
            crate::ventures::buy_stake(state, catalog, actor, *venture, (*seller, *price))?;
        }
        Command::SteerVenture { venture, pace } => {
            crate::ventures::steer(state, catalog, actor, *venture, *pace)?;
        }
        Command::IntegrateVenture { venture } => {
            crate::ventures::integrate(state, catalog, actor, *venture)?;
        }
        Command::FoundCompany { .. }
        | Command::SetPersonSalary { .. }
        | Command::SetLifestyle { .. }
        | Command::ContributeCapital { .. }
        | Command::LendToCompany { .. }
        | Command::WithdrawCapital { .. }
        | Command::BuyStake { .. }
        | Command::SellStake { .. }
        | Command::SelectCompany { .. }
        | Command::InvestPrivately { .. }
        | Command::SetHeir { .. }
        | Command::HandOver {} => return Err(CommandError::PersonOnly),
        Command::BidForStake {
            company,
            holder,
            share,
            price,
        } => crate::holdings::buy(
            state,
            catalog,
            crate::holdings::Buyer::Company(actor),
            (*company, *holder),
            (*share, *price),
        )?,
        Command::SpinOff { site, sell } => {
            crate::ventures::spin_off(state, catalog, actor, *site, *sell)?;
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
                    retrofit: 0,
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
            let rate = state
                .tariffs
                .for_product(catalog, from_country, to_country, *product)
                .ok_or_else(|| CommandError::Embargo {
                    from: catalog.countries.key(from_country).to_owned(),
                    to: catalog.countries.key(to_country).to_owned(),
                })?;
            let held = source.inventory.get(product);
            let available = held.map_or(0.0, |s| s.quantity);
            if *quantity > available + 1e-9 {
                return Err(CommandError::NotEnoughGoods { available });
            }
            // Customs on the value of the goods in stock (W3).
            let customs = held.map_or(Money::ZERO, |s| {
                s.value
                    .scale(rate * (*quantity / s.quantity.max(1e-12)).min(1.0))
            });
            // The way of the load (W5): market, state or own fleet.
            let plan = route.map(|(per_unit, _)| {
                crate::logistics::plan(
                    state,
                    catalog,
                    actor,
                    (*product, *quantity),
                    (from_country, to_country),
                    Money::times(per_unit, *quantity),
                )
            });
            let freight = if let (Some((_, days)), Some(plan)) = (route, &plan) {
                let cost = plan.cost;
                let company = state.company_mut(actor).expect("checked above");
                if company.ledger.cash() < cost + customs {
                    return Err(CommandError::NotEnoughCash {
                        needed: cost + customs,
                    });
                }
                company.ledger.expense(
                    CostType::Transport,
                    CostCenter::product(*from, *product),
                    Account::Cash,
                    cost,
                );
                if customs > Money::ZERO {
                    company.ledger.expense(
                        CostType::Customs,
                        CostCenter::product(*to, *product),
                        Account::Cash,
                        customs,
                    );
                }
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
            let lost = plan.is_some_and(|p| crate::logistics::book(state, catalog, actor, &p));
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
                    lost,
                }),
            }
        }
        Command::ProposeContract {
            seller,
            buyer,
            product,
            per_month,
            price,
            months,
            min_quality,
            penalty,
        } => {
            let terms = crate::contracts::Terms {
                seller: *seller,
                buyer: *buyer,
                product: *product,
                per_month: *per_month,
                price: *price,
                months: *months,
                min_quality: *min_quality,
                penalty: *penalty,
            };
            crate::contracts::propose(state, catalog, actor, &terms)?;
        }
        Command::BuyVehicles { vehicle, count } => {
            crate::logistics::buy(state, catalog, actor, *vehicle, *count)?;
        }
        Command::SellVehicles { vehicle, count } => {
            crate::logistics::sell(state, catalog, actor, *vehicle, *count)?;
        }
        Command::FoundSubsidiary {
            name,
            country,
            capital,
            focus,
        } => {
            crate::group::found(state, catalog, actor, (name, *country), *capital, *focus)?;
        }
        Command::MoveCapital { company, amount } => {
            crate::group::move_capital(state, actor, *company, *amount)?;
        }
        Command::TransferSite { site, to } => {
            crate::group::transfer_site(state, catalog, actor, *site, *to)?;
        }
        Command::SetSubsidiaryFocus { company, focus } => {
            crate::group::set_focus(state, catalog, actor, *company, *focus)?;
        }
        Command::GoPublic { share } => crate::stock::go_public(state, catalog, actor, *share)?,
        Command::SetBank { company, settings } => {
            crate::bank::set(state, catalog, actor, *company, *settings)?;
        }
        Command::IssueBond { amount, years } => {
            crate::bonds::issue(state, catalog, actor, *amount, *years)?;
        }
        Command::RedeemBond { bond } => crate::bonds::redeem(state, catalog, actor, *bond)?,
        Command::IssueShares { share } => {
            crate::stock::issue_shares(state, catalog, actor, *share)?;
        }
        Command::SetDividend { payout } => crate::dividends::set_policy(
            state,
            actor,
            crate::dividends::DividendPolicy::Share(*payout),
        )?,
        Command::SetDividendPolicy { policy } => {
            crate::dividends::set_policy(state, actor, *policy)?;
        }
        Command::SpecialDividend { amount } => {
            crate::dividends::special(state, catalog, actor, *amount)?;
        }
        Command::BuyShares { company, share } => {
            crate::stock::buy(state, catalog, actor, *company, *share)?;
        }
        Command::SellShares { company, share } => {
            crate::stock::sell(state, catalog, actor, *company, *share)?;
        }
        Command::TakeOver { company } => {
            crate::stock::take_over(state, catalog, actor, *company)?;
        }
        Command::BuyBackShares { share } => {
            crate::stock::buy_back(state, catalog, actor, *share)?;
        }
        Command::SetLogistics {
            mode,
            carry_for_others,
        } => {
            if !catalog.logistics.enabled {
                return Err(CommandError::NoLogistics);
            }
            let l = &mut state.company_mut(actor).expect("checked above").logistics;
            l.mode = *mode;
            l.carry_for_others = *carry_for_others;
        }
        Command::AnswerContract { contract, accept } => {
            crate::contracts::answer(state, actor, *contract, *accept)?;
        }
        Command::CancelContract { contract } => {
            crate::contracts::cancel(state, catalog, actor, *contract)?;
        }
        Command::TakeLoan { amount, years } => {
            if *amount <= Money::ZERO {
                return Err(CommandError::InvalidAmount);
            }
            let max = catalog.finance_model.max_term_years;
            if *years == 0 || *years > max {
                return Err(CommandError::InvalidTerm { max });
            }
            let cut = crate::central::premium_cut(catalog, state, actor);
            let company = state.company(actor).expect("checked above");
            let limit = crate::finance::credit_limit(catalog, company);
            if *amount > limit {
                return Err(CommandError::LoanTooLarge { limit });
            }
            // The market's banks, or a bank of the player that lends for less (K4); the
            // person's lifestyle moves the market's rate (PE3).
            let market = crate::finance::loan_rate(catalog, company, *amount, today, cut)
                + crate::private::loan_rate_offset(catalog, state, actor);
            let (rate, lender) =
                match crate::bank::lender_for(state, catalog, actor, *amount, market) {
                    Some((bank, rate)) => {
                        crate::bank::lend(state, bank, *amount);
                        (rate, Some(bank))
                    }
                    None => (market, None),
                };
            let company = state.company_mut(actor).expect("checked above");
            crate::finance::grant_loan(company, (*amount, *years), today, (rate, lender));
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
            let before = company.loans.len();
            let from_person = company.loans[*loan].from_person;
            let lender = crate::finance::repay(company, *loan, amount);
            let paid_off = company.loans.len() < before;
            if let Some((bank, amount)) = lender {
                crate::bank::receive(state, bank, amount, Money::ZERO);
            }
            if from_person {
                crate::private::receive_loan(state, amount, Money::ZERO);
            }
            if paid_off {
                crate::management::settle_topic(state, actor, crate::decision::Topic::Refinance);
            }
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
        Command::SetTraining { site, target } => {
            if target.is_some_and(|t| !(t.is_finite() && (0.0..=1.0).contains(&t))) {
                return Err(CommandError::InvalidTraining);
            }
            own_site(state, actor, *site)?;
            state
                .site_mut(*site)
                .expect("checked above")
                .training_target = *target;
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
        Command::Retrofit { site, slot } => {
            crate::regulation::retrofit(state, catalog, actor, (*site, *slot))?;
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
