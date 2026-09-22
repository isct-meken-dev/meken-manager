use crate::core::asset::AssetKey;
use crate::prelude::*;
use crate::util::iter::LayeredVec;
use slotmap::{SlotMap, new_key_type};
use std::collections::BTreeSet;
use std::fmt;

errors! {
    pub enum CutError {} {
        NotFound => "cut not found",
        LimitExceeded => "cut limit exceeded",
    }
    pub type CutResult<T>;
}

#[bulkderive(Persist!)]
#[derive(Default)]
pub struct CutSequence {
    pool: SlotMap<CutKey, Cut>,
    sequence: LayeredVec<CutKey>,
    paths: Vec<CutPath>,
}
impl CutSequence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_cut(&mut self, index: usize) -> CutResult<()> {
        todo!()
    }
    /// `index` を二分割する。元のカットは後方に引き継がれ、前方が新規カットになる。
    /// 新規カットのインデックスは `index` 。
    pub fn split_cut_keep_after(&mut self, index: usize) -> CutResult<()> {
        todo!()
    }
    /// `index` を二分割する。元のカットは前方に引き継がれ、後方が新規カットになる。
    /// 新規カットのインデックスは `index + 1` 。
    pub fn split_cut_keep_before(&mut self, index: usize) -> CutResult<()> {
        todo!()
    }

    pub fn normalize(&mut self) {
        let mut new_sequence = Vec::with_capacity(self.paths.len());
        for &key in self.sequence.iter() {
            if self.cut(key).unwrap().exists() {
                new_sequence.push(LayeredVec::Element(key));
            } else {
                self.pool.remove(key);
            }
        }
        self.sequence = LayeredVec::Sequence(new_sequence);
        self.update_paths();
    }

    pub fn remove(&mut self, key: CutKey) -> CutResult<()> {
        let cut = self.pool.get_mut(key).ok_or(CutError::NotFound)?;
        if cut.exists() {
            *cut = Cut::Removed;
            Ok(())
        } else {
            Err(CutError::NotFound)
        }
    }

    pub fn cut(&self, key: CutKey) -> CutResult<&Cut> {
        self.pool.get(key).ok_or(CutError::NotFound)
    }

    pub fn key(&self, index: usize) -> CutResult<CutKey> {
        let LayeredVec::Element(inner) =
            self.path(index)?.iter().fold(&self.sequence, |seq, &i| {
                let LayeredVec::Sequence(inner) = seq else {
                    unreachable!()
                };
                inner.get(i).unwrap()
            })
        else {
            unreachable!()
        };
        Ok(inner.clone())
    }

    pub fn index(&self, key: CutKey) -> CutResult<usize> {
        self.sequence
            .iter()
            .position(|k| *k == key)
            .ok_or(CutError::NotFound)
    }

    pub fn path(&self, index: usize) -> CutResult<&CutPath> {
        self.paths.get(index).ok_or(CutError::NotFound)
    }

    fn update_paths(&mut self) {
        self.paths.clear();
        self.paths
            .extend(self.sequence.path_iter().map(|v| CutPath::new_unchecked(v)));
    }
}

#[bulkderive(Persist!)]
#[derive(Clone, Default)]
pub struct CutPath(Vec<usize>);
impl<'a> CutPath {
    fn new_unchecked(path: &[usize]) -> Self {
        Self(path.to_vec())
    }

    pub fn depth(&self) -> usize {
        self.0.len()
    }
    pub fn path(&self) -> &[usize] {
        self.0.as_slice()
    }

    pub fn ascend(&mut self) {
        self.0.push(0)
    }
    pub fn increment(&mut self) -> CutResult<()> {
        let cap = match self.depth() {
            0 => return Err(CutError::LimitExceeded),
            1 => 999,
            2 => 26,
            _ => 9,
        };
        let last = self.0.last_mut().unwrap();
        if *last >= cap {
            return Err(CutError::LimitExceeded);
        }
        *last += 1;
        Ok(())
    }

    pub fn iter(&'a self) -> std::slice::Iter<'a, usize> {
        self.0.iter()
    }
}
impl fmt::Display for CutPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, &n) in self.0.iter().enumerate() {
            match i {
                0 => write!(f, "c{:03}", n + 1)?,
                1 => write!(f, "{}", (b'A' + n as u8) as char)?,
                _ => write!(f, "-{}", n + 1)?,
            }
        }
        Ok(())
    }
}

new_key_type! {
    pub struct CutKey;
}

#[bulkderive(Persist!)]
pub enum Cut {
    Exists { asset_keys: BTreeSet<AssetKey> },
    Removed,
}
impl Cut {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn exists(&self) -> bool {
        match self {
            Cut::Exists { .. } => true,
            Cut::Removed => false,
        }
    }
}
impl Default for Cut {
    fn default() -> Self {
        Self::Exists {
            asset_keys: BTreeSet::new(),
        }
    }
}
