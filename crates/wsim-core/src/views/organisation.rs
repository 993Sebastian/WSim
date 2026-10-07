//! The company's organisation (MA1–MA3; docs/BEDIENUNG.md, "Organisation"): the
//! positions of its sites, countries and continents and who holds them, their budgets,
//! the rules for types of positions, and the market of managers for a position.

use serde::{Deserialize, Serialize};

use super::{iso, usd};
use crate::catalog::{Catalog, SiteType};
use crate::command::site_type_key;
use crate::decision::Topic;
use crate::game::Game;
use crate::management::{self, shown_level};
use crate::money::Money;
use crate::staffing;
use crate::state::{
    CompanyId, ConcernStatus, GameState, Manager, ManagerId, Position, Role, RuleScope, SiteId,
    Unit, UnitLevel,
};

/// A skill as the player sees it: a level, never the number.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkillView {
    /// `fach.<bereich>`, `erkennen`, `urteil`, `fuehrung`, `risiko`, `fragefreude`.
    pub key: String,
    /// 0 (weak) … 4 (outstanding).
    pub level: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ManagerView {
    pub id: u32,
    pub name: String,
    /// Home country (ISO).
    pub home: String,
    pub continent: String,
    /// Function the manager is best at.
    pub focus: String,
    pub skills: Vec<SkillView>,
}

/// Another company's offer to a manager of the player (MA6).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PoachOfferView {
    pub company: String,
    pub salary_usd: f64,
    /// Last day it stands.
    pub until: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HolderView {
    pub manager: ManagerView,
    pub salary_usd: f64,
    pub since: String,
    /// What a dismissal costs now.
    pub severance_usd: f64,
    /// What he would ask for the position today (MA6).
    pub market_usd: f64,
    /// 0 unhappy, 1 mixed, 2 happy (MA6).
    pub satisfaction: u8,
    pub offer: Option<PoachOfferView>,
}

/// What a position may spend without asking (MA2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BudgetView {
    /// Shares of the reference per decision and per year.
    pub shares: (f64, f64),
    /// The shares of its level and role.
    pub defaults: (f64, f64),
    /// Set by the player.
    pub custom: bool,
    /// The site's revenue in the last twelve closed months, without revenue its costs.
    pub base_usd: f64,
    pub per_decision_usd: f64,
    pub per_year_usd: f64,
    pub spent_usd: f64,
}

/// A decision a position took itself.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DecisionLogView {
    pub date: String,
    /// Topic (text `thema.<topic>`).
    pub topic: String,
    /// Kind of the option (text `option.<kind>`).
    pub kind: String,
    pub product: Option<String>,
    pub amount_usd: f64,
    /// The position's estimate of the effect on a year's result.
    pub effect_usd: Option<f64>,
}

/// A topic the position does not ask about.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QuietTopicView {
    /// For the command `AskAgain`.
    pub id: Topic,
    /// Text `thema.<topic>`.
    pub topic: String,
    /// Declined: quiet until then; none: not to be asked again.
    pub until: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PositionView {
    /// `leitung` or the key of the function of a specialist position.
    pub role: String,
    /// Topics (`thema.<key>`) the position takes care of now; for an empty position those
    /// it would take over.
    pub topics: Vec<String>,
    pub holder: Option<HolderView>,
    /// Only for a filled position.
    pub budget: Option<BudgetView>,
    /// Its latest own decisions, the newest first.
    pub log: Vec<DecisionLogView>,
    pub quiet: Vec<QuietTopicView>,
    /// Its concerns waiting for an answer.
    pub open_concerns: u32,
    /// Text key of what the position does besides topics (MA5: the personnel member of
    /// the board sees candidates more sharply).
    pub effect: Option<String>,
    /// For heads: whether it fills the free positions of its unit itself (MA6).
    pub hires: Option<bool>,
}

