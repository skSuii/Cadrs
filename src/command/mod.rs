/// 命令核心定义：`Command` trait、`CommandResult` 与 `CommandContext` 等执行期数据结构。
pub mod command;
/// 命令历史：撤销/重做栈、事务与快照。
pub mod command_history;
/// 命令注册与管理：命令的注册、别名、执行、宏录制。
pub mod command_manager;

/// `command` 子模块核心类型的再导出：命令 trait、执行结果、上下文与构建器。
pub use command::{Command, CommandClone, CommandResult, CommandContext, CommandBuilder, CommandType, GenericCommand};
/// `command_history` 子模块类型的再导出：历史条目、条目类型、历史栈与快照。
pub use command_history::{HistoryAction, HistoryActionType, CommandHistory, HistorySnapshot};
/// `command_manager` 子模块类型的再导出：命令注册表与命令管理器。
pub use command_manager::{CommandRegistry, CommandManager};
