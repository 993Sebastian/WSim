//! Reading of older save formats.
//!
//! Formats 1 and 2 (M3/M4): numeric country IDs with a key list, cash instead of a
//! ledger, no sites. Format 1 additionally stored the derived country values, which
//! are ignored here.

use std::collections::BTreeSet;

use serde::Deserialize;

use super::{CatalogKeys, LoadError, catalog_keys};
use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::game::JournalEntry;
use crate::ids::{CountryId, IdKind};
use crate::ledger::Ledger;
use crate::money::Money;
use crate::rng::SimRng;
use crate::state::{Company, CompanyId, CompanyKind, GameSettings, GameState, PerId, StartForm};

#[derive(Deserialize)]
struct BodyV2 {
    catalog_keys: KeysV2,
    state: StateV2,
    journal: Vec<JournalEntry>,
}

#[derive(Deserialize)]
struct KeysV2 {
    countries: Vec<String>,
}

#[derive(Deserialize)]
struct StateV2 {
    settings: SettingsV2,
    date: Date,
    world_rng: SimRng,
    companies: Vec<CompanyV2>,
    player: CompanyId,
    game_over: bool,
}

#[derive(Deserialize)]
struct SettingsV2 {
    seed: u64,
    start_year: i32,
    start_country: u16,
    start_capital: Money,
    start_form: StartForm,
    company_name: String,
}

#[derive(Deserialize)]
struct CompanyV2 {
    name: String,
    kind: CompanyKind,
    headquarters: u16,
    cash: Money,
    founded: Date,
    rng: SimRng,
}

pub(super) fn decode_v2(
    body: &[u8],
    catalog: &Catalog,
) -> Result<(CatalogKeys, GameState, Vec<JournalEntry>), LoadError> {
    let old: BodyV2 = rmp_serde::from_slice(body).map_err(|e| LoadError::Corrupt(e.to_string()))?;
    let country = |index: u16| -> Result<CountryId, LoadError> {
        let key = old
            .catalog_keys
            .countries
            .get(usize::from(index))
            .ok_or_else(|| LoadError::Corrupt("country index".into()))?;
        catalog
            .countries
            .id(key)
            .ok_or_else(|| LoadError::ContentMissing {
                kind: IdKind::Country.name(),
                key: key.clone(),
            })
    };
    let s = old.state;
    let companies = s
        .companies
        .into_iter()
        .map(|c| {
            Ok(Company {
                brands: Vec::new(),
                advertising: Vec::new(),
                auction_until: None,
                development: Default::default(),
                owners: Vec::new(),
                name: c.name,
                kind: c.kind,
                headquarters: country(c.headquarters)?,
                founded: c.founded,
                rng: c.rng,
                ledger: Ledger::new(s.date, c.cash),
                technologies: BTreeSet::new(),
                bankrupt: false,
                loans: Vec::new(),
                loss_carryforward: Money::ZERO,
                sales_policies: Vec::new(),
                research: Default::default(),
                ai: None,
            })
        })
        .collect::<Result<Vec<_>, LoadError>>()?;
    let state = GameState {
        settings: GameSettings {
            seed: s.settings.seed,
            start_year: s.settings.start_year,
            start_country: country(s.settings.start_country)?,
            start_capital: s.settings.start_capital,
            start_form: s.settings.start_form,
            company_name: s.settings.company_name,
            research_ahead_factor: 1.0,
            market_scale: 1.0,
            ai: Default::default(),
        },
        date: s.date,
        world_rng: s.world_rng,
        countries: PerId::default(),
        companies,
        sites: Vec::new(),
        markets: PerId::default(),
        shipments: Vec::new(),
        plots: Vec::new(),
        routes: Default::default(),
        import_markets: Default::default(),
        deposits: PerId::default(),
        inventions: PerId::default(),
        developments: PerId::default(),
        milestones: PerId::default(),
        watched_markets: Vec::new(),
        standings: Vec::new(),
        offers: Vec::new(),
        next_offer: 0,
        player: s.player,
        game_over: s.game_over,
    };
    // Old saves listed only countries; report them as changed data unless identical.
    let mut keys = catalog_keys(catalog);
    keys.insert(
        IdKind::Country.name().to_owned(),
        old.catalog_keys.countries,
    );
    Ok((keys, state, old.journal))
}
