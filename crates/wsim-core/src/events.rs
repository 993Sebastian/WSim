//! Effects of the historical events (H1, Lastenheft §4.1; formulas in docs/FORMELN.md, H1).
//!
//! Effects with a duration are looked up in a table derived each month (not saved);
//! effects that act once run on the event's day.

use std::collections::BTreeMap;

use crate::calendar::Date;
use crate::catalog::{Catalog, EffectKind, EventEffect, HistoricalEvent};
use crate::ids::{BranchId, CountryId, GoodsGroupId, Id, ProductId};
use crate::ledger::{Account, CostCenter, CostType};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{CompanyId, GameState, SiteId};

/// The first day of a month on or after `date`.
pub fn month_on_or_after(date: Date) -> Date {
    if date.day() == 1 {
        date
    } else {
        date.first_of_next_month()
    }
}

/// Whether an effect with a duration acts in the month starting on `month`.
pub fn active_in(event: &HistoricalEvent, effect: &EventEffect, month: Date) -> bool {
    !effect.kind.once()
        && month_on_or_after(event.date) <= month
        && effect.until.is_none_or(|u| month < month_on_or_after(u))
}

/// Effects with a duration acting in the month starting on `month`.
pub fn active(catalog: &Catalog, month: Date) -> impl Iterator<Item = &EventEffect> {
    catalog.events.iter().flat_map(move |e| {
        e.effects
            .iter()
            .filter(move |effect| active_in(e, effect, month))
    })
}

/// The effects with a duration of one month, by country and goods group; derived, not
/// saved. Empty (all factors 1) when the game has the effects off.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EventTable {
    month: Option<Date>,
    groups: usize,
    /// Consumer and state demand factor at `country * groups + group`.
    demand: Vec<(f64, f64)>,
    /// Factor on the runs at `country * groups + group`.
    production: Vec<f64>,
    labor: Vec<f64>,
    /// Closure of each country: the countries it is open to (its bloc) and whether
    /// their own companies are shut out too.
    closure: Vec<Option<(usize, bool)>>,
    blocs: Vec<Vec<CountryId>>,
}

fn groups_or_all(groups: &[GoodsGroupId], count: usize) -> Vec<usize> {
    if groups.is_empty() {
        (0..count).collect()
    } else {
        groups.iter().map(|g| g.index()).collect()
    }
}

impl EventTable {
    pub fn new(catalog: &Catalog, month: Date, enabled: bool) -> Self {
        let n = catalog.countries.len();
        let g = catalog.goods_groups.len();
        let mut table = Self {
            month: Some(month),
            groups: g,
            ..Self::default()
        };
        if !enabled {
            return table;
        }
        for effect in active(catalog, month) {
            let countries = effect.countries.iter().map(|c| c.index());
            match &effect.kind {
                EffectKind::Demand {
                    groups,
                    consumer,
                    state,
                } => {
                    if table.demand.is_empty() {
                        table.demand = vec![(1.0, 1.0); n * g];
                    }
                    for c in countries {
                        for i in groups_or_all(groups, g) {
                            let d = &mut table.demand[c * g + i];
                            d.0 *= consumer;
                            d.1 *= state;
                        }
                    }
                }
                EffectKind::Production { groups, factor } => {
                    if table.production.is_empty() {
                        table.production = vec![1.0; n * g];
                    }
                    for c in countries {
                        for i in groups_or_all(groups, g) {
                            table.production[c * g + i] *= factor;
                        }
                    }
                }
                EffectKind::Labor { factor } => {
                    if table.labor.is_empty() {
                        table.labor = vec![1.0; n];
                    }
                    for c in countries {
                        table.labor[c] *= factor;
                    }
                }
                EffectKind::Closure { all } => {
                    if table.closure.is_empty() {
                        table.closure = vec![None; n];
                    }
                    table.blocs.push(effect.countries.clone());
                    let bloc = table.blocs.len() - 1;
                    for c in countries {
                        let all = *all || table.closure[c].is_some_and(|(_, a)| a);
                        table.closure[c] = Some((bloc, all));
                    }
                }
                // Trade is in the tariff table; the others act once.
                _ => {}
            }
        }
        table
    }

    pub fn month(&self) -> Option<Date> {
        self.month
    }

    /// Factors on consumer and state demand for goods of a group in a country.
    pub fn demand(&self, country: CountryId, group: GoodsGroupId) -> (f64, f64) {
        self.demand
            .get(country.index() * self.groups + group.index())
            .copied()
            .unwrap_or((1.0, 1.0))
    }

