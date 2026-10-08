//! The person's private account, founding and lifestyle (PE3; docs/PERSON.md §6–8,
//! formulas in docs/FORMELN.md, section PE3). Money moves between the person and the
//! companies only here: founding, capital, loans, salary and capital paid back.

use std::collections::BTreeMap;

use crate::calendar::Date;
use crate::catalog::{Catalog, LifestyleLevel};
use crate::command::{Command, CommandError};
use crate::ids::CountryId;
use crate::ledger::Account;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::state::{
    CompanyId, GameState, Holder, LifeEvent, LifeEventKind, Lifestyle, LifestyleSpell, Loan,
    Person, PrivateFlow, PrivateMonth, Role, Stake, StartForm, Unit, WealthPoint,
};

/// Closed months of movements kept.
const MONTHS_KEPT: usize = 12;
/// Depth of a chain of companies that still counts as control.
const CONTROL_DEPTH: usize = 8;

/// A twelfth of an academic's yearly wage in a country (MA1's salary group).
pub fn academic_monthly_wage(catalog: &Catalog, state: &GameState, country: CountryId) -> Money {
    let yearly = crate::management::yearly_wage(catalog, state, country);
    Money::from_usd(yearly / 12.0).unwrap_or(Money::ZERO)
}

/// What founding a company with `capital` costs in a country (register, notary).
pub fn founding_cost(
    catalog: &Catalog,
    state: &GameState,
    country: CountryId,
    capital: Money,
) -> Money {
    let f = &catalog.person.founding;
    let wage = academic_monthly_wage(catalog, state, country);
    capital
        .scale(f.cost_share)
        .max(wage.scale(f.cost_min_months))
}

/// The capital a start form needs at least: its buildings and facilities; investment
/// firm and bank need none.
pub fn form_cost(catalog: &Catalog, form: StartForm) -> Money {
    catalog
        .production_model
        .start_setup(form)
        .map_or(Money::ZERO, |s| s.cost(catalog))
}

/// The least start money: the cheapest start form with a site plus its founding costs.
pub fn start_minimum(catalog: &Catalog, state: &GameState, country: CountryId) -> Money {
    let wage = academic_monthly_wage(catalog, state, country);
    minimum_with(catalog, wage)
}

/// The least start money before a game exists (the new-game dialog): the wages of the
/// country in the start year.
pub fn start_minimum_at(catalog: &Catalog, country: CountryId, year: i32) -> Money {
    let date = Date::first_of_year(year);
    let values = crate::country_model::compute(catalog, country, date);
    let yearly = catalog.management.salary_group.map_or(0.0, |g| {
        values
            .hourly_wage_usd
            .get(crate::ids::Id::index(g))
            .copied()
            .unwrap_or(0.0)
            * catalog.country_model.annual_hours.value_at(f64::from(year))
    });
    minimum_with(
        catalog,
        Money::from_usd(yearly / 12.0).unwrap_or(Money::ZERO),
    )
}

fn minimum_with(catalog: &Catalog, wage: Money) -> Money {
    let f = &catalog.person.founding;
    [StartForm::Workshop, StartForm::Trading]
        .into_iter()
        .filter(|&form| catalog.production_model.start_setup(form).is_some())
        .map(|form| {
            let cost = form_cost(catalog, form);
            cost + cost.scale(f.cost_share).max(wage.scale(f.cost_min_months))
        })
        .min()
        .unwrap_or(Money::ZERO)
}

/// The level of lifestyle on a day: the default before the first.
pub fn lifestyle_at(catalog: &Catalog, person: &Person, date: Date) -> Lifestyle {
    person
        .lifestyles
        .iter()
        .rev()
        .find(|s| s.from <= date)
        .map_or(catalog.person.default_lifestyle, |s| s.level)
}

fn level<'a>(catalog: &'a Catalog, state: &GameState) -> &'a LifestyleLevel {
    let at = lifestyle_at(catalog, &state.person, state.date);
    catalog.person.lifestyle(at)
}

/// What the lifestyle costs in a month.
pub fn lifestyle_cost(catalog: &Catalog, state: &GameState, level: Lifestyle) -> Money {
    academic_monthly_wage(catalog, state, state.person.home)
        .scale(catalog.person.lifestyle(level).cost)
}

/// Whether the person controls a company: over half directly or through a chain of
/// companies it controls; failed companies not.
pub fn is_controlled(state: &GameState, company: CompanyId) -> bool {
    controlled_within(state, company, CONTROL_DEPTH)
}

