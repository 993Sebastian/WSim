//! Strategies of the player's company (MA4; docs/BEDIENUNG.md, "Strategie"): what holds
//! for each unit and field, where it comes from and which position carries it out.

use serde::{Deserialize, Serialize};

use super::concerns::position_view;
use super::organisation::{kind_text, level_key, unit_key};
use super::{ConcernPositionView, usd};
use std::collections::BTreeSet;

use crate::catalog::Catalog;
use crate::decision::Topic;
use crate::game::Game;
use crate::ids::{CountryId, ProductId};
use crate::management;
use crate::policy::{BuyerGroup, Scope};
use crate::state::{GameState, SiteId, Unit};
use crate::strategy::{self, StrategyField, StrategyScope, StrategyValue};

/// The position that follows a strategy at a site, with its manager.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StrategyCarrierView {
    pub position: ConcernPositionView,
    pub manager: String,
}

/// One field at one unit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StrategyEntryView {
    /// `preis`, `lager`, `personal`, `eigenfertigung`, `investition`, `reserve`.
    pub field: String,
    /// What holds, as `SetStrategy` takes it; `None`: no investment budget.
    pub value: Option<StrategyValue>,
    /// The unit it comes from (`firma`, `kontinent:europa`, `land:DEU`, `standort:3`);
    /// `None`: the default.
    pub origin: Option<String>,
    /// The unit sets it itself.
    pub own: bool,
    /// Investment budget per year of the setting that holds.
    pub budget_usd: Option<f64>,
    /// What is left this year of the budget that binds first, and where it is set.
    pub left_usd: Option<f64>,
    pub binding: Option<String>,
    /// Liquidity reserve at today's running costs.
    pub reserve_usd: Option<f64>,
    /// At sites: the position that follows it; `None` where the player decides himself.
    pub carrier: Option<StrategyCarrierView>,
}

/// A unit of the company with its strategies.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StrategyUnitView {
    /// `firma`, `kontinent:europa`, `land:DEU`, `standort:3` (as in the organisation).
    pub key: String,
    /// `firma`, `kontinent`, `land`, `standort`.
    pub level: String,
    /// Text key: the site type, `ebene.land`, `ebene.kontinent` or `ebene.firma`.
    pub kind_text: String,
    pub country: Option<String>,
    pub continent: Option<String>,
    pub site: Option<u32>,
    /// The unit above; `None` for the company.
    pub parent: Option<String>,
    /// The company's sites in the unit.
    pub sites: u32,
    pub entries: Vec<StrategyEntryView>,
}

/// Largest settings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StrategyLimitsView {
    pub min_margin_max: f64,
    pub stock_days_max: f64,
    pub wage_premium_max: f64,
    pub reserve_months_max: f64,
}

/// A price strategy: floor on the full unit cost and markup an offer starts at.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StrategyPriceView {
    /// `marktpreis`, `premium`, `kampfpreis`.
    pub kind: String,
    pub floor: f64,
    pub markup: f64,
}

/// A rule for who besides consumers and governments buys the company's goods (Lastenheft
/// §9.2, docs/FORMELN.md M8).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SalesChannelView {
    /// `haendler` (traders who export the goods) or `firmen` (other companies).
    pub buyer: String,
    /// The product and country the rule holds for; none: all.
    pub product: Option<String>,
    pub country: Option<String>,
    pub allowed: bool,
    pub min_price_usd: Option<f64>,
    pub max_per_month: Option<f64>,
    /// Text key of the product's unit (`einheit.<key>`).
    pub unit: Option<String>,
}

/// A product the company sells, for new rules.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SaleProductView {
    pub product: String,
    /// Text key of its unit (`einheit.<key>`).
    pub unit: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StrategyView {
    /// Without managers there is nobody to follow strategies.
    pub enabled: bool,
    /// Company, continents, countries, sites in the order of the organisation.
    pub units: Vec<StrategyUnitView>,
    pub limits: StrategyLimitsView,
    pub prices: Vec<StrategyPriceView>,
    /// Running costs of a month of all sites at their planned production.
    pub monthly_cost_usd: f64,
    /// Settings of the player.
    pub settings: u32,
    /// Who may buy the company's goods, the most general rules first.
    pub sales: Vec<SalesChannelView>,
    /// Products the company offers or has rules for, in the order of the data.
    pub sale_products: Vec<SaleProductView>,
    /// Countries of its sites and rules, in the order of the data.
    pub sale_countries: Vec<String>,
    /// The policy „Beteiligungen“ (ZA2).
    pub participations: super::ParticipationsView,
}

fn unit_text(catalog: &Catalog, product: ProductId) -> String {
    format!(
        "einheit.{}",
        catalog.units.key(catalog.products.get(product).unit)
    )
}

