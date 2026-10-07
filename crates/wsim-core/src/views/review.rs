//! The mandate to the board and the strategy reviews of the player's CEO (MA5;
//! docs/BEDIENUNG.md, "Strategieauftrag" and "Rücksprache").

use serde::{Deserialize, Serialize};

use super::{iso, usd};
use crate::command::site_type_key;
use crate::game::Game;
use crate::ids::{GoodsGroupId, Id};
use crate::management;
use crate::mandate::{Guideline, ReviewInterval};
use crate::review::{self, Chance, GoalCheck, Risk};
use crate::state::ConcernStatus;

/// Revenue and result of a part of the company.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FiguresView {
    /// Key of the continent or goods group.
    pub key: String,
    pub revenue_usd: f64,
    /// Result; for a goods group the margin of its products.
    pub result_usd: f64,
}

/// A goal against what was reached.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GoalView {
    /// `wachstum`, `rendite`, `eigenkapitalquote`, `rang`.
    pub goal: String,
    /// Shares as fractions (0.1 = 10 %), the rank as a place.
    pub target: f64,
    pub actual: Option<f64>,
    pub met: Option<bool>,
}

/// A chance in a report.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChanceView {
    /// `produkt` or `antrag`.
    pub kind: String,
    pub product: Option<String>,
    pub revenue_usd: f64,
    pub margin_usd: f64,
    /// The concern of a proposal, its topic and whether it is still open.
    pub concern: Option<u32>,
    pub topic: Option<String>,
    pub open: bool,
}

/// A risk in a report.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RiskView {
    /// `verlust`, `ziel`, `reserve` or `verschuldung`.
    pub kind: String,
    pub site: Option<u32>,
    pub site_kind_text: Option<String>,
    pub country: Option<String>,
    /// The loss, or the cash.
    pub amount_usd: Option<f64>,
    /// The liquidity reserve.
    pub limit_usd: Option<f64>,
    pub goal: Option<String>,
    /// Loans per total assets and the limit of the mandate.
    pub share: Option<f64>,
    pub max: Option<f64>,
}

/// One strategy review.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReviewView {
    pub date: String,
    pub from: String,
    /// Last day of the period.
    pub to: String,
    /// `monatlich`, `quartalsweise`, `halbjaehrlich`, `jaehrlich`.
    pub interval: String,
    pub manager: String,
    pub revenue_usd: f64,
    pub result_usd: f64,
    pub overhead_usd: f64,
    pub continents: Vec<FiguresView>,
    pub groups: Vec<FiguresView>,
    pub goals: Vec<GoalView>,
    pub chances: Vec<ChanceView>,
    pub risks: Vec<RiskView>,
}

/// The mandate as the form shows it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MandateView {
    /// `wachstum`, `ertrag`, `sicherheit` or `marktfuehrung`.
    pub guideline: String,
    /// The goods group of market leadership.
    pub leading_group: Option<String>,
    pub growth: Option<f64>,
    pub margin: Option<f64>,
    pub equity_ratio: Option<f64>,
    pub rank: Option<u32>,
    pub max_debt: Option<f64>,
    pub blocked_countries: Vec<String>,
    pub blocked_groups: Vec<String>,
    /// `monatlich`, `quartalsweise`, `halbjaehrlich`, `jaehrlich`.
    pub review: String,
}

/// A guideline with the aggressiveness it gives the rules.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GuidelineView {
    pub key: String,
    pub aggressiveness: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReviewsView {
    pub enabled: bool,
    pub mandate: MandateView,
    pub guidelines: Vec<GuidelineView>,
    pub intervals: Vec<String>,
    /// Goods groups and countries to choose from.
    pub groups: Vec<String>,
    pub countries: Vec<String>,
    /// The CEO's name; `None` without a CEO (no reviews then).
    pub ceo: Option<String>,
    /// The next review with a CEO.
    pub next_review: Option<String>,
    /// The goals of the mandate against today.
    pub goals_now: Vec<GoalView>,
    /// The reviews kept, the newest first.
    pub reviews: Vec<ReviewView>,
}

fn goal_view(g: &GoalCheck) -> GoalView {
    GoalView {
        goal: g.goal.key().to_owned(),
        target: g.target,
        actual: g.actual,
        met: g.met(),
    }
}

