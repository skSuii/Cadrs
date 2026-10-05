use std::fmt;

/// 可复制的命令对象接口。
///
/// 命令以 `Box<dyn Command>` 形式存放，而 trait 对象不是 `Clone`，因此由本方法
/// 提供副本；实现通常直接写 `Box::new(self.clone())`。
pub trait CommandClone {
    /// 复制当前命令。
    ///
    /// 返回：状态与被复制者一致的新命令对象，之后两者的修改互不影响。
    fn clone_command(&self) -> Box<dyn Command>;
}

/// 可执行命令的统一接口。
///
/// 命令名是注册表（`CommandRegistry`）中的查找键，命令管理器负责调用
/// [`Command::execute`]；是否写入历史由 [`Command::is_undoable`] 决定。
/// 多步交互命令应覆盖 [`Command::receive_input`] 以接收命令行输入。
pub trait Command: CommandClone {
    /// 命令名，注册与查找时会被转为小写。
    fn name(&self) -> &str;
    /// 命令说明，用于界面提示与帮助信息。
    fn description(&self) -> &str;
    /// 执行命令。
    ///
    /// - `context`：命令上下文，提供文档、当前图层、选择集与临时数据。
    /// 返回：执行结果；命令对文档的修改通过 `context` 中的可变文档完成。
    fn execute(&self, context: &mut CommandContext) -> CommandResult;
    /// 撤销本命令已经产生的修改。
    ///
    /// - `context`：与执行时相同的命令上下文。
    /// 返回：撤销结果；不支持的实现通常直接返回 `Success`。
    fn undo(&self, context: &mut CommandContext) -> CommandResult;
    /// 生成执行前的预览实体。
    ///
    /// 返回：供界面实时预览的实体；无预览时为 `None`。本方法不得修改上下文。
    fn preview(&self, context: &CommandContext) -> Option<super::super::data_structure::Entity>;
    /// 执行前是否要求已经选中实体。
    fn requires_selection(&self) -> bool;
    /// 允许参与本命令的实体类型名。
    ///
    /// 返回：类型名切片（如 `&["Dimension"]`）；空切片表示不限制类型。
    fn get_required_entity_types(&self) -> &[&'static str];
    /// 是否可撤销，决定本次执行是否记入命令历史。
    fn is_undoable(&self) -> bool;
    /// 向命令投递一行交互输入。
    ///
    /// - `_input`：用户键入的原始文本。
    /// 返回：默认实现固定返回 `Failed("命令不接受交互输入")`；需要多步交互的命令应覆盖本方法。
    fn receive_input(&mut self, _input: &str) -> CommandResult {
        CommandResult::Failed("命令不接受交互输入".to_string())
    }
}

/// 命令执行的结果。
#[derive(Debug, Clone, PartialEq)]
pub enum CommandResult {
    /// 执行成功，无后续输入需求。
    Success,
    /// 执行失败，字符串为面向用户的失败原因。
    Failed(String),
    /// 用户主动取消，命令未产生修改。
    Canceled,
    /// 命令等待继续输入，字符串为提示信息（如「指定下一点」）。
    RequireInput(String),
}

impl CommandResult {
    /// 是否执行成功（仅 [`CommandResult::Success`] 为真，`RequireInput` 不算成功）。
    pub fn is_success(&self) -> bool {
        matches!(self, CommandResult::Success)
    }

    /// 是否执行失败（仅 [`CommandResult::Failed`] 为真）。
    pub fn is_failed(&self) -> bool {
        matches!(self, CommandResult::Failed(_))
    }

    /// 是否被用户取消。
    pub fn is_canceled(&self) -> bool {
        matches!(self, CommandResult::Canceled)
    }

    /// 取出失败原因。
    ///
    /// 返回：`Failed` 时给出错误文本，其余变体一律为 `None`。
    pub fn error_message(&self) -> Option<&str> {
        match self {
            CommandResult::Failed(msg) => Some(msg),
            _ => None,
        }
    }
}

impl fmt::Display for CommandResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommandResult::Success => write!(f, "Success"),
            CommandResult::Failed(msg) => write!(f, "Failed: {}", msg),
            CommandResult::Canceled => write!(f, "Canceled"),
            CommandResult::RequireInput(prompt) => write!(f, "Require Input: {}", prompt),
        }
    }
}

