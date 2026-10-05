use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub description: String,
    pub members: Vec<super::data_structure::ObjectId>,
    pub is_selectable: bool,
    pub is_undoable: bool,
    pub is_highlighted: bool,
}

impl Default for Group {
    fn default() -> Self {
        Self::new()
    }
}

impl Group {
    #[inline]
    pub fn new() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            members: Vec::new(),
            is_selectable: true,
            is_undoable: true,
            is_highlighted: false,
        }
    }

    #[inline]
    pub fn with_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    #[inline]
    pub fn add(&mut self, object_id: super::data_structure::ObjectId) -> bool {
        if !self.members.contains(&object_id) {
            self.members.push(object_id);
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn add_multiple(&mut self, object_ids: &[super::data_structure::ObjectId]) -> usize {
        let mut added = 0;
        for id in object_ids {
            if self.add(id.clone()) {
                added += 1;
            }
        }
        added
    }

    #[inline]
    pub fn remove(&mut self, object_id: &super::data_structure::ObjectId) -> bool {
        let len = self.members.len();
        self.members.retain(|id| id != object_id);
        self.members.len() != len
    }

    #[inline]
    pub fn remove_at(&mut self, index: usize) -> bool {
        if index < self.members.len() {
            self.members.remove(index);
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn contains(&self, object_id: &super::data_structure::ObjectId) -> bool {
        self.members.contains(object_id)
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&super::data_structure::ObjectId> {
        self.members.get(index)
    }

    #[inline]
    pub fn clear(&mut self) {
        self.members.clear();
    }

    #[inline]
    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    #[inline]
    pub fn set_selectable(&mut self, selectable: bool) {
        self.is_selectable = selectable;
    }

    #[inline]
    pub fn set_undoable(&mut self, undoable: bool) {
        self.is_undoable = undoable;
    }

    #[inline]
    pub fn set_highlighted(&mut self, highlighted: bool) {
        self.is_highlighted = highlighted;
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &super::data_structure::ObjectId> {
        self.members.iter()
    }

    #[inline]
    pub fn members(&self) -> &[super::data_structure::ObjectId] {
        &self.members
    }

    #[inline]
    pub fn members_mut(&mut self) -> &mut Vec<super::data_structure::ObjectId> {
        &mut self.members
    }
}

impl Hash for Group {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Group({}): {} members", self.name, self.members.len())
    }
}

#[derive(Debug, Clone)]
pub struct GroupManager {
    groups: std::collections::HashMap<String, Group>,
    unnamed_group_counter: u32,
    is_grouping_enabled: bool,
    is_group_pick_style_enabled: bool,
}

impl Default for GroupManager {
    fn default() -> Self {
        Self::new()
    }
}

impl GroupManager {
    #[inline]
    pub fn new() -> Self {
        Self {
            groups: std::collections::HashMap::new(),
            unnamed_group_counter: 0,
            is_grouping_enabled: true,
            is_group_pick_style_enabled: true,
        }
    }

    #[inline]
    pub fn create(&mut self, name: Option<&str>) -> &mut Group {
        let group_name = if let Some(n) = name {
            n.to_string()
        } else {
            self.unnamed_group_counter += 1;
            format!("*GROUP{}", self.unnamed_group_counter)
        };
        let group = Group::with_name(&group_name);
        self.groups.insert(group_name.clone(), group);
        self.groups.get_mut(&group_name).unwrap()
    }

    #[inline]
    pub fn create_from_selection(
        &mut self,
        name: Option<&str>,
        object_ids: &[super::data_structure::ObjectId],
    ) -> String {
        let group = self.create(name);
        group.add_multiple(object_ids);
        group.name.clone()
    }

    #[inline]
    pub fn get(&self, name: &str) -> Option<&Group> {
        self.groups.get(name)
    }

    #[inline]
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Group> {
        self.groups.get_mut(name)
    }

    #[inline]
    pub fn has(&self, name: &str) -> bool {
        self.groups.contains_key(name)
    }

    #[inline]
    pub fn find_group(&self, object_id: &super::data_structure::ObjectId) -> Option<&str> {
        self.groups.iter()
            .find(|(_, group)| group.contains(object_id))
            .map(|(name, _)| name.as_str())
    }

    #[inline]
    pub fn find_groups(&self, object_id: &super::data_structure::ObjectId) -> Vec<&str> {
        self.groups.iter()
            .filter(|(_, group)| group.contains(object_id))
            .map(|(name, _)| name.as_str())
            .collect()
    }

    #[inline]
    pub fn rename(&mut self, old_name: &str, new_name: &str) -> bool {
        if self.groups.contains_key(new_name) {
            return false;
        }
        if let Some(group) = self.groups.remove(old_name) {
            let mut new_group = group;
            new_group.name = new_name.to_string();
            self.groups.insert(new_name.to_string(), new_group);
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn remove(&mut self, name: &str) -> bool {
        self.groups.remove(name).is_some()
    }

    #[inline]
    pub fn remove_group(&mut self, name: &str) -> Option<Group> {
        self.groups.remove(name)
    }

    #[inline]
    pub fn clear(&mut self) {
        self.groups.clear();
        self.unnamed_group_counter = 0;
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.groups.len()
    }

    #[inline]
    pub fn names(&self) -> Vec<&str> {
        self.groups.keys().map(|s| s.as_str()).collect()
    }

    #[inline]
    pub fn groups(&self) -> &std::collections::HashMap<String, Group> {
        &self.groups
    }

    #[inline]
    pub fn groups_mut(&mut self) -> &mut std::collections::HashMap<String, Group> {
        &mut self.groups
    }

    #[inline]
    pub fn set_grouping_enabled(&mut self, enabled: bool) {
        self.is_grouping_enabled = enabled;
    }

    #[inline]
    pub fn is_grouping_enabled(&self) -> bool {
        self.is_grouping_enabled
    }

    #[inline]
    pub fn set_group_pick_style_enabled(&mut self, enabled: bool) {
        self.is_group_pick_style_enabled = enabled;
    }

    #[inline]
    pub fn is_group_pick_style_enabled(&self) -> bool {
        self.is_group_pick_style_enabled
    }

    #[inline]
    pub fn add_to_group(
        &mut self,
        group_name: &str,
        object_id: super::data_structure::ObjectId,
    ) -> bool {
        if let Some(group) = self.groups.get_mut(group_name) {
            group.add(object_id)
        } else {
            false
        }
    }

    #[inline]
    pub fn remove_from_group(
        &mut self,
        group_name: &str,
        object_id: &super::data_structure::ObjectId,
    ) -> bool {
        if let Some(group) = self.groups.get_mut(group_name) {
            group.remove(object_id)
        } else {
            false
        }
    }

    #[inline]
    pub fn select_group(&self, name: &str) -> Option<&Group> {
        self.get(name)
    }

    #[inline]
    pub fn select_group_mut(&mut self, name: &str) -> Option<&mut Group> {
        self.get_mut(name)
    }

    #[inline]
    pub fn deselect_all(&mut self) {
        for group in self.groups.values_mut() {
            group.set_selectable(true);
        }
    }

    #[inline]
    pub fn lock_all(&mut self) {
        for group in self.groups.values_mut() {
            group.set_selectable(false);
        }
    }

    #[inline]
    pub fn highlight_all(&mut self) {
        for group in self.groups.values_mut() {
            group.set_highlighted(true);
        }
    }

    #[inline]
    pub fn clear_highlights(&mut self) {
        for group in self.groups.values_mut() {
            group.set_highlighted(false);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupProxy {
    pub group_name: String,
    pub member_index: usize,
    pub object_id: super::data_structure::ObjectId,
}

impl Default for GroupProxy {
    fn default() -> Self {
        Self {
            group_name: String::new(),
            member_index: 0,
            object_id: super::data_structure::ObjectId::default(),
        }
    }
}

impl GroupProxy {
    #[inline]
    pub fn new(group_name: &str, member_index: usize, object_id: super::data_structure::ObjectId) -> Self {
        Self {
            group_name: group_name.to_string(),
            member_index,
            object_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_structure::ObjectId;

    #[test]
    fn test_group_creation() {
        let group = Group::new();
        assert!(group.is_empty());
        assert_eq!(group.member_count(), 0);
    }

    #[test]
    fn test_group_operations() {
        let mut group = Group::new();
        let id1 = ObjectId::new();
        let id2 = ObjectId::new();
        let id3 = ObjectId::new();

        assert!(group.add(id1.clone()));
        assert!(group.add(id2));
        assert!(!group.add(id1.clone())); // duplicate
        assert_eq!(group.member_count(), 2);

        assert!(group.contains(&id1));
        assert!(!group.contains(&id3));

        assert!(group.remove(&id1));
        assert!(!group.remove(&id1)); // already removed
        assert_eq!(group.member_count(), 1);
    }

    #[test]
    fn test_group_clear() {
        let mut group = Group::new();
        group.add(ObjectId::new());
        group.add(ObjectId::new());
        assert_eq!(group.member_count(), 2);
        group.clear();
        assert!(group.is_empty());
    }

    #[test]
    fn test_group_manager() {
        let mut manager = GroupManager::new();
        let group = manager.create(Some("TestGroup"));
        assert_eq!(group.name, "TestGroup");
    }

    #[test]
    fn test_group_manager_operations() {
        let mut manager = GroupManager::new();
        let id1 = ObjectId::new();
        let id2 = ObjectId::new();

        let group_name = manager.create_from_selection(Some("MyGroup"), &[id1, id2]);
        assert_eq!(group_name, "MyGroup");

        assert!(manager.has("MyGroup"));
        assert_eq!(manager.count(), 1);

        let group = manager.get("MyGroup").unwrap();
        assert_eq!(group.member_count(), 2);
    }

    #[test]
    fn test_group_manager_find() {
        let mut manager = GroupManager::new();
        let id1 = ObjectId::new();
        let id2 = ObjectId::new();

        manager.create_from_selection(Some("Group1"), &[id1.clone()]);
        manager.create_from_selection(Some("Group2"), &[id2.clone()]);

        assert_eq!(manager.find_group(&id1), Some("Group1"));
        assert_eq!(manager.find_group(&id2), Some("Group2"));
    }

    #[test]
    fn test_group_manager_remove() {
        let mut manager = GroupManager::new();
        manager.create(Some("Group1"));
        manager.create(Some("Group2"));

        assert_eq!(manager.count(), 2);
        assert!(manager.remove("Group1"));
        assert_eq!(manager.count(), 1);
        assert!(!manager.remove("Group1")); // already removed
    }

    #[test]
    fn test_group_manager_selectable() {
        let mut manager = GroupManager::new();
        let group = manager.create(Some("TestGroup"));
        assert!(group.is_selectable);

        group.set_selectable(false);
        assert!(!group.is_selectable);
    }

    #[test]
    fn test_group_pick_style() {
        let mut manager = GroupManager::new();
        manager.create(Some("Group1"));
        manager.create(Some("Group2"));

        manager.set_group_pick_style_enabled(false);
        assert!(!manager.is_group_pick_style_enabled());

        manager.set_group_pick_style_enabled(true);
        assert!(manager.is_group_pick_style_enabled());
    }

    #[test]
    fn test_duplicate_add() {
        let mut group = Group::new();
        let id = ObjectId::new();

        assert!(group.add(id.clone()));
        assert!(!group.add(id)); // should not add duplicate
        assert_eq!(group.member_count(), 1);
    }
}
