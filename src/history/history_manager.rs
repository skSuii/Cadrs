use serde::{Serialize, Deserialize};
use std::time::SystemTime;
use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::sync::{Arc, RwLock};
use crate::data_structure::{Document, ObjectId, Entity};

pub trait HistoryListener: Send + Sync {
    fn on_history_changed(&self, _state: &UndoRedoState) {}
    fn on_action_executed(&self, _entry: &HistoryEntry) {}
    fn on_transaction_started(&self, _name: &str) {}
    fn on_transaction_completed(&self, _name: &str, _success: bool) {}
    fn on_undo(&self, _entry: &HistoryEntry) {}
    fn on_redo(&self, _entry: &HistoryEntry) {}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: u64,
    pub timestamp: SystemTime,
    pub action_type: String,
    pub entity_ids: Vec<String>,
    pub description: String,
    pub before_data: Option<Vec<u8>>,
    pub after_data: Option<Vec<u8>>,
    pub transaction_id: Option<u64>,
    pub user_data: Option<HashMap<String, String>>,
}

impl HistoryEntry {
    pub fn new(
        action_type: impl Into<String>,
        entity_ids: Vec<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().as_u128() as u64,
            timestamp: SystemTime::now(),
            action_type: action_type.into(),
            entity_ids,
            description: description.into(),
            before_data: None,
            after_data: None,
            transaction_id: None,
            user_data: None,
        }
    }

    pub fn with_before_data(mut self, data: Vec<u8>) -> Self {
        self.before_data = Some(data);
        self
    }

    pub fn with_after_data(mut self, data: Vec<u8>) -> Self {
        self.after_data = Some(data);
        self
    }

    pub fn with_transaction(mut self, transaction_id: u64) -> Self {
        self.transaction_id = Some(transaction_id);
        self
    }

    pub fn with_user_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        if self.user_data.is_none() {
            self.user_data = Some(HashMap::new());
        }
        self.user_data.as_mut().unwrap().insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoRedoState {
    pub can_undo: bool,
    pub can_redo: bool,
    pub undo_count: usize,
    pub redo_count: usize,
    pub current_position: usize,
    pub total_entries: usize,
    pub in_transaction: bool,
    pub transaction_name: Option<String>,
    pub saved_position: Option<usize>,
    pub is_modified: bool,
}

impl Default for UndoRedoState {
    fn default() -> Self {
        Self {
            can_undo: false,
            can_redo: false,
            undo_count: 0,
            redo_count: 0,
            current_position: 0,
            total_entries: 0,
            in_transaction: false,
            transaction_name: None,
            saved_position: None,
            is_modified: false,
        }
    }
}

