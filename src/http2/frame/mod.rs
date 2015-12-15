//! > ```text
//! > 6.  Frame Definitions
//! > 
//! >    This specification defines a number of frame types, each identified
//! >    by a unique 8-bit type code.  Each frame type serves a distinct
//! >    purpose in the establishment and management either of the connection
//! >    as a whole or of individual streams.
//! > 
//! >    The transmission of specific frame types can alter the state of a
//! >    connection.  If endpoints fail to maintain a synchronized view of the
//! >    connection state, successful communication within the connection will
//! >    no longer be possible.  Therefore, it is important that endpoints
//! >    have a shared comprehension of how the state is affected by the use
//! >    any given frame.
//! > ```

use std::io;

use ByteTendril;
use http2::stream::StreamId;
// RFC 7540, section 7, Error Codes
mod error_code;
pub use self::error_code::ErrorCode;

/// A frame header’s flags.
pub trait Flags: From<u8> + Copy {
    /// Get the bits set by this flags collection as a `u8`.
    fn bits(&self) -> u8;
}

impl Flags for u8 {
    #[inline]
    fn bits(&self) -> u8 {
        *self
    }
}

/// The frame type has no flags.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NoFlags;

impl From<u8> for NoFlags {
    fn from(_bits: u8) -> NoFlags {
        NoFlags
    }
}

impl Flags for NoFlags {
    fn bits(&self) -> u8 {
        0
    }
}

