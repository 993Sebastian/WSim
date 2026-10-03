//! Save games (Lastenheft §16.3).
//!
//! Layout of a save file:
//!
//! | Bytes | Content |
//! | --- | --- |
//! | 8 | `WSIMSAVE` |
//! | 4 | Length of the header (little endian) |
//! | n | Header as JSON (readable without unpacking, for the load menu) |
//! | rest | Body: MessagePack with field names, compressed with DEFLATE |
//!
//! The body stores catalog references by key (`DEU`), so a save remains loadable when
//! the data files change: IDs are translated to the new catalog on load. Older format
//! versions are upgraded step by step by the functions in [`MIGRATIONS`].
//! The core does no file IO; callers read and write the bytes.

use std::io::{Read, Write};
use std::sync::Arc;

use flate2::Compression;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::game::{Game, JournalEntry};
use crate::ids::{CountryId, Id};
use crate::message::{Message, Param, keys};
use crate::state::GameState;

/// Current save format. Raise it on every incompatible change of `GameState`, add a
/// migration and keep a save of the old version under `tests/fixtures/saves/`.
pub const SAVE_FORMAT_VERSION: u32 = 1;

const MAGIC: &[u8; 8] = b"WSIMSAVE";

/// Upgrades of the body, from version `n` to `n + 1`, operating on the untyped
/// MessagePack tree. Index 0 upgrades version 1 to 2, and so on.
type Migration = fn(&mut rmpv::Value) -> Result<(), String>;
const MIGRATIONS: &[Migration] = &[];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SaveHeader {
    pub format_version: u32,
    pub game_version: String,
    pub data_version: u32,
    pub date: Date,
    pub company_name: String,
}

/// Keys of the catalog entries the state refers to, in ID order at the time of saving.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct CatalogKeys {
    countries: Vec<String>,
}

