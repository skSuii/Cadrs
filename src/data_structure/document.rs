//! 文档：一张图纸的内存模型，是上层所有操作的聚合根。
//!
//! [`Document`] 以 [`ObjectId`] 为键统一管理实体、图层、块与块参照四张表，并区分模型空间
//! （ModelSpace）与图纸空间（PaperSpace）两套坐标环境；[`Document::new`] 会预置名为
//! `"ModelSpace"` 与 `"PaperSpace"` 的两个默认图层。增删对象只改变内存中的哈希表，
//! 不写盘；持久化由 `io` 模块负责。长度单位为 [`Units`]，角度为弧度。

use crate::data_structure::{Entity, Layer, Block, ObjectId, BlockReference};
use crate::geometry::Point;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// 图纸文档：持有全部实体、图层、块与块参照，并记录当前空间与单位制。
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

/// 文档长度单位，影响尺寸解读与导出时的换算，不改变已存储的坐标数值。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Units {
    /// 毫米，[`Document::new`] 的默认单位。
    Millimeters,
    /// 厘米。
    Centimeters,
    /// 米。
    Meters,
    /// 千米。
    Kilometers,
    /// 英寸。
    Inches,
    /// 英尺。
    Feet,
    /// 英里。
    Miles,
}

/// 文档的当前工作空间。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpaceType {
    /// 模型空间：按真实比例绘制几何图形。
    ModelSpace,
    /// 图纸空间：用于排版出图，可容纳视口。
    PaperSpace,
}

impl Document {
    /// 新建空白文档并预置两个默认图层。
    ///
    /// 会创建 `"ModelSpace"`（模型空间）与 `"PaperSpace"`（图纸空间）两个图层，其图层 id 分别
    /// 与 [`Document::model_space`]、[`Document::paper_space`] 返回的标识一致，因此新建文档的
    /// [`Document::layer_count`] 为 2。单位默认为 [`Units::Millimeters`]，当前空间默认为
    /// [`SpaceType::ModelSpace`]，实体、块、块参照表均为空。
    ///
    /// - `name`：文档名。
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

    /// 文档的唯一标识，随文档生成，不随保存/另存改变。
    #[inline]
    pub fn id(&self) -> &ObjectId {
        &self.id
    }

    /// 文档名，仅作标识用，不影响文件名与路径。
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 修改文档名，会修改 `self`。
    #[inline]
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// 文档格式版本号，新建文档时为 `"1.0"`，只读。
    #[inline]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// 当前长度单位。
    #[inline]
    pub fn units(&self) -> Units {
        self.units
    }

    /// 设置长度单位，会修改 `self`；仅影响后续按单位解读与导出的数值，不改动已有坐标。
    #[inline]
    pub fn set_units(&mut self, units: Units) {
        self.units = units;
    }

    /// 全部实体的只读视图，键为实体 id，包含模型空间与图纸空间中的实体。
    #[inline]
    pub fn entities(&self) -> &HashMap<ObjectId, Entity> {
        &self.entities
    }

    /// 全部实体的可变视图，绕过 [`Document::add_entity`] 直接改写不会触发任何副作用。
    #[inline]
    pub fn entities_mut(&mut self) -> &mut HashMap<ObjectId, Entity> {
        &mut self.entities
    }

    /// 登记一个实体，会修改 `self`。
    ///
    /// 以实体自身的 id 为键；若已存在同 id 实体则被直接覆盖，因此复用 id 会丢失旧对象。
    ///
    /// - `entity`：待加入的实体，所有权转移给文档。
    ///
    /// 返回该实体的 id，供后续查询、删除使用。
    #[inline]
    pub fn add_entity(&mut self, entity: Entity) -> ObjectId {
        let id = entity.id().clone();
        self.entities.insert(id.clone(), entity);
        id
    }

    /// 按 id 删除实体，会修改 `self`。
    ///
    /// 返回是否真正删除了实体：id 不存在时返回 `false`。
    #[inline]
    pub fn remove_entity(&mut self, entity_id: &ObjectId) -> bool {
        self.entities.remove(entity_id).is_some()
    }

    /// 按 id 查询实体，不存在时返回 [`None`]。
    #[inline]
    pub fn get_entity(&self, entity_id: &ObjectId) -> Option<&Entity> {
        self.entities.get(entity_id)
    }

    /// 按 id 查询实体的可变引用以便就地编辑，不存在时返回 [`None`]。
    #[inline]
    pub fn get_entity_mut(&mut self, entity_id: &ObjectId) -> Option<&mut Entity> {
        self.entities.get_mut(entity_id)
    }

    /// 判断指定 id 的实体是否存在。
    #[inline]
    pub fn entity_exists(&self, entity_id: &ObjectId) -> bool {
        self.entities.contains_key(entity_id)
    }

