use serde::{Serialize, Deserialize};
use std::fmt;

/// 选择方式，决定如何由拾取点（世界坐标）构造选择结果。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SelectionMode {
    /// 点选：命中拾取点的单个实体。
    Point,
    /// 窗口选择：只选中完全落在选择框内的实体，相交但未全含者不算。
    Window,
    /// 交叉选择：与选择框相交或落在框内的实体都算。
    Crossing,
    /// 栏选：与折线栏相交的实体。
    Fence,
    /// 全选：文档中全部实体。
    All,
    /// 上一次的选择集。
    Previous,
    /// 最近创建的一个实体。
    Last,
    /// 隐含选择：由其他操作推断出的实体。
    Implied,
    /// 不使用选择，返回空结果。
    None,
}

impl Default for SelectionMode {
    fn default() -> Self {
        SelectionMode::Point
    }
}

/// 选择操作选项，用于调整后续选择行为。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SelectionOption {
    /// 把结果加入现有选择集。
    Add,
    /// 从选择集中移除结果。
    Remove,
    /// 只允许选中一个实体。
    Single,
    /// 允许选中多个实体。
    Multiple,
    /// 输出选择过程的详细信息。
    Verbose,
}

/// 选择过滤器：限制可被选中的实体类型、图层与颜色。
///
/// 各字段为空表示该维度不加限制；多个维度同时给出时按「与」关系判定，
/// 任一维不满足即整体不匹配。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectionFilter {
    /// 允许的实体类型名，需与 `EntityType` 的调试名一致（如 `"Line"`）；空表示不限。
    pub entity_types: Vec<String>,
    /// 允许的图层标识字符串（`ObjectId::to_string` 的结果）；空表示不限。
    pub layers: Vec<String>,
    /// 允许的 RGB 颜色（每通道 0–255），与实体属性 `color` 的 `"r,g,b"` 文本比较；空表示不限。
    pub colors: Vec<(u8, u8, u8)>,
    /// 允许的线型名；当前匹配逻辑尚未使用该字段。
    pub linetypes: Vec<String>,
}

impl Default for SelectionFilter {
    fn default() -> Self {
        Self {
            entity_types: Vec::new(),
            layers: Vec::new(),
            colors: Vec::new(),
            linetypes: Vec::new(),
        }
    }
}

impl SelectionFilter {
    /// 创建不做任何限制的过滤器（各字段为空，可匹配任意实体）。
    pub fn all() -> Self {
        Self::default()
    }

    /// 创建只按实体类型过滤的过滤器，其余维度不限。
    ///
    /// - `types`：允许的类型名，需与 `EntityType` 的调试名完全一致（如 `"Line"`）。
    pub fn with_entity_types(types: Vec<String>) -> Self {
        Self {
            entity_types: types,
            ..Default::default()
        }
    }

    /// 创建只按图层过滤的过滤器，其余维度不限。
    ///
    /// - `layers`：允许的图层标识字符串（`ObjectId::to_string` 的结果）。
    pub fn with_layers(layers: Vec<String>) -> Self {
        Self {
            layers,
            ..Default::default()
        }
    }

    /// 判断实体是否通过过滤器。
    ///
    /// 类型、图层、颜色三个维度全部满足才返回 `true`；某维度列表为空表示不限制。
    /// 颜色维度要求实体属性 `color` 形如 `"r,g,b"` 且三个分量都能解析为 `u8`，否则视为不匹配。
    pub fn matches(&self, entity: &super::super::data_structure::Entity) -> bool {
        if !self.entity_types.is_empty() {
            let entity_type = format!("{:?}", entity.entity_type);
            if !self.entity_types.contains(&entity_type) {
                return false;
            }
        }

        if !self.layers.is_empty() {
            let layer = entity.layer_id.to_string();
            if !self.layers.contains(&layer) {
                return false;
            }
        }

        if !self.colors.is_empty() {
            let matched = entity.properties.get("color").and_then(|color| {
                let parts: Vec<&str> = color.split(',').collect();
                if parts.len() == 3 {
                    let r = parts[0].trim().parse::<u8>().ok()?;
                    let g = parts[1].trim().parse::<u8>().ok()?;
                    let b = parts[2].trim().parse::<u8>().ok()?;
                    Some((r, g, b))
                } else {
                    None
                }
            });
            if !matches!(matched, Some(rgb) if self.colors.contains(&rgb)) {
                return false;
            }
        }

        true
    }
}

