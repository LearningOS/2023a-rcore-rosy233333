#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{mmap, munmap};

/*
理想结果：输出 Test 04_6 ummap2 OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!
*/

#[no_mangle]
fn main() -> i32 {
    let start: usize = 0x10000000;
    let len: usize = 4096;
    let prot: usize = 3;
    assert_eq!(0, mmap(start, len, prot));
    assert_eq!(munmap(start, len + 1), -1);
    assert_eq!(munmap(start + 1, len - 1), -1);
    println!("Test 04_6 ummap2 OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!");
    0
}
