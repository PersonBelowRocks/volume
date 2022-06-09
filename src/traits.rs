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
    fn get(this: &Self, idx: Idx) -> Option<&Self::Item>;
    fn set(this: &mut Self, idx: Idx, item: Self::Item);
    fn swap(this: &mut Self, idx: Idx, item: Self::Item) -> Option<Self::Item>;
    fn contains(this: &Self, idx: Idx) -> bool;
}

pub trait VolumeGet<Idx>: Volume {
    fn get(this: &Self, idx: Idx) -> Option<&Self::Item>;
}

impl<T, I> VolumeGet<I> for T
where
    T: VolumeAccess<I>,
{
    #[inline]
    fn get(this: &Self, idx: I) -> Option<&Self::Item> {
        <Self as VolumeAccess<I>>::get(this, idx)
    }
}

pub trait VolumeSet<Idx>: Volume {
    fn set(this: &mut Self, idx: Idx, item: Self::Item);
}

impl<T, I> VolumeSet<I> for T
where
    T: VolumeAccess<I>,
{
    #[inline]
    fn set(this: &mut Self, idx: I, item: Self::Item) {
        <Self as VolumeAccess<I>>::set(this, idx, item)
    }
}

pub trait VolumeSwap<Idx>: Volume {
    fn swap(this: &mut Self, idx: Idx, item: Self::Item) -> Option<Self::Item>;
}

impl<T, I> VolumeSwap<I> for T
where
    T: VolumeAccess<I>,
{
    #[inline]
    fn swap(this: &mut Self, idx: I, item: Self::Item) -> Option<Self::Item> {
        <Self as VolumeAccess<I>>::swap(this, idx, item)
    }
}

pub trait VolumeContains<Idx>: Volume {
    fn contains(this: &Self, idx: Idx) -> bool;
}

impl<T, I> VolumeContains<I> for T
where
    T: VolumeAccess<I>,
{
    #[inline]
    fn contains(this: &Self, idx: I) -> bool {
        <Self as VolumeAccess<I>>::contains(this, idx)
    }
}

/// Provides a bunch of convenience methods related to volumes. Should be implemented for any type
/// acting as a volume. Generic functions operating on volumes should have this trait as a bound for their types.
pub trait Volume: Sized {
    /// The item that this volume will expose in its API. Should correspond with whatever is stored in the underlying memory of the volume.
    type Item;

    /// Wrapper around [`VolumeAccess<Idx>::access`], and requires [`VolumeAccess<Idx>`] to be implemented for the volume.
    /// Returns [`None`] if the given `idx` is invalid (depends on the implementation of [`VolumeAccess<Idx>`]).
    #[inline]
    fn get<Idx>(&self, idx: Idx) -> Option<&Self::Item>
    where
        Self: VolumeGet<Idx>,
    {
        <Self as VolumeGet<Idx>>::get(self, idx)
    }

    #[inline]
    fn set<Idx>(&mut self, idx: Idx, item: Self::Item)
    where
        Self: VolumeSet<Idx>,
    {
        <Self as VolumeSet<Idx>>::set(self, idx, item)
    }

    #[inline]
    fn swap<Idx>(&mut self, idx: Idx, item: Self::Item) -> Option<Self::Item>
    where
        Self: VolumeSwap<Idx>,
    {
        <Self as VolumeSwap<Idx>>::swap(self, idx, item)
    }

    #[inline]
    fn contains<Idx>(&self, idx: Idx) -> bool
    where
        Self: VolumeContains<Idx>,
    {
        <Self as VolumeContains<Idx>>::contains(self, idx)
    }
}
