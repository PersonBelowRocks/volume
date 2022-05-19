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
