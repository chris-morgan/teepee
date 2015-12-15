//! An implementation of HPACK: Header Compression for HTTP/2 (RFC 7541).

use std::collections::VecDeque;
use std::io;
use std::vec;
use TendrilSliceExt;
use ByteTendril;

mod integer;
mod string;

/// An arbitrary decode error. No details are retained on account of how all such errors are
/// unrecoverable and I’m not interested in lowering my efficiency so you can debug a bad HPACK
/// implementation a shade more easily.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DecodeError;

/// `Result<T, DecodeError>`
pub type DecodeResult<T> = Result<T, DecodeError>;

const STATIC_TABLE_LEN: usize = 61;

macro_rules! entry {
    ($name:expr, $value:expr) => {
        Entry {
            name: $name.to_tendril(),
            value: $value.to_tendril(),
        }
    }
}

lazy_static! {
    /// This table is taken from Appendix A, Static Table Definition.
    static ref STATIC_TABLE: [Entry; STATIC_TABLE_LEN] = [
        entry!(b":authority", b""),
        entry!(b":method", b"GET"),
        entry!(b":method", b"POST"),
        entry!(b":path", b"/"),
        entry!(b":path", b"/index.html"),
        entry!(b":scheme", b"http"),
        entry!(b":scheme", b"https"),
        entry!(b":status", b"200"),
        entry!(b":status", b"204"),
        entry!(b":status", b"206"),
        entry!(b":status", b"304"),
        entry!(b":status", b"400"),
        entry!(b":status", b"404"),
        entry!(b":status", b"500"),
        entry!(b"accept-charset", b""),
        entry!(b"accept-encoding", b"gzip, deflate"),
        entry!(b"accept-language", b""),
        entry!(b"accept-ranges", b""),
        entry!(b"accept", b""),
        entry!(b"access-control-allow-origin", b""),
        entry!(b"age", b""),
        entry!(b"allow", b""),
        entry!(b"authorization", b""),
        entry!(b"cache-control", b""),
        entry!(b"content-disposition", b""),
        entry!(b"content-encoding", b""),
        entry!(b"content-language", b""),
        entry!(b"content-length", b""),
        entry!(b"content-location", b""),
        entry!(b"content-range", b""),
        entry!(b"content-type", b""),
        entry!(b"cookie", b""),
        entry!(b"date", b""),
        entry!(b"etag", b""),
        entry!(b"expect", b""),
        entry!(b"expires", b""),
        entry!(b"from", b""),
        entry!(b"host", b""),
        entry!(b"if-match", b""),
        entry!(b"if-modified-since", b""),
        entry!(b"if-none-match", b""),
        entry!(b"if-range", b""),
        entry!(b"if-unmodified-since", b""),
        entry!(b"last-modified", b""),
        entry!(b"link", b""),
        entry!(b"location", b""),
        entry!(b"max-forwards", b""),
        entry!(b"proxy-authenticate", b""),
        entry!(b"proxy-authorization", b""),
        entry!(b"range", b""),
        entry!(b"referer", b""),
        entry!(b"refresh", b""),
        entry!(b"retry-after", b""),
        entry!(b"server", b""),
        entry!(b"set-cookie", b""),
        entry!(b"strict-transport-security", b""),
        entry!(b"transfer-encoding", b""),
        entry!(b"user-agent", b""),
        entry!(b"vary", b""),
        entry!(b"via", b""),
        entry!(b"www-authenticate", b""),
    ];
}

// We use core::nonzero if we can, but it’s not stable, so for stable support we patch it.
#[cfg(feature = "non_zero")]
use core::nonzero::NonZero;
#[cfg(feature = "non_zero")]
#[doc(hidden)]
pub type NonZeroU32 = NonZero<u32>;
#[cfg(not(feature = "non_zero"))]
#[doc(hidden)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub struct NonZeroU32(u32);

#[cfg(not(feature = "non_zero"))]
impl NonZeroU32 {
    unsafe fn new(value: u32) -> NonZeroU32 {
        NonZeroU32(value)
    }
}

#[cfg(not(feature = "non_zero"))]
impl ::std::ops::Deref for NonZeroU32 {
    type Target = u32;

    fn deref(&self) -> &u32 {
        &self.0
    }
}

/// An index into the indexing tables.
///
/// The tables do not use the value 0, hence the nonzeroness.
pub type Index = NonZeroU32;

