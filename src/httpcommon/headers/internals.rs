//! The internals of header representation. That is: `Item`.

use std::borrow::Cow;
use std::collections::hash_map;
use std::ops::Deref;
use std::any::Any;
use std::fmt;
use std::mem;
use std::slice;

use mucell::{MuCell, Ref};

use super::{ToHeader, Header, HeaderDisplayAdapter};

/// All the header field values, raw or typed, with a shared field name.
///
/// Each item can contain a raw and a typed representation in a combination defined below; for
/// safety, taking a mutable reference to either invalidates the other; immutable references do not
/// cause invalidation and so should be preferred where possible.
///
/// If a typed representation is invalidated, it is immediately dropped and replaced with `None`;
/// if a raw representation is invalidated, it is *not* dropped, but rather marked as invalid: this
/// is so that the outer vector can be reused, reducing allocation churn slightly. Trivial
/// performance improvement, though it also increases memory usage in the mean time. That is
/// expected to be a very small amount (<1KB across all headers) in most cases and acceptable.
///
/// The invariants about what this can contain are, unfortunately, quite complex.
/// Here is a list of the possible states, *not all of which represent a legal header*:
///
/// - `raw == None && typed == Single`:
///   there is a legal single-type header.
/// - `raw == None && typed == None`:
///   there is NOT a legal header here.
///   This occurs if you call `get_mut` into a single-type where parsing fails.
/// - `raw == None && typed == List with length 0`:
///   there is NOT a legal header here.
///   This occurs if you call `get_mut` into a list-type where parsing fails.
/// - `raw == None && typed == List with length > 0`:
///   there is a legal list-type header.
/// - `raw == Some(vec with length 1) && typed == Single`:
///   there is a legal single-type header.
/// - `raw == Some(vec with length > 0) && typed == List with length > 0`:
///   there is a legal list-type header.
/// - `raw == Some(vec with length > 0) && typed == None`:
///   there is a legal unknown-type header.
///
/// No other states may exist.
struct Inner {
    /// A raw, unparsed header. Each item in the outer vector is a header field value, the names of
    /// which were equivalent. Each inner vector is opaque data with no restrictions except that CR
    /// and LF may not appear, unless as part of an obs-fold rule (extremely much not recommended),
    /// though it is also recommended by RFC 7230 that it be US-ASCII.
    raw: Option<Vec<Vec<u8>>>,

    /// A strongly typed header which has been parsed from the raw value.
    typed: Typed,
}

/// The representation of a strongly typed header.
enum Typed {
    /// There is no header stored.
    // Yeah, we could have done `Option<InnerTyped>` and omitted this variant, but this optimises
    // better at present (reduces memory usage by one word).
    None,

    /// The header is of a type where there is one value, e.g. `Header: value`.
    ///
    /// This corresponds to any header not covered by the `List` variant.
    Single(Box<Header>),

    /// The header is of a type where there are multiple values, e.g. the equivalent forms
    /// `Header: val1, val2, val3`;
    /// `Header: val1`, `Header: val2`, `Header: val3`; and
    /// `Header: val1, val2`, `Header: val3`.
    ///
    /// This corresponds to the ABNF list extension `#rule` form described in RFC 7230 and used by
    /// quite a large number of headers.
    List(Box<ListHeader>),
}

impl fmt::Debug for Typed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Typed::None => f.write_str("None"),
            Typed::Single(ref h) => write!(f, "Single({})", HeaderDisplayAdapter(h)),
            Typed::List(ref h) => write!(f, "List({})", HeaderDisplayAdapter(h)),
        }
    }
}

#[doc(hidden)]
trait ListHeader: Header + ListHeaderClone {
    fn into_header_iter(self: Box<Self>) -> Box<Iterator<Item = Box<Header + 'static>> + 'static>;
    fn as_header_iter<'a>(&'a self) -> Box<Iterator<Item = &'a (Header + 'static)> + 'a>;
    fn is_empty(&self) -> bool;
}

mopafy!(ListHeader);

impl<H: ToHeader + Header + Clone> ListHeader for Vec<H> {
    fn into_header_iter(self: Box<Self>) -> Box<Iterator<Item = Box<Header>>> {
        #[inline] fn box_header<H: Header>(h: H) -> Box<Header> { Box::new(h) }
        Box::new(self.into_iter().map(box_header))
    }

    fn as_header_iter<'a>(&'a self) -> Box<Iterator<Item = &'a (Header + 'static)> + 'a> {
        #[inline] fn ref_header<H: Header>(h: &H) -> &(Header + 'static) { h }
        Box::new(self.iter().map(ref_header))
    }

    fn is_empty(&self) -> bool {
        (**self).is_empty()
    }
}

/// `Clone`, but producing boxed headers.
#[doc(hidden)]
pub trait ListHeaderClone {
    /// Clone self as a boxed header.
    #[inline]
    fn clone_boxed_list(&self) -> Box<ListHeader + 'static>;
}

impl<T: ListHeader + Clone + 'static> ListHeaderClone for T {
    fn clone_boxed_list(&self) -> Box<ListHeader + 'static> {
        Box::new(self.clone())
    }
}

impl Clone for Box<ListHeader + 'static> {
    fn clone(&self) -> Box<ListHeader + 'static> {
        self.clone_boxed_list()
    }
}

impl Header for Box<ListHeader> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

