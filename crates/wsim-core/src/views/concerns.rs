//! The concerns of the player's positions (MA2; docs/BEDIENUNG.md, "Anliegen"): what a
//! position asks, its options with what they do, cost and bring, and its recommendation.

use serde::{Deserialize, Serialize};

use super::organisation::role_key;
use super::{MessageView, iso, message_view, usd};
use crate::catalog::Catalog;
use crate::command::Command;
use crate::command::site_type_key;
use crate::decision::ChoiceKind;
use crate::game::Game;
use crate::ids::ProductId;
use crate::management;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::state::{Concern, ConcernReason, ConcernStatus, GameState, PriceMode, SiteId, Slot};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConcernOptionView {
    /// Kind of the option (text `option.<kind>`).
    pub kind: String,
    /// What the option does, step by step.
    pub steps: Vec<MessageView>,
    /// Counted against a budget.
    pub amount_usd: f64,
    /// The position's forecast of the effect on a year's result, as a range; none without
    /// an estimate.
    pub forecast_usd: Option<(f64, f64)>,
    /// One-off effect on the result.
    pub once_usd: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConcernView {
    pub id: u32,
    pub site: u32,
    /// `leitung` or the function of a specialist position.
    pub role: String,
    /// Text key of the site type.
    pub kind_text: String,
    pub country: String,
    /// The manager who asks.
    pub manager: String,
    /// Topic (text `thema.<topic>`).
    pub topic: String,
    /// Key of the product concerned.
    pub product: Option<String>,
    /// Why the position asks: `entscheidung`, `jahr`, `immer` or `kredit`.
    pub reason: String,
    pub options: Vec<ConcernOptionView>,
    pub recommended: usize,
    /// Why the position recommends its option.
    pub because: MessageView,
    /// What the position may spend on one decision, and what is left of its year.
    pub per_decision_usd: f64,
    pub left_usd: f64,
    /// Mean monthly result of the site in the last closed months.
    pub site_result_usd: f64,
    pub created: String,
    pub deadline: String,
    /// `offen`, `gewaehlt`, `delegiert`, `nicht_mehr_fragen`, `abgelehnt`, `abgelaufen`
    /// or `erledigt`.
    pub status: String,
    /// The option carried out.
    pub carried_out: Option<usize>,
    pub closed: Option<String>,
    /// Beyond the routine of the sites: a run of rounds halts for it ("wichtige").
    pub important: bool,
}

/// Open concerns of several sites alike: the same topic and recommendation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConcernGroupView {
    pub topic: String,
    /// Kind of the recommended option.
    pub kind: String,
    pub concerns: Vec<ConcernView>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConcernsView {
    /// Open concerns in groups, the earliest deadline first.
    pub open: Vec<ConcernGroupView>,
    /// Concerns closed in the last year, the newest first.
    pub closed: Vec<ConcernView>,
    pub deadline_days: u32,
    pub block_days: u32,
}

/// Closed concerns shown.
const CLOSED_SHOWN: usize = 30;

fn status_key(status: ConcernStatus) -> (&'static str, Option<usize>) {
    match status {
        ConcernStatus::Open => ("offen", None),
        ConcernStatus::Chosen(i) => ("gewaehlt", Some(i)),
        ConcernStatus::Delegated(i) => ("delegiert", Some(i)),
        ConcernStatus::Muted => ("nicht_mehr_fragen", None),
        ConcernStatus::Declined => ("abgelehnt", None),
        ConcernStatus::Expired => ("abgelaufen", None),
        ConcernStatus::Settled => ("erledigt", None),
    }
}

fn reason_key(reason: ConcernReason) -> &'static str {
    match reason {
        ConcernReason::Decision => "entscheidung",
        ConcernReason::Year => "jahr",
        ConcernReason::Always => "immer",
        ConcernReason::Finance => "kredit",
    }
}

fn product_param(catalog: &Catalog, product: ProductId) -> Param {
    Param::TextKey(format!("produkt.{}", catalog.products.key(product)))
}

fn unit_param(catalog: &Catalog, product: ProductId) -> Param {
    let unit = catalog.products.get(product).unit;
    Param::TextKey(format!("einheit.{}", catalog.units.key(unit)))
}

fn slot(state: &GameState, site: SiteId, slot: usize) -> Option<&Slot> {
    state.sites.get(site.index())?.slots.get(slot)
}

fn facility_param(catalog: &Catalog, sl: &Slot) -> Param {
    Param::TextKey(format!("anlage.{}", catalog.facilities.key(sl.facility)))
}

fn percent(share: f64) -> Param {
    // Shares of a few hundred percent at most; the cast is exact.
    Param::Integer((share * 100.0).round() as i64)
}

