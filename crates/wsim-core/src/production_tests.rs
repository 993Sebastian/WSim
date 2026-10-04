//! Scenario tests for production (M5) with the small chain of `test_support::production`.

use std::sync::Arc;

use crate::calendar::{Date, RoundLength};
use crate::catalog::{Catalog, SiteType, test_support};
use crate::command::{Command, CommandError};
use crate::game::Game;
use crate::ids::Id;
use crate::ledger::{Account, CostType};
use crate::money::Money;
use crate::save;
use crate::state::{GameSettings, SiteId, StartForm};

fn usd(v: f64) -> Money {
    Money::from_usd(v).unwrap()
}

fn new_game(catalog: Catalog) -> Game {
    let catalog = Arc::new(catalog);
    let settings = GameSettings {
        seed: 3,
        start_year: 1900,
        start_country: catalog.countries.id("AAA").unwrap(),
        start_capital: usd(10_000_000.0),
        start_form: StartForm::Workshop,
        company_name: "Hütte AG".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: Default::default(),
    };
    Game::new(catalog, settings).unwrap()
}

fn days(game: &mut Game, n: u32) {
    for _ in 0..n {
        game.advance(RoundLength::Day, |_| {});
    }
}

/// Mine with developed deposit (ready after 30 days) producing at full capacity.
fn mine(game: &mut Game) -> SiteId {
    let c = game.catalog().clone();
    let aaa = c.countries.id("AAA").unwrap();
    game.apply(Command::FoundSite {
        country: aaa,
        kind: SiteType::Extraction,
    })
    .unwrap();
    let site = SiteId(u32::try_from(game.state().sites.len() - 1).unwrap());
    game.apply(Command::BuildFacility {
        site,
        facility: c.facilities.id("mine").unwrap(),
        count: 1,
    })
    .unwrap();
    game.apply(Command::DevelopDeposit {
        site,
        deposit: c.deposits.id("grube").unwrap(),
    })
    .unwrap();
    game.apply(Command::SetProduction {
        site,
        slot: 0,
        recipe: c.recipes.id("erz_abbau"),
        utilization: 1.0,
    })
    .unwrap();
    site
}

fn stock(game: &Game, site: SiteId, product: &str) -> f64 {
    let p = game.catalog().products.id(product).unwrap();
    game.state()
        .site(site)
        .unwrap()
        .inventory
        .get(&p)
        .map_or(0.0, |s| s.quantity)
}

/// Hourly wage of a labor group in AAA.
fn wage(game: &Game, group: &str) -> f64 {
    let g = game.catalog().labor_groups.id(group).unwrap();
    let aaa = game.catalog().countries.id("AAA").unwrap();
    game.state().countries.get(aaa).hourly_wage_usd[g.index()]
}

#[test]
fn founding_and_building_cost_money() {
    let mut game = new_game(test_support::production());
    let site = mine(&mut game);
    let ledger = &game.state().company(game.player()).unwrap().ledger;
    // Site 100 000, mine 1 000 000, deposit 500 000.
    assert_eq!(ledger.cash(), usd(8_400_000.0));
    assert_eq!(ledger.balance(Account::FixedAssets), usd(100_000.0));
    assert_eq!(
        ledger.balance(Account::AssetsUnderConstruction),
        usd(1_500_000.0)
    );
    assert!(ledger.is_balanced());
    assert_eq!(game.state().site(site).unwrap().slots.len(), 1);

    // After construction and development everything is a fixed asset.
    days(&mut game, 31);
    let ledger = &game.state().company(game.player()).unwrap().ledger;
    assert_eq!(
        ledger.balance(Account::AssetsUnderConstruction),
        Money::ZERO
    );
    assert!(ledger.balance(Account::FixedAssets) > usd(1_590_000.0));
    assert!(ledger.is_balanced());
}

#[test]
fn the_mine_produces_after_development() {
    let mut game = new_game(test_support::production());
    let site = mine(&mut game);
    days(&mut game, 30);
    assert_eq!(
        stock(&game, site, "erz"),
        0.0,
        "deposit not ready before day 30"
    );
    days(&mut game, 10);
    // 100 runs per day from day 30 on, 1 t each.
    assert!(
        (stock(&game, site, "erz") - 1000.0).abs() < 1e-6,
        "{}",
        stock(&game, site, "erz")
    );
    // 200 hours per day at 8 hours per worker.
    let g = game.catalog().labor_groups.id("ungelernt").unwrap();
    assert!((game.state().site(site).unwrap().workforce.get(g) - 25.0).abs() < 1e-9);
    let deposit = game
        .state()
        .deposits
        .get(game.catalog().deposits.id("grube").unwrap());
    assert!((deposit.extracted - 1000.0).abs() < 1e-6);
}

