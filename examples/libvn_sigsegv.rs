extern crate libnv;

use libnv::libnv::{NvList, NvFlag};

pub fn main() {
    let mut list = NvList::new(NvFlag::Both).unwrap();
    let result = list.insert_number("Important year", 1776u64);
    assert!(result.is_ok());

    let res = list.contains_key("Important year").unwrap();
    println!("Important year: {}", res);
}
