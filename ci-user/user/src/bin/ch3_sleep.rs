#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{get_time, yield_};

/// 正确输出：（无报错信息）
/// get_time OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333! {...}
/// Test sleep OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!

#[no_mangle]
fn main() -> i32 {
    let current_time = get_time();
    assert!(current_time > 0);
    println!("get_time OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333! {}", current_time);
    let wait_for = current_time + 3000;
    while get_time() < wait_for {
        yield_();
    }
    println!("Test sleep OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!");
    0
}
