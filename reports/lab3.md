# 第而次实验报告

## 学习内容

进程管理的实现。

## chapter5练习的实现功能总结

为了实现内核前向兼容，我将之前在`TaskManager`中实现的一些方法转移到了`Processor`中，相对应地，也更改了其它位置对这些方法的调用。由此，将chapter4内实现的功能成功移植到该实验内，且可正常工作。

我实现了`spawn`系统调用，根据目标程序新建子进程，并且维护它和调用进程的父子关系。

我实现了stride调度算法，在TCB的inner字段中增加了记录stride调度算法所需的数据项`priority`和`stride`。增加了设置进程优先级的系统调用`set_priority`。最后，重写了`TaskManager::fetch`函数，让其每次选取`stride`值最小的进程进行调度，并且维护被调度的进程的`stride`值。

### 问答作业

#### 1. 实际情况是轮到 p1 执行吗？为什么？

轮不到。因为溢出原因，p2执行完后，stride值变为260-256=4。

#### 2. 为什么？尝试简单说明（不要求严格证明）。

在初始状态下，所有程序stride=0，符合STRIDE_MAX – STRIDE_MIN <= BigStride / 2条件。

每次执行一个stride最小的程序，设该程序原有的stride值为stride_min_original，因为优先级>=2，因此该程序增加的stride值 <= BigStride / 2。

该程序执行后，STRIDE_MIN >= stride_min_original，且STRIDE_MAX = max{STRIDE_MAX, stride_min_original + BigStride / 2} <= stride_min_original + BigStride / 2。

因此，STRIDE_MAX - STRIDE_MIN <= BigStride / 2。

由数学归纳法可证明，任意时刻均满足上式。

#### 3. 已知以上结论，考虑溢出的情况下，可以为 Stride 设计特别的比较器，让 BinaryHeap<Stride> 的 pop 方法能返回真正最小的 Stride。补全下列代码中的 partial_cmp 函数，假设两个 Stride 永远不会相等。

```Rust
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut result = self.0 - other.0;
        result = if result < 0 {result + 256} else {result};
        if result < 128 && result > 0 {
            Some(Ordering::Greater)
        }
        else if result > 128 {
            Some(Ordering::Less)
        }
        else {
            // result == 128或者非法输入，如self=other
            None
        }
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
```

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

    无

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

    系统调用`spawn`的实现参考了实验os源代码中`fork`和`exec`的实现。

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。