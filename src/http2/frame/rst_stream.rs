//! The RST_STREAM frame definition. See [RFC 7540, section 6.4][spec].
//!
//! [spec]: http://tools.ietf.org/html/rfc7540#section-6.4

use std::io;

use ByteTendril;
use super::{Frame, Header, ErrorCode, PayloadSize, NoFlags};

/// The RST_STREAM frame definition. See [RFC 7540, section 6.4][spec].
///
/// [spec]: http://tools.ietf.org/html/rfc7540#section-6.4
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RstStream {
    /// The reason why the stream is being terminated.
    pub error_code: ErrorCode,
}

impl Frame for RstStream {
    type Flags = NoFlags;
    const TYPE: u8 = 0x3;

    fn decode(header: Header<NoFlags>, payload: ByteTendril) -> Result<RstStream, ErrorCode> {
        if header.stream_identifier.0 == 0 {
            return Err(ErrorCode::PROTOCOL_ERROR);
        }
        // XXX: there is another connection error that can arise (again, PROTOCOL_ERROR):
        // if the stream is in the “idle” state. But this is not the right place to check *that*.
        // Just wanted to note it somewhere until it’s done.
        if payload.len32() != 4 {
            return Err(ErrorCode::FRAME_SIZE_ERROR);
        }
        Ok(RstStream {
            error_code: ErrorCode((payload[0] as u32) << 24 |
                                  (payload[1] as u32) << 16 |
                                  (payload[2] as u32) << 8 |
                                  payload[3] as u32),
        })
    }

    fn len(&self) -> PayloadSize {
        PayloadSize::Exact(4)
    }

    fn flags(&self) -> NoFlags {
        NoFlags
    }

    fn encode<W: io::Write>(self, w: &mut W) -> io::Result<()> {
        w.write_all(&[
            (self.error_code.0 >> 24) as u8,
            (self.error_code.0 >> 16) as u8,
            (self.error_code.0 >> 8) as u8,
            self.error_code.0 as u8,
        ])
    }
}

frame_tests! {
    RstStream;

    applies_to_stream {
        stream 0,
        payload [0, 0, 0, 0];

        Err(ErrorCode::PROTOCOL_ERROR)
    }

    frame_size_zero {
        stream 1,
        payload [];

        Err(ErrorCode::FRAME_SIZE_ERROR)
    }

    frame_size_three {
        stream 1,
        payload [0, 0, 0];

        Err(ErrorCode::FRAME_SIZE_ERROR)
    }

    frame_size_five {
        stream 1,
        payload [0, 0, 0, 0, 0];

        Err(ErrorCode::FRAME_SIZE_ERROR)
    }

    basic {
        stream 1,
        payload [0, 0, 0, 0];

        Ok(RstStream { error_code: ErrorCode::NO_ERROR })
    }

    another {
        stream 1,
        payload [0x12, 0x34, 0x56, 0x78];

        Ok(RstStream { error_code: ErrorCode(0x12345678) })
    }

    yet_another {
        stream 1,
        payload [0xff, 0xff, 0xff, 0xff];

        Ok(RstStream { error_code: ErrorCode(0xffffffff) })
    }
}
