//! The game state: everything that changes during a game and is saved.
//!
//! Catalog IDs in the state are saved as keys (see `ids`), so saves survive changes of
//! the data files without any manual translation.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::marker::PhantomData;

use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::calendar::Date;
use crate::catalog::{Catalog, SiteType};
pub use crate::country_model::CountryState;
use crate::ids::{
    self, CountryId, DepositId, FacilityId, GoodsGroupId, Id, LaborGroupId, ProductId, RecipeId,
    TechnologyId,
};
use crate::ledger::Ledger;
use crate::money::Money;
use crate::policy::SalesPolicy;
use crate::rng::SimRng;
use crate::transport::Routes;

/// Per-entry state for a catalog table, indexed by the table's IDs.
#[derive(Debug)]
pub struct PerId<I, T> {
    items: Vec<T>,
    _id: PhantomData<fn() -> I>,
}

impl<I, T: Clone> Clone for PerId<I, T> {
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
            _id: PhantomData,
        }
    }
}

impl<I, T: PartialEq> PartialEq for PerId<I, T> {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

impl<I, T> Default for PerId<I, T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            _id: PhantomData,
        }
    }
}

impl<I: Id, T> PerId<I, T> {
    pub fn from_fn(len: usize, mut f: impl FnMut(I) -> T) -> Self {
        Self {
            items: (0..len).map(|i| f(I::from_index(i))).collect(),
            _id: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn get(&self, id: I) -> &T {
        &self.items[id.index()]
    }

    pub fn get_mut(&mut self, id: I) -> &mut T {
        &mut self.items[id.index()]
    }

    pub fn iter(&self) -> impl Iterator<Item = (I, &T)> {
        self.items
            .iter()
            .enumerate()
            .map(|(i, t)| (I::from_index(i), t))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (I, &mut T)> {
        self.items
            .iter_mut()
            .enumerate()
            .map(|(i, t)| (I::from_index(i), t))
    }

    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }

    /// Grows or shrinks to `len` entries; new entries come from `f`.
    pub fn resize_with(&mut self, len: usize, f: impl FnMut() -> T) {
        self.items.resize_with(len, f);
    }
}

/// As a list normally; as a map from key to entry while saving, so the entries find
/// their place again when the catalog order changes.
impl<I: Id + Serialize, T: Serialize> Serialize for PerId<I, T> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        if ids::keys_active() {
            let mut map = s.serialize_map(Some(self.items.len()))?;
            for (i, item) in self.items.iter().enumerate() {
                map.serialize_entry(&I::from_index(i), item)?;
            }
            map.end()
        } else {
            self.items.serialize(s)
        }
    }
}

impl<'de, I: Id + Deserialize<'de>, T: Deserialize<'de> + Default> Deserialize<'de>
    for PerId<I, T>
{
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct PerIdVisitor<I, T>(PhantomData<(I, T)>);

        impl<'de, I: Id + Deserialize<'de>, T: Deserialize<'de> + Default> Visitor<'de>
            for PerIdVisitor<I, T>
        {
            type Value = PerId<I, T>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a list or a map of entries")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut items = Vec::new();
                while let Some(item) = seq.next_element()? {
                    items.push(item);
                }
                Ok(PerId {
                    items,
                    _id: PhantomData,
                })
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut entries: Vec<(I, T)> = Vec::new();
                while let Some(entry) = map.next_entry()? {
                    entries.push(entry);
                }
                let len = ids::active_len(I::KIND).unwrap_or_else(|| {
                    entries
                        .iter()
                        .map(|(i, _)| i.index() + 1)
                        .max()
                        .unwrap_or(0)
                });
                let mut items: Vec<T> = (0..len).map(|_| T::default()).collect();
                for (id, item) in entries {
                    if let Some(slot) = items.get_mut(id.index()) {
                        *slot = item;
                    }
                }
                Ok(PerId {
                    items,
                    _id: PhantomData,
                })
            }
        }

        d.deserialize_any(PerIdVisitor(PhantomData))
    }
}

/// Companies are numbered in the order they are founded; IDs are never reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CompanyId(pub u32);