#[derive(PartialEq)]
pub struct Item {
    inner: MuCell<Inner>,
}

impl fmt::Debug for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner = self.inner.borrow();
        write!(f, "Item {{ raw: {:?}, typed: {:?} }}", inner.raw, inner.typed)
    }
}

impl PartialEq for Inner {
    fn eq(&self, other: &Inner) -> bool {
        self.raw_cow() == other.raw_cow()
    }
}

#[doc(hidden)]
trait MyIteratorExt: Iterator {
    fn into_single(self) -> Option<Self::Item>;
}

impl<T: Iterator> MyIteratorExt for T {
    fn into_single(mut self) -> Option<T::Item> {
        let output = match self.next() {
            Some(x) => x,
            None => return None,
        };
        match self.next() {
            Some(_) => return None,
            None => Some(output),
        }
    }
}

struct ValueListIter<'a> {
    current_line: Option<&'a [u8]>,
    lines: slice::Iter<'a, Vec<u8>>,
}

macro_rules! DEBUG { ($($x:tt)*) => (println!($($x)*)) }
macro_rules! DEBUG { ($($x:tt)*) => (()) }

impl<'a> Iterator for ValueListIter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<&'a [u8]> {
        'next: loop {
            DEBUG!("Getting a line…");
            if self.current_line.is_none() {
                self.current_line = self.lines.next().map(|v| &**v);
            }
            let mut line = match self.current_line {
                Some(line) => &line[..],
                None => return None,
            };
            DEBUG!("Working with line  {:?}", line);

            // Strip leading whitespace
            match line.iter().position(|&c| c != b' ' && c != b'\t') {
                Some(start) => line = &line[start..],
                // It’s all whitespace, better give up and move along to the next.
                None => {
                    self.current_line = None;
                    continue 'next;
                },
            }
            DEBUG!("Line stripped, now {:?}", line);

            enum State {
                Normal,
                QuotedString,
                QuotedPair,
            }

            let mut state = State::Normal;
            let mut output = None;
            let mut iter = line.iter().enumerate();
            loop {
                let (i, &byte) = match iter.next() {
                    None => match state {
                        State::Normal => {
                            self.current_line = None;
                            output = Some(line);
                            DEBUG!("EOL, output        {:?}", output);
                            break;
                        },
                        _ => {
                            DEBUG!("Ran out of bytes in a non-Normal state, giving up on line");
                            // No confidence.
                            self.current_line = None;
                            break;
                        },
                    },
                    Some(v) => v,
                };

                state = match state {
                    State::Normal => match byte {
                        b',' => {
                            // End of a value
                            output = Some(&line[..i]);
                            self.current_line = Some(&line[i + 1..]);
                            DEBUG!("EOV, output        {:?}", output);
                            DEBUG!("     current_line  {:?}", self.current_line);
                            break;
                        },
                        b'"' => State::QuotedString,
                        // field-vchar VCHAR / obs-text
                        b'\t' | b' ' | b'\x21'...b'\x7e' | b'\x80'...b'\xff' => State::Normal,
                        _ => {
                            DEBUG!("Illegal characters in Normal state, giving up on line");
                            // No confidence in any of the rest of the line.
                            self.current_line = None;
                            break;
                        },
                    },
                    State::QuotedPair => match byte {
                        // HTAB / SP / VCHAR / obs-text
                        b'\t' | b' ' | b'\x21'...b'\x7e' | b'\x80'...b'\xff' => State::QuotedString,
                        _ => {
                            DEBUG!("Illegal characters in QuotedPair state, giving up on line");
                            // No confidence in any of the rest of the line.
                            self.current_line = None;
                            break;
                        },
                    },
                    State::QuotedString => match byte {
                        b'"' => State::Normal,
                        b'\\' => State::QuotedPair,
                        b'\t' | b' ' | b'!' | b'\x23'...b'\x5b' | b'\x5d'...b'\x7e'
                        | b'\x80'...b'\xff' => State::QuotedString,
                        _ => {
                            DEBUG!("Illegal characters in QuotedString state, giving up on line");
                            // No confidence in any of the rest of the line.
                            self.current_line = None;
                            break;
                        },
                    },
                }
            }

            match output {
                Some(ref mut line) => {
                    DEBUG!("Maybe got something to return, {:?}", &line[..]);
                    // Strip trailing whitespace
                    match line.iter().rposition(|&c| c != b' ' && c != b'\t') {
                        Some(end) => {
                            DEBUG!("Happy! Returning {:?}", &line[..end + 1]);
                            return Some(&line[..end + 1]);
                        },
                        // This wasn’t a value, so let’s move along to the next.
                        None => {
                            DEBUG!("Value was purely whitespace, skipping it");
                            //self.current_line = None;
                            continue 'next;
                        },
                    }
                },
                None => (),
            }

            // RFC 7230:
            // qdtext = HTAB / SP / "!" / %x23-5B ; '#'-'['
            //  / %x5D-7E ; ']'-'~'
            //  / obs-text
            // query = <query, see [RFC3986], Section 3.4>
            // quoted-pair = "\" ( HTAB / SP / VCHAR / obs-text )
            // quoted-string = DQUOTE *( qdtext / quoted-pair ) DQUOTE
        }
    }
}

#[doc(hidden)]
trait RawHeaderExt {
    fn to_value_list_iter(&self) -> ValueListIter;
}

