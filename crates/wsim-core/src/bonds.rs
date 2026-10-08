//! Corporate bonds (K2, Lastenheft §11.1; formulas in docs/FORMELN.md, K2).
//!
//! A company with enough equity sells bonds to investors outside the game: unsecured debt
//! with a fixed coupon by its credit grade, paid monthly, and repaid at once at maturity.

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::command::CommandError;
use crate::ledger::{Account, CostCenter, CostType};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::state::{Company, CompanyId, GameState};

/// A bond issued and not yet repaid.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bond {
    pub principal: Money,
    /// Real interest per year, fixed at issue.
    pub coupon: f64,
    pub issued: Date,
    pub maturity: Date,
    /// Key of the grade at issue (`bonitaet.<key>`).
    pub grade: String,
}

/// The figures behind a company's grade.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Standing {
    pub assets: Money,
    /// Loans and bonds.
    pub debt: Money,
    /// Earnings before interest and taxes per year; `None` without a closed month.
    pub ebit: Option<Money>,
    /// Interest paid per year.
    pub interest: Money,
}

impl Standing {
    pub fn debt_ratio(&self, amount: Money) -> Option<f64> {
        let assets = self.assets + amount;
        (assets > Money::ZERO).then(|| (self.debt + amount).to_usd() / assets.to_usd())
    }

    /// EBIT over interest with a new bond of `amount` at `coupon`; without interest
    /// unbounded for a profit, zero for a loss.
    pub fn coverage(&self, amount: Money, coupon: f64) -> Option<f64> {
        let ebit = self.ebit?;
        let interest = self.interest.to_usd() + amount.to_usd() * coupon;
        Some(if interest > 0.0 {
            ebit.to_usd() / interest
        } else if ebit > Money::ZERO {
            f64::INFINITY
        } else {
            0.0
        })
    }
}

/// Assets, debt, EBIT and interest of the last closed months (`gewinn_monate`).
pub fn standing(catalog: &Catalog, company: &Company) -> Standing {
    let l = &company.ledger;
    let window = catalog.bonds.earnings_months.max(1) as usize;
    let months: Vec<_> = l.months.iter().rev().take(window).collect();
    let (ebit, interest) = if months.is_empty() {
        (None, Money::ZERO)
    } else {
        let (mut ebit, mut interest) = (Money::ZERO, Money::ZERO);
        for m in &months {
            let paid = m
                .by_type
                .get(&CostType::Interest)
                .copied()
                .unwrap_or_default();
            let taxes = m.by_type.get(&CostType::Taxes).copied().unwrap_or_default();
            ebit += m.total() - paid - taxes;
            interest -= paid;
        }
        // At most 24 months; the cast is exact.
        #[allow(clippy::cast_precision_loss)]
        let year = 12.0 / months.len() as f64;
        (
            Some(ebit.scale(year)),
            interest.scale(year).max(Money::ZERO),
        )
    };
    Standing {
        assets: l.total_assets(),
        debt: l.balance(Account::Loans) + l.balance(Account::Bonds),
        ebit,
        interest,
    }
}

/// The coupon of a grade today; the finance department saves `cut` of the spread (ZA2).
/// Never below zero.
pub fn coupon(catalog: &Catalog, grade: usize, date: Date, cut: f64) -> f64 {
    let spread = catalog.bonds.grades.get(grade).map_or(0.0, |g| g.spread);
    (crate::finance::base_rate(catalog, date) + spread * (1.0 - cut)).max(0.0)
}

/// The grade (index into `bonitaet`) and coupon a new bond of `amount` would get; `None`
/// when no investor buys it.
pub fn grade_for(
    catalog: &Catalog,
    company: &Company,
    amount: Money,
    date: Date,
    cut: f64,
) -> Option<(usize, f64)> {
    let s = standing(catalog, company);
    let ratio = s.debt_ratio(amount)?;
    catalog.bonds.grades.iter().enumerate().find_map(|(i, g)| {
        let c = coupon(catalog, i, date, cut);
        let coverage = s.coverage(amount, c)?;
        (ratio <= g.debt_ratio_max && coverage >= g.coverage_min).then_some((i, c))
    })
}

/// The largest bond that still finds investors (zero if none does).
pub fn max_amount(catalog: &Catalog, company: &Company, date: Date, cut: f64) -> Money {
    let m = &catalog.bonds;
    if grade_for(catalog, company, m.volume_min, date, cut).is_none() {
        return Money::ZERO;
    }
    // The grade only worsens with the amount: bisect between a good and a bad amount.
    let (mut good, mut bad) = (m.volume_min, company.ledger.total_assets().scale(4.0));
    if grade_for(catalog, company, bad, date, cut).is_some() {
        return bad;
    }
    for _ in 0..48 {
        let mid = (good + bad).scale(0.5);
        if mid <= good || mid >= bad {
            break;
        }
        if grade_for(catalog, company, mid, date, cut).is_some() {
            good = mid;
        } else {
            bad = mid;
        }
    }
    good
}

