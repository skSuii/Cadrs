use serde::{Serialize, Deserialize};
use std::fmt;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionType {
    Add,
    Delete,
    Modify,
    Transform,
    LayerChange,
    VisibilityChange,
    BlockOperation,
    DimensionOperation,
    TextOperation,
    GroupOperation,
    BlockEdit,
}

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ActionType::Add => write!(f, "Add"),
            ActionType::Delete => write!(f, "Delete"),
            ActionType::Modify => write!(f, "Modify"),
            ActionType::Transform => write!(f, "Transform"),
            ActionType::LayerChange => write!(f, "LayerChange"),
            ActionType::VisibilityChange => write!(f, "VisibilityChange"),
            ActionType::BlockOperation => write!(f, "BlockOperation"),
            ActionType::DimensionOperation => write!(f, "DimensionOperation"),
            ActionType::TextOperation => write!(f, "TextOperation"),
            ActionType::GroupOperation => write!(f, "GroupOperation"),
            ActionType::BlockEdit => write!(f, "BlockEdit"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryAction {
    pub id: u64,
    pub action_type: ActionType,
    pub entity_ids: Vec<String>,
    pub before_data: Vec<u8>,
    pub after_data: Vec<u8>,
    pub description: String,
    pub timestamp: SystemTime,
    pub user: Option<String>,
}

impl HistoryAction {
    pub fn new(
        action_type: ActionType,
        entity_ids: Vec<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().as_u128() as u64,
            action_type,
            entity_ids,
            before_data: Vec::new(),
            after_data: Vec::new(),
            description: description.into(),
            timestamp: SystemTime::now(),
            user: None,
        }
    }

    pub fn with_data(
        action_type: ActionType,
        entity_ids: Vec<String>,
        before_data: Vec<u8>,
        after_data: Vec<u8>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().as_u128() as u64,
            action_type,
            entity_ids,
            before_data,
            after_data,
            description: description.into(),
            timestamp: SystemTime::now(),
            user: None,
        }
    }

    pub fn set_user(&mut self, user: impl Into<String>) {
        self.user = Some(user.into());
    }

    pub fn get_entity_count(&self) -> usize {
        self.entity_ids.len()
    }

    pub fn is_same_action(&self, other: &HistoryAction) -> bool {
        self.action_type == other.action_type
            && self.entity_ids == other.entity_ids
            && self.description == other.description
    }
}

impl fmt::Display for HistoryAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "HistoryAction(id={}, type={}, entities={}, desc=\"{}\")",
            self.id,
            self.action_type,
            self.entity_ids.len(),
            self.description
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorySnapshot {
    pub id: u64,
    pub timestamp: SystemTime,
    pub actions: Vec<HistoryAction>,
    pub document_checksum: String,
}

