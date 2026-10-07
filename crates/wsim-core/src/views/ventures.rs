//! Start-ups (SU1; docs/BEDIENUNG.md, "Beteiligungen").

use serde::{Deserialize, Serialize};

use super::{iso, usd};
use crate::catalog::SiteType;
use crate::game::Game;
use crate::ledger::Account;
use crate::state::{
    Holder, SiteId, Venture, VentureExit, VentureFailure, VenturePace, VentureStatus, VentureTarget,
};
use crate::ventures;

/// An owner of a start-up.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StakeView {
    /// `gruender`, `investoren`, `spieler` or `firma`.
    pub holder: String,
    /// The company's name for `firma`.
    pub company: Option<String>,
    pub share: f64,
}

/// A start-up as the player sees it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VentureView {
    pub id: u32,
    /// The inventor or the founder.
    pub name: String,
    /// Named after a historical inventor.
    pub inventor: bool,
    pub country: String,
    /// `technologie` (a new one) or `verbesserung` (the next level of a product).
    pub kind: String,
    /// Key of the technology or of the product.
    pub target: String,
    /// The level an improvement aims at.
    pub level: Option<u8>,
    /// Years ahead of history at the founding (new technologies).
    pub lead: f64,
    pub founded: String,
    /// Key of the phase (`startup.phase.<key>`); none once closed.
    pub phase: Option<String>,
    /// Number of the phase, from 1, and of all phases.
    pub phase_number: usize,
    pub phases: usize,
    /// Capital of the phase and what its round raised.
    pub capital_usd: f64,
    pub raised_usd: f64,
    /// Last day of the open round; none once funded.
    pub round_until: Option<String>,
    /// When the funded phase is decided.
    pub phase_until: Option<String>,
    /// The chance of success as the strategy department estimates it (0–1); none without a
    /// working department or once closed.
    pub chance: Option<f64>,
    /// `gering`, `mittel` or `hoch`; none once closed.
    pub chance_level: Option<String>,
    pub owners: Vec<StakeView>,
    /// `aktiv`, `erfolg`, `gescheitert` (at the end of a phase), `ohne_geld` (no money in
    /// time) or `ueberholt` (the world got there first).
    pub status: String,
    /// When it ended.
    pub ended: Option<String>,
    /// Value of the whole start-up now, and of a success as it looks today (SU2).
    pub value_usd: f64,
    pub success_value_usd: f64,
    /// The player's company: its share, what it paid (book value), its pledge to the open
    /// round and its grants.
    pub own_share: f64,
    pub own_book_usd: f64,
    pub own_pledge_usd: f64,
    pub own_grants_usd: f64,
    /// How the player may invest – `runde` (pledge to the open round) or `anteile` (shares
    /// of founders and investors) – and up to how much; none where not.
    pub invest_mode: Option<String>,
    pub invest_max_usd: Option<f64>,
    /// What all of it would fetch when selling now (with the discount).
    pub sale_value_usd: f64,
    /// The value of the player's shares now and the minimum price it offers them to the
    /// companies for until the next month start; none without an offer (ZA4).
    pub own_value_usd: f64,
    pub own_offer_usd: Option<f64>,
    /// More than half: the player may steer and integrate.
    pub majority: bool,
    /// Key of the pace (`startup.lenkung.<key>`).
    pub pace: String,
    /// What buying out the others costs; none without the majority, when blocked or
    /// already a subsidiary.
    pub integration_usd: Option<f64>,
    /// Another company holds a blocking minority.
    pub blocked: bool,
    /// A subsidiary of the player.
    pub subsidiary: bool,
    /// The company it belongs to, if another one's.
    pub parent: Option<String>,
    /// What a dollar pledged brings on average as the strategy department sees it; none
    /// without a working department or open round.
    pub expected_return: Option<f64>,
    /// After a success: `tochter` (went to its parent) or `boerse` (stock market), and the
    /// company it went to or became.
    pub exit: Option<String>,
    pub exit_company: Option<String>,
    /// The company whose research project it was (SU3).
    pub origin: Option<String>,
}

