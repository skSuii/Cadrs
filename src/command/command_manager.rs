use std::fmt;

use super::command::CommandClone;

/// 命令注册表：按名称保存命令对象，并维护别名到主名称的映射。
///
/// 名称在注册、查找、注销时统一转成小写，因此 `"LINE"` 与 `"line"` 等价；
/// 表中只保存 `Box<dyn Command>`，取出的是借用或经 `CommandClone` 得到的副本。
pub struct CommandRegistry {
    commands: std::collections::HashMap<String, Box<dyn super::Command>>,
    aliases: std::collections::HashMap<String, String>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self {
            commands: std::collections::HashMap::new(),
            aliases: std::collections::HashMap::new(),
        }
    }
}

impl CommandRegistry {
    /// 创建空注册表（无命令、无别名）。
    pub fn new() -> Self {
        Self::default()
    }

    /// 以命令自身名称（转小写）为键注册命令。
    ///
    /// 返回：注册成功为 `true`；同名命令已存在时为 `false`，且不覆盖原有实现。
    pub fn register<C: super::Command + 'static>(&mut self, command: C) -> bool {
        let name = command.name().to_lowercase();
        if self.commands.contains_key(&name) {
            return false;
        }
        self.commands.insert(name, Box::new(command));
        true
    }

    /// 注册命令并同时建立一个别名（别名同样转小写保存）。
    ///
    /// - `command`：待注册命令；`alias`：别名。
    /// 返回：主名称已存在时为 `false`，此时命令与别名都不会写入；否则为 `true`。
    pub fn register_with_alias<C: super::Command + 'static>(
        &mut self,
        command: C,
        alias: &str,
    ) -> bool {
        let name = command.name().to_lowercase();
        if self.commands.contains_key(&name) {
            return false;
        }
        self.commands.insert(name.clone(), Box::new(command));
        self.aliases.insert(alias.to_lowercase(), name);
        true
    }

    /// 注销命令，并清除指向它的全部别名。
    ///
    /// 返回：确实删除了命令为 `true`，命令原本不存在为 `false`。
    pub fn unregister(&mut self, name: &str) -> bool {
        let name = name.to_lowercase();
        self.aliases.retain(|_, v| *v != name);
        self.commands.remove(&name).is_some()
    }

    /// 按名称或别名查找命令（不区分大小写）。
    ///
    /// 返回：命中时给出命令的只读引用，未命中为 `None`。
    pub fn get(&self, name: &str) -> Option<&dyn super::Command> {
        let name = name.to_lowercase();
        if let Some(cmd) = self.commands.get(&name) {
            Some(cmd.as_ref())
        } else if let Some(real_name) = self.aliases.get(&name) {
            self.commands.get(real_name).map(|c| c.as_ref())
        } else {
            None
        }
    }

    /// 按名称或别名查找命令并取得可变引用，用于投递交互输入。
    ///
    /// 返回：命中时给出可变引用，未命中为 `None`。
    pub fn get_mut(&mut self, name: &str) -> Option<&mut (dyn super::Command + '_)> {
        let name = name.to_lowercase();
        let key = if self.commands.contains_key(&name) {
            name
        } else {
            self.aliases.get(&name)?.clone()
        };
        match self.commands.get_mut(&key) {
            Some(cmd) => Some(cmd.as_mut()),
            None => None,
        }
    }

    /// 判断名称或别名是否已被占用（不区分大小写）。
    pub fn command_exists(&self, name: &str) -> bool {
        let name = name.to_lowercase();
        self.commands.contains_key(&name) || self.aliases.contains_key(&name)
    }

    /// 列出已注册命令的主名称（不含别名，顺序不保证）。
    pub fn get_command_names(&self) -> Vec<&str> {
        self.commands.keys().map(|s| s.as_str()).collect()
    }

    /// 为已注册的命令追加别名。
    ///
    /// 返回：目标命令存在时写入别名并返回 `true`；目标命令未注册时返回 `false`。
    /// 一个别名只指向一条命令，重复添加会覆盖旧映射。
    pub fn add_alias(&mut self, alias: &str, command_name: &str) -> bool {
        if self.commands.contains_key(&command_name.to_lowercase()) {
            self.aliases.insert(alias.to_lowercase(), command_name.to_lowercase());
            true
        } else {
            false
        }
    }

    /// 删除别名。
    ///
    /// 返回：别名原本存在为 `true`，否则为 `false`；被指向的命令不受影响。
    pub fn remove_alias(&mut self, alias: &str) -> bool {
        self.aliases.remove(&alias.to_lowercase()).is_some()
    }

    /// 列出全部别名及其指向的命令名（顺序不保证）。
    pub fn get_aliases(&self) -> Vec<(&str, &str)> {
        self.aliases
            .iter()
            .map(|(alias, cmd)| (alias.as_str(), cmd.as_str()))
            .collect()
    }
}

impl fmt::Display for CommandRegistry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CommandRegistry(commands={}, aliases={})", self.commands.len(), self.aliases.len())
    }
}

