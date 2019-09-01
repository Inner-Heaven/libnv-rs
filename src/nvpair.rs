//! Solaris implementation of Name/Value pairs library.

use nvpair_sys as sys;

use crate::{NvError, NvResult};
use std::{ffi::CStr, ffi::CString, mem::MaybeUninit};
use ::std::convert::TryInto;

/// This allows usage of insert method with basic types. Implement this for your
/// own types if you don't want to convert to primitive types every time.
pub trait NvTypeOp {
    /// Add self to given list.
    fn add_to_list(&self, list: &mut NvList, name: &str) -> NvResult<()>;
}

#[repr(i32)]
#[derive(Copy, Clone, Debug)]
pub enum NvEncoding {
    /// A basic copy on insert operation.
    Native = 0,
    /// [XDR](https://tools.ietf.org/html/rfc4506) copy suitable for sending to remote host.
    Xdr    = 1,
}
/// Options available for creation of an `nvlist`
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NvFlag {
    /// No flags. Allows duplicate names of any type, but renders `get_` methods useless.
    None           = 0b000,
    /// An existing pair of the same type will be removed prior inserting.
    UniqueName     = 0b001,
    /// An existing pair of any type will be removed prior inserting.
    UniqueNameType = 0b010,
}

#[derive(Debug)]
pub struct NvList {
    ptr: *mut sys::nvlist_t,
}

impl Drop for NvList {
    fn drop(&mut self) { unsafe { sys::nvlist_free(self.ptr) } }
}

/// Return new list with no flags.
impl Default for NvList {
    fn default() -> NvList {
        NvList::new(NvFlag::UniqueNameType).expect("Failed to create new list")
    }
}
macro_rules! impl_list_op {
    ($type_:ty, $method:ident, false) => {
        impl NvTypeOp for $type_ {
            /// Add a `$type_` value to the `NvList`
            fn add_to_list(&self, list: &mut NvList, name: &str) -> NvResult<()> {
                return list.$method(name, *self);
            }
        }
    };
    ($type_:ty, $method:ident, true) => {
        impl NvTypeOp for $type_ {
            /// Add a `$type_` value to the `NvList`
            fn add_to_list(&self, list: &mut NvList, name: &str) -> NvResult<()> {
                return list.$method(name, &*self);
            }
        }
    };
}

macro_rules! nvpair_type_array_method {
    ($type_:ty, $rmethod_insert:ident, $smethod_insert:ident, $rmethod_get:ident, $smethod_get:ident) => {
        /// Add `&[$type_]` value to the list.
        pub fn $rmethod_insert(&mut self, name: &str, value: &mut [$type_]) -> NvResult<()> {
            let c_name = CString::new(name)?;
            let errno = unsafe { sys::$smethod_insert(self.ptr, c_name.as_ptr(), value.as_mut_ptr(), value.len() as u32) };
            if errno != 0 {
                Err(NvError::from_errno(errno))
            } else {
                Ok(())
            }
        }

                /// Get a `$type_` value by given name from the list.
        pub fn $rmethod_get<'a>(&'a self, name: &str) -> NvResult<&'a [$type_]> {
            let c_name = CString::new(name)?;
            let mut ptr = MaybeUninit::<*mut _>::zeroed();
            let mut len = 0;
            let errno = unsafe {
                sys::$smethod_get(self.ptr, c_name.as_ptr(), ptr.as_mut_ptr(), &mut len)
            };
            if errno != 0 {
                Err(NvError::from_errno(errno))
            } else {
                let ret = unsafe {
                    ptr.assume_init();
                    dbg!(len);
                    std::slice::from_raw_parts(*ptr.as_mut_ptr(), len.try_into().unwrap())
                 };
                Ok(ret)
            }
        }
    }
}
macro_rules! nvpair_type_method {
    ($type_:ty, $rmethod_insert:ident, $smethod_insert:ident, $rmethod_get:ident, $smethod_get:ident) => {
        /// Add `$type_` value to the list.
        pub fn $rmethod_insert(&mut self, name: &str, value: $type_) -> NvResult<()> {
            let c_name = CString::new(name)?;
            let errno = unsafe { sys::$smethod_insert(self.ptr, c_name.as_ptr(), value) };
            if errno != 0 {
                Err(NvError::from_errno(errno))
            } else {
                Ok(())
            }
        }

        /// Get a `$type_` value by given name from the list.
        pub fn $rmethod_get(&self, name: &str) -> NvResult<$type_> {
            let c_name = CString::new(name)?;
            let mut ptr = MaybeUninit::<$type_>::zeroed();
            let errno = unsafe {
                sys::$smethod_get(self.ptr, c_name.as_ptr(), ptr.as_mut_ptr())
            };
            if errno != 0 {
                Err(NvError::from_errno(errno))
            } else {
                let ret = unsafe { ptr.assume_init() };
                Ok(ret)
            }
        }
    }
}

