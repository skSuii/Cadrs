//! 图层层：图层的集合管理、分组、状态快照与过滤器。
//!
//! [`LayerManager`] 是图层的集中容器，创建时自带 `0`、`DEFPOINTS`、`VIEWPORT`、`CONSTRUCTION`
//! 四个图层并把 `0` 设为当前图层；`0` 图层不可删除、不可改名，批量冻结与锁定也会跳过它。
//! [`LayerStateManager`] 保存图层属性的快照以便恢复，[`LayerFilter`] 按名称、颜色、可见性等条件筛选图层。
//!
//! 注意与 `data_structure::Layer` 的区别：本模块的 [`Layer`] 描述图层自身的绘制与管理属性，
//! 实体归属哪个图层由文档模型记录。

/// 图层管理器：图层集合、分组、状态快照与过滤器。
pub mod layer_manager;