/// 命令管理器：在注册表之上维护当前命令、命令栈与宏录制状态。
///
/// 提供两类使用方式：一次性执行 [`CommandManager::execute_command`]，以及多步命令的
/// 生命周期管理（[`start_command`](CommandManager::start_command) →
/// [`continue_command`](CommandManager::continue_command) →
/// [`end_command`](CommandManager::end_command)）。执行时使用调用者传入的上下文。
pub struct CommandManager {
    registry: CommandRegistry,
    current_command: Option<Box<dyn super::Command>>,
    context: super::CommandContext<'static>,
    command_stack: Vec<Box<dyn super::Command>>,
    is_recording_macro: bool,
    macro_commands: Vec<Box<dyn super::Command>>,
}

impl Default for CommandManager {
    fn default() -> Self {
        Self {
            registry: CommandRegistry::new(),
            current_command: None,
            context: super::CommandContext::default(),
            command_stack: Vec::new(),
            is_recording_macro: false,
            macro_commands: Vec::new(),
        }
    }
}

impl CommandManager {
    /// 创建不含任何命令的管理器（空注册表、无当前命令、未在录制宏）。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册命令，委托给内部注册表。
    ///
    /// 返回：成功为 `true`，同名命令已存在为 `false`。
    pub fn register_command<C: super::Command + 'static>(&mut self, command: C) -> bool {
        self.registry.register(command)
    }

    /// 注销命令并清除其别名。
    ///
    /// 返回：删除成功为 `true`，命令不存在为 `false`。
    pub fn unregister_command(&mut self, name: &str) -> bool {
        self.registry.unregister(name)
    }

    /// 立即执行一次命名命令。
    ///
    /// - `name`：命令名或别名（不区分大小写）；`context`：本次执行使用的上下文，命令对
    ///   文档的修改经它完成，且不使用管理器内部的上下文。
    /// 返回：命令自身的执行结果；命令未注册时为 `Failed("Command '<name>' not found")`。
    /// 副作用：更新当前命令；若正在录制宏且执行成功，本次命令会被追加到宏中。
    pub fn execute_command(
        &mut self,
        name: &str,
        context: &mut super::CommandContext,
    ) -> super::CommandResult {
        if let Some(cmd) = self.registry.get(name) {
            self.current_command = Some(self.registry.get(name).unwrap().clone_command());
            let result = cmd.execute(context);
            if result.is_success() && self.is_recording_macro {
                if let Some(current) = &self.current_command {
                    self.macro_commands.push(current.clone_command());
                }
            }
            result
        } else {
            super::CommandResult::Failed(format!("Command '{}' not found", name))
        }
    }

    /// 启动一个多步命令：把它设为当前命令，但本次不执行。
    ///
    /// 返回：找到命令为 `Success`；未注册为 `Failed("Command '<name>' not found")`。
    pub fn start_command(&mut self, name: &str) -> super::CommandResult {
        if let Some(cmd) = self.registry.get(name) {
            self.current_command = Some(cmd.clone_command());
            super::CommandResult::Success
        } else {
            super::CommandResult::Failed(format!("Command '{}' not found", name))
        }
    }

    /// 把一行交互输入交给当前命令处理。
    ///
    /// - `input`：用户键入的原始文本，转交 [`Command::receive_input`](super::Command::receive_input)。
    /// 返回：命令的应答；当前没有活动命令时为 `Failed("No active command")`。
    pub fn continue_command(
        &mut self,
        input: &str,
    ) -> super::CommandResult {
        if let Some(ref mut cmd) = self.current_command {
            cmd.receive_input(input)
        } else {
            super::CommandResult::Failed("No active command".to_string())
        }
    }

    /// 结束当前命令：清空当前命令，但保留命令栈内容。
    ///
    /// 返回：恒为 `Success`。
    pub fn end_command(&mut self) -> super::CommandResult {
        self.current_command = None;
        super::CommandResult::Success
    }

    /// 取消当前命令并清空整个命令栈，无返回值。
    pub fn cancel_command(&mut self) {
        self.current_command = None;
        self.command_stack.clear();
    }

    /// 把命令压入命令栈；若当前没有活动命令，则同时把它设为当前命令。
    ///
    /// 返回：成功为 `Success`；命令未注册为 `Failed("Command '<name>' not found")`。
    pub fn push_command(&mut self, name: &str) -> super::CommandResult {
        if let Some(cmd) = self.registry.get(name) {
            self.command_stack.push(cmd.clone_command());
            if self.current_command.is_none() {
                self.current_command = Some(self.command_stack.last().unwrap().clone_command());
            }
            super::CommandResult::Success
        } else {
            super::CommandResult::Failed(format!("Command '{}' not found", name))
        }
    }

    /// 弹出栈顶命令，并把新的栈顶设为当前命令。
    ///
    /// 返回：被弹出的命令副本（所有权转移）；栈为空时为 `None` 且当前命令被清空。
    pub fn pop_command(&mut self) -> Option<Box<dyn super::Command>> {
        let cmd = self.command_stack.pop();
        self.current_command = self.command_stack.last().cloned();
        cmd
    }

    /// 取得当前命令的只读引用。
    ///
    /// 返回：没有活动命令时为 `None`。
    pub fn get_current_command(&self) -> Option<&dyn super::Command> {
        self.current_command.as_deref()
    }

    /// 开始录制宏：清空上一次的录制内容并进入录制状态。
    pub fn start_macro_recording(&mut self) {
        self.is_recording_macro = true;
        self.macro_commands.clear();
    }

    /// 结束宏录制并返回录制到的命令。
    ///
    /// 返回：录制期间成功执行的命令副本，顺序即执行顺序；未录制时为空列表。
    /// 副作用：退出录制状态，但不清空内部缓冲，重复调用返回同一批命令。
    pub fn stop_macro_recording(&mut self) -> Vec<Box<dyn super::Command>> {
        self.is_recording_macro = false;
        self.macro_commands.clone()
    }

    /// 判断当前是否处于宏录制状态。
    pub fn is_recording_macro(&self) -> bool {
        self.is_recording_macro
    }

    /// 列出已注册命令的名称，等价于 [`CommandRegistry::get_command_names`]。
    pub fn get_registered_commands(&self) -> Vec<&str> {
        self.registry.get_command_names()
    }

    /// 判断命令名或别名是否已注册，等价于 [`CommandRegistry::command_exists`]。
    pub fn command_exists(&self, name: &str) -> bool {
        self.registry.command_exists(name)
    }
}

