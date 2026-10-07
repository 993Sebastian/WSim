//! Positions, managers and their market (MA1, docs/FORMELN.md, docs/MANAGER.md).

use wsim_core::catalog::{
    AiHiringModel, Catalog, ConcernModel, ManagementFunction, ManagementLevel, ManagementModel,
    ManagerMarketModel, ManagerPoolModel, MandateModel, PoachingModel, SatisfactionModel, SiteType,
    SkillModel, StrategyModel,
};
use wsim_core::decision::Topic;
use wsim_core::money::Money;

use super::production::SITE_TYPES;
use super::{in_range, non_negative, positive, provenance};
use crate::messages;
use crate::raw::{RawManagerMarket, RawPriceStrategy, RawSpread};
use crate::read::{Ctx, Loc, RawData};
use crate::texts::TextIndex;

/// The levels of the hierarchy, from the site up.
const LEVELS: &[&str] = &["standort", "land", "kontinent", "vorstand"];

pub(super) fn management(ctx: &mut Ctx, catalog: &Catalog, raw: &RawData) -> ManagementModel {
    // Optional: without the section there are no managers.
    let Some((entry, rest)) = raw.management.split_first() else {
        return ManagementModel::default();
    };
    let first = ctx.describe(&entry.loc);
    for other in rest {
        ctx.error(
            &other.loc,
            messages::section_duplicate("management", &first),
        );
    }
    let v = &entry.value;
    let l = &entry.loc;

    let mut functions: Vec<ManagementFunction> = Vec::new();
    if v.functions.is_empty() {
        ctx.error(&l.field("bereiche"), messages::list_empty());
    }
    let topic_keys: Vec<&str> = Topic::ALL.iter().map(|t| t.key()).collect();
    for (i, f) in v.functions.iter().enumerate() {
        let fl = l.field("bereiche").index(i);
        if let Some(other) = functions.iter().find(|x| x.key == f.id) {
            ctx.error(
                &fl.field("id"),
                messages::duplicate_key("Bereich", &f.id, &other.key),
            );
        }
        let mut topics = Vec::new();
        for (j, key) in f.topics.iter().enumerate() {
            let tl = fl.field("themen").index(j);
            let Some(topic) = Topic::from_key(key) else {
                let suggested = crate::suggest::closest(key, topic_keys.iter().copied());
                ctx.error(&tl, messages::unknown_reference("Thema", key, suggested));
                continue;
            };
            let owner = functions
                .iter()
                .find(|x| x.topics.contains(&topic))
                .map(|x| x.key.clone())
                .or_else(|| topics.contains(&topic).then(|| f.id.clone()));
            if let Some(owner) = owner {
                ctx.error(&tl, messages::management_topic_twice(key, &owner));
                continue;
            }
            topics.push(topic);
        }
        functions.push(ManagementFunction {
            key: f.id.clone(),
            topics,
        });
    }

    let mut levels: Vec<ManagementLevel> = Vec::new();
    // Functions of a level's specialists and the topics it takes up (MA3).
    let level_lists = |ctx: &mut Ctx, level: &crate::raw::RawManagementLevel, ll: &Loc| {
        let mut indices: Vec<usize> = Vec::new();
        for (j, f) in level.specialists.iter().enumerate() {
            let fl = ll.field("fachstellen").index(j);
            match functions.iter().position(|x| x.key == *f) {
                Some(index) if indices.contains(&index) => {
                    ctx.error(&fl, messages::duplicate_key("Bereich", f, &level.id));
                }
                Some(index) => indices.push(index),
                None => {
                    let suggested =
                        crate::suggest::closest(f, functions.iter().map(|x| x.key.as_str()));
                    ctx.error(&fl, messages::unknown_reference("Bereich", f, suggested));
                }
            }
        }
        if level.id == "standort" && !level.specialists.is_empty() {
            ctx.error(
                &ll.field("fachstellen"),
                messages::management_site_specialists(),
            );
        }
        let mut topics: Vec<Topic> = Vec::new();
        for (j, key) in level.topics.iter().enumerate() {
            let tl = ll.field("themen").index(j);
            match Topic::from_key(key) {
                Some(topic) if topics.contains(&topic) => {
                    ctx.error(&tl, messages::duplicate_key("Thema", key, &level.id));
                }
                Some(topic) if !functions.iter().any(|f| f.topics.contains(&topic)) => {
                    ctx.error(&tl, messages::management_topic_without_function(key));
                }
                Some(topic) => topics.push(topic),
                None => {
                    let suggested = crate::suggest::closest(key, topic_keys.iter().copied());
                    ctx.error(&tl, messages::unknown_reference("Thema", key, suggested));
                }
            }
        }
        (indices, topics)
    };
    for key in LEVELS {
        let found: Vec<_> = v
            .levels
            .iter()
            .enumerate()
            .filter(|(_, x)| x.id == *key)
            .collect();
        match found.as_slice() {
            [] => ctx.error(&l.field("ebenen"), messages::entry_missing("Ebene", key)),
            [(i, level), more @ ..] => {
                let ll = l.field("ebenen").index(*i);
                for (k, _) in more {
                    ctx.error(
                        &l.field("ebenen").index(*k).field("id"),
                        messages::duplicate_key("Ebene", key, &ctx.describe(&ll)),
                    );
                }
                if level.check_days == 0 {
                    ctx.error(
                        &ll.field("pruefung_tage"),
                        messages::not_positive(f64::from(level.check_days)),
                    );
                }
                let (specialists, topics) = level_lists(ctx, level, &ll);
                levels.push(ManagementLevel {
                    key: (*key).to_owned(),
                    check_days: level.check_days.max(1),
                    salary_specialist: positive(
                        ctx,
                        level.salary_specialist,
                        &ll.field("gehalt_fach"),
                    ),
                    salary_head: positive(ctx, level.salary_head, &ll.field("gehalt_leitung")),
                    budget_specialist: budget(
                        ctx,
                        level.budget_specialist,
                        &ll.field("budget_fach"),
                    ),
                    budget_head: budget(ctx, level.budget_head, &ll.field("budget_leitung")),
                    specialists,
                    topics,
                });
            }
        }
    }
    for (i, level) in v.levels.iter().enumerate() {
        if !LEVELS.contains(&level.id.as_str()) {
            let suggested = crate::suggest::closest(&level.id, LEVELS.iter().copied());
            ctx.error(
                &l.field("ebenen").index(i).field("id"),
                messages::unknown_value(&level.id, LEVELS, suggested),
            );
        }
    }

    let site_keys: Vec<&str> = SITE_TYPES.iter().map(|(k, _)| *k).collect();
    let mut specialists: Vec<(SiteType, Vec<usize>)> = Vec::new();
    for (key, list) in &v.site_types {
        let sl = l.field("standorttypen").field(key);
        let Some(&(_, kind)) = SITE_TYPES.iter().find(|(k, _)| k == key) else {
            let suggested = crate::suggest::closest(key, site_keys.iter().copied());
            ctx.error(&sl, messages::unknown_value(key, &site_keys, suggested));
            continue;
        };
        let mut indices = Vec::new();
        for (j, f) in list.iter().enumerate() {
            let fl = sl.index(j);
            match functions.iter().position(|x| x.key == *f) {
                Some(index) if indices.contains(&index) => {
                    ctx.error(&fl, messages::duplicate_key("Bereich", f, key));
                }
                Some(index) => indices.push(index),
                None => {
                    let suggested =
                        crate::suggest::closest(f, functions.iter().map(|x| x.key.as_str()));
                    ctx.error(&fl, messages::unknown_reference("Bereich", f, suggested));
                }
            }
        }
        specialists.push((kind, indices));
    }

    // Topics of the routine and topics whose effect counts only costs (MA5): in both the
    // positions take the rules' option; a topic belongs to one list at most.
    let mut routine_topics = Vec::new();
    let mut rule_topics = Vec::new();
    for (list, keys) in [
        ("routine_themen", &v.routine_topics),
        ("regel_themen", &v.rule_topics),
    ] {
        for (i, key) in keys.iter().enumerate() {
            let tl = l.field(list).index(i);
            match Topic::from_key(key) {
                Some(topic) if routine_topics.contains(&topic) => {
                    ctx.error(&tl, messages::duplicate_key("Thema", key, "routine_themen"));
                }
                Some(topic) if rule_topics.contains(&topic) => {
                    ctx.error(&tl, messages::duplicate_key("Thema", key, "regel_themen"));
                }
                Some(topic) if list == "routine_themen" => routine_topics.push(topic),
                Some(topic) => rule_topics.push(topic),
                None => {
                    let suggested = crate::suggest::closest(key, topic_keys.iter().copied());
                    ctx.error(&tl, messages::unknown_reference("Thema", key, suggested));
                }
            }
        }
    }
    let fl = l.field("budget_sockel_gehaelter");
    let budget_floor = (
        non_negative(ctx, v.budget_floor.decision, &fl.field("entscheidung")),
        non_negative(ctx, v.budget_floor.year, &fl.field("jahr")),
    );
    let cl = l.field("anliegen");
    let c = &v.concerns;
    for (value, field) in [
        (c.deadline_days, "frist_tage"),
        (c.open_per_position, "offen_je_stelle"),
        (c.followup_days, "wirkzeit_tage"),
        (c.bundle_from, "buendel_ab"),
    ] {
        if value == 0 {
            ctx.error(&cl.field(field), messages::not_positive(0.0));
        }
    }
    let concerns = ConcernModel {
        deadline_days: c.deadline_days.max(1),
        block_days: c.block_days,
        open_per_position: c.open_per_position.max(1),
        followup_days: c.followup_days.max(1),
        estimate_error: in_range(ctx, c.estimate_error, 0.0, 1.0, &cl.field("schaetzfehler")),
        recommend_base: in_range(
            ctx,
            c.recommend_base,
            0.0,
            1.0,
            &cl.field("empfehlung_grund"),
        ),
        bundle_from: c.bundle_from.max(1),
    };

    let strategy = strategy(ctx, catalog, &v.strategy, &l.field("strategie"));
    let ml = l.field("strategieauftrag");
    let gl = ml.field("leitlinien");
    let g = &v.mandate.guidelines;
    let mandate = MandateModel {
        guidelines: [
            in_range(ctx, g.growth, 0.0, 1.0, &gl.field("wachstum")),
            in_range(ctx, g.profit, 0.0, 1.0, &gl.field("ertrag")),
            in_range(ctx, g.safety, 0.0, 1.0, &gl.field("sicherheit")),
            in_range(ctx, g.leadership, 0.0, 1.0, &gl.field("marktfuehrung")),
        ],
        proposals_max: v.mandate.proposals_max.max(1),
        chances_risks: v.mandate.chances_risks.max(1),
        reviews_kept: v.mandate.reviews_kept.max(1),
        personnel_sharpness: in_range(
            ctx,
            v.mandate.personnel_sharpness,
            0.0,
            1.0,
            &ml.field("personal_schaerfe"),
        ),
    };
    for (value, field) in [
        (v.mandate.proposals_max, "antraege_max"),
        (v.mandate.chances_risks, "chancen_risiken"),
        (v.mandate.reviews_kept, "ruecksprachen_behalten"),
    ] {
        if value == 0 {
            ctx.error(&ml.field(field), messages::not_positive(0.0));
        }
    }

    let market = market(ctx, &v.market, &l.field("markt"));

    let salary_group = catalog.labor_groups.id(&v.salary_group);
    if salary_group.is_none() {
        ctx.error(
            &l.field("gehalt_lohngruppe"),
            messages::unknown_reference("Arbeitskräftegruppe", &v.salary_group, None),
        );
    }
    let pl = l.field("pool");
    let pool = ManagerPoolModel {
        per_million_academics: positive(
            ctx,
            v.pool.per_million_academics,
            &pl.field("je_mio_akademiker"),
        ),
        min: v.pool.min,
        max: v.pool.max,
        leave_per_month: in_range(
            ctx,
            v.pool.leave_per_month,
            0.0,
            1.0,
            &pl.field("abgang_monat"),
        ),
    };
    if pool.min == 0 {
        ctx.error(&pl.field("min"), messages::not_positive(0.0));
    }
    if pool.max < pool.min {
        ctx.error(
            &pl.field("max"),
            messages::management_pool_bounds(pool.min, pool.max),
        );
    }
    let kl = l.field("faehigkeiten");
    let spread = |ctx: &mut Ctx, s: &RawSpread, loc: &Loc| {
        (
            in_range(ctx, s.mean, 0.0, 100.0, &loc.field("mittel")),
            in_range(ctx, s.spread, 0.0, 50.0, &loc.field("streuung")),
        )
    };
    let skills = SkillModel {
        focus: spread(ctx, &v.skills.focus, &kl.field("schwerpunkt")),
        other: spread(ctx, &v.skills.other, &kl.field("sonst")),
        general: spread(ctx, &v.skills.general, &kl.field("allgemein")),
        impression_blur: in_range(
            ctx,
            v.skills.impression_blur,
            0.0,
            50.0,
            &kl.field("eindruck_unschaerfe"),
        ),
    };
    ManagementModel {
        functions,
        levels,
        specialists,
        routine_topics,
        rule_topics,
        budget_floor,
        concerns,
        strategy,
        mandate,
        market,
        head_discount: in_range(
            ctx,
            v.head_discount,
            0.0,
            1.0,
            &l.field("leitung_ohne_fach_abschlag"),
        ),
        notice_base: in_range(ctx, v.notice_base, 0.0, 1.0, &l.field("bemerken_grund")),
        salary_group,
        severance_months: non_negative(ctx, v.severance_months, &l.field("abfindung_monate")),
        pool,
        skills,
        provenance: provenance(v.approximation, v.source.as_ref()),
    }
}

