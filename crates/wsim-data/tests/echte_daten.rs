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
    let p1900 = population.value_at(1900.0);
    assert!(
        (40e6..47e6).contains(&p1900),
        "Deutschland 1900 in heutigen Grenzen: {p1900}"
    );
    let mid = population.value_at(1905.5);
    assert!(mid > p1900 && mid < population.value_at(1913.0));
}

#[test]
fn all_countries_with_complete_values() {
    let data = load_dir(&data_dir()).data.expect("data loads");
    let c = &data.catalog;
    assert_eq!(c.countries.len(), 197);
    for (id, country) in c.countries.iter() {
        let key = c.countries.key(id);
        for series in [
            &country.values.population,
            &country.values.gdp_per_capita_usd,
            &country.values.gini,
        ] {
            let points = series.points();
            assert!(
                points.first().unwrap().0 <= 1900 && points.last().unwrap().0 >= 2026,
                "{key}"
            );
        }
        assert!(country.area_km2 > 0.0, "{key}");
    }
    let world = |year: f64| {
        c.countries
            .iter()
            .map(|(_, k)| k.values.population.value_at(year))
            .sum::<f64>()
    };
    assert!(
        (1.55e9..1.70e9).contains(&world(1900.0)),
        "{}",
        world(1900.0)
    );
    assert!(
        (1.95e9..2.15e9).contains(&world(1930.0)),
        "{}",
        world(1930.0)
    );

    let deu = c.countries.get(c.countries.id("DEU").unwrap());
    let mut neighbors: Vec<&str> = deu.neighbors.iter().map(|&n| c.countries.key(n)).collect();
    neighbors.sort_unstable();
    assert_eq!(
        neighbors,
        [
            "AUT", "BEL", "CHE", "CZE", "DNK", "FRA", "LUX", "NLD", "POL"
        ]
    );
    assert_eq!(data.texts.get("land.DEU"), Some("Deutschland"));
}

#[test]
fn every_core_message_has_a_text() {
    let data = load_dir(&data_dir()).data.expect("data loads");
    for key in wsim_core::message::keys::ALL {
        assert!(
            data.texts.get(key).is_some(),
            "Text „{key}“ fehlt in data/texte/de/"
        );
    }
    assert!(data.texts.get("art.land").is_some());
}

/// Plausibility of the country model with the shipped data (rough historical ranges).
#[test]
fn country_model_is_plausible() {
    use wsim_core::calendar::Date;
    use wsim_core::country_model::compute;

    let data = load_dir(&data_dir()).data.expect("data loads");
    let c = &data.catalog;
    let at =
        |key: &str, year: i32| compute(c, c.countries.id(key).unwrap(), Date::first_of_year(year));
    let group = |key: &str| c.labor_groups.id(key).unwrap().index();
    use wsim_core::ids::Id;

    let usa = at("USA", 1900);
    let deu = at("DEU", 1900);
    let ind = at("IND", 1900);
    assert_eq!(usa.price_level, 1.0, "Bezugsland");
    assert!(ind.price_level < 0.6, "{}", ind.price_level);

    for (name, state, range) in [
        ("DEU", &deu, 2.0..8.0),
        ("USA", &usa, 3.0..12.0),
        ("IND", &ind, 0.2..2.0),
    ] {
        let wage = state.hourly_wage_usd[group("ungelernt")];
        assert!(
            range.contains(&wage),
            "Stundenlohn ungelernt {name} 1900: {wage}"
        );
        assert!(state.hourly_wage_usd[group("akademiker.chemie")] > wage);
        let pools: f64 = state.labor_pool.iter().sum();
        assert!(
            (pools / state.labor_force - 1.0).abs() < 1e-9,
            "{name}: {pools} vs {}",
            state.labor_force
        );
        assert!(state.income_quintiles_usd.windows(2).all(|w| w[0] < w[1]));
        let mean = state.income_quintiles_usd.iter().sum::<f64>() / 5.0;
        assert!((mean / (state.gdp_per_capita_usd * state.price_level) - 1.0).abs() < 1e-9);
    }

    assert!(deu.grid_share < at("DEU", 1930).grid_share);
    assert!(at("RUS", 1918).stability < 0.2);
    assert!((at("USA", 1930).corporate_tax - 0.12).abs() < 0.01);
    assert!(at("USA", 1913).automation_affinity > at("DEU", 1913).automation_affinity);
    let chemie = c.specializations.id("chemie").unwrap().index();
    assert!(deu.research_efficiency[chemie] > usa.research_efficiency[chemie]);
    // Germany has more chemistry workers relative to its labor force than average.
    let share = |s: &wsim_core::country_model::CountryState| {
        s.labor_pool[group("fachkraft.chemie")] / s.labor_force
    };
    assert!(share(&deu) > share(&at("FRA", 1900)));
}
