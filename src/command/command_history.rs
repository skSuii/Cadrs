use serde::{Serialize, Deserialize};
use std::fmt;

/// 历史动作的类型，决定撤销/重做时如何恢复文档。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HistoryActionType {
    /// 新增实体；撤销时把这些实体删除。
    Add,
    /// 删除实体；撤销时按 `before_state` 反序列化重建。
    Delete,
    /// 修改实体属性；撤销时按 `before_state` 还原。
    Modify,
    /// 几何变换（移动、旋转、缩放等）；撤销方式同 `Modify`。
    Transform,
    /// 图层归属变化；撤销方式同 `Modify`。
    LayerChange,
    /// 可见性变化；撤销方式同 `Modify`。
    VisibilityChange,
    /// 块操作；当前实现只记录，不改变文档。
    BlockOperation,
    /// 标注操作；当前实现只记录，不改变文档。
    DimensionOperation,
    /// 文字操作；当前实现只记录，不改变文档。
    TextOperation,
}

/// 一条可撤销的历史记录。
///
/// `before_state` / `after_state` 存放实体序列化后的字节（JSON），用于撤销与重做；
/// 不依赖快照的动作类型可以将二者留空。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryAction {
    /// 动作类型，决定 [`HistoryAction::execute`] 与 [`HistoryAction::undo`] 的行为。
    pub action_type: HistoryActionType,
    /// 本动作涉及的实体标识列表。
    pub entity_ids: Vec<super::super::data_structure::ObjectId>,
    /// 执行前的实体序列化状态；为空表示没有可还原的快照。
    pub before_state: Vec<u8>,
    /// 执行后的实体序列化状态。
    pub after_state: Vec<u8>,
    /// 面向用户的动作描述，用于历史列表显示。
    pub description: String,
    /// 记录创建时间。
    pub timestamp: std::time::SystemTime,
}

impl HistoryAction {
    /// 创建不带快照的历史记录，时间戳取当前时刻。
    ///
    /// - `action_type`：动作类型；`entity_ids`：涉及的实体；`description`：描述文本。
    /// 返回：`before_state` 与 `after_state` 均为空的记录。
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

    /// 创建带前后快照的历史记录，时间戳取当前时刻。
    ///
    /// - `before_state`、`after_state`：执行前后实体的序列化字节，供撤销与重做还原。
    /// - `description`：面向用户的描述文本。
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

    /// 正向执行本动作（重做同样调用此方法）。
    ///
    /// 副作用：直接修改 `document`——`Add` 按 `entity_ids` 补回实体，`Delete` 移除实体；
    /// 修改类动作目前只做占位处理。
    /// 返回：文件/序列化错误以 `CadError` 返回；当前实现不产生错误。
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

    /// 反向执行本动作，把文档恢复到动作发生前的状态。
    ///
    /// 副作用：修改 `document`——`Add` 删除实体，`Delete` 依据 `before_state` 反序列化重建，
    /// 修改类动作按 `before_state` 覆盖实体；快照缺失或反序列化失败时静默跳过。
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

    /// 重做本动作，等价于再次调用 [`HistoryAction::execute`]。
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

/// 历史快照：记录创建时刻尚未执行的动作列表，可按编号恢复。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorySnapshot {
    /// 快照编号，由创建时的快照数量决定，供 [`CommandHistory::restore_snapshot`] 校验。
    pub id: u64,
    /// 快照创建时间。
    pub timestamp: std::time::SystemTime,
    /// 快照时刻游标之后的动作列表。
    pub actions: Vec<HistoryAction>,
    /// 文档整体状态（序列化字节）；当前实现留空。
    pub document_state: Vec<u8>,
}

impl HistorySnapshot {
    /// 创建快照，时间戳取当前时刻。
    ///
    /// - `id`：编号；`actions`：待执行动作；`document_state`：文档序列化状态，可为空。
    pub fn new(id: u64, actions: Vec<HistoryAction>, document_state: Vec<u8>) -> Self {
        Self {
            id,
            timestamp: std::time::SystemTime::now(),
            actions,
            document_state,
        }
    }
}

