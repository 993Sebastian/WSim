//! Research (Lastenheft §7.1; formulas in docs/FORMELN.md, M9).
//!
//! Research centers work on one technology each; their researchers (academics of the
//! technology's field) collect research points for the company. A technology is
//! acquired once the points reach its effort, which depends on how far the research is
//! ahead of history or how long ago the technology was invented.

use crate::calendar::Date;
use crate::catalog::{Catalog, SiteType};
use crate::development;
use crate::ids::{Id, ProductId, TechnologyId};
use crate::ledger::{Account, CostCenter, CostType};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::state::{CompanyId, GameState, SiteId};

/// Research points a company needs for a technology, with the reason.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Effort {
    /// Points at the historical invention year (data).
    pub base: f64,
    /// Above 1 when ahead of history, below 1 for latecomers.
    pub factor: f64,
    pub points: f64,
}

/// Effort for a technology on `date`; `None` for technologies known from the start.
pub fn effort(
    catalog: &Catalog,
    state: &GameState,
    technology: TechnologyId,
    date: Date,
) -> Option<Effort> {
    let t = catalog.technologies.get(technology);
    let base = t.research_effort?;
    let model = &catalog.research_model;
    let now = date.year_fraction();
    let historical = f64::from(t.invention_year);
    let first = state
        .inventions
        .get(technology)
        .map(|d| d.year_fraction())
        .filter(|&y| y < historical);
    let invented = first.unwrap_or(historical);
    let factor = if now < invented {
        let ahead = (historical - now) * state.settings.research_ahead_factor;
        crate::math::pow(model.ahead_base, ahead)
    } else {
        crate::math::pow(1.0 - model.latecomer_discount, now - invented).max(model.latecomer_min)
    };
    Some(Effort {
        base,
        factor,
        points: base * factor,
    })
}

/// Whether a company could start research on a technology: not yet known, all
/// prerequisites known.
pub fn can_research(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    technology: TechnologyId,
) -> bool {
    !state.knows(catalog, company, technology)
        && catalog
            .technologies
            .get(technology)
            .prerequisites
            .iter()
            .all(|&p| state.knows(catalog, company, p))
}

/// Researchers a research center wants for its current project, by labor group.
pub(crate) fn wanted_researchers(
    catalog: &Catalog,
    state: &GameState,
    site: SiteId,
    date: Date,
) -> f64 {
    let s = &state.sites[site.index()];
    if s.kind != SiteType::ResearchCenter || development::idle(s) {
        return 0.0;
    }
    s.slots
        .iter()
        .filter(|sl| sl.operating(date))
        .map(|sl| sl.full_runs(catalog) * sl.utilization)
        .sum()
}

/// One day of research for all companies. Returns messages for the player.
pub(crate) fn simulate_day(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let model = &catalog.research_model;
    let mut messages = crate::product_research::steer(state, catalog);
    messages.extend(develop(state, catalog, date));
    for index in 0..state.sites.len() {
        let s = &state.sites[index];
        let Some(technology) = s.research else {
            continue;
        };
        let owner = s.owner;
        if state.companies[owner.index()].bankrupt {
            continue;
        }
        if state.knows(catalog, owner, technology) {
            state.sites[index].research = None;
            continue;
        }
        let field = catalog.technologies.get(technology).field;
        let Some(group) = model.researchers.get(field.index()).copied().flatten() else {
            continue;
        };
        // The researchers not given to products work on the technology (Q1).
        let researchers = *s.workforce.get(group) * development::technology_share(s);
        if researchers <= 0.0 {
            continue;
        }
        let country = state.countries.get(s.country);
        let efficiency = country
            .research_efficiency
            .get(field.index())
            .copied()
            .unwrap_or(1.0);
        let material =
            Money::from_usd(researchers * model.material_usd_per_day * country.price_level)
                .unwrap_or(Money::ZERO);
        let site = SiteId(u32::try_from(index).expect("site count fits u32"));
        let company = &mut state.companies[owner.index()];
        company.ledger.expense(
            CostType::Research,
            CostCenter::site(site),
            Account::Cash,
            material,
        );
        *company.research.entry(technology).or_default() += researchers * efficiency;
    }

    for c in 0..state.companies.len() {
        let done: Vec<TechnologyId> = state.companies[c]
            .research
            .iter()
            .filter(|&(&t, &points)| {
                effort(catalog, state, t, date).is_some_and(|e| points >= e.points)
            })
            .map(|(&t, _)| t)
            .collect();
        for technology in done {
            let company = &mut state.companies[c];
            company.research.remove(&technology);
            company.technologies.insert(technology);
            let first = state.inventions.get_mut(technology);
            let new_to_the_world = first.is_none();
            if new_to_the_world {
                *first = Some(date);
                // The first inventor may patent it (P7).
                let id = CompanyId(u32::try_from(c).expect("company count fits u32"));
                messages.extend(crate::patents::claim(state, catalog, id, technology, date));
            }
            if new_to_the_world && !state.main_company.is_some_and(|p| p.index() == c) {
                messages.push(
                    Message::new(MessageKind::Info, keys::AI_INVENTION)
                        .with("firma", Param::Text(state.companies[c].name.clone()))
                        .with(
                            "technologie",
                            Param::TextKey(format!(
                                "technologie.{}",
                                catalog.technologies.key(technology)
                            )),
                        ),
                );
            }
            for site in &mut state.sites {
                if site.owner.index() == c && site.research == Some(technology) {
                    site.research = None;
                }
            }
            if state.main_company.is_some_and(|p| p.index() == c) {
                messages.push(
                    Message::new(MessageKind::Success, keys::RESEARCH_DONE).with(
                        "technologie",
                        Param::TextKey(format!(
                            "technologie.{}",
                            catalog.technologies.key(technology)
                        )),
                    ),
                );
            }
        }
    }
    messages
}

