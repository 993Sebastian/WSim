//! The mandate (Strategieauftrag) of a company to its board (MA5, docs/FORMELN.md): a
//! guideline that sets how aggressively the rules of its positions act, goals the
//! strategy review measures, limits for debt, countries and goods groups, and how often
//! the CEO reviews the strategy with the player. AI companies keep the default; their
//! rules use their own aggressiveness.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::command::{Command, CommandError};
use crate::deals::{self, DealObject};
use crate::decision::{Choice, Decision, Topic};
use crate::ids::{BranchId, ContinentId, CountryId, GoodsGroupId, Id};
use crate::money::Money;
use crate::state::{CompanyId, GameState};

/// What the board is to aim at.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Guideline {
    Growth,
    #[default]
    Profit,
    Safety,
    /// Market leadership in a goods group: its opportunities and offers first.
    Leadership(GoodsGroupId),
}

impl Guideline {
    /// Index into the aggressiveness per guideline of the data.
    pub fn index(self) -> usize {
        match self {
            Guideline::Growth => 0,
            Guideline::Profit => 1,
            Guideline::Safety => 2,
            Guideline::Leadership(_) => 3,
        }
    }

    /// Key of the texts (`auftrag.leitlinie.<key>`).
    pub fn key(self) -> &'static str {
        match self {
            Guideline::Growth => "wachstum",
            Guideline::Profit => "ertrag",
            Guideline::Safety => "sicherheit",
            Guideline::Leadership(_) => "marktfuehrung",
        }
    }
}

/// How often the CEO reviews the strategy with the player: at the end of every
/// calendar month, quarter, half year or year.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ReviewInterval {
    Monthly,
    #[default]
    Quarterly,
    HalfYearly,
    Yearly,
}

impl ReviewInterval {
    pub const ALL: [ReviewInterval; 4] = [
        ReviewInterval::Monthly,
        ReviewInterval::Quarterly,
        ReviewInterval::HalfYearly,
        ReviewInterval::Yearly,
    ];

    pub fn months(self) -> u32 {
        match self {
            ReviewInterval::Monthly => 1,
            ReviewInterval::Quarterly => 3,
            ReviewInterval::HalfYearly => 6,
            ReviewInterval::Yearly => 12,
        }
    }

    /// Key of the texts (`auftrag.ruecksprache.<key>`).
    pub fn key(self) -> &'static str {
        match self {
            ReviewInterval::Monthly => "monatlich",
            ReviewInterval::Quarterly => "quartalsweise",
            ReviewInterval::HalfYearly => "halbjaehrlich",
            ReviewInterval::Yearly => "jaehrlich",
        }
    }
}

/// The goals the strategy review measures, each optional.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Goals {
    /// Revenue growth per year (0.1 = 10 %).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub growth: Option<f64>,
    /// Result per revenue.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub margin: Option<f64>,
    /// Equity per total assets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equity_ratio: Option<f64>,
    /// Rank by revenue (1 = first).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<u32>,
}

/// How far the positions of a unit may go in a branch (MG10, docs/MANAGER_2.md 10.1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ReleaseStage {
    /// Everything within the budgets.
    #[default]
    Free,
    /// Operation is free, every expansion goes to the player as a concern.
    Ask,
    /// Operation and replacement only, no expansion.
    Hold,
    /// No expansion; existing sites keep running.
    Locked,
}

impl ReleaseStage {
    /// The key the views and texts use.
    pub fn key(self) -> &'static str {
        match self {
            ReleaseStage::Free => "frei",
            ReleaseStage::Ask => "rueckfrage",
            ReleaseStage::Hold => "halten",
            ReleaseStage::Locked => "gesperrt",
        }
    }
}

/// Where a release applies; the more special scope wins.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ReleaseScope {
    Company,
    Continent(ContinentId),
    Country(CountryId),
}

/// One cell of the branch release matrix: the stage of a branch (`None`: all other
/// branches) in a scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ReleaseCell {
    pub scope: ReleaseScope,
    pub branch: Option<BranchId>,
    pub stage: ReleaseStage,
}

/// The stage that rules a branch in a place: the cell of the country, else of its
/// continent, else of the company; in each scope the branch's own cell before the one for
/// all other branches. Free when nothing is set.
pub fn release_stage(
    catalog: &Catalog,
    cells: &[ReleaseCell],
    country: Option<CountryId>,
    continent: Option<ContinentId>,
    branch: Option<BranchId>,
) -> ReleaseStage {
    if cells.is_empty() {
        return ReleaseStage::Free;
    }
    let continent = continent.or_else(|| country.map(|c| catalog.countries.get(c).continent));
    let scopes = [
        country.map(ReleaseScope::Country),
        continent.map(ReleaseScope::Continent),
        Some(ReleaseScope::Company),
    ];
    for scope in scopes.into_iter().flatten() {
        let find = |b: Option<BranchId>| {
            cells
                .iter()
                .find(|c| c.scope == scope && c.branch == b)
                .map(|c| c.stage)
        };
        if let Some(stage) = branch.and_then(|b| find(Some(b))).or_else(|| find(None)) {
            return stage;
        }
    }
    ReleaseStage::Free
}

