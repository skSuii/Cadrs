//! 点图元模块：定义三维坐标点 [`Point`] 及其向量式运算。
//!
//! `Point` 是全部几何图元的坐标载体，坐标以模型空间的实数单位表示，不做量纲换算；
//! 仅涉及平面几何的方法（如 [`Point::angle_to`]、[`Point::subtract`]）只使用 `x`/`y`，
//! 结果落在 XY 平面内，`z` 视方法而定保留或归零。

use crate::math::Vector2;
use std::fmt;
use serde::{Serialize, Deserialize};

/// 三维坐标点，同时用于表示位置与位移向量。
///
/// 坐标单位为模型空间单位，角度单位为弧度；类型为 `Copy`，所有运算方法均返回新值，不修改 `self`。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    /// X 坐标，模型空间单位。
    pub x: f64,
    /// Y 坐标，模型空间单位。
    pub y: f64,
    /// Z 坐标，模型空间单位；二维构造出的点恒为 `0.0`。
    pub z: f64,
}

impl Point {
    /// 以给定的三个坐标构造点。
    ///
    /// - `x`：X 坐标。
    /// - `y`：Y 坐标。
    /// - `z`：Z 坐标，平面几何可传 `0.0`。
    #[inline]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            x,
            y,
            z,
        }
    }

    /// 以给定的平面坐标构造点，Z 坐标固定为 `0.0`。
    ///
    /// - `x`：X 坐标。
    /// - `y`：Y 坐标。
    #[inline]
    pub fn new2d(x: f64, y: f64) -> Self {
        Self::new(x, y, 0.0)
    }

    /// 返回坐标原点 `(0.0, 0.0, 0.0)`。
    #[inline]
    pub fn origin() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// 由二维向量构造点，Z 坐标置为 `0.0`。
    ///
    /// - `v`：源向量，借用读取，不发生所有权转移。
    #[inline]
    pub fn from_vector(v: &Vector2) -> Self {
        Self::new(v.x, v.y, 0.0)
    }

    /// 返回 X 坐标。
    #[inline]
    pub fn x(&self) -> f64 {
        self.x
    }

    /// 返回 Y 坐标。
    #[inline]
    pub fn y(&self) -> f64 {
        self.y
    }

    /// 返回 Z 坐标。
    #[inline]
    pub fn z(&self) -> f64 {
        self.z
    }

    /// 返回两点之间的三维欧氏距离，包含 Z 方向分量。
    ///
    /// - `other`：目标点。
    #[inline]
    pub fn distance_to(&self, other: &Self) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2)).sqrt()
    }
    
    /// 丢弃 Z 坐标，返回由 `x`/`y` 组成的二维向量。
    #[inline]
    pub fn to_vector2(&self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }
    
    /// 返回把该点当作向量时的三维模长 `sqrt(x² + y² + z²)`。
    #[inline]
    pub fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }
    
    /// 返回两点（向量）的三维点积。
    ///
    /// - `other`：参与点积的另一个点。
    #[inline]
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    
    /// 返回二维叉积 `x * other.y - y * other.x`，即 Z 分量。
    ///
    /// 结果为正表示 `other` 在该点逆时针方向，可用作共线/转向判断；Z 坐标不参与计算。
    ///
    /// - `other`：参与叉积的另一个点。
    #[inline]
    pub fn cross(&self, other: &Self) -> f64 {
        self.x * other.y - self.y * other.x
    }
    
    /// 返回 `self - other` 的二维向量差，Z 分量被丢弃。
    ///
    /// - `other`：被减去的点。
    #[inline]
    pub fn subtract(&self, other: &Self) -> Vector2 {
        Vector2::new(self.x - other.x, self.y - other.y)
    }
    
    /// 返回平移后的新点，Z 坐标保持不变。
    ///
    /// - `v`：平移向量，仅作用于 `x`/`y`。
    #[inline]
    pub fn add_vector(&self, v: &Vector2) -> Point {
        Point::new(self.x + v.x, self.y + v.y, self.z)
    }
    
    /// 返回以原点为中心缩放后的新点，三个坐标同时乘以 `s`。
    ///
    /// - `s`：缩放系数，负数会得到关于原点的对称点。
    #[inline]
    pub fn scale(&self, s: f64) -> Point {
        Point::new(self.x * s, self.y * s, self.z * s)
    }

    /// 返回两点连线的中点，三个坐标分量分别取平均。
    ///
    /// - `other`：另一个端点。
    #[inline]
    pub fn midpoint(&self, other: &Point) -> Point {
        Point::new(
            (self.x + other.x) / 2.0,
            (self.y + other.y) / 2.0,
            (self.z + other.z) / 2.0,
        )
    }

    /// 返回从该点指向 `other` 的方位角，单位为弧度，取值范围 `(-π, π]`。
    ///
    /// 以 X 轴正方向为 0、逆时针为正；仅使用 `x`/`y`，Z 坐标不影响结果。
    ///
    /// - `other`：目标点，与该点重合时返回 `atan2(0.0, 0.0)` 即 `0.0`。
    #[inline]
    pub fn angle_to(&self, other: &Point) -> f64 {
        (other.y - self.y).atan2(other.x - self.x)
    }
}

impl std::ops::Add for Point {
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point::new(self.x + other.x, self.y + other.y, self.z)
    }
}

impl std::ops::Sub for Point {
    type Output = Point;
    fn sub(self, other: Point) -> Point {
        Point::new(self.x - other.x, self.y - other.y, self.z)
    }
}

impl std::ops::Mul<f64> for Point {
    type Output = Point;
    fn mul(self, scalar: f64) -> Point {
        Point::new(self.x * scalar, self.y * scalar, self.z)
    }
}

impl std::ops::Neg for Point {
    type Output = Point;
    fn neg(self) -> Point {
        Point::new(-self.x, -self.y, -self.z)
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Point({:.6}, {:.6}, {:.6})", self.x, self.y, self.z)
    }
}

impl std::ops::Div<f64> for Point {
    type Output = Self;
    fn div(self, scalar: f64) -> Self {
        if scalar.abs() < 1e-10 {
            self
        } else {
            Point::new(self.x / scalar, self.y / scalar, self.z / scalar)
        }
    }
}
