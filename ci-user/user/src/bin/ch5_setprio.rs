#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::set_priority;

/// 正确输出：（无报错信息）
/// Test set_priority OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!

#[no_mangle]
pub fn main() -> i32 {
    assert_eq!(set_priority(10), 10);
    assert_eq!(set_priority(isize::MAX), isize::MAX);
    assert_eq!(set_priority(0), -1);
    assert_eq!(set_priority(1), -1);
    assert_eq!(set_priority(-10), -1);
    println!("Test set_priority OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!");
    0
}