fn controlled_within(state: &GameState, company: CompanyId, depth: usize) -> bool {
    let Some(c) = state.company(company).filter(|c| !c.bankrupt) else {
        return false;
    };
    let mut by_holder: Vec<(Holder, f64)> = Vec::new();
    for s in &c.owners {
        match by_holder.iter_mut().find(|(h, _)| *h == s.holder) {
            Some((_, v)) => *v += s.share,
            None => by_holder.push((s.holder, s.share)),
        }
    }
    by_holder.into_iter().any(|(h, v)| {
        v > 0.5
            && match h {
                Holder::Player => true,
                Holder::Company(parent) => {
                    depth > 0 && parent != company && controlled_within(state, parent, depth - 1)
                }
                _ => false,
            }
    })
}

/// The companies the person controls, in order of their ids.
pub fn controlled(state: &GameState) -> Vec<CompanyId> {
    (0..state.companies.len())
        // Few companies; the cast is exact.
        .map(|i| CompanyId(i as u32))
        .filter(|&c| is_controlled(state, c))
        .collect()
}

/// A company's value: its market value when listed (K1), else what investors judge it
/// worth (K3); without a stock market in the data its book value.
pub fn company_value(catalog: &Catalog, state: &GameState, company: CompanyId) -> Money {
    let Some(c) = state.company(company).filter(|c| !c.bankrupt) else {
        return Money::ZERO;
    };
    match &c.listing {
        Some(l) => l.value,
        None if catalog.stock.enabled => crate::stock::fair(catalog, c),
        None => crate::ranking::equity(c).max(Money::ZERO),
    }
}

/// The person's wealth today: balance, shares at value and open loans.
pub fn wealth(catalog: &Catalog, state: &GameState) -> WealthPoint {
    let mut shares = Money::ZERO;
    let mut loans = Money::ZERO;
    for (i, c) in state.companies.iter().enumerate() {
        if c.bankrupt {
            continue;
        }
        // Few companies; the cast is exact.
        let id = CompanyId(i as u32);
        let held = crate::person::share(state, id);
        if held > 0.0 {
            shares += company_value(catalog, state, id).scale(held);
        }
        loans += c
            .loans
            .iter()
            .filter(|l| l.from_person)
            .map(|l| l.balance)
            .sum();
    }
    WealthPoint {
        date: state.date,
        cash: state.person.account.balance,
        shares,
        loans,
    }
}

/// The salary of a solid CEO of a company (Stärke 50, MA1): the suggestion.
pub fn salary_suggestion(catalog: &Catalog, state: &GameState, company: CompanyId) -> Money {
    let Some(level) = catalog
        .management
        .levels
        .get(crate::management::level_of(Unit::Board))
    else {
        return Money::ZERO;
    };
    let Some(c) = state.company(company) else {
        return Money::ZERO;
    };
    let wage = crate::management::yearly_wage(catalog, state, c.headquarters);
    Money::from_usd(level.salary_head * wage).unwrap_or(Money::ZERO)
}

/// The salary at founding: what pays the default lifestyle after income tax, at most
/// the suggestion.
pub fn start_salary(catalog: &Catalog, state: &GameState, company: CompanyId) -> Money {
    let living = lifestyle_cost(catalog, state, catalog.person.default_lifestyle).scale(12.0);
    let tax = income_tax(catalog, state).min(0.9);
    living
        .scale(1.0 / (1.0 - tax))
        .min(salary_suggestion(catalog, state, company))
}

/// The highest salary the person may set: with co-owners a multiple of the suggestion.
pub fn salary_max(catalog: &Catalog, state: &GameState, company: CompanyId) -> Option<Money> {
    (crate::person::share(state, company) < 1.0 - 1e-9).then(|| {
        salary_suggestion(catalog, state, company).scale(catalog.person.salary_co_owner_max)
    })
}

fn income_tax(catalog: &Catalog, state: &GameState) -> f64 {
    catalog
        .person
        .income_tax
        .value(state.person.home, state.date.year_fraction())
        .clamp(0.0, 1.0)
}

/// Added to the rate of a new bank loan of a company the person controls.
pub fn loan_rate_offset(catalog: &Catalog, state: &GameState, company: CompanyId) -> f64 {
    if is_controlled(state, company) {
        level(catalog, state).interest
    } else {
        0.0
    }
}

