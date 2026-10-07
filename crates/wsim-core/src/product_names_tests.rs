//! Product names (M42): name parts by year, free names, house brands, the command,
//! saves and the replay of the journal.

use std::sync::Arc;

use crate::calendar::RoundLength;
use crate::catalog::{Catalog, NamePattern, NamingStyle, ProductNaming, test_support};
use crate::command::{Command, CommandError, NameError};
use crate::game::Game;
use crate::ids::{Id, ProductId};
use crate::money::Money;
use crate::product_names::{self, same_name};
use crate::save;
use crate::state::{CompanyId, GameSettings, StartForm};

/// The production catalog; its end products (bread, carriage, wheel) are named in the
/// style `technik` with three stems.
fn catalog() -> Catalog {
    let mut c = test_support::production();
    let group = c.goods_groups.id("erze").unwrap();
    let mut style_of_group = vec![None; c.goods_groups.len()];
    style_of_group[group.index()] = Some(0);
    let pattern = |text: &str, until| NamePattern {
        text: text.into(),
        from: None,
        until,
    };
    c.product_naming = ProductNaming {
        house_brand: 0.5,
        excluded: vec!["Tesla".into()],
        styles: vec![NamingStyle {
            key: "technik".into(),
            stems: vec!["Arvon".into(), "Belkor".into(), "Corvel".into()],
            patterns: vec![
                pattern("{stamm} Typ {zahl}", Some(1939)),
                pattern("{stamm} {zahl}", None),
            ],
            numbers: vec![2, 3],
            letters: Vec::new(),
            additions: Vec::new(),
        }],
        style_of_group,
    };
    c
}

fn settings(c: &Catalog, start_year: i32) -> GameSettings {
    GameSettings {
        seed: 11,
        start_year,
        start_country: c.countries.id("AAA").unwrap(),
        start_capital: Money::from_usd(1_000_000.0).unwrap(),
        start_form: StartForm::Workshop,
        company_name: "Namen AG".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: Default::default(),
        ventures: 1.0,
    }
}

fn new_game(start_year: i32) -> Game {
    let c = Arc::new(catalog());
    let s = settings(&c, start_year);
    Game::new(c, s).unwrap()
}

/// A game with a second company besides the player.
fn with_rival() -> (Game, CompanyId) {
    let mut game = new_game(1900);
    let state = game.state_mut();
    let mut rival = state.companies[0].clone();
    "Rivale AG".clone_into(&mut rival.name);
    state.companies.push(rival);
    (game, CompanyId(1))
}

fn product(game: &Game, key: &str) -> ProductId {
    game.catalog().products.id(key).unwrap()
}

fn name(text: &str) -> Option<String> {
    Some(text.to_owned())
}

#[test]
fn names_come_from_the_parts_of_their_year() {
    let game = new_game(1900);
    let (c, s) = (game.catalog(), game.state());
    let bread = product(&game, "brot");
    let first = product_names::generate(c, s, s.player, bread).unwrap();
    // Same state, same name: the stream belongs to company and product.
    assert_eq!(
        product_names::generate(c, s, s.player, bread).as_deref(),
        Some(first.as_str())
    );
    let three = product_names::suggestions(c, s, s.player, bread, 3);
    assert_eq!(three[0], first);
    assert!(!same_name(&three[0], &three[1]) && !same_name(&three[1], &three[2]));
    assert!(!same_name(&three[0], &three[2]));
    for n in &three {
        let (stem, rest) = n.split_once(' ').unwrap();
        assert!(["Arvon", "Belkor", "Corvel"].contains(&stem), "{n}");
        assert!(["2", "3", "Typ 2", "Typ 3"].contains(&rest), "{n}");
    }
    // Raw materials and semi-finished goods have no names.
    let ore = product(&game, "erz");
    assert!(product_names::generate(c, s, s.player, ore).is_none());
    // The pattern with "Typ" ends in 1939.
    let later = new_game(1950);
    let s = later.state();
    for n in product_names::suggestions(later.catalog(), s, s.player, bread, 5) {
        assert!(!n.contains("Typ"), "{n}");
    }
}

