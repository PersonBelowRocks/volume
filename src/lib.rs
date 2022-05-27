extern crate thiserror as te;

#[macro_use]
mod macros;

#[cfg(test)]
mod tests;

mod impls;
pub mod prelude;
mod spaces;
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
        #[inline]
        fn array<T: NumCast>(self) -> Option<[T; 3]> {
            Some([
                <T as NumCast>::from(self[0])?,
                <T as NumCast>::from(self[1])?,
                <T as NumCast>::from(self[2])?,
            ])
        }

        #[inline]
        fn from_xyz<T: PrimInt>(x: T, y: T, z: T) -> Self {
            Self::new(
                <N as NumCast>::from(x).unwrap(),
                <N as NumCast>::from(y).unwrap(),
                <N as NumCast>::from(z).unwrap(),
            )
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
    use num_traits::{NumCast, PrimInt};

    impl VolumeIdx for glam::IVec3 {
        #[inline]
        fn array<T: NumCast>(self) -> Option<[T; 3]> {
            Some([
                <T as NumCast>::from(self[0])?,
                <T as NumCast>::from(self[1])?,
                <T as NumCast>::from(self[2])?,
            ])
        }

        #[inline]
        fn from_xyz<N: PrimInt>(x: N, y: N, z: N) -> Self {
            Self::new(
                <i32 as NumCast>::from(x).unwrap(),
                <i32 as NumCast>::from(y).unwrap(),
                <i32 as NumCast>::from(z).unwrap(),
            )
        }
    }

    impl VolumeIdx for glam::UVec3 {
        #[inline]
        fn array<T: NumCast>(self) -> Option<[T; 3]> {
            Some([
                <T as NumCast>::from(self[0])?,
                <T as NumCast>::from(self[1])?,
                <T as NumCast>::from(self[2])?,
            ])
        }

        #[inline]
        fn from_xyz<T: PrimInt>(x: T, y: T, z: T) -> Self {
            Self::new(
                <u32 as NumCast>::from(x).unwrap(),
                <u32 as NumCast>::from(y).unwrap(),
                <u32 as NumCast>::from(z).unwrap(),
            )
        }
    }

    impl_boundingbox_from_glam_vec_range!(glam::IVec3, glam::UVec3);
}
