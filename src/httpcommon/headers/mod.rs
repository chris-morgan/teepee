//! HTTP headers.

use std::any::{Any, AnyRefExt};
use std::cast::{transmute, transmute_copy};
use std::intrinsics::TypeId;
use std::fmt;
use std::io::MemWriter;
use std::raw::TraitObject;
use std::str::{IntoMaybeOwned, SendStr};

use collections::hashmap::HashMap;

/// The data type of an HTTP header for encoding and decoding.
pub trait Header: Any {
    /// Parse a header from one or more header field values, returning some value if successful or
    /// `None` if parsing fails.
    ///
    /// Most headers only accept a single header field (i.e. they should return `None` if the outer
    /// slice contains other than one value), but some may accept multiple header field values; in
    /// such cases, they MUST be equivalent to having them all as a comma-separated single field
    /// (RFC 2616), with exceptions for things like dropping invalid values.
    fn parse_header(s: &[Vec<u8>]) -> Option<Self>;

    /// Introducing an `Err` value that does *not* come from the writer is incorrect behaviour and
    /// may lead to task failure in certain situations. (The primary case where this will happen is
    /// accessing a cached Box<Header> object as a different type; then, it is shoved into a buffer
    /// through fmt_header and then back into the new type through parse_header. Should the
    /// fmt_header call have return an Err, it will fail.
    fn fmt_header(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

// impl copied from std::any. Not especially nice, sorry :-(
impl<'a> AnyRefExt<'a> for &'a Header {
    #[inline]
    fn is<T: 'static>(self) -> bool {
        // Get TypeId of the type this function is instantiated with
        let t = TypeId::of::<T>();

        // Get TypeId of the type in the trait object
        let boxed = self.get_type_id();

        // Compare both TypeIds on equality
        t == boxed
    }

    #[inline]
    fn as_ref<T: 'static>(self) -> Option<&'a T> {
        if self.is::<T>() {
            Some(unsafe { self.as_ref_unchecked() })
        } else {
            None
        }
    }
}

/// An extension of `AnyRefExt` allowing unchecked downcasting of trait objects to `&T`.
trait UncheckedAnyRefExt<'a> {
    /// Returns a reference to the boxed value, assuming that it is of type `T`. This should only be
    /// called if you are ABSOLUTELY CERTAIN of `T` as you will get really wacky output if it’s not.
    unsafe fn as_ref_unchecked<T: 'static>(self) -> &'a T;
}

impl<'a> UncheckedAnyRefExt<'a> for &'a Header {
    #[inline]
    unsafe fn as_ref_unchecked<T: 'static>(self) -> &'a T {
        // Get the raw representation of the trait object
        let to: TraitObject = transmute_copy(&self);

        // Extract the data pointer
        transmute(to.data)
    }
}

/// An extension of `AnyMutRefExt` allowing unchecked downcasting of trait objects to `&mut T`.
trait UncheckedAnyMutRefExt<'a> {
    /// Returns a reference to the boxed value, assuming that it is of type `T`. This should only be
    /// called if you are ABSOLUTELY CERTAIN of `T` as you will get really wacky output if it’s not.
    unsafe fn as_mut_unchecked<T: 'static>(self) -> &'a mut T;
}

impl<'a> UncheckedAnyMutRefExt<'a> for &'a mut Header {
    #[inline]
    unsafe fn as_mut_unchecked<T: 'static>(self) -> &'a mut T {
        // Get the raw representation of the trait object
        let to: TraitObject = transmute_copy(&self);

        // Extract the data pointer
        transmute(to.data)
    }
}

/// A header marker, providing the glue between the header name and a type for that header.
///
/// Standard usage of this is very simple unit-struct marker types, like this:
///
/// ```rust
/// // The header data type
/// pub struct Foo {
///     ...
/// }
///
/// impl Header for Foo {
///     ...
/// }
///
/// // The marker type for accessing the header and specifying the name
/// pub struct FOO;
///
/// impl HeaderMarker<Foo> for FOO {
///     fn header_name(&self) -> SendStr {
///         Slice("foo")
///     }
/// }
/// ```
///
/// Then, accessing the header is done like this:
///
/// ```rust
/// let foo = request.headers.get(FOO).unwrap();
/// request.headers.set(FOO, foo);
/// ```
///
/// And lo! `foo` is a `Foo` object corresponding to the `foo` (or `Foo`, or `fOO`, &c.) header in
/// the request.
///
/// Authors are strongly advised that they should not implement `HeaderMarker<T>` for more than one
/// `T` on the same type; as well as going against the spirit of things, it would also require
/// explicit type specifiation every time, which would be a nuisance.
pub trait HeaderMarker<OutputType: Header + 'static> {
    /// The name of the header that shall be used for retreiving and setting.
    ///
    /// Normally this will be a static string, but occasionally it may be necessary to define it at
    /// runtime, for dynamic header handling.
    fn header_name(&self) -> SendStr;
}

