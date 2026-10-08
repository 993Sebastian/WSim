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
/// From this many own sites without a single manager the overview points to the
/// organisation (MA1).
const MANAGER_HINT_SITES: usize = 3;

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
    // The person's account falls short of the lifestyle (PE3).
    let short = state.person.short_since.is_some().then(|| {
        let mut h = hint(
            Message::new(MessageKind::Warning, keys::HINT_ACCOUNT_SHORT),
            None,
            None,
        );
        h.message.target = Some("person".to_owned());
        h
    });
    // Before the founding: the way to it (PE3).
    let Some(main) = state.main_company else {
        let mut h = hint(
            Message::new(MessageKind::Info, keys::HINT_FOUND_COMPANY),
            None,
            None,
        );
        h.message.target = Some("person".to_owned());
        return short.into_iter().chain(std::iter::once(h)).collect();
    };
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
        let month_closed = !state.companies[main.index()].ledger.months.is_empty();
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
    // End products the player sells without an own name (M42): named in the market.
    let catalog = game.catalog();
    let player = &state.companies[main.index()];
    let mut offered: Vec<crate::ids::ProductId> = state
        .sites
        .iter()
        .filter(|s| s.owner == main)
        .flat_map(|s| s.offers.keys().copied())
        .filter(|&p| {
            catalog.product_naming.style(catalog, p).is_some()
                && !player.product_names.contains_key(&p)
        })
        .collect();
    offered.sort();
    offered.dedup();
    for p in offered {
        let m = Message::new(MessageKind::Info, keys::HINT_NO_PRODUCT_NAME).with(
            "produkt",
            Param::TextKey(format!("produkt.{}", catalog.products.key(p))),
        );
        let mut h = hint(m, None, None);
        h.message.target = Some("markt".to_owned());
        found.push((3, h));
    }
    // Many sites and nobody to run them (MA1).
    let sites = state.sites.iter().filter(|s| s.owner == main).count();
    let managed = state
        .managers
        .values()
        .any(|m| m.job.as_ref().is_some_and(|j| j.company == main));
    if catalog.management.enabled() && sites >= MANAGER_HINT_SITES && !managed {
        let m = Message::new(MessageKind::Info, keys::HINT_NO_MANAGERS).with(
            "anzahl",
            Param::Integer(i64::try_from(sites).unwrap_or(i64::MAX)),
        );
        let mut h = hint(m, None, None);
        h.message.target = Some("organisation".to_owned());
        found.push((3, h));
    }
    // Concerns of the player's positions wait for an answer (MA2).
    let open: Vec<&crate::state::Concern> = state
        .concerns
        .iter()
        .filter(|c| c.company == main && c.status == crate::state::ConcernStatus::Open)
        .collect();
    if let Some(deadline) = open.iter().map(|c| c.deadline).min() {
        let m = Message::new(MessageKind::Info, keys::HINT_CONCERNS)
            .with(
                "anzahl",
                Param::Integer(i64::try_from(open.len()).unwrap_or(i64::MAX)),
            )
            .with("frist", Param::Date(deadline));
        let mut h = hint(m, None, None);
        h.message.target = Some("organisation".to_owned());
        found.push((1, h));
    }
    // Offers waiting for the player's answer expire (M30).
    for m in super::deals::offer_hints(game) {
        let mut h = hint(m, None, None);
        h.message.target = Some("wettbewerb".to_owned());
        found.push((1, h));
    }
    let ledger = &state.companies[main.index()].ledger;
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
    // A filled position takes care of what its topic is at its site (MA2), power and
    // deposits those of the country and the continent (MA3).
    use crate::decision::Topic;
    let topic = |key: &str| match key {
        keys::HINT_INPUT | keys::HINT_NO_PURCHASE => Some(Topic::Purchase),
        keys::HINT_NO_OFFER | keys::HINT_BELOW_COST | keys::HINT_UNSOLD => Some(Topic::Sale),
        keys::HINT_LABOR | keys::HINT_STAFF => Some(Topic::Wage),
        keys::HINT_NO_RECIPE | keys::HINT_IDLE => Some(Topic::Production),
        keys::HINT_NO_RESEARCH => Some(Topic::Research),
        keys::HINT_POWER => Some(Topic::Power),
        keys::HINT_DEPOSIT => Some(Topic::Deposit),
        _ => None,
    };
    found.retain(|(_, h)| match (h.site, topic(&h.message.key)) {
        (Some(site), Some(t)) => {
            !crate::management::covered(catalog, state, crate::state::SiteId(site), t)
        }
        _ => true,
    });
    // Stable order: urgency, then the order found (sites, facilities).
    if let Some(h) = short {
        found.push((1, h));
    }
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
