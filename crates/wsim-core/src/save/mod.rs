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
//! References to catalog entries are written as keys (`DEU`, `roheisen`), so a save
//! stays loadable when entries are added or reordered in the data files. Older format
//! versions are read with their own types (`legacy`) and converted.
//! The core does no file IO; callers read and write the bytes.

mod legacy;

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::sync::Arc;

use flate2::Compression;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::game::{Game, JournalEntry};
use crate::ids::{self, IdKind};
use crate::message::{Message, Param, keys};
use crate::state::GameState;

/// Current save format. Raise it on every incompatible change of `GameState`, read the
/// old format in `legacy` and keep a save of the old version under
/// `tests/fixtures/saves/`.
pub const SAVE_FORMAT_VERSION: u32 = 3;

const MAGIC: &[u8; 8] = b"WSIMSAVE";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SaveHeader {
    pub format_version: u32,
    pub game_version: String,
    pub data_version: u32,
    pub date: Date,
    pub company_name: String,
}

/// Keys of all catalog entries by kind at the time of saving (to detect changed data).
type CatalogKeys = BTreeMap<String, Vec<String>>;

fn catalog_keys(catalog: &Catalog) -> CatalogKeys {
    let table = catalog.key_table();
    IdKind::ALL
        .iter()
        .map(|&kind| (kind.name().to_owned(), table.keys(kind).to_vec()))
        .collect()
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
    let catalog = game.catalog();
    let state = game.state();
    let header = SaveHeader {
        format_version: SAVE_FORMAT_VERSION,
        game_version: crate::VERSION.to_owned(),
        data_version: catalog.data_version,
        date: state.date,
        company_name: state
            .company(state.player)
            .map(|c| c.name.clone())
            .unwrap_or_default(),
    };
    let header = serde_json::to_vec(&header).expect("header is serializable");
    let body = SaveBody {
        catalog_keys: catalog_keys(catalog),
        state: state.clone(),
        journal: game.journal().to_vec(),
    };
    let (body, _) = ids::with_keys(&catalog.key_table(), || rmp_serde::to_vec_named(&body));
    let body = body.expect("state is serializable");

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
    let mut body = Vec::new();
    DeflateDecoder::new(compressed)
        .read_to_end(&mut body)
        .map_err(|e| LoadError::Corrupt(e.to_string()))?;

    let (saved_keys, mut state, journal) = match header.format_version {
        0 => return Err(LoadError::TooOld { found: 0 }),
        1 | 2 => legacy::decode_v2(&body, &catalog)?,
        _ => {
            let (result, missing) = ids::with_keys(&catalog.key_table(), || {
                rmp_serde::from_slice::<SaveBody>(&body)
            });
            if let Some((kind, key)) = missing {
                return Err(LoadError::ContentMissing {
                    kind: kind.name(),
                    key,
                });
            }
            let body = result.map_err(|e| LoadError::Corrupt(e.to_string()))?;
            (body.catalog_keys, body.state, body.journal)
        }
    };
    let data_changed =
        header.data_version != catalog.data_version || saved_keys != catalog_keys(&catalog);
    check_consistency(&state, &catalog)?;
    state.fit_to_catalog(&catalog);
    let game = Game::from_parts(catalog, state, journal);
    Ok(LoadedGame {
        game,
        header,
        data_changed,
    })
}

/// Guards against saves whose content does not fit together (damaged files).
fn check_consistency(state: &GameState, catalog: &Catalog) -> Result<(), LoadError> {
    let corrupt = |what: &str| Err(LoadError::Corrupt(what.to_owned()));
    if state.company(state.player).is_none() {
        return corrupt("player company");
    }
    let countries = catalog.countries.len();
    let in_range = |id: crate::ids::CountryId| crate::ids::Id::index(id) < countries;
    if !in_range(state.settings.start_country)
        || !state.companies.iter().all(|c| in_range(c.headquarters))
        || !state.sites.iter().all(|s| in_range(s.country))
    {
        return corrupt("country reference");
    }
    let companies = state.companies.len();
    if state.sites.iter().any(|s| s.owner.index() >= companies) {
        return corrupt("site owner");
    }
    let sites = state.sites.len();
    if state
        .deposits
        .values()
        .any(|d| d.site.is_some_and(|s| s.index() >= sites))
    {
        return corrupt("deposit site");
    }
    Ok(())
}
