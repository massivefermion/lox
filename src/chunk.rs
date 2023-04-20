use std::fmt::{self, Debug};
use std::iter::IntoIterator;

#[derive(Clone)]
pub(crate) struct Chunk<T> {
    storage: Vec<T>,
}

impl<T> Chunk<T> {
    pub(crate) fn new() -> Chunk<T> {
        Chunk { storage: vec![] }
    }

    pub(crate) fn add(&mut self, item: T) -> usize {
        self.storage.push(item);
        self.storage.len() - 1
    }

    pub(crate) fn get(&self, address: usize) -> Option<&T> {
        self.storage.get(address)
    }

    pub(crate) fn set(&mut self, address: usize, item: T) {
        self.storage.remove(address);
        self.storage.insert(address, item);
    }

    pub(crate) fn size(&self) -> usize {
        self.storage.len()
    }
}

impl<'a, T> IntoIterator for &'a Chunk<T> {
    type Item = &'a T;
    type IntoIter = ChunkIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        ChunkIterator::from(&self.storage)
    }
}

impl<T: Debug> Debug for Chunk<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (offset, item) in self.storage.iter().enumerate() {
            let string_offset = format!("{:0>4}", offset);
            writeln!(f, "{}    {:?}", string_offset, item)?;
        }
        Ok(())
    }
}

pub(crate) struct ChunkIterator<'a, T> {
    storage: &'a Vec<T>,
    index: usize,
}

impl<'a, T> ChunkIterator<'a, T> {
    fn from(storage: &'a Vec<T>) -> ChunkIterator<'a, T> {
        ChunkIterator { storage, index: 0 }
    }
}

impl<'a, T> Iterator for ChunkIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.storage.len() {
            let item = self.storage.get(self.index);
            self.index += 1;
            return item;
        }
        None
    }
}
