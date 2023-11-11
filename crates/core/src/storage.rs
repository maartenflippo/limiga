use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

pub trait Indexer {
    fn index<'slice, Value>(&self, slice: &'slice [Value]) -> &'slice Value;
    fn index_mut<'slice, Value>(&self, slice: &'slice mut [Value]) -> &'slice mut Value;

    fn get_minimum_len(&self) -> usize;
}

pub struct KeyedVec<Key, Value> {
    key: PhantomData<Key>,
    values: Vec<Value>,
}

impl<Key, Value> KeyedVec<Key, Value> {
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Value> + '_ {
        self.values.iter_mut()
    }
}

impl<Key: StaticIndexer, Value: Default> KeyedVec<Key, Value> {
    pub fn with_static_len() -> Self {
        let mut values = Vec::new();
        values.resize_with(Key::get_len(), Value::default);

        KeyedVec {
            key: PhantomData,
            values,
        }
    }
}

impl<Key: Indexer, Value: Default> KeyedVec<Key, Value> {
    pub fn grow_to(&mut self, key: Key) {
        let minimum_len = key.get_minimum_len() + 1;
        if self.values.len() < minimum_len {
            self.values.resize_with(minimum_len, Value::default);
        }
    }
}

impl<Key: Indexer, Value: Clone> KeyedVec<Key, Value> {
    pub fn grow_to_with(&mut self, key: Key, value: Value) {
        let minimum_len = key.get_minimum_len() + 1;
        if self.values.len() < minimum_len {
            self.values.resize_with(minimum_len, || value.clone());
        }
    }
}

impl<Key, Value> Default for KeyedVec<Key, Value> {
    fn default() -> Self {
        KeyedVec {
            key: PhantomData,
            values: vec![],
        }
    }
}

impl<Key: Indexer, Value> Index<Key> for KeyedVec<Key, Value> {
    type Output = Value;

    fn index(&self, index: Key) -> &Self::Output {
        index.index(&self.values)
    }
}

impl<Key: Indexer, Value> IndexMut<Key> for KeyedVec<Key, Value> {
    fn index_mut(&mut self, index: Key) -> &mut Self::Output {
        index.index_mut(&mut self.values)
    }
}

pub struct Arena<Id, Value> {
    buffer: Vec<Value>,
    id: PhantomData<Id>,
}

pub struct ArenaSlot<'a, Id, Value> {
    arena: &'a mut Arena<Id, Value>,
}

impl<Id, Value> Default for Arena<Id, Value> {
    fn default() -> Self {
        Arena {
            buffer: vec![],
            id: PhantomData,
        }
    }
}

impl<Id, Value> Arena<Id, Value> {
    pub fn new_ref(&mut self) -> ArenaSlot<'_, Id, Value> {
        ArenaSlot { arena: self }
    }
}

impl<Id, Value> ArenaSlot<'_, Id, Value>
where
    Id: From<usize>,
{
    pub fn alloc(self, value: Value) -> Id {
        self.arena.buffer.push(value);

        let id = self.arena.buffer.len() - 1;
        Id::from(id)
    }

    pub fn id(&self) -> Id {
        Id::from(self.arena.buffer.len())
    }
}

impl<Id: Indexer, Value> Index<Id> for Arena<Id, Value> {
    type Output = Value;

    fn index(&self, index: Id) -> &Self::Output {
        index.index(&self.buffer)
    }
}

impl<Id: Indexer, Value> IndexMut<Id> for Arena<Id, Value> {
    fn index_mut(&mut self, index: Id) -> &mut Self::Output {
        index.index_mut(&mut self.buffer)
    }
}

pub trait StaticIndexer {
    fn get_len() -> usize;
}

impl StaticIndexer for () {
    fn get_len() -> usize {
        0
    }
}
