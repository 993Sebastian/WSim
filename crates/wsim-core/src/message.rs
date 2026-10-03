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
    pub const COMMAND_BANKRUPT: &str = "fehler.befehl.firma_insolvent";
    pub const COMMAND_UNKNOWN_SITE: &str = "fehler.befehl.standort_unbekannt";
    pub const COMMAND_NOT_OWNER: &str = "fehler.befehl.fremder_standort";
    pub const COMMAND_UNKNOWN_SLOT: &str = "fehler.befehl.anlage_unbekannt";
    pub const COMMAND_WRONG_SITE_TYPE: &str = "fehler.befehl.falscher_standorttyp";
    pub const COMMAND_TECHNOLOGY_UNKNOWN: &str = "fehler.befehl.technologie_fehlt";
    pub const COMMAND_NOT_ENOUGH_CASH: &str = "fehler.befehl.geld_fehlt";
    pub const COMMAND_DEPOSIT_UNAVAILABLE: &str = "fehler.befehl.lagerstaette_vergeben";
    pub const COMMAND_DEPOSIT_OTHER_COUNTRY: &str = "fehler.befehl.lagerstaette_anderes_land";
    pub const COMMAND_DEPOSIT_NOT_DISCOVERED: &str = "fehler.befehl.lagerstaette_unentdeckt";
    pub const COMMAND_SITE_HAS_DEPOSIT: &str = "fehler.befehl.standort_hat_lagerstaette";
    pub const COMMAND_RECIPE_NOT_FOR_FACILITY: &str = "fehler.befehl.rezept_passt_nicht";
    pub const COMMAND_RECIPE_NEEDS_DEPOSIT: &str = "fehler.befehl.rezept_braucht_lagerstaette";
    pub const COMMAND_INVALID_SHARE: &str = "fehler.befehl.anteil_ungueltig";
    pub const COMMAND_AUTOMATION_TOO_HIGH: &str = "fehler.befehl.automatisierung_zu_hoch";
    pub const COMMAND_INVALID_QUANTITY: &str = "fehler.befehl.menge_ungueltig";
    pub const COMMAND_NOT_ENOUGH_GOODS: &str = "fehler.befehl.ware_fehlt";
    pub const COMMAND_DIFFERENT_COUNTRIES: &str = "fehler.befehl.anderes_land";
    pub const COMMAND_INVALID_AMOUNT: &str = "fehler.befehl.betrag_ungueltig";
    pub const COMMAND_LOAN_TOO_LARGE: &str = "fehler.befehl.kredit_zu_hoch";
    pub const COMMAND_INVALID_TERM: &str = "fehler.befehl.laufzeit_ungueltig";
    pub const COMMAND_UNKNOWN_LOAN: &str = "fehler.befehl.kredit_unbekannt";
    pub const COMMAND_INVALID_PRICE: &str = "fehler.befehl.preis_ungueltig";
    pub const GAME_OVER_INSOLVENT: &str = "meldung.spielende_insolvenz";
    pub const COMPANY_INSOLVENT: &str = "meldung.firma_insolvent";
    pub const OVERDRAFT: &str = "warnung.konto_ueberzogen";
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
        COMMAND_BANKRUPT,
        COMMAND_UNKNOWN_SITE,
        COMMAND_NOT_OWNER,
        COMMAND_UNKNOWN_SLOT,
        COMMAND_WRONG_SITE_TYPE,
        COMMAND_TECHNOLOGY_UNKNOWN,
        COMMAND_NOT_ENOUGH_CASH,
        COMMAND_DEPOSIT_UNAVAILABLE,
        COMMAND_DEPOSIT_OTHER_COUNTRY,
        COMMAND_DEPOSIT_NOT_DISCOVERED,
        COMMAND_SITE_HAS_DEPOSIT,
        COMMAND_RECIPE_NOT_FOR_FACILITY,
        COMMAND_RECIPE_NEEDS_DEPOSIT,
        COMMAND_INVALID_SHARE,
        COMMAND_AUTOMATION_TOO_HIGH,
        COMMAND_INVALID_QUANTITY,
        COMMAND_NOT_ENOUGH_GOODS,
        COMMAND_DIFFERENT_COUNTRIES,
        COMMAND_INVALID_AMOUNT,
        COMMAND_LOAN_TOO_LARGE,
        COMMAND_INVALID_TERM,
        COMMAND_UNKNOWN_LOAN,
        COMMAND_INVALID_PRICE,
        GAME_OVER_INSOLVENT,
        COMPANY_INSOLVENT,
        OVERDRAFT,
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
