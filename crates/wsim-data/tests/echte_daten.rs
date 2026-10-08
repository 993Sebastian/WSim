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

    for key in ["eisenerz", "kohle", "stahl", "blech", "draht", "stabstahl"] {
        assert!(c.products.id(key).is_some(), "Produkt {key} fehlt");
    }
    let erz = c.products.id("eisenerz").unwrap();
    assert_eq!(c.products.get(erz).kind, ProductKind::RawMaterial);
    assert_eq!(c.products.get(erz).weight_kg, 1000.0);

    // Blast furnace and converter form one steelworks (Lastenheft §17.2).
    assert!(c.products.id("roheisen").is_none());
    let recipe = c.recipes.get(c.recipes.id("stahl_bessemer").unwrap());
    assert_eq!(recipe.product, c.products.id("stahl").unwrap());
    assert!(recipe.inputs.contains(&(erz, 1.9)));
    let metal = c.labor_groups.id("fachkraft.metall").unwrap();
    assert!(recipe.labor_hours.contains(&(metal, 5.2)));

    let works = c
        .facilities
        .get(c.facilities.id("stahlwerk_konverter").unwrap());
    assert_eq!(works.investment, Money::from_usd(85_000_000.0).unwrap());

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
    // 197 countries, small ones merged into 111 countries and regions (M34).
    assert_eq!(c.countries.len(), 111);
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
        ["AUT", "BEL", "CHE", "CZE", "DNK", "FRA", "NLD", "POL"]
    );
    assert_eq!(data.texts.get("land.DEU"), Some("Deutschland"));
    assert!(deu.members.is_empty());

    // Regions: Luxembourg went to Belgium, the Baltic states form a region of their own.
    let bel = c.countries.get(c.countries.id("BEL").unwrap());
    assert_eq!(bel.members, ["BEL", "LUX"]);
    assert_eq!(data.texts.get("teilland.LUX"), Some("Luxemburg"));
    assert!(c.countries.id("LUX").is_none());
    let baltic = c.countries.get(c.countries.id("XBA").unwrap());
    assert_eq!(baltic.members, ["LTU", "LVA", "EST"]);
    assert_eq!(data.texts.get("land.XBA"), Some("Baltikum"));
    let merged: usize = c
        .countries
        .iter()
        .map(|(_, k)| k.members.len().max(1))
        .sum();
    assert_eq!(merged, 197);
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
    for key in wsim_core::events::EFFECT_TEXTS {
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
            ventures: 1.0,
            tariff_dynamics: 1.0,
            event_effects: true,
            found_at_start: true,
            person: Default::default(),
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
                size: wsim_core::catalog::FacilitySize::Medium,
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
            facility: c.facilities.id("stahlwerk_konverter").unwrap(),
            count: 1,
            size: wsim_core::catalog::FacilitySize::Medium,
        },
    );
    apply(
        &mut game,
        Command::SetProduction {
            site: works,
            slot: 0,
            recipe: c.recipes.id("stahl_bessemer"),
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
    let steel = c.products.id("stahl").unwrap();
    let stock = &game.state().site(works).unwrap().inventory[&steel];
    // 250 t a day at full capacity.
    assert!(stock.quantity > 250.0 * 25.0, "{}", stock.quantity);
    let cost_per_t = stock.value.to_usd() / stock.quantity;
    // Ore, coal and labor in Britain 1903: well below the reference price of 900 USD.
    eprintln!("Herstellkosten Stahl 1903: {cost_per_t:.2} USD/t");
    assert!(
        (100.0..600.0).contains(&cost_per_t),
        "Herstellkosten Stahl: {cost_per_t} USD/t"
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
            ventures: 1.0,
            tariff_dynamics: 1.0,
            event_effects: true,
            found_at_start: true,
            person: Default::default(),
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
        ventures: 1.0,
        tariff_dynamics: 1.0,
        event_effects: true,
        found_at_start: true,
        person: Default::default(),
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
        ventures: 1.0,
        tariff_dynamics: 1.0,
        event_effects: true,
        found_at_start: true,
        person: Default::default(),
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
    // The start plants are completed on the first day and booked once as fixed assets.
    let mut game = again;
    game.advance(wsim_core::calendar::RoundLength::Day, |_| {});
    for company in &game.state().companies {
        let ledger = &company.ledger;
        assert!(ledger.is_balanced(), "{}", company.name);
        assert!(
            ledger.balance(wsim_core::ledger::Account::AssetsUnderConstruction) >= Money::ZERO,
            "{}",
            company.name
        );
    }
}

/// M16 (Lastenheft §9.1): At the start established companies saturate the markets; a
/// newcomer sells only what it wins from them, not every unit at any price.
#[test]
fn markets_are_saturated_and_newcomers_must_compete() {
    use std::sync::Arc;
    use wsim_core::calendar::RoundLength;
    use wsim_core::command::Command;
    use wsim_core::game::Game;
    use wsim_core::state::{AiSettings, GameSettings, PriceMode, SiteId, StartForm};

    let data = load_dir(&data_dir()).data.expect("data loads");
    let c = Arc::new(data.catalog);
    let settings = GameSettings {
        seed: 1,
        start_year: 1900,
        start_country: c.countries.id("DEU").unwrap(),
        start_capital: Money::from_usd(100_000.0).unwrap(),
        start_form: StartForm::Workshop,
        company_name: "Neuling".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: AiSettings {
            companies: 100,
            competence: 0.5,
            aggressiveness: 0.5,
        },
        ventures: 1.0,
        tariff_dynamics: 1.0,
        event_effects: true,
        found_at_start: true,
        person: Default::default(),
    };
    let mut game = Game::new(c.clone(), settings).unwrap();
    let nails = c.products.id("naegel").unwrap();
    let deu = c.countries.id("DEU").unwrap();
    let player = game.player();
    let site = SiteId(
        u32::try_from(
            game.state()
                .sites
                .iter()
                .position(|s| s.owner == player)
                .unwrap(),
        )
        .unwrap(),
    );
    // The newcomer asks 30 % more than the market.
    let price = wsim_core::market::market_price(&c, game.state(), deu, nails).scale(1.3);
    game.apply(Command::SetSale {
        site,
        product: nails,
        mode: Some(PriceMode::Fixed(price)),
        keep: 0.0,
    })
    .unwrap();
    for _ in 0..3 {
        game.advance(RoundLength::Month, |_| {});
    }
    let state = game.state();
    let s = &state.sites[site.index()];
    let made: f64 = s
        .slots
        .iter()
        .filter_map(|sl| {
            let r = c.recipes.get(sl.recipe?);
            (r.product == nails).then(|| sl.full_runs(&c) * r.output * 30.0)
        })
        .sum();
    let sold = s.offers[&nails].sold_last_month;
    assert!(
        sold < 0.5 * made,
        "Neuling verkauft {sold} von {made} trotz Aufschlag"
    );

    // The established companies serve the demand for end products everywhere.
    let mut covered = Vec::new();
    for (product, p) in c.products.iter() {
        if p.kind != ProductKind::EndProduct {
            continue;
        }
        let (mut demand, mut served) = (0.0, 0.0);
        for country in c.countries.ids() {
            let m = &state.markets.get(product).get(country).last_month;
            demand += m.demand;
            served += m.sold - m.exported;
        }
        if demand > 0.0 {
            covered.push((served / demand, c.products.key(product).to_string()));
        }
    }
    covered.sort_by(|a, b| a.0.total_cmp(&b.0));
    let well = covered.iter().filter(|(share, _)| *share >= 0.85).count();
    assert!(
        well * 5 >= covered.len() * 4,
        "zu viele Endprodukte unterversorgt: {covered:?}"
    );
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
        ventures: 1.0,
        tariff_dynamics: 1.0,
        event_effects: true,
        found_at_start: true,
        person: Default::default(),
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

/// Historical events reach the round report as world news (Lastenheft §4.1, §13.2).
#[test]
fn world_events_appear_in_the_round_report() {
    use std::sync::Arc;
    use wsim_core::calendar::{Date, RoundLength};
    use wsim_core::game::Game;
    use wsim_core::message::{MessageKind, Param};
    use wsim_core::state::{GameSettings, StartForm};

    let data = load_dir(&data_dir()).data.expect("data loads");
    for kind in [
        "krieg",
        "kriegsende",
        "krise",
        "revolution",
        "staatsgruendung",
        "abkommen",
        "katastrophe",
        "technik",
        "reform",
    ] {
        assert!(
            data.texts.get(&format!("ereignisart.{kind}")).is_some(),
            "{kind}"
        );
    }
    let c = Arc::new(data.catalog);
    assert!(c.events.windows(2).all(|w| w[0].date <= w[1].date));
    let settings = GameSettings {
        seed: 1,
        start_year: 1914,
        start_country: c.countries.id("DEU").unwrap(),
        start_capital: Money::from_usd(100_000.0).unwrap(),
        start_form: StartForm::Trading,
        company_name: "Ereignis".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: Default::default(),
        ventures: 1.0,
        tariff_dynamics: 1.0,
        event_effects: true,
        found_at_start: true,
        person: Default::default(),
    };
    let mut game = Game::new(c, settings).unwrap();
    let mut events = Vec::new();
    while game.date() < Date::new(1914, 9, 1).unwrap() {
        let report = game.advance(RoundLength::Month, |_| {});
        events.extend(
            report
                .messages
                .into_iter()
                .filter(|m| m.kind == MessageKind::WorldEvent),
        );
    }
    let war = events
        .iter()
        .find(|m| {
            m.params
                .iter()
                .any(|(_, p)| *p == Param::TextKey("ereignis.erster_weltkrieg".into()))
        })
        .expect("the First World War is reported");
    let countries = war
        .params
        .iter()
        .find(|(k, _)| k == "laender")
        .map(|(_, p)| p.clone());
    assert!(matches!(countries, Some(Param::Countries(c)) if c.contains(&"DEU".to_owned())));
}

/// History along the way (M28): the euro arrives as a world event and, for a company
/// at home in Germany, as a change of currency at the rate fixed by law.
#[test]
fn the_euro_arrives_as_world_event_and_change_of_currency() {
    use std::sync::Arc;
    use wsim_core::calendar::{Date, RoundLength};
    use wsim_core::game::Game;
    use wsim_core::message::{MessageKind, Param, keys};
    use wsim_core::state::{GameSettings, StartForm};

    let data = load_dir(&data_dir()).data.expect("data loads");
    let c = Arc::new(data.catalog);
    let settings = GameSettings {
        seed: 1,
        start_year: 1998,
        start_country: c.countries.id("DEU").unwrap(),
        start_capital: Money::from_usd(100_000.0).unwrap(),
        start_form: StartForm::Trading,
        company_name: "Euro".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: Default::default(),
        ventures: 1.0,
        tariff_dynamics: 1.0,
        event_effects: true,
        found_at_start: true,
        person: Default::default(),
    };
    let mut game = Game::new(c, settings).unwrap();
    let mut news = Vec::new();
    while game.date() <= Date::new(1999, 1, 1).unwrap() {
        let report = game.advance(RoundLength::Month, |_| {});
        news.extend(
            report
                .messages
                .into_iter()
                .filter(|m| m.kind == MessageKind::WorldEvent),
        );
    }
    let param = |m: &wsim_core::message::Message, name: &str| {
        m.params
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, p)| p.clone())
    };
    assert!(
        news.iter()
            .any(|m| param(m, "ereignis") == Some(Param::TextKey("ereignis.euro".into())))
    );
    let reforms: Vec<_> = news
        .iter()
        .filter(|m| m.key == keys::CURRENCY_REFORM)
        .collect();
    assert_eq!(reforms.len(), 1, "{reforms:?}");
    let euro = reforms[0];
    assert_eq!(param(euro, "land"), Some(Param::Country("DEU".into())));
    assert_eq!(
        param(euro, "neu"),
        Some(Param::TextKey("waehrung.euro".into()))
    );
    assert_eq!(
        param(euro, "alt"),
        Some(Param::TextKey("waehrung.d_mark".into()))
    );
    assert_eq!(param(euro, "faktor"), Some(Param::Number(1.95583)));
    assert_eq!(param(euro, "monat"), Some(Param::TextKey("monat.1".into())));
    assert_eq!(param(euro, "jahr"), Some(Param::Integer(1999)));
}

/// Plausibility of a world run (M16): over the first year with 100 AI companies no
/// market breaks down, prices stay near the reference prices, plants get their inputs,
/// no AI company fails, and a passive player's workshop earns no fortune. The limits
/// catch breakdowns; the start year still has transients (wood shipped from remote
/// forests, small markets filling the traders' stocks). The balance protocol
/// (`wsim run --protokoll`) checks tighter limits over longer runs.
#[test]
fn world_stays_plausible_in_the_first_year() {
    use std::sync::Arc;
    use wsim_core::calendar::RoundLength;
    use wsim_core::game::Game;
    use wsim_core::health;
    use wsim_core::ids::Id;
    use wsim_core::state::{AiSettings, GameSettings, StartForm};

    let data = load_dir(&data_dir()).data.expect("data loads");
    let c = Arc::new(data.catalog);
    let capital = 100_000.0;
    let settings = GameSettings {
        seed: 1,
        start_year: 1900,
        start_country: c.countries.id("DEU").unwrap(),
        start_capital: Money::from_usd(capital).unwrap(),
        start_form: StartForm::Workshop,
        company_name: "Werkstatt".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: AiSettings {
            companies: 100,
            competence: 0.5,
            aggressiveness: 0.5,
        },
        ventures: 1.0,
        tariff_dynamics: 1.0,
        event_effects: true,
        found_at_start: true,
        person: Default::default(),
    };
    let mut game = Game::new(c.clone(), settings).unwrap();
    let n = c.products.len();
    let mut demand = vec![0.0; n];
    let mut served = vec![0.0; n];
    let mut revenue = vec![0.0; n];
    let mut reference = vec![0.0; n];
    let mut planned = vec![0.0; n];
    let mut waiting = vec![0.0; n];
    for _ in 0..12 {
        game.advance(RoundLength::Month, |_| {});
        for h in health::last_month(game.state(), &c) {
            let i = h.product.index();
            demand[i] += h.outside_demand;
            served[i] += h.outside_sold;
            revenue[i] += h.revenue_usd;
            reference[i] += h.reference_value_usd;
            planned[i] += h.planned;
            waiting[i] += h.input_limited;
        }
    }

    let mut problems = Vec::new();
    for product in c.products.ids() {
        let i = product.index();
        let name = c.products.key(product);
        if demand[i] > 1e-9 && served[i] / demand[i] < 0.5 {
            let share = 100.0 * served[i] / demand[i];
            problems.push(format!("{name}: nur {share:.0} % der Nachfrage bedient"));
        }
        // A product made only as a by-product (petrol before cracking) may be cheap.
        if reference[i] > 1e-9 && health::made_as_main(&c, product, 1900) {
            let ratio = revenue[i] / reference[i];
            if !(0.4..=2.5).contains(&ratio) {
                problems.push(format!("{name}: Preis {ratio:.2} × Richtpreis"));
            }
        }
        if planned[i] > 1e-9 && waiting[i] / planned[i] > 0.3 {
            let share = 100.0 * waiting[i] / planned[i];
            problems.push(format!(
                "{name}: {share:.0} % der Erzeugung warten auf Vorprodukte"
            ));
        }
    }
    let state = game.state();
    let failed = state.companies.iter().filter(|c| c.bankrupt).count();
    if failed > 0 {
        problems.push(format!("{failed} KI-Firmen pleite"));
    }
    let player = state.company(game.player()).unwrap();
    let result = player
        .ledger
        .years
        .last()
        .map_or(0.0, |y| y.total().to_usd());
    if result > 0.8 * capital {
        problems.push(format!(
            "Werkstatt ohne Entscheidungen verdient {result:.0} USD im ersten Jahr"
        ));
    }
    assert!(
        problems.is_empty(),
        "Unplausibler Weltlauf 1900:\n{}",
        problems.join("\n")
    );
}

/// Every country can show money at any time of the game, and the conversion keeps the
/// history: Mark in 1900, the inflation of 1923, the euro of today (M21).
#[test]
fn currencies_cover_every_country_and_year() {
    let data = load_dir(&data_dir()).data.expect("data loads");
    let c = &data.catalog;
    let m = &c.currencies;
    for (country, _) in c.countries.iter() {
        let key = c.countries.key(country);
        let base = m
            .at_base(country)
            .unwrap_or_else(|| panic!("{key}: keine Währung 2026"));
        assert!(
            base.factor.is_finite() && base.factor > 0.0,
            "{key}: {base:?}"
        );
        for year in 1900..=2100 {
            let t = f64::from(year) + 0.37;
            let then = m.at_time(country, t).expect("currency");
            assert!(
                then.factor.is_finite() && then.factor > 0.0,
                "{key} {year}: {then:?}"
            );
        }
    }
    let deu = c.countries.id("DEU").expect("DEU");
    assert_eq!(m.at_base(deu).expect("currency").currency, "euro");
    let mark = m.at_time(deu, 1900.5).expect("currency");
    assert_eq!(mark.currency, "mark");
    // A nail price of 1,865 USD (2026) was about 200 Mark per tonne in 1900.
    let naegel = 1865.0 * mark.factor;
    assert!((150.0..250.0).contains(&naegel), "{naegel}");
    let inflation = m.at_time(deu, 1923.85).expect("currency");
    assert!(inflation.factor > 1e9, "{inflation:?}");
    assert_eq!(m.at_time(deu, 1950.0).expect("currency").currency, "d_mark");
    let usa = c.countries.id("USA").expect("USA");
    assert!((m.at_base(usa).expect("currency").factor - 1.0).abs() < 1e-12);
}

/// M42: Every AI company names the end products it makes or offers, with invented names
/// that are free for their product; new products get names during the game.
#[test]
fn companies_name_their_end_products() {
    use std::collections::{BTreeMap, BTreeSet};
    use std::sync::Arc;
    use wsim_core::calendar::RoundLength;
    use wsim_core::game::Game;
    use wsim_core::ids::ProductId;
    use wsim_core::product_names;
    use wsim_core::state::{AiSettings, CompanyId, GameSettings, StartForm};

    let data = load_dir(&data_dir()).data.expect("data loads");
    let c = Arc::new(data.catalog);
    let settings = GameSettings {
        seed: 3,
        start_year: 1960,
        start_country: c.countries.id("DEU").unwrap(),
        start_capital: Money::from_usd(100_000.0).unwrap(),
        start_form: StartForm::Workshop,
        company_name: "Namen".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: AiSettings {
            companies: 100,
            competence: 0.5,
            aggressiveness: 0.5,
        },
        ventures: 1.0,
        tariff_dynamics: 1.0,
        event_effects: true,
        found_at_start: true,
        person: Default::default(),
    };
    let mut game = Game::new(c.clone(), settings).unwrap();
    for _ in 0..3 {
        game.advance(RoundLength::Month, |_| {});
    }
    // Plants built at the end of the quarter get their names at the next operations of
    // their company, at most 14 days later.
    for _ in 0..15 {
        game.advance(RoundLength::Day, |_| {});
    }
    let state = game.state();
    let naming = &c.product_naming;
    let mut per_product: BTreeMap<ProductId, BTreeSet<String>> = BTreeMap::new();
    for (i, company) in state.companies.iter().enumerate() {
        if company.ai.is_none() || company.bankrupt {
            continue;
        }
        let id = CompanyId(u32::try_from(i).unwrap());
        for p in product_names::named_products(&c, state, id) {
            let key = c.products.key(p);
            let name = company
                .product_names
                .get(&p)
                .unwrap_or_else(|| panic!("{} has no name for {key}", company.name));
            assert!(!naming.is_excluded(name), "{name}");
            let style = naming.style(&c, p).unwrap();
            assert!(
                style.stems.iter().any(|s| name.starts_with(s.as_str())),
                "{name} ({key})"
            );
            assert!(
                per_product
                    .entry(p)
                    .or_default()
                    .insert(name.to_lowercase()),
                "{name} twice for {key}"
            );
        }
    }
    let named: usize = per_product.values().map(BTreeSet::len).sum();
    assert!(named > 20, "{named} names");
}

/// Start population (docs/FORMELN.md, Startbesetzung step 4, after M41): in every start
/// year the start plants make each input the start plants use (state markets and
/// electricity aside), also inputs needed too little to fill a facility of the data
/// size. Without them every stage above was scaled down to nothing: from 1920 bauxite
/// was missing (aluminium, pots), from 1970 pure silicon (transistors, colour TVs), and
/// a start in 2006 had no plants for mobile phones, laptops or their batteries.
#[test]
fn start_plants_make_every_input_they_use() {
    use std::sync::Arc;
    use wsim_core::game::Game;
    use wsim_core::ids::Id;
    use wsim_core::state::{AiSettings, GameSettings, StartForm};

    let data = load_dir(&data_dir()).data.expect("data loads");
    let c = Arc::new(data.catalog);
    let chains = [
        (1920, ["bauxit", "aluminium", "kochtopf", "roehrenradio"]),
        (
            1970,
            [
                "reinstsilizium",
                "transistor",
                "farbfernseher",
                "transistorradio",
            ],
        ),
        (
            2006,
            ["lithium", "lithium_ionen_akku", "mobiltelefon", "laptop"],
        ),
    ];
    for (year, products) in chains {
        let settings = GameSettings {
            seed: 3,
            start_year: year,
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
            ventures: 1.0,
            tariff_dynamics: 1.0,
            event_effects: true,
            found_at_start: true,
            person: Default::default(),
        };
        let game = Game::new(c.clone(), settings).unwrap();
        let state = game.state();
        let mut made = vec![0.0; c.products.len()];
        let mut used = vec![false; c.products.len()];
        for slot in state.sites.iter().flat_map(|s| &s.slots) {
            let Some(r) = slot.recipe.map(|r| c.recipes.get(r)) else {
                continue;
            };
            made[r.product.index()] +=
                f64::from(slot.count) * c.production_model.sizes.capacity(slot.size);
            for &(input, _) in &r.inputs {
                used[input.index()] = true;
            }
        }
        let missing: Vec<&str> = c
            .products
            .iter()
            .filter(|(id, p)| {
                used[id.index()]
                    && made[id.index()] == 0.0
                    && p.state_market.is_none()
                    && p.kind != ProductKind::Energy
            })
            .map(|(id, _)| c.products.key(id))
            .collect();
        assert!(
            missing.is_empty(),
            "{year}: no start plant makes {missing:?}"
        );
        for key in products {
            let p = c.products.id(key).unwrap();
            assert!(made[p.index()] > 0.0, "{year}: no start plant for {key}");
        }
        for company in &state.companies {
            assert!(company.ledger.is_balanced(), "{}", company.name);
        }
    }
}

/// MA0 (docs/FORMELN.md): the AI's rules put every decision to a decider before they
/// act. Forming the decisions for a decider that looks at them changes nothing; every
/// decision offers keeping things as they are, and every option can be assessed.
#[test]
fn decisions_of_the_ai_change_nothing() {
    use std::collections::BTreeSet;
    use std::sync::Arc;
    use wsim_core::calendar::{Date, RoundLength};
    use wsim_core::decision::{self, ChoiceKind, Recorder, Topic};
    use wsim_core::game::{Game, hash_of};
    use wsim_core::state::{AiSettings, GameSettings, StartForm};

    let data = load_dir(&data_dir()).data.expect("data loads");
    let c = Arc::new(data.catalog);
    let settings = GameSettings {
        seed: 5,
        start_year: 1960,
        start_country: c.countries.id("DEU").unwrap(),
        start_capital: Money::from_usd(100_000.0).unwrap(),
        start_form: StartForm::Workshop,
        company_name: "Entscheidungen".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: AiSettings {
            companies: 40,
            competence: 0.5,
            aggressiveness: 0.5,
        },
        ventures: 1.0,
        tariff_dynamics: 1.0,
        event_effects: true,
        found_at_start: true,
        person: Default::default(),
    };
    let mut game = Game::new(c.clone(), settings).unwrap();
    for _ in 0..2 {
        game.advance(RoundLength::Month, |_| {});
    }
    let mut topics = BTreeSet::new();
    // The end of a quarter, the start of a month and the start of a year.
    for (y, m, d) in [(1960, 3, 31), (1960, 4, 1), (1961, 1, 1)] {
        let date = Date::new(y, m, d).unwrap();
        let mut plain = game.state().clone();
        plain.date = date;
        let before = plain.clone();
        let mut looked = plain.clone();
        wsim_core::ai::decide(&mut plain, &c, date);
        let mut recorder = Recorder::default();
        wsim_core::ai::decide_with(&mut looked, &c, date, &mut recorder);
        assert_eq!(hash_of(&plain), hash_of(&looked), "{y}-{m}-{d}");
        for d in &recorder.decisions {
            assert!(d.rule < d.choices.len(), "{d:?}");
            let keep: Vec<_> = d
                .choices
                .iter()
                .filter(|c| c.kind == ChoiceKind::Keep)
                .collect();
            assert_eq!(keep.len(), 1, "{d:?}");
            assert!(keep[0].steps.is_empty());
            let assessed = decision::assess(&c, &before, d);
            assert_eq!(assessed.len(), d.choices.len());
            for (choice, a) in d.choices.iter().zip(&assessed) {
                assert!(a.amount >= Money::ZERO, "{d:?}");
                if choice.kind == ChoiceKind::Keep {
                    assert_eq!(a.amount, Money::ZERO);
                }
            }
            topics.insert(d.topic);
        }
    }
    for topic in [
        Topic::Production,
        Topic::Sale,
        Topic::Purchase,
        Topic::Cash,
        Topic::Advertising,
        Topic::Expansion,
    ] {
        assert!(topics.contains(&topic), "{topic:?} missing in {topics:?}");
    }
    assert!(topics.len() >= 10, "{topics:?}");
}

/// ZA2 (docs/BETEILIGUNGEN.md §8): central departments built too early ruin a workshop –
/// their employees cost wages and offices every month, working or not. One employee each
/// in finance and marketing: the losses of the first half year grow by their costs, and
/// within a year the workshop is insolvent, while without them it carries on.
#[test]
fn early_central_departments_ruin_a_workshop() {
    use std::sync::Arc;
    use wsim_core::calendar::RoundLength;
    use wsim_core::catalog::DepartmentKind;
    use wsim_core::command::Command;
    use wsim_core::game::Game;
    use wsim_core::state::{GameSettings, StartForm};

    let data = load_dir(&data_dir()).data.expect("data loads");
    let c = Arc::new(data.catalog);
    let settings = GameSettings {
        seed: 4,
        start_year: 1900,
        start_country: c.countries.id("DEU").unwrap(),
        start_capital: Money::from_usd(100_000.0).unwrap(),
        start_form: StartForm::Workshop,
        company_name: "Werkstatt".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: Default::default(),
        ventures: 1.0,
        tariff_dynamics: 1.0,
        event_effects: true,
        found_at_start: true,
        person: Default::default(),
    };
    let mut plain = Game::new(c.clone(), settings.clone()).unwrap();
    let mut early = Game::new(c.clone(), settings).unwrap();
    for kind in [DepartmentKind::Finance, DepartmentKind::Marketing] {
        early
            .apply(Command::StaffDepartment {
                department: kind,
                staff: 1,
            })
            .unwrap();
    }
    let (personnel, office) = wsim_core::central::monthly_cost(&c, early.state(), early.player());
    let monthly = personnel + office;
    assert!(monthly > Money::from_usd(5_000.0).unwrap(), "{monthly:?}");
    let result = |g: &Game| g.state().companies[0].ledger.year.total();
    for _ in 0..6 {
        plain.advance(RoundLength::Month, |_| {});
        early.advance(RoundLength::Month, |_| {});
    }
    let (a, b) = (result(&plain), result(&early));
    assert!(
        a - b > monthly.scale(5.0),
        "with departments {b:?}, without {a:?}, a month {monthly:?}"
    );
    assert!(early.state().companies[0].ledger.is_balanced());
    for _ in 0..6 {
        plain.advance(RoundLength::Month, |_| {});
        early.advance(RoundLength::Month, |_| {});
    }
    assert!(!plain.state().companies[0].bankrupt);
    assert!(early.state().companies[0].bankrupt);
}