/// The positions of a unit of the player: a site, a country or a continent (MA1, MA3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UnitOrgView {
    /// `standort`, `land`, `kontinent` or `vorstand`.
    pub level: String,
    /// The site's number (sites).
    pub site: Option<u32>,
    /// The country (sites and countries).
    pub country: Option<String>,
    /// The continent (continents).
    pub continent: Option<String>,
    /// Text key of the site type, else `ebene.land`, `ebene.kontinent` or `ebene.vorstand`.
    pub kind_text: String,
    /// The unit for the market of managers: `standort:3`, `land:DEU`, `kontinent:europa`,
    /// `vorstand`.
    pub key: String,
    pub positions: Vec<PositionView>,
    /// Next check of the positions; none without a manager there.
    pub next_check: Option<String>,
    /// Topics of the unit nobody takes care of: the player decides them.
    pub own_topics: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CountryOrgView {
    pub country: String,
    /// The positions of the country (MA3).
    pub unit: Option<UnitOrgView>,
    pub sites: Vec<UnitOrgView>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContinentOrgView {
    pub continent: String,
    /// The positions of the continent (MA3).
    pub unit: Option<UnitOrgView>,
    pub countries: Vec<CountryOrgView>,
}

/// A type of position budget rules apply to (MA3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PositionKindView {
    /// `standort`, `land` or `kontinent`.
    pub level: String,
    /// The site type as commands name it (`Factory` …), for sites.
    pub site_type: Option<SiteType>,
    /// Text key of the site type, `ebene.land` or `ebene.kontinent`.
    pub kind_text: String,
    /// `leitung` or the function.
    pub role: String,
}

