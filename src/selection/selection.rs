use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SelectionMode {
    Point,
    Window,
    Crossing,
    Fence,
    All,
    Previous,
    Last,
    Implied,
    None,
}

impl Default for SelectionMode {
    fn default() -> Self {
        SelectionMode::Point
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SelectionOption {
    Add,
    Remove,
    Single,
    Multiple,
    Verbose,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectionFilter {
    pub entity_types: Vec<String>,
    pub layers: Vec<String>,
    pub colors: Vec<(u8, u8, u8)>,
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
    pub fn all() -> Self {
        Self::default()
    }

    pub fn with_entity_types(types: Vec<String>) -> Self {
        Self {
            entity_types: types,
            ..Default::default()
        }
    }

    pub fn with_layers(layers: Vec<String>) -> Self {
        Self {
            layers,
            ..Default::default()
        }
    }

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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_mode(mode: SelectionMode) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }

    pub fn add(&mut self, entity_id: super::super::data_structure::ObjectId) {
        if !self.entities.contains(&entity_id) {
            self.entities.push(entity_id.clone());
            self.last_selected = Some(entity_id);
            self.selection_time = std::time::SystemTime::now();
        }
    }

    pub fn add_multiple(&mut self, entity_ids: &[super::super::data_structure::ObjectId]) {
        for id in entity_ids {
            self.add(id.clone());
        }
    }

    pub fn remove(&mut self, entity_id: &super::super::data_structure::ObjectId) {
        self.entities.retain(|id| id != entity_id);
    }

    pub fn remove_multiple(&mut self, entity_ids: &[super::super::data_structure::ObjectId]) {
        for id in entity_ids {
            self.remove(id);
        }
    }

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
