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
        members: Vec::new(),
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

/// Like `with_countries`, but `region` (a new key or one of the countries) takes over
/// `members` (M34): they leave the catalog and become its aliases.
pub fn with_region(keys: &[&str], region: &str, members: &[&str]) -> Catalog {
    let mut catalog = with_countries(
        &keys
            .iter()
            .copied()
            .filter(|k| *k == region || !members.contains(k))
            .collect::<Vec<_>>(),
    );
    if catalog.countries.id(region).is_none() {
        let c = country(&catalog, &[(1900, 1_000_000.0)]);
        catalog.countries.insert(region, c);
    }
    let id = catalog.countries.id(region).expect("region exists");
    catalog.countries.get_mut(id).members = members.iter().map(|m| (*m).to_owned()).collect();
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
        .insert("schuettgut", TransportClass { cost_factor: 1.0 })
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
    let product = |kind, reference: f64| Product {
        kind,
        branch,
        unit: t,
        usage: Usage::Industry,
        goods_group: group,
        transport_class: transport,
        reference_price: usd(reference),
        weight_kg: 1000.0,
        heating_value_mwh: None,
        consumer_demand: None,
        state_demand: None,
        state_market: None,
        replaces: Vec::new(),
        output_index: None,
        rent_share: 0.0,
        provenance: Provenance::default(),
    };
    let ore = c
        .products
        .insert("erz", product(ProductKind::RawMaterial, 10.0))
        .expect("new");
    let iron = c
        .products
        .insert("eisen", product(ProductKind::SemiFinished, 100.0))
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
        area_ha: None,
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
                reserve: Some(2_000_000.0),
                discovered: None,
                development_cost: usd(500_000.0),
                development_days: 30,
                // A mine of 100 t a day all year (the yearly output spreads over the year).
                max_output_per_year: 36_500.0,
                cost_factor: 1.0,
                provenance: Provenance::default(),
            },
        )
        .expect("new");
    // Consumer goods for market tests (no production recipes; tests put them in stock).
    use super::{ConsumerDemand, ConsumptionType, NeedClass, StateDemand, StateMarketOffer};
    let consumer = |kind, reference: f64, demand: ConsumerDemand| Product {
        consumer_demand: Some(demand),
        usage: Usage::Consumer,
        ..product(kind, reference)
    };
    c.products
        .insert(
            "brot",
            consumer(
                ProductKind::EndProduct,
                2.0,
                ConsumerDemand {
                    need_class: NeedClass::Basic,
                    consumption: ConsumptionType::Consumable {
                        per_capita_per_year: 200.0,
                    },
                    purchase_threshold: 20.0,
                    price_sensitivity: 1.5,
                    income_sensitivity: 1.0,
                    seasonality: None,
                    needs_grid: false,
                },
            ),
        )
        .expect("new");
    let carriage = c
        .products
        .insert(
            "kutsche",
            Product {
                state_market: Some(StateMarketOffer {
                    price: usd(500.0),
                    available_from: None,
                    available_until: None,
                }),
                ..consumer(
                    ProductKind::EndProduct,
                    500.0,
                    ConsumerDemand {
                        need_class: NeedClass::Luxury,
                        consumption: ConsumptionType::Durable {
                            service_life_years: 15.0,
                            max_ownership: 0.1,
                        },
                        purchase_threshold: 2.0,
                        price_sensitivity: 1.5,
                        income_sensitivity: 1.5,
                        seasonality: None,
                        needs_grid: false,
                    },
                )
            },
        )
        .expect("new");
    c.products
        .insert(
            "rad",
            Product {
                replaces: vec![carriage],
                ..consumer(
                    ProductKind::EndProduct,
                    100.0,
                    ConsumerDemand {
                        need_class: NeedClass::Durable,
                        consumption: ConsumptionType::Durable {
                            service_life_years: 10.0,
                            max_ownership: 0.5,
                        },
                        purchase_threshold: 1.0,
                        price_sensitivity: 2.0,
                        income_sensitivity: 2.0,
                        seasonality: None,
                        needs_grid: false,
                    },
                )
            },
        )
        .expect("new");
    let iron_product = c.products.id("eisen").expect("exists");
    c.products.get_mut(iron_product).state_demand = Some(StateDemand {
        per_million_gdp: 0.5,
        war_factor: 1.0,
        index: None,
    });

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

