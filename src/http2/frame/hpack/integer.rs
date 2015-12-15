//! The HPACK integer literal representation (RFC 7541, section 5.1).

use std::num::Wrapping;
use std::io;
use super::DecodeError;
use ByteTendril;

macro_rules! decode_n {
    ($name:ident, $doc:meta, $mask:expr) => {
        #[$doc]
        #[doc = ""]
        #[doc = "Returns the decoded number, consumed from the tendril."]
        #[doc = ""]
        #[doc = "A decoding error is returned if any of these situations is encountered:"]
        #[doc = ""]
        #[doc = "- The number overflows the output type;"]
        #[doc = "- The input finishes before the number is completely read (e.g. empty input)."]
        #[doc = ""]
        #[doc = "Nothing can be done if you hit a decoding error. You should give up."]
        #[inline]
        pub fn $name(input: &mut ByteTendril) -> Result<u32, DecodeError> {
            decode_masked($mask, input)
        }
    }
}
#[cfg(test)] decode_n!(decode8, doc = "Decode a primitive integer for N = 8.", 0b11111111);
decode_n!(decode7, doc = "Decode a primitive integer for N = 7.", 0b01111111);
decode_n!(decode6, doc = "Decode a primitive integer for N = 6.", 0b00111111);
decode_n!(decode5, doc = "Decode a primitive integer for N = 5.", 0b00011111);
decode_n!(decode4, doc = "Decode a primitive integer for N = 4.", 0b00001111);
//decode_n!(decode3, doc = "Decode a primitive integer for N = 3.", 0b00000111);
//decode_n!(decode2, doc = "Decode a primitive integer for N = 2.", 0b00000011);
//decode_n!(decode1, doc = "Decode a primitive integer for N = 1.", 0b00000001);

fn decode_masked(n_mask: u8, input: &mut ByteTendril) -> Result<u32, DecodeError> {
    let mut pop = 0;
    let mut i;
    'out_of_jail: loop {
        let mut octets = input.iter().map(|&b| b);
        let prefix = match octets.next() {
            Some(prefix) => prefix,
            None => return Err(DecodeError),
        };
        i = (prefix & n_mask) as u32;
        pop += 1;
        if i == n_mask as u32 {
            let mut m = 0;
            let mut m_mask = 0b1111111;
            for b in octets {
                // Poor man’s checked_shl. Seriously, we don’t have this!?
                let x = (Wrapping((b & 127) as u32) << m).0;
                if x & m_mask != x {
                    return Err(DecodeError);  // overflow
                }
                i = match i.checked_add(x) {
                    Some(i) => i,
                    None => return Err(DecodeError),  // overflow
                };
                m_mask <<= 7;
                // This check might seem desirable in case the user tries stuffing zeroes at us,
                // but in practice working with HTTP headers we’ve already limited that vector.
                //if m_mask == 0 {
                //    return Err(DecodeError),  // overflow
                //}
                m += 7;
                pop += 1;
                if b & 0b10000000 == 0 {
                    break 'out_of_jail;
                }
            }
            return Err(DecodeError);  // overflow
        }
        break;
    }
    input.pop_front(pop);
    Ok(i)
}

#[test]
fn test_decode() {
    macro_rules! as_expr { ($x:expr) => ($x) }
    macro_rules! t {
        ($method:ident($input:expr) => $expected:expr, $bytes_left:tt bytes left) => {{
            let input: &[u8] = &$input;
            let mut tendril = ByteTendril::from(input);
            assert_eq!($method(&mut tendril), $expected);
            assert_eq!(tendril.len32(), as_expr!($bytes_left));
        }}
    }
    t!(decode5([0b11101010]) => Ok(10), 0 bytes left);
    t!(decode5([0b00001010]) => Ok(10), 0 bytes left);
    t!(decode5([0b11111111, 0b10011010, 0b00001010]) => Ok(1337), 0 bytes left);
    t!(decode8([0b00101010]) => Ok(42), 0 bytes left);
}