impl RawHeaderExt for [Vec<u8>] {
    fn to_value_list_iter(&self) -> ValueListIter {
        ValueListIter {
            current_line: None,
            lines: self.iter(),
        }
    }
}

macro_rules! value_list_iter_tests {
    ($($name:ident: $input:expr, $expected:expr;)*) => {
        #[cfg(test)]
        mod value_list_iter_tests {
            use super::RawHeaderExt;
            $(
                #[test]
                fn $name() {
                    let input: &[&[u8]] = &$input;
                    let input = input.iter().map(|x| x.to_vec()).collect::<Vec<_>>();
                    let expected: &[&[u8]] = &$expected;
                    let computed = input.to_value_list_iter().collect::<Vec<_>>();
                    assert_eq!(&computed[..], expected);
                }
            )*
        }
    }
}

value_list_iter_tests! {
    // This is quite a half-hearted effort, frankly.
    // It’s certainly not any sort of thorough test.
    simple_single:   [b"foo"],                   [b"foo"];
    normal:          [b"foo, bar, charlie"],     [b"foo", b"bar", b"charlie"];
    compact:         [b"foo,bar"],               [b"foo", b"bar"];
    trailing_comma:  [b"foo ,bar,"],             [b"foo", b"bar"];
    mixtures:        [b"foo , ,bar,charlie   "], [b"foo", b"bar", b"charlie"];
    empty:           [b""],                      [];
    comma_only:      [b","],                     [];
    blanks:          [b",   ,"],                 [];
    multiline:       [b"foo, bar", b"", b"baz"], [b"foo", b"bar", b"baz"];
    quoted_string_1: [b"foo,\"bar,baz\",x"],     [b"foo", b"\"bar,baz\"", b"x"];
    quoted_string_2: [b"foo, \"bar,baz\" ,x"],   [b"foo", b"\"bar,baz\"", b"x"];
    no_backslashy:   [b"foo, bar\\, baz"],       [b"foo", b"bar\\", b"baz"];
    bad_quotes:      [b"foo, \"bar, baz", b"x"], [b"foo", b"x"];
    // TODO: add more and more interesting cases.
}

impl Inner {
    fn raw_mut(&mut self, invalidate_others: bool) -> &mut Vec<Vec<u8>> {
        if self.raw.is_none() {
            self.raw = Some(if invalidate_others {
                match mem::replace(&mut self.typed, Typed::None) {
                    Typed::None => vec![],
                    Typed::Single(single) => vec![single.into_raw()],
                    Typed::List(list) => vec![list.to_raw()],
                }
            } else {
                match self.typed {
                    Typed::None => vec![],
                    Typed::Single(ref single) => vec![single.to_raw()],
                    Typed::List(ref list) => vec![list.to_raw()],
                }
            });
        }
        match self.raw {
            Some(ref mut out) => out,
            None => unreachable!(),
        }
    }

    // Moo!
    fn raw_cow(&self) -> Option<Cow<[Vec<u8>]>> {
        match self.raw {
            Some(ref vec) => Some(Cow::Borrowed(&vec[..])),
            None => match self.typed {
                Typed::None => None,
                Typed::Single(ref single) => Some(Cow::Owned(vec![single.to_raw()])),
                Typed::List(ref list) => Some(Cow::Owned(vec![list.to_raw()])),
            }
        }
    }

    fn single_typed_mut<H: ToHeader + Header>
                       (&mut self, invalidate_others: bool)
                       -> Option<&mut H> {
        let already_happy = match self.typed {
            Typed::Single(ref mut h) => h.is::<H>(),
            _ => false,
        };
        if !already_happy {
            // It doesn’t matter whether typed is None, Single or List, we’ll need to have it
            // in raw form first. Fortunately raw_mut can do this for us!
            let h: Option<H> = {
                let raw = match self.raw_mut(invalidate_others).iter().into_single() {
                    Some(raw) => raw,
                    None => return None,
                };
                ToHeader::parse(&raw[..])
            };
            self.typed = match h {
                Some(h) => Typed::Single(Box::new(h)),
                None => Typed::None,
            };
        }
        if invalidate_others {
            self.raw = None;
        }
        match self.typed {
            Typed::Single(ref mut h) => Some(unsafe { h.downcast_mut_unchecked() }),
            _ => None,
        }
    }

    fn list_typed_mut<H: ToHeader + Header + Clone>
                     (&mut self, invalidate_others: bool)
                     -> &mut Vec<H> {
        match self.typed {
            Typed::List(ref mut h) if h.is::<Vec<H>>() => {
                if invalidate_others {
                    self.raw = None;
                }
                unsafe { h.downcast_mut_unchecked() }
            },
            _ => {
                // It doesn’t matter whether typed is None, Single or List, we’ll need to have it
                // in raw form first. Fortunately raw_mut can do this for us!
                let h = self.raw_mut(invalidate_others)
                            .to_value_list_iter()
                            .filter_map(|value| ToHeader::parse(value))
                            .collect::<Vec<H>>();
                // The vector may be empty, but we do NOT change it to Typed::None.
                // It MUST end up a Typed::List.
                self.typed = Typed::List(Box::new(h));
                if invalidate_others {
                    self.raw = None;
                }
                match self.typed {
                    Typed::List(ref mut h) => unsafe { h.downcast_mut_unchecked() },
                    _ => unreachable!(),
                }
            },
        }
    }

