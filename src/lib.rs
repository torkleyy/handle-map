extern crate fnv;

use std::borrow::Borrow;
use std::ops::{Index, IndexMut};

use fnv::FnvHashMap;

#[derive(Default)]
pub struct HandleMap<V> {
    generations: Vec<Generation>,
    keys_to_indices: FnvHashMap<String, Handle>,
    storage: Vec<V>,
}

impl<V> HandleMap<V> {
    #[inline]
    pub fn new() -> Self {
        HandleMap {
            generations: Vec::new(),
            keys_to_indices: Default::default(),
            storage: Vec::new(),
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        HandleMap {
            generations: Vec::new(),
            keys_to_indices: FnvHashMap::with_capacity_and_hasher(capacity, Default::default()),
            storage: Vec::with_capacity(capacity),
        }
    }

    pub fn handle<S: Borrow<String>>(&self, key: S) -> Option<Handle> {
        self.keys_to_indices.get(key.borrow()).map(|x| *x)
    }

    pub fn insert<S>(&mut self, key: S, value: V) -> Handle where S: Into<String> {
        let index = self.storage.len();

        let generation = self.bump_gen(index);

        let index = Handle {
            index: index,
            generation: generation
        };

        self.storage.push(value);
        self.keys_to_indices.insert(key.into(), index);

        index
    }

    pub fn pop(&mut self) -> Option<V> {
        if let Some(value) = self.storage.pop() {
            let index = self.storage.len();

            let key = self.keys_to_indices
                .iter()
                .find(|&(_, ref v)| v.index == index)
                .expect("Bug: No such key in the HashMap")
                .0
                .clone();

            self.keys_to_indices.remove(&key);

            Some(value)
        } else {
            None
        }
    }

    /// Removes an element and inserts a new one,
    /// invalidating the previous handles.
    ///
    /// If you just want to mutate an element,
    /// use `IndexMut` instead.
    pub fn replace(&mut self, index: Handle, value: V) -> V {
        use std::mem::replace;

        self.assert_alive(index);

        let index = index.index;

        let value = replace(&mut self.storage[index], value);
        self.bump_gen(index);

        value
    }

    fn assert_alive(&self, index: Handle) {
        if index.generation != self.generations[index.index] {
            panic!("Tried to use dead index (the element was removed)");
        }
    }

    fn bump_gen(&mut self, index: usize) -> Generation {
        if self.generations.len() > index {
            self.generations[index] += 1;

            self.generations[index]
        } else {
            self.generations.push(0);

            0
        }
    }
}

impl<V> Index<Handle> for HandleMap<V> {
    type Output = V;

    fn index(&self, index: Handle) -> &V {
        self.assert_alive(index);

        &self.storage[index.index]
    }
}

impl<V> IndexMut<Handle> for HandleMap<V> {
    fn index_mut(&mut self, index: Handle) -> &mut V {
        self.assert_alive(index);

        &mut self.storage[index.index]
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Handle {
    index: usize,
    generation: Generation,
}

impl Handle {
    pub fn index(&self) -> usize {
        self.index
    }
}

type Generation = u16;

#[cfg(test)]
mod tests {
    use super::{HandleMap};

    #[test]
    fn insert_and_get() {
        let mut map = HandleMap::new();

        let one_handle = map.insert("one", 1);
        let five_handle = map.insert("five", 5);

        assert_eq!(1, map[one_handle]);
        assert_eq!(5, map[five_handle]);
    }

    #[test]
    #[should_panic]
    fn generation_invalid() {
        let mut map = HandleMap::new();

        map.insert("one", 1);
        let five_handle = map.insert("five", 5);

        map.pop();

        map.insert("four", 4);

        map[five_handle];
    }
}
