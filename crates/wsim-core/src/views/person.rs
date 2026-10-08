//! The player as a person (PE2): profile, family, roles and chronicle.

use serde::{Deserialize, Serialize};

use super::{MessageView, iso, message_view, usd};
use crate::aging;
use crate::game::Game;
use crate::management;
use crate::message::{Message, MessageKind, Param};
use crate::money::Money;
use crate::person;
use crate::private;
use crate::state::{LifeEventKind, Lifestyle, Position, PrivateFlow, Role, Unit};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChildView {
    pub name: String,
    pub born: String,
    /// Age today, or at death.
    pub age: u32,
    pub died: Option<String>,
    /// The manager card: its id and, while employed, the position in words.
    pub manager: Option<u32>,
    pub position: Option<MessageView>,
    /// Years until the manager card.
    pub card_in: Option<u32>,
}

/// A company the person holds shares of.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HoldingView {
    /// Index of the company (commands name it).
    #[serde(default)]
    pub index: u32,
    pub company: String,
    pub share: f64,
    pub controlled: bool,
    /// The person leads it as CEO; otherwise the manager who does, if any.
    pub person_ceo: bool,
    pub ceo: Option<String>,
    /// The share at the company's value (PE3).
    #[serde(default)]
    pub value_usd: f64,
    /// What the person paid in and has not taken back.
    #[serde(default)]
    pub cost_basis_usd: f64,
    /// The most capital the company can pay back now (only as sole owner).
    #[serde(default)]
    pub withdraw_max_usd: f64,
    #[serde(default)]
    pub cash_usd: f64,
    /// The person's loans to the company.
    #[serde(default)]
    pub loans: Vec<PersonLoanView>,
    /// What investors pay for the whole share at once (PE5).
    #[serde(default)]
    pub investors_bid_usd: f64,
    /// The person can choose it as main company, and it is the main company now.
    #[serde(default)]
    pub selectable: bool,
    #[serde(default)]
    pub main: bool,
}

/// A loan of the person to a company (PE3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersonLoanView {
    /// Position among the company's loans (`RepayLoan`).
    pub index: u32,
    pub balance_usd: f64,
    pub rate: f64,
    pub instalment_usd: f64,
}

/// A level of lifestyle with its cost today and its effects (PE3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LifestyleOption {
    /// `bescheiden`, `buergerlich`, `gehoben`, `luxurioes` (text `lebensstil.<key>`).
    pub key: String,
    pub cost_usd: f64,
    pub interest: f64,
    pub salary_demand: f64,
    pub education: f64,
    pub mortality: f64,
}

/// The person's money (PE3): account, wealth, income and spending, lifestyle, salary.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersonMoneyView {
    pub cash_usd: f64,
    pub shares_usd: f64,
    pub loans_usd: f64,
    pub wealth_usd: f64,
    /// Wealth on the first day of each month, the earliest first.
    pub history: Vec<(String, f64)>,
    /// Income and spending of the last twelve closed months by kind (text
    /// `privat.<key>`), income positive.
    pub flows: Vec<(String, f64)>,
    pub lifestyle: String,
    /// A change chosen for the next month.
    pub lifestyle_next: Option<String>,
    /// First day the lifestyle may change again.
    pub lifestyle_change_from: String,
    pub lifestyles: Vec<LifestyleOption>,
    /// The account fell short of the lifestyle.
    pub short: bool,
    /// Yearly salary before tax, and what is paid now: only as CEO of the main company.
    pub salary_usd: f64,
    pub salary_paid: bool,
    pub salary_suggestion_usd: f64,
    pub salary_max_usd: Option<f64>,
    pub income_tax: f64,
    pub savings_rate: f64,
    pub loan_max_rate: f64,
    pub loan_max_years: u32,
}

