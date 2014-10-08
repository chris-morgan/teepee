//! Trait-based HTTP request parser.

use std::borrow::Cow;
use std::ptr;
use std::fmt;
use std::str;
use std::io::{self, Read};
use tendril::SliceExt;

use headers::Headers;
use method::Method;
use grammar::token::{Token, is_tchar};
use grammar::core::{CR, LF, SP, HTAB};

use self::Error::*;
use self::SpecificParseError::*;
use self::RawRequestTarget::*;

/// Any error encountered during parsing.
#[derive(Debug)]
pub enum Error {
    /// Any I/O error which means we should drop the connection.
    IoError(io::Error),
    /// An HTTP-message parse error.
    /// A server should respond 400 Bad Request; clients should probably
    /// complain of having received a bad response in some other way.
    ParseError(SpecificParseError),
    /// A field was longer than the buffer capacity and so could not be read.
    FieldTooLong,
}

/// The specific type of parse error encountered.
#[derive(Debug)]
pub enum SpecificParseError {
    /// The `method` was not a token.
    ///
    /// That is, at the start of the request there was not a sequence of tchars followed by a SP.
    BadMethod,

    /// The `request-target` specified was missing or not valid.
    ///
    /// This can include request-targets that would be legal with some methods but not with others,
    /// e.g. an asterisk-form for anything other than an OPTIONS request.
    BadRequestTarget,

    /// The `HTTP-version` did not follow the correct format.
    BadHttpVersion,

    /// A `header-field` was not syntactically valid.
    BadHeaderField,
}

macro_rules! parse_error {
    ($error:expr) => {
        //return Err(Error::ParseError($error));
        panic!("parse error {:?}", $error);
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
#[derive(PartialEq, Eq, Clone)]
pub enum RawRequestTarget<'a> {
    /// The most common form of request-target, and the form requests should be
    /// written in.
    OriginForm(Cow<'a, str>),
    /// A complete URL; most commonly used with proxies, but all servers MUST
    /// support it for future compatibility.
    AbsoluteForm(Cow<'a, str>),
    /// Only used for `CONNECT` requests through proxies.
    AuthorityForm(Cow<'a, str>),
    /// Only used for a server-wide `OPTIONS` request.
    AsteriskForm,
}

impl<'a> fmt::Debug for RawRequestTarget<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OriginForm(ref v) => f.write_str(&v),
            AbsoluteForm(ref v) => f.write_str(&v),
            AuthorityForm(ref v) => f.write_str(&v),
            AsteriskForm => f.write_str("*"),
        }
    }
}

impl<'a> RawRequestTarget<'a> {
    fn into_owned(self) -> RawRequestTarget<'static> {
        match self {
            RawRequestTarget::OriginForm(cow) =>
                RawRequestTarget::OriginForm(Cow::Owned(cow.into_owned())),
            RawRequestTarget::AbsoluteForm(cow) =>
                RawRequestTarget::AbsoluteForm(Cow::Owned(cow.into_owned())),
            RawRequestTarget::AuthorityForm(cow) =>
                RawRequestTarget::AuthorityForm(Cow::Owned(cow.into_owned())),
            AsteriskForm => AsteriskForm,
        }
    }
}

/// TODO.
pub struct BodyReader<R> {
    marker: ::std::marker::PhantomData<R>,
}

/// Parser!
pub struct Parser<R: Read, H: Handler> {
    inner: InnerBuffer<R>,
    handler: H,
}

impl<R: Read, H: Handler> Parser<R, H> {

    /// Construct a parser from the given reader with the given handler.
    pub fn new(reader: R, handler: H) -> Parser<R, H> {
        Parser {
            inner: InnerBuffer::new(reader),
            handler: handler,
        }
    }

    /*/// Deconstruct the parser to get the reader, buffered data and handler out.
    pub fn unwrap(self) -> (R, Vec<u8>, H) {
        (self.inner.reader, self.inner.buf, self.handler)
    }*/

