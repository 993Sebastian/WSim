//! Production chains from each end product down to the raw materials (M25; Lastenheft
//! §14.1): how each good is made, what that costs in the home country, what the player
//! makes or buys already, and where its own plants get stuck. It answers "what pays
//! next, and what do I need for it?".

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::usd;
use crate::catalog::{Catalog, ProductKind};
use crate::game::Game;
use crate::health;
use crate::ids::{ProductId, RecipeId};
use crate::market;
use crate::state::{GameState, Limit};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChainsView {
    /// Country of the headquarters: prices and costs are estimated there.
    pub country: String,
    /// The tops of the chains in data order: products nothing else is made of (end
    /// products, but also nails for the building trade or petroleum for the lamps).
    pub roots: Vec<String>,
    /// Every product of the chains, once, in data order.
    pub products: Vec<ChainProduct>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChainProduct {
    pub product: String,
    /// `rohstoff`, `halbzeug`, `komponente`, `endprodukt` or `energie`.
    pub kind: String,
    pub unit: String,
    pub reference_usd: f64,
    /// Market price in the home country.
    pub price_usd: f64,
    /// How it is made: the cheapest recipe the player may use, else the cheapest one
    /// invented so far; `None` for goods of the state market, electricity and goods not
    /// invented yet.
    pub recipe: Option<ChainRecipe>,
    /// Bought from the state market (no chain of its own).
    pub state_market: bool,
    /// The player makes, buys or offers it.
    pub makes: bool,
    pub buys: bool,
    pub sells: bool,
    /// Why own facilities for it made less than planned: `vorprodukt` with the input,
    /// `arbeitskraefte`, `strom`, `lagerstaette`.
    pub stuck: Vec<(String, Option<String>)>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChainRecipe {
    pub key: String,
    pub facility: String,
    /// Raw material from a deposit.
    pub extraction: bool,
    /// Technologies the player still needs (recipe and facility).
    pub missing: Vec<String>,
    /// Inputs per unit of output.
    pub inputs: Vec<(String, f64)>,
    /// Full cost of one unit in the home country, inputs at the market prices there and
    /// the facility at normal utilization (docs/FORMELN.md, M25).
    pub unit_cost_usd: f64,
    /// (price − cost) / price at the market price there.
    pub margin: Option<f64>,
}

fn kind_key(kind: ProductKind) -> &'static str {
    match kind {
        ProductKind::RawMaterial => "rohstoff",
        ProductKind::SemiFinished => "halbzeug",
        ProductKind::Component => "komponente",
        ProductKind::EndProduct => "endprodukt",
        ProductKind::Energy => "energie",
    }
}

/// Technologies of a recipe and its facility the player does not know.
fn missing(state: &GameState, catalog: &Catalog, recipe: RecipeId) -> Vec<String> {
    let r = catalog.recipes.get(recipe);
    let mut keys: Vec<String> = [r.technology, catalog.facilities.get(r.facility).technology]
        .into_iter()
        .flatten()
        .filter(|&t| !state.knows(catalog, state.player, t))
        .map(|t| catalog.technologies.key(t).to_owned())
        .collect();
    keys.dedup();
    keys
}

pub fn chains(game: &Game) -> ChainsView {
    let state = game.state();
    let catalog = game.catalog();
    let player = state.player;
    let home = state.companies[player.index()].headquarters;
    let country = state.countries.get(home);
    let price = |p: ProductId| market::market_price(catalog, state, home, p).to_usd();
    let utilization = catalog.market_model.normal_utilization;
    let year = state.date.year();

    // What the player makes, buys and sells, and where its plants get stuck.
    let mut makes = BTreeSet::new();
    let mut buys = BTreeSet::new();
    let mut sells = BTreeSet::new();
    let mut stuck: BTreeMap<ProductId, Vec<(String, Option<String>)>> = BTreeMap::new();
    for s in state.sites.iter().filter(|s| s.owner == player) {
        buys.extend(s.orders.keys().copied());
        sells.extend(s.offers.keys().copied());
        for sl in s.slots.iter().filter(|sl| sl.operating(state.date)) {
            let Some(r) = sl.recipe.map(|r| catalog.recipes.get(r)) else {
                continue;
            };
            makes.insert(r.product);
            let cause = match sl.limit {
                Some(Limit::Input(p)) => ("vorprodukt", Some(catalog.products.key(p).to_owned())),
                Some(Limit::Labor(_)) => ("arbeitskraefte", None),
                Some(Limit::Electricity) => ("strom", None),
                Some(Limit::Deposit) => ("lagerstaette", None),
                Some(Limit::Event) => ("ereignis", None),
                None => continue,
            };
            let list = stuck.entry(r.product).or_default();
            let cause = (cause.0.to_owned(), cause.1);
            if !list.contains(&cause) {
                list.push(cause);
            }
        }
    }

    // The recipe shown for each product: usable first, then cheapest.
    let recipe_of = |p: ProductId| -> Option<(RecipeId, f64)> {
        catalog
            .recipes
            .iter()
            .filter(|&(id, r)| r.product == p && health::first_year(catalog, id) <= year)
            .map(|(id, _)| {
                let cost = health::unit_cost(catalog, country, id, utilization, price).total();
                (id, missing(state, catalog, id).is_empty(), cost)
            })
            .min_by(|a, b| b.1.cmp(&a.1).then(a.2.total_cmp(&b.2)))
            .map(|(id, _, cost)| (id, cost))
    };

    // The products of all chains: their tops and, through the shown recipes, the inputs
    // down to raw materials. Electricity is a production factor, not a chain; goods of
    // the state market have none.
    let used: BTreeSet<ProductId> = catalog
        .recipes
        .values()
        .flat_map(|r| r.inputs.iter().map(|&(i, _)| i))
        .collect();
    let roots: Vec<ProductId> = catalog
        .products
        .iter()
        .filter(|&(id, p)| {
            !used.contains(&id)
                && p.kind != ProductKind::Energy
                && catalog.recipes.values().any(|r| r.product == id)
        })
        .map(|(id, _)| id)
        .collect();
    let mut chosen: BTreeMap<ProductId, Option<(RecipeId, f64)>> = BTreeMap::new();
    let mut open: Vec<ProductId> = roots.clone();
    while let Some(p) = open.pop() {
        if chosen.contains_key(&p) {
            continue;
        }
        let made = catalog.products.get(p).kind != ProductKind::Energy;
        let r = if made { recipe_of(p) } else { None };
        if let Some((id, _)) = r {
            open.extend(catalog.recipes.get(id).inputs.iter().map(|&(i, _)| i));
        }
        chosen.insert(p, r);
    }

    let products = catalog
        .products
        .iter()
        .filter(|(id, _)| chosen.contains_key(id))
        .map(|(id, p)| {
            let market_price = price(id);
            let recipe = chosen[&id].map(|(rid, cost)| {
                let r = catalog.recipes.get(rid);
                ChainRecipe {
                    key: catalog.recipes.key(rid).to_owned(),
                    facility: catalog.facilities.key(r.facility).to_owned(),
                    extraction: r.extraction,
                    missing: missing(state, catalog, rid),
                    inputs: r
                        .inputs
                        .iter()
                        .map(|&(i, q)| (catalog.products.key(i).to_owned(), q / r.output.max(1e-9)))
                        .collect(),
                    unit_cost_usd: cost,
                    margin: (market_price > 0.0).then(|| (market_price - cost) / market_price),
                }
            });
            ChainProduct {
                product: catalog.products.key(id).to_owned(),
                kind: kind_key(p.kind).to_owned(),
                unit: catalog.units.key(p.unit).to_owned(),
                reference_usd: usd(market::local_reference(catalog, state, home, id)),
                price_usd: market_price,
                recipe,
                state_market: p.state_market.is_some(),
                makes: makes.contains(&id),
                buys: buys.contains(&id),
                sells: sells.contains(&id),
                stuck: stuck.remove(&id).unwrap_or_default(),
            }
        })
        .collect();
    ChainsView {
        country: catalog.countries.key(home).to_owned(),
        roots: roots
            .iter()
            .map(|&p| catalog.products.key(p).to_owned())
            .collect(),
        products,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::catalog::{SiteType, test_support};
    use crate::command::Command;
    use crate::money::Money;
    use crate::state::{GameSettings, SiteId, StartForm};

    fn game() -> Game {
        let mut catalog = test_support::production();
        // Iron is the end product of the small chain ore → iron.
        let iron = catalog.products.id("eisen").unwrap();
        catalog.products.get_mut(iron).kind = ProductKind::EndProduct;
        let catalog = Arc::new(catalog);
        let settings = GameSettings {
            seed: 3,
            start_year: 1900,
            start_country: catalog.countries.id("AAA").unwrap(),
            start_capital: Money::from_usd(20_000_000.0).unwrap(),
            start_form: StartForm::Workshop,
            company_name: "Hütte AG".into(),
            research_ahead_factor: 1.0,
            market_scale: 1.0,
            ai: Default::default(),
            ventures: 1.0,
            tariff_dynamics: 1.0,
            event_effects: true,
            person: Default::default(),
        };
        Game::new(catalog, settings).unwrap()
    }

    #[test]
    fn a_chain_runs_from_the_end_product_to_the_raw_material() {
        let mut game = game();
        let view = chains(&game);
        let product = |view: &ChainsView, key: &str| {
            view.products
                .iter()
                .find(|p| p.product == key)
                .cloned()
                .unwrap()
        };
        assert_eq!(view.country, "AAA");
        assert!(view.roots.contains(&"eisen".to_owned()));
        assert!(!view.roots.contains(&"erz".to_owned()));
        let iron = product(&view, "eisen");
        let ore = product(&view, "erz");
        let r = iron.recipe.as_ref().unwrap();
        // The furnace of the year 2000 is not invented yet: the known one is shown.
        assert_eq!(r.key, "eisen_schmelzen");
        assert_eq!(r.inputs, vec![("erz".to_owned(), 2.0)]);
        assert!(r.missing.is_empty());
        assert!(r.unit_cost_usd > 2.0 * ore.price_usd);
        let margin = (iron.price_usd - r.unit_cost_usd) / iron.price_usd;
        assert!((r.margin.unwrap() - margin).abs() < 1e-12);
        assert!(ore.recipe.as_ref().unwrap().extraction);
        assert!(!iron.makes && !iron.buys && !iron.sells);

        // An own furnace that waits for ore.
        let c = game.catalog().clone();
        game.apply(Command::FoundSite {
            country: c.countries.id("AAA").unwrap(),
            kind: SiteType::Factory,
        })
        .unwrap();
        let site = SiteId(u32::try_from(game.state().sites.len() - 1).unwrap());
        game.apply(Command::BuildFacility {
            site,
            facility: c.facilities.id("ofen").unwrap(),
            count: 1,
            size: crate::catalog::FacilitySize::Medium,
        })
        .unwrap();
        game.apply(Command::SetProduction {
            site,
            slot: 0,
            recipe: c.recipes.id("eisen_schmelzen"),
            utilization: 1.0,
        })
        .unwrap();
        let state = game.state_mut();
        let ore = c.products.id("erz").unwrap();
        let slot = &mut state.sites[site.index()].slots[0];
        slot.ready = state.date;
        slot.limit = Some(Limit::Input(ore));
        let iron = product(&chains(&game), "eisen");
        assert!(iron.makes);
        assert_eq!(
            iron.stuck,
            vec![("vorprodukt".to_owned(), Some("erz".to_owned()))]
        );
    }
}
