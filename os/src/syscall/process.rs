//! Process management syscalls
use alloc::sync::Arc;

use crate::{
    config::MAX_SYSCALL_NUM,
    loader::get_app_data_by_name,
    mm::{translated_refmut, translated_str},
    task::{
        add_task, current_task, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next, TaskStatus, map_current_va_range, unmap_current_va_range,
    },
    mm::{PageTable, VirtAddr, MapPermission}
};
// 我添加的代码-开始
use crate::task::current_user_token;
use crate::timer::get_time_us;
use crate::task::get_task_info;
//use crate::mm::memory_set::{MemorySet, MapArea};
use crate::config::PAGE_SIZE;
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

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("kernel:pid[{}] sys_exit", current_task().unwrap().pid.0);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel:pid[{}] sys_yield", current_task().unwrap().pid.0);
    suspend_current_and_run_next();
    0
}

pub fn sys_getpid() -> isize {
    trace!("kernel: sys_getpid pid:{}", current_task().unwrap().pid.0);
    current_task().unwrap().pid.0 as isize
}

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

pub fn sys_exec(path: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_exec", current_task().unwrap().pid.0);
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    trace!("kernel::pid[{}] sys_waitpid [{}]", current_task().unwrap().pid.0, pid);
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

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_get_time NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    -1
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
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");

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
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");

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
        "kernel:pid[{}] sys_spawn NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    -1
}

// YOUR JOB: Set task priority.
pub fn sys_set_priority(_prio: isize) -> isize {
    trace!(
        "kernel:pid[{}] sys_set_priority NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    -1
}
