//! The internals of header representation. That is: `Item`.

use std::any::AnyRefExt;

use super::{Header, UncheckedAnyMutRefExt, fmt_header};

/// All the header field values, raw or typed, with a shared field name.
///
/// Invariants beyond those enforced by the type system:
///
/// - `raw == None` requires `!raw_valid`.
/// - `raw == None && typed == None` is not legal.
/// - `raw_valid == true` requires `raw` to be some with `.len() > 0`.
pub struct Item {
    /// Whether the raw header form is valid. If a mutable reference is taken to `typed`, this will
    /// be set to `true`, meaning that for the purposes of reading, `raw` must be considered to be
    /// invalid and must be produced once again. This exists as a slight efficiency improvement over
    /// just resetting `raw` to `None` in that, if the raw form is read again, the raw vectors can
    /// be reused; this is almost certain to be faster than dropping the vectors and creating new
    /// ones.
    raw_valid: bool,

    /// A raw, unparsed header. Each item in the outer vector is a header field value, the names of
    /// which were equivalent. Each inner vector is a string in the ISO-8859-1 character set, but
    /// could contain things in other character sets according to the rules of RFC 2047, e.g. in a
    /// *TEXT rule (RFC 2616 grammar).
    ///
    /// Reading from this is only valid if `raw_valid` is `true`.
    raw: Option<Vec<Vec<u8>>>,

    /// A strongly typed header which has been parsed from the raw value.
    typed: Option<Box<Header>:'static>,
}

impl Item {
    /// Construct a new Item from a raw representation.
    ///
    /// The vector given MUST contain at least one value, or this will fail.
    pub fn from_raw(raw: Vec<Vec<u8>>) -> Item {
        assert!(raw.len() > 0);
        Item {
            raw_valid: true,
            raw: Some(raw),
            typed: None,
        }
    }

