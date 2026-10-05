//! 二维变换：以「平移 + 旋转 + 缩放」三要素描述的仿射变换。
//!
//! 与 [`Matrix3`] 不同，[`Transform2D`] 保留可读的分解参数，便于编辑器直接读写、
//! 用 [`Transform2D::compose`] 叠加、用 [`Transform2D::inverse`] 求逆；
//! 需要批量作用或交给渲染时，再用 [`Transform2D::to_matrix`] 转成矩阵。
//! 约定：角度一律为弧度，逆时针为正；作用于点时的顺序是「先按各自轴缩放，再旋转，最后平移」；
//! `apply` / `compose` / `inverse` / `to_matrix` 都不修改 `self`。

use crate::math::Matrix3;
use crate::math::Vector2;
use std::fmt;

/// 二维仿射变换，由平移量、旋转角与各轴缩放系数组成。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform2D {
    /// 平移分量，在旋转与缩放之后叠加，单位为模型空间实数单位。
    pub translation: Vector2,
    /// 旋转角，单位弧度，逆时针为正。
    pub rotation: f64,
    /// X、Y 轴各自的缩放系数，默认 (1.0, 1.0)；取 0 表示把该轴压扁。
    pub scale: Vector2,
}

impl Transform2D {
    /// 恒等变换：无平移、无旋转、缩放为 (1.0, 1.0)。
    /// 与 [`Transform2D::identity`] 等价，同时也是 [`Transform2D::default`] 的取值。
    #[inline]
    pub fn new() -> Self {
        Self {
            translation: Vector2::zero(),
            rotation: 0.0,
            scale: Vector2::new(1.0, 1.0),
        }
    }

    /// [`Transform2D::new`] 的别名，用于强调其恒等语义。
    #[inline]
    pub fn identity() -> Self {
        Self::new()
    }

    /// 构造纯平移变换，旋转为 0、缩放保持 (1.0, 1.0)。
    /// - `x`、`y`：平移量。
    #[inline]
    pub fn from_translation(x: f64, y: f64) -> Self {
        Self {
            translation: Vector2::new(x, y),
            rotation: 0.0,
            scale: Vector2::new(1.0, 1.0),
        }
    }

    /// 构造纯旋转变换，绕原点旋转，不产生平移。
    /// - `angle`：旋转角，单位弧度，逆时针为正。
    #[inline]
    pub fn from_rotation(angle: f64) -> Self {
        Self {
            translation: Vector2::zero(),
            rotation: angle,
            scale: Vector2::new(1.0, 1.0),
        }
    }

    /// 构造纯缩放变换，以原点为基准，不产生平移。
    /// - `sx`、`sy`：X、Y 轴缩放系数，负值等效于镜像。
    #[inline]
    pub fn from_scale(sx: f64, sy: f64) -> Self {
        Self {
            translation: Vector2::zero(),
            rotation: 0.0,
            scale: Vector2::new(sx, sy),
        }
    }

    /// 展开为 3×3 齐次矩阵，最后一行固定为 `[0, 0, 1]`。
    /// 注意矩阵按行主序填写，其第 0、1、2 列分别对应 X、Y 输出与齐次分量；
    /// 不修改 `self`，也不缓存结果，每次调用都会重新计算。
    #[inline]
    pub fn to_matrix(&self) -> Matrix3 {
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        let sx = self.scale.x;
        let sy = self.scale.y;
        let tx = self.translation.x;
        let ty = self.translation.y;

        let data = [
            [sx * cos_r, sx * sin_r, tx],
            [-sy * sin_r, sy * cos_r, ty],
            [0.0, 0.0, 1.0],
        ];

        Matrix3::new(data)
    }

    /// 把变换作用于点：先按各轴缩放，再绕原点旋转，最后叠加平移。
    /// 不修改 `self` 与 `point`；与 [`Transform2D::to_matrix`] 的矩阵写法在
    /// 非等比缩放时并不一致，需要严格复现时请以本方法的实际结果为准。
    /// - `point`：待变换的点（按点处理，平移分量会叠加）。
    #[inline]
    pub fn apply(&self, point: &Vector2) -> Vector2 {
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        
        let scaled = Vector2::new(
            point.x * self.scale.x,
            point.y * self.scale.y,
        );
        
        Vector2::new(
            scaled.x * cos_r - scaled.y * sin_r + self.translation.x,
            scaled.x * sin_r + scaled.y * cos_r + self.translation.y,
        )
    }

