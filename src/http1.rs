//! HTTP/1.1 parsing and low-level representation.
//!
//! This experiment is in phantom types. ASSESSMENT: discontinued. Too
//! unergonomic. Try httpt instead, perhaps?

use status::StatusCode;
use grammar;
use grammar::token::Token;
use method::Method;
use std::io::{Read};
use std::marker::PhantomData;

use self::MessageType::{Request, Response};
use self::Step::{PreStartLine, PostStartLine, PreHeaderField, PostHeaderField,
                 PreMessageBody};
use self::Error::*;
use self::RawRequestTarget::*;

/// How ’bout a 64KB buffer?
///
/// Note that if this is increased any further, ws_one_len and ws_two_len will
/// need to be increased in size also, for they will no longer fit in u16.
const MAX_BUFFER_SIZE: usize = 0xffff;

enum Error {
    /// Any I/O error which means we should drop the connection.
    IoError,
    /// An HTTP-message parse error.
    /// A server should respond 400 Bad Request; clients should probably
    /// complain of having received a bad response in some other way.
    ParseError,
}

pub type ParseResult<T> = Result<T, Error>;

macro_rules! try2 {
    ($e:expr) => {
        match $e {
            Ok(ok) => ok,
            Err(_) => return Err(IoError),
        }
    }
}

macro_rules! parse_byte {
    ($reader:expr, $expected:pat) => {{
        let mut b = [0];
        match $reader.read(&mut b) {
            Ok(1) => match b[0] {
                b @ $expected => b,
                _ => return Err(Error::ParseError),
            },
            Ok(0) => return Err(Error::ParseError),
            _ => return Err(Error::IoError),
        }
    }}
}

/// The parts of an HTTP message, from RFC 7230.
///
/// This is the lowest level representation of the HTTP message.
pub struct Http1MessageBodyReader<R> {
    reader: R,
    // This will be bitflags
    //transfer_encodings: Vec<T>,
}

phantom_enum! {
    #[doc = "Phantom types representing whether the HTTP-message is a request \
             or a response."]
    pub enum MessageType {
        #[doc = "Phantom type indicating that the HTTP-message is a request."]
        Request,

        #[doc = "Phantom type indicating that the HTTP-message is a response."]
        Response
    }
}

phantom_enum! {
    #[doc = "Phantom types representing how far through the HTTP-message \
             parsing has gone"]
    pub enum Step {
        #[doc = "The next thing to be parsed is the start-line."]
        PreStartLine,

        #[doc = "The start-line has been read: now you can retrieve it."]
        PostStartLine,

        #[doc = "The next thing to be parsed is a header-field."]
        PreHeaderField,

        #[doc = "A header-field has been read: now you can retrieve it."]
        PostHeaderField,

        #[doc = "The next thing to be parsed is the message-body."]
        PreMessageBody
    }
}

/// HTTP/1 parser.
///
/// This operates as a state machine, encoded into the type system with phantom
/// types, to prevent you from accidentally doing the wrong thing. For example,
/// you can only read a request line at the start, and only once.
///
/// TODO: insert waaaay more details!
pub struct Parser<R, MessageType, Step> {
    /// The reader being read from.
    reader: R,

    /// The space in which values read are placed. This working space is
    /// allowed up to two things in it, which cumulatively
    ///
    /// Suppose, for example, we are reading a header-field "Name: Value".
    /// This will be stored in this working space, imagining it to be 32 bytes
    /// wide, thus: `NameValue.......................` (`.` representing
    /// arbitrary, irrelevant data). No need for a separator in there because
    /// of the `ws_one_len` and `ws_two_len` fields. In the example given,
    /// `ws_one_len` would be 4 and `ws_two_len` would be 5, the sum of them
    /// being less than 32, as required.
    ///
    /// This is used for the following:
    ///
    /// - In request-line: method and request-target
    /// - In status-line: reason-phrase only
    /// - In header-field: field-name and field-value
    working_space: [u8; MAX_BUFFER_SIZE],

    /// The length of the first item in the working space.
    ws_one_len: u16,

    /// The length of the second item in the working space,
    /// which begins at index `ws_one_len`.
    ws_two_len: u16,

    /// HTTP-Version. This is not meaningful until the start-line has been
    /// read. Each value is a single digit.
    http_version: (u8, u8),

    marker1: PhantomData<MessageType>,
    marker2: PhantomData<Step>,
}

impl<R: Read, MT: MessageType::Impl, S: Step::Impl> Parser<R, MT, S> {
    /// Change the step phantom type parameter.
    #[inline]
    fn step<NewStep: Step::Impl>(self) -> Parser<R, MT, NewStep> {
        Parser {
            reader: self.reader,
            working_space: self.working_space,
            ws_one_len: self.ws_one_len,
            ws_two_len: self.ws_two_len,
            http_version: self.http_version,
            marker1: PhantomData,
            marker2: PhantomData,
        }
    }
}

