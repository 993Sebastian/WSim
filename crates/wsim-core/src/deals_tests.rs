//! Scenario tests for offers between companies (M30, areas M31) with the small chain of
//! `test_support::production`.

use std::sync::Arc;

use crate::calendar::{Date, RoundLength};
use crate::catalog::{Catalog, SiteType, Span, test_support};
use crate::command::{Command, CommandError};
use crate::deals::{
    DealObject, OfferAnswer, OfferStatus, area_value, best_deal, brand_value, site_value,
};
use crate::game::Game;
use crate::ledger::{Account, CostType, Ledger};
use crate::message::keys;
use crate::money::Money;
use crate::state::{
    Advertising, AiState, Brand, CompanyId, Consignee, GameSettings, PriceMode, SaleOffer, SiteId,
    StartForm,
};

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
        ventures: 1.0,
        tariff_dynamics: 1.0,
        event_effects: true,
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
        staff: 0.0,
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
            size: crate::catalog::FacilitySize::Medium,
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
            size: crate::catalog::FacilitySize::Medium,
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
                    size: crate::catalog::FacilitySize::Medium,
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

/// The rival's brand for ores in AAA, with an advertising budget.
fn brand(game: &mut Game, company: CompanyId, awareness: f64) {
    let c = game.catalog().clone();
    let (country, group) = (
        c.countries.id("AAA").unwrap(),
        c.goods_groups.id("erze").unwrap(),
    );
    let company = &mut game.state_mut().companies[company.index()];
    company.brands.push(Brand {
        country,
        group,
        awareness,
    });
    company.advertising.push(Advertising {
        country,
        group,
        budget: usd(1_000.0),
    });
}

/// M31: an area goes with all its sites, the price split by their base values, and the
/// brand; the seller keeps neither awareness nor advertising for the group.
#[test]
fn an_area_changes_hands_with_its_sites_and_brand() {
    let (mut game, rival) = new_game(catalog(), false);
    let player = game.player();
    let c = game.catalog().clone();
    let ores = c.goods_groups.id("erze").unwrap();
    let aaa = c.countries.id("AAA").unwrap();
    let a = works(&mut game, rival);
    let b = works(&mut game, rival);
    // A works without production belongs to no area.
    let idle = {
        game.apply_as(
            rival,
            Command::FoundSite {
                country: aaa,
                kind: SiteType::Factory,
            },
        )
        .unwrap();
        SiteId(u32::try_from(game.state().sites.len() - 1).unwrap())
    };
    brand(&mut game, rival, 0.5);
    days(&mut game, 40);

    let value = area_value(game.state(), game.catalog(), rival, ores);
    let sites: Vec<SiteId> = value.sites.iter().map(|&(s, _)| s).collect();
    assert_eq!(sites, vec![a, b]);
    // W: the advertising for the awareness the brand has now.
    let awareness = game.state().companies[rival.index()].awareness(aaa, ores);
    let reach = crate::brand::reach_usd(game.state(), game.catalog(), aaa);
    let expected = reach * -crate::math::ln(1.0 - awareness);
    assert!(awareness > 0.4, "{awareness}");
    assert!((value.brand.to_usd() - expected).abs() < 0.01, "{value:?}");
    assert_eq!(
        value.brand,
        brand_value(game.state(), game.catalog(), rival, ores)
    );
    let parts: Money = value.sites.iter().map(|(_, v)| v.base).sum();
    assert_eq!(value.base, parts + value.brand);

    let price = value.base.scale(1.2);
    let cash = |game: &Game, id: CompanyId| game.state().companies[id.index()].ledger.cash();
    let (player_cash, rival_cash) = (cash(&game, player), cash(&game, rival));
    game.apply(Command::MakeOffer {
        seller: rival,
        object: DealObject::Area(ores),
        price,
    })
    .unwrap();
    let id = game.state().offers.last().unwrap().id;
    game.apply_as(rival, answer(id, OfferAnswer::Accept))
        .unwrap();

    let state = game.state();
    assert_eq!(state.sites[a.index()].owner, player);
    assert_eq!(state.sites[b.index()].owner, player);
    assert_eq!(state.sites[idle.index()].owner, rival);
    assert_eq!(cash(&game, player), player_cash - price);
    assert_eq!(cash(&game, rival), rival_cash + price);
    assert_eq!(
        state.companies[player.index()].awareness(aaa, ores),
        awareness
    );
    assert_eq!(state.companies[rival.index()].awareness(aaa, ores), 0.0);
    assert!(state.companies[rival.index()].advertising.is_empty());
    for company in [player, rival] {
        assert!(state.companies[company.index()].ledger.is_balanced());
        assert!(inventory_matches(&game, company));
    }
    // Nothing is left of the area.
    assert_eq!(
        game.apply(Command::MakeOffer {
            seller: rival,
            object: DealObject::Area(ores),
            price,
        }),
        Err(CommandError::NotSellersObject)
    );
}

