mod number;

/// Implement [`crate::Cuisiner`] for a [`zerocopy`] type.
#[macro_export]
macro_rules! impl_zerocopy {
    ($ty:ty: $raw:ty) => {
        impl $crate::Cuisiner for $ty {
            type Raw<B: $crate::ByteOrder> = $raw;

            fn try_from_raw<B: $crate::ByteOrder>(
                raw: Self::Raw<B>,
            ) -> Result<Self, $crate::CuisinerError> {
                Ok(raw.get())
            }

            fn try_to_raw<B: $crate::ByteOrder>(
                self,
            ) -> Result<Self::Raw<B>, $crate::CuisinerError> {
                Ok(Self::Raw::<B>::from(self))
            }
        }
    };
}
