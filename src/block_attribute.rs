use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub tag: String,
    pub value: String,
    pub position: crate::geometry::Point,
    pub text_style: crate::text::TextStyle,
    pub is_invisible: bool,
    pub is_constant: bool,
    pub is_verify: bool,
    pub is_locked: bool,
    pub alignment: crate::text::TextAlignment,
    pub height: f64,
    pub rotation: f64,
    pub width_factor: f64,
    pub oblique_angle: f64,
}

impl Default for Attribute {
    fn default() -> Self {
        Self {
            tag: String::new(),
            value: String::new(),
            position: crate::geometry::Point::origin(),
            text_style: crate::text::TextStyle::default(),
            is_invisible: false,
            is_constant: false,
            is_verify: false,
            is_locked: false,
            alignment: crate::text::TextAlignment::Left,
            height: 2.5,
            rotation: 0.0,
            width_factor: 1.0,
            oblique_angle: 0.0,
        }
    }
}

impl Attribute {
    #[inline]
    pub fn new(tag: &str, value: &str) -> Self {
        Self {
            tag: tag.to_string(),
            value: value.to_string(),
            ..Default::default()
        }
    }

    #[inline]
    pub fn with_position(mut self, position: crate::geometry::Point) -> Self {
        self.position = position;
        self
    }

    #[inline]
    pub fn set_tag(&mut self, tag: &str) {
        self.tag = tag.to_string();
    }

    #[inline]
    pub fn set_value(&mut self, value: &str) {
        self.value = value.to_string();
    }

    #[inline]
    pub fn is_visible(&self) -> bool {
        !self.is_invisible
    }
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}={}", self.tag, self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributeDefinition {
    pub tag: String,
    pub prompt: String,
    pub default_value: String,
    pub position: crate::geometry::Point,
    pub text_style: crate::text::TextStyle,
    pub field_length: u32,
    pub alignment: crate::text::TextAlignment,
    pub is_invisible: bool,
    pub is_constant: bool,
    pub is_verify: bool,
    pub is_locked: bool,
    pub height: f64,
    pub rotation: f64,
    pub width_factor: f64,
    pub oblique_angle: f64,
    pub mtext_bottom: bool,
}

impl Default for AttributeDefinition {
    fn default() -> Self {
        Self {
            tag: String::new(),
            prompt: String::new(),
            default_value: String::new(),
            position: crate::geometry::Point::origin(),
            text_style: crate::text::TextStyle::default(),
            field_length: 0,
            alignment: crate::text::TextAlignment::Left,
            is_invisible: false,
            is_constant: false,
            is_verify: false,
            is_locked: false,
            height: 2.5,
            rotation: 0.0,
            width_factor: 1.0,
            oblique_angle: 0.0,
            mtext_bottom: false,
        }
    }
}

impl AttributeDefinition {
    #[inline]
    pub fn new(tag: &str, prompt: &str) -> Self {
        Self {
            tag: tag.to_string(),
            prompt: prompt.to_string(),
            ..Default::default()
        }
    }

    #[inline]
    pub fn with_defaults(tag: &str, prompt: &str, position: crate::geometry::Point) -> Self {
        Self {
            tag: tag.to_string(),
            prompt: prompt.to_string(),
            position,
            ..Default::default()
        }
    }

