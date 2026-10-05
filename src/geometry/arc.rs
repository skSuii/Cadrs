//! 圆弧图元模块：定义由圆心、半径与起止角确定的圆弧 [`Arc`]。
//!
//! 圆弧是有向曲线：`start_angle` 与 `end_angle` 均为弧度（相对 X 轴正方向，逆时针为正），
//! 实际扫掠方向由 `is_counter_clockwise` 决定，两个角之差并不直接等于圆心角，因此计算
//! 长度、中点或取点时应使用 [`Arc::angle_span`]、[`Arc::point_at_parameter`] 等感知方向的方法。
//! 圆弧复用 [`Point`] 与 [`Circle`] 的坐标约定，长度单位为模型空间单位。

use crate::geometry::Point;
use crate::geometry::Circle;
use std::fmt;
use serde::{Serialize, Deserialize};

/// 圆弧，由圆心、半径、起止角与扫掠方向确定。
///
/// 角度字段单位为弧度。结构体不做构造期校验，半径非正或起止角相等时
/// [`Arc::is_valid`] 返回 `false`；类型为 `Copy`，所有方法均不修改自身。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Arc {
    /// 圆心，Z 坐标表示圆弧所在平面的高度。
    pub center: Point,
    /// 半径，模型空间单位。
    pub radius: f64,
    /// 起始角，弧度，相对圆心与 X 轴正方向的夹角。
    pub start_angle: f64,
    /// 终止角，弧度；与 `start_angle` 比较时会先归一到 `[0, 2π)`。
    pub end_angle: f64,
    /// 扫掠方向：`true` 表示从起始角逆时针扫向终止角，`false` 表示顺时针。
    pub is_counter_clockwise: bool,
}

impl Arc {
    /// 以圆心、半径与起止角构造圆弧，并自动推断扫掠方向。
    ///
    /// - `center`：圆心。
    /// - `radius`：半径，模型空间单位。
    /// - `start_angle`：起始角，弧度，可为负值。
    /// - `end_angle`：终止角，弧度，可为负值。
    ///
    /// 方向推断规则：取两个角归一到 `[0, 2π)` 后较短的扫掠方向，因此圆心角不超过 π；
    /// 若需要更长的弧（如 `From<Circle>` 生成的整圆）请直接构造结构体字段。
    #[inline]
    pub fn new(center: Point, radius: f64, start_angle: f64, end_angle: f64) -> Self {
        let is_ccw = Arc::calculate_direction(start_angle, end_angle);
        Self {
            center,
            radius,
            start_angle,
            end_angle,
            is_counter_clockwise: is_ccw,
        }
    }

    fn calculate_direction(start: f64, end: f64) -> bool {
        let mut start = start;
        let mut end = end;
        while start < 0.0 { start += 2.0 * std::f64::consts::PI; }
        while end < 0.0 { end += 2.0 * std::f64::consts::PI; }
        
        if end >= start {
            end - start <= std::f64::consts::PI
        } else {
            2.0 * std::f64::consts::PI - start + end <= std::f64::consts::PI
        }
    }

    /// 返回弧长，等于半径乘以圆心角。
    ///
    /// 圆心角由 [`Arc::angle_span`] 按 `is_counter_clockwise` 计算，结果非负。
    #[inline]
    pub fn length(&self) -> f64 {
        self.radius * self.angle_span()
    }

    /// 返回圆心角（扫掠角），单位为弧度，取值属于 `[0, 2π)`。
    ///
    /// 先把起止角归一到 `[0, 2π)`，再沿 `is_counter_clockwise` 指定的方向从起始角扫到终止角，
    /// 因此结果与两个角的书写顺序无关。
    #[inline]
    pub fn angle_span(&self) -> f64 {
        let start = self.normalize_angle(self.start_angle);
        let end = self.normalize_angle(self.end_angle);
        
        if self.is_counter_clockwise {
            if end >= start {
                end - start
            } else {
                2.0 * std::f64::consts::PI - start + end
            }
        } else {
            if end <= start {
                start - end
            } else {
                start + 2.0 * std::f64::consts::PI - end
            }
        }
    }

    /// 把任意角度归一到 `[0, 2π)`，返回左闭右开区间内的弧度值。
    ///
    /// - `angle`：任意弧度值；负数与超过 `2π` 的值都会被等价折算。
    #[inline]
    pub fn normalize_angle(&self, angle: f64) -> f64 {
        let two_pi = 2.0 * std::f64::consts::PI;
        ((angle % two_pi) + two_pi) % two_pi
    }

    /// 返回起点，即 [`Arc::point_at_angle`] 作用于 `start_angle` 的结果。
    #[inline]
    pub fn start_point(&self) -> Point {
        self.point_at_angle(self.start_angle)
    }

    /// 返回终点，即 [`Arc::point_at_angle`] 作用于 `end_angle` 的结果。
    #[inline]
    pub fn end_point(&self) -> Point {
        self.point_at_angle(self.end_angle)
    }

    /// 返回圆（弧所在圆）上指定角度对应的点，不判断该角度是否落在弧的范围内。
    ///
    /// - `angle`：弧度，相对 X 轴正方向、逆时针为正，不做归一化；返回点的 Z 坐标等于圆心 Z 坐标。
    #[inline]
    pub fn point_at_angle(&self, angle: f64) -> Point {
        Point::new(
            self.center.x + self.radius * angle.cos(),
            self.center.y + self.radius * angle.sin(),
            self.center.z,
        )
    }

