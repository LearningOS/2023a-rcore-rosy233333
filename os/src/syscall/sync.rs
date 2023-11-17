use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{block_current_and_run_next, current_process, current_task};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;
/// sleep syscall
pub fn sys_sleep(ms: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_sleep",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}
/// mutex create syscall
pub fn sys_mutex_create(blocking: bool) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_list[id] = mutex;

        // 在死锁检测结构中记录
        process_inner.mutex_dd.add_resource(id, 1);

        id as isize
    } else {
        process_inner.mutex_list.push(mutex);
        let id = process_inner.mutex_list.len() - 1;
        
        // 在死锁检测结构中记录
        process_inner.mutex_dd.add_resource(id, 1);

        id as isize
    }
}
/// mutex lock syscall
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_lock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();

    // 维护死锁检测机构数据结构
    let tid: usize = current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
    let deadlock_detect = process_inner.deadlock_detect;
    let mutex_dd = &mut process_inner.mutex_dd;
    let mut dd_updated = false;
    // 先更新程序的需求，看是否安全
    mutex_dd.need[tid][mutex_id] += 1;
    if deadlock_detect && mutex_dd.check_state() == -1 {
        // 驳回需求
        mutex_dd.need[tid][mutex_id] -= 1;
        return -0xdead;
    }
    // 如果现在资源足够分配，则尝试进行分配，看是否安全
    if mutex_dd.work[mutex_id] >= 1 {
        mutex_dd.need[tid][mutex_id] -= 1;
        mutex_dd.allocation[tid][mutex_id] += 1;
        mutex_dd.work[mutex_id] -= 1;
        if deadlock_detect && mutex_dd.check_state() == -1 {
            // 驳回需求，并恢复mutex_dd结构
            mutex_dd.allocation[tid][mutex_id] -= 1;
            mutex_dd.work[mutex_id] += 1;
            return -0xdead;
        }
        dd_updated = true;
    }

    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    mutex.lock();

    if !dd_updated {
        // 之前因为资源不足没有立即分配，因此这里根据现在的情况更新死锁检测数据结构
        let process = current_process();
        let mut process_inner = process.inner_exclusive_access();
        let mutex_dd = &mut process_inner.mutex_dd;

        assert!(mutex_dd.work[mutex_id] >= 1);
        mutex_dd.need[tid][mutex_id] -= 1;
        mutex_dd.allocation[tid][mutex_id] += 1;
        mutex_dd.work[mutex_id] -= 1;
        // 若开启死锁检测，且进入了不安全状态
        if deadlock_detect && mutex_dd.check_state() == -1 {
            // 驳回需求，恢复mutex_dd结构，释放已经分配的互斥锁
            mutex_dd.allocation[tid][mutex_id] -= 1;
            mutex_dd.work[mutex_id] += 1;
            mutex.unlock();
            return -0xdead;
        }
    }

    0
}
/// mutex unlock syscall
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_unlock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();

    // 更新死锁检测数据结构
    let tid: usize = current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
    let mutex_dd = &mut process_inner.mutex_dd;
    assert!(mutex_dd.allocation[tid][mutex_id] >= 1);
    mutex_dd.allocation[tid][mutex_id] -= 1;
    mutex_dd.work[mutex_id] += 1;
    // 释放资源肯定不会导致死锁，因此不用检测

    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    mutex.unlock();
    0
}
/// semaphore create syscall
pub fn sys_semaphore_create(res_count: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        process_inner.semaphore_list.len() - 1
    };
    // 在死锁检测结构中记录
    process_inner.semaphore_dd.add_resource(id, res_count);
    id as isize
}
/// semaphore up syscall
pub fn sys_semaphore_up(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_up",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());

    // 更新死锁检测数据结构
    let tid: usize = current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
    let semaphore_dd = &mut process_inner.semaphore_dd;
    // assert!(semaphore_dd.allocation[tid][sem_id] >= 1);
    if semaphore_dd.allocation[tid][sem_id] >= 1 {
        semaphore_dd.allocation[tid][sem_id] -= 1;
    }
    semaphore_dd.work[sem_id] += 1;
    // 释放资源肯定不会导致死锁，因此不用检测

    drop(process_inner);
    sem.up();
    0
}
/// semaphore down syscall
#[allow(unused_variables)]
#[allow(unused_assignments)]
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_down",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());

    // 维护死锁检测机构数据结构
    let tid: usize = current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
    let deadlock_detect = process_inner.deadlock_detect;
    let semaphore_dd = &mut process_inner.semaphore_dd;
    let mut dd_updated = false;
    // 先更新程序的需求，看是否安全
    semaphore_dd.need[tid][sem_id] += 1;
    if deadlock_detect && semaphore_dd.check_state() == -1 {
        // 驳回需求
        semaphore_dd.need[tid][sem_id] -= 1;
        return -0xdead;
    }
    // // 如果现在资源足够分配，则尝试进行分配，看是否安全
    // if semaphore_dd.work[sem_id] >= 1 {
    //     semaphore_dd.need[tid][sem_id] -= 1;
    //     semaphore_dd.allocation[tid][sem_id] += 1;
    //     semaphore_dd.work[sem_id] -= 1;
    //     if deadlock_detect && semaphore_dd.check_state() == -1 {
    //         // 驳回需求，并恢复mutex_dd结构
    //         semaphore_dd.allocation[tid][sem_id] -= 1;
    //         semaphore_dd.work[sem_id] += 1;
    //         return -0xdead;
    //     }
    //     dd_updated = true;
    // }

    semaphore_dd.need[tid][sem_id] -= 1;
    semaphore_dd.allocation[tid][sem_id] += 1;
    semaphore_dd.work[sem_id] -= 1;
    if deadlock_detect && semaphore_dd.check_state() == -1 {
        // 驳回需求，并恢复mutex_dd结构
        semaphore_dd.allocation[tid][sem_id] -= 1;
        semaphore_dd.work[sem_id] += 1;
        return -0xdead;
    }
    dd_updated = true;

    drop(process_inner);
    sem.down();

    // if !dd_updated {
    //     // 之前因为资源不足没有立即分配，因此这里根据现在的情况更新死锁检测数据结构
    //     let process = current_process();
    //     let mut process_inner = process.inner_exclusive_access();
    //     let semaphore_dd = &mut process_inner.semaphore_dd;

    //     assert!(semaphore_dd.work[sem_id] >= 1);
    //     semaphore_dd.need[tid][sem_id] -= 1;
    //     semaphore_dd.allocation[tid][sem_id] += 1;
    //     semaphore_dd.work[sem_id] -= 1;
    //     // if semaphore_dd.work[sem_id] >= 1 {
    //     //     semaphore_dd.work[sem_id] -= 1;
    //     // }
    //     // 若开启死锁检测，且进入了不安全状态
    //     if deadlock_detect && semaphore_dd.check_state() == -1 {
    //         // 驳回需求，恢复mutex_dd结构，释放已经分配的互斥锁
    //         semaphore_dd.allocation[tid][sem_id] -= 1;
    //         semaphore_dd.work[sem_id] += 1;
    //         sem.up();
    //         return -0xdead;
    //     }
    // }

    0
}
/// condvar create syscall
pub fn sys_condvar_create() -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}
/// condvar signal syscall
pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_signal",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}
/// condvar wait syscall
pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_wait",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let tid = current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());

    
    // 维护死锁检测数据结构
    let mutex_dd = &mut process_inner.mutex_dd;
    assert!(mutex_dd.allocation[tid][mutex_id] >= 1);
    mutex_dd.allocation[tid][mutex_id] -= 1;
    mutex_dd.need[tid][mutex_id] += 1;
    mutex_dd.work[mutex_id] += 1;

    drop(process_inner);
    condvar.wait(mutex);

    // 维护死锁检测数据结构
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let mutex_dd = &mut process_inner.mutex_dd;
    assert!(mutex_dd.need[tid][mutex_id] >= 1);
    assert!(mutex_dd.work[mutex_id] >= 1);
    mutex_dd.allocation[tid][mutex_id] += 1;
    mutex_dd.need[tid][mutex_id] -= 1;
    mutex_dd.work[mutex_id] -= 1;

    0
}
/// enable deadlock detection syscall
///
/// YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(_enabled: usize) -> isize {
    trace!("kernel: sys_enable_deadlock_detect");
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    if _enabled != 1 && _enabled != 0 {
        return -1;
    }
    process_inner.deadlock_detect = _enabled != 0;
    0
}