    #[inline]
    pub fn create_attribute(&self, value: &str) -> Attribute {
        Attribute {
            tag: self.tag.clone(),
            value: value.to_string(),
            position: self.position,
            text_style: self.text_style.clone(),
            is_invisible: self.is_invisible,
            is_constant: self.is_constant,
            is_verify: self.is_verify,
            is_locked: self.is_locked,
            alignment: self.alignment,
            height: self.height,
            rotation: self.rotation,
            width_factor: self.width_factor,
            oblique_angle: self.oblique_angle,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockTableRecord {
    pub name: String,
    pub objects: Vec<super::data_structure::ObjectId>,
    pub attribute_defs: Vec<AttributeDefinition>,
    pub origin: crate::geometry::Point,
    pub units: DrawingUnit,
    pub scaling: (f64, f64),
    pub description: String,
    pub is_explodable: bool,
    pub block_unit: BlockUnit,
    pub comments: String,
}

impl Default for BlockTableRecord {
    fn default() -> Self {
        Self::new("*Unnamed")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrawingUnit {
    Unitless,
    Inches,
    Feet,
    Miles,
    Millimeters,
    Centimeters,
    Meters,
    Kilometers,
    Microinches,
    Mils,
    Angstroms,
    Nanometers,
    Microns,
    Decimeters,
    Decameters,
    Hectometers,
    Gigameters,
    AstronomicalUnits,
    LightYears,
    Parsecs,
}

impl Default for DrawingUnit {
    fn default() -> Self {
        DrawingUnit::Unitless
    }
}

impl DrawingUnit {
    #[inline]
    pub fn conversion_factor(&self) -> f64 {
        match self {
            DrawingUnit::Unitless => 1.0,
            DrawingUnit::Inches => 25.4,
            DrawingUnit::Feet => 304.8,
            DrawingUnit::Miles => 1609344.0,
            DrawingUnit::Millimeters => 1.0,
            DrawingUnit::Centimeters => 10.0,
            DrawingUnit::Meters => 1000.0,
            DrawingUnit::Kilometers => 1000000.0,
            DrawingUnit::Microinches => 0.0000254,
            DrawingUnit::Mils => 0.0254,
            DrawingUnit::Angstroms => 0.0000001,
            DrawingUnit::Nanometers => 0.000001,
            DrawingUnit::Microns => 0.001,
            DrawingUnit::Decimeters => 100.0,
            DrawingUnit::Decameters => 10000.0,
            DrawingUnit::Hectometers => 100000.0,
            DrawingUnit::Gigameters => 1000000000.0,
            DrawingUnit::AstronomicalUnits => 149597870700000.0,
            DrawingUnit::LightYears => 9460730472580800000.0,
            DrawingUnit::Parsecs => 30856775814913673000.0,
        }
    }

    #[inline]
    pub fn to_millimeters(&self, value: f64) -> f64 {
        value * self.conversion_factor()
    }

    #[inline]
    pub fn from_millimeters(&self, value: f64) -> f64 {
        value / self.conversion_factor()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockUnit {
    Unitless,
    Inches,
    Feet,
    Millimeters,
    Centimeters,
    Meters,
}

impl Default for BlockUnit {
    fn default() -> Self {
        BlockUnit::Unitless
    }
}

impl BlockUnit {
    #[inline]
    pub fn from_drawing_unit(unit: DrawingUnit) -> Self {
        match unit {
            DrawingUnit::Unitless => BlockUnit::Unitless,
            DrawingUnit::Inches | DrawingUnit::Feet | DrawingUnit::Miles => BlockUnit::Inches,
            DrawingUnit::Millimeters => BlockUnit::Millimeters,
            DrawingUnit::Centimeters => BlockUnit::Centimeters,
            DrawingUnit::Meters => BlockUnit::Meters,
            _ => BlockUnit::Unitless,
        }
    }
}

impl BlockTableRecord {
    #[inline]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            objects: Vec::new(),
            attribute_defs: Vec::new(),
            origin: crate::geometry::Point::origin(),
            units: DrawingUnit::Unitless,
            scaling: (1.0, 1.0),
            description: String::new(),
            is_explodable: true,
            block_unit: BlockUnit::default(),
            comments: String::new(),
        }
    }

    #[inline]
    pub fn add_object(&mut self, object_id: super::data_structure::ObjectId) {
        if !self.objects.contains(&object_id) {
            self.objects.push(object_id);
        }
    }

    #[inline]
    pub fn remove_object(&mut self, object_id: &super::data_structure::ObjectId) {
        self.objects.retain(|id| id != object_id);
    }

    #[inline]
    pub fn add_attribute_def(&mut self, attr_def: AttributeDefinition) {
        self.attribute_defs.push(attr_def);
    }

    #[inline]
    pub fn remove_attribute_def(&mut self, tag: &str) {
        self.attribute_defs.retain(|def| def.tag != tag);
    }

    #[inline]
    pub fn get_attribute_def(&self, tag: &str) -> Option<&AttributeDefinition> {
        self.attribute_defs.iter().find(|def| def.tag == tag)
    }

    #[inline]
    pub fn object_count(&self) -> usize {
        self.objects.len()
    }

    #[inline]
    pub fn attribute_count(&self) -> usize {
        self.attribute_defs.len()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.objects.clear();
        self.attribute_defs.clear();
    }
}

#[derive(Debug, Clone)]
pub struct BlockTable {
    records: std::collections::HashMap<String, BlockTableRecord>,
    current_block: Option<String>,
}

impl Default for BlockTable {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockTable {
    #[inline]
    pub fn new() -> Self {
        let mut table = Self {
            records: std::collections::HashMap::new(),
            current_block: None,
        };
        table.register_builtin_blocks();
        table
    }

    fn register_builtin_blocks(&mut self) {
        self.records.insert("*Model_Space".to_string(), BlockTableRecord::new("*Model_Space"));
        self.records.insert("*Paper_Space".to_string(), BlockTableRecord::new("*Paper_Space"));
        self.records.insert("*Paper_Space0".to_string(), BlockTableRecord::new("*Paper_Space0"));
    }

    #[inline]
    pub fn register(&mut self, record: BlockTableRecord) -> bool {
        if record.name.is_empty() {
            return false;
        }
        self.records.insert(record.name.clone(), record);
        true
    }

    #[inline]
    pub fn get(&self, name: &str) -> Option<&BlockTableRecord> {
        self.records.get(name)
    }

    #[inline]
    pub fn get_mut(&mut self, name: &str) -> Option<&mut BlockTableRecord> {
        self.records.get_mut(name)
    }

    #[inline]
    pub fn has(&self, name: &str) -> bool {
        self.records.contains_key(name)
    }

    #[inline]
    pub fn remove(&mut self, name: &str) -> bool {
        if name.starts_with('*') {
            return false;
        }
        self.records.remove(name).is_some()
    }

    #[inline]
    pub fn rename(&mut self, old_name: &str, new_name: &str) -> bool {
        if old_name.starts_with('*') || new_name.starts_with('*') {
            return false;
        }
        if let Some(record) = self.records.remove(old_name) {
            let mut new_record = record;
            new_record.name = new_name.to_string();
            self.records.insert(new_name.to_string(), new_record);
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn set_current(&mut self, name: &str) -> bool {
        if self.records.contains_key(name) {
            self.current_block = Some(name.to_string());
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn current(&self) -> Option<&str> {
        self.current_block.as_deref()
    }

    #[inline]
    pub fn current_mut(&mut self) -> Option<&mut BlockTableRecord> {
        if let Some(ref name) = self.current_block {
            self.records.get_mut(name)
        } else {
            None
        }
    }

    #[inline]
    pub fn names(&self) -> Vec<&str> {
        self.records.keys().map(|s| s.as_str()).collect()
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.records.len()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.records.clear();
        self.current_block = None;
        self.register_builtin_blocks();
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InsertEntity {
    pub block_name: String,
    pub position: crate::geometry::Point,
    pub scale: (f64, f64, f64),
    pub rotation: f64,
    pub columns: u32,
    pub rows: u32,
    pub column_spacing: f64,
    pub row_spacing: f64,
    pub attributes: Vec<Attribute>,
}

impl Default for InsertEntity {
    fn default() -> Self {
        Self {
            block_name: String::new(),
            position: crate::geometry::Point::origin(),
            scale: (1.0, 1.0, 1.0),
            rotation: 0.0,
            columns: 1,
            rows: 1,
            column_spacing: 0.0,
            row_spacing: 0.0,
            attributes: Vec::new(),
        }
    }
}

impl InsertEntity {
    #[inline]
    pub fn new(block_name: &str) -> Self {
        Self {
            block_name: block_name.to_string(),
            ..Default::default()
        }
    }

    #[inline]
    pub fn with_transform(mut self, position: crate::geometry::Point, scale: (f64, f64, f64), rotation: f64) -> Self {
        self.position = position;
        self.scale = scale;
        self.rotation = rotation;
        self
    }

    #[inline]
    pub fn add_attribute(&mut self, attribute: Attribute) {
        self.attributes.push(attribute);
    }

    #[inline]
    pub fn set_attribute(&mut self, tag: &str, value: &str) {
        if let Some(attr) = self.attributes.iter_mut().find(|a| a.tag == tag) {
            attr.value = value.to_string();
        }
    }

    #[inline]
    pub fn get_attribute(&self, tag: &str) -> Option<&Attribute> {
        self.attributes.iter().find(|a| a.tag == tag)
    }

    #[inline]
    pub fn attribute_count(&self) -> usize {
        self.attributes.len()
    }

    #[inline]
    pub fn create_array(&self, columns: u32, rows: u32, col_spacing: f64, row_spacing: f64) -> Vec<InsertEntity> {
        let mut array = Vec::new();
        for row in 0..rows {
            for col in 0..columns {
                if row == 0 && col == 0 {
                    array.push(self.clone());
                } else {
                    let offset_x = col as f64 * col_spacing;
                    let offset_y = row as f64 * row_spacing;
                    let new_position = crate::geometry::Point::new(
                        self.position.x + offset_x,
                        self.position.y + offset_y,
                        self.position.z,
                    );
                    let mut new_insert = self.clone();
                    new_insert.position = new_position;
                    array.push(new_insert);
                }
            }
        }
        array
    }

    #[inline]
    pub fn transformation_matrix(&self) -> crate::math::Matrix4 {
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        let (sx, sy, sz) = self.scale;

        crate::math::Matrix4::new(
            sx * cos_r, sy * -sin_r, 0.0, self.position.x,
            sx * sin_r, sy * cos_r, 0.0, self.position.y,
            0.0, 0.0, sz, self.position.z,
            0.0, 0.0, 0.0, 1.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;
    use crate::data_structure::ObjectId;

    #[test]
    fn test_attribute_creation() {
        let attr = Attribute::new("TAG1", "Value1");
        assert_eq!(attr.tag, "TAG1");
        assert_eq!(attr.value, "Value1");
    }

    #[test]
    fn test_attribute_definition() {
        let def = AttributeDefinition::new("TAG1", "Enter value:");
        assert_eq!(def.tag, "TAG1");
        assert_eq!(def.prompt, "Enter value:");
    }

    #[test]
    fn test_attribute_definition_create_attribute() {
        let def = AttributeDefinition::new("TAG1", "Enter value:");
        let attr = def.create_attribute("TestValue");
        assert_eq!(attr.tag, "TAG1");
        assert_eq!(attr.value, "TestValue");
    }

    #[test]
    fn test_block_table_record() {
        let mut record = BlockTableRecord::new("TestBlock");
        assert_eq!(record.name, "TestBlock");
        assert!(record.object_count() == 0);
    }

    #[test]
    fn test_block_table_record_operations() {
        let mut record = BlockTableRecord::new("TestBlock");
        record.add_object(ObjectId::new());
        record.add_object(ObjectId::new());
        assert_eq!(record.object_count(), 2);
        record.clear();
        assert_eq!(record.object_count(), 0);
    }

    #[test]
    fn test_block_table() {
        let table = BlockTable::new();
        assert!(table.has("*Model_Space"));
        assert_eq!(table.count(), 3);
    }

    #[test]
    fn test_block_table_operations() {
        let mut table = BlockTable::new();
        let record = BlockTableRecord::new("MyBlock");
        assert!(table.register(record));
        assert!(table.has("MyBlock"));
        assert!(table.remove("MyBlock"));
        assert!(!table.has("MyBlock"));
    }

    #[test]
    fn test_insert_entity() {
        let insert = InsertEntity::new("TestBlock")
            .with_transform(Point::new(100.0, 100.0, 0.0), (2.0, 2.0, 1.0), 45.0);
        assert_eq!(insert.block_name, "TestBlock");
        assert_eq!(insert.scale, (2.0, 2.0, 1.0));
        assert!((insert.rotation - 45.0 * std::f64::consts::PI / 180.0).abs() < 1e-10);
    }

    #[test]
    fn test_insert_entity_attributes() {
        let mut insert = InsertEntity::new("TestBlock");
        insert.add_attribute(Attribute::new("ATTR1", "Value1"));
        insert.add_attribute(Attribute::new("ATTR2", "Value2"));
        assert_eq!(insert.attribute_count(), 2);
        insert.set_attribute("ATTR1", "NewValue");
        assert_eq!(insert.get_attribute("ATTR1").unwrap().value, "NewValue");
    }

    #[test]
    fn test_insert_entity_array() {
        let insert = InsertEntity::new("TestBlock")
            .with_transform(Point::origin(), (1.0, 1.0, 1.0), 0.0);
        let array = insert.create_array(2, 2, 10.0, 20.0);
        assert_eq!(array.len(), 4);
    }

    #[test]
    fn test_drawing_unit_conversion() {
        assert!((DrawingUnit::Inches.to_millimeters(1.0) - 25.4).abs() < 1e-10);
        assert!((DrawingUnit::Millimeters.to_millimeters(25.4) - 25.4).abs() < 1e-10);
    }

    #[test]
    fn test_block_unit_from_drawing_unit() {
        assert_eq!(BlockUnit::from_drawing_unit(DrawingUnit::Millimeters), BlockUnit::Millimeters);
        assert_eq!(BlockUnit::from_drawing_unit(DrawingUnit::Inches), BlockUnit::Inches);
    }
}