/// What a step of an option does, in words; none for steps without a text.
fn step(catalog: &Catalog, state: &GameState, command: &Command) -> Option<Message> {
    let m = |key: &str| Message::new(MessageKind::Info, key);
    Some(match command {
        Command::MothballFacility {
            site,
            slot: i,
            count,
        } => m(keys::STEP_MOTHBALL)
            .with("anzahl", Param::Integer(i64::from(*count)))
            .with("anlage", facility_param(catalog, slot(state, *site, *i)?)),
        Command::SellFacility {
            site,
            slot: i,
            count,
        } => m(keys::STEP_SELL)
            .with("anzahl", Param::Integer(i64::from(*count)))
            .with("anlage", facility_param(catalog, slot(state, *site, *i)?)),
        Command::RestartFacility { site, slot: i } => {
            m(keys::STEP_RESTART).with("anlage", facility_param(catalog, slot(state, *site, *i)?))
        }
        Command::BuildFacility {
            facility,
            count,
            size,
            ..
        } => m(keys::STEP_BUILD)
            .with("anzahl", Param::Integer(i64::from(*count)))
            .with(
                "anlage",
                Param::TextKey(format!("anlage.{}", catalog.facilities.key(*facility))),
            )
            .with(
                "groesse",
                Param::TextKey(format!("anlagengroesse.{}", size.key())),
            ),
        Command::TakeLoan { amount, years } => m(keys::STEP_LOAN)
            .with("betrag", Param::Money(*amount))
            .with("jahre", Param::Integer(i64::from(*years))),
        Command::RepayLoan { amount, .. } => {
            m(keys::STEP_REPAY).with("betrag", Param::Money(*amount))
        }
        Command::SetProduction {
            site,
            slot: i,
            recipe,
            utilization,
        } => {
            let anlage = facility_param(catalog, slot(state, *site, *i)?);
            match recipe {
                Some(r) => m(keys::STEP_PRODUCTION)
                    .with("anlage", anlage)
                    .with(
                        "rezept",
                        Param::TextKey(format!("rezept.{}", catalog.recipes.key(*r))),
                    )
                    .with("auslastung", percent(*utilization)),
                None => m(keys::STEP_PRODUCTION_OFF).with("anlage", anlage),
            }
        }
        Command::SetAutomation {
            site,
            slot: i,
            level,
        } => m(keys::STEP_AUTOMATION)
            .with("anlage", facility_param(catalog, slot(state, *site, *i)?))
            .with("stufe", percent(*level)),
        Command::SetSale { product, mode, .. } => match mode {
            None => m(keys::STEP_SALE_END).with("produkt", product_param(catalog, *product)),
            Some(PriceMode::Fixed(price)) => m(keys::STEP_PRICE)
                .with("produkt", product_param(catalog, *product))
                .with("preis", Param::Money(*price))
                .with("einheit", unit_param(catalog, *product)),
            Some(PriceMode::Market { markup, .. }) => m(keys::STEP_SALE_MARKET)
                .with("produkt", product_param(catalog, *product))
                .with("aufschlag", percent(*markup)),
        },
        Command::SetPrice { product, price, .. } => m(keys::STEP_PRICE)
            .with("produkt", product_param(catalog, *product))
            .with("preis", Param::Money(*price))
            .with("einheit", unit_param(catalog, *product)),
        Command::SetPurchase {
            product,
            target,
            max_price,
            ..
        } => {
            if *target <= 0.0 {
                m(keys::STEP_PURCHASE_END).with("produkt", product_param(catalog, *product))
            } else {
                m(keys::STEP_PURCHASE)
                    .with("produkt", product_param(catalog, *product))
                    .with("menge", Param::Number(*target))
                    .with("einheit", unit_param(catalog, *product))
                    .with("preis", Param::Money(*max_price))
            }
        }
        Command::TransferGoods {
            from,
            product,
            quantity,
            ..
        } => {
            let source = state.sites.get(from.index())?;
            m(keys::STEP_SUPPLY)
                .with("menge", Param::Number(*quantity))
                .with("einheit", unit_param(catalog, *product))
                .with("produkt", product_param(catalog, *product))
                .with("standort", Param::TextKey(site_type_key(source.kind)))
                .with(
                    "land",
                    Param::Country(catalog.countries.key(source.country).to_owned()),
                )
        }
        Command::SetWagePremium { premium, .. } => {
            m(keys::STEP_WAGE).with("aufschlag", percent(*premium))
        }
        Command::SetResearch { technology, .. } => match technology {
            Some(t) => m(keys::STEP_RESEARCH).with(
                "technologie",
                Param::TextKey(format!("technologie.{}", catalog.technologies.key(*t))),
            ),
            None => m(keys::STEP_RESEARCH_END),
        },
        Command::SetDevelopment { product, .. } => match product {
            Some(p) => m(keys::STEP_DEVELOP).with("produkt", product_param(catalog, *p)),
            None => m(keys::STEP_RESEARCH_END),
        },
        Command::DevelopDeposit { deposit, .. } => m(keys::STEP_DEPOSIT).with(
            "lagerstaette",
            Param::TextKey(format!("lagerstaette.{}", catalog.deposits.key(*deposit))),
        ),
        _ => return None,
    })
}

