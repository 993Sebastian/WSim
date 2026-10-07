//! Start-ups (SU1; formulas in docs/FORMELN.md).
//!
//! Inventors and young companies outside the game work on a technology not yet invented
//! or on the next development level of a product. Every phase needs a funding round of
//! new shares; most of them fail. A success brings the technology or the level into the
//! world earlier.

use std::collections::BTreeSet;

use crate::calendar::Date;
use crate::catalog::{Catalog, DepartmentKind, VentureModel};
use crate::ids::{CountryId, ProductId, TechnologyId};
use crate::management;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{
    CompanyId, GameState, Holder, Stake, Venture, VentureFailure, VentureStatus, VentureTarget,
};

/// Start-ups at the start of a month: rounds and phases of the active ones, then the new
/// ones, then closed ones leave the list. Returns messages for the player.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let m = &catalog.ventures;
    if m.phases.is_empty() {
        return Vec::new();
    }
    let month = management::month_number(date);
    let mut messages = Vec::new();
    for i in 0..state.ventures.len() {
        if state.ventures[i].status != VentureStatus::Active {
            continue;
        }
        let stream = Stream::Venture {
            id: state.ventures[i].id,
            month,
        };
        let mut rng = SimRng::for_stream(state.settings.seed, stream);
        messages.extend(advance(state, catalog, i, date, &mut rng));
    }
    found(state, catalog, date, month);
    prune(state, m, date);
    messages
}

/// Chance of a phase for a start-up with a lead of `lead` years.
pub fn phase_chance(m: &VentureModel, phase: usize, lead: f64) -> f64 {
    m.phases.get(phase).map_or(0.0, |p| {
        (p.chance * (1.0 - m.lead_chance * lead)).max(m.chance_min)
    })
}

/// Chance of success from the current phase on: the chance of the current phase, fixed
/// at its start, times those of the phases still to come.
pub fn success_chance(m: &VentureModel, v: &Venture) -> f64 {
    if v.status != VentureStatus::Active {
        return 0.0;
    }
    (v.phase + 1..m.phases.len())
        .map(|p| phase_chance(m, p, v.lead))
        .fold(v.chance, |a, c| a * c)
}

/// How well a company's strategy department sees start-ups: quality times coverage
/// (ZA2); `None` where it does not work.
pub fn insight(catalog: &Catalog, state: &GameState, company: CompanyId) -> Option<f64> {
    if state.companies[company.index()].departments.is_empty() {
        return None;
    }
    crate::central::performance(catalog, state, company, DepartmentKind::Strategy)
        .map(|p| p.quality * p.coverage)
}

/// What a company sees of the chance of success, and whether its strategy department
/// works on it: blurred by the draw of the start-up, the less the better the department
/// works. Without a working department the company sees only the level.
pub fn shown_chance(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    v: &Venture,
) -> (f64, bool) {
    let m = &catalog.ventures;
    let work = insight(catalog, state, company);
    let blur = m.blur * (1.0 - work.unwrap_or(0.0));
    let shown = (success_chance(m, v) * (1.0 + v.blur * blur)).clamp(0.0, 1.0);
    (shown, work.is_some())
}

/// Whether the world got to the target of a start-up first: the technology is invented,
/// in history or in the game, or someone reached the level.
fn overtaken(state: &GameState, catalog: &Catalog, target: VentureTarget, date: Date) -> bool {
    match target {
        VentureTarget::Technology(t) => {
            state.inventions.get(t).is_some()
                || catalog.technologies.get(t).invention_year <= date.year()
        }
        VentureTarget::Development { product, level } => {
            state.developments.get(product).len() >= usize::from(level)
        }
    }
}

/// One month of an active start-up: it is overtaken, its open round finds investors or
/// runs out, or its funded phase is decided.
fn advance(
    state: &mut GameState,
    catalog: &Catalog,
    i: usize,
    date: Date,
    rng: &mut SimRng,
) -> Option<Message> {
    let m = &catalog.ventures;
    if overtaken(state, catalog, state.ventures[i].target, date) {
        let v = &mut state.ventures[i];
        v.status = VentureStatus::Failed(date, VentureFailure::Overtaken);
        v.round_until = None;
        v.phase_until = None;
        return None;
    }
    let v = &mut state.ventures[i];
    if let Some(until) = v.round_until {
        if rng.chance(m.investor_chance) {
            let need = v.capital - v.raised;
            let valuation = m.phases.get(v.phase).map_or(1.0, |p| p.valuation);
            issue(v, &[(Holder::Investors, need)], valuation);
            v.raised = v.capital;
            v.round_until = None;
            let months = m.phases.get(v.phase).map_or(0, |p| p.months);
            v.phase_until = Some(date.add_months(months));
        } else if date >= until {
            v.status = VentureStatus::Failed(date, VentureFailure::Funding);
            v.round_until = None;
        }
        return None;
    }
    if v.phase_until.is_none_or(|until| date < until) {
        return None;
    }
    if !rng.chance(v.chance) {
        v.status = VentureStatus::Failed(date, VentureFailure::Phase);
        v.phase_until = None;
        return None;
    }
    let next = v.phase + 1;
    if next < m.phases.len() {
        let gdp = state
            .countries
            .get(state.ventures[i].country)
            .gdp_per_capita_usd;
        start_phase(m, &mut state.ventures[i], next, gdp, date);
        return None;
    }
    succeed(state, catalog, i, date)
}

