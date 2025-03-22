use crate::{ByteOrder, Cuisiner, CuisinerError};

mod array;
mod number;

macro_rules! impl_identity {
    ($ty:ty) => {
        impl Cuisiner for $ty {
            type Raw<B: ByteOrder> = $ty;

            fn try_from_raw<B: ByteOrder>(raw: Self::Raw<B>) -> Result<Self, CuisinerError> {
                Ok(raw)
            }

            fn try_to_raw<B: ByteOrder>(self) -> Result<Self::Raw<B>, CuisinerError> {
                Ok(self)
            }
        }
    };
}

impl_identity!(());
impl_identity!(u8);
