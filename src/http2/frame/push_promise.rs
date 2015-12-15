//! The PUSH_PROMISE frame definition. See [RFC 7540, section 6.6][spec].
//!
//! [spec]: http://tools.ietf.org/html/rfc7540#section-6.6

use std::io;

use ByteTendril;
use super::{Frame, Header, ErrorCode, PayloadSize};
use super::{decode_padding, encode_pad_length, encode_padding, encode_stream_id};
use super::super::stream::StreamId;
use super::hpack;

flags! {
    const END_HEADERS = 0x4,
    const PADDED = 0x8,
}

/// The PUSH_PROMISE frame definition. See [RFC 7540, section 6.6][spec].
///
/// [spec]: http://tools.ietf.org/html/rfc7540#section-6.6
#[derive(Debug, Eq, PartialEq)]
pub struct PushPromise {
    /// If `None`, the PADDED flag is not set and there is no padding;
    /// if `Some(n)`, the PADDED flag is set and the value is meaningful.
    pub pad_length: Option<u8>,

    /// Whether the END_HEADERS flag is set.
    pub end_headers: bool,

    /// > ```text
    /// >    Promised Stream ID:  An unsigned 31-bit integer that identifies the
    /// >       stream that is reserved by the PUSH_PROMISE.  The promised stream
    /// >       identifier MUST be a valid choice for the next stream sent by the
    /// >       sender (see "new stream identifier" in Section 5.1.1).
    /// > ```
    pub promised_stream_id: StreamId,

    /// A header block fragment ([Section 4.3][spec]) containing request header fields.
    ///
    /// [spec]: http://tools.ietf.org/html/rfc7540#section-4.3
    pub header_block: hpack::Fragment,
}

impl Frame for PushPromise {
    type Flags = Flags;
    const TYPE: u8 = 0x5;

    fn decode(header: Header<Flags>, mut payload: ByteTendril) -> Result<Self, ErrorCode> {
        let pad_length = try!(decode_padding(header.flags.contains(PADDED), &mut payload));
        let promised_stream_id = stream_id_from_be_slice!(&*payload, 0);
        payload.pop_front(4);
        Ok(PushPromise {
            pad_length: pad_length,
            end_headers: header.flags.contains(END_HEADERS),
            promised_stream_id: promised_stream_id,
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
        flags
    }

    fn encode<W: io::Write>(self, w: &mut W) -> io::Result<()> {
        try!(encode_pad_length(w, self.pad_length));
        try!(encode_stream_id(w, false, self.promised_stream_id));
        try!(self.header_block.encode(w));
        encode_padding(w, self.pad_length)
    }
}