#[test]
fn wages_are_paid_every_day() {
    let mut game = new_game(test_support::production());
    mine(&mut game);
    days(&mut game, 31);
    let month = &game.state().company(game.player()).unwrap().ledger.months[0];
    // January: one production day (31 Jan) with 25 workers × 8 h.
    let expected = 25.0 * 8.0 * wage(&game, "ungelernt");
    let paid = -month.by_type[&CostType::Personnel].to_usd();
    assert!((paid - expected).abs() < 0.01, "{paid} vs {expected}");
}

#[test]
fn yearly_cap_and_reserve_limit_extraction() {
    let mut game = new_game(test_support::production());
    let site = mine(&mut game);
    days(&mut game, 365);
    // At most 5 000 t per year.
    assert!((stock(&game, site, "erz") - 5_000.0).abs() < 1e-6);
    days(&mut game, 365 * 2);
    // The reserve of 10 000 t is exhausted after the second year.
    assert!((stock(&game, site, "erz") - 10_000.0).abs() < 1e-6);
    let deposit = game
        .state()
        .deposits
        .get(game.catalog().deposits.id("grube").unwrap());
    assert!((deposit.extracted - 10_000.0).abs() < 1e-6);
}

/// Mine plus furnace; ore is moved to the furnace once.
fn chain(game: &mut Game) -> (SiteId, SiteId) {
    let c = game.catalog().clone();
    let mine_site = mine(game);
    game.apply(Command::FoundSite {
        country: c.countries.id("AAA").unwrap(),
        kind: SiteType::Factory,
    })
    .unwrap();
    let works = SiteId(1);
    game.apply(Command::BuildFacility {
        site: works,
        facility: c.facilities.id("ofen").unwrap(),
        count: 1,
    })
    .unwrap();
    game.apply(Command::SetProduction {
        site: works,
        slot: 0,
        recipe: c.recipes.id("eisen_schmelzen"),
        utilization: 1.0,
    })
    .unwrap();
    days(game, 40);
    let ore = c.products.id("erz").unwrap();
    game.apply(Command::TransferGoods {
        from: mine_site,
        to: works,
        product: ore,
        quantity: 1000.0,
    })
    .unwrap();
    (mine_site, works)
}

#[test]
fn the_furnace_turns_ore_into_iron() {
    let mut game = new_game(test_support::production());
    let (_, works) = chain(&mut game);
    assert_eq!(stock(&game, works, "erz"), 1000.0);
    days(&mut game, 1);
    // 50 runs consume 100 t of ore; the iron needs two days.
    assert!((stock(&game, works, "erz") - 900.0).abs() < 1e-6);
    assert_eq!(stock(&game, works, "eisen"), 0.0);
    days(&mut game, 1);
    assert!((stock(&game, works, "eisen") - 50.0).abs() < 1e-6);

    let iron = game.catalog().products.id("eisen").unwrap();
    let s = &game.state().site(works).unwrap().inventory[&iron];
    // Ore of quality 50 → iron at base quality 60, minus a little wear.
    assert!(s.quality > 59.9 && s.quality <= 60.0, "{}", s.quality);
    assert!(
        game.state()
            .company(game.player())
            .unwrap()
            .ledger
            .is_balanced()
    );
}

#[test]
fn inventory_value_is_production_cost() {
    let mut game = new_game(test_support::production());
    let (_, works) = chain(&mut game);
    let ore = game.catalog().products.id("erz").unwrap();
    let ore_value_per_t = {
        let s = &game.state().site(works).unwrap().inventory[&ore];
        s.value.to_usd() / s.quantity
    };
    // Ore costs exactly its labor: 2 hours of unskilled work per t.
    assert!(
        (ore_value_per_t - 2.0 * wage(&game, "ungelernt")).abs() < 1e-3,
        "{ore_value_per_t}"
    );
    days(&mut game, 2);
    let iron = game.catalog().products.id("eisen").unwrap();
    let s = &game.state().site(works).unwrap().inventory[&iron];
    let per_t = s.value.to_usd() / s.quantity;
    let expected =
        2.0 * ore_value_per_t + wage(&game, "ungelernt") + wage(&game, "fachkraft.metall");
    assert!((per_t - expected).abs() < 1e-3, "{per_t} vs {expected}");
}

