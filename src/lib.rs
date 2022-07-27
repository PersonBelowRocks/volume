use std::ops::Range;

extern crate nalgebra as na;
extern crate thiserror as te;

mod error;
mod impls;
pub use error::*;

#[cfg(test)]
mod tests;

pub type Idx = na::Vector3<i64>;

#[derive(Copy, Clone, Debug)]
pub struct BoundingBox {
    pub pos1: Idx,
    pub pos2: Idx,
}

impl BoundingBox {
    pub fn min(&self) -> Idx {
        use std::cmp::min;

        [
            min(self.pos1.x, self.pos2.x),
            min(self.pos1.y, self.pos2.y),
            min(self.pos1.z, self.pos2.z),
        ]
        .into()
    }

    pub fn max(&self) -> Idx {
        use std::cmp::max;

        [
            max(self.pos1.x, self.pos2.x),
            max(self.pos1.y, self.pos2.y),
            max(self.pos1.z, self.pos2.z),
        ]
        .into()
    }

    pub fn contains(&self, idx: Idx) -> bool {
        let min = self.min();
        let max = self.max();

        (min.x..max.x).contains(&idx.x)
            && (min.y..max.y).contains(&idx.y)
            && (min.z..max.z).contains(&idx.z)
    }
}

impl std::fmt::Display for BoundingBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.pos1, self.pos2)
    }
}

impl From<Range<na::Vector3<i64>>> for BoundingBox {
    fn from(range: Range<na::Vector3<i64>>) -> Self {
        Self {
            pos1: range.start,
            pos2: range.end,
        }
    }
}

pub trait Volume: Sized {
    type Input;
    type Output;

    fn set(&mut self, idx: Idx, item: Self::Input) -> bool;

    fn get(&self, idx: Idx) -> Self::Output;

    fn bounding_box(&self) -> BoundingBox;

    fn contains(&self, idx: Idx) -> bool {
        self.bounding_box().contains(idx)
    }

    #[inline]
    fn apply<F, T, G>(&mut self, f: F)
    where
        F: Fn(Idx, Self::Output) -> Option<Self::Input>,
    {
        let min = self.bounding_box().min();
        let max = self.bounding_box().max();

        let iterator = (min.x..max.x).flat_map(|x| {
            (min.y..max.y).flat_map(move |y| (min.z..max.z).map(move |z| na::vector![x, y, z]))
        });

        for idx in iterator {
            let input = self.get(idx);
            if let Some(output) = f(idx, input) {
                self.set(idx, output);
            }
        }
    }

    #[inline]
    fn idx_apply<F>(&mut self, f: F)
    where
        F: Fn(Idx) -> Option<Self::Input>,
    {
        let min = self.bounding_box().min();
        let max = self.bounding_box().max();

        let iterator = (min.x..max.x).flat_map(|x| {
            (min.y..max.y).flat_map(move |y| (min.z..max.z).map(move |z| na::vector![x, y, z]))
        });

        for idx in iterator {
            if let Some(output) = f(idx) {
                self.set(idx, output);
            }
        }
    }

    #[inline]
    fn fill<Idx, B, T>(&mut self, item: Self::Input)
    where
        Self::Input: Copy,
    {
        self.idx_apply(|_| Some(item));
    }
}

pub struct Subvolume<'a, Vol: Volume>
where
    Vol: Volume,
{
    bounds: BoundingBox,
    vol: &'a mut Vol,
}

impl<'a, Vol> Subvolume<'a, Vol>
where
    Vol: Volume,
{
    pub fn new(vol: &'a mut Vol, bounds: BoundingBox) -> Result<Self, OversizedBounds> {
        if vol.contains(bounds.min()) && vol.contains(bounds.max()) {
            Ok(Self { bounds, vol })
        } else {
            Err(OversizedBounds {
                provided: bounds,
                expected: Some(vol.bounding_box()),
            })
        }
    }

    pub fn resize(self, bounds: BoundingBox) -> Result<Self, OversizedBounds> {
        if self.vol.contains(bounds.max()) && self.vol.contains(bounds.min()) {
            Ok(Self {
                bounds,
                vol: self.vol,
            })
        } else {
            Err(OversizedBounds {
                provided: bounds,
                expected: Some(self.vol.bounding_box()),
            })
        }
    }
}

impl<'a, Vol> Volume for Subvolume<'a, Vol>
where
    Vol: Volume,
{
    type Input = Vol::Input;

    type Output = Option<Vol::Output>;

    fn set(&mut self, idx: Idx, item: Self::Input) -> bool {
        if self.bounds.contains(idx) {
            self.vol.set(idx, item)
        } else {
            false
        }
    }

    fn get(&self, idx: Idx) -> Self::Output {
        self.bounds.contains(idx).then_some(self.vol.get(idx))
    }

    fn bounding_box(&self) -> BoundingBox {
        self.bounds
    }
}
