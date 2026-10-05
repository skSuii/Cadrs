//! 选择模块：选择集、选择模式、选择过滤器与选择管理器。
//!
//! 选择结果统一以 `ObjectId` 标识实体，选择集本身不持有实体数据；选择方式
//! （点选、窗口、交叉、栏选等）由 `SelectionMode` 描述，类型/图层/颜色等限制
//! 由 `SelectionFilter` 表达。命令通过 `command::CommandContext` 携带的选择集
//! 读取用户当前选中的实体。

/// 选择集、选择模式、选择过滤器与选择管理器的实现。
pub mod selection;

/// 常用选择类型的再导出。
pub use selection::{SelectionSet, SelectionMode, SelectionOption, SelectionFilter, SelectionManager, EntitySelector};