#[test]
fn commands_are_checked() {
    let mut game = new_game(test_support::production());
    let c = game.catalog().clone();
    let aaa = c.countries.id("AAA").unwrap();
    let bbb = c.countries.id("BBB").unwrap();
    game.apply(Command::FoundSite {
        country: aaa,
        kind: SiteType::Factory,
    })
    .unwrap();
    let works = SiteId(0);
    let err = |game: &mut Game, cmd| game.apply(cmd).unwrap_err();

    assert_eq!(
        err(
            &mut game,
            Command::BuildFacility {
                site: works,
                facility: c.facilities.id("mine").unwrap(),
                count: 1
            }
        ),
        CommandError::WrongSiteType {
            required: SiteType::Extraction
        }
    );
    assert_eq!(
        err(
            &mut game,
            Command::BuildFacility {
                site: works,
                facility: c.facilities.id("ofen_2000").unwrap(),
                count: 1
            }
        ),
        CommandError::TechnologyUnknown("hochofen_2000".into())
    );
    game.apply(Command::FoundSite {
        country: bbb,
        kind: SiteType::Extraction,
    })
    .unwrap();
    assert_eq!(
        err(
            &mut game,
            Command::DevelopDeposit {
                site: SiteId(1),
                deposit: c.deposits.id("grube").unwrap()
            }
        ),
        CommandError::DepositOtherCountry
    );
    game.apply(Command::BuildFacility {
        site: SiteId(1),
        facility: c.facilities.id("mine").unwrap(),
        count: 1,
    })
    .unwrap();
    assert_eq!(
        err(
            &mut game,
            Command::SetProduction {
                site: SiteId(1),
                slot: 0,
                recipe: c.recipes.id("erz_abbau"),
                utilization: 1.0
            }
        ),
        CommandError::RecipeNeedsDeposit
    );
    assert_eq!(
        err(
            &mut game,
            Command::SetProduction {
                site: SiteId(1),
                slot: 0,
                recipe: c.recipes.id("eisen_schmelzen"),
                utilization: 1.0
            }
        ),
        CommandError::RecipeNotForFacility
    );
    assert_eq!(
        err(
            &mut game,
            Command::SetAutomation {
                site: SiteId(1),
                slot: 0,
                level: 0.9
            }
        ),
        CommandError::AutomationTooHigh { max: 0.5 }
    );
    assert_eq!(
        err(
            &mut game,
            Command::TransferGoods {
                from: works,
                to: SiteId(1),
                product: c.products.id("erz").unwrap(),
                quantity: 1.0
            }
        ),
        CommandError::NoRoute {
            product: "erz".into(),
            from: "AAA".into(),
            to: "BBB".into()
        }
    );
    let expensive = Command::BuildFacility {
        site: works,
        facility: c.facilities.id("ofen").unwrap(),
        count: 1,
    };
    for _ in 0..4 {
        game.apply(expensive.clone()).unwrap();
    }
    assert!(matches!(
        err(&mut game, expensive),
        CommandError::NotEnoughCash { .. }
    ));
    assert_eq!(
        err(
            &mut game,
            Command::SetProduction {
                site: SiteId(9),
                slot: 0,
                recipe: None,
                utilization: 0.0
            }
        ),
        CommandError::UnknownSite
    );
}

#[test]
fn automation_saves_labor_and_costs_money() {
    let mut game = new_game(test_support::production());
    let site = mine(&mut game);
    let cash = game.state().company(game.player()).unwrap().ledger.cash();
    game.apply(Command::SetAutomation {
        site,
        slot: 0,
        level: 0.5,
    })
    .unwrap();
    // 0.5 × 1 000 000 × cost share 0.5
    assert_eq!(
        game.state().company(game.player()).unwrap().ledger.cash(),
        cash - usd(250_000.0)
    );
    days(&mut game, 31);
    let g = game.catalog().labor_groups.id("ungelernt").unwrap();
    let workers = *game.state().site(site).unwrap().workforce.get(g);
    // Labor factor 1 − 0.5 · 0.8 · (0.5 + 0.5 · affinity) is below 1.
    assert!(workers < 25.0 && workers > 12.0, "{workers}");
}

#[test]
fn the_labor_pool_limits_production() {
    let mut catalog = test_support::production();
    // Almost no skilled metal workers in the country.
    catalog.country_model.qualification_shares = vec![(1_000.0, vec![1.0 - 1e-7, 1e-7])];
    let mut game = new_game(catalog);
    let (_, works) = chain(&mut game);
    days(&mut game, 3);
    let g = game.catalog().labor_groups.id("fachkraft.metall").unwrap();
    let aaa = game.catalog().countries.id("AAA").unwrap();
    let pool = game.state().countries.get(aaa).labor_pool[g.index()];
    let workers = *game.state().site(works).unwrap().workforce.get(g);
    assert!(
        (workers - pool).abs() < 1e-12 && pool < 0.1,
        "{workers} {pool}"
    );
    // One metal hour per run: the furnace makes only pool × 8 runs a day.
    let runs = game.state().site(works).unwrap().slots[0].last_runs;
    assert!((runs - pool * 8.0).abs() < 1e-9, "{runs}");
}

