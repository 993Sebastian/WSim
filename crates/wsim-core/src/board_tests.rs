//! Scenario tests for the board, its own topics and the mandate (MA5) with
//! `test_support::management`.

use std::sync::Arc;

use crate::calendar::{Date, RoundLength};
use crate::catalog::{SiteType, Span, test_support};
use crate::command::{Command, CommandError};
use crate::deals::{DealObject, OfferAnswer, OfferStatus};
use crate::decision::{Choice, ChoiceKind, Decision, Topic, Verdict};
use crate::game::Game;
use crate::ledger::{Account, CostType, Ledger};
use crate::management::{self, ConcernAnswer};
use crate::management_tests::{
    cheap_furnaces, days, free, head, hire_sharp, mine_and_works, new_game, through_quarter_end,
    usd, weak_works,
};
use crate::mandate::{Goals, Guideline, Mandate};
use crate::message::keys;
use crate::money::Money;
use crate::save;
use crate::state::{
    AiState, CompanyId, ConcernReason, ConcernStatus, Position, Role, SiteId, Unit,
};

fn ceo() -> Position {
    Position::new(Unit::Board, Role::Head)
}

fn member(function: &str) -> Position {
    Position::new(Unit::Board, Role::Specialist(function.into()))
}

/// Message keys of the days.
fn news(game: &mut Game, n: u32) -> Vec<String> {
    let mut out = Vec::new();
    for _ in 0..n {
        let report = game.advance(RoundLength::Day, |_| {});
        out.extend(report.messages.into_iter().map(|m| m.key));
    }
    out
}

#[test]
fn the_board_heads_every_chain() {
    let mut game = new_game(test_support::management());
    let player = game.player();
    let (_, works) = mine_and_works(&mut game);
    let (c, state) = (game.catalog().clone(), game.state());
    assert!(management::units(&c, state, player).contains(&Unit::Board));
    let chain = management::chain(&c, state, player, Unit::Site(works), Topic::Expansion);
    assert_eq!(chain[chain.len() - 2..], [member("produktion"), ceo()]);
    assert_eq!(
        management::chain(&c, state, player, Unit::Board, Topic::Cash),
        vec![member("finanzen"), ceo()]
    );
    assert_eq!(
        management::seat_country(&c, state, player, Unit::Board),
        Some(state.companies[0].headquarters)
    );
    // The CEO caps the members of the board and the heads of the continents.
    let boss = hire_sharp(&mut game, ceo());
    let treasurer = hire_sharp(&mut game, member("finanzen"));
    let state = game.state();
    assert_eq!(
        management::superior(&c, state, player, &member("finanzen")),
        Some((ceo(), boss))
    );
    let k = c.countries.get(state.companies[0].headquarters).continent;
    assert_eq!(
        management::superior(
            &c,
            state,
            player,
            &Position::new(Unit::Continent(k), Role::Head)
        ),
        Some((ceo(), boss))
    );
    let salary = |id| state.managers[&id].job.as_ref().unwrap().salary;
    assert!(
        salary(boss) > salary(treasurer),
        "15 against 10 yearly wages"
    );
    let own = management::budget(&c, state, player, &member("finanzen"), salary(treasurer));
    let cap = management::budget(&c, state, player, &ceo(), salary(boss));
    assert!(own.0 <= cap.0 && own.1 <= cap.1);
    // Their salaries are overhead of the company.
    days(&mut game, 40);
    let overhead = crate::ledger::CostCenter {
        site: None,
        product: None,
    };
    let ledger = &game.state().companies[0].ledger;
    assert!(ledger.months.iter().any(|m| {
        m.by_center
            .get(&overhead)
            .and_then(|c| c.get(&CostType::Personnel))
            .is_some_and(|v| *v < Money::ZERO)
    }));
    assert!(ledger.is_balanced());
}

