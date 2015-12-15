//! The SETTINGS frame definition. See [RFC 7540, section 6.5][spec].
//!
//! [spec]: http://tools.ietf.org/html/rfc7540#section-6.5

use std::io;

use ByteTendril;
use super::{Frame, Header, ErrorCode, PayloadSize};

flags! {
    const ACK = 0x1,
}

// This should be kept up to date with the registered settings found in the IANA registry:
// http://www.iana.org/assignments/http2-parameters/http2-parameters.xhtml#settings
const SETTINGS_HEADER_TABLE_SIZE: u16 = 0x1;
const SETTINGS_ENABLE_PUSH: u16 = 0x2;
const SETTINGS_MAX_CONCURRENT_STREAMS: u16 = 0x3;
const SETTINGS_INITIAL_WINDOW_SIZE: u16 = 0x4;
const SETTINGS_MAX_FRAME_SIZE: u16 = 0x5;
const SETTINGS_MAX_HEADER_LIST_SIZE: u16 = 0x6;

/// The SETTINGS frame definition. See [RFC 7540, section 6.5][spec].
///
/// [spec]: http://tools.ietf.org/html/rfc7540#section-6.5
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Settings {
    /// This settings frame is an acknowledgment receipt.
    Acknowledgment,

    /// This settings frame is changing parameters.
    ///
    /// A `None` value in any of the parameters means simply that the value is unchanged.
    /// A `Some` value sets the value.
    Parameters {
        /// The HEADER_TABLE_SIZE setting specifies the maximum size of the header
        /// compression table used to decode header blocks, in octets. The initial value is 4,096.
        header_table_size: Option<u32>,

        /// The ENABLE_PUSH setting can be used to disable server push.
        /// Refer to [the spec][] for details of what this entails. The default is *true*.
        ///
        /// [the spec]: http://tools.ietf.org/html/rfc7540
        enable_push: Option<bool>,

        /// The MAX_CONCURRENT_STREAMS setting indicates the maximum number of concurrent
        /// streams that the sender will allow (thus, the receiver can create up to this many
        /// streams). The default is no limit, and a value of less than 100 is not recommended.
        max_concurrent_streams: Option<u32>,

        /// The INITIAL_WINDOW_SIZE setting indicates the sender’s initial window size (in
        /// octets) for stream-level flow control. The initial value is 65,536 (2¹⁶-1) octets.
        /// The maximum permissible value is 2³¹-1; higher values will produce a FLOW_CONTROL_ERROR
        /// connection error. Be careful when creating a SETTINGS frame manually not to break this.
        initial_window_size: Option<u32>,

        /// The MAX_FRAME_SIZE setting indicates the size of the largest frame payload
        /// that the sender is willing to receive, in octets. The initial value is the minimum
        /// permissible value, 16,384 (2¹⁴) octets. The maximum permissible value is 16,777,215
        /// (2²⁴-1) octets. Be careful when creating a SETTINGS frame manually not to break this.
        max_frame_size: Option<u32>,

        /// The MAX_HEADER_LIST_SIZE setting is advisory only, indicating the maximum
        /// header list size that the sender will accept, in octets, based on an overly complex
        /// and unrealistic (though still not difficult) algorithm defined in the spec, which I’m
        /// not going to reproduce here because it’s mildly insulting in its assumptions of
        /// overhead. That’s my prerogative as the writer of this documentation. What, did you
        /// think I should just have quoted the relevant blocks straight from [RFC 7540, section
        /// 6.5.2][spec] or something? Anyway, the default is unlimited, and a server is at liberty
        /// to balk at a lower limit than it advertises anyway, so this whole setting is really a
        /// complete and utter waste of time and I don’t quite why they bothered. It reminds me of
        /// [chunk extensions][] from the HTTP/1.1 specification: a marvellous idea that was in the
        /// specs from the start and would have been marvellous for things like indicating progress
        /// on a request or response, but *no one* implemented anything but dropping chunk
        /// extensions, and so the whole feature went to waste, acting only as parser bloat and
        /// leaving people to cleverly implement an inferior version of the same feature at a
        /// higher level. Anyway, enough rant, back to the serious business of documenting
        /// code. I hope you enjoyed this. I know I enjoy doing such things from time to time,
        /// which is of course why I do them. — [Chris Morgan](mailto:me@chrismorgan.info)
        ///
        /// [6.5.2]: http://tools.ietf.org/html/rfc7540#section-6.5.2
        /// [chunk extensions]: http://tools.ietf.org/html/rfc7230#section-4.1.1
        max_header_list_size: Option<u32>,
    }
}

