//! Reputation of companies per country and recalls (MN2, docs/FORMELN.md): quality above
//! the market's average raises it, accidents and recalls lower it; it weighs in the choice
//! of seller like the brand.

use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::ids::{CountryId, Id};
use crate::ledger::{Account, CostCenter, CostType};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{CompanyId, GameState, Reputation};

fn entry(state: &mut GameState, company: CompanyId, country: CountryId) -> &mut Reputation {
    let list = &mut state.companies[company.index()].reputations;
    let i = match list.iter().position(|r| r.country == country) {
        Some(i) => i,
        None => {
            list.push(Reputation {
                country,
                value: 0.5,
                sold: 0.0,
                quality_sum: 0.0,
                cost: Money::ZERO,
                low_quality: false,
            });
            list.len() - 1
        }
    };
    &mut list[i]
}

/// A sale of a company's site to end customers or the state in a country.
pub(crate) fn record_sale(
    state: &mut GameState,
    catalog: &Catalog,
    (company, country): (CompanyId, CountryId),
    quantity: f64,
    quality: f64,
    cost: Money,
) {
    let m = &catalog.market_model.reputation;
    if !m.enabled || quantity <= 0.0 {
        return;
    }
    let below = quality < m.recall_below;
    let r = entry(state, company, country);
    r.sold += quantity;
    r.quality_sum += quantity * quality;
    r.cost += cost;
    r.low_quality |= below;
}

/// An accident at a company's site lowers its reputation in the country.
pub(crate) fn accident(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    country: CountryId,
) {
    let m = &catalog.market_model.reputation;
    if !m.enabled {
        return;
    }
    let r = entry(state, company, country);
    r.value = (r.value - m.accident_loss).clamp(0.0, 1.0);
}

/// Month start: reputation from the month's quality against the market's average, recalls
/// of goods sold below the threshold, the way back to the middle without sales.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let m = &catalog.market_model.reputation;
    let mut news = Vec::new();
    if !m.enabled {
        return news;
    }
    // Average quality sold to end customers per country (companies only).
    let mut totals: Vec<(f64, f64)> = vec![(0.0, 0.0); catalog.countries.len()];
    for c in &state.companies {
        for r in &c.reputations {
            let t = &mut totals[r.country.index()];
            t.0 += r.sold;
            t.1 += r.quality_sum;
        }
    }
    let month = u32::try_from(date.year() * 12).unwrap_or(0) + date.month();
    for index in 0..state.companies.len() {
        let company = CompanyId(u32::try_from(index).unwrap_or(u32::MAX));
        if state.companies[index].bankrupt {
            continue;
        }
        let mut rng = SimRng::for_stream(
            state.settings.seed,
            Stream::Recalls {
                company: company.0,
                month,
            },
        );
        for i in 0..state.companies[index].reputations.len() {
            let r = &state.companies[index].reputations[i];
            let (country, sold, quality_sum, cost, low) =
                (r.country, r.sold, r.quality_sum, r.cost, r.low_quality);
            let mut value = r.value;
            if sold > 0.0 {
                let (all, all_quality) = totals[country.index()];
                let average = if all > 0.0 { all_quality / all } else { 50.0 };
                value += m.gain_per_month * (quality_sum / sold - average) / 50.0;
            } else {
                value += (0.5 - value) * m.return_per_month;
            }
            let recall = low && rng.next_f64() < m.recall_chance;
            if recall {
                value -= m.recall_loss;
                if cost > Money::ZERO {
                    state.companies[index].ledger.expense(
                        CostType::Other,
                        CostCenter::default(),
                        Account::Cash,
                        cost,
                    );
                }
                if state.is_main(company) {
                    news.push(
                        Message::new(MessageKind::Warning, keys::RECALL)
                            .with(
                                "land",
                                Param::Country(catalog.countries.key(country).to_owned()),
                            )
                            .with("kosten", Param::Money(cost)),
                    );
                }
            }
            let r = &mut state.companies[index].reputations[i];
            r.value = value.clamp(0.0, 1.0);
            r.sold = 0.0;
            r.quality_sum = 0.0;
            r.cost = Money::ZERO;
            r.low_quality = false;
        }
    }
    news
}
