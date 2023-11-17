use alloc::vec::Vec;
use core::iter;
use alloc::vec;

/// 死锁检测数据结构
#[derive(Clone)]
pub struct DdStruct {

    /// 死锁检测-工作向量，代表系统剩余的资源
    /// 下标：资源id
    pub work: Vec<isize>,
    /// 死锁检测-分配矩阵，代表已分配的资源
    /// 下标：任务id，资源id
    pub allocation: Vec<Vec<usize>>,
    /// 死锁检测-需求矩阵，代表任务还需要的资源
    /// 下标：任务id，资源id
    pub need: Vec<Vec<usize>>,
    /// 死锁检测-结束向量，代表任务是否结束
    /// 下标：任务id
    pub finish: Vec<bool>,

    /// 资源种类
    resource_count: usize,
    /// 任务数量
    task_count: usize
}

impl DdStruct {

    /// 构造函数
    pub fn new() -> Self {
        // let mut task_vec: Vec<Vec<usize>> = Vec::new();
        // task_vec.push(Vec::new());
        DdStruct {
            work: Vec::new(),
            allocation: vec![Vec::new()],
            need: vec![Vec::new()],
            finish: vec![false],
            resource_count: 0,
            task_count: 1,
        }
    }

    /// 增加一种资源
    /// 对于增加的资源，work为资源数量，allocation、need均为0，
    pub fn add_resource(&mut self, id: usize, count: usize) {
        if id < self.resource_count {
            self.work[id] = count as isize;
            self.allocation.iter_mut().for_each(|task|{
                task[id] = 0;
            });
            self.need.iter_mut().for_each(|task|{
                task[id] = 0;
            });
        }
        else {
            while self.resource_count < id {
                self.resource_count += 1;
                self.work.push(0);
                self.allocation.iter_mut().for_each(|task|{
                    task.push(0);
                });
                self.need.iter_mut().for_each(|task|{
                    task.push(0);
                });
            }
            self.resource_count += 1;
            self.work.push(count as isize);
            self.allocation.iter_mut().for_each(|task|{
                task.push(0);
            });
            self.need.iter_mut().for_each(|task|{
                task.push(0);
            });
        }
    }

    /// 清除一种资源
    pub fn delete_resource(&mut self, id: usize) {
        assert!(id < self.resource_count);
        self.work[id] = 0;
        self.allocation.iter_mut().for_each(|task|{
            task[id] = 0;
        });
        self.need.iter_mut().for_each(|task|{
            task[id] = 0;
        });
    }

    /// 增加一个任务
    /// 对于增加的任务，allocation、need均为0，finish为false。
    pub fn add_task(&mut self, id: usize) {

        if id < self.task_count {
            self.allocation[id].iter_mut().for_each(|resource| {
                *resource = 0;
            });
            self.need[id].iter_mut().for_each(|resource| {
                *resource = 0;
            });
            self.finish[id] = false;
        }
        else {
            while self.task_count <= id {
                self.task_count = self.task_count + 1;
                self.allocation.push(Vec::from_iter(iter::repeat(0).take(self.resource_count)));
                self.need.push(Vec::from_iter(iter::repeat(0).take(self.resource_count)));
                self.finish.push(false);
            }
        }
    }

    /// 清除一个任务
    pub fn delete_task(&mut self, id: usize) {
        assert!(id < self.task_count);
        self.finish[id] = true;
    }

    /// 检查该状态是否安全
    /// 安全，则返回0；不安全，则返回-1。
    pub fn check_state(&self) -> i32 {
        //若所有任务均执行完成，直接返回
        let mut ok = true;
        for task in 0 .. self.task_count {
            if self.finish[task] == false {
                ok = false;
            }
        }
        if ok == true {
            return 0;
        }

        // 找到一个系统可以执行完成的序列
        for task in 0 .. self.task_count {
            let mut ok = true;
            if self.finish[task] == true {
                ok = false;
            }
            else {
                for resource in 0 .. self.resource_count {
                    if (self.need[task][resource] as isize) < self.work[resource] {
                        ok = false;
                        break;
                    }
                }
            }

            if ok == true {
                // 这个task满足条件，可以执行完毕
                // 模拟这个task执行完毕后的情景
                let mut try_self = self.clone();
                for resource in 0 .. try_self.resource_count {
                    try_self.work[resource] += try_self.allocation[task][resource] as isize; 
                    try_self.allocation[task][resource] = 0;
                    // try_self.work[resource] -= (try_self.need[task][resource] as isize);
                    // try_self.allocation[task][resource] += try_self.need[task][resource];
                    try_self.need[task][resource] = 0;
                }
                try_self.finish[task] = true;
                if try_self.check_state() == 0 {
                    return 0;
                }
            }
        }
        return -1;
    }
}