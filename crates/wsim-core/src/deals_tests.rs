//! Scenario tests for offers between companies (M30) with the small chain of
//! `test_support::production`.

use std::sync::Arc;

use crate::calendar::{Date, RoundLength};
use crate::catalog::{Catalog, SiteType, Span, test_support};
use crate::command::{Command, CommandError};
use crate::deals::{DealObject, OfferAnswer, OfferStatus, site_value};
use crate::game::Game;
use crate::ledger::{Account, CostType, Ledger};
use crate::message::keys;
use crate::money::Money;
use crate::state::{AiState, CompanyId, Consignee, GameSettings, SiteId, StartForm};

fn usd(v: f64) -> Money {
    Money::from_usd(v).unwrap()
}

fn catalog() -> Catalog {
    let mut c = test_support::production();
    // Sites can be bought at once; the rules for young sites are tested on their own.
    c.deal_model.min_age_months = 0;
    c
}

/// The player (with its start workshop) and a rival with 50 million in cash; an AI
/// company that does not run its plants on its own if `ai`.
fn new_game(catalog: Catalog, ai: bool) -> (Game, CompanyId) {
    let catalog = Arc::new(catalog);
    let settings = GameSettings {
        seed: 3,
        start_year: 1900,
        start_country: catalog.countries.id("AAA").unwrap(),
        start_capital: usd(20_000_000.0),
        start_form: StartForm::Workshop,
        company_name: "Hütte AG".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: Default::default(),
    };
    let mut game = Game::new(catalog, settings).unwrap();
    let state = game.state_mut();
    let mut rival = state.companies[0].clone();
    "Rivale AG".clone_into(&mut rival.name);
    rival.ledger = Ledger::new(state.date, usd(50_000_000.0));
    rival.ai = ai.then(|| AiState {
        competence: 0.5,
        aggressiveness: 0.5,
        real: None,
        next_operations: Date::new(2100, 1, 1).unwrap(),
    });
    state.companies.push(rival);
    (game, CompanyId(1))
}

fn days(game: &mut Game, n: u32) -> Vec<String> {
    let mut keys = Vec::new();
    for _ in 0..n {
        let report = game.advance(RoundLength::Day, |_| {});
        keys.extend(report.messages.into_iter().map(|m| m.key));
    }
    keys
}

/// A works of `owner` in AAA with a furnace (ready after 20 days) smelting ore.
fn works(game: &mut Game, owner: CompanyId) -> SiteId {
    let c = game.catalog().clone();
    game.apply_as(
        owner,
        Command::FoundSite {
            country: c.countries.id("AAA").unwrap(),
            kind: SiteType::Factory,
        },
    )
    .unwrap();
    let site = SiteId(u32::try_from(game.state().sites.len() - 1).unwrap());
    game.apply_as(
        owner,
        Command::BuildFacility {
            site,
            facility: c.facilities.id("ofen").unwrap(),
            count: 1,
        },
    )
    .unwrap();
    game.apply_as(
        owner,
        Command::SetProduction {
            site,
            slot: 0,
            recipe: c.recipes.id("eisen_schmelzen"),
            utilization: 1.0,
        },
    )
    .unwrap();
    site
}

fn offer_site(seller: CompanyId, site: SiteId, price: f64) -> Command {
    Command::MakeOffer {
        seller,
        object: DealObject::Site(site),
        price: usd(price),
    }
}

fn answer(offer: u32, answer: OfferAnswer) -> Command {
    Command::AnswerOffer { offer, answer }
}

/// Inventory account equals the goods in the company's warehouses and on the way.
fn inventory_matches(game: &Game, company: CompanyId) -> bool {
    let state = game.state();
    let in_sites: Money = state
        .sites
        .iter()
        .filter(|s| s.owner == company)
        .flat_map(|s| s.inventory.values())
        .map(|s| s.value)
        .sum();
    let underway: Money = state
        .shipments
        .iter()
        .filter(|s| match s.to {
            Consignee::Site(site) => state.sites[site.index()].owner == company,
            Consignee::Importer(_) => false,
        })
        .map(|s| s.value)
        .sum();
    let books = state.companies[company.index()]
        .ledger
        .balance(Account::Inventory);
    (books - in_sites - underway).to_usd().abs() < 0.01
}

