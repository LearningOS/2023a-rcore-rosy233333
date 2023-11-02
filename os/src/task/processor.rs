//!Implementation of [`Processor`] and Intersection of control flow
//!
//! Here, the continuous operation of user apps in CPU is maintained,
//! the current running state of CPU is recorded,
//! and the replacement and transfer of control flow of different applications are executed.

use super::__switch;
use super::{fetch_task, TaskStatus};
use super::{TaskContext, TaskControlBlock};
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use alloc::sync::Arc;
use lazy_static::*;

// 我添加的代码-开始
use crate::timer::get_time_us;
use crate::syscall::process::TaskInfo;
use crate::mm::{VirtAddr, MapPermission};
// 我添加的代码-结束

/// Processor management structure
pub struct Processor {
    ///The task currently executing on the current processor
    current: Option<Arc<TaskControlBlock>>,

    ///The basic control flow of each core, helping to select and switch process
    idle_task_cx: TaskContext,
}

impl Processor {
    ///Create an empty Processor
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: TaskContext::zero_init(),
        }
    }

    ///Get mutable reference to `idle_task_cx`
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }

    ///Get current task in moving semanteme
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }

    ///Get current task in cloning semanteme
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }

    //我添加的代码-开始
    ///在当前进程中记录一次中断调用
    fn record_one_syscall(&mut self, syscall_id: usize) {
        let current_tcb_inner = &mut self.current.as_mut().unwrap().inner_exclusive_access();
        current_tcb_inner.syscall_times[syscall_id] = current_tcb_inner.syscall_times[syscall_id] + 1;
    }

    ///处理sys_task_info调用
    fn get_task_info(&self, _ti: *mut TaskInfo) -> isize {
        let current_tcb_inner = &self.current.as_ref().unwrap().inner_exclusive_access();
        let current_time_us = get_time_us();
        unsafe {
            (*_ti).status = current_tcb_inner.task_status;
            (*_ti).syscall_times = current_tcb_inner.syscall_times.clone();
            (*_ti).time = (current_time_us - current_tcb_inner.start_time_us) / 1000;
        }
        0
    }

    /// 映射一个虚拟页号范围
    fn map_current_va_range(&mut self, va_low: VirtAddr, va_high: VirtAddr, permission: MapPermission) -> isize {
        let current_tcb_inner = &mut self.current.as_mut().unwrap().inner_exclusive_access();
        let current_memory_set = &mut (current_tcb_inner.memory_set);

        current_memory_set.map_va_range(va_low, va_high, permission)
    }

    /// 取消映射一个虚拟页号范围
    fn unmap_current_va_range(&mut self, va_low: VirtAddr, va_high: VirtAddr) -> isize {
        let current_tcb_inner = &mut self.current.as_mut().unwrap().inner_exclusive_access();
        let current_memory_set = &mut (current_tcb_inner.memory_set);

        current_memory_set.unmap_va_range(va_low, va_high)
    }
    //我添加的代码-结束
}

lazy_static! {
    /// 处理器的实例
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe { UPSafeCell::new(Processor::new()) };
}

///The main part of process execution and scheduling
///Loop `fetch_task` to get the process that needs to run, and switch the process through `__switch`
pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(task) = fetch_task() {
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            // access coming task TCB exclusively
            let mut task_inner = task.inner_exclusive_access();
            // 我添加的代码-开始
            //需要判断任务是不是第一次被调度，通过判断started是否为false
            if task_inner.started == false {
                task_inner.start_time_us = get_time_us();
                task_inner.started = true;
            }
            // 我添加的代码-结束
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            task_inner.task_status = TaskStatus::Running;
            // release coming task_inner manually
            drop(task_inner);
            // release coming task TCB manually
            processor.current = Some(task);
            // release processor manually
            drop(processor);
            unsafe {
                __switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            warn!("no tasks available in run_tasks");
        }
    }
}

/// Get current task through take, leaving a None in its place
pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

/// Get a copy of the current task
pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

/// Get the current user token(addr of page table)
pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    task.get_user_token()
}

///Get the mutable reference to trap context of current task
pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_cx()
}

///Return to idle control flow for new scheduling
pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}

//我添加的代码-开始
///在当前进程中记录一次中断调用
pub fn record_one_syscall(syscall_id: usize) {
    let mut processor = PROCESSOR.exclusive_access();
    processor.record_one_syscall(syscall_id);
}

///处理sys_task_info调用
pub fn get_task_info(_ti: *mut TaskInfo) -> isize {
    let processor = PROCESSOR.exclusive_access();
    processor.get_task_info(_ti)
}

/// 映射一个虚拟页号范围
pub fn map_current_va_range(va_low: VirtAddr, va_high: VirtAddr, permission: MapPermission) -> isize {
    let mut processor = PROCESSOR.exclusive_access();
    processor.map_current_va_range(va_low, va_high, permission)
}

/// 映射一个虚拟页号范围
pub fn unmap_current_va_range(va_low: VirtAddr, va_high: VirtAddr) -> isize {
    let mut processor = PROCESSOR.exclusive_access();
    processor.unmap_current_va_range(va_low, va_high)
}
//我添加的代码-结束