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
    for kind in wsim_core::ids::IdKind::ALL {
        let key = format!("art.{}", kind.name());
        assert!(data.texts.get(&key).is_some(), "Text „{key}“ fehlt");
    }
    for key in wsim_core::reports::text_keys() {
        assert!(data.texts.get(key).is_some(), "Text „{key}“ fehlt");
    }
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

/// Chain 1 with the shipped data: ore and coal in Britain, pig iron in a blast furnace.
#[test]
fn chain_one_runs_in_britain() {
    use std::sync::Arc;
    use wsim_core::calendar::RoundLength;
    use wsim_core::catalog::SiteType;
    use wsim_core::command::Command;
    use wsim_core::game::Game;
    use wsim_core::money::Money;
    use wsim_core::state::{GameSettings, SiteId, StartForm};

    let data = load_dir(&data_dir()).data.expect("data loads");
    let c = Arc::new(data.catalog);
    let gbr = c.countries.id("GBR").unwrap();
    let mut game = Game::new(
        c.clone(),
        GameSettings {
            seed: 1,
            start_year: 1900,
            start_country: gbr,
            start_capital: Money::from_usd(200_000_000.0).unwrap(),
            start_form: StartForm::Workshop,
            company_name: "Teesside Iron".into(),
            research_ahead_factor: 1.0,
            market_scale: 1.0,
            ai: Default::default(),
        },
    )
    .unwrap();
    let apply = |game: &mut Game, cmd| game.apply(cmd).unwrap();
    let last_site = |game: &Game| SiteId(u32::try_from(game.state().sites.len() - 1).unwrap());
    let mut mines = Vec::new();
    for (deposit, facility, recipe) in [
        ("cleveland_hills", "erzbergwerk", "eisenerz_abbau"),
        ("suedwales", "kohlenzeche", "kohle_abbau"),
    ] {
        apply(
            &mut game,
            Command::FoundSite {
                country: gbr,
                kind: SiteType::Extraction,
            },
        );
        let site = last_site(&game);
        mines.push(site);
        apply(
            &mut game,
            Command::BuildFacility {
                site,
                facility: c.facilities.id(facility).unwrap(),
                count: 1,
            },
        );
        apply(
            &mut game,
            Command::DevelopDeposit {
                site,
                deposit: c.deposits.id(deposit).unwrap(),
            },
        );
        apply(
            &mut game,
            Command::SetProduction {
                site,
                slot: 0,
                recipe: c.recipes.id(recipe),
                utilization: 1.0,
            },
        );
    }
    apply(
        &mut game,
        Command::FoundSite {
            country: gbr,
            kind: SiteType::Factory,
        },
    );
    let works = last_site(&game);
    apply(
        &mut game,
        Command::BuildFacility {
            site: works,
            facility: c.facilities.id("hochofen").unwrap(),
            count: 1,
        },
    );
    apply(
        &mut game,
        Command::SetProduction {
            site: works,
            slot: 0,
            recipe: c.recipes.id("roheisen_kokshochofen"),
            utilization: 1.0,
        },
    );
    // Developing the Welsh coal field takes 720 days.
    for _ in 0..12 {
        game.advance(RoundLength::Quarter, |_| {});
    }
    let ore = c.products.id("eisenerz").unwrap();
    let coal = c.products.id("kohle").unwrap();
    for (site, product) in [(mines[0], ore), (mines[1], coal)] {
        let quantity = game.state().site(site).unwrap().inventory[&product].quantity;
        assert!(quantity > 100_000.0, "{quantity}");
        apply(
            &mut game,
            Command::TransferGoods {
                from: site,
                to: works,
                product,
                quantity,
            },
        );
    }
    game.advance(RoundLength::Month, |_| {});
    let pig_iron = c.products.id("roheisen").unwrap();
    let stock = &game.state().site(works).unwrap().inventory[&pig_iron];
    // 250 t a day at full capacity.
    assert!(stock.quantity > 250.0 * 25.0, "{}", stock.quantity);
    let cost_per_t = stock.value.to_usd() / stock.quantity;
    // Ore, coal and labor in Britain 1903: a few dozen dollars (2026) per tonne.
    eprintln!("Herstellkosten Roheisen 1903: {cost_per_t:.2} USD/t");
    assert!(
        (10.0..200.0).contains(&cost_per_t),
        "Herstellkosten Roheisen: {cost_per_t} USD/t"
    );
    assert!(
        game.state()
            .company(game.player())
            .unwrap()
            .ledger
            .is_balanced()
    );
}

