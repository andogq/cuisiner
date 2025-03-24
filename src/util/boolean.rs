use std::ops::Deref;

use crate::{ByteOrder, Cuisiner, CuisinerError};

/// A boolean value represented with some number of bytes, where non-zero is true, and zero is
/// false.
#[derive(Clone, Copy, Debug)]
pub struct ByteBoolean<const N: usize = 1>(bool);

impl<const N: usize> Cuisiner for ByteBoolean<N> {
    type Raw<B: ByteOrder> = [u8; N];

    fn try_from_raw<B: ByteOrder>(raw: Self::Raw<B>) -> Result<Self, CuisinerError> {
        let all_zero = raw.into_iter().all(|n| n == 0);
        Ok(Self(!all_zero))
    }

    fn try_to_raw<B: ByteOrder>(self) -> Result<Self::Raw<B>, CuisinerError> {
        Ok([if *self { 0xff } else { 0x00 }; N])
    }
}

impl<const N: usize> Deref for ByteBoolean<N> {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