/// 选择集：一组实体标识（`ObjectId`）、选择方式与选中时间。
///
/// 只保存标识、不持有实体数据；重复加入同一标识会被忽略。`last_selected`
/// 记录最近加入的实体，供「上一次选中」类命令使用。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectionSet {
    entities: Vec<super::super::data_structure::ObjectId>,
    mode: SelectionMode,
    last_selected: Option<super::super::data_structure::ObjectId>,
    selection_time: std::time::SystemTime,
}

impl Default for SelectionSet {
    fn default() -> Self {
        Self {
            entities: Vec::new(),
            mode: SelectionMode::Point,
            last_selected: None,
            selection_time: std::time::SystemTime::now(),
        }
    }
}

impl SelectionSet {
    /// 创建空选择集，选择方式为 [`SelectionMode::Point`]。
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建空选择集并指定选择方式。
    pub fn with_mode(mode: SelectionMode) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }

    /// 加入一个实体标识；已在集合中时不做任何改动。
    ///
    /// 副作用：成功加入时把它记为「最后选中」并刷新选中时间。
    pub fn add(&mut self, entity_id: super::super::data_structure::ObjectId) {
        if !self.entities.contains(&entity_id) {
            self.entities.push(entity_id.clone());
            self.last_selected = Some(entity_id);
            self.selection_time = std::time::SystemTime::now();
        }
    }

    /// 批量加入实体，逐个走 [`SelectionSet::add`] 的去重逻辑。
    pub fn add_multiple(&mut self, entity_ids: &[super::super::data_structure::ObjectId]) {
        for id in entity_ids {
            self.add(id.clone());
        }
    }

    /// 移除指定实体；该实体不在集合中时静默无操作。
    pub fn remove(&mut self, entity_id: &super::super::data_structure::ObjectId) {
        self.entities.retain(|id| id != entity_id);
    }

    /// 批量移除实体。
    pub fn remove_multiple(&mut self, entity_ids: &[super::super::data_structure::ObjectId]) {
        for id in entity_ids {
            self.remove(id);
        }
    }

    /// 清空选择集并清除「最后选中」记录（选中时间保持不变）。
    pub fn clear(&mut self) {
        self.entities.clear();
        self.last_selected = None;
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn count(&self) -> usize {
        self.entities.len()
    }

    pub fn get_selected(&self) -> &[super::super::data_structure::ObjectId] {
        &self.entities
    }

    pub fn get_last_selected(&self) -> Option<&super::super::data_structure::ObjectId> {
        self.last_selected.as_ref()
    }

    pub fn contains(&self, entity_id: &super::super::data_structure::ObjectId) -> bool {
        self.entities.contains(entity_id)
    }

    pub fn toggle(&mut self, entity_id: super::super::data_structure::ObjectId) {
        if self.contains(&entity_id) {
            self.remove(&entity_id);
        } else {
            self.add(entity_id);
        }
    }

    pub fn set_mode(&mut self, mode: SelectionMode) {
        self.mode = mode;
    }

    pub fn get_mode(&self) -> SelectionMode {
        self.mode
    }

    pub fn get_selection_time(&self) -> std::time::SystemTime {
        self.selection_time
    }

    pub fn select_all(&mut self, entity_ids: &[super::super::data_structure::ObjectId]) {
        self.entities = entity_ids.to_vec();
        self.last_selected = entity_ids.last().cloned();
        self.selection_time = std::time::SystemTime::now();
    }

    pub fn select_invert(&mut self, all_entity_ids: &[super::super::data_structure::ObjectId]) {
        let current: std::collections::HashSet<_> = self.entities.iter().collect();
        self.entities = all_entity_ids
            .iter()
            .filter(|id| !current.contains(id))
            .cloned()
            .collect();
    }
}

impl fmt::Display for SelectionSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SelectionSet(count={}, mode={})",
            self.entities.len(),
            format!("{:?}", self.mode)
        )
    }
}

pub trait EntitySelector {
    fn select(&self, mode: SelectionMode, points: &[crate::geometry::Point], document: &super::super::data_structure::Document, filter: Option<&SelectionFilter>) -> Vec<super::super::data_structure::ObjectId>;
    fn deselect(&mut self, entity_ids: &[super::super::data_structure::ObjectId]);
    fn clear(&mut self);
    fn get_selected(&self) -> &[super::super::data_structure::ObjectId];
}

