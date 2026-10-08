//! Tests of the controlling view (W7).

use crate::catalog::test_support;
use crate::game::Game;
use crate::ledger::{Account, CostCenter, CostType};
use crate::state::SiteId;
use crate::trade_tests::{days, iron, new_game, usd, warehouse};
use crate::views::{ControllingNode, controlling};

fn book(game: &mut Game, cost: CostType, center: CostCenter, amount: f64) {
    let player = game.player();
    let ledger = &mut game.state_mut().companies[player.index()].ledger;
    if amount >= 0.0 {
        ledger.income(cost, center, Account::Cash, usd(amount));
    } else {
        ledger.expense(cost, center, Account::Cash, usd(-amount));
    }
}

/// Every level is the sum of its children, and the margins follow the blocks.
fn adds_up(n: &ControllingNode) {
    let near = |a: f64, b: f64| (a - b).abs() < 1e-6;
    assert!(
        near(n.margin1_usd, n.revenue_usd + n.variable_usd),
        "{}",
        n.key
    );
    assert!(
        near(n.margin2_usd, n.margin1_usd + n.fixed_usd),
        "{}",
        n.key
    );
    assert!(near(n.result_usd, n.margin2_usd + n.other_usd), "{}", n.key);
    let costs: f64 = n.costs.iter().map(|c| c.usd).sum();
    assert!(near(costs, n.result_usd), "{}", n.key);
    if !n.children.is_empty() {
        let sum = |f: fn(&ControllingNode) -> f64| n.children.iter().map(f).sum::<f64>();
        assert!(near(sum(|c| c.result_usd), n.result_usd), "{}", n.key);
        assert!(near(sum(|c| c.revenue_usd), n.revenue_usd), "{}", n.key);
        assert!(near(sum(|c| c.fixed_usd), n.fixed_usd), "{}", n.key);
    }
    for c in &n.children {
        adds_up(c);
    }
}

fn find<'a>(n: &'a ControllingNode, key: &str) -> Option<&'a ControllingNode> {
    if n.key == key {
        return Some(n);
    }
    n.children.iter().find_map(|c| find(c, key))
}

fn sales(game: &mut Game, site: SiteId, revenue: f64) {
    let p = iron(game);
    book(
        game,
        CostType::Revenue,
        CostCenter::product(site, p),
        revenue,
    );
    book(
        game,
        CostType::Material,
        CostCenter::product(site, p),
        -0.4 * revenue,
    );
    book(game, CostType::Personnel, CostCenter::site(site), -100.0);
}

#[test]
fn margins_add_up_through_the_levels() {
    let mut game = new_game(test_support::trading());
    let player = game.player();
    let a = warehouse(&mut game, player, "AAA", 0.0);
    let b = warehouse(&mut game, player, "BBB", 0.0);
    sales(&mut game, a, 1000.0);
    sales(&mut game, b, 500.0);
    book(&mut game, CostType::Interest, CostCenter::default(), -50.0);
    let view = controlling(&game, "jahr");
    assert_eq!(view.period, "jahr");
    let root = view.root.expect("a year");
    assert_eq!(root.level, "firma");
    adds_up(&root);
    assert!((root.revenue_usd - 1500.0).abs() < 1e-6);
    assert!((root.margin1_usd - 900.0).abs() < 1e-6);
    assert!((root.margin2_usd - 700.0).abs() < 1e-6);
    assert!((root.result_usd - 650.0).abs() < 1e-6);
    let site = find(&root, &format!("standort:{}", a.0)).unwrap();
    assert_eq!(site.country.as_deref(), Some("AAA"));
    assert!((site.margin2_usd - 500.0).abs() < 1e-6);
    // The product and what the site spends for no product.
    assert_eq!(site.children.len(), 2);
    let central = find(&root, &format!("zentrale:{}", player.0)).unwrap();
    assert!((central.other_usd + 50.0).abs() < 1e-6);
    assert!(view.periods.contains(&"jahr".to_owned()));
}

#[test]
fn the_month_before_and_the_series() {
    let mut game = new_game(test_support::trading());
    let player = game.player();
    let a = warehouse(&mut game, player, "AAA", 0.0);
    // Jan 1–Jan 31.
    sales(&mut game, a, 1000.0);
    days(&mut game, 31);
    sales(&mut game, a, 2000.0);
    days(&mut game, 29);
    let view = controlling(&game, "monat");
    let root = view.root.expect("closed months");
    adds_up(&root);
    let site = find(&root, &format!("standort:{}", a.0)).unwrap();
    assert!(
        (site.result_usd - 1100.0).abs() < 1e-6,
        "{}",
        site.result_usd
    );
    assert!((site.previous_result_usd.unwrap() - 500.0).abs() < 1e-6);
    assert_eq!(site.series_usd.len(), 2);
    assert!(root.previous_result_usd.is_some());
    assert!(view.periods.contains(&"monat".to_owned()));
}

#[test]
fn the_group_holds_its_companies() {
    use crate::command::Command;
    use crate::group::SubsidiaryFocus;
    let mut catalog = test_support::trading();
    catalog.subsidiaries = crate::catalog::SubsidiaryModel {
        enabled: true,
        min_capital: usd(10_000.0),
        founding_cost: usd(1_000.0),
        competence: 0.5,
        aggressiveness: 0.5,
        logistics_cash_share: 0.5,
        logistics_min_return: 0.0,
        provenance: Default::default(),
    };
    let mut game = new_game(catalog);
    let player = game.player();
    let a = warehouse(&mut game, player, "AAA", 0.0);
    sales(&mut game, a, 1000.0);
    let aaa = game.catalog().countries.id("AAA").unwrap();
    game.apply(Command::FoundSubsidiary {
        name: "Tochter".into(),
        country: aaa,
        capital: usd(20_000.0),
        focus: SubsidiaryFocus::Production,
    })
    .unwrap();
    let view = controlling(&game, "jahr");
    let root = view.root.unwrap();
    assert_eq!(root.level, "konzern");
    assert_eq!(root.children.len(), 2);
    adds_up(&root);
}