/// The production catalog with freight: AAA (50° N, 10° E) and BBB (50° N, 20° E) are
/// coastal neighbors; carts, rail and steamships carry bulk goods.
pub fn trading() -> Catalog {
    use super::{Vehicle, Way};

    let mut c = production();
    for (key, lon, neighbor) in [("AAA", 10.0, "BBB"), ("BBB", 20.0, "AAA")] {
        let other = c.countries.id(neighbor).expect("exists");
        let id = c.countries.id(key).expect("exists");
        let country = c.countries.get_mut(id);
        country.capital = GeoPoint { lat: 50.0, lon };
        country.neighbors = vec![other];
    }
    let bulk = c.transport_classes.id("schuettgut").expect("exists");
    let series = |v: f64| TimeSeries::new(vec![(1900, v)]).expect("valid");
    for (key, way, cost, speed) in [
        ("karren", Way::Terrain, 2.0, 30.0),
        ("bahn", Way::Rail, 0.15, 200.0),
        ("dampfer", Way::Sea, 0.015, 400.0),
    ] {
        c.vehicles.insert(
            key,
            Vehicle {
                way,
                available_from: 1800,
                available_until: None,
                classes: vec![bulk],
                cost_per_tkm: series(cost),
                km_per_day: series(speed),
                provenance: Provenance::default(),
            },
        );
    }
    c
}

/// The production catalog with research: a laboratory with 10 researcher posts and a
/// turbine invented in 1902 (300 points, field metal).
pub fn research() -> Catalog {
    use super::{Facility, SiteType, Technology};
    use crate::money::Money;

    let mut c = production();
    let metal = c.specializations.id("metall").expect("exists");
    let group = c.labor_groups.id("fachkraft.metall").expect("exists");
    c.research_model.researchers = vec![Some(group)];
    c.facilities.insert(
        "labor",
        Facility {
            site_type: SiteType::ResearchCenter,
            investment: Money::from_usd(100_000.0).expect("valid"),
            build_days: 10,
            runs_per_day: 10.0,
            lifetime_years: 20,
            maintenance_share: 0.02,
            automation_max: 0.0,
            technology: None,
            area_ha: None,
            provenance: Provenance::default(),
        },
    );
    let smelting = c.technologies.id("schmelzen").expect("exists");
    c.technologies.insert(
        "turbine",
        Technology {
            field: metal,
            invention_year: 1902,
            prerequisites: vec![smelting],
            research_effort: Some(300.0),
            provenance: Provenance::default(),
        },
    );
    c
}

/// The production catalog where smelting needs 2 MWh per run, and a power plant that
/// makes 1 MWh from 0.1 t ore (100 runs per day, built in one day).
pub fn power() -> Catalog {
    use super::{Facility, Product, ProductKind, Recipe, SiteType};
    use crate::money::Money;

    let mut c = production();
    let ore = c.products.id("erz").expect("exists");
    let template = c.products.get(ore).clone();
    let power = c
        .products
        .insert(
            "strom",
            Product {
                kind: ProductKind::Energy,
                weight_kg: 0.0,
                ..template
            },
        )
        .expect("new");
    c.production_model.electricity = Some(power);
    let plant = c
        .facilities
        .insert(
            "kraftwerk",
            Facility {
                site_type: SiteType::PowerPlant,
                investment: Money::from_usd(200_000.0).expect("valid"),
                build_days: 1,
                runs_per_day: 100.0,
                lifetime_years: 20,
                maintenance_share: 0.02,
                automation_max: 0.0,
                technology: None,
                area_ha: None,
                provenance: Provenance::default(),
            },
        )
        .expect("new");
    let unskilled = c.labor_groups.id("ungelernt").expect("exists");
    c.recipes.insert(
        "strom_erzeugen",
        Recipe {
            product: power,
            output: 1.0,
            by_products: Vec::new(),
            duration_days: 1,
            facility: plant,
            technology: None,
            extraction: false,
            inputs: vec![(ore, 0.1)],
            labor_hours: vec![(unskilled, 0.1)],
            energy_mwh: 0.0,
            base_quality: 50.0,
            provenance: Provenance::default(),
        },
    );
    let smelting = c.recipes.id("eisen_schmelzen").expect("exists");
    c.recipes.get_mut(smelting).energy_mwh = 2.0;
    c
}

