//! Scenario tests for machine tenders (MAA): a tender brings offers over its days, an
//! accepted offer sets price, build time and machine values, and the ledger stays balanced.

use std::sync::Arc;

use crate::calendar::RoundLength;
use crate::catalog::{
    Catalog, FacilitySize, MachineBuilder, MachineOfferModel, SiteType, test_support,
};
use crate::command::{Command, CommandError};
use crate::game::Game;
use crate::money::Money;
use crate::product_research;
use crate::state::{GameSettings, SiteId, StartForm};

fn usd(v: f64) -> Money {
    Money::from_usd(v).unwrap()
}

fn model() -> MachineOfferModel {
    MachineOfferModel {
        enabled: true,
        tender_days: 28,
        valid_days: 14,
        offers_per_week: (1, 5),
        normal_share: 0.85,
        deviation: 0.35,
        outlier: (0.7, 1.0),
        output: 0.2,
        labor: 0.15,
        inputs: 0.1,
        scrap: 0.2,
        price_performance: 0.12,
        price_position: 0.08,
        area_spread: 0.1,
        build_time: 0.2,
        noise: 0.03,
        limit: 0.2,
        volume_discount: 0.1,
        ai_threshold: 0.02,
        ..Default::default()
    }
}

fn catalog() -> Catalog {
    let mut c = test_support::production();
    c.machine_offers = model();
    let country = c.countries.id("AAA").unwrap();
    for (key, tendency) in [("nordstahl", 0.2), ("sudwerk", -0.2), ("mittelbau", 0.0)] {
        c.machine_builders
            .insert(
                key,
                MachineBuilder {
                    country,
                    tendency,
                    from_year: 1800,
                    until_year: None,
                },
            )
            .unwrap();
    }
    c
}

fn game() -> Game {
    let catalog = Arc::new(catalog());
    let settings = GameSettings {
        seed: 9,
        start_year: 1900,
        start_country: catalog.countries.id("AAA").unwrap(),
        start_capital: usd(50_000_000.0),
        start_form: StartForm::Workshop,
        company_name: "Werk AG".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: Default::default(),
        ventures: 1.0,
        tariff_dynamics: 1.0,
        event_effects: true,
        research_mandatory: false,
        found_at_start: true,
        person: Default::default(),
    };
    Game::new(catalog, settings).unwrap()
}

fn factory(game: &mut Game) -> SiteId {
    let c = game.catalog().clone();
    game.apply(Command::FoundSite {
        country: c.countries.id("AAA").unwrap(),
        kind: SiteType::Factory,
    })
    .unwrap();
    SiteId(u32::try_from(game.state().sites.len() - 1).unwrap())
}

fn days(game: &mut Game, n: u32) {
    for _ in 0..n {
        game.advance(RoundLength::Day, |_| {});
    }
}

fn iron(game: &Game) -> crate::ids::ProductId {
    game.catalog().products.id("eisen").unwrap()
}

fn request(game: &mut Game, site: SiteId, count: u32) -> Result<(), CommandError> {
    let product = iron(game);
    game.apply(Command::RequestMachineOffers {
        site,
        product,
        size: FacilitySize::Medium,
        count,
    })
    .map(|_| ())
}

#[test]
fn a_tender_brings_offers_and_one_runs_per_product_and_size() {
    let mut game = game();
    let site = factory(&mut game);
    request(&mut game, site, 1).unwrap();
    assert!(matches!(
        request(&mut game, site, 1),
        Err(CommandError::TenderRunning)
    ));
    let mut news = 0;
    for _ in 0..28 {
        let report = game.advance(RoundLength::Day, |_| {});
        news += report
            .messages
            .iter()
            .filter(|m| m.key == crate::message::keys::MACHINE_OFFERS_NEW)
            .count();
    }
    // One to five offers a week, in four weeks.
    let t = &game.state().machine_tenders;
    assert_eq!(t.len(), 1, "the tender stays while offers are valid");
    let offers = &t[0].offers;
    assert!(news >= 1);
    assert!(!offers.is_empty() && offers.len() <= 20);
    for o in offers {
        for v in [
            o.factors.output,
            o.factors.labor,
            o.factors.inputs,
            o.factors.scrap,
            o.factors.area,
            o.factors.price,
        ] {
            assert!((0.8 - 1e-9..=1.2 + 1e-9).contains(&v), "factor {v}");
        }
        assert!(o.valid_until >= game.state().date.add_days(-1));
    }
    // After the offers lapsed the tender is gone and a new one may start.
    days(&mut game, 20);
    assert!(game.state().machine_tenders.is_empty());
    request(&mut game, site, 1).unwrap();
}

