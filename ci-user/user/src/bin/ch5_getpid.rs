#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::getpid;

/*
辅助测例 打印子进程 pid
*/

#[no_mangle]
pub fn main() -> i32 {
    let pid = getpid();
    println!("Test getpid OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333! pid = {}", pid);
    0
}
