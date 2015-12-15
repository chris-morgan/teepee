//! The GOAWAY frame definition. See [RFC 7540, section 6.8][spec].
//!
//! [spec]: http://tools.ietf.org/html/rfc7540#section-6.8

use std::io;

use ByteTendril;
use super::{Frame, Header, ErrorCode, PayloadSize, NoFlags};
use super::super::stream::StreamId;

/// The GOAWAY frame definition. See [RFC 7540, section 6.8][spec].
///
/// [spec]: http://tools.ietf.org/html/rfc7540#section-6.8
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoAway {
    /// The highest-numbered stream identifier which the sender might have taken action on or might
    /// yet take action on. This can also be zero if no streams have been acted on.
    pub last_stream_id: StreamId,

    /// The reason for closing the connection.
    pub error_code: ErrorCode,

    /// Opaque data, intended for diagnostic purposes only and carrying no semantic value.
    /// This could contain security- or privacy-sensitive data, so be careful what you do with it.
    pub additional_debug_data: ByteTendril,
}

impl Frame for GoAway {
    type Flags = NoFlags;
    const TYPE: u8 = 0x7;

    fn decode(header: Header<NoFlags>, mut payload: ByteTendril) -> Result<GoAway, ErrorCode> {
        if header.stream_identifier.0 != 0 {
            return Err(ErrorCode::PROTOCOL_ERROR);
        }
        if payload.len32() < 8 {
            return Err(ErrorCode::FRAME_SIZE_ERROR);
        }
        let last_stream_id = stream_id_from_be_slice!(&*payload, 0);
        let error_code = ErrorCode((payload[4] as u32) << 24 |
                                   (payload[5] as u32) << 16 |
                                   (payload[6] as u32) << 8 |
                                   payload[7] as u32);
        payload.pop_front(8);
        Ok(GoAway {
            last_stream_id: last_stream_id,
            error_code: error_code,
            additional_debug_data: payload,
        })
    }

    fn len(&self) -> PayloadSize {
        PayloadSize::Exact(8 + self.additional_debug_data.len32())
    }

    fn flags(&self) -> NoFlags {
        NoFlags
    }

    fn encode<W: io::Write>(self, w: &mut W) -> io::Result<()> {
        try!(w.write_all(&[
            (self.last_stream_id.0 >> 24) as u8,
            (self.last_stream_id.0 >> 16) as u8,
            (self.last_stream_id.0 >> 8) as u8,
            self.last_stream_id.0 as u8,
            (self.error_code.0 >> 24) as u8,
            (self.error_code.0 >> 16) as u8,
            (self.error_code.0 >> 8) as u8,
            self.error_code.0 as u8,
        ]));
        w.write_all(&self.additional_debug_data)
    }
}

frame_tests! {
    GoAway;

    applies_to_connection {
        stream 1,
        payload [0; 8];

        Err(ErrorCode::PROTOCOL_ERROR)
    }

    frame_size_zero {
        stream 0,
        payload [];

        Err(ErrorCode::FRAME_SIZE_ERROR)
    }

    frame_size_seven {
        stream 0,
        payload [0; 7];

        Err(ErrorCode::FRAME_SIZE_ERROR)
    }

    basic {
        stream 0,
        payload [0; 8];

        Ok(GoAway {
            last_stream_id: StreamId(0),
            error_code: ErrorCode::NO_ERROR,
            additional_debug_data: b"".to_tendril(),
        })
    }

    more_complex {
        stream 0,
        payload [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
                 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10];

        Ok(GoAway {
            last_stream_id: StreamId(0x01234567),
            error_code: ErrorCode(0x89abcdef),
            additional_debug_data: b"\xfe\xdc\xba\x98\x76\x54\x32\x10".to_tendril(),
        })
    }
}
