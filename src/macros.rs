macro_rules! impl_indexing {
    ($param:tt, $tgt:ty) => {
        impl<$param, Idx: crate::traits::VolumeIdx> std::ops::Index<Idx> for $tgt
        where
            $tgt: crate::traits::Volume,
        {
            type Output = <$tgt as crate::traits::Volume>::Item;

            #[inline(always)]
            fn index(&self, idx: Idx) -> &Self::Output {
                self.get(idx).unwrap()
            }
        }

        impl<$param, Idx: crate::traits::VolumeIdx> std::ops::IndexMut<Idx> for $tgt
        where
            $tgt: crate::traits::Volume,
        {
            #[inline(always)]
            fn index_mut(&mut self, idx: Idx) -> &mut Self::Output {
                self.get_mut(idx).unwrap()
            }
        }
    };
}

macro_rules! impl_debug {
    ($param:tt, $tgt:ty) => {
        impl<$param> std::fmt::Debug for $tgt
        where
            $tgt: crate::traits::Volume,
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let capacity = {
                    let bounds = self.bounding_box();
                    let [x, y, z] = crate::util::sub_ivec3(bounds.max(), bounds.min());
                    x * y * z
                };

                write!(f, "{} {{", stringify!($tgt))?;
                write!(f, "    bounds: {},", self.bounding_box())?;
                write!(f, "    capacity: {}", capacity)?;
                write!(f, "}}")
            }
        }
    };
}

#[cfg(any(feature = "nalgebra", feature = "glam"))]
macro_rules! impl_boundingbox_from_volumeidx_range {
    ($t:ty) => {
        impl From<std::ops::Range<$t>> for crate::prelude::BoundingBox
        where
            $t: crate::prelude::VolumeIdx,
        {
            #[inline(always)]
            fn from(range: std::ops::Range<$t>) -> Self {
                let pos1 = range.start.to_arr::<i64>().unwrap();
                let pos2 = range.end.to_arr::<i64>().unwrap();

                crate::prelude::BoundingBox::new(pos1, pos2)
            }
        }
    };
}

#[cfg(feature = "nalgebra")]
/// Implement [`From<std::ops::Range<nalgebra::Vector3<T>>>`] for [`crate::prelude::BoundingBox`]. T should implement [`num_traits::PrimInt`]
macro_rules! impl_boundingbox_from_na_vec_range {
    ($t:ty) => {
        impl_boundingbox_from_volumeidx_range!(nalgebra::Vector3<$t>);
    };

    ($t:ty, $($ts:ty),+) => {
        impl_boundingbox_from_na_vec_range!($t);
        impl_boundingbox_from_na_vec_range!($($ts),+);
    };
}

#[cfg(feature = "glam")]
macro_rules! impl_boundingbox_from_glam_vec_range {
    ($t:ty) => {
        impl_boundingbox_from_volumeidx_range!($t);
    };

    ($t:ty, $($ts:ty),+) => {
        impl_boundingbox_from_glam_vec_range!($t);
        impl_boundingbox_from_glam_vec_range!($($ts),+);
    };
}
