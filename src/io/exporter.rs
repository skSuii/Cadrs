//! 导出器占位实现：提供一个不写出任何格式的空导出器。
//! 用于在注册表尚未接入真实格式实现时占位，或作为测试替身；所有导出调用都会返回「不支持」错误。

use crate::data_structure::Document;
use crate::io::{Exporter, Error};

/// 空导出器：不识别任何扩展名，也不写出任何字节。
/// 与 [`crate::io::FormatRegistry`] 搭配可在没有真实导出器的场景下保持接口可用。
pub struct DefaultExporter;

impl DefaultExporter {
    /// 创建空导出器实例。
    /// 返回的实例不会创建文件，也不能导出任何格式。
    pub fn new() -> Self {
        Self {}
    }
}

impl Exporter for DefaultExporter {
    fn can_export(&self, extension: &str) -> bool {
        false
    }

    fn export_to_file(&self, doc: &Document, filename: &str) -> Result<(), Error> {
        Err(Error::ExportError("Not implemented".to_string()))
    }

    fn export_to_bytes(&self, doc: &Document) -> Result<Vec<u8>, Error> {
        Err(Error::ExportError("Not implemented".to_string()))
    }
}
