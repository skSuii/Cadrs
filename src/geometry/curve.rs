//! 曲线抽象模块：定义所有曲线图元共用的求值接口 [`Curve`] 与曲线种类枚举 [`CurveType`]。
//!
//! [`Curve`] 只描述“按参数求点”的最小契约，长度、导数、曲率等更复杂的能力由
//! `crate::geometry::analysis` 中的分析接口提供；这里的 `t` 一律为归一化参数（约定落在
//! `[0, 1]`），坐标单位为模型空间单位，角度为弧度。`CurveType` 用于在不持有具体图元时
//! 记录种类标签，便于序列化、约束求解与 IO 输出。

use super::point::Point;

/// 曲线统一接口，供 Tessellation 细分、Snap 捕捉与曲线分析等通用算法以 `&dyn Curve` 调用。
///
/// 参数 `t` 约定归一化到 `[0, 1]`，`0.0` 为一端、`1.0` 为另一端。
pub trait Curve {
    /// 返回参数 `t` 处的点；超出 `[0, 1]` 的取值行为由实现决定，可能外推或截断。
    fn point_at(&self, t: f64) -> Point;
    /// 返回参数 `t` 处的切向量，方向与参数增长方向一致，模长由实现决定（不保证为单位向量）。
    fn tangent_at(&self, t: f64) -> Point;
    /// 返回曲线的轴对齐包围盒 `(min, max)`，应与曲线实际范围一致。
    fn bounding_box(&self) -> (Point, Point);
    /// 返回参数的合法区间 `(t_min, t_max)`，用于判断参数是否在定义域内。
    fn parameter_range(&self) -> (f64, f64);
    /// 返回曲线长度，结果按给定公差控制逼近精度，公差越小越精确但计算越慢。
    fn length(&self, tolerance: f64) -> f64;
    /// 判断曲线是否闭合，即首尾端点是否重合。
    fn is_closed(&self) -> bool;
    /// 返回曲线的次数（阶数减一），直线为 `1`；用于判断连续性与选择求值算法。
    fn degree(&self) -> usize;
}

/// 曲线种类标签，覆盖本 SDK 支持的全部曲线图元。
///
/// 仅描述种类，不携带几何数据；需要几何信息时请使用对应的具体图元类型。
#[derive(Debug, Clone, PartialEq)]
pub enum CurveType {
    /// 直线段，对应 [`crate::geometry::Line`]。
    Line,
    /// 圆弧，对应 [`crate::geometry::Arc`]。
    Arc,
    /// 椭圆或椭圆弧，对应 [`crate::geometry::Ellipse`]。
    Ellipse,
    /// 多项式 B 样条曲线，对应 [`crate::geometry::BSpline`]。
    BSpline,
    /// 有理 B 样条曲线（带权因子），对应 [`crate::geometry::NURBS`]。
    NURBS,
    /// 多段线，对应 [`crate::geometry::Polyline`]。
    Polyline,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curve_trait_bounds() {
        let _ = CurveType::Line;
    }
}