impl Frame for Settings {
    type Flags = Flags;
    const TYPE: u8 = 0x4;

    fn decode(header: Header<Flags>, payload: ByteTendril) -> Result<Self, ErrorCode> {
        if header.stream_identifier.0 != 0 {
            return Err(ErrorCode::PROTOCOL_ERROR);
        }
        let len = payload.len32();
        if header.flags.contains(ACK) {
            if len == 0 {
                Ok(Settings::Acknowledgment)
            } else {
                Err(ErrorCode::FRAME_SIZE_ERROR)
            }
        } else {
            // >    A SETTINGS frame with a length other than a multiple of 6 octets MUST
            // >    be treated as a connection error (Section 5.4.1) of type
            // >    FRAME_SIZE_ERROR.
            //
            // >    The payload of a SETTINGS frame consists of zero or more parameters,
            // >    each consisting of an unsigned 16-bit setting identifier and an
            // >    unsigned 32-bit value.
            if len % 6 != 0 {
                return Err(ErrorCode::FRAME_SIZE_ERROR);
            }

            let mut header_table_size = None;
            let mut enable_push = None;
            let mut max_concurrent_streams = None;
            let mut initial_window_size = None;
            let mut max_frame_size = None;
            let mut max_header_list_size = None;

            let payload = &*payload;
            let mut i = 0;
            while i < len {
                let identifier = (payload[i as usize] as u16) << 8 |
                                 payload[i as usize + 1] as u16;
                let value = (payload[i as usize + 2] as u32) << 24 |
                            (payload[i as usize + 3] as u32) << 16 |
                            (payload[i as usize + 4] as u32) << 8 |
                            payload[i as usize + 5] as u32;
                match identifier {
                    SETTINGS_HEADER_TABLE_SIZE => header_table_size = Some(value),

                    SETTINGS_ENABLE_PUSH => {
                        match value {
                            0 => enable_push = Some(false),
                            1 => enable_push = Some(true),
                            _ => return Err(ErrorCode::PROTOCOL_ERROR),
                        }
                    },

                    SETTINGS_MAX_CONCURRENT_STREAMS => max_concurrent_streams = Some(value),

                    // > Values above the maximum flow-control window size of 2^31-1 MUST
                    // > be treated as a connection error (Section 5.4.1) of type
                    // > FLOW_CONTROL_ERROR.
                    SETTINGS_INITIAL_WINDOW_SIZE => {
                        if value > 0x7fffffff {
                            return Err(ErrorCode::FLOW_CONTROL_ERROR);
                        }
                        initial_window_size = Some(value);
                    },

                    // > The initial value is 2^14 (16,384) octets.  The value advertised
                    // > by an endpoint MUST be between this initial value and the maximum
                    // > allowed frame size (2^24-1 or 16,777,215 octets), inclusive.
                    // > Values outside this range MUST be treated as a connection error
                    // > (Section 5.4.1) of type PROTOCOL_ERROR.
                    SETTINGS_MAX_FRAME_SIZE => {
                        if value < 16384 || value > 16_777_215 {
                            return Err(ErrorCode::PROTOCOL_ERROR);
                        }
                        max_frame_size = Some(value);
                    },

                    SETTINGS_MAX_HEADER_LIST_SIZE => max_header_list_size = Some(value),

                    // > An endpoint that receives a SETTINGS frame with any unknown or
                    // > unsupported identifier MUST ignore that setting.
                    _ => (),
                }
                i += 6;
            }

            Ok(Settings::Parameters {
                header_table_size: header_table_size,
                enable_push: enable_push,
                max_concurrent_streams: max_concurrent_streams,
                initial_window_size: initial_window_size,
                max_frame_size: max_frame_size,
                max_header_list_size: max_header_list_size,
            })
        }
    }

    fn len(&self) -> PayloadSize {
        PayloadSize::Exact(match *self {
            Settings::Acknowledgment => 0,
            Settings::Parameters {
                header_table_size,
                enable_push,
                max_concurrent_streams,
                initial_window_size,
                max_frame_size,
                max_header_list_size,
            } => {
                let mut len = 0;
                if header_table_size.is_some() {
                    len += 6;
                }
                if enable_push.is_some() {
                    len += 6;
                }
                if max_concurrent_streams.is_some() {
                    len += 6;
                }
                if initial_window_size.is_some() {
                    len += 6;
                }
                if max_frame_size.is_some() {
                    len += 6;
                }
                if max_header_list_size.is_some() {
                    len += 6;
                }
                len
            }
        })
    }

