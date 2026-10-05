//! NURBS 曲线模块：定义带权因子的有理 B 样条 [`NURBS`]。
//!
//! 相比 [`crate::geometry::BSpline`]，每个控制点额外带一个权因子：权值不全为 `1.0` 时曲线为
//! 有理形式，可精确表示圆锥曲线（圆、椭圆、双曲线）。参数归一化到 `[0, 1]`；控制点与节点
//! 向量需满足 `knots.len() == control_points.len() + degree + 1`，否则求值可能 panic。

use crate::geometry::Point;
use std::fmt;
use serde::{Serialize, Deserialize};

/// 有理 B 样条曲线，由控制点、权因子、节点向量与次数确定。
///
/// 字段私有，只能通过构造函数写入；对象不校验长度关系，求值前请确认数据自洽。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NURBS {
    control_points: Vec<Point>,
    weights: Vec<f64>,
    knots: Vec<f64>,
    degree: usize,
}

impl NURBS {
    /// 以控制点、权因子、节点向量与次数直接构造曲线，不做任何校验。
    ///
    /// - `control_points`：控制点序列。
    /// - `weights`：与控制点一一对应的权因子，必须为正数；控制点影响力随权值增大而增强。
    /// - `knots`：节点向量，长度应为控制点数加次数加一。
    /// - `degree`：曲线次数。
    ///
    /// 三个序列长度不匹配时对象仍可创建，但 [`NURBS::point_at`] 会越界 panic。
    #[inline]
    pub fn new(control_points: Vec<Point>, weights: Vec<f64>, knots: Vec<f64>, degree: usize) -> Self {
        Self {
            control_points,
            weights,
            knots,
            degree,
        }
    }

    /// 由控制点与次数构造曲线，所有权因子取 `1.0`（退化为普通 B 样条）并自动生成均匀夹紧节点向量。
    ///
    /// - `points`：控制点序列，长度需至少为 2。
    /// - `degree`：曲线次数，需小于控制点数量。
    ///
    /// 控制点少于 2 个时内部计数下溢并 panic；由于权值全为 `1.0`，
    /// [`NURBS::is_rational`] 返回 `false`。
    #[inline]
    pub fn from_points(points: Vec<Point>, degree: usize) -> Self {
        let n = points.len() - 1;
        let weights = vec![1.0; points.len()];
        let mut knots = Vec::with_capacity(n + degree + 2);
        
        for _ in 0..=degree {
            knots.push(0.0);
        }
        for i in 1..=n - degree {
            let t = i as f64 / (n - degree + 1) as f64;
            knots.push(t);
        }
        for _ in 0..=degree {
            knots.push(1.0);
        }

        Self {
            control_points: points,
            weights,
            knots,
            degree,
        }
    }

    /// 按圆周采样控制点构造一条近似整圆的二次 NURBS 曲线。
    ///
    /// - `center`：圆心，采样点与曲线的 Z 坐标均取自该点。
    /// - `radius`：半径，模型空间单位。
    /// - `segments`：采样段数，也是生成的控制点数量；角度按 `2π / segments` 等分，取值过小
    ///   或为 `0` 会导致控制点不足或除零。
    ///
    /// 生成结果只是插值于采样点的近似圆，权因子全部为 `1.0`，并非精确有理圆表示。
    #[inline]
    pub fn with_circle(center: Point, radius: f64, segments: usize) -> Self {
        let mut points = Vec::with_capacity(segments);
        let mut weights = Vec::with_capacity(segments);
        
        for i in 0..segments {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / segments as f64;
            points.push(Point::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
                center.z,
            ));
            weights.push(1.0);
        }
        
