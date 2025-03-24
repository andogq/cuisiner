use std::num::NonZero;

use cuisiner::{ByteBoolean, ByteOrder, ConstU8, Cuisiner, CuisinerError, Reserved};
use zerocopy::{U16, U32};

#[derive(Clone, Cuisiner, Debug)]
#[cuisiner(assert_size = 100)]
struct SqliteHeader {
    header_string: HeaderString,
    page_size: PageSize,
    file_format_write_version: FileFormatVersion,
    file_format_read_version: FileFormatVersion,
    page_end_padding: Option<NonZero<u8>>,
    max_payload_fraction: ConstU8<64>,
    min_payload_fraction: ConstU8<32>,
    leaf_payload_fraction: ConstU8<32>,
    file_change_counter: u32,
    page_count: u32,
    freelist_trunk_page: u32,
    freelist_page_count: u32,
    schema_cookie: u32,
    schema_format: SchemaFormat,
    default_page_cache_size: u32,
    largest_root_btree_page: Option<NonZero<u32>>,
    text_encoding: TextEncoding,
    user_version: u32,
    incremental_vacuum_mode: ByteBoolean<4>,
    application_id: u32,
    _reserved: Reserved<20>,
    version_valid_for: u32,
    sqlite_version_number: VersionNumber,
}

#[derive(Clone, Debug)]
struct HeaderString;
impl HeaderString {
    const BYTES: [u8; 16] = *b"SQLite format 3\0";
}
impl Cuisiner for HeaderString {
    type Raw<B: ByteOrder> = [u8; 16];

    fn try_from_raw<B: ByteOrder>(raw: Self::Raw<B>) -> Result<Self, CuisinerError> {
        if raw != Self::BYTES {
            return Err(CuisinerError::Validation(format!(
                "invalid header string: {raw:?}"
            )));
        }

        Ok(Self)
    }

    fn try_to_raw<B: ByteOrder>(self) -> Result<Self::Raw<B>, CuisinerError> {
        Ok(Self::BYTES)
    }
}

#[derive(Clone, Debug)]
struct PageSize(u32);
impl PageSize {
    /// Minumum value of page size.
    const MIN: u32 = 512;
    /// Maximum encoded page size.
    const MAX: u32 = 32768;
    /// Page size of `1` encoded.
    const VALUE_FOR_1: u32 = 65536;
}
impl Cuisiner for PageSize {
    type Raw<B: ByteOrder> = U16<B>;

    fn try_from_raw<B: ByteOrder>(raw: Self::Raw<B>) -> Result<Self, CuisinerError> {
        Ok(Self(match raw.get() as u32 {
            1 => Self::VALUE_FOR_1,
            n @ Self::MIN..=Self::MAX if n.is_power_of_two() => n,
            n => {
                return Err(CuisinerError::Validation(format!(
                    "page size must be a power of 2 between {min} and {max} (found {n})",
                    min = Self::MIN,
                    max = Self::MAX
                )));
            }
        }))
    }

    fn try_to_raw<B: ByteOrder>(self) -> Result<Self::Raw<B>, CuisinerError> {
        Ok(U16::new(match self.0 {
            Self::VALUE_FOR_1 => 1,
            n @ Self::MIN..=Self::MAX => n as u16,
            n => {
                return Err(CuisinerError::Validation(format!(
                    "page size must be a power of 2 between {min} and {max} (found {n})",
                    min = Self::MIN,
                    max = Self::MAX,
                )));
            }
        }))
    }
}

#[derive(Clone, Cuisiner, Debug)]
#[cuisiner(repr = u8)]
enum FileFormatVersion {
    Legacy = 1,
    Wal = 2,
}

#[derive(Clone, Cuisiner, Debug)]
#[cuisiner(repr = u32)]
enum SchemaFormat {
    V1 = 1,
    V2 = 2,
    V3 = 3,
    V4 = 4,
}

#[derive(Clone, Cuisiner, Debug)]
#[cuisiner(repr = u32)]
enum TextEncoding {
    Utf8 = 1,
    Utf16Le = 2,
    Utf16Be = 3,
}

#[derive(Clone, Debug)]
struct VersionNumber {
    major: u16,
    minor: u16,
    patch: u16,
}
impl Cuisiner for VersionNumber {
    type Raw<B: ByteOrder> = U32<B>;

    fn try_from_raw<B: ByteOrder>(raw: Self::Raw<B>) -> Result<Self, CuisinerError> {
        let raw = raw.get();
        Ok(Self {
            major: (raw / 1_000_000) as u16,
            minor: (raw % 1_000_000 / 1_000) as u16,
            patch: (raw % 1_000) as u16,
        })
    }

    fn try_to_raw<B: ByteOrder>(self) -> Result<Self::Raw<B>, CuisinerError> {
        Ok(U32::new(
            self.major as u32 * 1_000_000 + self.minor as u32 * 1_000 + self.patch as u32,
        ))
    }
}