/// Researchers of a center work on the development of its products (Q1): each product
/// gets its share of the researchers' points per day, material as for technologies. The
/// points add to the company's total and to the progress towards the next generation.
fn develop(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let model = &catalog.research_model;
    let mut working: Vec<(CompanyId, ProductId)> = Vec::new();
    let mut leading_before: Vec<(ProductId, bool)> = Vec::new();
    for index in 0..state.sites.len() {
        manufacture(state, catalog, index);
        let s = &state.sites[index];
        let shares = development::site_shares(s);
        if shares.is_empty() {
            continue;
        }
        let owner = s.owner;
        if state.companies[owner.index()].bankrupt {
            continue;
        }
        for (product, share) in shares {
            if !development::can_develop(catalog, state, owner, product) {
                let s = &mut state.sites[index];
                s.development_shares.remove(&product);
                if s.development == Some(product) {
                    s.development = None;
                }
                continue;
            }
            let Some((field, _)) = development::basis(catalog, product) else {
                continue;
            };
            let Some(group) = model.researchers.get(field.index()).copied().flatten() else {
                continue;
            };
            let s = &state.sites[index];
            let researchers = *s.workforce.get(group) * share;
            if researchers <= 0.0 {
                continue;
            }
            let country = state.countries.get(s.country);
            let efficiency = country
                .research_efficiency
                .get(field.index())
                .copied()
                .unwrap_or(1.0);
            let material =
                Money::from_usd(researchers * model.material_usd_per_day * country.price_level)
                    .unwrap_or(Money::ZERO);
            let site = SiteId(u32::try_from(index).expect("site count fits u32"));
            state.companies[owner.index()].ledger.expense(
                CostType::Research,
                CostCenter::site(site),
                Account::Cash,
                material,
            );
            if state.is_main(owner) && !leading_before.iter().any(|&(p, _)| p == product) {
                let lead = development::leader_points(state, product);
                let own = development::points(state, owner, product);
                leading_before.push((product, lead > 0.0 && own >= lead));
            }
            development::add_points(state, owner, product, researchers * efficiency);
            if !working.contains(&(owner, product)) {
                working.push((owner, product));
            }
        }
    }

    let mut messages = Vec::new();
    // The player took the lead of a product (Q1).
    for (product, was_leading) in leading_before {
        let Some(player) = state.main_company else {
            continue;
        };
        if !was_leading
            && development::points(state, player, product)
                >= development::leader_points(state, product)
        {
            messages.push(
                Message::new(MessageKind::Success, keys::DEVELOPMENT_LEAD).with(
                    "produkt",
                    Param::TextKey(format!("produkt.{}", catalog.products.key(product))),
                ),
            );
        }
    }
    for (company, product) in working {
        let Some((next, needed)) = development::next_effort(catalog, state, company, product, date)
        else {
            continue;
        };
        let c = &mut state.companies[company.index()];
        if c.development.points.get(&product).copied().unwrap_or(0.0) < needed {
            continue;
        }
        c.development.points.remove(&product);
        c.development.levels.insert(product, next);
        let firsts = state.developments.get_mut(product);
        let first_in_world = firsts.len() < usize::from(next);
        if first_in_world {
            firsts.push(date);
        }
        // The last generation: research goes on, the level against the leader counts.
        let top = next >= model.development.levels;
        let product_key = || Param::TextKey(format!("produkt.{}", catalog.products.key(product)));
        // A successor model (B1): AI companies rename, the player gets a suggestion.
        let successor = crate::product_names::successor(catalog, state, company, product);
        if let Some(of) = state.news_of(company)
            && !state.is_main(company)
        {
            messages.push(
                Message::new(
                    MessageKind::Success,
                    if top {
                        keys::DEVELOPMENT_TOP
                    } else {
                        keys::DEVELOPMENT_DONE
                    },
                )
                .with("produkt", product_key())
                .with("stufe", Param::Integer(i64::from(next)))
                .of(of),
            );
        }
        if state.is_main(company) {
            let key = if top {
                keys::DEVELOPMENT_TOP
            } else {
                keys::DEVELOPMENT_DONE
            };
            messages.push(
                Message::new(MessageKind::Success, key)
                    .with("produkt", product_key())
                    .with("stufe", Param::Integer(i64::from(next))),
            );
            if let Some(name) = successor {
                let old = state.companies[company.index()]
                    .product_names
                    .get(&product)
                    .cloned()
                    .unwrap_or_default();
                messages.push(
                    Message::new(MessageKind::Info, keys::DEVELOPMENT_SUCCESSOR)
                        .with("produkt", product_key())
                        .with("alt", Param::Text(old))
                        .with("name", Param::Text(name)),
                );
            }
        } else if let Some(name) = successor {
            let rename = crate::command::Command::NameProduct {
                product,
                name: Some(name),
            };
            // A name taken meanwhile keeps the old one.
            let _ = crate::command::execute(state, catalog, company, &rename);
        }
        if !state.is_main(company) && first_in_world && player_offers(state, product) {
            messages.push(
                Message::new(MessageKind::Info, keys::DEVELOPMENT_RIVAL)
                    .with(
                        "firma",
                        Param::Text(state.companies[company.index()].name.clone()),
                    )
                    .with("produkt", product_key())
                    .with("stufe", Param::Integer(i64::from(next))),
            );
        }
    }
    messages
}

