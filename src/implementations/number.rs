macro_rules! impl_number {
    (base $ty:ty: $raw:ty) => {
        $crate::impl_zerocopy!($ty: $raw);
    };

    // Implement [`crate::Cuisiner`] for [`core::num::NonZero`] numbers. Assumes that
    // [`crate::Cuisiner`] is already implemented for the type.
    (non_zero $ty:ty: $raw:ty) => {
        impl $crate::Cuisiner for Option<core::num::NonZero<$ty>> {
            type Raw<B: $crate::ByteOrder> = $raw;

            fn try_from_raw<B: $crate::ByteOrder>(
                raw: Self::Raw<B>,
            ) -> Result<Self, $crate::CuisinerError> {
                Ok(
                    core::num::NonZero::try_from(<$ty as $crate::Cuisiner>::try_from_raw::<B>(
                        raw,
                    )?)
                    .ok(),
                )
            }

            fn try_to_raw<B: $crate::ByteOrder>(self) -> Result<Self::Raw<B>, $crate::CuisinerError> {
                <$ty as $crate::Cuisiner>::try_to_raw::<B>(self.map(|n| n.get()).unwrap_or(0))
            }
        }

        impl $crate::Cuisiner for core::num::NonZero<$ty> {
            type Raw<B: $crate::ByteOrder> = $raw;

            fn try_from_raw<B: $crate::ByteOrder>(
                raw: Self::Raw<B>,
            ) -> Result<Self, $crate::CuisinerError> {
                <Option<core::num::NonZero<$ty>> as $crate::Cuisiner>::try_from_raw::<B>(raw)?
                    .ok_or($crate::CuisinerError::Zero)
            }

            fn try_to_raw<B: $crate::ByteOrder>(self) -> Result<Self::Raw<B>, $crate::CuisinerError> {
                <$ty as $crate::Cuisiner>::try_to_raw::<B>(self.get())
            }
        }
    };

    ($ty:ty: $raw:ty) => {
        impl_number!(base $ty: $raw);
        impl_number!(non_zero $ty: $raw);
    };
}

impl_number!(base f32: zerocopy::byteorder::F32<B>);
impl_number!(base f64: zerocopy::byteorder::F64<B>);
impl_number!(i16: zerocopy::byteorder::I16<B>);
impl_number!(i32: zerocopy::byteorder::I32<B>);
impl_number!(i64: zerocopy::byteorder::I64<B>);
impl_number!(i128: zerocopy::byteorder::I128<B>);
impl_number!(u16: zerocopy::byteorder::U16<B>);
impl_number!(u32: zerocopy::byteorder::U32<B>);
impl_number!(u64: zerocopy::byteorder::U64<B>);
impl_number!(u128: zerocopy::byteorder::U128<B>);

#[cfg(test)]
mod test {
    use crate::{BigEndian, Cuisiner, LittleEndian};
    use proptest::prelude::*;

    trait TestableType: Clone + Cuisiner + std::fmt::Debug + PartialEq {
        fn be_bytes(&self) -> Vec<u8>;
        fn le_bytes(&self) -> Vec<u8>;

        fn test(&self) {
            // Test BE
            let be_bytes = <Self as Cuisiner>::to_bytes::<BigEndian>(self.clone()).unwrap();
            let be_native_bytes = self.be_bytes();
            let be_n = <Self as Cuisiner>::from_bytes::<BigEndian>(&be_bytes).unwrap();

            assert_eq!(be_bytes.as_slice(), be_native_bytes);
            assert_eq!(&be_n, self);

            // Test LE
            let le_bytes = <Self as Cuisiner>::to_bytes::<LittleEndian>(self.clone()).unwrap();
            let le_native_bytes = self.le_bytes();
            let le_n = <Self as Cuisiner>::from_bytes::<LittleEndian>(&le_bytes).unwrap();

            assert_eq!(le_bytes.as_slice(), le_native_bytes);
            assert_eq!(&le_n, self);
        }
    }

    macro_rules! impl_testable_type {
        ($ty:ty) => {
            impl TestableType for $ty {
                fn be_bytes(&self) -> Vec<u8> {
                    self.to_be_bytes().to_vec()
                }
                fn le_bytes(&self) -> Vec<u8> {
                    self.to_le_bytes().to_vec()
                }
            }
        };
    }

    impl_testable_type!(f32);
    impl_testable_type!(f64);
    impl_testable_type!(i16);
    impl_testable_type!(i32);
    impl_testable_type!(i64);
    impl_testable_type!(i128);
    impl_testable_type!(u16);
    impl_testable_type!(u32);
    impl_testable_type!(u64);
    impl_testable_type!(u128);

    proptest! {
        #[test]
        fn valid_f32(n in any::<f32>()) {
            n.test()
        }
        #[test]
        fn valid_f64(n in any::<f64>()) {
            n.test()
        }
        #[test]
        fn valid_i16(n in any::<i16>()) {
            n.test()
        }
        #[test]
        fn valid_i32(n in any::<i32>()) {
            n.test()
        }
        #[test]
        fn valid_i64(n in any::<i64>()) {
            n.test()
        }
        #[test]
        fn valid_i128(n in any::<i128>()) {
            n.test()
        }
        #[test]
        fn valid_u16(n in any::<u16>()) {
            n.test()
        }
        #[test]
        fn valid_u32(n in any::<u32>()) {
            n.test()
        }
        #[test]
        fn valid_u64(n in any::<u64>()) {
            n.test()
        }
        #[test]
        fn valid_u128(n in any::<u128>()) {
            n.test()
        }
    }
}
