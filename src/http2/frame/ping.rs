//! The PING frame definition. See [RFC 7540, section 6.7][spec].
//!
//! [spec]: http://tools.ietf.org/html/rfc7540#section-6.7

use std::io;

use ByteTendril;
use super::{Frame, Header, ErrorCode, PayloadSize};

flags! {
    const ACK = 0x1,
}

/// The PING frame definition. See [RFC 7540, section 6.7][spec].
///
/// [spec]: http://tools.ietf.org/html/rfc7540#section-6.7
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ping {
    /// Whether the ACK flag is set. (Thus, whether this PING frame is a response to another.)
    pub is_response: bool,

    /// Opaque data with no semantic significance but which must be preserved exactly.
    pub data: [u8; 8],
}

impl Frame for Ping {
    type Flags = Flags;
    const TYPE: u8 = 0x6;

    fn decode(header: Header<Flags>, payload: ByteTendril) -> Result<Ping, ErrorCode> {
        if header.stream_identifier.0 != 0 {
            return Err(ErrorCode::PROTOCOL_ERROR);
        }
        if payload.len32() != 8 {
            return Err(ErrorCode::FRAME_SIZE_ERROR);
        }
        let data = &*payload;
        Ok(Ping {
            is_response: header.flags.contains(ACK),
            data: [data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]],
        })
    }

    fn len(&self) -> PayloadSize {
        PayloadSize::Exact(8)
    }

    fn flags(&self) -> Flags {
        if self.is_response {
            ACK
        } else {
            Flags::empty()
        }
    }

    fn encode<W: io::Write>(self, w: &mut W) -> io::Result<()> {
        w.write_all(&self.data)
    }
}

frame_tests! {
    Ping;

    applies_to_connection {
        flags Flags::empty(),
        stream 1,
        payload [0; 8];

        Err(ErrorCode::PROTOCOL_ERROR)
    }

    ack_applies_to_connection {
        flags ACK,
        stream 1,
        payload [0; 8];

        Err(ErrorCode::PROTOCOL_ERROR)
    }

    frame_size_zero {
        flags Flags::empty(),
        stream 0,
        payload [];

        Err(ErrorCode::FRAME_SIZE_ERROR)
    }

    frame_size_seven {
        flags Flags::empty(),
        stream 0,
        payload [0; 7];

        Err(ErrorCode::FRAME_SIZE_ERROR)
    }

    frame_size_nine {
        flags Flags::empty(),
        stream 0,
        payload [0; 9];

        Err(ErrorCode::FRAME_SIZE_ERROR)
    }

    basic {
        flags Flags::empty(),
        stream 0,
        payload [0; 8];

        Ok(Ping { is_response: false, data: [0; 8] })
    }

    basic_ack {
        flags ACK,
        stream 0,
        payload [0; 8];

        Ok(Ping { is_response: true, data: [0; 8] })
    }

    another {
        flags Flags::empty(),
        stream 0,
        payload [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];

        Ok(Ping { is_response: false, data: [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef] })
    }

    another_ack {
        flags ACK,
        stream 0,
        payload [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];

        Ok(Ping { is_response: true, data: [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef] })
    }
}