/// Factor on what candidates ask for a position of a controlled company's board.
pub fn salary_demand_factor(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    unit: Unit,
) -> f64 {
    if unit == Unit::Board && is_controlled(state, company) {
        (1.0 + level(catalog, state).salary_demand).max(0.0)
    } else {
        1.0
    }
}

/// Points of education on every skill of a child born on `born`, at its card on `card`:
/// the lifestyles of its childhood weighted by days; before the first, the first.
pub fn education(catalog: &Catalog, person: &Person, born: Date, card: Date) -> f64 {
    let days = born.days_until(card);
    if days <= 0 {
        return 0.0;
    }
    let first = person
        .lifestyles
        .first()
        .map_or(catalog.person.default_lifestyle, |s| s.level);
    let mut sum = 0.0;
    let mut from = born;
    let mut current = first;
    for s in person.lifestyles.iter().filter(|s| s.from > born) {
        let until = s.from.min(card);
        if until > from {
            sum += catalog.person.lifestyle(current).education * f64::from(from.days_until(until));
            from = until;
        }
        current = s.level;
    }
    if card > from {
        sum += catalog.person.lifestyle(current).education * f64::from(from.days_until(card));
    }
    sum / f64::from(days)
}

/// Books a movement on the private account; the balance never goes below zero.
fn book(person: &mut Person, flow: PrivateFlow, amount: Money) {
    let a = &mut person.account;
    a.balance += amount;
    debug_assert!(!a.balance.is_negative(), "private balance below zero");
    *a.month.entry(flow).or_default() += amount;
}

/// The person at the start of a game (docs/FORMELN.md, PE3): the start money on the
/// account, or the company founded with it; the default lifestyle.
pub(crate) fn start(state: &mut GameState, catalog: &Catalog, today: Date) {
    state.person.lifestyles = vec![LifestyleSpell {
        from: today,
        level: catalog.person.default_lifestyle,
        forced: false,
    }];
    match state.main_company {
        // Founded at the start (games without an interface): the company runs as
        // before, without a salary for the person.
        Some(company) => {
            let capital = state.settings.start_capital;
            state.person.cost_basis.insert(company, capital);
        }
        None => {
            let money = state.settings.start_capital;
            book(&mut state.person, PrivateFlow::StartMoney, money);
            state.person.history.push(LifeEvent {
                date: today,
                kind: LifeEventKind::Began { money },
            });
        }
    }
    record_wealth(state, catalog);
}

/// Fits saves from before PE3: the default lifestyle and the main company's capital as
/// what the person paid in; the company pays no salary until the player sets one.
pub(crate) fn fit_loaded(state: &mut GameState, catalog: &Catalog) {
    if !state.person.lifestyles.is_empty() {
        return;
    }
    let today = state.date;
    state.person.lifestyles = vec![LifestyleSpell {
        from: today,
        level: catalog.person.default_lifestyle,
        forced: false,
    }];
    if let Some(company) = state.main_company {
        let capital = state.companies[company.index()]
            .ledger
            .balance(Account::Equity)
            .max(Money::ZERO);
        state.person.cost_basis.insert(company, capital);
    }
    record_wealth(state, catalog);
}

/// Executes a command of the person.
pub(crate) fn execute(
    state: &mut GameState,
    catalog: &Catalog,
    command: &Command,
) -> Result<(), CommandError> {
    if state.game_over {
        return Err(CommandError::GameOver);
    }
    match command {
        Command::FoundCompany {
            name,
            form,
            country,
            capital,
        } => found(state, catalog, name, *form, *country, *capital).map(|_| ()),
        Command::SetPersonSalary { amount } => set_salary(state, catalog, *amount),
        Command::SetLifestyle { level } => set_lifestyle(state, catalog, *level),
        Command::ContributeCapital { company, amount } => {
            contribute(state, catalog, *company, *amount)
        }
        Command::LendToCompany {
            company,
            amount,
            rate,
            years,
        } => lend(state, catalog, *company, *amount, (*rate, *years)),
        Command::WithdrawCapital { company, amount } => withdraw(state, catalog, *company, *amount),
        _ => Err(CommandError::PersonOnly),
    }
}

fn pay_from_account(person: &Person, amount: Money) -> Result<(), CommandError> {
    if amount <= Money::ZERO {
        return Err(CommandError::InvalidAmount);
    }
    if person.account.balance < amount {
        return Err(CommandError::NotEnoughPrivateMoney { needed: amount });
    }
    Ok(())
}