    // Pass `false` to convert_if_necessary if `typed_mut` was called with the same `H`
    // immediately before; otherwise pass `true`.
    fn single_typed_cow<H: ToHeader + Header + Clone>
                       (&self, convert_if_necessary: bool)
                       -> Option<Cow<H>> {
        match self.typed {
            Typed::Single(ref h) if h.is::<H>() => {
                Some(unsafe { Cow::Borrowed(h.downcast_ref_unchecked()) })
            },
            _ if convert_if_necessary => {
                self.raw_cow().and_then(
                    |raw| raw.iter().into_single().and_then(
                        |raw| ToHeader::parse(&**raw).map(|x| Cow::Owned(x))))
            },
            _ => None,
        }
    }

    // Pass `false` to convert_if_necessary if `typed_mut` was called with the same `H`
    // immediately before; otherwise pass `true`.
    fn list_typed_cow<H: ToHeader + Header + Clone>
                     (&self, convert_if_necessary: bool)
                     -> Cow<[H]> {
        match self.typed {
            Typed::List(ref h) if h.is::<Vec<H>>() => {
                unsafe { Cow::Borrowed(&**h.downcast_ref_unchecked::<Vec<H>>()) }
            },
            _ if convert_if_necessary => {
                Cow::Owned(self.raw_cow().unwrap_or(Cow::Borrowed(&[]))
                                         .to_value_list_iter()
                                         .filter_map(|value| ToHeader::parse(value))
                                         .collect())
            },
            _ => Cow::Owned(vec![]),
        }
    }

}

/// An immutable reference to a `MuCell`. Dereference to get at the object.
pub struct RawRef<'a> {
    _parent: Ref<'a, Inner>,
    _data: Cow<'a, [Vec<u8>]>,
}

impl<'a> RawRef<'a> {
    /// Construct a reference from the cell.
    #[allow(trivial_casts)]  // The `as *const $ty` cast
    fn from(cell: &'a MuCell<Inner>) -> Option<RawRef<'a>> {
        let parent = cell.borrow();
        let x: &'a Inner = unsafe { &*(&*parent as *const Inner) };
        match x.raw_cow() {
            Some(data) => Some(RawRef {
                _parent: parent,
                _data: data,
            }),
            None => None,
        }
    }
}

impl<'a> Deref for RawRef<'a> {
    type Target = [Vec<u8>];
    fn deref(&self) -> &[Vec<u8>] {
        &*self._data
    }
}

impl<'a> RawRef<'a> {
    /// Extract the owned data.
    ///
    /// Copies the data if it is not already owned.
    pub fn into_owned(self) -> Vec<Vec<u8>> {
        self._data.into_owned()
    }
}

//mucell_ref_type! {
//    //#[doc = "TODO"]
//    struct TypedRef<'a, T: 'static>(Inner),
//    impl Deref -> T,
//    data: Cow<'a, T> = |x| x.typed_cow()
//}

/// An immutable reference to a `MuCell`. Dereference to get at the object.
//$(#[$attr])*
pub struct TypedRef<'a, H: ToHeader + Header + Clone> {
    _parent: Ref<'a, Inner>,
    _data: Cow<'a, H>,
}

impl<'a, H: ToHeader + Header + Clone> TypedRef<'a, H> {
    /// Construct a reference from the cell.
    #[allow(trivial_casts)]  // The `as *const $ty` cast
    fn from(cell: &'a MuCell<Inner>, convert_if_necessary: bool) -> Option<TypedRef<'a, H>> {
        let parent = cell.borrow();
        let inner: &'a Inner = unsafe { &*(&*parent as *const Inner) };
        match inner.single_typed_cow(convert_if_necessary) {
            Some(data) => Some(TypedRef {
                _parent: parent,
                _data: data,
            }),
            None => None,
        }
    }
}

impl<'a, H: ToHeader + Header + Clone> Deref for TypedRef<'a, H> {
    type Target = H;
    fn deref<'b>(&'b self) -> &'b H {
        &*self._data
    }
}

impl<'a, H: ToHeader + Header + Clone> TypedRef<'a, H> {
    /// Extract the owned data.
    ///
    /// Copies the data if it is not already owned.
    pub fn into_owned(self) -> H {
        self._data.into_owned()
    }
}

/// An immutable reference to a `MuCell`. Dereference to get at the object.
//$(#[$attr])*
pub struct TypedListRef<'a, H: ToHeader + Header + Clone> {
    _parent: Option<Ref<'a, Inner>>,
    _data: Cow<'a, [H]>,
}

impl<'a, H: ToHeader + Header + Clone> TypedListRef<'a, H> {
    /// Construct a reference from the cell.
    #[allow(trivial_casts)]  // The `as *const $ty` cast
    fn from(cell: &'a MuCell<Inner>, convert_if_necessary: bool) -> TypedListRef<'a, H> {
        let parent = cell.borrow();
        let inner: &'a Inner = unsafe { &*(&*parent as *const Inner) };
        TypedListRef {
            _parent: Some(parent),
            _data: inner.list_typed_cow(convert_if_necessary),
        }
    }

    fn empty() -> TypedListRef<'a, H> {
        TypedListRef {
            _parent: None,
            _data: Cow::Borrowed(&[]),
        }
    }
}