    /// Factor on the runs of facilities making goods of a group in a country.
    pub fn production(&self, country: CountryId, group: GoodsGroupId) -> f64 {
        self.production
            .get(country.index() * self.groups + group.index())
            .copied()
            .unwrap_or(1.0)
    }

    /// Factor on the available workers of a country.
    pub fn labor(&self, country: CountryId) -> f64 {
        self.labor.get(country.index()).copied().unwrap_or(1.0)
    }

    /// The bloc a closed country is open to and whether its own companies are shut out.
    fn closure(&self, country: CountryId) -> Option<(&[CountryId], bool)> {
        let (bloc, all) = self.closure.get(country.index()).copied().flatten()?;
        Some((&self.blocs[bloc], all))
    }
}

/// Whether a company may not open or buy sites in a country, nor move its seat there
/// (closure, H1): companies from outside its bloc, with `alle` all but the state company.
pub fn closed_to(state: &GameState, company: CompanyId, country: CountryId) -> bool {
    let Some((bloc, all)) = state.events.closure(country) else {
        return false;
    };
    let top = &state.companies[crate::group::top(state, company).index()];
    if top.state_owned.is_some_and(|c| bloc.contains(&c)) {
        return false;
    }
    all || !bloc.contains(&top.headquarters)
}

/// Whether a country is closed to all but its state companies: a new company there is
/// one.
pub fn closed_to_newcomers(state: &GameState, country: CountryId) -> bool {
    state.events.closure(country).is_some_and(|(_, all)| all)
}

/// At the start of a game: in countries closed to all but the state, the AI companies of
/// the start set are state companies (the player's company is not).
pub(crate) fn start(state: &mut GameState) {
    for i in 0..state.companies.len() {
        let c = &state.companies[i];
        if c.ai.is_none() || c.subsidiary_of.is_some() {
            continue;
        }
        let home = c.headquarters;
        if closed_to_newcomers(state, home) {
            state.companies[i].state_owned = Some(home);
        }
    }
}

/// Drops of the stock market's sentiment at the start of the month `month`.
pub fn crashes(state: &GameState, catalog: &Catalog, month: Date) -> Vec<f64> {
    if !state.settings.event_effects {
        return Vec::new();
    }
    catalog
        .events
        .iter()
        .filter(|e| month_on_or_after(e.date) == month)
        .flat_map(|e| e.effects.iter())
        .filter_map(|effect| match effect.kind {
            EffectKind::StockCrash { drop } => Some(drop),
            _ => None,
        })
        .collect()
}

/// The texts the lines of `effect_messages` use.
pub const EFFECT_TEXTS: &[&str] = &[
    "meldung.folge.nachfrage",
    "meldung.folge.nachfrage_offen",
    "meldung.folge.handelssperre",
    "meldung.folge.handelssperre_offen",
    "meldung.folge.zoll",
    "meldung.folge.zoll_offen",
    "meldung.folge.zoll_alle",
    "meldung.folge.zoll_alle_offen",
    "meldung.folge.arbeitskraefte",
    "meldung.folge.arbeitskraefte_offen",
    "meldung.folge.produktion",
    "meldung.folge.produktion_offen",
    "meldung.folge.abschottung",
    "meldung.folge.abschottung_offen",
    "meldung.folge.abschottung_alle",
    "meldung.folge.abschottung_alle_offen",
    "meldung.folge.zerstoerung",
    "meldung.folge.enteignung",
    "meldung.folge.enteignung_alle",
    "meldung.folge.boersenkrach",
    "folge.alle_waren",
];