#[test]
fn a_concern_climbs_to_the_ceo() {
    // The head of the works always asks; the CEO decides within the budget.
    let (mut game, works, _) = weak_works(cheap_furnaces());
    game.apply(Command::SetBudget {
        position: head(works),
        shares: Some((0.0, 0.0)),
    })
    .unwrap();
    hire_sharp(&mut game, ceo());
    through_quarter_end(&mut game);
    let mothballed: u32 = game.state().sites[works.index()]
        .slots
        .iter()
        .filter(|sl| sl.mothballed())
        .map(|sl| sl.count)
        .sum();
    assert!(mothballed > 0, "the CEO decided");
    assert!(game.state().concerns.is_empty());
    let ps = management::position_state(game.state(), game.player(), &ceo()).unwrap();
    assert!(ps.spent > Money::ZERO, "on the CEO's budget");
}

/// A loan of 1,000 USD the board's rules would take.
fn cash_decision(company: CompanyId) -> Decision {
    Decision::new(
        Topic::Cash,
        company,
        Choice::one(
            ChoiceKind::Borrow,
            Command::TakeLoan {
                amount: usd(1_000.0),
                years: 5,
            },
        ),
    )
}

#[test]
fn loans_are_for_the_finance_member_and_the_ceo() {
    let mut game = new_game(cheap_furnaces());
    let player = game.player();
    let (_, works) = mine_and_works(&mut game);
    let cash = cash_decision(player);
    // Without a board nobody takes up the cash: nothing happens, nobody asks.
    let (verdict, concerns, _) = management::decide_now(game.state(), game.catalog(), &cash);
    assert_eq!(verdict, Verdict::Hold);
    assert!(concerns.is_empty());
    assert_eq!(management::place_of(game.state(), &cash), Some(Unit::Board));
    // The finance member borrows within its budget.
    hire_sharp(&mut game, member("finanzen"));
    let (verdict, _, _) = management::decide_now(game.state(), game.catalog(), &cash);
    assert_eq!(verdict, Verdict::Rule);

    // Building with a loan at the works: its head may not borrow.
    hire_sharp(&mut game, head(works));
    let c = game.catalog().clone();
    // Building is the only option, so that every position recommends it.
    let mut build = Decision::new(
        Topic::Expansion,
        player,
        Choice {
            kind: ChoiceKind::Build,
            steps: Vec::new(),
        }
        .then(
            Command::TakeLoan {
                amount: usd(1_000.0),
                years: 5,
            },
            false,
        )
        .then(
            Command::BuildFacility {
                site: works,
                facility: c.facilities.id("ofen").unwrap(),
                count: 1,
                size: crate::catalog::FacilitySize::Medium,
            },
            true,
        ),
    )
    .at(works);
    build.choices.truncate(1);
    let (verdict, concerns, _) = management::decide_now(game.state(), game.catalog(), &build);
    assert_eq!(verdict, Verdict::Hold);
    assert_eq!(concerns[0].reason, ConcernReason::Finance);
    // The CEO may.
    hire_sharp(&mut game, ceo());
    let (verdict, concerns, _) = management::decide_now(game.state(), game.catalog(), &build);
    assert_eq!(verdict, Verdict::Rule, "{concerns:?}");
    // Not beyond the debt the mandate allows: the CEO asks.
    game.apply(Command::SetMandate {
        mandate: Mandate {
            max_debt: Some(0.0),
            ..Mandate::default()
        },
    })
    .unwrap();
    let (verdict, concerns, _) = management::decide_now(game.state(), game.catalog(), &build);
    assert_eq!(verdict, Verdict::Hold);
    assert_eq!(concerns[0].reason, ConcernReason::Debt);
    assert_eq!(management::asker(&concerns[0]), &ceo());
    let view = crate::views::concerns(&{
        let mut g = game.clone();
        g.state_mut().concerns = concerns;
        g
    });
    let shown = &view.open[0].concerns[0];
    assert_eq!(shown.reason, "verschuldung");
    assert_eq!(shown.strategy_limit_usd, Some(0.0));
    assert_eq!(shown.asker.level, "vorstand");
}