impl<R: Read, MT: MessageType::Impl> Parser<R, MT, PreStartLine> {
    pub fn from_reader(reader: R) -> Parser<R, MT, PreStartLine> {
        Parser {
            reader: reader,
            working_space: [0u8; MAX_BUFFER_SIZE],
            ws_one_len: 0,
            ws_two_len: 0,
            http_version: (0, 0),
            marker1: PhantomData,
            marker2: PhantomData,
        }
    }
}

impl<R: Read> Parser<R, Request, PreStartLine> {
    /// Read the request-line from the request.
    ///
    /// ```ignore
    /// request-line = method SP request-target SP HTTP-version CRLF
    /// ```
    ///
    /// For reasons of guaranteed correctness (at the cost of ergonomics), the
    /// method, request-target and HTTP-version must all be read from the next
    /// step of the parser, with `get_method`, `get_request_target` and
    /// `get_http_version`.
    pub fn read_request_line(mut self) -> ParseResult<Parser<R, Request,
                                                             PostStartLine>> {
        // TODO: read method into working_space/ws_one_len.
        // TODO: read request-target into working_space/ws_two_len.
        // TODO: read HTTP-version into self.http_version
        Ok(self.step())
    }
}

impl<R: Read, MT: MessageType::Impl> Parser<R, MT, PostStartLine> {
    /// Get the HTTP-version from the start-line of the request or response.
    #[inline]
    pub fn get_http_version(&self) -> (u8, u8) {
        self.http_version
    }
}

impl<R: Read, MT: MessageType::Impl, S: Step::Impl> Parser<R, MT, S> {
    #[inline]
    /// Get the contents of the first item in the working space.
    fn get_working_space_one<'a>(&'a self) -> &'a [u8] {
        &self.working_space[0..self.ws_one_len as usize]
    }

    #[inline]
    /// Get the contents of the second item in the working space.
    fn get_working_space_two<'a>(&'a self) -> &'a [u8] {
        &self.working_space[self.ws_one_len as usize..self.ws_one_len as usize +
                                                      self.ws_two_len as usize]
    }

    /// Read an `HTTP-version`.
    fn read_http_version(&mut self) -> ParseResult<(u8, u8)> {
        let _ = parse_byte!(self.reader, b'H');
        let _ = parse_byte!(self.reader, b'T');
        let _ = parse_byte!(self.reader, b'T');
        let _ = parse_byte!(self.reader, b'P');
        let _ = parse_byte!(self.reader, b'/');
        let major = parse_byte!(self.reader, b'0'...b'9') - b'0';
        let _ = parse_byte!(self.reader, b'.');
        let minor = parse_byte!(self.reader, b'0'...b'9') - b'0';
        Ok((major, minor))
    }

    /// Read into working space
    fn read_into_working_space<F: Fn(u8) -> bool>
                              (&mut self, start_point: usize, rule: F)
                              -> ParseResult<()> {
        for i in start_point..MAX_BUFFER_SIZE {
            let mut b = [0];
            match self.reader.read(&mut b) {
                Ok(1) if rule(b[0]) => {
                    // TODO: manipulate memory directly for perf’s sake.
                    // (i.e. drop bounds checking)
                    self.working_space[i] = b[0];
                },
                Ok(_) => return Ok(()),
                Err(_) => return Err(Error::IoError),
            }
        }
        // Value too long.
        Err(Error::ParseError)
    }
}

/// A request-target in raw form using byte slices.
///
/// ```ignore
/// request-target = origin-form
///                / absolute-form
///                / authority-form
///                / asterisk-form
/// origin-form    = absolute-path [ "?" query ]
/// absolute-form  = absolute-URI
/// authority-form = authority
/// asterisk-form  = "*"
/// ```
///
/// `authority-form` only occurs for CONNECT requests, and `absolute-form` only
/// occurs for non-CONNECT requests.
pub enum RawRequestTarget<'a> {
    /// The most common form of request-target, and the form requests should be
    /// written in.
    OriginForm(&'a [u8]),
    /// A complete URL; most commonly used with proxies, but all servers MUST
    /// support it for future compatibility.
    AbsoluteForm(&'a [u8]),
    /// Only used for `CONNECT` requests through proxies.
    AuthorityForm(&'a [u8]),
    /// Only used for a server-wide `OPTIONS` request.
    AsteriskForm,
}

impl<R: Read> Parser<R, Request, PostStartLine> {
    /// Get the method from the request-line, as a simple token.
    ///
    /// See also `get_method` which produces a properly typed method.
    #[inline]
    pub fn get_method_raw<'a>(&'a self) -> Token<'a> {
        // We know that the method is valid because we verified it earlier.
        unsafe { Token::from_slice_nocheck(self.get_working_space_one()) }
    }

    /// Get the method from the request-line, strongly typed.
    ///
    /// This is typically desirable over `get_method_raw`, but will be
    /// undesirable for certain operations on account of possibly incurring a
    /// heap allocation.
    #[inline]
    pub fn get_method(&self) -> Method<'static> {
        Method::from_token(self.get_method_raw()).into_owned()
    }

    /// Get the `request-target` from the request-line.
    ///
    /// This may return the following:
    ///
    /// - `OriginForm`
    /// - `AbsoluteForm` iff method != `CONNECT`
    /// - `AuthorityForm` iff method == `CONNECT`
    /// - `AsteriskForm` iff method == `OPTIONS`
    ///
    /// This cannot fail, because the reading already done ensured the
    /// appropriate invariants in the syntax.
    #[inline]
    pub fn get_request_target<'a>(&'a self) -> RawRequestTarget<'a> {
        let raw = self.get_working_space_two();
        if raw == b"*" {
            AsteriskForm
        } else if raw[0] == b'/' {
            OriginForm(raw)
        } else if self.get_method_raw().as_bytes() == b"CONNECT" {
            // Note that it is possible for an authority-form to be valid as an
            // absolute-form as well (e.g. “www.example.com:8080” is). For this
            // reason we must take into account that authority-form is only
            // valid for CONNECT and absolute-form for anything *but* CONNECT.
            AuthorityForm(raw)
        } else {
            AbsoluteForm(raw)
        }
    }
}

