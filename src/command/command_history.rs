use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HistoryActionType {
    Add,
    Delete,
    Modify,
    Transform,
    LayerChange,
    VisibilityChange,
    BlockOperation,
    DimensionOperation,
    TextOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryAction {
    pub action_type: HistoryActionType,
    pub entity_ids: Vec<super::super::data_structure::ObjectId>,
    pub before_state: Vec<u8>,
    pub after_state: Vec<u8>,
    pub description: String,
    pub timestamp: std::time::SystemTime,
}

impl HistoryAction {
    pub fn new(
        action_type: HistoryActionType,
        entity_ids: Vec<super::super::data_structure::ObjectId>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            action_type,
            entity_ids,
            before_state: Vec::new(),
            after_state: Vec::new(),
            description: description.into(),
            timestamp: std::time::SystemTime::now(),
        }
    }

    pub fn with_states(
        action_type: HistoryActionType,
        entity_ids: Vec<super::super::data_structure::ObjectId>,
        before_state: Vec<u8>,
        after_state: Vec<u8>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            action_type,
            entity_ids,
            before_state,
            after_state,
            description: description.into(),
            timestamp: std::time::SystemTime::now(),
        }
    }

    pub fn execute(&self, document: &mut super::super::data_structure::Document) -> Result<(), super::super::error::CadError> {
        use super::super::error::CadError;

        match self.action_type {
            HistoryActionType::Add => {
                for entity_id in &self.entity_ids {
                    if let Some(entity) = document.get_entity(entity_id) {
                        if !document.entity_exists(entity_id) {
                            document.add_entity(entity.clone());
                        }
                    }
                }
                Ok(())
            }
            HistoryActionType::Delete => {
                for entity_id in &self.entity_ids {
                    document.remove_entity(entity_id);
                }
                Ok(())
            }
            HistoryActionType::Modify | HistoryActionType::Transform | HistoryActionType::LayerChange | HistoryActionType::VisibilityChange => {
                if !self.after_state.is_empty() {
                    for entity_id in &self.entity_ids {
                        let _ = document.get_entity_mut(entity_id);
                    }
                }
                Ok(())
            }
            HistoryActionType::BlockOperation => {
                Ok(())
            }
            HistoryActionType::DimensionOperation | HistoryActionType::TextOperation => {
                Ok(())
            }
        }
    }

    pub fn undo(&self, document: &mut super::super::data_structure::Document) -> Result<(), super::super::error::CadError> {
        use super::super::error::CadError;

        match self.action_type {
            HistoryActionType::Add => {
                for entity_id in &self.entity_ids {
                    document.remove_entity(entity_id);
                }
                Ok(())
            }
            HistoryActionType::Delete => {
                if let Ok(entity) = serde_json::from_slice::<super::super::data_structure::Entity>(&self.before_state) {
                    for _ in &self.entity_ids {
                        document.add_entity(entity.clone());
                    }
                }
                Ok(())
            }
            HistoryActionType::Modify | HistoryActionType::Transform | HistoryActionType::LayerChange | HistoryActionType::VisibilityChange => {
                if !self.before_state.is_empty() {
                    for entity_id in &self.entity_ids {
                        if let Some(entity) = document.get_entity_mut(entity_id) {
                            if let Some(serialized) = serde_json::to_vec(&entity).ok() {
                                if let Ok(restored) = serde_json::from_slice::<super::super::data_structure::Entity>(&self.before_state) {
                                    let _ = std::mem::replace(entity, restored);
                                }
                            }
                        }
                    }
                }
                Ok(())
            }
            HistoryActionType::BlockOperation | HistoryActionType::DimensionOperation | HistoryActionType::TextOperation => {
                Ok(())
            }
        }
    }

    pub fn redo(&self, document: &mut super::super::data_structure::Document) -> Result<(), super::super::error::CadError> {
        self.execute(document)
    }
}

impl fmt::Display for HistoryAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "HistoryAction(type={:?}, entities={}, desc=\"{}\")",
            self.action_type,
            self.entity_ids.len(),
            self.description
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorySnapshot {
    pub id: u64,
    pub timestamp: std::time::SystemTime,
    pub actions: Vec<HistoryAction>,
    pub document_state: Vec<u8>,
}

