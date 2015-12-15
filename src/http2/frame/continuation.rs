//! The CONTINUATION frame definition. See [RFC 7540, section 6.10][spec].
//!
//! [spec]: http://tools.ietf.org/html/rfc7540#section-6.10

use std::io;

use ByteTendril;
use super::{Frame, Header, ErrorCode, PayloadSize};
use super::hpack;

flags! {
    const END_HEADERS = 0x4,
}

/// The CONTINUATION frame definition. See [RFC 7540, section 6.10][spec].
///
/// [spec]: http://tools.ietf.org/html/rfc7540#section-6.10
#[derive(Debug, Eq, PartialEq)]
pub struct Continuation {
    /// Whether the END_HEADERS flag is set.
    pub end_headers: bool,

    /// A header block fragment ([Section 4.3][spec]).
    ///
    /// [spec]: http://tools.ietf.org/html/rfc7540#section-4.3
    pub header_block: hpack::Fragment,
}

impl Frame for Continuation {
    type Flags = Flags;
    const TYPE: u8 = 0x9;

    fn decode(header: Header<Flags>, payload: ByteTendril) -> Result<Self, ErrorCode> {
        Ok(Continuation {
            end_headers: header.flags.contains(END_HEADERS),
            header_block: hpack::Fragment::Decoder(hpack::InstructionDecoder::new(payload)),
        })
    }

    fn len(&self) -> PayloadSize {
        PayloadSize::Unknown
    }

    fn flags(&self) -> Flags {
        if self.end_headers {
            END_HEADERS
        } else {
            Flags::empty()
        }
    }

    fn encode<W: io::Write>(self, w: &mut W) -> io::Result<()> {
        self.header_block.encode(w)
    }
}
