use std::fmt;

pub trait CommandClone {
    fn clone_command(&self) -> Box<dyn Command>;
}

pub trait Command: CommandClone {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, context: &mut CommandContext) -> CommandResult;
    fn undo(&self, context: &mut CommandContext) -> CommandResult;
    fn preview(&self, context: &CommandContext) -> Option<super::super::data_structure::Entity>;
    fn requires_selection(&self) -> bool;
    fn get_required_entity_types(&self) -> &[&'static str];
    fn is_undoable(&self) -> bool;
    fn receive_input(&mut self, _input: &str) -> CommandResult {
        CommandResult::Failed("命令不接受交互输入".to_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandResult {
    Success,
    Failed(String),
    Canceled,
    RequireInput(String),
}

impl CommandResult {
    pub fn is_success(&self) -> bool {
        matches!(self, CommandResult::Success)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, CommandResult::Failed(_))
    }

    pub fn is_canceled(&self) -> bool {
        matches!(self, CommandResult::Canceled)
    }

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

pub struct CommandContext<'a> {
    pub document: Option<&'a mut super::super::data_structure::Document>,
    pub active_layer: String,
    pub current_ucs: crate::math::Matrix4,
    pub selection_set: crate::selection::SelectionSet,
    pub active_block: String,
    pub viewport: Option<crate::render::viewport::Viewport>,
    pub current_point: Option<crate::geometry::Point>,
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

    pub fn get_document(&self) -> Option<&super::super::data_structure::Document> {
        self.document.as_deref()
    }

    pub fn get_document_mut(&mut self) -> Option<&mut super::super::data_structure::Document> {
        self.document.as_deref_mut()
    }

    pub fn set_user_data<T: 'static>(&mut self, key: impl Into<String>, value: T) {
        self.user_data.insert(key.into(), Box::new(value));
    }

    pub fn get_user_data<T: 'static>(&self, key: &str) -> Option<&T> {
        self.user_data.get(key).and_then(|boxed| {
            boxed.downcast_ref::<T>()
        })
    }

    pub fn take_user_data<T: 'static>(&mut self, key: &str) -> Option<Box<T>> {
        self.user_data.remove(key).and_then(|boxed| {
            boxed.downcast::<T>().ok()
        })
    }

    pub fn add_selection(&mut self, entity_id: super::super::data_structure::ObjectId) {
        self.selection_set.add(entity_id);
    }

    pub fn clear_selection(&mut self) {
        self.selection_set.clear();
    }

    pub fn get_selected_entities(&self) -> Vec<super::super::data_structure::ObjectId> {
        self.selection_set.get_selected().to_vec()
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandType {
    EntityOperation,
    DrawingAid,
    DisplayControl,
    FileOperation,
    LayerControl,
    BlockOperation,
    Dimension,
    Text,
    Selection,
    Unknown,
}

impl CommandBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn command_type(mut self, cmd_type: CommandType) -> Self {
        self.command_type = cmd_type;
        self
    }

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