impl CompanyId {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// Sites are numbered in the order they are founded; IDs are never reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SiteId(pub u32);

impl SiteId {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StartForm {
    /// Small workshop that makes simple parts.
    Workshop,
    /// Small trading business.
    Trading,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompanyKind {
    Player,
    Ai,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Company {
    pub name: String,
    pub kind: CompanyKind,
    /// Country of the head office; decides taxes (Lastenheft §5.1).
    pub headquarters: CountryId,
    pub founded: Date,
    /// Own random stream for the company's decisions.
    pub rng: SimRng,
    pub ledger: Ledger,
    /// Technologies the company acquired after the start (research, later licences).
    pub technologies: BTreeSet<TechnologyId>,
    /// Bankrupt companies keep their history but no longer act.
    pub bankrupt: bool,
    #[serde(default)]
    pub loans: Vec<Loan>,
    /// Losses of earlier years that reduce future taxable profits.
    #[serde(default)]
    pub loss_carryforward: Money,
    /// Who may buy the company's goods (Lastenheft §9.2).
    #[serde(default)]
    pub sales_policies: Vec<SalesPolicy>,
    /// Research points collected per technology not yet acquired.
    #[serde(default)]
    pub research: BTreeMap<TechnologyId, f64>,
    /// Character and plans of an AI company (`None` for the player).
    #[serde(default)]
    pub ai: Option<AiState>,
    /// Shareholders (Lastenheft §11, §17.3); the shares add up to 1. Filled after loading
    /// older saves (`fit_to_catalog`).
    #[serde(default)]
    pub owners: Vec<Stake>,
    /// Brand awareness per country and goods group (M16).
    #[serde(default)]
    pub brands: Vec<Brand>,
    /// Advertising budgets per month (M16).
    #[serde(default)]
    pub advertising: Vec<Advertising>,
}

/// How well consumers in a country know a company's brand for a goods group (0–1).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Brand {
    pub country: CountryId,
    pub group: GoodsGroupId,
    pub awareness: f64,
}

/// Monthly advertising budget of a company in a country for a goods group.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Advertising {
    pub country: CountryId,
    pub group: GoodsGroupId,
    pub budget: Money,
}

/// Who holds shares of a company. The player is an owner, not a company, so that later
/// stages can let the player act as investor or bank and hold subsidiaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Holder {
    Player,
    Company(CompanyId),
    /// Founders, families and small shareholders.
    Private,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stake {
    pub holder: Holder,
    /// Share of the company (0–1).
    pub share: f64,
}

impl Stake {
    pub fn sole(holder: Holder) -> Vec<Stake> {
        vec![Stake { holder, share: 1.0 }]
    }
}

impl Company {
    /// Brand awareness in a country for a goods group (0 if unknown).
    pub fn awareness(&self, country: CountryId, group: GoodsGroupId) -> f64 {
        self.brands
            .iter()
            .find(|b| b.country == country && b.group == group)
            .map_or(0.0, |b| b.awareness)
    }

    /// The holder with more than half of the shares, if any.
    pub fn majority_holder(&self) -> Option<Holder> {
        let mut shares: Vec<(Holder, f64)> = Vec::new();
        for s in &self.owners {
            match shares.iter_mut().find(|(h, _)| *h == s.holder) {
                Some((_, v)) => *v += s.share,
                None => shares.push((s.holder, s.share)),
            }
        }
        shares.into_iter().find(|&(_, v)| v > 0.5).map(|(h, _)| h)
    }
}

/// What distinguishes one AI company from another (Lastenheft §10).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AiState {
    /// 0 = clumsy, 1 = very competent.
    pub competence: f64,
    /// 0 = cautious, 1 = aggressive.
    pub aggressiveness: f64,
    /// Key of the historical company it stands for.
    pub real: Option<String>,
    /// Day of the next operating decisions.
    pub next_operations: Date,
}

/// A bank loan, repaid in equal monthly instalments (annuity).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Loan {
    pub principal: Money,
    /// Outstanding amount.
    pub balance: Money,
    /// Real interest rate per year.
    pub rate: f64,
    pub start: Date,
    pub months: u32,
    pub instalment: Money,
}