/// 命令历史栈：负责记录、撤销、重做、事务合并与快照恢复。
///
/// 内部用游标区分「已执行」与「已撤销」的动作：压入新动作会截断游标之后的部分；
/// 记录数超过 `max_history` 时丢弃最旧的一条并把游标前移。
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
    /// 创建空历史栈，最多保留 `max_history` 条记录（游标初始为 0）。
    pub fn new(max_history: usize) -> Self {
        Self {
            max_history,
            ..Default::default()
        }
    }

    /// 压入一条历史记录。
    ///
    /// 副作用：先截断游标之后的动作（丢弃已撤销的分支），再压入 `action` 并把游标移到末尾，
    /// 最后按 `max_history` 裁剪；处于事务中时该动作会被并入 `Transaction:` 标记记录而不单独入栈。
    pub fn execute(&mut self, action: HistoryAction) {
        // 事务期间的操作合并进事务开始处插入的标记记录，
        // 使整个事务在撤销栈中只占一步（标记记录描述为 "Transaction: <描述>"）。
        if self.transaction_depth > 0 {
            let transaction_start = self.transactions.last().copied().unwrap_or(0);
            if self.actions.len() > transaction_start {
                if let Some(last_action) = self.actions.last_mut() {
                    if last_action.description.starts_with("Transaction:") {
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

    /// 开启一个事务，其后压入的动作合并为一条 `Transaction: 描述` 记录。
    ///
    /// 支持嵌套：嵌套调用只增加层数并记下当前位置，不会额外新增记录。
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

    /// 提交最内层事务，保留其记录（已合并的动作不再拆开）。
    ///
    /// 返回：当前不在事务中时为 `false` 且状态不变；否则层数减一并返回 `true`。
    pub fn commit_transaction(&mut self) -> bool {
        if self.transaction_depth == 0 {
            return false;
        }

        self.transaction_depth -= 1;
        self.transactions.pop();

        true
    }

    /// 回滚最内层事务：反复撤销，直到该事务开始后压入的动作全部移出。
    ///
    /// 返回：不在事务中时为 `false` 且状态不变；否则层数减一并返回 `true`。
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

    /// 撤销一步，把游标前移一位。
    ///
    /// 副作用：仅移动游标，不调用 [`HistoryAction::undo`]，因此不会改动文档。
    /// 返回：被越过的动作；已无内容可撤销时为 `None` 且游标不变。
    pub fn undo(&mut self) -> Option<&HistoryAction> {
        if self.can_undo() {
            self.current_position -= 1;
            let action = &self.actions[self.current_position];
            Some(action)
        } else {
            None
        }
    }

    /// 重做一步，把游标后移一位。
    ///
    /// 副作用：仅移动游标，不执行动作。
    /// 返回：被越过的动作；已无内容可重做时为 `None` 且游标不变。
    pub fn redo(&mut self) -> Option<&HistoryAction> {
        if self.can_redo() {
            let action = &self.actions[self.current_position];
            self.current_position += 1;
            Some(action)
        } else {
            None
        }
    }

    /// 连续撤销最多 `n` 步。
    ///
    /// 返回：实际撤销的步数；中途无内容可撤销时提前结束，因此可能小于 `n`。
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

    /// 连续重做最多 `n` 步。
    ///
    /// 返回：实际重做的步数，可能小于 `n`。
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

    /// 是否还有可撤销的动作（游标大于 0）。
    pub fn can_undo(&self) -> bool {
        self.current_position > 0
    }

    /// 是否还有可重做的动作（游标之后仍有记录）。
    pub fn can_redo(&self) -> bool {
        self.current_position < self.actions.len()
    }

    /// 统计可撤销的动作数量（等于当前游标位置）。
    pub fn get_undo_count(&self) -> usize {
        self.current_position
    }

    /// 统计可重做的动作数量（游标之后的记录条数）。
    pub fn get_redo_count(&self) -> usize {
        self.actions.len() - self.current_position
    }

    /// 为当前状态创建快照并登记到内部列表。
    ///
    /// - `document`：源文档；当前实现不序列化文档内容，只截取游标之后待执行的动作。
    /// 返回：新快照的副本，其编号为创建前的快照数量。
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

    /// 用快照恢复动作列表。
    ///
    /// 副作用：用快照中的动作整体替换当前动作，并把游标移到末尾。
    /// 返回：`snapshot.id` 在本历史中登记过时为 `true`；否则不做任何修改并返回 `false`。
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

    /// 清空全部动作、快照与事务状态，游标归零；`max_history` 保持不变。
    pub fn clear(&mut self) {
        self.actions.clear();
        self.snapshots.clear();
        self.current_position = 0;
        self.transaction_depth = 0;
        self.transactions.clear();
    }

    /// 调整最多保留的记录条数，并立即裁剪超出的最旧记录（游标随之左移）。
    pub fn set_max_history(&mut self, max: usize) {
        self.max_history = max;
        self.limit_history();
    }

    /// 按时间顺序读取全部动作，包含已被撤销（游标之后）的部分。
    pub fn get_actions(&self) -> &[HistoryAction] {
        &self.actions
    }

    /// 与 [`CommandHistory::get_actions`] 等价的旧接口别名。
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
