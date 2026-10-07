//! Invented product names (M42; rules in docs/FORMELN.md).
//!
//! Companies name their end products from the name parts of the data; real products,
//! models and brands never come up. The random numbers come from a stream per company
//! and product, so a name shifts no other random numbers.

use std::collections::BTreeSet;

use crate::catalog::{Catalog, NamePattern, NamingStyle};
use crate::ids::{Id, ProductId};
use crate::rng::{SimRng, Stream};
use crate::state::{CompanyId, GameState};

/// Longest allowed product name in characters.
pub const MAX_LENGTH: usize = 40;

/// Tries for a free name before the last one gets a number.
const ATTEMPTS: usize = 30;

/// Names compare without regard to case.
pub fn same_name(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// The stem of a name: its first word.
fn stem(name: &str) -> &str {
    name.split_whitespace().next().unwrap_or("")
}

fn company_id(index: usize) -> CompanyId {
    CompanyId(u32::try_from(index).unwrap_or(u32::MAX))
}

/// Whether a company other than `own` already calls `product` so.
pub fn taken(state: &GameState, product: ProductId, name: &str, own: CompanyId) -> bool {
    state.companies.iter().enumerate().any(|(i, c)| {
        company_id(i) != own
            && c.product_names
                .get(&product)
                .is_some_and(|n| same_name(n, name))
    })
}

/// A new name for a product of a company; `None` for products without names.
pub fn generate(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    product: ProductId,
) -> Option<String> {
    suggestions(catalog, state, company, product, 1)
        .into_iter()
        .next()
}

/// `count` different free names for a product of a company, the first the one `generate`
/// gives; empty for products without names.
pub fn suggestions(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    product: ProductId,
    count: usize,
) -> Vec<String> {
    let naming = &catalog.product_naming;
    let Some(style) = naming.style(catalog, product) else {
        return Vec::new();
    };
    let year = state.date.year();
    let patterns: Vec<&NamePattern> = style.patterns.iter().filter(|p| p.applies(year)).collect();
    if patterns.is_empty() || style.stems.is_empty() {
        return Vec::new();
    }
    // House brands: stems of the company's own names in this style.
    let own: Vec<&str> = state
        .company(company)
        .map(|c| {
            c.product_names
                .iter()
                .filter(|(p, _)| {
                    **p != product
                        && naming
                            .style(catalog, **p)
                            .is_some_and(|s| s.key == style.key)
                })
                .map(|(_, n)| stem(n))
                .filter(|s| !s.is_empty())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect()
        })
        .unwrap_or_default();
    // Stems another company uses belong to it, as long as free ones are left.
    let foreign: Vec<&str> = state
        .companies
        .iter()
        .enumerate()
        .filter(|(i, _)| company_id(*i) != company)
        .flat_map(|(_, c)| c.product_names.values().map(|n| stem(n)))
        .collect();
    let free: Vec<&str> = style
        .stems
        .iter()
        .map(String::as_str)
        .filter(|s| !foreign.iter().any(|f| same_name(f, s)))
        .collect();
    let all: Vec<&str> = style.stems.iter().map(String::as_str).collect();
    let mut rng = SimRng::for_stream(
        state.settings.seed,
        Stream::ProductName {
            company: company.0,
            product: u32::try_from(product.index()).unwrap_or(u32::MAX),
        },
    );
    let usable = |names: &[String], name: &str| {
        !name.is_empty()
            && name.chars().count() <= MAX_LENGTH
            && !naming.is_excluded(name)
            && !taken(state, product, name, company)
            && !names.iter().any(|n| same_name(n, name))
    };
    let mut names: Vec<String> = Vec::new();
    for _ in 0..count {
        let mut last = String::new();
        let mut found = None;
        for _ in 0..ATTEMPTS {
            let stem = if !own.is_empty() && rng.chance(naming.house_brand) {
                pick(&mut rng, &own)
            } else if free.is_empty() {
                pick(&mut rng, &all)
            } else {
                pick(&mut rng, &free)
            };
            let pattern = pick(&mut rng, &patterns);
            last = fill(&mut rng, style, &pattern.text, stem);
            if usable(&names, &last) {
                found = Some(last.clone());
                break;
            }
        }
        let name = found.unwrap_or_else(|| {
            (2..)
                .map(|n| format!("{last} {n}"))
                .find(|n| usable(&names, n))
                .expect("some number is free")
        });
        names.push(name);
    }
    names
}

/// The successor model of a product's name (B1, docs/FORMELN.md): the last number in the
/// name steps up to the next larger number of the style, else the generation mark steps
/// on or one is added ("Kelvor M80" → "Kelvor M90", "Kelvor Super" → "Kelvor Super II");
/// a name another company uses for the product is skipped. `None` without a name, for
/// styles without successor models and once the generations run out.
pub fn successor(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    product: ProductId,
) -> Option<String> {
    let naming = &catalog.product_naming;
    let style = naming.style(catalog, product)?;
    if style.successors.is_empty() {
        return None;
    }
    let mut name = state.company(company)?.product_names.get(&product)?.clone();
    for _ in 0..ATTEMPTS {
        name = next_model(style, &name)?;
        if name.chars().count() <= MAX_LENGTH
            && !naming.is_excluded(&name)
            && !taken(state, product, &name, company)
        {
            return Some(name);
        }
    }
    None
}

/// One step of `successor`, without checking the name.
fn next_model(style: &NamingStyle, name: &str) -> Option<String> {
    let mut words: Vec<String> = name.split_whitespace().map(str::to_owned).collect();
    let last = words.last()?.clone();
    if let Some(i) = style.successors.iter().position(|g| same_name(g, &last)) {
        let next = style.successors.get(i + 1)?;
        *words.last_mut()? = next.clone();
        return Some(words.join(" "));
    }
    // Only the last word with a number steps up.
    let stepped = words
        .iter_mut()
        .rev()
        .find(|w| w.chars().any(|c| c.is_ascii_digit()))
        .and_then(|w| {
            let start = w.find(|c: char| c.is_ascii_digit())?;
            let digits: String = w[start..]
                .chars()
                .take_while(char::is_ascii_digit)
                .collect();
            let number: u32 = digits.parse().ok()?;
            let next = style
                .numbers
                .iter()
                .copied()
                .filter(|&n| n > number)
                .min()?;
            *w = format!("{}{next}{}", &w[..start], &w[start + digits.len()..]);
            Some(())
        });
    if stepped.is_none() {
        words.push(style.successors.first()?.clone());
    }
    Some(words.join(" "))
}

fn pick<T: Copy>(rng: &mut SimRng, list: &[T]) -> T {
    let n = u64::try_from(list.len()).unwrap_or(1);
    list[usize::try_from(rng.below(n)).unwrap_or(0)]
}

fn pick_text(rng: &mut SimRng, list: &[String]) -> String {
    if list.is_empty() {
        return String::new();
    }
    let n = u64::try_from(list.len()).unwrap_or(1);
    list[usize::try_from(rng.below(n)).unwrap_or(0)].clone()
}

/// The pattern with its placeholders replaced, each occurrence drawn on its own.
fn fill(rng: &mut SimRng, style: &NamingStyle, pattern: &str, stem: &str) -> String {
    let mut text = pattern.replace("{stamm}", stem);
    while text.contains("{zahl}") {
        let number = if style.numbers.is_empty() {
            String::new()
        } else {
            pick(rng, &style.numbers).to_string()
        };
        text = text.replacen("{zahl}", &number, 1);
    }
    while text.contains("{buchstabe}") {
        let letter = pick_text(rng, &style.letters);
        text = text.replacen("{buchstabe}", &letter, 1);
    }
    while text.contains("{zusatz}") {
        let addition = pick_text(rng, &style.additions);
        text = text.replacen("{zusatz}", &addition, 1);
    }
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// End products with names a company makes (a facility with a recipe for it, also under
/// construction) or offers, in ID order.
pub fn named_products(catalog: &Catalog, state: &GameState, company: CompanyId) -> Vec<ProductId> {
    let mut products = BTreeSet::new();
    for s in state.sites.iter().filter(|s| s.owner == company) {
        let made = s
            .slots
            .iter()
            .filter_map(|sl| sl.recipe.map(|r| catalog.recipes.get(r).product));
        products.extend(made.chain(s.offers.keys().copied()));
    }
    products
        .into_iter()
        .filter(|&p| catalog.product_naming.style(catalog, p).is_some())
        .collect()
}

/// Names every product a new company makes (at its founding, not a command).
pub(crate) fn name_new_company(catalog: &Catalog, state: &mut GameState, company: CompanyId) {
    for product in named_products(catalog, state, company) {
        let unnamed = state
            .company(company)
            .is_some_and(|c| !c.product_names.contains_key(&product));
        if !unnamed {
            continue;
        }
        if let Some(name) = generate(catalog, state, company, product)
            && let Some(c) = state.company_mut(company)
        {
            c.product_names.insert(product, name);
        }
    }
}
