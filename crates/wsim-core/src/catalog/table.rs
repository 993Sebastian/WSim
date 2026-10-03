use std::collections::BTreeMap;
use std::marker::PhantomData;

use crate::ids::Id;

/// Catalog entries of one kind, addressable by dense ID and by data key.
#[derive(Clone, Debug)]
pub struct Table<I, T> {
    items: Vec<T>,
    keys: Vec<String>,
    by_key: BTreeMap<String, usize>,
    _id: PhantomData<I>,
}

impl<I: Id, T> Default for Table<I, T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            keys: Vec::new(),
            by_key: BTreeMap::new(),
            _id: PhantomData,
        }
    }
}

impl<I: Id, T> Table<I, T> {
    /// Adds an entry. Returns `None` if the key already exists.
    pub fn insert(&mut self, key: &str, item: T) -> Option<I> {
        if self.by_key.contains_key(key) {
            return None;
        }
        let index = self.items.len();
        self.items.push(item);
        self.keys.push(key.to_owned());
        self.by_key.insert(key.to_owned(), index);
        Some(I::from_index(index))
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

    pub fn id(&self, key: &str) -> Option<I> {
        self.by_key.get(key).map(|&i| I::from_index(i))
    }

    pub fn key(&self, id: I) -> &str {
        &self.keys[id.index()]
    }

    pub fn ids(&self) -> impl Iterator<Item = I> + '_ {
        (0..self.items.len()).map(I::from_index)
    }

    pub fn iter(&self) -> impl Iterator<Item = (I, &T)> + '_ {
        self.items
            .iter()
            .enumerate()
            .map(|(i, item)| (I::from_index(i), item))
    }
}
