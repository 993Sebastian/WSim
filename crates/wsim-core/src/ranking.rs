//! The player's standing among all companies (M29; docs/FORMELN.md): place by equity
//! and by revenue of the last twelve closed months. The places at the start of the game
//! and at each month end are kept for two years, to compare with a year before. Nothing
//! here feeds back into the simulation.

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::ledger::{Account, CostType};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::state::{Company, CompanyId, GameState};

/// Month ends kept: a comparison with a year before for a whole year.
const MONTHS_KEPT: usize = 24;

/// Places of the player at a point in time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Standing {
    /// Start of the game or first day after a month end.
    pub date: Date,
    pub equity: u32,
    /// `None` without revenue in the last twelve months: at the start every company
    /// would share the first place.
    pub revenue: Option<u32>,
    /// Active companies, the player included.
    pub companies: u32,
}

pub fn equity(company: &Company) -> Money {
    company.ledger.total_assets()
        - company.ledger.balance(Account::Loans)
        - company.ledger.balance(Account::Bonds)
}

/// Revenue of the last twelve closed months.
pub fn revenue_of_year(company: &Company) -> Money {
    company
        .ledger
        .months
        .iter()
        .rev()
        .take(12)
        .filter_map(|m| m.by_type.get(&CostType::Revenue))
        .copied()
        .sum()
}

fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// Places of a company among the active companies now: one more than the number of
/// companies ahead; equal values share a place.
pub fn standing(state: &GameState, company: CompanyId) -> Standing {
    let active: Vec<&Company> = state.companies.iter().filter(|c| !c.bankrupt).collect();
    let own = &state.companies[company.index()];
    let (e, r) = (equity(own), revenue_of_year(own));
    Standing {
        date: state.date,
        equity: count(active.iter().filter(|c| equity(c) > e).count()) + 1,
        revenue: (r > Money::ZERO)
            .then(|| count(active.iter().filter(|c| revenue_of_year(c) > r).count()) + 1),
        companies: count(active.len()),
    }
}

/// The player's places a year before the month of `date`, if recorded.
pub fn year_before(state: &GameState, date: Date) -> Option<Standing> {
    let target = Date::new(date.year() - 1, date.month(), 1)?;
    state.standings.iter().find(|s| s.date == target).copied()
}

/// Records the player's places (at the start of the game and on the first day after
/// each month end).
pub fn record(state: &mut GameState) {
    let now = standing(state, state.player);
    state.standings.push(now);
    if state.standings.len() > MONTHS_KEPT {
        state.standings.remove(0);
    }
}