/// What the founding dialog needs (PE3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FoundingView {
    /// Start forms with the capital they need at least.
    pub forms: Vec<super::StartFormOption>,
    /// The person's home: the suggested seat.
    pub country: String,
    pub capital_suggestion_usd: f64,
    /// Founding costs: this share of the capital, at least this amount in the home.
    pub cost_share: f64,
    pub cost_min_usd: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LifeEventView {
    pub date: String,
    pub text: MessageView,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersonView {
    pub name: String,
    pub born: String,
    pub age: u32,
    /// Country of residence (ISO).
    pub home: String,
    pub married: bool,
    pub children: Vec<ChildView>,
    /// Children still possible: the person is married and of age for them.
    pub children_possible: bool,
    pub holdings: Vec<HoldingView>,
    /// The chronicle, the latest first.
    pub history: Vec<LifeEventView>,
    #[serde(default)]
    pub money: Option<PersonMoneyView>,
    /// While the person has no company yet: the founding.
    #[serde(default)]
    pub founding: Option<FoundingView>,
    /// Heir, estate and tax (PE6).
    #[serde(default)]
    pub succession: Option<SuccessionView>,
}

/// Who inherits and what it costs (PE6).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SuccessionView {
    /// The chosen child (index), if any; otherwise the rule decides.
    pub chosen: Option<u32>,
    /// The heir by rule today; `None`: a nephew or niece.
    pub heir: Option<String>,
    /// Living children to choose from.
    pub choices: Vec<HeirChoiceView>,
    pub estate_usd: f64,
    pub tax_rate: f64,
    pub tax_usd: f64,
    /// Chance to die within the next year.
    pub death_chance_year: f64,
    /// Generation of the person, the first is 1.
    pub generation: u32,
    /// The persons before, the earliest first.
    pub ancestors: Vec<AncestorView>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HeirChoiceView {
    pub index: u32,
    pub name: String,
    pub age: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AncestorView {
    pub name: String,
    pub born: String,
    pub until: String,
    pub died: bool,
}

pub fn person(game: &Game) -> PersonView {
    let state = game.state();
    let catalog = game.catalog();
    let p = &state.person;
    let today = state.date;
    let children = p
        .children
        .iter()
        .map(|c| {
            let manager = c
                .manager
                .and_then(|id| state.managers.get(&id).map(|m| (id, m)));
            let position = manager.and_then(|(_, m)| {
                let job = m.job.as_ref()?;
                Some(message_view(
                    &Message::new(MessageKind::Info, "person.stelle")
                        .with("stelle", management::position_param(state, &job.position))
                        .with(
                            "ort",
                            crate::staffing::place_param(catalog, state, job.position.unit),
                        )
                        .with(
                            "firma",
                            Param::Text(state.companies[job.company.index()].name.clone()),
                        ),
                ))
            });
            let age = aging::age(c.born, c.died.unwrap_or(today));
            let card = catalog.person.card_age;
            ChildView {
                name: c.name.clone(),
                born: iso(c.born),
                age,
                died: c.died.map(iso),
                manager: manager.map(|(id, _)| id.0),
                position,
                card_in: (c.died.is_none() && age < card).then(|| card - age),
            }
        })
        .collect();
    let model = &catalog.person;
    let age = person::age(state, today);
    let holdings = state
        .companies
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            // Few companies; the cast is exact.
            let id = crate::state::CompanyId(i as u32);
            let share = person::share(state, id);
            (share > 0.0).then(|| HoldingView {
                index: u32::try_from(i).unwrap_or(u32::MAX),
                company: c.name.clone(),
                share,
                controlled: share > 0.5,
                person_ceo: p.ceo == Some(id),
                ceo: management::holder(state, id, &Position::new(Unit::Board, Role::Head))
                    .map(|m| state.managers[&m].name.clone()),
                value_usd: usd(private::company_value(catalog, state, id).scale(share)),
                cost_basis_usd: usd(p.cost_basis.get(&id).copied().unwrap_or(Money::ZERO)),
                withdraw_max_usd: if share >= 1.0 - 1e-9 {
                    usd(private::withdrawal_max(catalog, state, id))
                } else {
                    0.0
                },
                cash_usd: usd(c.ledger.cash()),
                investors_bid_usd: usd(crate::holdings::bid_of_investors(
                    catalog, state, id, share,
                )),
                selectable: crate::holdings::selectable(state).contains(&id),
                main: state.is_main(id),
                loans: c
                    .loans
                    .iter()
                    .enumerate()
                    .filter(|(_, l)| l.from_person)
                    .map(|(j, l)| PersonLoanView {
                        index: u32::try_from(j).unwrap_or(u32::MAX),
                        balance_usd: usd(l.balance),
                        rate: l.rate,
                        instalment_usd: usd(l.instalment),
                    })
                    .collect(),
            })
        })
        .collect();
    let company = |c: crate::state::CompanyId| {
        Param::Text(
            state
                .companies
                .get(c.index())
                .map_or_else(String::new, |x| x.name.clone()),
        )
    };
    let history = p
        .history
        .iter()
        .rev()
        .map(|e| {
            let m = |key: &str| Message::new(MessageKind::Info, key);
            let text = match &e.kind {
                LifeEventKind::Start { company: c } => {
                    m("person.ereignis.start").with("firma", company(*c))
                }
                LifeEventKind::Takeover { company: c } => {
                    m("person.ereignis.uebernahme").with("firma", company(*c))
                }
                LifeEventKind::ChildBorn { name } => {
                    m("person.ereignis.geburt").with("name", Param::Text(name.clone()))
                }
                LifeEventKind::Career { name } => {
                    m("person.ereignis.beruf").with("name", Param::Text(name.clone()))
                }
                LifeEventKind::ChildDied { name } => {
                    m("person.ereignis.kind_tot").with("name", Param::Text(name.clone()))
                }
                LifeEventKind::CeoHandedOver { company: c, to } => m("person.ereignis.vorsitz_ab")
                    .with("firma", company(*c))
                    .with("name", Param::Text(to.clone())),
                LifeEventKind::CeoTakenBack { company: c } => {
                    m("person.ereignis.vorsitz_an").with("firma", company(*c))
                }
                LifeEventKind::Began { money } => {
                    m("person.ereignis.beginn").with("betrag", Param::Money(*money))
                }
                LifeEventKind::Founded {
                    company: c,
                    capital,
                } => m("person.ereignis.gruendung")
                    .with("firma", company(*c))
                    .with("betrag", Param::Money(*capital)),
                LifeEventKind::LifestyleChanged { level } => m("person.ereignis.lebensstil").with(
                    "stufe",
                    Param::TextKey(format!("lebensstil.{}", level.key())),
                ),
                LifeEventKind::TookControl { company: c } => {
                    m("person.ereignis.kontrolle").with("firma", company(*c))
                }
                LifeEventKind::Sold {
                    company: c,
                    share,
                    proceeds,
                } => m("person.ereignis.verkauf")
                    .with("firma", company(*c))
                    .with("anteil", Param::Number((share * 1000.0).round() / 10.0))
                    .with("betrag", Param::Money(*proceeds)),
                LifeEventKind::Succession {
                    from,
                    to,
                    died,
                    tax,
                } => m(if *died {
                    "person.ereignis.erbfall"
                } else {
                    "person.ereignis.uebergabe"
                })
                .with("name", Param::Text(from.clone()))
                .with("erbe", Param::Text(to.clone()))
                .with("betrag", Param::Money(*tax)),
            };
            LifeEventView {
                date: iso(e.date),
                text: message_view(&text),
            }
        })
        .collect();
    let money = money(game);
    // Founding is open at any time (PE5).
    let founding = Some(founding(game));
    let succession = Some(succession(game));
    PersonView {
        money: Some(money),
        founding,
        succession,
        name: p.name.clone(),
        born: iso(p.born),
        age,
        home: catalog.countries.key(p.home).to_owned(),
        married: p.married,
        children,
        children_possible: model.enabled
            && p.married
            && age < model.children_ages[1]
            && p.children.len() < usize::try_from(model.children_max).unwrap_or(0),
        holdings,
        history,
    }
}

