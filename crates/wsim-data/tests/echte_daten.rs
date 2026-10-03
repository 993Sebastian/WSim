//! The shipped data in `data/` must load without errors or warnings.

use std::path::Path;

use wsim_core::catalog::ProductKind;
use wsim_core::money::Money;
use wsim_data::load_dir;

fn data_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

#[test]
fn shipped_data_loads_cleanly() {
    let outcome = load_dir(&data_dir());
    let findings: Vec<String> = outcome
        .report
        .findings()
        .iter()
        .map(ToString::to_string)
        .collect();
    assert!(
        findings.is_empty(),
        "Befunde in data/:\n{}",
        findings.join("\n\n")
    );
    assert!(outcome.data.is_some());
}

#[test]
fn chain_one_is_complete() {
    let data = load_dir(&data_dir()).data.expect("data loads");
    let c = &data.catalog;

    for key in [
        "eisenerz",
        "kohle",
        "roheisen",
        "stahl",
        "blech",
        "draht",
        "stabstahl",
    ] {
        assert!(c.products.id(key).is_some(), "Produkt {key} fehlt");
    }
    let erz = c.products.id("eisenerz").unwrap();
    assert_eq!(c.products.get(erz).kind, ProductKind::RawMaterial);
    assert_eq!(c.products.get(erz).weight_kg, 1000.0);

    let recipe = c
        .recipes
        .get(c.recipes.id("roheisen_kokshochofen").unwrap());
    assert_eq!(recipe.product, c.products.id("roheisen").unwrap());
    assert!(recipe.inputs.contains(&(erz, 1.7)));
    let metal = c.labor_groups.id("fachkraft.metall").unwrap();
    assert!(recipe.labor_hours.contains(&(metal, 1.8)));

    let furnace = c.facilities.get(c.facilities.id("hochofen").unwrap());
    assert_eq!(furnace.investment, Money::from_usd(60_000_000.0).unwrap());

    // Every deposit of chain 1 lies in a country that exists.
    assert!(
        c.deposits
            .iter()
            .all(|(_, d)| c.countries.key(d.country).len() == 3)
    );
}

#[test]
fn labor_groups_follow_qualifications() {
    let data = load_dir(&data_dir()).data.expect("data loads");
    let c = &data.catalog;
    // ungelernt + angelernt + (fachkraft + akademiker) × 9 Fachrichtungen
    assert_eq!(c.labor_groups.len(), 2 + 2 * c.specializations.len());
    assert!(c.labor_groups.id("ungelernt").is_some());
    assert!(c.labor_groups.id("akademiker.kaufmaennisch").is_some());
    assert!(c.labor_groups.id("ungelernt.metall").is_none());
}

#[test]
fn country_values_interpolate() {
    let data = load_dir(&data_dir()).data.expect("data loads");
    let c = &data.catalog;
    let deu = c.countries.get(c.countries.id("DEU").unwrap());
    let population = &deu.values.population;
    assert_eq!(population.value_at(1900.0), 44_000_000.0);
    let mid = population.value_at(1906.5);
    assert!(mid > 44_000_000.0 && mid < 52_600_000.0);
}