/// `FoundCompany`: the capital goes into the new company, the founding costs leave the
/// game; the person holds all shares and leads it.
pub(crate) fn found(
    state: &mut GameState,
    catalog: &Catalog,
    name: &str,
    form: StartForm,
    country: CountryId,
    capital: Money,
) -> Result<CompanyId, CommandError> {
    if state.main_company.is_some() {
        return Err(CommandError::AlreadyFounded);
    }
    if crate::ids::Id::index(country) >= catalog.countries.len() {
        return Err(CommandError::UnknownCountry);
    }
    let name =
        crate::command::check_company_name(Some(state), name, None).map_err(CommandError::Name)?;
    if capital <= Money::ZERO {
        return Err(CommandError::InvalidAmount);
    }
    let min = form_cost(catalog, form);
    if capital < min {
        return Err(CommandError::CapitalTooLow { min });
    }
    let cost = founding_cost(catalog, state, country, capital);
    pay_from_account(&state.person, capital + cost)?;
    book(&mut state.person, PrivateFlow::Capital, -capital);
    book(&mut state.person, PrivateFlow::FoundingCost, -cost);
    let id = crate::game::push_player_company(state, name, country, capital);
    crate::game::start_setup(state, catalog, id, (form, country, capital))
        .map_err(|_| CommandError::CapitalTooLow { min })?;
    state.main_company = Some(id);
    let today = state.date;
    let p = &mut state.person;
    p.ceo = Some(id);
    p.cost_basis.insert(id, capital);
    p.history.push(LifeEvent {
        date: today,
        kind: LifeEventKind::Founded {
            company: id,
            capital,
        },
    });
    state.person.salary = start_salary(catalog, state, id);
    crate::ranking::record(state);
    Ok(id)
}

fn set_salary(state: &mut GameState, catalog: &Catalog, amount: Money) -> Result<(), CommandError> {
    let company = state.main_company.ok_or(CommandError::NoCompany)?;
    if amount.is_negative() {
        return Err(CommandError::InvalidAmount);
    }
    if let Some(max) = salary_max(catalog, state, company)
        && amount > max
    {
        return Err(CommandError::SalaryTooHigh { max });
    }
    state.person.salary = amount;
    Ok(())
}

/// The first day a new lifestyle may start: the next month, and the months of the data
/// after the last change the person chose.
pub fn next_lifestyle_change(catalog: &Catalog, person: &Person, today: Date) -> Date {
    let next_month = today.first_of_month().add_months(1);
    let last = person
        .lifestyles
        .iter()
        .skip(1)
        .filter(|s| !s.forced && s.from <= today)
        .map(|s| s.from)
        .max();
    match last {
        Some(from) => next_month.max(from.add_months(catalog.person.lifestyle_months)),
        None => next_month,
    }
}

fn set_lifestyle(
    state: &mut GameState,
    catalog: &Catalog,
    level: Lifestyle,
) -> Result<(), CommandError> {
    let today = state.date;
    let p = &mut state.person;
    // A change not yet in effect is replaced.
    if p.lifestyles.last().is_some_and(|s| s.from > today) {
        p.lifestyles.pop();
    }
    if lifestyle_at(catalog, p, today) == level {
        return Ok(());
    }
    let from = today.first_of_month().add_months(1);
    let earliest = next_lifestyle_change(catalog, p, today);
    if earliest > from {
        return Err(CommandError::LifestyleChangedRecently { from: earliest });
    }
    p.lifestyles.push(LifestyleSpell {
        from,
        level,
        forced: false,
    });
    p.history.push(LifeEvent {
        date: today,
        kind: LifeEventKind::LifestyleChanged { level },
    });
    Ok(())
}

fn own_company(state: &GameState, company: CompanyId) -> Result<(), CommandError> {
    let c = state
        .company(company)
        .ok_or(CommandError::UnknownCompany(company))?;
    if c.bankrupt {
        return Err(CommandError::CompanyBankrupt);
    }
    if !is_controlled(state, company) {
        return Err(CommandError::NotControlled);
    }
    Ok(())
}

