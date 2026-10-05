//! Shi CAD SDK 的门面（facade）模块，仅声明并导出全部子模块。
//!
//! 本文件不含具体实现，按职责可分为四类：
//! - 基础设施：`math`（向量与矩阵）、`geometry`（点与各类曲线）、`data_structure`（Entity 实体、Document 文档、Layer 图层、ObjectId）；
//! - 交互与编辑：`command`、`selection`、`grid`、`snap`、`grip_editor`、`edit`、`constraint`、`parameter`；
//! - 图形对象：`dimension`、`geometric_tolerance`、`hatch`、`text`、`multileader`、`layer`、`line_type`、`block_attribute`、`welding_symbol`、`group`、`dynamic_block`；
//! - 数据交换与呈现：`io`、`xref`、`xdata`、`ole`、`sheet_set`、`layout`、`render`、`spatial`、`performance`、`surface_texture`。
//!
//! 调用者一般直接引入所需子模块（如 `cadrs::grid::Grid`），也可使用本文件再导出的
//! [`Entity`](data_structure::Entity)、[`ObjectId`](data_structure::ObjectId)、
//! [`Layer`](data_structure::Layer)、[`Document`](data_structure::Document) 与 [`Vector2`](math::Vector2)。

/// 几何曲线：点、直线、圆、圆弧、椭圆及求交等基础几何运算。
pub mod geometry;
/// 数学基础：二维/三维向量、矩阵与数值工具。
pub mod math;
/// 统一错误类型 `CadError`、结果别名 `CadResult` 及校验辅助函数。
pub mod error;
/// 文档对象模型：Entity 实体、Document 文档、Layer 图层与 ObjectId 标识。
pub mod data_structure;

/// 对外 API 聚合层：命令、事件与脚本绑定入口。
pub mod api;
/// 块属性（Block Attribute）定义与取值。
pub mod block_attribute;
/// 命令系统：`Command` trait、命令注册表、命令管理器与撤销历史。
pub mod command;
/// 几何约束与尺寸约束的求解。
pub mod constraint;
/// 尺寸标注：线性、角度、半径、坐标等标注对象。
pub mod dimension;
/// 动态块：参数、动作与可见性状态。
pub mod dynamic_block;
/// 编辑操作：移动、复制、旋转、镜像、修剪等变换与修改。
pub mod edit;
/// 几何公差（形位公差）标注及公差带符号。
pub mod geometric_tolerance;
/// 栅格与正交：Grid 栅格生成/捕捉与 Ortho 正交约束。
pub mod grid;
/// 夹点编辑器：夹点定义、命中检测与拖拽编辑。
pub mod grip_editor;
/// 编组（Group）：实体的命名集合。
pub mod group;
/// 图案填充：Hatch 填充对象、填充图案与渐变填充。
pub mod hatch;
/// 操作历史：历史记录条目与文档快照。
pub mod history;
/// 文件读写：DXF 等格式的导入导出与序列化。
pub mod io;
/// 图层：图层属性、状态与过滤器。
pub mod layer;
/// 布局与图纸空间视口（Viewport）。
pub mod layout;
/// 线型：线型定义、图案段与加载。
pub mod line_type;
/// 测量：距离、角度、面积、半径、弧长及单位换算。
pub mod measurement;
/// 多重引线（Multileader）标注。
pub mod multileader;
/// OLE 对象嵌入。
pub mod ole;
/// 参数化设计的参数与表达式。
pub mod parameter;
/// 性能统计与剖析工具。
pub mod performance;
/// 渲染：视口、细分（Tessellation）与图元绘制数据生成。
pub mod render;
/// 选择：选择集、选择模式、选择过滤器与选择管理器。
pub mod selection;
/// 图纸集（Sheet Set）管理。
pub mod sheet_set;
/// 对象捕捉（Snap）：端点、中点、圆心等特征点求解。
pub mod snap;
/// 空间索引：R-Tree、四叉树与网格索引。
pub mod spatial;
/// 曲面纹理贴图参数。
pub mod surface_texture;
/// 文字：单行/多行文字、字段与文字样式。
pub mod text;
/// 焊接符号标注。
pub mod welding_symbol;
/// 扩展数据（XDATA）：附着在实体上的自定义数据。
pub mod xdata;
/// 外部参照（XRef）：外部图纸的引用与加载。
pub mod xref;

/// 二维向量类型 `Vector2` 的再导出。
pub use math::Vector2;
/// 文档对象模型核心类型的再导出：Entity 实体、ObjectId、Layer 图层、Document 文档。
pub use data_structure::{Entity, ObjectId, Layer, Document};