    fn flags(&self) -> Flags {
        if let Settings::Acknowledgment = *self {
            ACK
        } else {
            Flags::empty()
        }
    }

    fn encode<W: io::Write>(self, w: &mut W) -> io::Result<()> {
        // An acknowledgment as an empty payload.
        if let Settings::Parameters {
            header_table_size,
            enable_push,
            max_concurrent_streams,
            initial_window_size,
            max_frame_size,
            max_header_list_size,
        } = self {
            // Six bytes per setting, six possible settings, maximum write size of 36 bytes.
            let mut buf = [0; 36];
            let mut i = 0;
            macro_rules! w {
                ($value:expr, $identifier:ident) => {
                    if let Some(value) = $value {
                        let value = value as u32;
                        buf[i] = ($identifier >> 8) as u8;
                        buf[i + 1] = $identifier as u8;
                        buf[i + 2] = (value >> 24) as u8;
                        buf[i + 3] = (value >> 16) as u8;
                        buf[i + 4] = (value >> 8) as u8;
                        buf[i + 5] = value as u8;
                        i += 6;
                    }
                }
            }
            w!(header_table_size, SETTINGS_HEADER_TABLE_SIZE);
            w!(enable_push, SETTINGS_ENABLE_PUSH);
            w!(max_concurrent_streams, SETTINGS_MAX_CONCURRENT_STREAMS);
            w!(initial_window_size, SETTINGS_INITIAL_WINDOW_SIZE);
            w!(max_frame_size, SETTINGS_MAX_FRAME_SIZE);
            w!(max_header_list_size, SETTINGS_MAX_HEADER_LIST_SIZE);
            w.write_all(&buf[..i])
        } else {
            Ok(())
        }
    }
}

