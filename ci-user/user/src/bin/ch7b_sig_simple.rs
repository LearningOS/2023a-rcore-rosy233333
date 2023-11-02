#![no_std]
#![no_main]

extern crate user_lib;

use user_lib::*;

fn func() {
    println!("user_sig_test passed31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333");
    sigreturn();
}

#[no_mangle]
pub fn main() -> i32 {
    let mut new = SignalAction::default();
    let mut old = SignalAction::default();
    new.handler = func as usize;

    println!("signal_simple: sigaction");
    if sigaction(SIGUSR1, Some(&new), Some(&mut old)) < 0 {
        panic!("Sigaction failed!");
    }
    println!("signal_simple: kill");
    if kill(getpid() as usize, SIGUSR1) < 0 {
        println!("Kill failed!");
        exit(1);
    }
    println!("signal_simple: Done");
    0
}
