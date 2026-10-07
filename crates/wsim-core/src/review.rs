//! The strategy review of the CEO with the player (MA5, docs/FORMELN.md): at the end of
//! every period of the mandate a report from the ledger – revenue and result of the
//! company, its continents and goods groups, the goals against what was reached, chances
//! and risks – and up to `antraege_max` proposals of the CEO as concerns.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::command::Command;
use crate::decision::{self, Choice, ChoiceKind, Decision, Topic};
use crate::ids::{ContinentId, GoodsGroupId, ProductId};
use crate::ledger::{CostType, PeriodResult};
use crate::management;
use crate::mandate::{self, ReviewInterval};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{
    CompanyId, Concern, ConcernOption, ConcernReason, ConcernStatus, GameState, ManagerId,
    Position, Role, SiteId, Unit,
};
use crate::strategy;

/// Revenue and result of a part of the company in a period.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Figures {
    pub revenue: Money,
    pub result: Money,
}

/// A goal of the mandate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Goal {
    Growth,
    Margin,
    EquityRatio,
    Rank,
}

impl Goal {
    /// Key of the texts (`ziel.<key>`).
    pub fn key(self) -> &'static str {
        match self {
            Goal::Growth => "wachstum",
            Goal::Margin => "rendite",
            Goal::EquityRatio => "eigenkapitalquote",
            Goal::Rank => "rang",
        }
    }
}

/// A goal against what was reached; `None` where there is no value yet.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GoalCheck {
    pub goal: Goal,
    pub target: f64,
    pub actual: Option<f64>,
}

impl GoalCheck {
    /// Whether the goal is reached: a rank at most the target, the others at least.
    pub fn met(&self) -> Option<bool> {
        self.actual.map(|a| match self.goal {
            Goal::Rank => a <= self.target,
            _ => a >= self.target,
        })
    }
}

/// A chance the CEO sees.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Chance {
    /// A product with a high margin per revenue in the period.
    Product {
        product: ProductId,
        revenue: Money,
        margin: Money,
    },
    /// A proposal of the CEO: its concern.
    Proposal { concern: u32 },
}

/// A risk the CEO sees.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Risk {
    /// A site with a loss in the period.
    Loss { site: SiteId, result: Money },
    /// A goal missed.
    Goal(Goal),
    /// The cash below the liquidity reserve of the company.
    Reserve { cash: Money, reserve: Money },
    /// Loans per total assets over the limit of the mandate.
    Debt { share: f64, max: f64 },
}

/// One strategy review.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Review {
    /// First day after the period.
    pub date: Date,
    /// First day of the first month of the period on the books.
    pub from: Date,
    pub interval: ReviewInterval,
    pub manager: ManagerId,
    pub total: Figures,
    /// Result booked on no site: the company's overhead (salaries of the board, interest …).
    pub overhead: Money,
    pub continents: Vec<(ContinentId, Figures)>,
    /// Revenue and margin of the products of each goods group.
    pub groups: Vec<(GoodsGroupId, Figures)>,
    pub goals: Vec<GoalCheck>,
    pub chances: Vec<Chance>,
    pub risks: Vec<Risk>,
}

/// Whether a period of `interval` ends with the month before `next` (calendar periods).
pub fn period_ends(interval: ReviewInterval, next: Date) -> bool {
    let closed = if next.month() == 1 {
        12
    } else {
        next.month() - 1
    };
    closed.is_multiple_of(interval.months())
}

/// The next review of a period of `interval` from `date` on: the first day after the
/// period that contains `date`.
pub fn next_review(interval: ReviewInterval, date: Date) -> Date {
    let mut d = date.first_of_month().add_months(1);
    while !period_ends(interval, d) {
        d = d.add_months(1);
    }
    d
}

fn revenue(m: &PeriodResult) -> Money {
    m.by_type
        .get(&CostType::Revenue)
        .copied()
        .unwrap_or(Money::ZERO)
}

/// The goals of a company's mandate against today (docs/FORMELN.md, MA5): growth of the
/// revenue of the last twelve closed months against the twelve before, result per
/// revenue of the last twelve months, equity per total assets, place by revenue.
pub fn goals(state: &GameState, company: CompanyId) -> Vec<GoalCheck> {
    let c = &state.companies[company.index()];
    let g = c.mandate.goals;
    let months = &c.ledger.months;
    let last: Vec<&PeriodResult> = months.iter().rev().take(12).collect();
    let before: Vec<&PeriodResult> = months.iter().rev().skip(12).take(12).collect();
    let sum = |ms: &[&PeriodResult], f: fn(&PeriodResult) -> Money| -> Money {
        ms.iter().map(|m| f(m)).sum()
    };
    let growth = (before.len() == 12)
        .then(|| (sum(&last, revenue), sum(&before, revenue)))
        .filter(|(_, b)| *b > Money::ZERO)
        .map(|(a, b)| a.to_usd() / b.to_usd() - 1.0);
    let sales = sum(&last, revenue);
    let margin =
        (sales > Money::ZERO).then(|| sum(&last, PeriodResult::total).to_usd() / sales.to_usd());
    let assets = c.ledger.total_assets();
    let equity_ratio =
        (assets > Money::ZERO).then(|| crate::ranking::equity(c).to_usd() / assets.to_usd());
    let rank = crate::ranking::standing(state, company)
        .revenue
        .map(f64::from);
    [
        (Goal::Growth, g.growth, growth),
        (Goal::Margin, g.margin, margin),
        (Goal::EquityRatio, g.equity_ratio, equity_ratio),
        (Goal::Rank, g.rank.map(f64::from), rank),
    ]
    .into_iter()
    .filter_map(|(goal, target, actual)| {
        target.map(|target| GoalCheck {
            goal,
            target,
            actual,
        })
    })
    .collect()
}

