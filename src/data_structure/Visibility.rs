//! 可见性、变换与文字/填充辅助类型。
//!
//! 本文件提供 [`Visibility`]（实体显示状态）、[`Transform`]（4×4 矩阵形式的位姿，与
//! `entity.rs` 中的轻量 `Transform` 不同）以及填充边界（[`HatchBoundary`]、[`BoundaryType`]、
//! [`HatchEdge`]、[`EdgeType`]）、标注类型 [`DimensionType`] 与文字排版
//! （[`TextStyle`]、[`TextAlignment`]）等定义，供实体属性与 IO 层复用。
//! 旋转角一律使用弧度，长度单位为文档单位。

use serde::{Serialize, Deserialize};
use std::fmt;

/// 实体的显示状态。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Visibility {
    /// 正常显示。
    Visible,
    /// 隐藏：不显示，但仍可参与编辑与选择。
    Hidden,
    /// 冻结：不显示且不参与重生成等计算。
    Frozen,
    /// 解冻：从冻结状态恢复为可重新生成，本身不表示是否显示。
    Thawed,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Visible
    }
}

impl Visibility {
    /// 是否处于正常显示状态；仅 [`Visibility::Visible`] 为真，`Thawed` 不算可见。
    pub fn is_visible(&self) -> bool {
        matches!(self, Visibility::Visible)
    }

    /// 是否处于隐藏状态；仅 [`Visibility::Hidden`] 为真。
    pub fn is_hidden(&self) -> bool {
        matches!(self, Visibility::Hidden)
    }

    /// 是否处于冻结状态；仅 [`Visibility::Frozen`] 为真。
    pub fn is_frozen(&self) -> bool {
        matches!(self, Visibility::Frozen)
    }
}

/// 三维仿射变换：4×4 齐次矩阵加可读的平移/旋转/缩放分量。
///
/// `matrix` 与各分量字段不自动同步——[`Transform::translation`] 与 [`Transform::scale`]
/// 会同时写入两边，[`Transform::rotation`] 只写分量、不改矩阵。手工改动字段时需自行保持一致。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    /// 行主序的 4×4 齐次变换矩阵。
    pub matrix: [[f64; 4]; 4],
    /// X 方向平移量。
    pub translation_x: f64,
    /// Y 方向平移量。
    pub translation_y: f64,
    /// Z 方向平移量。
    pub translation_z: f64,
    /// 绕 X 轴旋转角，单位为弧度。
    pub rotation_x: f64,
    /// 绕 Y 轴旋转角，单位为弧度。
    pub rotation_y: f64,
    /// 绕 Z 轴旋转角，单位为弧度。
    pub rotation_z: f64,
    /// X 方向缩放系数，1.0 为不缩放。
    pub scale_x: f64,
    /// Y 方向缩放系数，1.0 为不缩放。
    pub scale_y: f64,
    /// Z 方向缩放系数，1.0 为不缩放。
    pub scale_z: f64,
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl Transform {
    /// 单位变换：矩阵为单位阵，平移与旋转为 0，缩放为 1，表示不做任何变换。
    pub fn identity() -> Self {
        Self {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            translation_x: 0.0,
            translation_y: 0.0,
            translation_z: 0.0,
            rotation_x: 0.0,
            rotation_y: 0.0,
            rotation_z: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            scale_z: 1.0,
        }
    }

    /// 构造纯平移变换：矩阵第 4 列的平移分量与分量字段同时被写入。
    ///
    /// - `tx`：X 方向平移量；
    /// - `ty`：Y 方向平移量；
    /// - `tz`：Z 方向平移量。
    pub fn translation(tx: f64, ty: f64, tz: f64) -> Self {
        let mut transform = Self::identity();
        transform.translation_x = tx;
        transform.translation_y = ty;
        transform.translation_z = tz;

        transform.matrix[0][3] = tx;
        transform.matrix[1][3] = ty;
        transform.matrix[2][3] = tz;

        transform
    }

    /// 构造纯旋转变换：只记录三个旋转角，`matrix` 保持单位阵。
    ///
    /// - `rx`：绕 X 轴旋转角，弧度；
    /// - `ry`：绕 Y 轴旋转角，弧度；
    /// - `rz`：绕 Z 轴旋转角，弧度。
    pub fn rotation(rx: f64, ry: f64, rz: f64) -> Self {
        let mut transform = Self::identity();
        transform.rotation_x = rx;
        transform.rotation_y = ry;
        transform.rotation_z = rz;
        transform
    }

    /// 构造纯缩放变换：矩阵对角线元素与分量字段同时被写入。
    ///
    /// - `sx`：X 方向缩放系数；
    /// - `sy`：Y 方向缩放系数；
    /// - `sz`：Z 方向缩放系数。
    pub fn scale(sx: f64, sy: f64, sz: f64) -> Self {
        let mut transform = Self::identity();
        transform.scale_x = sx;
        transform.scale_y = sy;
        transform.scale_z = sz;

        transform.matrix[0][0] = sx;
        transform.matrix[1][1] = sy;
        transform.matrix[2][2] = sz;

        transform
    }

    /// 是否为单位变换，判断依据是各分量字段（平移全 0、旋转全 0、缩放全 1），不检查 `matrix`。
    pub fn is_identity(&self) -> bool {
        self.translation_x == 0.0 && self.translation_y == 0.0 && self.translation_z == 0.0 &&
        self.rotation_x == 0.0 && self.rotation_y == 0.0 && self.rotation_z == 0.0 &&
        self.scale_x == 1.0 && self.scale_y == 1.0 && self.scale_z == 1.0
    }
}

