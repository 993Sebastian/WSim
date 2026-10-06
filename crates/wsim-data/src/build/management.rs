//! Positions, managers and their market (MA1, docs/FORMELN.md, docs/MANAGER.md).

use wsim_core::catalog::{
    Catalog, ManagementFunction, ManagementLevel, ManagementModel, ManagerPoolModel, SiteType,
    SkillModel,
};
use wsim_core::decision::Topic;

use super::production::SITE_TYPES;
use super::{in_range, non_negative, positive, provenance};
use crate::messages;
use crate::raw::RawSpread;
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
                levels.push(ManagementLevel {
                    key: (*key).to_owned(),
                    check_days: level.check_days.max(1),
                    salary_specialist: positive(
                        ctx,
                        level.salary_specialist,
                        &ll.field("gehalt_fach"),
                    ),
                    salary_head: positive(ctx, level.salary_head, &ll.field("gehalt_leitung")),
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