/// Whether the player sells a product somewhere.
pub(crate) fn player_offers(state: &GameState, product: ProductId) -> bool {
    state
        .sites
        .iter()
        .any(|s| state.is_main(s.owner) && s.offers.contains_key(&product))
}

/// Manufacturing research of a center (Q6): each product's share of the researchers
/// collects points as development does, material paid alike.
fn manufacture(state: &mut GameState, catalog: &Catalog, index: usize) {
    let model = &catalog.research_model;
    let s = &state.sites[index];
    let shares = development::site_process_shares(s);
    let owner = s.owner;
    if shares.is_empty() || state.companies[owner.index()].bankrupt {
        return;
    }
    for (product, share) in shares {
        if !development::can_develop(catalog, state, owner, product) {
            state.sites[index].process_shares.remove(&product);
            continue;
        }
        let Some((field, _)) = development::basis(catalog, product) else {
            continue;
        };
        let Some(group) = model.researchers.get(field.index()).copied().flatten() else {
            continue;
        };
        let s = &state.sites[index];
        let researchers = *s.workforce.get(group) * share;
        if researchers <= 0.0 {
            continue;
        }
        let country = state.countries.get(s.country);
        let efficiency = country
            .research_efficiency
            .get(field.index())
            .copied()
            .unwrap_or(1.0);
        let material =
            Money::from_usd(researchers * model.material_usd_per_day * country.price_level)
                .unwrap_or(Money::ZERO);
        let site = SiteId(u32::try_from(index).expect("site count fits u32"));
        state.companies[owner.index()].ledger.expense(
            CostType::Research,
            CostCenter::site(site),
            Account::Cash,
            material,
        );
        development::add_process_points(state, owner, product, researchers * efficiency);
    }
}