/// The living market of managers (MA6): experience, satisfaction, resignation,
/// poaching and the hiring of the AI companies.
fn market(ctx: &mut Ctx, v: &RawManagerMarket, l: &Loc) -> ManagerMarketModel {
    let el = l.field("erfahrung");
    let sl = l.field("zufriedenheit");
    let s = &v.satisfaction;
    let percent = |ctx: &mut Ctx, value: u8, loc: &Loc| {
        if value > 100 {
            ctx.error(loc, messages::out_of_range(f64::from(value), 0.0, 100.0));
        }
        value.min(100)
    };
    let bands = [
        percent(ctx, s.bands[0], &sl.field("stufen").index(0)),
        percent(ctx, s.bands[1], &sl.field("stufen").index(1)),
    ];
    if bands[0] > bands[1] {
        ctx.error(
            &sl.field("stufen"),
            messages::management_satisfaction_bands(bands[0], bands[1]),
        );
    }
    let satisfaction = SatisfactionModel {
        start: percent(ctx, s.start, &sl.field("start")),
        base: in_range(ctx, s.base, 0.0, 100.0, &sl.field("basis")),
        salary_weight: non_negative(ctx, s.salary_weight, &sl.field("gehalt_gewicht")),
        loss_penalty: non_negative(ctx, s.loss_penalty, &sl.field("verlust_abzug")),
        overruled_penalty: non_negative(ctx, s.overruled_penalty, &sl.field("uebergangen_abzug")),
        adjust: in_range(ctx, s.adjust, 0.0, 1.0, &sl.field("anpassung")),
        bands,
    };
    let kl = l.field("kuendigung");
    let pl = l.field("abwerbung");
    let p = &v.poaching;
    let poaching = PoachingModel {
        min_strength: in_range(ctx, p.min_strength, 0.0, 100.0, &pl.field("staerke_min")),
        lead: in_range(ctx, p.lead, 0.0, 100.0, &pl.field("vorsprung")),
        markup: non_negative(ctx, p.markup, &pl.field("aufschlag")),
        ignored_penalty: percent(ctx, p.ignored_penalty, &pl.field("ignoriert_abzug")),
        ai_counter_max: in_range(ctx, p.ki_gegen_max, 1.0, 10.0, &pl.field("ki_gegen_max")),
        pause_months: p.pause_months,
    };
    let al = l.field("ki");
    let a = &v.ai;
    let usd = |ctx: &mut Ctx, value: f64, loc: &Loc| {
        let value = non_negative(ctx, value, loc);
        Money::from_usd(value).unwrap_or(Money::ZERO)
    };
    let ai = AiHiringModel {
        per_month: a.per_month,
        ceo_revenue: usd(ctx, a.ceo_revenue_usd, &al.field("umsatz_ceo_usd")),
        site_revenue: usd(ctx, a.site_revenue_usd, &al.field("umsatz_standort_usd")),
        salary_share: in_range(ctx, a.salary_share, 0.0, 1.0, &al.field("gehalt_anteil")),
        competence_ceo: in_range(ctx, a.competence_ceo, 0.0, 1.0, &al.field("kompetenz_ceo")),
        competence_heads: in_range(
            ctx,
            a.competence_heads,
            0.0,
            1.0,
            &al.field("kompetenz_leitung"),
        ),
    };
    ManagerMarketModel {
        experience_chance: in_range(
            ctx,
            v.experience.chance_per_month,
            0.0,
            1.0,
            &el.field("chance_monat"),
        ),
        experience_room: percent(ctx, v.experience.room, &el.field("spielraum")),
        satisfaction,
        resignation_threshold: percent(ctx, v.resignation.threshold, &kl.field("schwelle")),
        resignation_chance: in_range(
            ctx,
            v.resignation.chance_max,
            0.0,
            1.0,
            &kl.field("chance_max"),
        ),
        poaching,
        ai,
    }
}

