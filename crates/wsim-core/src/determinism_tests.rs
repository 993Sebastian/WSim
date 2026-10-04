//! Reproducibility (Lastenheft §16.1), round loop and save games.

use std::sync::Arc;

use crate::calendar::{Date, GAME_END, RoundLength};
use crate::catalog::{Catalog, test_support};
use crate::command::{Command, CommandError, NameError};
use crate::game::{Game, JournalEntry, NewGameError, Progress};
use crate::ids::{CountryId, Id};
use crate::message::keys;
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::save::{self, LoadError, SAVE_FORMAT_VERSION};
use crate::state::{GameSettings, Holder, Stake, StartForm};

fn settings(seed: u64) -> GameSettings {
    GameSettings {
        seed,
        start_year: 1900,
        start_country: CountryId::from_index(1),
        start_capital: Money::from_usd(100_000.0).unwrap(),
        start_form: StartForm::Workshop,
        company_name: "Muster & Söhne".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: Default::default(),
    }
}

fn catalog() -> Arc<Catalog> {
    Arc::new(test_support::sample())
}

fn date(y: i32, m: u32, d: u32) -> Date {
    Date::new(y, m, d).unwrap()
}

#[derive(Clone, Debug)]
enum Action {
    Rename(String),
    Round(RoundLength),
}

/// A fixed, pseudo-random mix of decisions and rounds of all lengths.
fn script(seed: u64, rounds: usize) -> Vec<Action> {
    let mut rng = SimRng::for_stream(seed, Stream::World);
    let mut actions = Vec::new();
    for _ in 0..rounds {
        if rng.chance(0.3) {
            actions.push(Action::Rename(format!("Firma {}", rng.below(1000))));
        }
        actions.push(Action::Round(
            RoundLength::ALL[usize::try_from(rng.below(4)).unwrap()],
        ));
    }
    actions
}

fn play(game: &mut Game, actions: &[Action]) {
    for action in actions {
        match action {
            Action::Rename(name) => game
                .apply(Command::RenameCompany { name: name.clone() })
                .unwrap(),
            Action::Round(length) => {
                game.advance(*length, |_| {});
            }
        }
    }
}

fn played(seed: u64, actions: &[Action]) -> Game {
    let mut game = Game::new(catalog(), settings(seed)).unwrap();
    play(&mut game, actions);
    game
}

#[test]
fn same_seed_and_decisions_give_the_same_state() {
    let actions = script(1, 60);
    let a = played(42, &actions);
    let b = played(42, &actions);
    assert_eq!(a.state_hash(), b.state_hash());
    assert_eq!(a.state(), b.state());
    assert!(
        a.date() > date(1901, 1, 1),
        "the script covers more than a year"
    );
}

#[test]
fn different_seeds_give_different_states() {
    let actions = script(1, 5);
    assert_ne!(
        played(1, &actions).state_hash(),
        played(2, &actions).state_hash()
    );
}

#[test]
fn replaying_the_journal_reproduces_the_game() {
    let game = played(7, &script(3, 40));
    let replayed = Game::replay(catalog(), settings(7), game.journal()).unwrap();
    assert_eq!(replayed.state_hash(), game.state_hash());
    assert_eq!(replayed.journal(), game.journal());
}

#[test]
fn replay_rejects_a_journal_that_does_not_fit() {
    let journal = [JournalEntry::Round {
        from: date(1905, 1, 1),
        length: RoundLength::Day,
    }];
    assert!(Game::replay(catalog(), settings(7), &journal).is_err());
}

#[test]
fn save_load_and_continue_equals_playing_through() {
    let actions = script(5, 50);
    let (first, second) = actions.split_at(30);

    let straight = played(9, &actions);

    let half = played(9, first);
    let bytes = save::encode(&half);
    let mut loaded = save::decode(&bytes, catalog()).unwrap();
    assert!(!loaded.data_changed);
    assert_eq!(loaded.game.state(), half.state());
    play(&mut loaded.game, second);

    assert_eq!(loaded.game.state_hash(), straight.state_hash());
    assert_eq!(loaded.game.journal(), straight.journal());
}

#[test]
fn a_month_round_simulates_every_day_with_progress() {
    let mut game = Game::new(catalog(), settings(1)).unwrap();
    let mut progress: Vec<Progress> = Vec::new();
    let report = game.advance(RoundLength::Month, |p| progress.push(p));
    assert_eq!(
        (report.from, report.to, report.days),
        (date(1900, 1, 1), date(1900, 2, 1), 31)
    );
    assert_eq!(progress.len(), 31);
    assert_eq!(progress.last().unwrap().done, 31);
    assert!(progress.iter().all(|p| p.total == 31));
    assert_eq!(game.date(), date(1900, 2, 1));
}

