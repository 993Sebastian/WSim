//! The company's organisation (MA1; docs/BEDIENUNG.md, "Organisation"): the positions of
//! its sites and who holds them, and the market of managers for a position.

use serde::{Deserialize, Serialize};

use super::{iso, usd};
use crate::command::site_type_key;
use crate::decision::Topic;
use crate::game::Game;
use crate::management::{self, shown_level};
use crate::money::Money;
use crate::state::{
    CompanyId, ConcernStatus, GameState, Manager, ManagerId, Position, Role, SiteId,
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HolderView {
    pub manager: ManagerView,
    pub salary_usd: f64,
    pub since: String,
    /// What a dismissal costs now.
    pub severance_usd: f64,
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
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SiteOrgView {
    pub site: u32,
    /// Text key of the site type.
    pub kind_text: String,
    pub country: String,
    pub positions: Vec<PositionView>,
    /// Next check of the positions; none without a manager there.
    pub next_check: Option<String>,
    /// Topics nobody takes care of here: the player decides them.
    pub own_topics: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CountryOrgView {
    pub country: String,
    pub sites: Vec<SiteOrgView>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContinentOrgView {
    pub continent: String,
    pub countries: Vec<CountryOrgView>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrganisationView {
    /// False without manager data.
    pub enabled: bool,
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
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CurrentPositionView {
    pub site: u32,
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
    pub site: u32,
    pub role: String,
    pub kind_text: String,
    pub country: String,
    pub continent: String,
    /// Free candidates, those of the site's continent first.
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

fn manager_view(game: &Game, id: ManagerId, m: &Manager) -> ManagerView {
    let c = game.catalog();
    let skills = management::skill_keys(c)
        .into_iter()
        .map(|key| {
            let value = management::skill(m, &key).unwrap_or(0);
            let impression = m.impression.get(&key).copied().unwrap_or(0);
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

/// The positions of a site with the topics each takes care of, and the topics left to the
/// player.
fn site_view(game: &Game, site: SiteId) -> SiteOrgView {
    let c = game.catalog();
    let m = &c.management;
    let state = game.state();
    let s = &state.sites[site.index()];
    let held = |role: &Role| {
        management::holder(
            state,
            &Position {
                site,
                role: role.clone(),
            },
        )
    };
    let head = held(&Role::Head);
    let specialists = m.specialists_of(s.kind);
    let mut head_topics = Vec::new();
    let mut own_topics = Vec::new();
    let mut positions = Vec::new();
    let arising = |topics: &[Topic]| -> Vec<Topic> {
        topics
            .iter()
            .copied()
            .filter(|&t| management::arises(s.kind, t))
            .collect()
    };
    for (index, function) in m.functions.iter().enumerate() {
        let filled =
            specialists.contains(&index) && held(&Role::Specialist(function.key.clone())).is_some();
        if !filled {
            let topics = arising(&function.topics);
            head_topics.extend(topics.iter().copied());
            if head.is_none() {
                own_topics.extend(topics);
            }
        }
    }
    let holder_view = |id: ManagerId| {
        let manager = &state.managers[&id];
        let job = manager.job.as_ref().expect("holds the position");
        HolderView {
            manager: manager_view(game, id, manager),
            salary_usd: usd(job.salary),
            since: iso(job.since),
            severance_usd: usd(job.salary.scale(m.severance_months / 12.0)),
        }
    };
    let player = game.player();
    let position_view = |role: Role, topics: &[Topic], holder: Option<ManagerId>| {
        let position = Position { site, role };
        let ps = management::position_state(state, player, &position);
        let budget = holder
            .and_then(|id| state.managers[&id].job.as_ref())
            .map(|job| {
                let shares = management::budget_shares(c, state, player, &position);
                let (per_decision, per_year) =
                    management::budget(c, state, player, &position, job.salary);
                BudgetView {
                    shares,
                    defaults: management::default_shares(c, &position.role),
                    custom: ps.is_some_and(|p| p.budget.is_some()),
                    base_usd: usd(management::budget_base(state, player, site)),
                    per_decision_usd: usd(per_decision),
                    per_year_usd: usd(per_year),
                    spent_usd: usd(management::spent(state, player, &position)),
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
                x.company == player && x.position == position && x.status == ConcernStatus::Open
            })
            .count();
        PositionView {
            role: role_key(&position.role),
            topics: topic_keys(topics),
            holder: holder.map(holder_view),
            budget,
            log,
            quiet,
            open_concerns: u32::try_from(open_concerns).unwrap_or(u32::MAX),
        }
    };
    positions.push(position_view(Role::Head, &head_topics, head));
    for &index in specialists {
        let function = &m.functions[index];
        let role = Role::Specialist(function.key.clone());
        let holder = held(&role);
        positions.push(position_view(role, &arising(&function.topics), holder));
    }
    let staffed = positions.iter().any(|p| p.holder.is_some());
    SiteOrgView {
        site: site.0,
        kind_text: site_type_key(s.kind),
        country: c.countries.key(s.country).to_owned(),
        positions,
        next_check: staffed
            .then(|| management::next_check(c, site, state.date))
            .flatten()
            .map(iso),
        own_topics: topic_keys(&own_topics),
    }
}

/// The player's organisation: continents, countries and sites with their positions.
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
    if m.enabled() {
        for continent in c.continents.ids() {
            let mut countries: Vec<CountryOrgView> = Vec::new();
            for country in c.countries.ids() {
                if c.countries.get(country).continent != continent {
                    continue;
                }
                let sites: Vec<SiteOrgView> = own
                    .iter()
                    .filter(|&&s| state.sites[s.index()].country == country)
                    .map(|&s| site_view(game, s))
                    .collect();
                if !sites.is_empty() {
                    countries.push(CountryOrgView {
                        country: c.countries.key(country).to_owned(),
                        sites,
                    });
                }
            }
            if !countries.is_empty() {
                continents.push(ContinentOrgView {
                    continent: c.continents.key(continent).to_owned(),
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
    OrganisationView {
        enabled: m.enabled(),
        continents,
        managers: u32::try_from(employed().count()).unwrap_or(u32::MAX),
        salaries_usd: usd(employed().map(|j| j.salary).sum::<Money>()),
        check_days: m.levels.first().map_or(0, |l| l.check_days),
        severance_months: m.severance_months,
        candidates: u32::try_from(state.managers.values().filter(|x| x.job.is_none()).count())
            .unwrap_or(u32::MAX),
        budget_floor: m.budget_floor,
    }
}

/// The position of a site of the player by its role key, if it exists.
fn player_position(game: &Game, site: u32, role: &str) -> Option<Position> {
    let state = game.state();
    let s = state.sites.get(usize::try_from(site).ok()?)?;
    if s.owner != game.player() {
        return None;
    }
    let position = Position {
        site: SiteId(site),
        role: if role == "leitung" {
            Role::Head
        } else {
            Role::Specialist(role.to_owned())
        },
    };
    management::positions(game.catalog(), state, position.site)
        .contains(&position)
        .then_some(position)
}

/// Candidates for a position of the player (`leitung` or a function key); `None` for a
/// position the player's site does not have.
pub fn manager_market(game: &Game, site: u32, role: &str) -> Option<ManagerMarketView> {
    let position = player_position(game, site, role)?;
    let c = game.catalog();
    let state: &GameState = game.state();
    let s = &state.sites[position.site.index()];
    let continent = c.countries.get(s.country).continent;
    let player: CompanyId = game.player();
    let candidate = |id: ManagerId, m: &Manager| {
        let demand = management::salary_demand(c, state, m, &position);
        let (demand, current) = match &m.job {
            Some(j) => (
                j.salary.max(demand),
                Some(CurrentPositionView {
                    site: j.position.site.0,
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
        site,
        role: role.to_owned(),
        kind_text: site_type_key(s.kind),
        country: c.countries.key(s.country).to_owned(),
        continent: c.continents.key(continent).to_owned(),
        candidates,
        own,
    })
}
