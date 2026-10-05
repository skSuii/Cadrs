//! 三维向量：世界空间中的点、方向与位移。
//!
//! [`Vector3`] 与 [`crate::math::Vector2`] 语义一致，只是多出 `z` 分量，用于三维图元、
//! 细分（Tessellation）结果与渲染管线；分量均为模型空间实数单位，运算不修改 `self`。
//! 平面内的工作（Entity 实体、Document 文档、Layer 图层）应优先用二维类型，
//! 只有需要高度或空间法向时才使用本类型。

use std::ops::{Add, Sub, Mul, Div, Index, IndexMut};
use std::fmt;

/// 三维向量 / 空间点，三个分量均为模型空间实数坐标。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3 {
    /// X 分量。
    pub x: f64,
    /// Y 分量。
    pub y: f64,
    /// Z 分量，向上为正。
    pub z: f64,
}

impl Vector3 {
    /// 用三个分量直接构造向量。
    /// - `x`、`y`、`z`：各轴分量。
    #[inline]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// 零向量 (0, 0, 0)，即空间原点。
    #[inline]
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }

    /// X 轴单位向量 (1, 0, 0)。
    #[inline]
    pub fn unit_x() -> Self {
        Self { x: 1.0, y: 0.0, z: 0.0 }
    }

    /// Y 轴单位向量 (0, 1, 0)。
    #[inline]
    pub fn unit_y() -> Self {
        Self { x: 0.0, y: 1.0, z: 0.0 }
    }

    /// Z 轴单位向量 (0, 0, 1)。
    #[inline]
    pub fn unit_z() -> Self {
        Self { x: 0.0, y: 0.0, z: 1.0 }
    }

    /// 欧几里得模长，恒为非负；零向量返回 0.0。
    #[inline]
    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// 模长的平方，省去开方，适合只做长度比较的场合（如公差 Tolerance 判定）。
    #[inline]
    pub fn magnitude_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// 返回同方向的单位向量，不修改原向量；模长为 0 时原样返回零向量。
    #[inline]
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag == 0.0 {
            *self
        } else {
            Self::new(self.x / mag, self.y / mag, self.z / mag)
        }
    }

    /// 点积（内积），为 0 表示两向量垂直。
    /// - `other`：另一个向量。
    #[inline]
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// 叉积，返回同时垂直于 `self` 与 `other` 的向量，方向遵循右手定则。
    /// 两向量平行（含任一为零向量）时返回零向量。
    /// - `other`：另一个向量。
    #[inline]
    pub fn cross(&self, other: &Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    /// 到另一点的距离，恒为非负，不修改任一向量。
    /// - `other`：目标点。
    #[inline]
    pub fn distance_to(&self, other: &Self) -> f64 {
        (*self - *other).magnitude()
    }

    /// 线性插值，`t = 0` 返回 `self`，`t = 1` 返回 `other`；`t` 不裁剪，可外插。
    /// - `other`：另一端向量。
    /// - `t`：插值参数。
    #[inline]
    pub fn lerp(&self, other: &Self, t: f64) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
            self.z + (other.z - self.z) * t,
        )
    }

    /// 展开为 `[x, y, z]` 数组，便于直接传给图形 API 或按分量写入缓冲。
    #[inline]
    pub fn as_array(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
}

impl Add for Vector3 {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub for Vector3 {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Mul<f64> for Vector3 {
    type Output = Self;
    #[inline]
    fn mul(self, scalar: f64) -> Self {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl Div<f64> for Vector3 {
    type Output = Self;
    #[inline]
    fn div(self, scalar: f64) -> Self {
        Self::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}

impl fmt::Display for Vector3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector3_basic_operations() {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(4.0, 5.0, 6.0);

        assert_eq!(v1 + v2, Vector3::new(5.0, 7.0, 9.0));
        assert_eq!(v1 - v2, Vector3::new(-3.0, -3.0, -3.0));
    }

    #[test]
    fn test_vector3_cross_product() {
        let v1 = Vector3::unit_x();
        let v2 = Vector3::unit_y();
        let result = v1.cross(&v2);

        assert_eq!(result, Vector3::unit_z());
    }
}
