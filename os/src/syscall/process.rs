//! Process management syscalls
//!

use alloc::sync::Arc;

#[allow(unused_imports)]
use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE_BITS},
    fs::{open_file, OpenFlags},
    mm::{translated_refmut, translated_str},
    task::{
        add_task, current_task, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next, TaskStatus, TaskControlBlock,
    },
    mm::{PageTable, VirtAddr, MapPermission}
};
// 我添加的代码-开始
//use crate::task::current_user_token;
use crate::timer::get_time_us;
//use crate::mm::memory_set::{MemorySet, MapArea};
use crate::config::PAGE_SIZE;
use crate::task::processor::{map_current_va_range, unmap_current_va_range, get_task_info};
// 我添加的代码-结束

/// 包含sec和usec的代表时间的结构体
#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    /// 秒
    pub sec: usize,
    /// 微秒
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    pub status: TaskStatus,
    /// The numbers of syscall called by task
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    pub time: usize,
}

/// 程序退出
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("kernel:pid[{}] sys_exit", current_task().unwrap().pid.0);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

/// 程序放弃占有cpu
pub fn sys_yield() -> isize {
    //trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// 获得进程的pid
pub fn sys_getpid() -> isize {
    trace!("kernel: sys_getpid pid:{}", current_task().unwrap().pid.0);
    let result = current_task().unwrap().pid.0 as isize;
    result
}

/// fork进程
pub fn sys_fork() -> isize {
    trace!("kernel:pid[{}] sys_fork", current_task().unwrap().pid.0);
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

/// 在当前进程执行特定程序
pub fn sys_exec(path: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_exec", current_task().unwrap().pid.0);
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let task = current_task().unwrap();
        task.exec(all_data.as_slice());
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    //trace!("kernel: sys_waitpid");
    let task = current_task().unwrap();
    // find a child process

    // ---- access current PCB exclusively
    let mut inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after being removed from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child PCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB automatically
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
    trace!("kernel: sys_task_info");
    
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
        let ti_status_pa = map_user_va_to_pa(ti_status_va as usize) as *mut TaskStatus;
        let ti_time_pa = map_user_va_to_pa(ti_time_va as usize) as *mut usize;
        *ti_status_pa = ti_temp.status;
        *ti_time_pa = ti_temp.time;
        for i in 0..MAX_SYSCALL_NUM {
            let ti_syscall_time_va = &((*_ti).syscall_times[i]) as *const u32;
            let ti_syscall_time_pa = map_user_va_to_pa(ti_syscall_time_va as usize) as *mut u32;
            *ti_syscall_time_pa = ti_temp.syscall_times[i];
        }
    }
    
    // 我添加的代码-结束
    0
}

// YOUR JOB: Implement mmap.
/// 映射内存
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap");

    // 我添加的代码-开始
    // 判断地址是否对齐
    if _start & (PAGE_SIZE - 1) != 0 {
        return -1;
    }
    
    // 判断port是否合法
    if (_port & (!0x7) != 0) || (_port & 0x7 == 0) {
        return -1;
    }

    let va_low = VirtAddr::from(_start);
    // 要映射的地址不包含va_high
    let va_high = VirtAddr::from(_start + _len);
    let mut permission: MapPermission = MapPermission::U;
    if _port & (1 << 0) != 0 {
        permission = permission | MapPermission::R;
    }
    if _port & (1 << 1) != 0 {
        permission = permission | MapPermission::W;
    }
    if _port & (1 << 2) != 0 {
        permission = permission | MapPermission::X;
    }

    map_current_va_range(va_low, va_high, permission)
    // 我添加的代码-结束
}

// YOUR JOB: Implement munmap.
/// 取消内存映射
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap");

    // 我添加的代码-开始
    // 判断地址是否对齐
    if _start & (PAGE_SIZE - 1) != 0 {
        return -1;
    }

    let va_low = VirtAddr::from(_start);
    // 要释放的地址不包含va_high
    let va_high = VirtAddr::from(_start + _len);

    unmap_current_va_range(va_low, va_high)
    // 我添加的代码-结束
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel:pid[{}] sys_sbrk", current_task().unwrap().pid.0);
    if let Some(old_brk) = current_task().unwrap().change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

/// YOUR JOB: Implement spawn.
/// HINT: fork + exec =/= spawn
pub fn sys_spawn(_path: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_spawn",
        current_task().unwrap().pid.0
    );
    // 我添加的代码-开始
    // 参考了实验os源代码中`fork`和`exec`的实现
    let token = current_user_token();
    let path = translated_str(token, _path);
    trace!("1");
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        trace!("2");
        let all_data = app_inode.read_all();
        trace!("3");

        let current_task = current_task().unwrap();
        // 创建新进程
        // let new_task = Arc::new(TaskControlBlock::new(all_data.as_slice()));
        let new_task = current_task.fork();
        new_task.exec(all_data.as_slice());

        // 维护父子关系
        // current_task.inner_exclusive_access().children.push(Arc::clone(&new_task));
        // new_task.inner_exclusive_access().parent = Some(Arc::downgrade(&current_task));

        let new_pid = new_task.pid.0;
        add_task(new_task);
        trace!("4");
        new_pid as isize
    } else {
        -1
    }
    // 我添加的代码-结束
}

/// YOUR JOB: Set task priority.
/// 设置优先级
pub fn sys_set_priority(_prio: isize) -> isize {
    trace!(
        "kernel:pid[{}] sys_set_priority",
        current_task().unwrap().pid.0
    );
    // 我添加的代码-开始
    if _prio >= 2 {
        let task = current_task().unwrap();
        task.inner_exclusive_access().priority = _prio as usize;
        _prio
    }
    else {
        -1
    }
    // 我添加的代码-结束
}
