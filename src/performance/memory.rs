//! 内存复用设施：线性分配器与对象池。
//!
//! 用于几何计算、渲染等高频路径，避免反复向系统申请内存：
//! [`SimpleArena`] 顺序分配、整体重置；[`ObjectPool`] 复用已归还的对象。
//! 本模块不涉及并发，两个类型都不是线程安全的。

/// 线性（bump）分配器：在一块预分配的字节缓冲区中顺序切分，只能整体重置。
///
/// 分配出的切片不能单独释放，适合「一轮计算内产生大量小数据、结束后整块丢弃」的场景。
pub struct SimpleArena {
    buffer: Vec<u8>,
    position: usize,
}

impl SimpleArena {
    /// 创建容量为 `capacity` 字节的分配器，缓冲区立即分配并清零。
    ///
    /// - `capacity`：可用字节总数，创建后不可扩容，超出即分配失败。
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0u8; capacity],
            position: 0,
        }
    }

    /// 把已用偏移归零，使全部容量可再次分配；此前发放的切片随之失效。
    ///
    /// 副作用：修改 `self`，不释放缓冲区也不清零其中内容。
    #[inline]
    pub fn reset(&mut self) {
        self.position = 0;
    }

    /// 缓冲区的总字节容量，创建后固定不变。
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    /// 已分配的字节数（当前分配偏移），`reset` 之后为 0。
    #[inline]
    pub fn used(&self) -> usize {
        self.position
    }

    /// 从当前位置切出 `size` 字节的可写切片，并把偏移前移。
    ///
    /// - `size`：需要的字节数；
    /// 返回：成功时返回长度恰为 `size` 的切片（生命周期与 `&mut self` 绑定）；
    /// 剩余容量不足时返回 `None`，此时偏移不变，既不扩容也不回绕。
    #[inline]
    pub fn allocate(&mut self, size: usize) -> Option<&mut [u8]> {
        if self.position + size <= self.buffer.len() {
            let ptr = &mut self.buffer[self.position..self.position + size];
            self.position += size;
            Some(ptr)
        } else {
            None
        }
    }
}

/// 对象池：复用已归还的对象以减少分配次数。
///
/// 池空时以 `T::default()` 顶替，池满时归还的对象被直接丢弃；不保证归还顺序。
pub struct ObjectPool<T> {
    pool: Vec<T>,
    max_size: usize,
}

impl<T: Default> ObjectPool<T> {
    /// 创建空的对象池。
    ///
    /// - `max_size`：缓存数量上限，超出后 [`ObjectPool::release`] 会丢弃对象。
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: Vec::with_capacity(max_size),
            max_size,
        }
    }

    /// 取出一个对象：池非空时返回最近归还的那个，否则返回 `T::default()`（可能触发新分配）。
    ///
    /// 副作用：修改 `self`，成功取出时池内数量减一。
    #[inline]
    pub fn get(&mut self) -> T {
        self.pool.pop().unwrap_or_default()
    }

    /// 归还一个对象供后续复用。
    ///
    /// - `obj`：要缓存的对象；副作用：修改 `self`，池中数量已达上限时该对象被丢弃。
    #[inline]
    pub fn release(&mut self, obj: T) {
        if self.pool.len() < self.max_size {
            self.pool.push(obj);
        }
    }

    /// 当前池中缓存的对象数量，不会超过创建时给定的上限。
    #[inline]
    pub fn size(&self) -> usize {
        self.pool.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena() {
        let mut arena = SimpleArena::new(1024);
        let chunk = arena.allocate(100);
        assert!(chunk.is_some());
        assert_eq!(arena.used(), 100);
    }

    #[test]
    fn test_object_pool() {
        let mut pool: ObjectPool<i32> = ObjectPool::new(10);
        let val = pool.get();
        assert_eq!(val, 0);
        
        pool.release(42);
        let val = pool.get();
        assert_eq!(val, 42);
    }
}
