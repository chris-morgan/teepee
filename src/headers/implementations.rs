//! Implementations of `Header` and `ToHeader` for various types.

use std::str;
use std::fmt;

use super::{Header, ToHeader};

impl ToHeader for usize {
    fn parse(raw: &[u8]) -> Option<usize> {
        str::from_utf8(raw).ok().and_then(|s| s.parse().ok())
    }
}

impl Header for usize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", *self)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt;
    use headers::{Header, ToHeader, HeaderDisplayAdapter};

    fn eq<H: Header + ToHeader + Eq + fmt::Debug>(raw: &[u8], typed: H) {
        assert_eq!(format!("{}", HeaderDisplayAdapter(&typed)).as_bytes(), raw);
        assert_eq!(H::parse(raw), Some(typed));
    }

    fn bad<H: ToHeader + Eq + fmt::Debug>(raw: &[u8]) {
        assert_eq!(H::parse(raw), None);
    }

    #[test]
    fn test_usize() {
        eq(b"0", 0usize);
        eq(b"1", 1usize);
        eq(b"123456789", 123456789usize);
        bad::<usize>(b"-1");
        bad::<usize>(b"0xdeadbeef");
        bad::<usize>(b"deadbeef");
        bad::<usize>(b"1234567890123467901245790");
        bad::<usize>(b"1,000");
    }
}
