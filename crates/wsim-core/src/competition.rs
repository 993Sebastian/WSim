//! What happened in the markets month by month (M24; docs/FORMELN.md): the price and
//! sales series of every market for the charts, and the news about the competitors in
//! the player's markets for the round report. Neither acts on the simulation.

use std::collections::BTreeMap;

use crate::catalog::Catalog;
use crate::ids::{CountryId, ProductId};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::state::{CompanyId, GameState, MarketHistory, WatchedMarket};

/// At the end of a month, after the markets and the offers closed it.
pub(crate) fn month_end(state: &mut GameState, catalog: &Catalog) -> Vec<Message> {
    record_history(state, catalog);
    competitor_news(state, catalog)
}

/// The player's markets: products and countries where one of its sites has an offer.
fn player_markets(state: &GameState) -> Vec<(ProductId, CountryId)> {
    let player = state.player;
    let mut markets: Vec<(ProductId, CountryId)> = state
        .sites
        .iter()
        .filter(|s| s.owner == player)
        .flat_map(|s| s.offers.keys().map(move |&p| (p, s.country)))
        .collect();
    markets.sort_unstable();
    markets.dedup();
    markets
}

/// Appends the closed month to the series of every market that sold something within
/// the kept months.
fn record_history(state: &mut GameState, catalog: &Catalog) {
    let keep = usize::try_from(catalog.market_model.history_months).unwrap_or(usize::MAX);
    let player = state.player;
    let mut own: BTreeMap<(ProductId, CountryId), f64> = BTreeMap::new();
    for s in state.sites.iter().filter(|s| s.owner == player) {
        for (&p, o) in &s.offers {
            *own.entry((p, s.country)).or_default() += o.sold_last_month;
        }
    }
    for (product, markets) in state.markets.iter_mut() {
        for (country, m) in markets.iter_mut() {
            let t = &m.last_month;
            let h = &mut m.history;
            if h.is_empty() && t.sold <= 1e-9 {
                continue;
            }
            h.price.push(if t.sold > 1e-9 {
                t.revenue.scale(1.0 / t.sold)
            } else {
                Money::ZERO
            });
            h.sold.push(narrow(t.sold));
            let mine = own.get(&(product, country)).copied().unwrap_or(0.0);
            if mine > 1e-9 && h.own.is_empty() {
                h.own = vec![0.0; h.price.len() - 1];
            }
            if !h.own.is_empty() {
                h.own.push(narrow(mine));
            }
            if h.price.len() > keep {
                let drop = h.price.len() - keep;
                h.price.drain(..drop);
                h.sold.drain(..drop);
                if !h.own.is_empty() {
                    h.own.drain(..drop);
                }
            }
            if h.sold.iter().all(|&q| q <= 0.0) {
                *h = MarketHistory::default();
            } else if h.own.iter().all(|&q| q <= 0.0) {
                h.own.clear();
            }
        }
    }
}

/// Quantities for the charts; single precision keeps the save small.
#[allow(clippy::cast_possible_truncation)]
fn narrow(q: f64) -> f32 {
    q as f32
}

