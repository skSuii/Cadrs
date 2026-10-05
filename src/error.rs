//! # CAD SDK 错误处理模块
//!
//! 提供整个 SDK 统一的错误类型 [`CadError`] 与结果别名 [`CadResult`]，
//! 以及参数校验（[`validation`]）与安全数值运算（[`numeric`]）辅助函数。
//!
//! - 各子模块用 `CadError` 的不同变体区分失败来源（几何、渲染、IO、解析、验证、
//!   变换、索引、文档、图层、块、实体、捕捉、命令、格式、序列化），并可通过
//!   `user_message()` 转成面向最终用户的中文提示。
//! - 命令系统另用 `command::CommandResult` 表示执行结果；其失败分支通常记录由
//!   `CadError::Command` 转写而来的信息。
//! - 多个 `From` 实现（`std::io::Error`、`ParseFloatError`、`ParseIntError`、
//!   `serde_json::Error`，启用 `io` 特性时还有 `quick_xml::Error`）使这些调用点
//!   可直接使用 `?` 传播错误。

use thiserror::Error;
use std::num::ParseFloatError;
use std::num::ParseIntError;

/// CAD SDK 的主要错误类型
#[derive(Error, Debug)]
pub enum CadError {
    /// 几何计算错误
    #[error("几何错误: {message}")]
    Geometry {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 渲染错误
    #[error("渲染错误: {message}")]
    Render {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// IO 错误
    #[error("IO 错误: {source}")]
    IO {
        #[from]
        source: std::io::Error,
    },

    /// 解析错误
    #[error("解析错误: {message}")]
    Parse {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 验证错误
    #[error("验证错误: {message}")]
    Validation {
        message: String,
        field: Option<String>,
    },

    /// 变换错误
    #[error("变换错误: {message}")]
    Transform {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 索引错误
    #[error("索引错误: {message}")]
    Index {
        message: String,
        index: Option<usize>,
        max_index: Option<usize>,
    },

    /// 文档错误
    #[error("文档错误: {message}")]
    Document {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 层错误
    #[error("层错误: {message}")]
    Layer {
        message: String,
        layer_name: Option<String>,
    },

    /// 块错误
    #[error("块错误: {message}")]
    Block {
        message: String,
        block_name: Option<String>,
    },

    /// 实体错误
    #[error("实体错误: {entity_type} - {message}")]
    Entity {
        entity_type: String,
        entity_id: Option<String>,
        message: String,
    },

    /// 捕捉错误
    #[error("捕捉错误: {message}")]
    Snap {
        message: String,
        snap_type: Option<String>,
    },

    /// 命令错误
    #[error("命令错误: {command} - {message}")]
    Command {
        command: String,
        message: String,
    },

    /// 文件格式错误
    #[error("文件格式错误: {format} - {message}")]
    Format {
        format: String,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 序列化错误
    #[error("序列化错误: {message}")]
    Serialization {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 未知错误
    #[error("未知错误: {message}")]
    Unknown {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl CadError {
    /// 创建一个几何错误
    pub fn geometry(message: impl Into<String>) -> Self {
        CadError::Geometry {
            message: message.into(),
            source: None,
        }
    }

    /// 创建一个几何错误（带源错误）
    pub fn geometry_with_source(
        message: impl Into<String>,
        source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        CadError::Geometry {
            message: message.into(),
            source: Some(source.into()),
        }
    }

    /// 创建一个渲染错误
    pub fn render(message: impl Into<String>) -> Self {
        CadError::Render {
            message: message.into(),
            source: None,
        }
    }

    /// 创建一个验证错误
    pub fn validation(message: impl Into<String>) -> Self {
        CadError::Validation {
            message: message.into(),
            field: None,
        }
    }

    /// 创建一个验证错误（带字段名）
    pub fn validation_with_field(message: impl Into<String>, field: impl Into<String>) -> Self {
        CadError::Validation {
            message: message.into(),
            field: Some(field.into()),
        }
    }

    /// 创建一个变换错误
    pub fn transform(message: impl Into<String>) -> Self {
        CadError::Transform {
            message: message.into(),
            source: None,
        }
    }

    /// 创建一个索引错误
    pub fn index(message: impl Into<String>, index: usize, max_index: usize) -> Self {
        CadError::Index {
            message: message.into(),
            index: Some(index),
            max_index: Some(max_index),
        }
    }

    /// 创建一个命令错误
    pub fn command(command: impl Into<String>, message: impl Into<String>) -> Self {
        CadError::Command {
            command: command.into(),
            message: message.into(),
        }
    }

    /// 创建一个文档错误
    pub fn document(message: impl Into<String>) -> Self {
        CadError::Document {
            message: message.into(),
            source: None,
        }
    }

    /// 创建一个层错误
    pub fn layer(message: impl Into<String>, layer_name: impl Into<String>) -> Self {
        CadError::Layer {
            message: message.into(),
            layer_name: Some(layer_name.into()),
        }
    }

    /// 创建一个块错误
    pub fn block(message: impl Into<String>, block_name: impl Into<String>) -> Self {
        CadError::Block {
            message: message.into(),
            block_name: Some(block_name.into()),
        }
    }

    /// 创建一个实体错误
    pub fn entity(
        entity_type: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        CadError::Entity {
            entity_type: entity_type.into(),
            entity_id: None,
            message: message.into(),
        }
    }

    /// 创建一个捕捉错误
    pub fn snap(message: impl Into<String>, snap_type: impl Into<String>) -> Self {
        CadError::Snap {
            message: message.into(),
            snap_type: Some(snap_type.into()),
        }
    }

    /// 创建一个文件格式错误
    pub fn format(format: impl Into<String>, message: impl Into<String>) -> Self {
        CadError::Format {
            format: format.into(),
            message: message.into(),
            source: None,
        }
    }

    /// 创建一个序列化错误
    pub fn serialization(message: impl Into<String>) -> Self {
        CadError::Serialization {
            message: message.into(),
            source: None,
        }
    }

    /// 检查是否是临时错误（可以重试）
    pub fn is_transient(&self) -> bool {
        matches!(self, CadError::IO { .. } | CadError::Parse { .. })
    }

    /// 检查是否是致命错误（需要用户干预）
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            CadError::Validation { .. } | CadError::Format { .. }
        )
    }

    /// 获取错误的用户友好消息
    pub fn user_message(&self) -> String {
        match self {
            CadError::Geometry { message, .. } => format!("几何计算问题: {}", message),
            CadError::Render { message, .. } => format!("渲染问题: {}", message),
            CadError::IO { source, .. } => format!("文件操作问题: {}", source),
            CadError::Parse { message, .. } => format!("数据解析问题: {}", message),
            CadError::Validation { message, field, .. } => {
                if let Some(field) = field {
                    format!("输入验证问题 ({}): {}", field, message)
                } else {
                    format!("输入验证问题: {}", message)
                }
            }
            CadError::Transform { message, .. } => format!("坐标变换问题: {}", message),
            CadError::Index { message, index, max_index, .. } => {
                if let (Some(idx), Some(max)) = (index, max_index) {
                    format!("{} (索引 {} 超出范围 0-{})", message, idx, max)
                } else {
                    format!("索引问题: {}", message)
                }
            }
            CadError::Document { message, .. } => format!("文档操作问题: {}", message),
            CadError::Layer { message, layer_name, .. } => {
                if let Some(name) = layer_name {
                    format!("图层 '{}' 问题: {}", name, message)
                } else {
                    format!("图层问题: {}", message)
                }
            }
            CadError::Block { message, block_name, .. } => {
                if let Some(name) = block_name {
                    format!("图块 '{}' 问题: {}", name, message)
                } else {
                    format!("图块问题: {}", message)
                }
            }
            CadError::Entity {
                entity_type, message, ..
            } => format!("{} 对象问题: {}", entity_type, message),
            CadError::Snap { message, .. } => format!("捕捉点问题: {}", message),
            CadError::Command { command, message, .. } => {
                format!("命令 '{}' 执行问题: {}", command, message)
            }
            CadError::Format { format, message, .. } => {
                format!("{} 文件格式问题: {}", format, message)
            }
            CadError::Serialization { message, .. } => format!("数据保存问题: {}", message),
            CadError::Unknown { message, .. } => format!("意外问题: {}", message),
        }
    }
}



impl From<ParseFloatError> for CadError {
    fn from(error: ParseFloatError) -> Self {
        CadError::Parse {
            message: error.to_string(),
            source: Some(Box::new(error)),
        }
    }
}

impl From<ParseIntError> for CadError {
    fn from(error: ParseIntError) -> Self {
        CadError::Parse {
            message: error.to_string(),
            source: Some(Box::new(error)),
        }
    }
}

impl From<serde_json::Error> for CadError {
    fn from(error: serde_json::Error) -> Self {
        CadError::Serialization {
            message: error.to_string(),
            source: Some(Box::new(error)),
        }
    }
}

#[cfg(feature = "io")]
impl From<quick_xml::Error> for CadError {
    fn from(error: quick_xml::Error) -> Self {
        CadError::Parse {
            message: error.to_string(),
            source: Some(Box::new(error)),
        }
    }
}

/// CAD SDK 的结果类型
pub type CadResult<T> = Result<T, CadError>;

/// 验证 trait，用于输入验证
pub trait Validate {
    /// 验证自身，返回验证结果
    fn validate(&self) -> CadResult<()>;
}

/// 数值验证辅助函数
pub mod validation {
    use super::*;

    /// 验证值是否为正数
    #[inline]
    pub fn positive(value: f64, name: &str) -> CadResult<f64> {
        if value > 0.0 {
            Ok(value)
        } else {
            Err(CadError::validation_with_field(
                format!("{} 必须大于 0，实际值: {}", name, value),
                name,
            ))
        }
    }

    /// 验证值是否非负
    #[inline]
    pub fn non_negative(value: f64, name: &str) -> CadResult<f64> {
        if value >= 0.0 {
            Ok(value)
        } else {
            Err(CadError::validation_with_field(
                format!("{} 不能为负数，实际值: {}", name, value),
                name,
            ))
        }
    }

    /// 验证值是否在范围内
    #[inline]
    pub fn in_range(value: f64, min: f64, max: f64, name: &str) -> CadResult<f64> {
        if value >= min && value <= max {
            Ok(value)
        } else {
            Err(CadError::validation_with_field(
                format!("{} 必须在 [{}, {}] 范围内，实际值: {}", name, min, max, value),
                name,
            ))
        }
    }

    /// 验证索引是否在范围内
    #[inline]
    pub fn index<T>(index: usize, max: usize, name: &str) -> CadResult<usize> {
        if index < max {
            Ok(index)
        } else {
            Err(CadError::index(
                format!("{} 索引超出范围", name),
                index,
                max.saturating_sub(1),
            ))
        }
    }

    /// 验证字符串是否非空
    #[inline]
    pub fn non_empty<'a>(value: &'a str, name: &str) -> CadResult<&'a str> {
        if !value.is_empty() {
            Ok(value)
        } else {
            Err(CadError::validation_with_field(
                format!("{} 不能为空字符串", name),
                name,
            ))
        }
    }

    /// 验证缩放因子
    #[inline]
    pub fn scale_factor(value: f64, name: &str) -> CadResult<f64> {
        if value.is_finite() && value > 0.0 {
            Ok(value)
        } else {
            Err(CadError::validation_with_field(
                format!("{} 必须是有效正数，实际值: {}", name, value),
                name,
            ))
        }
    }

    /// 验证角度（弧度）
    #[inline]
    pub fn angle_radians(value: f64, name: &str) -> CadResult<f64> {
        if value.is_finite() {
            Ok(value)
        } else {
            Err(CadError::validation_with_field(
                format!("{} 必须是有效角度值，实际值: {}", name, value),
                name,
            ))
        }
    }

    /// 验证角度（度）
    #[inline]
    pub fn angle_degrees(value: f64, name: &str) -> CadResult<f64> {
        if value.is_finite() {
            Ok(value)
        } else {
            Err(CadError::validation_with_field(
                format!("{} 必须是有效角度值，实际值: {}", name, value),
                name,
            ))
        }
    }

    /// 验证坐标值
    #[inline]
    pub fn coordinate(value: f64, name: &str) -> CadResult<f64> {
        if value.is_finite() {
            Ok(value)
        } else {
            Err(CadError::validation_with_field(
                format!("{} 必须是有效坐标值，实际值: {}", name, value),
                name,
            ))
        }
    }
}

/// 数值运算辅助模块
pub mod numeric {
    use super::*;

    /// 安全乘法（带溢出检查）
    #[inline]
    pub fn safe_mul(a: f64, b: f64) -> CadResult<f64> {
        let result = a * b;
        if result.is_finite() {
            Ok(result)
        } else {
            Err(CadError::geometry("乘法运算导致数值溢出"))
        }
    }

    /// 安全加法（带溢出检查）
    #[inline]
    pub fn safe_add(a: f64, b: f64) -> CadResult<f64> {
        let result = a + b;
        if result.is_finite() {
            Ok(result)
        } else {
            Err(CadError::geometry("加法运算导致数值溢出"))
        }
    }

    /// 安全除法（带除零检查）
    #[inline]
    pub fn safe_div(a: f64, b: f64, context: &str) -> CadResult<f64> {
        if b.abs() > 1e-15 {
            let result = a / b;
            if result.is_finite() {
                Ok(result)
            } else {
                Err(CadError::geometry(format!("除法运算导致数值溢出: {}", context)))
            }
        } else {
            Err(CadError::geometry(format!("除以零: {}", context)))
        }
    }

    /// 安全平方根
    #[inline]
    pub fn safe_sqrt(value: f64, context: &str) -> CadResult<f64> {
        if value >= 0.0 {
            let result = value.sqrt();
            if result.is_finite() {
                Ok(result)
            } else {
                Err(CadError::geometry(format!("平方根运算导致数值溢出: {}", context)))
            }
        } else {
            Err(CadError::geometry(format!("负数平方根: {}", context)))
        }
    }

    /// 安全角度转换（度到弧度）
    #[inline]
    pub fn degrees_to_radians(degrees: f64) -> CadResult<f64> {
        if degrees.is_finite() {
            Ok(degrees * std::f64::consts::PI / 180.0)
        } else {
            Err(CadError::geometry("角度值无效"))
        }
    }

    /// 安全角度转换（弧度到度）
    #[inline]
    pub fn radians_to_degrees(radians: f64) -> CadResult<f64> {
        if radians.is_finite() {
            Ok(radians * 180.0 / std::f64::consts::PI)
        } else {
            Err(CadError::geometry("弧度值无效"))
        }
    }

    /// 浮点数近似相等比较
    #[inline]
    pub fn approx_eq(a: f64, b: f64, epsilon: f64) -> bool {
        (a - b).abs() <= epsilon
    }

    /// 浮点数近似为零检查
    #[inline]
    pub fn approx_zero(value: f64, epsilon: f64) -> bool {
        value.abs() <= epsilon
    }

    /// 线性插值
    #[inline]
    pub fn lerp(a: f64, b: f64, t: f64) -> CadResult<f64> {
        if t.is_finite() && t >= 0.0 && t <= 1.0 {
            Ok(a + (b - a) * t)
        } else {
            Err(CadError::geometry("插值参数 t 必须在 [0, 1] 范围内"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_positive() {
        assert!(validation::positive(1.0, "test").is_ok());
        assert!(validation::positive(0.0, "test").is_err());
        assert!(validation::positive(-1.0, "test").is_err());
    }

    #[test]
    fn test_validation_in_range() {
        assert!(validation::in_range(5.0, 0.0, 10.0, "test").is_ok());
        assert!(validation::in_range(0.0, 0.0, 10.0, "test").is_ok());
        assert!(validation::in_range(10.0, 0.0, 10.0, "test").is_ok());
        assert!(validation::in_range(-1.0, 0.0, 10.0, "test").is_err());
        assert!(validation::in_range(11.0, 0.0, 10.0, "test").is_err());
    }

    #[test]
    fn test_numeric_safe_div() {
        assert!(numeric::safe_div(10.0, 2.0, "test").is_ok());
        assert!(numeric::safe_div(10.0, 0.0, "test").is_err());
    }

    #[test]
    fn test_numeric_safe_sqrt() {
        assert!(numeric::safe_sqrt(16.0, "test").is_ok());
        assert!(numeric::safe_sqrt(-1.0, "test").is_err());
    }

    #[test]
    fn test_approx_eq() {
        assert!(numeric::approx_eq(1.0, 1.000001, 0.0001));
        assert!(!numeric::approx_eq(1.0, 1.001, 0.0001));
    }

    #[test]
    fn test_user_message() {
        let error = CadError::validation_with_field("test message", "test_field");
        assert!(error.user_message().contains("test_field"));
    }

    #[test]
    fn test_error_from_io() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let cad_error: CadError = io_error.into();
        assert!(matches!(cad_error, CadError::IO { .. }));
    }
}