/// `ContributeCapital`: into the cash as subscribed capital; with co-owners the
/// person's share grows by the company's value.
fn contribute(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    amount: Money,
) -> Result<(), CommandError> {
    own_company(state, company)?;
    pay_from_account(&state.person, amount)?;
    let value = company_value(catalog, state, company);
    let held = crate::person::share(state, company);
    book(&mut state.person, PrivateFlow::Capital, -amount);
    *state.person.cost_basis.entry(company).or_default() += amount;
    let c = &mut state.companies[company.index()];
    c.ledger.transfer(Account::Cash, Account::Equity, amount);
    if held < 1.0 - 1e-9 {
        let (v, e) = (value.to_usd().max(0.0), amount.to_usd());
        let keep = if v + e > 0.0 { v / (v + e) } else { 0.0 };
        let mut owners: Vec<Stake> = c
            .owners
            .iter()
            .filter(|s| s.holder != Holder::Player)
            .map(|s| Stake {
                holder: s.holder,
                share: s.share * keep,
            })
            .collect();
        owners.insert(
            0,
            Stake {
                holder: Holder::Player,
                share: held * keep + (1.0 - keep),
            },
        );
        c.owners = owners;
    }
    Ok(())
}

/// `LendToCompany`: an annuity loan of the person, booked as a loan of the company.
fn lend(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    amount: Money,
    (rate, years): (f64, u32),
) -> Result<(), CommandError> {
    own_company(state, company)?;
    let m = &catalog.person;
    if !(rate.is_finite() && (0.0..=m.loan_max_rate).contains(&rate))
        || years == 0
        || years > m.loan_max_years
    {
        return Err(CommandError::LoanTerms {
            max_rate: m.loan_max_rate,
            max_years: m.loan_max_years,
        });
    }
    pay_from_account(&state.person, amount)?;
    book(&mut state.person, PrivateFlow::LoanGiven, -amount);
    let months = years * 12;
    let today = state.date;
    let c = &mut state.companies[company.index()];
    c.loans.push(Loan {
        principal: amount,
        balance: amount,
        rate,
        start: today,
        months,
        instalment: crate::finance::instalment(amount, rate, months),
        lender: None,
        from_person: true,
    });
    c.ledger.transfer(Account::Cash, Account::Loans, amount);
    Ok(())
}

/// The most capital a company can pay back now: what the person paid in, the
/// subscribed capital, and the cash above the liquidity reserve.
pub fn withdrawal_max(catalog: &Catalog, state: &GameState, company: CompanyId) -> Money {
    let Some(c) = state.company(company) else {
        return Money::ZERO;
    };
    let paid = state
        .person
        .cost_basis
        .get(&company)
        .copied()
        .unwrap_or(Money::ZERO);
    let reserve = crate::strategy::monthly_cost(catalog, state, company).scale(
        crate::strategy::reserve_months(catalog, state, company, Unit::Board),
    );
    paid.min(c.ledger.balance(Account::Equity))
        .min(c.ledger.cash() - reserve)
        .max(Money::ZERO)
}

/// `WithdrawCapital`: only while the person holds all shares.
fn withdraw(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    amount: Money,
) -> Result<(), CommandError> {
    own_company(state, company)?;
    if amount <= Money::ZERO {
        return Err(CommandError::InvalidAmount);
    }
    if crate::person::share(state, company) < 1.0 - 1e-9 {
        return Err(CommandError::NotSoleOwner);
    }
    let max = withdrawal_max(catalog, state, company);
    if amount > max {
        return Err(CommandError::WithdrawalTooHigh { max });
    }
    state.companies[company.index()]
        .ledger
        .transfer(Account::Equity, Account::Cash, amount);
    if let Some(paid) = state.person.cost_basis.get_mut(&company) {
        *paid -= amount;
    }
    book(&mut state.person, PrivateFlow::CapitalRepaid, amount);
    Ok(())
}

/// Interest and repayment of the person's loans, paid by the companies (`finance`).
pub(crate) fn receive_loan(state: &mut GameState, repayment: Money, interest: Money) {
    if repayment > Money::ZERO {
        book(&mut state.person, PrivateFlow::LoanRepaid, repayment);
    }
    if interest > Money::ZERO {
        book(&mut state.person, PrivateFlow::LoanInterest, interest);
    }
}

/// A dividend to the person, the withholding tax already kept back (PE4).
pub(crate) fn receive_dividend(state: &mut GameState, gross: Money, tax: Money) {
    book(&mut state.person, PrivateFlow::Dividend, gross);
    if tax > Money::ZERO {
        book(&mut state.person, PrivateFlow::DividendTax, -tax);
    }
}

