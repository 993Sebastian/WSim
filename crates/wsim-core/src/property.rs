//! Private real estate of the person (PV3, docs/FORMELN.md).

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::{Catalog, PropertyModel};
use crate::command::CommandError;
use crate::ids::CountryId;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{GameState, PrivateFlow};

/// Offset of the stream of the property market among the streams of the person.
const PROPERTY_STREAM: u32 = 4 << 24;

/// The kinds of property.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PropertyKind {
    Apartment,
    Multi,
    Office,
    Hall,
    Land,
}

impl PropertyKind {
    pub const ALL: [PropertyKind; 5] = [
        PropertyKind::Apartment,
        PropertyKind::Multi,
        PropertyKind::Office,
        PropertyKind::Hall,
        PropertyKind::Land,
    ];

    pub fn key(self) -> &'static str {
        match self {
            PropertyKind::Apartment => "wohnung",
            PropertyKind::Multi => "mehrfamilienhaus",
            PropertyKind::Office => "buero",
            PropertyKind::Hall => "halle",
            PropertyKind::Land => "grundstueck",
        }
    }
}

/// A mortgage on a property.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mortgage {
    pub balance: Money,
    /// Yearly rate, fixed.
    pub rate: f64,
    /// What is repaid each month.
    pub repay: Money,
}

/// A property of the person.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Property {
    pub kind: PropertyKind,
    pub country: CountryId,
    /// Quality of the location and size, 0.7–1.4.
    pub quality: f64,
    /// Condition 0–100.
    pub condition: f64,
    /// What it cost without the costs of buying.
    pub paid: Money,
    /// The rent as a multiple of the market rent.
    pub rent_factor: f64,
    /// No rent until this day while it is renovated.
    pub renovating_until: Option<Date>,
    pub mortgage: Option<Mortgage>,
    /// The site of the person's firm it is let to (PV5c).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant: Option<u32>,
}

/// A property for sale.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropertyOffer {
    pub id: u32,
    pub kind: PropertyKind,
    pub country: CountryId,
    pub quality: f64,
    pub condition: f64,
    pub price: Money,
}

fn model(catalog: &Catalog) -> Option<&PropertyModel> {
    catalog
        .person
        .assets
        .as_ref()
        .and_then(|a| a.property.as_ref())
}

/// Own sites in a country raise demand, rents and prices.
fn site_effect(catalog: &Catalog, state: &GameState, m: &PropertyModel, country: CountryId) -> f64 {
    let _ = catalog;
    let n = state
        .sites
        .iter()
        .filter(|s| s.country == country && crate::private::is_controlled(state, s.owner))
        .count();
    // Site counts are small.
    #[allow(clippy::cast_precision_loss)]
    (m.site_effect * n as f64).min(m.site_effect_max)
}

/// The value of a property (or offer data) today.
pub fn value_of(
    catalog: &Catalog,
    state: &GameState,
    m: &PropertyModel,
    (kind, country, quality, condition): (PropertyKind, CountryId, f64, f64),
) -> Money {
    let wage = crate::management::yearly_wage(catalog, state, country);
    let z = 1.0 - m.condition_weight * (1.0 - condition / 100.0);
    let usd = m.kinds[kind as usize].price_years
        * wage
        * quality
        * z
        * (1.0 + site_effect(catalog, state, m, country));
    Money::from_usd(usd).unwrap_or(Money::ZERO)
}

pub fn value(catalog: &Catalog, state: &GameState, m: &PropertyModel, p: &Property) -> Money {
    value_of(
        catalog,
        state,
        m,
        (p.kind, p.country, p.quality, p.condition),
    )
}

/// The market rent per year.
pub fn market_rent(catalog: &Catalog, state: &GameState, m: &PropertyModel, p: &Property) -> Money {
    value(catalog, state, m, p).scale(m.kinds[p.kind as usize].rent_yield)
}