/// 命令执行上下文：命令访问文档与选择集的通道。
///
/// 字段全部公开；`document` 是对目标文档的可变借用，命令对实体的一切修改都应
/// 通过它完成。上下文只在本命令执行期间有效。
pub struct CommandContext<'a> {
    /// 目标文档；为 `None` 时命令只能做不涉及文档的操作。
    pub document: Option<&'a mut super::super::data_structure::Document>,
    /// 新建实体默认归属的图层名。
    pub active_layer: String,
    /// 当前用户坐标系（UCS）到世界坐标系的变换矩阵。
    pub current_ucs: crate::math::Matrix4,
    /// 当前选择集，命令据此读取用户已选中的实体。
    pub selection_set: crate::selection::SelectionSet,
    /// 当前块空间名，模型空间为 `"ModelSpace"`。
    pub active_block: String,
    /// 当前视口；未关联视图时为 `None`。
    pub viewport: Option<crate::render::viewport::Viewport>,
    /// 最近一次拾取点（世界坐标），供需要基点的命令使用。
    pub current_point: Option<crate::geometry::Point>,
    /// 命令自定义的临时数据，按键存取，随上下文一起销毁。
    pub user_data: std::collections::HashMap<String, Box<dyn std::any::Any>>,
}

impl<'a> Default for CommandContext<'a> {
    fn default() -> Self {
        Self {
            document: None,
            active_layer: "0".to_string(),
            current_ucs: crate::math::Matrix4::identity(),
            selection_set: crate::selection::SelectionSet::new(),
            active_block: "ModelSpace".to_string(),
            viewport: None,
            current_point: None,
            user_data: std::collections::HashMap::new(),
        }
    }
}

impl<'a> CommandContext<'a> {
    /// 以给定文档创建上下文。
    ///
    /// 其余字段取默认值：图层 `"0"`、块空间 `"ModelSpace"`、单位矩阵、空选择集、无拾取点。
    pub fn new(document: &'a mut super::super::data_structure::Document) -> Self {
        Self {
            document: Some(document),
            active_layer: "0".to_string(),
            current_ucs: crate::math::Matrix4::identity(),
            selection_set: crate::selection::SelectionSet::new(),
            active_block: "ModelSpace".to_string(),
            viewport: None,
            current_point: None,
            user_data: std::collections::HashMap::new(),
        }
    }

    /// 只读借用目标文档。
    ///
    /// 返回：上下文未绑定文档时为 `None`。
    pub fn get_document(&self) -> Option<&super::super::data_structure::Document> {
        self.document.as_deref()
    }

    /// 可变借用目标文档，供命令增删改实体。
    ///
    /// 返回：上下文未绑定文档时为 `None`。
    pub fn get_document_mut(&mut self) -> Option<&mut super::super::data_structure::Document> {
        self.document.as_deref_mut()
    }

    /// 写入命令的临时数据，同一键的旧值被覆盖。
    ///
    /// - `key`：字符串键；`value`：任意 `'static` 类型，取出时须使用相同类型。
    pub fn set_user_data<T: 'static>(&mut self, key: impl Into<String>, value: T) {
        self.user_data.insert(key.into(), Box::new(value));
    }

    /// 读取命令的临时数据。
    ///
    /// 返回：键存在且存储类型正是 `T` 时给出共享引用，键不存在或类型不符均为 `None`。
    pub fn get_user_data<T: 'static>(&self, key: &str) -> Option<&T> {
        self.user_data.get(key).and_then(|boxed| {
            boxed.downcast_ref::<T>()
        })
    }

    /// 取出并移除命令的临时数据（所有权转移）。
    ///
    /// 返回：键存在且存储类型正是 `T` 时给出数据本身，否则为 `None`。
    pub fn take_user_data<T: 'static>(&mut self, key: &str) -> Option<Box<T>> {
        self.user_data.remove(key).and_then(|boxed| {
            boxed.downcast::<T>().ok()
        })
    }

    /// 把一个实体加入当前选择集，已存在时不重复添加。
    pub fn add_selection(&mut self, entity_id: super::super::data_structure::ObjectId) {
        self.selection_set.add(entity_id);
    }

    /// 清空当前选择集，同时清除「最后选中」记录。
    pub fn clear_selection(&mut self) {
        self.selection_set.clear();
    }

    /// 复制出当前选中的全部实体标识。
    ///
    /// 返回：`ObjectId` 列表，顺序与加入选择集的顺序一致；未选中任何实体时为空列表。
    pub fn get_selected_entities(&self) -> Vec<super::super::data_structure::ObjectId> {
        self.selection_set.get_selected().to_vec()
    }
}

