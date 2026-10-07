//! Scenario tests for plots of land (M35) with the small chain of
//! `test_support::production`.

use std::sync::Arc;

use crate::calendar::{Date, RoundLength};
use crate::catalog::{
    Catalog, Location, LocationModel, PlotClass, PlotModel, SiteType, test_support,
};
use crate::command::{Command, CommandError};
use crate::deals::{DealObject, OfferAnswer};
use crate::game::Game;
use crate::ledger::{Account, CostType, Ledger};
use crate::money::Money;
use crate::plots;
use crate::state::{CompanyId, GameSettings, PlotId, SiteId, StartForm, Tenure};

fn usd(v: f64) -> Money {
    Money::from_usd(v).unwrap()
}

/// Plots of 4–8 ha (rural ones twice as large); a furnace (2 million USD) takes 2 ha
/// plus a fifth for ways.
fn catalog() -> Catalog {
    let mut c = test_support::production();
    c.deal_model.min_age_months = 0;
    let location =
        |share, area_factor, price_factor, hiring, sea_freight, delivery_cost| LocationModel {
            share,
            area_factor,
            price_factor,
            hiring,
            sea_freight,
            delivery_cost,
        };
    c.plot_model = PlotModel {
        classes: vec![PlotClass {
            key: "standard".into(),
            area_ha: (4.0, 8.0),
            shares: [1.0; 3],
        }],
        locations: [
            location(0.5, 1.0, 2.0, 0.1, 1.0, 0.0),
            location(0.25, 1.0, 1.2, 0.0, 0.8, 0.0),
            location(0.25, 2.0, 0.5, -0.1, 1.0, 0.02),
        ],
        land_price_usd_per_ha: 100_000.0,
        investment_per_ha_usd: 1_000_000.0,
        ..PlotModel::default()
    };
    c
}

fn new_game(seed: u64) -> Game {
    let catalog = Arc::new(catalog());
    let settings = GameSettings {
        seed,
        start_year: 1900,
        start_country: catalog.countries.id("AAA").unwrap(),
        start_capital: usd(20_000_000.0),
        start_form: StartForm::Workshop,
        company_name: "Hütte AG".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: Default::default(),
        ventures: 1.0,
    };
    Game::new(catalog, settings).unwrap()
}

fn aaa(game: &Game) -> crate::ids::CountryId {
    game.catalog().countries.id("AAA").unwrap()
}

/// A free plot in AAA at `location`, the smallest first.
fn free_plot(game: &Game, location: Location) -> PlotId {
    let country = aaa(game);
    let (i, _) = game
        .state()
        .plots
        .iter()
        .enumerate()
        .filter(|(_, p)| p.country == country && p.site.is_none() && p.location == location)
        .min_by(|a, b| a.1.area_ha.total_cmp(&b.1.area_ha).then(a.0.cmp(&b.0)))
        .expect("a free plot");
    PlotId(u32::try_from(i).unwrap())
}

fn last_site(game: &Game) -> SiteId {
    SiteId(u32::try_from(game.state().sites.len() - 1).unwrap())
}

fn books(game: &Game) -> &Ledger {
    &game.state().companies[0].ledger
}

#[test]
fn countries_offer_land_in_proportion_to_their_economy() {
    let game = new_game(3);
    let state = game.state();
    let c = game.catalog();
    for (country, _) in c.countries.iter() {
        let v = state.countries.get(country);
        let target = 25.0 * v.population * v.gdp_per_capita_usd / 1e9;
        let (all, _) = plots::land(state, country);
        assert!(all >= target && all < target + 16.0, "{all} for {target}");
    }
    // The same seed draws the same plots, another seed others.
    assert_eq!(new_game(3).state().plots, state.plots);
    assert_ne!(new_game(4).state().plots, state.plots);
    // Ports only where there is a coast; the sizes follow class and location.
    for p in &state.plots {
        let factor = if p.location == Location::Rural {
            2.0
        } else {
            1.0
        };
        assert!((4.0 * factor..=8.0 * factor).contains(&p.area_ha), "{p:?}");
    }

    // The economy grows: on 1 January more plots come on the market.
    let mut game = new_game(3);
    let before = game.state().plots.len();
    for _ in 0..4 {
        game.advance(RoundLength::Quarter, |_| {});
    }
    assert_eq!(game.state().date, Date::new(1901, 1, 1).unwrap());
    assert!(game.state().plots.len() > before);
    assert!(
        game.state()
            .plots
            .iter()
            .skip(before)
            .all(|p| p.since == 1901)
    );
}

