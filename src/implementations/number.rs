use zerocopy::ByteOrder;

/// Implement [`crate::Cuisiner`] for [`core::num::NonZero`] numbers. Assumes that
/// [`crate::Cuisiner`] is already implemented for the type.
macro_rules! impl_non_zero_number {
    ($ty:ty: $raw:ty) => {
        impl $crate::Cuisiner for Option<core::num::NonZero<$ty>> {
            type Raw<B: ByteOrder> = $raw;

            fn try_from_raw<B: ByteOrder>(
                raw: Self::Raw<B>,
            ) -> Result<Self, $crate::CuisinerError> {
                Ok(
                    core::num::NonZero::try_from(<$ty as $crate::Cuisiner>::try_from_raw::<B>(
                        raw,
                    )?)
                    .ok(),
                )
            }

            fn try_to_raw<B: ByteOrder>(self) -> Result<Self::Raw<B>, $crate::CuisinerError> {
                <$ty as $crate::Cuisiner>::try_to_raw::<B>(self.map(|n| n.get()).unwrap_or(0))
            }
        }

        impl $crate::Cuisiner for core::num::NonZero<$ty> {
            type Raw<B: ByteOrder> = $raw;

            fn try_from_raw<B: ByteOrder>(
                raw: Self::Raw<B>,
            ) -> Result<Self, $crate::CuisinerError> {
                <Option<core::num::NonZero<$ty>> as $crate::Cuisiner>::try_from_raw::<B>(raw)?
                    .ok_or($crate::CuisinerError::Zero)
            }

            fn try_to_raw<B: ByteOrder>(self) -> Result<Self::Raw<B>, $crate::CuisinerError> {
                <$ty as $crate::Cuisiner>::try_to_raw::<B>(self.get())
            }
        }
    };
}

/// Implement [`crate::Cuisiner`] for a [`zerocopy`] type.
macro_rules! impl_zerocopy {
    (base $ty:ty: $raw:ty) => {
        impl $crate::Cuisiner for $ty {
            type Raw<B: ByteOrder> = $raw;

            fn try_from_raw<B: ByteOrder>(raw: Self::Raw::<B>) -> Result<Self, $crate::CuisinerError> {
                Ok(raw.get())
            }

            fn try_to_raw<B: ByteOrder>(self) -> Result<Self::Raw::<B>, $crate::CuisinerError> {
                Ok(Self::Raw::<B>::from(self))
            }
        }
    };

    ($ty:ty: $raw:ty) => {
        impl_zerocopy!(base $ty: $raw);
        impl_non_zero_number!($ty: $raw);
    };
}

// impl_zerocopy!(base f32: zerocopy::byteorder::F32 );
// impl_zerocopy!(base f64: zerocopy::byteorder::F64 );
impl_zerocopy!(i16: zerocopy::byteorder::I16<B>);
impl_zerocopy!(i32: zerocopy::byteorder::I32<B>);
impl_zerocopy!(i64: zerocopy::byteorder::I64<B>);
impl_zerocopy!(i128: zerocopy::byteorder::I128<B>);
impl_zerocopy!(u16: zerocopy::byteorder::U16<B>);
impl_zerocopy!(u32: zerocopy::byteorder::U32<B>);
impl_zerocopy!(u64: zerocopy::byteorder::U64<B>);
impl_zerocopy!(u128: zerocopy::byteorder::U128<B>);
