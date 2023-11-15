//! File and filesystem-related syscalls
use easy_fs::Stat;

use crate::fs::{open_file, OpenFlags, ROOT_INODE};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token};

// 我新增的代码-开始
use crate::mm::PageTable;
use crate::mm::VirtAddr;
use crate::config::PAGE_SIZE_BITS;
use easy_fs::StatMode;
// 我新增的代码-结束

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_write", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_read", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        trace!("kernel: sys_read .. file.read");
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    trace!("kernel:pid[{}] sys_open", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    trace!("kernel:pid[{}] sys_close", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

// 我添加的代码-开始
/// 将某个存储单元的用户空间地址转化为物理地址，以供读写
fn map_user_va_to_pa(user_va: usize) -> usize {
    let user_page_table = PageTable::from_token(current_user_token());
    let vpn = VirtAddr(user_va).floor();
    let offset = VirtAddr(user_va).page_offset();
    let ppn = user_page_table.translate(vpn).unwrap().ppn();
    (ppn.0 << PAGE_SIZE_BITS) + offset
}
// 我添加的代码-结束

/// YOUR JOB: Implement fstat.
pub fn sys_fstat(_fd: usize, _st: *mut Stat) -> isize {
    trace!(
        "kernel:pid[{}] sys_fstat",
        current_task().unwrap().pid.0
    );
    // 我添加的代码-开始
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    let mut st_temp: Stat = Stat::new();
    if _fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[_fd] {
        let result = file.fstat(&mut st_temp);
        if result != 0 {
            result.try_into().unwrap()
        }
        else {
            unsafe {
                *(map_user_va_to_pa(&((*_st).ino) as *const u64 as usize) as *mut u64) = st_temp.ino;
                *(map_user_va_to_pa(&((*_st).mode) as *const StatMode as usize) as *mut StatMode) = st_temp.mode;
                *(map_user_va_to_pa(&((*_st).nlink) as *const u32 as usize) as *mut u32) = st_temp.nlink;
            }
            0
        }
    }
    else {
        -1
    }
    // 我添加的代码-结束
}

/// YOUR JOB: Implement linkat.
pub fn sys_linkat(_old_name: *const u8, _new_name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_linkat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    // 我添加的代码-开始
    let token = current_user_token();
    let old_name = translated_str(token, _old_name);
    let new_name = translated_str(token, _new_name);
    ROOT_INODE.create_and_link(&old_name, &new_name)
    // 我添加的代码-结束
}

/// YOUR JOB: Implement unlinkat.
pub fn sys_unlinkat(_name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_unlinkat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    // 我添加的代码-开始
    let token = current_user_token();
    let name = translated_str(token, _name);
    ROOT_INODE.find_and_unlink(&name)
    // 我添加的代码-结束
}