/// An AI company asks the core markup for an area with all its works.
#[test]
fn an_ai_company_asks_more_for_its_whole_production() {
    let mut c = catalog();
    c.deal_model.ai.chance = Span::fixed(0.0);
    for whole in [true, false] {
        let (mut game, rival) = new_game(c.clone(), true);
        let ores = game.catalog().goods_groups.id("erze").unwrap();
        let aaa = game.catalog().countries.id("AAA").unwrap();
        works(&mut game, rival);
        if !whole {
            // A second works with a furnace that makes nothing yet.
            game.apply_as(
                rival,
                Command::FoundSite {
                    country: aaa,
                    kind: SiteType::Factory,
                },
            )
            .unwrap();
            let site = SiteId(u32::try_from(game.state().sites.len() - 1).unwrap());
            let furnace = game.catalog().facilities.id("ofen").unwrap();
            game.apply_as(
                rival,
                Command::BuildFacility {
                    site,
                    facility: furnace,
                    count: 1,
                    size: crate::catalog::FacilitySize::Medium,
                },
            )
            .unwrap();
        }
        days(&mut game, 40);
        let value = area_value(game.state(), game.catalog(), rival, ores);
        // Above the usual minimum (+20 %), below the one for the core (+80 %).
        game.apply(Command::MakeOffer {
            seller: rival,
            object: DealObject::Area(ores),
            price: value.base.scale(1.5),
        })
        .unwrap();
        let news = days(&mut game, 2);
        if whole {
            assert!(
                news.contains(&keys::OFFER_COUNTER_AREA.to_owned()),
                "{news:?}"
            );
            // The minimum of the next day, when the plants have aged a day.
            let counter = game.state().offers.last().unwrap().price.to_usd();
            let core = value.base.to_usd() * 1.2 * 1.5;
            assert!((counter / core - 1.0).abs() < 1e-3, "{counter} vs {core}");
        } else {
            assert!(
                news.contains(&keys::OFFER_BOUGHT_AREA.to_owned()),
                "{news:?}"
            );
        }
    }
}

/// An AI company bids for the area of a competitor in its own market: the shares of
/// all its sites add up, which a single site does not offer.
#[test]
fn an_ai_company_bids_for_a_competitors_area() {
    let mut c = catalog();
    c.deal_model.ai.min_price_usd = 0.0;
    let (mut game, rival) = new_game(c, true);
    let player = game.player();
    let iron = game.catalog().products.id("eisen").unwrap();
    let aaa = game.catalog().countries.id("AAA").unwrap();
    let ores = game.catalog().goods_groups.id("erze").unwrap();
    let mine = works(&mut game, rival);
    let a = works(&mut game, player);
    let b = works(&mut game, player);
    days(&mut game, 40);
    // Last month each sold a quarter of the market.
    let state = game.state_mut();
    state.markets.get_mut(iron).get_mut(aaa).last_month.sold = 400.0;
    for site in [mine, a, b] {
        state.sites[site.index()].offers.insert(
            iron,
            SaleOffer {
                mode: PriceMode::Market {
                    markup: 0.0,
                    floor: Money::ZERO,
                },
                price: usd(100.0),
                keep: 0.0,
                sold_today: 0.0,
                sold_month: 0.0,
                sold_last_month: 100.0,
                to_traders_month: 0.0,
                to_companies_month: 0.0,
            },
        );
    }
    let deal = best_deal(game.state(), game.catalog(), rival).expect("a deal");
    assert_eq!((deal.0, deal.1), (DealObject::Area(ores), player));
}

