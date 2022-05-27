use crate::prelude::*;

#[cfg(test)]
mod util {
    use crate::util;

    #[test]
    fn sum_ivec3() {
        let v1 = [4, 6, 1];
        let v2 = [6, 7, 3];

        assert_eq!(util::sum_ivec3(v1, v2), [10i32, 13, 4]);

        let v1 = [-5, 3, 0];
        let v2 = [-0, 2, 6432];

        assert_eq!(util::sum_ivec3(v1, v2), [-5i32, 5, 6432]);
    }

    #[test]
    fn sub_ivec3() {
        let v1 = [5, 3, -2];
        let v2 = [-4, -9, 7];

        assert_eq!(util::sub_ivec3(v1, v2), [9, 12, -9]);
    }

    #[test]
    fn cast_ivec3() {
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
        let bs = util::boxed_slice(600i32, 10);
        let expected = [600i32; 10].to_vec().into_boxed_slice();

        assert_eq!(bs, expected);
    }
}

#[cfg(test)]
mod heap_volume {
    use crate::prelude::*;

    /// Build an example heap volume of `u8`s with a bounding box of (0, 0, 0) -> (6, 6, 6), filled with `10`.
    /// # Example
    /// ```
    /// use crate::prelude::*;
    ///
    /// let vol = example_heap_volume();
    /// assert_eq!(vol.bounding_box(), BoundingBox::new([0, 0, 0], [6, 6, 6]));
    /// vol.iter().map(|v| assert_eq!(v, &10));
    /// ```
    fn example_heap_volume() -> HeapVolume<u8> {
        HeapVolume::new(10u8, BoundingBox::new([0, 0, 0], [6, 6, 6]))
    }

    #[test]
    fn heap_volume_creation() {
        let vol = example_heap_volume();

        for z in 0..6 {
            for y in 0..6 {
                for x in 0..6 {
                    let idx = [x, y, z];

                    assert!(vol.contains(idx));
                    assert_eq!(vol.get(idx), Some(&10u8));
                }
            }
        }
    }

    #[test]
    fn heap_volume_iteration() {
        let vol = example_heap_volume();
        let mut idx_iterator = vol.bounding_box().into_iter();

        for z in 0..6i64 {
            for y in 0..6i64 {
                for x in 0..6i64 {
                    let idx = [x, y, z];

                    assert_eq!(idx_iterator.next(), Some(idx));
                }
            }
        }

        assert_eq!(idx_iterator.next(), None);

        let vol = HeapVolume::new(10u8, BoundingBox::new([0, 0, 0], [6, 5, 7]));
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
            assert_eq!(val, &10);
        }

        assert_eq!(c, vol.bounding_box().capacity());
    }

    #[test]
    fn heap_volume_access() {
        const N: u8 = 40;

        let mut vol = HeapVolume::new(N, BoundingBox::new([0, 0, 0], [8, 8, 8]));

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
    fn heap_volume_unusual_bounds() {
        let mut vol = HeapVolume::new(10, BoundingBox::new([-9, -9, -9], [-2, -2, -2]));

        assert_eq!(vol.get([0i32, 0, 0]), None);
        assert_eq!(vol.get([3i32, 3, 3]), None);
        assert_eq!(vol.get([-4i32, -4, -4]), Some(&10));

        assert_eq!(vol.swap([-4i32, -4, -4], 50), Some(10));
        assert_eq!(vol.get([-4i32, -4, -4]), Some(&50));

        let mut idx_iterator = vol.iter_indices();
        let mut c = 0;
        for z in -9..-2i64 {
            for y in -9..-2i64 {
                for x in -9..-2i64 {
                    let idx = idx_iterator.next();
                    c += 1;

                    assert_eq!(idx, Some([x, y, z]));
                }
            }
        }

        assert_eq!(idx_iterator.next(), None);
        assert_eq!(c, vol.bounding_box().capacity());

        assert_eq!(vol.get([-9, -9, -9]), Some(&10));
        assert_eq!(vol.get([-2, -2, -2]), None);
    }

    #[test]
    fn heap_volume_insertion() {
        let mut vol1 = HeapVolume::new(10, BoundingBox::new_origin([16i32, 16, 16]));
        let vol2 = HeapVolume::new(20, BoundingBox::new([4i32, 4, 4], [10i32, 10, 10]));

        vol1.insert([0i32, 0, 0], &vol2).unwrap();

        for n in 0..4i32 {
            assert_eq!(vol1[[n, n, n]], 10);
        }

        for n in 4..10i32 {
            assert_eq!(vol1[[n, n, n]], 20);
        }

        for n in 10..16i32 {
            assert_eq!(vol1[[n, n, n]], 10);
        }

        assert_eq!(vol1[[4i32, 4, 4]], 20);
        assert_eq!(vol1[[9i32, 4, 4]], 20);
        assert_eq!(vol1[[4i32, 9, 4]], 20);
        assert_eq!(vol1[[4i32, 4, 9]], 20);

        assert_eq!(vol1[[4i32, 9, 9]], 20);
        assert_eq!(vol1[[9i32, 9, 4]], 20);
        assert_eq!(vol1[[9i32, 9, 9]], 20);
        assert_eq!(vol1[[9i32, 4, 9]], 20);
    }
}

#[cfg(feature = "nalgebra")]
#[test]
fn nalgebra_bounding_box_support() {
    extern crate nalgebra as na;

    let v1 = na::vector![0, 0, 0];
    let v2 = na::vector![10, 10, 10];

    let vol = HeapVolume::new(10, v1..v2);

    assert_eq!(
        vol.bounding_box(),
        BoundingBox::new([0, 0, 0], [10, 10, 10])
    );
}
