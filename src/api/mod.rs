//! 对外 API 聚合层：统一导出错误类型与插件框架。
//!
//! 本模块不含具体实现，只声明子模块并再导出插件相关的常用类型：
//! - [`error`]：错误枚举 [`CADError`](error::CADError) 与结果别名 `CADResult`；
//! - [`plugin`]：插件 trait、插件上下文、命令/菜单/工具栏描述，以及插件注册表与加载器。
//!
//! 注：同目录下的 `cad_api.rs` 未在此声明，不属于本模块的公开 API。

/// 错误类型 [`CADError`](error::CADError) 与结果别名 `CADResult`。
pub mod error;
/// 插件框架：插件 trait、上下文、界面描述、注册表与加载器。
pub mod plugin;

pub use plugin::{
    Plugin, PluginContext, PluginManager, PluginRegistry, PluginLoader,
    PluginCommand, PluginMenu, PluginToolbar, PluginInfo,
    CommandParameter, ParameterType, MenuItem, MenuPosition,
    ToolbarItem, ToolbarPosition, HookType, HookFunction, LogLevel,
};