macro_rules! flags {
    (//$(#![$attr:meta])*
        $($(#[$Flag_attr:meta])* const $Flag:ident = $value:expr),+,
    ) => {
        bitflags! {
            #[allow(missing_docs)]
            //$(#[$attr])*
            flags Flags: u8 {
                $(
                    $(#[$Flag_attr])*
                    #[allow(missing_docs)]
                    const $Flag = $value
                ),+
            }
        }

        impl From<u8> for Flags {
            #[inline]
            fn from(bits: u8) -> Flags {
                Flags::all() & Flags { bits: bits }
            }
        }

        impl super::Flags for Flags {
            #[inline]
            fn bits(&self) -> u8 {
                self.bits()
            }
        }
    }
}

//pub struct Connection;

//pub struct Message;

/// The length of a payload.
pub enum PayloadSize {
    /// The payload length is known ahead of time and is this value.
    Exact(u32),

    /// The payload length will only be determined after writing the value.
    /// Since the header comes before the payload, this means that the payload will
    /// need to be written to intermediate storage in order to calculate the length.
    Unknown,
}

/// A frame payload.
pub trait Frame: Sized {
    /// The flags used by this frame type’s header.
    type Flags: Flags;

    /// The frame type code.
    const TYPE: u8;

    /// Decode a payload into a new object.
    fn decode(header: Header<Self::Flags>, payload: ByteTendril) -> Result<Self, ErrorCode>;

    /// Calculate the length of the payload.
    fn len(&self) -> PayloadSize;

    /// Calculate the flags that should be written for the representation of the frame.
    fn flags(&self) -> Self::Flags;

    /// Write the payload to the writer, consuming it.
    fn encode<W: io::Write>(self, w: &mut W) -> io::Result<()>;

    // TODO: is this what we want from the design?
    //fn apply(connection: &mut Connection) -> Result<Message, ErrorCode> { unimplemented!(); }

    /// Write the frame (with a partially complete header).
    fn write_frame<W>(self, mut header: Header<Self::Flags>, w: &mut W) -> io::Result<()>
    where W: io::Write {
        header.flags = self.flags();
        match self.len() {
            PayloadSize::Exact(len) => {
                header.length = len;
                try!(w.write_all(&header.encode()));
                self.encode(w)
            },
            PayloadSize::Unknown => {
                // As the payload size is not known, we must buffer it and calculate the length.
                // TODO: optimise this in some way (e.g. slab allocation of intermediate vectors).
                let mut buffer = vec![];
                try!(self.encode(&mut buffer));
                header.length = buffer.len() as u32;
                try!(w.write_all(&buffer));
                w.write_all(&buffer)
            },
        }
    }
}

/// > ```text
/// > 4.1.  Frame Format
/// > 
/// >    All frames begin with a fixed 9-octet header followed by a variable-
/// >    length payload.
/// > 
/// >     +-----------------------------------------------+
/// >     |                 Length (24)                   |
/// >     +---------------+---------------+---------------+
/// >     |   Type (8)    |   Flags (8)   |
/// >     +-+-------------+---------------+-------------------------------+
/// >     |R|                 Stream Identifier (31)                      |
/// >     +=+=============================================================+
/// >     |                   Frame Payload (0...)                      ...
/// >     +---------------------------------------------------------------+
/// > 
/// >                           Figure 1: Frame Layout
/// > ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Header<F: Flags = NoFlags> {
    /// > Length:  The length of the frame payload expressed as an unsigned
    /// >    24-bit integer.  Values greater than 2^14 (16,384) MUST NOT be
    /// >    sent unless the receiver has set a larger value for
    /// >    SETTINGS_MAX_FRAME_SIZE.
    /// >
    /// >    The 9 octets of the frame header are not included in this value.
    pub length: u32,

    /// > Type:  The 8-bit type of the frame.  The frame type determines the
    /// >    format and semantics of the frame.  Implementations MUST ignore
    /// >    and discard any frame that has a type that is unknown.
    pub type_: u8,

    /// > Flags:  An 8-bit field reserved for boolean flags specific to the
    /// >    frame type.
    /// >
    /// >    Flags are assigned semantics specific to the indicated frame type.
    /// >    Flags that have no defined semantics for a particular frame type
    /// >    MUST be ignored and MUST be left unset (0x0) when sending.
    pub flags: F,

    // > R: A reserved 1-bit field.  The semantics of this bit are undefined,
    // >    and the bit MUST remain unset (0x0) when sending and MUST be
    // >    ignored when receiving.

    /// > Stream Identifier:  A stream identifier (see Section 5.1.1) expressed
    /// >    as an unsigned 31-bit integer.  The value 0x0 is reserved for
    /// >    frames that are associated with the connection as a whole as
    /// >    opposed to an individual stream.
    pub stream_identifier: StreamId,
}

impl<F: Flags> Header<F> {
    /// Decode a header. This cannot panic and no guarantees are made about the header’s validity.
    #[inline]
    pub fn decode(bytes: [u8; 9]) -> Header<F> {
        Header {
            length: (bytes[0] as u32) << 16 |
                    (bytes[1] as u32) << 8 |
                    (bytes[2] as u32),
            type_: bytes[3],
            flags: F::from(bytes[4]),
            stream_identifier: StreamId((bytes[5] as u32 & 0b01111111) << 24 |
                                        (bytes[6] as u32) << 16 |
                                        (bytes[7] as u32) << 8 |
                                        (bytes[8] as u32)),
        }
    }

    /// Encode a header. This cannot panic and no guarantees are made about the header’s validity.
    #[inline]
    pub fn encode(self) -> [u8; 9] {
        [
            (self.length >> 16) as u8,
            (self.length >> 8) as u8,
            self.length as u8,
            self.type_,
            self.flags.bits(),
            (self.stream_identifier.0 >> 24) as u8 & 0b01111111,
            (self.stream_identifier.0 >> 16) as u8,
            (self.stream_identifier.0 >> 8) as u8,
            self.stream_identifier.0 as u8,
        ]
    }

    #[inline]
    fn change_flags_type<F2: Flags>(self) -> Header<F2> {
        Header {
            length: self.length,
            type_: self.type_,
            flags: F2::from(self.flags.bits()),
            stream_identifier: self.stream_identifier,
        }
    }
}

macro_rules! extract {
    (flags; flags $v:expr, $($k2:tt $v2:expr,)*) => ($v);
    (flags; $k:tt $v:expr, $($k2:tt $v2:expr,)*) => (extract!(flags; $($k2 $v2,)*));
    (flags;) => (NoFlags);
    (stream; stream $v:expr, $($k2:tt $v2:expr,)*) => ($v);
    (stream; $k:tt $v:expr, $($k2:tt $v2:expr,)*) => (extract!(stream; $($k2 $v2,)*));
    (payload; payload $v:expr, $($k2:tt $v2:expr,)*) => ($v);
    (payload; $k:tt $v:expr, $($k2:tt $v2:expr,)*) => (extract!(payload; $($k2 $v2,)*));
}

macro_rules! frame_test_decode {
    ($frame:ident; $($k:tt $v:expr),*; $decoded:expr) => {{
        let payload = extract!(payload; $($k $v,)*).to_tendril();
        let expected = $decoded;
        let decoded = $frame::decode(
            Header {
                length: payload.len32(),
                type_: $frame::TYPE,
                flags: extract!(flags; $($k $v,)*),
                stream_identifier: StreamId(extract!(stream; $($k $v,)*)),
            }, payload);
        assert_eq!(decoded, expected);
    }}
}

macro_rules! frame_test_encode {
    ($frame:ident; $($k:tt $v:expr),*; Ok($decoded:expr)) => {{
        let mut encoded = vec![];
        let partial_header: Header<<$frame as Frame>::Flags> = Header {
            length: 0,
            type_: $frame::TYPE,
            flags: extract!(flags; $($k $v,)*),
            stream_identifier: StreamId(extract!(stream; $($k $v,)*)),
        };
        $decoded.write_frame(partial_header, &mut encoded).unwrap();

        let expected: &[u8] = &extract!(payload; $($k $v,)*);
        assert_eq!(&encoded[9..], expected);
        let len = encoded.len() - 9;
        assert_eq!(&encoded[0..3], &[(len >> 16) as u8, (len >> 8) as u8, len as u8]);
        assert_eq!(encoded[3], partial_header.type_);
        assert_eq!(encoded[4], partial_header.flags.bits());
        assert_eq!(&encoded[5..9], &[
            (partial_header.stream_identifier.0 >> 24) as u8 & 0b01111111,
            (partial_header.stream_identifier.0 >> 16) as u8,
            (partial_header.stream_identifier.0 >> 8) as u8,
            partial_header.stream_identifier.0 as u8,
        ]);
    }}
}

macro_rules! frame_test {
    ($frame:ident; decode only, $($x:tt)*) => {
        frame_test_decode!($frame; $($x)*);
    };

    ($frame:ident; $($k:tt $v:expr),*; Err($decoded:expr)) => {
        frame_test_decode!($frame; $($k $v),*; Err($decoded));
    };

    ($($x:tt)*) => {
        frame_test_decode!($($x)*);
        frame_test_encode!($($x)*);
    };
}

macro_rules! frame_tests {
    ($type_:ident; $( $name:ident { $($x:tt)* } )*) => {
        $(
            #[test]
            fn $name() {
                #[allow(unused_imports)]
                use http2::frame::Flags as _Flags;
                #[allow(unused_imports)]
                use http2::stream::StreamId;
                #[allow(unused_imports)]
                use {ByteTendril, TendrilSliceExt};
                frame_test!($type_; $($x)*);
            }
        )*
    };
}

// This should be kept up to date with the registered frame types found in the IANA registry:
// http://www.iana.org/assignments/http2-parameters/http2-parameters.xhtml#frame-type

// RFC 7540, section 6, Frame Definitions
pub mod data;
pub mod headers;
pub mod priority;
pub mod rst_stream;
pub mod settings;
pub mod push_promise;
pub mod ping;
pub mod goaway;
pub mod window_update;
pub mod continuation;

macro_rules! define_frame_types {
    ($($path:ident :: $ty:ident),*$(,)*) => {
        /// The payload of a frame.
        // Can’t be Clone because hpack::Fragment isn’t, so some types wouldn’t work.
        #[derive(Debug, Eq, PartialEq)]
        pub enum Payload {
            $(#[allow(missing_docs)] $ty($path::$ty),)*
            /// A frame of unknown type, which must be ignored.
            UnknownType,
        }
        impl Payload {
            /// Decode the payload of a frame.
            pub fn decode(header: Header<u8>, payload: ByteTendril) -> Result<Self, ErrorCode> {
                match header.type_ {
                    $(<$path::$ty as Frame>::TYPE => {
                        Ok(Payload::$ty(try!(<$path::$ty as Frame>::decode(
                            header.change_flags_type(), payload))))
                    },)*
                    _ => {
                        Ok(Payload::UnknownType)
                        // > […] Implementations MUST ignore
                        // > and discard any frame that has a type that is unknown.
                    },
                }
            }
        }
    }
}

define_frame_types! {
    data::Data,
    headers::Headers,
    priority::Priority,
    rst_stream::RstStream,
    settings::Settings,
    push_promise::PushPromise,
    ping::Ping,
    goaway::GoAway,
    window_update::WindowUpdate,
    continuation::Continuation,
}

//const CONNECTION_PRELUDE: &'static [u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

static PADDING: [u8; 256] = [0; 256];

fn decode_padding(padded: bool, payload: &mut ByteTendril) -> Result<Option<u8>, ErrorCode> {
    if padded {
        match payload.get(0) {
            Some(&pad_length) => {
                payload.pop_front(1);
                if payload.try_pop_back(pad_length as u32).is_err() {
                    // > The total number of padding octets is determined by the value of the
                    // > Pad Length field.  If the length of the padding is the length of the
                    // > frame payload or greater, the recipient MUST treat this as a
                    // > connection error (Section 5.4.1) of type PROTOCOL_ERROR.
                    // Note that len(padding) == len(payload) permits an empty payload,
                    // because of that one octet of padding length.
                    Err(ErrorCode::PROTOCOL_ERROR)
                } else {
                    Ok(Some(pad_length))
                }
            },
            None => Err(ErrorCode::PROTOCOL_ERROR),
        }
    } else {
        Ok(None)
    }
}

fn encode_pad_length<W: io::Write>(w: &mut W, pad_length: Option<u8>) -> io::Result<()> {
    if let Some(pad_length) = pad_length {
        w.write_all(&[pad_length])
    } else {
        Ok(())
    }
}

fn encode_padding<W: io::Write>(w: &mut W, pad_length: Option<u8>) -> io::Result<()> {
    if let Some(pad_length) = pad_length {
        w.write_all(&PADDING[..pad_length as usize])
    } else {
        Ok(())
    }
}

fn encode_stream_id<W: io::Write>(w: &mut W, leading_bit: bool, stream_id: StreamId) -> io::Result<()> {
    w.write_all(&[
        (stream_id.0 >> 24) as u8 | if leading_bit { 0b10000000 } else { 0 },
        (stream_id.0 >> 16) as u8,
        (stream_id.0 >> 8) as u8,
        stream_id.0 as u8,
    ])
}

pub mod hpack;

#[test]
fn header_encoding_and_decoding() {
    macro_rules! t {
        ($encoded:expr, $decoded:expr) => {{
            assert_eq!(Header::decode($encoded), $decoded);
            assert_eq!(Header::encode($decoded), $encoded);
        }}
    }

    t!([0, 0, 0, 0, 0, 0, 0, 0, 0],
       Header {
           length: 0,
           type_: 0,
           flags: 0,
           stream_identifier: StreamId(0),
       });

    t!([1, 2, 3, 4, 5, 6, 7, 8, 9],
       Header {
           length: 0x010203,
           type_: 0x04,
           flags: 0x05,
           stream_identifier: StreamId(0x06070809),
       });

    t!([0x01, 0x23, 0x45, 0x67, 0x89, 0x2b, 0xcd, 0xef, 0x10],
       Header {
           length: 0x012345,
           type_: 0x67,
           flags: 0x89,
           stream_identifier: StreamId(0x2bcdef10),
       });

    t!([0xff, 0xff, 0xff, 0xff, 0xff, 0x7f, 0xff, 0xff, 0xff],
       Header {
           length: 0xffffff,
           type_: 0xff,
           flags: 0xff,
           stream_identifier: StreamId(0x7fffffff),
       });

    assert_eq!(Header::decode([0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x10]),
               Header {
                   length: 0x012345,
                   type_: 0x67,
                   flags: 0x89,
                   stream_identifier: StreamId(0x2bcdef10),
               });

    assert_eq!(Header::decode([0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]),
               Header {
                   length: 0xffffff,
                   type_: 0xff,
                   flags: 0xff,
                   stream_identifier: StreamId(0x7fffffff),
               });
}