impl Header for Box<Header> {
    fn parse_header(_raw: &[Vec<u8>]) -> Option<Box<Header>> {
        // Dummy impl; XXX: split to ToHeader/FromHeader?
        None
    }

    fn fmt_header(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_header(f)
    }
}

impl<'a> Header for &'a Header {
    fn parse_header(_raw: &[Vec<u8>]) -> Option<&'a Header> {
        // Dummy impl; XXX: split to ToHeader/FromHeader?
        None
    }

    fn fmt_header(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_header(f)
    }
}

/// All the header field values, raw or typed, with a shared field name.
enum Item {
    /// A raw, unparsed header. Each item in the outer vector is a header field value, the names of
    /// which were equivalent. Each inner vector is a string in the ISO-8859-1 character set, but
    /// could contain things in other character sets according to the rules of RFC 2047, e.g. in a
    /// *TEXT rule (RFC 2616 grammar).
    Raw(Vec<Vec<u8>>),

    /// A strongly typed header which has been parsed from the raw value.
    Typed(Box<Header>:'static),
}

/// A collection of HTTP headers.
pub struct Headers {
    data: HashMap<SendStr, Item>,
}

impl Headers {
    /// Construct a new header collection.
    pub fn new() -> Headers {
        Headers {
            data: HashMap::new(),
        }
    }
}

impl<H: Header + Clone + 'static, M: HeaderMarker<H>> Headers {
    fn mostly_get<'a>(&'a mut self, header_marker: &M) -> Option<&'a mut Item> {
        let name = header_marker.header_name();
        let item = match self.data.find_mut(&name) {
            // Yes, there's a header… or something… by that name
            Some(v) => v,
            // Header? There is no header!
            None => return None,
        };

        let (insert_parsed, parsed): (bool, Option<H>) = match *item {
            // We've parsed this header before, as some type or other.
            // Question is: is it the right type?

            // Yes, it's the right type, so we can immediately return it.
            // Well, we could, except that that makes the borrow checker keep the item borrow alive,
            // preventing the insert in the other cases. So we must refactor it to return at the
            // end instead.
            Typed(ref h) if h.is::<H>() => {
                (false, None)
            },

            // No, it was parsed as a different type.
            // Very well then, we will turn it back into a string first.
            Typed(ref h) => {
                let raw = fmt_header(h);
                (true, Header::parse_header([raw]))
            },

            // We haven't parsed it before, so let's have a go at that.
            Raw(ref raw) => (true, Header::parse_header(raw.as_slice())),
        };

        if insert_parsed {
            match parsed {
                // It parsed. Let's store the new value (replacing the raw one)
                Some(v) => *item = Typed(box v),
                // If the header doesn't parse, that's the same as it being absent.
                None => return None,
            }
        }
        Some(item)
    }

    /// Retrieve a header value. The value is a clone of the one that is stored internally.
    ///
    /// The interface is strongly typed; see TODO for a more detailed explanation of how it works.
    pub fn get(&mut self, header_marker: M) -> Option<H> {
        // At this point, we know that item is None, or Some(&mut Typed(h)) and that h.is::<H>().
        // On that basis, we can use as_ref_unchecked instead of as_ref, to save a virtual call.
        match self.mostly_get(&header_marker) {
            Some(&Typed(ref h)) => Some(unsafe { h.as_ref_unchecked::<H>() }.clone()),
            _ => None,
        }
    }

    /// Get a reference to a header value.
    ///
    /// Bear in mind that because of the internals, this method (and also `get` and `get_mut_ref`)
    /// takes `&mut self`; in consequence, you won't be able to take references to two headers at
    /// once. That is, in fact, why `get` is there---to provide a convenient way to avoid that
    /// problem, by immediately cloning the header value.
    ///
    /// The interface is strongly typed; see TODO for a more detailed explanation of how it works.
    pub fn get_ref<'a>(&'a mut self, header_marker: M) -> Option<&'a H> {
        match self.mostly_get(&header_marker) {
            Some(&Typed(ref h)) => Some(unsafe { h.as_ref_unchecked::<H>() }),
            _ => None,
        }
    }

    /// Get a mutable reference to a header value.
    ///
    /// The interface is strongly typed; see TODO for a more detailed explanation of how it works.
    pub fn get_mut_ref<'a>(&'a mut self, header_marker: M) -> Option<&'a mut H> {
        match self.mostly_get(&header_marker) {
            Some(&Typed(ref mut h)) => Some(unsafe { h.as_mut_unchecked::<H>() }),
            _ => None,
        }
    }

    /// Set the named header to the given value.
    pub fn set(&mut self, header_marker: M, value: H) {
        self.data.insert(header_marker.header_name(), Typed(box value));
    }

    /// Remove a header from the collection.
    /// Returns true if the named header was present.
    pub fn remove(&mut self, header_marker: &M) {
        self.data.remove(&header_marker.header_name());
    }
}

