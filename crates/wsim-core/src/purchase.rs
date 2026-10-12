//! Works and their branches (FW1, docs/FORMELN.md): under mandatory research a works makes
//! the products of one branch. The first product fixes the branch; another branch is
//! refused until the company builds a new works.

use crate::catalog::Catalog;
use crate::command::CommandError;
use crate::ids::{BranchId, ProductId};
use crate::state::{GameState, Site};

/// The branch a works makes its products in when `product` is made there (FW1). Without
/// mandatory research every branch is allowed, as before.
pub fn works_branch(
    state: &GameState,
    catalog: &Catalog,
    site: &Site,
    product: ProductId,
) -> Result<BranchId, CommandError> {
    let branch = catalog.products.get(product).branch;
    if state.settings.research_mandatory
        && let Some(works) = site.branch
        && works != branch
    {
        return Err(CommandError::OtherBranch(
            catalog.branches.key(works).to_owned(),
        ));
    }
    Ok(branch)
}

/// What a product needs before a works can make it (the recipe, the company's knowledge of
/// the product and its technology, the works' branch): the recipe and the branch.
pub fn check_product(
    state: &GameState,
    catalog: &Catalog,
    actor: crate::state::CompanyId,
    site: &Site,
    product: ProductId,
) -> Result<(crate::ids::RecipeId, BranchId), CommandError> {
    let recipe = crate::product_research::main_recipe(catalog, product)
        .ok_or_else(|| CommandError::NoRecipe(catalog.products.key(product).to_owned()))?;
    let r = catalog.recipes.get(recipe);
    if !state.knows_product(catalog, actor, product) {
        return Err(CommandError::ProductUnknown(
            catalog.products.key(product).to_owned(),
        ));
    }
    if let Some(t) = r.technology.filter(|&t| !state.knows(catalog, actor, t)) {
        return Err(crate::command::unknown_technology(catalog, t));
    }
    let branch = works_branch(state, catalog, site, product)?;
    Ok((recipe, branch))
}
