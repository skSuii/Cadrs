//! 4×4 矩阵：三维仿射变换与透视投影。
//!
//! 数据按行主序存放，平移分量位于第 3 列（`m[0..3][3]`），第 3 行才是齐次行，
//! 与 [`crate::math::Matrix3`] 的二维写法不同；这是图形管线与 CAD 三维视图沿用的约定。
//! 常见用法是先由 [`Matrix4::from_translation`] / [`Matrix4::rotation_2d`] /
//! [`Matrix4::scale_2d`] 组合出视图矩阵，再用 [`Matrix4::transform_point`] 批量投影点。

use serde::{Serialize, Deserialize};

/// 4×4 实数矩阵，按行主序存储，可表示三维平移、旋转、缩放与透视投影。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Matrix4 {
    /// 行主序的矩阵元素：`m[行][列]`，平移量在第 3 列。
    pub m: [[f64; 4]; 4],
}

impl Matrix4 {
    /// 用 16 个元素按行主序构造矩阵，不做校验。
    /// 参数命名遵循 `m<行><列>`，例如 `m03` 是第 0 行第 3 列（X 方向平移量）。
    pub fn new(
        m00: f64, m01: f64, m02: f64, m03: f64,
        m10: f64, m11: f64, m12: f64, m13: f64,
        m20: f64, m21: f64, m22: f64, m23: f64,
        m30: f64, m31: f64, m32: f64, m33: f64,
    ) -> Self {
        Self {
            m: [
                [m00, m01, m02, m03],
                [m10, m11, m12, m13],
                [m20, m21, m22, m23],
                [m30, m31, m32, m33],
            ],
        }
    }

    /// 单位矩阵，对应恒等变换，也是 [`Matrix4::default`] 的取值。
    pub fn identity() -> Self {
        Self::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        )
    }

    /// 三维平移矩阵，位移写入第 3 列。
    /// - `x`、`y`、`z`：各轴平移量，单位与坐标系一致。
    pub fn from_translation(x: f64, y: f64, z: f64) -> Self {
        Self::new(
            1.0, 0.0, 0.0, x,
            0.0, 1.0, 0.0, y,
            0.0, 0.0, 1.0, z,
            0.0, 0.0, 0.0, 1.0,
        )
    }

    /// 平面平移的便捷写法，Z 方向位移固定为 0。
    /// - `x`、`y`：平面内平移量。
    pub fn translation_2d(x: f64, y: f64) -> Self {
        Self::from_translation(x, y, 0.0)
    }

    /// 绕 Z 轴的旋转矩阵，保持 Z 分量不变。
    /// - `angle`：逆时针旋转角，单位弧度（不是角度），正方向由 Z 轴右手定则决定。
    pub fn rotation_2d(angle: f64) -> Self {
        let (s, c) = angle.sin_cos();
        Self::new(
            c, -s, 0.0, 0.0,
            s, c, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        )
    }

    /// 平面非等比缩放矩阵，Z 方向缩放保持 1.0。
    /// - `sx`、`sy`：X、Y 方向缩放系数；取 0 会把该轴压成零（矩阵奇异），负值表示镜像。
    pub fn scale_2d(sx: f64, sy: f64) -> Self {
        Self::new(
            sx, 0.0, 0.0, 0.0,
            0.0, sy, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        )
    }

    /// 标准矩阵乘法 `self × other`，即先施加 `other` 再施加 `self`。
    /// 乘法不可交换，不修改任一操作数；[`std::ops::Mul`] 是本方法的运算符写法。
    /// - `other`：右乘矩阵。
    pub fn multiply(&self, other: &Matrix4) -> Matrix4 {
        let mut result = [[0.0f64; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += self.m[i][k] * other.m[k][j];
                }
                result[i][j] = sum;
            }
        }
        Self { m: result }
    }

    /// 对点 `(x, y, z)` 施加变换并做透视除法，返回变换后的三维坐标。
    /// 齐次分量 `w` 的绝对值小于 1e-12 时判定为无穷远点，直接返回 `(0.0, 0.0, 0.0)`
    /// 而不是 NaN/Inf，调用方需自行判断该退化结果。
    /// - `x`、`y`、`z`：待变换点的坐标。
    pub fn transform_point(&self, x: f64, y: f64, z: f64) -> (f64, f64, f64) {
        let w = self.m[3][0] * x + self.m[3][1] * y + self.m[3][2] * z + self.m[3][3];
        if w.abs() < 1e-12 {
            return (0.0, 0.0, 0.0);
        }
        (
            (self.m[0][0] * x + self.m[0][1] * y + self.m[0][2] * z + self.m[0][3]) / w,
            (self.m[1][0] * x + self.m[1][1] * y + self.m[1][2] * z + self.m[1][3]) / w,
            (self.m[2][0] * x + self.m[2][1] * y + self.m[2][2] * z + self.m[2][3]) / w,
        )
    }

    /// 按第 0 行做余子式展开求行列式；为 0 表示矩阵奇异，变换不可逆。
    pub fn determinant(&self) -> f64 {
        let m = &self.m;
        let mut det = 0.0;
        for c in 0..4 {
            let mut sub = [[0.0f64; 3]; 3];
            for i in 1..4 {
                let mut sub_col = 0;
                for j in 0..4 {
                    if j == c {
                        continue;
                    }
                    sub[i - 1][sub_col] = m[i][j];
                    sub_col += 1;
                }
            }
            let minor = sub[0][0] * (sub[1][1] * sub[2][2] - sub[1][2] * sub[2][1])
                - sub[0][1] * (sub[1][0] * sub[2][2] - sub[1][2] * sub[2][0])
                + sub[0][2] * (sub[1][0] * sub[2][1] - sub[1][1] * sub[2][0]);
            det += if c % 2 == 0 { 1.0 } else { -1.0 } * m[0][c] * minor;
        }
        det
    }
}

impl Default for Matrix4 {
    fn default() -> Self {
        Self::identity()
    }
}

impl std::ops::Mul for Matrix4 {
    type Output = Matrix4;
    fn mul(self, other: Matrix4) -> Matrix4 {
        self.multiply(&other)
    }
}

impl std::ops::Mul<&Matrix4> for Matrix4 {
    type Output = Matrix4;
    fn mul(self, other: &Matrix4) -> Matrix4 {
        self.multiply(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let m = Matrix4::identity();
        assert_eq!(m, Matrix4::default());
        assert!((m.determinant() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_translation() {
        let m = Matrix4::from_translation(1.0, 2.0, 3.0);
        let (x, y, z) = m.transform_point(0.0, 0.0, 0.0);
        assert!((x - 1.0).abs() < 1e-10);
        assert!((y - 2.0).abs() < 1e-10);
        assert!((z - 3.0).abs() < 1e-10);
    }
}
