use crate::prelude::*;
use crate::util;

pub(crate) mod heap_volume {
    use crate::{spaces::Space, traits::VolumeAccess};

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
        pub fn ws_get<Idx: VolumeIdx>(&self, idx: Idx) -> Option<&<Self as Volume>::Item> {
            self.get(Space::Worldspace(idx))
        }

        #[inline]
        pub fn ws_get_mut<Idx: VolumeIdx>(
            &mut self,
            idx: Idx,
        ) -> Option<&mut <Self as Volume>::Item> {
            self.get_mut(Space::Worldspace(idx))
        }

        #[inline]
        pub fn ws_swap<Idx: VolumeIdx>(
            &mut self,
            idx: Idx,
            item: <Self as Volume>::Item,
        ) -> Option<<Self as Volume>::Item> {
            self.swap(Space::Worldspace(idx), item)
        }
    }

    impl<T: Clone> HeapVolume<T> {
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

    impl<T: PartialEq> std::cmp::PartialEq for HeapVolume<T> {
        #[inline]
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

    impl<T, Idx: VolumeIdx> VolumeAccess<Idx> for HeapVolume<T> {
        #[inline]
        fn access(&self, idx: Idx) -> Option<&Self::Item> {
            self.access(Space::Localspace(idx))
        }

        #[inline]
        fn access_mut(&mut self, idx: Idx) -> Option<&mut Self::Item> {
            self.access_mut(Space::Localspace(idx))
        }
    }

    impl<T, Idx: VolumeIdx> VolumeAccess<Space<Idx>> for HeapVolume<T> {
        #[inline]
        fn access(&self, idx: Space<Idx>) -> Option<&Self::Item> {
            match idx {
                Space::Worldspace(pos) => {
                    let pos = pos.array::<i64>()?;
                    self.access(Space::Localspace(util::sub_ivec3(
                        pos,
                        self.bounding_box().min(),
                    )))
                }
                Space::Localspace(pos) => {
                    let [x, y, z] = pos.array::<usize>()?;
                    self.inner.get(x)?.get(y)?.get(z)
                }
            }
        }

        #[inline]
        fn access_mut(&mut self, idx: Space<Idx>) -> Option<&mut Self::Item> {
            match idx {
                Space::Worldspace(pos) => {
                    let pos = pos.array::<i64>()?;
                    self.access_mut(Space::Localspace(util::sub_ivec3(
                        pos,
                        self.bounding_box().min(),
                    )))
                }
                Space::Localspace(pos) => {
                    let [x, y, z] = pos.array::<usize>()?;
                    self.inner.get_mut(x)?.get_mut(y)?.get_mut(z)
                }
            }
        }
    }

    impl<T> Volume for HeapVolume<T> {
        type Item = T;

        #[inline]
        fn bounding_box(&self) -> BoundingBox {
            self.bounds
        }
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
    use std::any::type_name;

    use crate::traits::VolumeAccess;

    use super::*;

    type StackVolumeStorage<const X: usize, const Y: usize, const Z: usize, T> = [[[T; Z]; Y]; X];

    /// Stack allocated volume with size known at compile time. Faster to allocate/create than [`HeapVolume`] but not as flexible.
    pub struct StackVolume<const X: usize, const Y: usize, const Z: usize, T> {
        inner: StackVolumeStorage<X, Y, Z, T>,
    }

    impl<const X: usize, const Y: usize, const Z: usize, T> std::fmt::Debug
        for StackVolume<X, Y, Z, T>
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let capacity = self.bounding_box().capacity();

            write!(f, "StackVolume<{}> {{", type_name::<T>())?;
            write!(f, "    bounds: {},", self.bounding_box())?;
            write!(f, "    capacity: {}", capacity)?;
            write!(f, "}}")
        }
    }

    impl<const X: usize, const Y: usize, const Z: usize, T: PartialEq> std::cmp::PartialEq
        for StackVolume<X, Y, Z, T>
    {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.bounding_box() == other.bounding_box()
                && !(self.iter().zip(other.iter()).any(|(a, b)| a != b))
        }
    }

    impl<const X: usize, const Y: usize, const Z: usize, T: Copy> StackVolume<X, Y, Z, T> {
        #[inline]
        pub fn filled(item: T) -> Self {
            Self {
                inner: [[[item; Z]; Y]; X],
            }
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
        fn access(&self, idx: Idx) -> Option<&Self::Item> {
            let [x, y, z] = idx.array::<usize>()?;
            self.inner.get(x)?.get(y)?.get(z)
        }

        #[inline]
        fn access_mut(&mut self, idx: Idx) -> Option<&mut Self::Item> {
            let [x, y, z] = idx.array::<usize>()?;
            self.inner.get_mut(x)?.get_mut(y)?.get_mut(z)
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

    impl<const X: usize, const Y: usize, const Z: usize, T, Idx: VolumeIdx> std::ops::IndexMut<Idx>
        for StackVolume<X, Y, Z, T>
    {
        #[inline]
        fn index_mut(&mut self, idx: Idx) -> &mut Self::Output {
            self.get_mut(idx).unwrap()
        }
    }

    impl<const X: usize, const Y: usize, const Z: usize, T> Volume for StackVolume<X, Y, Z, T> {
        type Item = T;

        #[inline]
        fn bounding_box(&self) -> BoundingBox {
            BoundingBox::new([0; 3], [X, Y, Z])
        }
    }
}