/// The salary of the month as CEO of the main company, booked as personnel costs of the
/// board on the month's last day; from the founding by days.
pub(crate) fn pay_salary(state: &mut GameState, last: Date) {
    let Some(company) = state.person.ceo.filter(|&c| state.is_main(c)) else {
        return;
    };
    let c = &state.companies[company.index()];
    if c.bankrupt || state.person.salary <= Money::ZERO {
        return;
    }
    let start = last.first_of_month();
    let next = last.next_day();
    let from = c.founded.max(start);
    let days = i64::from(from.days_until(next).max(0));
    let month = i64::from(start.days_until(next).max(1));
    let pay = Money::from_units(state.person.salary.units() / 12 * days / month);
    if pay <= Money::ZERO {
        return;
    }
    crate::management::book_personnel(state, company, Unit::Board, pay);
    book(&mut state.person, PrivateFlow::Salary, pay);
}

/// End of a month (docs/FORMELN.md, PE3): interest on the balance, income tax, the
/// lifestyle; an account that falls short warns, and a second month short lowers the
/// lifestyle to the first level.
pub(crate) fn month_end(state: &mut GameState, catalog: &Catalog, last: Date) -> Vec<Message> {
    let mut news = Vec::new();
    let rate = catalog
        .person
        .savings_rate
        .value(state.person.home, last.year_fraction());
    let interest = state.person.account.balance.scale(rate / 12.0);
    if interest > Money::ZERO {
        book(&mut state.person, PrivateFlow::Interest, interest);
    }
    let taxed: Money = [
        PrivateFlow::Salary,
        PrivateFlow::Interest,
        PrivateFlow::LoanInterest,
    ]
    .iter()
    .filter_map(|f| state.person.account.month.get(f))
    .copied()
    .sum();
    let tax = taxed
        .max(Money::ZERO)
        .scale(income_tax(catalog, state))
        .min(state.person.account.balance);
    if tax > Money::ZERO {
        book(&mut state.person, PrivateFlow::IncomeTax, -tax);
    }
    let current = lifestyle_at(catalog, &state.person, last);
    let cost = lifestyle_cost(catalog, state, current);
    let paid = cost.min(state.person.account.balance);
    if paid > Money::ZERO {
        book(&mut state.person, PrivateFlow::Lifestyle, -paid);
    }
    if paid < cost {
        match state.person.short_since {
            None => {
                state.person.short_since = Some(last);
                news.push(
                    Message::new(MessageKind::Warning, keys::PERSON_ACCOUNT_SHORT)
                        .with("betrag", Param::Money(cost - paid)),
                );
            }
            Some(_) if current != Lifestyle::Modest => {
                let from = last.next_day();
                let p = &mut state.person;
                if p.lifestyles.last().is_some_and(|s| s.from > last) {
                    p.lifestyles.pop();
                }
                p.lifestyles.push(LifestyleSpell {
                    from,
                    level: Lifestyle::Modest,
                    forced: true,
                });
                p.history.push(LifeEvent {
                    date: last,
                    kind: LifeEventKind::LifestyleChanged {
                        level: Lifestyle::Modest,
                    },
                });
                news.push(Message::new(
                    MessageKind::Warning,
                    keys::PERSON_LIFESTYLE_LOWERED,
                ));
            }
            Some(_) => {}
        }
    } else {
        state.person.short_since = None;
    }
    news
}

/// First day of a month: the last month's movements are closed, the wealth recorded.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) {
    let a = &mut state.person.account;
    let flows = std::mem::take(&mut a.month);
    a.months.push(PrivateMonth {
        start: date.add_days(-1).first_of_month(),
        flows,
    });
    if a.months.len() > MONTHS_KEPT {
        a.months.remove(0);
    }
    record_wealth(state, catalog);
}

fn record_wealth(state: &mut GameState, catalog: &Catalog) {
    let point = wealth(catalog, state);
    let w = &mut state.person.account.wealth;
    if w.last().is_some_and(|p| p.date == point.date) {
        w.pop();
    }
    w.push(point);
}

/// Movements of the last closed months by kind, summed.
pub fn last_months(state: &GameState) -> BTreeMap<PrivateFlow, Money> {
    let mut sums = BTreeMap::new();
    for m in &state.person.account.months {
        for (&f, &v) in &m.flows {
            *sums.entry(f).or_default() += v;
        }
    }
    sums
}

/// Whether a company is the person's and led by the person as CEO.
pub fn led_by_person(state: &GameState, company: CompanyId) -> bool {
    state.person.ceo == Some(company)
        && crate::management::holder(
            state,
            company,
            &crate::state::Position::new(Unit::Board, Role::Head),
        )
        .is_none()
}
