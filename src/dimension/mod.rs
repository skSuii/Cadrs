//! 尺寸标注（Dimension）子系统：把测量对象转换为可渲染的标注实体。
//!
//! 本模块按标注形态拆分子模块，并统一重导出其中的类型与函数：
//!
//! - `linear`：线性与对齐标注，同时提供子系统共用的样式与几何结构
//! - `angular`：角度标注（圆弧 / 三点 / 两线）与弧长标注
//! - `radial`：半径、直径与小半径标注
//! - `ordinate`：坐标标注
//! - `leader`：引线标注
//! - `strokes`：把标注分解为矢量笔画折线，供 Tessellation 与各导出格式复用
//!
//! 典型流程：先用 [`DimensionStyle`] 描述外观，再由各标注的构造函数生成几何，
//! 最后用 `Entity::from` 得到 [`crate::data_structure::EntityType::Dimension`] 实体并加入
//! [`crate::data_structure::Document`]。坐标与长度使用世界坐标下的图形单位，角度一律为弧度。

/// 线性与对齐标注，并定义子系统共用的 [`DimensionStyle`]、[`DimensionGeometry`] 及相关枚举。
pub mod linear;
/// 角度标注（圆弧 / 三点 / 两线）与弧长标注，以及角度单位的文本格式化。
pub mod angular;
/// 半径、直径与小半径标注。
pub mod radial;
/// 坐标标注，以特征点的 X 或 Y 坐标作为测量值。
pub mod ordinate;
/// 引线标注（Leader）：带箭头与文字的引线。
pub mod leader;
/// 标注的矢量笔画分解，含内置的数字与 CAD 符号笔画字体。
pub mod strokes;

pub use linear::*;
pub use angular::*;
pub use radial::*;
pub use ordinate::*;
pub use leader::*;
pub use strokes::{text_strokes, text_width, make_linear, make_radial, make_angular, decompose};
