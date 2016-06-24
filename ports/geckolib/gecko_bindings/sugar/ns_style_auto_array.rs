use std::iter::{once, Chain, Once, IntoIterator};
use std::slice::{IterMut};
use structs::nsStyleAutoArray;

impl<T> nsStyleAutoArray<T> {
    pub fn iter_mut(&mut self) -> Chain<Once<&mut T>, IterMut<T>> {
        once(&mut self.mFirstElement).chain(self.mOtherElements.iter_mut())
    }

    pub fn len(&self) -> usize {
        1 + self.mOtherElements.len()
    }
}

impl<'a, T> IntoIterator for &'a mut nsStyleAutoArray<T> {
    type Item = &'a mut T;
    type IntoIter = Chain<Once<&'a mut T>, IterMut<'a, T>>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    } 
}