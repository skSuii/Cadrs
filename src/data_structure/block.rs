//! 块定义：可被多次插入复用的实体集合。
//!
//! [`Block`] 以自身坐标系保存一组实体，并记录插入基点 `origin`；在图纸中出现的是指向块的
//! [`super::block_reference::BlockReference`]，同一块可被插入任意多次。块内实体不直接参与
//! 文档的实体表统计，须经由块参照展开后才出现在图纸上。

use super::entity_id::ObjectId;
use super::layer::Layer;
use super::super::geometry::Point;
use super::entity::Entity;

/// 块定义：一组实体及其插入基点，可被块参照多次引用。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Block {
    id: ObjectId,
    name: String,
    entities: Vec<Entity>,
    origin: Point,
    description: String,
}

impl Block {
    /// 新建空块定义。
    ///
    /// 实体列表为空、插入基点为坐标原点 [`Point::origin`]、描述为空字符串；id 自动生成，
    /// 需通过 [`super::document::Document::add_block`] 加入文档后才能被块参照引用。
    ///
    /// - `name`：块名，块参照按名称或 id 引用本块，本方法不做重名校验。
    #[inline]
    pub fn new(name: String) -> Self {
        Self {
            id: ObjectId::new(),
            name,
            entities: Vec::new(),
            origin: Point::origin(),
            description: String::new(),
        }
    }

    /// 块的唯一标识，作为文档块表的键，也是块参照 `block_id` 的取值。
    #[inline]
    pub fn id(&self) -> &ObjectId {
        &self.id
    }

    /// 块名，可被块参照按名称引用。
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 重命名块，会修改 `self`；已按旧名引用的块参照可能因此无法解析。
    #[inline]
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// 插入基点，即块被插入时与插入点对齐的块内坐标，默认原点。
    #[inline]
    pub fn origin(&self) -> Point {
        self.origin
    }

    /// 设置插入基点，会修改 `self`；改变基点会使既有块参照的显示位置整体平移。
    #[inline]
    pub fn set_origin(&mut self, origin: Point) {
        self.origin = origin;
    }

    /// 块内实体数量。
    #[inline]
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// 块描述，未设置时为空字符串。
    #[inline]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// 设置块描述，会修改 `self`。
    #[inline]
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    /// 向块内追加实体，会修改 `self`。
    ///
    /// 采用追加语义，不做 id 去重；同一实体被加入多个块时各处共享同一 id。
    #[inline]
    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    /// 块内实体的只读切片，顺序为加入顺序。
    #[inline]
    pub fn get_entities(&self) -> &[Entity] {
        &self.entities
    }

    /// 按 id 查找块内实体，不存在时返回 [`None`]。
    #[inline]
    pub fn get_entity(&self, id: &ObjectId) -> Option<&Entity> {
        self.entities.iter().find(|e| e.id() == id)
    }

    /// 按 id 删除块内实体，会修改 `self`。
    ///
    /// 返回是否删除成功：id 不存在时返回 `false`，此时块内容不变。
    #[inline]
    pub fn remove_entity(&mut self, id: &ObjectId) -> bool {
        if let Some(index) = self.entities.iter().position(|e| e.id() == id) {
            self.entities.remove(index);
            true
        } else {
            false
        }
    }
}