/// Goods of one kind in a site's warehouse, valued at production or purchase cost.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Stock {
    pub quantity: f64,
    pub value: Money,
    /// Average quality 0–100.
    pub quality: f64,
}

impl Stock {
    pub fn add(&mut self, quantity: f64, value: Money, quality: f64) {
        let total = self.quantity + quantity;
        if total > 0.0 {
            self.quality = (self.quality * self.quantity + quality * quantity) / total;
        }
        self.quantity = total;
        self.value += value;
    }

    /// Removes `quantity` (at most what is there); returns its value at average cost.
    pub fn take(&mut self, quantity: f64) -> Money {
        let quantity = quantity.min(self.quantity).max(0.0);
        if quantity >= self.quantity {
            let value = self.value;
            *self = Stock {
                quality: self.quality,
                ..Stock::default()
            };
            return value;
        }
        let value = self.value.scale(quantity / self.quantity);
        self.quantity -= quantity;
        self.value -= value;
        value
    }
}

/// Goods in production that leave the facility on `finish`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Batch {
    pub finish: Date,
    pub outputs: Vec<(ProductId, f64)>,
    pub quality: f64,
    /// Production cost, distributed over the outputs by quantity.
    pub value: Money,
}

/// A facility built at a site.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Slot {
    pub facility: FacilityId,
    /// Day the construction is finished; until then the facility does not produce.
    pub ready: Date,
    /// Identical units of the facility working together (capacity, staff and cost × count).
    #[serde(default = "one_unit")]
    pub count: u32,
    /// Investment including automation upgrades (basis of depreciation and maintenance).
    pub cost: Money,
    pub recipe: Option<RecipeId>,
    /// Planned share of the capacity, 0–1.
    pub utilization: f64,
    /// Degree of automation, 0 to the facility's maximum.
    pub automation: f64,
    /// Condition of the equipment, 1 = new.
    pub condition: f64,
    pub batches: Vec<Batch>,
    /// Runs on the last production day (for reports).
    pub last_runs: f64,
    /// What held the facility below its plan on the last production day.
    #[serde(default)]
    pub limit: Option<Limit>,
}

/// What kept a facility below its planned production (shown as cause in the UI).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Limit {
    Input(ProductId),
    Labor(LaborGroupId),
    Electricity,
    Deposit,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Site {
    pub owner: CompanyId,
    pub country: CountryId,
    pub kind: SiteType,
    pub founded: Date,
    /// Cost of land and buildings (depreciated over the building lifetime).
    pub building_cost: Money,
    /// Deposit worked by this site (extraction sites only).
    pub deposit: Option<DepositId>,
    pub slots: Vec<Slot>,
    pub inventory: BTreeMap<ProductId, Stock>,
    /// Employed persons per labor group.
    pub workforce: PerId<LaborGroupId, f64>,
    /// Staffing is adjusted at the start of each month and after changes.
    pub staffing_due: bool,
    /// Goods offered for sale on the country's market.
    #[serde(default)]
    pub offers: BTreeMap<ProductId, SaleOffer>,
    /// Goods bought on the country's market to keep a stock.
    #[serde(default)]
    pub orders: BTreeMap<ProductId, PurchaseOrder>,
    /// Technology the research center works on.
    #[serde(default)]
    pub research: Option<TechnologyId>,
    /// Premium over the country's wages (M18, 0.1 = 10 %).
    #[serde(default)]
    pub wage_premium: f64,
}

