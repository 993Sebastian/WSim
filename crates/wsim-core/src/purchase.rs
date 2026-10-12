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
