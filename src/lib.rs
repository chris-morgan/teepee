//! The Teepee project’s HTTP library.

#![doc(html_logo_url = "http://teepee.rs/logo.100.png",
       html_root_url = "http://www.rust-ci.org/teepee/teepee/doc/")]

#![feature(concat_idents, plugin)]

#![warn(non_upper_case_globals, missing_docs, unused_results)]

#![plugin(phf_macros)]

extern crate phf;

#[macro_use]
extern crate mucell;

#[macro_use]
extern crate mopa;

extern crate tendril;
extern crate smallvec;

pub mod method;
pub mod status;
pub mod headers;
pub mod grammar;
