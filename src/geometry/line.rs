//! 直线段图元模块：定义由两个端点确定的有限长线段 [`Line`]。
//!
//! `Line` 表示线段而非无限长直线，端点顺序决定方向：方向向量、参数化取值与偏移方向都以
//! `start` → `end` 为准。长度与坐标均为模型空间单位，仅涉及平面的方法忽略 Z 坐标。

use crate::geometry::Point;
use crate::math::Vector2;
use std::fmt;
use serde::{Serialize, Deserialize};

/// 由起点与终点确定的直线段。
///
/// 端点为 [`Point`]，类型为 `Copy`；所有查询方法都不修改线段本身。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Line {
    /// 起点，也是参数 `t = 0.0` 对应的位置。
    pub start: Point,
    /// 终点，也是参数 `t = 1.0` 对应的位置。
    pub end: Point,
}

impl Line {
    /// 以起点、终点构造线段。
    ///
    /// - `start`：起点。
    /// - `end`：终点；与 `start` 重合时表示零长度线段，此类线段的方向向量为零向量。
    #[inline]
    pub fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }

    /// 构造平面线段的语义别名，行为与 [`Line::new`] 完全一致，Z 坐标不会被特殊处理。
    ///
    /// - `start`：起点。
    /// - `end`：终点。
    #[inline]
    pub fn new2d(start: Point, end: Point) -> Self {
        Self { start, end }
    }

    /// 返回由起点指向终点的单位方向向量。
    ///
    /// 仅使用 `x`/`y`；线段为零长度时归一化结果为零向量，不会 panic。
    #[inline]
    pub fn direction(&self) -> Vector2 {
        (self.end.to_vector2() - self.start.to_vector2()).normalize()
    }

    /// 返回线段的三维长度，等于两端点之间的距离，非负。
    #[inline]
    pub fn length(&self) -> f64 {
        self.start.distance_to(&self.end)
    }

    /// 返回线段中点，三个坐标分量分别取两端点的平均。
    #[inline]
    pub fn midpoint(&self) -> Point {
        Point::new(
            (self.start.x + self.end.x) / 2.0,
            (self.start.y + self.end.y) / 2.0,
            (self.start.z + self.end.z) / 2.0,
        )
    }

    /// 返回参数 `t` 处的点，按线性插值计算。
    ///
    /// - `t`：参数值，`0.0` 对应 `start`、`1.0` 对应 `end`；不做区间限制，区间外的值会外推。
    ///
    /// # 示例
    /// ```
    /// # use cadrs::geometry::{Line, Point};
    /// let line = Line::new(Point::new(0.0, 0.0, 0.0), Point::new(10.0, 10.0, 0.0));
    /// let p = line.point_at_parameter(0.5);
    /// assert!((p.x - 5.0).abs() < 1e-10);
    /// ```
    #[inline]
    pub fn point_at_parameter(&self, t: f64) -> Point {
        Point::new(
            self.start.x + (self.end.x - self.start.x) * t,
            self.start.y + (self.end.y - self.start.y) * t,
            self.start.z + (self.end.z - self.start.z) * t,
        )
    }

    /// 判断线段是否水平，即两端点 Y 坐标之差小于 `1e-10`。
    ///
    /// 使用绝对容差比较，与线段长度无关；零长度线段同时满足水平与垂直。
    #[inline]
    pub fn is_horizontal(&self) -> bool {
        (self.end.y - self.start.y).abs() < 1e-10
    }

    /// 判断线段是否垂直，即两端点 X 坐标之差小于 `1e-10`。
    ///
    /// 同样使用绝对容差比较。
    #[inline]
    pub fn is_vertical(&self) -> bool {
        (self.end.x - self.start.x).abs() < 1e-10
    }

    /// 返回线段上距离给定点最近的点。
    ///
    /// 结果被限制在线段范围内（参数截断到 `[0, 1]`），因此位于端点外侧时返回对应端点；
    /// 仅使用 `x`/`y`，返回点的 Z 坐标取自线段上的插值位置。
    ///
    /// - `p`：查询点。
    #[inline]
    pub fn closest_point(&self, p: &Point) -> Point {
        let d = self.end.to_vector2() - self.start.to_vector2();
        let to_point = p.to_vector2() - self.start.to_vector2();
        let len_sq = d.dot(&d);
        if len_sq < 1e-30 {
            return self.start;
        }
        let t = (to_point.dot(&d) / len_sq).clamp(0.0, 1.0);
        self.point_at_parameter(t)
    }

    /// 返回到给定点的最短距离，非负。
    ///
    /// 距离按 [`Line::closest_point`] 的投影点计算，点位于端点外侧时取到最近端点的距离。
    ///
    /// - `p`：查询点。
    #[inline]
    pub fn distance_to_point(&self, p: &Point) -> f64 {
        p.distance_to(&self.closest_point(p))
    }

    /// 返回起点，等价于公开字段 `start`。
    #[inline]
    pub fn start_point(&self) -> Point {
        self.start
    }

    /// 返回终点，等价于公开字段 `end`。
    #[inline]
    pub fn end_point(&self) -> Point {
        self.end
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Line({} -> {})", self.start, self.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::validation;

    #[test]
    fn test_line_creation() {
        let start = Point::new(0.0, 0.0, 0.0);
        let end = Point::new(1.0, 1.0, 0.0);
        let line = Line::new(start, end);
        
        assert_eq!(line.start, start);
        assert_eq!(line.end, end);
    }

    #[test]
    fn test_line_length() {
        let start = Point::new(0.0, 0.0, 0.0);
        let end = Point::new(3.0, 4.0, 0.0);
        let line = Line::new(start, end);
        
        assert!((line.length() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_line_midpoint() {
        let start = Point::new(0.0, 0.0, 0.0);
        let end = Point::new(2.0, 2.0, 0.0);
        let line = Line::new(start, end);
        let midpoint = line.midpoint();
        
        assert!((midpoint.x - 1.0).abs() < 1e-10);
        assert!((midpoint.y - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_line_direction() {
        let start = Point::new(0.0, 0.0, 0.0);
        let end = Point::new(3.0, 4.0, 0.0);
        let line = Line::new(start, end);
        let direction = line.direction();
        
        assert!((direction.magnitude() - 1.0).abs() < 1e-10);
        assert!((direction.x - 0.6).abs() < 1e-10);
        assert!((direction.y - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_line_point_at_parameter() {
        let start = Point::new(0.0, 0.0, 0.0);
        let end = Point::new(10.0, 10.0, 0.0);
        let line = Line::new(start, end);
        
        let t0 = line.point_at_parameter(0.0);
        assert_eq!(t0, start);
        
        let t1 = line.point_at_parameter(1.0);
        assert_eq!(t1, end);
        
        let t05 = line.point_at_parameter(0.5);
        assert!((t05.x - 5.0).abs() < 1e-10);
        assert!((t05.y - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_line_is_horizontal() {
        let horizontal = Line::new(Point::new(0.0, 0.0, 0.0), Point::new(10.0, 0.0, 0.0));
        let not_horizontal = Line::new(Point::new(0.0, 0.0, 0.0), Point::new(10.0, 1.0, 0.0));
        
        assert!(horizontal.is_horizontal());
        assert!(!not_horizontal.is_horizontal());
    }

    #[test]
    fn test_line_is_vertical() {
        let vertical = Line::new(Point::new(0.0, 0.0, 0.0), Point::new(0.0, 10.0, 0.0));
        let not_vertical = Line::new(Point::new(0.0, 0.0, 0.0), Point::new(1.0, 10.0, 0.0));
        
        assert!(vertical.is_vertical());
        assert!(!not_vertical.is_vertical());
    }

    #[test]
    fn test_line_closest_point() {
        let line = Line::new(Point::new(0.0, 0.0, 0.0), Point::new(10.0, 0.0, 0.0));
        
        let point_on_line = line.closest_point(&Point::new(5.0, 0.0, 0.0));
        assert!((point_on_line.x - 5.0).abs() < 1e-10);
        assert!((point_on_line.y - 0.0).abs() < 1e-10);
        
        let point_below = line.closest_point(&Point::new(5.0, -5.0, 0.0));
        assert!((point_below.x - 5.0).abs() < 1e-10);
        assert!((point_below.y - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_line_distance_to_point() {
        let line = Line::new(Point::new(0.0, 0.0, 0.0), Point::new(10.0, 0.0, 0.0));
        
        let distance = line.distance_to_point(&Point::new(5.0, 5.0, 0.0));
        assert!((distance - 5.0).abs() < 1e-10);
        
        let distance_on_line = line.distance_to_point(&Point::new(5.0, 0.0, 0.0));
        assert!((distance_on_line - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_line_display() {
        let line = Line::new(Point::new(1.0, 2.0, 0.0), Point::new(3.0, 4.0, 0.0));
        let display = format!("{}", line);
        assert!(display.contains("Line"));
        assert!(display.contains("1"));
        assert!(display.contains("2"));
        assert!(display.contains("3"));
        assert!(display.contains("4"));
    }

    #[test]
    fn test_line_clone() {
        let line1 = Line::new(Point::new(1.0, 2.0, 0.0), Point::new(3.0, 4.0, 0.0));
        let line2 = line1;
        assert_eq!(line1.start, line2.start);
        assert_eq!(line1.end, line2.end);
    }

    #[test]
    fn test_line_zero_length() {
        let start = Point::new(5.0, 5.0, 0.0);
        let end = Point::new(5.0, 5.0, 0.0);
        let line = Line::new(start, end);
        
        assert!((line.length() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_validation_positive() {
        assert!(validation::positive(1.0, "test").is_ok());
        assert!(validation::positive(0.0, "test").is_err());
        assert!(validation::positive(-1.0, "test").is_err());
    }

    #[test]
    fn test_validation_in_range() {
        assert!(validation::in_range(5.0, 0.0, 10.0, "test").is_ok());
        assert!(validation::in_range(-1.0, 0.0, 10.0, "test").is_err());
    }

    #[test]
    fn test_validation_non_negative() {
        assert!(validation::non_negative(0.0, "test").is_ok());
        assert!(validation::non_negative(1.0, "test").is_ok());
        assert!(validation::non_negative(-1.0, "test").is_err());
    }

    #[test]
    fn test_validation_scale_factor() {
        assert!(validation::scale_factor(1.0, "test").is_ok());
        assert!(validation::scale_factor(0.0, "test").is_err());
        assert!(validation::scale_factor(-1.0, "test").is_err());
        assert!(validation::scale_factor(f64::INFINITY, "test").is_err());
    }

    #[test]
    fn test_validation_coordinate() {
        assert!(validation::coordinate(1.0, "test").is_ok());
        assert!(validation::coordinate(f64::NAN, "test").is_err());
        assert!(validation::coordinate(f64::INFINITY, "test").is_err());
    }
}
