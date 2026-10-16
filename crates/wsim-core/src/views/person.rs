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
    /// At a private school now.
    #[serde(default)]
    pub private_school: bool,
    /// Ambition and interest in the firm 0–100, the branch of its talent (PV4).
    #[serde(default)]
    pub ambition: u32,
    #[serde(default)]
    pub interest: u32,
    #[serde(default)]
    pub talent: Option<String>,
    /// What it studies (`ingenieur`, `kaufmann`, `jurist`, `naturwissenschaftler`, `frei`).
    #[serde(default)]
    pub education: Option<String>,
    /// From this age on the education can be chosen; the child's age is `age`.
    #[serde(default)]
    pub can_study: bool,
}

/// What the person can do privately and what it costs a month (USD).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PrivateLifeView {
    pub seeking_partner: bool,
    pub partner_cost_usd: f64,
    /// Chance per month to find a partner.
    pub partner_chance: f64,
    pub school_cost_usd: f64,
    /// Points on every skill for a whole childhood at a private school.
    pub school_education: f64,
    pub health_care: bool,
    pub care_cost_usd: f64,
    /// Factor on the chance to die.
    pub care_mortality: f64,
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
    /// The person's yearly salary from the company, the suggestion (a solid CEO's) and
    /// the most it may be with co-owners; whether a salary may be set (PE3).
    pub salary_usd: f64,
    pub salary_suggestion_usd: f64,
    pub salary_max_usd: Option<f64>,
    pub salary_possible: bool,
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
    /// The net worth by class (text `privat.vermoegen_<key>`); sums to `wealth_usd`.
    pub classes: Vec<(String, f64)>,
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
    /// Yearly salary before tax, and whether it is paid now: by the company the person
    /// leads as CEO, else by the main company as long as the person owns part of it.
    pub salary_usd: f64,
    pub salary_paid: bool,
    /// Paid as owner, not as CEO.
    #[serde(default)]
    pub salary_as_owner: bool,
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
    /// Partner search, private schools and health care; none without them in the data.
    #[serde(default)]
    pub private_life: Option<PrivateLifeView>,
    pub children: Vec<ChildView>,
    /// Children still possible: the person is married and of age for them.
    pub children_possible: bool,
    pub holdings: Vec<HoldingView>,
    /// The chronicle, the latest first.
    pub history: Vec<LifeEventView>,
    #[serde(default)]
    pub money: Option<PersonMoneyView>,
    /// Background, time budget and health (PV1b).
    #[serde(default)]
    pub wellbeing: Option<WellbeingView>,
    /// Deposits, bonds, gold and funds (PV2).
    #[serde(default)]
    pub assets: Option<AssetsView>,
    /// Standing, clubs, contacts and possessions (PV6a).
    #[serde(default)]
    pub circle: Option<CircleView>,
    /// Family holding and foundation (PV6c).
    #[serde(default)]
    pub trust: Option<TrustView>,
    /// The relationship, waiting events and the family tree (PV6b).
    #[serde(default)]
    pub life: Option<LifeView>,
    /// While the person has no company yet: the founding.
    #[serde(default)]
    pub founding: Option<FoundingView>,
    /// Heir, estate and tax (PE6).
    #[serde(default)]
    pub succession: Option<SuccessionView>,
}

