//! The core syntax rules defined in [RFC 7230, section 1.2 Syntax Notation]
//! (https://tools.ietf.org/html/rfc7230#section-1.2):
//!
//! ```ignore
//! The following core rules are included by reference, as defined in
//! [RFC5234], Appendix B.1: ALPHA (letters), CR (carriage return), CRLF
//! (CR LF), CTL (controls), DIGIT (decimal 0-9), DQUOTE (double quote),
//! HEXDIG (hexadecimal 0-9/A-F/a-f), HTAB (horizontal tab), LF (line
//! feed), OCTET (any 8-bit sequence of data), SP (space), and VCHAR (any
//! visible [USASCII] character).
//! ```

/// ALPHA: letters
#[inline]
pub fn is_alpha(octet: u8) -> bool {
    (octet >= b'A' && octet <= b'Z') || (octet >= b'a' && octet <= b'z')
}

/// CR: carriage return
pub const CR: u8 = b'\r';

/// CRLF: CR LF
pub const CRLF: [u8; 2] = [CR, LF];

/// CTL: controls
#[inline]
pub fn is_ctl(octet: u8) -> bool {
    octet < 32 || octet == 127
}

/// DIGIT: decimal 0-9
#[inline]
pub fn is_digit(octet: u8) -> bool {
    octet >= b'0' && octet <= b'9'
}

/// DQUOTE: double quote
pub const DQUOTE: u8 = b'"';

/// HEXDIG: hexadecimal 0–9/A–F/a–f
#[inline]
pub fn is_hexdig(octet: u8) -> bool {
    (octet >= b'A' && octet <= b'F') ||
    (octet >= b'a' && octet <= b'f') ||
    is_digit(octet)
}

/// HTAB: horizontal tab
pub const HTAB: u8 = b'\t';

/// LF: line feed
pub const LF: u8 = b'\n';

/// OCTET: any 8-bit sequence of data (typechecking ensures this to be true)
#[inline]
pub fn is_octet(_: u8) -> bool { true }

/// SP: US-ASCII SP, space (32)
pub const SP: u8 = b' ';

/// VCHAR: any visible US-ASCII character (the inverse of `is_ctl()`)
#[inline]
pub fn is_vchar(octet: u8) -> bool {
    octet > 31 && octet < 127
}