    /// Parse the message!
    pub fn parse(&mut self) -> Result<(), Error> {
        macro_rules! b {
            () => {
                match self.inner.take_byte() {
                    Err(e) => return Err(e),
                    Ok(o) => o,
                }
            }
        }

        macro_rules! handler {
            ($method:ident$(, $args:expr)*) => {
                match self.handler.$method($($args),*) {
                    ParserInstruction::Continue => (),
                    ParserInstruction::Stop => {
                        unimplemented!()
                    }
                }
            }
        }

        macro_rules! parse_byte {
            ($expected:pat, $error:expr) => {
                match self.inner.take_byte() {
                    Ok(ok @ $expected) => ok,
                    Ok(_) => parse_error!($error),
                    Err(e) => return Err(e),
                }
            }
        }

        // RFC 7230, section 3.5 Message Parsing Robustness: "In the interest of robustness, a
        // server that is expecting to receive and parse a request-line SHOULD ignore at least one
        // empty line (CRLF) received prior to the request-line." Doing this for arbitrarily many
        // lines is probably not a great idea, so we'll go for just one line (CR or LF or CRLF).
        let _ = try!(self.inner.take_crlf(None));

        // Now we're onto the actual request-line. First up is `method`.
        self.inner.set_marker1_start();

        if try!(self.inner.take_bytes_while(is_tchar)) == 0 {
            parse_error!(SpecificParseError::BadMethod);
        }

        self.inner.set_marker1_end();

        let _ = parse_byte!(SP, SpecificParseError::BadMethod);

        /// The permissible forms of request-target are influenced by the method.
        /// Therefore we track which one we're dealing with.
        #[derive(PartialEq)]
        enum NotableMethod {
            /// The authority-form is only permitted for CONNECT requests (through proxies);
            /// they cannot use absolute-form or origin-form either.
            Connect,
            /// The asterisk-form is only permitted for OPTIONS requests.
            Options,
            /// Any other method may only be origin-form or absolute-form.
            TotallyBoring,
        }

        let notable_method = match self.inner.get_marker1() {
            b"CONNECT" => NotableMethod::Connect,
            b"OPTIONS" => NotableMethod::Options,
            _ => NotableMethod::TotallyBoring,
        };

        self.inner.set_marker2_start();

        #[derive(PartialEq)]
        enum Form { Origin, Absolute, Authority, Asterisk }
        // Next, we come to `request-target`. TODO: do a little more validation (notably, _ doesn't
        // cut it, check the grammar for authority and absolute-URI).
        let form = match b!() {
            b'/' if notable_method == NotableMethod::Connect => parse_error!(SpecificParseError::BadRequestTarget),
            b'/' => Form::Origin,
            b'*' if notable_method == NotableMethod::Options => Form::Asterisk,
            b'*' => parse_error!(SpecificParseError::BadRequestTarget),
            SP | HTAB | CR | LF => parse_error!(SpecificParseError::BadRequestTarget),
            _ if notable_method == NotableMethod::Connect => Form::Authority,
            _ => Form::Absolute,
        };

        let len = try!(self.inner.take_bytes_while(|b| b != SP && b != HTAB &&
                                                       b != CR && b != LF));
        if len > 0 && form == Form::Asterisk {
            parse_error!(SpecificParseError::BadRequestTarget);
        }
        self.inner.set_marker2_end();

        // Now comes `SP HTTP-version CRLF`. Or we might get the HTTP/0.9 `CRLF`.

        let version = match self.inner.take_byte() {
            Ok(SP) => {
                let _ = parse_byte!(b'H', SpecificParseError::BadHttpVersion);
                let _ = parse_byte!(b'T', SpecificParseError::BadHttpVersion);
                let _ = parse_byte!(b'T', SpecificParseError::BadHttpVersion);
                let _ = parse_byte!(b'P', SpecificParseError::BadHttpVersion);
                let _ = parse_byte!(b'/', SpecificParseError::BadHttpVersion);
                let major = parse_byte!(b'0'...b'9', SpecificParseError::BadHttpVersion) - b'0';
                let _ = parse_byte!(b'.', SpecificParseError::BadHttpVersion);
                let minor = parse_byte!(b'0'...b'9', SpecificParseError::BadHttpVersion) - b'0';
                try!(self.inner.take_crlf(Some(SpecificParseError::BadHttpVersion)));
                (major, minor)
            },
            Ok(CR) => {
                let _ = self.inner.optionally_take_byte(|b| b == LF);
                (0, 9)
            },
            Ok(LF) => (0, 9),
            Ok(_) => parse_error!(SpecificParseError::BadHttpVersion),
            Err(e) => return Err(e),
        };

        {
            let method = Method::from_token(unsafe {
                Token::from_slice_nocheck(self.inner.get_marker1())
            });

            let request_target = match form {
                Form::Asterisk => AsteriskForm,
                _ => {
                    let content = match str::from_utf8(self.inner.get_marker2()) {
                        Ok(ok) => Cow::Borrowed(ok),
                        Err(_) => parse_error!(SpecificParseError::BadRequestTarget),
                    };
                    match form {
                        Form::Origin => RawRequestTarget::OriginForm(content),
                        Form::Authority => RawRequestTarget::AuthorityForm(content),
                        Form::Absolute => RawRequestTarget::AbsoluteForm(content),
                        Form::Asterisk => unreachable!(),
                    }
                },
            };

            handler!(on_request_line, method, request_target, version);
        }
        self.inner.reset_markers();

        // Now we're onto the header fields.
        loop {
            // header-field = field-name ":" OWS field-value OWS

            // field-name = token
            self.inner.set_marker1_start();
            match try!(self.inner.take_byte()) {
                // CR or LF will mean "end of header fields".
                CR => {
                    let _ = try!(self.inner.optionally_take_byte(|b| b == LF));
                    break;
                },
                LF => break,
                b if is_tchar(b) => (),
                _ => parse_error!(SpecificParseError::BadHeaderField),
            }
            let _ = try!(self.inner.take_bytes_while(is_tchar));
            self.inner.set_marker1_end();

            // ":" OWS
            let _ = parse_byte!(b':', SpecificParseError::BadHeaderField);
            let _ = try!(self.inner.take_bytes_while(|b| b == SP || b == HTAB));

            // field-value = *( field-content / obs-fold )
            // field-content = field-vchar [ 1*( SP / HTAB ) field-vchar ]
            // field-vchar = VCHAR / obs-text
            // obs-fold = CRLF 1*( SP / HTAB )
            // Note that the header-field is permitted to have OWS at the end, so for a header
            // field like "Key: value \r\n", the value should be "value" rather than "value ".
            // For simplicity and to cope with the most common case of no whitespace efficiently,
            // this check is done at the end.
            self.inner.set_marker2_start();
            loop {
                let _ = try!(self.inner.take_bytes_while(|b| b != CR && b != LF));
                let cr = try!(self.inner.optionally_take_byte(|b| b == CR));
                let lf = try!(self.inner.optionally_take_byte(|b| b == LF));
                debug_assert!(cr || lf);
                match try!(self.inner.peek_byte()) {
                    SP | HTAB => {
                        // obs-fold; we turn the CR and/or LF, AND the SP/HTAB, into as many SP.
                        // This way we don't need to mess about with moving data inside the buffer.
                        if cr && lf {
                            self.inner.buf[self.inner.pos - 2] = SP;
                        }
                        self.inner.buf[self.inner.pos - 1] = SP;
                        self.inner.buf[self.inner.pos] = SP;
                        let _ = try!(self.inner.take_byte());  // Can't fail, try! for consistency
                    },
                    _ => break,
                }
            }
            self.inner.set_marker2_end();
            {
                let (name, value) = self.inner.take_marked_areas();

                // Strip the trailing CRLF from the header-value.
                // Then strip the trailing OSP from the header-value.
                // The combination of the two leads to the mildly ambiguous behaviour of treating
                // a trailing obs-fold as OWS and stripping it. This is what I think should be
                // done, but it's not what the grammar would actually have one do.
                let value = match value.iter().rposition(|&b| b != CR && b != LF &&
                                                              b != SP && b != HTAB) {
                    Some(n) => &value[..n + 1],
                    None => { let v: &[u8] = &[]; v },
                };

                handler!(on_header_field, unsafe { Token::from_slice_nocheck(name) }, value);
            }
        }

        Ok(())
    }

}

