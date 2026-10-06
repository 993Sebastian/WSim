//! Compact numeric IDs for catalog entries.
//!
//! Data files reference entries by readable keys (`roheisen`, `DEU`). When the catalog
//! is built, every key gets a dense index so the simulation can use plain vectors.
//!
//! IDs serialize as their index, except while a [`KeyTable`] is active: then they are
//! written and read as their keys. Save games use this, so they stay loadable when
//! entries are added to or reordered in the data files.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Kinds of catalog entries. The name is the German text key suffix (`art.<name>`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IdKind {
    Unit,
    Continent,
    Branch,
    GoodsGroup,
    TransportClass,
    Qualification,
    Specialization,
    LaborGroup,
    Country,
    Product,
    Facility,
    Recipe,
    Technology,
    Deposit,
    Vehicle,
    Milestone,
}

impl IdKind {
    pub const ALL: [IdKind; 16] = [
        IdKind::Unit,
        IdKind::Continent,
        IdKind::Branch,
        IdKind::GoodsGroup,
        IdKind::TransportClass,
        IdKind::Qualification,
        IdKind::Specialization,
        IdKind::LaborGroup,
        IdKind::Country,
        IdKind::Product,
        IdKind::Facility,
        IdKind::Recipe,
        IdKind::Technology,
        IdKind::Deposit,
        IdKind::Vehicle,
        IdKind::Milestone,
    ];

    pub fn name(self) -> &'static str {
        match self {
            IdKind::Unit => "einheit",
            IdKind::Continent => "kontinent",
            IdKind::Branch => "branche",
            IdKind::GoodsGroup => "warengruppe",
            IdKind::TransportClass => "transportklasse",
            IdKind::Qualification => "qualifikation",
            IdKind::Specialization => "fachrichtung",
            IdKind::LaborGroup => "arbeitskraeftegruppe",
            IdKind::Country => "land",
            IdKind::Product => "produkt",
            IdKind::Facility => "anlage",
            IdKind::Recipe => "rezept",
            IdKind::Technology => "technologie",
            IdKind::Deposit => "lagerstaette",
            IdKind::Vehicle => "verkehrsmittel",
            IdKind::Milestone => "etappe",
        }
    }

    fn slot(self) -> usize {
        self as usize
    }
}

/// Common behaviour of all catalog IDs.
pub trait Id: Copy + Ord {
    const KIND: IdKind;
    fn from_index(index: usize) -> Self;
    fn index(self) -> usize;
}

/// Keys of all catalog entries by kind, in ID order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyTable {
    keys: Vec<Vec<String>>,
    index: Vec<BTreeMap<String, usize>>,
    /// Former keys that now belong to an entry, with their rank (1 = first alias).
    aliases: Vec<BTreeMap<String, (usize, u32)>>,
}

impl KeyTable {
    pub fn new(keys: Vec<Vec<String>>) -> Self {
        let index = keys
            .iter()
            .map(|list| {
                list.iter()
                    .enumerate()
                    .map(|(i, k)| (k.clone(), i))
                    .collect()
            })
            .collect();
        Self {
            keys,
            index,
            aliases: Vec::new(),
        }
    }

    /// Lets `alias` (a key of earlier data, e.g. a country merged into a region) read
    /// as the entry `index`. Own keys win over aliases; of several aliases for the same
    /// entry, the one added first wins (see `PerId`).
    pub fn add_alias(&mut self, kind: IdKind, alias: &str, index: usize) {
        if self.lookup_ranked(kind, alias).is_some() {
            return;
        }
        if self.aliases.len() <= kind.slot() {
            self.aliases.resize_with(kind.slot() + 1, BTreeMap::new);
        }
        let ranks = &mut self.aliases[kind.slot()];
        let rank = 1 + ranks.values().filter(|(i, _)| *i == index).count();
        ranks.insert(
            alias.to_owned(),
            (index, u32::try_from(rank).unwrap_or(u32::MAX)),
        );
    }

    pub fn keys(&self, kind: IdKind) -> &[String] {
        self.keys.get(kind.slot()).map_or(&[], Vec::as_slice)
    }

    /// Index of `key` and its rank: 0 for an own key, from 1 for aliases.
    fn lookup_ranked(&self, kind: IdKind, key: &str) -> Option<(usize, u32)> {
        if let Some(&index) = self.index.get(kind.slot()).and_then(|m| m.get(key)) {
            return Some((index, 0));
        }
        self.aliases.get(kind.slot())?.get(key).copied()
    }
}