/// The relationship, waiting events and the family tree (PV6b).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LifeView {
    /// The relationship 0–100; none without a partner.
    pub bond: Option<f64>,
    pub contract: bool,
    pub events: Vec<PersonalEventView>,
    /// The persons of the family, the oldest generation first.
    pub tree: Vec<TreeNodeView>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersonalEventView {
    pub id: u32,
    /// Text `ereignis.<key>` and, per option, `ereignis.<key>.<option>`.
    pub key: String,
    pub options: Vec<PersonalEventOptionView>,
    pub until: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersonalEventOptionView {
    pub key: String,
    pub cost_usd: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TreeNodeView {
    /// −n for the n-th predecessor, 0 for the person, 1 for the children.
    pub generation: i32,
    pub name: String,
    pub born: String,
    pub died: Option<String>,
    /// `person`, `vorgaenger`, `kind`, or `partner`.
    pub role: String,
    /// What the person does in the company, if anything.
    pub position: Option<MessageView>,
}

/// Family holding and foundation (PV6c).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrustView {
    pub holding: bool,
    pub holding_cost_usd: f64,
    pub capital_usd: f64,
    pub foundation_min_usd: f64,
    pub gift_tax: f64,
    pub yield_year: f64,
    pub payout_year_usd: f64,
    pub standing_points: f64,
    /// Companies whose dividends the co-heirs share: company name, share.
    pub claims: Vec<(String, f64)>,
}

/// Standing, clubs, contacts and possessions (PV6a).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CircleView {
    pub standing: f64,
    pub target: f64,
    /// Offset on the rate of a new loan in points (negative: cheaper) and the discount
    /// the broker gets on properties.
    pub loan_offset: f64,
    pub broker_discount: f64,
    pub clubs: Vec<ClubView>,
    pub contacts: Vec<ContactView>,
    pub goods: Vec<GoodView>,
    pub owned: Vec<OwnedGoodView>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClubView {
    pub key: String,
    pub standing_min: f64,
    pub lifestyle_min: u32,
    pub fee_usd: f64,
    pub member: bool,
    /// Whether the person could join now.
    pub open: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContactView {
    pub role: String,
    pub name: String,
    pub strength: f64,
    pub until: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GoodView {
    pub key: String,
    pub price_usd: f64,
    pub upkeep: f64,
    pub representation: f64,
    pub available: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnedGoodView {
    pub key: String,
    pub value_usd: f64,
}

/// What the person holds besides companies and the terms for more (PV2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetsView {
    pub deposits: Vec<DepositView>,
    pub bonds: Vec<GovBondView>,
    pub gold_ounces: f64,
    pub gold_price_usd: f64,
    pub gold_value_usd: f64,
    pub gold_cost_usd: f64,
    /// Fee and half spread of gold, as shares.
    pub gold_fee: f64,
    pub gold_spread: f64,
    pub funds: Vec<FundView>,
    /// The plan: fund key and monthly amount.
    pub plan: Option<(String, f64)>,
    /// Rates of new deposits and coupons of new bonds now.
    pub deposit_rate: f64,
    pub bond_coupon: f64,
    pub deposit_months: [u32; 2],
    pub bond_months: [u32; 2],
    pub deposit_penalty: f64,
    pub bond_spread: f64,
    /// Real estate (PV3); `None` if the data has none.
    pub property: Option<PropertyView>,
}

/// Properties held and for sale (PV3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropertyView {
    pub owned: Vec<OwnedPropertyView>,
    pub offers: Vec<PropertyOfferView>,
    /// Least equity share, costs of buying as a share, rent band, mortgage rate now.
    pub equity_min: f64,
    pub costs: f64,
    pub rent_band: [f64; 2],
    pub mortgage_rate: f64,
    pub sale_discount: f64,
    pub renovation_cost: f64,
    /// Sites of the person's firms a property may be let to (PV5c).
    pub sites: Vec<TenantSiteView>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TenantSiteView {
    pub index: u32,
    pub name: Option<String>,
    pub country: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnedPropertyView {
    /// Text `person.immobilie_<kind>`.
    pub kind: String,
    pub country: String,
    pub condition: f64,
    pub value_usd: f64,
    pub paid_usd: f64,
    pub market_rent_usd: f64,
    pub rent_factor: f64,
    pub vacancy: f64,
    pub debt_usd: f64,
    pub renovating_until: Option<String>,
    /// Index of the site it is let to.
    pub tenant: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropertyOfferView {
    pub id: u32,
    pub kind: String,
    pub country: String,
    pub condition: f64,
    pub price_usd: f64,
    pub market_rent_usd: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DepositView {
    pub principal_usd: f64,
    pub rate: f64,
    pub until: String,
    pub paid_usd: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GovBondView {
    pub face_usd: f64,
    pub coupon: f64,
    pub until: String,
    pub quote: f64,
    pub value_usd: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FundView {
    /// `gemanagt` or `index` (text `person.fonds_<key>`).
    pub key: String,
    pub open: bool,
    pub from: i32,
    pub units: f64,
    pub nav: f64,
    pub value_usd: f64,
    pub cost_usd: f64,
    /// Yearly running cost, load on money paid in, order fee.
    pub running_cost: f64,
    pub load: f64,
    pub fee: f64,
}

/// Background, time and health of the person (PV1b).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WellbeingView {
    /// The background (text `person.hintergrund_<key>`), if one was chosen.
    pub background: Option<String>,
    /// Effects now: research and loyalty as shares, loan rate in points.
    pub research: f64,
    pub loyalty: f64,
    pub loan_rate: f64,
    pub experience: f64,
    /// Shares of the time in percent: firm, family, network, rest.
    pub time: [u32; 4],
    /// The least share for the firm.
    pub min_firm: f64,
    /// Days the concerns wait less for lack of time.
    pub delay_days: i32,
    pub health: f64,
    /// Factor on the chance to die by health.
    pub mortality_factor: f64,
    /// The day the absence ends, while the person is ill.
    pub ill_until: Option<String>,
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
    /// The shares of the will and, if none is set, the statutory succession (PV4).
    #[serde(default)]
    pub shares: Vec<EstateShareView>,
    #[serde(default)]
    pub will_set: bool,
    /// Whether the data has children's traits, education and succession shares.
    #[serde(default)]
    pub shares_enabled: bool,
    /// Shares of companies the co-heirs hold: company and share.
    #[serde(default)]
    pub heir_stakes: Vec<(String, f64)>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EstateShareView {
    /// `partner`, or the index of a child.
    pub partner: bool,
    pub child: Option<u32>,
    pub name: String,
    pub percent: f64,
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
    /// The obituary (PV6b).
    #[serde(default)]
    pub wealth_usd: f64,
    #[serde(default)]
    pub firms: u32,
    #[serde(default)]
    pub workers: u32,
    #[serde(default)]
    pub standing: f64,
    #[serde(default)]
    pub children: u32,
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
                private_school: c.private_school,
                ambition: c.ambition,
                interest: c.interest,
                talent: c.talent.map(|b| catalog.branches.key(b).to_owned()),
                education: c.education.map(|e| e.key().to_owned()),
                can_study: catalog
                    .person
                    .legacy
                    .as_ref()
                    .is_some_and(|l| age >= l.education_from)
                    && c.died.is_none()
                    && c.manager.is_none(),
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
                salary_usd: usd(private::salary_of(state, id)),
                salary_suggestion_usd: usd(private::salary_suggestion(catalog, state, id)),
                salary_max_usd: private::salary_max(catalog, state, id).map(usd),
                salary_possible: private::pays_salary(state, id),
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
                LifeEventKind::Married => m("person.ereignis.heirat"),
                LifeEventKind::Separated { paid } => {
                    m("person.ereignis.trennung").with("betrag", Param::Money(*paid))
                }
                LifeEventKind::Personal { key, option } => m("person.ereignis.persoenlich")
                    .with("ereignis", Param::TextKey(format!("lebensereignis.{key}")))
                    .with(
                        "antwort",
                        Param::TextKey(format!("lebensereignis.{key}.{option}")),
                    ),
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
    let wellbeing = crate::wellbeing::time(catalog, state).map(|t| {
        let e = crate::wellbeing::effect(catalog, state);
        WellbeingView {
            background: p.background.map(|b| b.key().to_owned()),
            research: e.research,
            loyalty: e.loyalty,
            loan_rate: e.loan_rate,
            experience: p.experience,
            time: [t.firm, t.family, t.network, t.rest],
            min_firm: crate::wellbeing::min_firm(catalog, state),
            delay_days: crate::wellbeing::delay_days(catalog, state),
            health: crate::wellbeing::health(catalog, state).unwrap_or(0.0),
            mortality_factor: crate::wellbeing::mortality_factor(catalog, state),
            ill_until: crate::wellbeing::is_ill(state, today)
                .then(|| p.ill_until.map(iso))
                .flatten(),
        }
    });
    let assets = catalog.person.assets.as_ref().map(|m| {
        let pf = &state.person.portfolio;
        AssetsView {
            deposits: pf
                .deposits
                .iter()
                .map(|d| DepositView {
                    principal_usd: usd(d.principal),
                    rate: d.rate,
                    until: iso(d.until),
                    paid_usd: usd(d.paid),
                })
                .collect(),
            bonds: pf
                .bonds
                .iter()
                .map(|b| {
                    let quote = crate::savings::bond_quote(catalog, state, m, b);
                    GovBondView {
                        face_usd: usd(b.face),
                        coupon: b.coupon,
                        until: iso(b.until),
                        quote,
                        value_usd: usd(b.face.scale(quote)),
                    }
                })
                .collect(),
            gold_ounces: pf.gold_ounces,
            gold_price_usd: usd(state.assets.gold_price),
            gold_value_usd: usd(crate::savings::gold_value(state)),
            gold_cost_usd: usd(pf.gold_cost),
            gold_fee: crate::savings::order_fee(catalog, state, m.gold_fee),
            gold_spread: m.gold_spread,
            funds: crate::savings::FundKind::ALL
                .iter()
                .map(|&k| {
                    let t = m.funds[k as usize];
                    let pos = pf.funds[k as usize];
                    FundView {
                        key: k.key().to_owned(),
                        open: crate::savings::fund_open(catalog, state, k),
                        from: t.from,
                        units: pos.units,
                        nav: crate::savings::nav(state, k),
                        value_usd: pos.units * crate::savings::nav(state, k),
                        cost_usd: usd(pos.cost),
                        running_cost: t.cost,
                        load: t.load,
                        fee: crate::savings::order_fee(catalog, state, t.fee),
                    }
                })
                .collect(),
            plan: pf.plan.map(|p| (p.fund.key().to_owned(), usd(p.amount))),
            deposit_rate: crate::savings::deposit_rate(catalog, state, m),
            bond_coupon: crate::savings::bond_coupon(catalog, state, m),
            deposit_months: m.deposit_months,
            bond_months: m.bond_months,
            deposit_penalty: m.deposit_penalty,
            bond_spread: m.bond_spread,
            property: m.property.as_ref().map(|pm| PropertyView {
                owned: pf
                    .properties
                    .iter()
                    .map(|p| OwnedPropertyView {
                        kind: p.kind.key().to_owned(),
                        country: catalog.countries.key(p.country).to_owned(),
                        condition: p.condition,
                        value_usd: usd(crate::property::value(catalog, state, pm, p)),
                        paid_usd: usd(p.paid),
                        market_rent_usd: usd(crate::property::market_rent(catalog, state, pm, p)),
                        rent_factor: p.rent_factor,
                        vacancy: crate::property::vacancy(pm, p),
                        debt_usd: usd(p.mortgage.map_or(Money::ZERO, |h| h.balance)),
                        renovating_until: p.renovating_until.map(iso),
                        tenant: p.tenant,
                    })
                    .collect(),
                offers: state
                    .assets
                    .offers
                    .iter()
                    .map(|o| PropertyOfferView {
                        id: o.id,
                        kind: o.kind.key().to_owned(),
                        country: catalog.countries.key(o.country).to_owned(),
                        condition: o.condition,
                        price_usd: usd(o.price),
                        market_rent_usd: usd(crate::property::value_of(
                            catalog,
                            state,
                            pm,
                            (o.kind, o.country, o.quality, o.condition),
                        )
                        .scale(pm.kinds[o.kind as usize].rent_yield)),
                    })
                    .collect(),
                equity_min: pm.equity_min,
                costs: pm.costs,
                rent_band: pm.rent_band,
                mortgage_rate: crate::property::mortgage_rate(catalog, state, pm),
                sale_discount: pm.sale_discount,
                renovation_cost: pm.reno_cost,
                sites: state
                    .sites
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| crate::private::is_controlled(state, s.owner))
                    .map(|(i, s)| TenantSiteView {
                        index: u32::try_from(i).unwrap_or(u32::MAX),
                        name: s.name.clone(),
                        country: catalog.countries.key(s.country).to_owned(),
                    })
                    .collect(),
            }),
        }
    });
    PersonView {
        assets,
        wellbeing,
        circle: circle_view(game),
        trust: trust_view(game),
        life: life_view(game),
        money: Some(money),
        founding,
        succession,
        name: p.name.clone(),
        born: iso(p.born),
        age,
        home: catalog.countries.key(p.home).to_owned(),
        married: p.married,
        private_life: model.private_life.enabled.then(|| {
            let pl = &model.private_life;
            let wage = crate::private::academic_monthly_wage(catalog, state, p.home);
            PrivateLifeView {
                seeking_partner: p.seeking_partner,
                partner_cost_usd: usd(wage.scale(pl.partner_cost)),
                partner_chance: pl.partner_chance,
                school_cost_usd: usd(wage.scale(pl.school_cost)),
                school_education: pl.school_education,
                health_care: p.health_care,
                care_cost_usd: usd(wage.scale(pl.care_cost)),
                care_mortality: pl.care_mortality,
            }
        }),
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
        tax_usd: usd(crate::heirs::tax_due(catalog, state)),
        death_chance_year: 1.0 - crate::math::pow(1.0 - month, 12.0),
        shares_enabled: catalog.person.legacy.is_some(),
        will_set: !p.will.is_empty(),
        shares: crate::heirs::shares(catalog, state)
            .into_iter()
            .map(|(h, f)| match h {
                crate::state::Heir::Partner => EstateShareView {
                    partner: true,
                    child: None,
                    name: String::new(),
                    percent: f * 100.0,
                },
                crate::state::Heir::Child(i) => EstateShareView {
                    partner: false,
                    child: Some(i),
                    name: p
                        .children
                        .get(i as usize)
                        .map_or_else(String::new, |c| c.name.clone()),
                    percent: f * 100.0,
                },
            })
            .collect(),
        heir_stakes: p
            .heir_stakes
            .iter()
            .map(|(&c, &q)| (state.companies[c.index()].name.clone(), q))
            .collect(),
        generation: u32::try_from(p.ancestors.len()).unwrap_or(0) + 1,
        ancestors: p
            .ancestors
            .iter()
            .map(|a| AncestorView {
                name: a.name.clone(),
                born: iso(a.born),
                until: iso(a.until),
                died: a.died,
                wealth_usd: usd(a.wealth),
                firms: a.firms,
                workers: a.workers,
                standing: a.standing,
                children: a.children,
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
        PrivateFlow::Family => "privat.familie",
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
        PrivateFlow::Bonus => "privat.sonderzahlung",
        PrivateFlow::AssetBought => "privat.anlagekauf",
        PrivateFlow::AssetSold => "privat.anlageverkauf",
        PrivateFlow::InvestFee => "privat.anlagegebuehr",
        PrivateFlow::Rent => "privat.miete",
        PrivateFlow::AdvisorFee => "privat.beratergebuehr",
        PrivateFlow::InheritancePayout => "privat.erbausgleich",
        PrivateFlow::Education => "privat.ausbildung",
        PrivateFlow::PropertyCost => "privat.immobilienkosten",
        PrivateFlow::MortgageInterest => "privat.hypothekzins",
        PrivateFlow::MortgageRepay => "privat.hypothektilgung",
        PrivateFlow::Donation => "privat.spende",
        PrivateFlow::GoodBought => "privat.besitzkauf",
        PrivateFlow::GoodSold => "privat.besitzverkauf",
        PrivateFlow::CircleCost => "privat.kreiskosten",
        PrivateFlow::Personal => "privat.ereignisse",
        PrivateFlow::Trust => "privat.bindung",
        PrivateFlow::FoundationPayout => "privat.stiftungsertrag",
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
        classes: w
            .classes()
            .into_iter()
            .map(|(c, v)| (c.key().to_owned(), usd(v)))
            .collect(),
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
        salary_usd: usd(private::salary_total(state)),
        salary_paid: private::salary_payer(state).is_some(),
        salary_as_owner: p.ceo.is_none() && private::salary_payer(state).is_some(),
        salary_suggestion_usd: private::salary_payer(state)
            .or(main)
            .map_or(0.0, |c| usd(private::salary_suggestion(catalog, state, c))),
        salary_max_usd: private::salary_payer(state)
            .or(main)
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

fn trust_view(game: &Game) -> Option<TrustView> {
    let catalog = game.catalog();
    let state = game.state();
    let m = crate::trust::model(catalog)?;
    let wage = private::academic_monthly_wage(catalog, state, state.person.home);
    let t = &state.person.trust;
    Some(TrustView {
        holding: t.holding,
        holding_cost_usd: wage.scale(m.holding_cost).to_usd(),
        capital_usd: t.capital.to_usd(),
        foundation_min_usd: wage.scale(12.0 * m.foundation_min).to_usd(),
        gift_tax: m.gift_tax,
        yield_year: m.yield_year,
        payout_year_usd: crate::trust::payout_year(catalog, state).to_usd(),
        standing_points: crate::trust::standing_points(catalog, state),
        claims: t
            .claims
            .iter()
            .filter_map(|(id, share)| {
                state
                    .companies
                    .get(id.index())
                    .map(|c| (c.name.clone(), *share))
            })
            .collect(),
    })
}

fn circle_view(game: &Game) -> Option<CircleView> {
    let catalog = game.catalog();
    let state = game.state();
    let m = crate::standing::model(catalog)?;
    let level = private::lifestyle_at(catalog, &state.person, state.date) as usize;
    let have = crate::standing::standing(catalog, state);
    let wage = private::academic_monthly_wage(catalog, state, state.person.home);
    Some(CircleView {
        standing: have,
        target: crate::standing::target(catalog, state),
        loan_offset: crate::standing::loan_rate_offset(catalog, state),
        broker_discount: crate::standing::broker_discount(catalog, state),
        clubs: m
            .clubs
            .iter()
            .enumerate()
            .map(|(i, c)| ClubView {
                key: c.key.clone(),
                standing_min: c.standing,
                lifestyle_min: u32::from(c.lifestyle),
                fee_usd: usd(wage.scale(c.fee)),
                member: state
                    .person
                    .circle
                    .clubs
                    .contains(&u32::try_from(i).unwrap_or(u32::MAX)),
                open: have >= c.standing && level >= usize::from(c.lifestyle),
            })
            .collect(),
        contacts: state
            .person
            .circle
            .contacts
            .iter()
            .map(|c| ContactView {
                role: c.role.key().to_owned(),
                name: c.name.clone(),
                strength: c.strength,
                until: iso(c.until),
            })
            .collect(),
        goods: m
            .goods
            .iter()
            .enumerate()
            .map(|(i, g)| GoodView {
                key: g.key.clone(),
                price_usd: crate::standing::good_price(catalog, state, i).map_or(0.0, usd),
                upkeep: g.upkeep,
                representation: g.representation,
                available: state.date.year() >= g.from,
            })
            .collect(),
        owned: state
            .person
            .circle
            .goods
            .iter()
            .filter_map(|o| {
                let g = m.goods.get(o.kind as usize)?;
                Some(OwnedGoodView {
                    key: g.key.clone(),
                    value_usd: usd(crate::standing::good_price(catalog, state, o.kind as usize)
                        .unwrap_or(o.paid)
                        .scale(0.8)),
                })
            })
            .collect(),
    })
}

fn life_view(game: &Game) -> Option<LifeView> {
    let catalog = game.catalog();
    let state = game.state();
    let m = crate::personal::model(catalog)?;
    let wage = private::academic_monthly_wage(catalog, state, state.person.home);
    let p = &state.person;
    let events = p
        .life
        .pending
        .iter()
        .filter_map(|e| {
            if e.kind == crate::personal::CONTRACT {
                return Some(PersonalEventView {
                    id: e.id,
                    key: "ehevertrag".to_owned(),
                    options: vec![
                        PersonalEventOptionView {
                            key: "mit".to_owned(),
                            cost_usd: usd(wage.scale(m.contract_cost)),
                        },
                        PersonalEventOptionView {
                            key: "ohne".to_owned(),
                            cost_usd: 0.0,
                        },
                    ],
                    until: iso(e.until),
                });
            }
            let t = m.events.get(e.kind as usize)?;
            Some(PersonalEventView {
                id: e.id,
                key: t.key.clone(),
                options: t
                    .options
                    .iter()
                    .map(|o| PersonalEventOptionView {
                        key: o.key.clone(),
                        cost_usd: usd(wage.scale(o.cost)),
                    })
                    .collect(),
                until: iso(e.until),
            })
        })
        .collect();
    let mut tree: Vec<TreeNodeView> = Vec::new();
    let count = i32::try_from(p.ancestors.len()).unwrap_or(0);
    for (i, a) in p.ancestors.iter().enumerate() {
        tree.push(TreeNodeView {
            generation: i32::try_from(i).unwrap_or(0) - count,
            name: a.name.clone(),
            born: iso(a.born),
            died: a.died.then(|| iso(a.until)),
            role: "vorgaenger".to_owned(),
            position: None,
        });
    }
    tree.push(TreeNodeView {
        generation: 0,
        name: p.name.clone(),
        born: iso(p.born),
        died: None,
        role: "person".to_owned(),
        position: None,
    });
    for c in &p.children {
        tree.push(TreeNodeView {
            generation: 1,
            name: c.name.clone(),
            born: iso(c.born),
            died: c.died.map(iso),
            role: "kind".to_owned(),
            position: c
                .manager
                .and_then(|id| state.managers.get(&id))
                .and_then(|x| x.job.as_ref())
                .map(|j| {
                    message_view(
                        &Message::new(MessageKind::Info, "person.kind_position").with(
                            "stelle",
                            crate::management::position_param(state, &j.position),
                        ),
                    )
                }),
        });
    }
    Some(LifeView {
        bond: crate::personal::bond(catalog, state),
        contract: p.life.contract,
        events,
        tree,
    })
}
