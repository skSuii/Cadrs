//! 导入器占位实现：提供一个不解析任何格式的空导入器。
//! 用于在注册表尚未接入真实格式实现时占位，或作为测试替身；所有导入调用都会返回「不支持」错误。

use crate::data_structure::Document;
use crate::io::{Importer, Error};

/// 空导入器：不识别任何扩展名，也不产出任何数据。
/// 与 [`crate::io::FormatRegistry`] 搭配可在没有真实导入器的场景下保持接口可用。
pub struct DefaultImporter;

impl DefaultImporter {
    /// 创建空导入器实例。
    /// 返回的实例不会读取文件，也不能导入任何格式。
    pub fn new() -> Self {
        Self {}
    }
}

impl Importer for DefaultImporter {
    fn can_import(&self, extension: &str) -> bool {
        false
    }

    fn import_from_file(&self, filename: &str) -> Result<Document, Error> {
        Err(Error::UnsupportedFormat("Not implemented".to_string()))
    }

    fn import_from_bytes(&self, data: &[u8], extension: &str) -> Result<Document, Error> {
        Err(Error::UnsupportedFormat("Not implemented".to_string()))
    }
}