/// Why the position recommends its option (docs/MANAGER.md 6.2).
fn because(c: &Concern) -> Message {
    let m = |key: &str| Message::new(MessageKind::Info, key);
    let kind = c.decision.choices.get(c.recommended).map(|o| o.kind);
    let option = c.options.get(c.recommended);
    if kind == Some(ChoiceKind::Keep) {
        return m(keys::BECAUSE_WAIT);
    }
    match option {
        Some(o) if o.forecast.is_some_and(|(_, high)| high > Money::ZERO) => {
            let (low, high) = o.forecast.unwrap_or_default();
            m(keys::BECAUSE_RESULT)
                .with("von", Param::Money(low))
                .with("bis", Param::Money(high))
        }
        Some(o) if o.once > Money::ZERO => {
            m(keys::BECAUSE_PROCEEDS).with("betrag", Param::Money(o.once))
        }
        _ => m(keys::BECAUSE_RULE),
    }
}

fn concern_view(game: &Game, c: &Concern) -> ConcernView {
    let catalog = game.catalog();
    let state = game.state();
    let site = &state.sites[c.position.site.index()];
    let (status, carried_out) = status_key(c.status);
    let holder = state.managers.get(&c.manager);
    let salary = holder
        .and_then(|m| m.job.as_ref())
        .map_or(Money::ZERO, |j| j.salary);
    let (per_decision, per_year) =
        management::budget(catalog, state, c.company, &c.position, salary);
    let left = (per_year - management::spent(state, c.company, &c.position)).max(Money::ZERO);
    let options = c
        .decision
        .choices
        .iter()
        .zip(&c.options)
        .map(|(choice, o)| ConcernOptionView {
            kind: choice.kind.key().to_owned(),
            steps: choice
                .steps
                .iter()
                .filter_map(|s| step(catalog, state, &s.command))
                .map(|m| message_view(&m))
                .collect(),
            amount_usd: usd(o.amount),
            forecast_usd: o.forecast.map(|(a, b)| (usd(a), usd(b))),
            once_usd: usd(o.once),
        })
        .collect();
    ConcernView {
        id: c.id,
        site: c.position.site.0,
        role: role_key(&c.position.role),
        kind_text: site_type_key(site.kind),
        country: catalog.countries.key(site.country).to_owned(),
        manager: holder.map(|m| m.name.clone()).unwrap_or_default(),
        topic: c.decision.topic.key().to_owned(),
        product: c
            .decision
            .product
            .map(|p| catalog.products.key(p).to_owned()),
        reason: reason_key(c.reason).to_owned(),
        options,
        recommended: c.recommended,
        because: message_view(&because(c)),
        per_decision_usd: usd(per_decision),
        left_usd: usd(left),
        site_result_usd: usd(management::mean_result(state, c.company, c.position.site)),
        created: iso(c.created),
        deadline: iso(c.deadline),
        status: status.to_owned(),
        carried_out,
        closed: c.closed.map(iso),
        important: management::important(catalog, c),
    }
}

/// The concerns of the player's positions: the open ones grouped, then the closed ones.
pub fn concerns(game: &Game) -> ConcernsView {
    let state = game.state();
    let player = game.player();
    let model = &game.catalog().management.concerns;
    let mut open: Vec<&Concern> = state
        .concerns
        .iter()
        .filter(|c| c.company == player && c.status == ConcernStatus::Open)
        .collect();
    open.sort_by_key(|c| (c.deadline, c.id));
    let mut groups: Vec<ConcernGroupView> = Vec::new();
    for c in open {
        let view = concern_view(game, c);
        let kind = view
            .options
            .get(view.recommended)
            .map(|o| o.kind.clone())
            .unwrap_or_default();
        match groups
            .iter_mut()
            .find(|g| g.topic == view.topic && g.kind == kind)
        {
            Some(g) => g.concerns.push(view),
            None => groups.push(ConcernGroupView {
                topic: view.topic.clone(),
                kind,
                concerns: vec![view],
            }),
        }
    }
    let mut closed: Vec<&Concern> = state
        .concerns
        .iter()
        .filter(|c| c.company == player && c.status != ConcernStatus::Open)
        .collect();
    closed.sort_by_key(|c| std::cmp::Reverse((c.closed, c.id)));
    ConcernsView {
        open: groups,
        closed: closed
            .into_iter()
            .take(CLOSED_SHOWN)
            .map(|c| concern_view(game, c))
            .collect(),
        deadline_days: model.deadline_days,
        block_days: model.block_days,
    }
}
