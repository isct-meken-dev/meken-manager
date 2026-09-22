use crate::prelude::*;

errors! {
    pub enum IterError {} {}
    pub type IterResult<T>;
}

#[bulkderive(Persist!)]
#[derive(Clone)]
pub enum LayeredVec<T> {
    Element(T),
    Sequence(Vec<Self>),
}
impl<T: 'static> LayeredVec<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn iter(&self) -> LayeredVecIter<'_, T> {
        todo!()
    }
    pub fn path_iter(&self) -> PathIter<'_, T> {
        todo!()
    }
}
impl<T> Default for LayeredVec<T> {
    fn default() -> Self {
        Self::Sequence(Vec::new())
    }
}

pub struct LayeredVecIter<'a, T>(Vec<&'a LayeredVec<T>>);
impl<'a, T> Iterator for LayeredVecIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(current) = self.0.pop() {
            match current {
                LayeredVec::Element(elem) => {
                    return Some(elem);
                }
                LayeredVec::Sequence(seq) => {
                    for child in seq.iter().rev() {
                        self.0.push(child);
                    }
                }
            }
        }
        None
    }
}

pub struct PathIter<'a, T> {
    _t: &'a T,
}
impl<'a, T> PathIter<'a, T> {
    fn new(root: LayeredVec<T>) -> Self {
        todo!()
    }
}
impl<'a, T> Iterator for PathIter<'a, T> {
    type Item = &'a [usize];

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
