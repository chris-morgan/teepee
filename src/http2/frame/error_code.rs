//! Error Codes ([RFC 7540, section 7][spec]).
//!
//! [spec]: http://tools.ietf.org/html/rfc7540#section-7

use std::fmt;

/// An error code (a 32-bit quantity; any value is permitted).
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ErrorCode(pub u32);

impl fmt::Debug for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorCode::NO_ERROR => f.write_str("NO_ERROR"),
            ErrorCode::PROTOCOL_ERROR => f.write_str("PROTOCOL_ERROR"),
            ErrorCode::INTERNAL_ERROR => f.write_str("INTERNAL_ERROR"),
            ErrorCode::FLOW_CONTROL_ERROR => f.write_str("FLOW_CONTROL_ERROR"),
            ErrorCode::SETTINGS_TIMEOUT => f.write_str("SETTINGS_TIMEOUT"),
            ErrorCode::STREAM_CLOSED => f.write_str("STREAM_CLOSED"),
            ErrorCode::FRAME_SIZE_ERROR => f.write_str("FRAME_SIZE_ERROR"),
            ErrorCode::REFUSED_STREAM => f.write_str("REFUSED_STREAM"),
            ErrorCode::CANCEL => f.write_str("CANCEL"),
            ErrorCode::COMPRESSION_ERROR => f.write_str("COMPRESSION_ERROR"),
            ErrorCode::CONNECT_ERROR => f.write_str("CONNECT_ERROR"),
            ErrorCode::ENHANCE_YOUR_CALM => f.write_str("ENHANCE_YOUR_CALM"),
            ErrorCode::INADEQUATE_SECURITY => f.write_str("INADEQUATE_SECURITY"),
            ErrorCode::HTTP_1_1_REQUIRED => f.write_str("HTTP_1_1_REQUIRED"),
            ErrorCode(code) => write!(f, "ErrorCode({})", code),
        }
    }
}

// The descriptions are taken from RFC 7540, Section 11.4 (Error Code Registry).
// This should be kept up to date with the registered error codes found in the IANA registry:
// http://www.iana.org/assignments/http2-parameters/http2-parameters.xhtml#error-code
impl ErrorCode {
    /// Graceful shutdown
    pub const NO_ERROR: ErrorCode = ErrorCode(0x0);

    /// Protocol error detected
    pub const PROTOCOL_ERROR: ErrorCode = ErrorCode(0x1);

    /// Implementation fault
    pub const INTERNAL_ERROR: ErrorCode = ErrorCode(0x2);

    /// Flow-control limits exceeded
    pub const FLOW_CONTROL_ERROR: ErrorCode = ErrorCode(0x3);

    /// Settings not acknowledged
    pub const SETTINGS_TIMEOUT: ErrorCode = ErrorCode(0x4);

    /// Frame received for closed stream
    pub const STREAM_CLOSED: ErrorCode = ErrorCode(0x5);

    /// Frame size incorrect
    pub const FRAME_SIZE_ERROR: ErrorCode = ErrorCode(0x6);

    /// Stream not processed
    pub const REFUSED_STREAM: ErrorCode = ErrorCode(0x7);

    /// Stream cancelled
    pub const CANCEL: ErrorCode = ErrorCode(0x8);

    /// Compression state not updated
    pub const COMPRESSION_ERROR: ErrorCode = ErrorCode(0x9);

    /// TCP connection error for CONNECT method
    pub const CONNECT_ERROR: ErrorCode = ErrorCode(0xa);

    /// Processing capacity exceeded
    pub const ENHANCE_YOUR_CALM: ErrorCode = ErrorCode(0xb);

    /// Negotiated TLS parameters not acceptable
    pub const INADEQUATE_SECURITY: ErrorCode = ErrorCode(0xc);

    /// Use HTTP/1.1 for the request
    pub const HTTP_1_1_REQUIRED: ErrorCode = ErrorCode(0xd);
}
