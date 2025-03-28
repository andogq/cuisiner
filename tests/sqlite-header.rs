use std::num::NonZero;

use cuisiner::{ByteBoolean, ByteOrder, ConstU8, Cuisiner, CuisinerError, Reserved};
use zerocopy::{U16, U32};

#[allow(dead_code)]
const HEADER_SIZE: usize = 100;
const HEADER_STRING_LEN: usize = 16;

#[derive(Clone, Cuisiner, Debug)]
#[cuisiner(assert_size = HEADER_SIZE)]
struct SqliteHeader {
    #[cuisiner(offset = 0, size = HEADER_STRING_LEN)]
    header_string: HeaderString,
    #[cuisiner(offset = HEADER_STRING_LEN, size = 2)]
    page_size: PageSize,
    #[cuisiner(offset = 18, size = 1)]
    file_format_write_version: FileFormatVersion,
    #[cuisiner(offset = 19, size = 1)]
    file_format_read_version: FileFormatVersion,
    #[cuisiner(offset = 20, size = 1)]
    page_end_padding: Option<NonZero<u8>>,
    #[cuisiner(offset = 21, size = 1)]
    max_payload_fraction: ConstU8<64>,
    #[cuisiner(offset = 22, size = 1)]
    min_payload_fraction: ConstU8<32>,
    #[cuisiner(offset = 23, size = 1)]
    leaf_payload_fraction: ConstU8<32>,
    #[cuisiner(offset = 24, size = 4)]
    file_change_counter: u32,
    #[cuisiner(offset = 28, size = 4)]
    page_count: u32,
    #[cuisiner(offset = 32, size = 4)]
    freelist_trunk_page: u32,
    #[cuisiner(offset = 36, size = 4)]
    freelist_page_count: u32,
    #[cuisiner(offset = 40, size = 4)]
    schema_cookie: u32,
    #[cuisiner(offset = 44, size = 4)]
    schema_format: SchemaFormat,
    #[cuisiner(offset = 48, size = 4)]
    default_page_cache_size: u32,
    #[cuisiner(offset = 52, size = 4)]
    largest_root_btree_page: Option<NonZero<u32>>,
    #[cuisiner(offset = 56, size = 4)]
    text_encoding: TextEncoding,
    #[cuisiner(offset = 60, size = 4)]
    user_version: u32,
    #[cuisiner(offset = 64, size = 4)]
    incremental_vacuum_mode: ByteBoolean<4>,
    #[cuisiner(offset = 68, size = 4)]
    application_id: u32,
    #[cuisiner(offset = 72, size = 20)]
    _reserved: Reserved<20>,
    #[cuisiner(offset = 92, size = 4)]
    version_valid_for: u32,
    #[cuisiner(offset = 96, size = 4)]
    sqlite_version_number: VersionNumber,
}

#[derive(Clone, Debug)]
struct HeaderString;
impl HeaderString {
    const BYTES: [u8; HEADER_STRING_LEN] = *b"SQLite format 3\0";
}
impl Cuisiner for HeaderString {
    type Raw<B: ByteOrder> = [u8; HEADER_STRING_LEN];

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