#[test]
fn a_site_changes_hands_with_its_books() {
    let (mut game, rival) = new_game(catalog(), false);
    let player = game.player();
    let site = works(&mut game, player);
    days(&mut game, 40);
    let value = site_value(game.state(), game.catalog(), site);
    assert!(value.fixed_assets > Money::ZERO);
    assert_eq!(value.under_construction, Money::ZERO);
    let price = value.book() + value.inventory + usd(1_000_000.0);
    let cash = |game: &Game, c: CompanyId| game.state().companies[c.index()].ledger.cash();
    let (player_cash, rival_cash) = (cash(&game, player), cash(&game, rival));
    game.apply_as(
        rival,
        Command::MakeOffer {
            seller: player,
            object: DealObject::Site(site),
            price,
        },
    )
    .unwrap();
    game.apply(answer(0, OfferAnswer::Accept)).unwrap();

    let state = game.state();
    let s = &state.sites[site.index()];
    assert_eq!(s.owner, rival);
    assert_eq!(s.acquired, Some(state.date));
    assert_eq!(state.offers[0].status, OfferStatus::Accepted);
    assert_eq!(cash(&game, player), player_cash + price);
    assert_eq!(cash(&game, rival), rival_cash - price);
    // The seller gains the price above the book values; the buyer books goodwill.
    let other = |c: CompanyId| {
        game.state().companies[c.index()]
            .ledger
            .month
            .by_type
            .get(&CostType::Other)
            .copied()
            .unwrap_or(Money::ZERO)
    };
    assert_eq!(other(player), usd(1_000_000.0));
    let goodwill = s.goodwill.expect("goodwill");
    assert_eq!(goodwill.amount, usd(1_000_000.0));
    fn books(game: &Game, c: CompanyId) -> &Ledger {
        &game.state().companies[c.index()].ledger
    }
    assert_eq!(
        books(&game, rival).balance(Account::Goodwill),
        usd(1_000_000.0)
    );
    for c in [player, rival] {
        assert!(books(&game, c).is_balanced());
        assert!(inventory_matches(&game, c));
    }

    // The goodwill is written off from now on.
    days(&mut game, 30);
    let left = books(&game, rival).balance(Account::Goodwill);
    assert!(left < usd(1_000_000.0) && left > usd(990_000.0), "{left:?}");
    assert!(books(&game, rival).is_balanced());
}

#[test]
fn offers_follow_the_rules() {
    let mut c = catalog();
    c.deal_model.min_age_months = 12;
    let (mut game, rival) = new_game(c, false);
    let player = game.player();
    let site = works(&mut game, player);
    // Too young, own object, no price, not enough cash.
    assert_eq!(
        game.apply_as(rival, offer_site(player, site, 1_000.0)),
        Err(CommandError::SiteTooYoung { months: 12 })
    );
    game.state_mut().sites[site.index()].founded = Date::new(1898, 1, 1).unwrap();
    assert_eq!(
        game.apply(offer_site(player, site, 1_000.0)),
        Err(CommandError::OwnObject)
    );
    assert_eq!(
        game.apply_as(rival, offer_site(player, site, 0.0)),
        Err(CommandError::InvalidPrice)
    );
    assert!(matches!(
        game.apply_as(rival, offer_site(player, site, 90_000_000.0)),
        Err(CommandError::NotEnoughCash { .. })
    ));
    assert_eq!(
        game.apply_as(rival, offer_site(rival, site, 1_000.0)),
        Err(CommandError::OwnObject)
    );

    // One open offer per buyer and object; only the seller answers.
    game.apply_as(rival, offer_site(player, site, 1_000.0))
        .unwrap();
    assert_eq!(
        game.apply_as(rival, offer_site(player, site, 2_000.0)),
        Err(CommandError::OfferExists)
    );
    assert_eq!(
        game.apply_as(rival, answer(0, OfferAnswer::Accept)),
        Err(CommandError::NotYourTurn)
    );
    assert_eq!(
        game.apply(answer(0, OfferAnswer::Counter { price: usd(500.0) })),
        Err(CommandError::NoCounter)
    );
    assert_eq!(
        game.apply(answer(7, OfferAnswer::Accept)),
        Err(CommandError::UnknownOffer)
    );
    // A counter-offer puts the buyer on turn; a second one is not possible.
    game.apply(answer(
        0,
        OfferAnswer::Counter {
            price: usd(3_000.0),
        },
    ))
    .unwrap();
    assert_eq!(
        game.apply(answer(0, OfferAnswer::Accept)),
        Err(CommandError::NotYourTurn)
    );
    assert_eq!(
        game.apply_as(
            rival,
            answer(
                0,
                OfferAnswer::Counter {
                    price: usd(4_000.0)
                }
            )
        ),
        Err(CommandError::NoCounter)
    );
    game.apply_as(rival, answer(0, OfferAnswer::Decline))
        .unwrap();
    assert_eq!(game.state().offers[0].status, OfferStatus::Declined);
    // After a decline the same buyer waits a year for the same object.
    let until = game.state().date.add_months(12);
    assert_eq!(
        game.apply_as(rival, offer_site(player, site, 9_000.0)),
        Err(CommandError::OfferBlocked { until })
    );
    // Withdrawing is for the company that named the price.
    let other = works(&mut game, player);
    game.state_mut().sites[other.index()].founded = Date::new(1898, 1, 1).unwrap();
    game.apply_as(rival, offer_site(player, other, 1_000.0))
        .unwrap();
    assert_eq!(
        game.apply(Command::WithdrawOffer { offer: 1 }),
        Err(CommandError::NotYourTurn)
    );
    game.apply_as(rival, Command::WithdrawOffer { offer: 1 })
        .unwrap();
    assert_eq!(game.state().offers[1].status, OfferStatus::Withdrawn);
}

