//! Regulation and environment (H2, docs/FORMELN.md): emissions, CO2 price, retrofitting,
//! bans, safety rules raising wages, antitrust, the environmental image.

use crate::calendar::Date;
use crate::catalog::{Catalog, Recipe, RegulationKind};
use crate::command::CommandError;
use crate::ids::{CountryId, Id, ProductId};
use crate::ledger::Account;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::state::{CompanyId, GameState, SiteId};
use crate::strategy::{StrategyField, StrategyValue};

/// The rules in force in a country in a month.
#[derive(Clone, Debug, PartialEq)]
pub struct CountryRules {
    /// Retrofit level units with pollutants need now.
    pub required: u32,
    /// Highest level announced, and when it will be required.
    pub announced: u32,
    pub deadline: Option<Date>,
    /// Factor on the hourly wages.
    pub wage_factor: f64,
    pub no_production: Vec<ProductId>,
    pub no_sales: Vec<ProductId>,
    pub antitrust: bool,
}

impl Default for CountryRules {
    fn default() -> Self {
        CountryRules {
            required: 0,
            announced: 0,
            deadline: None,
            wage_factor: 1.0,
            no_production: Vec::new(),
            no_sales: Vec::new(),
            antitrust: false,
        }
    }
}

/// The regulations of the current month by country; derived, not saved.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RegulationTable {
    month: Option<Date>,
    countries: Vec<CountryRules>,
}

/// Whether a regulation is in force in `month`: from the month of its date.
pub fn in_force(date: Date, month: Date) -> bool {
    date.first_of_month() <= month
}

impl RegulationTable {
    pub fn new(catalog: &Catalog, month: Date) -> Self {
        let mut countries = vec![CountryRules::default(); catalog.countries.len()];
        for r in catalog
            .regulations
            .iter()
            .filter(|r| in_force(r.date, month))
        {
            for &c in &r.countries {
                let rules = &mut countries[c.index()];
                match &r.kind {
                    RegulationKind::Retrofit { level, months } => {
                        let due = r.date.add_months(*months).first_of_month();
                        if due <= month {
                            rules.required = rules.required.max(*level);
                        }
                        if *level > rules.announced {
                            rules.announced = *level;
                            rules.deadline = Some(due);
                        }
                    }
                    RegulationKind::Safety { wage_surcharge } => {
                        rules.wage_factor *= 1.0 + wage_surcharge;
                    }
                    RegulationKind::Ban {
                        products,
                        production,
                        sales,
                    } => {
                        if *production {
                            rules.no_production.extend(products);
                        }
                        if *sales {
                            rules.no_sales.extend(products);
                        }
                    }
                    RegulationKind::Antitrust => rules.antitrust = true,
                }
            }
        }
        for rules in &mut countries {
            if rules.announced <= rules.required {
                rules.deadline = None;
            }
            rules.no_production.sort();
            rules.no_production.dedup();
            rules.no_sales.sort();
            rules.no_sales.dedup();
        }
        RegulationTable {
            month: Some(month),
            countries,
        }
    }

    pub fn month(&self) -> Option<Date> {
        self.month
    }

    /// The rules of a country; none before the table was built.
    pub fn country(&self, country: CountryId) -> CountryRules {
        self.countries
            .get(country.index())
            .cloned()
            .unwrap_or_default()
    }

    pub fn production_banned(&self, country: CountryId, product: ProductId) -> bool {
        self.countries
            .get(country.index())
            .is_some_and(|r| r.no_production.binary_search(&product).is_ok())
    }

    pub fn sales_banned(&self, country: CountryId, product: ProductId) -> bool {
        self.countries
            .get(country.index())
            .is_some_and(|r| r.no_sales.binary_search(&product).is_ok())
    }

    pub fn wage_factor(&self, country: CountryId) -> f64 {
        self.countries
            .get(country.index())
            .map_or(1.0, |r| r.wage_factor)
    }

    pub fn required(&self, country: CountryId) -> u32 {
        self.countries
            .get(country.index())
            .map_or(0, |r| r.required)
    }

    fn announced(&self, country: CountryId) -> u32 {
        self.countries
            .get(country.index())
            .map_or(0, |r| r.announced)
    }

    fn antitrust(&self, country: CountryId) -> bool {
        self.countries
            .get(country.index())
            .is_some_and(|r| r.antitrust)
    }
}

