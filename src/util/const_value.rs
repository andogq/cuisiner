use zerocopy::{I16, I32, I64, I128, Isize, U16, U32, U64, U128, Usize};

use crate::{ByteOrder, Cuisiner, CuisinerError};

macro_rules! impl_const_value {
    ($ident:ident, |$value:ident: $ty:ty| $value_to_raw:expr, |$raw:ident: $raw_ty:ty| $raw_to_value:expr) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $ident<const N: $ty>;

        impl<const N: $ty> Cuisiner for $ident<N> {
            type Raw<B: ByteOrder> = $raw_ty;

            fn try_from_raw<B: ByteOrder>($raw: Self::Raw<B>) -> Result<Self, CuisinerError> {
                let raw = $raw_to_value;
                if raw != N {
                    return Err(CuisinerError::Validation(format!(
                        "expected {N}, found {raw}"
                    )));
                }

                Ok(Self)
            }

            fn try_to_raw<B: ByteOrder>(self) -> Result<Self::Raw<B>, CuisinerError> {
                let $value = N;
                Ok($value_to_raw)
            }
        }
    };
}

impl_const_value! {
    ConstU8,
    |value: u8| value,
    |raw: u8| raw
}
impl_const_value! {
    ConstU16,
    |value: u16| U16::<B>::new(value),
    |raw: U16<B>| raw.get()
}
impl_const_value! {
    ConstU32,
    |value: u32| U32::<B>::new(value),
    |raw: U32<B>| raw.get()
}
impl_const_value! {
    ConstU64,
    |value: u64| U64::<B>::new(value),
    |raw: U64<B>| raw.get()
}
impl_const_value! {
    ConstU128,
    |value: u128| U128::<B>::new(value),
    |raw: U128<B>| raw.get()
}
impl_const_value! {
    ConstUsize,
    |value: usize| Usize::<B>::new(value),
    |raw: Usize<B>| raw.get()
}
impl_const_value! {
    ConstI8,
    |value: i8| value,
    |raw: i8| raw
}
impl_const_value! {
    ConstI16,
    |value: i16| I16::<B>::new(value),
    |raw: I16<B>| raw.get()
}
impl_const_value! {
    ConstI32,
    |value: i32| I32::<B>::new(value),
    |raw: I32<B>| raw.get()
}
impl_const_value! {
    ConstI64,
    |value: i64| I64::<B>::new(value),
    |raw: I64<B>| raw.get()
}
impl_const_value! {
    ConstI128,
    |value: i128| I128::<B>::new(value),
    |raw: I128<B>| raw.get()
}
impl_const_value! {
    ConstIsize,
    |value: isize| Isize::<B>::new(value),
    |raw: Isize<B>| raw.get()
}
