//! 选择集与选择管理：记录"哪些实体被选中"。
//!
//! [`SelectionSet`] 以 [`ObjectId`] 的哈希集合保存一批实体的引用，只存 id 不存实体本身，
//! 因此实体被文档删除后选择集中仍可能残留失效 id；集合天然去重，重复加入同一 id 不改变
//! 元素个数。[`SelectionManager`] 在其上增加了"当前选择 + 命名存档"的管理：可把当前选择
//! 存为具名选择集并在之后恢复。

use std::collections::HashSet;
use super::ObjectId;

/// 一组被选中实体的标识集合，元素唯一、无序。
#[derive(Debug, Clone)]
pub struct SelectionSet {
    entity_ids: HashSet<ObjectId>,
    name: String,
    description: String,
}

impl SelectionSet {
    /// 新建空选择集，名称与描述均为空字符串。
    #[inline]
    pub fn new() -> Self {
        Self {
            entity_ids: HashSet::new(),
            name: String::new(),
            description: String::new(),
        }
    }

    /// 新建空选择集并指定名称，用于存档后按名恢复。
    ///
    /// - `name`：选择集名称。
    #[inline]
    pub fn with_name(name: String) -> Self {
        Self {
            entity_ids: HashSet::new(),
            name,
            description: String::new(),
        }
    }

    /// 已选中实体标识的只读视图，可交给渲染或删除等操作遍历。
    #[inline]
    pub fn entity_ids(&self) -> &HashSet<ObjectId> {
        &self.entity_ids
    }

    /// 选择集是否为空。
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entity_ids.is_empty()
    }

    /// 已选中实体的个数，重复加入的 id 只计一次。
    #[inline]
    pub fn count(&self) -> usize {
        self.entity_ids.len()
    }

    /// 加入一个实体，会修改 `self`；已存在时保持原状。
    #[inline]
    pub fn add(&mut self, entity_id: ObjectId) {
        self.entity_ids.insert(entity_id);
    }

    /// 批量加入实体，会修改 `self`；已有元素不受影响。
    ///
    /// - `entity_ids`：待加入的标识切片，按值复制入集合，调用方仍保留原数据。
    #[inline]
    pub fn add_multiple(&mut self, entity_ids: &[ObjectId]) {
        for id in entity_ids {
            self.entity_ids.insert(id.clone());
        }
    }

    /// 从选择集中移除一个实体，会修改 `self`。
    ///
    /// 返回该实体此前是否在集合中；不在集合中时返回 `false`。
    #[inline]
    pub fn remove(&mut self, entity_id: &ObjectId) -> bool {
        self.entity_ids.remove(entity_id)
    }

    /// 清空选择集，会修改 `self`；名称与描述保留不变。
    #[inline]
    pub fn clear(&mut self) {
        self.entity_ids.clear();
    }

    /// 判断指定实体是否已被选中。
    #[inline]
    pub fn contains(&self, entity_id: &ObjectId) -> bool {
        self.entity_ids.contains(entity_id)
    }

    /// 求并集，返回新选择集，不修改 `self` 与 `other`。
    ///
    /// 结果沿用 `self` 的名称与描述。
    #[inline]
    pub fn union(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.entity_ids.extend(other.entity_ids.iter().cloned());
        result
    }

    /// 求交集，返回新选择集，不修改 `self` 与 `other`。
    ///
    /// 只保留同时存在于两者中的实体；结果沿用 `self` 的名称与描述。
    #[inline]
    pub fn intersection(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.entity_ids.retain(|id| other.entity_ids.contains(id));
        result
    }

    /// 求差集，返回新选择集，不修改 `self` 与 `other`。
    ///
    /// 保留在 `self` 中但不在 `other` 中的实体；结果沿用 `self` 的名称与描述。
    #[inline]
    pub fn difference(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.entity_ids.retain(|id| !other.entity_ids.contains(id));
        result
    }

    /// 选择集名称。
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 设置选择集名称，会修改 `self`。
    #[inline]
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// 选择集描述，未设置时为空字符串。
    #[inline]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// 设置选择集描述，会修改 `self`。
    #[inline]
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }
}

