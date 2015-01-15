//! HTTP headers.

use std::any::{Any, TypeId};
use std::fmt;
use std::borrow::Cow;
use std::marker;
use std::mem;

use std::collections::hash_map::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};

use self::internals::Item;
pub use self::internals::{TypedRef, TypedListRef, RawRef};

mod internals;

/// A trait defining the parsing of a header from a raw value.
pub trait ToHeader {
    /// Parse a header from a header field value, returning some value if successful or `None` if
    /// parsing fails.
    ///
    /// For single‐type headers, this will only be called once, with the single field value. For
    /// list‐type headers, this will be called for each value in each comma‐separated field value.
    /// That is, for the combination of HTTP headers `Foo: bar, baz` and `Foo: quux`, any `Foo`
    /// header will get this method called three times with the raw values `b"bar"`, `b"baz"` and
    /// `b"quux"` in order. If any individual one of these fails to parse, it is no problem—that
    /// individual item will be the only one that is dropped. It is only where there is a genuine
    /// syntax error (e.g. an unclosed `quoted-string`) where an entire line will be dropped—and
    /// even then, any other lines will still be handled if possible.
    fn parse(raw_field_value: &[u8]) -> Option<Self>;
}

/// The data type of an HTTP header for encoding and decoding.
pub trait Header: Any + HeaderClone {
    /// Convert the header to its raw value, writing it to the formatter.
    ///
    /// Implementers MUST only write `SP` (0x20), `HTAB` (0x09), `VCHAR` (visible US-ASCII
    /// characters, 0x21–0x7E) or `obs`-text (0x80–0xFF), though the use of obs-text is not
    /// advised. Things like carriage returns, line feeds and null bytes are Definitely Forbidden.
    /// For list‐style headers there is an additional restriction: commas are only permitted inside
    /// appropriately quoted strings, on pain of Undefined Behaviour. This is probably a good rule
    /// to stick to in general, partially so on account of there being nothing stopping a
    /// Header‐implementing type from being used as a list‐style header.
    //
    // (Well, I guess for HTTP/1 you could *probably* get away with obs-fold (e.g. `CR LF SP`), but
    // I can’t remember off the top of my head how that’ll work for HTTP/2, and I’m definitely not
    // advertising it in the public docs. Hence double slashes, not triple.)
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result;

    /// Convert the header to its raw value, producing a new byte vector.
    ///
    /// The default implementation will almost always be sufficient, being based on the `fmt`
    /// method of this trait. This method only exists as it is because I consider it conceivable at
    /// present that there may be cases where there is a better choice. It might be shifted out of
    /// the trait later.
    #[unstable = "might be removed from the trait"]
    fn to_raw(&self) -> Vec<u8> {
        format!("{}", HeaderDisplayAdapter(&*self)).into_bytes()
    }

    /// Convert the header to its raw value, consuming self.
    ///
    /// The `Box<Self>` aspect is to satisfy object safety. Hopefully this measure won’t be
    /// necessary at some point in the future.
    ///
    /// The default implementation will almost always be sufficient, being based on the `to_raw`
    /// method of this trait. This method only exists as it is because I consider it conceivable at
    /// present that there may be cases where there is a better choice. It might be shifted out of
    /// the trait later.
    #[unstable = "might be removed from the trait"]
    fn into_raw(self: Box<Self>) -> Vec<u8> {
        self.to_raw()
    }
}

mopafy!(Header);

/// `Clone`, but producing boxed headers.
#[doc(hidden)]
pub trait HeaderClone {
    /// Clone self as a boxed header.
    #[inline]
    fn clone_boxed(&self) -> Box<Header>;
}

impl<T: Header + Clone> HeaderClone for T {
    fn clone_boxed(&self) -> Box<Header> {
        Box::new(self.clone())
    }
}

impl<T: ToHeader + Header + Clone + 'static> Header for Vec<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;
        for h in self {
            if first {
                first = false;
            } else {
                try!(f.write_str(", "));
            }
            try!(h.fmt(f));
        }
        Ok(())
    }
}

// Would that this one was not necessary. But alas, it is until we get negative impl bounds in the
// language, so that Headers.set can work. (It’ll still be convenient to have the Header impl for
// Vec<T>, but we can negate on the ToHeader-ness, subtle though the point be.)
impl<T: ToHeader + Header + Clone + 'static> ToHeader for Vec<T> {
    fn parse(_raw_field_value: &[u8]) -> Option<Self> {
        panic!("dummy impl, Vec<T> only implements ToHeader to work around type system deficiencies")
    }
}