fn review_view(game: &Game, r: &review::Review) -> ReviewView {
    let c = game.catalog();
    let state = game.state();
    let figures = |key: String, f: review::Figures| FiguresView {
        key,
        revenue_usd: usd(f.revenue),
        result_usd: usd(f.result),
    };
    let chances = r
        .chances
        .iter()
        .map(|chance| match *chance {
            Chance::Product {
                product,
                revenue,
                margin,
            } => ChanceView {
                kind: "produkt".into(),
                product: Some(c.products.key(product).to_owned()),
                revenue_usd: usd(revenue),
                margin_usd: usd(margin),
                concern: None,
                topic: None,
                open: false,
            },
            Chance::Proposal { concern } => {
                let x = state.concerns.iter().find(|x| x.id == concern);
                ChanceView {
                    kind: "antrag".into(),
                    product: x
                        .and_then(|x| x.decision.product)
                        .map(|p| c.products.key(p).to_owned()),
                    revenue_usd: 0.0,
                    margin_usd: 0.0,
                    concern: Some(concern),
                    topic: x.map(|x| x.decision.topic.key().to_owned()),
                    open: x.is_some_and(|x| x.status == ConcernStatus::Open),
                }
            }
        })
        .collect();
    let risks = r
        .risks
        .iter()
        .map(|risk| {
            let mut v = RiskView {
                kind: String::new(),
                site: None,
                site_kind_text: None,
                country: None,
                amount_usd: None,
                limit_usd: None,
                goal: None,
                share: None,
                max: None,
            };
            match *risk {
                Risk::Loss { site, result } => {
                    let s = &state.sites[site.index()];
                    v.kind = "verlust".into();
                    v.site = Some(site.0);
                    v.site_kind_text = Some(site_type_key(s.kind));
                    v.country = Some(c.countries.key(s.country).to_owned());
                    v.amount_usd = Some(usd(result));
                }
                Risk::Goal(goal) => {
                    v.kind = "ziel".into();
                    v.goal = Some(goal.key().to_owned());
                }
                Risk::Reserve { cash, reserve } => {
                    v.kind = "reserve".into();
                    v.amount_usd = Some(usd(cash));
                    v.limit_usd = Some(usd(reserve));
                }
                Risk::Debt { share, max } => {
                    v.kind = "verschuldung".into();
                    v.share = Some(share);
                    v.max = Some(max);
                }
            }
            v
        })
        .collect();
    ReviewView {
        date: iso(r.date),
        from: iso(r.from),
        to: iso(r.date.add_days(-1)),
        interval: r.interval.key().to_owned(),
        manager: state
            .managers
            .get(&r.manager)
            .map_or_else(String::new, |m| m.name.clone()),
        revenue_usd: usd(r.total.revenue),
        result_usd: usd(r.total.result),
        overhead_usd: usd(r.overhead),
        continents: r
            .continents
            .iter()
            .map(|&(k, f)| figures(c.continents.key(k).to_owned(), f))
            .collect(),
        groups: r
            .groups
            .iter()
            .map(|&(g, f)| figures(c.goods_groups.key(g).to_owned(), f))
            .collect(),
        goals: r.goals.iter().map(goal_view).collect(),
        chances,
        risks,
    }
}

/// The player's mandate and the reviews of the CEO.
pub fn reviews(game: &Game) -> ReviewsView {
    let c = game.catalog();
    let state = game.state();
    let player = game.player();
    let company = &state.companies[player.index()];
    let m = &company.mandate;
    let guidelines = [
        Guideline::Growth,
        Guideline::Profit,
        Guideline::Safety,
        Guideline::Leadership(GoodsGroupId::from_index(0)),
    ];
    let ceo = management::ceo_of(state, player);
    ReviewsView {
        enabled: c.management.enabled(),
        mandate: MandateView {
            guideline: m.guideline.key().to_owned(),
            leading_group: m.first_group().map(|g| c.goods_groups.key(g).to_owned()),
            growth: m.goals.growth,
            margin: m.goals.margin,
            equity_ratio: m.goals.equity_ratio,
            rank: m.goals.rank,
            max_debt: m.max_debt,
            blocked_countries: m
                .blocked_countries
                .iter()
                .map(|&k| c.countries.key(k).to_owned())
                .collect(),
            blocked_groups: m
                .blocked_groups
                .iter()
                .map(|&g| c.goods_groups.key(g).to_owned())
                .collect(),
            review: m.review.key().to_owned(),
        },
        guidelines: if c.management.enabled() {
            guidelines
                .iter()
                .map(|g| GuidelineView {
                    key: g.key().to_owned(),
                    aggressiveness: c.management.mandate.guidelines[g.index()],
                })
                .collect()
        } else {
            Vec::new()
        },
        intervals: ReviewInterval::ALL
            .iter()
            .map(|i| i.key().to_owned())
            .collect(),
        groups: c.goods_groups.keys().to_vec(),
        countries: c.countries.keys().to_vec(),
        ceo: ceo.map(|id| state.managers[&id].name.clone()),
        next_review: ceo.map(|_| iso(review::next_review(m.review, state.date))),
        goals_now: review::goals(state, player).iter().map(goal_view).collect(),
        reviews: company
            .reviews
            .iter()
            .rev()
            .map(|r| review_view(game, r))
            .collect(),
    }
}