/// 命令构建器：按名称、描述与分类创建命令对象。
///
/// 采用链式调用设置字段，最后由 [`CommandBuilder::build`] 校验并生成
/// `Box<dyn Command>`；未显式设置分类时使用 [`CommandType::EntityOperation`]。
pub struct CommandBuilder {
    name: String,
    description: String,
    command_type: CommandType,
}

impl Default for CommandBuilder {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            command_type: CommandType::EntityOperation,
        }
    }
}

/// 命令分类，决定 [`CommandBuilder::build`] 生成的实现及其选择/撤销行为。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandType {
    /// 实体操作：要求先选择实体，可撤销。
    EntityOperation,
    /// 绘图辅助（栅格、捕捉等）：不要求选择，不可撤销。
    DrawingAid,
    /// 显示控制（缩放、平移等）：不要求选择，不可撤销。
    DisplayControl,
    /// 文件操作（打开、保存等）：不要求选择，不可撤销。
    FileOperation,
    /// 图层控制：要求先选择实体，可撤销。
    LayerControl,
    /// 块操作：要求先选择实体，可撤销。
    BlockOperation,
    /// 尺寸标注：要求实体类型为 `"Dimension"`，可撤销。
    Dimension,
    /// 文字：不要求选择，可撤销。
    Text,
    /// 选择操作：不要求选择，不可撤销。
    Selection,
    /// 未知分类：执行与撤销均固定失败，仅作占位。
    Unknown,
}

impl CommandBuilder {
    /// 创建使用默认值的构建器（分类为 [`CommandType::EntityOperation`]）。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置命令名，注册表中以此名（小写）查找；为空时 [`CommandBuilder::build`] 报错。
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// 设置命令说明；为空时 [`CommandBuilder::build`] 报错。
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// 设置命令分类，决定生成实现的类型、选择要求与可撤销性。
    pub fn command_type(mut self, cmd_type: CommandType) -> Self {
        self.command_type = cmd_type;
        self
    }

    /// 校验并生成命令对象。
    ///
    /// 返回：名称为空报 `CadError::Command("build", "命令名称不能为空")`，描述为空报
    /// `CadError::Command`；否则按 [`CommandType`] 生成对应实现。
    pub fn build(self) -> Result<Box<dyn Command>, crate::error::CadError> {
        use crate::error::CadError;

        if self.name.is_empty() {
            return Err(CadError::command("build", "命令名称不能为空"));
        }

        if self.description.is_empty() {
            return Err(CadError::command(&self.name, "命令描述不能为空"));
        }

        match self.command_type {
            CommandType::EntityOperation => {
                Ok(Box::new(EntityCommand::new(self.name, self.description)))
            }
            CommandType::DrawingAid => {
                Ok(Box::new(DrawingAidCommand::new(self.name, self.description)))
            }
            CommandType::DisplayControl => {
                Ok(Box::new(DisplayControlCommand::new(self.name, self.description)))
            }
            CommandType::FileOperation => {
                Ok(Box::new(FileOperationCommand::new(self.name, self.description)))
            }
            CommandType::LayerControl => {
                Ok(Box::new(LayerControlCommand::new(self.name, self.description)))
            }
            CommandType::BlockOperation => {
                Ok(Box::new(BlockOperationCommand::new(self.name, self.description)))
            }
            CommandType::Dimension => {
                Ok(Box::new(DimensionCommand::new(self.name, self.description)))
            }
            CommandType::Text => {
                Ok(Box::new(TextCommand::new(self.name, self.description)))
            }
            CommandType::Selection => {
                Ok(Box::new(SelectionCommand::new(self.name, self.description)))
            }
            CommandType::Unknown => {
                Ok(Box::new(GenericCommand::new(self.name, self.description,
                    |_| CommandResult::Failed("命令未实现".to_string()),
                    |_| CommandResult::Failed("无法撤销未实现的命令".to_string())
                )))
            }
        }
    }
}

