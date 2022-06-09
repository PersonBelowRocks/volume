use crate::builtins::*;
use crate::util;

pub(crate) mod heap_volume {
    use super::*;

    type HeapVolumeStorage<T> = Box<[Box<[Box<[T]>]>]>;

    /// Heap allocated volume. Slower to allocate/create than [`StackVolume`] but is more flexible and can have more exotic bounds.
    pub struct HeapVolume<T> {
        inner: HeapVolumeStorage<T>,
        bounds: BoundingBox,
    }

    impl_indexing!(T, HeapVolume<T>);
    impl_debug!(T, HeapVolume<T>);

    impl<T> HeapVolume<T> {
        #[inline]
        fn ls_get(&self, idx: [usize; 3]) -> Option<&<Self as Volume>::Item> {
            let [x, y, z] = idx;
            self.inner.get(x)?.get(y)?.get(z)
        }

        #[inline]
        fn ls_get_mut(&mut self, idx: [usize; 3]) -> Option<&mut <Self as Volume>::Item> {
            let [x, y, z] = idx;
            self.inner.get_mut(x)?.get_mut(y)?.get_mut(z)
        }

        #[inline]
        pub fn bounding_box(&self) -> BoundingBox {
            self.bounds
        }
    }

    impl<T: Clone> HeapVolume<T> {
        /// Create a new heap allocated volume with the provided `bounds` and filled with the provided `item`.
        ///
        /// # Example
        /// ```rust
        /// use volume::{HeapVolume, BoundingBox, Volume, VolumeAccess};
        ///
        /// let vol = HeapVolume::new(10, BoundingBox::new_origin([20usize, 20, 20]));
        ///
        /// assert_eq!(vol.bounding_box().dimensions(), [20, 20, 20]);
        /// assert_eq!(vol.get([8i32, 7, 2]), Some(&10));
        /// ```
        #[inline]
        pub fn new(item: T, bounds: impl Into<BoundingBox>) -> Self {
            use util::boxed_slice;

            let bounds: BoundingBox = bounds.into();

            let [x, y, z] = util::cast_ivec3(bounds.dimensions()).unwrap();

            Self {
                inner: boxed_slice(boxed_slice(boxed_slice(item, z), y), x),
                bounds,
            }
        }
    }

