use std::fmt;

use super::command::CommandClone;

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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<C: super::Command + 'static>(&mut self, command: C) -> bool {
        let name = command.name().to_lowercase();
        if self.commands.contains_key(&name) {
            return false;
        }
        self.commands.insert(name, Box::new(command));
        true
    }

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

    pub fn unregister(&mut self, name: &str) -> bool {
        let name = name.to_lowercase();
        self.aliases.retain(|_, v| *v != name);
        self.commands.remove(&name).is_some()
    }

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

    pub fn command_exists(&self, name: &str) -> bool {
        let name = name.to_lowercase();
        self.commands.contains_key(&name) || self.aliases.contains_key(&name)
    }

    pub fn get_command_names(&self) -> Vec<&str> {
        self.commands.keys().map(|s| s.as_str()).collect()
    }

    pub fn add_alias(&mut self, alias: &str, command_name: &str) -> bool {
        if self.commands.contains_key(&command_name.to_lowercase()) {
            self.aliases.insert(alias.to_lowercase(), command_name.to_lowercase());
            true
        } else {
            false
        }
    }

    pub fn remove_alias(&mut self, alias: &str) -> bool {
        self.aliases.remove(&alias.to_lowercase()).is_some()
    }

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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_command<C: super::Command + 'static>(&mut self, command: C) -> bool {
        self.registry.register(command)
    }

    pub fn unregister_command(&mut self, name: &str) -> bool {
        self.registry.unregister(name)
    }

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

    pub fn start_command(&mut self, name: &str) -> super::CommandResult {
        if let Some(cmd) = self.registry.get(name) {
            self.current_command = Some(cmd.clone_command());
            super::CommandResult::Success
        } else {
            super::CommandResult::Failed(format!("Command '{}' not found", name))
        }
    }

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

    pub fn end_command(&mut self) -> super::CommandResult {
        self.current_command = None;
        super::CommandResult::Success
    }

    pub fn cancel_command(&mut self) {
        self.current_command = None;
        self.command_stack.clear();
    }

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

    pub fn pop_command(&mut self) -> Option<Box<dyn super::Command>> {
        let cmd = self.command_stack.pop();
        self.current_command = self.command_stack.last().cloned();
        cmd
    }

    pub fn get_current_command(&self) -> Option<&dyn super::Command> {
        self.current_command.as_deref()
    }

    pub fn start_macro_recording(&mut self) {
        self.is_recording_macro = true;
        self.macro_commands.clear();
    }

    pub fn stop_macro_recording(&mut self) -> Vec<Box<dyn super::Command>> {
        self.is_recording_macro = false;
        self.macro_commands.clone()
    }

    pub fn is_recording_macro(&self) -> bool {
        self.is_recording_macro
    }

    pub fn get_registered_commands(&self) -> Vec<&str> {
        self.registry.get_command_names()
    }

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