/// How a sale offer is priced.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum PriceMode {
    Fixed(Money),
    /// Follows supply and demand, never below `floor`; starts at the market price
    /// plus `markup` (e.g. 0.1 = 10 % above).
    Market {
        markup: f64,
        floor: Money,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SaleOffer {
    pub mode: PriceMode,
    /// Current asking price per unit.
    pub price: Money,
    /// Stock kept back (e.g. for own production).
    pub keep: f64,
    pub sold_today: f64,
    pub sold_month: f64,
    /// Sold in the last closed month (for the views).
    #[serde(default)]
    pub sold_last_month: f64,
    /// Sold to traders and to other companies in the running month (for the limits of
    /// the sales policies).
    #[serde(default)]
    pub to_traders_month: f64,
    #[serde(default)]
    pub to_companies_month: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PurchaseOrder {
    /// Stock the site wants to have.
    pub target: f64,
    pub max_price: Money,
    pub min_quality: f64,
    pub bought_month: f64,
    /// Bought in the last closed month (for the views).
    #[serde(default)]
    pub bought_last_month: f64,
}

/// Trade on a market in a period.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Trade {
    pub demand: f64,
    pub sold: f64,
    pub revenue: Money,
    /// Sold from traders' imports.
    #[serde(default)]
    pub imported: f64,
    /// Bought by traders for export.
    #[serde(default)]
    pub exported: f64,
    /// Demand of consumers and government, without companies' purchase orders.
    #[serde(default)]
    pub outside_demand: f64,
    /// Of that, served (by companies, traders or the state market).
    #[serde(default)]
    pub outside_sold: f64,
}

impl Trade {
    pub fn unmet(&self) -> f64 {
        (self.demand - self.sold).max(0.0)
    }

    fn add(&mut self, other: &Trade) {
        self.demand += other.demand;
        self.sold += other.sold;
        self.revenue += other.revenue;
        self.imported += other.imported;
        self.exported += other.exported;
        self.outside_demand += other.outside_demand;
        self.outside_sold += other.outside_sold;
    }
}

/// Market of one product in one country.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Market {
    /// Price index (smoothed average of sales); zero until the market starts.
    pub price: Money,
    /// Consumer demand per day by income fifth (poorest first), set monthly.
    pub consumer_rate: [f64; 5],
    /// Government demand per day.
    pub state_rate: f64,
    /// Units owned per inhabitant by income fifth (durables).
    pub ownership: [f64; 5],
    /// Consumer purchases in the running month by income fifth.
    pub bought: [f64; 5],
    pub today: Trade,
    pub month: Trade,
    pub last_month: Trade,
    /// Goods imported by traders and offered here.
    #[serde(default)]
    pub imports: Stock,
    /// Asking price of the traders.
    #[serde(default)]
    pub import_price: Money,
    /// Demand per day that companies in the country do not serve (smoothed); traders
    /// import to cover it.
    #[serde(default)]
    pub open_demand: f64,
    /// First day without any seller or buyer that is not booked yet (see
    /// `market::settle_idle`).
    #[serde(default)]
    pub idle_since: Option<Date>,
}

impl Market {
    pub(crate) fn record_day(&mut self, day: Trade) {
        self.today = day;
        self.month.add(&day);
    }

    pub(crate) fn close_month(&mut self) {
        self.last_month = std::mem::take(&mut self.month);
    }
}

/// Goods on the way between countries.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shipment {
    pub product: ProductId,
    pub quantity: f64,
    pub quality: f64,
    /// Inventory value for companies; purchase price plus transport for traders.
    pub value: Money,
    pub from: CountryId,
    pub to: Consignee,
    pub arrival: Date,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Consignee {
    /// An own site of the sending company.
    Site(SiteId),
    /// The traders' stock on the market of a country.
    Importer(CountryId),
}

/// State of a deposit.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DepositState {
    /// Quantity extracted so far from the whole deposit (the reserve is shared).
    pub extracted: f64,
    /// Fields that companies develop and work separately (docs/FORMELN.md, M10).
    #[serde(default)]
    pub concessions: Vec<Concession>,
    // Before M10 a deposit had a single site. Read from old saves and moved into the
    // first concession by `fit_to_catalog`.
    #[serde(default, rename = "site", skip_serializing_if = "Option::is_none")]
    legacy_site: Option<SiteId>,
    #[serde(default, rename = "ready", skip_serializing_if = "Option::is_none")]
    legacy_ready: Option<Date>,
    #[serde(
        default,
        rename = "development_cost",
        skip_serializing_if = "Option::is_none"
    )]
    legacy_development_cost: Option<Money>,
    #[serde(
        default,
        rename = "extracted_this_year",
        skip_serializing_if = "Option::is_none"
    )]
    legacy_extracted_this_year: Option<f64>,
}

/// A field of a deposit that one site develops and works.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Concession {
    pub site: Option<SiteId>,
    /// Day the development is finished.
    pub ready: Option<Date>,
    pub development_cost: Money,
    pub extracted_this_year: f64,
    /// Share of the deposit's yearly output (and development cost) of this field.
    pub share: f64,
}

