//! 命令与撤销/重做管理：以命令对象记录编辑操作的先后顺序。
//!
//! [`CommandManager`] 维护撤销栈与重做栈，栈中存放 [`Command`]；每个命令按操作类别
//! （[`CommandType`]）携带改动前后的 [`CommandState`] 快照。[`CommandManager::execute`] 只把
//! 命令压入栈并清空重做栈——它不真正修改文档，几何数据的实际变更仍须由调用方完成，
//! 这里仅登记历史与维护历史长度上限。一次连续编辑可用 [`CommandManager::begin_batch`]
//! 归并成一个可整体撤销的批量命令。

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use super::{Entity, ObjectId, Layer, Block, BlockReference};

/// 命令的操作类别，用于分类与界面提示。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandType {
    /// 新增实体。
    AddEntity,
    /// 删除实体。
    RemoveEntity,
    /// 修改实体。
    ModifyEntity,
    /// 新增图层。
    AddLayer,
    /// 删除图层。
    RemoveLayer,
    /// 修改图层。
    ModifyLayer,
    /// 新增块定义。
    AddBlock,
    /// 删除块定义。
    RemoveBlock,
    /// 新增块参照。
    AddBlockRef,
    /// 删除块参照。
    RemoveBlockRef,
    /// 成组：把若干命令合并为一次撤销单位。
    Group,
    /// 解组。
    Ungroup,
}

/// 一条可撤销命令：类别、描述、时间戳与可选的前后状态快照。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    command_type: CommandType,
    description: String,
    timestamp: std::time::SystemTime,
    before_state: Option<CommandState>,
    after_state: Option<CommandState>,
}

/// 文档状态快照：某一时刻的实体、图层、块与块参照集合，用于撤销/重做时还原。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandState {
    entities: HashMap<ObjectId, Entity>,
    layers: HashMap<ObjectId, Layer>,
    blocks: HashMap<ObjectId, Block>,
    block_references: HashMap<ObjectId, BlockReference>,
}

impl Command {
    fn new(command_type: CommandType, description: String) -> Self {
        Self {
            command_type,
            description,
            timestamp: std::time::SystemTime::now(),
            before_state: None,
            after_state: None,
        }
    }

    /// 命令的操作类别。
    pub fn command_type(&self) -> &CommandType {
        &self.command_type
    }

    /// 命令描述，用于撤销/重做菜单项的文字。
    pub fn description(&self) -> &str {
        &self.description
    }

    /// 命令创建时刻，[`CommandManager::execute`] 用它判定命令的先后顺序。
    pub fn timestamp(&self) -> &std::time::SystemTime {
        &self.timestamp
    }
}

/// 命令管理器：维护撤销栈、重做栈与历史长度上限。
pub struct CommandManager {
    undo_stack: Vec<Command>,
    redo_stack: Vec<Command>,
    max_history_size: usize,
    current_state: CommandState,
}