impl<'a, H: ToHeader + Header + Clone> Deref for TypedListRef<'a, H> {
    type Target = [H];
    fn deref<'b>(&'b self) -> &'b [H] {
        &*self._data
    }
}

impl<'a, H: ToHeader + Header + Clone> TypedListRef<'a, H> {
    /// Extract the owned data.
    ///
    /// Copies the data if it is not already owned.
    pub fn into_owned(self) -> Vec<H> {
        self._data.into_owned()
    }
}

/*************************************************************************************************/

impl Item {
    /// Construct a new Item from a raw representation.
    pub fn from_raw(raw: Vec<Vec<u8>>) -> Item {
        assert!(raw.len() > 0);
        Item {
            inner: MuCell::new(Inner {
                raw: Some(raw),
                typed: Typed::None,
            }),
        }
    }

    /// Construct a new Item from a single-typed representation.
    pub fn from_single_typed<H: ToHeader + Header + Clone>(typed: H) -> Item {
        Item {
            inner: MuCell::new(Inner {
                raw: None,
                typed: Typed::Single(Box::new(typed)),
            }),
        }
    }

    /// Construct a new Item from a list-typed representation.
    pub fn from_list_typed<H: ToHeader + Header + Clone>(typed: Vec<H>) -> Item {
        Item {
            inner: MuCell::new(Inner {
                raw: None,
                typed: Typed::List(Box::new(typed)),
            }),
        }
    }

    /// Returns true if the item contains at least one legal value
    /// that would be written in an HTTP message.
    ///
    /// An Item can be become invalid when `get_mut` is called and parsing fails. In such cases,
    /// `raw` will be `None` and `typed` will be either `None` or an empty `List`. (Because for
    /// list headers an empty value is no value.)
    pub fn is_valid(&self) -> bool {
        match *self.inner.borrow() {
            Inner { raw: Some(_), .. } => true,
            Inner { typed: Typed::Single(_), .. } => true,
            Inner { typed: Typed::List(ref list), .. } if !list.is_empty() => true,
            _ => false,
        }
    }

    /// Get a mutable reference to the raw representation of the header values.
    ///
    /// Because you may modify the raw representation through this mutable reference, calling this
    /// invalidates the typed representation; next time you want to access the value in typed
    /// fashion, it will be parsed from the raw form.
    ///
    /// Only use this if you need to mutate the raw form; if you don't, use `raw`.
    pub fn raw_mut(&mut self) -> &mut Vec<Vec<u8>> {
        self.inner.borrow_mut().raw_mut(true)
    }

    /// Get a reference to the raw representation of the header values.
    ///
    /// If a valid raw representation exists, it will be used, making this a very cheap operation;
    /// if it does not, then the typed representation will be converted to raw form and you will
    /// then get a reference to that. If there are no immutable references already taken, it will
    /// be stored just in case you do it again and you’ll get a reference, otherwise you’ll get the
    /// owned vector. But in summary, it doesn't much matter; you'll get an object that you can
    /// dereference to get your raw reference.
    ///
    /// See also `raw_mut`, if you wish to mutate the raw representation.
    pub fn raw(&self) -> Option<RawRef> {
        self.inner.try_mutate(|inner| { let _ = inner.raw_mut(false); });
        RawRef::from(&self.inner)
    }

    /// Set the raw form of the header.
    ///
    /// This invalidates the typed representation.
    pub fn set_raw(&mut self, raw: Vec<Vec<u8>>) {
        let inner = self.inner.borrow_mut();
        inner.raw = Some(raw);
        inner.typed = Typed::None;
    }

    /// Get a mutable reference to the single-typed representation of the header values.
    ///
    /// Because you may modify the typed representation through this mutable reference, calling
    /// this invalidates any other representation; most notably, if this method returns `None`,
    /// know that any data that was in `self` before has been irretrievably lost and you may be
    /// left with an `Item` devoid of data. Next time you want to access the value in any other
    /// fashion, it will be produced from this typed form.
    ///
    /// Only use this if you need to mutate the typed form; if you don't, use `single_typed`.
    pub fn single_typed_mut<H: ToHeader + Header>(&mut self) -> Option<&mut H> {
        self.inner.borrow_mut().single_typed_mut(true)
    }

    /// Get a mutable reference to the list-typed representation of the header values.
    ///
    /// Because you may modify the typed representation through this mutable reference, calling
    /// this invalidates any other representation; any values or lines which do not parse will be
    /// irretrievably lost. Next time you want to access the value in any other fashion, it will be
    /// produced from this typed form.
    ///
    /// Only use this if you need to mutate the typed form; if you don't, use `typed`.
    pub fn list_typed_mut<H: ToHeader + Header + Clone>(&mut self) -> &mut Vec<H> {
        self.inner.borrow_mut().list_typed_mut(true)
    }

    /// Get a reference to the single-typed representation of the header values.
    ///
    /// If a valid typed representation exists, it will be used, making this a very cheap
    /// operation; if it does not, then the raw representation will be converted to typed form and
    /// you will then get a reference to that. If there are no immutable references already taken,
    /// it will be stored just in case you do it again and you’ll get a reference, otherwise you’ll
    /// get the owned vector. In summary, it doesn't much matter; you'll get an object that you
    /// can dereference to get your typed reference.
    ///
    /// See also `single_typed_mut`, if you wish to mutate the single-typed representation.
    pub fn single_typed<H: ToHeader + Header + Clone>(&self) -> Option<TypedRef<H>> {
        let convert_if_necessary = self.inner.try_mutate(|inner| {
            let _ = inner.single_typed_mut::<H>(false);
        });
        TypedRef::from(&self.inner, convert_if_necessary)
    }