impl DepositState {
    /// Free fields of a deposit at the given market scale.
    pub fn new(count: u32) -> Self {
        let share = 1.0 / f64::from(count.max(1));
        Self {
            concessions: (0..count.max(1))
                .map(|_| Concession {
                    share,
                    ..Concession::default()
                })
                .collect(),
            ..Self::default()
        }
    }

    pub fn concession_of(&self, site: SiteId) -> Option<&Concession> {
        self.concessions.iter().find(|c| c.site == Some(site))
    }

    pub fn concession_of_mut(&mut self, site: SiteId) -> Option<&mut Concession> {
        self.concessions.iter_mut().find(|c| c.site == Some(site))
    }

    /// Moves the single site of a save from before M10 into one concession that keeps
    /// the whole deposit, so old games behave as before.
    fn migrate_legacy(&mut self) {
        if let Some(site) = self.legacy_site.take() {
            self.concessions = vec![Concession {
                site: Some(site),
                ready: self.legacy_ready.take(),
                development_cost: self.legacy_development_cost.take().unwrap_or(Money::ZERO),
                extracted_this_year: self.legacy_extracted_this_year.take().unwrap_or(0.0),
                share: 1.0,
            }];
        }
        self.legacy_ready = None;
        self.legacy_development_cost = None;
        self.legacy_extracted_this_year = None;
    }
}

/// Settings chosen when starting a game (Lastenheft §15). Together with the journal
/// of decisions they determine the whole game.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GameSettings {
    pub seed: u64,
    pub start_year: i32,
    /// Country of the head office.
    pub start_country: CountryId,
    pub start_capital: Money,
    pub start_form: StartForm,
    pub company_name: String,
    /// Multiplies the cost escalation of research ahead of history (Lastenheft §15).
    #[serde(default = "one")]
    pub research_ahead_factor: f64,
    /// Share of the real world's quantities the markets work with (docs/FORMELN.md, M10).
    #[serde(default = "one")]
    pub market_scale: f64,
    /// AI competitors (Lastenheft §10, §15).
    #[serde(default)]
    pub ai: AiSettings,
}

/// Number and character of the AI companies; difficulty presets fill these values.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct AiSettings {
    pub companies: u32,
    /// 0 = clumsy, 1 = very competent.
    pub competence: f64,
    /// 0 = cautious, 1 = aggressive.
    pub aggressiveness: f64,
}

impl Default for AiSettings {
    /// Games from before M10 have no AI companies.
    fn default() -> Self {
        Self {
            companies: 0,
            competence: 0.5,
            aggressiveness: 0.5,
        }
    }
}

fn one() -> f64 {
    1.0
}