#[test]
fn the_finance_member_borrows_at_the_check() {
    let mut c = test_support::management();
    // Always short of cash; budgets far above the loans.
    c.ai_model.behavior.cash_min_months = 1e6;
    c.ai_model.behavior.cash_max_months = 2e6;
    c.management.budget_floor = (1_000.0, 3_000.0);
    let mut game = new_game(c);
    mine_and_works(&mut game);
    hire_sharp(&mut game, member("finanzen"));
    days(&mut game, 31);
    let state = game.state();
    assert!(!state.companies[0].loans.is_empty(), "the member borrowed");
    assert!(state.concerns.is_empty());
    let ps = management::position_state(state, game.player(), &member("finanzen")).unwrap();
    assert!(ps.spent > Money::ZERO);
    assert!(state.companies[0].ledger.is_balanced());
}

/// The player with a mine, a works and a finance member who may sell them, and a rival
/// company (not AI) with 50 million in cash.
fn seller_game(floor: f64) -> (Game, CompanyId, SiteId, SiteId) {
    let mut c = test_support::management();
    c.deal_model.min_age_months = 0;
    c.management.budget_floor = (floor, floor * 3.0);
    let mut game = new_game(c);
    let (mine, works) = mine_and_works(&mut game);
    let state = game.state_mut();
    let mut rival = state.companies[0].clone();
    "Rivale AG".clone_into(&mut rival.name);
    rival.ledger = Ledger::new(state.date, usd(50_000_000.0));
    rival.positions.clear();
    state.companies.push(rival);
    hire_sharp(&mut game, member("finanzen"));
    days(&mut game, 25);
    (game, CompanyId(1), mine, works)
}

fn offer_for(game: &mut Game, rival: CompanyId, site: SiteId, share: f64) -> u32 {
    let value = crate::deals::site_value(game.state(), game.catalog(), site).base;
    let player = game.player();
    game.apply_as(
        rival,
        Command::MakeOffer {
            seller: player,
            object: DealObject::Site(site),
            price: value.scale(share),
        },
    )
    .unwrap();
    game.state().offers.last().unwrap().id
}

fn status(game: &Game, id: u32) -> OfferStatus {
    game.state()
        .offers
        .iter()
        .find(|o| o.id == id)
        .unwrap()
        .status
}

#[test]
fn the_board_answers_offers_like_the_ai() {
    let (mut game, rival, mine, works) = seller_game(1_000.0);
    // Far too little: declined at the check.
    let low = offer_for(&mut game, rival, mine, 0.3);
    let keys = news(&mut game, 31);
    assert_eq!(status(&game, low), OfferStatus::Declined);
    assert!(
        keys.iter().any(|k| k == keys::BOARD_DECLINED_SITE),
        "{keys:?}"
    );
    // Enough: sold.
    let high = offer_for(&mut game, rival, works, 3.0);
    let keys = news(&mut game, 31);
    assert_eq!(status(&game, high), OfferStatus::Accepted);
    assert!(keys.iter().any(|k| k == keys::BOARD_SOLD_SITE), "{keys:?}");
    assert_eq!(game.state().sites[works.index()].owner, rival);
    for c in &game.state().companies {
        assert!(c.ledger.is_balanced());
    }
}

