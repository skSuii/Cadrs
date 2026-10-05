pub mod document;
pub mod entity;
pub mod entity_factory;
pub mod layer;
pub mod block;
pub mod block_reference;
pub mod selection;
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