    impl<T: Clone> Clone for HeapVolume<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
                bounds: self.bounds,
            }
        }
    }

    impl<T, Idx: VolumeIdx> VolumeAccess<Idx> for HeapVolume<T> {
        #[inline]
        fn get(this: &Self, idx: Idx) -> Option<&Self::Item> {
            this.ls_get(idx.array::<usize>()?)
        }

        #[inline]
        fn set(this: &mut Self, idx: Idx, item: Self::Item) {
            if let Some(slot) = idx.array::<usize>().and_then(|i| this.ls_get_mut(i)) {
                *slot = item;
            }
        }

        #[inline]
        fn swap(this: &mut Self, idx: Idx, item: Self::Item) -> Option<Self::Item> {
            use std::mem::replace;

            let slot = this.ls_get_mut(idx.array::<usize>()?)?;
            Some(replace(slot, item))
        }

        #[inline]
        fn contains(this: &Self, idx: Idx) -> bool {
            if let Some([x, y, z]) = idx.array::<usize>() {
                x < this.inner.len() && y < this.inner[0].len() && z < this.inner[0][0].len()
            } else {
                false
            }
        }
    }

    #[derive(Copy, Clone)]
    pub struct Worldspace<Idx: VolumeIdx>(pub Idx);

    impl<Idx: VolumeIdx> Worldspace<Idx> {
        #[inline]
        fn to_ls(self, b: BoundingBox) -> Option<Idx> {
            use util::sub_ivec3;

            let [x, y, z] = sub_ivec3(self.0.array::<i64>()?, b.min());

            Some(Idx::from_xyz(x, y, z))
        }
    }

    impl<T, Idx: VolumeIdx> VolumeAccess<Worldspace<Idx>> for HeapVolume<T>
    where
        Self: VolumeAccess<Idx>,
    {
        #[inline]
        fn get(this: &Self, idx: Worldspace<Idx>) -> Option<&Self::Item> {
            let ls_idx = idx.to_ls(this.bounding_box())?;
            <Self as VolumeAccess<Idx>>::get(this, ls_idx)
        }

        #[inline]
        fn set(this: &mut Self, idx: Worldspace<Idx>, item: Self::Item) {
            if let Some(ls_idx) = idx.to_ls(this.bounding_box()) {
                <Self as VolumeAccess<Idx>>::set(this, ls_idx, item)
            }
        }

        #[inline]
        fn swap(this: &mut Self, idx: Worldspace<Idx>, item: Self::Item) -> Option<Self::Item> {
            let ls_idx = idx.to_ls(this.bounding_box())?;
            <Self as VolumeAccess<Idx>>::swap(this, ls_idx, item)
        }

        #[inline]
        fn contains(this: &Self, idx: Worldspace<Idx>) -> bool {
            this.bounding_box().contains(idx.0)
        }
    }

    impl<T> Volume for HeapVolume<T> {
        type Item = T;
    }

    impl<const X: usize, const Y: usize, const Z: usize, T> From<[[[T; Z]; Y]; X]> for HeapVolume<T> {
        #[inline]
        fn from(array: [[[T; Z]; Y]; X]) -> Self {
            let data = {
                array
                    .into_iter()
                    .map(|array2| {
                        array2
                            .into_iter()
                            .map(|array3| {
                                let mut v = Vec::with_capacity(Z);
                                v.extend(array3.into_iter());
                                v.into_boxed_slice()
                            })
                            .collect::<Vec<_>>()
                            .into_boxed_slice()
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            };

            Self {
                inner: data,
                bounds: BoundingBox::new([0, 0, 0], [X, Y, Z]),
            }
        }
    }
}

pub(crate) mod stack_volume {
    use crate::builtins::*;

    type StackVolumeStorage<const X: usize, const Y: usize, const Z: usize, T> = [[[T; Z]; Y]; X];

    /// Stack allocated volume with size known at compile time. Faster to allocate/create than [`HeapVolume`] but not as flexible.
    pub struct StackVolume<const X: usize, const Y: usize, const Z: usize, T> {
        inner: StackVolumeStorage<X, Y, Z, T>,
    }

    impl<const X: usize, const Y: usize, const Z: usize, T: Copy> StackVolume<X, Y, Z, T> {
        #[inline]
        pub fn filled(item: T) -> Self {
            Self {
                inner: [[[item; Z]; Y]; X],
            }
        }
    }

    impl<const X: usize, const Y: usize, const Z: usize, T: Clone> Clone for StackVolume<X, Y, Z, T> {
        #[inline]
        fn clone(&self) -> Self {
            self.inner.clone().into()
        }
    }

    impl<const X: usize, const Y: usize, const Z: usize, T> StackVolume<X, Y, Z, T> {
        #[inline]
        fn ls_get(&self, idx: [usize; 3]) -> Option<&<Self as Volume>::Item> {
            let [x, y, z] = idx;
            self.inner.get(x)?.get(y)?.get(z)
        }

        #[inline]
        fn ls_get_mut(&mut self, idx: [usize; 3]) -> Option<&mut <Self as Volume>::Item> {
            let [x, y, z] = idx;
            self.inner.get_mut(x)?.get_mut(y)?.get_mut(z)
        }
    }

    impl<const X: usize, const Y: usize, const Z: usize, T: Copy + Default> Default
        for StackVolume<X, Y, Z, T>
    {
        #[inline]
        fn default() -> Self {
            Self::filled(T::default())
        }
    }

    impl<const X: usize, const Y: usize, const Z: usize, T> From<[[[T; Z]; Y]; X]>
        for StackVolume<X, Y, Z, T>
    {
        #[inline]
        fn from(arr: StackVolumeStorage<X, Y, Z, T>) -> Self {
            Self { inner: arr }
        }
    }

    impl<const X: usize, const Y: usize, const Z: usize, T> From<StackVolume<X, Y, Z, T>>
        for [[[T; Z]; Y]; X]
    {
        #[inline]
        fn from(vol: StackVolume<X, Y, Z, T>) -> Self {
            vol.inner
        }
    }

    impl<const X: usize, const Y: usize, const Z: usize, T> From<StackVolume<X, Y, Z, T>>
        for HeapVolume<T>
    {
        #[inline]
        fn from(vol: StackVolume<X, Y, Z, T>) -> Self {
            Self::from(vol.inner)
        }
    }

    impl<const X: usize, const Y: usize, const Z: usize, T, Idx: VolumeIdx> VolumeAccess<Idx>
        for StackVolume<X, Y, Z, T>
    {
        #[inline]
        fn get(this: &Self, idx: Idx) -> Option<&Self::Item> {
            this.ls_get(idx.array::<usize>()?)
        }

        #[inline]
        fn set(this: &mut Self, idx: Idx, item: Self::Item) {
            if let Some(slot) = idx.array::<usize>().and_then(|i| this.ls_get_mut(i)) {
                *slot = item;
            }
        }

        #[inline]
        fn swap(this: &mut Self, idx: Idx, item: Self::Item) -> Option<Self::Item> {
            use std::mem::replace;

            let slot = this.ls_get_mut(idx.array::<usize>()?)?;
            Some(replace(slot, item))
        }

        #[inline]
        fn contains(_this: &Self, idx: Idx) -> bool {
            if let Some([x, y, z]) = idx.array::<usize>() {
                x < X && y < Y && z < Z
            } else {
                false
            }
        }
    }

    impl<const X: usize, const Y: usize, const Z: usize, T, Idx: VolumeIdx> std::ops::Index<Idx>
        for StackVolume<X, Y, Z, T>
    {
        type Output = <Self as Volume>::Item;

        #[inline]
        fn index(&self, idx: Idx) -> &Self::Output {
            self.get(idx).unwrap()
        }
    }

    impl<const X: usize, const Y: usize, const Z: usize, T> Volume for StackVolume<X, Y, Z, T> {
        type Item = T;
    }
}
