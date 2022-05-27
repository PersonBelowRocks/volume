use crate::prelude::*;
use crate::util;

pub(crate) mod heap_volume {
    use super::*;

    type HeapVolumeStorage<T> = Box<[Box<[Box<[T]>]>]>;

    pub struct HeapVolume<T> {
        inner: HeapVolumeStorage<T>,
        bounds: BoundingBox,
    }

    impl_indexing!(T, HeapVolume<T>);
    impl_debug!(T, HeapVolume<T>);

    impl<T: Clone> HeapVolume<T> {
        #[inline(always)]
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

    impl<T: PartialEq> std::cmp::PartialEq for HeapVolume<T> {
        #[inline(always)]
        fn eq(&self, other: &Self) -> bool {
            self.bounding_box() == other.bounding_box()
                && !(self.iter().zip(other.iter()).any(|(a, b)| a != b))
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

    impl<T> Volume for HeapVolume<T> {
        type Item = T;

        #[inline]
        fn ls_get<Idx: VolumeIdx>(&self, idx: Idx) -> Option<&Self::Item> {
            let [x, y, z] = idx.array::<usize>()?;

            self.inner.get(x)?.get(y)?.get(z)
        }

        #[inline]
        fn ls_get_mut<Idx: VolumeIdx>(&mut self, idx: Idx) -> Option<&mut Self::Item> {
            let [x, y, z] = idx.array::<usize>()?;

            self.inner.get_mut(x)?.get_mut(y)?.get_mut(z)
        }

        #[inline]
        fn bounding_box(&self) -> BoundingBox {
            self.bounds
        }
    }

    impl<const X: usize, const Y: usize, const Z: usize, T> From<[[[T; Z]; Y]; X]> for HeapVolume<T> {
        #[inline(always)]
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
    use super::*;

    type StackVolumeStorage<const X: usize, const Y: usize, const Z: usize, T> = [[[T; Z]; Y]; X];

    pub struct StackVolume<const X: usize, const Y: usize, const Z: usize, T> {
        inner: StackVolumeStorage<X, Y, Z, T>,
    }

    impl<const X: usize, const Y: usize, const Z: usize, T: Copy> StackVolume<X, Y, Z, T> {
        #[inline(always)]
        pub fn filled(item: T) -> Self {
            Self {
                inner: [[[item; Z]; Y]; X],
            }
        }
    }

    impl<const X: usize, const Y: usize, const Z: usize, T: Copy + Default> Default
        for StackVolume<X, Y, Z, T>
    {
        #[inline(always)]
        fn default() -> Self {
            Self::filled(T::default())
        }
    }

    impl<const X: usize, const Y: usize, const Z: usize, T> StackVolume<X, Y, Z, Option<T>>
    where
        Option<T>: Copy,
    {
        #[inline(always)]
        pub fn new_none() -> Self {
            Self::filled(None)
        }
    }

    impl<const X: usize, const Y: usize, const Z: usize, T> From<StackVolumeStorage<X, Y, Z, T>>
        for StackVolume<X, Y, Z, T>
    {
        fn from(arr: StackVolumeStorage<X, Y, Z, T>) -> Self {
            Self { inner: arr }
        }
    }

    impl<const X: usize, const Y: usize, const Z: usize, T> From<StackVolume<X, Y, Z, T>>
        for StackVolumeStorage<X, Y, Z, T>
    {
        fn from(vol: StackVolume<X, Y, Z, T>) -> Self {
            vol.inner
        }
    }

    impl<const X: usize, const Y: usize, const Z: usize, T> From<StackVolume<X, Y, Z, T>>
        for HeapVolume<T>
    {
        #[inline(always)]
        fn from(vol: StackVolume<X, Y, Z, T>) -> Self {
            Self::from(vol.inner)
        }
    }

    impl<const X: usize, const Y: usize, const Z: usize, T, Idx: VolumeIdx> std::ops::Index<Idx>
        for StackVolume<X, Y, Z, T>
    {
        type Output = <Self as Volume>::Item;

        fn index(&self, idx: Idx) -> &Self::Output {
            self.get(idx).unwrap()
        }
    }

    impl<const X: usize, const Y: usize, const Z: usize, T, Idx: VolumeIdx> std::ops::IndexMut<Idx>
        for StackVolume<X, Y, Z, T>
    {
        fn index_mut(&mut self, idx: Idx) -> &mut Self::Output {
            self.get_mut(idx).unwrap()
        }
    }

    impl<const X: usize, const Y: usize, const Z: usize, T> Volume for StackVolume<X, Y, Z, T> {
        type Item = T;

        fn ls_get<Idx: VolumeIdx>(&self, idx: Idx) -> Option<&Self::Item> {
            let [x, y, z] = idx.array::<usize>()?;

            self.inner.get(x)?.get(y)?.get(z)
        }

        fn ls_get_mut<Idx: VolumeIdx>(&mut self, idx: Idx) -> Option<&mut Self::Item> {
            let [x, y, z] = idx.array::<usize>()?;

            self.inner.get_mut(x)?.get_mut(y)?.get_mut(z)
        }

        fn bounding_box(&self) -> BoundingBox {
            BoundingBox::new([0; 3], [X, Y, Z])
        }
    }
}
