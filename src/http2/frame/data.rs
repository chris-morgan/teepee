//! The DATA frame definition. See [RFC 7540, section 6.1][spec].
//!
//! [spec]: http://tools.ietf.org/html/rfc7540#section-6.1

use std::io;

use ByteTendril;
use super::{Frame, Header, ErrorCode, PayloadSize};
use super::{decode_padding, encode_pad_length, encode_padding};

flags! {
    const END_STREAM = 0x1,
    const PADDED = 0x8,
}

/// The DATA frame definition. See [RFC 7540, section 6.1][spec].
///
/// [spec]: http://tools.ietf.org/html/rfc7540#section-6.1
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Data {
    /// If `None`, the PADDED flag is not set and there is no padding;
    /// if `Some(n)`, the PADDED flag is set and the value is meaningful.
    pub pad_length: Option<u8>,

    /// Whether the END_STREAM flag is set.
    pub end_stream: bool,

    /// Application data.
    pub data: ByteTendril,
}

impl Frame for Data {
    type Flags = Flags;
    const TYPE: u8 = 0x0;

    fn decode(header: Header<Flags>, mut payload: ByteTendril) -> Result<Self, ErrorCode> {
        if header.stream_identifier.0 == 0 {
            return Err(ErrorCode::PROTOCOL_ERROR);
        }
        let pad_length = try!(decode_padding(header.flags.contains(PADDED), &mut payload));
        Ok(Data {
            pad_length: pad_length,
            end_stream: header.flags.contains(END_STREAM),
            data: payload,
        })
    }

    fn len(&self) -> PayloadSize {
        PayloadSize::Exact(self.pad_length.map(|x| x as u32 + 1).unwrap_or(0) + self.data.len32())
    }

    fn flags(&self) -> Flags {
        let mut flags = Flags::empty();
        if self.pad_length.is_some() {
            flags = flags | PADDED;
        }
        if self.end_stream {
            flags = flags | END_STREAM;
        }
        flags
    }

    fn encode<W: io::Write>(self, w: &mut W) -> io::Result<()> {
        try!(encode_pad_length(w, self.pad_length));
        try!(w.write_all(&self.data));
        encode_padding(w, self.pad_length)
    }
}

frame_tests! {
    Data;

    applies_to_stream {
        flags Flags::empty(),
        stream 0,
        payload [0];

        Err(ErrorCode::PROTOCOL_ERROR)
    }

    small {
        flags Flags::empty(),
        stream 1,
        payload [0];

        Ok(Data { pad_length: None, end_stream: false, data: [0].to_tendril() })
    }

    empty {
        flags Flags::empty(),
        stream 1,
        payload [];

        Ok(Data { pad_length: None, end_stream: false, data: [].to_tendril() })
    }

    empty_padding {
        flags PADDED,
        stream 1,
        payload [0];

        Ok(Data { pad_length: Some(0), end_stream: false, data: [].to_tendril() })
    }

    bad_padding {
        flags PADDED,
        stream 1,
        payload [1];

        Err(ErrorCode::PROTOCOL_ERROR)
    }

    oodles_of_padding {
        flags PADDED,
        stream 1,
        payload [255,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        Ok(Data { pad_length: Some(255), end_stream: false, data: [].to_tendril() })
    }

    end_stream_flag {
        flags END_STREAM,
        stream 1,
        payload [];

        Ok(Data { pad_length: None, end_stream: true, data: [].to_tendril() })
    }
}
