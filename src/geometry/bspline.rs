//! B 样条曲线模块：定义由控制点、节点向量与次数描述的多项式 B 样条 [`BSpline`]。
//!
//! 曲线参数归一化到 `[0, 1]`，`point_at(0.0)` 与 `point_at(1.0)` 为首末端点；控制点只起
//! 牵引作用，除端点外曲线一般不过控制点。合法对象需满足 `knots.len() == control_points.len()
//! + degree + 1`，越界访问会导致 panic，因此调用求值方法前应先用 [`BSpline::is_valid`] 校验。
//! 需要权因子（例如精确表示圆）时请使用 [`crate::geometry::NURBS`]。

use crate::geometry::Point;
use std::fmt;
use serde::{Serialize, Deserialize};

/// 多项式 B 样条曲线，由控制点、节点向量与次数确定。
///
/// 字段私有，只能通过构造函数或 [`BSpline::control_points_mut`] 修改；曲线不携带 Z 向特殊
/// 语义，三个坐标分量一视同仁。所有求值方法都不修改自身。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BSpline {
    control_points: Vec<Point>,
    knots: Vec<f64>,
    degree: usize,
}

impl BSpline {
    /// 以给定的控制点、节点向量与次数直接构造曲线，不做任何校验。
    ///
    /// - `control_points`：控制点序列，长度至少为 2 才有几何意义。
    /// - `knots`：节点向量，必须非递减，且长度为控制点数加次数加一。
    /// - `degree`：曲线次数（阶数减一），常用的三次曲线为 `3`。
    ///
    /// 参数不满足上述不变式时对象仍可创建，但 [`BSpline::is_valid`] 返回 `false`，
    /// 调用 [`BSpline::point_at`] 可能 panic。
    #[inline]
    pub fn new(control_points: Vec<Point>, knots: Vec<f64>, degree: usize) -> Self {
        Self {
            control_points,
            knots,
            degree,
        }
    }

    /// 由控制点与次数构造曲线，自动生成均匀夹紧（clamped）节点向量。
    ///
    /// - `points`：控制点序列，`0` 号与末号控制点即曲线端点。
    /// - `degree`：曲线次数，需小于控制点数量，否则内部长度计算下溢并 panic。
    ///
    /// 节点向量两端各重复 `degree + 1` 次 `0.0` 与 `1.0`，中间为均匀分布，
    /// 因此生成的对象满足 [`BSpline::is_valid`]。
    ///
    /// # 示例
    /// ```
    /// # use cadrs::geometry::{BSpline, Point};
    /// let spline = BSpline::from_points(
    ///     vec![
    ///         Point::new(0.0, 0.0, 0.0),
    ///         Point::new(1.0, 1.0, 0.0),
    ///         Point::new(2.0, 1.0, 0.0),
    ///         Point::new(3.0, 0.0, 0.0),
    ///     ],
    ///     2,
    /// );
    /// assert!(spline.is_valid());
    /// ```
    #[inline]
    pub fn from_points(points: Vec<Point>, degree: usize) -> Self {
        let m = points.len();
        let mut knots = Vec::with_capacity(m + degree + 1);

        for _ in 0..=degree {
            knots.push(0.0);
        }
        for i in 1..(m - degree) {
            knots.push(i as f64 / (m - degree) as f64);
        }
        for _ in 0..=degree {
            knots.push(1.0);
        }

        Self {
            control_points: points,
            knots,
            degree,
        }
    }

    /// 返回控制点的只读切片，长度即控制点数量。
    #[inline]
    pub fn control_points(&self) -> &[Point] {
        &self.control_points
    }

    /// 返回控制点的可变切片，可就地拖动控制点以重塑曲线。
    ///
    /// 返回的切片长度不可改变；节点向量不会随之更新，调用者需自行保证节点与控制点数量的
    /// 匹配关系仍然成立。
    #[inline]
    pub fn control_points_mut(&mut self) -> &mut [Point] {
        &mut self.control_points
    }

    /// 返回节点向量的只读切片，长度应为控制点数加次数加一。
    #[inline]
    pub fn knots(&self) -> &[f64] {
        &self.knots
    }

    /// 返回曲线次数，即多项式最高幂次。
    #[inline]
    pub fn degree(&self) -> usize {
        self.degree
    }

    /// 返回曲线阶数，等于次数加一（`degree + 1`）。
    #[inline]
    pub fn order(&self) -> usize {
        self.degree + 1
    }

    /// 判断曲线数据是否自洽。
    ///
    /// 要求控制点不少于 2 个，且节点向量长度恰好等于 `control_points.len() + degree + 1`；
    /// 不检查节点是否非递减，也不检查控制点坐标是否有限。
    #[inline]
    pub fn is_valid(&self) -> bool {
        let n = self.control_points.len();
        if n < 2 {
            return false;
        }
        let expected_knots = n + self.degree + 1;
        self.knots.len() == expected_knots
    }

