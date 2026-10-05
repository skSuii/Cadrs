use crate::data_structure::{Entity, Layer, Block, ObjectId, BlockReference};
use crate::geometry::Point;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    id: ObjectId,
    name: String,
    version: String,
    units: Units,
    entities: HashMap<ObjectId, Entity>,
    layers: HashMap<ObjectId, Layer>,
    blocks: HashMap<ObjectId, Block>,
    block_references: HashMap<ObjectId, BlockReference>,
    model_space: ObjectId,
    paper_space: ObjectId,
    active_space: SpaceType,
    properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Units {
    Millimeters,
    Centimeters,
    Meters,
    Kilometers,
    Inches,
    Feet,
    Miles,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpaceType {
    ModelSpace,
    PaperSpace,
}

impl Document {
    #[inline]
    pub fn new(name: String) -> Self {
        let model_space_id = ObjectId::new();
        let paper_space_id = ObjectId::new();

        let model_space_layer_id = model_space_id.clone();
        let paper_space_layer_id = paper_space_id.clone();

        let mut doc = Self {
            id: ObjectId::new(),
            name,
            version: "1.0".to_string(),
            units: Units::Millimeters,
            entities: HashMap::new(),
            layers: HashMap::new(),
            blocks: HashMap::new(),
            block_references: HashMap::new(),
            model_space: model_space_id,
            paper_space: paper_space_id,
            active_space: SpaceType::ModelSpace,
            properties: HashMap::new(),
        };

        let mut model_space_layer = Layer::new("ModelSpace".to_string());
        model_space_layer.set_description("Default model space layer".to_string());
        doc.layers.insert(model_space_layer_id, model_space_layer);

        let mut paper_space_layer = Layer::new("PaperSpace".to_string());
        paper_space_layer.set_description("Default paper space layer".to_string());
        doc.layers.insert(paper_space_layer_id, paper_space_layer);

        doc
    }

    #[inline]
    pub fn id(&self) -> &ObjectId {
        &self.id
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    #[inline]
    pub fn version(&self) -> &str {
        &self.version
    }

    #[inline]
    pub fn units(&self) -> Units {
        self.units
    }

    #[inline]
    pub fn set_units(&mut self, units: Units) {
        self.units = units;
    }

    #[inline]
    pub fn entities(&self) -> &HashMap<ObjectId, Entity> {
        &self.entities
    }

    #[inline]
    pub fn entities_mut(&mut self) -> &mut HashMap<ObjectId, Entity> {
        &mut self.entities
    }

    #[inline]
    pub fn add_entity(&mut self, entity: Entity) -> ObjectId {
        let id = entity.id().clone();
        self.entities.insert(id.clone(), entity);
        id
    }

    #[inline]
    pub fn remove_entity(&mut self, entity_id: &ObjectId) -> bool {
        self.entities.remove(entity_id).is_some()
    }

    #[inline]
    pub fn get_entity(&self, entity_id: &ObjectId) -> Option<&Entity> {
        self.entities.get(entity_id)
    }

    #[inline]
    pub fn get_entity_mut(&mut self, entity_id: &ObjectId) -> Option<&mut Entity> {
        self.entities.get_mut(entity_id)
    }

    #[inline]
    pub fn entity_exists(&self, entity_id: &ObjectId) -> bool {
        self.entities.contains_key(entity_id)
    }

    #[inline]
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    #[inline]
    pub fn layers(&self) -> &HashMap<ObjectId, Layer> {
        &self.layers
    }

    #[inline]
    pub fn layers_mut(&mut self) -> &mut HashMap<ObjectId, Layer> {
        &mut self.layers
    }

    #[inline]
    pub fn add_layer(&mut self, layer: Layer) -> ObjectId {
        let id = layer.id().clone();
        self.layers.insert(id.clone(), layer);
        id
    }

    #[inline]
    pub fn remove_layer(&mut self, layer_id: &ObjectId) -> bool {
        self.layers.remove(layer_id).is_some()
    }

    #[inline]
    pub fn get_layer(&self, layer_id: &ObjectId) -> Option<&Layer> {
        self.layers.get(layer_id)
    }

    #[inline]
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    #[inline]
    pub fn blocks(&self) -> &HashMap<ObjectId, Block> {
        &self.blocks
    }

    #[inline]
    pub fn add_block(&mut self, block: Block) -> ObjectId {
        let id = block.id().clone();
        self.blocks.insert(id.clone(), block);
        id
    }

    #[inline]
    pub fn get_block(&self, block_id: &ObjectId) -> Option<&Block> {
        self.blocks.get(block_id)
    }

    #[inline]
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    #[inline]
    pub fn block_references(&self) -> &HashMap<ObjectId, BlockReference> {
        &self.block_references
    }

    #[inline]
    pub fn add_block_reference(&mut self, block_ref: BlockReference) -> ObjectId {
        let id = block_ref.id().clone();
        self.block_references.insert(id.clone(), block_ref);
        id
    }

    #[inline]
    pub fn get_block_reference(&self, block_ref_id: &ObjectId) -> Option<&BlockReference> {
        self.block_references.get(block_ref_id)
    }

    #[inline]
    pub fn block_reference_count(&self) -> usize {
        self.block_references.len()
    }

    #[inline]
    pub fn model_space(&self) -> &ObjectId {
        &self.model_space
    }

    #[inline]
    pub fn paper_space(&self) -> &ObjectId {
        &self.paper_space
    }

    #[inline]
    pub fn active_space(&self) -> SpaceType {
        self.active_space
    }

    #[inline]
    pub fn set_active_space(&mut self, space: SpaceType) {
        self.active_space = space;
    }

    #[inline]
    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    #[inline]
    pub fn set_property(&mut self, key: String, value: String) {
        self.properties.insert(key, value);
    }

    #[inline]
    pub fn bounding_box(&self) -> Option<(Point, Point)> {
        if self.entities.is_empty() {
            return None;
        }

        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;

        for entity in self.entities.values() {
            if let Some((min, max)) = entity.bounding_box() {
                min_x = min_x.min(min.x);
                max_x = max_x.max(max.x);
                min_y = min_y.min(min.y);
                max_y = max_y.max(max.y);
            }
        }

        Some((
            Point::new(min_x, min_y, 0.0),
            Point::new(max_x, max_y, 0.0),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_structure::{EntityType, EntityGeometry};

    #[test]
    fn test_document_creation() {
        let doc = Document::new("TestDrawing".to_string());
        
        assert_eq!(doc.name(), "TestDrawing");
        assert_eq!(doc.entity_count(), 0);
        assert!(doc.layer_count() > 0);
    }

    #[test]
    fn test_document_add_entity() {
        let mut doc = Document::new("TestDrawing".to_string());
        let entity = Entity::new(
            EntityType::Point,
            EntityGeometry::Point(Point::origin()),
        );
        
        let entity_id = doc.add_entity(entity);
        assert_eq!(doc.entity_count(), 1);
        assert!(doc.get_entity(&entity_id).is_some());
    }

    #[test]
    fn test_document_add_layer() {
        let mut doc = Document::new("TestDrawing".to_string());
        let layer = Layer::new("MyLayer".to_string());
        
        let layer_id = doc.add_layer(layer);
        assert_eq!(doc.layer_count(), 3);
        assert!(doc.get_layer(&layer_id).is_some());
    }
}
