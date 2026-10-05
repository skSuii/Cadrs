//! 图案填充模块。
//!
//! 提供 `Hatch` 填充对象（图案填充、渐变填充、实体填充三种形式）、填充图案
//! `HatchPattern` 与标准图案库 `PatternLibrary`。填充范围由边界路径给出，
//! 外边界减去孔洞后即为实际覆盖区域；填充对象自身只描述样式，不改动文档。

/// 填充对象、填充图案、渐变填充与边界路径的实现。
pub mod hatch;

/// 常用类型的再导出：填充对象、填充图案与渐变填充。
pub use hatch::{Hatch, HatchPattern, GradientFill};