/// A header marker, providing the glue between the header name and a type for that header.
///
/// Standard usage of this is very simple unit-struct marker types, like this:
///
/// ```rust,ignore
/// #[macro_use] extern crate httpcommon;
/// use httpcommon::headers::{ToHeader, Header};
///
/// // The header data type
/// #[derive(Clone)]
/// pub struct Foo {
///     ...
/// }
///
/// impl ToHeader for Foo {
///     ...
/// }
///
/// impl Header for Foo {
///     ...
/// }
///
/// // The marker type for accessing the header and specifying the name
/// define_single_header_marker!(FOO: Foo = "foo");
/// ```
///
/// Then, accessing the header is done like this:
///
/// ```rust
/// # #[macro_use] extern crate httpcommon;
/// # #[derive(Clone)] struct Foo;
/// # impl httpcommon::headers::ToHeader for Foo {
/// #     fn parse(_raw: &[u8]) -> Option<Foo> { Some(Foo) }
/// # }
/// # impl httpcommon::headers::Header for Foo {
/// #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { Ok(()) }
/// # }
/// # define_single_header_marker!(FOO: Foo = "foo");
/// # struct Request { headers: httpcommon::headers::Headers }
/// # fn main() {
/// # let mut request = Request { headers: httpcommon::headers::Headers::new() };
/// # request.headers.set(FOO, Foo);
/// // Of course, this is assuming that we *know* the header is there
/// let foo = request.headers.get(FOO).unwrap().into_owned();
/// request.headers.set(FOO, foo);
/// # }
/// ```
///
/// And lo! `foo` is a `Foo` object corresponding to the `foo` (or `Foo`, or `fOO`, &c.) header in
/// the request.
// PhantomFn is for 'a’s sake. Ugly, very.
pub trait Marker<'a>: marker::PhantomFn<&'a ()> {
    /// The fundamental header type under consideration (for list headers, H rather than Vec<H>).
    type Base: Header + ToHeader + Clone;

    /// The output of Headers.get(marker).
    type Get: internals::Get<'a>;

    /// The output of Headers.get_mut(marker).
    type GetMut: internals::GetMut<'a>;

    /// The argument to Headers.set(marker, ___).
    type Set: Header + ToHeader + Clone + 'static;

    /// The name of the header that shall be used for retreiving and setting.
    ///
    /// Normally this will be a static string, but occasionally it may be necessary to define it at
    /// runtime, for dynamic header handling.
    fn header_name(&self) -> Cow<'static, str>;
}

/// Define a single-type header marker.
///
/// Examples:
///
/// ```rust,ignore
/// define_single_header_marker!(FOO: Foo = "foo");
/// define_single_header_marker!(DATE: Tm = "date");
/// define_single_header_marker!(CONTENT_LENGTH: uint = "content-length");
/// ```
#[macro_export]
macro_rules! define_single_header_marker {
    ($marker:ident: $ty:ty = $name:expr) => {
        struct $marker;

        impl<'a> $crate::headers::Marker<'a> for $marker {
            type Base = $ty;
            type Get = Option<$crate::headers::TypedRef<'a, $ty>>;
            type GetMut = Option<&'a mut $ty>;
            type Set = $ty;

            fn header_name(&self) -> ::std::borrow::Cow<'static, str> {
                ::std::borrow::Cow::Borrowed($name)
            }
        }
    }
}

/// Define a single-type header marker.
///
/// Examples:
///
/// ```rust,ignore
/// define_list_header_marker!(ALLOW: Method = "allow");
/// define_list_header_marker!(ACCEPT: Accept = "accept");
/// ```
#[macro_export]
macro_rules! define_list_header_marker {
    ($marker:ident: $ty:ty = $name:expr) => {
        struct $marker;

        impl<'a> $crate::headers::Marker<'a> for $marker {
            type Base = $ty;
            type Get = $crate::headers::TypedListRef<'a, $ty>;
            type GetMut = &'a mut Vec<$ty>;
            type Set = Vec<$ty>;

            fn header_name(&self) -> ::std::borrow::Cow<'static, str> {
                ::std::borrow::Cow::Borrowed($name)
            }
        }
    }
}

impl Clone for Box<Header> {
    fn clone(&self) -> Box<Header> {
        self.clone_boxed()
    }
}

impl Header for Box<Header> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl Header for &'static Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

