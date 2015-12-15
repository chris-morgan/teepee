//! Stream identifier matters.

/// A stream identifier.
///
/// This is only a 31-bit quantity, but that the most significant bit is zero is not enforced at
/// the type level; you must maintain that yourself.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StreamId(pub u32);

macro_rules! stream_id_from_be_slice {
    ($slice:expr, $offset:expr) => {{
        let slice = $slice;
        $crate::http2::stream::StreamId((slice[$offset] as u32 & 0b01111111) << 24 |
                                        (slice[$offset + 1] as u32) << 16 |
                                        (slice[$offset + 2] as u32) << 8 |
                                        (slice[$offset + 3] as u32))
    }}
}