impl Default for SelectionSet {
    fn default() -> Self {
        Self::new()
    }
}

/// 选择管理器：维护"当前选择"并支持把选择存档为具名选择集。
#[derive(Debug, Clone)]
pub struct SelectionManager {
    selection_sets: Vec<SelectionSet>,
    current_selection: SelectionSet,
}

impl SelectionManager {
    /// 新建选择管理器，当前选择为空，已存档的选择集列表为空。
    #[inline]
    pub fn new() -> Self {
        Self {
            selection_sets: Vec::new(),
            current_selection: SelectionSet::new(),
        }
    }

    /// 当前选择的只读引用。
    #[inline]
    pub fn current_selection(&self) -> &SelectionSet {
        &self.current_selection
    }

    /// 当前选择的可变引用，可直接批量改写，会修改 `self`。
    #[inline]
    pub fn current_selection_mut(&mut self) -> &mut SelectionSet {
        &mut self.current_selection
    }

    /// 把单个实体加入当前选择，会修改 `self`；重复选择同一实体不产生变化。
    #[inline]
    pub fn select(&mut self, entity_id: ObjectId) {
        self.current_selection.add(entity_id);
    }

    /// 把多个实体加入当前选择，会修改 `self`。
    ///
    /// - `entity_ids`：待选中的标识切片。
    #[inline]
    pub fn select_multiple(&mut self, entity_ids: &[ObjectId]) {
        self.current_selection.add_multiple(entity_ids);
    }

    /// 从当前选择中移除一个实体，会修改 `self`；实体本就不在选择中时静默无操作。
    #[inline]
    pub fn deselect(&mut self, entity_id: &ObjectId) {
        self.current_selection.remove(entity_id);
    }

    /// 清空当前选择，会修改 `self`；已存档的选择集不受影响。
    #[inline]
    pub fn clear_selection(&mut self) {
        self.current_selection.clear();
    }

    /// 把当前选择存档为具名选择集并清空当前选择，会修改 `self`。
    ///
    /// - `name`：存档名，注意名称写在被存入的那份副本上，当前选择随后被重置为新的空集。
    ///
    /// 本方法无返回值，也不检查重名，同名存档会累积多条。
    #[inline]
    pub fn save_selection(&mut self, name: String) {
        self.selection_sets.push(self.current_selection.clone());
        self.current_selection.set_name(name);
        self.current_selection = SelectionSet::new();
    }

    /// 按名称载入已存档的选择集，会覆盖当前选择并修改 `self`。
    ///
    /// - `name`：存档名，按名称精确匹配，取最先命中的一个。
    ///
    /// 返回载入后的当前选择引用；没有同名存档时返回 [`None`]，此时当前选择保持不变。
    #[inline]
    pub fn load_selection(&mut self, name: &str) -> Option<&SelectionSet> {
        for selection in &self.selection_sets {
            if selection.name() == name {
                self.current_selection = selection.clone();
                return Some(&self.current_selection);
            }
        }
        None
    }
}

impl Default for SelectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_set_creation() {
        let selection = SelectionSet::new();
        
        assert!(selection.is_empty());
        assert_eq!(selection.count(), 0);
    }

    #[test]
    fn test_selection_set_add() {
        let mut selection = SelectionSet::new();
        let id = ObjectId::new();
        
        selection.add(id.clone());
        assert_eq!(selection.count(), 1);
        assert!(selection.contains(&id));
    }

    #[test]
    fn test_selection_set_remove() {
        let mut selection = SelectionSet::new();
        let id = ObjectId::new();
        
        selection.add(id.clone());
        selection.remove(&id);
        assert!(selection.is_empty());
    }

    #[test]
    fn test_selection_manager() {
        let mut manager = SelectionManager::new();
        
        manager.select(ObjectId::new());
        assert_eq!(manager.current_selection().count(), 1);
        
        manager.clear_selection();
        assert!(manager.current_selection().is_empty());
    }
}
