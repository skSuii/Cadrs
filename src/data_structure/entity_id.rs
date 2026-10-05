//! 对象标识：以 UUID 表示的全局唯一 ID。
//!
//! [`ObjectId`] 是实体、图层、块、块参照共用的键类型，文档内部以它为哈希键建立索引，
//! 因此修改标识会导致按旧标识的查询失败。区分两种"空"语义：[`ObjectId::new`] 生成随机
//! 非空标识用于新建对象，[`ObjectId::nil`] / [`ObjectId::null`] 生成全零标识表示"未设置"。

use uuid::Uuid;
use std::fmt;
use rand::Rng;

/// 对象的全局唯一标识，内部为 128 位 UUID。
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ObjectId(Uuid);

impl ObjectId {
    /// 生成一个新的随机标识，每次调用结果不同，用于新建实体/图层/块。
    #[inline]
    pub fn new() -> Self {
        let bytes: [u8; 16] = rand::random();
        Self(Uuid::from_bytes(bytes))
    }

    /// 全零的"空"标识，用作尚未归属图层、无来源对象等默认值。
    #[inline]
    pub fn nil() -> Self {
        Self(Uuid::nil())
    }

    /// 与 [`ObjectId::nil`] 等价的别名，语义上表示"无对象"。
    #[inline]
    pub fn null() -> Self {
        Self(Uuid::nil())
    }

    /// 底层 UUID 的引用，可用于序列化或与外部库互操作。
    #[inline]
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
    
    /// 取标识前 8 字节按本机字节序解释得到的 64 位整数，用作轻量哈希/句柄。
    ///
    /// 该值因平台字节序不同而不同，不保证跨平台一致，也不保证与原 UUID 互逆。
    #[inline]
    pub fn get_id(&self) -> u64 {
        let bytes = self.0.as_bytes();
        u64::from_ne_bytes(bytes[0..8].try_into().unwrap())
    }
}

impl Default for ObjectId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::str::FromStr for ObjectId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_id_creation() {
        let id = ObjectId::new();
        assert_ne!(id.as_uuid(), &Uuid::nil());
    }

    #[test]
    fn test_object_id_uniqueness() {
        let id1 = ObjectId::new();
        let id2 = ObjectId::new();
        assert_ne!(id1, id2);
    }
}