/// Government demand for steel bars in Britain 1900 has a plausible size.
#[test]
fn government_demand_for_steel_bars() {
    use std::sync::Arc;
    use wsim_core::game::Game;
    use wsim_core::money::Money;
    use wsim_core::state::{GameSettings, StartForm};

    let data = load_dir(&data_dir()).data.expect("data loads");
    let c = Arc::new(data.catalog);
    let gbr = c.countries.id("GBR").unwrap();
    let game = Game::new(
        c.clone(),
        GameSettings {
            seed: 1,
            start_year: 1900,
            start_country: gbr,
            start_capital: Money::from_usd(1_000_000.0).unwrap(),
            start_form: StartForm::Workshop,
            company_name: "Test".into(),
            research_ahead_factor: 1.0,
            market_scale: 1.0,
            ai: Default::default(),
        },
    )
    .unwrap();
    let per_year = game
        .state()
        .markets
        .get(c.products.id("stabstahl").unwrap())
        .get(gbr)
        .state_rate
        * 365.0;
    // Railways, bridges, public buildings: several hundred thousand tonnes a year.
    assert!((200_000.0..2_000_000.0).contains(&per_year), "{per_year}");
}

#[test]
fn freight_routes_are_plausible() {
    use wsim_core::transport::Routes;

    let data = load_dir(&data_dir()).data.expect("data loads");
    let c = &data.catalog;
    let country = |key: &str| c.countries.id(key).unwrap();
    let class = |key: &str| c.transport_classes.id(key).unwrap();
    let routes = Routes::new(c, 1900, None);
    let bulk = class("schuettgut");

    // Coal across the Atlantic: about 2–4 USD of 1900 per tonne, three to four weeks.
    let atlantic = routes.get(bulk, country("GBR"), country("USA")).unwrap();
    assert!(atlantic.by_sea);
    assert!((60.0..200.0).contains(&atlantic.cost_per_t), "{atlantic:?}");
    assert!((10.0..40.0).contains(&atlantic.days), "{atlantic:?}");
    // Piece goods cost more than bulk on the same way.
    let piece = routes
        .get(class("stueckgut"), country("GBR"), country("USA"))
        .unwrap();
    assert!(piece.cost_per_t > atlantic.cost_per_t);

    // Landlocked countries reach the sea through their neighbors.
    let swiss = routes.get(bulk, country("CHE"), country("USA")).unwrap();
    assert!(swiss.cost_per_t > atlantic.cost_per_t);
    // Neighbors without a sea leg: by land.
    let alps = routes.get(bulk, country("CHE"), country("AUT")).unwrap();
    assert!(!alps.by_sea, "{alps:?}");

    // Electricity only flows through the national grid in stage 1.
    assert!(
        routes
            .get(class("leitung"), country("DEU"), country("FRA"))
            .is_none()
    );
    // Every country can be reached from Britain with bulk goods.
    for (id, _) in c.countries.iter() {
        assert!(
            routes.get(bulk, country("GBR"), id).is_some(),
            "kein Weg nach {}",
            c.countries.key(id)
        );
    }
    // Freight gets cheaper over time (motor ships).
    let later = Routes::new(c, 1930, None)
        .get(bulk, country("GBR"), country("USA"))
        .unwrap();
    assert!(later.cost_per_t < atlantic.cost_per_t);
}

#[test]
fn start_forms_give_a_workshop_or_an_office() {
    use std::sync::Arc;
    use wsim_core::calendar::RoundLength;
    use wsim_core::catalog::SiteType;
    use wsim_core::game::{Game, NewGameError};
    use wsim_core::state::{GameSettings, StartForm};

    let data = load_dir(&data_dir()).data.expect("data loads");
    let c = Arc::new(data.catalog);
    let settings = |form, capital: f64| GameSettings {
        seed: 4,
        start_year: 1900,
        start_country: c.countries.id("DEU").unwrap(),
        start_capital: Money::from_usd(capital).unwrap(),
        start_form: form,
        company_name: "Start".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: Default::default(),
    };

    let mut workshop = Game::new(c.clone(), settings(StartForm::Workshop, 100_000.0)).unwrap();
    let site = &workshop.state().sites[0];
    assert_eq!(site.kind, SiteType::Factory);
    assert_eq!(site.slots.len(), 1);
    let nails = c.products.id("naegel").unwrap();
    let wire = c.products.id("draht").unwrap();
    assert!(site.offers.contains_key(&nails));
    assert!(site.orders.contains_key(&wire));
    let ledger = &workshop.state().companies[0].ledger;
    assert_eq!(ledger.cash(), Money::from_usd(40_000.0).unwrap());
    assert!(ledger.is_balanced());
    workshop.advance(RoundLength::Month, |_| {});
    assert!(workshop.state().companies[0].ledger.is_balanced());

    let trading = Game::new(c.clone(), settings(StartForm::Trading, 100_000.0)).unwrap();
    assert_eq!(trading.state().sites[0].kind, SiteType::SalesOffice);
    assert_eq!(
        trading.state().companies[0].ledger.cash(),
        Money::from_usd(85_000.0).unwrap()
    );

    assert!(matches!(
        Game::new(c.clone(), settings(StartForm::Workshop, 50_000.0)),
        Err(NewGameError::StartFormTooExpensive { .. })
    ));
}