/// A budget rule of the player (MA3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BudgetRuleView {
    pub kind: PositionKindView,
    /// `firma`, `kontinent` or `land`.
    pub scope: String,
    /// Key of the continent or country.
    pub scope_key: Option<String>,
    pub shares: (f64, f64),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrganisationView {
    /// False without manager data.
    pub enabled: bool,
    /// The board: CEO and members (MA5); none without a site.
    pub board: Option<UnitOrgView>,
    pub continents: Vec<ContinentOrgView>,
    pub managers: u32,
    /// Salaries of all managers per year.
    pub salaries_usd: f64,
    /// Days between the checks of a site's positions.
    pub check_days: u32,
    pub severance_months: f64,
    /// Free candidates in all markets.
    pub candidates: u32,
    /// Least budget of a position in its yearly salaries: per decision and per year.
    pub budget_floor: (f64, f64),
    /// The player's budget rules (MA3).
    pub rules: Vec<BudgetRuleView>,
    /// Types of positions the company has, for new rules.
    pub kinds: Vec<PositionKindView>,
    /// The headquarters and the central departments (ZA1–ZA3).
    pub central: super::CentralView,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CurrentPositionView {
    /// The unit as in `UnitOrgView::key`.
    pub unit: String,
    pub site: Option<u32>,
    pub role: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandidateView {
    pub manager: ManagerView,
    /// Salary per year for the position.
    pub demand_usd: f64,
    /// An own manager's position now.
    pub current: Option<CurrentPositionView>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ManagerMarketView {
    /// The unit as in `UnitOrgView::key`.
    pub unit: String,
    pub site: Option<u32>,
    pub role: String,
    pub kind_text: String,
    pub country: Option<String>,
    pub continent: String,
    /// Free candidates, those of the unit's continent first.
    pub candidates: Vec<CandidateView>,
    /// The company's managers on other positions.
    pub own: Vec<CandidateView>,
}

/// Key of a role in views: `leitung` or the function.
pub fn role_key(role: &Role) -> String {
    match role {
        Role::Head => "leitung".into(),
        Role::Specialist(f) => f.clone(),
    }
}

/// Key of a level in views.
pub(super) fn level_key(unit: Unit) -> &'static str {
    match unit {
        Unit::Site(_) => "standort",
        Unit::Country(_) => "land",
        Unit::Continent(_) => "kontinent",
        Unit::Board => "vorstand",
    }
}

/// A unit as views and the market name it: `standort:3`, `land:DEU`, `kontinent:europa`,
/// `vorstand`.
pub fn unit_key(catalog: &Catalog, unit: Unit) -> String {
    match unit {
        Unit::Site(s) => format!("standort:{}", s.0),
        Unit::Country(c) => format!("land:{}", catalog.countries.key(c)),
        Unit::Continent(k) => format!("kontinent:{}", catalog.continents.key(k)),
        Unit::Board => "vorstand".into(),
    }
}

/// The unit a key names (`unit_key`).
pub fn unit_from_key(catalog: &Catalog, key: &str) -> Option<Unit> {
    if key == "vorstand" {
        return Some(Unit::Board);
    }
    let (level, rest) = key.split_once(':')?;
    match level {
        "standort" => rest.parse().ok().map(|n| Unit::Site(SiteId(n))),
        "land" => catalog.countries.id(rest).map(Unit::Country),
        "kontinent" => catalog.continents.id(rest).map(Unit::Continent),
        _ => None,
    }
}

/// Text key of a unit's kind: the site type, else the level.
pub(super) fn kind_text(state: &GameState, unit: Unit) -> String {
    match unit {
        Unit::Site(s) => site_type_key(state.sites[s.index()].kind),
        Unit::Country(_) => "ebene.land".into(),
        Unit::Continent(_) => "ebene.kontinent".into(),
        Unit::Board => "ebene.vorstand".into(),
    }
}

fn manager_view(game: &Game, id: ManagerId, m: &Manager) -> ManagerView {
    let c = game.catalog();
    // The personnel member of the board sees more sharply (MA5).
    let share = management::impression_share(c, game.state(), game.player());
    let skills = management::skill_keys(c)
        .into_iter()
        .map(|key| {
            let value = management::skill(m, &key).unwrap_or(0);
            let impression =
                management::shown_impression(m.impression.get(&key).copied().unwrap_or(0), share);
            SkillView {
                level: shown_level(value, impression),
                key,
            }
        })
        .collect();
    ManagerView {
        id: id.0,
        name: m.name.clone(),
        home: c.countries.key(m.home).to_owned(),
        continent: c
            .continents
            .key(c.countries.get(m.home).continent)
            .to_owned(),
        focus: m.focus.clone(),
        skills,
    }
}

fn topic_keys(topics: &[Topic]) -> Vec<String> {
    topics.iter().map(|t| t.key().to_owned()).collect()
}

/// The positions of a unit with the topics each takes care of, and the topics left to the
/// player.
fn unit_view(game: &Game, unit: Unit) -> UnitOrgView {
    let c = game.catalog();
    let m = &c.management;
    let state = game.state();
    let player = game.player();
    let held = |role: &Role| {
        management::holder(
            state,
            player,
            &Position {
                unit,
                role: role.clone(),
            },
        )
    };
    let head = held(&Role::Head);
    let all = management::positions(c, state, player, unit);
    let has = |f: &str| {
        all.iter()
            .any(|p| matches!(&p.role, Role::Specialist(k) if k == f))
    };
    let mut head_topics = Vec::new();
    let mut own_topics = Vec::new();
    let arising = |topics: &[Topic]| -> Vec<Topic> {
        topics
            .iter()
            .copied()
            .filter(|&t| management::arises(c, state, unit, t))
            .collect()
    };
    for function in &m.functions {
        let filled = has(&function.key) && held(&Role::Specialist(function.key.clone())).is_some();
        if !filled {
            let topics = arising(&function.topics);
            head_topics.extend(topics.iter().copied());
            if head.is_none() {
                own_topics.extend(topics);
            }
        }
    }
    // Topics a position above takes care of are not the player's; those of a continent
    // or the board only where one of its countries has nobody for them (MA3, MA5).
    let countries: Vec<crate::ids::CountryId> = match unit {
        Unit::Site(_) => Vec::new(),
        Unit::Country(k) => vec![k],
        Unit::Continent(_) | Unit::Board => state
            .sites
            .iter()
            .filter(|s| {
                s.owner == player
                    && match unit {
                        Unit::Continent(k) => c.countries.get(s.country).continent == k,
                        _ => true,
                    }
            })
            .map(|s| s.country)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect(),
    };
    own_topics.retain(|&t| match unit {
        Unit::Site(s) => !management::covered(c, state, s, t),
        _ => countries
            .iter()
            .any(|&k| management::first_taker(c, state, player, Unit::Country(k), t).is_none()),
    });
    let holder_view = |id: ManagerId| {
        let manager = &state.managers[&id];
        let job = manager.job.as_ref().expect("holds the position");
        HolderView {
            manager: manager_view(game, id, manager),
            salary_usd: usd(job.salary),
            since: iso(job.since),
            severance_usd: usd(job.salary.scale(m.severance_months / 12.0)),
            market_usd: usd(staffing::market_value(c, state, manager)),
            satisfaction: staffing::satisfaction_level(c, staffing::satisfaction(c, job)),
            offer: state
                .poach_offers
                .iter()
                .find(|o| o.manager == id)
                .map(|o| PoachOfferView {
                    company: state
                        .companies
                        .get(o.bidder.index())
                        .map_or_else(String::new, |x| x.name.clone()),
                    salary_usd: usd(o.salary),
                    until: iso(o.until),
                }),
        }
    };
    let position_view = |position: &Position, topics: &[Topic]| {
        let holder = management::holder(state, player, position);
        let ps = management::position_state(state, player, position);
        let budget = holder
            .and_then(|id| state.managers[&id].job.as_ref())
            .map(|job| {
                let shares = management::budget_shares(c, state, player, position);
                let (per_decision, per_year) =
                    management::budget(c, state, player, position, job.salary);
                BudgetView {
                    shares,
                    defaults: management::rule_for(c, state, player, position)
                        .map_or_else(|| management::default_shares(c, position), |r| r.shares),
                    custom: ps.is_some_and(|p| p.budget.is_some()),
                    base_usd: usd(management::budget_base(c, state, player, unit)),
                    per_decision_usd: usd(per_decision),
                    per_year_usd: usd(per_year),
                    spent_usd: usd(management::spent(state, player, position)),
                }
            });
        let log = ps
            .map(|p| {
                p.log
                    .iter()
                    .rev()
                    .map(|l| DecisionLogView {
                        date: iso(l.date),
                        topic: l.topic.key().to_owned(),
                        kind: l.kind.key().to_owned(),
                        product: l.product.map(|x| c.products.key(x).to_owned()),
                        amount_usd: usd(l.amount),
                        effect_usd: l.effect.map(usd),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let mut quiet: Vec<QuietTopicView> = ps
            .map(|p| {
                p.muted
                    .iter()
                    .map(|&t| (t, None))
                    .chain(
                        p.blocked
                            .iter()
                            .filter(|(t, until)| **until > state.date && !p.muted.contains(t))
                            .map(|(&t, &until)| (t, Some(iso(until)))),
                    )
                    .map(|(id, until)| QuietTopicView {
                        id,
                        topic: id.key().to_owned(),
                        until,
                    })
                    .collect()
            })
            .unwrap_or_default();
        quiet.sort_by_key(|q| q.id);
        let open_concerns = state
            .concerns
            .iter()
            .filter(|x| {
                x.company == player
                    && management::asker(x) == position
                    && x.status == ConcernStatus::Open
            })
            .count();
        let personnel = management::function_of(c, Topic::Wage).map(|i| &m.functions[i].key);
        let effect = match (&position.role, unit) {
            (Role::Specialist(f), Unit::Board) if Some(f) == personnel => {
                Some("organisation.wirkung_personal".to_owned())
            }
            _ => None,
        };
        PositionView {
            role: role_key(&position.role),
            topics: topic_keys(topics),
            holder: holder.map(holder_view),
            budget,
            log,
            quiet,
            open_concerns: u32::try_from(open_concerns).unwrap_or(u32::MAX),
            effect,
            hires: (position.role == Role::Head).then(|| ps.is_some_and(|p| p.hires)),
        }
    };
    let positions = all
        .iter()
        .map(|p| match &p.role {
            Role::Head => position_view(p, &head_topics),
            Role::Specialist(f) => {
                let topics = m
                    .function(f)
                    .map_or_else(Vec::new, |i| arising(&m.functions[i].topics));
                position_view(p, &topics)
            }
        })
        .collect::<Vec<_>>();
    let staffed = positions.iter().any(|p| p.holder.is_some());
    let country = match unit {
        Unit::Site(s) => Some(state.sites[s.index()].country),
        Unit::Country(k) => Some(k),
        Unit::Continent(_) | Unit::Board => None,
    };
    UnitOrgView {
        level: level_key(unit).into(),
        site: unit_site(unit),
        country: country.map(|k| c.countries.key(k).to_owned()),
        continent: match unit {
            Unit::Continent(k) => Some(c.continents.key(k).to_owned()),
            _ => None,
        },
        kind_text: kind_text(state, unit),
        key: unit_key(c, unit),
        positions,
        next_check: staffed
            .then(|| management::next_check(c, unit, state.date))
            .flatten()
            .map(iso),
        own_topics: topic_keys(&own_topics),
    }
}

fn unit_site(unit: Unit) -> Option<u32> {
    match unit {
        Unit::Site(s) => Some(s.0),
        _ => None,
    }
}

fn kind_view(state: &GameState, level: UnitLevel, role: &Role) -> PositionKindView {
    let (level_key, site_type, kind_text) = match level {
        UnitLevel::Site(t) => ("standort", Some(t), site_type_key(t)),
        UnitLevel::Country => ("land", None, "ebene.land".into()),
        UnitLevel::Continent => ("kontinent", None, "ebene.kontinent".into()),
        UnitLevel::Board => ("vorstand", None, "ebene.vorstand".into()),
    };
    let _ = state;
    PositionKindView {
        level: level_key.into(),
        site_type,
        kind_text,
        role: role_key(role),
    }
}

/// The player's organisation: continents and countries with their positions, and the
/// sites with theirs.
pub fn organisation(game: &Game) -> OrganisationView {
    let c = game.catalog();
    let m = &c.management;
    let state = game.state();
    let player = game.player();
    let mut continents: Vec<ContinentOrgView> = Vec::new();
    let own: Vec<SiteId> = state
        .sites
        .iter()
        .enumerate()
        .filter(|(_, s)| s.owner == player)
        // Few sites; the cast is exact.
        .map(|(i, _)| SiteId(i as u32))
        .collect();
    let with_positions = |unit: Unit| {
        let view = unit_view(game, unit);
        (!view.positions.is_empty()).then_some(view)
    };
    if m.enabled() {
        for continent in c.continents.ids() {
            let mut countries: Vec<CountryOrgView> = Vec::new();
            for country in c.countries.ids() {
                if c.countries.get(country).continent != continent {
                    continue;
                }
                let sites: Vec<UnitOrgView> = own
                    .iter()
                    .filter(|&&s| state.sites[s.index()].country == country)
                    .map(|&s| unit_view(game, Unit::Site(s)))
                    .collect();
                if !sites.is_empty() {
                    countries.push(CountryOrgView {
                        country: c.countries.key(country).to_owned(),
                        unit: with_positions(Unit::Country(country)),
                        sites,
                    });
                }
            }
            if !countries.is_empty() {
                continents.push(ContinentOrgView {
                    continent: c.continents.key(continent).to_owned(),
                    unit: with_positions(Unit::Continent(continent)),
                    countries,
                });
            }
        }
    }
    let employed = || {
        state
            .managers
            .values()
            .filter_map(|x| x.job.as_ref())
            .filter(|j| j.company == player)
    };
    let company = &state.companies[player.index()];
    let rules = company
        .budget_rules
        .iter()
        .map(|r| {
            let (scope, scope_key) = match r.scope {
                RuleScope::Company => ("firma", None),
                RuleScope::Continent(k) => ("kontinent", Some(c.continents.key(k).to_owned())),
                RuleScope::Country(k) => ("land", Some(c.countries.key(k).to_owned())),
            };
            BudgetRuleView {
                kind: kind_view(state, r.kind.level, &r.kind.role),
                scope: scope.into(),
                scope_key,
                shares: r.shares,
            }
        })
        .collect();
    // The types of positions of the company's units, in the order of the units.
    let mut kinds: Vec<PositionKindView> = Vec::new();
    for unit in management::units(c, state, player) {
        for p in management::positions(c, state, player, unit) {
            if let Some(kind) = management::kind_of(state, &p) {
                let view = kind_view(state, kind.level, &kind.role);
                if !kinds.contains(&view) {
                    kinds.push(view);
                }
            }
        }
    }
    OrganisationView {
        enabled: m.enabled(),
        board: m.enabled().then(|| with_positions(Unit::Board)).flatten(),
        continents,
        managers: u32::try_from(employed().count()).unwrap_or(u32::MAX),
        salaries_usd: usd(employed().map(|j| j.salary).sum::<Money>()),
        check_days: m.levels.first().map_or(0, |l| l.check_days),
        severance_months: m.severance_months,
        candidates: u32::try_from(state.managers.values().filter(|x| x.job.is_none()).count())
            .unwrap_or(u32::MAX),
        budget_floor: m.budget_floor,
        rules,
        kinds,
        central: super::central(game),
    }
}

/// The position of a unit of the player by its role key, if it exists.
fn player_position(game: &Game, unit: Unit, role: &str) -> Option<Position> {
    let position = Position {
        unit,
        role: if role == "leitung" {
            Role::Head
        } else {
            Role::Specialist(role.to_owned())
        },
    };
    management::positions(game.catalog(), game.state(), game.player(), unit)
        .contains(&position)
        .then_some(position)
}

/// Candidates for a position of the player (`leitung` or a function key) of a unit
/// (`unit_key`); `None` for a position the player does not have.
pub fn manager_market(game: &Game, unit: &str, role: &str) -> Option<ManagerMarketView> {
    let c = game.catalog();
    let state: &GameState = game.state();
    let unit = unit_from_key(c, unit)?;
    let position = player_position(game, unit, role)?;
    let player: CompanyId = game.player();
    let seat = management::seat_country(c, state, player, unit)?;
    let continent = c.countries.get(seat).continent;
    let candidate = |id: ManagerId, m: &Manager| {
        let demand = management::salary_demand(c, state, player, m, &position);
        let (demand, current) = match &m.job {
            Some(j) => (
                j.salary.max(demand),
                Some(CurrentPositionView {
                    unit: unit_key(c, j.position.unit),
                    site: unit_site(j.position.unit),
                    role: role_key(&j.position.role),
                }),
            ),
            None => (demand, None),
        };
        CandidateView {
            manager: manager_view(game, id, m),
            demand_usd: usd(demand),
            current,
        }
    };
    let fits = |m: &Manager| match &position.role {
        Role::Specialist(f) => m.focus == *f,
        Role::Head => false,
    };
    let mut free: Vec<(bool, bool, &str, ManagerId)> = state
        .managers
        .iter()
        .filter(|(_, m)| m.job.is_none())
        .map(|(&id, m)| {
            let elsewhere = c.countries.get(m.home).continent != continent;
            (elsewhere, !fits(m), m.name.as_str(), id)
        })
        .collect();
    free.sort();
    let candidates = free
        .iter()
        .map(|&(_, _, _, id)| candidate(id, &state.managers[&id]))
        .collect();
    let own = state
        .managers
        .iter()
        .filter(|(_, m)| {
            m.job
                .as_ref()
                .is_some_and(|j| j.company == player && j.position != position)
        })
        .map(|(&id, m)| candidate(id, m))
        .collect();
    Some(ManagerMarketView {
        unit: unit_key(c, unit),
        site: unit_site(unit),
        role: role.to_owned(),
        kind_text: kind_text(state, unit),
        country: match unit {
            Unit::Continent(_) => None,
            _ => Some(c.countries.key(seat).to_owned()),
        },
        continent: c.continents.key(continent).to_owned(),
        candidates,
        own,
    })
}
