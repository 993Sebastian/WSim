//! The game state: everything that changes during a game and is saved.
//!
//! Rule: every field that holds a catalog ID must be handled in [`GameState::remap`],
//! so that saves stay loadable when the data files change.

use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::ids::{CountryId, Id};
use crate::money::Money;
use crate::rng::SimRng;

/// Per-entry state for a catalog table, indexed by the table's IDs.
#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PerId<I, T> {
    items: Vec<T>,
    #[serde(skip)]
    _id: PhantomData<fn() -> I>,
}

impl<I, T: Clone> Clone for PerId<I, T> {
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
            _id: PhantomData,
        }
    }
}

impl<I, T: PartialEq> PartialEq for PerId<I, T> {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

impl<I: Id, T> PerId<I, T> {
    pub fn from_fn(len: usize, mut f: impl FnMut(I) -> T) -> Self {
        Self {
            items: (0..len).map(|i| f(I::from_index(i))).collect(),
            _id: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn get(&self, id: I) -> &T {
        &self.items[id.index()]
    }

    pub fn get_mut(&mut self, id: I) -> &mut T {
        &mut self.items[id.index()]
    }

    pub fn iter(&self) -> impl Iterator<Item = (I, &T)> {
        self.items
            .iter()
            .enumerate()
            .map(|(i, t)| (I::from_index(i), t))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (I, &mut T)> {
        self.items
            .iter_mut()
            .enumerate()
            .map(|(i, t)| (I::from_index(i), t))
    }

    /// Reorders the entries for a new catalog. `new_ids[old_index]` gives the new ID of
    /// each saved entry; entries that are new in the catalog are created with `create`.
    #[must_use]
    pub fn remap(self, new_ids: &[I], new_len: usize, mut create: impl FnMut(I) -> T) -> Self {
        let mut slots: Vec<Option<T>> = (0..new_len).map(|_| None).collect();
        for (item, &new_id) in self.items.into_iter().zip(new_ids) {
            slots[new_id.index()] = Some(item);
        }
        let items = slots
            .into_iter()
            .enumerate()
            .map(|(i, slot)| slot.unwrap_or_else(|| create(I::from_index(i))))
            .collect();
        Self {
            items,
            _id: PhantomData,
        }
    }
}

/// Companies are numbered in the order they are founded; IDs are never reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CompanyId(pub u32);

impl CompanyId {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StartForm {
    /// Small workshop that makes simple parts.
    Workshop,
    /// Small trading business.
    Trading,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompanyKind {
    Player,
    Ai,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Company {
    pub name: String,
    pub kind: CompanyKind,
    /// Country of the head office; decides taxes (Lastenheft §5.1).
    pub headquarters: CountryId,
    pub cash: Money,
    pub founded: Date,
    /// Own random stream for the company's decisions.
    pub rng: SimRng,
}

/// Current values of a country, derived from the yearly data for the current day.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CountryState {
    pub population: f64,
    pub gdp_per_capita_usd: f64,
}

impl CountryState {
    /// Yearly values apply to the middle of the year (see docs/FORMELN.md).
    pub fn at(catalog: &Catalog, country: CountryId, date: Date) -> Self {
        let values = &catalog.countries.get(country).values;
        let t = date.year_fraction() - 0.5;
        Self {
            population: values.population.value_at(t),
            gdp_per_capita_usd: values.gdp_per_capita_usd.value_at(t),
        }
    }
}

/// Settings chosen when starting a game (Lastenheft §15). Together with the journal
/// of decisions they determine the whole game.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GameSettings {
    pub seed: u64,
    pub start_year: i32,
    /// Country of the head office.
    pub start_country: CountryId,
    pub start_capital: Money,
    pub start_form: StartForm,
    pub company_name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GameState {
    pub settings: GameSettings,
    /// The current day, not yet simulated.
    pub date: Date,
    /// Random stream for world-level processes.
    pub world_rng: SimRng,
    pub countries: PerId<CountryId, CountryState>,
    /// Indexed by [`CompanyId`].
    pub companies: Vec<Company>,
    pub player: CompanyId,
    pub game_over: bool,
}

impl GameState {
    pub fn company(&self, id: CompanyId) -> Option<&Company> {
        self.companies.get(id.index())
    }

    pub fn company_mut(&mut self, id: CompanyId) -> Option<&mut Company> {
        self.companies.get_mut(id.index())
    }

    /// Translates all catalog IDs after the catalog changed (see `save`).
    pub(crate) fn remap(&mut self, countries: &[CountryId], catalog: &Catalog) {
        let map = |id: CountryId| countries[id.index()];
        let date = self.date;
        let old = std::mem::replace(&mut self.countries, PerId::from_fn(0, |_| unreachable!()));
        self.countries = old.remap(countries, catalog.countries.len(), |id| {
            CountryState::at(catalog, id, date)
        });
        self.settings.start_country = map(self.settings.start_country);
        for company in &mut self.companies {
            company.headquarters = map(company.headquarters);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remap_reorders_and_fills_new_entries() {
        let per: PerId<CountryId, &str> =
            PerId::from_fn(2, |id: CountryId| if id.index() == 0 { "a" } else { "b" });
        // Old 0 → new 1, old 1 → new 2, new 0 did not exist before.
        let new_ids = [CountryId::from_index(1), CountryId::from_index(2)];
        let remapped = per.remap(&new_ids, 3, |_| "neu");
        let values: Vec<_> = remapped.iter().map(|(_, v)| *v).collect();
        assert_eq!(values, ["neu", "a", "b"]);
    }
}
