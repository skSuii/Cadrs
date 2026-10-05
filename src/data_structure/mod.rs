//! 核心数据结构层：定义 CAD 文档的内存模型。
//!
//! 一个 [`Document`] 由若干 [`Entity`]（几何实体）、[`Layer`]（图层）与 [`Block`]（块定义）
//! 组成，所有对象以 [`ObjectId`] 为全局唯一键；[`BlockReference`] 表示块在图纸中的插入实例，
//! [`SelectionSet`] 记录被选中的实体集合。上层（渲染、捕捉、IO、命令）均通过本模块读写图纸数据。
//! 坐标系为右手直角坐标系，长度单位为文档 [`document::Units`]，角度一律使用弧度。

/// 文档容器：实体、图层、块与块参照的聚合根。
pub mod document;
/// 实体类型、通用属性与几何数据枚举。
pub mod entity;
/// 按几何数据构造实体并自动推断 [`entity::EntityType`] 的工厂函数集合。
pub mod entity_factory;
/// 图层：颜色、线宽与可见性设置。
pub mod layer;
/// 块定义：可复用的实体集合及插入基点。
pub mod block;
/// 块参照：块在图纸中的一次插入实例。
pub mod block_reference;
/// 选择集与管理器：记录当前选中的实体。
pub mod selection;
/// 对象唯一标识 [`ObjectId`]。
pub mod entity_id;

pub use entity::EntityGeometry;
pub use entity_id::ObjectId;

pub use document::Document;
pub use entity::{Entity, EntityType, Visibility, Transform, TextStyle, TextAlignment, HatchBoundary, BoundaryType, HatchEdge, EdgeType, DimensionType};
pub use layer::Layer;
pub use block::{Block};
pub use block_reference::BlockReference;
pub use selection::SelectionSet;
pub use entity_factory::{make_entity, clone_with_new_id, make_line, make_circle, make_point, make_arc, make_ellipse, make_spline, make_polyline, make_solid, make_hatch};