#[test]
fn an_answer_beyond_the_budget_comes_as_a_concern() {
    let (mut game, rival, _, works) = seller_game(1.0);
    game.apply(Command::SetBudget {
        position: member("finanzen"),
        shares: Some((0.0, 0.0)),
    })
    .unwrap();
    let id = offer_for(&mut game, rival, works, 3.0);
    days(&mut game, 31);
    assert_eq!(
        status(&game, id),
        OfferStatus::Open,
        "not the member's to sell"
    );
    let concern = game.state().concerns[0].clone();
    assert_eq!(concern.decision.topic, Topic::OfferAnswer);
    assert_eq!(concern.reason, ConcernReason::Always);
    let view = crate::views::concerns(&game);
    let shown = &view.open[0].concerns[0];
    assert_eq!(shown.options[0].kind, "annehmen");
    assert_eq!(shown.options[0].steps[0].key, keys::STEP_ACCEPT_SITE);
    assert!(
        shown.options[0].amount_usd > 0.0,
        "the book value of the works"
    );
    // The player follows the recommendation: sold.
    game.apply(Command::AnswerConcern {
        concern: concern.id,
        answer: ConcernAnswer::Delegate,
    })
    .unwrap();
    assert_eq!(status(&game, id), OfferStatus::Accepted);

    // A concern about an offer that closed meanwhile is settled.
    let (mut game, rival, _, works) = seller_game(1.0);
    game.apply(Command::SetBudget {
        position: member("finanzen"),
        shares: Some((0.0, 0.0)),
    })
    .unwrap();
    let id = offer_for(&mut game, rival, works, 3.0);
    days(&mut game, 31);
    assert_eq!(game.state().concerns[0].status, ConcernStatus::Open);
    game.apply_as(rival, Command::WithdrawOffer { offer: id })
        .unwrap();
    days(&mut game, 1);
    assert_eq!(game.state().concerns[0].status, ConcernStatus::Settled);
    // Answering it directly settles it too.
    let id = offer_for(&mut game, rival, works, 3.0);
    let next = game.state().next_concern;
    days(&mut game, 31);
    let open = game
        .state()
        .concerns
        .iter()
        .find(|c| c.id >= next)
        .unwrap()
        .clone();
    assert_eq!(open.status, ConcernStatus::Open);
    game.apply(Command::AnswerOffer {
        offer: id,
        answer: OfferAnswer::Decline,
    })
    .unwrap();
    let after = game
        .state()
        .concerns
        .iter()
        .find(|c| c.id == open.id)
        .unwrap();
    assert_eq!(after.status, ConcernStatus::Settled);
}

/// The player and a rival (AI, not acting on its own) both smelting iron in AAA: the
/// rival's works is in the player's business.
fn buyer_game(floor: f64) -> (Game, CompanyId, SiteId) {
    let mut c = test_support::management();
    c.deal_model.min_age_months = 0;
    c.deal_model.ai.min_advantage = 0.0;
    c.deal_model.ai.min_price_usd = 0.0;
    c.deal_model.ai.chance = Span::fixed(0.0);
    c.deal_model.insolvency_days = 30;
    c.management.budget_floor = (floor, floor * 3.0);
    let mut game = new_game(c);
    let (_, works) = mine_and_works(&mut game);
    let ore = game.catalog().products.id("erz").unwrap();
    let _ = ore;
    let c = game.catalog().clone();
    game.apply(Command::SetProduction {
        site: works,
        slot: 0,
        recipe: c.recipes.id("eisen_schmelzen"),
        utilization: 1.0,
    })
    .unwrap();
    let state = game.state_mut();
    let mut rival = state.companies[0].clone();
    "Rivale AG".clone_into(&mut rival.name);
    rival.ledger = Ledger::new(state.date, usd(50_000_000.0));
    rival.positions.clear();
    rival.ai = Some(AiState {
        competence: 0.5,
        aggressiveness: 0.5,
        real: None,
        next_operations: Date::new(2100, 1, 1).unwrap(),
    });
    state.companies.push(rival);
    let rival = CompanyId(1);
    let aaa = c.countries.id("AAA").unwrap();
    game.apply_as(
        rival,
        Command::FoundSite {
            country: aaa,
            kind: SiteType::Factory,
        },
    )
    .unwrap();
    let site = SiteId(u32::try_from(game.state().sites.len() - 1).unwrap());
    game.apply_as(
        rival,
        Command::BuildFacility {
            site,
            facility: c.facilities.id("ofen").unwrap(),
            count: 1,
            size: crate::catalog::FacilitySize::Medium,
        },
    )
    .unwrap();
    game.apply_as(
        rival,
        Command::SetProduction {
            site,
            slot: 0,
            recipe: c.recipes.id("eisen_schmelzen"),
            utilization: 1.0,
        },
    )
    .unwrap();
    days(&mut game, 25);
    hire_sharp(&mut game, member("finanzen"));
    (game, rival, site)
}

