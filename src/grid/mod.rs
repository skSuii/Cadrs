//! 栅格与正交模块。
//!
//! `Grid` 按间距生成栅格点并把输入坐标捕捉到栅格；`Ortho` 把输入点约束到
//! 水平/垂直方向。两者都只处理调用者传入的坐标，不读写文档。
//!
//! 与 `snap`（对象捕捉）模块的区别：本模块捕捉规则栅格点，`snap` 捕捉实体上的
//! 特征点（端点、中点、圆心等）；栅格间距与角度单位随 `GridSettings` 定义。

/// 栅格、栅格设置、栅格点与正交约束的实现。
pub mod grid;

/// 常用类型的再导出：栅格及其设置、栅格点，正交及其设置。
pub use grid::{Grid, GridSettings, GridPoint, GridType, GridSnapStyle, Ortho, OrthoMode, OrthoSettings};