/// Pollutants per run of a recipe in a unit retrofitted to `level`.
pub fn pollutant(catalog: &Catalog, recipe: &Recipe, level: u32) -> f64 {
    catalog
        .environment
        .retrofit
        .iter()
        .take(level as usize)
        .fold(recipe.pollutant_kg, |p, l| p * (1.0 - l.reduction))
}

/// CO2 price in USD per t in a country today.
pub fn co2_price(catalog: &Catalog, country: CountryId, date: Date) -> f64 {
    catalog
        .environment
        .co2_price
        .value(country, date.year_fraction())
        .max(0.0)
}

/// The highest retrofit level available in a year.
pub fn available(catalog: &Catalog, year: i32) -> u32 {
    let n = catalog
        .environment
        .retrofit
        .iter()
        .filter(|l| l.from_year <= year)
        .count();
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// What the next retrofit level of a unit costs; `None` if there is none to take.
pub fn retrofit_cost(
    catalog: &Catalog,
    state: &GameState,
    site: SiteId,
    slot: usize,
) -> Option<Money> {
    let s = state.sites.get(site.index())?;
    let sl = s.slots.get(slot)?;
    let recipe = catalog.recipes.get(sl.recipe?);
    if recipe.pollutant_kg <= 0.0 || sl.retrofit >= available(catalog, state.date.year()) {
        return None;
    }
    let level = catalog.environment.retrofit.get(sl.retrofit as usize)?;
    Some(sl.cost.scale(level.cost_share))
}

/// `Retrofit`: the unit takes its next level, paid as an investment.
pub(crate) fn retrofit(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    (site, slot): (SiteId, usize),
) -> Result<Money, CommandError> {
    let s = state
        .sites
        .get(site.index())
        .ok_or(CommandError::UnknownSite)?;
    if s.owner != actor {
        return Err(CommandError::NotOwner);
    }
    if slot >= s.slots.len() {
        return Err(CommandError::UnknownSlot);
    }
    let cost = retrofit_cost(catalog, state, site, slot).ok_or(CommandError::NoRetrofit)?;
    let ledger = &mut state.companies[actor.index()].ledger;
    if ledger.cash() < cost {
        return Err(CommandError::NotEnoughCash { needed: cost });
    }
    ledger.transfer(Account::FixedAssets, Account::Cash, cost);
    let sl = &mut state.sites[site.index()].slots[slot];
    sl.cost += cost;
    sl.retrofit += 1;
    Ok(cost)
}

/// Whether the company's strategy asks to overfulfil the environmental rules at a site.
fn overfulfils(catalog: &Catalog, state: &GameState, company: CompanyId, site: SiteId) -> bool {
    let c = &state.companies[company.index()];
    if c.strategies.is_empty() {
        return false;
    }
    matches!(
        crate::strategy::setting(
            catalog,
            state,
            c,
            crate::state::Unit::Site(site),
            StrategyField::Environment
        )
        .map(|s| s.value),
        Some(StrategyValue::Environment(true))
    )
}

/// At a month start: the emissions of the closed month move to the previous month; news
/// of regulations coming into force where the main company works; the sites retrofit
/// what the rules (or the strategy) ask for, as far as the cash goes.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    for c in &mut state.companies {
        c.emissions_last = std::mem::take(&mut c.emissions);
    }
    if let Some(main) = state.main_company {
        let home: Vec<CountryId> = state
            .sites
            .iter()
            .filter(|s| s.owner == main)
            .map(|s| s.country)
            .chain(state.company(main).map(|c| c.headquarters))
            .collect();
        for r in catalog
            .regulations
            .iter()
            .filter(|r| r.date.first_of_month() == date)
            .filter(|r| r.countries.iter().any(|c| home.contains(c)))
        {
            let mut m = Message::new(MessageKind::Warning, keys::REGULATION_NEW)
                .with("regel", Param::TextKey(format!("regulierung.{}", r.key)))
                .with(
                    "laender",
                    Param::Countries(
                        r.countries
                            .iter()
                            .map(|&c| catalog.countries.key(c).to_owned())
                            .collect(),
                    ),
                );
            m = match &r.kind {
                RegulationKind::Retrofit { level, months } => m
                    .with("art", Param::TextKey("regulierungsart.auflage".into()))
                    .with("stufe", Param::Integer(i64::from(*level)))
                    .with("bis", Param::Date(r.date.add_months(*months))),
                RegulationKind::Safety { .. } => m.with(
                    "art",
                    Param::TextKey("regulierungsart.arbeitsschutz".into()),
                ),
                RegulationKind::Ban { .. } => {
                    m.with("art", Param::TextKey("regulierungsart.verbot".into()))
                }
                RegulationKind::Antitrust => m.with(
                    "art",
                    Param::TextKey("regulierungsart.kartellaufsicht".into()),
                ),
            };
            news.push(m);
        }
    }
    let year = date.year();
    let top = available(catalog, year);
    for i in 0..state.sites.len() {
        // Few sites; the cast is exact.
        let site = SiteId(i as u32);
        let (owner, country) = (state.sites[i].owner, state.sites[i].country);
        if state.companies[owner.index()].bankrupt {
            continue;
        }
        let target = if overfulfils(catalog, state, owner, site) {
            top
        } else {
            state.regulation.announced(country).min(top)
        };
        let mut done = 0u32;
        for slot in 0..state.sites[i].slots.len() {
            let sl = &state.sites[i].slots[slot];
            let polluting = sl
                .recipe
                .is_some_and(|r| catalog.recipes.get(r).pollutant_kg > 0.0);
            if !polluting {
                continue;
            }
            while state.sites[i].slots[slot].retrofit < target {
                if retrofit(state, catalog, owner, (site, slot)).is_err() {
                    break;
                }
                done += 1;
            }
        }
        if done > 0 && state.is_main(owner) {
            news.push(
                Message::new(MessageKind::Info, keys::REGULATION_RETROFITTED)
                    .with(
                        "standort",
                        Param::TextKey(crate::command::site_type_key(state.sites[i].kind)),
                    )
                    .with(
                        "land",
                        Param::Country(catalog.countries.key(country).to_owned()),
                    )
                    .with("anzahl", Param::Integer(i64::from(done))),
            );
        }
        if state.is_main(owner)
            && let Some(deadline) = state.regulation.country(country).deadline
            && deadline <= date.add_months(12)
            && state.sites[i].slots.iter().any(|sl| {
                sl.recipe
                    .is_some_and(|r| catalog.recipes.get(r).pollutant_kg > 0.0)
                    && sl.retrofit < state.regulation.announced(country)
            })
        {
            news.push(
                Message::new(MessageKind::Warning, keys::REGULATION_DEADLINE)
                    .with(
                        "standort",
                        Param::TextKey(crate::command::site_type_key(state.sites[i].kind)),
                    )
                    .with(
                        "land",
                        Param::Country(catalog.countries.key(country).to_owned()),
                    )
                    .with("bis", Param::Date(deadline)),
            );
        }
    }
    news
}