/// Strategies of the managers (MA4): price floors and start markups, the largest
/// settings. The defaults of the stock strategy are the AI's days, so they must be
/// settable.
fn strategy(
    ctx: &mut Ctx,
    catalog: &Catalog,
    raw: &crate::raw::RawStrategy,
    loc: &Loc,
) -> StrategyModel {
    // Markups as a sale offer accepts them (`Command::SetSale`).
    let price = |ctx: &mut Ctx, p: &RawPriceStrategy, loc: &Loc| {
        (
            positive(ctx, p.floor, &loc.field("untergrenze")),
            in_range(ctx, p.markup, -0.9, 2.0, &loc.field("aufschlag")),
        )
    };
    let model = StrategyModel {
        premium: price(ctx, &raw.premium, &loc.field("premium")),
        fight: price(ctx, &raw.fight, &loc.field("kampfpreis")),
        min_margin_max: positive(ctx, raw.min_margin_max, &loc.field("mindestmarge_max")),
        stock_days_max: positive(ctx, raw.stock_days_max, &loc.field("lager_tage_max")),
        reserve_months_max: positive(
            ctx,
            raw.reserve_months_max,
            &loc.field("liquiditaet_monate_max"),
        ),
    };
    let ai = &catalog.ai_model;
    let needed = ai
        .behavior
        .stock_low_days
        .max(ai.start.input_stock_days)
        .max(ai.behavior.stock_target_days);
    if model.stock_days_max < needed {
        ctx.error(
            &loc.field("lager_tage_max"),
            messages::management_stock_days_max(model.stock_days_max, needed),
        );
    }
    model
}