struct InnerBuffer<R: Read> {
    reader: R,
    /// The buffer around the reader, storing prepared data.
    buf: Vec<u8>,
    marker1_start: Option<usize>,
    marker1_end: Option<usize>,
    marker2_start: Option<usize>,
    marker2_end: Option<usize>,
    pos: usize,
}

impl<R: Read> InnerBuffer<R> {
    /// Create a new `InnerBuffer` with a 64KB buffer.
    ///
    /// See
    pub fn new(reader: R) -> InnerBuffer<R> {
        InnerBuffer::new_from_buf(reader, Vec::with_capacity(65536))
    }

    /// Create a new `InnerBuffer` with the specified buffer.
    ///
    /// The full reserved capcity of the buffer will be used, and any data already in the vector
    /// will be used before the reader is read from; that is to say, you can prefill the buffer.
    ///
    /// You should be careful in the size of buffer you select, for interoperability, for any
    /// elements yielded from the parser as a slice of it will not be able to be larger.
    ///
    /// As an example of this in practice, RFC 7230, section 3.1.1 (Request Line) says "It is
    /// RECOMMENDED that all HTTP senders and recipients support, at a minimum, request-line
    /// lengths of 8000 octets." This translates to a recommendation that the combination of method
    /// and request-target should be permitted to be at least 7988 bytes. As it happens, these two
    /// are treated separately in this parser, so a 4KB buffer would permit a method of 4KB and a
    /// request-target of 4KB, which is greater than the 8000 octets mentioned, but not a method of
    /// 1KB and request-target of 7KB (a much more plausible scenario). For these sorts of reasons,
    /// we strongly recommend that you do not use a buffer of less than 8KB (8,192 bytes), with
    /// a practical recommendation of 64KB (65,536 bytes/octets), a convenient default which
    /// purportedly balances "things" well.
    ///
    /// You can specify the size of the buffer by passing in as your buffer `Vec::with_capacity`
    pub fn new_from_buf(reader: R, buf: Vec<u8>) -> InnerBuffer<R> {
        InnerBuffer {
            reader: reader,
            buf: buf,
            marker1_start: None,
            marker1_end: None,
            marker2_start: None,
            marker2_end: None,
            pos: 0,
        }
    }

