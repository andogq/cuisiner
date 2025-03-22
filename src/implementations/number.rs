/// Implement [`crate::Cuisiner`] for [`core::num::NonZero`] numbers. Assumes that
/// [`crate::Cuisiner`] is already implemented for the type.
macro_rules! impl_non_zero_number {
    ($ty:ty: $raw:ty => $byte_order:ty) => {
        impl $crate::Cuisiner<$byte_order> for Option<core::num::NonZero<$ty>> {
            type Raw = $raw;

            fn try_from_raw(raw: Self::Raw) -> Result<Self, $crate::CuisinerError> {
                Ok(core::num::NonZero::try_from(
                    <$ty as $crate::Cuisiner<$byte_order>>::try_from_raw(raw)?,
                )
                .ok())
            }

            fn try_to_raw(self) -> Result<$raw, $crate::CuisinerError> {
                <$ty as $crate::Cuisiner<$byte_order>>::try_to_raw(
                    self.map(|n| n.get()).unwrap_or(0),
                )
            }
        }

        impl $crate::Cuisiner<$byte_order> for core::num::NonZero<$ty> {
            type Raw = $raw;

            fn try_from_raw(raw: Self::Raw) -> Result<Self, $crate::CuisinerError> {
                <Option<core::num::NonZero<$ty>> as $crate::Cuisiner<$byte_order>>::try_from_raw(
                    raw,
                )?
                .ok_or($crate::CuisinerError::Zero)
            }

            fn try_to_raw(self) -> Result<Self::Raw, $crate::CuisinerError> {
                <$ty as $crate::Cuisiner<$byte_order>>::try_to_raw(self.get())
            }
        }
    };
}

/// Implement [`crate::Cuisiner`] for a [`zerocopy`] type.
macro_rules! impl_zerocopy {
    (base $ty:ty: $raw:ty => $byte_order:ty) => {
        impl $crate::Cuisiner<$byte_order> for $ty {
            type Raw = $raw;

            fn try_from_raw(raw: $raw) -> Result<Self, $crate::CuisinerError> {
                Ok(raw.get())
            }

            fn try_to_raw(self) -> Result<$raw, $crate::CuisinerError> {
                Ok(<$raw>::from(self))
            }
        }
    };

    ($ty:ty: $raw:ty => $byte_order:ty) => {
        impl_zerocopy!(base $ty: $raw => $byte_order);
        impl_non_zero_number!($ty: $raw => $byte_order);
    };

    ($byte_order:ty) => {
        impl_zerocopy!(base f32: zerocopy::byteorder::F32::<$byte_order> => $byte_order);
        impl_zerocopy!(base f64: zerocopy::byteorder::F64::<$byte_order> => $byte_order);
        impl_zerocopy!(i16: zerocopy::byteorder::I16::<$byte_order> => $byte_order);
        impl_zerocopy!(i32: zerocopy::byteorder::I32::<$byte_order> => $byte_order);
        impl_zerocopy!(i64: zerocopy::byteorder::I64::<$byte_order> => $byte_order);
        impl_zerocopy!(i128: zerocopy::byteorder::I128::<$byte_order> => $byte_order);
        impl_zerocopy!(u16: zerocopy::byteorder::U16::<$byte_order> => $byte_order);
        impl_zerocopy!(u32: zerocopy::byteorder::U32::<$byte_order> => $byte_order);
        impl_zerocopy!(u64: zerocopy::byteorder::U64::<$byte_order> => $byte_order);
        impl_zerocopy!(u128: zerocopy::byteorder::U128::<$byte_order> => $byte_order);
    };
}

impl_zerocopy!(zerocopy::byteorder::BigEndian);
// impl_zerocopy!(zerocopy::little_endian);