/// The sales channels of the player's company and what new rules can name.
fn sales_channels(
    game: &Game,
    own: &[SiteId],
) -> (Vec<SalesChannelView>, Vec<SaleProductView>, Vec<String>) {
    let c = game.catalog();
    let state = game.state();
    let company = &state.companies[game.player().index()];
    let parts = |scope: Scope| -> (Option<ProductId>, Option<CountryId>) {
        match scope {
            Scope::Company => (None, None),
            Scope::Country(k) => (None, Some(k)),
            Scope::Product(p) => (Some(p), None),
            Scope::ProductInCountry(p, k) => (Some(p), Some(k)),
        }
    };
    let mut products: BTreeSet<ProductId> = own
        .iter()
        .flat_map(|&s| state.sites[s.index()].offers.keys().copied())
        .collect();
    let mut countries: BTreeSet<CountryId> = own
        .iter()
        .map(|&s| state.sites[s.index()].country)
        .collect();
    let mut rules: Vec<_> = company.sales_policies.iter().collect();
    // General rules first: the company, then countries, products, products in countries.
    rules.sort_by_key(|r| {
        let (p, k) = parts(r.scope);
        (p.is_some(), k.is_some(), r.buyer, p, k)
    });
    let sales = rules
        .into_iter()
        .map(|r| {
            let (product, country) = parts(r.scope);
            products.extend(product);
            countries.extend(country);
            SalesChannelView {
                buyer: match r.buyer {
                    BuyerGroup::Traders => "haendler",
                    BuyerGroup::Companies => "firmen",
                }
                .into(),
                product: product.map(|p| c.products.key(p).to_owned()),
                country: country.map(|k| c.countries.key(k).to_owned()),
                allowed: r.rule.allowed,
                min_price_usd: r.rule.min_price.map(usd),
                max_per_month: r.rule.max_per_month,
                unit: product.map(|p| unit_text(c, p)),
            }
        })
        .collect();
    let products = products
        .into_iter()
        .map(|p| SaleProductView {
            product: c.products.key(p).to_owned(),
            unit: unit_text(c, p),
        })
        .collect();
    let countries = countries
        .into_iter()
        .map(|k| c.countries.key(k).to_owned())
        .collect();
    (sales, products, countries)
}

/// Key of a scope as the views name units; the company is `firma`.
pub fn scope_key(catalog: &Catalog, scope: StrategyScope) -> String {
    match scope {
        StrategyScope::Company => "firma".into(),
        StrategyScope::Continent(k) => unit_key(catalog, Unit::Continent(k)),
        StrategyScope::Country(c) => unit_key(catalog, Unit::Country(c)),
        StrategyScope::Site(s) => unit_key(catalog, Unit::Site(s)),
    }
}

/// The scope a key names (`scope_key`).
pub fn scope_from_key(catalog: &Catalog, key: &str) -> Option<StrategyScope> {
    if key == "firma" {
        return Some(StrategyScope::Company);
    }
    super::unit_from_key(catalog, key).map(StrategyScope::of)
}

/// The topic whose position follows a field.
fn topic_of(field: StrategyField) -> Topic {
    match field {
        StrategyField::Price => Topic::Sale,
        StrategyField::Stock => Topic::Purchase,
        StrategyField::Wages => Topic::Wage,
        StrategyField::Supply => Topic::OwnSupply,
        StrategyField::Investment | StrategyField::Reserve => Topic::Expansion,
    }
}

fn entry(game: &Game, unit: Option<Unit>, field: StrategyField, monthly: f64) -> StrategyEntryView {
    let c = game.catalog();
    let state = game.state();
    let player = game.player();
    let company = &state.companies[player.index()];
    let (value, origin) = match unit {
        Some(u) => strategy::effective(c, state, player, u, field),
        // The company holds only what is set for the company.
        None => company
            .strategies
            .iter()
            .find(|s| s.scope == StrategyScope::Company && s.value.field() == field)
            .map_or((strategy::default_value(c, field), None), |s| {
                (Some(s.value), Some(s.scope))
            }),
    };
    let own = origin.is_some_and(|o| Some(o) == unit.map(StrategyScope::of))
        || (unit.is_none() && origin == Some(StrategyScope::Company));
    let year = state.date.year();
    let budgets = match unit {
        Some(u) => strategy::investment_budgets(c, state, player, u, year),
        None => company
            .strategies
            .iter()
            .filter(|s| s.scope == StrategyScope::Company)
            .filter_map(|s| strategy::budget_of(s, year))
            .collect(),
    };
    let binding = (field == StrategyField::Investment)
        .then(|| strategy::binding_budget(&budgets))
        .flatten();
    let carrier = match unit {
        Some(Unit::Site(s)) => {
            management::first_taker(c, state, player, Unit::Site(s), topic_of(field)).and_then(
                |p| {
                    let manager = management::holder(state, player, &p)?;
                    Some(StrategyCarrierView {
                        position: position_view(c, state, &p),
                        manager: state.managers[&manager].name.clone(),
                    })
                },
            )
        }
        _ => None,
    };
    StrategyEntryView {
        field: field.key().to_owned(),
        value,
        origin: origin.map(|o| scope_key(c, o)),
        own,
        budget_usd: match value {
            Some(StrategyValue::Investment(m)) => Some(usd(m)),
            _ => None,
        },
        left_usd: binding.map(|b| usd(b.left)),
        binding: binding.map(|b| scope_key(c, b.scope)),
        reserve_usd: match value {
            Some(StrategyValue::Reserve(months)) => Some(monthly * months),
            _ => None,
        },
        carrier,
    }
}

