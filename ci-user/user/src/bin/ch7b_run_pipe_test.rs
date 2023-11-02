#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{exec, fork, wait};

#[no_mangle]
pub fn main() -> i32 {
    for i in 0..1000 {
        if fork() == 0 {
            exec("ch7b_pipe_large_test\0", &[0 as *const u8]);
        } else {
            let mut _unused: i32 = 0;
            wait(&mut _unused);
            println!("Iter {} OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333.", i);
        }
    }
    0
}
