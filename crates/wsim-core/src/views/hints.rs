//! What the player should look at now (docs/BEDIENUNG.md, overview "Zu erledigen"):
//! stalled facilities, missing purchases and offers, sales below cost, cash. Derived
//! from the same views the player sees; display hints only, the simulation does not
//! use them.

use serde::{Deserialize, Serialize};

use super::{MessageView, message_view, production};
use crate::game::Game;
use crate::ledger::Ledger;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;

/// Inputs whose stock lasts less than this many days without a purchase order are
/// reported (display hint).
const STOCK_LOW_DAYS: f64 = 7.0;
/// With a loss last month, the cash running out within this many months is reported.
const CASH_WARNING_MONTHS: f64 = 12.0;

/// A hint with the place to act on it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HintView {
    pub message: MessageView,
    /// Site the hint concerns; the interface opens its plant view.
    pub site: Option<u32>,
    /// Area of the plant view: `anlagen`, `einkauf`, `verkauf`, `personal`, `kosten`.
    pub area: Option<String>,
}

fn hint(message: Message, site: Option<u32>, area: Option<&str>) -> HintView {
    let mut view = message_view(&message);
    view.target = Some(
        if site.is_some() {
            "produktion"
        } else {
            "finanzen"
        }
        .to_owned(),
    );
    HintView {
        message: view,
        site,
        area: area.map(str::to_owned),
    }
}