/// The report of a period from the ledger: the last closed months of the period.
pub fn report(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    (manager, next): (ManagerId, Date),
) -> Review {
    let c = &state.companies[company.index()];
    let interval = c.mandate.review;
    let n = usize::try_from(interval.months()).unwrap_or(1);
    let months = &c.ledger.months[c.ledger.months.len().saturating_sub(n)..];
    let from = months.first().and_then(|m| m.start).unwrap_or(next);
    let mut total = Figures::default();
    let mut continents: BTreeMap<ContinentId, Figures> = BTreeMap::new();
    let mut groups: BTreeMap<GoodsGroupId, Figures> = BTreeMap::new();
    let mut sites: BTreeMap<SiteId, Money> = BTreeMap::new();
    let mut products: BTreeMap<ProductId, Figures> = BTreeMap::new();
    let continent = |site: SiteId| {
        catalog
            .countries
            .get(state.sites[site.index()].country)
            .continent
    };
    for m in months {
        total.revenue += revenue(m);
        total.result += m.total();
        for (&site, &v) in &m.by_site {
            continents.entry(continent(site)).or_default().result += v;
            *sites.entry(site).or_default() += v;
        }
        for (&site, &v) in &m.site_revenue {
            continents.entry(continent(site)).or_default().revenue += v;
        }
        for (&p, &v) in &m.by_product {
            groups
                .entry(catalog.products.get(p).goods_group)
                .or_default()
                .result += v;
            products.entry(p).or_default().result += v;
        }
        for (&p, &v) in &m.product_revenue {
            groups
                .entry(catalog.products.get(p).goods_group)
                .or_default()
                .revenue += v;
            products.entry(p).or_default().revenue += v;
        }
    }
    let on_sites: Money = sites.values().copied().sum();
    let goals = goals(state, company);
    let shown = usize::try_from(catalog.management.mandate.chances_risks).unwrap_or(0);

    // Chances: the products with the highest margin per revenue.
    let mut best: Vec<(ProductId, Figures)> = products
        .into_iter()
        .filter(|(_, f)| f.revenue > Money::ZERO && f.result > Money::ZERO)
        .collect();
    best.sort_by(|a, b| {
        let share = |f: &Figures| f.result.to_usd() / f.revenue.to_usd();
        share(&b.1).total_cmp(&share(&a.1)).then(a.0.cmp(&b.0))
    });
    let chances = best
        .into_iter()
        .take(shown)
        .map(|(product, f)| Chance::Product {
            product,
            revenue: f.revenue,
            margin: f.result,
        })
        .collect();

    // Risks: the largest losses of sites, goals missed, the reserve, the debt.
    let mut losses: Vec<(SiteId, Money)> = sites
        .into_iter()
        .filter(|(_, v)| *v < Money::ZERO)
        .collect();
    losses.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
    let mut risks: Vec<Risk> = losses
        .into_iter()
        .take(shown)
        .map(|(site, result)| Risk::Loss { site, result })
        .collect();
    risks.extend(
        goals
            .iter()
            .filter(|g| g.met() == Some(false))
            .map(|g| Risk::Goal(g.goal)),
    );
    let months_reserve = strategy::reserve_months(catalog, state, company, Unit::Board);
    if months_reserve > 0.0 {
        let reserve = strategy::monthly_cost(catalog, state, company).scale(months_reserve);
        let cash = c.ledger.cash();
        if cash < reserve {
            risks.push(Risk::Reserve { cash, reserve });
        }
    }
    if let Some(max) = c.mandate.max_debt {
        let share = mandate::debt_share(state, company, Money::ZERO);
        if share > max {
            risks.push(Risk::Debt { share, max });
        }
    }

    Review {
        date: next,
        from,
        interval,
        manager,
        total,
        overhead: total.result - on_sites,
        continents: continents.into_iter().collect(),
        groups: groups.into_iter().collect(),
        goals,
        chances,
        risks,
    }
}

