//! The concerns of the player's positions (MA2, MA3; docs/BEDIENUNG.md, "Anliegen"): what
//! a position asks, its way there, its options with what they do, cost and bring, and its
//! recommendation.

use serde::{Deserialize, Serialize};

use super::organisation::{role_key, unit_key};
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
use crate::state::{
    Concern, ConcernReason, ConcernStatus, GameState, Position, PriceMode, SiteId, Slot, Unit,
};
use crate::strategy;

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

/// A position in views of concerns: its role and unit (MA3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConcernPositionView {
    /// `leitung` or the function of a specialist position.
    pub role: String,
    /// `standort`, `land` or `kontinent`.
    pub level: String,
    /// Text key of the site type, `ebene.land` or `ebene.kontinent`.
    pub kind_text: String,
    pub country: Option<String>,
    pub continent: Option<String>,
    /// The unit as in the organisation (`standort:3`, `land:DEU` …).
    pub unit: String,
}

/// A position a concern passed on its way, with its manager and recommendation (MA3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HopView {
    pub position: ConcernPositionView,
    pub manager: String,
    /// Kind of the option it recommended.
    pub recommended: String,
}

/// A site's part of a strategic concern (MA3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConcernPartView {
    pub site: Option<u32>,
    pub kind_text: Option<String>,
    pub country: Option<String>,
    pub product: Option<String>,
    pub option: ConcernOptionView,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConcernView {
    pub id: u32,
    /// The site the decision is about, if any.
    pub site: Option<u32>,
    pub site_kind_text: Option<String>,
    pub site_country: Option<String>,
    /// The position that asks the player.
    pub asker: ConcernPositionView,
    /// The positions on the way, the first first; empty where it asks itself (MA3).
    pub path: Vec<HopView>,
    /// The sites' parts of a strategic concern (MA3).
    pub parts: Vec<ConcernPartView>,
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
    /// For the reasons `reserve` and `investition` (MA4): the liquidity reserve, or what
    /// is left of the investment budget that binds first and where it is set
    /// (`firma`, `land:DEU` …).
    pub strategy_limit_usd: Option<f64>,
    pub strategy_scope: Option<String>,
    /// Mean monthly result of the site in the last closed months.
    pub site_result_usd: Option<f64>,
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
        ConcernReason::Reserve => "reserve",
        ConcernReason::Investment => "investition",
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

/// A position as concerns show it.
pub(super) fn position_view(
    catalog: &Catalog,
    state: &GameState,
    p: &Position,
) -> ConcernPositionView {
    let (level, kind_text, country, continent) = match p.unit {
        Unit::Site(s) => {
            let site = &state.sites[s.index()];
            (
                "standort",
                site_type_key(site.kind),
                Some(site.country),
                None,
            )
        }
        Unit::Country(k) => ("land", "ebene.land".to_owned(), Some(k), None),
        Unit::Continent(k) => ("kontinent", "ebene.kontinent".to_owned(), None, Some(k)),
    };
    ConcernPositionView {
        role: role_key(&p.role),
        level: level.to_owned(),
        kind_text,
        country: country.map(|k| catalog.countries.key(k).to_owned()),
        continent: continent.map(|k| catalog.continents.key(k).to_owned()),
        unit: unit_key(catalog, p.unit),
    }
}

/// An option as the concern shows it.
fn option_view(
    catalog: &Catalog,
    state: &GameState,
    choice: &crate::decision::Choice,
    o: &crate::state::ConcernOption,
) -> ConcernOptionView {
    ConcernOptionView {
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
    }
}

/// The limit of the strategy a concern ran into (MA4): the reserve, or the investment
/// budget that binds first at its place.
fn strategy_limit(
    catalog: &Catalog,
    state: &GameState,
    c: &Concern,
) -> (Option<Money>, Option<String>) {
    let place = management::place_of(state, &c.decision).or_else(|| {
        c.parts
            .first()
            .and_then(|p| management::place_of(state, &p.decision))
    });
    let Some(place) = place else {
        return (None, None);
    };
    match c.reason {
        ConcernReason::Reserve => {
            let months = strategy::reserve_months(catalog, state, c.company, place);
            let reserve = strategy::monthly_cost(catalog, state, c.company).scale(months);
            (Some(reserve), None)
        }
        ConcernReason::Investment => {
            let budgets =
                strategy::investment_budgets(catalog, state, c.company, place, state.date.year());
            strategy::binding_budget(&budgets).map_or((None, None), |b| {
                (Some(b.left), Some(super::scope_key(catalog, b.scope)))
            })
        }
        _ => (None, None),
    }
}

fn concern_view(game: &Game, c: &Concern) -> ConcernView {
    let catalog = game.catalog();
    let state = game.state();
    let asker = management::asker(c);
    let site = c.decision.site.and_then(|s| state.sites.get(s.index()));
    let (status, carried_out) = status_key(c.status);
    let holder = state.managers.get(&c.manager);
    let salary = holder
        .and_then(|m| m.job.as_ref())
        .map_or(Money::ZERO, |j| j.salary);
    let (per_decision, per_year) = management::budget(catalog, state, c.company, asker, salary);
    let left = (per_year - management::spent(state, c.company, asker)).max(Money::ZERO);
    let (strategy_limit, strategy_scope) = strategy_limit(catalog, state, c);
    let options = c
        .decision
        .choices
        .iter()
        .zip(&c.options)
        .map(|(choice, o)| option_view(catalog, state, choice, o))
        .collect();
    let name = |id| {
        state
            .managers
            .get(&id)
            .map_or_else(String::new, |m| m.name.clone())
    };
    let path = c
        .path
        .iter()
        .map(|h| HopView {
            position: position_view(catalog, state, &h.position),
            manager: name(h.manager),
            recommended: c
                .decision
                .choices
                .get(h.recommended)
                .map_or_else(String::new, |o| o.kind.key().to_owned()),
        })
        .collect();
    let parts = c
        .parts
        .iter()
        .filter_map(|p| {
            let choice = p.decision.choices.get(p.recommended)?;
            let s = p.decision.site.and_then(|s| state.sites.get(s.index()));
            Some(ConcernPartView {
                site: p.decision.site.map(|s| s.0),
                kind_text: s.map(|s| site_type_key(s.kind)),
                country: s.map(|s| catalog.countries.key(s.country).to_owned()),
                product: p
                    .decision
                    .product
                    .map(|x| catalog.products.key(x).to_owned()),
                option: option_view(catalog, state, choice, &p.option),
            })
        })
        .collect();
    ConcernView {
        id: c.id,
        site: c.decision.site.map(|s| s.0),
        site_kind_text: site.map(|s| site_type_key(s.kind)),
        site_country: site.map(|s| catalog.countries.key(s.country).to_owned()),
        asker: position_view(catalog, state, asker),
        path,
        parts,
        manager: name(c.manager),
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
        strategy_limit_usd: strategy_limit.map(usd),
        strategy_scope,
        site_result_usd: c
            .decision
            .site
            .map(|s| usd(management::mean_result(state, c.company, s))),
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