    /// Start the first marked region which will be kept in the buffer until taken.
    ///
    /// This may only be called before any marker methods, or after `take_marked_areas` or
    /// `reset_markers`.
    ///
    /// Multiple calls, to adjust the marker position after setting it initially, are fine.
    pub fn set_marker1_start(&mut self) {
        debug_assert!(self.marker1_end == None);
        debug_assert!(self.marker2_start == None);
        debug_assert!(self.marker2_end == None);
        self.marker1_start = Some(self.pos);
    }

    /// Finish the first marked region.
    ///
    /// This may only be called after `set_marker1_start` and before `set_marker2_start`.
    ///
    /// Multiple calls, to adjust the marker position after setting it initially, are fine.
    pub fn set_marker1_end(&mut self) {
        debug_assert!(self.pos >= self.marker1_start.unwrap());
        debug_assert!(self.marker2_start == None);
        debug_assert!(self.marker2_end == None);
        self.marker1_end = Some(self.pos);
    }

    /// Get the contents of the first marked region.
    ///
    /// This may only be called after `set_marker1_end`.
    pub fn get_marker1(&self) -> &[u8] {
        &self.buf[self.marker1_start.unwrap()..self.marker1_end.unwrap()]
    }

    /// Start the second marked region.
    ///
    /// This may only be called after `set_marker1_end` and before `set_marker2_end`.
    ///
    /// Multiple calls, to adjust the marker position after setting it initially, are fine.
    pub fn set_marker2_start(&mut self) {
        debug_assert!(self.marker1_start != None);
        debug_assert!(self.marker1_end != None);
        debug_assert!(self.marker2_end == None);
        self.marker2_start = Some(self.pos);
    }

    /// Finish the second marked region.
    ///
    /// This may only be called after `set_marker2_start` and before `take_marked_areas` or
    /// `reset_markers`.
    ///
    /// Multiple calls, to adjust the marker position after setting it initially, are fine.
    pub fn set_marker2_end(&mut self) {
        debug_assert!(self.marker1_start != None);
        debug_assert!(self.marker1_end != None);
        debug_assert!(self.pos >= self.marker2_start.unwrap());
        self.marker2_end = Some(self.pos);
    }

    /// Get the contents of the second marked region.
    ///
    /// This may only be called after `set_marker2_end`.
    pub fn get_marker2(&self) -> &[u8] {
        &self.buf[self.marker2_start.unwrap()..self.marker2_end.unwrap()]
    }

