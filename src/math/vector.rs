//! 二维向量：SDK 的基础向量类型，承载平面坐标、方向与位移。
//!
//! [`Vector2`] 既表示点（相对原点）也表示方向（相对零点），所有分量均为模型空间的
//! 实数单位，角度一律用弧度；向量运算不修改 `self`，均返回新值。
//! 上层 Entity 实体的位置、Snap 捕捉点、Viewport 视口参数与几何图元都复用本类型，
//! 齐次坐标下的仿射变换见 [`crate::math::Matrix3`] 与 [`crate::math::Transform2D`]。

use std::ops::{Add, Sub, Mul, Div, Neg};

/// 二维向量 / 平面点，分量 `x`、`y` 为模型空间实数坐标。
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Vector2 {
    /// 水平分量；作为点时是 X 坐标。
    pub x: f64,
    /// 垂直分量；作为点时是 Y 坐标，向上为正。
    pub y: f64,
}

impl Vector2 {
    /// 用两个分量直接构造向量。
    /// - `x`：水平分量。
    /// - `y`：垂直分量。
    #[inline]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// 零向量 (0, 0)，即坐标原点，也用作方向未知时的占位值。
    #[inline]
    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }

    /// 读取水平分量（按值返回，不改动向量）。
    #[inline]
    pub fn x(&self) -> f64 {
        self.x
    }

    /// 读取垂直分量（按值返回，不改动向量）。
    #[inline]
    pub fn y(&self) -> f64 {
        self.y
    }

    /// 欧几里得模长（向量长度），恒为非负；零向量返回 0.0。
    #[inline]
    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// 返回同方向的单位向量，不修改原向量。
    /// 零向量（模长为 0）无法定向，此时返回零向量而不是 NaN。
    #[inline]
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self::new(self.x / len, self.y / len)
        } else {
            Self::zero()
        }
    }
    
    /// 从 `self` 转到 `other` 的有向角，单位弧度，逆时针为正。
    /// 返回值落在 (-π, π]；两向量任一为零向量时结果无意义。
    /// - `other`：终止方向。
    #[inline]
    pub fn angle_to(&self, other: &Self) -> f64 {
        let dot = self.x * other.x + self.y * other.y;
        let det = self.x * other.y - self.y * other.x;
        det.atan2(dot)
    }
    
    /// [`Vector2::length`] 的别名，便于与 `Point::magnitude` 等接口保持一致。
    #[inline]
    pub fn magnitude(&self) -> f64 {
        self.length()
    }
    
    /// 点积（内积），等于两向量模长与夹角余弦之积；可用于夹角与投影计算。
    /// - `other`：另一个向量。
    #[inline]
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y
    }
    
    /// [`Vector2::normalize`] 的别名；零向量同样返回零向量。
    #[inline]
    pub fn normalized(&self) -> Self {
        self.normalize()
    }
    
    /// 向量加法，等价于 `*self + *other`，不修改 `self`。
    /// - `other`：被加向量。
    #[inline]
    pub fn add(&self, other: &Self) -> Self {
        *self + *other
    }
    
    /// 向量减法 `self - other`，结果为由 `other` 指向 `self` 的位移。
    /// - `other`：减数向量。
    #[inline]
    pub fn sub(&self, other: &Self) -> Self {
        *self - *other
    }
    
    /// 按标量等比缩放，等价于 `*self * scalar`。
    /// - `scalar`：缩放系数，负数会同时反向。
    #[inline]
    pub fn scale(&self, scalar: f64) -> Self {
        *self * scalar
    }
    
    /// 线性插值，`t = 0` 返回 `self`，`t = 1` 返回 `other`；`t` 不做区间裁剪，可外插。
    /// - `other`：另一端向量。
    /// - `t`：插值参数。
    #[inline]
    pub fn lerp(&self, other: &Self, t: f64) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }

    /// 向量相对 X 轴正向的极角，单位弧度，逆时针为正。
    /// 返回值落在 (-π, π]；零向量返回 0.0。
    /// # 示例
    /// ```
    /// let v = cadrs::math::Vector2::new(0.0, 1.0);
    /// assert!((v.angle() - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
    /// ```
    #[inline]
    pub fn angle(&self) -> f64 {
        self.y.atan2(self.x)
    }
}

impl Add for Vector2 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Vector2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl Mul<f64> for Vector2 {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self {
        Self::new(self.x * scalar, self.y * scalar)
    }
}

impl Div<f64> for Vector2 {
    type Output = Self;
    fn div(self, scalar: f64) -> Self {
        Self::new(self.x / scalar, self.y / scalar)
    }
}

impl Neg for Vector2 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

impl std::fmt::Display for Vector2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vector2({:.6}, {:.6})", self.x, self.y)
    }
}
