//! Types related to task management

use super::TaskContext;
use crate::config::MAX_SYSCALL_NUM;//我添加的代码

/// The task control block (TCB) of a task.
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    //我添加的代码-开始
    ///使用桶计数存储的系统调用计数
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    ///程序的开始时间
    pub start_time: usize
    //我添加的代码-结束
}

/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}