fn succession(game: &Game) -> SuccessionView {
    let state = game.state();
    let catalog = game.catalog();
    let p = &state.person;
    let today = state.date;
    let estate = crate::heirs::estate(catalog, state).max(Money::ZERO);
    let rate = crate::heirs::tax_rate(catalog, state);
    let month = crate::heirs::death_chance(catalog, state, today);
    SuccessionView {
        chosen: p.heir,
        heir: crate::heirs::heir(state).map(|i| p.children[i].name.clone()),
        choices: p
            .children
            .iter()
            .enumerate()
            .filter(|(_, c)| c.died.is_none())
            .map(|(i, c)| HeirChoiceView {
                index: u32::try_from(i).unwrap_or(u32::MAX),
                name: c.name.clone(),
                age: aging::age(c.born, today),
            })
            .collect(),
        estate_usd: usd(estate),
        tax_rate: rate,
        tax_usd: usd(estate.scale(rate)),
        death_chance_year: 1.0 - crate::math::pow(1.0 - month, 12.0),
        generation: u32::try_from(p.ancestors.len()).unwrap_or(0) + 1,
        ancestors: p
            .ancestors
            .iter()
            .map(|a| AncestorView {
                name: a.name.clone(),
                born: iso(a.born),
                until: iso(a.until),
                died: a.died,
            })
            .collect(),
    }
}

