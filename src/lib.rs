#![doc = include_str!("../README.md")]

mod implementations;
mod util;

use thiserror::Error;
use zerocopy::{FromBytes, Immutable, IntoBytes, Unaligned};

pub use assert_layout::assert_layout;
pub use cuisiner_derive::Cuisiner;
pub use zerocopy::{self, BigEndian, ByteOrder, LittleEndian, NativeEndian, NetworkEndian};

pub use self::util::*;

pub trait Cuisiner: Sized {
    type Raw<B: ByteOrder>: FromBytes + IntoBytes + Immutable + Unaligned;

    /// Attempt to convert this value from a raw value.
    fn try_from_raw<B: ByteOrder>(raw: Self::Raw<B>) -> Result<Self, CuisinerError>;

    /// Attempt to convert this value into the raw value.
    fn try_to_raw<B: ByteOrder>(self) -> Result<Self::Raw<B>, CuisinerError>;

    /// Read the provided bytes and attempt to parse out the type.
    fn from_bytes<B: ByteOrder>(bytes: &[u8]) -> Result<Self, CuisinerError> {
        let (raw, _) =
            Self::Raw::<B>::read_from_prefix(bytes).map_err(|_| CuisinerError::SizeError {
                required: std::mem::size_of::<Self::Raw<B>>(),
                found: bytes.len(),
            })?;
        Self::try_from_raw(raw)
    }

    /// Convert a value to it's raw representation.
    fn to_bytes<B: ByteOrder>(self) -> Result<Vec<u8>, CuisinerError> {
        Ok(self.try_to_raw::<B>()?.as_bytes().to_vec())
    }
}

#[derive(Debug, Error)]
pub enum CuisinerError {
    #[error("zero encountered in an unexpected location")]
    Zero,

    #[error(
        "incorrect buffer size for serialising or deserialising (required {required}, found {found})"
    )]
    SizeError { required: usize, found: usize },

    #[error("error when validating: {0}")]
    Validation(String),
}