/// A collection of HTTP headers.
///
/// Usage
/// -----
///
/// The primary methods you will care about are:
///
/// Typed header access
/// ```````````````````
///
/// Unlike most HTTP libraries, this one cares about correctness and strong typing; headers are
/// semantically typed values, not just sequences of characters. This may lead to some surprises for
/// people used to other environments. For example, you might think that the `Connection` header is
/// a scalar, having seen it with the value `close` most frequently when present; therefore you
/// might expect to check `*request.headers.get_ref(CONNECTION) == Close`. Well, it's not: it's
/// actually a linear value, `Vec<Connection>` instead of `Connection`, so you'll actually be
/// wanting to check something more like `request.headers.get_ref(CONNECTION).map(|c|
/// c.contains(&Close)) == Some(true)`. Yes, this is more cumbersome than what you might write in
/// another language, such as `request.headers["Connection"] == "close"`, but it's actually correct,
/// whereas the one people would often write is very definitely incorrect.
///
/// There are four methods for this:
///
/// - `get`: cloned value, if it exists.
/// - `get_ref`: reference to the value, if it exists.
/// - `get_mut_ref`: mutable reference to the value, if it exists.
/// - `set`: assign the value.
///
/// One thing out of the ordinary to be aware of is that all of these methods take `&mut self`, even
/// `get` and `get_ref`; this is not ideal, but it is thus for a very good reason, an outcome of the
/// hybrid typed/raw approach employed. The main practical effect of this is that you cannot take
/// references to more than one header at once; where possible, use `get_ref`, but it is
/// acknowledged that it will not always be feasible to use it: this is why `get` exists, which
/// clones the value, thus releasing the lock on the header collection.
///
/// Raw header access
/// `````````````````
///
/// Largely you should prefer typed access, but sometimes raw header access is convenient or even
/// necessary.
///
/// There are again four methods for this, corresponding to the typed techniques:
///
/// - `get_raw`: cloned value, if it exists.
/// - `get_raw_ref`: reference to the value, if it exists.
/// - `get_raw_mut_ref`: mutable reference to the value, if it exists.
/// - `set_raw`: assign the value.
///
/// Aside: what is a header?
/// ------------------------
///
/// When we speak of a header in this library, we are not referring to the HTTP concept of a *header
/// field*; we are dealing with a slightly higher abstraction than that.
///
/// In HTTP/1.1, a message header is defined like this (RFC 7230, section 3.2 Header Fields):
///
/// ```ignore
///     header-field   = field-name ":" OWS field-value OWS
///
///     field-name     = token
///     field-value    = *( field-content / obs-fold )
///     field-content  = field-vchar [ 1*( SP / HTAB ) field-vchar ]
///     field-vchar    = VCHAR / obs-text
///
///     obs-fold       = CRLF 1*( SP / HTAB )
///                    ; obsolete line folding
///                    ; see Section 3.2.4    message-header = field-name ":" [ field-value ]
/// ```
///
/// This is something all web developers should be at least basically familiar with.
///
/// The interesting part comes a little later in that section and is to do with how message headers
/// *combine*:
///
/// ```ignore
/// A sender MUST NOT generate multiple header fields with the same field
/// name in a message unless either the entire field value for that
/// header field is defined as a comma-separated list [i.e., #(values)]
/// or the header field is a well-known exception (as noted below).
///
/// A recipient MAY combine multiple header fields with the same field
/// name into one "field-name: field-value" pair, without changing the
/// semantics of the message, by appending each subsequent field value to
/// the combined field value in order, separated by a comma.  The order
/// in which header fields with the same field name are received is
/// therefore significant to the interpretation of the combined field
/// value; a proxy MUST NOT change the order of these field values when
/// forwarding a message.
/// ```
///
/// In this library, what we call a header is not a single message header, but rather the
/// combination of all message headers with the same field name; that is, a field-name plus *all*
/// related field-values. One can still access them separately through the raw interface, but the
/// preferred technique is accessing them through the typed interface, where all such
/// identically-named message headers will be merged.
///
/// Representation
/// --------------
///
/// A knowledge of how headers are represented internally may assist you in using them efficiently.
///
/// Headers are stored in a hash map; the keys are header names and the values are what for the
/// purpose of this description will be dubbed *items*.
///
/// At this point it is worth recalling that in a request or response, there can be multiple header
/// fields with the same name; this is why the raw representation of each header item is `Vec<Vec<u8>>`
/// rather than `Vec<u8>` each header field can
/// Each header name is thus associated with an
/// item.
#[derive(PartialEq)]
pub struct Headers {
    data: HashMap<Cow<'static, str>, Item>,
}

impl Headers {
    /// Construct a new header collection.
    pub fn new() -> Headers {
        Headers {
            data: HashMap::new(),
        }
    }

    /// Get a reference to a header value.
    ///
    /// The interface is strongly typed; see TODO for a more detailed explanation of how it works.
    pub fn get<'a, M: Marker<'a>>(&'a self, marker: M) -> M::Get {
        internals::Get::get(self.data.get(&marker.header_name()))
    }

    /// Get a mutable reference to a header value.
    ///
    /// The interface is strongly typed; see TODO for a more detailed explanation of how it works.
    pub fn get_mut<'a, M: Marker<'a>>(&'a mut self, marker: M) -> M::GetMut {
        internals::GetMut::get_mut(self.data.entry(marker.header_name()))
    }

