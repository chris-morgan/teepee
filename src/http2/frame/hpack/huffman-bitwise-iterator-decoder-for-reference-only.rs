// This is an unrolled bitwise-iterator decoder.
// It’s substantially faster than a HashMap bitwise approach (probably the most obvious and
// simplest approach), 10–50× faster than one such implementation in Rust.
// But wait! One can do better still! Large computed lookup tables allows bytewise iteration with
// far less branching; such an implementation (depending on 130KB of computed lookup tables) is
// twice as fast again as this approach. Still, I’ll keep this file around for reference.

/// The HPACK Huffman decoder.
///
/// Give it a byte iterator and it’ll yield the bytes.
pub struct HuffmanDecoder<I: Iterator<Item = u8>> {
    iter: Bits<I>,
}

impl<I: Iterator<Item = u8>> HuffmanDecoder<I> {
    /// Constructs a new `HuffmanDecoder` from the given byte iterator.
    pub fn new(iter: I) -> HuffmanDecoder<I> {
        HuffmanDecoder {
            iter: Bits::new(iter),
        }
    }
}

/// A decode error is returned if either of these situations is encountered:
///
/// - The EOS token (30 1 bits in a row) appears in the input;
/// - Illegal padding (more than seven bits, or padding of other than ones) occurrs,
///   probably indicating an incomplete input.
///
/// Nothing can be done if you hit a decoding error. You should give up.
impl<I: Iterator<Item = u8>> Iterator for HuffmanDecoder<I> {
    type Item = Result<u8, DecodeError>;