#[test]
fn the_board_offers_for_what_its_business_needs() {
    let (mut game, rival, site) = buyer_game(1_000.0);
    let player = game.player();
    let keys = news(&mut game, 31);
    let offer = game
        .state()
        .offers
        .iter()
        .find(|o| o.buyer == player)
        .expect("an offer of the board")
        .clone();
    assert_eq!(offer.seller, rival);
    assert_eq!(offer.object, DealObject::Site(site));
    assert!(keys.iter().any(|k| k == keys::BOARD_OFFER_SITE), "{keys:?}");

    // Beyond the budget: a concern to the player instead.
    let (mut game, _, _) = buyer_game(0.0001);
    days(&mut game, 31);
    assert!(!game.state().offers.iter().any(|o| o.buyer == player));
    let concern = &game.state().concerns[0];
    assert_eq!(concern.decision.topic, Topic::Offer);
    let view = crate::views::concerns(&game);
    assert_eq!(
        view.open[0].concerns[0].options[0].steps[0].key,
        keys::STEP_BID_SITE
    );

    // Not in a country the mandate blocks.
    let (mut game, _, _) = buyer_game(1_000.0);
    let aaa = game.catalog().countries.id("AAA").unwrap();
    game.apply(Command::SetMandate {
        mandate: Mandate {
            blocked_countries: [aaa].into(),
            ..Mandate::default()
        },
    })
    .unwrap();
    days(&mut game, 31);
    assert!(!game.state().offers.iter().any(|o| o.buyer == player));
    assert!(game.state().concerns.is_empty());
}

#[test]
fn the_board_bids_in_an_auction() {
    let (mut game, rival, site) = buyer_game(1_000.0);
    let player = game.player();
    // The rival goes bankrupt; its works is auctioned.
    game.state_mut().companies[rival.index()].ledger.expense(
        CostType::Other,
        crate::ledger::CostCenter::default(),
        Account::Cash,
        usd(1e12),
    );
    let catalog = game.catalog().clone();
    crate::finance::check_insolvency(game.state_mut(), &catalog);
    assert!(crate::deals::in_auction(game.state(), rival));
    days(&mut game, 31);
    let bid = game
        .state()
        .offers
        .iter()
        .find(|o| o.buyer == player && o.seller == rival);
    let won = game.state().sites[site.index()].owner == player;
    assert!(bid.is_some() || won, "{:?}", game.state().offers);
}

#[test]
fn the_mandate_is_checked_and_steers_the_rules() {
    let mut game = new_game(test_support::management());
    let player = game.player();
    let invalid = [
        Mandate {
            goals: Goals {
                growth: Some(f64::NAN),
                ..Goals::default()
            },
            ..Mandate::default()
        },
        Mandate {
            goals: Goals {
                margin: Some(2.0),
                ..Goals::default()
            },
            ..Mandate::default()
        },
        Mandate {
            goals: Goals {
                rank: Some(0),
                ..Goals::default()
            },
            ..Mandate::default()
        },
        Mandate {
            max_debt: Some(1.5),
            ..Mandate::default()
        },
    ];
    for mandate in invalid {
        assert_eq!(
            game.apply(Command::SetMandate { mandate }),
            Err(CommandError::InvalidMandate)
        );
    }
    let aggressiveness =
        |game: &Game| crate::ai::traits(game.catalog(), game.state(), game.player()).1;
    assert_eq!(aggressiveness(&game), 0.5, "profit by default");
    for (guideline, expected) in [(Guideline::Growth, 0.8), (Guideline::Safety, 0.2)] {
        game.apply(Command::SetMandate {
            mandate: Mandate {
                guideline,
                ..Mandate::default()
            },
        })
        .unwrap();
        assert_eq!(aggressiveness(&game), expected);
    }
    // Founding in a blocked country stays undone, without asking.
    let aaa = game.catalog().countries.id("AAA").unwrap();
    mine_and_works(&mut game);
    hire_sharp(&mut game, ceo());
    game.apply(Command::SetMandate {
        mandate: Mandate {
            blocked_countries: [aaa].into(),
            ..Mandate::default()
        },
    })
    .unwrap();
    let found = Decision::new(
        Topic::Expansion,
        player,
        Choice::one(
            ChoiceKind::Build,
            Command::FoundSite {
                country: aaa,
                kind: SiteType::Factory,
            },
        ),
    );
    let (verdict, concerns, _) = management::decide_now(game.state(), game.catalog(), &found);
    assert_eq!(verdict, Verdict::Hold);
    assert!(concerns.is_empty());
    // AI companies keep their character.
    let ai = game.state().companies.iter().position(|c| c.ai.is_some());
    if let Some(i) = ai {
        let a = game.state().companies[i].ai.clone().unwrap();
        let id = CompanyId(u32::try_from(i).unwrap());
        assert_eq!(
            crate::ai::traits(game.catalog(), game.state(), id),
            (a.competence, a.aggressiveness)
        );
    }
}

