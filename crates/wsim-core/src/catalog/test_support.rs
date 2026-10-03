//! Small catalogs for unit tests of the core.

use super::{Catalog, Continent, Country, CountryProfile, CountryValues, GeoPoint, Provenance};
use crate::time_series::TimeSeries;

fn country(catalog: &Catalog, population: &[(i32, f64)]) -> Country {
    Country {
        continent: catalog.continents.id("europa").expect("continent exists"),
        area_km2: 100_000.0,
        capital: GeoPoint {
            lat: 50.0,
            lon: 10.0,
        },
        landlocked: false,
        neighbors: Vec::new(),
        values: CountryValues {
            population: TimeSeries::new(population.to_vec()).expect("valid"),
            gdp_per_capita_usd: TimeSeries::new(vec![(1900, 5_000.0), (1930, 8_000.0)])
                .expect("valid"),
            gini: TimeSeries::new(vec![(1900, 0.45)]).expect("valid"),
            stability: None,
            corporate_tax: None,
            dividend_tax: None,
        },
        profile: CountryProfile::default(),
        provenance: Provenance::default(),
    }
}

/// A catalog with the given countries (keys) in this order.
pub fn with_countries(keys: &[&str]) -> Catalog {
    let mut catalog = Catalog {
        data_version: 1,
        ..Catalog::default()
    };
    catalog.continents.insert("europa", Continent);
    for (i, key) in keys.iter().enumerate() {
        let base = 1_000_000.0 * (i as f64 + 1.0);
        let c = country(
            &catalog,
            &[(1900, base), (1910, base * 1.1), (1930, base * 1.3)],
        );
        catalog.countries.insert(key, c);
    }
    catalog
}

pub fn sample() -> Catalog {
    with_countries(&["AAA", "BBB"])
}