    /// 复合变换：旋转角相加、各轴缩放相乘，平移量取 `other` 的平移经 `self` 旋转后的结果。
    /// 注意 `self` 自身的平移不参与复合，因此只有在 `self` 无平移（或调用方自行补平移）时，
    /// 结果才等价于「先施加 `other`、再施加 `self`」；缩放为各轴独立，否则结果只是近似。
    /// 不修改任一操作数。
    /// - `other`：被复合的内层变换。
    #[inline]
    pub fn compose(&self, other: &Self) -> Self {
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        
        let new_scale = Vector2::new(
            self.scale.x * other.scale.x,
            self.scale.y * other.scale.y,
        );
        
        let new_rotation = self.rotation + other.rotation;
        
        let tx = other.translation.x;
        let ty = other.translation.y;
        
        Self {
            translation: Vector2::new(
                tx * cos_r - ty * sin_r,
                tx * sin_r + ty * cos_r,
            ),
            rotation: new_rotation,
            scale: new_scale,
        }
    }
    
    /// 生成一个新变换：平移分量被替换为 `(x, y)`，旋转与缩放保持不变。
    /// 采用移动语义的链式写法，返回新值而非修改原值。
    /// - `x`、`y`：新的平移量，会覆盖原有平移（不是累加）。
    #[inline]
    pub fn translate(mut self, x: f64, y: f64) -> Self {
        self.translation = Vector2::new(x, y);
        self
    }
    
    /// 生成一个新变换：旋转角被替换为 `angle`，平移与缩放保持不变。
    /// - `angle`：新的旋转角，单位弧度，逆时针为正；覆盖原值而非叠加。
    #[inline]
    pub fn rotate(mut self, angle: f64) -> Self {
        self.rotation = angle;
        self
    }
    
    /// 生成一个新变换：缩放系数被替换为 `(sx, sy)`，平移与旋转保持不变。
    /// - `sx`、`sy`：新的各轴缩放系数，覆盖原值而非累乘。
    #[inline]
    pub fn scale(mut self, sx: f64, sy: f64) -> Self {
        self.scale = Vector2::new(sx, sy);
        self
    }
    
    /// 逆变换：旋转角取反、缩放系数取各自倒数，并反解平移量以抵消原平移与旋转。
    /// 缩放为 (1, 1)（纯平移、纯旋转或二者复合）时精确可逆；
    /// 非等比或非单位缩放下的平移分量只是近似反解，回代后与原点存在偏差。
    /// 某轴缩放绝对值不超过 1e-10（含 0）时无法求倒数，该轴逆缩放记作 0，结果退化，
    /// 调用方需自行规避奇异变换。不修改 `self`。
    #[inline]
    pub fn inverse(&self) -> Self {
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        
        let inv_scale_x = if self.scale.x.abs() > 1e-10 { 1.0 / self.scale.x } else { 0.0 };
        let inv_scale_y = if self.scale.y.abs() > 1e-10 { 1.0 / self.scale.y } else { 0.0 };
        
        let inv_rot_cos = cos_r;
        let inv_rot_sin = -sin_r;
        
        let inv_translation = Vector2::new(
            -self.translation.x * inv_scale_x * inv_rot_cos - self.translation.y * inv_scale_y * inv_rot_sin,
            self.translation.x * inv_scale_x * inv_rot_sin - self.translation.y * inv_scale_y * inv_rot_cos,
        );
        
        Self {
            translation: inv_translation,
            rotation: -self.rotation,
            scale: Vector2::new(inv_scale_x, inv_scale_y),
        }
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Transform2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Transform2D(translation: {}, rotation: {}, scale: {})", 
               self.translation, self.rotation, self.scale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform2d_identity() {
        let t = Transform2D::identity();
        let p = Vector2::new(1.0, 2.0);
        assert_eq!(t.apply(&p), p);
    }

    #[test]
    fn test_transform2d_translation() {
        let t = Transform2D::from_translation(1.0, 2.0);
        let p = Vector2::new(0.0, 0.0);
        assert_eq!(t.apply(&p), Vector2::new(1.0, 2.0));
    }
}
