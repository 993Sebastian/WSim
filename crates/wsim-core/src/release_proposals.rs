//! Proposals of the managers to release a branch (MG10 §10.5, docs/FORMELN.md, MG10b):
//! once a year the CEO of the player's main company counts the chances the release holds
//! back; for a branch with at least `release_proposal_from` of them he proposes a looser
//! stage as a concern, at most once a year and branch.

use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::command::Command;
use crate::decision::{Choice, ChoiceKind, Decision, Topic};
use crate::ids::{BranchId, ContinentId, CountryId};
use crate::management;
use crate::mandate::{self, Mandate, ReleaseCell, ReleaseScope, ReleaseStage};
use crate::money::Money;
use crate::state::{
    CompanyId, Concern, ConcernOption, ConcernReason, ConcernStatus, GameState, Position, Role,
    Unit,
};

/// A place of a decision: country and continent, as far as known.
type Place = (Option<CountryId>, Option<ContinentId>);

/// The cell that rules a branch in a place: its scope, from the country up.
fn ruling_scope(
    catalog: &Catalog,
    cells: &[ReleaseCell],
    country: Option<CountryId>,
    continent: Option<ContinentId>,
    branch: BranchId,
) -> ReleaseScope {
    let continent = continent.or_else(|| country.map(|c| catalog.countries.get(c).continent));
    let scopes = [
        country.map(ReleaseScope::Country),
        continent.map(ReleaseScope::Continent),
        Some(ReleaseScope::Company),
    ];
    for scope in scopes.into_iter().flatten() {
        let has = |b: Option<BranchId>| cells.iter().any(|c| c.scope == scope && c.branch == b);
        if has(Some(branch)) || has(None) {
            return scope;
        }
    }
    ReleaseScope::Company
}

/// The mandate with the branch's own cell in the ruling scope at a stage.
pub fn with_stage(
    mandate: &Mandate,
    scope: ReleaseScope,
    branch: BranchId,
    stage: ReleaseStage,
) -> Mandate {
    let mut m = mandate.clone();
    match m
        .release
        .iter_mut()
        .find(|c| c.scope == scope && c.branch == Some(branch))
    {
        Some(cell) => cell.stage = stage,
        None => m.release.push(ReleaseCell {
            scope,
            branch: Some(branch),
            stage,
        }),
    }
    m.release.sort_by_key(|c| (c.scope, c.branch));
    m
}

/// January: the proposals of the year.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) {
    let model = &catalog.management.mandate;
    if date.month() != 1 || model.release_proposal_from == 0 {
        return;
    }
    let Some(me) = state.main_company else {
        return;
    };
    let c = &state.companies[me.index()];
    if c.bankrupt || c.mandate.release.is_empty() {
        return;
    }
    let Some(ceo) = management::ceo_of(state, me) else {
        return;
    };
    let year_ago = date.add_days(-365);
    state.release_asked.retain(|&(_, _, d)| d >= year_ago);
    // The chances the release holds back, per branch, with the place of the first.
    let mut held: Vec<(BranchId, u32, Place)> = Vec::new();
    for d in crate::ai::opportunities(state, catalog, me, 24) {
        let Some(product) = d.product else { continue };
        if !matches!(
            mandate::release_of(catalog, state, &d),
            ReleaseStage::Hold | ReleaseStage::Locked
        ) {
            continue;
        }
        let branch = catalog.products.get(product).branch;
        let place = match management::place_of(state, &d) {
            Some(Unit::Site(s)) => (state.sites.get(s.index()).map(|x| x.country), None),
            Some(Unit::Country(c)) => (Some(c), None),
            Some(Unit::Continent(k)) => (None, Some(k)),
            _ => (None, None),
        };
        match held.iter_mut().find(|h| h.0 == branch) {
            Some(h) => h.1 += 1,
            None => held.push((branch, 1, place)),
        }
    }
    for (branch, count, (country, continent)) in held {
        if count < model.release_proposal_from
            || state
                .release_asked
                .iter()
                .any(|&(c, b, _)| c == me && b == branch)
        {
            continue;
        }
        ask(
            state,
            catalog,
            (me, ceo),
            (branch, country, continent),
            date,
        );
    }
}

pub(crate) fn ask(
    state: &mut GameState,
    catalog: &Catalog,
    (me, ceo): (CompanyId, crate::state::ManagerId),
    (branch, country, continent): (BranchId, Option<CountryId>, Option<ContinentId>),
    date: Date,
) {
    let mandate = state.companies[me.index()].mandate.clone();
    let scope = ruling_scope(catalog, &mandate.release, country, continent, branch);
    let set = |stage| Command::SetMandate {
        mandate: with_stage(&mandate, scope, branch, stage),
    };
    let id = state.next_concern;
    state.next_concern += 1;
    state.release_asked.push((me, branch, date));
    let zero = ConcernOption {
        amount: Money::ZERO,
        forecast: None,
        once: Money::ZERO,
    };
    state.concerns.push(Concern {
        id,
        company: me,
        position: Position::new(Unit::Board, Role::Head),
        manager: ceo,
        decision: Decision {
            topic: Topic::Expansion,
            company: me,
            site: None,
            product: None,
            choices: vec![
                Choice::one(ChoiceKind::Adjust, set(ReleaseStage::Free)),
                Choice::one(ChoiceKind::Adjust, set(ReleaseStage::Ask)),
                Choice::keep(),
            ],
            rule: 0,
        },
        recommended: 1,
        options: vec![zero.clone(), zero.clone(), zero],
        reason: ConcernReason::Release,
        path: Vec::new(),
        parts: Vec::new(),
        created: date,
        deadline: date
            .add_days(i32::try_from(catalog.management.concerns.deadline_days).unwrap_or(30)),
        status: ConcernStatus::Open,
        closed: None,
        memory: None,
        dissent: None,
        statement: None,
    });
}