impl<R: Read> Parser<R, Request, PostStartLine> {
    /// Get the reason-phrase from the status-line, as raw bytes.
    #[inline]
    pub fn get_reason_phrase_raw<'a>(&'a self) -> &'a [u8] {
        self.get_working_space_one()
    }
}

impl<R: Read> Parser<R, Response, PreStartLine> {
    /// Read the status-line from the response.
    ///
    /// ```ignore
    /// status-line = HTTP-version SP status-code SP reason-phrase CRLF
    /// ```
    ///
    /// For reasons of guaranteed correctness with runtime efficiency (at the
    /// cost of ergonomics), the status code is returned immediately, but the
    /// reason-phrase and HTTP-version must be read from the next step of the
    /// parser, with `get_reason_phrase` and `get_http_version`.
    pub fn read_status_line(mut self) -> ParseResult<(Parser<R, Response,
                                                             PostStartLine>,
                                                      StatusCode)> {

        // FIXME: make this efficient.
        self.http_version = try!(self.read_http_version());

        let _ = parse_byte!(self.reader, b' ');

        let status = StatusCode::from_u16(
            (parse_byte!(self.reader, b'1'...b'5') - b'0') as u16 * 100
            + (parse_byte!(self.reader, b'0'...b'9') - b'0') as u16 * 10
            + (parse_byte!(self.reader, b'0'...b'9') - b'0') as u16).unwrap();

        let _ = parse_byte!(self.reader, b' ');

        // TODO: read reason-phrase into working_space/ws_one_len.

        // reason-phrase = *( HTAB / SP / VCHAR / obs-text )
        try!(self.read_into_working_space(
            0, |o| o == grammar::core::HTAB || o == grammar::core::SP ||
            // NOTE: I am *disallowing* obs-text (0x80-0xff) for now, so that
            // it can be a string. This may not be a good idea. Evaluate.
            grammar::core::is_vchar(o)));

        Ok((self.step(), status))
    }
}

impl<R: Read, MT: MessageType::Impl> Parser<R, MT, PreHeaderField> {
    /// Read a `header-line` from the reader.
    ///
    /// The next step of the state machine has `get_header_name()` and
    /// `get_header_value()` to retrieve the read values.
    pub fn read_header_line(mut self) -> ParseResult<Parser<R, MT,
                                                            PostHeaderField>> {
        // TODO: read header-name into working_space/ws_one_len

        let _ = parse_byte!(self.reader, b':');
        //self.consume_whitespace();

        // TODO: read header-value into working_space/ws_two_len

        //self.consume_line_ending();

        Ok(self.step())
    }
}

impl<R: Read, MT: MessageType::Impl> Parser<R, MT, PostHeaderField> {
    /// Get the header-name from the header-line, as raw bytes.
    #[inline]
    pub fn get_header_name<'a>(&'a self) -> &'a [u8] {
        self.get_working_space_one()
    }

    /// Get the header-value from the header-line, as raw bytes.
    #[inline]
    pub fn get_header_value<'a>(&'a self) -> &'a [u8] {
        self.get_working_space_two()
    }
}

/*
fn example() {
    let request;
    let request = request.read_request_line();
    let method = request.get_method();
    let v = request.get_http_version();
    { do something with request.get_request_target(); }

    let request = loop {
        let request = match request.read_header_line() {
            YesThereIsAHeaderLine(request) => request,
            YesWeHaveNoHeaders(request) => break request,
        };
        let (name, value) = request.get_header();
    }

    let request = request.get_body();
}

fn example() {
    let request;
    let (method, v, request_target) = request.read_request_line();

    while let Some((name, value)) = request.read_header_line() {
        ...
    }

    let request = request.get_body();
}
*/