/// A company's mandate to its board.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Mandate {
    #[serde(default)]
    pub guideline: Guideline,
    #[serde(default)]
    pub goals: Goals,
    /// Loans per total assets at most after a loan.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_debt: Option<f64>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub blocked_countries: BTreeSet<CountryId>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub blocked_groups: BTreeSet<GoodsGroupId>,
    /// The release of branches per scope (MG10); empty = everything free.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub release: Vec<ReleaseCell>,
    /// The release also holds for takeovers of sites in branches that are not free (MG10).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub release_stakes: bool,
    #[serde(default)]
    pub review: ReviewInterval,
}

impl Mandate {
    pub fn is_default(&self) -> bool {
        *self == Mandate::default()
    }

    /// The goods group whose opportunities and offers come first.
    pub fn first_group(&self) -> Option<GoodsGroupId> {
        match self.guideline {
            Guideline::Leadership(g) => Some(g),
            _ => None,
        }
    }
}

/// The aggressiveness the guideline gives the rules of a company without an AI
/// character: that of the data, 0.5 without a management model.
pub fn aggressiveness(catalog: &Catalog, mandate: &Mandate) -> f64 {
    if !catalog.management.enabled() {
        return 0.5;
    }
    catalog.management.mandate.guidelines[mandate.guideline.index()]
}

/// Debt per total assets after borrowing `more` (docs/FORMELN.md, MA5).
pub fn debt_share(state: &GameState, company: CompanyId, more: Money) -> f64 {
    let c = &state.companies[company.index()];
    let loans: Money = c.loans.iter().map(|l| l.balance).sum::<Money>()
        + c.ledger.balance(crate::ledger::Account::Bonds)
        + more;
    let assets = c.ledger.total_assets() + more;
    if assets <= Money::ZERO {
        return if loans > Money::ZERO {
            f64::INFINITY
        } else {
            0.0
        };
    }
    loans.to_usd() / assets.to_usd()
}

/// The loans a company can still take up while its debt stays within `max` of the total
/// assets: L + m ≤ max · (A + m).
pub fn loan_room(state: &GameState, company: CompanyId, max: f64) -> Money {
    let c = &state.companies[company.index()];
    let loans: Money = c.loans.iter().map(|l| l.balance).sum::<Money>()
        + c.ledger.balance(crate::ledger::Account::Bonds);
    let assets = c.ledger.total_assets();
    if max >= 1.0 {
        // Any loan keeps the share below 1 while the assets cover the loans.
        return if loans <= assets {
            Money::from_usd(1.0e14).unwrap_or(Money::ZERO)
        } else {
            Money::ZERO
        };
    }
    let room = (assets.scale(max) - loans).scale(1.0 / (1.0 - max));
    room.max(Money::ZERO)
}

/// The loans an option takes up.
pub fn borrowed(choice: &Choice) -> Money {
    choice
        .steps
        .iter()
        .map(|s| match s.command {
            Command::TakeLoan { amount, .. } | Command::IssueBond { amount, .. } => amount,
            _ => Money::ZERO,
        })
        .sum()
}

/// Whether an option's loans would take the company's debt over the limit of its
/// mandate.
pub fn over_debt(state: &GameState, company: CompanyId, choice: &Choice) -> bool {
    let more = borrowed(choice);
    if more <= Money::ZERO {
        return false;
    }
    state.companies[company.index()]
        .mandate
        .max_debt
        .is_some_and(|max| debt_share(state, company, more) > max)
}

/// Whether an object of an offer lies in a blocked country or goods group: a site in a
/// blocked country or with products of a blocked group, an area of a blocked group or
/// with a site in a blocked country.
pub fn blocked_object(
    catalog: &Catalog,
    state: &GameState,
    mandate: &Mandate,
    seller: CompanyId,
    object: DealObject,
) -> bool {
    if mandate.blocked_countries.is_empty() && mandate.blocked_groups.is_empty() {
        return false;
    }
    let site_blocked = |site: crate::state::SiteId| {
        let s = &state.sites[site.index()];
        mandate.blocked_countries.contains(&s.country)
            || deals::site_groups(state, catalog, site)
                .iter()
                .any(|g| mandate.blocked_groups.contains(g))
    };
    match object {
        DealObject::Site(site) => site_blocked(site),
        DealObject::Area(group) => {
            mandate.blocked_groups.contains(&group)
                || deals::area_sites(state, catalog, seller, group)
                    .iter()
                    .any(|&s| {
                        mandate
                            .blocked_countries
                            .contains(&state.sites[s.index()].country)
                    })
        }
        DealObject::License(_) => false,
    }
}