/// Records the places after a month end; after the end of a year also reports them,
/// by equity and (with revenue) by revenue, each with the place a year before.
pub fn month_end(state: &mut GameState) -> Vec<Message> {
    record(state);
    let Some(&now) = state.standings.last() else {
        return Vec::new();
    };
    if now.date.ordinal() != 1 || now.companies < 2 {
        return Vec::new();
    }
    let before = year_before(state, now.date);
    let place = |p: u32| Param::Integer(i64::from(p));
    let report = |criterion: &str, now_place: u32, before_place: Option<u32>| {
        let key = if before_place.is_some() {
            keys::RANK_YEAR_END_COMPARED
        } else {
            keys::RANK_YEAR_END
        };
        let message = Message::new(MessageKind::Info, key)
            .with("jahr", Param::Integer(i64::from(now.date.year() - 1)))
            .with("kriterium", Param::TextKey(criterion.to_owned()))
            .with("platz", place(now_place))
            .with("firmen", place(now.companies));
        match before_place {
            Some(b) => message.with("vorher", place(b)),
            None => message,
        }
    };
    let mut messages = vec![report(
        keys::RANK_BY_EQUITY,
        now.equity,
        before.map(|b| b.equity),
    )];
    if let Some(revenue) = now.revenue {
        messages.push(report(
            keys::RANK_BY_REVENUE,
            revenue,
            before.and_then(|b| b.revenue),
        ));
    }
    messages
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::calendar::RoundLength;
    use crate::catalog::test_support;
    use crate::game::Game;
    use crate::ledger::CostCenter;
    use crate::state::{GameSettings, StartForm};

    fn usd(v: f64) -> Money {
        Money::from_usd(v).unwrap()
    }

    /// The player and three rivals with the player's books: one richer, one equal and
    /// one poorer.
    fn game() -> Game {
        let catalog = Arc::new(test_support::production());
        let settings = GameSettings {
            seed: 5,
            start_year: 1900,
            start_country: catalog.countries.id("AAA").unwrap(),
            start_capital: usd(100_000.0),
            start_form: StartForm::Workshop,
            company_name: "Rang AG".into(),
            research_ahead_factor: 1.0,
            market_scale: 1.0,
            ai: Default::default(),
            ventures: 1.0,
            tariff_dynamics: 1.0,
        };
        let mut game = Game::new(catalog, settings).unwrap();
        let state = game.state_mut();
        for (name, change) in [("Reich", 50_000.0), ("Gleich", 0.0), ("Arm", -50_000.0)] {
            let mut rival = state.companies[0].clone();
            name.clone_into(&mut rival.name);
            if change > 0.0 {
                rival
                    .ledger
                    .transfer(Account::Cash, Account::Equity, usd(change));
            } else if change < 0.0 {
                rival.ledger.expense(
                    CostType::Personnel,
                    CostCenter::default(),
                    Account::Cash,
                    usd(-change),
                );
            }
            state.companies.push(rival);
        }
        game
    }

    #[test]
    fn places_count_the_companies_ahead() {
        let mut game = game();
        let place = |game: &Game, c: u32| {
            let s = standing(game.state(), CompanyId(c));
            (s.equity, s.revenue, s.companies)
        };
        // Equal values share a place; without revenue there is no place by revenue.
        assert_eq!(place(&game, 0), (2, None, 4));
        assert_eq!(place(&game, 2), (2, None, 4));
        assert_eq!(place(&game, 3), (4, None, 4));

        // Revenue of the closed months counts.
        let ledger = &mut game.state_mut().companies[0].ledger;
        ledger.income(
            CostType::Revenue,
            CostCenter::default(),
            Account::Cash,
            usd(1_000.0),
        );
        ledger.close_month(Date::new(1900, 2, 1).unwrap());
        assert_eq!(place(&game, 0), (2, Some(1), 4));
        assert_eq!(place(&game, 1), (1, None, 4));

        // A bankrupt company no longer counts.
        game.state_mut().companies[1].bankrupt = true;
        assert_eq!(place(&game, 0), (1, Some(1), 3));
    }

    #[test]
    fn the_year_end_report_compares_with_a_year_before() {
        let mut game = game();
        // The start is recorded as the first standing.
        assert_eq!(game.state().standings.len(), 1);
        assert_eq!(game.state().standings[0].date, game.date());
        let mut reports = Vec::new();
        while game.date() < Date::new(1902, 1, 1).unwrap() {
            let report = game.advance(RoundLength::Month, |_| {});
            reports.extend(
                report
                    .messages
                    .into_iter()
                    .filter(|m| m.key.starts_with("meldung.rang")),
            );
        }
        // One report by equity per year end, compared with the start and with a year
        // before; the workshop sells nothing here, so there is none by revenue.
        assert_eq!(reports.len(), 2, "{reports:?}");
        assert!(
            reports
                .iter()
                .all(|m| m.key == keys::RANK_YEAR_END_COMPARED)
        );
        let param = |m: &Message, name: &str| {
            m.params
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, p)| p.clone())
        };
        assert_eq!(param(&reports[0], "jahr"), Some(Param::Integer(1900)));
        assert_eq!(param(&reports[1], "jahr"), Some(Param::Integer(1901)));
        assert_eq!(
            param(&reports[1], "kriterium"),
            Some(Param::TextKey(keys::RANK_BY_EQUITY.into()))
        );
        assert_eq!(param(&reports[1], "firmen"), Some(Param::Integer(4)));
        assert_eq!(param(&reports[1], "vorher"), Some(Param::Integer(2)));
        // Two years of month ends are kept.
        let standings = &game.state().standings;
        assert_eq!(standings.len(), MONTHS_KEPT);
        let last = *standings.last().unwrap();
        assert_eq!(last.date, Date::new(1902, 1, 1).unwrap());
        assert_eq!(
            year_before(game.state(), last.date).map(|s| s.date),
            Date::new(1901, 1, 1)
        );
    }
}
