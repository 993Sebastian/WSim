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
        ventures: 1.0,
        tariff_dynamics: 1.0,
        event_effects: true,
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
        size: crate::catalog::FacilitySize::Medium,
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
    let mut catalog = test_support::production();
    let grube = catalog.deposits.id("grube").unwrap();
    let d = catalog.deposits.get_mut(grube);
    d.max_output_per_year = 5_000.0;
    d.reserve = Some(10_000.0);
    let mut game = new_game(catalog);
    let site = mine(&mut game);
    days(&mut game, 100);
    // The yearly output spreads over the year (M33): by 10 April a hundred days' share.
    let share = 5_000.0 * 100.0 / 365.0;
    assert!(
        (stock(&game, site, "erz") - share).abs() < 1.0,
        "{}",
        stock(&game, site, "erz")
    );
    days(&mut game, 265);
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

/// M41: the workable reserve grows with the output index of the raw material. With the
/// index doubled from 1901 the reserve of 5 000 t yields 10 000 t (before, 5 000 t).
#[test]
fn the_reserve_grows_with_the_output_index() {
    let mut catalog = test_support::production();
    let grube = catalog.deposits.id("grube").unwrap();
    let d = catalog.deposits.get_mut(grube);
    d.max_output_per_year = 5_000.0;
    d.reserve = Some(5_000.0);
    let ore = d.resource;
    catalog.products.get_mut(ore).output_index =
        Some(crate::time_series::TimeSeries::new(vec![(1900, 1.0), (1901, 2.0)]).unwrap());
    assert_eq!(catalog.reserve(grube, 1900), Some(5_000.0));
    assert_eq!(catalog.reserve(grube, 1901), Some(10_000.0));
    let mut game = new_game(catalog);
    mine(&mut game);
    days(&mut game, 365 * 3);
    let deposit = game.state().deposits.get(grube);
    assert!(
        (deposit.extracted - 10_000.0).abs() < 1e-6,
        "{}",
        deposit.extracted
    );
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
        size: crate::catalog::FacilitySize::Medium,
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
                count: 1,
                size: crate::catalog::FacilitySize::Medium,
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
                count: 1,
                size: crate::catalog::FacilitySize::Medium,
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
        size: crate::catalog::FacilitySize::Medium,
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
        size: crate::catalog::FacilitySize::Medium,
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
        size: crate::catalog::FacilitySize::Medium,
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

/// M18: a site that pays a higher wage premium gets scarce workers first and hires them
/// away from a site in the same country that pays less.
#[test]
fn a_higher_wage_premium_hires_workers_away() {
    let mut catalog = test_support::production();
    // Almost no skilled metal workers in the country.
    catalog.country_model.qualification_shares = vec![(1_000.0, vec![1.0 - 1e-7, 1e-7])];
    let mut game = new_game(catalog);
    let (mine_site, works) = chain(&mut game);
    let c = game.catalog().clone();
    game.apply(Command::FoundSite {
        country: c.countries.id("AAA").unwrap(),
        kind: SiteType::Factory,
    })
    .unwrap();
    let second = SiteId(2);
    game.apply(Command::BuildFacility {
        site: second,
        facility: c.facilities.id("ofen").unwrap(),
        count: 1,
        size: crate::catalog::FacilitySize::Medium,
    })
    .unwrap();
    game.apply(Command::SetProduction {
        site: second,
        slot: 0,
        recipe: c.recipes.id("eisen_schmelzen"),
        utilization: 1.0,
    })
    .unwrap();
    days(&mut game, 21);
    game.apply(Command::TransferGoods {
        from: mine_site,
        to: second,
        product: c.products.id("erz").unwrap(),
        quantity: 500.0,
    })
    .unwrap();
    days(&mut game, 1);
    let g = c.labor_groups.id("fachkraft.metall").unwrap();
    let aaa = c.countries.id("AAA").unwrap();
    let pool = game.state().countries.get(aaa).labor_pool[g.index()];
    let metal = |game: &Game, site: SiteId| *game.state().site(site).unwrap().workforce.get(g);
    // Same premium: nobody hires away; the older furnace keeps the workers.
    assert!((metal(&game, works) - pool).abs() < 1e-12, "{pool}");
    assert_eq!(metal(&game, second), 0.0);

    game.apply(Command::SetWagePremium {
        site: second,
        premium: 0.2,
    })
    .unwrap();
    days(&mut game, 1);
    assert!((metal(&game, second) - pool).abs() < 1e-12);
    assert!(metal(&game, works) < 1e-12);
    assert!(game.state().site(second).unwrap().slots[0].last_runs > 0.0);
    // The furnace that lost its workers cannot hire them back.
    days(&mut game, 3);
    assert!(metal(&game, works) < 1e-12);
    for company in &game.state().companies {
        assert!(company.ledger.is_balanced());
    }
}

#[test]
fn the_wage_premium_raises_the_wage_bill() {
    let mut game = new_game(test_support::production());
    let site = mine(&mut game);
    game.apply(Command::SetWagePremium { site, premium: 0.5 })
        .unwrap();
    days(&mut game, 31);
    let month = &game.state().company(game.player()).unwrap().ledger.months[0];
    let expected = 25.0 * 8.0 * wage(&game, "ungelernt") * 1.5;
    let paid = -month.by_type[&CostType::Personnel].to_usd();
    assert!((paid - expected).abs() < 0.01, "{paid} vs {expected}");
    // The ore carries the higher wages in its value.
    let ore = game.catalog().products.id("erz").unwrap();
    let s = &game.state().site(site).unwrap().inventory[&ore];
    let per_t = s.value.to_usd() / s.quantity;
    assert!((per_t - 2.0 * wage(&game, "ungelernt") * 1.5).abs() < 1e-3);

    let err = |game: &mut Game, premium| {
        game.apply(Command::SetWagePremium { site, premium })
            .unwrap_err()
    };
    let invalid = CommandError::InvalidWagePremium { max: 1.0 };
    assert_eq!(err(&mut game, 1.5), invalid);
    assert_eq!(err(&mut game, -0.1), invalid);
    assert_eq!(err(&mut game, f64::NAN), invalid);
}

/// M18: the asking price can be set directly, also while it follows the market.
#[test]
fn prices_can_be_set_in_both_modes() {
    use crate::state::PriceMode;
    let mut game = new_game(test_support::production());
    let (_, works) = chain(&mut game);
    let c = game.catalog().clone();
    let iron = c.products.id("eisen").unwrap();
    let offer = |game: &Game| game.state().site(works).unwrap().offers[&iron].clone();
    game.apply(Command::SetSale {
        site: works,
        product: iron,
        mode: Some(PriceMode::Market {
            markup: 0.0,
            floor: usd(50.0),
        }),
        keep: 0.0,
    })
    .unwrap();
    game.apply(Command::SetPrice {
        site: works,
        product: iron,
        price: usd(80.0),
    })
    .unwrap();
    assert_eq!(offer(&game).price, usd(80.0));
    assert!(matches!(offer(&game).mode, PriceMode::Market { .. }));
    // Never below the floor.
    game.apply(Command::SetPrice {
        site: works,
        product: iron,
        price: usd(30.0),
    })
    .unwrap();
    assert_eq!(offer(&game).price, usd(50.0));
    game.apply(Command::SetSale {
        site: works,
        product: iron,
        mode: Some(PriceMode::Fixed(usd(120.0))),
        keep: 0.0,
    })
    .unwrap();
    game.apply(Command::SetPrice {
        site: works,
        product: iron,
        price: usd(110.0),
    })
    .unwrap();
    assert_eq!(offer(&game).price, usd(110.0));
    assert_eq!(offer(&game).mode, PriceMode::Fixed(usd(110.0)));
    assert_eq!(
        game.apply(Command::SetPrice {
            site: works,
            product: c.products.id("erz").unwrap(),
            price: usd(1.0),
        })
        .unwrap_err(),
        CommandError::NoOffer("erz".into())
    );
    assert_eq!(
        game.apply(Command::SetPrice {
            site: works,
            product: iron,
            price: Money::ZERO,
        })
        .unwrap_err(),
        CommandError::InvalidPrice
    );
}

/// M18: expected unit costs by kind, and the wages of the hours used booked on the product.
#[test]
fn unit_costs_add_up_and_follow_the_wage_premium() {
    let mut game = new_game(test_support::production());
    let (_, works) = chain(&mut game);
    let c = game.catalog().clone();
    let iron = c.products.id("eisen").unwrap();
    let aaa = c.countries.id("AAA").unwrap();
    let costs = crate::production::unit_costs(&c, game.state(), works);
    assert_eq!(costs.len(), 1);
    let u = &costs[0];
    assert_eq!(u.product, iron);
    assert!((u.output_per_day - 50.0).abs() < 1e-9);
    let ore_price =
        crate::market::market_price(&c, game.state(), aaa, c.products.id("erz").unwrap());
    assert!((u.material - 2.0 * ore_price.to_usd()).abs() < 1e-9);
    let labor = wage(&game, "ungelernt") + wage(&game, "fachkraft.metall");
    assert!((u.labor - labor).abs() < 1e-9, "{} vs {labor}", u.labor);
    // Furnace: 2 000 000 USD over 20 years plus 3.65 % maintenance, 50 t a day.
    let facility = 2_000_000.0 * (1.0 / 20.0 + 0.0365) / 365.0 / 50.0;
    assert!((u.facility - facility).abs() < 1e-9);
    assert!((u.total() - u.variable() - u.facility).abs() < 1e-9);

    game.apply(Command::SetWagePremium {
        site: works,
        premium: 0.25,
    })
    .unwrap();
    let higher = crate::production::unit_costs(&c, game.state(), works);
    assert!((higher[0].labor - 1.25 * labor).abs() < 1e-9);

    // Without sales the product's result is zero: its costs went into the stock.
    days(&mut game, 3);
    let ledger = &game.state().company(game.player()).unwrap().ledger;
    assert_eq!(ledger.month.by_product[&iron], Money::ZERO);
    assert!(ledger.month.site_type(works, CostType::Personnel) < Money::ZERO);
}
