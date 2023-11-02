#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::yield_;

/*
理想结果：三个程序交替输出 ABC
*/

const WIDTH: usize = 10;
const HEIGHT: usize = 5;

#[no_mangle]
fn main() -> i32 {
    for i in 0..HEIGHT {
        let buf = ['C' as u8; WIDTH];
        println!(
            "{} [{}/{}]",
            core::str::from_utf8(&buf).unwrap(),
            i + 1,
            HEIGHT
        );
        yield_();
    }
    println!("Test write C OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!");
    0
}