/// Indexing tables.
pub struct Tables {
    static_: &'static [Entry; STATIC_TABLE_LEN],
    dynamic: VecDeque<Entry>,
    /// >    The size of the dynamic table is the sum of the size of its entries.
    ///
    /// We maintain this manually.
    size: u32,
    max_size: u32,
    protocol_max_size: u32,
}

impl Tables {
    /// Constructs a new set of indexing tables.
    pub fn new() -> Tables {
        Tables {
            static_: &*STATIC_TABLE,
            dynamic: VecDeque::new(),
            size: 0,
            // 4096 is the default SETTINGS_HEADER_TABLE_SIZE value in HTTP/2
            max_size: 4096,
            protocol_max_size: 4096,
        }
    }

    /// Get the entry contained at the given index.
    pub fn get(&self, index: Index) -> DecodeResult<&Entry> {
        let index = *index as usize;
        match index {
            1...STATIC_TABLE_LEN => Ok(&self.static_[index - 1]),
            _ => {
                match self.dynamic.get(index - STATIC_TABLE_LEN - 1) {
                    Some(entry) => Ok(&entry),
                    None => Err(DecodeError),
                }
            }
        }
    }

    /// Insert the entry into the dynamic table.
    /// Old entries may be evicted by doing this.
    ///
    /// Returns a `DecodeError` if the protocol max size has been lowered
    /// without the table’s max size having been accordingly lowered.
    pub fn insert(&mut self, entry: Entry) -> DecodeResult<()> {
        if self.max_size > self.protocol_max_size {
            // As noted in set_protocol_max_size, I’ve decided that inserting a new entry is an
            // error if the protocol max size has been changed without a table max size adjustment
            // to match.
            Err(DecodeError)
        } else {
            // See RFC 7541, section 4.4 (Entry Eviction When Adding New Entries).
            let size = entry.size();
            if size > self.max_size {
                self.size = 0;
                self.dynamic.clear();
            } else {
                self.size += size;
                self.evict_as_required();
                self.dynamic.push_front(entry);
            }
            Ok(())
        }
    }

    /// Adjust the maximum index table size that the protocol will permit.
    ///
    /// This is called by the protocol (HTTP/2), not by fragment decoding.
    pub fn set_protocol_max_size(&mut self, max_size: u32) {
        // The spec isn’t *entirely* clear on how this should be handled. :-(
        //
        // > An encoder can choose to use less capacity than this maximum size
        // > (see Section 6.3), but the chosen size MUST stay lower than or equal
        // > to the maximum set by the protocol.
        //
        // (The protocol’s maximum is SETTINGS_HEADER_TABLE_SIZE in HTTP/2.)
        //
        // OK, so imagine we get a protocol max size change which leaves max_size >
        // protocol_max_size. What do we do now? The invariant specified has been violated.
        // Should the SETTINGS frame be considered an error inasmuch as the dynamic table hasn’t
        // been reduced yet? Or should we consider it OK, so long as a max_size reduction is the
        // very next instruction to the table? The former seems unreasonable; the latter seems a
        // little more reasonable. I’ll go with the latter until further notice.
        self.protocol_max_size = max_size;
    }

    /// Set the maximum index table size permitted, before eviction occurs.
    pub fn set_max_size(&mut self, max_size: u32) -> DecodeResult<()> {
        if max_size > self.protocol_max_size {
            Err(DecodeError)
        } else {
            self.max_size = max_size;
            self.evict_as_required();
            Ok(())
        }
    }

    fn evict_as_required(&mut self) {
        while self.size > self.max_size {
            match self.dynamic.pop_back() {
                Some(entry) => self.size -= entry.size(),
                None => unreachable!(),
            }
        }
    }
}

/// A header entry yielded by decoding a header block.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Entry {
    /// The header field name.
    pub name: ByteTendril,
    /// The header field value.
    pub value: ByteTendril,
}

impl Entry {
    /// > The size of an entry is the sum of its name's length in octets (as
    /// > defined in Section 5.2), its value's length in octets, and 32.
    fn size(&self) -> u32 {
        self.name.len32() + self.value.len32() + 32
    }
}

/// The indexing behaviour of a `Instruction::LiteralHeader`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LiteralHeaderMode {
    /// 6.2.1. Literal Header Field with Incremental Indexing
    IncrementalIndexing,
    /// 6.2.2. Literal Header Field without Indexing
    WithoutIndexing,
    /// 6.2.3. Literal Header Field Never Indexed
    NeverIndexed,
}

