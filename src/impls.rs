use crate::{BoundingBox, Idx, Volume};

fn idx_usize(idx: Idx) -> Option<[usize; 3]> {
    Some([
        idx.x.try_into().ok()?,
        idx.y.try_into().ok()?,
        idx.z.try_into().ok()?,
    ])
}

impl<T: Copy, const X: usize, const Y: usize, const Z: usize> Volume for [[[T; Z]; Y]; X] {
    type Input = T;
    type Output = Option<T>;

    fn set(&mut self, idx: crate::Idx, item: Self::Input) -> bool {
        if let Some([x, y, z]) = idx_usize(idx) {
            self.get_mut(x).map(|i| {
                i.get_mut(y)
                    .map(|ii| ii.get_mut(z).map(|slot| *slot = item))
            });
            true
        } else {
            false
        }
    }

    fn get(&self, idx: crate::Idx) -> Self::Output {
        let [x, y, z] = idx_usize(idx)?;
        <[[[T; Z]; Y]]>::get(self, x)?.get(y)?.get(z).cloned()
    }

    fn bounding_box(&self) -> BoundingBox {
        (na::vector![0, 0, 0]..na::vector![X as i64, Y as i64, Z as i64]).into()
    }
}