macro_rules! encode_n {
    ($name:ident, $doc:meta, $mask:expr) => {
        #[$doc]
        #[inline]
        pub fn $name<W: io::Write>(w: &mut W, leading_bits: u8, i: u32) -> io::Result<()> {
            encode_masked(w, $mask, leading_bits, i)
        }
    }
}

#[cfg(test)] encode_n!(encode8, doc = "Encode for N = 8.", 0b11111111);
encode_n!(encode7, doc = "Encode for N = 7.", 0b01111111);
encode_n!(encode6, doc = "Encode for N = 6.", 0b00111111);
encode_n!(encode5, doc = "Encode for N = 5.", 0b00011111);
encode_n!(encode4, doc = "Encode for N = 4.", 0b00001111);
//encode_n!(encode3, doc = "Encode for N = 3.", 0b00000111);
//encode_n!(encode2, doc = "Encode for N = 2.", 0b00000011);
//encode_n!(encode1, doc = "Encode for N = 1.", 0b00000001);

fn encode_masked<W>(w: &mut W, n_mask: u8, leading_bits: u8, mut i: u32) -> io::Result<()>
where W: io::Write {
    debug_assert!(leading_bits & !n_mask == leading_bits,
                  "leading_bits has more than n bits full");
    if i < n_mask as u32 {
        w.write_all(&[leading_bits | i as u8])
    } else {
        i -= n_mask as u32;
        let v1 = leading_bits | n_mask;
        let v2 = i as u8;
        match i {
            // 0...2⁷ - 1
            0...127 => w.write_all(&[v1, v2]),
            // 2⁷...2¹⁴ - 1
            128...16383 => w.write_all(&[v1, v2 | 128, (i >> 7) as u8]),
            // 2¹⁴...2²¹ - 1
            16384...2097151 => w.write_all(&[v1, v2 | 128, (i >> 7) as u8 | 128, (i >> 14) as u8]),
            // 2²¹...2²⁸ - 1
            2097152...268435455 => w.write_all(&[v1, v2 | 128, (i >> 7) as u8 | 128,
                                             (i >> 14) as u8 | 128, (i >> 21) as u8]),
            // 2²⁸...2³² - 1
            _ => w.write_all(&[v1, v2 | 128, (i >> 7) as u8 | 128,
                           (i >> 14) as u8 | 128, (i >> 21) as u8 | 128, (i >> 28) as u8]),
        }

        /*let v1 = leading_bits | n_mask;
        i -= n_mask as u32;
        let v2 = i as u8;
        if i < 128 {  // 0...2⁷ - 1
            w.write_all(&[v1, v2])
        } else {
            let v2 |= 128;
            let v3 = (i >> 7) as u8;
            if i < 16384 {  // 2⁷...2¹⁴ - 1
                w.write_all(&[v1, v2, v3])
            } else {
                let v3 |= 128;
                let v4 = (i >> 14) as u8;
                if i < 2097152 { // 2¹⁴...2²¹ - 1
                    w.write_all(&[v1, v2, v3, v4])
                } else {
                    let v4 |= 128;
                    let v5 = (i >> 21) as u8;
                    if i < 268435456 {  // 2²¹...2²⁸ - 1
                        w.write_all(&[v1, v2, v3, v4, v5])
                    } else {  // 2²⁸...2³² - 1
                        let v5 |= 128;
                        let v6 = (i >> 28) as u8;
                        w.write_all(&[v1, v2, v3, v4, v5, v6])
                    }
                }
            }
        }*/
    }
}

#[test]
fn test_encode() {
    macro_rules! t {
        ($method:ident($leading_bits:expr, $input:expr) => $expected:expr) => {{
            let mut output = vec![];
            let expected: &[u8] = &$expected;
            assert!($method(&mut output, $leading_bits, $input).is_ok());
            assert_eq!(&*output, expected);
        }}
    }
    t!(encode5(0b11100000, 10) => [0b11101010]);
    t!(encode5(0b00000000, 10) => [0b00001010]);
    t!(encode5(0b11100000, 1337) => [0b11111111, 0b10011010, 0b00001010]);
    t!(encode5(0b00000000, 1337) => [0b00011111, 0b10011010, 0b00001010]);
    t!(encode8(0b00000000, 42) => [0b00101010]);
}