/// The company view lists areas with their sites, brand and value; offers and hints
/// name the goods group.
#[test]
fn the_player_sees_areas_and_offers_for_them() {
    let (mut game, rival) = new_game(catalog(), false);
    let player = game.player();
    let ores = game.catalog().goods_groups.id("erze").unwrap();
    let a = works(&mut game, player);
    let b = works(&mut game, player);
    brand(&mut game, player, 0.3);
    days(&mut game, 40);
    let detail = crate::views::company_detail(&game, player.0).expect("own company");
    assert_eq!(detail.areas.len(), 1);
    let area = &detail.areas[0];
    assert_eq!(area.group, "erze");
    assert_eq!(area.sites, vec![a.0, b.0]);
    assert_eq!(area.brand.len(), 1);
    assert_eq!(area.brand[0].0, "AAA");
    assert!(area.value.brand_usd > 0.0);
    assert!(area.new_build_usd > area.value.brand_usd);
    assert_eq!(area.blocked, None);

    game.apply_as(
        rival,
        Command::MakeOffer {
            seller: player,
            object: DealObject::Area(ores),
            price: usd(5_000_000.0),
        },
    )
    .unwrap();
    let offers = crate::views::offers(&game).offers;
    let o = &offers[0];
    assert_eq!(o.object.kind, "bereich");
    assert_eq!(o.object.group.as_deref(), Some("erze"));
    assert_eq!(o.object.site_count, 2);
    assert_eq!(o.object.products, vec!["eisen".to_owned()]);
    assert!(o.value.as_ref().is_some_and(|v| v.brand_usd > 0.0));
    let hint = crate::views::overview(&game)
        .hints
        .into_iter()
        .find(|h| h.message.key == keys::HINT_OFFER_AREA)
        .expect("a hint to answer");
    assert!(hint.message.params.iter().any(|(k, _)| k == "warengruppe"));

    // The player's own areas carry no state for offers of its own.
    let own = crate::views::company_detail(&game, player.0).unwrap();
    assert_eq!(
        (own.areas[0].blocked.clone(), own.areas[0].open_offer),
        (None, None)
    );
}

/// The game of `new_game` with auctions (M38) and a third company, "Pleite AG" (AI),
/// that owns a works and has just become insolvent. The rival smelts iron as well, so
/// the works is in its business (M30: it bids only for what it would buy anyway).
fn insolvency(rival_cash: f64) -> (Game, CompanyId, CompanyId, SiteId) {
    let mut c = catalog();
    c.deal_model.insolvency_days = 30;
    let (mut game, rival) = new_game(c, true);
    works(&mut game, rival);
    let state = game.state_mut();
    state.companies[rival.index()].ledger = Ledger::new(state.date, usd(rival_cash));
    let mut broke = state.companies[rival.index()].clone();
    "Pleite AG".clone_into(&mut broke.name);
    broke.ledger = Ledger::new(state.date, usd(10_000_000.0));
    state.companies.push(broke);
    let broke = CompanyId(2);
    let site = works(&mut game, broke);
    days(&mut game, 25);
    let state = game.state_mut();
    state.companies[broke.index()].ledger.expense(
        CostType::Other,
        crate::ledger::CostCenter::default(),
        Account::Cash,
        usd(1e12),
    );
    let catalog = game.catalog().clone();
    let messages = crate::finance::check_insolvency(game.state_mut(), &catalog);
    assert!(
        messages
            .iter()
            .any(|m| m.key == keys::COMPANY_INSOLVENT_AUCTION),
        "{messages:?}"
    );
    (game, rival, broke, site)
}