    /// Get a reference to the list-typed representation of the header values.
    ///
    /// If a valid typed representation exists, it will be used, making this a very cheap
    /// operation; if it does not, then the raw representation will be converted to typed form and
    /// you will then get a reference to that. If there are no immutable references already taken,
    /// it will be stored just in case you do it again and you’ll get a reference, otherwise you’ll
    /// get the owned vector. In summary, it doesn't much matter; you'll get an object that you
    /// can dereference to get your typed reference.
    ///
    /// See also `list_typed_mut`, if you wish to mutate the list-typed representation.
    pub fn list_typed<H: ToHeader + Header + Clone>(&self) -> TypedListRef<H> {
        let convert_if_necessary = self.inner.try_mutate(|inner| {
            let _ = inner.list_typed_mut::<H>(false);
        });
        TypedListRef::from(&self.inner, convert_if_necessary)
    }

    /// Set the typed form of the header as a single-type.
    ///
    /// This invalidates the raw representation.
    pub fn set_single_typed<H: ToHeader + Header + Clone>(&mut self, value: H) {
        let inner = self.inner.borrow_mut();
        inner.raw = None;
        inner.typed = Typed::Single(Box::new(value));
    }

    /// Set the typed form of the header as a list-type.
    ///
    /// This invalidates the raw representation.
    pub fn set_list_typed<H: ToHeader + Header + Clone>(&mut self, value: Vec<H>) {
        let inner = self.inner.borrow_mut();
        inner.raw = None;
        inner.typed = Typed::List(Box::new(value));
    }
}

#[doc(hidden)]
pub trait Get<'a> {
    fn get(item: Option<&'a Item>) -> Self;
}

impl<'a, T: ToHeader + Header + Clone> Get<'a> for Option<TypedRef<'a, T>> {
    fn get(item: Option<&'a Item>) -> Self {
        // TODO: consider shifting that method into here, if appropriate; ditto for all the rest
        item.and_then(|item| item.single_typed())
    }
}

impl<'a, T: ToHeader + Header + Clone> Get<'a> for TypedListRef<'a, T> {
    fn get(item: Option<&'a Item>) -> Self {
        match item {
            Some(item) => item.list_typed(),
            None => TypedListRef::empty(),
        }
    }
}

#[doc(hidden)]
pub trait GetMut<'a> {
    fn get_mut(entry: hash_map::Entry<'a, Cow<'static, str>, Item>) -> Self;
}

impl<'a, T: ToHeader + Header + Clone> GetMut<'a> for Option<&'a mut T> {
    fn get_mut(entry: hash_map::Entry<'a, Cow<'static, str>, Item>) -> Self {
        match entry.get() {
            Ok(item) => item.single_typed_mut(),
            Err(_vacant) => None,
        }
    }
}

impl<'a, T: ToHeader + Header + Clone> GetMut<'a> for &'a mut Vec<T> {
    fn get_mut(entry: hash_map::Entry<'a, Cow<'static, str>, Item>) -> Self {
        match entry.get() {
            Ok(item) => item,
            Err(vacant) => vacant.insert(Item::from_list_typed::<T>(vec![]))
        }.list_typed_mut()
    }
}

// -- START TODO UPDATING FOR FIRST CLASS LISTS --

#[cfg(skip)]
#[cfg(test)]
#[allow(unused_mut)]
mod tests {
    use super::{Item, Inner, Typed};
    use super::super::{ToHeader, Header};
    use std::fmt;
    use std::str;
    use mucell::MuCell;

    fn mkitem<H: Header>(raw: Option<Vec<Vec<u8>>>,
                         typed: Typed) -> Item {
        Item {
            inner: MuCell::new(Inner {
                raw: raw,
                typed: typed,
            })
        }
    }

    #[derive(PartialEq, Eq, Clone, Debug)]
    struct StrongType(Vec<u8>);
    #[allow(non_camel_case_types)]
    type st = StrongType;

    impl ToHeader for StrongType {
        fn parse(raw: &[u8]) -> Option<StrongType> {
            Some(StrongType(raw.to_vec()))
        }
    }