#[test]
fn a_site_buys_or_leases_its_plot() {
    let mut game = new_game(3);
    let city = free_plot(&game, Location::City);
    let value = plots::value(game.catalog(), game.state(), city);
    let cash = books(&game).cash();
    game.apply(Command::FoundSiteOnPlot {
        plot: city,
        kind: SiteType::Factory,
        lease: false,
    })
    .unwrap();
    let bought = last_site(&game);
    let building = game.catalog().production_model.site_cost(SiteType::Factory);
    assert_eq!(books(&game).cash(), cash - building - value);
    assert_eq!(books(&game).balance(Account::Land), value);
    let p = &game.state().plots[city.index()];
    assert_eq!((p.site, p.tenure), (Some(bought), Tenure::Owned(value)));
    assert_eq!(game.state().sites[bought.index()].plot, Some(city));
    // A plot can hold one site only.
    assert_eq!(
        game.apply(Command::FoundSiteOnPlot {
            plot: city,
            kind: SiteType::Factory,
            lease: true,
        }),
        Err(CommandError::PlotTaken)
    );

    // Leased: nothing now, rent from the next month on.
    let port = free_plot(&game, Location::Port);
    game.apply(Command::FoundSiteOnPlot {
        plot: port,
        kind: SiteType::Factory,
        lease: true,
    })
    .unwrap();
    let leased = last_site(&game);
    assert_eq!(books(&game).balance(Account::Land), value);
    game.advance(RoundLength::Month, |_| {});
    let rent = plots::value(game.catalog(), game.state(), port).scale(0.05 / 12.0);
    let month = &books(&game).month;
    assert_eq!(month.by_type.get(&CostType::Rent).copied(), Some(-rent));
    assert_eq!(month.by_site.get(&leased).copied(), Some(-rent));

    // Bought later at today's value.
    let today = plots::value(game.catalog(), game.state(), port);
    game.apply(Command::BuyPlot { site: leased }).unwrap();
    assert_eq!(books(&game).balance(Account::Land), value + today);
    assert_eq!(
        game.apply(Command::BuyPlot { site: leased }),
        Err(CommandError::PlotOwned)
    );
    assert!(books(&game).is_balanced());

    // Extraction sites stand on their concession.
    let free = free_plot(&game, Location::Rural);
    assert_eq!(
        game.apply(Command::FoundSiteOnPlot {
            plot: free,
            kind: SiteType::Extraction,
            lease: true,
        }),
        Err(CommandError::PlotNotNeeded)
    );
    let country = aaa(&game);
    game.apply(Command::FoundSite {
        country,
        kind: SiteType::Extraction,
    })
    .unwrap();
    assert_eq!(game.state().sites[last_site(&game).index()].plot, None);
}

#[test]
fn a_full_plot_takes_no_more_facilities() {
    let mut game = new_game(3);
    let plot = free_plot(&game, Location::City);
    let area = game.state().plots[plot.index()].area_ha;
    game.apply(Command::FoundSiteOnPlot {
        plot,
        kind: SiteType::Factory,
        lease: true,
    })
    .unwrap();
    let site = last_site(&game);
    let furnace = game.catalog().facilities.id("ofen").unwrap();
    // 2.4 ha per furnace with ways: one or two fit on 4–8 ha, never four.
    let fit = plots::units_that_fit(
        game.catalog(),
        game.state(),
        site,
        (furnace, crate::catalog::FacilitySize::Medium),
    );
    assert_eq!(fit, (area / 2.4 + 1e-9).floor() as u32);
    game.apply(Command::BuildFacility {
        site,
        facility: furnace,
        count: fit,
        size: crate::catalog::FacilitySize::Medium,
    })
    .unwrap();
    let Err(CommandError::PlotTooSmall { needed_ha, area_ha }) =
        game.apply(Command::BuildFacility {
            site,
            facility: furnace,
            count: 1,
            size: crate::catalog::FacilitySize::Medium,
        })
    else {
        panic!("the plot is full");
    };
    assert!((needed_ha - 2.4 * f64::from(fit + 1)).abs() < 1e-9);
    assert_eq!(area_ha, area);
}

#[test]
fn the_choice_weighs_land_against_deliveries() {
    let game = new_game(3);
    let (c, state) = (game.catalog(), game.state());
    let country = aaa(&game);
    let cheapest = |rural: bool| {
        state
            .plots
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                p.country == country
                    && p.site.is_none()
                    && p.area_ha >= 3.0
                    && (rural || p.location != Location::Rural)
            })
            .map(|(i, _)| plots::value(c, state, PlotId(u32::try_from(i).unwrap())))
            .min()
            .unwrap()
    };
    let chosen = |revenue: f64| plots::choose(c, state, country, 3.0, revenue).expect("a plot");
    // Without sales in the country: the plot with the lowest rent.
    assert_eq!(plots::value(c, state, chosen(0.0)), cheapest(true));
    // With large sales the deliveries from the country side cost more than dearer land.
    let near = chosen(100_000_000.0);
    assert_ne!(state.plots[near.index()].location, Location::Rural);
    assert_eq!(plots::value(c, state, near), cheapest(false));
    // Nothing holds 50 ha: the largest free plot.
    let largest = plots::choose(c, state, country, 50.0, 0.0).unwrap();
    let max = state
        .plots
        .iter()
        .filter(|p| p.country == country)
        .map(|p| p.area_ha)
        .fold(0.0, f64::max);
    assert_eq!(state.plots[largest.index()].area_ha, max);
}

