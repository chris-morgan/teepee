//! The PRIORITY frame definition. See [RFC 7540, section 6.3][spec].
//!
//! [spec]: http://tools.ietf.org/html/rfc7540#section-6.3

use std::io;

use ByteTendril;
use super::{Frame, Header, ErrorCode, PayloadSize, NoFlags};
use super::super::stream::StreamId;

/// The PRIORITY frame definition. See [RFC 7540, section 6.3][spec].
///
/// [spec]: http://tools.ietf.org/html/rfc7540#section-6.3
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Priority {
    /// Whether the stream dependency is exclusive (see [Section 5.3][spec]).
    ///
    /// [spec]: http://tools.ietf.org/html/rfc7540#section-5.3
    pub exclusive: bool,

    /// The identifier for the stream that this stream depends on (see [Section 5.3][spec]).
    ///
    /// [spec]: http://tools.ietf.org/html/rfc7540#section-5.3
    pub stream_dependency: StreamId,

    /// The priority weight for the stream (with origin 0 rather than 1).
    pub weight: u8,
}

impl Frame for Priority {
    type Flags = NoFlags;
    const TYPE: u8 = 0x2;

    fn decode(header: Header<NoFlags>, payload: ByteTendril) -> Result<Priority, ErrorCode> {
        if header.stream_identifier.0 == 0 {
            return Err(ErrorCode::PROTOCOL_ERROR);
        }
        if payload.len32() != 5 {
            return Err(ErrorCode::FRAME_SIZE_ERROR);
        }
        let stream_dependency = stream_id_from_be_slice!(&*payload, 0);
        if stream_dependency.0 == 0 {
            // XXX: the spec *DOES NOT SAY* (in section 6.3, anyway) what should be done here.
            // Seriously. But it’s obviously illegal to depend on the connection as a whole.
            // I’m figuring on treating it the same as if header.stream_identifier is 0.
            return Err(ErrorCode::PROTOCOL_ERROR);
        }
        Ok(Priority {
            exclusive: payload[0] & 0b10000000 == 0b10000000,
            stream_dependency: stream_dependency,
            weight: payload[4],
        })
    }

    fn len(&self) -> PayloadSize {
        PayloadSize::Exact(5)
    }

    fn flags(&self) -> NoFlags {
        NoFlags
    }

    fn encode<W: io::Write>(self, w: &mut W) -> io::Result<()> {
        w.write_all(&[
            (self.stream_dependency.0 >> 24) as u8 | if self.exclusive { 0b10000000 } else { 0 },
            (self.stream_dependency.0 >> 16) as u8,
            (self.stream_dependency.0 >> 8) as u8,
            self.stream_dependency.0 as u8,
            self.weight,
        ])
    }
}

frame_tests! {
    Priority;

    applies_to_stream {
        stream 0,
        payload [0, 0, 0, 0, 0];

        Err(ErrorCode::PROTOCOL_ERROR)
    }

    frame_size_zero {
        stream 1,
        payload [];

        Err(ErrorCode::FRAME_SIZE_ERROR)
    }

    frame_size_four {
        stream 1,
        payload [0, 0, 0, 0];

        Err(ErrorCode::FRAME_SIZE_ERROR)
    }

    frame_size_six {
        stream 1,
        payload [0, 0, 0, 0, 0, 0];

        Err(ErrorCode::FRAME_SIZE_ERROR)
    }

    needs_nonzero_stream_dependency {
        stream 1,
        payload [0, 0, 0, 0, 0];

        Err(ErrorCode::PROTOCOL_ERROR)
    }

    basic {
        stream 1,
        payload [0, 0, 0, 1, 0];

        Ok(Priority { exclusive: false, stream_dependency: StreamId(1), weight: 0 })
    }

    another {
        stream 1,
        payload [0x12, 0x34, 0x56, 0x78, 0x9a];

        Ok(Priority { exclusive: false, stream_dependency: StreamId(0x12345678), weight: 0x9a })
    }

    exclusive {
        stream 1,
        payload [0xff, 0xff, 0xff, 0xff, 0xff];

        Ok(Priority { exclusive: true, stream_dependency: StreamId(0x7fffffff), weight: 0xff })
    }
}
