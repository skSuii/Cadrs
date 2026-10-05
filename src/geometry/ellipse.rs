//! 椭圆图元模块：定义带旋转角的椭圆 [`Ellipse`]。
//!
//! 椭圆由中心、半长轴、半短轴与绕中心的旋转角确定，位于与 XY 平面平行的平面上；
//! `rotation` 与所有内部角度均为弧度，长度与面积为模型空间单位。椭圆是完整闭合曲线，
//! 若只需椭圆的一部分请在上层按参数区间裁剪。

use crate::geometry::Point;
use std::fmt;
use serde::{Serialize, Deserialize};

/// 椭圆，由中心 [`Point`]、半轴长度与旋转角确定。
///
/// 结构体不校验半轴大小关系，也不要求 `semi_major >= semi_minor`；类型为 `Copy`，
/// 所有方法均不修改自身。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ellipse {
    /// 椭圆中心，Z 坐标表示该椭圆所在平面的高度。
    pub center: Point,
    /// 半长轴长度（局部 X 方向），模型空间单位。
    pub semi_major: f64,
    /// 半短轴长度（局部 Y 方向），模型空间单位。
    pub semi_minor: f64,
    /// 绕中心的旋转角，弧度，逆时针为正；`0.0` 表示长轴与 X 轴重合。
    pub rotation: f64,
}

impl Ellipse {
    /// 以中心、半轴长度与旋转角构造椭圆。
    ///
    /// - `center`：中心。
    /// - `semi_major`：半长轴，局部 X 方向长度。
    /// - `semi_minor`：半短轴，局部 Y 方向长度。
    /// - `rotation`：旋转角，弧度，逆时针为正。
    #[inline]
    pub fn new(center: Point, semi_major: f64, semi_minor: f64, rotation: f64) -> Self {
        Self {
            center,
            semi_major,
            semi_minor,
            rotation,
        }
    }

    /// 返回离心率 `sqrt(1 - (b/a)²)`，圆为 `0.0`，越接近 `1.0` 越扁。
    ///
    /// 当 `semi_major` 为 `0.0`（退化情形）时直接返回 `0.0`；
    /// 若 `semi_minor` 大于 `semi_major`，平方根内为负而返回 `NaN`。
    #[inline]
    pub fn eccentricity(&self) -> f64 {
        if self.semi_major == 0.0 {
            0.0
        } else {
            (1.0 - (self.semi_minor / self.semi_major).powi(2)).sqrt()
        }
    }

    /// 返回椭圆面积 `πab`，`a`、`b` 分别为半长轴与半短轴。
    #[inline]
    pub fn area(&self) -> f64 {
        std::f64::consts::PI * self.semi_major * self.semi_minor
    }

    /// 返回周长的近似值（Ramanujan 第一近似），误差随离心率增大而增大。
    ///
    /// 椭圆周长没有初等闭式解，需要高精度时请改用数值积分或 Tessellation 细分累加。
    #[inline]
    pub fn circumference_approx(&self) -> f64 {
        let a = self.semi_major;
        let b = self.semi_minor;
        std::f64::consts::PI * (3.0 * (a + b) - ((3.0 * a + b) * (a + 3.0 * b)).sqrt())
    }

    /// 返回椭圆上参数 `t` 处的点，参数按整周归一化。
    ///
    /// - `t`：参数值，`0.0` 与 `1.0` 都对应局部 X 轴正方向的端点；不做区间限制，
    ///   超出 `[0, 1]` 的值按周期 `1.0` 折算，因此曲线首尾相接。
    ///
    /// 返回点已应用 `rotation` 旋转，Z 坐标等于中心 Z 坐标。
    #[inline]
    pub fn point_at_parameter(&self, t: f64) -> Point {
        let angle = t * 2.0 * std::f64::consts::PI;
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        let cos_rot = self.rotation.cos();
        let sin_rot = self.rotation.sin();

        let local_x = self.semi_major * cos_angle;
        let local_y = self.semi_minor * sin_angle;

        Point::new(
            self.center.x + local_x * cos_rot - local_y * sin_rot,
            self.center.y + local_x * sin_rot + local_y * cos_rot,
            self.center.z,
        )
    }

    /// 判断点是否落在椭圆内部或边界上。
    ///
    /// 判定在椭圆局部坐标系中进行，使用 `1e-10` 的绝对容差，边界上的点算作包含；
    /// 只比较 `x`/`y`，忽略查询点与中心的 Z 坐标差异。
    ///
    /// - `p`：查询点。
    #[inline]
    pub fn contains_point(&self, p: &Point) -> bool {
        let dx = p.x - self.center.x;
        let dy = p.y - self.center.y;
        let cos_rot = self.rotation.cos();
        let sin_rot = self.rotation.sin();

        let local_x = dx * cos_rot + dy * sin_rot;
        let local_y = -dx * sin_rot + dy * cos_rot;

        ((local_x / self.semi_major).powi(2) + (local_y / self.semi_minor).powi(2)) <= 1.0 + 1e-10
    }

    /// 返回长轴全长，即 `semi_major` 的两倍。
    #[inline]
    pub fn major_axis(&self) -> f64 {
        self.semi_major * 2.0
    }

    /// 返回短轴全长，即 `semi_minor` 的两倍。
    #[inline]
    pub fn minor_axis(&self) -> f64 {
        self.semi_minor * 2.0
    }

    /// 返回中心点，等价于公开字段 `center`。
    #[inline]
    pub fn center(&self) -> Point {
        self.center
    }
}

impl fmt::Display for Ellipse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Ellipse(center: {}, semi_major: {}, semi_minor: {}, rotation: {:.2}°)",
            self.center,
            self.semi_major,
            self.semi_minor,
            self.rotation.to_degrees()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ellipse_creation() {
        let ellipse = Ellipse::new(Point::origin(), 5.0, 3.0, 0.0);
        
        assert_eq!(ellipse.semi_major, 5.0);
        assert_eq!(ellipse.semi_minor, 3.0);
    }

    #[test]
    fn test_ellipse_eccentricity() {
        let ellipse = Ellipse::new(Point::origin(), 5.0, 3.0, 0.0);
        let ecc = ellipse.eccentricity();
        
        assert!(ecc > 0.0 && ecc < 1.0);
    }

    #[test]
    fn test_ellipse_area() {
        let ellipse = Ellipse::new(Point::origin(), 5.0, 3.0, 0.0);
        let area = ellipse.area();
        
        assert!((area - 15.0 * std::f64::consts::PI).abs() < 1e-10);
    }
}