#[derive(Clone)]
struct EntityCommand {
    name: String,
    description: String,
}

impl EntityCommand {
    fn new(name: String, description: String) -> Self {
        Self { name, description }
    }
}

impl CommandClone for EntityCommand {
    fn clone_command(&self) -> Box<dyn Command> { Box::new(self.clone()) }
}

impl Command for EntityCommand {
    fn name(&self) -> &str { &self.name }
    fn description(&self) -> &str { &self.description }
    fn execute(&self, _: &mut CommandContext) -> CommandResult { CommandResult::Success }
    fn undo(&self, _: &mut CommandContext) -> CommandResult { CommandResult::Success }
    fn preview(&self, _: &CommandContext) -> Option<super::super::data_structure::Entity> { None }
    fn requires_selection(&self) -> bool { true }
    fn get_required_entity_types(&self) -> &[&'static str] { &[] }
    fn is_undoable(&self) -> bool { true }
}

#[derive(Clone)]
struct DrawingAidCommand {
    name: String,
    description: String,
}

impl DrawingAidCommand {
    fn new(name: String, description: String) -> Self {
        Self { name, description }
    }
}

impl CommandClone for DrawingAidCommand {
    fn clone_command(&self) -> Box<dyn Command> { Box::new(self.clone()) }
}

impl Command for DrawingAidCommand {
    fn name(&self) -> &str { &self.name }
    fn description(&self) -> &str { &self.description }
    fn execute(&self, _: &mut CommandContext) -> CommandResult { CommandResult::Success }
    fn undo(&self, _: &mut CommandContext) -> CommandResult { CommandResult::Success }
    fn preview(&self, _: &CommandContext) -> Option<super::super::data_structure::Entity> { None }
    fn requires_selection(&self) -> bool { false }
    fn get_required_entity_types(&self) -> &[&'static str] { &[] }
    fn is_undoable(&self) -> bool { false }
}

#[derive(Clone)]
struct DisplayControlCommand {
    name: String,
    description: String,
}

impl DisplayControlCommand {
    fn new(name: String, description: String) -> Self {
        Self { name, description }
    }
}

impl CommandClone for DisplayControlCommand {
    fn clone_command(&self) -> Box<dyn Command> { Box::new(self.clone()) }
}

impl Command for DisplayControlCommand {
    fn name(&self) -> &str { &self.name }
    fn description(&self) -> &str { &self.description }
    fn execute(&self, _: &mut CommandContext) -> CommandResult { CommandResult::Success }
    fn undo(&self, _: &mut CommandContext) -> CommandResult { CommandResult::Success }
    fn preview(&self, _: &CommandContext) -> Option<super::super::data_structure::Entity> { None }
    fn requires_selection(&self) -> bool { false }
    fn get_required_entity_types(&self) -> &[&'static str] { &[] }
    fn is_undoable(&self) -> bool { false }
}

#[derive(Clone)]
struct FileOperationCommand {
    name: String,
    description: String,
}

impl FileOperationCommand {
    fn new(name: String, description: String) -> Self {
        Self { name, description }
    }
}

impl CommandClone for FileOperationCommand {
    fn clone_command(&self) -> Box<dyn Command> { Box::new(self.clone()) }
}

impl Command for FileOperationCommand {
    fn name(&self) -> &str { &self.name }
    fn description(&self) -> &str { &self.description }
    fn execute(&self, _: &mut CommandContext) -> CommandResult { CommandResult::Success }
    fn undo(&self, _: &mut CommandContext) -> CommandResult { CommandResult::Success }
    fn preview(&self, _: &CommandContext) -> Option<super::super::data_structure::Entity> { None }
    fn requires_selection(&self) -> bool { false }
    fn get_required_entity_types(&self) -> &[&'static str] { &[] }
    fn is_undoable(&self) -> bool { false }
}

#[derive(Clone)]
struct LayerControlCommand {
    name: String,
    description: String,
}

impl LayerControlCommand {
    fn new(name: String, description: String) -> Self {
        Self { name, description }
    }
}

impl CommandClone for LayerControlCommand {
    fn clone_command(&self) -> Box<dyn Command> { Box::new(self.clone()) }
}

impl Command for LayerControlCommand {
    fn name(&self) -> &str { &self.name }
    fn description(&self) -> &str { &self.description }
    fn execute(&self, _: &mut CommandContext) -> CommandResult { CommandResult::Success }
    fn undo(&self, _: &mut CommandContext) -> CommandResult { CommandResult::Success }
    fn preview(&self, _: &CommandContext) -> Option<super::super::data_structure::Entity> { None }
    fn requires_selection(&self) -> bool { true }
    fn get_required_entity_types(&self) -> &[&'static str] { &[] }
    fn is_undoable(&self) -> bool { true }
}

#[derive(Clone)]
struct BlockOperationCommand {
    name: String,
    description: String,
}

impl BlockOperationCommand {
    fn new(name: String, description: String) -> Self {
        Self { name, description }
    }
}

impl CommandClone for BlockOperationCommand {
    fn clone_command(&self) -> Box<dyn Command> { Box::new(self.clone()) }
}

impl Command for BlockOperationCommand {
    fn name(&self) -> &str { &self.name }
    fn description(&self) -> &str { &self.description }
    fn execute(&self, _: &mut CommandContext) -> CommandResult { CommandResult::Success }
    fn undo(&self, _: &mut CommandContext) -> CommandResult { CommandResult::Success }
    fn preview(&self, _: &CommandContext) -> Option<super::super::data_structure::Entity> { None }
    fn requires_selection(&self) -> bool { true }
    fn get_required_entity_types(&self) -> &[&'static str] { &[] }
    fn is_undoable(&self) -> bool { true }
}

#[derive(Clone)]
struct DimensionCommand {
    name: String,
    description: String,
}

impl DimensionCommand {
    fn new(name: String, description: String) -> Self {
        Self { name, description }
    }
}

impl CommandClone for DimensionCommand {
    fn clone_command(&self) -> Box<dyn Command> { Box::new(self.clone()) }
}

impl Command for DimensionCommand {
    fn name(&self) -> &str { &self.name }
    fn description(&self) -> &str { &self.description }
    fn execute(&self, _: &mut CommandContext) -> CommandResult { CommandResult::Success }
    fn undo(&self, _: &mut CommandContext) -> CommandResult { CommandResult::Success }
    fn preview(&self, _: &CommandContext) -> Option<super::super::data_structure::Entity> { None }
    fn requires_selection(&self) -> bool { true }
    fn get_required_entity_types(&self) -> &[&'static str] { &["Dimension"] }
    fn is_undoable(&self) -> bool { true }
}

#[derive(Clone)]
struct TextCommand {
    name: String,
    description: String,
}

impl TextCommand {
    fn new(name: String, description: String) -> Self {
        Self { name, description }
    }
}

impl CommandClone for TextCommand {
    fn clone_command(&self) -> Box<dyn Command> { Box::new(self.clone()) }
}

impl Command for TextCommand {
    fn name(&self) -> &str { &self.name }
    fn description(&self) -> &str { &self.description }
    fn execute(&self, _: &mut CommandContext) -> CommandResult { CommandResult::Success }
    fn undo(&self, _: &mut CommandContext) -> CommandResult { CommandResult::Success }
    fn preview(&self, _: &CommandContext) -> Option<super::super::data_structure::Entity> { None }
    fn requires_selection(&self) -> bool { false }
    fn get_required_entity_types(&self) -> &[&'static str] { &[] }
    fn is_undoable(&self) -> bool { true }
}

#[derive(Clone)]
struct SelectionCommand {
    name: String,
    description: String,
}

impl SelectionCommand {
    fn new(name: String, description: String) -> Self {
        Self { name, description }
    }
}

impl CommandClone for SelectionCommand {
    fn clone_command(&self) -> Box<dyn Command> { Box::new(self.clone()) }
}

impl Command for SelectionCommand {
    fn name(&self) -> &str { &self.name }
    fn description(&self) -> &str { &self.description }
    fn execute(&self, _: &mut CommandContext) -> CommandResult { CommandResult::Success }
    fn undo(&self, _: &mut CommandContext) -> CommandResult { CommandResult::Success }
    fn preview(&self, _: &CommandContext) -> Option<super::super::data_structure::Entity> { None }
    fn requires_selection(&self) -> bool { false }
    fn get_required_entity_types(&self) -> &[&'static str] { &[] }
    fn is_undoable(&self) -> bool { false }
}

/// 通用命令：把执行与撤销逻辑以闭包注入的命令实现。
///
/// 适合快速包装一段自定义逻辑，两个闭包分别对应 [`Command::execute`] 与
/// [`Command::undo`]，需为 `Send + Sync`，因为命令对象会被复制到管理器与宏中。
#[derive(Clone)]
pub struct GenericCommand {
    name: String,
    description: String,
    execute_fn: std::sync::Arc<dyn Fn(&mut CommandContext) -> CommandResult + Send + Sync>,
    undo_fn: std::sync::Arc<dyn Fn(&mut CommandContext) -> CommandResult + Send + Sync>,
    requires_selection: bool,
    required_entity_types: Vec<&'static str>,
}

impl GenericCommand {
    /// 用名称、描述与两个闭包创建命令。
    ///
    /// - `execute_fn`：执行体，接收命令上下文并返回结果；`undo_fn`：撤销体，语义与执行体相反。
    /// 返回：默认不要求选择实体、不限制实体类型，可用 [`GenericCommand::with_selection_requirement`] 调整。
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        execute_fn: impl Fn(&mut CommandContext) -> CommandResult + 'static + Send + Sync,
        undo_fn: impl Fn(&mut CommandContext) -> CommandResult + 'static + Send + Sync,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            execute_fn: std::sync::Arc::new(execute_fn),
            undo_fn: std::sync::Arc::new(undo_fn),
            requires_selection: false,
            required_entity_types: Vec::new(),
        }
    }