impl NvList {
    /// Make a copy of a pointer. Danger zone.
    pub fn as_ptr(&self) -> *mut sys::nvlist_t { self.ptr }

    pub fn new(flags: NvFlag) -> NvResult<Self> {
        let mut raw_list = std::ptr::null_mut();
        let errno = unsafe { sys::nvlist_alloc(&mut raw_list, flags as u32, 0) };
        if errno != 0 {
            Err(NvError::from_errno(errno))
        } else {
            Ok(NvList { ptr: raw_list })
        }
    }
    pub fn is_empty(&self) -> bool {
        let ret = unsafe { sys::nvlist_empty(self.as_ptr()) };
        ret != sys::boolean::B_FALSE
    }

    pub fn exists(&self, name: &str) -> NvResult<bool> {
        let c_name = CString::new(name)?;
        let ret = unsafe { sys::nvlist_exists(self.as_ptr(), c_name.as_ptr()) };
        Ok(ret != sys::boolean::B_FALSE)
    }
    pub fn insert<T: NvTypeOp>(&mut self, name: &str, value: T) -> NvResult<()> {
        value.add_to_list(self, name)
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
        let errno = unsafe { sys::nvlist_add_boolean_value(self.ptr, c_name.as_ptr(), v) };
        if errno != 0 {
            Err(NvError::from_errno(errno))
        } else {
            Ok(())
        }
    }

    /// Get a `bool` from the list.
    pub fn get_bool(&self, name: &str) -> NvResult<bool> {
        let c_name = CString::new(name)?;
        let mut ptr = MaybeUninit::<sys::boolean::Type>::zeroed();

        let errno = unsafe {
            sys::nvlist_lookup_boolean_value(self.ptr, c_name.as_ptr(), ptr.as_mut_ptr())
        };
        if errno != 0 {
            Err(NvError::from_errno(errno))
        } else {
            let ret = unsafe { ptr.assume_init() };
            Ok(ret != sys::boolean::B_FALSE)
        }
    }
    /// Add a `&str` to the list.
    pub fn insert_string(&mut self, name: &str, value: &str) -> NvResult<()> {
        let c_name = CString::new(name)?;
        let c_value = CString::new(value)?;
        let errno = unsafe { sys::nvlist_add_string(self.ptr, c_name.as_ptr(), c_value.as_ptr()) };
        if errno != 0 {
            Err(NvError::from_errno(errno))
        } else {
            Ok(())
        }
    }
    /// Get a `String` from the list.
    pub fn get_string(&self, name: &str) -> NvResult<String> {
        let c_name = CString::new(name)?;
        let mut ptr = MaybeUninit::<*mut _>::zeroed();
        let errno = unsafe {
            sys::nvlist_lookup_string(self.ptr, c_name.as_ptr(), ptr.as_mut_ptr())
        };
        if errno != 0 {
            Err(NvError::from_errno(errno))
        } else {
            let ret = unsafe {
                ptr.assume_init();
                let val = CStr::from_ptr(*ptr.as_ptr());
                val.to_str()?.to_owned()
            };
            Ok(ret)
        }
    }
    /// Get a `String` from the list.
    pub fn get_str(&self, name: &str) -> NvResult<&str> {
        let c_name = CString::new(name)?;
        let mut ptr = MaybeUninit::<*mut _>::zeroed();
        let errno = unsafe {
            sys::nvlist_lookup_string(self.ptr, c_name.as_ptr(), ptr.as_mut_ptr())
        };
        if errno != 0 {
            Err(NvError::from_errno(errno))
        } else {
            let ret = unsafe {
                ptr.assume_init();
                let val = CStr::from_ptr(*ptr.as_ptr());
                val.to_str()?
            };
            Ok(ret)
        }
    }

