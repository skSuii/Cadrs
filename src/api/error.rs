// API 层错误类型：为对外接口提供统一的失败表达。
//
// [`CADError`] 覆盖几何、IO、解析、验证、版本与插件等各类失败，[`CADResult`] 是其结果别名。
// 调用者可用 `?` 直接向上传播，也可按变体分支做差异化处理（例如提示重试、回滚事务）。
use thiserror::Error;
use std::fmt;

/// CAD 操作可能返回的错误，按来源分类，每个变体携带人类可读的中文失败原因。
///
/// 由 `thiserror` 派生 `std::error::Error`，可直接配合 `?` 运算符向上传播。
#[derive(Debug, Error)]
pub enum CADError {
    /// 几何运算失败，例如图形退化、无法求交或坐标非法；字符串为具体原因。
    #[error("几何错误: {0}")]
    GeometryError(String),
    
    /// 文件读写失败；`std::io::Error` 会经 `From` 实现自动转换为此变体。
    #[error("文件IO错误: {0}")]
    IOError(String),
    
    /// 输入数据解析失败，例如 DXF 字段格式非法或缺少必需字段。
    #[error("解析错误: {0}")]
    ParseError(String),
    
    /// 数据校验未通过，例如坐标为 NaN、半径为负或必填字段为空。
    #[error("验证错误: {0}")]
    ValidationError(String),
    
    /// 按名称或标识查找对象失败；字符串为未找到目标的描述。
    #[error("未找到: {0}")]
    NotFound(String),
    
    /// 当前状态下不允许的操作，例如未打开文档就保存、删除不可删除的图层。
    #[error("无效操作: {0}")]
    InvalidOperation(String),
    
    /// 类型或单位换算失败，例如颜色、坐标在不同表示之间无法互转。
    #[error("转换错误: {0}")]
    ConversionError(String),
    
    /// 版本不兼容，例如文件格式版本高于当前 SDK，或插件 ABI 与宿主不匹配。
    #[error("版本不兼容: {0}")]
    VersionError(String),
    
    /// 插件相关失败：加载、注册重名、命令不存在或插件命令执行出错。
    #[error("插件错误: {0}")]
    PluginError(String),
}

/// CAD 操作的统一结果别名：`Ok` 携带成功值，`Err` 携带 [`CADError`]。
pub type CADResult<T> = Result<T, CADError>;

impl From<std::io::Error> for CADError {
    fn from(error: std::io::Error) -> Self {
        CADError::IOError(error.to_string())
    }
}

impl From<std::fmt::Error> for CADError {
    fn from(error: std::fmt::Error) -> Self {
        CADError::GeometryError(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = CADError::GeometryError("Invalid point".to_string());
        assert_eq!(format!("{}", error), "几何错误: Invalid point");
    }

    #[test]
    fn test_result_type() {
        let result: CADResult<f64> = Ok(42.0);
        assert!(result.is_ok());
        
        let result: CADResult<f64> = Err(CADError::NotFound("Point".to_string()));
        assert!(result.is_err());
    }
}