    /// Clear the markers.
    ///
    /// After calling this, you may call `set_marker1_start` again.
    pub fn reset_markers(&mut self) {
        self.marker1_start = None;
        self.marker1_end = None;
        self.marker2_start = None;
        self.marker2_end = None;
    }

    /// Retrieve the marked areas and reset the markers.
    ///
    /// Returns all the contents that have been read since `start_mark` was called.
    ///
    /// This may only be called after `set_marker2_end`.
    pub fn take_marked_areas(&mut self) -> (&[u8], &[u8]) {
        (&self.buf[self.marker1_start.take().unwrap()..self.marker1_end.take().unwrap()],
         &self.buf[self.marker2_start.take().unwrap()..self.marker2_end.take().unwrap()])
    }

    /// Peek the next byte and consume it if it matches the predicate.
    ///
    /// Returns `Ok(true)` if the next byte matches the predicate and is therefore consumed.
    /// Returns `Ok(false)` if the next byte does not match and is therefore not consumed.
    /// Returns `Err` if there is an error reading.
    /// TODO: this includes EOF, is that really reasonable?
    #[inline]
    pub fn optionally_take_byte<F: FnOnce(u8) -> bool>(&mut self, pred: F) -> Result<bool, Error> {
        if pred(try!(self.peek_byte())) {
            self.pos += 1;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[inline]
    pub fn take_crlf(&mut self, error_if_no_crlf: Option<SpecificParseError>)
                    -> Result<(), Error> {
        let cr = try!(self.optionally_take_byte(|b| b == CR));
        let lf = try!(self.optionally_take_byte(|b| b == LF));
        match error_if_no_crlf {
            Some(e) => {
                if !cr && !lf {
                    Err(Error::ParseError(e))
                } else {
                    Ok(())
                }
            },
            _ => Ok(()),
        }
    }

    /// Consume bytes as they match the predicate.
    ///
    /// Returns `Ok` with the number of bytes that matched the predicate and were consumed.
    /// Returns `Err` if there is an error reading.
    #[inline]
    pub fn take_bytes_while<F: Fn(u8) -> bool>(&mut self, pred: F) -> Result<usize, Error> {
        let mut n = 0;
        while pred(try!(self.peek_byte())) {
            self.pos += 1;
            n += 1;
        }
        Ok(n)
    }

    /// Take a look at the next byte, but don't consume it.
    #[inline]
    pub fn peek_byte(&mut self) -> Result<u8, Error> {
        let byte = match self.buf.get(self.pos) {
            Some(&byte) => byte,
            // Run out of bytes, must read more (the slow path, definitely)
            None => try!(self.read_more_please()),
        };
        Ok(byte)
    }

    /// Read the next byte and consume it.
    #[inline]
    pub fn take_byte(&mut self) -> Result<u8, Error> {
        let byte = try!(self.peek_byte());
        self.pos += 1;
        Ok(byte)
    }

    #[cold]
    #[inline(never)]
    fn read_more_please(&mut self) -> Result<u8, Error> {
        // First of all, do we have a marker active? If we do, we can't throw those bytes away.
        match self.marker1_start {
            None => {
                // nothing special to do, just set the position back to the start.
                self.pos = 0;
            },
            Some(0) => {
                // The marked field has filled the entire buffer. This simply won't do;
                // we can't do anything meaningful with it and must complain.
                // This may well be the consequence of malicious user input.
                return Err(Error::FieldTooLong)
            },
            Some(old_marker) => {
                self.marker1_start = Some(0);
                match self.marker1_end {
                    Some(ref mut m) => *m -= old_marker,
                    None => (),
                }
                match self.marker2_start {
                    Some(ref mut m) => *m -= old_marker,
                    None => (),
                }
                match self.marker2_end {
                    Some(ref mut m) => *m -= old_marker,
                    None => (),
                }
                self.pos -= old_marker;
                // TODO(Chris): as a possible future optimisation, we could keep track of a marker
                // maximum length, and not move if we have enough spare at the end. But as a
                // general rule, we shouldn't be hitting this stuff frequently at all, so it's
                // probably a minor optimisation.
                unsafe {
                    let dst = self.buf.as_mut_ptr();
                    let src = self.buf.as_ptr().offset(old_marker as isize);
                    let len = self.pos;
                    ptr::copy(src, dst, len);
                }
            }
        }

        // We want to be able to use the entire buffer capacity for the read, so we set the length.
        // There will probably be uninitialised or uncleared data at the end, but we're only
        // writing to it so that's OK.
        let capacity = self.buf.capacity();
        unsafe { self.buf.set_len(capacity) }

        let bytes_read = match self.reader.read(&mut self.buf[self.pos..]) {
            Ok(bytes) => bytes,
            Err(io_error) => {
                unsafe { self.buf.set_len(self.pos) }
                return Err(Error::IoError(io_error))
            },
        };
        assert!(bytes_read > 0);

        // Now let's set the length again, for Safety and Happiness and Great Good, cutting off
        // that junk data that we don't care about.
        unsafe { self.buf.set_len(self.pos + bytes_read) }

        Ok(*unsafe { self.buf.get_unchecked(self.pos) })
    }
}

/// Directions to the parser about what to do next.
///
/// This is the type returned by all the `Handler` methods.
// unstable: may be switched to bitflags should some more operations appear desirable
pub enum ParserInstruction {
    /// Keep going. This is normally what you want.
    Continue,
    /// Stop parsing. You will want to be a little careful in using this as it will leave the
    /// reader stuck in the middle of the HTTP message. It should normally only be used if you're
    /// reading a message from a remote host and are going to terminate the connection (possibly
    /// after sending an error message).
    Stop,
}

/// The methods are in the order that they will be called.
///
/// All methods take `&mut self` and `&mut Parser`, and some then take
/// additional arguments.
///
/// ```abnf
/// request-line = method SP request-target SP HTTP-version CRLF
/// ```
///
pub trait Handler {
    /// The HTTP message has begun.
    ///
    /// Because many (perhaps most) implementations will not need to do anything here,
    /// there is a default implementation that does nothing.
    fn on_message_begin(&mut self) -> ParserInstruction { ParserInstruction::Continue }

    /// The `request-line` has been read.
    /// This comprises the `method`, `request-target` and `HTTP-version`.
    ///
    /// The HTTP-version digits are defined as in the range 0-9.
    fn on_request_line(&mut self, method: Method, request_target: RawRequestTarget,
                       http_version: (u8, u8)) -> ParserInstruction;

    /// A `header-field` has been read.
    /// This comprises a `field-name` and a `field-value`.
    ///
    /// As far as the `obs-fold` rule is concerned, it is accepted and converted to as many SP
    /// bytes. That is, if the header value has `CR LF SP` inside it, it will become `SP SP SP`.
    /// According to RFC 7230, section 3.2.4, it would be legal, if not in a `message/http`
    /// container, to "reject the message by sending a 400 (Bad Request), preferably with a
    /// representation explaining that obsolete line folding is unacceptable", but "[replacing it]
    /// with one or more SP octets prior to interpreting the field value or forwarding the message
    /// downstream" is acceptable behaviour, and is more straightforward, so Teepee simply chooses
    /// to forcibly follow that behaviour.
    fn on_header_field(&mut self, field_name: Token, field_value: &[u8]) -> ParserInstruction;

    /// The header fields are all finished and the body is about to come.
    ///
    /// Because many (perhaps most) implementations will not need to do anything here,
    /// there is a default implementation that does nothing.
    fn on_headers_complete(&mut self) -> ParserInstruction { ParserInstruction::Continue }

    /// ARGH! TODO! PANIC! I don’t know what goes here.
    fn on_body<R: Read>(&mut self, reader: BodyReader<R>) -> ParserInstruction;

    /// The HTTP message has finished.
    ///
    /// There is no default implementation for this method because you should probably do something
    /// with `keep_alive`.
    fn on_message_complete(&mut self, keep_alive: bool) -> ParserInstruction;
}

macro_rules! as_expr { ($expr:expr) => ($expr) }

macro_rules! headers {
    ($($name:tt: $value:expr),*) => {{
        let mut headers = Headers::new();
        $(
            headers.insert_raw_line(as_expr!($name), $value);
        )*
        headers
    }};
    ($($name:tt: $value:expr),*,) => (headers!($($name:expr: $value:expr),*));
}

#[test]
fn test_eager_request_parsing() {
    use std::io::MemReader;

    let mut parser = Parser::new(MemReader::new(b"\
        GET / HTTP/1.1\r\n\
        Header: value\r\n\
        Header2: value2\r\n obs-fold and trailing whitespace \r\n\
        Header3:value\r\n \r\n\
        Header4:\t    loads of white   \r\n\
        Header4: and an extra line!\r\n\
        Header5:\r\n\
        \r\n".to_vec()), EagerRequest::blank());
    match parser.parse() {
        Ok(_) => (),
        Err(e) => {
            panic!("huh!? {:?}", e)
            //println!("«{:?}»",
            //         ::std::str::from_utf8(&parser.inner.buf[parser.inner.pos..])),
        }
    }
    //parser.parse().unwrap();
    assert_eq!(parser.handler, EagerRequest {
        method: ::method::Get,
        request_target: OriginForm(Cow::Borrowed(b"/")),
        http_version: (1, 1),
        headers: headers![
            "Header": b"value",
            "Header2": b"value2   obs-fold and trailing whitespace",
            "Header3": b"value",
            "Header4": b"loads of white",
            "Header4": b"and an extra line!",
            "Header5": b""],
        body: None,
    });
}

/// A request, read eagerly from a reader and stored in a convenient struct.
///
/// This may not be the most efficient way of handling things in many cases, but it is very easy.
pub struct EagerRequest {
    /// The `method` read from the request.
    pub method: Method<'static>,
    /// The `request-target` read from the request.
    pub request_target: RawRequestTarget<'static>,
    /// The `HTTP-version` read from the request.
    pub http_version: (u8, u8),
    ///// A vector of (`field-name`, `field-value`), in all comprising the `header-field`s.
    //pub header_fields: Vec<(Token<'static>, Vec<u8>)>,
    /// The collection of `header-field`s read from the request.
    pub headers: Headers,
    /// The message-body read from the request, if present.
    pub body: Option<Vec<u8>>,
}

impl fmt::Debug for EagerRequest {
    /// Formats the message as approximately HTTP, but with just LF instead of CR LF line endings.
    /// The body is also not done in the same way.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{method} {request_target:?} HTTP/{major}.{minor}\n",
                    method = self.method,
                    request_target = self.request_target,
                    major = self.http_version.0,
                    minor = self.http_version.1));
        try!(write!(f, "{}", self.headers));
        /*for &(ref name, ref value) in self.header_fields.iter() {
            try!(write!(f, "{}: ", *name));
            try!(f.write_str(value));
            try!(f.write_str("\n"));
        }*/
        try!(f.write_str("\n"));
        match self.body {
            None => f.write_str("<no body>"),
            // Forgive me for this gross violation of fmt’s str guarantees. TODO: eradicate it.
            Some(ref body) => f.write_str(unsafe { str::from_utf8_unchecked(body) }),
        }
    }
}