/// A research project of the player that could become a start-up (SU3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpinOffView {
    pub site: u32,
    pub country: String,
    /// `technologie` or `verbesserung`, the key of the technology or product, and the
    /// level an improvement aims at.
    pub kind: String,
    pub target: String,
    pub level: Option<u8>,
    /// Share of the effort done (0–1).
    pub progress: f64,
    /// Years until history invents the technology (none for a level), and the months its
    /// phases would still take.
    pub lead: Option<f64>,
    pub months: Option<u32>,
    /// Other companies whose research centers work on the same target.
    pub rivals: u32,
    /// Key of the phase it would begin in, its value then and what all of it fetches
    /// from investors (with the discount); none where it cannot be spun off.
    pub phase: Option<String>,
    pub value_usd: Option<f64>,
    pub sale_value_usd: Option<f64>,
    /// Why not: `fortschritt` (too early) or `nicht_moeglich` (invented or reached).
    pub reason: Option<String>,
}

/// The start-ups of the world (SU1, SU2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VenturesView {
    /// What the game calls them in this year (`startup.bezeichnung.<key>`); none without
    /// start-ups in the data.
    pub label: Option<String>,
    /// New ones a year at the setting of the game.
    pub per_year: f64,
    /// Whether the strategy department estimates their chances; else only levels.
    pub estimated: bool,
    /// Newest first.
    pub active: Vec<VentureView>,
    /// Ended most recently first.
    pub closed: Vec<VentureView>,
    /// Founded since the start of the game.
    pub founded: u32,
    /// In the list: succeeded and failed ones.
    pub succeeded: u32,
    pub failed: u32,
    /// Years closed ones stay in the list (historical inventors always).
    pub keep_years: u32,
    /// The player's stakes (SU2): their book value in the balance sheet, their value now,
    /// how many start-ups, and the cash.
    pub portfolio_book_usd: f64,
    pub portfolio_value_usd: f64,
    pub holdings: u32,
    pub cash_usd: f64,
    /// Shares of the rights: blocking minority and majority.
    pub blocking: f64,
    pub majority: f64,
    /// Premium when buying between rounds or out, discount when selling, effect of a grant
    /// of the phase's capital on the gap to a sure phase.
    pub buy_premium: f64,
    pub sale_discount: f64,
    pub grant_effect: f64,
    /// The paces a majority can choose (`startup.lenkung.<key>`).
    pub paces: Vec<String>,
    /// When stakes offered to the companies go to the best bid (the next month start), and
    /// the highest premium on the value a company bids (ZA4).
    pub offers_settle: String,
    pub company_premium_max: f64,
    /// The projects of the player's research centers (SU3) and the progress from which
    /// one may be spun off.
    pub spin_offs: Vec<SpinOffView>,
    pub spin_off_min: f64,
}

