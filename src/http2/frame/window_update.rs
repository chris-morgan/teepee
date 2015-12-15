//! The WINDOW_UPDATE frame definition. See [RFC 7540, section 6.9][spec].
//!
//! [spec]: http://tools.ietf.org/html/rfc7540#section-6.9

use std::io;

use ByteTendril;
use super::{Frame, Header, ErrorCode, PayloadSize, NoFlags};

/// The WINDOW_UPDATE frame definition. See [RFC 7540, section 6.9][spec].
///
/// [spec]: http://tools.ietf.org/html/rfc7540#section-6.9
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WindowUpdate {
    /// An unsigned 31-bit integer indicating the number of octets that the
    /// sender can transmit in addition to the existing flow-control window.
    /// This may not be zero, nor may it exceed 2³¹-1.
    pub window_size_increment: u32,
}

impl Frame for WindowUpdate {
    type Flags = NoFlags;
    const TYPE: u8 = 0x8;

    fn decode(_header: Header<NoFlags>, payload: ByteTendril) -> Result<Self, ErrorCode> {
        if payload.len32() != 4 {
            return Err(ErrorCode::FRAME_SIZE_ERROR);
        }
        let increment = ((payload[0] &0b01111111) as u32) << 24 |
                        (payload[1] as u32) << 16 |
                        (payload[2] as u32) << 8 |
                        payload[3] as u32;
        if increment == 0 {
            return Err(ErrorCode::PROTOCOL_ERROR);
        }
        Ok(WindowUpdate {
            window_size_increment: increment,
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
            (self.window_size_increment >> 24) as u8,
            (self.window_size_increment >> 16) as u8,
            (self.window_size_increment >> 8) as u8,
            self.window_size_increment as u8,
        ])
    }
}

frame_tests! {
    WindowUpdate;

    frame_size_zero {
        stream 0,
        payload [];

        Err(ErrorCode::FRAME_SIZE_ERROR)
    }

    frame_size_three {
        stream 0,
        payload [0; 3];

        Err(ErrorCode::FRAME_SIZE_ERROR)
    }

    frame_size_five {
        stream 0,
        payload [0; 5];

        Err(ErrorCode::FRAME_SIZE_ERROR)
    }

    zero_increment {
        stream 0,
        payload [0; 4];

        Err(ErrorCode::PROTOCOL_ERROR)
    }

    simple {
        stream 0,
        payload [0x01, 0x23, 0x45, 0x67];

        Ok(WindowUpdate { window_size_increment: 0x01234567 })
    }

    increment_decode {
        decode only,

        stream 0,
        payload [0xfe, 0xdc, 0xba, 0x98];

        Ok(WindowUpdate { window_size_increment: 0x7edcba98 })
    }

    another {
        stream 1,
        payload [0x7e, 0xdc, 0xba, 0x98];

        Ok(WindowUpdate { window_size_increment: 0x7edcba98 })
    }
}