thread_local! {
    static ACTIVE: RefCell<Option<KeyTable>> = const { RefCell::new(None) };
    static MISSING: RefCell<Option<(IdKind, String)>> = const { RefCell::new(None) };
    static LAST_RANK: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// Rank of the last ID read as key (0: own key, from 1: alias). `PerId` uses it to
/// prefer an entry's own state over that of the entries merged into it.
pub(crate) fn take_last_rank() -> u32 {
    LAST_RANK.with(|r| r.replace(0))
}

/// Runs `f` with IDs (de)serialized as keys. Returns the first key that was not found
/// in `table` while reading, if any.
pub fn with_keys<R>(table: &KeyTable, f: impl FnOnce() -> R) -> (R, Option<(IdKind, String)>) {
    ACTIVE.with(|a| *a.borrow_mut() = Some(table.clone()));
    MISSING.with(|m| *m.borrow_mut() = None);
    let result = f();
    ACTIVE.with(|a| *a.borrow_mut() = None);
    let missing = MISSING.with(|m| m.borrow_mut().take());
    (result, missing)
}

pub(crate) fn keys_active() -> bool {
    ACTIVE.with(|a| a.borrow().is_some())
}

pub(crate) fn active_len(kind: IdKind) -> Option<usize> {
    ACTIVE.with(|a| a.borrow().as_ref().map(|t| t.keys(kind).len()))
}

fn key_of(kind: IdKind, index: usize) -> Option<String> {
    ACTIVE.with(|a| {
        a.borrow()
            .as_ref()
            .and_then(|t| t.keys(kind).get(index).cloned())
    })
}

/// Serializes an ID as key (with active table) or index.
pub(crate) fn serialize_id<S: Serializer>(
    kind: IdKind,
    index: usize,
    s: S,
) -> Result<S::Ok, S::Error> {
    match key_of(kind, index) {
        Some(key) => s.serialize_str(&key),
        None => s.serialize_u64(index as u64),
    }
}

struct IdVisitor<I>(PhantomData<I>);

impl<I: Id> Visitor<'_> for IdVisitor<I> {
    type Value = I;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "an ID of kind {}", I::KIND.name())
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<I, E> {
        // With a key table, an index must also exist in the catalog.
        let max = active_len(I::KIND).map_or(usize::from(u16::MAX) + 1, |n| n);
        usize::try_from(v)
            .ok()
            .filter(|&i| i < max)
            .map(I::from_index)
            .ok_or_else(|| E::custom("ID out of range"))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<I, E> {
        u64::try_from(v)
            .map_err(|_| E::custom("negative ID"))
            .and_then(|v| self.visit_u64(v))
    }

    fn visit_str<E: de::Error>(self, key: &str) -> Result<I, E> {
        let found = ACTIVE.with(|a| a.borrow().as_ref().map(|t| t.lookup_ranked(I::KIND, key)));
        match found {
            Some(Some((index, rank))) => {
                LAST_RANK.with(|r| r.set(rank));
                Ok(I::from_index(index))
            }
            Some(None) => {
                MISSING.with(|m| {
                    m.borrow_mut()
                        .get_or_insert_with(|| (I::KIND, key.to_owned()));
                });
                Err(E::custom(format!("unknown {} {key}", I::KIND.name())))
            }
            None => Err(E::custom("ID given as key without key table")),
        }
    }
}

pub(crate) fn deserialize_id<'de, I: Id, D: Deserializer<'de>>(d: D) -> Result<I, D::Error> {
    d.deserialize_any(IdVisitor::<I>(PhantomData))
}

macro_rules! define_id {
    ($($(#[$meta:meta])* $name:ident => $kind:ident),* $(,)?) => {$(
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u16);

        impl Id for $name {
            const KIND: IdKind = IdKind::$kind;

            fn from_index(index: usize) -> Self {
                Self(u16::try_from(index).expect(concat!("too many entries for ", stringify!($name))))
            }

            fn index(self) -> usize {
                usize::from(self.0)
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                serialize_id(IdKind::$kind, self.index(), s)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                deserialize_id(d)
            }
        }
    )*};
}

define_id!(
    UnitId => Unit,
    ContinentId => Continent,
    BranchId => Branch,
    GoodsGroupId => GoodsGroup,
    TransportClassId => TransportClass,
    QualificationId => Qualification,
    SpecializationId => Specialization,
    /// A qualification, optionally combined with a specialization (`fachkraft.metall`).
    LaborGroupId => LaborGroup,
    CountryId => Country,
    ProductId => Product,
    FacilityId => Facility,
    RecipeId => Recipe,
    TechnologyId => Technology,
    DepositId => Deposit,
    VehicleId => Vehicle,
    /// A goal for the player after the introduction (M23).
    MilestoneId => Milestone,
);

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> KeyTable {
        let mut keys = vec![Vec::new(); IdKind::ALL.len()];
        keys[IdKind::Country.slot()] = vec!["AAA".into(), "BBB".into()];
        KeyTable::new(keys)
    }

    #[test]
    fn ids_are_indices_without_table() {
        let bytes = rmp_serde::to_vec(&CountryId::from_index(1)).unwrap();
        assert_eq!(
            rmp_serde::from_slice::<CountryId>(&bytes).unwrap(),
            CountryId::from_index(1)
        );
    }

    #[test]
    fn ids_are_keys_with_table() {
        let (bytes, _) = with_keys(&table(), || {
            rmp_serde::to_vec(&CountryId::from_index(1)).unwrap()
        });
        let text: String = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(text, "BBB");

        // Reading into a table where BBB comes first.
        let mut keys = vec![Vec::new(); IdKind::ALL.len()];
        keys[IdKind::Country.slot()] = vec!["BBB".into(), "AAA".into()];
        let (id, missing) = with_keys(&KeyTable::new(keys), || {
            rmp_serde::from_slice::<CountryId>(&bytes).unwrap()
        });
        assert_eq!(id, CountryId::from_index(0));
        assert!(missing.is_none());
    }

    #[test]
    fn missing_keys_are_reported() {
        let bytes = rmp_serde::to_vec("CCC").unwrap();
        let (result, missing) = with_keys(&table(), || rmp_serde::from_slice::<CountryId>(&bytes));
        assert!(result.is_err());
        assert_eq!(missing, Some((IdKind::Country, "CCC".into())));
    }
}