pub fn ventures(game: &Game) -> VenturesView {
    let c = game.catalog();
    let m = &c.ventures;
    let state = game.state();
    let player = game.player();
    let view = |v: &Venture| -> VentureView {
        let (kind, target, level) = match v.target {
            VentureTarget::Technology(t) => ("technologie", c.technologies.key(t), None),
            VentureTarget::Development { product, level } => {
                ("verbesserung", c.products.key(product), Some(level))
            }
        };
        let active = v.status == VentureStatus::Active;
        let majority =
            v.parent == Some(player) || ventures::share_of(v, player) > m.stakes.majority;
        let blocked = ventures::blocked(m, v, player);
        let invest = if !active || v.parent.is_some_and(|p| p != player) {
            None
        } else if v.round_until.is_some() {
            Some(("runde", v.capital - v.raised))
        } else {
            let outside: f64 = v
                .owners
                .iter()
                .filter(|s| matches!(s.holder, Holder::Private | Holder::Investors))
                .map(|s| s.share)
                .sum();
            let price = ventures::value(m, v).scale((1.0 + m.stakes.buy_premium) * outside);
            (price > crate::money::Money::ZERO).then_some(("anteile", price))
        };
        let (chance, chance_level) = if active {
            let (shown, works) = ventures::shown_chance(c, state, player, v);
            let (medium, high) = m.chance_levels;
            let level = if shown >= high {
                "hoch"
            } else if shown >= medium {
                "mittel"
            } else {
                "gering"
            };
            (works.then_some(shown), Some(level.to_owned()))
        } else {
            (None, None)
        };
        let (status, ended) = match v.status {
            VentureStatus::Active => ("aktiv", None),
            VentureStatus::Succeeded(d) => ("erfolg", Some(iso(d))),
            VentureStatus::Failed(d, why) => (
                match why {
                    VentureFailure::Phase => "gescheitert",
                    VentureFailure::Funding => "ohne_geld",
                    VentureFailure::Overtaken => "ueberholt",
                },
                Some(iso(d)),
            ),
        };
        VentureView {
            id: v.id,
            name: v.name.clone(),
            inventor: v.inventor,
            country: c.countries.key(v.country).to_owned(),
            kind: kind.to_owned(),
            target: target.to_owned(),
            level,
            lead: v.lead,
            founded: iso(v.founded),
            phase: active
                .then(|| m.phases.get(v.phase).map(|p| p.key.clone()))
                .flatten(),
            phase_number: (v.phase + 1).min(m.phases.len()),
            phases: m.phases.len(),
            capital_usd: usd(v.capital),
            raised_usd: usd(v.raised),
            round_until: v.round_until.map(iso),
            phase_until: v.phase_until.map(iso),
            chance,
            chance_level,
            owners: v
                .owners
                .iter()
                .map(|s| {
                    let (holder, company) = match s.holder {
                        Holder::Private => ("gruender", None),
                        Holder::Investors => ("investoren", None),
                        Holder::Player => ("spieler", None),
                        Holder::Company(id) => ("firma", state.company(id).map(|x| x.name.clone())),
                    };
                    StakeView {
                        holder: holder.to_owned(),
                        company,
                        share: s.share,
                    }
                })
                .collect(),
            status: status.to_owned(),
            ended,
            value_usd: usd(ventures::value(m, v)),
            success_value_usd: usd(if active {
                ventures::expected_success_value(state, m, v)
            } else {
                ventures::success_value(m, v)
            }),
            own_share: ventures::share_of(v, player),
            own_book_usd: usd(ventures::amount_of(&v.book, player)),
            own_pledge_usd: usd(ventures::amount_of(&v.pledges, player)),
            own_grants_usd: usd(ventures::amount_of(&v.grants, player)),
            invest_mode: invest.map(|(mode, _)| mode.to_owned()),
            invest_max_usd: invest.map(|(_, max)| usd(max)),
            sale_value_usd: usd(ventures::value(m, v).scale(1.0 - m.stakes.sale_discount)),
            own_value_usd: usd(ventures::value(m, v).scale(ventures::share_of(v, player))),
            own_offer_usd: v
                .sales
                .iter()
                .find(|&&(c, _)| c == player)
                .map(|&(_, minimum)| usd(minimum)),
            majority,
            pace: v.pace.key().to_owned(),
            integration_usd: (active && majority && !blocked && v.parent != Some(player))
                .then(|| usd(ventures::integration_price(m, v, player))),
            blocked,
            subsidiary: v.parent == Some(player),
            parent: v
                .parent
                .filter(|&p| p != player)
                .and_then(|p| state.company(p))
                .map(|x| x.name.clone()),
            expected_return: (active && v.round_until.is_some() && chance.is_some())
                .then(|| ventures::expected_return(c, state, player, v)),
            exit: v.exit.map(|e| {
                match e {
                    VentureExit::Parent(_) => "tochter",
                    VentureExit::Listed(_) => "boerse",
                }
                .to_owned()
            }),
            exit_company: match v.exit {
                Some(VentureExit::Parent(p) | VentureExit::Listed(Some(p))) => {
                    state.company(p).map(|x| x.name.clone())
                }
                _ => None,
            },
            origin: v
                .origin
                .and_then(|o| state.company(o))
                .map(|x| x.name.clone()),
        }
    };
    let mut active: Vec<VentureView> = state
        .ventures
        .iter()
        .rev()
        .filter(|v| v.status == VentureStatus::Active)
        .map(view)
        .collect();
    active.sort_by(|a, b| b.founded.cmp(&a.founded).then(b.id.cmp(&a.id)));
    let mut closed: Vec<VentureView> = state
        .ventures
        .iter()
        .filter(|v| v.status != VentureStatus::Active)
        .map(view)
        .collect();
    closed.sort_by(|a, b| b.ended.cmp(&a.ended).then(b.id.cmp(&a.id)));
    let count = |f: fn(&VentureStatus) -> bool| {
        // Few start-ups; the cast is exact.
        state.ventures.iter().filter(|v| f(&v.status)).count() as u32
    };
    let company = &state.companies[player.index()];
    let holdings = state
        .ventures
        .iter()
        .filter(|v| v.status == VentureStatus::Active)
        .filter(|v| {
            ventures::share_of(v, player) > 0.0
                || ventures::amount_of(&v.pledges, player) > crate::money::Money::ZERO
        });
    let (held, worth) = holdings.fold((0u32, 0.0), |(n, sum), v| {
        (
            n + 1,
            sum + ventures::value(m, v).to_usd() * ventures::share_of(v, player)
                + ventures::amount_of(&v.pledges, player).to_usd(),
        )
    });
    VenturesView {
        portfolio_book_usd: usd(company.ledger.balance(Account::Participations)),
        portfolio_value_usd: worth,
        holdings: held,
        cash_usd: usd(company.ledger.cash()),
        blocking: m.stakes.blocking,
        majority: m.stakes.majority,
        buy_premium: m.stakes.buy_premium,
        sale_discount: m.stakes.sale_discount,
        grant_effect: m.stakes.grant_effect,
        paces: VenturePace::ALL
            .iter()
            .map(|p| p.key().to_owned())
            .collect(),
        offers_settle: iso(state.date.first_of_month().add_months(1)),
        company_premium_max: c.deal_model.ai.bid_markup.at(1.0),
        spin_offs: spin_offs(game),
        spin_off_min: m.stakes.spin_off_progress_min,
        label: m.label(state.date.year()).map(str::to_owned),
        per_year: m.per_year * state.settings.ventures,
        estimated: ventures::insight(c, state, player).is_some(),
        active,
        closed,
        founded: state.next_venture,
        succeeded: count(|s| matches!(s, VentureStatus::Succeeded(_))),
        failed: count(|s| matches!(s, VentureStatus::Failed(..))),
        keep_years: m.keep_years,
    }
}