    /// Construct a new Item from a typed representation.
    pub fn from_typed<H: Header + 'static>(typed: H) -> Item {
        Item {
            raw_valid: false,
            raw: None,
            typed: Some(box typed as Box<Header>),
        }
    }

    fn raw_mut_ref_internal<'a>(&'a mut self, invalidate_typed: bool) -> &'a mut Vec<Vec<u8>> {
        if self.raw_valid == true {
            // All is good; we'll return the value in good time.
        } else {
            self.raw_valid = true;
            match (&mut self.raw, &mut self.typed) {
                (&Some(ref mut raw), &Some(ref typed)) => {
                    raw.truncate(1);
                    raw.as_mut_slice()[0] = fmt_header(typed);
                },

                (ref mut raw @ &None, &Some(ref typed)) => {
                    **raw = Some(vec![fmt_header(typed)]);
                },

                _ => unreachable!(),
            }
        }
        if invalidate_typed {
            self.typed = None;
        }
        self.raw.get_mut_ref()
    }

    /// Get a mutable reference to the raw representation of the header values.
    ///
    /// Because you may modify the raw representation through this mutable reference, calling this
    /// this invalidates the typed representation; next time you want to access the value in typed
    /// fashion, it will be parsed from the raw form.
    ///
    /// Only use this if you need to mutate the raw form; if you don't, use `raw_ref`.
    pub fn raw_mut_ref<'a>(&'a mut self) -> &'a mut Vec<Vec<u8>> {
        self.raw_mut_ref_internal(true)
    }

    /// Get a reference to the raw representation of the header values.
    ///
    /// If a valid raw representation exists, it will be used, making this a very cheap operation;
    /// if it does not, then the typed representation will be converted to raw form and you will
    /// then get a reference to that. In summary, it doesn't much matter; you'll get your raw
    /// reference.
    ///
    /// See also `raw_mut_ref`, if you wish to mutate the raw representation.
    pub fn raw_ref<'a>(&'a mut self) -> &'a Vec<Vec<u8>> {
        &*self.raw_mut_ref_internal(false)
    }

    /// Set the raw form of the header.
    ///
    /// This invalidates the typed representation.
    pub fn set_raw(&mut self, raw: Vec<Vec<u8>>) {
        self.raw_valid = true;
        self.raw = Some(raw);
        self.typed = None;
    }

    fn typed_mut_ref_internal<'a, H: Header + 'static>
                             (&'a mut self, invalidate_raw: bool) -> Option<&'a mut H> {
        match self.typed {
            None => {
                debug_assert_eq!(self.raw_valid, true);
                debug_assert!(self.raw.is_some());
                if invalidate_raw {
                    self.raw_valid = false;
                }
                let h: Option<H> = Header::parse_header(self.raw.get_ref().as_slice());
                match h {
                    Some(h) => {
                        self.typed = Some(box h as Box<Header>);
                        Some(unsafe { self.typed.get_mut_ref().as_mut_unchecked::<H>() })
                    },
                    None => None,
                }
            },
            Some(ref mut h) if h.is::<H>() => {
                if invalidate_raw {
                    self.raw_valid = false;
                }
                Some(unsafe { h.as_mut_unchecked::<H>() })
            },
            Some(ref mut h) => {
                if !self.raw_valid {
                    match self.raw {
                        Some(ref mut raw) => {
                            raw.truncate(1);
                            raw.as_mut_slice()[0] = fmt_header(h);
                        },
                        None => {
                            self.raw = Some(vec![fmt_header(h)]);
                        },
                    }
                    self.raw_valid = !invalidate_raw;
                } else if invalidate_raw {
                    self.raw_valid = false;
                }
                let otyped: Option<H> = Header::parse_header(self.raw.get_ref().as_slice());
                match otyped {
                    Some(typed) => {
                        *h = box typed as Box<Header>;
                        Some(unsafe { h.as_mut_unchecked::<H>() })
                    },
                    None => None,
                }
            },
        }
    }

    /// Get a mutable reference to the raw representation of the header values.
    ///
    /// Because you may modify the raw representation through this mutable reference, calling this
    /// this invalidates the typed representation; next time you want to access the value in typed
    /// fashion, it will be parsed from the raw form.
    ///
    /// Only use this if you need to mutate the raw form; if you don't, use `raw_ref`.
    pub fn typed_mut_ref<'a, H: Header + 'static>(&'a mut self) -> Option<&'a mut H> {
        self.typed_mut_ref_internal(true)
    }

    /// Get a reference to the raw representation of the header values.
    ///
    /// If a valid raw representation exists, it will be used, making this a very cheap operation;
    /// if it does not, then the typed representation will be converted to raw form and you will
    /// then get a reference to that. In summary, it doesn't much matter; you'll get your raw
    /// reference.
    ///
    /// See also `raw_mut_ref`, if you wish to mutate the raw representation.
    pub fn typed_ref<'a, H: Header + 'static>(&'a mut self) -> Option<&'a H> {
        self.typed_mut_ref_internal(false).map(|h| &*h)
    }

    /// Set the raw form of the header.
    ///
    /// This invalidates the typed representation.
    pub fn set_typed<'a, H: Header + 'static>(&mut self, value: H) {
        self.raw_valid = false;
        self.typed = Some(box value as Box<Header>);
    }

    #[cfg(test)]
    fn assert_invariants(&self) {
        assert!(self.raw.is_some() || !self.raw_valid);
        assert!(self.raw.is_some() || self.typed.is_some());
        assert!(!self.raw_valid || self.raw.get_ref().len() > 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Header;
    use std::fmt;
    use std::any::AnyRefExt;

    fn mkitem<H: Header + 'static>(raw_valid: bool,
                                   raw: Option<Vec<Vec<u8>>>,
                                   typed: Option<H>) -> Item {
        Item {
            raw_valid: raw_valid,
            raw: raw,
            typed: typed.map(|h| box h as Box<Header>),
        }
    }

    #[deriving(Eq, Clone, Show)]
    struct StrongType(Vec<Vec<u8>>);

    impl Header for StrongType {
        fn parse_header(raw: &[Vec<u8>]) -> Option<StrongType> {
            Some(StrongType(raw.iter().map(|x| x.clone()).collect()))
        }

        fn fmt_header(&self, w: &mut Writer) -> fmt::Result {
            let StrongType(ref vec) = *self;
            let mut first = true;
            for field in vec.iter() {
                try!(w.write(field.as_slice()));
                if !first {
                    try!(w.write([',' as u8, ' ' as u8]));
                }
                first = false;
            }
            Ok(())
        }
    }

    #[deriving(Eq, Clone, Show)]
    struct NonParsingStrongType(StrongType);

    impl Header for NonParsingStrongType {
        fn parse_header(_raw: &[Vec<u8>]) -> Option<NonParsingStrongType> {
            None
        }

        fn fmt_header(&self, w: &mut Writer) -> fmt::Result {
            let NonParsingStrongType(ref st) = *self;
            st.fmt_header(w)
        }
    }

    fn assert_headers_eq<H: Header + Clone + Eq + fmt::Show + 'static>(item: &Item, other: &Item) {
        item.assert_invariants();
        assert_eq!(item.raw_valid, other.raw_valid);
        assert_eq!(item.raw, other.raw);
        if item.typed.is_some() || other.typed.is_some() {
            let it = item.typed.get_ref();
            let ot = other.typed.get_ref();
            let ir = it.as_ref::<H>().unwrap();
            let or = ot.as_ref::<H>().unwrap();
            assert_eq!(ir, or);
        }
    }

    fn dummy_raw() -> Vec<Vec<u8>> {
        vec![vec!['a' as u8, 'b' as u8], vec!['c' as u8, 'd' as u8]]
    }

    fn dummy_st() -> StrongType {
        StrongType(dummy_raw())
    }

    fn dummy_npst() -> NonParsingStrongType {
        NonParsingStrongType(dummy_st())
    }

    #[test]
    #[should_fail]
    fn test_from_raw_with_empty_vector() {
        // Would not satisfy invariants, explicitly fails.
        let _raw = Item::from_raw(vec![]);
    }

    #[test]
    fn test_fresh_item_from_raw() {
        let item = Item::from_raw(vec![vec![]]);
        assert_headers_eq::<StrongType>(&item,
                                        &mkitem(true, Some(vec![vec![]]), None::<StrongType>));
    }

    #[test]
    fn test_fresh_item_from_typed() {
        let item = Item::from_typed(dummy_st());
        assert_headers_eq::<StrongType>(&item, &mkitem(false, None, Some(dummy_st())));
    }

    #[test]
    fn test_get_raw() {
    }
}
