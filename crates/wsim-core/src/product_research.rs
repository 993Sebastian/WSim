//! Research per product (docs/FORMELN.md, Q6): the player researches products, not
//! technologies. The technologies of the data stay behind them as building blocks –
//! effort, invention year, patents, licenses and start-ups work on them as before.

use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::ids::{Id, ProductId, RecipeId, TechnologyId};
use crate::state::{CompanyId, GameState};

/// The recipe that stands for a product: its first recipe that does not extract, else its
/// first one.
pub fn main_recipe(catalog: &Catalog, product: ProductId) -> Option<RecipeId> {
    let mut first = None;
    for r in catalog.recipes.ids() {
        let recipe = catalog.recipes.get(r);
        if recipe.product != product {
            continue;
        }
        if !recipe.extraction {
            return Some(r);
        }
        first.get_or_insert(r);
    }
    first
}

/// The technologies of a recipe and its facility with all their prerequisites, every
/// prerequisite before the technology that needs it.
fn recipe_technologies(catalog: &Catalog, recipe: RecipeId) -> Vec<TechnologyId> {
    fn visit(catalog: &Catalog, t: TechnologyId, out: &mut Vec<TechnologyId>) {
        if out.contains(&t) {
            return;
        }
        for &p in &catalog.technologies.get(t).prerequisites {
            visit(catalog, p, out);
        }
        out.push(t);
    }
    let r = catalog.recipes.get(recipe);
    let mut out = Vec::new();
    for t in [catalog.facilities.get(r.facility).technology, r.technology]
        .into_iter()
        .flatten()
    {
        visit(catalog, t, &mut out);
    }
    out
}

/// Technologies every company knows under mandatory research (FW1): those of the basic
/// products' main recipes and of all extraction recipes, prerequisites included.
pub fn basic_technologies(catalog: &Catalog) -> std::collections::BTreeSet<TechnologyId> {
    let mut out = std::collections::BTreeSet::new();
    for &p in &catalog.basic_products {
        out.extend(technologies(catalog, p));
    }
    for r in catalog.recipes.ids() {
        if catalog.recipes.get(r).extraction {
            out.extend(recipe_technologies(catalog, r));
        }
    }
    out
}

/// The technologies of a recipe and its facility a company still lacks.
pub fn recipe_missing(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    recipe: RecipeId,
) -> Vec<TechnologyId> {
    recipe_technologies(catalog, recipe)
        .into_iter()
        .filter(|&t| !state.knows(catalog, company, t))
        .collect()
}

/// Under mandatory research (FW1) a company keeps the know-how for what its sites make
/// when it starts: the start set, the start cast and historical companies.
pub(crate) fn grant_running(state: &mut GameState, catalog: &Catalog, company: CompanyId) {
    if !state.settings.research_mandatory {
        return;
    }
    let recipes: Vec<RecipeId> = state
        .sites
        .iter()
        .filter(|s| s.held_by(company))
        .flat_map(|s| &s.slots)
        .filter_map(|sl| sl.recipe)
        .collect();
    let mut known = Vec::new();
    for &r in &recipes {
        known.extend(recipe_technologies(catalog, r));
    }
    let c = &mut state.companies[company.index()];
    c.technologies.extend(known);
    c.known_products
        .extend(recipes.iter().map(|&r| catalog.recipes.get(r).product));
}

/// The technologies a product needs (its main recipe), prerequisites first.
pub fn technologies(catalog: &Catalog, product: ProductId) -> Vec<TechnologyId> {
    main_recipe(catalog, product).map_or_else(Vec::new, |r| recipe_technologies(catalog, r))
}

/// The technologies of a product a company still lacks, prerequisites first.
pub fn missing(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    product: ProductId,
) -> Vec<TechnologyId> {
    technologies(catalog, product)
        .into_iter()
        .filter(|&t| !state.knows(catalog, company, t))
        .collect()
}

/// Whether a company may make a product: it knows a recipe for it and the recipe's
/// facility.
pub fn known(catalog: &Catalog, state: &GameState, company: CompanyId, product: ProductId) -> bool {
    state.knows_product(catalog, company, product)
        && catalog.recipes.ids().any(|r| {
            let recipe = catalog.recipes.get(r);
            recipe.product == product
                && [
                    catalog.facilities.get(recipe.facility).technology,
                    recipe.technology,
                ]
                .into_iter()
                .flatten()
                .all(|t| state.knows(catalog, company, t))
        })
}