#[test]
fn the_personnel_member_sees_more_sharply() {
    let mut game = new_game(test_support::management());
    mine_and_works(&mut game);
    let player = game.player();
    assert_eq!(
        management::impression_share(game.catalog(), game.state(), player),
        1.0
    );
    let id = hire_sharp(&mut game, member("personal"));
    let share = management::impression_share(game.catalog(), game.state(), player);
    assert!((share - 0.2).abs() < 1e-9, "{share}");
    assert_eq!(management::shown_impression(15, share), 3);
    assert_eq!(management::shown_impression(-15, share), -3);
    // Without the member the impression is whole again.
    game.apply(Command::DismissManager { manager: id }).unwrap();
    assert_eq!(
        management::impression_share(game.catalog(), game.state(), player),
        1.0
    );
}

#[test]
fn boards_and_mandates_replay_and_load_identically() {
    let mut c = test_support::management();
    c.ai_model.behavior.cash_min_months = 1e6;
    c.ai_model.behavior.cash_max_months = 2e6;
    c.management.budget_floor = (1_000.0, 3_000.0);
    let catalog = Arc::new(c);
    let setup = |game: &mut Game| {
        mine_and_works(game);
        let ids = free(game);
        for (id, position) in ids.into_iter().zip([ceo(), member("finanzen")]) {
            game.apply(Command::HireManager {
                manager: id,
                position,
            })
            .unwrap();
        }
        game.apply(Command::SetMandate {
            mandate: Mandate {
                guideline: Guideline::Growth,
                max_debt: Some(0.9),
                goals: Goals {
                    growth: Some(0.1),
                    rank: Some(3),
                    ..Goals::default()
                },
                ..Mandate::default()
            },
        })
        .unwrap();
    };
    let play = |game: &mut Game| {
        for _ in 0..5 {
            game.advance(RoundLength::Week, |_| {});
        }
    };
    let mut a = new_game((*catalog).clone());
    setup(&mut a);
    play(&mut a);
    play(&mut a);
    assert!(
        !a.state().companies[0].loans.is_empty(),
        "the board borrowed"
    );

    let mut b = new_game((*catalog).clone());
    setup(&mut b);
    play(&mut b);
    let mut loaded = save::decode(&save::encode(&b), catalog.clone())
        .unwrap()
        .game;
    assert_eq!(
        loaded.state().companies[0].mandate,
        b.state().companies[0].mandate
    );
    play(&mut loaded);
    assert_eq!(loaded.state_hash(), a.state_hash());

    let replayed = Game::replay(catalog, a.state().settings.clone(), a.journal()).unwrap();
    assert_eq!(replayed.state_hash(), a.state_hash());
}

// MA5c: the strategy review.