/// Text key of a kind of movement on the private account.
fn flow_key(f: PrivateFlow) -> &'static str {
    match f {
        PrivateFlow::StartMoney => "privat.startgeld",
        PrivateFlow::Capital => "privat.einlage",
        PrivateFlow::FoundingCost => "privat.gruendungskosten",
        PrivateFlow::Salary => "privat.gehalt",
        PrivateFlow::IncomeTax => "privat.einkommensteuer",
        PrivateFlow::Interest => "privat.zins",
        PrivateFlow::Lifestyle => "privat.lebensstil",
        PrivateFlow::LoanGiven => "privat.darlehen",
        PrivateFlow::LoanRepaid => "privat.tilgung",
        PrivateFlow::LoanInterest => "privat.darlehenszins",
        PrivateFlow::CapitalRepaid => "privat.rueckzahlung",
        PrivateFlow::InheritanceTax => "privat.erbschaftsteuer",
        PrivateFlow::Dividend => "privat.dividende",
        PrivateFlow::DividendTax => "privat.quellensteuer",
        PrivateFlow::StakeBought => "privat.anteilskauf",
        PrivateFlow::StakeSold => "privat.anteilsverkauf",
        PrivateFlow::GainTax => "privat.veraeusserungsteuer",
    }
}

fn money(game: &Game) -> PersonMoneyView {
    let state = game.state();
    let catalog = game.catalog();
    let m = &catalog.person;
    let p = &state.person;
    let today = state.date;
    let w = private::wealth(catalog, state);
    let current = private::lifestyle_at(catalog, p, today);
    let next = p
        .lifestyles
        .last()
        .filter(|s| s.from > today)
        .map(|s| s.level.key().to_owned());
    let main = state.main_company;
    let year = today.year_fraction();
    PersonMoneyView {
        cash_usd: usd(w.cash),
        shares_usd: usd(w.shares),
        loans_usd: usd(w.loans),
        wealth_usd: usd(w.total()),
        history: p
            .account
            .wealth
            .iter()
            .map(|x| (iso(x.date), usd(x.total())))
            .collect(),
        flows: private::last_months(state)
            .into_iter()
            .filter(|(_, v)| *v != Money::ZERO)
            .map(|(f, v)| (flow_key(f).to_owned(), usd(v)))
            .collect(),
        lifestyle: current.key().to_owned(),
        lifestyle_next: next,
        lifestyle_change_from: iso(private::next_lifestyle_change(catalog, p, today)),
        lifestyles: Lifestyle::ALL
            .iter()
            .map(|&l| {
                let x = m.lifestyle(l);
                LifestyleOption {
                    key: l.key().to_owned(),
                    cost_usd: usd(private::lifestyle_cost(catalog, state, l)),
                    interest: x.interest,
                    salary_demand: x.salary_demand,
                    education: x.education,
                    mortality: x.mortality,
                }
            })
            .collect(),
        short: p.short_since.is_some(),
        salary_usd: usd(p.salary),
        salary_paid: main.is_some_and(|c| p.ceo == Some(c)),
        salary_suggestion_usd: main
            .map_or(0.0, |c| usd(private::salary_suggestion(catalog, state, c))),
        salary_max_usd: main
            .and_then(|c| private::salary_max(catalog, state, c))
            .map(usd),
        income_tax: m.income_tax.value(p.home, year),
        savings_rate: m.savings_rate.value(p.home, year),
        loan_max_rate: m.loan_max_rate,
        loan_max_years: m.loan_max_years,
    }
}

fn founding(game: &Game) -> FoundingView {
    let state = game.state();
    let catalog = game.catalog();
    let f = &catalog.person.founding;
    let home = state.person.home;
    let forms = super::new_game_options(catalog)
        .start_forms
        .into_iter()
        .map(|mut o| {
            if let Some(form) = super::start_form_from_key(&o.key) {
                o.cost_usd = usd(private::form_cost(catalog, form));
            }
            o
        })
        .collect();
    FoundingView {
        forms,
        country: catalog.countries.key(home).to_owned(),
        capital_suggestion_usd: usd(state.person.account.balance.scale(f.capital_suggestion)),
        cost_share: f.cost_share,
        cost_min_usd: usd(
            private::academic_monthly_wage(catalog, state, home).scale(f.cost_min_months)
        ),
    }
}
