//! 几何模块：提供 Shi SDK 的基础几何图元与其常用算法。
//!
//! 本模块定义二维/三维点、直线、圆、圆弧、椭圆、多段线、B 样条与 NURBS 等图元类型，
//! 并在此基础上提供距离、求交、凸包、偏移等几何算法与曲线分析接口。所有坐标与长度均为
//! 模型空间的实数单位，角度一律以弧度表示；曲线参数 `t` 通常归一化到 `[0, 1]`。
//!
//! 与其他模块的关系：上层 ENTITY 实体、Document 文档、Layer 图层、Dimension 标注、
//! Tessellation 细分与 Snap 捕捉均以本模块的图元作为几何载体；向量运算复用
//! [`crate::math::Vector2`]，图元本身只描述形状，不携带图层、线型等文档属性。

/// 二维点模块，定义 [`point::Point`]。
pub mod point;
pub use point::Point;

/// 直线段模块，定义 [`line::Line`]。
pub mod line;
pub use line::Line;

/// 圆模块，定义 [`circle::Circle`]。
pub mod circle;
pub use circle::Circle;

/// 圆弧模块，定义 [`arc::Arc`]。
pub mod arc;
pub use arc::Arc;

/// 椭圆模块，定义 [`ellipse::Ellipse`]。
pub mod ellipse;
pub use ellipse::Ellipse;

/// 曲线抽象模块，定义 [`curve::Curve`] 特征与 [`curve::CurveType`] 分类。
pub mod curve;
pub use curve::Curve;

/// 常用几何算法模块，提供距离、凸包、偏移、裁剪等自由函数。
pub mod algorithms;
pub use algorithms::*;

/// 求交模块，提供图元之间的相交判定与交点的 [`intersection::IntersectionResult`] 描述。
pub mod intersection;
pub use intersection::IntersectionResult;

/// 布尔运算模块（仅在启用 `boolean` 特性时编译）。
#[cfg(feature = "boolean")]
pub mod boolean;

/// 扩展几何模块，提供多段线、椭圆弧与样条拟合多段线等补充图元。
pub mod extended_geometry;
pub use extended_geometry::{Polyline, EllipseArc, SplineFittedPolyline};

/// B 样条曲线模块，定义 [`bspline::BSpline`]。
pub mod bspline;
pub use bspline::BSpline;

/// NURBS 曲线模块，定义 [`nurbs::NURBS`]。
pub mod nurbs;
pub use nurbs::NURBS;

/// [`Point`] 的二维别名，等价于同一类型，便于在二维接口处表达意图。
pub type Point2D = Point;
/// [`Line`] 的二维别名，等价于同一类型。
pub type Line2D = Line;
/// [`Circle`] 的二维别名，等价于同一类型。
pub type Circle2D = Circle;
/// [`Arc`] 的二维别名，等价于同一类型。
pub type Arc2D = Arc;
/// [`Ellipse`] 的二维别名，等价于同一类型。
pub type Ellipse2D = Ellipse;

pub use crate::math::Vector2;