impl Headers {
    fn mostly_get_raw<'a>(&'a mut self, name: SendStr) -> Option<&'a mut Item> {
        let item = match self.data.find_mut(&name) {
            // Yes, there's a header… or something… by that name
            Some(v) => v,
            // Header? There is no header!
            None => return None,
        };

        let insert_raw = match *item {
            Typed(ref h) => Some(Raw(vec!(fmt_header(h)))),

            // We haven't parsed it before, so let's have a go at that.
            Raw(_) => None,
        };

        match insert_raw {
            Some(raw) => *item = raw,
            None => (),
        }

        Some(item)
    }

    /// Get the raw values of a header, by name.
    ///
    /// The returned value is a slice of each header field value.
    pub fn get_raw<'a, S: IntoMaybeOwned<'static>>(&'a mut self, name: S) -> Option<&'a [Vec<u8>]> {
        match self.mostly_get_raw(name.into_maybe_owned()) {
            Some(&Raw(ref raw)) => Some(raw.as_slice()),
            _ => None,
        }
    }

    /// Get a mutable reference to the raw values of a header, by name.
    ///
    /// The returned vector contains each header field value.
    pub fn get_raw_mut<'a, S: IntoMaybeOwned<'static>>(&'a mut self, name: S)
                                                      -> Option<&'a mut Vec<Vec<u8>>> {
        match self.mostly_get_raw(name.into_maybe_owned()) {
            Some(&Raw(ref mut raw)) => Some(raw),
            _ => None,
        }
    }
}

/// An adapter which provides `std::fmt::Show` as equivalent to `Header.fmt_header`, so that you can
/// actually *use* the thing.
struct HeaderShowAdapter<'a, H>(pub &'a H);

impl<'a, H: Header> fmt::Show for HeaderShowAdapter<'a, H> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let HeaderShowAdapter(h) = *self;
        h.fmt_header(f)
    }
}

#[inline]
/// Convert a typed header into the raw HTTP header field value.
pub fn fmt_header<H: Header>(h: &H) -> Vec<u8> {
    format_args!(format_but_not_utf8, "{}", HeaderShowAdapter(h))
}

// Parallel to ::std::fmt::{format, format_unsafe}, but returning Vec<u8> rather than Box<str>.
#[inline]
#[doc(hidden)]
pub fn format_but_not_utf8(args: &fmt::Arguments) -> Vec<u8> {
    let mut output = MemWriter::new();
    fmt::write(&mut output as &mut Writer, args).unwrap();
    output.unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expect<H: Header + std::fmt::Show + Eq>(h: Option<H>, h_expected: H, raw: &[u8]) {
        let h = h.unwrap();
        assert_eq!(fmt_header(&h).as_slice(), raw);
        assert_eq!(h, h_expected);
    }

    #[test]
    fn test_basics() {
        let mut headers = Headers::new();

        assert_eq!(headers.get(EXPIRES), None);

        headers.set(EXPIRES, Past);
        assert_eq!(headers.mostly_get(&EXPIRES), &mut Typed(Box<Past>));
        expect(headers.get(EXPIRES), Past, bytes!("0"));
        assert_eq!(headers.get_raw("expires"), vec!(vec!('0' as u8)));
        expect(headers.get(EXPIRES), Past, bytes!("0"));

        headers.remove(&EXPIRES);
        assert_eq!(headers.get(EXPIRES), None);

        assert_eq!(headers.get(DATE), None);
        let now = time::now();
        let now_raw = fmt_header(&now);
        headers.set(DATE, now.clone());
        expect(headers.get(DATE), now.clone(), now_raw.as_slice());
    }
}
