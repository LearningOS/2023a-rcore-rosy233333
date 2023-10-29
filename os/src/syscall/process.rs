//! Process management syscalls

use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE_BITS},
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus,
    },
    mm::{PageTable, VirtAddr, PhysAddr}
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
#[repr(C)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
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

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    // 我添加的代码-开始
    // 在第3章的sys_get_time上修改
    let us = get_time_us();
    unsafe {
        //*ts = TimeVal {
        //    sec: us / 1_000_000,
        //    usec: us % 1_000_000,
        //};
        let ts_sec_va = &((*_ts).sec) as *const usize;
        let ts_usec_va = &((*_ts).usec) as *const usize;
        let ts_sec_pa = map_user_va_to_pa(ts_sec_va as usize) as *mut usize;
        let ts_usec_pa = map_user_va_to_pa(ts_usec_va as usize) as *mut usize;
        *ts_sec_pa = us / 1_000_000;
        *ts_usec_pa = us % 1_000_000;
    }
    // 我添加的代码-结束
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    
    // 我添加的代码-开始
    // 在第3章，我自己实现的的sys_task_info上修改
    let mut ti_temp = TaskInfo{
        status: TaskStatus::Running,
        syscall_times: [0; MAX_SYSCALL_NUM],
        time: 0
    };
    get_task_info(&mut ti_temp as *mut TaskInfo);

    unsafe{
        let ti_status_va = &((*_ti).status) as *const TaskStatus;
        let ti_time_va = &((*_ti).time) as *const usize;
        let ti_status_pa = map_user_va_to_pa(ti_status_va) as *mut TaskStatus;
        let ti_time_pa = map_user_va_to_pa(ti_time_va) as *mut usize;
        *ti_status_pa = ti_temp.status;
        *ti_time_pa = ti_temp.time;
        for i in 0..MAX_SYSCALL_NUM {
            let ti_syscall_time_va = &((*_ti).syscall_times[i]) as *const usize;
            let ti_syscall_time_pa = map_user_va_to_pa(ti_syscall_time_va) as *mut usize;
            *ti_syscall_time_pa = ti_temp.syscall_times[i];
        }
    }
    
    // 我添加的代码-结束
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    -1
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    -1
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
