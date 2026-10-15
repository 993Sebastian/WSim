//! PD9 – state programs that pay a share of the project cost and the bonus for the first
//! company to release a product (docs/FORMELN.md).

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::ledger::{Account, CostCenter, CostType};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::projects;
use crate::state::{Brand, GameState, Reputation, SiteId};

/// The program a project was started under, frozen at the start.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Subsidy {
    /// Index into the programs of the data.
    pub program: u32,
    pub share: f64,
    pub cap: Money,
    pub received: Money,
}

/// The best program for a project that starts now at `site`.
pub(crate) fn pick_subsidy(
    state: &GameState,
    catalog: &Catalog,
    site: SiteId,
    date: Date,
) -> Option<Subsidy> {
    let m = catalog.research_model.projects.as_ref()?;
    let level = projects::price_level(state, site);
    let (i, p) = m
        .subsidies
        .iter()
        .enumerate()
        .filter(|(_, p)| p.from_year <= date.year() && date.year() <= p.to_year && p.share > 0.0)
        .max_by(|a, b| a.1.share.total_cmp(&b.1.share).then(b.0.cmp(&a.0)))?;
    Some(Subsidy {
        program: u32::try_from(i).unwrap_or(u32::MAX),
        share: p.share,
        cap: Money::from_usd(p.cap_usd * level)?,
        received: Money::ZERO,
    })
}

/// The grant for a booking of `cost` on the project; it is paid out to the company.
pub(crate) fn grant(state: &mut GameState, index: usize, cost: Money) {
    let p = &mut state.dev_projects[index];
    let Some(s) = p.subsidy.as_mut() else {
        return;
    };
    let left = s.cap - s.received;
    let want = cost.scale(s.share);
    let pay = if want < left { want } else { left };
    if pay <= Money::ZERO {
        return;
    }
    s.received += pay;
    let (company, site) = (p.company, p.site);
    state.companies[company.index()].ledger.income(
        CostType::Research,
        CostCenter::site(site),
        Account::Cash,
        pay,
    );
}

/// At the release: the first company to know the product gets awareness and reputation in
/// the country of its laboratory.
pub(crate) fn first_mover(
    state: &mut GameState,
    catalog: &Catalog,
    index: usize,
    already_known: bool,
) -> Vec<Message> {
    let Some(f) = catalog
        .research_model
        .projects
        .as_ref()
        .and_then(|m| m.first_mover.as_ref())
    else {
        return Vec::new();
    };
    if already_known {
        return Vec::new();
    }
    let (company, product, site) = {
        let p = &state.dev_projects[index];
        (p.company, p.product, p.site)
    };
    let country = state.sites[site.index()].country;
    let group = catalog.products.get(product).goods_group;
    state.dev_projects[index].first_mover = true;
    let c = &mut state.companies[company.index()];
    match c
        .brands
        .iter_mut()
        .find(|b| b.country == country && b.group == group)
    {
        Some(b) => b.awareness += f.awareness * (1.0 - b.awareness),
        None => c.brands.push(Brand {
            country,
            group,
            awareness: f.awareness,
        }),
    }
    match c.reputations.iter_mut().find(|r| r.country == country) {
        Some(r) => r.value = (r.value + f.reputation).min(1.0),
        None => c.reputations.push(Reputation {
            country,
            value: (0.5 + f.reputation).min(1.0),
            sold: 0.0,
            quality_sum: 0.0,
            cost: Money::ZERO,
            low_quality: false,
        }),
    }
    if !state.is_main(company) {
        return Vec::new();
    }
    vec![
        Message::new(MessageKind::Success, keys::PROJECT_FIRST_MOVER).with(
            "produkt",
            Param::TextKey(format!("produkt.{}", catalog.products.key(product))),
        ),
    ]
}