impl CatalogKeys {
    fn of(catalog: &Catalog) -> Self {
        Self {
            countries: catalog
                .countries
                .ids()
                .map(|id| catalog.countries.key(id).to_owned())
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SaveBody {
    catalog_keys: CatalogKeys,
    state: GameState,
    journal: Vec<JournalEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadError {
    NotASave,
    TooNew {
        found: u32,
    },
    TooOld {
        found: u32,
    },
    Corrupt(String),
    /// The save refers to an entry that no longer exists in the data.
    ContentMissing {
        kind: &'static str,
        key: String,
    },
}

impl LoadError {
    pub fn message(&self) -> Message {
        let version = |v: u32| Param::Integer(i64::from(v));
        match self {
            LoadError::NotASave => Message::error(keys::SAVE_NOT_A_SAVE),
            LoadError::TooNew { found } => Message::error(keys::SAVE_TOO_NEW)
                .with("version", version(*found))
                .with("unterstuetzt", version(SAVE_FORMAT_VERSION)),
            LoadError::TooOld { found } => {
                Message::error(keys::SAVE_TOO_OLD).with("version", version(*found))
            }
            LoadError::Corrupt(detail) => {
                Message::error(keys::SAVE_CORRUPT).with("detail", Param::Text(detail.clone()))
            }
            LoadError::ContentMissing { kind, key } => Message::error(keys::SAVE_CONTENT_MISSING)
                .with("art", Param::TextKey(format!("art.{kind}")))
                .with("id", Param::Text(key.clone())),
        }
    }
}

#[derive(Debug)]
pub struct LoadedGame {
    pub game: Game,
    pub header: SaveHeader,
    /// The data files differ from those the game was saved with.
    pub data_changed: bool,
}

pub fn encode(game: &Game) -> Vec<u8> {
    let state = game.state();
    let header = SaveHeader {
        format_version: SAVE_FORMAT_VERSION,
        game_version: crate::VERSION.to_owned(),
        data_version: game.catalog().data_version,
        date: state.date,
        company_name: state
            .company(state.player)
            .map(|c| c.name.clone())
            .unwrap_or_default(),
    };
    let header = serde_json::to_vec(&header).expect("header is serializable");
    let body = SaveBody {
        catalog_keys: CatalogKeys::of(game.catalog()),
        state: state.clone(),
        journal: game.journal().to_vec(),
    };
    let body = rmp_serde::to_vec_named(&body).expect("state is serializable");

    let mut out = Vec::with_capacity(body.len() / 4 + header.len() + 16);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(
        &u32::try_from(header.len())
            .expect("small header")
            .to_le_bytes(),
    );
    out.extend_from_slice(&header);
    let mut encoder = DeflateEncoder::new(out, Compression::default());
    encoder.write_all(&body).expect("writing to memory");
    encoder.finish().expect("writing to memory")
}

/// Reads only the header, e.g. for a list of saves.
pub fn read_header(bytes: &[u8]) -> Result<SaveHeader, LoadError> {
    split(bytes).map(|(header, _)| header)
}

fn split(bytes: &[u8]) -> Result<(SaveHeader, &[u8]), LoadError> {
    let rest = bytes
        .strip_prefix(MAGIC.as_slice())
        .ok_or(LoadError::NotASave)?;
    let (length, rest) = rest.split_first_chunk::<4>().ok_or(LoadError::NotASave)?;
    let length = usize::try_from(u32::from_le_bytes(*length)).map_err(|_| LoadError::NotASave)?;
    if rest.len() < length {
        return Err(LoadError::Corrupt("header truncated".into()));
    }
    let (header, body) = rest.split_at(length);
    let header: SaveHeader =
        serde_json::from_slice(header).map_err(|e| LoadError::Corrupt(e.to_string()))?;
    Ok((header, body))
}

pub fn decode(bytes: &[u8], catalog: Arc<Catalog>) -> Result<LoadedGame, LoadError> {
    let (header, compressed) = split(bytes)?;
    if header.format_version > SAVE_FORMAT_VERSION {
        return Err(LoadError::TooNew {
            found: header.format_version,
        });
    }
    if header.format_version == 0 {
        return Err(LoadError::TooOld {
            found: header.format_version,
        });
    }
    let mut body = Vec::new();
    DeflateDecoder::new(compressed)
        .read_to_end(&mut body)
        .map_err(|e| LoadError::Corrupt(e.to_string()))?;
    let body = migrate(header.format_version, body)?;
    let SaveBody {
        catalog_keys,
        mut state,
        journal,
    } = rmp_serde::from_slice(&body).map_err(|e| LoadError::Corrupt(e.to_string()))?;

    let current = CatalogKeys::of(&catalog);
    let data_changed = header.data_version != catalog.data_version || catalog_keys != current;
    if catalog_keys != current {
        let countries = translate::<CountryId>(
            &catalog_keys.countries,
            |key| catalog.countries.id(key),
            "land",
        )?;
        state.remap(&countries, &catalog);
    }
    check_consistency(&state, &catalog)?;
    let game = Game::from_parts(catalog, state, journal);
    Ok(LoadedGame {
        game,
        header,
        data_changed,
    })
}

/// New ID for every saved key; fails if a key no longer exists.
fn translate<I: Id>(
    saved: &[String],
    lookup: impl Fn(&str) -> Option<I>,
    kind: &'static str,
) -> Result<Vec<I>, LoadError> {
    saved
        .iter()
        .map(|key| {
            lookup(key).ok_or_else(|| LoadError::ContentMissing {
                kind,
                key: key.clone(),
            })
        })
        .collect()
}

fn migrate(from: u32, body: Vec<u8>) -> Result<Vec<u8>, LoadError> {
    let first = usize::try_from(from - 1).expect("small");
    let steps = MIGRATIONS.get(first..).unwrap_or_default();
    if steps.is_empty() {
        return Ok(body);
    }
    let corrupt = |e: &dyn std::fmt::Display| LoadError::Corrupt(e.to_string());
    let mut value = rmpv::decode::read_value(&mut body.as_slice()).map_err(|e| corrupt(&e))?;
    for step in steps {
        step(&mut value).map_err(|e| corrupt(&e))?;
    }
    let mut out = Vec::new();
    rmpv::encode::write_value(&mut out, &value).map_err(|e| corrupt(&e))?;
    Ok(out)
}

/// Guards against saves whose content does not fit the catalog (damaged files).
fn check_consistency(state: &GameState, catalog: &Catalog) -> Result<(), LoadError> {
    let corrupt = |what: &str| Err(LoadError::Corrupt(what.to_owned()));
    let countries = catalog.countries.len();
    if state.countries.len() != countries {
        return corrupt("country count");
    }
    if state.company(state.player).is_none() {
        return corrupt("player company");
    }
    let in_range = |id: CountryId| id.index() < countries;
    if !in_range(state.settings.start_country)
        || !state.companies.iter().all(|c| in_range(c.headquarters))
    {
        return corrupt("country reference");
    }
    Ok(())
}
