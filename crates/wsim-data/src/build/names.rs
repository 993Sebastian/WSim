//! Name parts for the product names of companies (M42, docs/FORMELN.md).

use wsim_core::catalog::{Catalog, NamePattern, NamingStyle, ProductNaming};
use wsim_core::ids::{GoodsGroupId, Id};
use wsim_core::product_names::same_name;

use super::{HISTORY_YEARS, Keys, in_range, resolve, year};
use crate::messages;
use crate::raw::RawNamingStyle;
use crate::read::{Ctx, Loc, RawData};

/// Placeholders of product name patterns and the list each one draws from.
const PLACEHOLDERS: &[(&str, &str)] = &[
    ("stamm", "staemme"),
    ("zahl", "zahlen"),
    ("buchstabe", "buchstaben"),
    ("zusatz", "zusaetze"),
];

pub(super) fn product_naming(
    ctx: &mut Ctx,
    catalog: &Catalog,
    raw: &RawData,
    groups: &Keys,
) -> ProductNaming {
    // Optional: without the section no product gets a name.
    let Some((entry, rest)) = raw.product_naming.split_first() else {
        return ProductNaming::default();
    };
    let first = ctx.describe(&entry.loc);
    for other in rest {
        ctx.error(
            &other.loc,
            messages::section_duplicate("produktnamen", &first),
        );
    }
    let v = &entry.value;
    let l = &entry.loc;
    let house_brand = in_range(ctx, v.house_brand, 0.0, 1.0, &l.field("hausmarke"));
    words(ctx, &v.excluded, &l.field("ausgeschlossen"), &[]);
    let mut style_of_group = vec![None; catalog.goods_groups.len()];
    let mut styles: Vec<NamingStyle> = Vec::new();
    for (i, s) in v.styles.iter().enumerate() {
        let sl = l.field("stile").index(i);
        if styles.iter().any(|x| x.key == s.id) {
            ctx.error(&sl.field("id"), messages::naming_word_twice(&s.id));
        }
        if s.goods_groups.is_empty() {
            ctx.error(&sl.field("warengruppen"), messages::list_empty());
        }
        for (j, key) in s.goods_groups.iter().enumerate() {
            let gl = sl.field("warengruppen").index(j);
            let group: GoodsGroupId = resolve(ctx, groups, key, &gl);
            if !groups.index.contains_key(key) {
                continue;
            }
            match style_of_group[group.index()] {
                Some(k) => {
                    let first: &NamingStyle = &styles[k];
                    ctx.error(&gl, messages::naming_group_twice(key, &first.key));
                }
                None => style_of_group[group.index()] = Some(styles.len()),
            }
        }
        styles.push(style(ctx, s, &sl, &v.excluded));
    }
    ProductNaming {
        house_brand,
        excluded: v.excluded.clone(),
        styles,
        style_of_group,
    }
}

fn style(ctx: &mut Ctx, s: &RawNamingStyle, l: &Loc, excluded: &[String]) -> NamingStyle {
    if s.stems.is_empty() {
        ctx.error(&l.field("staemme"), messages::list_empty());
    }
    words(ctx, &s.stems, &l.field("staemme"), excluded);
    words(ctx, &s.letters, &l.field("buchstaben"), excluded);
    words(ctx, &s.additions, &l.field("zusaetze"), excluded);
    for (i, n) in s.numbers.iter().enumerate() {
        let nl = l.field("zahlen").index(i);
        if *n == 0 {
            ctx.error(&nl, messages::naming_number_zero());
        } else if s.numbers[..i].contains(n) {
            ctx.error(&nl, messages::naming_word_twice(&n.to_string()));
        }
    }
    if s.patterns.is_empty() {
        ctx.error(&l.field("muster"), messages::list_empty());
    }
    let mut patterns = Vec::new();
    for (i, p) in s.patterns.iter().enumerate() {
        let pl = l.field("muster").index(i);
        let tl = pl.field("text");
        let mut has_stem = false;
        for name in placeholders(&p.text) {
            let Some(&(_, field)) = PLACEHOLDERS.iter().find(|(n, _)| *n == name) else {
                ctx.error(&tl, messages::naming_placeholder_unknown(name));
                continue;
            };
            let empty = match name {
                "stamm" => {
                    has_stem = true;
                    false
                }
                "zahl" => s.numbers.is_empty(),
                "buchstabe" => s.letters.is_empty(),
                _ => s.additions.is_empty(),
            };
            if empty {
                ctx.error(&tl, messages::naming_list_needed(name, field));
            }
        }
        if !has_stem {
            ctx.error(&tl, messages::naming_stem_missing());
        }
        if let Some(a) = p.ab {
            year(ctx, a, HISTORY_YEARS, &pl.field("ab"));
        }
        if let Some(b) = p.bis {
            year(ctx, b, HISTORY_YEARS, &pl.field("bis"));
        }
        if let (Some(a), Some(b)) = (p.ab, p.bis)
            && a > b
        {
            ctx.error(&pl, messages::naming_years(a, b));
        }
        patterns.push(NamePattern {
            text: p.text.clone(),
            from: p.ab,
            until: p.bis,
        });
    }
    if !s.patterns.is_empty() && !s.patterns.iter().any(|p| p.ab.is_none() && p.bis.is_none()) {
        ctx.error(&l.field("muster"), messages::naming_no_open_pattern());
    }
    NamingStyle {
        key: s.id.clone(),
        stems: s.stems.clone(),
        patterns,
        numbers: s.numbers.clone(),
        letters: s.letters.clone(),
        additions: s.additions.clone(),
    }
}

/// Placeholders of a pattern such as `{stamm} {zahl}`.
fn placeholders(pattern: &str) -> impl Iterator<Item = &str> {
    pattern
        .split('{')
        .skip(1)
        .filter_map(|part| part.split_once('}').map(|(name, _)| name))
}

/// Checks a list of name parts: no empty or repeated entries, no word from `excluded`.
fn words(ctx: &mut Ctx, list: &[String], loc: &Loc, excluded: &[String]) {
    for (i, w) in list.iter().enumerate() {
        let il = loc.index(i);
        let text = w.trim();
        if text.is_empty() {
            ctx.error(&il, messages::naming_word_empty());
            continue;
        }
        if list[..i].iter().any(|o| same_name(o.trim(), text)) {
            ctx.error(&il, messages::naming_word_twice(text));
        }
        if let Some(word) = text
            .split_whitespace()
            .find(|word| excluded.iter().any(|e| same_name(e, word)))
        {
            ctx.error(&il, messages::naming_excluded(word));
        }
    }
}