/// The share of the year the property stands empty at its rent.
pub fn vacancy(m: &PropertyModel, p: &Property) -> f64 {
    (m.kinds[p.kind as usize].vacancy + m.vacancy_slope * (p.rent_factor - 1.0).max(0.0))
        .clamp(0.0, 1.0)
}

/// What the person has in property, less the mortgages.
pub fn equity(catalog: &Catalog, state: &GameState) -> Money {
    let Some(m) = model(catalog) else {
        return Money::ZERO;
    };
    state
        .person
        .portfolio
        .properties
        .iter()
        .map(|p| value(catalog, state, m, p) - p.mortgage.map_or(Money::ZERO, |h| h.balance))
        .sum()
}

/// The rate of a new mortgage.
pub fn mortgage_rate(catalog: &Catalog, state: &GameState, m: &PropertyModel) -> f64 {
    catalog
        .person
        .savings_rate
        .value(state.person.home, state.date.year_fraction())
        + m.mortgage_premium
}

/// A property action of the person.
pub(crate) fn apply(
    state: &mut GameState,
    catalog: &Catalog,
    action: crate::savings::Investment,
) -> Result<(), CommandError> {
    use crate::savings::Investment as I;
    let Some(m) = model(catalog).cloned() else {
        return Err(CommandError::NotPossible);
    };
    match action {
        I::BuyProperty { offer, mortgage } => {
            let i = state
                .assets
                .offers
                .iter()
                .position(|o| o.id == offer)
                .ok_or(CommandError::NotPossible)?;
            if !(0.0..=1.0 - m.equity_min + 1e-9).contains(&mortgage) {
                return Err(CommandError::NotPossible);
            }
            let o = state.assets.offers[i].clone();
            let loan = o.price.scale(mortgage);
            let costs = o.price.scale(m.costs);
            let own = o.price - loan + costs;
            if state.person.account.balance < own {
                return Err(CommandError::NotEnoughPrivateMoney {
                    needed: own - state.person.account.balance,
                });
            }
            state.assets.offers.remove(i);
            crate::private::book_flow(state, PrivateFlow::AssetBought, -(o.price - loan));
            crate::private::book_flow(state, PrivateFlow::InvestFee, -costs);
            let rate = mortgage_rate(catalog, state, &m);
            state.person.portfolio.properties.push(Property {
                kind: o.kind,
                country: o.country,
                quality: o.quality,
                condition: o.condition,
                paid: o.price,
                rent_factor: 1.0,
                renovating_until: None,
                tenant: None,
                mortgage: (loan > Money::ZERO).then(|| Mortgage {
                    balance: loan,
                    rate,
                    repay: loan.scale(m.repay_year / 12.0),
                }),
            });
        }
        I::SellProperty { index } => {
            let i = index as usize;
            if i >= state.person.portfolio.properties.len() {
                return Err(CommandError::NotPossible);
            }
            let p = state.person.portfolio.properties.remove(i);
            sell(state, catalog, &m, &p, m.sale_discount);
        }
        I::SetRent { index, factor } => {
            let p = state
                .person
                .portfolio
                .properties
                .get_mut(index as usize)
                .ok_or(CommandError::NotPossible)?;
            if !(m.rent_band[0]..=m.rent_band[1]).contains(&factor) {
                return Err(CommandError::NotPossible);
            }
            p.rent_factor = factor;
        }
        I::Renovate { index } => {
            let i = index as usize;
            let p = state
                .person
                .portfolio
                .properties
                .get(i)
                .ok_or(CommandError::NotPossible)?
                .clone();
            if p.kind == PropertyKind::Land || p.renovating_until.is_some() {
                return Err(CommandError::NotPossible);
            }
            let cost = value(catalog, state, &m, &p).scale(m.reno_cost);
            if state.person.account.balance < cost {
                return Err(CommandError::NotEnoughPrivateMoney {
                    needed: cost - state.person.account.balance,
                });
            }
            crate::private::book_flow(state, PrivateFlow::PropertyCost, -cost);
            let p = &mut state.person.portfolio.properties[i];
            p.condition = (p.condition + m.reno_points).min(100.0);
            p.renovating_until = Some(state.date.add_months(m.reno_months));
        }
        I::AssignProperty { index, site } => {
            let kind = state
                .person
                .portfolio
                .properties
                .get(index as usize)
                .ok_or(CommandError::NotPossible)?
                .kind;
            if let Some(s) = site {
                let ok = state
                    .sites
                    .get(s as usize)
                    .is_some_and(|x| crate::private::is_controlled(state, x.owner));
                if !ok || kind == PropertyKind::Land {
                    return Err(CommandError::NotPossible);
                }
            }
            state.person.portfolio.properties[index as usize].tenant = site;
        }
        _ => return Err(CommandError::NotPossible),
    }
    Ok(())
}

