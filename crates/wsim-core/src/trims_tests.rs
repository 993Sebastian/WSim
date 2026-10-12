//! Tests of the trim levels (MN2): premium wins the richest fifth, simple the poorest.

use std::sync::Arc;

use crate::calendar::RoundLength;
use crate::catalog::{Trim, TrimEffect, TrimModel, test_support};
use crate::command::{Command, CommandError};
use crate::game::Game;
use crate::money::Money;
use crate::production;
use crate::state::{GameSettings, StartForm};

fn trims() -> TrimModel {
    TrimModel {
        enabled: true,
        simple: TrimEffect {
            material: 0.85,
            labor: 0.9,
            quality: -15.0,
        },
        premium: TrimEffect {
            material: 1.25,
            labor: 1.3,
            quality: 15.0,
        },
        ai_premium_from: 0.75,
        ai_simple_up_to: 0.25,
    }
}

fn game() -> Game {
    let mut c = test_support::production();
    c.production_model.trims = trims();
    // Iron stands in for an end product here.
    let iron = c.products.id("eisen").unwrap();
    c.products.get_mut(iron).kind = crate::catalog::ProductKind::EndProduct;
    let catalog = Arc::new(c);
    let settings = GameSettings {
        seed: 3,
        start_year: 1900,
        start_country: catalog.countries.id("AAA").unwrap(),
        start_capital: Money::from_usd(10_000_000.0).unwrap(),
        start_form: StartForm::Workshop,
        company_name: "Ausstattung AG".into(),
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

#[test]
fn a_trim_changes_inputs_labor_and_quality() {
    let mut g = game();
    let c = g.catalog().clone();
    g.apply(Command::FoundSite {
        country: c.countries.id("AAA").unwrap(),
        kind: crate::catalog::SiteType::Factory,
    })
    .unwrap();
    let site = crate::state::SiteId(u32::try_from(g.state().sites.len() - 1).unwrap());
    let s = site.index();
    let product = c.products.id("eisen").unwrap();
    let standard = production::site_effect(&c, g.state(), &g.state().sites[s], product);
    // Only end products have trims.
    let ore = c.products.id("erz").unwrap();
    assert!(matches!(
        g.apply(Command::SetTrim {
            site,
            product: ore,
            trim: Trim::Premium
        }),
        Err(CommandError::NoTrim)
    ));
    g.apply(Command::SetTrim {
        site,
        product,
        trim: Trim::Premium,
    })
    .unwrap();
    let premium = production::site_effect(&c, g.state(), &g.state().sites[s], product);
    assert!((premium.inputs - standard.inputs * 1.25).abs() < 1e-12);
    assert!((premium.labor - standard.labor * 1.3).abs() < 1e-12);
    assert!((premium.quality - standard.quality - 15.0).abs() < 1e-12);
    // Back to standard leaves nothing stored; the setting survives save and load.
    g.apply(Command::SetTrim {
        site,
        product,
        trim: Trim::Simple,
    })
    .unwrap();
    let loaded = crate::save::decode(&crate::save::encode(&g), g.catalog().clone())
        .unwrap()
        .game;
    assert_eq!(
        production::trim(&loaded.state().sites[s], product),
        Trim::Simple
    );
    g.apply(Command::SetTrim {
        site,
        product,
        trim: Trim::Standard,
    })
    .unwrap();
    assert!(g.state().sites[s].trims.is_empty());
    g.advance(RoundLength::Day, |_| {});
}

#[test]
fn ai_companies_choose_by_skill() {
    let mut c = test_support::production();
    c.production_model.trims = trims();
    assert_eq!(crate::trims::ai_choice(&c, 0.9), Trim::Premium);
    assert_eq!(crate::trims::ai_choice(&c, 0.5), Trim::Standard);
    assert_eq!(crate::trims::ai_choice(&c, 0.1), Trim::Simple);
}

/// MN2: with the data's trim effects, premium (dearer, better) wins the richest fifth and
/// simple (cheaper, worse) the poorest.
#[test]
fn premium_wins_the_rich_and_simple_the_poor() {
    let c = test_support::production();
    let m = &c.market_model;
    let t = trims();
    // Price follows the cost: material and labor factors as a rough mark-up.
    let price = |e: TrimEffect| libm::log((e.material + e.labor) / 2.0);
    let appeal =
        |q: usize, e: TrimEffect| crate::market::appeal(m, q, price(e), 50.0 + e.quality, 0.5);
    assert!(appeal(4, t.premium) > appeal(4, t.simple));
    assert!(appeal(0, t.simple) > appeal(0, t.premium));
}