    /// 返回弧上参数 `t` 处的点，按扫掠方向等角速度取点。
    ///
    /// - `t`：参数值，`0.0` 对应起点、`1.0` 对应终点；不做区间限制，区间外的值会沿该方向外推。
    #[inline]
    pub fn point_at_parameter(&self, t: f64) -> Point {
        let angle_span = self.angle_span();
        let angle = if self.is_counter_clockwise {
            self.start_angle + t * angle_span
        } else {
            self.start_angle - t * angle_span
        };
        self.point_at_angle(angle)
    }

    /// 返回弧的中点，即参数 `t = 0.5` 处的点，按圆心角而非弦长平分。
    #[inline]
    pub fn midpoint(&self) -> Point {
        self.point_at_parameter(0.5)
    }

    /// 判断圆弧是否可用，要求半径大于 `0.0` 且起止角不相等（按浮点值精确比较）。
    ///
    /// `end_angle` 与 `start_angle` 相差整周（如相差 `2π`）时视为起止角不等，判定为有效。
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.radius > 0.0 && self.start_angle != self.end_angle
    }

    /// 返回从圆心指向给定点的弧度角，取值 `(-π, π]`，逆时针为正，不做归一化。
    ///
    /// - `point`：查询点，与圆心重合时返回 `0.0`。
    #[inline]
    pub fn angle_from_center(&self, point: &Point) -> f64 {
        let dx = point.x - self.center.x;
        let dy = point.y - self.center.y;
        dy.atan2(dx)
    }

    /// 由起点、圆心与终点构造圆弧。
    ///
    /// - `p1`：圆弧起点，与 `center` 的距离决定半径。
    /// - `center`：圆心。
    /// - `p2`：圆弧终点。
    /// - `is_ccw`：期望的扫掠方向，`true` 为逆时针；实际写入的 `is_counter_clockwise`
    ///   仍取该方向下不超过 π 的短弧，可能与期望方向取反。
    ///
    /// # 示例
    /// ```
    /// # use cadrs::geometry::{Arc, Point};
    /// let arc = Arc::from_three_points(
    ///     Point::new(1.0, 0.0, 0.0),
    ///     Point::origin(),
    ///     Point::new(0.0, 2.0, 0.0),
    ///     true,
    /// );
    /// assert!((arc.radius() - 1.0).abs() < 1e-10);
    /// ```
    #[inline]
    pub fn from_three_points(p1: Point, center: Point, p2: Point, is_ccw: bool) -> Self {
        let radius = p1.distance_to(&center);
        let start_angle = Arc::calculate_angle_from_points(&center, &p1);
        let end_angle = Arc::calculate_angle_from_points(&center, &p2);
        
        let two_pi = 2.0 * std::f64::consts::PI;
        let mut start = start_angle;
        let mut end = end_angle;
        while start < 0.0 { start += two_pi; }
        while end < 0.0 { end += two_pi; }
        
        let is_counter_clockwise = if is_ccw {
            if end >= start {
                end - start <= std::f64::consts::PI
            } else {
                two_pi - start + end <= std::f64::consts::PI
            }
        } else {
            if end <= start {
                start - end <= std::f64::consts::PI
            } else {
                start + two_pi - end <= std::f64::consts::PI
            }
        };

        Self {
            center,
            radius,
            start_angle,
            end_angle,
            is_counter_clockwise,
        }
    }

    fn calculate_angle_from_points(center: &Point, point: &Point) -> f64 {
        let dx = point.x - center.x;
        let dy = point.y - center.y;
        dy.atan2(dx)
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

    /// 返回起始角，单位为弧度。
    #[inline]
    pub fn start_angle(&self) -> f64 {
        self.start_angle
    }

    /// 返回终止角，单位为弧度。
    #[inline]
    pub fn end_angle(&self) -> f64 {
        self.end_angle
    }
}

impl From<Circle> for Arc {
    fn from(circle: Circle) -> Self {
        Self {
            center: circle.center,
            radius: circle.radius,
            start_angle: 0.0,
            end_angle: 2.0 * std::f64::consts::PI,
            is_counter_clockwise: true,
        }
    }
}

impl fmt::Display for Arc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Arc(center: {}, radius: {}, start: {:.2}°, end: {:.2}°)",
            self.center,
            self.radius,
            self.start_angle.to_degrees(),
            self.end_angle.to_degrees()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arc_creation() {
        let arc = Arc::new(Point::origin(), 5.0, 0.0, std::f64::consts::PI / 2.0);
        
        assert_eq!(arc.radius, 5.0);
        assert!(arc.is_valid());
    }

    #[test]
    fn test_arc_angle_span() {
        let arc = Arc::new(Point::origin(), 5.0, 0.0, std::f64::consts::PI / 2.0);
        
        assert!((arc.angle_span() - std::f64::consts::PI / 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_arc_length() {
        let arc = Arc::new(Point::origin(), 1.0, 0.0, std::f64::consts::PI / 2.0);
        
        assert!((arc.length() - std::f64::consts::PI / 2.0).abs() < 1e-10);
    }
}
