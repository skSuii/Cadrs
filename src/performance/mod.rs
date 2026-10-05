//! 性能工具层：内存复用与批处理辅助设施。
//!
//! 面向几何计算、渲染等热路径的小工具，本层不创建线程、不阻塞调用者：
//! - [`memory`]：线性分配器 [`SimpleArena`](memory::SimpleArena) 与对象池 [`ObjectPool`](memory::ObjectPool)，减少重复分配；
//! - [`parallel`]：批处理器 [`ParallelBatcher`](parallel::ParallelBatcher)，只做切块与顺序调用，是否真正并行由调用者决定。

/// 内存管理：线性分配器与对象池。
pub mod memory;
/// 批处理：把长序列切成固定大小的块逐个处理。
pub mod parallel;