impl UndoRedoState {
    pub fn from_manager(manager: &DocumentHistoryManager) -> Self {
        Self {
            can_undo: manager.can_undo(),
            can_redo: manager.can_redo(),
            undo_count: manager.get_undo_count(),
            redo_count: manager.get_redo_count(),
            current_position: manager.current_position(),
            total_entries: manager.total_entries(),
            in_transaction: manager.in_transaction(),
            transaction_name: manager.get_transaction_name(),
            saved_position: manager.saved_position(),
            is_modified: manager.is_modified(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Transaction {
    id: u64,
    name: String,
    start_time: SystemTime,
    entries: Vec<HistoryEntry>,
}

impl Transaction {
    pub fn new(id: u64, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            start_time: SystemTime::now(),
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: HistoryEntry) {
        self.entries.push(entry);
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

pub struct DocumentHistoryManager {
    entries: Vec<HistoryEntry>,
    current_position: usize,
    max_entries: usize,
    transactions: Vec<Transaction>,
    transaction_stack: Vec<u64>,
    saved_position: Option<usize>,
    listeners: Vec<Arc<dyn HistoryListener>>,
    name: String,
    is_modified: bool,
    metadata: HashMap<String, String>,
    batch_depth: usize,
    pending_entries: Vec<HistoryEntry>,
}

impl Default for DocumentHistoryManager {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            current_position: 0,
            max_entries: 1000,
            transactions: Vec::new(),
            transaction_stack: Vec::new(),
            saved_position: None,
            listeners: Vec::new(),
            name: "Untitled".to_string(),
            is_modified: false,
            metadata: HashMap::new(),
            batch_depth: 0,
            pending_entries: Vec::new(),
        }
    }
}

impl DocumentHistoryManager {
    pub fn new(name: impl Into<String>) -> Self {
        let mut manager = Self::default();
        manager.name = name.into();
        manager.saved_position = Some(0);
        manager
    }

    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_listener(&mut self, listener: Arc<dyn HistoryListener>) {
        self.listeners.push(listener);
    }

    fn notify_listeners<F>(&self, f: F)
    where
        F: Fn(&dyn HistoryListener),
    {
        let state = UndoRedoState::from_manager(self);
        for listener in &self.listeners {
            f(listener.as_ref());
            listener.on_history_changed(&state);
        }
    }

    fn notify_action<F>(&self, entry: &HistoryEntry, f: F)
    where
        F: Fn(&dyn HistoryListener, &HistoryEntry),
    {
        for listener in &self.listeners {
            f(listener.as_ref(), entry);
        }
    }

    pub fn record(&mut self, entry: HistoryEntry) {
        if self.batch_depth > 0 {
            self.pending_entries.push(entry);
            return;
        }

        self.entries.truncate(self.current_position);
        self.entries.push(entry.clone());
        self.current_position = self.entries.len();
        self.is_modified = true;

        self.limit_entries();
        self.notify_action(&entry, |listener, e| listener.on_action_executed(e));
        self.notify_listeners(|listener| {});
    }

    pub fn begin_transaction(&mut self, name: impl Into<String>) -> u64 {
        let transaction_id = uuid::Uuid::new_v4().as_u128() as u64;
        let transaction = Transaction::new(transaction_id, name);
        self.transactions.push(transaction);
        self.transaction_stack.push(transaction_id);

        for listener in &self.listeners {
            listener.on_transaction_started(&self.name);
        }

        transaction_id
    }

    pub fn commit_transaction(&mut self) -> bool {
        if self.transaction_stack.is_empty() {
            return false;
        }

        let transaction_id = self.transaction_stack.pop().unwrap();
        if let Some(transaction) = self.transactions.iter_mut()
            .find(|t| t.id == transaction_id)
            .filter(|t| !t.is_empty())
        {
            for listener in &self.listeners {
                listener.on_transaction_completed(&transaction.name, true);
            }
        }

        true
    }

    pub fn rollback_transaction(&mut self) -> bool {
        if self.transaction_stack.is_empty() {
            return false;
        }

        let transaction_id = self.transaction_stack.pop().unwrap();
        let transaction_name = self.transactions.iter()
            .find(|t| t.id == transaction_id)
            .map(|t| t.name.clone());

        while self.find_last_transaction_entry(transaction_id).is_some() {
            self.undo();
        }

        if let Some(name) = transaction_name {
            for listener in &self.listeners {
                listener.on_transaction_completed(&name, false);
            }
        }

        true
    }

    fn find_last_transaction_entry(&self, transaction_id: u64) -> Option<usize> {
        self.entries[..self.current_position]
            .iter()
            .enumerate()
            .rev()
            .find(|(_, e)| e.transaction_id == Some(transaction_id))
            .map(|(idx, _)| idx)
    }

    pub fn undo(&mut self) -> Option<&HistoryEntry> {
        if !self.can_undo() {
            return None;
        }

        self.current_position = self.current_position.saturating_sub(1);
        let entry = &self.entries[self.current_position];
        self.is_modified = true;

        self.notify_action(entry, |listener, e| listener.on_undo(e));
        self.notify_listeners(|listener| {});

        Some(entry)
    }

    pub fn redo(&mut self) -> Option<&HistoryEntry> {
        if !self.can_redo() {
            return None;
        }

        let entry = &self.entries[self.current_position];
        self.current_position += 1;
        self.is_modified = true;

        self.notify_action(entry, |listener, e| listener.on_redo(e));
        self.notify_listeners(|listener| {});

        Some(entry)
    }

    pub fn undo_n(&mut self, n: usize) -> usize {
        let mut count = 0;
        for _ in 0..n {
            if self.undo().is_some() {
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    pub fn redo_n(&mut self, n: usize) -> usize {
        let mut count = 0;
        for _ in 0..n {
            if self.redo().is_some() {
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    pub fn can_undo(&self) -> bool {
        self.current_position > 0
    }

    pub fn can_redo(&self) -> bool {
        self.current_position < self.entries.len()
    }

    pub fn get_undo_count(&self) -> usize {
        self.current_position
    }

    pub fn get_redo_count(&self) -> usize {
        self.entries.len().saturating_sub(self.current_position)
    }

    pub fn current_position(&self) -> usize {
        self.current_position
    }

    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn in_transaction(&self) -> bool {
        !self.transaction_stack.is_empty()
    }

    pub fn get_transaction_name(&self) -> Option<String> {
        self.transaction_stack.last()
            .and_then(|id| self.transactions.iter()
                .find(|t| t.id == *id)
                .map(|t| t.name.clone()))
    }

    pub fn get_transaction_depth(&self) -> usize {
        self.transaction_stack.len()
    }

    pub fn mark_saved(&mut self) {
        self.saved_position = Some(self.current_position);
        self.is_modified = false;
    }

    pub fn saved_position(&self) -> Option<usize> {
        self.saved_position
    }

    pub fn is_saved(&self) -> bool {
        if let Some(saved) = self.saved_position {
            saved == self.current_position && !self.is_modified
        } else {
            false
        }
    }

    pub fn is_modified(&self) -> bool {
        self.is_modified
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_position = 0;
        self.transactions.clear();
        self.transaction_stack.clear();
        self.saved_position = Some(0);
        self.is_modified = false;
        self.notify_listeners(|listener| {});
    }

    pub fn clear_undo_history(&mut self) {
        self.entries.truncate(self.current_position);
        self.notify_listeners(|listener| {});
    }

    pub fn set_max_entries(&mut self, max: usize) {
        self.max_entries = max;
        self.limit_entries();
    }

    pub fn max_entries(&self) -> usize {
        self.max_entries
    }

    fn limit_entries(&mut self) {
        while self.entries.len() > self.max_entries {
            self.entries.remove(0);
            self.current_position = self.current_position.saturating_sub(1);
            if let Some(saved) = &mut self.saved_position {
                if *saved > 0 {
                    *saved = saved.saturating_sub(1);
                }
            }
        }
    }

    pub fn start_batch(&mut self) {
        self.batch_depth += 1;
    }

    pub fn end_batch(&mut self) {
        if self.batch_depth > 0 {
            self.batch_depth -= 1;
            let pending = std::mem::take(&mut self.pending_entries);
            for entry in pending {
                self.record(entry);
            }
        }
    }

    pub fn is_in_batch(&self) -> bool {
        self.batch_depth > 0
    }

    pub fn get_entry_at(&self, index: usize) -> Option<&HistoryEntry> {
        self.entries.get(index)
    }

    pub fn get_entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    pub fn get_undo_stack(&self) -> &[HistoryEntry] {
        &self.entries[..self.current_position]
    }

    pub fn get_redo_stack(&self) -> &[HistoryEntry] {
        &self.entries[self.current_position..]
    }

    pub fn find_entries<F>(&self, predicate: F) -> Vec<&HistoryEntry>
    where
        F: Fn(&HistoryEntry) -> bool,
    {
        self.entries.iter().filter(|e| predicate(e)).collect()
    }

    pub fn find_entries_by_type(&self, action_type: &str) -> Vec<&HistoryEntry> {
        self.find_entries(|e| &e.action_type == action_type)
    }

    pub fn find_entries_by_entity(&self, entity_id: &str) -> Vec<&HistoryEntry> {
        self.find_entries(|e| e.entity_ids.iter().any(|id| id == entity_id))
    }

    pub fn get_entries_in_time_range(
        &self,
        start: SystemTime,
        end: SystemTime,
    ) -> Vec<&HistoryEntry> {
        self.entries.iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .collect()
    }

    pub fn get_state(&self) -> UndoRedoState {
        UndoRedoState::from_manager(self)
    }

    pub fn get_summary(&self) -> Vec<String> {
        self.entries.iter()
            .take(self.current_position)
            .enumerate()
            .map(|(i, e)| format!("[{}] {} - {}", i + 1, e.action_type, e.description))
            .collect()
    }

    pub fn get_detailed_summary(&self) -> Vec<(usize, String, String, String)> {
        self.entries.iter()
            .take(self.current_position)
            .enumerate()
            .map(|(i, e)| {
                let time = e.timestamp.duration_since(SystemTime::UNIX_EPOCH)
                    .map(|d| format!("{:?}", d))
                    .unwrap_or_else(|_| "Unknown".to_string());
                (i + 1, e.action_type.clone(), e.description.clone(), time)
            })
            .collect()
    }

    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }

    pub fn export_history(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self.entries[..self.current_position])
    }

    pub fn import_history(&mut self, data: &[u8]) -> Result<(), serde_json::Error> {
        let entries: Vec<HistoryEntry> = serde_json::from_slice(data)?;
        self.entries = entries;
        self.current_position = self.entries.len();
        self.is_modified = true;
        self.notify_listeners(|listener| {});
        Ok(())
    }

    pub fn apply_undo(&self, doc: &mut Document, entry: &HistoryEntry) -> Result<(), String> {
        if let Some(before_data) = &entry.before_data {
            for entity_id_str in &entry.entity_ids {
                if let Ok(entity_id) = entity_id_str.parse::<ObjectId>() {
                    if let Some(entity) = doc.get_entity_mut(&entity_id) {
                        if let Ok(restored) = serde_json::from_slice::<Entity>(before_data) {
                            *entity = restored;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn apply_redo(&self, doc: &mut Document, entry: &HistoryEntry) -> Result<(), String> {
        if let Some(after_data) = &entry.after_data {
            for entity_id_str in &entry.entity_ids {
                if let Ok(entity_id) = entity_id_str.parse::<ObjectId>() {
                    if let Some(entity) = doc.get_entity_mut(&entity_id) {
                        if let Ok(restored) = serde_json::from_slice::<Entity>(after_data) {
                            *entity = restored;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn execute_undo(&mut self, doc: &mut Document) -> Result<(), String> {
        if let Some(entry) = self.undo().cloned() {
            match entry.action_type.as_str() {
                "Add" => {
                    for entity_id_str in &entry.entity_ids {
                        if let Ok(entity_id) = entity_id_str.parse::<ObjectId>() {
                            doc.remove_entity(&entity_id);
                        }
                    }
                }
                "Delete" => {
                    if let Some(before_data) = &entry.before_data {
                        for entity_id_str in &entry.entity_ids {
                            if let Ok(entity_id) = entity_id_str.parse::<ObjectId>() {
                                if let Ok(entity) = serde_json::from_slice::<Entity>(&before_data) {
                                    doc.add_entity(entity);
                                }
                            }
                        }
                    }
                }
                _ => {
                    self.apply_undo(doc, &entry)?;
                }
            }
            Ok(())
        } else {
            Err("Nothing to undo".to_string())
        }
    }

    pub fn execute_redo(&mut self, doc: &mut Document) -> Result<(), String> {
        if let Some(entry) = self.redo().cloned() {
            match entry.action_type.as_str() {
                "Add" => {
                    if let Some(after_data) = &entry.after_data {
                        for entity_id_str in &entry.entity_ids {
                            if let Ok(entity_id) = entity_id_str.parse::<ObjectId>() {
                                if let Ok(entity) = serde_json::from_slice::<Entity>(&after_data) {
                                    doc.add_entity(entity);
                                }
                            }
                        }
                    }
                }
                "Delete" => {
                    for entity_id_str in &entry.entity_ids {
                        if let Ok(entity_id) = entity_id_str.parse::<ObjectId>() {
                            doc.remove_entity(&entity_id);
                        }
                    }
                }
                _ => {
                    self.apply_redo(doc, &entry)?;
                }
            }
            Ok(())
        } else {
            Err("Nothing to redo".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_undo_redo() {
        let mut manager = DocumentHistoryManager::new("Test");

        assert!(!manager.can_undo());
        assert!(!manager.can_redo());

        manager.record(HistoryEntry::new("Add", vec!["entity1".to_string()], "Add entity 1"));

        assert!(manager.can_undo());
        assert!(!manager.can_redo());
        assert_eq!(manager.get_undo_count(), 1);

        manager.undo();
        assert!(!manager.can_undo());
        assert!(manager.can_redo());

        manager.redo();
        assert!(manager.can_undo());
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_transaction() {
        let mut manager = DocumentHistoryManager::new("Test");

        manager.begin_transaction("Move operation");
        manager.record(HistoryEntry::new("Transform", vec!["e1".to_string()], "Transform 1"));
        manager.record(HistoryEntry::new("Transform", vec!["e2".to_string()], "Transform 2"));
        manager.commit_transaction();

        assert_eq!(manager.get_undo_count(), 1);

        manager.undo();
        assert_eq!(manager.get_undo_count(), 0);
    }

    #[test]
    fn test_modified_state() {
        let mut manager = DocumentHistoryManager::new("Test");

        assert!(!manager.is_modified());

        manager.record(HistoryEntry::new("Add", vec!["e1".to_string()], "Add"));

        assert!(manager.is_modified());

        manager.mark_saved();
        assert!(!manager.is_modified());

        manager.record(HistoryEntry::new("Modify", vec!["e1".to_string()], "Modify"));

        assert!(manager.is_modified());
    }

    #[test]
    fn test_batch_operations() {
        let mut manager = DocumentHistoryManager::new("Test");

        manager.start_batch();
        manager.record(HistoryEntry::new("Add", vec!["e1".to_string()], "Add 1"));
        manager.record(HistoryEntry::new("Add", vec!["e2".to_string()], "Add 2"));
        manager.end_batch();

        assert_eq!(manager.get_undo_count(), 1);
    }
}