/// The research catalog with managers (MA1, MA2): five functions, the four levels, heads
/// and specialists for works and mines, heads only in laboratories; salaries follow the
/// metal workers, whose qualification also counts as academics. One name group for
/// everybody.
pub fn management() -> Catalog {
    use super::{
        ConcernModel, ManagementFunction, ManagementLevel, ManagementModel, ManagerPoolModel,
        NameGroup, SiteType, SkillModel,
    };
    use crate::decision::Topic;

    let mut c = research();
    let function = |key: &str, topics: Vec<Topic>| ManagementFunction {
        key: key.to_owned(),
        topics,
    };
    let level = |key: &str, check_days, salary_specialist, salary_head| ManagementLevel {
        key: key.to_owned(),
        check_days,
        salary_specialist,
        salary_head,
        budget_specialist: (0.02, 0.05),
        budget_head: (0.05, 0.10),
    };
    c.management = ManagementModel {
        functions: vec![
            function(
                "produktion",
                vec![
                    Topic::Production,
                    Topic::Overcapacity,
                    Topic::Idle,
                    Topic::Restart,
                    Topic::Expansion,
                ],
            ),
            function("einkauf_lager", vec![Topic::Purchase, Topic::OwnSupply]),
            function("vertrieb_marketing", vec![Topic::Sale]),
            function("personal", vec![Topic::Wage]),
            function("forschung", vec![Topic::Research, Topic::Development]),
        ],
        levels: vec![
            level("standort", 7, 1.5, 2.5),
            level("land", 30, 3.0, 4.0),
            level("kontinent", 30, 5.0, 7.0),
            level("vorstand", 91, 10.0, 15.0),
        ],
        specialists: vec![
            (SiteType::Factory, vec![0, 1, 2, 3]),
            (SiteType::Extraction, vec![0, 2]),
        ],
        routine_topics: vec![
            Topic::Production,
            Topic::Sale,
            Topic::Purchase,
            Topic::OwnSupply,
            Topic::Wage,
        ],
        budget_floor: (1.0, 3.0),
        concerns: ConcernModel {
            deadline_days: 30,
            block_days: 90,
            open_per_position: 2,
            followup_days: 91,
            estimate_error: 0.5,
            recommend_base: 0.5,
        },
        head_discount: 0.2,
        notice_base: 0.5,
        salary_group: c.labor_groups.id("fachkraft.metall"),
        severance_months: 3.0,
        pool: ManagerPoolModel {
            per_million_academics: 4.0,
            min: 6,
            max: 12,
            leave_per_month: 0.15,
        },
        skills: SkillModel {
            focus: (60.0, 15.0),
            other: (35.0, 15.0),
            general: (50.0, 15.0),
            impression_blur: 15.0,
        },
        provenance: Provenance::default(),
    };
    let names = |list: &[&str]| list.iter().map(|n| (*n).to_owned()).collect::<Vec<_>>();
    c.name_groups = vec![NameGroup {
        key: "test".into(),
        countries: Vec::new(),
        is_default: true,
        surnames: names(&["Berg", "Holm", "Lind", "Stein", "Wald"]),
        first_names: names(&["Anna", "Erik", "Karl", "Lena", "Nils", "Olga"]),
        surname_first: false,
        places: names(&["Nord"]),
        legal_forms: names(&["AG"]),
        patterns: names(&["{familienname} {rechtsform}"]),
        branch_words: Vec::new(),
    }];
    c
}