impl fmt::Display for Transform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Transform(trans=({}, {}, {}), rot=({}, {}, {}), scale=({}, {}, {}))",
            self.translation_x, self.translation_y, self.translation_z,
            self.rotation_x, self.rotation_y, self.rotation_z,
            self.scale_x, self.scale_y, self.scale_z
        )
    }
}

/// 填充边界的类型标记。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HatchBoundary {
    /// 外部边界：最外围的填充范围。
    External,
    /// 外边界：位于外边界内侧的平行边界。
    Outer,
    /// 内部边界：填充区域中的孔洞。
    Inner,
}

/// 边界路径的几何语义分类，决定填充时按外轮廓还是孔洞处理。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BoundaryType {
    /// 外部边界。
    External,
    /// 折线边界：整条路径按闭合折线处理。
    Polyline,
    /// 派生边界：由其他图元计算得出。
    Derived,
    /// 最外层边界。
    Outermost,
    /// 凹口边界：外轮廓上的缺口。
    Notch,
    /// 凸包边界：由点集凸包生成。
    Hull,
}

/// 填充边界上的一条边，直线边只需起终点。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HatchEdge {
    /// 边起点坐标。
    pub start_point: crate::geometry::Point,
    /// 边终点坐标。
    pub end_point: crate::geometry::Point,
    /// 凸度：弧段圆心角四分之一的正切值，0 表示直线边，正值表示逆时针凸出。
    pub bulge: f64,
    /// 边的几何类型，决定 `bulge` 是否被解释为弧段。
    pub edge_type: EdgeType,
}

/// 填充边的几何类型。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EdgeType {
    /// 直线边。
    Line,
    /// 圆弧边，由凸度描述弧度。
    CircularArc,
    /// 椭圆弧边。
    EllipticArc,
    /// 样条边。
    Spline,
}

/// 标注的测量方式，决定测量值的含义与文字前缀。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DimensionType {
    /// 线性标注：测量沿指定方向的距离。
    Linear,
    /// 对齐标注：测量两点间的真实距离。
    Aligned,
    /// 角度标注：测量夹角，值以弧度计。
    Angular,
    /// 半径标注：测量半径，结果加 `R` 前缀。
    Radial,
    /// 直径标注：测量直径，结果加 `⌀` 前缀。
    Diameter,
    /// 弧长标注：测量圆弧长度。
    ArcLength,
    /// 坐标标注：标注相对原点的坐标值。
    Ordinate,
}

/// 文字样式：字体、字高与排版参数。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    /// 样式名，默认 `"Standard"`。
    pub name: String,
    /// 字体名，默认 `"Arial"`。
    pub font: String,
    /// 字高，单位为文档单位，默认 `2.5`。
    pub height: f64,
    /// 字宽因子，1.0 为不压缩不拉伸。
    pub width_factor: f64,
    /// 倾斜角，单位为弧度，正值向右倾斜。
    pub oblique_angle: f64,
    /// 是否反向书写（左右镜像）。
    pub is_backwards: bool,
    /// 是否倒置书写（上下颠倒）。
    pub is_upside_down: bool,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            name: "Standard".to_string(),
            font: "Arial".to_string(),
            height: 2.5,
            width_factor: 1.0,
            oblique_angle: 0.0,
            is_backwards: false,
            is_upside_down: false,
        }
    }
}

/// 文字相对插入点的对齐方式，水平与垂直方向组合使用。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextAlignment {
    /// 左对齐。
    Left,
    /// 水平居中。
    Center,
    /// 右对齐。
    Right,
    /// 垂直居中。
    Middle,
    /// 顶对齐。
    Top,
    /// 水平居中且顶对齐。
    MiddleTop,
    /// 水平垂直均居中。
    MiddleMiddle,
    /// 水平居中且底对齐。
    MiddleBottom,
    /// 底对齐。
    Bottom,
}

impl Default for TextAlignment {
    fn default() -> Self {
        TextAlignment::Left
    }
}