/// The name of an `Instruction::LiteralHeader`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LiteralHeaderName {
    /// The name is found in the tables with this index.
    Index(Index),
    /// The name is a literal value.
    Literal(ByteTendril),
}

/// A typed representation of an instruction from a header block.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Instruction {
    /// RFC 7541, section 6.1. Indexed Header Field Representation
    IndexedHeader {
        /// The index of the header field.
        index: Index,
    },

    /// RFC 7541, section 6.2. Literal Header Field Representation
    LiteralHeader {
        /// The indexing behaviour: indexed, non-indexed or never indexed.
        mode: LiteralHeaderMode,
        /// The name (a string literal or an index) of the header to yield.
        name: LiteralHeaderName,
        /// The value of the header to yield.
        value: ByteTendril,
    },

    /// RFC 7541, section 6.3. Dynamic Table Size Update
    DynamicTableSizeUpdate {
        /// The new maximum size for the dynamic table. This MUST NOT exceed the protocol’s limit,
        /// which in HTTP/2 is the SETTINGS_HEADER_TABLE_SIZE parameter.
        max_size: u32,
    },
}

impl Instruction {
    /// Encode the instruction to the writer.
    ///
    /// Note that if `self` is not a legal instruction (e.g. using a non-existent table index) this
    /// will produce a similarly illegal encoded instruction.
    fn encode<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        match *self {
            Instruction::IndexedHeader { index } => {
                integer::encode7(writer, 0b10000000, *index)
            },

            Instruction::LiteralHeader { mode, ref name, ref value } => {
                let index = match *name {
                    LiteralHeaderName::Index(index) => *index,
                    LiteralHeaderName::Literal(_) => 0,
                };
                match mode {
                    LiteralHeaderMode::IncrementalIndexing =>
                        try!(integer::encode6(writer, 0b01000000, index)),
                    LiteralHeaderMode::WithoutIndexing =>
                        try!(integer::encode4(writer, 0b00000000, index)),
                    LiteralHeaderMode::NeverIndexed =>
                        try!(integer::encode4(writer, 0b00010000, index)),
                }
                if let LiteralHeaderName::Literal(ref name) = *name {
                    try!(string::encode_plain(writer, name));
                }
                string::encode_plain(writer, value)
            },

            Instruction::DynamicTableSizeUpdate { max_size } => {
                integer::encode5(writer, 0b00100000, max_size)
            },
        }
    }
}

/// A header block decoder which just decodes instructions.
///
/// Input is provided as a mutable reference to a `ByteTendril` and steadily consumed as you
/// iterate over it.
///
/// This is purely the instruction decoder; it doesn’t apply the decoded instructions to anything
/// and is intended to be used with `InstructionExecutor` which can apply the instructions to a set
/// of indexing tables, yielding the headers produced.
// Clone doesn’t make sense because of the strict application order of the instructions.
// Anything using indexing (which any serious header block fragments will) would be ruined.
#[derive(Debug, PartialEq, Eq)]
pub struct InstructionDecoder {
    input: ByteTendril,
}

impl InstructionDecoder {
    /// Constructs a new `InstructionDecoder` from the given input and static/dynamic tables.
    pub fn new(input: ByteTendril) -> InstructionDecoder {
        InstructionDecoder {
            input: input,
        }
    }
}

macro_rules! try2 {
    ($expr:expr) => (match $expr {
        Ok(x) => x,
        Err(e) => return Some(Err(e)),
    })
}

impl Iterator for InstructionDecoder {
    type Item = DecodeResult<Instruction>;

