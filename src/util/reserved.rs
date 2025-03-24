use crate::{ByteOrder, Cuisiner, CuisinerError};

/// Sequence of reserved bytes. Doesn't validate when deserialising, and will serialise to `0`.
#[derive(Clone, Debug)]
pub struct Reserved<const N: usize>;

impl<const N: usize> Cuisiner for Reserved<N> {
    type Raw<B: ByteOrder> = [u8; N];

    fn try_from_raw<B: ByteOrder>(_raw: Self::Raw<B>) -> Result<Self, CuisinerError> {
        Ok(Self)
    }

    fn try_to_raw<B: ByteOrder>(self) -> Result<Self::Raw<B>, CuisinerError> {
        Ok([0; N])
    }
}
