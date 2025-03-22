mod implementations;

use thiserror::Error;
use zerocopy::{ByteOrder, FromBytes, Immutable, IntoBytes};

pub use cuisiner_derive::Cuisiner;
pub use zerocopy::{self, BigEndian, LittleEndian, NativeEndian, NetworkEndian};

#[derive(Debug, Error)]
pub enum CuisinerError {
    #[error("zero encountered in an unexpected location")]
    Zero,

    #[error("incorrect buffer size for serialising or deserialising")]
    SizeError,
}

pub trait Cuisiner<B>: Sized
where
    B: ByteOrder,
{
    type Raw: FromBytes + IntoBytes + Immutable;

    /// Attempt to convert this value from a raw value.
    fn try_from_raw(raw: Self::Raw) -> Result<Self, CuisinerError>;

    /// Attempt to convert this value into the raw value.
    fn try_to_raw(self) -> Result<Self::Raw, CuisinerError>;

    /// Read the provided bytes and attempt to parse out the type.
    fn from_bytes(bytes: &[u8]) -> Result<Self, CuisinerError> {
        let raw = Self::Raw::read_from_bytes(bytes).map_err(|_| CuisinerError::SizeError)?;
        Self::try_from_raw(raw)
    }

    /// Convert a value to it's raw representation.
    fn to_bytes(self) -> Result<Vec<u8>, CuisinerError> {
        Ok(self.try_to_raw()?.as_bytes().to_vec())
    }
}

mod sample {
    use super::*;

    // #[derive(Cuisiner)]
    // #[cuisiner(big_endian)]
    struct MyStruct {
        a_field: u32,
        another: i64,
    }

    #[derive(zerocopy::FromBytes, zerocopy::IntoBytes, zerocopy::Immutable)]
    struct MyStructRaw {
        a_field: <u32 as Cuisiner<zerocopy::BigEndian>>::Raw,
        another: <i64 as Cuisiner<zerocopy::BigEndian>>::Raw,
    }

    impl Cuisiner<zerocopy::BigEndian> for MyStruct {
        type Raw = MyStructRaw;

        fn try_from_raw(raw: MyStructRaw) -> Result<Self, CuisinerError> {
            Ok(Self {
                a_field: <_ as Cuisiner<zerocopy::BigEndian>>::try_from_raw(raw.a_field)?,
                another: <_ as Cuisiner<zerocopy::BigEndian>>::try_from_raw(raw.another)?,
            })
        }

        fn try_to_raw(self) -> Result<MyStructRaw, CuisinerError> {
            Ok(MyStructRaw {
                a_field: <_ as Cuisiner<zerocopy::BigEndian>>::try_to_raw(self.a_field)?,
                another: <_ as Cuisiner<zerocopy::BigEndian>>::try_to_raw(self.another)?,
            })
        }
    }
}