    fn next(&mut self) -> Option<DecodeResult<Instruction>> {
        let b = match self.input.get(0) {
            Some(&b) => b,
            None => return None,
        };
        Some(match b {
            0b10000000...0b11111111 => {
                // Leading 1: 6.1, indexed header field representation.
                match try2!(integer::decode7(&mut self.input)) {
                    // > The index value of 0 is not used.  It MUST be treated as a decoding
                    // > error if found in an indexed header field representation.
                    0 => Err(DecodeError),
                    index => Ok(Instruction::IndexedHeader {
                        index: unsafe { Index::new(index) },
                    }),
                }
            },
            0b01000000...0b01111111 => {
                // Leading 01: 6.2.1, literal header field with incremental indexing.
                Ok(Instruction::LiteralHeader {
                    mode: LiteralHeaderMode::IncrementalIndexing,
                    name: match try2!(integer::decode6(&mut self.input)) {
                        0 => LiteralHeaderName::Literal(try2!(string::decode(&mut self.input))),
                        index => LiteralHeaderName::Index(unsafe { Index::new(index) }),
                    },
                    value: try2!(string::decode(&mut self.input)),
                })
            },
            0b00000000...0b00011111 => {
                // Leading 0000: 6.2.2, literal header field without indexing.
                // Leading 0001: 6.2.3, literal header field never indexed.
                Ok(Instruction::LiteralHeader {
                    mode: if b < 0b00010000 {
                        LiteralHeaderMode::WithoutIndexing
                    } else {
                        LiteralHeaderMode::NeverIndexed
                    },
                    name: match try2!(integer::decode4(&mut self.input)) {
                        0 => LiteralHeaderName::Literal(try2!(string::decode(&mut self.input))),
                        index => LiteralHeaderName::Index(unsafe { Index::new(index) }),
                    },
                    value: try2!(string::decode(&mut self.input)),
                })
            },
            _ => {
                // Leading 001: 6.3, dynamic table size update.
                Ok(Instruction::DynamicTableSizeUpdate {
                    max_size: try2!(integer::decode5(&mut self.input)),
                })
            },
        })
    }
}

/// A part of the header block decoder which executes decoded instructions.
///
/// This is intended to be used in conjunction with `InstructionDecoder`, which performs the
/// decoding of the input bytes into instructions in the type system.
///
/// This applier works directly on the tables given to it, applying the decoded instructions; the
/// values it yields are the header entries (name/value pairs). If you wish to just decode the
/// instructions without applying them, use `InstructionDecoder` directly.
pub struct InstructionExecutor<'tables, I>
where I: Iterator, I::Item: InstructionOrDecodeResultInstruction {
    instructions: I,
    tables: &'tables mut Tables,
}

#[doc(hidden)]
pub trait InstructionOrDecodeResultInstruction {
    fn into_result_instruction(self) -> DecodeResult<Instruction>;
}

impl InstructionOrDecodeResultInstruction for Instruction {
    #[inline]
    fn into_result_instruction(self) -> DecodeResult<Instruction> {
        Ok(self)
    }
}

impl InstructionOrDecodeResultInstruction for DecodeResult<Instruction> {
    #[inline]
    fn into_result_instruction(self) -> DecodeResult<Instruction> {
        self
    }
}

impl<'tables, I> InstructionExecutor<'tables, I>
where I: Iterator, I::Item: InstructionOrDecodeResultInstruction {
    /// Constructs a new `InstructionsExecutor` from the given instructions iterator and tables.
    // This isn’t called `new` so that Decoder can use that name. (i.e. trivial ergonomics.)
    pub fn from_instructions(instructions: I, tables: &'tables mut Tables)
            -> InstructionExecutor<'tables, I> {
        InstructionExecutor {
            instructions: instructions,
            tables: tables,
        }
    }
}

impl<'tables, I> Iterator for InstructionExecutor<'tables, I>
where I: Iterator, I::Item: InstructionOrDecodeResultInstruction {
    type Item = DecodeResult<Entry>;

    fn next(&mut self) -> Option<DecodeResult<Entry>> {
        loop {
            match self.instructions.next().map(|i| i.into_result_instruction()) {
                Some(Ok(Instruction::IndexedHeader { index })) => {
                    // See section 3.2 and 2.3 on static+dynamic tables
                    return Some(self.tables.get(index).map(|entry| entry.clone()));
                },
                Some(Ok(Instruction::LiteralHeader { mode, name, value })) => {
                    let name = match name {
                        LiteralHeaderName::Index(i) => try2!(self.tables.get(i)).name.clone(),
                        LiteralHeaderName::Literal(name) => name,
                    };
                    let entry = Entry {
                        name: name,
                        value: value,
                    };
                    if mode == LiteralHeaderMode::IncrementalIndexing {
                        try2!(self.tables.insert(entry.clone()));
                    }
                    return Some(Ok(entry));
                },
                Some(Ok(Instruction::DynamicTableSizeUpdate { max_size })) => {
                    // New maximum size MUST be <= HTTP/2’s SETTINGS_HEADER_TABLE_SIZE.
                    // Reducing the maximum size can cause entries to be evicted.
                    try2!(self.tables.set_max_size(max_size));
                    continue;
                },
                Some(Err(e)) => return Some(Err(e)),
                None => return None,
            }
        }
    }
}