    /// Set the named header to the given value.
    pub fn set<M: Marker<'static>>(&mut self, marker: M, value: M::Set) {
        // Houston, we have a minor problem here. Unlike Get and GetMut which were unambiguous,
        // here we have for single headers an impl for T and for list headers one for Vec<T>.
        // We’d like to do `internals::Set::set(self.data.entry(marker.header_name()), value)`,
        // but this wouldn’t work because of the conflicting Set implementations.
        // So what do we do? We cheat! Yay for cheating!
        let entry = self.data.entry(marker.header_name());
        if TypeId::of::<Vec<M::Base>>() == TypeId::of::<M::Set>() {
            // It’s a list header.
            // And now we want to transmute it, but we can’t do that so simply because of generics
            // and monomorphisation and blah blah blah. So we do even more black magic, copying the
            // value into a new type and forgetting the old value.
            // TODO: determine whether this is *efficient* when optimised, i.e. noop.
            let value_vec: Vec<M::Base> = unsafe { mem::transmute_copy(&value) };
            unsafe { mem::forget(value) }
            match entry.get() {
                Ok(item) => item.set_list_typed(value_vec),
                Err(vacant) => {
                    let _ = vacant.insert(Item::from_list_typed(value_vec));
                },
            }
        } else {
            // It’s a single header.
            match entry.get() {
                Ok(item) => item.set_single_typed(value),
                Err(vacant) => {
                    let _ = vacant.insert(Item::from_single_typed(value));
                },
            }
        }
    }

    /// Get the raw values of a header, by name.
    ///
    /// The returned value is a slice of each header field value.
    #[inline]
    pub fn get_raw<'a, M: Marker<'a>>(&'a self, header_marker: M) -> Option<RawRef> {
        self.data.get(&header_marker.header_name()).and_then(|item| item.raw())
    }

    /// Get a mutable reference to the raw values of a header, by name.
    ///
    /// The returned vector contains each header field value.
    #[inline]
    pub fn get_raw_mut<'a, M: Marker<'a>>
                      (&'a mut self, header_marker: M)
                      -> Option<&mut Vec<Vec<u8>>> {
        self.data.get_mut(&header_marker.header_name()).map(|item| item.raw_mut())
    }

    /// Set the raw value of a header, by name.
    ///
    /// This invalidates the typed representation.
    #[inline]
    pub fn set_raw<'a, M: Marker<'a>>(&'a mut self, header_marker: M, value: Vec<Vec<u8>>) {
        match self.data.entry(header_marker.header_name()) {
            Vacant(entry) => { let _ = entry.insert(Item::from_raw(value)); },
            Occupied(entry) => entry.into_mut().set_raw(value),
        }
    }

    /// Remove a header from the collection.
    /// Returns true if the named header was present.
    pub fn remove<'a, M: Marker<'a>>(&'a mut self, header_marker: &M) -> bool {
        self.data.remove(&header_marker.header_name()).is_some()
    }

    /// Returns true if the named header exists in the collection.
    pub fn contains<'a, M: Marker<'a>>(&'a self, header_marker: &M) -> bool {
        match self.data.get(&header_marker.header_name()) {
            Some(item) => item.is_valid(),
            None => false,
        }
    }

    // TODO: make this more like a normal collection. Compare with what I did for AnyMap.
    // Methods to consider adding as appropriate/possible: entry, capacity, reserve, shrink_to_fit,
    // iter, iter_mut, len, is_empty, drain, clear.
    // Also impl Debug.
}

/// An adapter which provides `std::fmt::Display` as equivalent to `Header.fmt`, so that you can
/// actually *use* the thing.
pub struct HeaderDisplayAdapter<'a, H: Header + ?Sized>(pub &'a H);

impl<'a, H: Header + ?Sized> fmt::Display for HeaderDisplayAdapter<'a, H> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test_broken)]
mod tests {
    use super::*;

    fn expect<H: Header + std::fmt::Display + Eq>(h: Option<H>, h_expected: H, raw: &[u8]) {
        let h = h.unwrap();
        assert_eq!(fmt(&h).as_slice(), raw);
        assert_eq!(h, h_expected);
    }

    #[test]
    fn test_basics() {
        let mut headers = Headers::new();

        assert_eq!(headers.get(EXPIRES), None);

        headers.set(EXPIRES, Past);
        assert_eq!(headers.mostly_get(&EXPIRES), &mut Typed(Box::new(Past)));
        expect(headers.get(EXPIRES), Past, b"0");
        assert_eq!(headers.get_raw("expires"), vec![vec![b'0']]);
        expect(headers.get(EXPIRES), Past, b"0");

        headers.remove(&EXPIRES);
        assert_eq!(headers.get(EXPIRES), None);

        assert_eq!(headers.get(DATE), None);
        let now = time::now();
        let now_raw = fmt(&now);
        headers.set(DATE, now.clone());
        expect(headers.get(DATE), now.clone(), now_raw.as_slice());
    }
}