#[test]
fn a_licence_after_a_counter_offer() {
    let (mut game, rival) = new_game(catalog(), false);
    let player = game.player();
    let c = game.catalog().clone();
    let t = c.technologies.id("hochofen_2000").unwrap();
    game.state_mut().companies[rival.index()]
        .technologies
        .insert(t);
    let licence = |price: f64| Command::MakeOffer {
        seller: rival,
        object: DealObject::License(t),
        price: usd(price),
    };
    // No licence on what the seller does not know.
    assert_eq!(
        game.apply_as(
            rival,
            Command::MakeOffer {
                seller: player,
                object: DealObject::License(t),
                price: usd(1.0),
            }
        ),
        Err(CommandError::LicenseNotPossible)
    );
    game.apply(licence(100_000.0)).unwrap();
    game.apply_as(
        rival,
        answer(
            0,
            OfferAnswer::Counter {
                price: usd(200_000.0),
            },
        ),
    )
    .unwrap();
    game.apply(answer(0, OfferAnswer::Accept)).unwrap();
    let state = game.state();
    assert!(state.knows(&c, player, t));
    assert!(state.knows(&c, rival, t));
    let licences = |id: CompanyId| {
        state.companies[id.index()]
            .ledger
            .month
            .by_type
            .get(&CostType::Licenses)
            .copied()
    };
    assert_eq!(licences(player), Some(usd(-200_000.0)));
    assert_eq!(licences(rival), Some(usd(200_000.0)));
    // A second licence for the same buyer is pointless now.
    assert_eq!(
        game.apply(licence(1.0)),
        Err(CommandError::LicenseNotPossible)
    );
}

#[test]
fn ai_companies_answer_and_offers_expire() {
    let mut c = catalog();
    // No offers of its own: only its answers are tested here.
    c.deal_model.ai.chance = Span::fixed(0.0);
    let (mut game, rival) = new_game(c, true);
    let player = game.player();
    let a = works(&mut game, rival);
    let b = works(&mut game, rival);
    let d = works(&mut game, rival);
    days(&mut game, 40);
    let base =
        |game: &Game, site: SiteId| site_value(game.state(), game.catalog(), site).base.to_usd();
    assert!(base(&game, a) > 0.0);

    // Far too little: declined the next day (the offer's day is simulated first).
    game.apply(offer_site(rival, a, base(&game, a) * 0.3))
        .unwrap();
    let news = days(&mut game, 2);
    assert!(
        news.contains(&keys::OFFER_DECLINED_SITE.to_owned()),
        "{news:?}"
    );

    // A little too little: the AI names its minimum, and the player takes it.
    let value_b = base(&game, b);
    game.apply(offer_site(rival, b, value_b)).unwrap();
    let news = days(&mut game, 2);
    assert!(
        news.contains(&keys::OFFER_COUNTER_SITE.to_owned()),
        "{news:?}"
    );
    let counter = game.state().offers.last().unwrap().clone();
    assert!(counter.counter && counter.price.to_usd() > value_b);
    game.apply(answer(counter.id, OfferAnswer::Accept)).unwrap();
    assert_eq!(game.state().sites[b.index()].owner, player);

    // Enough: sold the next day, with news for the player.
    game.apply(offer_site(rival, d, base(&game, d) * 3.0))
        .unwrap();
    let news = days(&mut game, 2);
    assert!(
        news.contains(&keys::OFFER_BOUGHT_SITE.to_owned()),
        "{news:?}"
    );
    assert_eq!(game.state().sites[d.index()].owner, player);

    // An offer nobody answers expires after the validity.
    game.apply_as(rival, offer_site(player, b, 1_000.0))
        .unwrap();
    let news = days(&mut game, 70);
    assert!(
        news.contains(&keys::OFFER_EXPIRED_SITE.to_owned()),
        "{news:?} {:?}",
        game.state().offers
    );
    for id in [player, rival] {
        assert!(game.state().companies[id.index()].ledger.is_balanced());
        assert!(inventory_matches(&game, id));
    }
}