#[test]
fn the_city_draws_workers_and_the_port_cheapens_freight() {
    let mut game = new_game(3);
    for location in [Location::City, Location::Port, Location::Rural] {
        let plot = free_plot(&game, location);
        game.apply(Command::FoundSiteOnPlot {
            plot,
            kind: SiteType::Factory,
            lease: true,
        })
        .unwrap();
    }
    let (c, state) = (game.catalog(), game.state());
    let n = state.sites.len();
    let site = |i: usize| &state.sites[i];
    assert_eq!(plots::hiring(c, state, site(n - 3)), 0.1);
    assert_eq!(plots::hiring(c, state, site(n - 1)), -0.1);
    assert_eq!(plots::sea_freight(c, state, site(n - 2)), 0.8);
    assert_eq!(plots::delivery_cost(c, state, site(n - 1)), 0.02);
    assert_eq!(plots::delivery_cost(c, state, site(n - 3)), 0.0);
}

#[test]
fn a_bankrupt_company_gives_its_plots_back() {
    let mut game = new_game(3);
    let plot = free_plot(&game, Location::City);
    game.apply(Command::FoundSiteOnPlot {
        plot,
        kind: SiteType::Factory,
        lease: true,
    })
    .unwrap();
    let site = last_site(&game);
    crate::ai::release_assets(game.state_mut(), CompanyId(0));
    assert_eq!(game.state().plots[plot.index()].site, None);
    assert_eq!(game.state().sites[site.index()].plot, None);
}

#[test]
fn saves_from_before_plots_get_bought_plots() {
    let mut game = new_game(3);
    let plot = free_plot(&game, Location::City);
    game.apply(Command::FoundSiteOnPlot {
        plot,
        kind: SiteType::Factory,
        lease: true,
    })
    .unwrap();
    // As in a save from before M35: no plots at all.
    let state = game.state_mut();
    state.plots.clear();
    for s in &mut state.sites {
        s.plot = None;
    }
    let catalog = game.catalog().clone();
    game.state_mut().fit_to_catalog(&catalog);
    let state = game.state();
    assert!(!state.plots.is_empty());
    for s in state
        .sites
        .iter()
        .filter(|s| s.kind != SiteType::Extraction)
    {
        let p = &state.plots[s.plot.expect("a plot").index()];
        assert_eq!(p.tenure, Tenure::Owned(Money::ZERO));
        assert!(p.area_ha >= plots::site_area(&catalog, s, None));
    }
    assert!(state.companies[0].ledger.is_balanced());
}

#[test]
fn a_sold_site_takes_its_bought_plot_along() {
    let mut game = new_game(3);
    let state = game.state_mut();
    let mut rival = state.companies[0].clone();
    "Rivale AG".clone_into(&mut rival.name);
    rival.ledger = Ledger::new(state.date, usd(50_000_000.0));
    state.companies.push(rival);
    let rival = CompanyId(1);
    let plot = free_plot(&game, Location::City);
    game.apply(Command::FoundSiteOnPlot {
        plot,
        kind: SiteType::Factory,
        lease: false,
    })
    .unwrap();
    let site = last_site(&game);
    let land = books(&game).balance(Account::Land);
    assert!(land > Money::ZERO);
    let value = crate::deals::site_value(game.state(), game.catalog(), site);
    assert_eq!(value.land_book, land);
    game.apply_as(
        rival,
        Command::MakeOffer {
            seller: CompanyId(0),
            object: DealObject::Site(site),
            price: value.base + usd(1_000.0),
        },
    )
    .unwrap();
    game.apply(Command::AnswerOffer {
        offer: 0,
        answer: OfferAnswer::Accept,
    })
    .unwrap();
    let state = game.state();
    assert_eq!(
        state.companies[0].ledger.balance(Account::Land),
        Money::ZERO
    );
    assert_eq!(state.companies[1].ledger.balance(Account::Land), value.land);
    assert_eq!(state.plots[plot.index()].tenure, Tenure::Owned(value.land));
    assert_eq!(state.plots[plot.index()].site, Some(site));
    for c in &state.companies {
        assert!(c.ledger.is_balanced(), "{}", c.name);
    }
}
