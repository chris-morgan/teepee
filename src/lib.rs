//! The Teepee project’s HTTP library.

#![doc(html_logo_url = "http://teepee.rs/logo.100.png",
       html_root_url = "http://www.rust-ci.org/teepee/teepee/doc/")]

#![feature(concat_idents, plugin, const_fn, associated_consts)]
#![cfg_attr(feature = "nonzero", feature(nonzero))]

#![cfg_attr(test, feature(test))]

#![warn(non_upper_case_globals, missing_docs, unused_results)]

//#![allow(missing_docs, dead_code, unused_variables, unused_results, unused_assignments)]  // For while developing

#![plugin(phf_macros)]

#[cfg(feature = "nonzero")]
extern crate core;

#[cfg(test)]
extern crate test;

extern crate phf;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate mucell;

#[macro_use]
extern crate mopa;

#[macro_use]
extern crate lazy_static;

extern crate tendril;
extern crate smallvec;

pub mod method;
pub mod status;
pub mod headers;
pub mod grammar;

pub mod http2;

/// I don’t care about non-atomic byte tendrils, so let’s just call it ByteTendril.
pub type ByteTendril = tendril::Tendril<tendril::fmt::Bytes, tendril::Atomic>;

trait TendrilSliceExt {
    fn to_tendril(&self) -> ByteTendril;
}

impl TendrilSliceExt for [u8] {
    #[inline]
    fn to_tendril(&self) -> ByteTendril {
        ByteTendril::from_slice(self)
    }
}