    fn next(&mut self) -> Option<Result<u8, DecodeError>> {
        macro_rules! optional_bit_pattern {
            (0) => (Some(false)); 
            (1) => (Some(true)); 
            (-) => (None); 
            (_) => (_);
        }

        // Optional bits pattern
        macro_rules! b {
            ($($b:tt)*) => (($(optional_bit_pattern!($b),)*));
        }

        macro_rules! next {
            (1 bit) => ((self.iter.next(),));
            (2 bits) => ((self.iter.next(), self.iter.next()));
            (3 bits) => ((self.iter.next(), self.iter.next(), self.iter.next()));
            (4 bits) => ((self.iter.next(), self.iter.next(), self.iter.next(), self.iter.next()));
            (5 bits) => ((self.iter.next(), self.iter.next(), self.iter.next(), self.iter.next(),
                          self.iter.next()));
        }

        macro_rules! next1 {
            ($zero:expr, $one:expr) => {
                match next!(1 bit) {
                    b!(0) => Some(Ok($zero)),
                    b!(1) => Some(Ok($one)),
                    b!(-) => Some(Err(DecodeError)),
                }
            }
        }

        match next!(5 bits) {
            b!(0 0 0 0 0) => Some(Ok(b'0')),
            b!(0 0 0 0 1) => Some(Ok(b'1')),
            b!(0 0 0 1 0) => Some(Ok(b'2')),
            b!(0 0 0 1 1) => Some(Ok(b'a')),
            b!(0 0 1 0 0) => Some(Ok(b'c')),
            b!(0 0 1 0 1) => Some(Ok(b'e')),
            b!(0 0 1 1 0) => Some(Ok(b'i')),
            b!(0 0 1 1 1) => Some(Ok(b'o')),
            b!(0 1 0 0 0) => Some(Ok(b's')),
            b!(0 1 0 0 1) => Some(Ok(b't')),
            b!(0 1 0 1 0) => next1!(b' ', b'%'),
            b!(0 1 0 1 1) => next1!(b'-', b'.'),
            b!(0 1 1 0 0) => next1!(b'/', b'3'),
            b!(0 1 1 0 1) => next1!(b'4', b'5'),
            b!(0 1 1 1 0) => next1!(b'6', b'7'),
            b!(0 1 1 1 1) => next1!(b'8', b'9'),
            b!(1 0 0 0 0) => next1!(b'=', b'A'),
            b!(1 0 0 0 1) => next1!(b'_', b'b'),
            b!(1 0 0 1 0) => next1!(b'd', b'f'),
            b!(1 0 0 1 1) => next1!(b'g', b'h'),
            b!(1 0 1 0 0) => next1!(b'l', b'm'),
            b!(1 0 1 0 1) => next1!(b'n', b'p'),
            b!(1 0 1 1 0) => next1!(b'r', b'u'),
            b!(1 0 1 1 1) => match next!(2 bits) {
                b!(0 0) => Some(Ok(b':')),
                b!(0 1) => Some(Ok(b'B')),
                b!(1 0) => Some(Ok(b'C')),
                b!(1 1) => Some(Ok(b'D')),
                _ => Some(Err(DecodeError)),
            },
            b!(1 1 0 0 0) => match next!(2 bits) {
                b!(0 0) => Some(Ok(b'E')),
                b!(0 1) => Some(Ok(b'F')),
                b!(1 0) => Some(Ok(b'G')),
                b!(1 1) => Some(Ok(b'H')),
                _ => Some(Err(DecodeError)),
            },
            b!(1 1 0 0 1) => match next!(2 bits) {
                b!(0 0) => Some(Ok(b'I')),
                b!(0 1) => Some(Ok(b'J')),
                b!(1 0) => Some(Ok(b'K')),
                b!(1 1) => Some(Ok(b'L')),
                _ => Some(Err(DecodeError)),
            },
            b!(1 1 0 1 0) => match next!(2 bits) {
                b!(0 0) => Some(Ok(b'M')),
                b!(0 1) => Some(Ok(b'N')),
                b!(1 0) => Some(Ok(b'O')),
                b!(1 1) => Some(Ok(b'P')),
                _ => Some(Err(DecodeError)),
            },
            b!(1 1 0 1 1) => match next!(2 bits) {
                b!(0 0) => Some(Ok(b'Q')),
                b!(0 1) => Some(Ok(b'R')),
                b!(1 0) => Some(Ok(b'S')),
                b!(1 1) => Some(Ok(b'T')),
                _ => Some(Err(DecodeError)),
            },
            b!(1 1 1 0 0) => match next!(2 bits) {
                b!(0 0) => Some(Ok(b'U')),
                b!(0 1) => Some(Ok(b'V')),
                b!(1 0) => Some(Ok(b'W')),
                b!(1 1) => Some(Ok(b'Y')),
                _ => Some(Err(DecodeError)),
            },
            b!(1 1 1 0 1) => match next!(2 bits) {
                b!(0 0) => Some(Ok(b'j')),
                b!(0 1) => Some(Ok(b'k')),
                b!(1 0) => Some(Ok(b'q')),
                b!(1 1) => Some(Ok(b'v')),
                _ => Some(Err(DecodeError)),
            },
            b!(1 1 1 1 0) => match next!(2 bits) {
                b!(0 0) => Some(Ok(b'w')),
                b!(0 1) => Some(Ok(b'x')),
                b!(1 0) => Some(Ok(b'y')),
                b!(1 1) => Some(Ok(b'z')),
                _ => Some(Err(DecodeError)),
            },
            b!(1 1 1 1 1) => match next!(3 bits) {
                b!(0 0 0) => Some(Ok(b'&')),
                b!(0 0 1) => Some(Ok(b'*')),
                b!(0 1 0) => Some(Ok(b',')),
                b!(0 1 1) => Some(Ok(b';')),
                b!(1 0 0) => Some(Ok(b'X')),
                b!(1 0 1) => Some(Ok(b'Z')),
                b!(1 1 0) => match next!(2 bits) {
                    b!(0 0) => Some(Ok(b'!')),
                    b!(0 1) => Some(Ok(b'"')),
                    b!(1 0) => Some(Ok(b'(')),
                    b!(1 1) => Some(Ok(b')')),
                    _ => Some(Err(DecodeError)),
                },
                b!(1 1 1) => match next!(2 bits) {
                    b!(0 0) => Some(Ok(b'?')),
                    b!(0 1) => next1!(b'\'', b'+'),
                    b!(1 0) => match next!(1 bit) {
                        b!(0) => Some(Ok(b'|')),
                        b!(1) => next1!(b'#', b'>'),
                        _ => Some(Err(DecodeError)),
                    },
                    b!(1 1) => match next!(3 bits) {
                        b!(0 0 0) => Some(Ok(0)),
                        b!(0 0 1) => Some(Ok(b'$')),
                        b!(0 1 0) => Some(Ok(b'@')),
                        b!(0 1 1) => Some(Ok(b'[')),
                        b!(1 0 0) => Some(Ok(b']')),
                        b!(1 0 1) => Some(Ok(b'~')),
                        b!(1 1 0) => next1!(b'^', b'}'),
                        b!(1 1 1) => match next!(2 bits) {
                            b!(0 0) => Some(Ok(b'<')),
                            b!(0 1) => Some(Ok(b'`')),
                            b!(1 0) => Some(Ok(b'{')),
                            b!(1 1) => match next!(4 bits) {
                                b!(0 0 0 0) => Some(Ok(b'\\')),
                                b!(0 0 0 1) => Some(Ok(195)),
                                b!(0 0 1 0) => Some(Ok(208)),
                                b!(0 0 1 1) => next1!(128, 130),
                                b!(0 1 0 0) => next1!(131, 162),
                                b!(0 1 0 1) => next1!(184, 194),
                                b!(0 1 1 0) => next1!(224, 226),
                                b!(0 1 1 1) => match next!(2 bits) {
                                    b!(0 0) => Some(Ok(153)),
                                    b!(0 1) => Some(Ok(161)),
                                    b!(1 0) => Some(Ok(167)),
                                    b!(1 1) => Some(Ok(172)),
                                    _ => Some(Err(DecodeError)),
                                },
                                b!(1 0 0 0) => match next!(2 bits) {
                                    b!(0 0) => Some(Ok(176)),
                                    b!(0 1) => Some(Ok(177)),
                                    b!(1 0) => Some(Ok(179)),
                                    b!(1 1) => Some(Ok(209)),
                                    _ => Some(Err(DecodeError)),
                                },
                                b!(1 0 0 1) => match next!(2 bits) {
                                    b!(0 0) => Some(Ok(216)),
                                    b!(0 1) => Some(Ok(217)),
                                    b!(1 0) => Some(Ok(227)),
                                    b!(1 1) => Some(Ok(229)),
                                    _ => Some(Err(DecodeError)),
                                },
                                b!(1 0 1 0) => match next!(2 bits) {
                                    b!(0 0) => Some(Ok(230)),
                                    b!(0 1) => next1!(129, 132),
                                    b!(1 0) => next1!(133, 134),
                                    b!(1 1) => next1!(136, 146),
                                    _ => Some(Err(DecodeError)),
                                },
                                b!(1 0 1 1) => match next!(3 bits) {
                                    b!(0 0 0) => Some(Ok(154)),
                                    b!(0 0 1) => Some(Ok(156)),
                                    b!(0 1 0) => Some(Ok(160)),
                                    b!(0 1 1) => Some(Ok(163)),
                                    b!(1 0 0) => Some(Ok(164)),
                                    b!(1 0 1) => Some(Ok(169)),
                                    b!(1 1 0) => Some(Ok(170)),
                                    b!(1 1 1) => Some(Ok(173)),
                                    _ => Some(Err(DecodeError)),
                                },
                                b!(1 1 0 0) => match next!(3 bits) {
                                    b!(0 0 0) => Some(Ok(178)),
                                    b!(0 0 1) => Some(Ok(181)),
                                    b!(0 1 0) => Some(Ok(185)),
                                    b!(0 1 1) => Some(Ok(186)),
                                    b!(1 0 0) => Some(Ok(187)),
                                    b!(1 0 1) => Some(Ok(189)),
                                    b!(1 1 0) => Some(Ok(190)),
                                    b!(1 1 1) => Some(Ok(196)),
                                    _ => Some(Err(DecodeError)),
                                },
                                b!(1 1 0 1) => match next!(3 bits) {
                                    b!(0 0 0) => Some(Ok(198)),
                                    b!(0 0 1) => Some(Ok(228)),
                                    b!(0 1 0) => Some(Ok(232)),
                                    b!(0 1 1) => Some(Ok(233)),
                                    b!(1 0 0) => next1!(1, 135),
                                    b!(1 0 1) => next1!(137, 138),
                                    b!(1 1 0) => next1!(139, 140),
                                    b!(1 1 1) => next1!(141, 143),
                                    _ => Some(Err(DecodeError)),
                                },
                                b!(1 1 1 0) => match next!(4 bits) {
                                    b!(0 0 0 0) => Some(Ok(147)),
                                    b!(0 0 0 1) => Some(Ok(149)),
                                    b!(0 0 1 0) => Some(Ok(150)),
                                    b!(0 0 1 1) => Some(Ok(151)),
                                    b!(0 1 0 0) => Some(Ok(152)),
                                    b!(0 1 0 1) => Some(Ok(155)),
                                    b!(0 1 1 0) => Some(Ok(157)),
                                    b!(0 1 1 1) => Some(Ok(158)),
                                    b!(1 0 0 0) => Some(Ok(165)),
                                    b!(1 0 0 1) => Some(Ok(166)),
                                    b!(1 0 1 0) => Some(Ok(168)),
                                    b!(1 0 1 1) => Some(Ok(174)),
                                    b!(1 1 0 0) => Some(Ok(175)),
                                    b!(1 1 0 1) => Some(Ok(180)),
                                    b!(1 1 1 0) => Some(Ok(182)),
                                    b!(1 1 1 1) => Some(Ok(183)),
                                    _ => Some(Err(DecodeError)),
                                },
                                b!(1 1 1 1) => match next!(4 bits) {
                                    b!(0 0 0 0) => Some(Ok(188)),
                                    b!(0 0 0 1) => Some(Ok(191)),
                                    b!(0 0 1 0) => Some(Ok(197)),
                                    b!(0 0 1 1) => Some(Ok(231)),
                                    b!(0 1 0 0) => Some(Ok(239)),
                                    b!(0 1 0 1) => next1!(9, 142),
                                    b!(0 1 1 0) => next1!(144, 145),
                                    b!(0 1 1 1) => next1!(148, 159),
                                    b!(1 0 0 0) => next1!(171, 206),
                                    b!(1 0 0 1) => next1!(215, 225),
                                    b!(1 0 1 0) => next1!(236, 237),
                                    b!(1 0 1 1) => match next!(2 bits) {
                                        b!(0 0) => Some(Ok(199)),
                                        b!(0 1) => Some(Ok(207)),
                                        b!(1 0) => Some(Ok(234)),
                                        b!(1 1) => Some(Ok(235)),
                                        _ => Some(Err(DecodeError)),
                                    },
                                    b!(1 1 0 0) => match next!(3 bits) {
                                        b!(0 0 0) => Some(Ok(192)),
                                        b!(0 0 1) => Some(Ok(193)),
                                        b!(0 1 0) => Some(Ok(200)),
                                        b!(0 1 1) => Some(Ok(201)),
                                        b!(1 0 0) => Some(Ok(202)),
                                        b!(1 0 1) => Some(Ok(205)),
                                        b!(1 1 0) => Some(Ok(210)),
                                        b!(1 1 1) => Some(Ok(213)),
                                        _ => Some(Err(DecodeError)),
                                    },
                                    b!(1 1 0 1) => match next!(3 bits) {
                                        b!(0 0 0) => Some(Ok(218)),
                                        b!(0 0 1) => Some(Ok(219)),
                                        b!(0 1 0) => Some(Ok(238)),
                                        b!(0 1 1) => Some(Ok(240)),
                                        b!(1 0 0) => Some(Ok(242)),
                                        b!(1 0 1) => Some(Ok(243)),
                                        b!(1 1 0) => Some(Ok(255)),
                                        b!(1 1 1) => next1!(203, 204),
                                        _ => Some(Err(DecodeError)),
                                    },
                                    b!(1 1 1 0) => match next!(4 bits) {
                                        b!(0 0 0 0) => Some(Ok(211)),
                                        b!(0 0 0 1) => Some(Ok(212)),
                                        b!(0 0 1 0) => Some(Ok(214)),
                                        b!(0 0 1 1) => Some(Ok(221)),
                                        b!(0 1 0 0) => Some(Ok(222)),
                                        b!(0 1 0 1) => Some(Ok(223)),
                                        b!(0 1 1 0) => Some(Ok(241)),
                                        b!(0 1 1 1) => Some(Ok(244)),
                                        b!(1 0 0 0) => Some(Ok(245)),
                                        b!(1 0 0 1) => Some(Ok(246)),
                                        b!(1 0 1 0) => Some(Ok(247)),
                                        b!(1 0 1 1) => Some(Ok(248)),
                                        b!(1 1 0 0) => Some(Ok(250)),
                                        b!(1 1 0 1) => Some(Ok(251)),
                                        b!(1 1 1 0) => Some(Ok(252)),
                                        b!(1 1 1 1) => Some(Ok(253)),
                                        _ => Some(Err(DecodeError)),
                                    },
                                    b!(1 1 1 1) => match next!(4 bits) {
                                        b!(0 0 0 0) => Some(Ok(254)),
                                        b!(0 0 0 1) => next1!(2, 3),
                                        b!(0 0 1 0) => next1!(4, 5),
                                        b!(0 0 1 1) => next1!(6, 7),
                                        b!(0 1 0 0) => next1!(8, 11),
                                        b!(0 1 0 1) => next1!(12, 14),
                                        b!(0 1 1 0) => next1!(15, 16),
                                        b!(0 1 1 1) => next1!(17, 18),
                                        b!(1 0 0 0) => next1!(19, 20),
                                        b!(1 0 0 1) => next1!(21, 23),
                                        b!(1 0 1 0) => next1!(24, 25),
                                        b!(1 0 1 1) => next1!(26, 27),
                                        b!(1 1 0 0) => next1!(28, 29),
                                        b!(1 1 0 1) => next1!(30, 31),
                                        b!(1 1 1 0) => next1!(127, 220),
                                        b!(1 1 1 1) => match next!(1 bit) {
                                            b!(0) => Some(Ok(249)),
                                            b!(1) => match next!(2 bits) {
                                                b!(0 0) => Some(Ok(10)),
                                                b!(0 1) => Some(Ok(13)),
                                                b!(1 0) => Some(Ok(22)),
                                                b!(1 1) => Some(Err(DecodeError)),  // EOS
                                                _ => Some(Err(DecodeError)),
                                            },
                                            _ => Some(Err(DecodeError)),
                                        },
                                        _ => Some(Err(DecodeError)),
                                    },
                                    _ => Some(Err(DecodeError)),
                                },
                                _ => Some(Err(DecodeError)),
                            },
                            _ => Some(Err(DecodeError)),
                        },
                        _ => Some(Err(DecodeError)),
                    },
                    _ => Some(Err(DecodeError)),
                },
                b!(- - -) |
                b!(1 - -) |
                b!(1 1 -) => None,  // up to seven bits of padding
                _ => Some(Err(DecodeError)),
            },
            b!(- - - - -) |
            b!(1 - - - -) |
            b!(1 1 - - -) |
            b!(1 1 1 - -) |
            b!(1 1 1 1 -) => None,  // up to seven bits of padding
            _ => Some(Err(DecodeError)),
        }
    }
}

struct Bits<I: Iterator<Item = u8>> {
    iter: I,
    // current_octet would be Option<u8>, but that would be less efficient. As it is, mask is a u8
    // with exactly one bit set if current_octet would be Some, and with no bits set if it would be
    // None. We thus inline the discriminant.
    // (Option<NonZero<(u8, u8)>> for the pair of them would do as well, but is unstable and I
    // would prefer to minimise unstable dependencies; this is an easy place to do that.)
    current_octet: u8,
    mask: u8,
}

impl<I: Iterator<Item = u8>> Bits<I> {
    fn new(iter: I) -> Bits<I> {
        Bits {
            iter: iter,
            current_octet: 0,
            mask: 0b00000000,
        }
    }
}

impl<I: Iterator<Item = u8>> Iterator for Bits<I> {
    type Item = bool;

    fn next(&mut self) -> Option<bool> {
        if self.mask == 0 {
            match self.iter.next() {
                Some(octet) => {
                    self.current_octet = octet;
                    self.mask = 0b10000000;
                },
                None => return None,
            }
        }
        let out = self.current_octet & self.mask != 0;
        self.mask >>= 1;
        Some(out)
    }
}