/// A new game with 100 AI companies (Lastenheft §10): historical companies first, the
/// rest generated, every company's books balanced.
#[test]
fn ai_start_population() {
    use std::collections::BTreeSet;
    use std::sync::Arc;
    use wsim_core::game::Game;
    use wsim_core::state::{AiSettings, GameSettings, StartForm};

    let data = load_dir(&data_dir()).data.expect("data loads");
    let c = Arc::new(data.catalog);
    let settings = GameSettings {
        seed: 7,
        start_year: 1900,
        start_country: c.countries.id("DEU").unwrap(),
        start_capital: Money::from_usd(100_000.0).unwrap(),
        start_form: StartForm::Workshop,
        company_name: "Start".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: AiSettings {
            companies: 100,
            competence: 0.5,
            aggressiveness: 0.5,
        },
    };
    let game = Game::new(c.clone(), settings.clone()).unwrap();
    let state = game.state();
    assert!((state.settings.market_scale - 0.1).abs() < 1e-12);
    let ai: Vec<_> = state.companies.iter().filter(|x| x.ai.is_some()).collect();
    assert_eq!(ai.len(), 100);
    assert!(ai.iter().any(|x| x.name == "Fried. Krupp"));
    let names: BTreeSet<String> = state
        .companies
        .iter()
        .map(|x| x.name.to_lowercase())
        .collect();
    assert_eq!(names.len(), state.companies.len(), "names are unique");
    for company in &state.companies {
        assert!(company.ledger.is_balanced(), "{}", company.name);
        assert!(company.ledger.cash() >= Money::ZERO, "{}", company.name);
    }
    // Every AI company owns plants; concessions point to extraction sites on them.
    for (i, _) in state.companies.iter().enumerate().skip(1) {
        assert!(state.sites.iter().any(|s| s.owner.0 as usize == i));
    }
    for (d, ds) in state.deposits.iter() {
        for field in &ds.concessions {
            if let Some(site) = field.site {
                assert_eq!(state.sites[site.0 as usize].deposit, Some(d));
            }
        }
    }
    // Same settings, same world.
    let again = Game::new(c, settings).unwrap();
    assert_eq!(game.state_hash(), again.state_hash());
}

/// AI companies decide through commands; a game with them stays reproducible, balanced
/// and loadable.
#[test]
fn ai_world_is_reproducible() {
    use std::sync::Arc;
    use wsim_core::calendar::RoundLength;
    use wsim_core::game::Game;
    use wsim_core::save;
    use wsim_core::state::{AiSettings, GameSettings, StartForm};

    let data = load_dir(&data_dir()).data.expect("data loads");
    let c = Arc::new(data.catalog);
    let settings = GameSettings {
        seed: 11,
        start_year: 1900,
        start_country: c.countries.id("GBR").unwrap(),
        start_capital: Money::from_usd(100_000.0).unwrap(),
        start_form: StartForm::Workshop,
        company_name: "Start".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: AiSettings {
            companies: 40,
            competence: 0.8,
            aggressiveness: 0.7,
        },
    };
    let run = || {
        let mut game = Game::new(c.clone(), settings.clone()).unwrap();
        for _ in 0..4 {
            game.advance(RoundLength::Month, |_| {});
        }
        game
    };
    let game = run();
    assert_eq!(game.state_hash(), run().state_hash());
    for company in &game.state().companies {
        assert!(company.ledger.is_balanced(), "{}", company.name);
    }
    let loaded = save::decode(&save::encode(&game), c.clone()).unwrap().game;
    assert_eq!(loaded.state_hash(), game.state_hash());
    let replayed = Game::replay(c.clone(), settings.clone(), game.journal()).unwrap();
    assert_eq!(replayed.state_hash(), game.state_hash());
    // The AI acted: some sites changed their production or prices.
    assert!(
        game.state()
            .sites
            .iter()
            .flat_map(|s| s.slots.iter())
            .any(|sl| (sl.utilization - c.ai_model.start.utilization).abs() > 1e-9)
    );
}