fn equity(company: &Company) -> Money {
    crate::ranking::equity(company)
}

/// Issues a bond of `amount` for `years` (docs/FORMELN.md, K2).
pub(crate) fn issue(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    amount: Money,
    years: u32,
) -> Result<(), CommandError> {
    let m = &catalog.bonds;
    if !m.enabled {
        return Err(CommandError::NoBonds);
    }
    if years < m.term_min_years || years > m.term_max_years {
        return Err(CommandError::BondTerm {
            min: m.term_min_years,
            max: m.term_max_years,
        });
    }
    if amount < m.volume_min {
        return Err(CommandError::BondTooSmall { min: m.volume_min });
    }
    let company = &state.companies[actor.index()];
    if equity(company) < m.equity_min {
        return Err(CommandError::BondCompanyTooSmall { min: m.equity_min });
    }
    let cut = crate::central::premium_cut(catalog, state, actor);
    let date = state.date;
    let Some((grade, coupon)) = grade_for(catalog, company, amount, date, cut) else {
        return Err(CommandError::NoBondInvestors {
            max: max_amount(catalog, company, date, cut),
        });
    };
    let c = &mut state.companies[actor.index()];
    c.ledger.transfer(Account::Cash, Account::Bonds, amount);
    let cost = amount.scale(m.cost_share);
    if cost > Money::ZERO {
        c.ledger
            .expense(CostType::Other, CostCenter::default(), Account::Cash, cost);
    }
    c.bonds.push(Bond {
        principal: amount,
        coupon,
        issued: date,
        maturity: date.add_months(years * 12),
        grade: m.grades[grade].key.clone(),
    });
    Ok(())
}

/// What buying back a bond early costs.
pub fn redeem_price(catalog: &Catalog, bond: &Bond) -> Money {
    bond.principal.scale(1.0 + catalog.bonds.redeem_premium)
}

/// Buys a bond back before maturity at its price above par; the premium is interest.
pub(crate) fn redeem(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    index: usize,
) -> Result<(), CommandError> {
    let c = &mut state.companies[actor.index()];
    let Some(bond) = c.bonds.get(index) else {
        return Err(CommandError::UnknownBond);
    };
    let price = redeem_price(catalog, bond);
    if c.ledger.cash() < price {
        return Err(CommandError::NotEnoughCash { needed: price });
    }
    let principal = bond.principal;
    c.ledger.transfer(Account::Bonds, Account::Cash, principal);
    let premium = price - principal;
    if premium > Money::ZERO {
        c.ledger.expense(
            CostType::Interest,
            CostCenter::default(),
            Account::Cash,
            premium,
        );
    }
    c.bonds.remove(index);
    Ok(())
}

/// End of a month (`last_day`): coupons, and the bonds due repaid. Returns the messages
/// for the player.
pub(crate) fn month_end(state: &mut GameState, last_day: Date) -> Vec<Message> {
    let mut news = Vec::new();
    let next = last_day.next_day();
    let player = state.player;
    for (i, c) in state.companies.iter_mut().enumerate() {
        if c.bankrupt || c.bonds.is_empty() {
            continue;
        }
        let center = CostCenter::default();
        for b in &c.bonds {
            let interest = b.principal.scale(b.coupon / 12.0);
            if interest > Money::ZERO {
                c.ledger
                    .expense(CostType::Interest, center, Account::Cash, interest);
            }
        }
        let (due, open): (Vec<Bond>, Vec<Bond>) =
            c.bonds.drain(..).partition(|b| b.maturity <= next);
        c.bonds = open;
        for b in due {
            c.ledger
                .transfer(Account::Bonds, Account::Cash, b.principal);
            if i == player.index() {
                news.push(
                    Message::new(MessageKind::Info, keys::BOND_REPAID)
                        .with("betrag", Param::Money(b.principal)),
                );
            }
        }
    }
    news
}

/// Whether an AI company should issue a bond instead of a loan of `amount`, and for how
/// many years.
pub(crate) fn ai_prefers(
    state: &GameState,
    catalog: &Catalog,
    id: CompanyId,
    amount: Money,
) -> Option<u32> {
    let m = &catalog.bonds;
    if !m.enabled || amount < m.volume_min {
        return None;
    }
    let c = &state.companies[id.index()];
    if equity(c) < m.equity_min {
        return None;
    }
    let cut = crate::central::premium_cut(catalog, state, id);
    let (_, coupon) = grade_for(catalog, c, amount, state.date, cut)?;
    let loan = crate::finance::loan_rate(catalog, c, amount, state.date, cut);
    (coupon + m.ai_advantage_min <= loan).then_some(m.ai_term_years)
}