    /// 文档中的实体总数。
    #[inline]
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// 全部图层的只读视图，包含新建文档时预置的两个默认图层。
    #[inline]
    pub fn layers(&self) -> &HashMap<ObjectId, Layer> {
        &self.layers
    }

    /// 全部图层的可变视图，绕过 [`Document::add_layer`] 直接改写不会做任何校验。
    #[inline]
    pub fn layers_mut(&mut self) -> &mut HashMap<ObjectId, Layer> {
        &mut self.layers
    }

    /// 登记一个图层，会修改 `self`。
    ///
    /// - `layer`：待加入的图层。
    ///
    /// 返回该图层的 id，可赋给实体的 `layer_id` 完成归属。同 id 图层会被覆盖。
    #[inline]
    pub fn add_layer(&mut self, layer: Layer) -> ObjectId {
        let id = layer.id().clone();
        self.layers.insert(id.clone(), layer);
        id
    }

    /// 按 id 删除图层，会修改 `self`。
    ///
    /// 返回是否真正删除；本方法不会解除其他图层或实体对该 id 的引用，仍使用该图层的
    /// 实体将查找失败，调用方需自行迁移。
    #[inline]
    pub fn remove_layer(&mut self, layer_id: &ObjectId) -> bool {
        self.layers.remove(layer_id).is_some()
    }

    /// 按 id 查询图层，不存在时返回 [`None`]。
    #[inline]
    pub fn get_layer(&self, layer_id: &ObjectId) -> Option<&Layer> {
        self.layers.get(layer_id)
    }

    /// 文档中的图层总数；新建文档时为 2（两个默认图层）。
    #[inline]
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// 全部块定义的只读视图。
    #[inline]
    pub fn blocks(&self) -> &HashMap<ObjectId, Block> {
        &self.blocks
    }

    /// 登记一个块定义，会修改 `self`。
    ///
    /// - `block`：待加入的块定义。
    ///
    /// 返回该块的 id，作为块参照的引用目标。同 id 块会被覆盖。
    #[inline]
    pub fn add_block(&mut self, block: Block) -> ObjectId {
        let id = block.id().clone();
        self.blocks.insert(id.clone(), block);
        id
    }

    /// 按 id 查询块定义，不存在时返回 [`None`]。
    #[inline]
    pub fn get_block(&self, block_id: &ObjectId) -> Option<&Block> {
        self.blocks.get(block_id)
    }

    /// 文档中的块定义总数，不含块参照。
    #[inline]
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    /// 全部块参照的只读视图。
    #[inline]
    pub fn block_references(&self) -> &HashMap<ObjectId, BlockReference> {
        &self.block_references
    }

    /// 登记一个块参照，会修改 `self`。
    ///
    /// - `block_ref`：待加入的块参照，其 `block_id` 应指向已存在的块定义。
    ///
    /// 返回该块参照的 id。同 id 会被覆盖。
    #[inline]
    pub fn add_block_reference(&mut self, block_ref: BlockReference) -> ObjectId {
        let id = block_ref.id().clone();
        self.block_references.insert(id.clone(), block_ref);
        id
    }

    /// 按 id 查询块参照，不存在时返回 [`None`]。
    #[inline]
    pub fn get_block_reference(&self, block_ref_id: &ObjectId) -> Option<&BlockReference> {
        self.block_references.get(block_ref_id)
    }

    /// 文档中的块参照总数。
    #[inline]
    pub fn block_reference_count(&self) -> usize {
        self.block_references.len()
    }

    /// 模型空间的标识；该 id 同时是默认 `"ModelSpace"` 图层的 id。
    #[inline]
    pub fn model_space(&self) -> &ObjectId {
        &self.model_space
    }

    /// 图纸空间的标识；该 id 同时是默认 `"PaperSpace"` 图层的 id。
    #[inline]
    pub fn paper_space(&self) -> &ObjectId {
        &self.paper_space
    }

    /// 当前活动空间。
    #[inline]
    pub fn active_space(&self) -> SpaceType {
        self.active_space
    }

    /// 切换当前活动空间，会修改 `self`；仅改变后续操作的默认目标空间。
    #[inline]
    pub fn set_active_space(&mut self, space: SpaceType) {
        self.active_space = space;
    }

    /// 文档级自定义属性的只读视图，不参与几何运算。
    #[inline]
    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    /// 写入或覆盖一个文档级属性，会修改 `self`。
    ///
    /// - `key`：属性名，同名时旧值被覆盖；
    /// - `value`：属性值。
    #[inline]
    pub fn set_property(&mut self, key: String, value: String) {
        self.properties.insert(key, value);
    }

    /// 全文档包围盒，返回 `(最小角点, 最大角点)`，Z 分量固定为 0。
    ///
    /// 只累加能为自身提供包围盒的实体（点、直线、圆），其余实体（圆弧、椭圆、折线、样条、
    /// 文本、标注、填充等）被忽略；文档为空或所有实体都不提供包围盒时返回 [`None`]。
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