fn unit_view(game: &Game, unit: Option<Unit>, (sites, monthly): (u32, f64)) -> StrategyUnitView {
    let c = game.catalog();
    let state: &GameState = game.state();
    let entries = StrategyField::ALL
        .iter()
        .map(|&f| entry(game, unit, f, monthly))
        .collect();
    let Some(u) = unit else {
        return StrategyUnitView {
            key: "firma".into(),
            level: "firma".into(),
            kind_text: "ebene.firma".into(),
            country: None,
            continent: None,
            site: None,
            parent: None,
            sites,
            entries,
        };
    };
    let (country, continent, parent) = match u {
        Unit::Site(s) => {
            let k = state.sites[s.index()].country;
            (Some(k), None, unit_key(c, Unit::Country(k)))
        }
        Unit::Country(k) => (
            Some(k),
            None,
            unit_key(c, Unit::Continent(c.countries.get(k).continent)),
        ),
        Unit::Continent(k) => (None, Some(k), "firma".into()),
        // The board is the company's level, shown as `firma`.
        Unit::Board => (None, None, "firma".into()),
    };
    StrategyUnitView {
        key: unit_key(c, u),
        level: level_key(u).to_owned(),
        kind_text: kind_text(state, u),
        country: country.map(|k| c.countries.key(k).to_owned()),
        continent: continent.map(|k| c.continents.key(k).to_owned()),
        site: match u {
            Unit::Site(s) => Some(s.0),
            _ => None,
        },
        parent: Some(parent),
        sites,
        entries,
    }
}

/// The strategies of the player's company for each of its units.
pub fn strategy(game: &Game) -> StrategyView {
    let c = game.catalog();
    let state = game.state();
    let player = game.player();
    let company = &state.companies[player.index()];
    let own: Vec<SiteId> = state
        .sites
        .iter()
        .enumerate()
        .filter(|(_, s)| s.owner == player)
        // Few sites; the cast is exact.
        .map(|(i, _)| SiteId(i as u32))
        .collect();
    let monthly = usd(strategy::monthly_cost(c, state, player));
    let set_at = |scope: StrategyScope| company.strategies.iter().any(|s| s.scope == scope);
    let count = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    let mut units = vec![unit_view(game, None, (count(own.len()), monthly))];
    for continent in c.continents.ids() {
        let in_continent: Vec<SiteId> = own
            .iter()
            .copied()
            .filter(|&s| c.countries.get(state.sites[s.index()].country).continent == continent)
            .collect();
        let countries: Vec<_> = c
            .countries
            .ids()
            .filter(|&k| c.countries.get(k).continent == continent)
            .filter(|&k| {
                in_continent
                    .iter()
                    .any(|&s| state.sites[s.index()].country == k)
                    || set_at(StrategyScope::Country(k))
            })
            .collect();
        if countries.is_empty() && !set_at(StrategyScope::Continent(continent)) {
            continue;
        }
        units.push(unit_view(
            game,
            Some(Unit::Continent(continent)),
            (count(in_continent.len()), monthly),
        ));
        for country in countries {
            let sites: Vec<SiteId> = in_continent
                .iter()
                .copied()
                .filter(|&s| state.sites[s.index()].country == country)
                .collect();
            units.push(unit_view(
                game,
                Some(Unit::Country(country)),
                (count(sites.len()), monthly),
            ));
            for site in sites {
                units.push(unit_view(game, Some(Unit::Site(site)), (1, monthly)));
            }
        }
    }
    let (sales, sale_products, sale_countries) = sales_channels(game, &own);
    let m = &c.management.strategy;
    let (_, aggressiveness) = crate::ai::traits(c, state, player);
    let rules_floor = c.ai_model.behavior.floor_factor.at(aggressiveness);
    StrategyView {
        enabled: c.management.enabled(),
        units,
        limits: StrategyLimitsView {
            min_margin_max: m.min_margin_max,
            stock_days_max: m.stock_days_max,
            wage_premium_max: c.production_model.wage_premium_max,
            reserve_months_max: m.reserve_months_max,
        },
        prices: [
            ("marktpreis", (rules_floor, 0.0)),
            ("premium", m.premium),
            ("kampfpreis", m.fight),
        ]
        .into_iter()
        .map(|(kind, (floor, markup))| StrategyPriceView {
            kind: kind.into(),
            floor,
            markup,
        })
        .collect(),
        monthly_cost_usd: monthly,
        settings: count(company.strategies.len()),
        sales,
        sale_products,
        sale_countries,
        participations: super::participations(game),
    }
}