#[test]
fn an_accepted_offer_sets_price_build_time_and_machine_values() {
    let mut game = game();
    let site = factory(&mut game);
    let player = game.player();
    request(&mut game, site, 2).unwrap();
    days(&mut game, 10);
    let tender = game.state().machine_tenders[0].clone();
    let offer = tender.offers[0].clone();
    let cash = game.state().companies[player.index()].ledger.cash();
    game.apply(Command::AcceptMachineOffer {
        tender: tender.id,
        offer: offer.id,
    })
    .unwrap();
    let s = game.state();
    assert!(s.machine_tenders.is_empty());
    let slot = s.sites[site.index()].slots.last().unwrap();
    let c = game.catalog().clone();
    assert_eq!(slot.count, 2);
    assert_eq!(slot.machine.unwrap().builder, Some(offer.builder));
    assert_eq!(slot.investment(&c), offer.price);
    assert_eq!(slot.ready, s.date.add_days(offer.build_days as i32));
    assert_eq!(slot.recipe, product_research::main_recipe(&c, iron(&game)));
    let l = &s.companies[player.index()].ledger;
    assert!(cash - l.cash() >= offer.price);
    assert!(l.is_balanced());
    // Finished, the machine moves its price from the construction account.
    days(&mut game, offer.build_days + 1);
    let l = &game.state().companies[player.index()].ledger;
    assert!(l.is_balanced());
}

#[test]
fn offers_are_the_same_in_a_replay() {
    let mut game = game();
    let site = factory(&mut game);
    request(&mut game, site, 1).unwrap();
    days(&mut game, 20);
    let replay = Game::replay(
        game.catalog().clone(),
        game.state().settings.clone(),
        game.journal(),
    )
    .unwrap();
    assert_eq!(replay.state().machine_tenders, game.state().machine_tenders);
}

#[test]
fn a_larger_purchase_gets_a_volume_discount() {
    let c = catalog();
    let m = &c.machine_offers;
    let base = crate::procurement::Base {
        facility: c.facilities.id("ofen").unwrap(),
        recipe: c.recipes.id("eisen_schmelzen").unwrap(),
        size: FacilitySize::Medium,
        count: 1,
        unit: usd(1_000_000.0),
        scale: 1.0,
        days: 100,
    };
    let price = |n: u32| {
        let mut rng = crate::rng::SimRng::for_stream(1, crate::rng::Stream::World);
        let (f, _, _) = crate::procurement::draw_offer(&mut rng, m, 0.0, n, &base);
        f.price
    };
    let (p1, p2, p4) = (price(1), price(2), price(4));
    // The same draw: the second unit saves most, the fourth hardly more.
    let (d2, d4) = (1.0 - p2 / p1, 1.0 - p4 / p1);
    assert!(d2 > 0.0 && d4 > d2);
    assert!((d4 - d2) < d2, "{d2} {d4}");
}

#[test]
fn the_production_view_lists_tenders_with_costs_and_land() {
    let mut g = game();
    let site = factory(&mut g);
    request(&mut g, site, 2).unwrap();
    // Before any offer the tender shows its day and last day.
    let early = crate::views::production(&g);
    assert_eq!(early.tenders.len(), 1);
    assert_eq!(early.tenders[0].day, 1);
    assert_eq!(early.tenders[0].days, 28);
    assert_eq!(early.tenders[0].count, 2);
    assert!(early.tenders[0].offers.is_empty());
    days(&mut g, 28);
    let view = crate::views::production(&g);
    let tender = &view.tenders[0];
    assert!(!tender.offers.is_empty());
    for o in &tender.offers {
        assert!(o.price_usd > 0.0 && o.catalog_price_usd > 0.0);
        assert!(o.build_days > 0 && o.ready > view.date);
        assert!(o.area_total_m2 > 0.0, "land of the units is shown");
        assert!(o.unit_cost_usd >= o.material_usd && o.material_usd >= 0.0);
        assert!(o.catalog_unit_cost_usd > 0.0);
    }
}

#[test]
fn a_decision_on_the_day_takes_the_best_offer_and_builds_on_its_terms() {
    let mut g = game();
    let site = factory(&mut g);
    let product = iron(&g);
    let catalog = g.catalog().clone();
    let me = g.player();
    let kind = crate::procurement::TenderKind::Build {
        product,
        size: FacilitySize::Medium,
        count: 1,
    };
    let mut state = g.state().clone();
    let best = crate::procurement::best_offer_now(&mut state, &catalog, me, (site, kind))
        .expect("an offer");
    // The same state and the same call give the same offer; ids of tenders move on.
    let mut again = g.state().clone();
    let same = crate::procurement::best_offer_now(&mut again, &catalog, me, (site, kind)).unwrap();
    assert_eq!(best, same);
    assert_ne!(state.next_machine_tender, g.state().next_machine_tender);
    // Building on its terms keeps its price and machine values.
    let facility = catalog
        .recipes
        .get(product_research::main_recipe(&catalog, product).unwrap())
        .facility;
    crate::command::build_facility_with_offer(
        &mut state,
        &catalog,
        me,
        (site, facility, 1, FacilitySize::Medium),
        &best,
    )
    .unwrap();
    let slot = state.sites[site.index()].slots.last().unwrap();
    assert_eq!(slot.machine.map(|m| m.output), Some(best.factors.output));
    assert_eq!(slot.cost, best.price);
    assert!(state.companies[me.index()].ledger.is_balanced());
}