/// One line per effect of an event for its news (texts `meldung.folge.<art>`, open-ended
/// ones `meldung.folge.<art>_offen`).
pub fn effect_messages(catalog: &Catalog, event: &HistoricalEvent) -> Vec<Message> {
    let countries = |list: &[CountryId]| {
        Param::Countries(
            list.iter()
                .map(|&c| catalog.countries.key(c).to_owned())
                .collect(),
        )
    };
    let groups = |list: &[GoodsGroupId]| {
        Param::TextKeys(if list.is_empty() {
            vec!["folge.alle_waren".to_owned()]
        } else {
            list.iter()
                .map(|&g| format!("warengruppe.{}", catalog.goods_groups.key(g)))
                .collect()
        })
    };
    let percent = |x: f64| Param::Number((x * 100.0).round());
    event
        .effects
        .iter()
        .map(|effect| {
            let (art, params): (&str, Vec<(&str, Param)>) = match &effect.kind {
                EffectKind::Demand {
                    groups: g,
                    consumer,
                    state,
                } => (
                    "nachfrage",
                    vec![
                        ("gruppen", groups(g)),
                        ("konsum", percent(*consumer)),
                        ("staat", percent(*state)),
                    ],
                ),
                EffectKind::Embargo { against } => {
                    ("handelssperre", vec![("gegen", countries(against))])
                }
                EffectKind::Tariff { against, surcharge } => (
                    if against.is_empty() {
                        "zoll_alle"
                    } else {
                        "zoll"
                    },
                    vec![
                        ("gegen", countries(against)),
                        ("aufschlag", percent(*surcharge)),
                    ],
                ),
                EffectKind::Labor { factor } => {
                    ("arbeitskraefte", vec![("faktor", percent(*factor))])
                }
                EffectKind::Production { groups: g, factor } => (
                    "produktion",
                    vec![("gruppen", groups(g)), ("faktor", percent(*factor))],
                ),
                EffectKind::Closure { all } => (
                    if *all {
                        "abschottung_alle"
                    } else {
                        "abschottung"
                    },
                    Vec::new(),
                ),
                EffectKind::Destruction { share } => {
                    ("zerstoerung", vec![("anteil", percent(*share))])
                }
                EffectKind::Expropriation {
                    foreign_only,
                    compensation,
                } => (
                    if *foreign_only {
                        "enteignung"
                    } else {
                        "enteignung_alle"
                    },
                    vec![("entschaedigung", percent(*compensation))],
                ),
                EffectKind::StockCrash { drop } => {
                    ("boersenkrach", vec![("einbruch", percent(*drop))])
                }
            };
            let open = !effect.kind.once() && effect.until.is_none();
            let key = format!("meldung.folge.{art}{}", if open { "_offen" } else { "" });
            let mut m = Message::new(MessageKind::Info, &key)
                .with(
                    "ereignis",
                    Param::TextKey(format!("ereignis.{}", event.key)),
                )
                .with("laender", countries(&effect.countries));
            if let Some(u) = effect.until {
                m = m.with("bis", Param::Date(u));
            }
            for (name, p) in params {
                m = m.with(name, p);
            }
            m
        })
        .collect()
}

/// Whether an effect concerns a country: it acts there or trade with it is cut.
fn touches(effect: &EventEffect, country: CountryId) -> bool {
    effect.countries.contains(&country)
        || match &effect.kind {
            EffectKind::Embargo { against } | EffectKind::Tariff { against, .. } => {
                against.contains(&country)
            }
            _ => false,
        }
}