/// Newcomers, leavers and price cuts of the competitors in the player's markets since
/// the last month's end.
fn competitor_news(state: &mut GameState, catalog: &Catalog) -> Vec<Message> {
    let player = state.player;
    let cut = catalog.market_model.price_cut_report;
    let mut news = Vec::new();
    let mut watched = Vec::new();
    for (product, country) in player_markets(state) {
        // Each competitor's lowest price here.
        let mut now: BTreeMap<CompanyId, Money> = BTreeMap::new();
        for s in state
            .sites
            .iter()
            .filter(|s| s.country == country && s.owner != player)
        {
            if state.companies[s.owner.index()].bankrupt {
                continue;
            }
            if let Some(o) = s.offers.get(&product) {
                let e = now.entry(s.owner).or_insert(o.price);
                *e = (*e).min(o.price);
            }
        }
        // With the competitor's own name for the product, if it gave one (M42).
        let message = |(key, named): (&str, &str), company: CompanyId| {
            let c = &state.companies[company.index()];
            let name = c.product_names.get(&product);
            let m = Message::new(MessageKind::Info, if name.is_some() { named } else { key })
                .with("firma", Param::Text(c.name.clone()))
                .with(
                    "produkt",
                    Param::TextKey(format!("produkt.{}", catalog.products.key(product))),
                )
                .with(
                    "land",
                    Param::Country(catalog.countries.key(country).to_owned()),
                );
            match name {
                Some(n) => m.with("name", Param::Text(n.clone())),
                None => m,
            }
        };
        let before = state
            .watched_markets
            .iter()
            .find(|w| w.product == product && w.country == country);
        let mut sellers = Vec::new();
        for (&company, &price) in &now {
            let reference = before.map(|w| w.sellers.iter().find(|(c, _)| *c == company));
            match reference {
                // A market seen for the first time: nothing to compare with.
                None => sellers.push((company, price)),
                Some(None) => {
                    news.push(
                        message((keys::AI_NEW_SELLER, keys::AI_NEW_SELLER_NAMED), company)
                            .with("preis", Param::Money(price)),
                    );
                    sellers.push((company, price));
                }
                Some(Some(&(_, high))) if price.to_usd() <= high.to_usd() * (1.0 - cut) => {
                    // To a whole percent (a cut of 0.11 is 10.999… in floating point).
                    #[allow(clippy::cast_possible_truncation)]
                    let percent = ((1.0 - price.to_usd() / high.to_usd()) * 100.0).round() as i64;
                    news.push(
                        message((keys::AI_PRICE_CUT, keys::AI_PRICE_CUT_NAMED), company)
                            .with("prozent", Param::Integer(percent))
                            .with("preis", Param::Money(price)),
                    );
                    sellers.push((company, price));
                }
                Some(Some(&(_, high))) => sellers.push((company, high.max(price))),
            }
        }
        if let Some(w) = before {
            for &(company, _) in &w.sellers {
                if !now.contains_key(&company) {
                    news.push(message(
                        (keys::AI_SELLER_GONE, keys::AI_SELLER_GONE_NAMED),
                        company,
                    ));
                }
            }
        }
        watched.push(WatchedMarket {
            product,
            country,
            sellers,
        });
    }
    state.watched_markets = watched;
    news
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::catalog::{SiteType, test_support};
    use crate::command::{self, Command};
    use crate::game::Game;
    use crate::state::{GameSettings, PriceMode, SaleOffer, SiteId, StartForm};

    fn usd(v: f64) -> Money {
        Money::from_usd(v).unwrap()
    }

    fn new_game(history_months: u32) -> Game {
        let mut catalog = test_support::production();
        catalog.market_model.history_months = history_months;
        let catalog = Arc::new(catalog);
        let settings = GameSettings {
            seed: 3,
            start_year: 1900,
            start_country: catalog.countries.id("AAA").unwrap(),
            start_capital: usd(20_000_000.0),
            start_form: StartForm::Workshop,
            company_name: "Hütte AG".into(),
            research_ahead_factor: 1.0,
            market_scale: 1.0,
            ai: Default::default(),
        };
        Game::new(catalog, settings).unwrap()
    }

    fn offer(price: f64, sold_last_month: f64) -> SaleOffer {
        SaleOffer {
            mode: PriceMode::Market {
                markup: 0.0,
                floor: Money::ZERO,
            },
            price: usd(price),
            keep: 0.0,
            sold_today: 0.0,
            sold_month: 0.0,
            sold_last_month,
            to_traders_month: 0.0,
            to_companies_month: 0.0,
        }
    }

    /// A works of `company` in AAA.
    fn works(state: &mut GameState, catalog: &Catalog, company: CompanyId) -> SiteId {
        let found = Command::FoundSite {
            country: catalog.countries.id("AAA").unwrap(),
            kind: SiteType::Factory,
        };
        command::execute(state, catalog, company, &found).unwrap();
        SiteId(u32::try_from(state.sites.len() - 1).unwrap())
    }

    /// A second company like the player's.
    fn rival(state: &mut GameState, name: &str) -> CompanyId {
        let mut company = state.companies[0].clone();
        name.clone_into(&mut company.name);
        state.companies.push(company);
        CompanyId(u32::try_from(state.companies.len() - 1).unwrap())
    }

    #[test]
    fn the_market_series_keeps_the_last_months() {
        let mut game = new_game(3);
        let catalog = game.catalog().clone();
        let state = game.state_mut();
        let iron = catalog.products.id("eisen").unwrap();
        let aaa = catalog.countries.id("AAA").unwrap();
        let own = works(state, &catalog, state.player);
        let month = |state: &mut GameState, sold: f64, revenue: f64, mine: f64| {
            let m = state.markets.get_mut(iron).get_mut(aaa);
            m.last_month.sold = sold;
            m.last_month.revenue = usd(revenue);
            if mine > 0.0 {
                state.sites[own.index()]
                    .offers
                    .insert(iron, offer(100.0, mine));
            }
            record_history(state, &catalog);
            state.markets.get(iron).get(aaa).history.clone()
        };
        // Nothing sold yet: no series.
        assert!(month(state, 0.0, 0.0, 0.0).is_empty());
        let h = month(state, 10.0, 1000.0, 0.0);
        assert_eq!(h.price, vec![usd(100.0)]);
        assert!(h.own.is_empty());
        // A month without sales keeps its place; the player's first sale fills the past.
        month(state, 0.0, 0.0, 0.0);
        let h = month(state, 20.0, 2400.0, 5.0);
        assert_eq!(h.price, vec![usd(100.0), Money::ZERO, usd(120.0)]);
        assert_eq!(h.sold, vec![10.0, 0.0, 20.0]);
        assert_eq!(h.own, vec![0.0, 0.0, 5.0]);
        // Three months kept.
        let h = month(state, 30.0, 3000.0, 5.0);
        assert_eq!(h.sold, vec![0.0, 20.0, 30.0]);
        assert_eq!(h.own, vec![0.0, 5.0, 5.0]);
        // Gone after three months without sales.
        state.sites[own.index()].offers.clear();
        for _ in 0..3 {
            month(state, 0.0, 0.0, 0.0);
        }
        assert!(state.markets.get(iron).get(aaa).history.is_empty());
    }

    #[test]
    fn competitors_are_reported_when_they_come_go_or_cut_their_price() {
        let mut game = new_game(24);
        let catalog = game.catalog().clone();
        let state = game.state_mut();
        let iron = catalog.products.id("eisen").unwrap();
        let own = works(state, &catalog, state.player);
        state.sites[own.index()]
            .offers
            .insert(iron, offer(100.0, 0.0));
        let a = rival(state, "Alpha AG");
        let alpha = works(state, &catalog, a);
        state.sites[alpha.index()]
            .offers
            .insert(iron, offer(100.0, 0.0));
        let keys_of =
            |news: Vec<Message>| -> Vec<String> { news.into_iter().map(|m| m.key).collect() };

        // The first month's end only looks.
        assert!(competitor_news(state, &catalog).is_empty());
        // Small cuts add up until they reach a tenth.
        state.sites[alpha.index()]
            .offers
            .get_mut(&iron)
            .unwrap()
            .price = usd(95.0);
        assert!(competitor_news(state, &catalog).is_empty());
        state.sites[alpha.index()]
            .offers
            .get_mut(&iron)
            .unwrap()
            .price = usd(89.0);
        let news = competitor_news(state, &catalog);
        assert_eq!(keys_of(news.clone()), vec![keys::AI_PRICE_CUT]);
        assert!(
            news[0]
                .params
                .contains(&("prozent".into(), Param::Integer(11)))
        );
        assert!(
            news[0]
                .params
                .contains(&("firma".into(), Param::Text("Alpha AG".into())))
        );
        // Measured from there on, after a rise from the new high.
        assert!(competitor_news(state, &catalog).is_empty());
        state.sites[alpha.index()]
            .offers
            .get_mut(&iron)
            .unwrap()
            .price = usd(120.0);
        assert!(competitor_news(state, &catalog).is_empty());
        state.sites[alpha.index()]
            .offers
            .get_mut(&iron)
            .unwrap()
            .price = usd(105.0);
        assert_eq!(
            keys_of(competitor_news(state, &catalog)),
            vec![keys::AI_PRICE_CUT]
        );

        // A newcomer, then Alpha leaves.
        let b = rival(state, "Beta AG");
        let beta = works(state, &catalog, b);
        state.sites[beta.index()]
            .offers
            .insert(iron, offer(90.0, 0.0));
        assert_eq!(
            keys_of(competitor_news(state, &catalog)),
            vec![keys::AI_NEW_SELLER]
        );
        state.sites[alpha.index()].offers.clear();
        assert_eq!(
            keys_of(competitor_news(state, &catalog)),
            vec![keys::AI_SELLER_GONE]
        );
        // Markets the player left are not watched any more.
        state.sites[own.index()].offers.clear();
        assert!(competitor_news(state, &catalog).is_empty());
        assert!(state.watched_markets.is_empty());
    }
}