/// A header block decoder which decodes and executes the instructions against index tables.
pub type Decoder<'tables> = InstructionExecutor<'tables, InstructionDecoder>;

impl<'tables> Decoder<'tables> {
    /// Constructs a new `InstructionExecutor` from the given input and static/dynamic tables.
    pub fn new(input: ByteTendril, tables: &'tables mut Tables)
            -> Decoder<'tables> {
        InstructionExecutor {
            instructions: InstructionDecoder::new(input),
            tables: tables,
        }
    }
}

/// A header block fragment.
///
/// This is for the frame payload types to use so that they can have lazy header decoding without
/// needing intermediate storage, and yet store values for encoding.
#[derive(Debug, Eq, PartialEq)]
pub enum Fragment {
    /// The fragment is encoded in a tendril and must be decoded to get its contents.
    /// This is the normal case when receiving frames. The value may not be completely legal.
    Decoder(InstructionDecoder),

    /// The fragment is of actual known instructions, not encoded in a tendril.
    /// This is the normal case when constructing frames.
    Instructions(Vec<Instruction>),

    // XXX: I’m pretty confident that I’ll want a non-allocating (iterator-based) version of
    // `Instructions`, which will probably mean a generic of some form.
}

impl Fragment {
    /// Encode the header block fragment to a writer.
    ///
    /// If `self` is an `InstructionDecoder` which yields a decode error,
    /// writing will cease and an I/O error of kind `InvalidData` will be returned.
    pub fn encode<W: io::Write>(self, writer: &mut W) -> io::Result<()> {
        match self {
            Fragment::Decoder(decoder) => for instruction in decoder {
                match instruction {
                    Ok(i) => try!(i.encode(writer)),
                    Err(_) => return Err(io::Error::new(io::ErrorKind::InvalidData,
                                                        "FragmentEncoder hit failed decoding")),
                }
            },
            Fragment::Instructions(vec) => for instruction in vec {
                try!(instruction.encode(writer));
            },
        }
        Ok(())
    }
}

impl IntoIterator for Fragment {
    type Item = DecodeResult<Instruction>;
    type IntoIter = FragmentDecoder;

    fn into_iter(self) -> FragmentDecoder {
        match self {
            Fragment::Decoder(x) => FragmentDecoder::Decoder(x),
            Fragment::Instructions(x) => FragmentDecoder::Instructions(x.into_iter()),
        }
    }
}

/// The decoding implementation for `Fragment`.
///
/// Use this as an `Iterator<Item = DecodeResult<Instruction>>`.
pub enum FragmentDecoder {
    /// Comes from a `Fragment::Decoder`.
    Decoder(InstructionDecoder),
    /// Comes from a `Fragment::Instructions`.
    Instructions(vec::IntoIter<Instruction>),
}

impl Iterator for FragmentDecoder {
    type Item = DecodeResult<Instruction>;

