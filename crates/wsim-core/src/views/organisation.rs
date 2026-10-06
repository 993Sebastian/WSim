//! The company's organisation (MA1; docs/BEDIENUNG.md, "Organisation"): the positions of
//! its sites and who holds them, and the market of managers for a position.

use serde::{Deserialize, Serialize};

use super::{iso, usd};
use crate::catalog::SiteType;
use crate::command::site_type_key;
use crate::decision::Topic;
use crate::game::Game;
use crate::management::{self, shown_level};
use crate::money::Money;
use crate::state::{CompanyId, GameState, Manager, ManagerId, Position, Role, SiteId};

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PositionView {
    /// `leitung` or the key of the function of a specialist position.
    pub role: String,
    /// Topics (`thema.<key>`) the position takes care of now; for an empty position those
    /// it would take over.
    pub topics: Vec<String>,
    pub holder: Option<HolderView>,
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
    // The routine of the sites does not run in laboratories (their topics follow with
    // the budgets of MA2).
    let runs = s.kind != SiteType::ResearchCenter;
    for (index, function) in m.functions.iter().enumerate().filter(|_| runs) {
        let filled =
            specialists.contains(&index) && held(&Role::Specialist(function.key.clone())).is_some();
        if !filled {
            head_topics.extend(function.topics.iter().copied());
            if head.is_none() {
                own_topics.extend(function.topics.iter().copied());
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
    positions.push(PositionView {
        role: "leitung".into(),
        topics: topic_keys(&head_topics),
        holder: head.map(holder_view),
    });
    for &index in specialists {
        let function = &m.functions[index];
        positions.push(PositionView {
            role: function.key.clone(),
            topics: topic_keys(&function.topics),
            holder: held(&Role::Specialist(function.key.clone())).map(holder_view),
        });
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
