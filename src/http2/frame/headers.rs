//! The HEADERS frame definition. See [RFC 7540, section 6.2][spec].
//!
//! [spec]: http://tools.ietf.org/html/rfc7540#section-6.2

use std::io;

use ByteTendril;
use super::{Frame, Header, ErrorCode, PayloadSize};
use super::{decode_padding, encode_pad_length, encode_padding};
use super::hpack;
use super::priority::Priority;

flags! {
    const END_STREAM = 0x1,
    const END_HEADERS = 0x4,
    const PADDED = 0x8,
    const PRIORITY = 0x20,
}

/// The HEADERS frame definition. See [RFC 7540, section 6.2][spec].
///
/// [spec]: http://tools.ietf.org/html/rfc7540#section-6.2
#[derive(Debug, Eq, PartialEq)]
pub struct Headers {
    /// If `None`, the PADDED flag is not set and there is no padding;
    /// if `Some(n)`, the PADDED flag is set and the value is meaningful.
    pub pad_length: Option<u8>,

    /// Whether the END_STREAM flag is set.
    pub end_stream: bool,

    /// Whether the END_HEADERS flag is set.
    pub end_headers: bool,

    /// The priority details, if the PRIORITY flag is set.
    /// It’s basically an inline PRIORITY frame.
    pub priority: Option<Priority>,

    /// A header block fragment ([Section 4.3][spec]).
    ///
    /// [spec]: http://tools.ietf.org/html/rfc7540#section-4.3
    pub header_block: hpack::Fragment,
}

impl Frame for Headers {
    type Flags = Flags;
    const TYPE: u8 = 0x1;

    fn decode(header: Header<Flags>, mut payload: ByteTendril) -> Result<Self, ErrorCode> {
        let pad_length = try!(decode_padding(header.flags.contains(PADDED), &mut payload));
        let priority = if header.flags.contains(PRIORITY) {
            let priority = try!(Priority::decode(header.change_flags_type(),
                                                 payload.subtendril(0, 5)));
            payload.pop_front(5);
            Some(priority)
        } else {
            None
        };
        Ok(Headers {
            pad_length: pad_length,
            end_stream: header.flags.contains(END_STREAM),
            end_headers: header.flags.contains(END_HEADERS),
            priority: priority,
            header_block: hpack::Fragment::Decoder(hpack::InstructionDecoder::new(payload)),
        })
    }

    fn len(&self) -> PayloadSize {
        PayloadSize::Unknown
    }

    fn flags(&self) -> Flags {
        let mut flags = Flags::empty();
        if self.pad_length.is_some() {
            flags = flags | PADDED;
        }
        if self.end_headers {
            flags = flags | END_HEADERS;
        }
        if self.end_stream {
            flags = flags | END_STREAM;
        }
        if self.priority.is_some() {
            flags = flags | PRIORITY;
        }
        flags
    }

    fn encode<W: io::Write>(self, w: &mut W) -> io::Result<()> {
        try!(encode_pad_length(w, self.pad_length));
        if let Some(priority) = self.priority {
            try!(priority.encode(w));
        }
        try!(self.header_block.encode(w));
        encode_padding(w, self.pad_length)
    }
}