#[test]
fn an_insolvent_site_goes_to_the_highest_bid_at_the_second_price() {
    let (mut game, rival, broke, site) = insolvency(50_000_000.0);
    let state = game.state();
    assert!(crate::deals::in_auction(state, broke));
    let minimum = crate::deals::auction_minimum(state, game.catalog(), site);
    assert!(minimum > Money::ZERO);
    // The site waits: no staff, no production.
    assert!(state.sites[site.index()].workforce.values().sum::<f64>() < 1e-9);
    // Bids start at the minimum.
    let low = Command::MakeOffer {
        seller: broke,
        object: DealObject::Site(site),
        price: minimum.scale(0.5),
    };
    assert_eq!(
        game.apply(low),
        Err(CommandError::BelowMinimumBid { minimum })
    );
    // Licences and areas of an insolvent company are not for sale.
    use crate::ids::Id;
    let licence = Command::MakeOffer {
        seller: broke,
        object: DealObject::License(crate::ids::TechnologyId::from_index(0)),
        price: minimum,
    };
    assert_eq!(game.apply(licence), Err(CommandError::SellerBankrupt));
    let before = game.state().companies[0].ledger.cash();
    let bid = minimum.scale(10.0);
    game.apply(Command::MakeOffer {
        seller: broke,
        object: DealObject::Site(site),
        price: bid,
    })
    .unwrap();
    let keys = days(&mut game, 32);
    assert!(keys.iter().any(|k| k == keys::AUCTION_WON), "{keys:?}");
    let state = game.state();
    assert_eq!(state.sites[site.index()].owner, game.player());
    assert!(!crate::deals::in_auction(state, broke));
    // The rival bid less: the player pays its price (the second bid), not the own one.
    let paid = before - state.companies[0].ledger.cash();
    assert!(
        paid >= minimum && paid < bid,
        "{paid:?} {minimum:?} {bid:?}"
    );
    let offer = state
        .offers
        .iter()
        .find(|o| o.seller == broke)
        .expect("kept while it blocks");
    assert_eq!(offer.status, OfferStatus::Accepted);
    assert!(state.companies[rival.index()].ledger.is_balanced());
    assert!(state.companies[0].ledger.is_balanced());
    assert!(state.companies[broke.index()].ledger.is_balanced());
}

#[test]
fn an_ai_company_takes_over_an_insolvent_site_at_the_minimum() {
    let (mut game, rival, broke, site) = insolvency(50_000_000.0);
    let minimum = crate::deals::auction_minimum(game.state(), game.catalog(), site);
    let cash = game.state().companies[rival.index()].ledger.cash();
    days(&mut game, 32);
    let state = game.state();
    assert_eq!(state.sites[site.index()].owner, rival);
    // The only bidder pays the minimum of the last day.
    let paid = cash - state.companies[rival.index()].ledger.cash();
    assert!(
        (paid.to_usd() - minimum.to_usd()).abs() < 0.2 * minimum.to_usd(),
        "{paid:?} {minimum:?}"
    );
    let _ = broke;
}

#[test]
fn a_site_nobody_wants_is_given_up_after_the_auction() {
    let (mut game, _, broke, site) = insolvency(0.0);
    days(&mut game, 20);
    // Still the insolvent company's while the auction runs.
    assert_eq!(game.state().sites[site.index()].owner, broke);
    assert!(crate::deals::in_auction(game.state(), broke));
    days(&mut game, 12);
    let state = game.state();
    assert_eq!(state.sites[site.index()].owner, broke);
    assert!(!crate::deals::in_auction(state, broke));
    assert_eq!(state.companies[broke.index()].auction_until, None);
}
