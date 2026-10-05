//! 导入器清单与分派：聚合本 SDK 内置的各格式导入器，并按扩展名选择实现。
//! [`ImporterRegistry::new`] 会预置 DXF/SVG/DWG/IGES/STEP 五个导入器，通过 [`ImporterRegistry::get_importer`]
//! 按扩展名线性查找；本模块的错误以中文 `String` 返回，与 [`crate::io::io::Error`] 分开，便于上层直接展示。

use super::{Importer, FormatInfo};
use super::io::SUPPORTED_FORMATS;
use super::dxf::DXFImporter;
use super::svg::SVGImporter;
use super::dwg::DWGImporter;
use super::iges::IGESImporter;
use super::step::STEPImporter;
use crate::data_structure::Document;
use std::path::Path;

/// 内置导入器清单，持有按注册顺序排列的导入器实例。
/// 查找按插入顺序进行，因此同名扩展名由先插入者胜出；实例由本注册表独占持有。
pub struct ImporterRegistry {
    importers: Vec<Box<dyn Importer>>,
}

impl ImporterRegistry {
    /// 创建注册表并一次性登记全部内置导入器（DXF、SVG、DWG、IGES、STEP）。
    /// 返回可直接使用的实例，无需再手工注册。
    /// # 示例
    /// ```
    /// use cadrs::io::importer_impl::ImporterRegistry;
    /// let registry = ImporterRegistry::new();
    /// assert!(registry.get_importer("dxf").is_some());
    /// ```
    pub fn new() -> Self {
        let mut registry = Self {
            importers: Vec::new(),
        };
        registry.register_all();
        registry
    }

    fn register_all(&mut self) {
        self.importers.push(Box::new(DXFImporter::new()));
        self.importers.push(Box::new(SVGImporter::new()));
        self.importers.push(Box::new(DWGImporter::new()));
        self.importers.push(Box::new(IGESImporter::new()));
        self.importers.push(Box::new(STEPImporter::new()));
    }

    /// 按扩展名查找第一个声明支持该扩展名的导入器。
    /// - `extension`：扩展名，大小写不敏感，由各导入器的 `can_import` 自行判断。
    /// 返回导入器的共享引用；全部不匹配时返回 `None`。
    pub fn get_importer(&self, extension: &str) -> Option<&dyn Importer> {
        for importer in &self.importers {
            if importer.can_import(extension) {
                return Some(importer.as_ref());
            }
        }
        None
    }

    /// 按文件扩展名导入磁盘文件，扩展名取最后一个点之后的部分并转为小写。
    /// - `filename`：源文件路径。
    /// 返回新的文档；文件名无扩展名或格式不被支持时返回中文错误描述，读取失败则透传导入器的错误文本。
    pub fn import_from_file(&self, filename: &str) -> Result<Document, String> {
        let path = Path::new(filename);
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
            .ok_or("无法获取文件扩展名")?;

        if let Some(importer) = self.get_importer(&extension) {
            importer.import_from_file(filename)
                .map_err(|e| e.to_string())
        } else {
            Err(format!("不支持的文件格式: {}", extension))
        }
    }

    /// 从内存字节解析文档，扩展名由调用者给出。
    /// - `data`：完整文件内容字节。
    /// - `extension`：格式提示，会转为小写后用于选择导入器。
    /// 返回新的文档；格式不被支持或内容非法时返回中文错误描述。
    pub fn import_from_bytes(&self, data: &[u8], extension: &str) -> Result<Document, String> {
        let ext = extension.to_lowercase();
        if let Some(importer) = self.get_importer(&ext) {
            importer.import_from_bytes(data, &ext)
                .map_err(|e| e.to_string())
        } else {
            Err(format!("不支持的文件格式: {}", ext))
        }
    }

    /// 列出本 SDK 声明的全部格式信息（即 [`SUPPORTED_FORMATS`] 的内容）。
    /// 返回静态数据的引用切片，与注册表实际登记情况无关，不反映哪些格式真正可导入。
    pub fn supported_formats(&self) -> Vec<&FormatInfo> {
        SUPPORTED_FORMATS.iter().collect()
    }
}

impl Default for ImporterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 按扩展名直接新建一个导入器实例，无需先构造注册表。
/// - `extension`：扩展名，大小写不敏感；`iges`/`igs` 与 `step`/`stp` 各自复用同一实现。
/// 返回拥有所有权的导入器；未知扩展名返回 `None`。
/// # 示例
/// ```
/// use cadrs::io::importer_impl::get_importer;
/// assert!(get_importer("dxf").is_some());
/// assert!(get_importer("unknown").is_none());
/// ```
pub fn get_importer(extension: &str) -> Option<Box<dyn Importer>> {
    match extension.to_lowercase().as_str() {
        "dxf" => Some(Box::new(DXFImporter::new())),
        "svg" => Some(Box::new(SVGImporter::new())),
        "dwg" => Some(Box::new(DWGImporter::new())),
        "iges" | "igs" => Some(Box::new(IGESImporter::new())),
        "step" | "stp" => Some(Box::new(STEPImporter::new())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_importer_registry_formats() {
        let registry = ImporterRegistry::new();
        let formats = registry.supported_formats();
        assert!(formats.len() > 0);
    }

    #[test]
    fn test_get_importer() {
        assert!(get_importer("dxf").is_some());
        assert!(get_importer("svg").is_some());
        assert!(get_importer("dwg").is_some());
        assert!(get_importer("iges").is_some());
        assert!(get_importer("step").is_some());
        assert!(get_importer("unknown").is_none());
    }
}
