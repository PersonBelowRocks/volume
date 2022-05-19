use crate::types::*;
use crate::util;
use num_traits::NumCast;

pub(crate) trait BasicCloneFill<T: Clone> {
    fn clone_fill(size: [usize; 3], item: T) -> Self;
}

pub trait VolumeIdx: Sized {
    /// Unpack the index and cast the components to a type of your choosing.
    /// Will return [`None`] if the cast failed (e.g., components are signed and negative and attempted to cast to an unsigned type).
    fn unpack<T: NumCast>(self) -> Option<(T, T, T)>;

    #[inline(always)]
    fn to_arr<T: NumCast>(self) -> Option<[T; 3]> {
        let (x, y, z) = self.unpack::<T>()?;
        Some([x, y, z])
    }
}

pub trait RelocatableVolume: Volume {
    fn mut_bounding_box(&mut self) -> &mut BoundingBox;
}

pub trait ResizableVolume
where
    Self: RelocatableVolume,
    <Self as Volume>::Item: Copy,
{
    fn resize(&mut self, new_bounds: BoundingBox, filling: <Self as Volume>::Item) -> Self;
}

pub trait Volume: Sized {
    type Item;

    /// Get immutable reference to the item at the given `idx`. Returns [`None`] if the index was invalid (e.g., out of bounds).
    fn get<Idx: VolumeIdx>(&self, idx: Idx) -> Option<&Self::Item>;

    /// Get mutable reference to the item at the given `idx`. Returns [`None`] if the index was invalid (e.g., out of bounds).
    fn get_mut<Idx: VolumeIdx>(&mut self, idx: Idx) -> Option<&mut Self::Item>;

    fn bounding_box(&self) -> BoundingBox;

    /// Swap the item at the given `idx` with the provided `item`, returning the previous item.
    /// Returns [`None`] if the index was invalid (e.g., out of bounds).
    #[inline(always)]
    fn swap<Idx: VolumeIdx>(&mut self, idx: Idx, item: Self::Item) -> Option<Self::Item> {
        let slot = self.get_mut(idx)?;

        Some(std::mem::replace(slot, item))
    }

    #[inline(always)]
    fn contains<Idx: VolumeIdx>(&self, idx: Idx) -> bool {
        self.bounding_box().contains::<Idx>(idx)
    }

    #[inline(always)]
    fn iter_indices(&self) -> BoundingBoxIterator {
        self.bounding_box().into_iter()
    }

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
        let at = at.to_arr::<i64>().unwrap();

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
        let at = at.to_arr::<i64>().unwrap();

        for rhs_idx in rhs.iter_indices() {
            if let Some(&item) = rhs.get(rhs_idx) {
                self.swap(util::sum_ivec3(at, rhs_idx), item);
            }
        }
    }
}