    /// 返回曲线在参数 `t` 处的点。
    ///
    /// - `t`：归一化参数，内部截断到 `[0, 1]`；`t >= 1.0` 直接返回末控制点，因此曲线右端点
    ///   总能取到，但该实现为端点近似而非严格求值。
    #[inline]
    pub fn point_at(&self, t: f64) -> Point {
        self.evaluate_point(t)
    }

    #[inline]
    fn evaluate_point(&self, t: f64) -> Point {
        let t = t.clamp(0.0, 1.0);
        if t >= 1.0 {
            return *self.control_points.last().unwrap_or(&Point::origin());
        }
        let basis = self.compute_basis_functions(t);

        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;

        for (i, &b) in basis.iter().enumerate() {
            if b > 0.0 {
                x += self.control_points[i].x * b;
                y += self.control_points[i].y * b;
                z += self.control_points[i].z * b;
            }
        }

        Point::new(x, y, z)
    }

    fn compute_basis_functions(&self, t: f64) -> Vec<f64> {
        let n = self.control_points.len();
        let p = self.degree;
        let mut basis = vec![0.0; n];

        if p == 0 {
            for i in 0..n {
                if t >= self.knots[i] && t < self.knots[i + 1] {
                    basis[i] = 1.0;
                }
            }
            return basis;
        }

        let mut ndu = vec![vec![0.0; p + 1]; n + 1];
        for i in 0..=n {
            ndu[i][0] = if t >= self.knots[i] && t < self.knots[i + 1] { 1.0 } else { 0.0 };
        }

        for j in 1..=p {
            for i in 0..n {
                let mut saved = 0.0;
                let denom1 = self.knots[i + j] - self.knots[i];
                let denom2 = self.knots[i + j + 1] - self.knots[i + 1];

                if denom1 != 0.0 {
                    saved = ((t - self.knots[i]) / denom1) * ndu[i][j - 1];
                }
                if denom2 != 0.0 {
                    ndu[i][j] = ((self.knots[i + j + 1] - t) / denom2) * ndu[i + 1][j - 1] + saved;
                } else {
                    ndu[i][j] = saved;
                }
            }
        }

        for i in 0..n {
            basis[i] = ndu[i][p];
        }

        basis
    }

    /// 返回曲线的一阶导数近似向量，用于估算切线方向。
    ///
    /// - `_t`：参数占位，当前实现忽略该值，结果对整条曲线恒定。
    ///
    /// 计算方式为各相邻控制点差向量按 `degree / (knots[i + degree + 1] - knots[i + 1])` 加权
    /// 求和，只反映控制多边形的整体走向；次数为 `0` 时返回原点，节点间距为 `0` 时结果为
    /// `NaN` 或无穷。
    #[inline]
    pub fn derivative(&self, _t: f64) -> Point {
        if self.degree == 0 {
            return Point::origin();
        }

        let mut dx = 0.0;
        let mut dy = 0.0;
        let mut dz = 0.0;
        
        for i in 0..self.control_points.len() - 1 {
            let factor = self.degree as f64 / (self.knots[i + self.degree + 1] - self.knots[i + 1]);
            let point_diff = Point::new(
                self.control_points[i + 1].x - self.control_points[i].x,
                self.control_points[i + 1].y - self.control_points[i].y,
                self.control_points[i + 1].z - self.control_points[i].z,
            );
            dx += point_diff.x * factor;
            dy += point_diff.y * factor;
            dz += point_diff.z * factor;
        }
        
        Point::new(dx, dy, dz)
    }
}

impl fmt::Display for BSpline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BSpline(degree: {}, control_points: {}, knots: {})",
            self.degree,
            self.control_points.len(),
            self.knots.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bspline_creation() {
        let points = vec![
            Point::new(0.0, 0.0, 0.0),
            Point::new(1.0, 1.0, 0.0),
            Point::new(2.0, 1.0, 0.0),
            Point::new(3.0, 0.0, 0.0),
        ];
        let spline = BSpline::from_points(points, 2);
        
        assert_eq!(spline.degree(), 2);
        assert!(spline.is_valid());
    }

    #[test]
    fn test_bspline_point_at() {
        let points = vec![
            Point::new(0.0, 0.0, 0.0),
            Point::new(1.0, 1.0, 0.0),
            Point::new(2.0, 1.0, 0.0),
            Point::new(3.0, 0.0, 0.0),
        ];
        let spline = BSpline::from_points(points, 2);
        
        let start = spline.point_at(0.0);
        assert!((start.x - 0.0).abs() < 1e-10);
        
        let end = spline.point_at(1.0);
        assert!((end.x - 3.0).abs() < 1e-10);
    }
}
