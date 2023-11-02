#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{spawn, wait, waitpid};

/// 程序行为：先后产生 3 个有特定返回值的程序，检查 waitpid 能够获取正确返回值。

/// 理想输出：
/// new child i
/// Test wait OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!
/// Test waitpid OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!

#[no_mangle]
pub fn main() -> i32 {
    let cpid = spawn("ch5_exit0\0");
    assert!(cpid >= 0, "child pid invalid");
    println!("new child {}", cpid);
    let mut exit_code: i32 = 0;
    let exit_pid = wait(&mut exit_code);
    assert_eq!(exit_pid, cpid, "error exit pid");
    assert_eq!(exit_code, 66778, "error exit code");
    println!("Test wait OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!");
    let (cpid0, cpid1) = (spawn("ch5_exit0\0"), spawn("ch5_exit1\0"));
    let exit_pid = waitpid(cpid1 as usize, &mut exit_code);
    assert_eq!(exit_pid, cpid1, "error exit pid");
    assert_eq!(exit_code, -233, "error exit code");
    let exit_pid = wait(&mut exit_code);
    assert_eq!(exit_pid, cpid0, "error exit pid");
    assert_eq!(exit_code, 66778, "error exit code");
    println!("Test waitpid OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!");
    0
}