impl CommandManager {
    /// 新建命令管理器：两个栈均为空，历史长度上限为 100，当前状态为空白快照。
    #[inline]
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history_size: 100,
            current_state: CommandState::new(),
        }
    }

    /// 设置历史长度上限，会修改 `self`。
    ///
    /// - `size`：撤销栈允许容纳的最大命令数；超限时最旧的命令被丢弃，新上限不会立即裁剪已有历史。
    #[inline]
    pub fn set_max_history_size(&mut self, size: usize) {
        self.max_history_size = size;
    }

    /// 当前历史长度上限，默认 100。
    #[inline]
    pub fn max_history_size(&self) -> usize {
        self.max_history_size
    }

    /// 登记一条已执行的命令，会修改 `self`。
    ///
    /// 命令按时间戳入栈：若其时间戳不晚于栈顶命令，则视为过期命令被静默丢弃（不返回错误、
    /// 不入栈）；成功入栈后重做栈被清空，且撤销栈超出上限时丢弃最旧的命令。
    /// 本方法不修改 `current_state`，也不改动文档内容。
    ///
    /// - `command`：待登记的命令。
    #[inline]
    pub fn execute(&mut self, command: Command) {
        if let Some(last_command) = self.undo_stack.last() {
            if command.timestamp() <= last_command.timestamp() {
                return;
            }
        }

        self.undo_stack.push(command.clone());
        self.redo_stack.clear();

        while self.undo_stack.len() > self.max_history_size {
            self.undo_stack.remove(0);
        }
    }

    /// 撤销一步：弹出撤销栈顶命令并将其压入重做栈，会修改 `self`。
    ///
    /// 返回被弹出的命令，供调用方按其快照还原文档；撤销栈为空时返回 [`None`]，此时状态不变。
    #[inline]
    pub fn undo(&mut self) -> Option<Command> {
        if let Some(command) = self.undo_stack.pop() {
            self.redo_stack.push(command.clone());
            Some(command)
        } else {
            None
        }
    }

    /// 重做一步：弹出重做栈顶命令并压回撤销栈，会修改 `self`。
    ///
    /// 返回被重做的命令，供调用方按其快照重新应用；重做栈为空时返回 [`None`]。
    #[inline]
    pub fn redo(&mut self) -> Option<Command> {
        if let Some(command) = self.redo_stack.pop() {
            self.undo_stack.push(command.clone());
            Some(command)
        } else {
            None
        }
    }

    /// 当前是否可撤销，即撤销栈非空。
    #[inline]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// 当前是否可重做，即重做栈非空。
    #[inline]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// 下一次撤销将回退的命令描述；无可撤销内容时返回 [`None`]。
    #[inline]
    pub fn undo_description(&self) -> Option<String> {
        self.undo_stack.last().map(|c| c.description().to_string())
    }

    /// 下一次重做将恢复的命令描述；无可重做内容时返回 [`None`]。
    #[inline]
    pub fn redo_description(&self) -> Option<String> {
        self.redo_stack.last().map(|c| c.description().to_string())
    }

    /// 撤销栈中的命令数量。
    #[inline]
    pub fn undo_stack_size(&self) -> usize {
        self.undo_stack.len()
    }

    /// 重做栈中的命令数量。
    #[inline]
    pub fn redo_stack_size(&self) -> usize {
        self.redo_stack.len()
    }

    /// 清空撤销栈与重做栈，会修改 `self`；`current_state` 与文档内容保持不变。
    #[inline]
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// 开启一批命令的收集器，会修改 `self`（借用期间独占管理器）。
    ///
    /// 返回 [`BatchCommand`]；在其上收集完命令后须调用 [`BatchCommand::finish`] 才会在
    /// 撤销栈中登记一条成组命令。批量对象释放前管理器无法用于其他操作。
    #[inline]
    pub fn begin_batch(&mut self) -> BatchCommand {
        BatchCommand::new(self)
    }

    /// 记录"新增实体"命令，会修改 `self`。
    ///
    /// 命令描述形如 `Add Line`；命令入栈成功后立即用 `entity` 更新内部当前状态快照。
    ///
    /// - `entity`：被加入文档的实体。
    ///
    /// 返回登记的命令；若其时间戳不晚于栈顶，命令未入栈但仍会更新快照。
    #[inline]
    pub fn add_entity(&mut self, entity: &Entity) -> Command {
        let mut command = Command::new(
            CommandType::AddEntity,
            format!("Add {}", entity.entity_type()),
        );
        command.after_state = Some(self.current_state.clone());
        self.execute(command.clone());
        self.current_state.add_entity(entity);
        command
    }

    /// 记录"删除实体"命令，会修改 `self`。
    ///
    /// 命令描述形如 `Remove Line`；入栈后用 `entity_id` 从内部快照中移除该实体，
    /// 但本方法不触碰文档，调用方需自行删除实体。
    ///
    /// - `entity_id`：被删除实体的标识；
    /// - `entity`：被删除实体的数据，仅用于生成描述文字。
    ///
    /// 返回登记的命令。
    #[inline]
    pub fn remove_entity(&mut self, entity_id: &ObjectId, entity: &Entity) -> Command {
        let mut command = Command::new(
            CommandType::RemoveEntity,
            format!("Remove {}", entity.entity_type()),
        );
        command.before_state = Some(self.current_state.clone());
        self.execute(command.clone());
        self.current_state.remove_entity(entity_id);
        command
    }

    /// 记录"修改实体"命令，会修改 `self`。
    ///
    /// 命令描述按修改前的实体类型生成，形如 `Modify Circle`；入栈后在内部快照中把
    /// `entity_id` 对应的实体替换为 `new_entity`。
    ///
    /// - `entity_id`：被修改实体的标识；
    /// - `old_entity`：修改前的实体，仅用于生成描述文字；
    /// - `new_entity`：修改后的实体，写入内部快照。
    ///
    /// 返回登记的命令。注意快照中 `before_state` 与 `after_state` 记录的是同一份当前状态，
    /// 真正的差异数据需由调用方另行保存。
    #[inline]
    pub fn modify_entity(&mut self, entity_id: &ObjectId, old_entity: &Entity, new_entity: &Entity) -> Command {
        let mut command = Command::new(
            CommandType::ModifyEntity,
            format!("Modify {}", old_entity.entity_type()),
        );
        command.before_state = Some(self.current_state.clone());
        command.after_state = Some(self.current_state.clone());
        self.execute(command.clone());
        self.current_state.remove_entity(entity_id);
        self.current_state.add_entity(new_entity);
        command
    }
}

impl Default for CommandManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 批量命令收集器：借用 [`CommandManager`]，把若干命令归并成一次撤销单位。
pub struct BatchCommand<'a> {
    manager: &'a mut CommandManager,
    commands: Vec<Command>,
    description: String,
}

impl<'a> BatchCommand<'a> {
    fn new(manager: &'a mut CommandManager) -> Self {
        Self {
            manager,
            commands: Vec::new(),
            description: String::new(),
        }
    }