/// The CEO's proposals (docs/FORMELN.md, MA5): new chains or countries and the best
/// offer, up to `antraege_max`, each with the CEO's recommendation; none about what an
/// open concern already asks.
fn proposals(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    (manager, next): (ManagerId, Date),
) -> Vec<Concern> {
    let m = &catalog.management;
    let max = usize::try_from(m.mandate.proposals_max).unwrap_or(0);
    if max == 0 {
        return Vec::new();
    }
    let c = &state.companies[company.index()];
    let offer = crate::deals::best_deal_for(state, catalog, company, Some(&c.mandate)).map(
        |(object, seller, price)| {
            Decision::new(
                crate::deals::deal_topic(object),
                company,
                Choice::one(
                    ChoiceKind::Offer,
                    Command::MakeOffer {
                        seller,
                        object,
                        price,
                    },
                ),
            )
        },
    );
    let mut decisions =
        crate::ai::opportunities(state, catalog, company, max - usize::from(offer.is_some()));
    decisions.extend(offer);
    let open: Vec<(Topic, Option<Unit>, Option<ProductId>)> = state
        .concerns
        .iter()
        .filter(|x| x.company == company && x.status == ConcernStatus::Open)
        .map(|x| {
            (
                x.decision.topic,
                management::place_of(state, &x.decision),
                x.decision.product,
            )
        })
        .collect();
    let x = &state.managers[&manager];
    let mut rng = SimRng::for_stream(
        state.settings.seed,
        Stream::Review {
            company: company.0,
            month: u32::try_from(next.year()).unwrap_or(0) * 12 + next.month(),
        },
    );
    let ceo = Position::new(Unit::Board, Role::Head);
    let mut out = Vec::new();
    for d in decisions {
        let key = (d.topic, management::place_of(state, &d), d.product);
        if open.contains(&key) {
            continue;
        }
        let assessments = decision::assess(catalog, state, &d);
        let skill = management::function_of(catalog, d.topic).map_or(0.0, |f| {
            f64::from(management::expertise(x, &m.functions[f].key)) * (1.0 - m.head_discount)
        });
        let recommended = management::recommend_with(
            &m.concerns,
            &mut rng,
            (&d, &assessments),
            (skill, f64::from(x.judgment)),
        );
        let error = m.concerns.estimate_error * (1.0 - skill / 100.0);
        let options = assessments
            .iter()
            .map(|a| ConcernOption {
                amount: a.amount,
                forecast: a.effect.map(|e| management::forecast(e, error)),
                once: a.once,
            })
            .collect();
        out.push(Concern {
            id: 0,
            company,
            position: ceo.clone(),
            manager,
            decision: d,
            recommended,
            options,
            reason: ConcernReason::Proposal,
            path: Vec::new(),
            parts: Vec::new(),
            created: next,
            deadline: next.add_days(i32::try_from(m.concerns.deadline_days).unwrap_or(i32::MAX)),
            status: ConcernStatus::Open,
            closed: None,
        });
    }
    out
}

/// The reviews due after a month end, once the books of the month are closed and the
/// places recorded: for every company with a CEO whose period ended. Returns the
/// messages for the player.
pub fn month_end(state: &mut GameState, catalog: &Catalog, next: Date) -> Vec<Message> {
    if !catalog.management.enabled() {
        return Vec::new();
    }
    let due: Vec<(CompanyId, ManagerId)> = (0..state.companies.len())
        .map(|i| CompanyId(u32::try_from(i).unwrap_or(u32::MAX)))
        .filter(|&id| {
            let c = &state.companies[id.index()];
            !c.bankrupt && c.ai.is_none() && period_ends(c.mandate.review, next)
        })
        .filter_map(|id| management::ceo_of(state, id).map(|m| (id, m)))
        .collect();
    let mut news = Vec::new();
    let kept = usize::try_from(catalog.management.mandate.reviews_kept).unwrap_or(0);
    for (company, manager) in due {
        let mut review = report(catalog, state, company, (manager, next));
        let concerns = proposals(catalog, state, company, (manager, next));
        let count = concerns.len();
        for mut concern in concerns {
            concern.id = state.next_concern;
            state.next_concern += 1;
            review.chances.push(Chance::Proposal {
                concern: concern.id,
            });
            state.concerns.push(concern);
        }
        if company == state.player {
            let ceo = Position::new(Unit::Board, Role::Head);
            news.push(
                Message::new(MessageKind::Info, keys::REVIEW)
                    .with("stelle", management::position_param(state, &ceo))
                    .with("name", Param::Text(state.managers[&manager].name.clone()))
                    .with("von", Param::Date(review.from))
                    .with("bis", Param::Date(next.add_days(-1)))
                    .with(
                        "antraege",
                        Param::Integer(i64::try_from(count).unwrap_or(i64::MAX)),
                    ),
            );
        }
        let reviews = &mut state.companies[company.index()].reviews;
        reviews.push(review);
        let excess = reviews.len().saturating_sub(kept);
        reviews.drain(..excess);
    }
    news
}
