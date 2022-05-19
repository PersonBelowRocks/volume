use crate::prelude::*;
use crate::util;
use num_traits::{NumCast, PrimInt};

#[derive(te::Error, Debug)]
#[non_exhaustive]
pub enum InsertError {
    #[error("inserting volume A into volume B at the given index would cause parts of A to be outside of B")]
    VolumeEscapesBounds,
}

impl<N: PrimInt> VolumeIdx for [N; 3] {
    #[inline(always)]
    fn unpack<T: NumCast>(self) -> Option<(T, T, T)> {
        Some((
            <T as NumCast>::from(self[0])?,
            <T as NumCast>::from(self[1])?,
            <T as NumCast>::from(self[2])?,
        ))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BoundingBox {
    min: [i64; 3],
    max: [i64; 3],
}

impl BoundingBox {
    /// Construct a new bounding box spanning the two positions.
    /// The "closest" and "furthest" corners of the box spanned by the positions will be extracted and stored in the resulting [`BoundingBox`]
    /// # Panics
    /// Panics if `pos1` and `pos2` cannot be cast to `[i64; 3]`.
    #[inline(always)]
    pub fn new<N: PrimInt>(pos1: [N; 3], pos2: [N; 3]) -> Self {
        use std::cmp::{max, min};

        let min = [
            min(pos1[0], pos2[0]),
            min(pos1[1], pos2[1]),
            min(pos1[2], pos2[2]),
        ];
        let max = [
            max(pos1[0], pos2[0]),
            max(pos1[1], pos2[1]),
            max(pos1[2], pos2[2]),
        ];

        Self {
            min: util::cast_ivec3(min).unwrap(),
            max: util::cast_ivec3(max).unwrap(),
        }
    }

    #[inline(always)]
    pub fn capacity(&self) -> i128 {
        let [x, y, z] = util::sub_ivec3(self.max(), self.min());
        (x as i128) * (y as i128) * (z as i128)
    }

    #[inline(always)]
    pub fn contains<Idx: VolumeIdx>(&self, idx: Idx) -> bool {
        let (x, y, z) = match idx.unpack::<i64>() {
            Some(tuple) => tuple,
            None => return false,
        };

        (self.min[0]..self.max[0]).contains(&x)
            && (self.min[1]..self.max[1]).contains(&y)
            && (self.min[2]..self.max[2]).contains(&z)
    }

    #[inline(always)]
    pub fn intersection(&self, rhs: &BoundingBox) -> Option<Self> {
        use std::cmp::{max, min};

        if !self.overlaps(rhs) {
            return None;
        }

        let pos1 = [
            max(self.min[0], rhs.min[0]),
            max(self.min[1], rhs.min[1]),
            max(self.min[2], rhs.min[2]),
        ];

        let pos2 = [
            min(self.max[0], rhs.max[0]),
            min(self.max[1], rhs.max[1]),
            min(self.max[2], rhs.max[2]),
        ];

        Some(Self::new(pos1, pos2))
    }

    #[inline(always)]
    pub fn overlaps(&self, rhs: &BoundingBox) -> bool {
        self.min[0] < rhs.max[0]
            && self.max[0] > rhs.min[0]
            && self.min[1] < rhs.max[1]
            && self.max[1] > rhs.min[1]
            && self.min[2] < rhs.max[2]
            && self.max[2] > rhs.min[2]
    }

    #[inline(always)]
    pub fn max(&self) -> [i64; 3] {
        self.max
    }

    #[inline(always)]
    pub fn min(&self) -> [i64; 3] {
        self.min
    }

    #[inline(always)]
    pub fn x_span(&self) -> i64 {
        i64::abs(self.max[0] - self.min[0])
    }

    #[inline(always)]
    pub fn y_span(&self) -> i64 {
        i64::abs(self.max[1] - self.min[1])
    }

    #[inline(always)]
    pub fn z_span(&self) -> i64 {
        i64::abs(self.max[2] - self.min[2])
    }

    #[inline(always)]
    pub fn dimensions(&self) -> [i64; 3] {
        [self.x_span(), self.y_span(), self.z_span()]
    }
}

impl std::fmt::Display for BoundingBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [sx, sy, sz] = self.min();
        let [bx, by, bz] = self.max();

        write!(
            f,
            "BoundingBox {{ min: ({sx}, {sy}, {sz}), max: ({bx}, {by}, {bz}) }}"
        )
    }
}

impl IntoIterator for BoundingBox {
    type Item = [i64; 3];
    type IntoIter = BoundingBoxIterator;

    fn into_iter(self) -> Self::IntoIter {
        BoundingBoxIterator {
            current: self.min,
            bounding_box: self,
        }
    }
}

pub struct BoundingBoxIterator {
    current: [i64; 3],
    bounding_box: BoundingBox,
}

impl Iterator for BoundingBoxIterator {
    type Item = [i64; 3];

    #[inline(always)]
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let bb = self.bounding_box;

        let out = if self.current[2] >= bb.max()[2] {
            None
        } else {
            Some(self.current)
        };

        self.current[0] += 1;
        if self.current[0] >= bb.max()[0] {
            self.current[0] = bb.min()[0];
            self.current[1] += 1;

            if self.current[1] >= bb.max()[1] {
                self.current[1] = bb.min()[1];
                self.current[2] += 1;
            }
        }

        out
    }
}

pub struct VolumeIterator<'a, Vol: Volume> {
    pub(crate) volume: &'a Vol,
    pub(crate) bb_iterator: BoundingBoxIterator,
}

impl<'a, Vol: Volume> Iterator for VolumeIterator<'a, Vol> {
    type Item = &'a <Vol as Volume>::Item;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.bb_iterator.next()?;
        self.volume.get(idx)
    }
}
