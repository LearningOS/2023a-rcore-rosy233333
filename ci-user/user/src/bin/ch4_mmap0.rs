#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::mmap;

/*
理想结果：输出 Test 04_1 OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!
*/

#[no_mangle]
fn main() -> i32 {
    let start: usize = 0x10000000;
    let len: usize = 4096;
    let prot: usize = 3;
    assert_eq!(0, mmap(start, len, prot));
    for i in start..(start + len) {
        let addr: *mut u8 = i as *mut u8;
        unsafe {
            *addr = i as u8;
        }
    }
    for i in start..(start + len) {
        let addr: *mut u8 = i as *mut u8;
        unsafe {
            assert_eq!(*addr, i as u8);
        }
    }
    println!("Test 04_1 OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!");
    0
}