#[test]
fn the_command_checks_the_name() {
    let (mut game, rival) = with_rival();
    let player = game.state().player;
    let bread = product(&game, "brot");
    let ore = product(&game, "erz");
    let name_it = |product, text: &str| Command::NameProduct {
        product,
        name: name(text),
    };
    assert_eq!(
        game.apply(name_it(ore, "Arvon 2")),
        Err(CommandError::NotNameable("erz".into()))
    );
    assert_eq!(
        game.apply(name_it(bread, "   ")),
        Err(CommandError::Name(NameError::Empty))
    );
    assert_eq!(
        game.apply(name_it(bread, &"x".repeat(41))),
        Err(CommandError::Name(NameError::TooLong { max: 40 }))
    );
    assert_eq!(
        game.apply(name_it(bread, "Neuer tesla")),
        Err(CommandError::Name(NameError::Excluded {
            name: "Neuer tesla".into()
        }))
    );
    game.apply(name_it(bread, "  Mein   Brot ")).unwrap();
    let names =
        |game: &Game, id: CompanyId| game.state().companies[id.index()].product_names.clone();
    assert_eq!(names(&game, player)[&bread], "Mein Brot");
    // Taken for the same product, free for another one.
    assert_eq!(
        game.apply_as(rival, name_it(bread, "mein brot")),
        Err(CommandError::Name(NameError::Taken {
            name: "mein brot".into()
        }))
    );
    let carriage = product(&game, "kutsche");
    game.apply_as(rival, name_it(carriage, "Mein Brot"))
        .unwrap();
    // Renaming to the own name is fine; `None` removes it.
    game.apply(name_it(bread, "MEIN BROT")).unwrap();
    game.apply(Command::NameProduct {
        product: bread,
        name: None,
    })
    .unwrap();
    assert!(names(&game, player).is_empty());
}

#[test]
fn stems_belong_to_their_company() {
    let (mut game, rival) = with_rival();
    let player = game.state().player;
    let (bread, carriage, wheel) = (
        product(&game, "brot"),
        product(&game, "kutsche"),
        product(&game, "rad"),
    );
    for (p, n) in [(bread, "Arvon 2"), (carriage, "Belkor 3")] {
        game.apply(Command::NameProduct {
            product: p,
            name: name(n),
        })
        .unwrap();
    }
    // The rival takes the stem nobody uses.
    let (c, s) = (game.catalog(), game.state());
    let rivals = product_names::suggestions(c, s, rival, wheel, 2);
    assert!(
        rivals.iter().all(|n| n.starts_with("Corvel ")),
        "{rivals:?}"
    );
    game.apply_as(
        rival,
        Command::NameProduct {
            product: wheel,
            name: Some(rivals[0].clone()),
        },
    )
    .unwrap();
    // The player stays with its own stems and never takes the rival's name.
    let (c, s) = (game.catalog(), game.state());
    let own = product_names::suggestions(c, s, player, wheel, 3);
    assert!(
        own.iter()
            .all(|n| n.starts_with("Arvon ") || n.starts_with("Belkor ")),
        "{own:?}"
    );
    assert!(own.iter().all(|n| !same_name(n, &rivals[0])));
}

#[test]
fn names_survive_saves_and_the_replay() {
    let mut game = new_game(1900);
    let bread = product(&game, "brot");
    game.apply(Command::NameProduct {
        product: bread,
        name: name("Arvon 3"),
    })
    .unwrap();
    game.advance(RoundLength::Day, |_| {});
    let player = game.state().player;
    let loaded = save::decode(&save::encode(&game), game.catalog().clone())
        .unwrap()
        .game;
    assert_eq!(
        loaded.state().companies[player.index()].product_names[&bread],
        "Arvon 3"
    );
    assert_eq!(loaded.state_hash(), game.state_hash());
    let c = Arc::new(catalog());
    let s = settings(&c, 1900);
    let replayed = Game::replay(c, s, game.journal()).unwrap();
    assert_eq!(replayed.state_hash(), game.state_hash());
}
