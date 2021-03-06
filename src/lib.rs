// #![deny(missing_docs)]

//! Rust bindings to Name/Value pairs libraries [libnv](man_page) and [nvpair](zol_nvpair)
//! It kinda acts like `Map<&str, T>` for poor people.
//! Library split into two modules: `libnv` and `nvpairs`. `libnv` is FreeBSD implementation that
//! isn't compatible with `nvpairs` that is Solaris implementation.
//!
//! [man_page]: https://www.freebsd.org/cgi/man.cgi?query=nv
//! [zol_nvpair]: https://github.com/zfsonlinux/zfs/tree/master/module/nvpair

extern crate libc;
#[macro_use] extern crate quick_error;

#[cfg(feature = "libnv")] extern crate libnv_sys;

#[cfg(feature = "nvpair")]
extern crate nvpair_sys;

#[cfg(feature = "libnv")] pub mod libnv;

#[cfg(feature = "nvpair")] pub mod nvpair;

use std::{ffi::NulError, io};
quick_error! {
    #[derive(Debug)]
    /// Error kinds for Name/Value library.
    pub enum NvError {
        /// Name a.k.a. key can't contain NULL byte. You going to get this error if you try so.
        InvalidString(err: NulError) {
            from()
        }
        /// error return by ffi. See libc for more information.
        NativeError(code: i32) {}
        /// Trying to set an error on n/v list that already has error
        AlreadySet {}
        /// No value found for given name.
        NotFound {}
        /// Library failed to allocate.
        OutOfMemory {}
        /// Other IO errors
        Io(err: io::Error) {}
        /// Operation not support on a list given flags used to create the list.
        OperationNotSupported {}
        /// Got non-utf8 string from the library.
        InvalidStringEncoding(err: std::str::Utf8Error) {
            from()
        }
    }
}
impl NvError {
    pub(crate) fn from_errno(errno: i32) -> Self {
        match errno {
            libc::ENOENT => NvError::NotFound,
            libc::ENOMEM => NvError::OutOfMemory,
            45 => NvError::OperationNotSupported,
            n => NvError::Io(io::Error::from_raw_os_error(n)),
        }
    }
}

/// Short-cut to Result<T, NvError>.
pub type NvResult<T> = Result<T, NvError>;
