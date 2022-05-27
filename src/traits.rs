use crate::types::*;
use crate::util;
use num_traits::NumCast;
use num_traits::PrimInt;

pub(crate) trait BasicCloneFill<T: Clone> {
    fn clone_fill(size: [usize; 3], item: T) -> Self;
}

pub trait VolumeIdx: Sized + Copy {
    /// Create a new index from X, Y, and Z components.
    /// # Panics
    /// Implementors may panic if `N` is not a valid type to build `Self` from.
    fn from_xyz<N: PrimInt>(x: N, y: N, z: N) -> Self;

    /// Cast this index to an array of an integer type.
    /// Returns `None` if the cast failed.
    fn array<T: NumCast + PrimInt>(self) -> Option<[T; 3]>;
}
pub trait Volume: Sized {
    type Item;

    /// Get a reference to the item at the given index in localspace.
    /// Implementors must make sure that this function returns [`None`] if the index is out of bounds.
    ///
    /// This function is used by [`Volume::get`], which internally (by default) uses the volume's bounding box's minimum
    /// to convert the index to localspace (index - bounding box minimum). This is assumed to be the case for a type implementing volume.
    /// If this for whatever reason is not the case for your volume (which is already a sign of problems), then override [`Volume::get`]
    /// to the appropriate implementation.
    fn ls_get<Idx: VolumeIdx>(&self, idx: Idx) -> Option<&Self::Item>;

    /// Get a mutable reference to the item at the given index in localspace.
    /// Implementors must make sure that this function returns [`None`] if the index is out of bounds.
    ///
    /// This function is used by [`Volume::get_mut`], which internally (by default) uses the volume's bounding box's minimum
    /// to convert the index to localspace (index - bounding box minimum). This is assumed to be the case for a type implementing volume.
    /// If this for whatever reason is not the case for your volume (which is already a sign of problems), then override [`Volume::get_mut`]
    /// to the appropriate implementation.
    fn ls_get_mut<Idx: VolumeIdx>(&mut self, idx: Idx) -> Option<&mut Self::Item>;

    /// Get a bounding box representing this volume's bounds. Implementors must assume that any position within the bounding box is a valid worldspace index
    /// so that [`Volume::get`] and [`Volume::get_mut`] do not return [`None`] when given the index.
    ///
    /// Much like an iterator's size hint, unsafe code SHOULD NOT rely on [`Volume::bounding_box`] for anything potentially bad.
    fn bounding_box(&self) -> BoundingBox;

    /// Converts a worldspace index to a localspace index by using this volume's bounding box's minimum.
    /// Returns [`None`] if the conversion was unsucessful (e.g., if the index is less than the bounding box's minimum, making it OOB).
    #[inline(always)]
    fn to_ls<Idx: VolumeIdx>(&self, idx: Idx) -> Option<[u64; 3]> {
        let [x, y, z] = idx.array::<i64>()?;
        let min = self.bounding_box().min();
        let ls_idx = util::sub_ivec3([x, y, z], min);

        util::cast_ivec3(ls_idx)
    }

    /// Get a reference to the item at the given worldspace index. Returns [`None`] if the index was invalid (e.g., out of bounds).
    /// Uses [`Volume::to_ls`] internally to convert the worldspace index to a localspace index, after which the implementor must handle it.
    #[inline(always)]
    fn get<Idx: VolumeIdx>(&self, idx: Idx) -> Option<&Self::Item> {
        let ls_idx = self.to_ls(idx)?;

        self.ls_get(ls_idx)
    }

    /// Get a mutable reference to the item at the given worldpace index. Returns [`None`] if the index was invalid (e.g., out of bounds).
    /// Uses [`Volume::to_ls`] internally to convert the worldspace index to a localspace index, after which the implementor must handle it.
    #[inline(always)]
    fn get_mut<Idx: VolumeIdx>(&mut self, idx: Idx) -> Option<&mut Self::Item> {
        let ls_idx = self.to_ls(idx)?;

        self.ls_get_mut(ls_idx)
    }

    /// Swap the item at the given worldspace index with the provided `item`, returning the previous item.
    /// Returns [`None`] if the index was invalid (e.g., out of bounds).
    ///
    /// Relies on [`Volume::get_mut`] internally.
    #[inline(always)]
    fn swap<Idx: VolumeIdx>(&mut self, idx: Idx, item: Self::Item) -> Option<Self::Item> {
        let slot = self.get_mut(idx)?;

        Some(std::mem::replace(slot, item))
    }

    /// Checks if this volume contains the worldspace index.
    #[inline(always)]
    fn contains<Idx: VolumeIdx>(&self, idx: Idx) -> bool {
        self.bounding_box().contains::<Idx>(idx)
    }

    /// Iterate over the worldspace indices of this volume.
    #[inline(always)]
    fn iter_indices(&self) -> BoundingBoxIterator {
        self.bounding_box().into_iter()
    }

    /// Iterate over the elements in this volume.
    #[inline(always)]
    fn iter(&self) -> VolumeIterator<'_, Self> {
        VolumeIterator {
            volume: self,
            bb_iterator: self.iter_indices(),
        }
    }

    #[inline(always)]
    fn insert<Idx, Rhs>(&mut self, at: Idx, rhs: &Rhs) -> Result<(), InsertError>
    where
        Rhs: Volume<Item = Self::Item>,
        Idx: VolumeIdx,
        Self::Item: Copy,
    {
        let at = at.array::<i64>().unwrap();

        let rhs_min = rhs.bounding_box().min();
        let rhs_max = rhs.bounding_box().max();

        if !self.contains(util::sum_ivec3(at, rhs_min))
            || !self.contains(util::sum_ivec3(at, rhs_max))
        {
            return Err(InsertError::VolumeEscapesBounds);
        }

        for rhs_idx in rhs.iter_indices() {
            let item = *rhs.get(rhs_idx).unwrap();
            self.swap(util::sum_ivec3(at, rhs_idx), item).unwrap();
        }

        Ok(())
    }

    #[inline(always)]
    fn insert_anyways<Idx, Rhs>(&mut self, at: Idx, rhs: &Rhs)
    where
        Rhs: Volume<Item = Self::Item>,
        Idx: VolumeIdx,
        Self::Item: Copy,
    {
        let at = at.array::<i64>().unwrap();

        for rhs_idx in rhs.iter_indices() {
            if let Some(&item) = rhs.get(rhs_idx) {
                self.swap(util::sum_ivec3(at, rhs_idx), item);
            }
        }
    }
}
