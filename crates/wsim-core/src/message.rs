//! Messages for the player. The core never builds display text: a message is a text
//! key plus parameters, rendered with the texts in `data/texte/` (Lastenheft §16.4).

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::money::Money;

/// Kind of a message; decides colour and sound (Lastenheft §13.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageKind {
    Info,
    Success,
    Warning,
    Crisis,
    WorldEvent,
    Stock,
    /// Rejected command or invalid input.
    Error,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Param {
    Text(String),
    Integer(i64),
    Number(f64),
    Money(Money),
    Date(Date),
    /// Key of a country, shown with its name.
    Country(String),
    /// Another text, looked up by its key.
    TextKey(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub kind: MessageKind,
    pub key: String,
    pub params: Vec<(String, Param)>,
}

impl Message {
    pub fn new(kind: MessageKind, key: &str) -> Self {
        Self {
            kind,
            key: key.to_owned(),
            params: Vec::new(),
        }
    }

    pub fn error(key: &str) -> Self {
        Self::new(MessageKind::Error, key)
    }

    #[must_use]
    pub fn with(mut self, name: &str, value: Param) -> Self {
        self.params.push((name.to_owned(), value));
        self
    }
}

/// All text keys the core uses. Each must exist in `data/texte/<sprache>/`; a test in
/// `wsim-data` checks this.
pub mod keys {
    pub const NEW_GAME_START_YEAR: &str = "fehler.spielstart.startjahr";
    pub const NEW_GAME_START_CAPITAL: &str = "fehler.spielstart.startkapital";
    pub const NEW_GAME_START_COUNTRY: &str = "fehler.spielstart.startland";
    pub const COMMAND_GAME_OVER: &str = "fehler.befehl.spielende";
    pub const COMMAND_UNKNOWN_COMPANY: &str = "fehler.befehl.firma_unbekannt";
    pub const NAME_EMPTY: &str = "fehler.name.leer";
    pub const NAME_TOO_LONG: &str = "fehler.name.zu_lang";
    pub const NAME_TAKEN: &str = "fehler.name.vergeben";
    pub const SAVE_NOT_A_SAVE: &str = "fehler.spielstand.kein_spielstand";
    pub const SAVE_TOO_NEW: &str = "fehler.spielstand.zu_neu";
    pub const SAVE_TOO_OLD: &str = "fehler.spielstand.zu_alt";
    pub const SAVE_CORRUPT: &str = "fehler.spielstand.beschaedigt";
    pub const SAVE_CONTENT_MISSING: &str = "fehler.spielstand.inhalt_fehlt";
    pub const GAME_END: &str = "meldung.spielende";
    pub const NEW_YEAR: &str = "meldung.neues_jahr";

    pub const ALL: &[&str] = &[
        NEW_GAME_START_YEAR,
        NEW_GAME_START_CAPITAL,
        NEW_GAME_START_COUNTRY,
        COMMAND_GAME_OVER,
        COMMAND_UNKNOWN_COMPANY,
        NAME_EMPTY,
        NAME_TOO_LONG,
        NAME_TAKEN,
        SAVE_NOT_A_SAVE,
        SAVE_TOO_NEW,
        SAVE_TOO_OLD,
        SAVE_CORRUPT,
        SAVE_CONTENT_MISSING,
        GAME_END,
        NEW_YEAR,
    ];
}