/// The projects of the player's research centers and whether each may be spun off.
fn spin_offs(game: &Game) -> Vec<SpinOffView> {
    let c = game.catalog();
    let state = game.state();
    let player = game.player();
    let company = &state.companies[player.index()];
    let mut out = Vec::new();
    for (i, s) in state.sites.iter().enumerate() {
        if s.owner != player || s.kind != SiteType::ResearchCenter {
            continue;
        }
        // A handful of sites; the cast cannot overflow.
        let site = SiteId(i as u32);
        let (kind, target, level, progress) = match (s.research, s.development) {
            (Some(t), _) => {
                let effort =
                    crate::research::effort(c, state, t, state.date).map_or(0.0, |e| e.points);
                let points = company.research.get(&t).copied().unwrap_or(0.0);
                let progress = if effort > 0.0 { points / effort } else { 0.0 };
                ("technologie", c.technologies.key(t), None, progress)
            }
            (None, Some(p)) => {
                let next = crate::development::next_effort(c, state, player, p, state.date);
                let points = company.development.points.get(&p).copied().unwrap_or(0.0);
                let progress = next.map_or(0.0, |(_, e)| if e > 0.0 { points / e } else { 0.0 });
                (
                    "verbesserung",
                    c.products.key(p),
                    next.map(|(level, _)| level),
                    progress,
                )
            }
            (None, None) => continue,
        };
        let plan = ventures::spin_off_plan(c, state, player, site, state.date);
        let (phase, value, sale, reason) = match &plan {
            Ok(plan) => {
                let value = ventures::spin_off_value(c, state, player, plan);
                (
                    c.ventures.phases.get(plan.phase).map(|p| p.key.clone()),
                    Some(usd(value)),
                    Some(usd(value.scale(1.0 - c.ventures.stakes.sale_discount))),
                    None,
                )
            }
            Err(crate::command::CommandError::SpinOffTooEarly { .. }) => {
                (None, None, None, Some("fortschritt".to_owned()))
            }
            Err(_) => (None, None, None, Some("nicht_moeglich".to_owned())),
        };
        let (lead, months) = match &plan {
            Ok(plan) => (
                matches!(plan.target, crate::state::VentureTarget::Technology(_))
                    .then_some(plan.lead),
                Some(ventures::months_from(&c.ventures, plan.phase)),
            ),
            Err(_) => (None, None),
        };
        let target_of = match (s.research, s.development) {
            (Some(t), _) => crate::state::VentureTarget::Technology(t),
            (None, Some(product)) => crate::state::VentureTarget::Development {
                product,
                level: level.unwrap_or(1),
            },
            (None, None) => continue,
        };
        let rivals = ventures::rivals(state, player, target_of);
        out.push(SpinOffView {
            site: site.0,
            country: c.countries.key(s.country).to_owned(),
            kind: kind.to_owned(),
            target: target.to_owned(),
            level,
            progress: progress.clamp(0.0, 1.0),
            lead,
            months,
            rivals,
            phase,
            value_usd: value,
            sale_value_usd: sale,
            reason,
        });
    }
    out
}
