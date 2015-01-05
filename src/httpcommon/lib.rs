//! Common HTTP functionality for the Teepee project.
//!
//! Anything shared between both client and server belongs in here, but this crate is not expected
//! to be used directly.
//!
//! Any crate using types from this crate should re‐export them. For example, the ``status`` module
//! should be exported in the root of the HTTP client crate ``httpc`` so that people can write
//! ``httpc::status`` instead of ``httpcommon::status``.

#![doc(html_logo_url = "http://teepee.rs/logo.100.png",
       html_root_url = "http://www.rust-ci.org/teepee/teepee/doc/")]

#![feature(globs, macro_rules, phase, slicing_syntax, unsafe_destructor, associated_types)]

#![warn(non_upper_case_globals, unused_typecasts, missing_docs, unused_results)]
//#![warn(unstable)]

#[phase(plugin)]
extern crate phf_mac;
extern crate phf;

#[phase(plugin, link)]
extern crate mucell;

pub mod method;
pub mod status;
pub mod headers;
pub mod grammar;
