use crate::prelude::*;
use crate::util;

pub(crate) mod heap_volume {
    use num_traits::PrimInt;

    use super::*;

    type HeapVolumeStorage<T> = Box<[Box<[Box<[T]>]>]>;

    pub struct HeapVolume<T> {
        inner: HeapVolumeStorage<T>,
        bounds: BoundingBox,
    }

    pub struct HeapVolumeBuilder<T> {
        bounding_box: Option<BoundingBox>,
        filling: T,
    }

    impl<T: Clone> HeapVolumeBuilder<T> {
        pub fn new(filling: T) -> Self {
            Self {
                bounding_box: None,
                filling,
            }
        }

        pub fn with_dimensions(mut self, dimensions: [usize; 3]) -> Self {
            self.bounding_box = Some(BoundingBox::new([0, 0, 0], dimensions));
            self
        }

        pub fn with_position<N: PrimInt>(mut self, position: [N; 3]) -> Self {
            self.bounding_box = Some(BoundingBox::new(
                util::cast_ivec3::<i64, _>(position).unwrap(),
                util::sum_ivec3(
                    util::cast_ivec3::<i64, _>(position).unwrap(),
                    self.bounding_box
                        .map(|b| b.dimensions())
                        .unwrap_or([0i64, 0, 0]),
                ),
            ));

            self
        }

        pub fn with_bounds(self, pos1: [i64; 3], pos2: [i64; 3]) -> Self {
            self.with_bounding_box(BoundingBox::new(pos1, pos2))
        }

        pub fn with_bounding_box(mut self, bounding_box: BoundingBox) -> Self {
            self.bounding_box = Some(bounding_box);
            self
        }

        pub fn build(self) -> HeapVolume<T> {
            HeapVolume::clone_fill(self.bounding_box.unwrap(), self.filling)
        }
    }

    impl_indexing!(T, HeapVolume<T>);
    impl_debug!(T, HeapVolume<T>);

    impl<T: Clone> HeapVolume<T> {
        fn clone_fill(bounds: BoundingBox, item: T) -> Self {
            use util::boxed_slice;

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

    impl<T> Volume for HeapVolume<T> {
        type Item = T;

        #[inline]
        fn get<Idx: VolumeIdx>(&self, idx: Idx) -> Option<&Self::Item> {
            let (x, y, z) = idx.unpack::<usize>()?;

            Some(&self.inner[x][y][z])
        }

        #[inline]
        fn get_mut<Idx: VolumeIdx>(&mut self, idx: Idx) -> Option<&mut Self::Item> {
            let (x, y, z) = idx.unpack::<usize>()?;

            Some(&mut self.inner[x][y][z])
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

        fn get<Idx: VolumeIdx>(&self, idx: Idx) -> Option<&Self::Item> {
            let (x, y, z) = idx.unpack::<usize>()?;

            self.inner.get(x)?.get(y)?.get(z)
        }

        fn get_mut<Idx: VolumeIdx>(&mut self, idx: Idx) -> Option<&mut Self::Item> {
            let (x, y, z) = idx.unpack::<usize>()?;

            self.inner.get_mut(x)?.get_mut(y)?.get_mut(z)
        }

        fn bounding_box(&self) -> BoundingBox {
            BoundingBox::new([0; 3], [X, Y, Z])
        }
    }
}
