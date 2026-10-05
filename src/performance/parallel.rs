//! 批处理：把长序列切成固定大小的块依次处理。
//!
//! [`ParallelBatcher`] 只负责切块与顺序调用，不创建线程、不做负载均衡；
//! 若需要真正的并行，由调用者在处理函数内部自行调度。

use std::marker::PhantomData;

/// 批处理器：按固定批大小把输入切块，并逐块调用调用者提供的函数。
///
/// 类型参数 `T` 只用于约束输入元素类型，结构体本身不持有任何元素。
pub struct ParallelBatcher<T> {
    batch_size: usize,
    _phantom: PhantomData<T>,
}

impl<T: Clone> ParallelBatcher<T> {
    /// 创建批处理器。
    ///
    /// - `batch_size`：每批的元素个数，必须大于 0，否则调用 [`ParallelBatcher::process_batches`] 会 panic。
    pub fn new(batch_size: usize) -> Self {
        Self { 
            batch_size,
            _phantom: PhantomData,
        }
    }

    /// 按 `batch_size` 顺序切分 `items`，对每块调用一次 `f` 并收集返回值。
    ///
    /// - `items`：待处理元素，空切片返回空向量；
    /// - `f`：接收单个切片的闭包，最后一块可能不足 `batch_size`；
    /// 返回：与块数等长的结果向量，顺序与输入一致；不修改 `items`，仅借用 `&self`。
    pub fn process_batches<F, R>(&self, items: &[T], f: F) -> Vec<R>
    where F: Fn(&[T]) -> R, R: Clone {
        let mut results = Vec::new();
        
        for chunk in items.chunks(self.batch_size) {
            results.push(f(chunk));
        }
        
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_batcher() {
        let batcher = ParallelBatcher::new(10);
        let items: Vec<i32> = (0..100).collect();
        
        let batches: Vec<Vec<i32>> = batcher.process_batches(&items, |batch| batch.to_vec());
        
        assert_eq!(batches.len(), 10);
        assert_eq!(batches[0].len(), 10);
    }
}