/// Shares of the revenue per decision and per year: each 0–1, the first at most the
/// second.
fn budget(ctx: &mut Ctx, [decision, year]: [f64; 2], loc: &Loc) -> (f64, f64) {
    let decision = in_range(ctx, decision, 0.0, 1.0, &loc.index(0));
    let year = in_range(ctx, year, 0.0, 1.0, &loc.index(1));
    if decision > year {
        ctx.error(loc, messages::management_budget_order(decision, year));
    }
    (decision, year)
}

/// Prefix of the texts of the functions.
const FUNCTION_TEXT: &str = "bereich";

/// Every function needs its text `bereich.<id>`; texts of unknown functions are unused.
pub(super) fn check_texts(
    ctx: &mut Ctx,
    model: &ManagementModel,
    raw: &RawData,
    texts: &TextIndex,
    report_unused: bool,
) {
    let Some(entry) = raw.management.first() else {
        return;
    };
    let functions = entry.loc.field("bereiche");
    for (i, f) in model.functions.iter().enumerate() {
        let key = format!("{FUNCTION_TEXT}.{}", f.key);
        if texts.texts.get(&key).is_none() {
            ctx.error(
                &functions.index(i).field("id"),
                messages::text_missing(&key, crate::LANGUAGE),
            );
        }
    }
    if !report_unused {
        return;
    }
    for (text_key, loc) in &texts.locations {
        if let Some((FUNCTION_TEXT, rest)) = text_key.split_once('.')
            && !model.functions.iter().any(|f| f.key == rest)
        {
            ctx.warning(loc, messages::text_unused(text_key));
        }
    }
}