frame_tests! {
    Settings;

    applies_to_connection {
        flags Flags::empty(),
        stream 1,
        payload [];

        Err(ErrorCode::PROTOCOL_ERROR)
    }

    ack {
        flags ACK,
        stream 0,
        payload [];

        Ok(Settings::Acknowledgment)
    }

    ack_must_be_empty {
        flags ACK,
        stream 0,
        payload [0, 0, 0, 0, 0, 0];

        Err(ErrorCode::FRAME_SIZE_ERROR)
    }

    badly_sized_1 {
        flags Flags::empty(),
        stream 0,
        payload [0];

        Err(ErrorCode::FRAME_SIZE_ERROR)
    }

    badly_sized_2 {
        flags Flags::empty(),
        stream 0,
        payload [0; 5];

        Err(ErrorCode::FRAME_SIZE_ERROR)
    }

    badly_sized_3 {
        flags Flags::empty(),
        stream 0,
        payload [0; 7];

        Err(ErrorCode::FRAME_SIZE_ERROR)
    }

    badly_sized_4 {
        flags Flags::empty(),
        stream 0,
        payload [0; 35];

        Err(ErrorCode::FRAME_SIZE_ERROR)
    }

    badly_sized_5 {
        flags Flags::empty(),
        stream 0,
        payload [0; 37];

        Err(ErrorCode::FRAME_SIZE_ERROR)
    }

    empty {
        flags Flags::empty(),
        stream 0,
        payload [];

        Ok(Settings::Parameters {
            header_table_size: None,
            enable_push: None,
            max_concurrent_streams: None,
            initial_window_size: None,
            max_frame_size: None,
            max_header_list_size: None,
        })
    }

    basic {
        flags Flags::empty(),
        stream 0,
        payload [0, 1, 0x12, 0x34, 0x56, 0x78];

        Ok(Settings::Parameters {
            header_table_size: Some(0x12345678),
            enable_push: None,
            max_concurrent_streams: None,
            initial_window_size: None,
            max_frame_size: None,
            max_header_list_size: None,
        })
    }

    unknown_setting {
        decode only,

        flags Flags::empty(),
        stream 0,
        payload [0; 6];

        Ok(Settings::Parameters {
            header_table_size: None,
            enable_push: None,
            max_concurrent_streams: None,
            initial_window_size: None,
            max_frame_size: None,
            max_header_list_size: None,
        })
    }

    duplicated_setting {
        decode only,

        flags Flags::empty(),
        stream 0,
        payload [0, 1, 0x12, 0x34, 0x56, 0x78,
                 0, 1, 0x9a, 0xbc, 0xde, 0xf0];

        Ok(Settings::Parameters {
            header_table_size: Some(0x9abcdef0),
            enable_push: None,
            max_concurrent_streams: None,
            initial_window_size: None,
            max_frame_size: None,
            max_header_list_size: None,
        })
    }

    long_setting_decode {
        decode only,

        flags Flags::empty(),
        stream 0,
        payload [0, 1, 0x12, 0x34, 0x56, 0x78,
                 0, 6, 0x56, 0x78, 0x9a, 0xbc,
                 0, 3, 0x23, 0x45, 0x67, 0x89,
                 9, 8, 0x00, 0x00, 0x00, 0x00,
                 0, 4, 0x34, 0x56, 0x78, 0x9a,
                 0, 5, 0x00, 0x67, 0x89, 0xab,
                 0, 2, 0x00, 0x00, 0x00, 0x01,
                 1, 2, 0x00, 0x00, 0x00, 0x00];

        Ok(Settings::Parameters {
            header_table_size: Some(0x12345678),
            enable_push: Some(true),
            max_concurrent_streams: Some(0x23456789),
            initial_window_size: Some(0x3456789a),
            max_frame_size: Some(0x6789ab),
            max_header_list_size: Some(0x56789abc),
        })
    }

    long_setting {
        flags Flags::empty(),
        stream 0,
        payload [0, 1, 0x12, 0x34, 0x56, 0x78,
                 0, 2, 0x00, 0x00, 0x00, 0x01,
                 0, 3, 0x23, 0x45, 0x67, 0x89,
                 0, 4, 0x34, 0x56, 0x78, 0x9a,
                 0, 5, 0x00, 0x67, 0x89, 0xab,
                 0, 6, 0x56, 0x78, 0x9a, 0xbc];

        Ok(Settings::Parameters {
            header_table_size: Some(0x12345678),
            enable_push: Some(true),
            max_concurrent_streams: Some(0x23456789),
            initial_window_size: Some(0x3456789a),
            max_frame_size: Some(0x6789ab),
            max_header_list_size: Some(0x56789abc),
        })
    }

    enable_push_false {
        flags Flags::empty(),
        stream 0,
        payload [0, 2, 0x00, 0x00, 0x00, 0x00];

        Ok(Settings::Parameters {
            header_table_size: None,
            enable_push: Some(false),
            max_concurrent_streams: None,
            initial_window_size: None,
            max_frame_size: None,
            max_header_list_size: None,
        })
    }

    bad_enable_push {
        flags Flags::empty(),
        stream 0,
        payload [0, 2, 0x12, 0x34, 0x56, 0x78];

        Err(ErrorCode::PROTOCOL_ERROR)
    }

    maximal_initial_window_size {
        flags Flags::empty(),
        stream 0,
        payload [0, 4, 0x7f, 0xff, 0xff, 0xff];

        Ok(Settings::Parameters {
            header_table_size: None,
            enable_push: None,
            max_concurrent_streams: None,
            initial_window_size: Some(0x7fffffff),
            max_frame_size: None,
            max_header_list_size: None,
        })
    }

    excessive_initial_window_size {
        flags Flags::empty(),
        stream 0,
        payload [0, 4, 0x80, 0x00, 0x00, 0x00];

        Err(ErrorCode::FLOW_CONTROL_ERROR)
    }

    maximal_max_frame_size {
        flags Flags::empty(),
        stream 0,
        payload [0, 5, 0x00, 0xff, 0xff, 0xff];

        Ok(Settings::Parameters {
            header_table_size: None,
            enable_push: None,
            max_concurrent_streams: None,
            initial_window_size: None,
            max_frame_size: Some(0x00ffffff),
            max_header_list_size: None,
        })
    }

    excessively_large_max_frame_size {
        flags Flags::empty(),
        stream 0,
        payload [0, 5, 0x01, 0x00, 0x00, 0x00];

        Err(ErrorCode::PROTOCOL_ERROR)
    }

    minimal_max_frame_size {
        flags Flags::empty(),
        stream 0,
        payload [0, 5, 0x00, 0x00, 0x40, 0x00];

        Ok(Settings::Parameters {
            header_table_size: None,
            enable_push: None,
            max_concurrent_streams: None,
            initial_window_size: None,
            max_frame_size: Some(0x00004000),
            max_header_list_size: None,
        })
    }

    excessively_small_max_frame_size {
        flags Flags::empty(),
        stream 0,
        payload [0, 5, 0x00, 0x00, 0x3f, 0xff];

        Err(ErrorCode::PROTOCOL_ERROR)
    }
}