/// The effects with a duration acting on a country this month, one line each.
pub fn active_messages(state: &GameState, catalog: &Catalog, country: CountryId) -> Vec<Message> {
    if !state.settings.event_effects {
        return Vec::new();
    }
    let month = state.date.first_of_month();
    catalog
        .events
        .iter()
        .filter(|e| {
            e.effects
                .iter()
                .any(|effect| active_in(e, effect, month) && touches(effect, country))
        })
        .flat_map(|e| {
            e.effects
                .iter()
                .zip(effect_messages(catalog, e))
                .filter(|(effect, _)| active_in(e, effect, month) && touches(effect, country))
                .map(|(_, m)| m)
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The effects that act once, on the day of their event. Returns the messages.
pub(crate) fn simulate_day(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    if !state.settings.event_effects {
        return news;
    }
    let first = catalog.events.partition_point(|e| e.date < date);
    for (i, event) in catalog.events.iter().enumerate().skip(first) {
        if event.date != date {
            break;
        }
        for (j, effect) in event.effects.iter().enumerate() {
            match effect.kind {
                EffectKind::Destruction { share } => {
                    let stream = Stream::Event {
                        event: u32::try_from(i).unwrap_or(u32::MAX),
                        effect: u32::try_from(j).unwrap_or(u32::MAX),
                    };
                    let mut rng = SimRng::for_stream(state.settings.seed, stream);
                    news.extend(destroy(
                        state,
                        catalog,
                        (event, &effect.countries),
                        share,
                        &mut rng,
                    ));
                }
                EffectKind::Expropriation {
                    foreign_only,
                    compensation,
                } => news.extend(expropriate(
                    state,
                    catalog,
                    (event, &effect.countries),
                    foreign_only,
                    compensation,
                )),
                _ => {}
            }
        }
    }
    news
}

/// Units of `count` hit by a share: the whole part, and one more with the chance of the
/// rest.
fn hit(count: u32, share: f64, rng: &mut SimRng) -> u32 {
    let x = f64::from(count) * share;
    let whole = libm::floor(x);
    let extra = u32::from(rng.next_f64() < x - whole);
    // Whole part of at most `count`: the cast cannot overflow.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let whole = whole as u32;
    (whole + extra).min(count)
}

/// Destroys a share of the facility units and stocks at the sites in the countries.
fn destroy(
    state: &mut GameState,
    catalog: &Catalog,
    (event, countries): (&HistoricalEvent, &[CountryId]),
    share: f64,
    rng: &mut SimRng,
) -> Vec<Message> {
    let date = state.date;
    // Per company of the player's group: units and value lost, by country.
    let mut lost: BTreeMap<CountryId, (u32, Money)> = BTreeMap::new();
    let player = state.main_company;
    for index in 0..state.sites.len() {
        if !countries.contains(&state.sites[index].country) {
            continue;
        }
        let site = SiteId(u32::try_from(index).expect("site count fits u32"));
        let owner = state.sites[index].owner;
        let mut units = 0;
        let mut loss = Money::ZERO;
        let mut slot = 0;
        while slot < state.sites[index].slots.len() {
            let sl = &state.sites[index].slots[slot];
            if sl.ready > date {
                slot += 1;
                continue;
            }
            let k = hit(sl.count, share, rng);
            if k == 0 {
                slot += 1;
                continue;
            }
            let (book, _) = crate::production::sale_value(catalog, sl, k, date);
            let s = &mut state.sites[index];
            if k == s.slots[slot].count {
                let open = s.slots.remove(slot).batches;
                crate::production::deliver(catalog, s, open);
            } else {
                let sl = &mut s.slots[slot];
                let cost = sl.share_of_cost(k);
                sl.cost -= cost;
                sl.count -= k;
                slot += 1;
            }
            units += k;
            loss += book;
            state.companies[owner.index()].ledger.expense(
                CostType::Other,
                CostCenter::site(site),
                Account::FixedAssets,
                book,
            );
        }
        let mut goods = Money::ZERO;
        for stock in state.sites[index].inventory.values_mut() {
            let value = stock.value.scale(share);
            stock.quantity *= 1.0 - share;
            stock.value -= value;
            goods += value;
        }
        if goods > Money::ZERO {
            state.companies[owner.index()].ledger.expense(
                CostType::Other,
                CostCenter::site(site),
                Account::Inventory,
                goods,
            );
        }
        state.sites[index].staffing_due = true;
        if player.is_some_and(|p| crate::group::same_group(state, p, owner))
            && (units > 0 || goods > Money::ZERO)
        {
            let e = lost.entry(state.sites[index].country).or_default();
            e.0 += units;
            e.1 += loss + goods;
        }
    }
    lost.into_iter()
        .map(|(country, (units, value))| {
            Message::new(MessageKind::Crisis, keys::EVENT_DESTRUCTION)
                .with(
                    "ereignis",
                    Param::TextKey(format!("ereignis.{}", event.key)),
                )
                .with(
                    "land",
                    Param::Country(catalog.countries.key(country).to_owned()),
                )
                .with("einheiten", Param::Integer(i64::from(units)))
                .with("verlust", Param::Money(value))
        })
        .collect()
}

/// Whether a site in the countries is seized: those of companies from outside them, or
/// with `foreign_only` false all but the state companies'.
fn seized(state: &GameState, site: SiteId, countries: &[CountryId], foreign_only: bool) -> bool {
    let s = &state.sites[site.index()];
    let owner = &state.companies[s.owner.index()];
    if !countries.contains(&s.country) || owner.bankrupt {
        return false;
    }
    let top = &state.companies[crate::group::top(state, s.owner).index()];
    if top.state_owned.is_some_and(|c| countries.contains(&c)) {
        return false;
    }
    !foreign_only || !countries.contains(&top.headquarters)
}

/// The state company of a country; founded if there is none (or it failed).
fn state_company(
    state: &mut GameState,
    catalog: &Catalog,
    country: CountryId,
    branch: BranchId,
    news: &mut Vec<Message>,
) -> CompanyId {
    if let Some(i) = state
        .companies
        .iter()
        .position(|c| c.state_owned == Some(country) && !c.bankrupt)
    {
        return CompanyId(u32::try_from(i).expect("company count fits u32"));
    }
    let date = state.date;
    let id = crate::ai::push_company(state, catalog, (country, branch), date, Money::ZERO, None);
    let c = &mut state.companies[id.index()];
    c.state_owned = Some(country);
    news.push(
        Message::new(MessageKind::Info, keys::EVENT_STATE_COMPANY)
            .with("firma", Param::Text(c.name.clone()))
            .with(
                "land",
                Param::Country(catalog.countries.key(country).to_owned()),
            ),
    );
    id
}

/// The branch of the first product a site makes, for the name of the state company.
fn site_branch(state: &GameState, catalog: &Catalog, site: SiteId) -> Option<BranchId> {
    let product: ProductId = state.sites[site.index()]
        .slots
        .iter()
        .find_map(|sl| sl.recipe.map(|r| catalog.recipes.get(r).product))?;
    Some(catalog.products.get(product).branch)
}

/// Hands the seized sites in the countries to their state companies.
fn expropriate(
    state: &mut GameState,
    catalog: &Catalog,
    (event, countries): (&HistoricalEvent, &[CountryId]),
    foreign_only: bool,
    compensation: f64,
) -> Vec<Message> {
    let mut news = Vec::new();
    let sites: Vec<SiteId> = (0..state.sites.len())
        .map(|i| SiteId(u32::try_from(i).expect("site count fits u32")))
        .filter(|&s| seized(state, s, countries, foreign_only))
        .collect();
    let date = state.date;
    let player = state.main_company;
    // The player's losses by country: sites, loss, compensation.
    let mut mine: BTreeMap<CountryId, (u32, Money, Money)> = BTreeMap::new();
    for site in sites {
        let country = state.sites[site.index()].country;
        let branch = site_branch(state, catalog, site).unwrap_or(BranchId::from_index(0));
        let to = state_company(state, catalog, country, branch, &mut news);
        let from = state.sites[site.index()].owner;
        let v = crate::deals::site_value(state, catalog, site);
        let center = CostCenter::site(site);
        let book = v.book() + v.inventory;
        let paid = book.scale(compensation);

        let l = &mut state.companies[from.index()].ledger;
        l.transfer(Account::Cash, Account::FixedAssets, v.fixed_assets);
        l.transfer(
            Account::Cash,
            Account::AssetsUnderConstruction,
            v.under_construction,
        );
        l.transfer(Account::Cash, Account::Goodwill, v.goodwill);
        l.transfer(Account::Cash, Account::Inventory, v.inventory);
        l.transfer(Account::Cash, Account::Land, v.land_book);
        if book > paid {
            l.expense(CostType::Other, center, Account::Cash, book - paid);
        }

        let working = book.scale(catalog.event_model.working_capital_share);
        let l = &mut state.companies[to.index()].ledger;
        l.transfer(Account::FixedAssets, Account::Equity, v.fixed_assets);
        l.transfer(
            Account::AssetsUnderConstruction,
            Account::Equity,
            v.under_construction,
        );
        l.transfer(Account::Inventory, Account::Equity, v.inventory);
        l.transfer(Account::Land, Account::Equity, v.land);
        l.transfer(Account::Cash, Account::Equity, working);

        // The state company can run what it took over.
        let mut known = Vec::new();
        for sl in &state.sites[site.index()].slots {
            known.extend(catalog.facilities.get(sl.facility).technology);
            if let Some(r) = sl.recipe {
                known.extend(catalog.recipes.get(r).technology);
            }
        }
        state.companies[to.index()].technologies.extend(known);

        let s = &mut state.sites[site.index()];
        s.owner = to;
        s.acquired = Some(date);
        crate::management::release_site(state, site);
        let s = &mut state.sites[site.index()];
        s.goodwill = None;
        s.staffing_due = true;
        if let Some(plot) = s.plot
            && matches!(
                state.plots[plot.index()].tenure,
                crate::state::Tenure::Owned(_)
            )
        {
            state.plots[plot.index()].tenure = crate::state::Tenure::Owned(v.land);
        }
        if player.is_some_and(|p| crate::group::same_group(state, p, from)) {
            let e = mine.entry(country).or_default();
            e.0 += 1;
            e.1 += book - paid;
            e.2 += paid;
        }
    }
    news.extend(mine.into_iter().map(|(country, (sites, loss, paid))| {
        Message::new(MessageKind::Crisis, keys::EVENT_EXPROPRIATION)
            .with(
                "ereignis",
                Param::TextKey(format!("ereignis.{}", event.key)),
            )
            .with(
                "land",
                Param::Country(catalog.countries.key(country).to_owned()),
            )
            .with("standorte", Param::Integer(i64::from(sites)))
            .with("verlust", Param::Money(loss))
            .with("entschaedigung", Param::Money(paid))
    }));
    news
}