fn one_unit() -> u32 {
    1
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GameState {
    pub settings: GameSettings,
    /// The current day, not yet simulated.
    pub date: Date,
    /// Random stream for world-level processes.
    pub world_rng: SimRng,
    /// Derived from catalog and date; not saved, recomputed after loading.
    #[serde(skip)]
    pub countries: PerId<CountryId, CountryState>,
    /// Indexed by [`CompanyId`].
    pub companies: Vec<Company>,
    /// Indexed by [`SiteId`].
    #[serde(default)]
    pub sites: Vec<Site>,
    #[serde(default)]
    pub deposits: PerId<DepositId, DepositState>,
    /// Day each technology was first acquired by research in this game.
    #[serde(default)]
    pub inventions: PerId<TechnologyId, Option<Date>>,
    /// Markets by product and country.
    #[serde(default)]
    pub markets: PerId<ProductId, PerId<CountryId, Market>>,
    /// Goods on the way, in the order they were sent.
    #[serde(default)]
    pub shipments: Vec<Shipment>,
    /// Cheapest transport routes of the current year; derived, not saved.
    #[serde(skip)]
    pub routes: Routes,
    /// Markets where traders hold imported goods; derived, not saved.
    #[serde(skip)]
    pub import_markets: BTreeSet<(ProductId, CountryId)>,
    pub player: CompanyId,
    pub game_over: bool,
}

impl GameState {
    pub fn company(&self, id: CompanyId) -> Option<&Company> {
        self.companies.get(id.index())
    }

    pub fn company_mut(&mut self, id: CompanyId) -> Option<&mut Company> {
        self.companies.get_mut(id.index())
    }

    pub fn site(&self, id: SiteId) -> Option<&Site> {
        self.sites.get(id.index())
    }

    pub fn site_mut(&mut self, id: SiteId) -> Option<&mut Site> {
        self.sites.get_mut(id.index())
    }

    /// Whether a company may use a technology: known to everyone at the start of the
    /// game or common knowledge by now (docs/FORMELN.md, M9), or acquired by the company.
    pub fn knows(&self, catalog: &Catalog, company: CompanyId, technology: TechnologyId) -> bool {
        let year = catalog.technologies.get(technology).invention_year;
        year <= self.settings.start_year
            || year.saturating_add(catalog.research_model.public_domain_years) <= self.date.year()
            || self
                .company(company)
                .is_some_and(|c| c.technologies.contains(&technology))
    }

    /// Recomputes the derived country values. They change monthly: the values of the
    /// first day of the current month apply to the whole month.
    pub(crate) fn refresh_countries(&mut self, catalog: &Catalog) {
        let month = self.date.first_of_month();
        let mut values = crate::country_model::compute_all(catalog, month).into_iter();
        let scale = self.settings.market_scale;
        let min_pool = catalog.ai_model.min_labor_pool;
        self.countries = PerId::from_fn(catalog.countries.len(), |_| {
            let mut c = values.next().expect("one per country");
            crate::country_model::apply_market_scale(&mut c, scale, min_pool);
            c
        });
        if self.routes.year() != self.date.year() {
            self.routes = Routes::new(catalog, self.date.year(), Some(&self.routes));
        }
    }

    /// Fits per-entry state to the catalog after loading: new deposits and labor
    /// groups get empty entries, derived values are recomputed.
    pub(crate) fn fit_to_catalog(&mut self, catalog: &Catalog) {
        for (i, company) in self.companies.iter_mut().enumerate() {
            company.ledger.fit_accounts();
            if company.owners.is_empty() {
                let holder = if i == self.player.index() {
                    Holder::Player
                } else {
                    Holder::Private
                };
                company.owners = Stake::sole(holder);
            }
        }
        self.deposits
            .resize_with(catalog.deposits.len(), DepositState::default);
        let scale = self.settings.market_scale;
        for (id, d) in self.deposits.iter_mut() {
            d.migrate_legacy();
            if d.concessions.is_empty() {
                *d = DepositState {
                    extracted: d.extracted,
                    ..DepositState::new(crate::population::concession_count(catalog, id, scale))
                };
            }
        }
        self.inventions
            .resize_with(catalog.technologies.len(), || None);
        let countries = catalog.countries.len();
        self.markets
            .resize_with(catalog.products.len(), PerId::default);
        for (_, markets) in self.markets.iter_mut() {
            markets.resize_with(countries, Market::default);
        }
        let groups = catalog.labor_groups.len();
        for site in &mut self.sites {
            site.workforce.resize_with(groups, || 0.0);
        }
        self.import_markets = self
            .markets
            .iter()
            .flat_map(|(p, markets)| {
                markets
                    .iter()
                    .filter(|(_, m)| m.imports.quantity > 1e-9)
                    .map(move |(c, _)| (p, c))
            })
            .collect();
        self.refresh_countries(catalog);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stock_keeps_value_and_quality() {
        let mut stock = Stock::default();
        stock.add(10.0, Money::from_usd(100.0).unwrap(), 40.0);
        stock.add(10.0, Money::from_usd(300.0).unwrap(), 60.0);
        assert_eq!(stock.quality, 50.0);
        assert_eq!(stock.take(5.0), Money::from_usd(100.0).unwrap());
        assert_eq!(stock.value, Money::from_usd(300.0).unwrap());
        // Taking more than there is empties the stock exactly.
        assert_eq!(stock.take(100.0), Money::from_usd(300.0).unwrap());
        assert_eq!((stock.quantity, stock.value), (0.0, Money::ZERO));
    }
}
