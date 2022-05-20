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

    #[derive(thiserror::Error, Debug)]
    #[error("could not convert vector range to valid bounding box")]
    pub struct VecRangeConversionError;

    impl<N: PrimInt> TryFrom<std::ops::Range<na::Vector3<N>>> for BoundingBox {
        type Error = VecRangeConversionError;

        #[inline(always)]
        fn try_from(range: std::ops::Range<na::Vector3<N>>) -> Result<Self, Self::Error> {
            let pos1 = range.start.to_arr::<i64>().ok_or(VecRangeConversionError)?;
            let pos2 = range.end.to_arr::<i64>().ok_or(VecRangeConversionError)?;

            Ok(BoundingBox::new(pos1, pos2))
        }
    }

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
}