#[test]
fn an_ai_company_offers_for_a_site_in_its_business() {
    let mut c = catalog();
    c.deal_model.ai.chance = Span::fixed(1.0);
    c.deal_model.ai.min_advantage = 0.0;
    c.deal_model.ai.min_price_usd = 0.0;
    let (mut game, rival) = new_game(c, true);
    let player = game.player();
    works(&mut game, rival);
    let site = works(&mut game, player);
    let news = days(&mut game, 32);
    assert!(
        news.contains(&keys::OFFER_RECEIVED_SITE.to_owned()),
        "{news:?}"
    );
    let offer = game
        .state()
        .offers
        .iter()
        .find(|o| o.buyer == rival && o.seller == player)
        .expect("an offer to the player")
        .clone();
    assert_eq!(offer.status, OfferStatus::Open);
    assert!(
        matches!(offer.object, DealObject::Site(s) if game.state().sites[s.index()].owner == player)
    );
    // Not more than one new offer to the player per month.
    let this_month = game
        .state()
        .offers
        .iter()
        .filter(|o| o.seller == player && o.date.first_of_month() == offer.date.first_of_month())
        .count();
    assert_eq!(this_month, 1);
    let _ = site;
}

#[test]
fn the_player_sees_offers_to_answer() {
    let (mut game, rival) = new_game(catalog(), false);
    let player = game.player();
    let site = works(&mut game, player);
    days(&mut game, 30);
    game.apply_as(rival, offer_site(player, site, 500_000.0))
        .unwrap();
    let overview = crate::views::overview(&game);
    let hint = overview
        .hints
        .iter()
        .find(|h| h.message.key == keys::HINT_OFFER_SITE)
        .expect("a hint to answer the offer");
    assert_eq!(hint.message.target.as_deref(), Some("wettbewerb"));
    let offers = crate::views::offers(&game).offers;
    assert_eq!(offers.len(), 1);
    let o = &offers[0];
    assert_eq!(o.role, "verkaeufer");
    assert_eq!(o.company, "Rivale AG");
    assert!(o.answer && o.can_counter && !o.can_withdraw);
    assert_eq!(o.object.kind, "standort");
    assert!(o.value.as_ref().is_some_and(|v| v.base_usd > 0.0));

    // After a counter-offer the rival answers; the player may withdraw it.
    game.apply(answer(
        o.id,
        OfferAnswer::Counter {
            price: usd(900_000.0),
        },
    ))
    .unwrap();
    let o = &crate::views::offers(&game).offers[0];
    assert!(!o.answer && !o.can_counter && o.can_withdraw);
    assert!(
        crate::views::overview(&game)
            .hints
            .iter()
            .all(|h| h.message.key != keys::HINT_OFFER_SITE)
    );
    // The company view lists the rival's would-be purchase from the player's side.
    let detail = crate::views::company_detail(&game, player.0).expect("own company");
    assert_eq!(detail.sites.len(), 1);
    assert_eq!(detail.sites[0].blocked, None);
    let rows = crate::views::companies(&game).companies;
    assert_eq!(rows.len(), 2);
}

/// A power plant of `owner` in AAA with `count` units (100 MWh a day each).
fn power_plant(game: &mut Game, owner: CompanyId, count: u32) -> SiteId {
    let c = game.catalog().clone();
    game.apply_as(
        owner,
        Command::FoundSite {
            country: c.countries.id("AAA").unwrap(),
            kind: SiteType::PowerPlant,
        },
    )
    .unwrap();
    let site = SiteId(u32::try_from(game.state().sites.len() - 1).unwrap());
    game.apply_as(
        owner,
        Command::BuildFacility {
            site,
            facility: c.facilities.id("kraftwerk").unwrap(),
            count,
        },
    )
    .unwrap();
    game.apply_as(
        owner,
        Command::SetProduction {
            site,
            slot: 0,
            recipe: c.recipes.id("strom_erzeugen"),
            utilization: 1.0,
        },
    )
    .unwrap();
    site
}

fn power_catalog() -> Catalog {
    let mut c = test_support::power();
    c.deal_model.min_age_months = 0;
    c
}

