//! Display texts from `data/texte/<sprache>/` (Lastenheft §16.4).

use std::collections::BTreeMap;

use crate::messages;
use crate::read::{Ctx, Loc};
use crate::report::Path;
use crate::yaml::Kind;

/// All display texts of one language, keyed like `produkt.roheisen`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Texts {
    entries: BTreeMap<String, String>,
}

impl Texts {
    /// Texts from key–text pairs (tools and tests; the game loads them from `data/`).
    pub fn from_pairs<'a>(pairs: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        Self {
            entries: pairs
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.entries.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

#[derive(Default)]
pub(crate) struct TextIndex {
    pub texts: Texts,
    pub locations: BTreeMap<String, Loc>,
}

pub(crate) fn read_text_file(ctx: &mut Ctx, file: usize, index: &mut TextIndex) {
    let files = ctx.files;
    let Some(root) = &files[file].root else {
        return;
    };
    let file_loc = Loc {
        file,
        path: Path::default(),
    };
    let entries = match &root.kind {
        Kind::Null => return ctx.warning(&file_loc, messages::empty_file()),
        Kind::Map(entries) => entries,
        _ => return ctx.error(&file_loc, messages::text_not_scalar()),
    };
    for (key, value) in entries {
        let key = key.scalar_text().unwrap_or_default();
        let loc = file_loc.field(key);
        let text = match &value.kind {
            Kind::Null => {
                ctx.error(&loc, messages::text_empty());
                continue;
            }
            Kind::Seq(_) | Kind::Map(_) => {
                ctx.error(&loc, messages::text_not_scalar());
                continue;
            }
            _ => value.scalar_text().unwrap_or_default(),
        };
        if text.trim().is_empty() {
            ctx.error(&loc, messages::text_empty());
            continue;
        }
        if let Some(first) = index.locations.get(key) {
            let first = ctx.describe(first);
            ctx.error(&file_loc.key(key), messages::text_duplicate(key, &first));
            continue;
        }
        index.texts.entries.insert(key.to_owned(), text.to_owned());
        index.locations.insert(key.to_owned(), file_loc.key(key));
    }
}