impl HistorySnapshot {
    pub fn new(id: u64, actions: Vec<HistoryAction>, document_state: Vec<u8>) -> Self {
        Self {
            id,
            timestamp: std::time::SystemTime::now(),
            actions,
            document_state,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHistory {
    current_position: usize,
    max_history: usize,
    actions: Vec<HistoryAction>,
    snapshots: Vec<HistorySnapshot>,
    transaction_depth: usize,
    transactions: Vec<usize>,
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self {
            current_position: 0,
            max_history: 100,
            actions: Vec::new(),
            snapshots: Vec::new(),
            transaction_depth: 0,
            transactions: Vec::new(),
        }
    }
}

impl CommandHistory {
    pub fn new(max_history: usize) -> Self {
        Self {
            max_history,
            ..Default::default()
        }
    }

    pub fn execute(&mut self, action: HistoryAction) {
        if self.transaction_depth > 0 {
            let transaction_start = self.transactions.last().copied().unwrap_or(0);
            if self.actions.len() > transaction_start {
                if let Some(last_action) = self.actions.last_mut() {
                    if last_action.description == "Transaction" {
                        last_action.entity_ids.extend(action.entity_ids);
                        return;
                    }
                }
            }
        }

        self.actions.truncate(self.current_position);

        self.actions.push(action);
        self.current_position = self.actions.len();

        self.limit_history();
    }

    pub fn begin_transaction(&mut self, description: impl Into<String>) {
        self.transaction_depth += 1;
        self.transactions.push(self.actions.len());

        self.actions.push(HistoryAction::new(
            HistoryActionType::Modify,
            Vec::new(),
            format!("Transaction: {}", description.into()),
        ));
        self.current_position = self.actions.len();
    }

    pub fn commit_transaction(&mut self) -> bool {
        if self.transaction_depth == 0 {
            return false;
        }

        self.transaction_depth -= 1;
        self.transactions.pop();

        true
    }

    pub fn rollback_transaction(&mut self) -> bool {
        if self.transaction_depth == 0 || self.transactions.is_empty() {
            return false;
        }

        let transaction_start = self.transactions.pop().unwrap();
        self.transaction_depth -= 1;

        while self.actions.len() > transaction_start {
            self.undo();
        }

        true
    }

    pub fn undo(&mut self) -> Option<&HistoryAction> {
        if self.can_undo() {
            self.current_position -= 1;
            let action = &self.actions[self.current_position];
            Some(action)
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

    pub fn get_undo_count(&self) -> usize {
        self.current_position
    }

    pub fn get_redo_count(&self) -> usize {
        self.actions.len() - self.current_position
    }

    pub fn create_snapshot(&mut self, document: &super::super::data_structure::Document) -> HistorySnapshot {
        let snapshot = HistorySnapshot::new(
            self.snapshots.len() as u64,
            self.actions[self.current_position.min(self.actions.len())..]
                .to_vec(),
            Vec::new(),
        );
        self.snapshots.push(snapshot.clone());
        snapshot
    }

    pub fn restore_snapshot(&mut self, snapshot: &HistorySnapshot) -> bool {
        if self.snapshots.iter().any(|s| s.id == snapshot.id) {
            self.actions = snapshot.actions.clone();
            self.current_position = self.actions.len();
            true
        } else {
            false
        }
    }

    fn limit_history(&mut self) {
        while self.actions.len() > self.max_history {
            self.actions.remove(0);
            self.current_position = self.current_position.saturating_sub(1);
        }
    }

    pub fn clear(&mut self) {
        self.actions.clear();
        self.snapshots.clear();
        self.current_position = 0;
        self.transaction_depth = 0;
        self.transactions.clear();
    }

    pub fn set_max_history(&mut self, max: usize) {
        self.max_history = max;
        self.limit_history();
    }

    pub fn get_actions(&self) -> &[HistoryAction] {
        &self.actions
    }

    pub fn get_action_history(&self) -> &[HistoryAction] {
        &self.actions
    }
}

impl fmt::Display for CommandHistory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CommandHistory(undo={}, redo={}, total={})",
            self.get_undo_count(),
            self.get_redo_count(),
            self.actions.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_undo_redo() {
        let mut history = CommandHistory::new(10);

        assert!(!history.can_undo());
        assert!(!history.can_redo());

        let action = HistoryAction::new(
            HistoryActionType::Add,
            vec![crate::data_structure::ObjectId::new()],
            "Add entity",
        );
        history.execute(action);

        assert!(history.can_undo());
        assert!(!history.can_redo());

        assert_eq!(history.get_undo_count(), 1);

        let undone = history.undo();
        assert!(undone.is_some());
        assert!(history.can_redo());

        let redone = history.redo();
        assert!(redone.is_some());
        assert!(history.can_undo());
    }

    #[test]
    fn test_transaction() {
        let mut history = CommandHistory::new(100);

        history.begin_transaction("Move entities");
        history.execute(HistoryAction::new(HistoryActionType::Transform, vec![], "Transform 1"));
        history.execute(HistoryAction::new(HistoryActionType::Transform, vec![], "Transform 2"));

        assert!(history.commit_transaction());

        history.undo();
        assert_eq!(history.get_undo_count(), 0);
    }
}
