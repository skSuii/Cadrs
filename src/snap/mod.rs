//! 对象捕捉（Snap）层：把光标位置吸附到实体的特征点上。
//!
//! 三个子模块各司其职：
//! - [`snap_point`]：[`SnapManager`] 按捕捉类型与捕捉半径在世界坐标中挑选最近的特征点；
//! - [`osnap`]：[`OsnapTracker`] 面向交互，维护当前捕捉标记与提示文字；
//! - [`candidates`]：直接枚举候选点与实体交点，供上层自行排序取舍。
//!
//! 坐标一律为世界坐标；返回 `None` 表示捕捉未启用、实体缺少对应类型的特征点，
//! 或候选点到光标的距离不小于捕捉半径（半径本身不命中）。

/// 捕捉类型、优先级、快照结构与捕捉计算 trait [`SnapCalculator`]。
pub mod snap_point;
/// 对象捕捉跟踪器（Osnap）：捕捉模式、标记、设置与命中跟踪。
pub mod osnap;
/// 捕捉候选点与实体交点的直接枚举函数。
pub mod candidates;

pub use snap_point::{SnapPoint, SnapType, SnapPriority, SnapManager, SnapCalculator, Snapshot};
pub use osnap::{OsnapTracker, OsnapMode, OsnapSettings, OsnapMarker};
pub use candidates::{snap_candidates, intersection_candidates, intersection_candidates_capped, entity_segments_capped};