#[test]
fn a_new_year_is_reported() {
    let mut game = Game::new(catalog(), settings(1)).unwrap();
    let mut reports = Vec::new();
    for _ in 0..4 {
        reports.push(game.advance(RoundLength::Quarter, |_| {}));
    }
    assert!(reports[..3].iter().all(|r| r.messages.is_empty()));
    let message = &reports[3].messages[0];
    assert_eq!(message.key, keys::NEW_YEAR);
}

#[test]
fn country_values_follow_the_yearly_data() {
    let mut game = Game::new(catalog(), settings(1)).unwrap();
    let aaa = CountryId::from_index(0);
    // Yearly values apply to the middle of the year.
    assert_eq!(game.state().countries.get(aaa).population, 1_000_000.0);
    while game.date() < date(1905, 7, 2) {
        game.advance(RoundLength::Day, |_| {});
        if game.date() == date(1900, 7, 2) {
            let p = game.state().countries.get(aaa).population;
            assert!((p - 1_000_000.0).abs() < 1.0, "{p}");
        }
    }
    let p = game.state().countries.get(aaa).population;
    assert!((p - 1_050_000.0).abs() < 50.0, "{p}");
}

#[test]
fn the_game_ends_on_31_december_2100() {
    let mut s = settings(1);
    s.start_year = 2026;
    let mut game = Game::new(catalog(), s).unwrap();
    let mut last = None;
    while !game.is_over() {
        last = Some(game.advance(RoundLength::Quarter, |_| {}));
    }
    assert_eq!(game.date(), GAME_END);
    assert!(
        last.unwrap()
            .messages
            .iter()
            .any(|m| m.key == keys::GAME_END)
    );
    assert_eq!(game.advance(RoundLength::Month, |_| {}).days, 0);
    let rename = game.apply(Command::RenameCompany { name: "Neu".into() });
    assert_eq!(rename, Err(CommandError::GameOver));
}

#[test]
fn new_game_checks_the_settings() {
    let with = |f: fn(&mut GameSettings)| {
        let mut s = settings(1);
        f(&mut s);
        Game::new(catalog(), s).err()
    };
    assert_eq!(
        with(|s| s.start_year = 1899),
        Some(NewGameError::StartYear { year: 1899 })
    );
    assert_eq!(
        with(|s| s.start_year = 2027),
        Some(NewGameError::StartYear { year: 2027 })
    );
    assert_eq!(
        with(|s| s.start_capital = Money::ZERO),
        Some(NewGameError::StartCapital)
    );
    assert_eq!(
        with(|s| s.start_country = CountryId::from_index(5)),
        Some(NewGameError::StartCountry)
    );
    assert_eq!(
        with(|s| s.company_name = "   ".into()),
        Some(NewGameError::Name(NameError::Empty))
    );
    assert_eq!(
        with(|s| s.company_name = "x".repeat(61)),
        Some(NewGameError::Name(NameError::TooLong { max: 60 }))
    );
    assert!(with(|_| {}).is_none());
}

#[test]
fn rejected_commands_stay_out_of_the_journal() {
    let mut game = Game::new(catalog(), settings(1)).unwrap();
    let error = game
        .apply(Command::RenameCompany { name: " ".into() })
        .unwrap_err();
    assert_eq!(error, CommandError::Name(NameError::Empty));
    assert_eq!(error.message().key, keys::NAME_EMPTY);
    assert!(game.journal().is_empty());

    game.apply(Command::RenameCompany {
        name: "  Krupp & Co.  ".into(),
    })
    .unwrap();
    assert_eq!(
        game.state().company(game.player()).unwrap().name,
        "Krupp & Co."
    );
    assert_eq!(game.journal().len(), 1);
}

#[test]
fn save_header_is_readable_without_loading() {
    let game = played(1, &script(1, 3));
    let header = save::read_header(&save::encode(&game)).unwrap();
    assert_eq!(header.format_version, SAVE_FORMAT_VERSION);
    assert_eq!(header.date, game.date());
    assert_eq!(header.data_version, 1);
}

#[test]
fn broken_saves_are_rejected() {
    let bytes = save::encode(&played(1, &script(1, 3)));
    assert_eq!(
        save::decode(b"kein Spielstand", catalog()).unwrap_err(),
        LoadError::NotASave
    );

    let mut truncated = bytes.clone();
    truncated.truncate(bytes.len() - 20);
    assert!(matches!(
        save::decode(&truncated, catalog()),
        Err(LoadError::Corrupt(_))
    ));

    let newer = bytes
        .windows(17)
        .position(|w| w == b"\"format_version\":")
        .unwrap();
    let mut too_new = bytes.clone();
    too_new[newer + 17] = b'9';
    assert_eq!(
        save::decode(&too_new, catalog()).unwrap_err(),
        LoadError::TooNew { found: 9 }
    );
}

