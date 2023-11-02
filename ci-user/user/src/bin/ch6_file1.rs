#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::{close, fstat, open, OpenFlags, Stat, StatMode};

/// 测试 fstat，输出　Test fstat OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333! 就算正确。

#[no_mangle]
pub fn main() -> i32 {
    let fname = "fname1\0";
    let fd = open(fname, OpenFlags::CREATE | OpenFlags::WRONLY);
    assert!(fd > 0);
    let fd = fd as usize;
    let stat: Stat = Stat::new();
    let ret = fstat(fd, &stat);
    assert_eq!(ret, 0);
    assert_eq!(stat.mode, StatMode::FILE);
    assert_eq!(stat.nlink, 1);
    close(fd);
    // unlink(fname);
    // It's recommended to rebuild the disk image. This program will not clean the file "fname1".
    println!("Test fstat OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!");
    0
}