impl PartialEq for EagerRequest {
    fn eq(&self, other: &EagerRequest) -> bool {
        self.method == other.method &&
        self.request_target == other.request_target &&
        self.http_version == other.http_version &&
        self.headers == other.headers &&
        //self.header_fields == other.header_fields &&
        self.body == other.body
    }
}

impl Eq for EagerRequest { }

impl EagerRequest {
    /// Construct a blank `EagerRequest` object with cheap but memory-safe dummy data.
    pub fn blank() -> EagerRequest {
        EagerRequest {
            method: Method::from_token(unsafe { Token::from_slice_nocheck(b"UNINITIALISED") }),
            request_target: AsteriskForm,
            http_version: (0, 0),
            headers: Headers::new(),
            //header_fields: vec![],
            body: None,
        }
    }
}

impl Handler for EagerRequest {
    fn on_request_line(&mut self, method: Method, request_target: RawRequestTarget,
                       http_version: (u8, u8)) -> ParserInstruction {
        self.method = method.into_owned();
        self.request_target = request_target.into_owned();
        self.http_version = http_version;
        ParserInstruction::Continue
    }

    fn on_header_field(&mut self, field_name: Token, field_value: &[u8]) -> ParserInstruction {
        self.headers.insert_raw_line(field_name.to_tendril(), field_value.to_tendril());
        ParserInstruction::Continue
    }

    fn on_body<R: Read>(&mut self, _reader: BodyReader<R>) -> ParserInstruction {
        unimplemented!();
        //ParserInstruction::Continue
    }

    fn on_message_complete(&mut self, _keep_alive: bool) -> ParserInstruction {
        ParserInstruction::Continue
    }
}