#[test]
fn saves_that_do_not_fit_the_catalog_are_rejected() {
    // The keys match, but the head office points to a country that does not exist. It
    // is written as a number (no key), which reading rejects as outside the catalog.
    let game = played(1, &script(1, 3));
    let mut state = game.state().clone();
    state.companies[0].headquarters = CountryId::from_index(9);
    let tampered = save::encode(&Game::from_parts(catalog(), state, Vec::new()));
    assert_eq!(
        save::decode(&tampered, catalog()).unwrap_err(),
        LoadError::Corrupt("ID out of range".into())
    );
}

#[test]
fn saves_survive_changed_data() {
    let game = played(1, &script(2, 10));
    let bytes = save::encode(&game);

    // A new country sorts before the others; all IDs shift by one.
    let changed = Arc::new(test_support::with_countries(&["AAB", "AAA", "BBB"]));
    let loaded = save::decode(&bytes, changed.clone()).unwrap();
    assert!(loaded.data_changed);
    let state = loaded.game.state();
    let bbb = changed.countries.id("BBB").unwrap();
    assert_eq!(state.settings.start_country, bbb);
    assert_eq!(state.company(state.player).unwrap().headquarters, bbb);
    // Country values are derived and follow the new data.
    let expected = crate::country_model::compute(&changed, bbb, state.date);
    assert_eq!(state.countries.get(bbb), &expected);
    assert_eq!(state.countries.len(), 3);

    // A country that disappeared from the data cannot be loaded.
    let reduced = Arc::new(test_support::with_countries(&["AAA"]));
    assert_eq!(
        save::decode(&bytes, reduced).unwrap_err(),
        LoadError::ContentMissing {
            kind: "land",
            key: "BBB".into()
        }
    );
}

/// Saves of every format version. They must stay loadable in every later version.
const FIXTURES: &[(u32, &[u8])] = &[
    (1, include_bytes!("../tests/fixtures/saves/v1.wsim")),
    (2, include_bytes!("../tests/fixtures/saves/v2.wsim")),
    (3, include_bytes!("../tests/fixtures/saves/v3.wsim")),
];

#[test]
fn saves_of_all_versions_stay_loadable() {
    assert_eq!(
        FIXTURES.last().unwrap().0,
        SAVE_FORMAT_VERSION,
        "fixture for the current version missing"
    );
    for &(version, bytes) in FIXTURES {
        let loaded = save::decode(bytes, catalog()).unwrap_or_else(|e| panic!("v{version}: {e:?}"));
        let state = loaded.game.state();
        assert_eq!(loaded.header.format_version, version);
        assert_eq!(state.date, date(1900, 4, 1));
        assert_eq!(state.company(state.player).unwrap().name, "Fixture GmbH");
        assert_eq!(
            state.company(state.player).unwrap().ledger.cash(),
            Money::from_usd(250_000.0).unwrap()
        );
        assert_eq!(state.countries.len(), 2, "country values are recomputed");
        // Owners came with the preparation for investors (Lastenheft §17.3).
        let player = state.company(state.player).unwrap();
        assert_eq!(player.majority_holder(), Some(Holder::Player), "v{version}");
        for c in &state.companies {
            let total: f64 = c.owners.iter().map(|s| s.share).sum();
            assert!((total - 1.0).abs() < 1e-9, "v{version}: {}", c.name);
        }
    }
}

#[test]
fn the_player_owns_the_own_company_and_ai_companies_are_private() {
    let mut s = settings(5);
    s.ai.companies = 3;
    let game = Game::new(catalog(), s).unwrap();
    let state = game.state();
    for (i, c) in state.companies.iter().enumerate() {
        let expected = if i == state.player.index() {
            Holder::Player
        } else {
            Holder::Private
        };
        assert_eq!(c.majority_holder(), Some(expected), "{}", c.name);
    }
    // Two holders with half each: no majority.
    let mut c = state.companies[0].clone();
    c.owners = vec![
        Stake {
            holder: Holder::Player,
            share: 0.5,
        },
        Stake {
            holder: Holder::Private,
            share: 0.5,
        },
    ];
    assert_eq!(c.majority_holder(), None);
}

/// Writes the fixture for the current version. Run once per format version:
/// `cargo test -p wsim-core -- --ignored write_fixture`
#[test]
#[ignore = "writes a fixture file"]
fn write_fixture() {
    let mut s = settings(11);
    s.company_name = "Fixture GmbH".into();
    s.start_capital = Money::from_usd(250_000.0).unwrap();
    let mut game = Game::new(catalog(), s).unwrap();
    game.advance(RoundLength::Quarter, |_| {});
    let path = format!(
        "{}/tests/fixtures/saves/v{SAVE_FORMAT_VERSION}.wsim",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(path, save::encode(&game)).unwrap();
}