/// The hints for the player's company, most urgent first.
pub fn hints(game: &Game) -> Vec<HintView> {
    let state = game.state();
    let view = production(game);
    let mut found: Vec<(u8, HintView)> = Vec::new();
    for s in &view.sites {
        let at = |key: &'static str| {
            Message::new(MessageKind::Warning, key)
                .with("standort", Param::TextKey(s.kind_text.clone()))
                .with("land", Param::Country(s.country.clone()))
        };
        let research = s.kind == crate::catalog::SiteType::ResearchCenter;
        for sl in &s.slots {
            let anlage = Param::TextKey(format!("anlage.{}", sl.facility));
            let Some(cause) = &sl.cause else { continue };
            let (rank, kind, key, area) = match cause.key.as_str() {
                "ursache.vorprodukt" => (1, MessageKind::Warning, keys::HINT_INPUT, "einkauf"),
                "ursache.arbeitskraefte" => (1, MessageKind::Warning, keys::HINT_LABOR, "personal"),
                "ursache.strom" => (1, MessageKind::Warning, keys::HINT_POWER, "anlagen"),
                "ursache.lagerstaette" => (2, MessageKind::Warning, keys::HINT_DEPOSIT, "anlagen"),
                "ursache.kein_rezept" if !research => {
                    (3, MessageKind::Info, keys::HINT_NO_RECIPE, "anlagen")
                }
                "ursache.ruht" => (3, MessageKind::Info, keys::HINT_IDLE, "anlagen"),
                _ => continue,
            };
            let mut m = at(key).with("anlage", anlage);
            m.kind = kind;
            if let Some(product) = cause.detail.as_ref().filter(|_| area == "einkauf") {
                m = m.with("produkt", Param::TextKey(format!("produkt.{product}")));
            }
            found.push((rank, hint(m, Some(s.index), Some(area))));
        }
        for input in s.inputs.iter().filter(|v| !v.own && !v.ordered) {
            let Some(days) = input.days.filter(|&d| d < STOCK_LOW_DAYS) else {
                continue;
            };
            // Already reported as a stalled facility.
            if found.iter().any(|(_, h)| {
                h.site == Some(s.index)
                    && h.message.key == keys::HINT_INPUT
                    && h.message.params.get("produkt")
                        == Some(&super::ParamView::TextKey(format!(
                            "produkt.{}",
                            input.product
                        )))
            }) {
                continue;
            }
            let m = at(keys::HINT_NO_PURCHASE)
                .with(
                    "produkt",
                    Param::TextKey(format!("produkt.{}", input.product)),
                )
                .with("tage", Param::Integer(days.floor() as i64));
            found.push((1, hint(m, Some(s.index), Some("einkauf"))));
        }
        let used: Vec<&str> = s.inputs.iter().map(|v| v.product.as_str()).collect();
        for sl in &s.slots {
            let Some(product) = &sl.product else { continue };
            if used.contains(&product.as_str())
                || s.offers.iter().any(|o| &o.product == product)
                || found
                    .iter()
                    .any(|(_, h)| h.site == Some(s.index) && h.message.key == keys::HINT_NO_OFFER)
            {
                continue;
            }
            let m = at(keys::HINT_NO_OFFER)
                .with("produkt", Param::TextKey(format!("produkt.{product}")));
            found.push((1, hint(m, Some(s.index), Some("verkauf"))));
        }
        let month_closed = !state.companies[state.player.index()]
            .ledger
            .months
            .is_empty();
        for o in &s.offers {
            let produkt = Param::TextKey(format!("produkt.{}", o.product));
            if let Some(margin) = o.margin.filter(|&m| m < 0.0) {
                let m = at(keys::HINT_BELOW_COST)
                    .with("produkt", produkt.clone())
                    .with("marge", Param::Integer((margin * 100.0).round() as i64));
                found.push((2, hint(m, Some(s.index), Some("verkauf"))));
            }
            if month_closed && o.stock > 1e-9 && o.sold_last_month <= 1e-9 && o.sold_month <= 1e-9 {
                let money = |usd: f64| Param::Money(Money::from_usd(usd).unwrap_or(Money::ZERO));
                let m = at(keys::HINT_UNSOLD)
                    .with("produkt", produkt)
                    .with("preis", money(o.price_usd))
                    .with("markt", money(o.market_price_usd));
                found.push((1, hint(m, Some(s.index), Some("verkauf"))));
            }
        }
        if s.staff.iter().any(|l| l.employed + 0.05 < l.needed)
            && !found
                .iter()
                .any(|(_, h)| h.site == Some(s.index) && h.message.key == keys::HINT_LABOR)
        {
            found.push((
                2,
                hint(at(keys::HINT_STAFF), Some(s.index), Some("personal")),
            ));
        }
        if research && s.research.is_none() && s.development.is_none() {
            let mut m = at(keys::HINT_NO_RESEARCH);
            m.kind = MessageKind::Info;
            let mut h = hint(m, Some(s.index), None);
            h.message.target = Some("forschung".to_owned());
            found.push((3, h));
        }
    }
    // Offers waiting for the player's answer expire (M30).
    for m in super::deals::offer_hints(game) {
        let mut h = hint(m, None, None);
        h.message.target = Some("wettbewerb".to_owned());
        found.push((1, h));
    }
    let ledger = &state.companies[state.player.index()].ledger;
    let cash = ledger.cash();
    if cash < Money::ZERO {
        let m = Message::new(MessageKind::Crisis, keys::HINT_OVERDRAWN)
            .with("betrag", Param::Money(-cash));
        found.push((0, hint(m, None, None)));
    } else if let Some(loss) = ledger
        .months
        .last()
        .map(|m| m.total())
        .filter(|r| *r < Money::ZERO)
    {
        let months = cash.to_usd() / (-loss).to_usd();
        if months < CASH_WARNING_MONTHS {
            let m = Message::new(MessageKind::Warning, keys::HINT_CASH)
                .with("monate", Param::Integer(months.floor() as i64));
            found.push((0, hint(m, None, None)));
        }
    }
    // Stable order: urgency, then the order found (sites, facilities).
    found.sort_by_key(|(rank, _)| *rank);
    found.into_iter().map(|(_, h)| h).collect()
}

/// A closed month of the player's books.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MonthView {
    /// First day of the month.
    pub month: String,
    pub revenue_usd: f64,
    pub result_usd: f64,
    /// Cash at the end of the month.
    pub cash_usd: f64,
}

/// The closed months kept in the books (oldest first), with the cash at each month's
/// end worked back from today's cash and the cash flows.
pub fn history(ledger: &Ledger) -> Vec<MonthView> {
    let mut cash = ledger.cash() - ledger.month.cash_flow.total();
    let mut months: Vec<MonthView> = ledger
        .months
        .iter()
        .rev()
        .map(|m| {
            let view = MonthView {
                month: m.start.map(super::iso).unwrap_or_default(),
                revenue_usd: m
                    .by_type
                    .get(&crate::ledger::CostType::Revenue)
                    .copied()
                    .unwrap_or(Money::ZERO)
                    .to_usd(),
                result_usd: m.total().to_usd(),
                cash_usd: cash.to_usd(),
            };
            cash -= m.cash_flow.total();
            view
        })
        .collect();
    months.reverse();
    months
}
