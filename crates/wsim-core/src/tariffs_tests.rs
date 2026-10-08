//! Tests of the tariffs (W3).

use crate::catalog::test_support;
use crate::catalog::{Catalog, Embargo, TariffDynamics, TariffModel, TariffZone};
use crate::ids::{CountryId, GoodsGroupId, Id};
use crate::state::PerId;
use crate::tariffs::TariffTable;
use crate::time_series::TimeSeries;

fn series(points: &[(i32, f64)]) -> TimeSeries {
    TimeSeries::new(points.to_vec()).unwrap()
}

/// Three countries: A with its own tariff, B and C with the default; A and B in a zone
/// from 1950, B and C under an embargo until 1960.
fn catalog() -> Catalog {
    let mut catalog = test_support::with_countries(&["AAA", "BBB", "CCC"]);
    let groups = catalog.goods_groups.len().max(1);
    catalog.tariffs = TariffModel {
        default: Some(series(&[(1900, 0.2), (2000, 0.1)])),
        countries: vec![Some(series(&[(1900, 0.4), (2026, 0.04)])), None, None],
        groups: vec![0.5; groups],
        products: Vec::new(),
        zones: vec![TariffZone {
            key: "zone".into(),
            factor: 0.0,
            members: vec![
                (CountryId::from_index(0), 1950, None),
                (CountryId::from_index(1), 1950, Some(1990)),
            ],
        }],
        embargoes: vec![Embargo {
            countries: (CountryId::from_index(1), CountryId::from_index(2)),
            from: 1940,
            until: Some(1960),
        }],
        dynamics: TariffDynamics {
            deviation: 0.01,
            min: 0.0,
            max: 0.6,
            levels: vec![("normal".into(), 1.0)],
            default_level: 0,
        },
        provenance: Default::default(),
    };
    catalog
}

#[test]
fn the_rate_depends_on_importer_group_zone_and_embargo() {
    let catalog = catalog();
    let g = GoodsGroupId::from_index(0);
    let (a, b, c) = (
        CountryId::from_index(0),
        CountryId::from_index(1),
        CountryId::from_index(2),
    );
    let t = TariffTable::new(&catalog, 1900, &PerId::default());
    // Same country: no tariff; otherwise the importer's tariff times the group factor.
    assert_eq!(t.rate(&catalog, a, a, g), Some(0.0));
    assert!((t.rate(&catalog, b, a, g).unwrap() - 0.2).abs() < 1e-12);
    assert!((t.rate(&catalog, a, b, g).unwrap() - 0.1).abs() < 1e-12);
    // In the zone from 1950 until before 1990.
    let t = TariffTable::new(&catalog, 1950, &PerId::default());
    assert_eq!(t.rate(&catalog, b, a, g), Some(0.0));
    assert!(t.rate(&catalog, c, a, g).unwrap() > 0.0);
    // The embargo blocks both directions, and only in its years.
    assert_eq!(t.rate(&catalog, b, c, g), None);
    assert_eq!(t.rate(&catalog, c, b, g), None);
    let t = TariffTable::new(&catalog, 1990, &PerId::default());
    assert!(t.rate(&catalog, b, a, g).unwrap() > 0.0);
    assert!(t.rate(&catalog, b, c, g).is_some());
}

#[test]
fn without_tariffs_trade_is_free() {
    let catalog = test_support::with_countries(&["AAA", "BBB"]);
    let t = TariffTable::new(&catalog, 1950, &PerId::default());
    assert_eq!(
        t.rate(
            &catalog,
            CountryId::from_index(0),
            CountryId::from_index(1),
            GoodsGroupId::from_index(0)
        ),
        Some(0.0)
    );
}

#[test]
fn offsets_apply_after_the_data_and_stay_in_bounds() {
    let catalog = catalog();
    let mut offsets: PerId<CountryId, f64> = PerId::from_fn(3, |_| 0.0);
    *offsets.get_mut(CountryId::from_index(0)) = 1.0;
    // Before the end of the data the offsets do not count.
    let t = TariffTable::new(&catalog, 2026, &offsets);
    assert!((t.average(CountryId::from_index(0)) - 0.04).abs() < 1e-12);
    // After it they do, up to the maximum.
    let t = TariffTable::new(&catalog, 2027, &offsets);
    assert!((t.average(CountryId::from_index(0)) - 0.6).abs() < 1e-12);
}

#[test]
fn the_dynamics_move_tariffs_after_the_data_only() {
    use crate::game::Game;
    use crate::state::{GameSettings, StartForm};
    use std::sync::Arc;
    let start = |dynamics: f64| {
        let mut catalog = test_support::trading();
        let n = catalog.countries.len();
        catalog.tariffs = TariffModel {
            default: Some(series(&[(1900, 0.2), (2026, 0.1)])),
            countries: vec![None; n],
            groups: vec![1.0; catalog.goods_groups.len()],
            dynamics: TariffDynamics {
                deviation: 0.05,
                min: 0.0,
                max: 0.15,
                levels: vec![("normal".into(), 1.0)],
                default_level: 0,
            },
            ..TariffModel::default()
        };
        let catalog = Arc::new(catalog);
        let settings = GameSettings {
            seed: 9,
            start_year: 1900,
            start_country: CountryId::from_index(0),
            start_capital: crate::money::Money::from_usd(1_000_000.0).unwrap(),
            start_form: StartForm::Trading,
            company_name: "Zoll AG".into(),
            research_ahead_factor: 1.0,
            market_scale: 1.0,
            ai: Default::default(),
            ventures: 1.0,
            tariff_dynamics: dynamics,
            event_effects: true,
            person: Default::default(),
        };
        Game::new(catalog, settings).unwrap()
    };
    let mut game = start(1.0);
    let catalog = game.catalog().clone();
    // Within the data nothing changes.
    crate::tariffs::new_year(game.state_mut(), &catalog, 2026);
    assert!(game.state().tariff_offsets.is_empty());
    // After it every country moves, within the bounds, the same with the same seed.
    let mut paths = Vec::new();
    for _ in 0..2 {
        let mut game = start(1.0);
        let mut path = Vec::new();
        for year in 2027..2060 {
            crate::tariffs::new_year(game.state_mut(), &catalog, year);
            let t = TariffTable::new(&catalog, year, &game.state().tariff_offsets);
            let z = t.average(CountryId::from_index(0));
            assert!((0.0..=0.15).contains(&z), "{z}");
            path.push(z);
        }
        paths.push(path);
    }
    assert_eq!(paths[0], paths[1]);
    assert!(paths[0].iter().any(|&z| (z - 0.1).abs() > 1e-6));
    // Without dynamics they stay.
    let mut fixed = start(0.0);
    crate::tariffs::new_year(fixed.state_mut(), &catalog, 2030);
    assert!(fixed.state().tariff_offsets.is_empty());
}