/// An AI company gives up a power plant its works need only for what a new one costs
/// (200 000 USD in `test_support::power`); without such works the usual minimum holds.
#[test]
fn an_ai_company_keeps_the_power_plant_its_works_need() {
    let mut c = power_catalog();
    c.deal_model.ai.chance = Span::fixed(0.0);
    let anew = 200_000.0;
    for needed in [true, false] {
        let (mut game, rival) = new_game(c.clone(), true);
        let player = game.player();
        if needed {
            // One furnace at full load: 50 runs at 2 MWh a day.
            works(&mut game, rival);
        }
        let plant = power_plant(&mut game, rival, 1);
        days(&mut game, 5);
        let base = site_value(game.state(), game.catalog(), plant)
            .base
            .to_usd();
        // Even as a core site the usual minimum stays below the offer.
        assert!(base * 1.2 * 1.5 < 0.95 * anew, "{base}");
        game.apply(offer_site(rival, plant, 0.95 * anew)).unwrap();
        let news = days(&mut game, 2);
        if needed {
            assert!(
                news.contains(&keys::OFFER_COUNTER_SITE.to_owned()),
                "{news:?}"
            );
            let counter = game.state().offers.last().unwrap();
            assert!(counter.counter);
            assert_eq!(counter.price, usd(anew));
            assert_eq!(game.state().sites[plant.index()].owner, rival);
        } else {
            assert!(
                news.contains(&keys::OFFER_BOUGHT_SITE.to_owned()),
                "{news:?}"
            );
            assert_eq!(game.state().sites[plant.index()].owner, player);
        }
    }
}

/// Electricity cannot be traded: an AI company bids for a power plant only where its
/// own plants do not cover its works.
#[test]
fn an_ai_company_bids_for_a_power_plant_only_where_it_lacks_power() {
    let mut c = power_catalog();
    c.deal_model.ai.chance = Span::fixed(1.0);
    c.deal_model.ai.min_price_usd = 0.0;
    for own_plants in [0, 2] {
        let (mut game, rival) = new_game(c.clone(), true);
        let player = game.player();
        works(&mut game, rival);
        if own_plants > 0 {
            power_plant(&mut game, rival, own_plants);
        }
        let plant = power_plant(&mut game, player, 1);
        days(&mut game, 32);
        let bid = game
            .state()
            .offers
            .iter()
            .any(|o| o.buyer == rival && o.object == DealObject::Site(plant));
        assert_eq!(bid, own_plants == 0, "own plants: {own_plants}");
    }
}

/// An AI company's only laboratory is not for sale below a new one while it researches
/// or wants to research (also between two research goals).
#[test]
fn an_ai_company_keeps_its_only_laboratory_while_it_researches() {
    let mut c = test_support::research();
    c.deal_model.min_age_months = 0;
    c.deal_model.ai.chance = Span::fixed(0.0);
    // Investment of the laboratory; research centres cost nothing as a site here.
    let anew = 100_000.0;
    for (researching, wants) in [(true, false), (false, true), (false, false)] {
        let mut c = c.clone();
        if wants {
            c.ai_model.behavior.research_min_revenue_usd = 0.0;
        }
        let (mut game, rival) = new_game(c, true);
        let player = game.player();
        let lab = {
            let c = game.catalog().clone();
            game.apply_as(
                rival,
                Command::FoundSite {
                    country: c.countries.id("AAA").unwrap(),
                    kind: SiteType::ResearchCenter,
                },
            )
            .unwrap();
            let site = SiteId(u32::try_from(game.state().sites.len() - 1).unwrap());
            game.apply_as(
                rival,
                Command::BuildFacility {
                    site,
                    facility: c.facilities.id("labor").unwrap(),
                    count: 1,
                },
            )
            .unwrap();
            if researching {
                game.apply_as(
                    rival,
                    Command::SetResearch {
                        site,
                        technology: c.technologies.id("turbine"),
                    },
                )
                .unwrap();
            }
            site
        };
        days(&mut game, 15);
        let base = site_value(game.state(), game.catalog(), lab).base.to_usd();
        assert!(base * 1.2 * 1.5 < 0.95 * anew, "{base}");
        game.apply(offer_site(rival, lab, 0.95 * anew)).unwrap();
        let news = days(&mut game, 2);
        if researching || wants {
            assert!(
                news.contains(&keys::OFFER_COUNTER_SITE.to_owned()),
                "{news:?}"
            );
            assert_eq!(game.state().offers.last().unwrap().price, usd(anew));
        } else {
            assert!(
                news.contains(&keys::OFFER_BOUGHT_SITE.to_owned()),
                "{news:?}"
            );
            assert_eq!(game.state().sites[lab.index()].owner, player);
        }
    }
}