    /// 向本批次添加一条命令，会修改 `self`。
    ///
    /// - `command`：待纳入批次的命令；仅暂存，[`BatchCommand::finish`] 时才写入撤销栈。
    #[inline]
    pub fn add_command(&mut self, command: Command) {
        self.commands.push(command);
    }

    /// 设置批次描述，会修改 `self`。
    ///
    /// - `description`：批次名，作为成组命令的描述显示在撤销列表中。
    #[inline]
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    /// 结束批次并提交，会消耗 `self` 并修改被借用的管理器。
    ///
    /// 批次非空时，向管理器登记一条 [`CommandType::Group`] 命令；批次为空则不登记任何内容。
    /// 已收集的子命令不会单独入栈，因此整批只能作为一个整体撤销。
    #[inline]
    pub fn finish(mut self) {
        if !self.commands.is_empty() {
            let batch_command = Command::new(
                CommandType::Group,
                self.description.clone(),
            );
            self.manager.execute(batch_command);
        }
    }
}

impl CommandState {
    /// 新建空白状态快照，四类对象集合均为空。
    #[inline]
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            layers: HashMap::new(),
            blocks: HashMap::new(),
            block_references: HashMap::new(),
        }
    }

    /// 把实体写入快照，会修改 `self`。
    ///
    /// - `entity`：待记录的实体，按值复制；同 id 的旧记录被覆盖。
    #[inline]
    pub fn add_entity(&mut self, entity: &Entity) {
        self.entities.insert(entity.id().clone(), entity.clone());
    }

    /// 从快照中移除实体，会修改 `self`。
    ///
    /// - `entity_id`：待移除实体的标识；不存在时静默无操作。
    #[inline]
    pub fn remove_entity(&mut self, entity_id: &ObjectId) {
        self.entities.remove(entity_id);
    }

    /// 快照中的实体集合，键为实体 id。
    #[inline]
    pub fn entities(&self) -> &HashMap<ObjectId, Entity> {
        &self.entities
    }

    /// 快照中的图层集合，键为图层 id。
    #[inline]
    pub fn layers(&self) -> &HashMap<ObjectId, Layer> {
        &self.layers
    }

    /// 快照中的块定义集合，键为块 id。
    #[inline]
    pub fn blocks(&self) -> &HashMap<ObjectId, Block> {
        &self.blocks
    }

    /// 快照中的块参照集合，键为块参照 id。
    #[inline]
    pub fn block_references(&self) -> &HashMap<ObjectId, BlockReference> {
        &self.block_references
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_manager_creation() {
        let manager = CommandManager::new();
        
        assert!(!manager.can_undo());
        assert!(!manager.can_redo());
        assert_eq!(manager.undo_stack_size(), 0);
        assert_eq!(manager.redo_stack_size(), 0);
    }

    #[test]
    fn test_command_execution() {
        let mut manager = CommandManager::new();
        let command = Command::new(
            CommandType::AddEntity,
            "Add Line".to_string(),
        );
        
        manager.execute(command);
        
        assert!(manager.can_undo());
        assert!(!manager.can_redo());
        assert_eq!(manager.undo_stack_size(), 1);
    }

    #[test]
    fn test_undo_redo() {
        let mut manager = CommandManager::new();
        let command = Command::new(
            CommandType::AddEntity,
            "Add Circle".to_string(),
        );
        
        manager.execute(command);
        assert!(manager.can_undo());
        
        let undone = manager.undo();
        assert!(undone.is_some());
        assert!(!manager.can_undo());
        assert!(manager.can_redo());
        
        let redone = manager.redo();
        assert!(redone.is_some());
        assert!(manager.can_undo());
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_clear_history() {
        let mut manager = CommandManager::new();
        
        for i in 0..10 {
            let command = Command::new(
                CommandType::AddEntity,
                format!("Command {}", i),
            );
            manager.execute(command);
        }
        
        assert_eq!(manager.undo_stack_size(), 10);
        
        manager.clear();
        assert_eq!(manager.undo_stack_size(), 0);
        assert_eq!(manager.redo_stack_size(), 0);
    }

    #[test]
    fn test_max_history_size() {
        let mut manager = CommandManager::new();
        manager.set_max_history_size(5);
        
        for i in 0..10 {
            let command = Command::new(
                CommandType::AddEntity,
                format!("Command {}", i),
            );
            manager.execute(command);
        }
        
        assert_eq!(manager.undo_stack_size(), 5);
    }

    #[test]
    fn test_command_descriptions() {
        let mut manager = CommandManager::new();
        
        let command = Command::new(
            CommandType::AddEntity,
            "Add Rectangle".to_string(),
        );
        manager.execute(command);
        
        assert_eq!(manager.undo_description(), Some("Add Rectangle".to_string()));
        assert_eq!(manager.redo_description(), None);
    }
}
