use crate::prelude::VolumeIdx;

pub enum Space<Idx: VolumeIdx> {
    Localspace(Idx),
    Worldspace(Idx),
}