/// A phase begins: its capital and chance are fixed and its round opens.
fn start_phase(m: &VentureModel, v: &mut Venture, phase: usize, gdp_per_capita: f64, date: Date) {
    let Some(p) = m.phases.get(phase) else {
        return;
    };
    let (low, high) = m.capital_factor;
    let income = (gdp_per_capita / m.reference_gdp_usd).clamp(low, high);
    v.phase = phase;
    v.capital = p.capital.scale(income * (1.0 + m.lead_capital * v.lead));
    v.chance = phase_chance(m, phase, v.lead);
    v.raised = Money::ZERO;
    v.round_until = Some(date.add_months(m.deadline_months));
    v.phase_until = None;
}

/// A funded round: the givers get shares for their amounts at the value after the round,
/// all earlier owners keep the value before it.
fn issue(v: &mut Venture, givers: &[(Holder, Money)], valuation: f64) {
    let before = v.capital.to_usd() * valuation;
    let raised: f64 = givers.iter().map(|(_, a)| a.to_usd()).sum();
    let after = before + raised;
    if after <= 0.0 || raised <= 0.0 {
        return;
    }
    let keep = before / after;
    for s in &mut v.owners {
        s.share *= keep;
    }
    for &(holder, amount) in givers {
        let share = amount.to_usd() / after;
        match v.owners.iter_mut().find(|s| s.holder == holder) {
            Some(s) => s.share += share,
            None => v.owners.push(Stake { holder, share }),
        }
    }
}

/// All phases passed: the technology counts as invented from today if that is earlier
/// than so far, or the level as reached.
fn succeed(state: &mut GameState, catalog: &Catalog, i: usize, date: Date) -> Option<Message> {
    let v = &mut state.ventures[i];
    v.status = VentureStatus::Succeeded(date);
    v.phase = catalog.ventures.phases.len();
    v.round_until = None;
    v.phase_until = None;
    let (name, country, target) = (v.name.clone(), v.country, v.target);
    let named = |key: &str| {
        Message::new(MessageKind::Info, key)
            .with("name", Param::Text(name.clone()))
            .with(
                "land",
                Param::Country(catalog.countries.key(country).to_owned()),
            )
    };
    match target {
        VentureTarget::Technology(t) => {
            let historical = f64::from(catalog.technologies.get(t).invention_year);
            let first = state.inventions.get_mut(t);
            if first.is_some() || date.year_fraction() >= historical {
                return None;
            }
            *first = Some(date);
            Some(named(keys::VENTURE_INVENTION).with(
                "technologie",
                Param::TextKey(format!("technologie.{}", catalog.technologies.key(t))),
            ))
        }
        VentureTarget::Development { product, level } => {
            let firsts = state.developments.get_mut(product);
            if firsts.len() + 1 != usize::from(level) {
                return None;
            }
            firsts.push(date);
            crate::research::player_offers(state, product).then(|| {
                named(keys::VENTURE_DEVELOPMENT)
                    .with(
                        "produkt",
                        Param::TextKey(format!("produkt.{}", catalog.products.key(product))),
                    )
                    .with("stufe", Param::Integer(i64::from(level)))
                    .with(
                        "jahre",
                        Param::Number(catalog.research_model.development.public_domain_years),
                    )
            })
        }
    }
}

/// Technologies a new start-up could work on, with their lead in years: neither invented
/// in history nor in the game, all prerequisites invented, at most the lead of the data
/// ahead and no target of an active start-up.
fn new_technologies(state: &GameState, catalog: &Catalog, date: Date) -> Vec<(TechnologyId, f64)> {
    let m = &catalog.ventures;
    let year = date.year();
    let invented = |t: TechnologyId| {
        state.inventions.get(t).is_some() || catalog.technologies.get(t).invention_year <= year
    };
    let targeted: BTreeSet<TechnologyId> = state
        .ventures
        .iter()
        .filter(|v| v.status == VentureStatus::Active)
        .filter_map(|v| match v.target {
            VentureTarget::Technology(t) => Some(t),
            VentureTarget::Development { .. } => None,
        })
        .collect();
    let max = i64::from(m.lead_years_max);
    catalog
        .technologies
        .iter()
        .filter(|(id, t)| {
            let lead = i64::from(t.invention_year) - i64::from(year);
            t.research_effort.is_some()
                && lead <= max
                && !invented(*id)
                && !targeted.contains(id)
                && t.prerequisites.iter().all(|&p| invented(p))
        })
        .map(|(id, t)| (id, f64::from(t.invention_year - year)))
        .collect()
}