        Self::from_points(points, 2)
    }

    /// 返回控制点的只读切片。
    #[inline]
    pub fn control_points(&self) -> &[Point] {
        &self.control_points
    }

    /// 返回权因子的只读切片，与 [`NURBS::control_points`] 一一对应。
    #[inline]
    pub fn weights(&self) -> &[f64] {
        &self.weights
    }

    /// 返回节点向量的只读切片。
    #[inline]
    pub fn knots(&self) -> &[f64] {
        &self.knots
    }

    /// 返回曲线次数。
    #[inline]
    pub fn degree(&self) -> usize {
        self.degree
    }

    /// 判断曲线是否为有理形式，即是否存在与 `1.0` 相差超过 `1e-10` 的权因子。
    ///
    /// 权值全部等于 `1.0` 时返回 `false`，此时曲线等价于普通 B 样条。
    #[inline]
    pub fn is_rational(&self) -> bool {
        self.weights.iter().any(|&w| (w - 1.0).abs() > 1e-10)
    }

    /// 返回曲线在参数 `t` 处的点。
    ///
    /// - `t`：归一化参数，先截断到 `[0, 1]` 再求值；求值按齐次坐标加权平均，当加权和接近
    ///   `0.0`（如权因子全为 `0`）时返回坐标原点而不是 panic。
    ///
    /// # 示例
    /// ```
    /// # use cadrs::geometry::{NURBS, Point};
    /// let nurbs = NURBS::from_points(
    ///     vec![
    ///         Point::new(0.0, 0.0, 0.0),
    ///         Point::new(1.0, 1.0, 0.0),
    ///         Point::new(2.0, 1.0, 0.0),
    ///         Point::new(3.0, 0.0, 0.0),
    ///     ],
    ///     2,
    /// );
    /// assert!(!nurbs.is_rational());
    /// assert!((nurbs.point_at(0.0).x - 0.0).abs() < 1e-10);
    /// ```
    #[inline]
    pub fn point_at(&self, t: f64) -> Point {
        self.evaluate_point(t.clamp(0.0, 1.0))
    }

    fn evaluate_point(&self, t: f64) -> Point {
        let basis = self.compute_rational_basis(t);
        
        let mut wx = 0.0;
        let mut wy = 0.0;
        let mut wz = 0.0;
        let mut w = 0.0;
        
        for (i, &b) in basis.iter().enumerate() {
            if b > 0.0 {
                wx += self.control_points[i].x * self.weights[i] * b;
                wy += self.control_points[i].y * self.weights[i] * b;
                wz += self.control_points[i].z * self.weights[i] * b;
                w += self.weights[i] * b;
            }
        }
        
        if w.abs() < 1e-10 {
            Point::origin()
        } else {
            Point::new(wx / w, wy / w, wz / w)
        }
    }

    fn compute_rational_basis(&self, t: f64) -> Vec<f64> {
        let n = self.control_points.len();
        let mut basis = vec![0.0; n];
        
        let mut ndu = vec![vec![0.0; self.degree + 1]; n];
        ndu[0][0] = if t >= self.knots[0] && t < self.knots[1] { 1.0 } else { 0.0 };

        for j in 1..=self.degree {
            for i in 0..=n - j - 1 {
                let denom1 = self.knots[i + j - 1] - self.knots[i];
                let denom2 = self.knots[i + j] - self.knots[i + 1];
                
                let left = if denom1 != 0.0 {
                    ((t - self.knots[i]) / denom1) * ndu[i][j - 1]
                } else {
                    0.0
                };
                
                let right = if denom2 != 0.0 {
                    ((self.knots[i + j] - t) / denom2) * ndu[i + 1][j - 1]
                } else {
                    0.0
                };
                
                ndu[i][j] = left + right;
            }
        }

        for i in 0..n {
            basis[i] = ndu[i][self.degree];
        }

        basis
    }
}

impl fmt::Display for NURBS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NURBS(degree: {}, control_points: {}, rational: {})",
            self.degree,
            self.control_points.len(),
            self.is_rational()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nurbs_creation() {
        let points = vec![
            Point::new(0.0, 0.0, 0.0),
            Point::new(1.0, 1.0, 0.0),
            Point::new(2.0, 1.0, 0.0),
            Point::new(3.0, 0.0, 0.0),
        ];
        let nurbs = NURBS::from_points(points, 2);
        
        assert_eq!(nurbs.degree(), 2);
    }

    #[test]
    fn test_nurbs_is_rational() {
        let points = vec![
            Point::new(0.0, 0.0, 0.0),
            Point::new(1.0, 1.0, 0.0),
        ];
        let nurbs = NURBS::from_points(points, 1);
        
        assert!(!nurbs.is_rational());
    }

    #[test]
    fn test_nurbs_point_at() {
        let points = vec![
            Point::new(0.0, 0.0, 0.0),
            Point::new(1.0, 1.0, 0.0),
            Point::new(2.0, 1.0, 0.0),
            Point::new(3.0, 0.0, 0.0),
        ];
        let nurbs = NURBS::from_points(points, 2);
        
        let start = nurbs.point_at(0.0);
        assert!((start.x - 0.0).abs() < 1e-10);
    }
}