/// Whether an option of a company's positions goes against the bans of its mandate
/// (docs/FORMELN.md, MA5): founding a site in a blocked country, building for a product
/// of a blocked goods group, or an offer for a blocked object.
pub fn banned(catalog: &Catalog, state: &GameState, d: &Decision, choice: &Choice) -> bool {
    let mandate = &state.companies[d.company.index()].mandate;
    let grows = grows(d.topic);
    if grows
        && !choice.steps.is_empty()
        && matches!(
            release_of(catalog, state, d),
            ReleaseStage::Hold | ReleaseStage::Locked
        )
    {
        return true;
    }
    if grows
        && d.product.is_some_and(|p| {
            mandate
                .blocked_groups
                .contains(&catalog.products.get(p).goods_group)
        })
    {
        return true;
    }
    if mandate.release_stakes
        && choice.steps.iter().any(|s| {
            matches!(s.command, Command::MakeOffer { object: DealObject::Site(site), .. }
                if stake_held(catalog, state, mandate, site))
        })
    {
        return true;
    }
    if mandate.blocked_countries.is_empty() && mandate.blocked_groups.is_empty() {
        return false;
    }
    choice.steps.iter().any(|s| match s.command {
        Command::FoundSite { country, .. } | Command::FoundSiteInHall { country, .. } => {
            mandate.blocked_countries.contains(&country)
        }
        Command::FoundSiteOnPlot { plot, .. } => state
            .plots
            .get(plot.index())
            .is_some_and(|p| mandate.blocked_countries.contains(&p.country)),
        Command::MakeOffer { seller, object, .. } => {
            blocked_object(catalog, state, mandate, seller, object)
        }
        _ => false,
    })
}

/// Whether the release holds back a takeover of this site: its branch is held or locked
/// in its country (MG10, with the switch „also for stakes“).
fn stake_held(
    catalog: &Catalog,
    state: &GameState,
    mandate: &Mandate,
    site: crate::state::SiteId,
) -> bool {
    let s = &state.sites[site.index()];
    matches!(
        release_stage(catalog, &mandate.release, Some(s.country), None, s.branch),
        ReleaseStage::Hold | ReleaseStage::Locked
    )
}

fn grows(topic: Topic) -> bool {
    matches!(
        topic,
        Topic::Expansion | Topic::Power | Topic::Deposit | Topic::Bottleneck
    )
}

/// The branch release that rules a decision: the branch of its product at its place.
pub fn release_of(catalog: &Catalog, state: &GameState, d: &Decision) -> ReleaseStage {
    let cells = &state.companies[d.company.index()].mandate.release;
    if cells.is_empty() {
        return ReleaseStage::Free;
    }
    let (country, continent) = match crate::management::place_of(state, d) {
        Some(crate::state::Unit::Site(s)) => (state.sites.get(s.index()).map(|x| x.country), None),
        Some(crate::state::Unit::Country(c)) => (Some(c), None),
        Some(crate::state::Unit::Continent(c)) => (None, Some(c)),
        _ => (None, None),
    };
    let branch = d.product.map(|p| catalog.products.get(p).branch);
    release_stage(catalog, cells, country, continent, branch)
}

/// Whether a decision's expansion goes to the player although the budget allows it.
pub fn must_ask(catalog: &Catalog, state: &GameState, d: &Decision) -> bool {
    grows(d.topic) && release_of(catalog, state, d) == ReleaseStage::Ask
}

/// `SetMandate`: the company's mandate to its board, checked against the bounds of
/// docs/FORMELN.md (MA5).
pub(crate) fn set(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    mandate: &Mandate,
) -> Result<(), CommandError> {
    if !catalog.management.enabled() || !valid(catalog, mandate) {
        return Err(CommandError::InvalidMandate);
    }
    state.companies[company.index()].mandate = mandate.clone();
    Ok(())
}

fn valid(catalog: &Catalog, m: &Mandate) -> bool {
    let between = |x: Option<f64>, lo: f64, hi: f64| {
        x.is_none_or(|x| x.is_finite() && (lo..=hi).contains(&x))
    };
    let groups = catalog.goods_groups.len();
    between(m.goals.growth, -1.0, 10.0)
        && between(m.goals.margin, -1.0, 1.0)
        && between(m.goals.equity_ratio, 0.0, 1.0)
        && m.goals.rank.is_none_or(|r| r >= 1)
        && between(m.max_debt, 0.0, 1.0)
        && m.first_group().is_none_or(|g| g.index() < groups)
        && m.blocked_groups.iter().all(|g| g.index() < groups)
        && m.blocked_countries
            .iter()
            .all(|c| c.index() < catalog.countries.len())
        && m.release.iter().all(|c| {
            c.branch.is_none_or(|b| b.index() < catalog.branches.len())
                && match c.scope {
                    ReleaseScope::Company => true,
                    ReleaseScope::Continent(x) => x.index() < catalog.continents.len(),
                    ReleaseScope::Country(x) => x.index() < catalog.countries.len(),
                }
        })
        && m.release
            .windows(2)
            .all(|w| (w[0].scope, w[0].branch) < (w[1].scope, w[1].branch))
}