    nvpair_type_method!(i8, insert_i8, nvlist_add_int8, get_i8, nvlist_lookup_int8);
    nvpair_type_method!(u8, insert_u8, nvlist_add_uint8, get_u8, nvlist_lookup_uint8);
    nvpair_type_method!(i16, insert_i16, nvlist_add_int16, get_i16, nvlist_lookup_int16);
    nvpair_type_method!(u16, insert_u16, nvlist_add_uint16, get_u16, nvlist_lookup_uint16);
    nvpair_type_method!(i32, insert_i32, nvlist_add_int32, get_i32, nvlist_lookup_int32);
    nvpair_type_method!(u32, insert_u32, nvlist_add_uint32, get_u32, nvlist_lookup_uint32);
    nvpair_type_method!(i64, insert_i64, nvlist_add_int64, get_i64, nvlist_lookup_int64);
    nvpair_type_method!(u64, insert_u64, nvlist_add_uint64, get_u64, nvlist_lookup_uint64);
    nvpair_type_array_method!(i8, insert_i8_array, nvlist_add_int8_array, get_i8_array, nvlist_lookup_int8_array);
    nvpair_type_array_method!(u8, insert_u8_array, nvlist_add_uint8_array, get_u8_array, nvlist_lookup_uint8_array);
    nvpair_type_array_method!(i16, insert_i16_array, nvlist_add_int16_array, get_i16_array, nvlist_lookup_int16_array);
    nvpair_type_array_method!(u16, insert_u16_array, nvlist_add_uint16_array, get_u16_array, nvlist_lookup_uint16_array);
    nvpair_type_array_method!(i32, insert_i32_array, nvlist_add_int32_array, get_i32_array, nvlist_lookup_int32_array);
    nvpair_type_array_method!(u32, insert_u32_array, nvlist_add_uint32_array, get_u32_array, nvlist_lookup_uint32_array);
    nvpair_type_array_method!(i64, insert_i64_array, nvlist_add_int64_array, get_i64_array, nvlist_lookup_int64_array);
    nvpair_type_array_method!(u64, insert_u64_array, nvlist_add_uint64_array, get_u64_array, nvlist_lookup_uint64_array);
}


