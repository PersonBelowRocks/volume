use crate::types::*;
use num_traits::NumCast;
use num_traits::PrimInt;

pub trait VolumeIdx: Sized + Copy {
    /// Create a new index from X, Y, and Z components.
    /// # Panics
    /// Implementors may panic if `N` is not a valid type to build `Self` from.
    fn from_xyz<N: PrimInt>(x: N, y: N, z: N) -> Self;

    /// Cast this index to an array of an integer type.
    /// Returns `None` if the cast failed.
    fn array<T: NumCast + PrimInt>(self) -> Option<[T; 3]>;
}

pub trait VolumeAccess<Idx>: Volume {
    fn access(&self, idx: Idx) -> Option<&Self::Item>;
    fn access_mut(&mut self, idx: Idx) -> Option<&mut Self::Item>;
}

pub trait Volume: Sized {
    type Item;

    #[inline]
    fn get<Idx>(&self, idx: Idx) -> Option<&Self::Item>
    where
        Self: VolumeAccess<Idx>,
    {
        <Self as VolumeAccess<Idx>>::access(self, idx)
    }

    #[inline]
    fn get_mut<Idx>(&mut self, idx: Idx) -> Option<&mut Self::Item>
    where
        Self: VolumeAccess<Idx>,
    {
        <Self as VolumeAccess<Idx>>::access_mut(self, idx)
    }

    /// Get a bounding box representing this volume's bounds. Implementors must assume that any position within the bounding box is a valid worldspace index
    /// so that [`Volume::get`] and [`Volume::get_mut`] do not return [`None`] when given the index.
    ///
    /// Much like an iterator's size hint, unsafe code SHOULD NOT rely on [`Volume::bounding_box`] for anything potentially bad.
    fn bounding_box(&self) -> BoundingBox;

    /// Swap the item at the given worldspace index with the provided `item`, returning the previous item.
    /// Returns [`None`] if the index was invalid (e.g., out of bounds).
    ///
    /// Relies on [`Volume::get_mut`] internally.
    #[inline]
    fn swap<Idx>(&mut self, idx: Idx, item: Self::Item) -> Option<Self::Item>
    where
        Self: VolumeAccess<Idx>,
    {
        Some(std::mem::replace(self.get_mut(idx)?, item))
    }

    /// Checks if this volume contains the worldspace index.
    #[inline]
    fn contains<Idx: VolumeIdx>(&self, idx: Idx) -> bool {
        self.bounding_box().contains::<Idx>(idx)
    }

    /// Iterate over the worldspace indices of this volume.
    #[inline]
    fn iter_indices(&self) -> BoundingBoxIterator {
        self.bounding_box().into_iter()
    }

    /// Iterate over the elements in this volume.
    #[inline]
    fn iter(&self) -> VolumeIterator<'_, Self>
    where
        Self: VolumeAccess<[i64; 3]>,
    {
        VolumeIterator {
            volume: self,
            bb_iterator: self.iter_indices(),
        }
    }
}