    #[inline]
    fn next(&mut self) -> Option<DecodeResult<Instruction>> {
        match *self {
            FragmentDecoder::Decoder(ref mut x) => x.next(),
            FragmentDecoder::Instructions(ref mut x) => x.next().map(|x| Ok(x)),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match *self {
            FragmentDecoder::Decoder(ref x) => x.size_hint(),
            FragmentDecoder::Instructions(ref x) => x.size_hint(),
        }
    }
}

// Now come the tests. The c_* tests come from RFC 7541, Appendix C.
// These tests are at present moderately weak, as there is very little negative testing.
// TODO: make negative tests where appropriate. (Frankly, not many are needed.)

macro_rules! t {
    ($name:ident$(, [SETTINGS_HEADER_TABLE_SIZE = $protocol_max:expr])*, $({
        input = $input:expr;
        instructions = $instructions:expr;
        dynamic table = $size:expr, $dynamic_table:expr;
        headers = $headers:expr;
    }),+) => {
        #[test]
        fn $name() {
            let mut tables = Tables::new();
            $(
                tables.set_protocol_max_size($protocol_max);
                assert_eq!(tables.set_max_size($protocol_max), Ok(()));
            )*
            $(
                let input = ByteTendril::from($input as &[u8]);
                let mut headers = vec![];
                let mut instructions = vec![];
                let mut failed = false;
                {
                    let decoder = InstructionDecoder::new(input)
                        .inspect(|instruction| match *instruction {
                            Ok(ref i) => instructions.push(i.clone()),
                            Err(_) => (),
                        });
                    for entry in InstructionExecutor::from_instructions(decoder, &mut tables) {
                        match entry {
                            Ok(entry) => headers.push(entry),
                            Err(_) => {
                                failed = true;
                                break;
                            }
                        }
                    }
                }
                if failed {
                    panic!("Decoding failed.\n\
                            Instructions read: {:?}\n\
                            Headers: {:?}", instructions, headers);
                }
                assert_eq!(&*instructions, &$instructions);
                assert_eq!(tables.size, $size);
                assert_eq!(&*tables.dynamic.iter().collect::<Vec<_>>(),
                           &$dynamic_table as &[&Entry]);
                assert_eq!(&*headers, &$headers);
            )+
        }
    }
}

macro_rules! t2 {
    ($name:ident, $name_huffman:ident$(, [SETTINGS_HEADER_TABLE_SIZE = $protocol_max:expr])*, $({
        input = $input:expr;
        input/huffman = $input_huffman:expr;
        instructions = $instructions:expr;
        dynamic table = $size:expr, $dynamic_table:expr;
        headers = $headers:expr;
    }),+) => {
        t!($name$(, [SETTINGS_HEADER_TABLE_SIZE = $protocol_max])*, $({
            input = $input;
            instructions = $instructions;
            dynamic table = $size, $dynamic_table;
            headers = $headers;
        }),+);

        t!($name_huffman$(, [SETTINGS_HEADER_TABLE_SIZE = $protocol_max])*, $({
            input = $input_huffman;
            instructions = $instructions;
            dynamic table = $size, $dynamic_table;
            headers = $headers;
        }),+);
    }
}

#[cfg(test)]
use self::LiteralHeaderMode::*;
#[cfg(test)]
use self::Instruction::*;

t!(c_2_1_literal_header_field_with_indexing, {
    input = b"\x40\x0acustom-key\x0dcustom-header";
    instructions = [
        LiteralHeader {
            mode: IncrementalIndexing,
            name: LiteralHeaderName::Literal(b"custom-key".to_tendril()),
            value: b"custom-header".to_tendril(),
        },
    ];
    dynamic table = 55, [
        &entry!(b"custom-key", b"custom-header"),
    ];
    headers = [
        entry!(b"custom-key", b"custom-header"),
    ];
});

t!(c_2_2_literal_header_field_without_indexing, {
    input = b"\x04\x0c/sample/path";
    instructions = [
        LiteralHeader {
            mode: WithoutIndexing,
            name: LiteralHeaderName::Index(unsafe { Index::new(4) }),
            value: b"/sample/path".to_tendril(),
        },
    ];
    dynamic table = 0, [];
    headers = [
        entry!(b":path", b"/sample/path"),
    ];
});

t!(c_2_3_literal_header_field_never_indexed, {
    input = b"\x10\x08password\x06secret";
    instructions = [
        LiteralHeader {
            mode: NeverIndexed,
            name: LiteralHeaderName::Literal(b"password".to_tendril()),
            value: b"secret".to_tendril(),
        },
    ];
    dynamic table = 0, [];
    headers = [
        entry!(b"password", b"secret"),
    ];
});

t!(c_2_4_indexed_header_field, {
    input = b"\x82";
    instructions = [
        IndexedHeader {
            index: unsafe { Index::new(2) },
        }
    ];
    dynamic table = 0, [];
    headers = [entry!(b":method", b"GET")];
});

t2!(c_3_request_examples_without_huffman_coding,
    c_4_request_examples_with_huffman_coding,
{
    input = b"\x82\x86\x84\x41\x0fwww.example.com";
    input/huffman = b"\x82\x86\x84\x41\x8c\xf1\xe3\xc2\xe5\xf2\x3a\x6b\xa0\xab\x90\xf4\xff";
    instructions = [
        IndexedHeader {
            index: unsafe { Index::new(2) },
        },
        IndexedHeader {
            index: unsafe { Index::new(6) },
        },
        IndexedHeader {
            index: unsafe { Index::new(4) },
        },
        LiteralHeader {
            mode: IncrementalIndexing,
            name: LiteralHeaderName::Index(unsafe { Index::new(1) }),
            value: b"www.example.com".to_tendril(),
        },
    ];
    dynamic table = 57, [
        &entry!(b":authority", b"www.example.com"),
    ];
    headers = [
        entry!(b":method", b"GET"),
        entry!(b":scheme", b"http"),
        entry!(b":path", b"/"),
        entry!(b":authority", b"www.example.com"),
    ];
}, {
    input = b"\x82\x86\x84\xbe\x58\x08no-cache";
    input/huffman = b"\x82\x86\x84\xbe\x58\x86\xa8\xeb\x10\x64\x9c\xbf";
    instructions = [
        IndexedHeader {
            index: unsafe { Index::new(2) },
        },
        IndexedHeader {
            index: unsafe { Index::new(6) },
        },
        IndexedHeader {
            index: unsafe { Index::new(4) },
        },
        IndexedHeader {
            index: unsafe { Index::new(62) },
        },
        LiteralHeader {
            mode: IncrementalIndexing,
            name: LiteralHeaderName::Index(unsafe { Index::new(24) }),
            value: b"no-cache".to_tendril(),
        },
    ];
    dynamic table = 110, [
        &entry!(b"cache-control", b"no-cache"),
        &entry!(b":authority", b"www.example.com"),
    ];
    headers = [
        entry!(b":method", b"GET"),
        entry!(b":scheme", b"http"),
        entry!(b":path", b"/"),
        entry!(b":authority", b"www.example.com"),
        entry!(b"cache-control", b"no-cache"),
    ];
}, {
    input = b"\x82\x87\x85\xbf\x40\x0acustom-key\x0ccustom-value";
    input/huffman = b"\x82\x87\x85\xbf\x40\x88\x25\xa8\x49\xe9\x5b\xa9\x7d\x7f\x89\x25\xa8\x49\xe9\x5b\xb8\xe8\xb4\xbf";
    instructions = [
        IndexedHeader {
            index: unsafe { Index::new(2) },
        },
        IndexedHeader {
            index: unsafe { Index::new(7) },
        },
        IndexedHeader {
            index: unsafe { Index::new(5) },
        },
        IndexedHeader {
            index: unsafe { Index::new(63) },
        },
        LiteralHeader {
            mode: IncrementalIndexing,
            name: LiteralHeaderName::Literal(b"custom-key".to_tendril()),
            value: b"custom-value".to_tendril(),
        },
    ];
    dynamic table = 164, [
        &entry!(b"custom-key", b"custom-value"),
        &entry!(b"cache-control", b"no-cache"),
        &entry!(b":authority", b"www.example.com"),
    ];
    headers = [
        entry!(b":method", b"GET"),
        entry!(b":scheme", b"https"),
        entry!(b":path", b"/index.html"),
        entry!(b":authority", b"www.example.com"),
        entry!(b"custom-key", b"custom-value"),
    ];
});

t2!(c_5_response_examples_without_huffman_coding,
    c_6_response_examples_with_huffman_coding,
    [SETTINGS_HEADER_TABLE_SIZE = 256],
{
    input = b"\x48\x03302\x58\x07private\x61\x1dMon, 21 Oct 2013 20:13:21 GMT\x6e\x17https://www.example.com";
    input/huffman = b"\x48\x82\x64\x02\x58\x85\xae\xc3\x77\x1a\x4b\x61\x96\xd0\x7a\xbe\x94\x10\x54\xd4\x44\xa8\x20\x05\x95\x04\x0b\x81\x66\xe0\x82\xa6\x2d\x1b\xff\x6e\x91\x9d\x29\xad\x17\x18\x63\xc7\x8f\x0b\x97\xc8\xe9\xae\x82\xae\x43\xd3";
    instructions = [
        LiteralHeader {
            mode: IncrementalIndexing,
            name: LiteralHeaderName::Index(unsafe { Index::new(8) }),
            value: b"302".to_tendril(),
        },
        LiteralHeader {
            mode: IncrementalIndexing,
            name: LiteralHeaderName::Index(unsafe { Index::new(24) }),
            value: b"private".to_tendril(),
        },
        LiteralHeader {
            mode: IncrementalIndexing,
            name: LiteralHeaderName::Index(unsafe { Index::new(33) }),
            value: b"Mon, 21 Oct 2013 20:13:21 GMT".to_tendril(),
        },
        LiteralHeader {
            mode: IncrementalIndexing,
            name: LiteralHeaderName::Index(unsafe { Index::new(46) }),
            value: b"https://www.example.com".to_tendril(),
        },
    ];
    dynamic table = 222, [
        &entry!(b"location", b"https://www.example.com"),
        &entry!(b"date", b"Mon, 21 Oct 2013 20:13:21 GMT"),
        &entry!(b"cache-control", b"private"),
        &entry!(b":status", b"302"),
    ];
    headers = [
        entry!(b":status", b"302"),
        entry!(b"cache-control", b"private"),
        entry!(b"date", b"Mon, 21 Oct 2013 20:13:21 GMT"),
        entry!(b"location", b"https://www.example.com"),
    ];
}, {
    input = b"\x48\x03307\xc1\xc0\xbf";
    input/huffman = b"\x48\x83\x64\x0e\xff\xc1\xc0\xbf";
    instructions = [
        LiteralHeader {
            mode: IncrementalIndexing,
            name: LiteralHeaderName::Index(unsafe { Index::new(8) }),
            value: b"307".to_tendril(),
        },
        IndexedHeader {
            index: unsafe { Index::new(65) },
        },
        IndexedHeader {
            index: unsafe { Index::new(64) },
        },
        IndexedHeader {
            index: unsafe { Index::new(63) },
        },
    ];
    dynamic table = 222, [
        &entry!(b":status", b"307"),
        &entry!(b"location", b"https://www.example.com"),
        &entry!(b"date", b"Mon, 21 Oct 2013 20:13:21 GMT"),
        &entry!(b"cache-control", b"private"),
    ];
    headers = [
        entry!(b":status", b"307"),
        entry!(b"cache-control", b"private"),
        entry!(b"date", b"Mon, 21 Oct 2013 20:13:21 GMT"),
        entry!(b"location", b"https://www.example.com"),
    ];
}, {
    input = b"\x88\xc1\x61\x1dMon, 21 Oct 2013 20:13:22 GMT\xc0\x5a\x04gzip\x77\x38foo=ASDJKHQKBZXOQWEOPIUAXQWEOIU; max-age=3600; version=1";
    input/huffman = b"\x88\xc1\x61\x96\xd0\x7a\xbe\x94\x10\x54\xd4\x44\xa8\x20\x05\x95\x04\x0b\x81\x66\xe0\x84\xa6\x2d\x1b\xff\xc0\x5a\x83\x9b\xd9\xab\x77\xad\x94\xe7\x82\x1d\xd7\xf2\xe6\xc7\xb3\x35\xdf\xdf\xcd\x5b\x39\x60\xd5\xaf\x27\x08\x7f\x36\x72\xc1\xab\x27\x0f\xb5\x29\x1f\x95\x87\x31\x60\x65\xc0\x03\xed\x4e\xe5\xb1\x06\x3d\x50\x07";
    instructions = [
        IndexedHeader {
            index: unsafe { Index::new(8) },
        },
        IndexedHeader {
            index: unsafe { Index::new(65) },
        },
        LiteralHeader {
            mode: IncrementalIndexing,
            name: LiteralHeaderName::Index(unsafe { Index::new(33) }),
            value: b"Mon, 21 Oct 2013 20:13:22 GMT".to_tendril(),
        },
        IndexedHeader {
            index: unsafe { Index::new(64) },
        },
        LiteralHeader {
            mode: IncrementalIndexing,
            name: LiteralHeaderName::Index(unsafe { Index::new(26) }),
            value: b"gzip".to_tendril(),
        },
        LiteralHeader {
            mode: IncrementalIndexing,
            name: LiteralHeaderName::Index(unsafe { Index::new(55) }),
            value: b"foo=ASDJKHQKBZXOQWEOPIUAXQWEOIU; max-age=3600; version=1".to_tendril(),
        },
    ];
    dynamic table = 215, [
        &entry!(b"set-cookie", b"foo=ASDJKHQKBZXOQWEOPIUAXQWEOIU; max-age=3600; version=1"),
        &entry!(b"content-encoding", b"gzip"),
        &entry!(b"date", b"Mon, 21 Oct 2013 20:13:22 GMT"),
    ];
    headers = [
        entry!(b":status", b"200"),
        entry!(b"cache-control", b"private"),
        entry!(b"date", b"Mon, 21 Oct 2013 20:13:22 GMT"),
        entry!(b"location", b"https://www.example.com"),
        entry!(b"content-encoding", b"gzip"),
        entry!(b"set-cookie", b"foo=ASDJKHQKBZXOQWEOPIUAXQWEOIU; max-age=3600; version=1"),
    ];
});