/// Sets the housing relief of every site from the flats let to it, and returns the leases
/// of offices and halls as (company, site) pairs (docs/FORMELN.md, PV5c).
fn company_use(state: &mut GameState, m: &PropertyModel) -> Vec<(usize, usize)> {
    let mut relief = vec![0.0_f64; state.sites.len()];
    let mut leases = Vec::new();
    for (i, p) in state.person.portfolio.properties.iter().enumerate() {
        let Some(s) = p.tenant.map(|s| s as usize) else {
            continue;
        };
        if s >= relief.len() || !crate::private::is_controlled(state, state.sites[s].owner) {
            continue;
        }
        match p.kind {
            PropertyKind::Apartment => relief[s] += m.company_relief,
            PropertyKind::Multi => relief[s] += 2.0 * m.company_relief,
            PropertyKind::Office | PropertyKind::Hall => leases.push((i, s)),
            PropertyKind::Land => {}
        }
    }
    for (site, r) in state.sites.iter_mut().zip(relief) {
        site.housing_relief = r.min(m.company_relief_max);
    }
    leases
}

/// Sells a property already taken out of the portfolio: the mortgage is repaid, the
/// tax on the gain paid.
fn sell(state: &mut GameState, catalog: &Catalog, m: &PropertyModel, p: &Property, discount: f64) {
    let proceeds = value(catalog, state, m, p).scale(1.0 - discount);
    let debt = p.mortgage.map_or(Money::ZERO, |h| h.balance);
    let tax = (proceeds - p.paid)
        .max(Money::ZERO)
        .scale(crate::private::capital_tax_rate(catalog, state));
    // A sale below the debt leaves the rest to the person's account, which may fall short.
    crate::private::book_flow_floor(state, PrivateFlow::AssetSold, proceeds - debt);
    if tax > Money::ZERO {
        crate::private::book_flow_floor(state, PrivateFlow::GainTax, -tax);
    }
}