impl super::CommandClone for Box<dyn super::Command> {
    fn clone_command(&self) -> Box<dyn super::Command> {
        (**self).clone_command()
    }
}

impl super::Command for Box<dyn super::Command> {
    fn name(&self) -> &str {
        (**self).name()
    }

    fn description(&self) -> &str {
        (**self).description()
    }

    fn execute(&self, context: &mut super::CommandContext) -> super::CommandResult {
        (**self).execute(context)
    }

    fn undo(&self, context: &mut super::CommandContext) -> super::CommandResult {
        (**self).undo(context)
    }

    fn preview(&self, context: &super::CommandContext) -> Option<super::super::data_structure::Entity> {
        (**self).preview(context)
    }

    fn requires_selection(&self) -> bool {
        (**self).requires_selection()
    }

    fn get_required_entity_types(&self) -> &[&'static str] {
        (**self).get_required_entity_types()
    }

    fn is_undoable(&self) -> bool {
        (**self).is_undoable()
    }
}

impl Clone for Box<dyn super::Command> {
    fn clone(&self) -> Self {
        self.clone_command()
    }
}

impl fmt::Display for CommandManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CommandManager(commands={}, current={})",
            self.registry.get_command_names().len(),
            self.current_command.as_ref().map(|c| c.name()).unwrap_or("None")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_registry() {
        let mut registry = CommandRegistry::new();

        struct TestCommand;
        impl super::super::CommandClone for TestCommand {
            fn clone_command(&self) -> Box<dyn super::super::Command> { Box::new(TestCommand) }
        }

        impl super::super::Command for TestCommand {
            fn name(&self) -> &str { "test" }
            fn description(&self) -> &str { "Test command" }
            fn execute(&self, _: &mut super::super::CommandContext) -> super::super::CommandResult {
                super::super::CommandResult::Success
            }
            fn undo(&self, _: &mut super::super::CommandContext) -> super::super::CommandResult {
                super::super::CommandResult::Success
            }
            fn preview(&self, _: &super::super::CommandContext) -> Option<super::super::super::data_structure::Entity> { None }
            fn requires_selection(&self) -> bool { false }
            fn get_required_entity_types(&self) -> &[&'static str] { &[] }
            fn is_undoable(&self) -> bool { true }
        }

        assert!(registry.register(TestCommand));
        assert!(!registry.register(TestCommand));

        assert!(registry.command_exists("test"));
        assert!(registry.get("test").is_some());

        assert_eq!(registry.get_command_names(), vec!["test"]);
    }

    #[test]
    fn test_alias() {
        let mut registry = CommandRegistry::new();

        struct TestCommand;
        impl super::super::CommandClone for TestCommand {
            fn clone_command(&self) -> Box<dyn super::super::Command> { Box::new(TestCommand) }
        }

        impl super::super::Command for TestCommand {
            fn name(&self) -> &str { "test" }
            fn description(&self) -> &str { "Test command" }
            fn execute(&self, _: &mut super::super::CommandContext) -> super::super::CommandResult {
                super::super::CommandResult::Success
            }
            fn undo(&self, _: &mut super::super::CommandContext) -> super::super::CommandResult {
                super::super::CommandResult::Success
            }
            fn preview(&self, _: &super::super::CommandContext) -> Option<super::super::super::data_structure::Entity> { None }
            fn requires_selection(&self) -> bool { false }
            fn get_required_entity_types(&self) -> &[&'static str] { &[] }
            fn is_undoable(&self) -> bool { true }
        }

        registry.register_with_alias(TestCommand, "t");
        assert!(registry.command_exists("t"));
        assert!(registry.get("t").is_some());
    }
}
