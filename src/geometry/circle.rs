//! 圆图元模块：定义由圆心与半径确定的完整圆 [`Circle`]。
//!
//! 圆位于与 XY 平面平行的平面上，圆心沿用 [`Point`] 的 Z 坐标，但角度、包含判断等平面
//! 运算只使用 `x`/`y`。长度与面积均为模型空间单位，角度一律为弧度；需要圆弧范围时请使用
//! [`crate::geometry::Arc`]。

use crate::geometry::Point;
use crate::math::Vector2;
use std::fmt;
use serde::{Serialize, Deserialize};

/// 完整圆，由圆心 [`Point`] 与半径确定。
///
/// 半径为模型空间单位。结构体不校验半径取值，`radius` 为 `0.0` 时面积为 `0.0` 且
/// 曲率为无穷；类型为 `Copy`，所有方法均不修改自身。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Circle {
    /// 圆心，Z 坐标表示该圆所在平面的高度。
    pub center: Point,
    /// 半径，模型空间单位；负值不会被拒绝，但几何意义未定义。
    pub radius: f64,
}

impl Circle {
    /// 以圆心与半径构造圆。
    ///
    /// - `center`：圆心。
    /// - `radius`：半径，模型空间单位。
    #[inline]
    pub fn new(center: Point, radius: f64) -> Self {
        Self { center, radius }
    }

    /// 构造平面圆的语义别名，行为与 [`Circle::new`] 完全一致。
    ///
    /// - `center`：圆心。
    /// - `radius`：半径。
    #[inline]
    pub fn new2d(center: Point, radius: f64) -> Self {
        Self { center, radius }
    }

    /// 返回直径，等于半径的两倍。
    #[inline]
    pub fn diameter(&self) -> f64 {
        self.radius * 2.0
    }

    /// 返回周长 `2πr`。
    #[inline]
    pub fn circumference(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }

    /// 返回圆面积 `πr²`。
    #[inline]
    pub fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }

    /// 返回圆上指定角度的点。
    ///
    /// - `angle`：从圆心出发、相对 X 轴正方向的弧度角，逆时针为正；不要求落在
    ///   `[0, 2π)`，超出范围的角按周期性等价处理。返回点的 Z 坐标等于圆心 Z 坐标。
    ///
    /// # 示例
    /// ```
    /// # use cadrs::geometry::{Circle, Point};
    /// let circle = Circle::new(Point::origin(), 1.0);
    /// let p = circle.point_at_angle(0.0);
    /// assert!((p.x - 1.0).abs() < 1e-10 && p.y.abs() < 1e-10);
    /// ```
    #[inline]
    pub fn point_at_angle(&self, angle: f64) -> Point {
        Point::new(
            self.center.x + self.radius * angle.cos(),
            self.center.y + self.radius * angle.sin(),
            self.center.z,
        )
    }

    /// 返回圆上某点处的单位切向量，方向为逆时针（角度增大的方向）。
    ///
    /// - `p`：圆上的点；传入非圆上的点时会按该点相对圆心的方向给出对应半径处的切向量。
    ///   当 `p` 与圆心重合时切向量为零向量。
    #[inline]
    pub fn tangent_at_point(&self, p: &Point) -> Vector2 {
        let to_point = p.to_vector2() - self.center.to_vector2();
        Vector2::new(-to_point.y, to_point.x).normalize()
    }

    /// 判断点是否落在圆内或圆周上。
    ///
    /// 判定使用 `1e-10` 的绝对容差，因此边界上的点算作包含；仅比较 `x`/`y`，
    /// 忽略查询点与圆心的 Z 坐标差异。
    ///
    /// - `p`：查询点。
    #[inline]
    pub fn contains_point(&self, p: &Point) -> bool {
        self.center.distance_to(p) <= self.radius + 1e-10
    }

    /// 把任意角度归一到 `[0, 2π)`。
    ///
    /// - `angle`：任意弧度值，负数与超过 `2π` 的值都会被等价折算；返回值为左闭右开区间内的角。
    #[inline]
    pub fn normalize_angle(&self, angle: f64) -> f64 {
        let two_pi = 2.0 * std::f64::consts::PI;
        ((angle % two_pi) + two_pi) % two_pi
    }

    /// 返回从圆心指向给定点的弧度角，取值 `(-π, π]`，逆时针为正。
    ///
    /// 与 [`Circle::normalize_angle`] 不同，本方法不做归一化；结果只取决于 `x`/`y` 差向量。
    ///
    /// - `p`：查询点，与圆心重合时返回 `0.0`。
    #[inline]
    pub fn angle_from_center(&self, p: &Point) -> f64 {
        let to_point = p.to_vector2() - self.center.to_vector2();
        to_point.y.atan2(to_point.x)
    }

    /// 返回半径，等价于公开字段 `radius`。
    #[inline]
    pub fn radius(&self) -> f64 {
        self.radius
    }

    /// 返回圆心，等价于公开字段 `center`。
    #[inline]
    pub fn center(&self) -> Point {
        self.center
    }
}

impl fmt::Display for Circle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Circle(center: {}, radius: {})", self.center, self.radius)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_creation() {
        let center = Point::new(0.0, 0.0, 0.0);
        let circle = Circle::new(center, 5.0);
        
        assert_eq!(circle.radius, 5.0);
        assert!((circle.diameter() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_circle_area() {
        let circle = Circle::new(Point::origin(), 1.0);
        assert!((circle.area() - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_circle_point_at_angle() {
        let circle = Circle::new(Point::origin(), 1.0);
        let p = circle.point_at_angle(0.0);
        
        assert!((p.x - 1.0).abs() < 1e-10);
        assert!((p.y - 0.0).abs() < 1e-10);
    }
}
