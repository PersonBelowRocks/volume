use te::Error;

use crate::BoundingBox;

#[derive(Copy, Clone, Debug, Error)]
#[error("expected bounds of at least {expected:?}, got {provided:?}")]
pub struct OversizedBounds {
    pub(crate) provided: BoundingBox,
    pub(crate) expected: Option<BoundingBox>,
}
