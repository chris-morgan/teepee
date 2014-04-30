//! The Teepee HTTP server.

#![crate_id = "httpd#0.1-pre"]
#![comment = "The Teepee HTTP server"]
#![license = "MIT/ASL2"]
#![crate_type = "dylib"]
#![crate_type = "rlib"]

#![doc(html_root_url = "http://www.rust-ci.org/teepee/teepee/doc/")]

#![deny(unnecessary_qualification)]
#![deny(non_uppercase_statics)]
#![deny(unnecessary_typecast)]
#![deny(missing_doc)]
//#![deny(unstable)]
#![deny(unused_result)]
#![deny(deprecated_owned_vector)]

extern crate httpcommon;

pub use httpcommon::status;