/// The weak works of the MA2 tests with a CEO who notices and judges everything.
fn reviewed_works() -> (Game, SiteId, crate::state::ManagerId) {
    let (mut game, works, _) = weak_works(cheap_furnaces());
    let boss = hire_sharp(&mut game, ceo());
    (game, works, boss)
}

#[test]
fn the_ceo_reviews_the_strategy_at_the_end_of_each_period() {
    let (mut game, works, boss) = reviewed_works();
    // Quarterly by default: nothing before the end of March.
    let keys = news(&mut game, 40);
    assert!(!keys.iter().any(|k| k == keys::REVIEW));
    assert!(game.state().companies[0].reviews.is_empty());
    while game.date() < Date::new(1900, 4, 1).unwrap() {
        let keys = news(&mut game, 1);
        if game.date() == Date::new(1900, 4, 1).unwrap() {
            assert!(keys.iter().any(|k| k == keys::REVIEW), "{keys:?}");
        }
    }
    let state = game.state();
    let reviews = &state.companies[0].reviews;
    assert_eq!(reviews.len(), 1);
    let r = &reviews[0];
    assert_eq!(r.date, Date::new(1900, 4, 1).unwrap());
    assert_eq!(r.from, Date::new(1900, 1, 1).unwrap());
    assert_eq!(r.manager, boss);
    // From the ledger: the last three closed months.
    let months = &state.companies[0].ledger.months;
    let last = &months[months.len() - 3..];
    let revenue: Money = last
        .iter()
        .filter_map(|m| m.by_type.get(&CostType::Revenue))
        .copied()
        .sum();
    let result: Money = last.iter().map(|m| m.total()).sum();
    assert_eq!(r.total.revenue, revenue);
    assert_eq!(r.total.result, result);
    assert!(revenue > Money::ZERO, "the works sold iron");
    // The parts add up: continents and overhead, goods groups.
    let continents: Money = r.continents.iter().map(|(_, f)| f.result).sum();
    assert_eq!(continents + r.overhead, r.total.result);
    let groups: Money = r.groups.iter().map(|(_, f)| f.revenue).sum();
    assert_eq!(groups, r.total.revenue, "all revenue comes from products");
    // The weak works makes a loss: a risk.
    let works_result: Money = last
        .iter()
        .filter_map(|m| m.by_site.get(&works))
        .copied()
        .sum();
    if works_result < Money::ZERO {
        assert!(r.risks.contains(&crate::review::Risk::Loss {
            site: works,
            result: works_result
        }));
    }
    let view = crate::views::reviews(&game);
    assert_eq!(view.reviews.len(), 1);
    assert_eq!(view.reviews[0].to, "1900-03-31");
    assert_eq!(
        view.ceo.as_deref(),
        Some(game.state().managers[&boss].name.as_str())
    );
    assert_eq!(view.next_review.as_deref(), Some("1900-07-01"));

    // Monthly: one every month, the last eight kept.
    game.apply(Command::SetMandate {
        mandate: Mandate {
            review: crate::mandate::ReviewInterval::Monthly,
            goals: Goals {
                margin: Some(0.5),
                ..Goals::default()
            },
            ..Mandate::default()
        },
    })
    .unwrap();
    while game.date() < Date::new(1901, 1, 1).unwrap() {
        game.advance(RoundLength::Month, |_| {});
    }
    let reviews = &game.state().companies[0].reviews;
    assert_eq!(reviews.len(), 8);
    assert_eq!(reviews[7].date, Date::new(1901, 1, 1).unwrap());
    assert_eq!(reviews[7].from, Date::new(1900, 12, 1).unwrap());
    let goal = reviews[7].goals[0];
    assert_eq!(goal.goal, crate::review::Goal::Margin);
    if goal.met() == Some(false) {
        assert!(
            reviews[7]
                .risks
                .contains(&crate::review::Risk::Goal(crate::review::Goal::Margin))
        );
    }

    // Without a CEO no review.
    let (mut game, _, boss) = reviewed_works();
    game.apply(Command::DismissManager { manager: boss })
        .unwrap();
    through_quarter_end(&mut game);
    assert!(game.state().companies[0].reviews.is_empty());
}