    impl Header for StrongType {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str(unsafe { str::from_utf8_unchecked(&self.0[..]) })
        }
    }

    #[derive(PartialEq, Eq, Clone, Debug)]
    struct NonParsingStrongType(StrongType);
    #[allow(non_camel_case_types)]
    type np = NonParsingStrongType;

    impl ToHeader for NonParsingStrongType {
        fn parse(_raw: &[u8]) -> Option<NonParsingStrongType> {
            None
        }
    }

    impl Header for NonParsingStrongType {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            self.0.fmt(f)
        }
    }

    fn assert_headers_eq<H: Header + Clone + PartialEq + fmt::Debug>(item: &Item, other: &Item) {
        let item = item.inner.borrow();
        let other = other.inner.borrow();
        assert_eq!(item.raw, other.raw);
        match (&item.typed, &other.typed) {
            (&Typed::None, &Typed::None) => (),
            (&Typed::Single(ref a), &Typed::Single(ref b)) => {
                assert_eq!(a.downcast_ref::<H>(), b.downcast_ref::<H>());
            },
            (&Typed::List(ref a), &Typed::List(ref b)) => {
                assert_eq!(a.downcast_ref::<Vec<H>>(), b.downcast_ref::<Vec<H>>());
            },
            _ => panic!("Inner.typed did not match between two items"),
        }
    }

    // Dummy 1: multiple headers
    fn d1raw() -> Vec<Vec<u8>> {
        vec![b"ab".to_vec(), b"cd".to_vec()]
        //vec![vec![b'a', b'b'], vec![b'c', b'd']]
    }
    fn d1st() -> StrongType { StrongType(d1raw()) }
    fn d1np() -> NonParsingStrongType { NonParsingStrongType(d1st()) }

    // Dummy 2: 1, but merged
    fn d2raw() -> Vec<Vec<u8>> {
        vec![b"ab, cd".to_vec()]
        //vec![vec![b'a', b'b', b',', b' ', b'c', b'd']]
    }
    //fn d2st() -> StrongType { StrongType(d2raw()) }
    //fn d2np() -> NonParsingStrongType { NonParsingStrongType(d2st()) }

    // Dummy 3: multiple headers, different from 1
    fn d3raw() -> Vec<Vec<u8>> {
        vec![b"12".to_vec(), b"34".to_vec()]
        //vec![vec![b'1', b'2'], vec![b'3', b'4']]
    }
    fn d3st() -> StrongType { StrongType(d3raw()) }
    fn d3np() -> NonParsingStrongType { NonParsingStrongType(d3st()) }

    // Dummy 4: 3, but merged
    fn d4raw() -> Vec<Vec<u8>> {
        vec![b"12, 34".to_vec()]
        //vec![vec![b'1', b'2', b',', b' ', b'3', b'4']]
    }
    fn d4st() -> StrongType { StrongType(d4raw()) }
    //fn d4np() -> NonParsingStrongType { NonParsingStrongType(d4st()) }

    #[test]
    #[should_panic]
    fn test_from_raw_with_empty_vector() {
        // Would not satisfy invariants, explicitly fails.
        let _raw = Item::from_raw(vec![]);
    }

    #[test]
    fn test_fresh_item_from_raw() {
        let item = Item::from_raw(vec![vec![]]);
        assert_headers_eq::<StrongType>(&item,
                                        &mkitem(Some(vec![vec![]]), Typed::None));
    }

    #[test]
    fn test_fresh_item_from_single_typed() {
        let item = Item::from_single_typed(d1st());
        assert_headers_eq::<StrongType>(&item, &mkitem(None, Typed::Single(Box::new(d1st()))));
    }

    macro_rules! _raw {
        (1) => (Some(d1raw()));
        (2) => (Some(d1raw()));
        (3) => (Some(d1raw()));
        (4) => (Some(d1raw()));
        (-) => (None);
    }

    macro_rules! _typed {
        (1 s $x:ident) => (Typed::Single(Box::new(concat_idents!(d1_s_, $x)())));
        (2 s $x:ident) => (Typed::Single(Box::new(concat_idents!(d2_s_, $x)())));
        (3 s $x:ident) => (Typed::Single(Box::new(concat_idents!(d3_s_, $x)())));
        (4 s $x:ident) => (Typed::Single(Box::new(concat_idents!(d4_s_, $x)())));
        (1 l $x:ident) => (Typed::List(Box::new(concat_idents!(d1_l_, $x)())));
        (2 l $x:ident) => (Typed::List(Box::new(concat_idents!(d2_l_, $x)())));
        (3 l $x:ident) => (Typed::List(Box::new(concat_idents!(d3_l_, $x)())));
        (4 l $x:ident) => (Typed::List(Box::new(concat_idents!(d4_l_, $x)())));
        (- $x:tt) => (Typed::None);
    }
    macro_rules! _bool((t)=>(true);(f)=>(false));
    macro_rules! _item {
        ($raw:tt, $ty_n:tt $ty_m:tt $ty_ty:tt) => {
            mkitem(_raw!($raw), _typed!($ty_n $ty_m $ty_ty))
        }
    }
    macro_rules! t {
        (
            $fn_name:ident =>
            ($s1:tt, $s2a:tt $s2b:ident)
            $method:ident ( $(,$args:expr)* )
            ($e1:tt, $e2a:tt $e2b:ident)
        ) => {
            #[test]
            fn $fn_name() {
                let mut item = _item!($s1, $s2a $s2b);
                let _ = item.$method($($args),*);
                assert_headers_eq::<StrongType>(&item, &_item!($e1, $e2a $e2b));
            }
        };
        (
            $fn_name:ident =>
            ($s1:tt, $s2a:tt $s2b:ident)
            $method:ident ( $(,$args:expr)* ) / $T:ident
            ($e1:tt, $e2a:tt $e2b:ident)
        ) => {
            #[test]
            fn $fn_name() {
                let mut item = _item!($s1, $s2a $s2b);
                let _ = item.$method::<$T>($($args),*);
                assert_headers_eq::<StrongType>(&item, &_item!($e1, $e2a $e2b));
                assert!(item == _item!($e1, $e2a $e2b));
            }
        }
    }

    // Now we get down to what is essentially just a big table of tests, verifying every class of
    // possible behaviour and ensuring that the output is sound.
    //
    // I could try explaining it all in detail, but having considered the matter, I've decided that
    // it's straightforward enough that explanation might be a bit of a waste. I'm sorry if you
    // disagree with me—I can see that it is a bit of a tangle. Stick at it and you should be able
    // to understand it. I'm sorry I caused you trouble.

    t!(set_raw_with_raw         => (1, - s st) set_raw(,d3raw()) (3, - s st));
    t!(set_raw_with_typed       => (-, 1 s st) set_raw(,d3raw()) (3, - s st));
    t!(set_raw_with_both        => (1, 3 s st) set_raw(,d3raw()) (3, - s st));

    t!(raw_mut_with_raw         => (1, - s st) raw_mut()     (1, - s st));
    t!(raw_mut_with_typed       => (-, 1 s st) raw_mut()     (2, - s st));
    t!(raw_mut_with_both        => (1, 3 s st) raw_mut()     (1, - s st));

    t!(raw_with_raw             => (1, - s st) raw()         (1, - s st));
    t!(raw_with_typed           => (-, 1 s st) raw()         (2, 1 s st));
    t!(raw_with_both            => (1, 3 s st) raw()         (1, 3 s st));

    t!(set_single_typed_with_raw                  => (1, - s st) set_single_typed(,d3st()) (-, 3 s st));
    t!(set_single_typed_with_typed                => (-, 1 s st) set_single_typed(,d3st()) (-, 3 s st));
    t!(set_single_typed_with_other                => (-, 1 s np) set_single_typed(,d3st()) (-, 3 s st));
    t!(set_single_typed_with_other_and_raw        => (1, 1 s np) set_single_typed(,d3st()) (-, 3 s st));

    t!(single_typed_mut_with_raw                  => (1, - s st) single_typed_mut()/st (-, 1 s st));
    t!(single_typed_mut_with_typed                => (-, 1 s st) single_typed_mut()/st (-, 1 s st));
    t!(single_typed_mut_with_other                => (-, 3 s np) single_typed_mut()/st (-, 4 s st));
    t!(single_typed_mut_with_other_and_raw        => (1, 3 s np) single_typed_mut()/st (-, 1 s st));
    t!(single_typed_mut_with_other_np             => (-, 3 s st) single_typed_mut()/np (-, 3 s st));
    t!(single_typed_mut_with_other_and_raw_np     => (1, 3 s st) single_typed_mut()/np (-, 3 s st));

    t!(single_typed_with_raw                      => (1, - s st) single_typed()/st     (1, 1 s st));
    t!(single_typed_with_typed                    => (-, 1 s st) single_typed()/st     (-, 1 s st));
    t!(single_typed_with_other                    => (-, 3 s np) single_typed()/st     (4, 4 s st));
    t!(single_typed_with_other_and_raw            => (1, 3 s np) single_typed()/st     (1, 1 s st));
    t!(single_typed_with_other_np                 => (-, 3 s st) single_typed()/np     (4, 3 s st));
    t!(single_typed_with_other_and_raw_np         => (1, 3 s st) single_typed()/np     (1, 3 s st));

    macro_rules! eq {
        (
            $fn_name:ident =>
            ($s1:tt, $s2a:tt $s2b:ident)
            ($e1:tt, $e2a:tt $e2b:ident)
        ) => {
            #[test]
            fn $fn_name() {
                //assert_eq!(_item!($s1, $s2a $s2b), _item!($e1, $e2a $e2b));
                let a = _item!($s1, $s2a $s2b);
                let b = _item!($e1, $e2a $e2b);
                assert!(a == b, "The two are not equal!\n{:?}\n{:?}", a, b)
            }
        }
    }

    macro_rules! ne {
        (
            $fn_name:ident =>
            ($s1:tt, $s2a:tt $s2b:ident)
            ($e1:tt, $e2a:tt $e2b:ident)
        ) => {
            #[test]
            fn $fn_name() {
                let a = _item!($s1, $s2a $s2b);
                let b = _item!($e1, $e2a $e2b);
                assert!(a != b, "The two are equal!\n{:?}\n{:?}", a, b)
            }
        }
    }

    // raw = raw
    eq!(raw_eq_raw_with_different_typed => (2, 1 st) (2, 1 np));
    eq!(raw_eq_raw_with_same_typed      => (2, 1 st) (2, 1 st));
    eq!(raw_eq_raw_with_one_typed       => (2, 1 st) (2, - st));
    eq!(raw_eq_raw_with_neither_typed   => (2, - st) (2, - st));
    ne!(raw_ne_raw_with_different_typed => (2, 1 st) (4, 3 np));
    ne!(raw_ne_raw_with_same_typed      => (2, 1 st) (4, 3 st));
    ne!(raw_ne_raw_with_one_typed       => (2, 1 st) (4, - st));
    ne!(raw_ne_raw_with_neither_typed   => (2, - st) (4, - st));

    // raw = typed
    eq!(raw_eq_typed_with_one_raw         => (2, 3 st) (-, 1 st));
    ne!(raw_ne_typed_with_one_raw         => (1, 1 st) (-, 1 st));
    eq!(raw_eq_typed_with_different_typed => (2, 1 st) (-, 1 np));

    // typed = typed
    eq!(typed_eq_typed_with_1              => (-, 1 st) (-, 1 np));
    eq!(typed_eq_typed_with_2              => (-, 1 st) (-, 1 st));
}