/// At a month start: offers; rents, upkeep, mortgages, decay, damage.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    let Some(m) = model(catalog).cloned() else {
        return news;
    };
    if state.person.name.is_empty() {
        return news;
    }
    let month = crate::management::month_number(date);
    let mut rng = SimRng::for_stream(
        state.settings.seed,
        Stream::Person {
            month: month + PROPERTY_STREAM,
        },
    );
    if month.is_multiple_of(m.offer_months) {
        refresh_offers(state, catalog, &m, &mut rng);
    }
    let leases = company_use(state, &m);
    let mut i = 0;
    while i < state.person.portfolio.properties.len() {
        let p = state.person.portfolio.properties[i].clone();
        let terms = m.kinds[p.kind as usize];
        let v = value(catalog, state, &m, &p);
        let empty = p.renovating_until.is_some_and(|d| date < d);
        if p.renovating_until.is_some() && !empty {
            state.person.portfolio.properties[i].renovating_until = None;
        }
        // Income first, then the expenses.
        let (fee_share, vacancy_cut, damage_cut) = crate::services::housing(catalog, state);
        // A lease to the own firm has no vacancy; the firm pays the rent (PV5c).
        let lessee = leases
            .iter()
            .find(|&&(k, _)| k == i)
            .map(|&(_, s)| state.sites[s].owner);
        let rent = if empty {
            Money::ZERO
        } else if lessee.is_some() {
            market_rent(catalog, state, &m, &p).scale(p.rent_factor / 12.0)
        } else {
            market_rent(catalog, state, &m, &p)
                .scale(p.rent_factor * (1.0 - vacancy(&m, &p) * (1.0 - vacancy_cut)) / 12.0)
        };
        if let (Some(company), true) = (lessee, rent > Money::ZERO) {
            crate::management::book_personnel(state, company, crate::state::Unit::Board, rent);
        }
        if rent > Money::ZERO {
            crate::private::book_flow(state, PrivateFlow::Rent, rent);
        }
        let mut cost = v.scale(terms.upkeep / 12.0) + rent.scale(fee_share);
        let mut damaged = false;
        if p.kind != PropertyKind::Land && rng.chance(m.damage_chance * (1.0 - damage_cut)) {
            cost += v.scale(m.damage_cost);
            damaged = true;
        }
        let due_mortgage = p.mortgage.map_or(Money::ZERO, |h| {
            h.balance.scale(h.rate / 12.0) + h.repay.min(h.balance)
        });
        if state.person.account.balance < cost + due_mortgage {
            // Cannot pay: emergency sale at a discount (docs/FORMELN.md, PV3).
            let p = state.person.portfolio.properties.remove(i);
            sell(state, catalog, &m, &p, m.distress_discount);
            news.push(
                Message::new(MessageKind::Warning, keys::PROPERTY_DISTRESS).with(
                    "art",
                    Param::TextKey(format!("person.immobilie_{}", p.kind.key())),
                ),
            );
            continue;
        }
        crate::private::book_flow(state, PrivateFlow::PropertyCost, -cost);
        let q = &mut state.person.portfolio.properties[i];
        if let Some(h) = q.mortgage.as_mut() {
            let interest = h.balance.scale(h.rate / 12.0);
            let repay = h.repay.min(h.balance);
            h.balance -= repay;
            let done = h.balance <= Money::ZERO;
            if done {
                q.mortgage = None;
            }
            crate::private::book_flow(state, PrivateFlow::MortgageInterest, -interest);
            crate::private::book_flow(state, PrivateFlow::MortgageRepay, -repay);
        }
        let q = &mut state.person.portfolio.properties[i];
        q.condition = (q.condition - terms.decay / 12.0 * 0.5).max(0.0);
        if damaged {
            q.condition = (q.condition - m.damage_points).max(0.0);
        }
        i += 1;
    }
    news
}

/// New offers for the home country and the countries of own sites.
fn refresh_offers(state: &mut GameState, catalog: &Catalog, m: &PropertyModel, rng: &mut SimRng) {
    let mut countries: Vec<CountryId> = vec![state.person.home];
    let mut own: Vec<CountryId> = state
        .sites
        .iter()
        .filter(|s| crate::private::is_controlled(state, s.owner))
        .map(|s| s.country)
        .collect();
    own.sort();
    own.dedup();
    countries.extend(own.into_iter().filter(|&c| c != state.person.home));
    countries.truncate(m.offer_countries as usize);
    state.assets.offers.clear();
    let discount = crate::standing::broker_discount(catalog, state);
    for country in countries {
        for kind in PropertyKind::ALL {
            let lerp =
                |r: &mut SimRng, span: [f64; 2]| span[0] + (span[1] - span[0]) * r.next_f64();
            let quality = lerp(rng, m.offer_quality);
            let condition = if kind == PropertyKind::Land {
                100.0
            } else {
                lerp(rng, m.offer_condition)
            };
            let spread = lerp(rng, m.offer_spread);
            let v = value_of(catalog, state, m, (kind, country, quality, condition));
            let id = state.assets.next_offer;
            state.assets.next_offer += 1;
            state.assets.offers.push(PropertyOffer {
                id,
                kind,
                country,
                quality,
                condition,
                price: v.scale((1.0 + spread) * (1.0 - discount)),
            });
        }
    }
}