impl HistorySnapshot {
    pub fn new(id: u64, actions: Vec<HistoryAction>, checksum: String) -> Self {
        Self {
            id,
            timestamp: SystemTime::now(),
            actions,
            document_checksum: checksum,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryManager {
    actions: Vec<HistoryAction>,
    current_position: usize,
    max_actions: usize,
    transaction_depth: usize,
    transaction_stack: Vec<usize>,
    saved_position: Option<usize>,
    is_recording: bool,
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self {
            actions: Vec::new(),
            current_position: 0,
            max_actions: 500,
            transaction_depth: 0,
            transaction_stack: Vec::new(),
            saved_position: None,
            is_recording: true,
        }
    }
}

impl HistoryManager {
    pub fn new(max_actions: usize) -> Self {
        Self {
            max_actions,
            ..Default::default()
        }
    }

    pub fn record(&mut self, action: HistoryAction) {
        if !self.is_recording {
            return;
        }

        if self.transaction_depth > 0 {
            let _transaction_start = *self.transaction_stack.last().unwrap_or(&0);

            // 事务期间的所有操作合并进事务开始处插入的那条记录，
            // 使整个事务在撤销栈中只占一步。
            // 注意：标记记录的描述是 "Transaction: <描述>"，必须用前缀匹配。
            if let Some(last) = self.actions.last() {
                if last.description.starts_with("Transaction:") && last.action_type == ActionType::Modify {
                    let combined_entities: Vec<String> = last.entity_ids.iter()
                        .chain(action.entity_ids.iter())
                        .cloned()
                        .collect();

                    let last_action = self.actions.last_mut().unwrap();
                    last_action.entity_ids = combined_entities;
                    return;
                }
            }
        }

        self.actions.truncate(self.current_position);
        self.actions.push(action);
        self.current_position = self.actions.len();

        while self.actions.len() > self.max_actions {
            self.actions.remove(0);
            self.current_position = self.current_position.saturating_sub(1);
        }
    }

    pub fn begin_transaction(&mut self, description: impl Into<String>) {
        self.transaction_depth += 1;
        let position = self.actions.len();
        self.transaction_stack.push(position);

        let transaction_action = HistoryAction::new(
            ActionType::Modify,
            Vec::new(),
            format!("Transaction: {}", description.into()),
        );
        self.actions.push(transaction_action);
        self.current_position = self.actions.len();
    }

    pub fn commit_transaction(&mut self) -> bool {
        if self.transaction_depth == 0 {
            return false;
        }

        if let Some(_start) = self.transaction_stack.pop() {
            if let Some(last) = self.actions.last() {
                if last.description.starts_with("Transaction:") {
                    if last.entity_ids.is_empty() {
                        self.actions.pop();
                        self.current_position = self.actions.len();
                    }
                }
            }
        }

        self.transaction_depth = self.transaction_depth.saturating_sub(1);
        true
    }

    pub fn rollback_transaction(&mut self) -> bool {
        if self.transaction_depth == 0 || self.transaction_stack.is_empty() {
            return false;
        }

        let start = self.transaction_stack.pop().unwrap();
        self.transaction_depth = self.transaction_depth.saturating_sub(1);

        while self.actions.len() > start {
            self.undo();
        }

        true
    }

    pub fn undo(&mut self) -> Option<&HistoryAction> {
        if self.can_undo() {
            self.current_position = self.current_position.saturating_sub(1);
            Some(&self.actions[self.current_position])
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<&HistoryAction> {
        if self.can_redo() {
            let action = &self.actions[self.current_position];
            self.current_position += 1;
            Some(action)
        } else {
            None
        }
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
        self.current_position < self.actions.len()
    }

    pub fn undo_count(&self) -> usize {
        self.current_position
    }

    pub fn redo_count(&self) -> usize {
        self.actions.len().saturating_sub(self.current_position)
    }

    pub fn save_position(&mut self) {
        self.saved_position = Some(self.current_position);
    }

    pub fn restore_position(&mut self) -> bool {
        if let Some(pos) = self.saved_position {
            while self.current_position > pos {
                self.undo();
            }
            while self.current_position < pos {
                self.redo();
            }
            self.saved_position = None;
            true
        } else {
            false
        }
    }

    pub fn mark_saved(&mut self) {
        self.saved_position = Some(self.current_position);
    }

    pub fn is_saved(&self) -> bool {
        if let Some(saved_pos) = self.saved_position {
            saved_pos == self.current_position
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.actions.clear();
        self.current_position = 0;
        self.transaction_depth = 0;
        self.transaction_stack.clear();
        self.saved_position = None;
    }

    pub fn set_max_actions(&mut self, max: usize) {
        self.max_actions = max;
        while self.actions.len() > self.max_actions {
            self.actions.remove(0);
        }
    }

    pub fn enable_recording(&mut self, enable: bool) {
        self.is_recording = enable;
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    pub fn get_actions(&self) -> &[HistoryAction] {
        &self.actions
    }

    pub fn get_visible_actions(&self) -> &[HistoryAction] {
        &self.actions[..self.current_position]
    }

    pub fn get_action_at(&self, index: usize) -> Option<&HistoryAction> {
        self.actions.get(index)
    }

    pub fn get_last_action(&self) -> Option<&HistoryAction> {
        self.actions.last()
    }

    pub fn find_actions_by_type(&self, action_type: ActionType) -> Vec<&HistoryAction> {
        self.actions.iter()
            .filter(|a| a.action_type == action_type)
            .collect()
    }

    pub fn find_actions_by_entity(&self, entity_id: &str) -> Vec<&HistoryAction> {
        self.actions.iter()
            .filter(|a| a.entity_ids.iter().any(|id| id == entity_id))
            .collect()
    }

    pub fn get_action_summary(&self) -> Vec<String> {
        self.actions.iter()
            .take(self.current_position)
            .map(|a| format!("[{}] {}", a.action_type, a.description))
            .collect()
    }
}

impl fmt::Display for HistoryManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "HistoryManager(undo={}, redo={}, total={})",
            self.undo_count(),
            self.redo_count(),
            self.actions.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_basic_operations() {
        let mut history = HistoryManager::new(100);

        assert!(!history.can_undo());
        assert!(!history.can_redo());

        history.record(HistoryAction::new(
            ActionType::Add,
            vec!["entity1".to_string()],
            "Add entity",
        ));

        assert!(history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo_count(), 1);

        history.undo();
        assert!(!history.can_undo());
        assert!(history.can_redo());

        history.redo();
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_transaction() {
        let mut history = HistoryManager::new(100);

        history.begin_transaction("Move operation");
        history.record(HistoryAction::new(ActionType::Transform, vec!["e1".to_string()], "Transform 1"));
        history.record(HistoryAction::new(ActionType::Transform, vec!["e2".to_string()], "Transform 2"));

        assert!(history.commit_transaction());

        history.undo();
        assert!(!history.can_undo());
    }

    #[test]
    fn test_max_actions() {
        let mut history = HistoryManager::new(3);

        for i in 0..5 {
            history.record(HistoryAction::new(
                ActionType::Add,
                vec![format!("entity{}", i)],
                format!("Add {}", i),
            ));
        }

        assert_eq!(history.actions.len(), 3);
        assert_eq!(history.undo_count(), 3);
    }
}
