//! 数学模块：Shi SDK 的向量、矩阵与二维变换基础设施。
//!
//! 本模块不依赖任何上层业务概念，只提供数值类型与算子：二维/三维向量、
//! 3×3 与 4×4 矩阵、以平移 + 旋转 + 缩放描述的二维仿射变换。
//! 约定：坐标与长度均为模型空间的实数单位，角度一律使用弧度，逆时针为正；
//! 所有运算方法都不修改操作数，而是返回新值。
//!
//! 与其他模块的关系：Entity 实体、Document 文档、Layer 图层等上层类型复用这里的
//! [`Vector2`] 表示平面位置；[`Matrix3`] 与 [`Matrix4`] 服务于二维变换复合与三维
//! 视图投影；[`Transform2D`] 是编辑操作（移动、旋转、缩放）直接读写的变换描述。

/// 二维向量与平面点。
pub mod vector;
pub use vector::Vector2;

/// 3×3 矩阵，表示含平移的二维仿射变换。
pub mod matrix;
pub use matrix::Matrix3;

/// 4×4 矩阵，表示三维变换与透视投影。
pub mod matrix4;
pub use matrix4::Matrix4;

/// 以平移 + 旋转 + 缩放描述的二维变换。
pub mod transformation;
pub use transformation::Transform2D;

/// [`Transform2D`] 的别名，保留旧命名以便既有代码继续编译。
pub type Transformation = Transform2D;