/// Products a new start-up could improve, with the level: those that can be developed
/// and are made in the world, whose next level no one has reached and no active start-up
/// works on.
fn improvements(state: &GameState, catalog: &Catalog, date: Date) -> Vec<(ProductId, u8)> {
    let levels = catalog.research_model.development.levels;
    let made: BTreeSet<ProductId> = state
        .sites
        .iter()
        .filter(|s| !state.companies[s.owner.index()].bankrupt)
        .flat_map(|s| s.slots.iter())
        .filter(|sl| sl.operating(date))
        .filter_map(|sl| sl.recipe)
        .map(|r| catalog.recipes.get(r).product)
        .collect();
    let targeted: BTreeSet<ProductId> = state
        .ventures
        .iter()
        .filter(|v| v.status == VentureStatus::Active)
        .filter_map(|v| match v.target {
            VentureTarget::Development { product, .. } => Some(product),
            VentureTarget::Technology(_) => None,
        })
        .collect();
    made.into_iter()
        .filter(|p| !targeted.contains(p))
        .filter_map(|p| {
            let next = u8::try_from(state.developments.get(p).len() + 1).ok()?;
            (next <= levels && crate::development::basis(catalog, p).is_some()).then_some((p, next))
        })
        .collect()
}

/// New start-ups of a month (docs/FORMELN.md, SU1).
fn found(state: &mut GameState, catalog: &Catalog, date: Date, month: u32) {
    let m = &catalog.ventures;
    let mean = m.per_year * state.settings.ventures / 12.0;
    if mean.is_nan() || mean <= 0.0 {
        return;
    }
    let mut rng = SimRng::for_stream(state.settings.seed, Stream::Ventures { month });
    let whole = mean.floor();
    // A handful a month; the cast is exact.
    let count = whole as usize + usize::from(rng.chance(mean - whole));
    if count == 0 {
        return;
    }
    let mut technologies = new_technologies(state, catalog, date);
    let mut products = improvements(state, catalog, date);
    let countries: Vec<(CountryId, f64)> = catalog
        .countries
        .ids()
        .map(|c| {
            let v = state.countries.get(c);
            (
                c,
                (v.population * v.gdp_per_capita_usd * v.development).max(0.0),
            )
        })
        .collect();
    if countries.is_empty() {
        return;
    }
    let total: f64 = countries.iter().map(|c| c.1).sum();
    let mut names: BTreeSet<String> = state.ventures.iter().map(|v| v.name.clone()).collect();
    for _ in 0..count {
        let new = rng.chance(m.new_share);
        let pick = |rng: &mut SimRng, n: usize| {
            usize::try_from(rng.below(u64::try_from(n).unwrap_or(1))).unwrap_or(0)
        };
        let (target, lead) = if (new || products.is_empty()) && !technologies.is_empty() {
            let (t, lead) = technologies.remove(pick(&mut rng, technologies.len()));
            (VentureTarget::Technology(t), lead)
        } else if !products.is_empty() {
            let (product, level) = products.remove(pick(&mut rng, products.len()));
            (VentureTarget::Development { product, level }, 0.0)
        } else {
            return;
        };
        let inventor = match target {
            VentureTarget::Technology(t) => m
                .inventors
                .iter()
                .find(|x| x.technology == t && !names.contains(&x.name)),
            VentureTarget::Development { .. } => None,
        };
        let (name, country) = match inventor {
            Some(x) => {
                names.insert(x.name.clone());
                (x.name.clone(), x.country)
            }
            None => {
                let country = management::pick_country(&mut rng, &countries, total);
                let name = management::manager_name(catalog, &mut rng, country, &mut names);
                (name, country)
            }
        };
        let blur = 2.0 * rng.next_f64() - 1.0;
        let mut v = Venture {
            id: state.next_venture,
            name,
            inventor: inventor.is_some(),
            country,
            target,
            lead,
            founded: date,
            phase: 0,
            capital: Money::ZERO,
            chance: 0.0,
            raised: Money::ZERO,
            round_until: None,
            phase_until: None,
            owners: Stake::sole(Holder::Private),
            status: VentureStatus::Active,
            blur,
        };
        let gdp = state.countries.get(country).gdp_per_capita_usd;
        start_phase(m, &mut v, 0, gdp, date);
        state.next_venture += 1;
        state.ventures.push(v);
    }
}

/// Closed start-ups leave the list after the years of the data; those of historical
/// inventors stay, so that each founds only once.
fn prune(state: &mut GameState, m: &VentureModel, date: Date) {
    let months = m.keep_years.saturating_mul(12);
    state.ventures.retain(|v| match v.status {
        VentureStatus::Active => true,
        VentureStatus::Succeeded(d) | VentureStatus::Failed(d, _) => {
            v.inventor || d.add_months(months) > date
        }
    });
}
