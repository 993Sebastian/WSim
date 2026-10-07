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
    /// Keys of several countries, shown as a list of names.
    Countries(Vec<String>),
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
    pub const COMMAND_INVALID_STRATEGY: &str = "fehler.befehl.vorgabe_ungueltig";
    pub const COMMAND_INVALID_MANDATE: &str = "fehler.befehl.auftrag_ungueltig";
    pub const COMMAND_AUTOMATION_TOO_HIGH: &str = "fehler.befehl.automatisierung_zu_hoch";
    pub const COMMAND_INVALID_QUANTITY: &str = "fehler.befehl.menge_ungueltig";
    pub const COMMAND_NOT_ENOUGH_GOODS: &str = "fehler.befehl.ware_fehlt";
    pub const COMMAND_DIFFERENT_COUNTRIES: &str = "fehler.befehl.anderes_land";
    pub const COMMAND_INVALID_AMOUNT: &str = "fehler.befehl.betrag_ungueltig";
    pub const COMMAND_LOAN_TOO_LARGE: &str = "fehler.befehl.kredit_zu_hoch";
    pub const COMMAND_INVALID_TERM: &str = "fehler.befehl.laufzeit_ungueltig";
    pub const COMMAND_UNKNOWN_LOAN: &str = "fehler.befehl.kredit_unbekannt";
    pub const COMMAND_INVALID_PRICE: &str = "fehler.befehl.preis_ungueltig";
    pub const COMMAND_INVALID_WAGE_PREMIUM: &str = "fehler.befehl.lohnaufschlag_ungueltig";
    pub const COMMAND_NO_OFFER: &str = "fehler.befehl.kein_angebot";
    pub const COMMAND_NO_ROUTE: &str = "fehler.befehl.keine_route";
    pub const COMMAND_UNDER_CONSTRUCTION: &str = "fehler.befehl.anlage_im_bau";
    pub const COMMAND_ALREADY_MOTHBALLED: &str = "fehler.befehl.schon_stillgelegt";
    pub const COMMAND_NOT_MOTHBALLED: &str = "fehler.befehl.nicht_stillgelegt";
    pub const COMMAND_TOO_MANY_UNITS: &str = "fehler.befehl.zu_viele_einheiten";
    pub const COMMAND_NO_FREE_PLOT: &str = "fehler.befehl.kein_grundstueck";
    pub const COMMAND_UNKNOWN_PLOT: &str = "fehler.befehl.grundstueck_unbekannt";
    pub const COMMAND_PLOT_TAKEN: &str = "fehler.befehl.grundstueck_belegt";
    pub const COMMAND_PLOT_NOT_NEEDED: &str = "fehler.befehl.grundstueck_unnoetig";
    pub const COMMAND_PLOT_TOO_SMALL: &str = "fehler.befehl.grundstueck_zu_klein";
    pub const COMMAND_PLOT_OWNED: &str = "fehler.befehl.grundstueck_gekauft";
    pub const COMMAND_BELOW_MINIMUM_BID: &str = "fehler.befehl.unter_mindestgebot";
    pub const GAME_OVER_INSOLVENT: &str = "meldung.spielende_insolvenz";
    pub const COMPANY_INSOLVENT: &str = "meldung.firma_insolvent";
    pub const COMPANY_INSOLVENT_AUCTION: &str = "meldung.firma_insolvent_versteigerung";
    pub const AUCTION_WON: &str = "meldung.versteigerung.ersteigert";
    pub const AUCTION_LOST: &str = "meldung.versteigerung.verloren";
    pub const AUCTION_SOLD: &str = "meldung.versteigerung.verkauft";
    pub const DEVELOPMENT_DONE: &str = "meldung.weiterentwicklung.erreicht";
    pub const DEVELOPMENT_TOP: &str = "meldung.weiterentwicklung.ausgereizt";
    pub const DEVELOPMENT_RIVAL: &str = "meldung.weiterentwicklung.wettbewerber";
    pub const DEVELOPMENT_SUCCESSOR: &str = "meldung.weiterentwicklung.nachfolger";
    pub const OVERDRAFT: &str = "warnung.konto_ueberzogen";
    pub const NAME_EMPTY: &str = "fehler.name.leer";
    pub const NAME_TOO_LONG: &str = "fehler.name.zu_lang";
    pub const NAME_TAKEN: &str = "fehler.name.vergeben";
    pub const NAME_EXCLUDED: &str = "fehler.name.echter_name";
    pub const COMMAND_NOT_NAMEABLE: &str = "fehler.befehl.ohne_produktnamen";
    pub const AI_NEW_SELLER_NAMED: &str = "meldung.ki.anbieter_neu_name";
    pub const AI_SELLER_GONE_NAMED: &str = "meldung.ki.anbieter_weg_name";
    pub const AI_PRICE_CUT_NAMED: &str = "meldung.ki.preissenkung_name";
    pub const SAVE_NOT_A_SAVE: &str = "fehler.spielstand.kein_spielstand";
    pub const SAVE_TOO_NEW: &str = "fehler.spielstand.zu_neu";
    pub const SAVE_TOO_OLD: &str = "fehler.spielstand.zu_alt";
    pub const SAVE_CORRUPT: &str = "fehler.spielstand.beschaedigt";
    pub const SAVE_CONTENT_MISSING: &str = "fehler.spielstand.inhalt_fehlt";
    pub const GAME_END: &str = "meldung.spielende";
    pub const NEW_YEAR: &str = "meldung.neues_jahr";
    pub const RESEARCH_DONE: &str = "meldung.forschung_abgeschlossen";
    pub const COMMAND_NOT_RESEARCHABLE: &str = "fehler.befehl.nicht_erforschbar";
    pub const COMMAND_NOT_DEVELOPABLE: &str = "fehler.befehl.nicht_weiterentwickelbar";
    pub const NEW_GAME_RESEARCH_FACTOR: &str = "fehler.spielstart.forschungsfaktor";
    pub const NEW_GAME_TOO_MANY_COMPANIES: &str = "fehler.spielstart.zu_viele_firmen";
    pub const NEW_GAME_START_FORM: &str = "fehler.spielstart.startform";
    pub const WORLD_EVENT: &str = "meldung.weltereignis";
    pub const AI_FOUNDED: &str = "meldung.ki.gruendung";
    pub const AI_EXPANDS: &str = "meldung.ki.ausbau";
    pub const AI_INVENTION: &str = "meldung.ki.erfindung";
    pub const AI_MOTHBALLS: &str = "meldung.ki.stilllegung";
    pub const AI_SELLS: &str = "meldung.ki.verkauf";
    pub const INPUT_MISSING: &str = "warnung.vorprodukt_fehlt";
    pub const MILESTONE: &str = "meldung.etappe";
    pub const AI_NEW_SELLER: &str = "meldung.ki.anbieter_neu";
    pub const AI_SELLER_GONE: &str = "meldung.ki.anbieter_weg";
    pub const AI_PRICE_CUT: &str = "meldung.ki.preissenkung";
    pub const CURRENCY_REFORM: &str = "meldung.waehrungsreform";
    pub const CURRENCY_REFORM_TITLE: &str = "meldung.waehrungsreform.titel";
    pub const EVENT_KIND_CURRENCY: &str = "ereignisart.waehrung";
    pub const RANK_YEAR_END: &str = "meldung.rang";
    pub const RANK_YEAR_END_COMPARED: &str = "meldung.rang_vorjahr";
    pub const RANK_BY_EQUITY: &str = "rang.eigenkapital";
    pub const RANK_BY_REVENUE: &str = "rang.umsatz";
    pub const COMMAND_OWN_OBJECT: &str = "fehler.befehl.eigenes_angebot";
    pub const COMMAND_SELLER_BANKRUPT: &str = "fehler.befehl.verkaeufer_insolvent";
    pub const COMMAND_NOT_SELLERS_OBJECT: &str = "fehler.befehl.nicht_im_besitz";
    pub const COMMAND_SITE_TOO_YOUNG: &str = "fehler.befehl.standort_zu_jung";
    pub const COMMAND_LICENSE_NOT_POSSIBLE: &str = "fehler.befehl.lizenz_unmoeglich";
    pub const COMMAND_OFFER_EXISTS: &str = "fehler.befehl.angebot_offen";
    pub const COMMAND_OFFER_BLOCKED: &str = "fehler.befehl.angebot_gesperrt";
    pub const COMMAND_UNKNOWN_OFFER: &str = "fehler.befehl.angebot_unbekannt";
    pub const COMMAND_NOT_YOUR_TURN: &str = "fehler.befehl.nicht_am_zug";
    pub const COMMAND_NO_COUNTER: &str = "fehler.befehl.kein_gegenangebot";
    pub const COMMAND_BUYER_CANNOT_PAY: &str = "fehler.befehl.kaeufer_zahlt_nicht";
    pub const COMMAND_UNKNOWN_MANAGER: &str = "fehler.befehl.manager_unbekannt";
    pub const COMMAND_MANAGER_EMPLOYED: &str = "fehler.befehl.manager_angestellt";
    pub const COMMAND_NOT_YOUR_MANAGER: &str = "fehler.befehl.manager_nicht_bei_dir";
    pub const COMMAND_UNKNOWN_POSITION: &str = "fehler.befehl.stelle_unbekannt";
    pub const COMMAND_POSITION_TAKEN: &str = "fehler.befehl.stelle_besetzt";
    pub const COMMAND_UNKNOWN_CONCERN: &str = "fehler.befehl.anliegen_unbekannt";
    pub const COMMAND_CONCERN_CLOSED: &str = "fehler.befehl.anliegen_erledigt";
    pub const COMMAND_UNKNOWN_OPTION: &str = "fehler.befehl.option_unbekannt";
    pub const COMMAND_NOT_A_HEAD: &str = "fehler.befehl.keine_leitung";
    pub const COMMAND_SALARY_NOT_HIGHER: &str = "fehler.befehl.gehalt_nicht_hoeher";
    pub const COMMAND_MANAGER_FREE: &str = "fehler.befehl.manager_frei";
    pub const COMMAND_OWN_MANAGER: &str = "fehler.befehl.eigener_manager";
    pub const COMMAND_MANAGER_HAS_OFFER: &str = "fehler.befehl.manager_hat_angebot";
    pub const COMMAND_MANAGER_COURTED: &str = "fehler.befehl.manager_umworben";
    pub const COMMAND_NO_POACH_OFFER: &str = "fehler.befehl.kein_abwerbeangebot";
    pub const COMMAND_UNKNOWN_COUNTRY: &str = "fehler.befehl.land_unbekannt";
    pub const COMMAND_SAME_HEADQUARTERS: &str = "fehler.befehl.hauptsitz_gleich";
    pub const COMMAND_RELOCATION_UNDER_WAY: &str = "fehler.befehl.umzug_laeuft";
    pub const COMMAND_UNKNOWN_DEPARTMENT: &str = "fehler.befehl.abteilung_unbekannt";
    pub const COMMAND_INVALID_PARTICIPATIONS: &str = "fehler.befehl.beteiligungen_ungueltig";
    pub const COMMAND_NO_ADVANTAGE: &str = "fehler.befehl.kein_vorteil";
    pub const COMMAND_UNKNOWN_VENTURE: &str = "fehler.befehl.startup_unbekannt";
    pub const COMMAND_VENTURE_CLOSED: &str = "fehler.befehl.startup_beendet";
    pub const COMMAND_VENTURE_OF_OTHER: &str = "fehler.befehl.startup_tochter";
    pub const COMMAND_AMOUNT_TOO_HIGH: &str = "fehler.befehl.betrag_zu_hoch";
    pub const COMMAND_NOT_ENOUGH_SHARES: &str = "fehler.befehl.anteil_zu_klein";
    pub const COMMAND_NO_MAJORITY: &str = "fehler.befehl.keine_mehrheit";
    pub const COMMAND_VENTURE_BLOCKED: &str = "fehler.befehl.sperrminoritaet";
    pub const COMMAND_NO_SPIN_OFF: &str = "fehler.befehl.keine_ausgruendung";
    pub const COMMAND_SPIN_OFF_TOO_EARLY: &str = "fehler.befehl.ausgruendung_zu_frueh";
    pub const COMMAND_NO_STAKE_OFFER: &str = "fehler.befehl.kein_verkaufsangebot";
    // Headquarters and central departments (ZA1–ZA3).
    pub const HEADQUARTERS_MOVED: &str = "meldung.hauptsitz.umgezogen";
    pub const HEADQUARTERS_STAFF_LEFT: &str = "meldung.hauptsitz.angestellte_geblieben";
    pub const RIVAL_HEADQUARTERS: &str = "meldung.hauptsitz.konkurrenz";
    // Start-ups (SU1).
    pub const VENTURE_INVENTION: &str = "meldung.startup.erfindung";
    pub const VENTURE_DEVELOPMENT: &str = "meldung.startup.weiterentwicklung";
    pub const VENTURE_REFUND: &str = "meldung.startup.rueckzahlung";
    pub const VENTURE_LOST: &str = "meldung.startup.verloren";
    pub const VENTURE_BONUS: &str = "meldung.startup.forschungsbonus";
    pub const VENTURE_PARENT: &str = "meldung.startup.tochter";
    pub const VENTURE_PAID_OUT: &str = "meldung.startup.ausgezahlt";
    pub const VENTURE_BOUGHT_OUT: &str = "meldung.startup.uebernommen";
    pub const VENTURE_LISTED: &str = "meldung.startup.boersengang";
    pub const VENTURE_NEW_COMPANY: &str = "meldung.startup.neue_firma";
    // Spin-offs and AI companies in start-ups (SU3).
    pub const VENTURE_SPIN_OFF: &str = "meldung.startup.ausgruendung";
    pub const VENTURE_TAKEN_OVER: &str = "meldung.startup.uebernahme";
    // Stakes offered to the companies (ZA4).
    pub const VENTURE_STAKE_SOLD: &str = "meldung.startup.anteil_verkauft";
    pub const VENTURE_STAKE_UNSOLD: &str = "meldung.startup.anteil_nicht_verkauft";
    pub const VENTURE_STAKE_BEST_BID: &str = "meldung.startup.anteil_bestes_gebot";
    // The market of managers (MA6).
    pub const MANAGER_RESIGNED: &str = "meldung.manager.kuendigung";
    pub const MANAGER_HIRED_BY_HEAD: &str = "meldung.manager.eingestellt";
    pub const MANAGER_POACH: &str = "meldung.manager.abwerbung";
    pub const MANAGER_POACH_DECIDED: &str = "meldung.manager.abwerbung_entschieden";
    pub const MANAGER_LEFT: &str = "meldung.manager.gewechselt";
    pub const MANAGER_STAYED: &str = "meldung.manager.geblieben";
    pub const MANAGER_POACH_WON: &str = "meldung.manager.abgeworben";
    pub const MANAGER_POACH_KEPT: &str = "meldung.manager.gehalten";
    pub const STEP_MATCH: &str = "schritt.gegenangebot.manager";
    pub const STEP_LET_GO: &str = "schritt.gehen_lassen";
    pub const STEP_REFINANCE: &str = "schritt.umschulden";
    pub const STEP_INVEST: &str = "schritt.beteiligen";
    pub const STEP_RAISE: &str = "schritt.gehalt_erhoehen";
    pub const BECAUSE_REFINANCE: &str = "anliegen.begruendung.umschuldung";
    pub const BECAUSE_VENTURE: &str = "anliegen.begruendung.startup";
    pub const BECAUSE_SALARY_ROUND: &str = "anliegen.begruendung.gehaltsrunde";
    pub const BECAUSE_FIT: &str = "anliegen.begruendung.passung";
    pub const BECAUSE_SPREAD: &str = "anliegen.begruendung.streuung";
    pub const BECAUSE_SHORTCUT: &str = "anliegen.begruendung.abkuerzung";
    pub const BECAUSE_KEEP: &str = "anliegen.begruendung.halten";
    pub const BECAUSE_LET_GO: &str = "anliegen.begruendung.ziehen_lassen";
    pub const CONCERN_NEW: &str = "meldung.anliegen.neu";
    pub const CONCERN_EXPIRED: &str = "meldung.anliegen.verfallen";
    pub const CONCERN_NEW_COUNTRY: &str = "meldung.anliegen.neu_land";
    pub const CONCERN_NEW_CONTINENT: &str = "meldung.anliegen.neu_kontinent";
    pub const CONCERN_EXPIRED_COUNTRY: &str = "meldung.anliegen.verfallen_land";
    pub const CONCERN_EXPIRED_CONTINENT: &str = "meldung.anliegen.verfallen_kontinent";
    pub const CONCERN_NEW_BOARD: &str = "meldung.anliegen.neu_vorstand";
    pub const CONCERN_EXPIRED_BOARD: &str = "meldung.anliegen.verfallen_vorstand";
    pub const CONCERN_EFFECT: &str = "meldung.anliegen.folge";
    /// The CEO's strategy review (MA5).
    pub const REVIEW: &str = "meldung.ruecksprache";
    // What the options of a concern do, and why a position recommends one (MA2).
    pub const STEP_MOTHBALL: &str = "schritt.stilllegen";
    pub const STEP_SELL: &str = "schritt.verkaufen";
    pub const STEP_RESTART: &str = "schritt.anfahren";
    pub const STEP_BUILD: &str = "schritt.bauen";
    pub const STEP_LOAN: &str = "schritt.kredit";
    pub const STEP_REPAY: &str = "schritt.tilgen";
    pub const STEP_PRODUCTION: &str = "schritt.produktion";
    pub const STEP_PRODUCTION_OFF: &str = "schritt.produktion_aus";
    pub const STEP_AUTOMATION: &str = "schritt.automatisierung";
    pub const STEP_SALE_END: &str = "schritt.verkauf_ende";
    pub const STEP_PRICE: &str = "schritt.preis";
    pub const STEP_SALE_MARKET: &str = "schritt.verkauf_markt";
    pub const STEP_PURCHASE_END: &str = "schritt.einkauf_ende";
    pub const STEP_PURCHASE: &str = "schritt.einkauf";
    pub const STEP_SUPPLY: &str = "schritt.lieferung";
    pub const STEP_WAGE: &str = "schritt.lohn";
    pub const STEP_RESEARCH: &str = "schritt.forschung";
    pub const STEP_RESEARCH_END: &str = "schritt.forschung_ende";
    pub const STEP_DEVELOP: &str = "schritt.weiterentwicklung";
    pub const STEP_DEPOSIT: &str = "schritt.lagerstaette";
    // What the board decided about offers, and the steps of offers in concerns (MA5).
    pub const BOARD_OFFER_SITE: &str = "meldung.vorstand.angebot.standort";
    pub const BOARD_OFFER_LICENSE: &str = "meldung.vorstand.angebot.lizenz";
    pub const BOARD_OFFER_AREA: &str = "meldung.vorstand.angebot.bereich";
    pub const BOARD_SOLD_SITE: &str = "meldung.vorstand.verkauft.standort";
    pub const BOARD_SOLD_LICENSE: &str = "meldung.vorstand.verkauft.lizenz";
    pub const BOARD_SOLD_AREA: &str = "meldung.vorstand.verkauft.bereich";
    pub const BOARD_BOUGHT_SITE: &str = "meldung.vorstand.gekauft.standort";
    pub const BOARD_BOUGHT_LICENSE: &str = "meldung.vorstand.gekauft.lizenz";
    pub const BOARD_BOUGHT_AREA: &str = "meldung.vorstand.gekauft.bereich";
    pub const BOARD_DECLINED_SITE: &str = "meldung.vorstand.abgelehnt.standort";
    pub const BOARD_DECLINED_LICENSE: &str = "meldung.vorstand.abgelehnt.lizenz";
    pub const BOARD_DECLINED_AREA: &str = "meldung.vorstand.abgelehnt.bereich";
    pub const BOARD_COUNTER_SITE: &str = "meldung.vorstand.gegenangebot.standort";
    pub const BOARD_COUNTER_LICENSE: &str = "meldung.vorstand.gegenangebot.lizenz";
    pub const BOARD_COUNTER_AREA: &str = "meldung.vorstand.gegenangebot.bereich";
    pub const STEP_BID_SITE: &str = "schritt.bieten.standort";
    pub const STEP_BID_LICENSE: &str = "schritt.bieten.lizenz";
    pub const STEP_BID_AREA: &str = "schritt.bieten.bereich";
    pub const STEP_ACCEPT_SITE: &str = "schritt.annehmen.standort";
    pub const STEP_ACCEPT_LICENSE: &str = "schritt.annehmen.lizenz";
    pub const STEP_ACCEPT_AREA: &str = "schritt.annehmen.bereich";
    pub const STEP_COUNTER_SITE: &str = "schritt.gegenangebot.standort";
    pub const STEP_COUNTER_LICENSE: &str = "schritt.gegenangebot.lizenz";
    pub const STEP_COUNTER_AREA: &str = "schritt.gegenangebot.bereich";
    pub const STEP_DECLINE_SITE: &str = "schritt.ablehnen.standort";
    pub const STEP_DECLINE_LICENSE: &str = "schritt.ablehnen.lizenz";
    pub const STEP_DECLINE_AREA: &str = "schritt.ablehnen.bereich";
    pub const BECAUSE_RESULT: &str = "anliegen.begruendung.ergebnis";
    pub const BECAUSE_PROCEEDS: &str = "anliegen.begruendung.erloes";
    pub const BECAUSE_WAIT: &str = "anliegen.begruendung.abwarten";
    pub const BECAUSE_RULE: &str = "anliegen.begruendung.regel";
    pub const OFFER_RECEIVED_SITE: &str = "meldung.angebot.erhalten.standort";
    pub const OFFER_RECEIVED_LICENSE: &str = "meldung.angebot.erhalten.lizenz";
    pub const OFFER_COUNTER_SITE: &str = "meldung.angebot.gegenangebot.standort";
    pub const OFFER_COUNTER_LICENSE: &str = "meldung.angebot.gegenangebot.lizenz";
    pub const OFFER_BOUGHT_SITE: &str = "meldung.angebot.gekauft.standort";
    pub const OFFER_BOUGHT_LICENSE: &str = "meldung.angebot.gekauft.lizenz";
    pub const OFFER_SOLD_SITE: &str = "meldung.angebot.verkauft.standort";
    pub const OFFER_SOLD_LICENSE: &str = "meldung.angebot.verkauft.lizenz";
    pub const OFFER_DECLINED_SITE: &str = "meldung.angebot.abgelehnt.standort";
    pub const OFFER_DECLINED_LICENSE: &str = "meldung.angebot.abgelehnt.lizenz";
    pub const OFFER_EXPIRED_SITE: &str = "meldung.angebot.abgelaufen.standort";
    pub const OFFER_EXPIRED_LICENSE: &str = "meldung.angebot.abgelaufen.lizenz";
    pub const OFFER_RECEIVED_AREA: &str = "meldung.angebot.erhalten.bereich";
    pub const OFFER_COUNTER_AREA: &str = "meldung.angebot.gegenangebot.bereich";
    pub const OFFER_BOUGHT_AREA: &str = "meldung.angebot.gekauft.bereich";
    pub const OFFER_SOLD_AREA: &str = "meldung.angebot.verkauft.bereich";
    pub const OFFER_DECLINED_AREA: &str = "meldung.angebot.abgelehnt.bereich";
    pub const OFFER_EXPIRED_AREA: &str = "meldung.angebot.abgelaufen.bereich";
    pub const AI_BUYS_SITE: &str = "meldung.ki.kauf_standort";
    pub const AI_BUYS_AREA: &str = "meldung.ki.kauf_bereich";
    // Hints of the overview (views::hints).
    pub const HINT_INPUT: &str = "hinweis.vorprodukt_fehlt";
    pub const HINT_LABOR: &str = "hinweis.arbeitskraefte";
    pub const HINT_POWER: &str = "hinweis.strom";
    pub const HINT_DEPOSIT: &str = "hinweis.lagerstaette";
    pub const HINT_NO_RECIPE: &str = "hinweis.ohne_verfahren";
    pub const HINT_IDLE: &str = "hinweis.ruht";
    pub const HINT_NO_PURCHASE: &str = "hinweis.kein_einkauf";
    pub const HINT_NO_OFFER: &str = "hinweis.kein_verkauf";
    pub const HINT_BELOW_COST: &str = "hinweis.unter_stueckkosten";
    pub const HINT_UNSOLD: &str = "hinweis.nichts_verkauft";
    pub const HINT_STAFF: &str = "hinweis.personal";
    pub const HINT_NO_RESEARCH: &str = "hinweis.forschung_ohne_projekt";
    pub const HINT_NO_PRODUCT_NAME: &str = "hinweis.produkt_ohne_namen";
    pub const HINT_NO_MANAGERS: &str = "hinweis.ohne_manager";
    pub const HINT_CONCERNS: &str = "hinweis.anliegen";
    pub const HINT_OVERDRAWN: &str = "hinweis.konto_ueberzogen";
    pub const HINT_CASH: &str = "hinweis.kasse";
    pub const HINT_OFFER_SITE: &str = "hinweis.angebot.standort";
    pub const HINT_OFFER_LICENSE: &str = "hinweis.angebot.lizenz";
    pub const HINT_COUNTER_SITE: &str = "hinweis.gegenangebot.standort";
    pub const HINT_COUNTER_LICENSE: &str = "hinweis.gegenangebot.lizenz";
    pub const HINT_OFFER_AREA: &str = "hinweis.angebot.bereich";
    pub const HINT_COUNTER_AREA: &str = "hinweis.gegenangebot.bereich";

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
        COMMAND_INVALID_STRATEGY,
        COMMAND_INVALID_MANDATE,
        COMMAND_AUTOMATION_TOO_HIGH,
        COMMAND_INVALID_QUANTITY,
        COMMAND_NOT_ENOUGH_GOODS,
        COMMAND_DIFFERENT_COUNTRIES,
        COMMAND_INVALID_AMOUNT,
        COMMAND_LOAN_TOO_LARGE,
        COMMAND_INVALID_TERM,
        COMMAND_UNKNOWN_LOAN,
        COMMAND_INVALID_PRICE,
        COMMAND_INVALID_WAGE_PREMIUM,
        COMMAND_NO_OFFER,
        COMMAND_NO_ROUTE,
        COMMAND_UNDER_CONSTRUCTION,
        COMMAND_ALREADY_MOTHBALLED,
        COMMAND_NOT_MOTHBALLED,
        COMMAND_TOO_MANY_UNITS,
        COMMAND_NO_FREE_PLOT,
        COMMAND_UNKNOWN_PLOT,
        COMMAND_PLOT_TAKEN,
        COMMAND_PLOT_NOT_NEEDED,
        COMMAND_PLOT_TOO_SMALL,
        COMMAND_PLOT_OWNED,
        COMMAND_BELOW_MINIMUM_BID,
        GAME_OVER_INSOLVENT,
        COMPANY_INSOLVENT,
        COMPANY_INSOLVENT_AUCTION,
        AUCTION_WON,
        AUCTION_LOST,
        AUCTION_SOLD,
        DEVELOPMENT_DONE,
        DEVELOPMENT_TOP,
        DEVELOPMENT_RIVAL,
        DEVELOPMENT_SUCCESSOR,
        OVERDRAFT,
        NAME_EMPTY,
        NAME_TOO_LONG,
        NAME_TAKEN,
        NAME_EXCLUDED,
        COMMAND_NOT_NAMEABLE,
        AI_NEW_SELLER_NAMED,
        AI_SELLER_GONE_NAMED,
        AI_PRICE_CUT_NAMED,
        SAVE_NOT_A_SAVE,
        SAVE_TOO_NEW,
        SAVE_TOO_OLD,
        SAVE_CORRUPT,
        SAVE_CONTENT_MISSING,
        GAME_END,
        NEW_YEAR,
        RESEARCH_DONE,
        COMMAND_NOT_RESEARCHABLE,
        COMMAND_NOT_DEVELOPABLE,
        NEW_GAME_RESEARCH_FACTOR,
        NEW_GAME_TOO_MANY_COMPANIES,
        NEW_GAME_START_FORM,
        WORLD_EVENT,
        AI_FOUNDED,
        AI_EXPANDS,
        AI_INVENTION,
        MILESTONE,
        AI_NEW_SELLER,
        AI_SELLER_GONE,
        AI_PRICE_CUT,
        CURRENCY_REFORM,
        CURRENCY_REFORM_TITLE,
        EVENT_KIND_CURRENCY,
        RANK_YEAR_END,
        RANK_YEAR_END_COMPARED,
        RANK_BY_EQUITY,
        RANK_BY_REVENUE,
        COMMAND_OWN_OBJECT,
        COMMAND_SELLER_BANKRUPT,
        COMMAND_NOT_SELLERS_OBJECT,
        COMMAND_SITE_TOO_YOUNG,
        COMMAND_LICENSE_NOT_POSSIBLE,
        COMMAND_OFFER_EXISTS,
        COMMAND_OFFER_BLOCKED,
        COMMAND_UNKNOWN_OFFER,
        COMMAND_NOT_YOUR_TURN,
        COMMAND_NO_COUNTER,
        COMMAND_BUYER_CANNOT_PAY,
        COMMAND_UNKNOWN_MANAGER,
        COMMAND_MANAGER_EMPLOYED,
        COMMAND_NOT_YOUR_MANAGER,
        COMMAND_UNKNOWN_POSITION,
        COMMAND_POSITION_TAKEN,
        COMMAND_UNKNOWN_CONCERN,
        COMMAND_CONCERN_CLOSED,
        COMMAND_UNKNOWN_OPTION,
        COMMAND_NOT_A_HEAD,
        COMMAND_SALARY_NOT_HIGHER,
        COMMAND_MANAGER_FREE,
        COMMAND_OWN_MANAGER,
        COMMAND_MANAGER_HAS_OFFER,
        COMMAND_MANAGER_COURTED,
        COMMAND_NO_POACH_OFFER,
        COMMAND_UNKNOWN_COUNTRY,
        COMMAND_SAME_HEADQUARTERS,
        COMMAND_RELOCATION_UNDER_WAY,
        COMMAND_UNKNOWN_DEPARTMENT,
        COMMAND_INVALID_PARTICIPATIONS,
        COMMAND_NO_ADVANTAGE,
        COMMAND_UNKNOWN_VENTURE,
        COMMAND_VENTURE_CLOSED,
        COMMAND_VENTURE_OF_OTHER,
        COMMAND_AMOUNT_TOO_HIGH,
        COMMAND_NOT_ENOUGH_SHARES,
        COMMAND_NO_MAJORITY,
        COMMAND_VENTURE_BLOCKED,
        COMMAND_NO_SPIN_OFF,
        COMMAND_SPIN_OFF_TOO_EARLY,
        COMMAND_NO_STAKE_OFFER,
        HEADQUARTERS_MOVED,
        HEADQUARTERS_STAFF_LEFT,
        RIVAL_HEADQUARTERS,
        VENTURE_INVENTION,
        VENTURE_DEVELOPMENT,
        VENTURE_REFUND,
        VENTURE_LOST,
        VENTURE_BONUS,
        VENTURE_PARENT,
        VENTURE_PAID_OUT,
        VENTURE_BOUGHT_OUT,
        VENTURE_LISTED,
        VENTURE_NEW_COMPANY,
        VENTURE_SPIN_OFF,
        VENTURE_TAKEN_OVER,
        VENTURE_STAKE_SOLD,
        VENTURE_STAKE_UNSOLD,
        VENTURE_STAKE_BEST_BID,
        MANAGER_RESIGNED,
        MANAGER_HIRED_BY_HEAD,
        MANAGER_POACH,
        MANAGER_POACH_DECIDED,
        MANAGER_LEFT,
        MANAGER_STAYED,
        MANAGER_POACH_WON,
        MANAGER_POACH_KEPT,
        STEP_MATCH,
        STEP_LET_GO,
        STEP_REFINANCE,
        STEP_INVEST,
        STEP_RAISE,
        BECAUSE_REFINANCE,
        BECAUSE_VENTURE,
        BECAUSE_SALARY_ROUND,
        BECAUSE_FIT,
        BECAUSE_SPREAD,
        BECAUSE_SHORTCUT,
        BECAUSE_KEEP,
        BECAUSE_LET_GO,
        CONCERN_NEW,
        CONCERN_EXPIRED,
        CONCERN_NEW_COUNTRY,
        CONCERN_NEW_CONTINENT,
        CONCERN_NEW_BOARD,
        CONCERN_EXPIRED_BOARD,
        CONCERN_EXPIRED_COUNTRY,
        CONCERN_EXPIRED_CONTINENT,
        CONCERN_EFFECT,
        REVIEW,
        STEP_MOTHBALL,
        STEP_SELL,
        STEP_RESTART,
        STEP_BUILD,
        STEP_LOAN,
        STEP_REPAY,
        STEP_PRODUCTION,
        STEP_PRODUCTION_OFF,
        STEP_AUTOMATION,
        STEP_SALE_END,
        STEP_PRICE,
        STEP_SALE_MARKET,
        STEP_PURCHASE_END,
        STEP_PURCHASE,
        STEP_SUPPLY,
        STEP_WAGE,
        STEP_RESEARCH,
        STEP_RESEARCH_END,
        STEP_DEVELOP,
        STEP_DEPOSIT,
        BOARD_OFFER_SITE,
        BOARD_OFFER_LICENSE,
        BOARD_OFFER_AREA,
        BOARD_SOLD_SITE,
        BOARD_SOLD_LICENSE,
        BOARD_SOLD_AREA,
        BOARD_BOUGHT_SITE,
        BOARD_BOUGHT_LICENSE,
        BOARD_BOUGHT_AREA,
        BOARD_DECLINED_SITE,
        BOARD_DECLINED_LICENSE,
        BOARD_DECLINED_AREA,
        BOARD_COUNTER_SITE,
        BOARD_COUNTER_LICENSE,
        BOARD_COUNTER_AREA,
        STEP_BID_SITE,
        STEP_BID_LICENSE,
        STEP_BID_AREA,
        STEP_ACCEPT_SITE,
        STEP_ACCEPT_LICENSE,
        STEP_ACCEPT_AREA,
        STEP_COUNTER_SITE,
        STEP_COUNTER_LICENSE,
        STEP_COUNTER_AREA,
        STEP_DECLINE_SITE,
        STEP_DECLINE_LICENSE,
        STEP_DECLINE_AREA,
        BECAUSE_RESULT,
        BECAUSE_PROCEEDS,
        BECAUSE_WAIT,
        BECAUSE_RULE,
        OFFER_RECEIVED_SITE,
        OFFER_RECEIVED_LICENSE,
        OFFER_COUNTER_SITE,
        OFFER_COUNTER_LICENSE,
        OFFER_BOUGHT_SITE,
        OFFER_BOUGHT_LICENSE,
        OFFER_SOLD_SITE,
        OFFER_SOLD_LICENSE,
        OFFER_DECLINED_SITE,
        OFFER_DECLINED_LICENSE,
        OFFER_EXPIRED_SITE,
        OFFER_EXPIRED_LICENSE,
        OFFER_RECEIVED_AREA,
        OFFER_COUNTER_AREA,
        OFFER_BOUGHT_AREA,
        OFFER_SOLD_AREA,
        OFFER_DECLINED_AREA,
        OFFER_EXPIRED_AREA,
        AI_BUYS_SITE,
        AI_BUYS_AREA,
        AI_MOTHBALLS,
        AI_SELLS,
        INPUT_MISSING,
        HINT_INPUT,
        HINT_LABOR,
        HINT_POWER,
        HINT_DEPOSIT,
        HINT_NO_RECIPE,
        HINT_IDLE,
        HINT_NO_PURCHASE,
        HINT_NO_OFFER,
        HINT_BELOW_COST,
        HINT_UNSOLD,
        HINT_STAFF,
        HINT_NO_RESEARCH,
        HINT_NO_PRODUCT_NAME,
        HINT_NO_MANAGERS,
        HINT_CONCERNS,
        HINT_OVERDRAWN,
        HINT_CASH,
        HINT_OFFER_SITE,
        HINT_OFFER_LICENSE,
        HINT_COUNTER_SITE,
        HINT_COUNTER_LICENSE,
        HINT_OFFER_AREA,
        HINT_COUNTER_AREA,
    ];
}
