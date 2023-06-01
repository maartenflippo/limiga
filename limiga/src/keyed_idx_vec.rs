use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

pub struct KeyedVec<Idx, Val> {
    buffer: Vec<Val>,
    idx: PhantomData<Idx>,
}

pub trait Key {
    fn to_index(&self) -> usize;
}

impl<Idx: Key, Val: Clone> KeyedVec<Idx, Val> {
    pub fn resize(&mut self, key: Idx, value: Val) {
        self.buffer.resize(key.to_index() + 1, value);
    }
}

impl<Idx, Val> Default for KeyedVec<Idx, Val> {
    fn default() -> Self {
        KeyedVec {
            buffer: vec![],
            idx: PhantomData,
        }
    }
}

impl<Idx: Key, Val> Index<Idx> for KeyedVec<Idx, Val> {
    type Output = Val;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.buffer[index.to_index()]
    }
}

impl<Idx: Key, Val> IndexMut<Idx> for KeyedVec<Idx, Val> {
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        &mut self.buffer[index.to_index()]
    }
}
