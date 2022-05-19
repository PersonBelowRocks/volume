#[test]
fn sum_ivec3() {
    use crate::util;

    let v1 = [4, 6, 1];
    let v2 = [6, 7, 3];

    assert_eq!(util::sum_ivec3(v1, v2), [10i32, 13, 4]);

    let v1 = [-5, 3, 0];
    let v2 = [-0, 2, 6432];

    assert_eq!(util::sum_ivec3(v1, v2), [-5i32, 5, 6432]);
}

#[test]
fn sub_ivec3() {
    use crate::util;

    let v1 = [5, 3, -2];
    let v2 = [-4, -9, 7];

    assert_eq!(util::sub_ivec3(v1, v2), [9, 12, -9]);
}

#[test]
fn cast_ivec3() {
    use crate::util;

    let v = [60i32, 43, 90];
    assert_eq!(util::cast_ivec3(v), Some([60u8, 43, 90]));

    let v = [400, 60, 30]; // v[0] doesn't fit in a u8 so this fails
    assert_eq!(util::cast_ivec3::<u8, _>(v), None);

    let v = [-2, 60, 10]; // v[0] is negative so it can't be represented as a u8
    assert_eq!(util::cast_ivec3::<u8, _>(v), None);

    let v = [u32::MAX, u32::MIN, 7];
    assert_eq!(
        util::cast_ivec3(v),
        Some([u32::MAX as i64, u32::MIN as i64, 7])
    );
    assert_eq!(util::cast_ivec3::<i32, _>(v), None); // u32::MAX doesn't go in a i32

    let v = [i64::MAX, 7, 3];
    assert_eq!(util::cast_ivec3::<i32, _>(v), None); // i64::MAX is too big for a i32
}

#[test]
fn boxed_slice() {
    use crate::util;

    let bs = util::boxed_slice(600i32, 10);
    let expected = [600i32; 10].to_vec().into_boxed_slice();

    assert_eq!(bs, expected);
}

#[test]
fn heap_volume_creation() {
    use crate::prelude::*;

    let vol = HeapVolume::filled([3, 3, 3], 80u8);

    for z in 0..3 {
        for y in 0..3 {
            for x in 0..3 {
                let idx = [x, y, z];

                assert!(vol.contains(idx));
                assert_eq!(vol.get(idx), Some(&80u8));
            }
        }
    }
}

#[test]
fn heap_volume_iteration() {
    use crate::prelude::*;

    let vol = HeapVolume::filled([3, 3, 3], 80u8);
    let mut idx_iterator = vol.bounding_box().into_iter();

    for z in 0..3i64 {
        for y in 0..3i64 {
            for x in 0..3i64 {
                let idx = [x, y, z];

                assert_eq!(idx_iterator.next(), Some(idx));
            }
        }
    }

    assert_eq!(idx_iterator.next(), None);

    let vol = HeapVolume::filled([6, 5, 7], 40u8);
    let mut idx_iterator = vol.bounding_box().into_iter();

    let mut c = 0;
    #[allow(clippy::while_let_on_iterator)] // bit more explicit this way
    while let Some(_) = idx_iterator.next() {
        c += 1;
    }

    assert_eq!(c, 6 * 5 * 7);
    assert_eq!(c, vol.bounding_box().capacity());

    let mut vol_iterator = vol.iter();
    let mut c = 0;
    #[allow(clippy::while_let_on_iterator)] // bit more explicit this way
    while let Some(val) = vol_iterator.next() {
        c += 1;
        assert_eq!(val, &40);
    }

    assert_eq!(c, vol.bounding_box().capacity());
}

#[test]
fn heap_volume_access() {
    use crate::prelude::*;

    const N: u8 = 40;

    let mut vol = HeapVolume::filled([8, 8, 8], N);
    assert_eq!(vol.get([5i32, 5, 5]), Some(&N));
    assert_eq!(vol.swap([5i32, 5, 5], 80), Some(N));
    assert_eq!(vol.get([5i32, 5, 5]), Some(&80));

    let slot = vol.get_mut([5i32, 5, 5]).unwrap();
    assert_eq!(slot, &mut 80);

    *slot += 10;
    assert_eq!(vol.get([5i32, 5, 5]), Some(&90));

    assert_eq!(vol[[5i32, 5, 5]], 90u8);
    vol[[5i32, 5, 5]] = 10;
    assert_eq!(vol[[5i32, 5, 5]], 10);
}

#[test]
fn heap_volume_bounding_box() {}
