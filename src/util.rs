use num_traits::{NumCast, PrimInt};

pub(crate) fn boxed_slice<T: Clone>(item: T, len: usize) -> Box<[T]> {
    vec![item; len].into_boxed_slice()
}

#[inline(always)]
pub(crate) fn cast_ivec3<T: NumCast, N: PrimInt>(arr: [N; 3]) -> Option<[T; 3]> {
    let [x, y, z] = arr;

    Some([
        <T as NumCast>::from(x)?,
        <T as NumCast>::from(y)?,
        <T as NumCast>::from(z)?,
    ])
}

#[inline(always)]
pub(crate) fn sum_ivec3<N: std::ops::Add<Output = N> + Copy>(lhs: [N; 3], rhs: [N; 3]) -> [N; 3] {
    [lhs[0] + rhs[0], lhs[1] + rhs[1], lhs[2] + rhs[2]]
}

#[inline(always)]
pub(crate) fn sub_ivec3<N: std::ops::Sub<Output = N> + Copy>(lhs: [N; 3], rhs: [N; 3]) -> [N; 3] {
    [lhs[0] - rhs[0], lhs[1] - rhs[1], lhs[2] - rhs[2]]
}
