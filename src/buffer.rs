use std::ops::{Index, IndexMut};
use std::slice::Iter;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Buffer<T, const N: usize> {
    inner: [T; N]
}

impl<T, const N: usize> Buffer<T, N> {
    pub fn new(inital_data: [T; N]) -> Self {
        Self {
            inner: inital_data,
        }
    }

    pub fn insert_data(&mut self, data: T) {
        self.inner.rotate_right(1);
        self[0] = data;
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.inner.iter()
    }

    pub fn len(&self) -> usize {
        N
    }
}

impl<T, const N: usize> Index<usize> for Buffer<T, N> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl<T, const N: usize> IndexMut<usize> for Buffer<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        &mut self.inner[index]
    }
}

impl<T, const N: usize> IntoIterator for Buffer<T, N> {
    type Item = T;

    type IntoIter = <[T; N] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a Buffer<T, N> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}