// vim: tw=80
//! Rust FFI bindings for FreeBSD's libnv library
//!
//! These are raw, `unsafe` FFI bindings.  Here be dragons!  You probably
//! shouldn't use this crate directly.  Instead, you should use the
//! [`libnv`](https://crates.io/crates/libnv) crate.
#![cfg_attr(crossdocs, doc = "")]
#![cfg_attr(crossdocs, doc = "These docs are just stubs!  Don't trust them.")]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[cfg(not(crossdocs))]
include!("ffi.rs");

#[cfg(crossdocs)] mod fakes;
#[cfg(crossdocs)] pub use fakes::*;