#[test]
fn ledgers_stay_balanced_for_a_year() {
    let mut game = new_game(test_support::production());
    chain(&mut game);
    game.advance(RoundLength::Quarter, |_| {});
    for _ in 0..4 {
        game.advance(RoundLength::Quarter, |_| {});
    }
    let ledger = &game.state().company(game.player()).unwrap().ledger;
    assert!(ledger.is_balanced());
    assert_eq!(ledger.years.len(), 1);
    let assets: Money = Account::ALL
        .iter()
        .filter(|a| a.is_asset())
        .map(|&a| ledger.balance(a))
        .sum();
    let claims: Money = Account::ALL
        .iter()
        .filter(|a| !a.is_asset())
        .map(|&a| ledger.balance(a))
        .sum();
    assert_eq!(assets, claims);
    // The year's result equals the change in equity from operations.
    let result = ledger.years[0].total() + ledger.year.total();
    assert_eq!(
        ledger.balance(Account::RetainedEarnings) + ledger.balance(Account::Result),
        result
    );
}

#[test]
fn production_is_reproducible_through_save_and_replay() {
    let catalog = Arc::new(test_support::production());
    let play = |game: &mut Game| {
        for _ in 0..6 {
            game.advance(RoundLength::Week, |_| {});
        }
    };
    let mut a = new_game((*catalog).clone());
    chain(&mut a);
    play(&mut a);
    play(&mut a);

    let mut b = new_game((*catalog).clone());
    chain(&mut b);
    play(&mut b);
    let mut loaded = save::decode(&save::encode(&b), catalog.clone())
        .unwrap()
        .game;
    play(&mut loaded);
    assert_eq!(loaded.state_hash(), a.state_hash());

    let replayed = Game::replay(catalog, a.state().settings.clone(), a.journal()).unwrap();
    assert_eq!(replayed.state_hash(), a.state_hash());
    assert!(a.date() > Date::new(1900, 3, 1).unwrap());
}

/// Power plant fed with ore from the mine.
fn power_plant(game: &mut Game, mine_site: SiteId) -> SiteId {
    let c = game.catalog().clone();
    game.apply(Command::FoundSite {
        country: c.countries.id("AAA").unwrap(),
        kind: SiteType::PowerPlant,
    })
    .unwrap();
    let plant = SiteId(u32::try_from(game.state().sites.len() - 1).unwrap());
    game.apply(Command::BuildFacility {
        site: plant,
        facility: c.facilities.id("kraftwerk").unwrap(),
        count: 1,
    })
    .unwrap();
    game.apply(Command::SetProduction {
        site: plant,
        slot: 0,
        recipe: c.recipes.id("strom_erzeugen"),
        utilization: 1.0,
    })
    .unwrap();
    game.apply(Command::TransferGoods {
        from: mine_site,
        to: plant,
        product: c.products.id("erz").unwrap(),
        quantity: 100.0,
    })
    .unwrap();
    plant
}

#[test]
fn the_grid_limits_electric_production() {
    let mut game = new_game(test_support::power());
    let (_, works) = chain(&mut game);
    days(&mut game, 2);
    let aaa = game.catalog().countries.id("AAA").unwrap();
    let grid = game.state().countries.get(aaa).grid_share;
    assert!(grid > 0.1 && grid < 0.9, "{grid}");
    let runs = game.state().sites[works.index()].slots[0].last_runs;
    assert!(
        (runs - 50.0 * grid).abs() < 1e-6,
        "{runs} vs {}",
        50.0 * grid
    );
}

#[test]
fn own_power_plant_supplies_and_feeds_in() {
    let mut game = new_game(test_support::power());
    let (mine_site, works) = chain(&mut game);
    days(&mut game, 20);
    let plant = power_plant(&mut game, mine_site);
    let rest = stock(&game, mine_site, "erz");
    game.apply(Command::TransferGoods {
        from: mine_site,
        to: works,
        product: game.catalog().products.id("erz").unwrap(),
        quantity: rest,
    })
    .unwrap();
    // Half the furnace: 25 runs need 50 MWh, the plant makes 100.
    game.apply(Command::SetProduction {
        site: works,
        slot: 0,
        recipe: game.catalog().recipes.id("eisen_schmelzen"),
        utilization: 0.5,
    })
    .unwrap();
    days(&mut game, 3);
    let state = game.state();
    let runs = state.sites[works.index()].slots[0].last_runs;
    assert!((runs - 25.0).abs() < 1e-6, "{runs}");
    assert!((state.sites[plant.index()].slots[0].last_runs - 100.0).abs() < 1e-6);
    assert_eq!(
        stock(&game, plant, "strom"),
        0.0,
        "surplus went into the grid"
    );
    let power = game.catalog().products.id("strom").unwrap();
    let ledger = &state.companies[game.player().index()].ledger;
    assert!(ledger.month.by_product[&power] != Money::ZERO);
    assert!(ledger.month.by_type[&CostType::Revenue] > Money::ZERO);
    assert!(ledger.is_balanced());
}
