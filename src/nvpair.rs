//! Solaris implementation of Name/Value pairs library.

use nvpair_sys as sys;

use crate::{NvError, NvResult};
use std::ffi::CString;
use std::mem::MaybeUninit;

#[repr(i32)]
#[derive(Copy, Clone, Debug)]
pub enum NvEncoding {
    Native = 0,
    Xdr = 1,
}
/// Options available for creation of an `nvlist`
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NvFlag {
    /// An existing pair of the same type will be removed prior inserting.
    UniqueName = 1,
    /// An existing pair of any type will be removed prior inserting.
    UniqueNameType = 2,
}

#[derive(Debug)]
pub struct NvList {
    ptr: *mut sys::nvlist_t
}

impl Drop for NvList {
    fn drop(&mut self) {
        unsafe {
            sys::nvlist_free(self.ptr)
        }
    }
}

/// Return new list with no flags.
impl Default for NvList {
    fn default() -> NvList { NvList::new(NvFlag::UniqueNameType).expect("Failed to create new list") }
}

impl NvList {
    /// Make a copy of a pointer. Danger zone.
    pub fn as_ptr(&self) -> *mut sys::nvlist_t { self.ptr }

    pub fn new(flags: NvFlag) -> NvResult<Self> {
        let mut raw_list = std::ptr::null_mut();
        let errno = unsafe {
            sys::nvlist_alloc(&mut raw_list, flags as u32, 0)
        };
        if errno != 0 {
            Err(NvError::NativeError(errno))
        } else {
            Ok(NvList { ptr: raw_list })
        }
    }
    pub fn is_empty(&self) -> bool {
        let ret = unsafe { sys::nvlist_empty(self.as_ptr() as *mut _) };
        ret != sys::boolean::B_FALSE
    }

    /// Add a `bool` to the list.
    pub fn insert_bool(&mut self, name: &str, value: bool) -> NvResult<()> {
        let c_name = CString::new(name)?;
        let v = {
            if value {
                sys::boolean::B_TRUE
            } else {
                sys::boolean::B_FALSE
            }
        };
        let errno = unsafe {
            sys::nvlist_add_boolean_value(self.ptr, c_name.as_ptr(), v)
        };
        if errno != 0 {
            Err(NvError::NativeError(errno))
        } else {
            Ok(())
        }
    }

    pub fn get_bool(&self, name: &str) -> NvResult<bool> {
        let c_name = CString::new(name)?;
        let mut ptr = MaybeUninit::<sys::boolean::Type>::zeroed();

        let errno = unsafe {
            sys::nvlist_lookup_boolean_value(self.ptr, c_name.as_ptr(), ptr.as_mut_ptr())
        };
        if errno != 0 {
            Err(NvError::NativeError(errno))
        } else {
            let ret = unsafe {
                ptr.assume_init()
            };
            Ok(ret != sys::boolean::B_FALSE)
        }

    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let mut list = NvList::default();
        assert!(list.is_empty());
        list.insert_bool("does_it_work", true);
        assert!(!list.is_empty());
        assert!(list.get_bool("does_it_work").unwrap());
    }
}