    /// 设置执行前是否必须已有选择，以及允许参与命令的实体类型。
    ///
    /// - `required`：为 `true` 时调用方需先选中实体；`entity_types`：允许的类型名，空表示不限制。
    /// 返回：修改后的自身，便于链式调用。
    pub fn with_selection_requirement(mut self, required: bool, entity_types: Vec<&'static str>) -> Self {
        self.requires_selection = required;
        self.required_entity_types = entity_types;
        self
    }
}

impl CommandClone for GenericCommand {
    fn clone_command(&self) -> Box<dyn Command> { Box::new(self.clone()) }
}

impl Command for GenericCommand {

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn execute(&self, context: &mut CommandContext) -> CommandResult {
        (self.execute_fn)(context)
    }

    fn undo(&self, context: &mut CommandContext) -> CommandResult {
        (self.undo_fn)(context)
    }

    fn preview(&self, _context: &CommandContext) -> Option<super::super::data_structure::Entity> {
        None
    }

    fn requires_selection(&self) -> bool {
        self.requires_selection
    }

    fn get_required_entity_types(&self) -> &[&'static str] {
        &self.required_entity_types
    }

    fn is_undoable(&self) -> bool {
        true
    }
}

impl fmt::Display for GenericCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Command({})", self.name)
    }
}
