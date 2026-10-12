//! Trim levels of end products (MN2, docs/FORMELN.md): a site makes each end product in
//! one level; simple saves inputs and labor at lower quality, premium the reverse.

use crate::catalog::{Catalog, ProductKind, Trim};
use crate::command::{self, Command, CommandError};
use crate::ids::ProductId;
use crate::state::{CompanyId, GameState, SiteId};

/// Sets the trim level of an end product at a site (`SetTrim`).
pub(crate) fn set(
    state: &mut GameState,
    catalog: &Catalog,
    site: SiteId,
    product: ProductId,
    trim: Trim,
) -> Result<(), CommandError> {
    if !catalog.production_model.trims.enabled
        || catalog.products.get(product).kind != ProductKind::EndProduct
    {
        return Err(CommandError::NoTrim);
    }
    let s = &mut state.sites[site.index()];
    if trim == Trim::Standard {
        s.trims.remove(&product);
    } else {
        s.trims.insert(product, trim);
    }
    Ok(())
}

/// The level an AI company makes its consumer goods in: premium from a skill on, simple
/// up to another, else standard.
pub fn ai_choice(catalog: &Catalog, skill: f64) -> Trim {
    let m = &catalog.production_model.trims;
    if skill >= m.ai_premium_from {
        Trim::Premium
    } else if skill <= m.ai_simple_up_to {
        Trim::Simple
    } else {
        Trim::Standard
    }
}

/// AI companies at a month start: their consumer end products in the level of their
/// skill – the player's command.
pub(crate) fn ai_month(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    own: &[SiteId],
) {
    if !catalog.production_model.trims.enabled {
        return;
    }
    let Some(ai) = &state.companies[company.index()].ai else {
        return;
    };
    let wanted = ai_choice(catalog, ai.skill());
    for &site in own {
        let Some(s) = state.site(site) else { continue };
        let products: Vec<ProductId> = s
            .slots
            .iter()
            .filter_map(|sl| sl.recipe.map(|r| catalog.recipes.get(r).product))
            .filter(|&p| {
                let x = catalog.products.get(p);
                x.kind == ProductKind::EndProduct && x.consumer_demand.is_some()
            })
            .collect();
        for product in products {
            if crate::production::trim(&state.sites[site.index()], product) != wanted {
                let _ = command::execute(
                    state,
                    catalog,
                    company,
                    &Command::SetTrim {
                        site,
                        product,
                        trim: wanted,
                    },
                );
            }
        }
    }
}
