//! Start-ups (SU1; docs/BEDIENUNG.md, "Beteiligungen").

use serde::{Deserialize, Serialize};

use super::{iso, usd};
use crate::game::Game;
use crate::state::{Holder, Venture, VentureFailure, VentureStatus, VentureTarget};
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
}

/// The start-ups of the world (SU1).
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
    VenturesView {
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
