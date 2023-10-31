//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the operating system.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.

mod context;
mod switch;
#[allow(clippy::module_inception)]
mod task;

use crate::loader::{get_app_data, get_num_app};
use crate::sync::UPSafeCell;
use crate::timer::get_time_us;//我添加的代码
use crate::trap::TrapContext;
use alloc::vec::Vec;
use lazy_static::*;
use switch::__switch;
pub use task::{TaskControlBlock, TaskStatus};

pub use context::TaskContext;

// 我添加的代码-开始
use crate::syscall::process::TaskInfo;
use crate::mm::VirtAddr;
use crate::mm::MapPermission;
// 我添加的代码-结束

/// The task manager, where all the tasks are managed.
///
/// Functions implemented on `TaskManager` deals with all task state transitions
/// and task context switching. For convenience, you can find wrappers around it
/// in the module level.
///
/// Most of `TaskManager` are hidden behind the field `inner`, to defer
/// borrowing checks to runtime. You can see examples on how to use `inner` in
/// existing functions on `TaskManager`.
pub struct TaskManager {
    /// total number of tasks
    num_app: usize,
    /// use inner value to get mutable access
    inner: UPSafeCell<TaskManagerInner>,
}

/// The task manager inner in 'UPSafeCell'
struct TaskManagerInner {
    /// task list
    tasks: Vec<TaskControlBlock>,
    /// id of current `Running` task
    current_task: usize,
}

lazy_static! {
    /// a `TaskManager` global instance through lazy_static!
    pub static ref TASK_MANAGER: TaskManager = {
        println!("init TASK_MANAGER");
        let num_app = get_num_app();
        println!("num_app = {}", num_app);
        let mut tasks: Vec<TaskControlBlock> = Vec::new();
        for i in 0..num_app {
            tasks.push(TaskControlBlock::new(get_app_data(i), i));
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}

impl TaskManager {
    /// Run the first task in task list.
    ///
    /// Generally, the first task in task list is an idle task (we call it zero process later).
    /// But in ch4, we load apps statically, so the first task is a real app.
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let next_task = &mut inner.tasks[0];
        next_task.task_status = TaskStatus::Running;
        //因为是第一个任务，因此一定是第一次调度，可以直接更新start_time
        next_task.start_time_us = get_time_us(); //我添加的代码
        next_task.started = true; //我添加的代码
        let next_task_cx_ptr = &next_task.task_cx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        // before this, we should drop local variables that must be dropped manually
        unsafe {
            __switch(&mut _unused as *mut _, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    /// Change the status of current `Running` task into `Ready`.
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let cur = inner.current_task;
        inner.tasks[cur].task_status = TaskStatus::Ready;
    }

    /// Change the status of current `Running` task into `Exited`.
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let cur = inner.current_task;
        inner.tasks[cur].task_status = TaskStatus::Exited;
    }

    /// Find next task to run and return task id.
    ///
    /// In this case, we only return the first `Ready` task in task list.
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    /// Get the current 'Running' task's token.
    fn get_current_token(&self) -> usize {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_user_token()
    }

    /// Get the current 'Running' task's trap contexts.
    fn get_current_trap_cx(&self) -> &'static mut TrapContext {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_trap_cx()
    }

    /// Change the current 'Running' task's program break
    pub fn change_current_program_brk(&self, size: i32) -> Option<usize> {
        let mut inner = self.inner.exclusive_access();
        let cur = inner.current_task;
        inner.tasks[cur].change_program_brk(size)
    }

    /// Switch current `Running` task to the task we have found,
    /// or there is no `Ready` task and we can exit with all applications completed
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;
            //我添加的代码-开始
            //需要判断任务是不是第一次被调度，通过判断started是否为false
            if inner.tasks[next].started == false {
                inner.tasks[next].start_time_us = get_time_us();
                inner.tasks[next].started = true;
            }
            //我添加的代码-结束
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            // go back to user mode
        } else {
            panic!("All applications completed!");
        }
    }

    //我添加的代码-开始
    ///在当前进程中记录一次中断调用
    fn record_one_syscall(&self, syscall_id: usize) {
        let mut inner = self.inner.exclusive_access();
        let current_task = inner.current_task;
        let current_tcb = &mut (inner.tasks[current_task]);
        current_tcb.syscall_times[syscall_id] = current_tcb.syscall_times[syscall_id] + 1;
    }

    ///处理sys_task_info调用
    fn get_task_info(&self, _ti: *mut TaskInfo) -> isize {
        let inner = self.inner.exclusive_access();
        let current_tcb = &inner.tasks[inner.current_task];
        let current_time_us = get_time_us();
        unsafe {
            (*_ti).status = current_tcb.task_status;
            (*_ti).syscall_times = current_tcb.syscall_times.clone();
            (*_ti).time = (current_time_us - current_tcb.start_time_us) / 1000;
        }
        0
    }

    /// 映射一个虚拟页号范围
    fn map_current_va_range(&self, va_low: VirtAddr, va_high: VirtAddr, permission: MapPermission) -> isize {
        let mut inner = self.inner.exclusive_access();
        let current_task = inner.current_task;
        let current_memory_set = &mut (inner.tasks[current_task].memory_set);

        current_memory_set.map_va_range(va_low, va_high, permission)
    }

    /// 取消映射一个虚拟页号范围
    fn unmap_current_va_range(&self, va_low: VirtAddr, va_high: VirtAddr) -> isize {
        let mut inner = self.inner.exclusive_access();
        let current_task = inner.current_task;
        let current_memory_set = &mut (inner.tasks[current_task].memory_set);

        current_memory_set.unmap_va_range(va_low, va_high)
    }
    //我添加的代码-结束
}

/// Run the first task in task list.
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

/// Switch current `Running` task to the task we have found,
/// or there is no `Ready` task and we can exit with all applications completed
fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

/// Change the status of current `Running` task into `Ready`.
fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

/// Change the status of current `Running` task into `Exited`.
fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

/// Get the current 'Running' task's token.
pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

/// Get the current 'Running' task's trap contexts.
pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}

/// Change the current 'Running' task's program break
pub fn change_program_brk(size: i32) -> Option<usize> {
    TASK_MANAGER.change_current_program_brk(size)
}

//我添加的代码-开始
///在当前进程中记录一次中断调用
pub fn record_one_syscall(syscall_id: usize) {
    TASK_MANAGER.record_one_syscall(syscall_id);
}

///处理sys_task_info调用
pub fn get_task_info(_ti: *mut TaskInfo) -> isize {
    TASK_MANAGER.get_task_info(_ti)
}

/// 映射一个虚拟页号范围
pub fn map_current_va_range(va_low: VirtAddr, va_high: VirtAddr, permission: MapPermission) -> isize {
    TASK_MANAGER.map_current_va_range(va_low, va_high, permission)
}

/// 映射一个虚拟页号范围
pub fn unmap_current_va_range(va_low: VirtAddr, va_high: VirtAddr) -> isize {
    TASK_MANAGER.unmap_current_va_range(va_low, va_high)
}
//我添加的代码-结束