impl_list_op!{bool, insert_bool, false}
impl_list_op!{i8, insert_i8, false}
impl_list_op!{u8, insert_u8, false}
impl_list_op!{i16, insert_i16, false}
impl_list_op!{u16, insert_u16, false}
impl_list_op!{i32, insert_i32, false}
impl_list_op!{u32, insert_u32, false}
impl_list_op!{i64, insert_i64, false}
impl_list_op!{u64, insert_u64, false}
impl_list_op!{&str, insert_string, false}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        assert!(list.is_empty());
        assert!(!list.exists("does_it_work").unwrap());
        list.insert_bool("does_it_work", true).unwrap();
        assert!(!list.is_empty());
        assert!(list.get_bool("does_it_work").unwrap());
        assert!(list.exists("does_it_work").unwrap());
    }
    #[test]
    fn nvop_boolean() {
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert("works", true).unwrap();
        assert!(list.exists("works").unwrap());
    }
    #[test]
    fn nvop_string() {
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert("works", "yay").unwrap();
        assert!(list.exists("works").unwrap());
    }

    #[test]
    fn nvop_i8() {
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert("works", 5 as i8).unwrap();
        assert!(list.exists("works").unwrap());
    }

    #[test]
    fn nvop_u8() {
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert("works", 5 as u8).unwrap();
        assert!(list.exists("works").unwrap());
    }

    #[test]
    fn nvop_i16() {
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert("works", 5 as i16).unwrap();
        assert!(list.exists("works").unwrap());
    }

    #[test]
    fn nvop_u16() {
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert("works", 5 as u16).unwrap();
        assert!(list.exists("works").unwrap());
    }

    #[test]
    fn nvop_i32() {
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert("works", 5 as i32).unwrap();
        assert!(list.exists("works").unwrap());
    }

    #[test]
    fn nvop_u32() {
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert("works", 5 as u32).unwrap();
        assert!(list.exists("works").unwrap());
    }

    #[test]
    fn nvop_i64() {
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert("works", 5 as i64).unwrap();
        assert!(list.exists("works").unwrap());
    }

    #[test]
    fn nvop_u64() {
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert("works", 5 as u64).unwrap();
        assert!(list.exists("works").unwrap());
    }

    #[test]
    fn cr_i8() {
        let val = 4;
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert_i8("random", val).expect("Failed to insert int8");
        let ret = list.get_i8("random").unwrap();
        assert_eq!(val, ret);
    }

    #[test]
    fn cr_u8() {
        let val = 4;
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert_u8("random", val).expect("Failed to insert uint8");
        let ret = list.get_u8("random").unwrap();
        assert_eq!(val, ret);
    }
    #[test]
    fn cr_i16() {
        let val = 4;
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert_i16("random", val).expect("Failed to insert int16");
        let ret = list.get_i16("random").unwrap();
        assert_eq!(val, ret);
    }
    #[test]
    fn cr_u16() {
        let val = 4;
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert_u16("random", val).expect("Failed to insert uint16");
        let ret = list.get_u16("random").unwrap();
        assert_eq!(val, ret);
    }
    #[test]
    fn cr_i32() {
        let val = 4;
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert_i32("random", val).expect("Failed to insert int32");
        let ret = list.get_i32("random").unwrap();
        assert_eq!(val, ret);
    }
    #[test]
    fn cr_u32() {
        let val = 4;
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert_u32("random", val).expect("Failed to insert uint32");
        let ret = list.get_u32("random").unwrap();
        assert_eq!(val, ret);
    }
    #[test]
    fn cr_i64() {
        let val = 4;
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert_i64("random", val).expect("Failed to insert int64");
        let ret = list.get_i64("random").unwrap();
        assert_eq!(val, ret);
    }
    #[test]
    fn cr_u64() {
        let val = 4;
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert_u64("random", val).expect("Failed to insert uint64");
        let ret = list.get_u64("random").unwrap();
        assert_eq!(val, ret);
    }

    #[test]
    fn cr_string() {
        let val = "yes";
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert_string("is_it_ready?", val).unwrap();
        let ret = list.get_string("is_it_ready?").unwrap();
        assert_eq!(val, &ret);

        let ret = list.get_str("is_it_ready?").unwrap();
        assert_eq!(val, ret);
    }

    #[test]
    fn cr_i8_array() {
        let mut val = [1,2,3,4 as i8];
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert_i8_array("works", &mut val as &mut [i8]).unwrap();
        assert!(list.exists("works").unwrap());
        let ret = list.get_i8_array("works").unwrap();
        assert_eq!(4, ret.len());
        assert_eq!(&val, &ret);
    }
    #[test]
    fn cr_u8_array() {
        let mut val = [1,2,3,4 as u8];
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert_u8_array("works", &mut val as &mut [u8]).unwrap();
        assert!(list.exists("works").unwrap());
        let ret = list.get_u8_array("works").unwrap();
        assert_eq!(4, ret.len());
        assert_eq!(&val, &ret);
    }

    #[test]
    fn cr_i16_array() {
        let mut val = [1,2,3,4 as i16];
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert_i16_array("works", &mut val as &mut [i16]).unwrap();
        assert!(list.exists("works").unwrap());
        let ret = list.get_i16_array("works").unwrap();
        assert_eq!(4, ret.len());
        assert_eq!(&val, &ret);
    }
    #[test]
    fn cr_u16_array() {
        let mut val = [1,2,3,4 as u16];
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert_u16_array("works", &mut val as &mut [u16]).unwrap();
        assert!(list.exists("works").unwrap());
        let ret = list.get_u16_array("works").unwrap();
        assert_eq!(4, ret.len());
        assert_eq!(&val, &ret);
    }
    #[test]
    fn cr_i32_array() {
        let mut val = [1,2,3,4 as i32];
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert_i32_array("works", &mut val as &mut [i32]).unwrap();
        assert!(list.exists("works").unwrap());
        let ret = list.get_i32_array("works").unwrap();
        assert_eq!(4, ret.len());
        assert_eq!(&val, &ret);
    }
    #[test]
    fn cr_u32_array() {
        let mut val = [1,2,3,4 as u32];
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert_u32_array("works", &mut val as &mut [u32]).unwrap();
        assert!(list.exists("works").unwrap());
        let ret = list.get_u32_array("works").unwrap();
        assert_eq!(4, ret.len());
        assert_eq!(&val, &ret);
    }

    #[test]
    fn cr_i64_array() {
        let mut val = [1,2,3,4 as i64];
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert_i64_array("works", &mut val as &mut [i64]).unwrap();
        assert!(list.exists("works").unwrap());
        let ret = list.get_i64_array("works").unwrap();
        assert_eq!(4, ret.len());
        assert_eq!(&val, &ret);
    }
    #[test]
    fn cr_u64_array() {
        let mut val = [1,2,3,4 as u64];
        let mut list = NvList::new(NvFlag::UniqueNameType).unwrap();
        list.insert_u64_array("works", &mut val as &mut [u64]).unwrap();
        assert!(list.exists("works").unwrap());
        let ret = list.get_u64_array("works").unwrap();
        assert_eq!(4, ret.len());
        assert_eq!(&val, &ret);
    }
}