/// Books the CO2 cost of `runs` of a recipe at a site and counts the emissions; returns
/// the cost (part of the product's value).
pub(crate) fn emit(
    state: &mut GameState,
    catalog: &Catalog,
    (site, slot): (SiteId, usize),
    recipe: &Recipe,
    runs: f64,
) -> Money {
    let s = &state.sites[site.index()];
    let (owner, country) = (s.owner, s.country);
    let level = s.slots[slot].retrofit;
    let co2 = recipe.co2_t * runs;
    let pollutant = pollutant(catalog, recipe, level) * runs;
    let e = &mut state.companies[owner.index()].emissions;
    e.co2_t += co2;
    e.pollutant_kg += pollutant;
    Money::from_usd(co2 * co2_price(catalog, country, state.date)).unwrap_or(Money::ZERO)
}

/// Factor on a company's gain of brand awareness from its pollution per revenue against
/// the average of all active companies (docs/FORMELN.md, H2).
pub fn image_factors(catalog: &Catalog, state: &GameState) -> Vec<f64> {
    let weight = catalog.environment.image_weight;
    let revenue = |c: &crate::state::Company| {
        c.ledger
            .months
            .last()
            .map_or(0.0, |m| {
                m.by_type
                    .get(&crate::ledger::CostType::Revenue)
                    .copied()
                    .unwrap_or_default()
                    .to_usd()
            })
            .max(0.0)
    };
    let (mut p, mut r) = (0.0, 0.0);
    for c in state.companies.iter().filter(|c| !c.bankrupt) {
        p += c.emissions_last.pollutant_kg;
        r += revenue(c);
    }
    let mean = if r > 0.0 { p / r } else { 0.0 };
    state
        .companies
        .iter()
        .map(|c| {
            let own = revenue(c);
            if weight <= 0.0 || mean <= 0.0 || own <= 0.0 {
                return 1.0;
            }
            let x = c.emissions_last.pollutant_kg / own;
            1.0 - weight * (x / mean - 1.0).clamp(-1.0, 1.0)
        })
        .collect()
}

