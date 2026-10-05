//! 块参照：块定义在图纸中的一次插入实例。
//!
//! [`BlockReference`] 只保存引用信息（目标块 id、插入点、缩放、旋转），不复制块内实体；
//! 因此修改块定义会同时影响所有引用它的块参照。插入点与块定义的插入基点对齐，缩放为二维
//! 向量（X/Y 可不等比），旋转角以弧度计。

use super::entity_id::ObjectId;
use super::super::geometry::Point;
use crate::Vector2;

/// 块参照：指向某个块定义的一次插入，记录其位置、缩放与旋转。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlockReference {
    id: ObjectId,
    block_id: ObjectId,
    insertion_point: Point,
    scale: Vector2,
    rotation: f64,
}

impl BlockReference {
    /// 新建块参照。
    ///
    /// 缩放默认为 `(1.0, 1.0)`、旋转默认为 `0.0` 弧度；本方法不校验 `block_id` 是否真的
    /// 存在于文档中，解析失败的引用需由调用方处理。
    ///
    /// - `block_id`：目标块定义的标识；
    /// - `insertion_point`：插入点坐标，与块定义的插入基点对齐。
    #[inline]
    pub fn new(block_id: ObjectId, insertion_point: Point) -> Self {
        Self {
            id: ObjectId::new(),
            block_id,
            insertion_point,
            scale: Vector2::new(1.0, 1.0),
            rotation: 0.0,
        }
    }

    /// 块参照自身的唯一标识，与目标块定义的 id 不同。
    #[inline]
    pub fn id(&self) -> &ObjectId {
        &self.id
    }

    /// 目标块定义的标识，用于在文档块表中查找 [`super::block::Block`]。
    #[inline]
    pub fn block_id(&self) -> &ObjectId {
        &self.block_id
    }

    /// 插入点坐标，块定义中的插入基点将落在该位置。
    #[inline]
    pub fn insertion_point(&self) -> Point {
        self.insertion_point
    }

    /// 移动块参照，会修改 `self`。
    #[inline]
    pub fn set_insertion_point(&mut self, point: Point) {
        self.insertion_point = point;
    }

    /// 二维缩放系数，X/Y 方向可不等比。
    #[inline]
    pub fn scale(&self) -> Vector2 {
        self.scale
    }

    /// 设置缩放系数，会修改 `self`；零或负值会导致图形退化或镜像，本方法不做校验。
    #[inline]
    pub fn set_scale(&mut self, scale: Vector2) {
        self.scale = scale;
    }

    /// 插入旋转角，单位为弧度。
    #[inline]
    pub fn rotation(&self) -> f64 {
        self.rotation
    }

    /// 设置插入旋转角，会修改 `self`，单位为弧度。
    #[inline]
    pub fn set_rotation(&mut self, rotation: f64) {
        self.rotation = rotation;
    }
}