pub struct SelectionManager {
    selection_sets: std::collections::HashMap<String, SelectionSet>,
    current_set_name: String,
    selection_preview: Option<Vec<super::super::data_structure::ObjectId>>,
    last_selection_mode: SelectionMode,
}

impl Default for SelectionManager {
    fn default() -> Self {
        Self {
            selection_sets: std::collections::HashMap::new(),
            current_set_name: "DEFAULT".to_string(),
            selection_preview: None,
            last_selection_mode: SelectionMode::Point,
        }
    }
}

impl SelectionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_selection_set(&mut self, name: &str) -> bool {
        if !self.selection_sets.contains_key(name) {
            self.selection_sets.insert(name.to_string(), SelectionSet::new());
            true
        } else {
            false
        }
    }

    pub fn switch_selection_set(&mut self, name: &str) -> bool {
        if self.selection_sets.contains_key(name) {
            self.current_set_name = name.to_string();
            true
        } else {
            false
        }
    }

    pub fn get_current_selection_set(&self) -> Option<&SelectionSet> {
        self.selection_sets.get(&self.current_set_name)
    }

    pub fn get_current_selection_set_mut(&mut self) -> Option<&mut SelectionSet> {
        self.selection_sets.get_mut(&self.current_set_name)
    }

    pub fn add_to_selection(&mut self, entity_id: super::super::data_structure::ObjectId) {
        if let Some(set) = self.selection_sets.get_mut(&self.current_set_name) {
            set.add(entity_id);
        }
    }

    pub fn remove_from_selection(&mut self, entity_id: &super::super::data_structure::ObjectId) {
        if let Some(set) = self.selection_sets.get_mut(&self.current_set_name) {
            set.remove(entity_id);
        }
    }

    pub fn clear_selection(&mut self) {
        if let Some(set) = self.selection_sets.get_mut(&self.current_set_name) {
            set.clear();
        }
    }

    pub fn get_selected_entities(&self) -> &[super::super::data_structure::ObjectId] {
        if let Some(set) = self.selection_sets.get(&self.current_set_name) {
            set.get_selected()
        } else {
            &[]
        }
    }

    pub fn get_all_selection_sets(&self) -> Vec<&str> {
        self.selection_sets.keys().map(|s| s.as_str()).collect()
    }

    pub fn set_selection_preview(&mut self, entity_ids: Option<Vec<super::super::data_structure::ObjectId>>) {
        self.selection_preview = entity_ids;
    }

    pub fn get_selection_preview(&self) -> Option<&[super::super::data_structure::ObjectId]> {
        self.selection_preview.as_ref().map(|v| v.as_slice())
    }

    pub fn confirm_selection(&mut self) {
        if let Some(preview) = &self.selection_preview {
            if let Some(set) = self.selection_sets.get_mut(&self.current_set_name) {
                set.add_multiple(preview);
            }
        }
        self.selection_preview = None;
    }

    pub fn cancel_selection_preview(&mut self) {
        self.selection_preview = None;
    }

    pub fn last_selection_mode(&self) -> SelectionMode {
        self.last_selection_mode
    }

    pub fn set_last_selection_mode(&mut self, mode: SelectionMode) {
        self.last_selection_mode = mode;
    }
}

impl fmt::Display for SelectionManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SelectionManager(sets={}, current={})",
            self.selection_sets.len(),
            self.current_set_name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_structure::{ObjectId, Entity, EntityType, EntityGeometry};
    use crate::geometry::Point;

    #[test]
    fn test_selection_set() {
        let mut set = SelectionSet::new();

        let id1 = ObjectId::new();
        let id2 = ObjectId::new();

        set.add(id1.clone());
        assert_eq!(set.count(), 1);

        set.add(id2.clone());
        assert_eq!(set.count(), 2);

        set.remove(&id1);
        assert_eq!(set.count(), 1);

        assert!(set.contains(&id2));
        assert!(!set.contains(&id1));
    }

    #[test]
    fn test_selection_filter() {
        let filter = SelectionFilter::with_entity_types(vec!["Line".to_string(), "Circle".to_string()]);

        let line = Entity::new(EntityType::Line, EntityGeometry::Point(Point::origin()));
        let arc = Entity::new(EntityType::Arc, EntityGeometry::Point(Point::origin()));

        assert!(filter.matches(&line));
        assert!(!filter.matches(&arc));
    }
}