/// `antitrust` as the error of a command.
pub(crate) fn check_antitrust(
    catalog: &Catalog,
    state: &GameState,
    buyer: CompanyId,
    target: CompanyId,
) -> Result<(), CommandError> {
    match antitrust(catalog, state, buyer, target) {
        Some((product, country, share)) => Err(CommandError::Antitrust {
            product: catalog.products.key(product).to_owned(),
            country: catalog.countries.key(country).to_owned(),
            share,
        }),
        None => Ok(()),
    }
}

/// The first market an acquisition by `buyer` of `target` would dominate where antitrust
/// applies: product, country and the combined share (docs/FORMELN.md, H2).
pub fn antitrust(
    catalog: &Catalog,
    state: &GameState,
    buyer: CompanyId,
    target: CompanyId,
) -> Option<(ProductId, CountryId, f64)> {
    let max = catalog.environment.antitrust_share_max;
    if max <= 0.0 || max >= 1.0 {
        return None;
    }
    let group = crate::group::members(state, crate::group::top(state, buyer));
    // Sales last month per (country, product): all, the buyer's group, the target.
    let mut sold: std::collections::BTreeMap<(CountryId, ProductId), [f64; 3]> =
        std::collections::BTreeMap::new();
    for s in &state.sites {
        if !state.regulation.antitrust(s.country) {
            continue;
        }
        for (&p, o) in &s.offers {
            if o.sold_last_month <= 0.0 {
                continue;
            }
            let e = sold.entry((s.country, p)).or_default();
            e[0] += o.sold_last_month;
            if group.contains(&s.owner) {
                e[1] += o.sold_last_month;
            }
            if s.owner == target {
                e[2] += o.sold_last_month;
            }
        }
    }
    sold.into_iter()
        .find_map(|((country, product), [all, own, theirs])| {
            let share = (own + theirs) / all;
            (own > 0.0 && theirs > 0.0 && share > max).then_some((product, country, share))
        })
}

/// The regulations in force in a country as texts for the country view, the oldest
/// first.
pub fn country_messages(catalog: &Catalog, state: &GameState, country: CountryId) -> Vec<Message> {
    let month = state.date.first_of_month();
    catalog
        .regulations
        .iter()
        .filter(|r| in_force(r.date, month) && r.countries.contains(&country))
        .map(|r| {
            let m = |key: &str| {
                Message::new(MessageKind::Info, key)
                    .with("regel", Param::TextKey(format!("regulierung.{}", r.key)))
                    .with("seit", Param::Date(r.date))
            };
            match &r.kind {
                RegulationKind::Retrofit { level, months } => m("landdetail.regel.auflage")
                    .with("stufe", Param::Integer(i64::from(*level)))
                    .with("bis", Param::Date(r.date.add_months(*months))),
                RegulationKind::Safety { wage_surcharge } => m("landdetail.regel.arbeitsschutz")
                    .with(
                        "aufschlag",
                        Param::Number((wage_surcharge * 1000.0).round() / 10.0),
                    ),
                RegulationKind::Ban {
                    products,
                    production,
                    sales,
                } => m(match (production, sales) {
                    (true, true) => "landdetail.regel.verbot_beides",
                    (true, false) => "landdetail.regel.verbot_herstellung",
                    _ => "landdetail.regel.verbot_verkauf",
                })
                .with(
                    "produkte",
                    Param::TextKeys(
                        products
                            .iter()
                            .map(|&p| format!("produkt.{}", catalog.products.key(p)))
                            .collect(),
                    ),
                ),
                RegulationKind::Antitrust => m("landdetail.regel.kartell").with(
                    "anteil",
                    Param::Number((catalog.environment.antitrust_share_max * 100.0).round()),
                ),
            }
        })
        .collect()
}
