extern crate thiserror as te;

#[macro_use]
mod macros;

#[cfg(test)]
mod tests;

mod impls;
pub mod prelude;
pub mod traits;
pub mod types;
mod util;

#[cfg(feature = "nalgebra")]
pub use nalgebra_support::*;

#[cfg(feature = "nalgebra")]
mod nalgebra_support {
    extern crate nalgebra as na;

    use crate::prelude::*;
    use num_traits::{NumCast, PrimInt};

    impl<N: PrimInt> VolumeIdx for na::Vector3<N> {
        #[inline(always)]
        fn unpack<T: NumCast>(self) -> Option<(T, T, T)> {
            Some((
                <T as NumCast>::from(self[0])?,
                <T as NumCast>::from(self[1])?,
                <T as NumCast>::from(self[2])?,
            ))
        }
    }

    impl_boundingbox_from_na_vec_range!(i8, u8, i16, u16, i32, u32, i64);

    impl From<BoundingBox> for std::ops::Range<na::Vector3<i64>> {
        #[inline(always)]
        fn from(bb: BoundingBox) -> Self {
            na::Vector3::<i64>::from(bb.min())..na::Vector3::<i64>::from(bb.max())
        }
    }
}

#[cfg(feature = "glam")]
pub use glam_support::*;

#[cfg(feature = "glam")]
mod glam_support {
    extern crate glam;

    use crate::prelude::*;
    use num_traits::NumCast;

    impl VolumeIdx for glam::IVec3 {
        #[inline(always)]
        fn unpack<T: NumCast>(self) -> Option<(T, T, T)> {
            Some((
                <T as NumCast>::from(self[0])?,
                <T as NumCast>::from(self[1])?,
                <T as NumCast>::from(self[2])?,
            ))
        }
    }

    impl VolumeIdx for glam::UVec3 {
        #[inline(always)]
        fn unpack<T: NumCast>(self) -> Option<(T, T, T)> {
            Some((
                <T as NumCast>::from(self[0])?,
                <T as NumCast>::from(self[1])?,
                <T as NumCast>::from(self[2])?,
            ))
        }
    }

    impl_boundingbox_from_glam_vec_range!(glam::IVec3, glam::UVec3);
}