/// Whether a technology exists in the world on `date`: its historical year has come or a
/// company invented it early.
fn invented(catalog: &Catalog, state: &GameState, t: TechnologyId, date: Date) -> bool {
    catalog.technologies.get(t).invention_year <= date.year() || state.inventions.get(t).is_some()
}

/// Whether a product exists in the world: some recipe for it uses only invented
/// technologies. It can be bought then.
pub fn known_in_world(catalog: &Catalog, state: &GameState, product: ProductId) -> bool {
    catalog.recipes.ids().any(|r| {
        catalog.recipes.get(r).product == product
            && recipe_technologies(catalog, r)
                .into_iter()
                .all(|t| invented(catalog, state, t, state.date))
    })
}

/// Inputs of a product's main recipe that do not exist in the world yet.
pub fn missing_inputs(catalog: &Catalog, state: &GameState, product: ProductId) -> Vec<ProductId> {
    main_recipe(catalog, product).map_or_else(Vec::new, |r| {
        catalog
            .recipes
            .get(r)
            .inputs
            .iter()
            .map(|&(p, _)| p)
            .filter(|&p| !known_in_world(catalog, state, p))
            .collect()
    })
}

/// Whether a company can start researching a product: it does not know it yet, there is
/// something to research, and every input of the product exists in the world.
pub fn can_research(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    product: ProductId,
) -> bool {
    !known(catalog, state, company, product)
        && (!missing(catalog, state, company, product).is_empty()
            || own_effort(catalog, state, company, product).is_some())
        && missing_inputs(catalog, state, product).is_empty()
}

/// The technology to work on next for a product: the first missing one whose
/// prerequisites the company knows.
pub fn next(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    product: ProductId,
) -> Option<TechnologyId> {
    missing(catalog, state, company, product)
        .into_iter()
        .find(|&t| crate::research::can_research(catalog, state, company, t))
}

/// Under mandatory research (FW1): the research field and the points a company needs to
/// learn a product it may not make yet, once its technologies are known – as much as the
/// first development level. `None` when there is nothing of that kind to learn.
pub fn own_effort(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    product: ProductId,
) -> Option<(crate::ids::SpecializationId, f64)> {
    if state.knows_product(catalog, company, product) {
        return None;
    }
    let (field, base) = crate::development::basis(catalog, product)?;
    Some((
        field,
        base * catalog.research_model.development.effort_share,
    ))
}

/// Research points a product takes in all and those collected for it so far.
pub fn effort(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    product: ProductId,
    date: Date,
) -> (f64, f64) {
    let collected = state.company(company).map_or(0.0, |c| {
        missing(catalog, state, company, product)
            .iter()
            .map(|t| c.research.get(t).copied().unwrap_or(0.0))
            .sum()
    });
    let needed: f64 = missing(catalog, state, company, product)
        .into_iter()
        .filter_map(|t| crate::research::effort(catalog, state, t, date))
        .map(|e| e.points)
        .sum();
    let own = own_effort(catalog, state, company, product).map_or(0.0, |(_, e)| e);
    let own_collected = state
        .company(company)
        .and_then(|c| c.product_points.get(&product).copied())
        .unwrap_or(0.0);
    (needed + own, collected + own_collected)
}

/// Every day before the research: a center that researches a product works on its next
/// technology; once the product is known the center is free again (with a message for
/// the player).
pub(crate) fn steer(state: &mut GameState, catalog: &Catalog) -> Vec<crate::message::Message> {
    use crate::message::{Message, MessageKind, Param, keys};
    let mut news = Vec::new();
    for index in 0..state.sites.len() {
        let Some(product) = state.sites[index].research_product else {
            continue;
        };
        let owner = state.sites[index].owner;
        if known(catalog, state, owner, product) {
            state.sites[index].research_product = None;
            state.sites[index].research = None;
            if state.is_main(owner) {
                news.push(
                    Message::new(MessageKind::Success, keys::PRODUCT_RESEARCHED).with(
                        "produkt",
                        Param::TextKey(format!("produkt.{}", catalog.products.key(product))),
                    ),
                );
            }
            continue;
        }
        let current = state.sites[index].research;
        if current.is_none_or(|t| state.knows(catalog, owner, t)) {
            state.sites[index].research = next(catalog, state, owner, product);
        }
    }
    news
}

/// Ids of all products, for views.
pub fn all(catalog: &Catalog) -> impl Iterator<Item = ProductId> + '_ {
    (0..catalog.products.len()).map(ProductId::from_index)
}
