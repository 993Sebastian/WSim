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

/// A small production chain in country AAA (and an empty country BBB):
/// ore (extracted at a mine from deposit `grube`) → iron (smelted in a furnace).
pub fn production() -> Catalog {
    use super::{
        Branch, CountryModel, Deposit, Facility, GoodsGroup, LaborGroup, Product, ProductKind,
        ProductionModel, Qualification, Recipe, SiteType, Specialization, Technology,
        TransportClass, Unit, Usage,
    };
    use crate::ids::Id;
    use crate::money::Money;

    let mut c = with_countries(&["AAA", "BBB"]);
    let usd = |v: f64| Money::from_usd(v).expect("valid");
    let t = c
        .units
        .insert(
            "t",
            Unit {
                weight_kg: Some(1000.0),
            },
        )
        .expect("new");
    let branch = c.branches.insert("bergbau", Branch).expect("new");
    let group = c.goods_groups.insert("erze", GoodsGroup).expect("new");
    let transport = c
        .transport_classes
        .insert("schuettgut", TransportClass)
        .expect("new");
    let unskilled = c
        .qualifications
        .insert(
            "ungelernt",
            Qualification {
                rank: 1,
                has_specialization: false,
            },
        )
        .expect("new");
    let skilled = c
        .qualifications
        .insert(
            "fachkraft",
            Qualification {
                rank: 3,
                has_specialization: true,
            },
        )
        .expect("new");
    let metal = c
        .specializations
        .insert("metall", Specialization)
        .expect("new");
    let g_unskilled = c
        .labor_groups
        .insert(
            "ungelernt",
            LaborGroup {
                qualification: unskilled,
                specialization: None,
            },
        )
        .expect("new");
    let g_metal = c
        .labor_groups
        .insert(
            "fachkraft.metall",
            LaborGroup {
                qualification: skilled,
                specialization: Some(metal),
            },
        )
        .expect("new");
    c.country_model = CountryModel {
        price_reference: c.countries.id("AAA"),
        price_elasticity: 0.35,
        price_min: 0.2,
        price_max: 1.5,
        annual_hours: TimeSeries::new(vec![(1900, 2920.0)]).expect("valid"),
        qualification_shares: vec![(1_000.0, vec![0.8, 0.2])],
        wage_factors: vec![(1_000.0, vec![1.0, 2.0])],
        specialization_shares: vec![Vec::new(), vec![1.0]],
        ..CountryModel::default()
    };
    let product = |kind| Product {
        kind,
        branch,
        unit: t,
        usage: Usage::Industry,
        goods_group: group,
        transport_class: transport,
        weight_kg: 1000.0,
        heating_value_mwh: None,
        consumer_demand: None,
        state_demand: None,
        state_market: None,
        replaces: Vec::new(),
        provenance: Provenance::default(),
    };
    let ore = c
        .products
        .insert("erz", product(ProductKind::RawMaterial))
        .expect("new");
    let iron = c
        .products
        .insert("eisen", product(ProductKind::SemiFinished))
        .expect("new");
    let smelting = c
        .technologies
        .insert(
            "schmelzen",
            Technology {
                field: metal,
                invention_year: 1800,
                prerequisites: Vec::new(),
                research_effort: None,
                provenance: Provenance::default(),
            },
        )
        .expect("new");
    c.technologies
        .insert(
            "hochofen_2000",
            Technology {
                field: metal,
                invention_year: 2000,
                prerequisites: vec![smelting],
                research_effort: Some(1000.0),
                provenance: Provenance::default(),
            },
        )
        .expect("new");
    let facility = |site_type, investment, build_days, runs_per_day, technology| Facility {
        site_type,
        investment: usd(investment),
        build_days,
        runs_per_day,
        lifetime_years: 20,
        maintenance_share: 0.0365,
        automation_max: 0.5,
        technology,
        provenance: Provenance::default(),
    };
    let mine = c
        .facilities
        .insert(
            "mine",
            facility(SiteType::Extraction, 1_000_000.0, 10, 100.0, None),
        )
        .expect("new");
    let furnace = c
        .facilities
        .insert(
            "ofen",
            facility(SiteType::Factory, 2_000_000.0, 20, 50.0, Some(smelting)),
        )
        .expect("new");
    c.facilities
        .insert(
            "ofen_2000",
            facility(
                SiteType::Factory,
                1.0,
                1,
                1.0,
                c.technologies.id("hochofen_2000"),
            ),
        )
        .expect("new");
    c.recipes
        .insert(
            "erz_abbau",
            Recipe {
                product: ore,
                output: 1.0,
                by_products: Vec::new(),
                duration_days: 1,
                facility: mine,
                technology: None,
                extraction: true,
                inputs: Vec::new(),
                labor_hours: vec![(g_unskilled, 2.0)],
                energy_mwh: 0.0,
                base_quality: 50.0,
                provenance: Provenance::default(),
            },
        )
        .expect("new");
    c.recipes
        .insert(
            "eisen_schmelzen",
            Recipe {
                product: iron,
                output: 1.0,
                by_products: Vec::new(),
                duration_days: 2,
                facility: furnace,
                technology: Some(smelting),
                extraction: false,
                inputs: vec![(ore, 2.0)],
                labor_hours: vec![(g_unskilled, 1.0), (g_metal, 1.0)],
                energy_mwh: 0.0,
                base_quality: 60.0,
                provenance: Provenance::default(),
            },
        )
        .expect("new");
    c.deposits
        .insert(
            "grube",
            Deposit {
                country: c.countries.id("AAA").expect("exists"),
                resource: ore,
                reserve: Some(10_000.0),
                discovered: None,
                development_cost: usd(500_000.0),
                development_days: 30,
                max_output_per_year: 5_000.0,
                cost_factor: 1.0,
                provenance: Provenance::default(),
            },
        )
        .expect("new");
    c.production_model = ProductionModel {
        site_cost: vec![
            (SiteType::Extraction, usd(100_000.0)),
            (SiteType::Factory, usd(200_000.0)),
        ],
        ..ProductionModel::default()
    };
    let _ = (g_unskilled.index(), g_metal.index());
    c
}
