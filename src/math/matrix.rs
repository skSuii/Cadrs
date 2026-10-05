//! 3×3 矩阵：二维仿射变换与齐次坐标下的空间算子。
//!
//! 数据按行主序存放，第 0、1 行分别给出 X、Y 的输出系数，其第 2 列存放平移量，
//! 第 2 行固定为齐次分量 `[0, 0, 1]`。
//! [`Matrix3`] 多用于把 [`crate::math::Transform2D`] 的平移 / 旋转 / 缩放复合为单个矩阵，
//! 再一次性作用于大量点（渲染、捕捉与几何求交）；矩阵本身不携带单位信息，
//! 其平移分量沿用调用方坐标系的实数单位。

use std::ops::{Add, Sub, Mul, Index, IndexMut};
use std::fmt;

/// 3×3 实数矩阵，按行主序存储，可表示二维仿射变换（含平移）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix3 {
    data: [[f64; 3]; 3],
}

impl Matrix3 {
    /// 用行主序的二维数组直接构造矩阵，不做任何校验或归一化。
    /// - `data`：`data[行][列]`，共 3 行 3 列。
    #[inline]
    pub fn new(data: [[f64; 3]; 3]) -> Self {
        Self { data }
    }

    /// 单位矩阵，作为矩阵乘法的不动元，对应恒等变换。
    #[inline]
    pub fn identity() -> Self {
        let mut data = [[0.0; 3]; 3];
        data[0][0] = 1.0;
        data[1][1] = 1.0;
        data[2][2] = 1.0;
        Self { data }
    }

    /// 全零矩阵，对应把任意点映射到原点的退化变换（行列式为 0，不可逆）。
    #[inline]
    pub fn zero() -> Self {
        Self { data: [[0.0; 3]; 3] }
    }

    /// 读取指定位置的元素，不做边界检查，越界会 panic。
    /// - `row`：行下标，0 起。
    /// - `col`：列下标，0 起。
    #[inline]
    pub fn get(&self, row: usize, col: usize) -> f64 {
        self.data[row][col]
    }

    /// 就地写入指定位置的元素，会修改 `self`；越界会 panic。
    /// - `row`：行下标，0 起。
    /// - `col`：列下标，0 起。
    /// - `value`：新值。
    #[inline]
    pub fn set(&mut self, row: usize, col: usize, value: f64) {
        self.data[row][col] = value;
    }

    /// 3×3 行列式；为 0 表示矩阵奇异，变换把平面压扁到直线或点，不存在逆矩阵。
    #[inline]
    pub fn determinant(&self) -> f64 {
        let a = self.data[0][0];
        let b = self.data[0][1];
        let c = self.data[0][2];
        let d = self.data[1][0];
        let e = self.data[1][1];
        let f = self.data[1][2];
        let g = self.data[2][0];
        let h = self.data[2][1];
        let i = self.data[2][2];

        a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g)
    }

    /// 转置矩阵：行列互换，不修改 `self`。
    /// 旋转矩阵转置即反向旋转；含平移的行主序矩阵转置后平移量会落到最后一列。
    #[inline]
    pub fn transpose(&self) -> Self {
        let mut result = Self::zero();
        for i in 0..3 {
            for j in 0..3 {
                result.data[i][j] = self.data[j][i];
            }
        }
        result
    }

    /// 标准矩阵乘法 `self × other`，即先施加 `other` 再施加 `self`。
    /// 乘法不可交换，参数顺序会影响结果；不修改任一操作数。
    /// - `other`：右乘矩阵。
    #[inline]
    pub fn multiply(&self, other: &Self) -> Self {
        let mut result = Self::zero();
        for i in 0..3 {
            for j in 0..3 {
                let mut sum = 0.0;
                for k in 0..3 {
                    sum = sum + self.data[i][k] * other.data[k][j];
                }
                result.data[i][j] = sum;
            }
        }
        result
    }

    /// 把二维点视为齐次坐标 `(x, y, 1)` 做仿射变换，第 2 行参与平移计算。
    /// 不修改 `v`，也不做透视除法；调用方应保证第 2 行为 `[0, 0, 1]`。
    /// - `v`：待变换的点或向量，按点的语义处理（会叠加平移分量）。
    #[inline]
    pub fn multiply_vector(&self, v: &super::Vector2) -> super::Vector2 {
        let x = self.data[0][0] * v.x + self.data[0][1] * v.y + self.data[0][2];
        let y = self.data[1][0] * v.x + self.data[1][1] * v.y + self.data[1][2];
        super::Vector2::new(x, y)
    }
}

impl Add for Matrix3 {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
        let mut data = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                data[i][j] = self.data[i][j] + other.data[i][j];
            }
        }
        Self { data }
    }
}

impl Sub for Matrix3 {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
        let mut data = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                data[i][j] = self.data[i][j] - other.data[i][j];
            }
        }
        Self { data }
    }
}

impl Mul<f64> for Matrix3 {
    type Output = Self;
    #[inline]
    fn mul(self, scalar: f64) -> Self {
        let mut data = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                data[i][j] = self.data[i][j] * scalar;
            }
        }
        Self { data }
    }
}

impl Index<(usize, usize)> for Matrix3 {
    type Output = f64;
    #[inline]
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.data[index.0][index.1]
    }
}

impl IndexMut<(usize, usize)> for Matrix3 {
    #[inline]
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.data[index.0][index.1]
    }
}

impl fmt::Display for Matrix3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[{}, {}, {}]", self.data[0][0], self.data[0][1], self.data[0][2])?;
        writeln!(f, "[{}, {}, {}]", self.data[1][0], self.data[1][1], self.data[1][2])?;
        write!(f, "[{}, {}, {}]", self.data[2][0], self.data[2][1], self.data[2][2])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix3_identity() {
        let identity = Matrix3::identity();
        assert_eq!(identity.get(0, 0), 1.0);
        assert_eq!(identity.get(1, 1), 1.0);
        assert_eq!(identity.get(2, 2), 1.0);
    }

    #[test]
    fn test_matrix3_determinant() {
        let m = Matrix3::identity();
        assert_eq!(m.determinant(), 1.0);
    }
}