#[test]
fn the_ceo_proposes_an_offer_at_the_review() {
    // Too little budget to offer at the board's checks: the CEO proposes it.
    let (mut game, rival, site) = buyer_game(0.0001);
    let boss = hire_sharp(&mut game, ceo());
    let next = game.state().next_concern;
    through_quarter_end(&mut game);
    let proposal = game
        .state()
        .concerns
        .iter()
        .find(|c| c.id >= next && c.reason == ConcernReason::Proposal)
        .expect("a proposal")
        .clone();
    assert_eq!(proposal.decision.topic, Topic::Offer);
    assert_eq!(proposal.position, ceo());
    assert_eq!(proposal.manager, boss);
    let review = game.state().companies[0].reviews.last().unwrap();
    assert!(review.chances.contains(&crate::review::Chance::Proposal {
        concern: proposal.id
    }));
    let view = crate::views::concerns(&game);
    let shown = view
        .open
        .iter()
        .flat_map(|g| &g.concerns)
        .find(|c| c.id == proposal.id)
        .unwrap();
    assert_eq!(shown.reason, "antrag");
    assert!(shown.important);
    // Approved: the offer goes out.
    game.apply(Command::AnswerConcern {
        concern: proposal.id,
        answer: ConcernAnswer::Choose(0),
    })
    .unwrap();
    let player = game.player();
    let offer = game
        .state()
        .offers
        .iter()
        .find(|o| o.buyer == player)
        .unwrap();
    assert_eq!(offer.seller, rival);
    assert_eq!(offer.object, DealObject::Site(site));
}

#[test]
fn a_new_site_answered_later_gets_its_steps() {
    // A concern founds a site and builds on it; meanwhile another site was founded.
    let mut game = new_game(cheap_furnaces());
    mine_and_works(&mut game);
    let c = game.catalog().clone();
    let aaa = c.countries.id("AAA").unwrap();
    let planned = SiteId(u32::try_from(game.state().sites.len()).unwrap());
    let choice = Choice {
        kind: ChoiceKind::Build,
        steps: Vec::new(),
    }
    .then(
        Command::FoundSite {
            country: aaa,
            kind: SiteType::Factory,
        },
        true,
    )
    .then(
        Command::BuildFacility {
            site: planned,
            facility: c.facilities.id("ofen").unwrap(),
            count: 1,
            size: crate::catalog::FacilitySize::Medium,
        },
        true,
    );
    let player = game.player();
    let decision = Decision::new(Topic::Expansion, player, choice);
    let boss = hire_sharp(&mut game, ceo());
    let concern = crate::state::Concern {
        id: game.state().next_concern,
        company: player,
        position: ceo(),
        manager: boss,
        decision,
        recommended: 0,
        options: vec![
            crate::state::ConcernOption {
                amount: Money::ZERO,
                forecast: None,
                once: Money::ZERO,
            };
            2
        ],
        reason: ConcernReason::Proposal,
        path: Vec::new(),
        parts: Vec::new(),
        created: game.date(),
        deadline: game.date().add_days(30),
        status: ConcernStatus::Open,
        closed: None,
    };
    let id = concern.id;
    let state = game.state_mut();
    state.next_concern += 1;
    state.concerns.push(concern);
    // Another site takes the planned number.
    game.apply(Command::FoundSite {
        country: aaa,
        kind: SiteType::Warehouse,
    })
    .unwrap();
    game.apply(Command::AnswerConcern {
        concern: id,
        answer: ConcernAnswer::Choose(0),
    })
    .unwrap();
    let sites = &game.state().sites;
    let new = sites.last().unwrap();
    assert_eq!(new.kind, SiteType::Factory);
    assert_eq!(new.slots.len(), 1, "the furnace is on the new works");
    assert!(
        sites[planned.index()].slots.is_empty(),
        "not on the warehouse"
    );
}
