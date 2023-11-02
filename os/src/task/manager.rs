//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::config::BIG_STRIDE;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch_old(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop_front()
    }

    // 我添加的代码-开始
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        // 选择stride最小的进程、
        let mut select_index: usize = 0;
        let mut min_stride: usize = usize::MAX;
        for index in 0 .. self.ready_queue.len() {
            let tcb = &(self.ready_queue[index]);
            let stride = tcb.inner_exclusive_access().stride;
            if stride <= min_stride {
                min_stride = stride;
                select_index = index;
            }
        }

        let option_task = self.ready_queue.swap_remove_back(select_index);

        // 更新stride
        if let Some(task) = &option_task {
            let stride = task.inner_exclusive_access().stride;
            let priority = task.inner_exclusive_access().priority;
            task.inner_exclusive_access().stride = stride + BIG_STRIDE / priority;
        }
        option_task
    }
    // 我添加的代码-结束
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
