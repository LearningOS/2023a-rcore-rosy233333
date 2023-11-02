#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::{close, open, unlink, write, OpenFlags};

/// 测试大量 open/unlink，输出 Test mass open/unlink OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333! 就算正确。

#[no_mangle]
pub fn main() -> i32 {
    let test_str = "some random long long long long long long long long string".repeat(50);
    let fname = "fname3\0";
    for i in 0..10 {
        let fd = open(fname, OpenFlags::CREATE | OpenFlags::WRONLY);
        if fd == -1 {
            panic!("failed to crate file");
        }
        let fd = fd as usize;
        for _ in 0..50 {
            write(fd, test_str.as_bytes());
        }
        close(fd);
        assert_eq!(unlink(fname), 0);
        let fd = open(fname, OpenFlags::RDONLY);
        assert!(fd < 0);
        println!("test iteration {}", i)
    }
    println!("Test mass open/unlink OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!");
    0
}
