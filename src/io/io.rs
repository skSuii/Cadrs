//! 数据交换核心抽象：格式无关的导入/导出接口与注册表。
//! 定义 [`Importer`] / [`Exporter`] 两个 trait（按扩展名声明能力、并可给出优先级），
//! 由 [`FormatRegistry`] 按扩展名登记实现，再由 [`UnifiedDataExchange`] 统一驱动文件与内存数据的转换。
//! 单位、图层映射、坐标系统与公差等行为由 [`ImportOptions`] / [`ExportOptions`] 描述，所有失败统一返回 [`Error`]。

use thiserror::Error;
use crate::data_structure::Document;
use std::sync::LazyLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::any::Any;
use std::fmt;

/// 数据交换过程中的统一错误类型，覆盖读写、解析、导出、注册与格式转换各类失败。
/// 携带的文本仅用于诊断与展示，调用者应按下层返回的变体决定是否回退到其它格式或提示用户。
#[derive(Debug, Error)]
pub enum Error {
    /// 底层文件或流读写失败（如文件不存在、无权限、编码非法）。
    #[error("IO error: {0}")]
    Io(String),
    
    /// 扩展名没有对应的导入器/导出器，或文件名缺少扩展名。
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    
    /// 输入内容不是合法或完整的图纸数据，解析中途失败。
    #[error("Parse error: {0}")]
    ParseError(String),
    
    /// 写出目标格式失败，通常发生在写盘阶段。
    #[error("Export error: {0}")]
    ExportError(String),
    
    /// 向注册表登记导入器/导出器失败，例如同一扩展名已被占用。
    #[error("Registration error: {0}")]
    RegistrationError(String),
    
    /// 外部格式与内部数据模型之间无法无损映射。
    #[error("Conversion error: {0}")]
    ConversionError(String),
}

/// 外部文件到 Document 的导入器接口。
/// 实现需声明自己支持的扩展名，并把文件内容转换为新的 Document；实现本身应可跨线程共享且不持有可变状态。
pub trait Importer: Send + Sync {
    /// 判断本导入器是否处理该扩展名，匹配通常不区分大小写且忽略前导点。
    fn can_import(&self, extension: &str) -> bool;

    /// 读取磁盘文件并解析为新的 Document。
    /// - `filename`：源文件路径，其扩展名决定实际使用的解析分支。
    /// 返回解析得到的文档；读取失败或内容非法时返回 [`Error`]，已存在的文档不受影响。
    fn import_from_file(&self, filename: &str) -> Result<Document, Error>;

    /// 从内存字节解析为新的 Document。
    /// - `data`：完整文件内容，不包含任何额外包装。
    /// - `extension`：格式提示，实现通常先用它校验自身是否适用。
    /// 返回解析得到的文档；格式不符或内容损坏时返回 [`Error`]。
    fn import_from_bytes(&self, data: &[u8], extension: &str) -> Result<Document, Error>;
    
    /// 同一扩展名存在多个实现时的优先级，数值越大越优先。默认为 100。
    fn priority(&self) -> u32 { 100 }
}

/// Document 到外部格式的导出器接口。
/// 实现需声明自己支持的扩展名，并保证导出过程只读入参文档，不修改调用者的 Document。
pub trait Exporter: Send + Sync {
    /// 判断本导出器是否处理该扩展名，匹配通常不区分大小写且忽略前导点。
    fn can_export(&self, extension: &str) -> bool;

    /// 把文档写出到磁盘文件，同名文件会被覆盖（导出具有写盘副作用）。
    /// - `doc`：源文档，仅读取，不会被修改。
    /// - `filename`：目标路径，其扩展名应与本导出器支持的格式一致。
    /// 成功返回 `()`；创建文件或写入失败时返回 [`Error`]。
    fn export_to_file(&self, doc: &Document, filename: &str) -> Result<(), Error>;

    /// 把文档序列化为内存字节，不产生磁盘副作用。
    /// - `doc`：源文档，仅读取，不会被修改。
    /// 返回完整的文件内容字节；无法表示文档内容时返回 [`Error`]。
    fn export_to_bytes(&self, doc: &Document) -> Result<Vec<u8>, Error>;
    
    /// 同一扩展名存在多个实现时的优先级，数值越大越优先。默认为 100。
    fn priority(&self) -> u32 { 100 }
}

/// 单个文件格式的描述信息，用于在界面或错误提示中展示可选格式及其版本。
/// 仅描述元数据，不参与实际的读写行为。
#[derive(Debug, Clone)]
pub struct FormatInfo {
    /// 小写且不带前导点的扩展名，如 `"dxf"`。
    pub extension: String,
    /// 面向用户的格式简称，如 `"DXF"`。
    pub name: String,
    /// 一句话说明，如 `"AutoCAD Drawing Exchange Format"`。
    pub description: String,
    /// 是否为二进制格式；为 `false` 表示内容可按文本处理。
    pub is_binary: bool,
    /// 该格式的 MIME 类型，供外部系统识别。
    pub mime_type: String,
    /// 已知版本号列表，初始为空，由 [`FormatInfo::with_version`] 逐个追加。
    pub versions: Vec<String>,
}

impl FormatInfo {
    /// 按扩展名构造格式信息，并根据扩展名推断 MIME 类型。
    /// - `extension`：不带点的扩展名，大小写不敏感，未知扩展名会落到 `application/octet-stream`。
    /// - `name`、`description`：展示用文本，原样保存。
    /// - `is_binary`：标记内容是否为二进制。
    /// 返回版本列表为空的格式信息，可继续用 [`FormatInfo::with_version`] 追加版本。
    /// # 示例
    /// ```
    /// use cadrs::io::FormatInfo;
    /// let format = FormatInfo::new("dxf", "DXF", "AutoCAD Format", false);
    /// assert_eq!(format.extension, "dxf");
    /// ```
    pub fn new(extension: &str, name: &str, description: &str, is_binary: bool) -> Self {
        let mime = match extension.to_lowercase().as_str() {
            "dxf" => "application/dxf",
            "svg" => "image/svg+xml",
            "json" => "application/json",
            "dwg" => "application/dwg",
            "iges" | "igs" => "applicationiges",
            "/step" | "stp" => "application/step",
            "obj" => "model/obj",
            "fbx" => "application/fbx",
            _ => "application/octet-stream",
        };
        
        Self {
            extension: extension.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            is_binary,
            mime_type: mime.to_string(),
            versions: Vec::new(),
        }
    }
    
    /// 追加一个受支持版本号并返回自身，便于链式描述多个版本。
    /// - `version`：版本标识文本，原样追加到 [`FormatInfo::versions`] 末尾。
    /// 返回追加后的格式信息；重复版本不会被去重。
    pub fn with_version(mut self, version: &str) -> Self {
        self.versions.push(version.to_string());
        self
    }
}

/// 本 SDK 声明支持的格式清单（DXF/SVG/JSON/DWG/IGES/STEP/OBJ/FBX 及其版本）。
/// 由 [`LazyLock`] 在首次访问时构造，内容为常量，仅用于展示与选择，不代表对应导入器或导出器一定已注册。
pub static SUPPORTED_FORMATS: LazyLock<Vec<FormatInfo>> = LazyLock::new(|| {
    vec![
        FormatInfo::new("dxf", "DXF", "AutoCAD Drawing Exchange Format", false)
            .with_version("R12").with_version("R14").with_version("2000").with_version("2018"),
        FormatInfo::new("svg", "SVG", "Scalable Vector Graphics", false)
            .with_version("1.1").with_version("2.0"),
        FormatInfo::new("json", "JSON", "JavaScript Object Notation", false),
        FormatInfo::new("dwg", "DWG", "AutoCAD Drawing Database Binary Format", true)
            .with_version("R12").with_version("R14").with_version("2000").with_version("2018"),
        FormatInfo::new("iges", "IGES", "Initial Graphics Exchange Specification", false)
            .with_version("5.3").with_version("6.0"),
        FormatInfo::new("igs", "IGES", "Initial Graphics Exchange Specification", false)
            .with_version("5.3").with_version("6.0"),
        FormatInfo::new("step", "STEP", "Standard for the Exchange of Product model data", false)
            .with_version("AP203").with_version("AP214").with_version("AP242"),
        FormatInfo::new("stp", "STEP", "Standard for the Exchange of Product model data", false)
            .with_version("AP203").with_version("AP214").with_version("AP242"),
        FormatInfo::new("obj", "OBJ", "Wavefront OBJ Geometry File", false),
        FormatInfo::new("fbx", "FBX", "Autodesk FBX Interchange File", true)
            .with_version("7.0").with_version("7.4").with_version("7.5"),
    ]
});

/// 按扩展名登记导入器与导出器的注册表，是 [`UnifiedDataExchange`] 的查找后端。
/// 扩展名键统一为小写且不带前导点；同一扩展名只能登记一个导入器和一个导出器。
pub struct FormatRegistry {
    importers: HashMap<String, Box<dyn Importer>>,
    exporters: HashMap<String, Box<dyn Exporter>>,
    format_info: HashMap<String, FormatInfo>,
}

impl Default for FormatRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FormatRegistry {
    /// 创建空的注册表，此时可导入与可导出的扩展名列表均为空。
    /// 返回一个尚未登记任何格式、且格式信息表为空的实例。
    pub fn new() -> Self {
        Self {
            importers: HashMap::new(),
            exporters: HashMap::new(),
            format_info: HashMap::new(),
        }
    }
    
    /// 登记一个导入器实例，同时补一条占位的格式信息。
    /// - `importer`：导入器实现，所有权转移给注册表。
    /// 成功时会修改注册表；若该扩展名已存在导入器，则返回 [`Error::RegistrationError`] 且注册表保持不变。
    /// 注意：当前实现以内部占位扩展名 `"unknown"` 作为键，因此第二次登记必然冲突。
    pub fn register_importer<I: Importer + 'static>(&mut self, importer: I) -> Result<(), Error> {
        let ext = self.extract_extension(&importer);
        if self.importers.contains_key(&ext) {
            return Err(Error::RegistrationError(
                format!("Importer for '{}' already registered", ext)
            ));
        }
        
        self.importers.insert(ext.clone(), Box::new(importer));
        self.format_info.insert(ext.clone(), FormatInfo::new(
            &ext, &ext.to_uppercase(), 
            &format!("{} format", ext.to_uppercase()), 
            false
        ));
        
        Ok(())
    }
    
    /// 登记一个导出器实例；若该扩展名尚无格式信息则一并补一条占位记录。
    /// - `exporter`：导出器实现，所有权转移给注册表。
    /// 成功时会修改注册表；同一扩展名重复登记返回 [`Error::RegistrationError`]。
    /// 注意：当前实现以内部占位扩展名 `"unknown"` 作为键，因此第二次登记必然冲突。
    pub fn register_exporter<E: Exporter + 'static>(&mut self, exporter: E) -> Result<(), Error> {
        let ext = self.extract_extension(&exporter);
        if self.exporters.contains_key(&ext) {
            return Err(Error::RegistrationError(
                format!("Exporter for '{}' already registered", ext)
            ));
        }
        
        self.exporters.insert(ext.clone(), Box::new(exporter));
        if !self.format_info.contains_key(&ext) {
            self.format_info.insert(ext.clone(), FormatInfo::new(
                &ext, &ext.to_uppercase(), 
                &format!("{} format", ext.to_uppercase()), 
                false
            ));
        }
        
        Ok(())
    }
    
    fn extract_extension<T: Any>(&self, _obj: &T) -> String {
        "unknown".to_string()
    }
    
    /// 按扩展名查找导入器。
    /// - `extension`：扩展名，匹配前会去掉前导点并转为小写，因此 `".DXF"` 与 `"dxf"` 等价。
    /// 返回已登记导入器的共享引用；未登记时返回 `None`，调用者通常应转为「不支持的格式」提示。
    pub fn get_importer(&self, extension: &str) -> Option<&dyn Importer> {
        let ext = extension.trim_start_matches('.').to_lowercase();
        self.importers.get(&ext).map(|b| b.as_ref())
    }
    
    /// 按扩展名查找导出器。
    /// - `extension`：扩展名，匹配前会去掉前导点并转为小写。
    /// 返回已登记导出器的共享引用；未登记时返回 `None`。
    pub fn get_exporter(&self, extension: &str) -> Option<&dyn Exporter> {
        let ext = extension.trim_start_matches('.').to_lowercase();
        self.exporters.get(&ext).map(|b| b.as_ref())
    }
    
    /// 查询格式的展示信息。
    /// - `extension`：扩展名，匹配前会去掉前导点并转为小写。
    /// 返回格式信息引用；未登记该扩展名时返回 `None`。
    pub fn get_format_info(&self, extension: &str) -> Option<&FormatInfo> {
        let ext = extension.trim_start_matches('.').to_lowercase();
        self.format_info.get(&ext)
    }
    
    /// 列出已登记导入器的扩展名。
    /// 返回按字典序升序排列的小写扩展名列表；无导入器时返回空列表，供界面过滤文件选择范围。
    pub fn supported_import_extensions(&self) -> Vec<String> {
        let mut exts: Vec<_> = self.importers.keys().cloned().collect();
        exts.sort();
        exts
    }
    
    /// 列出已登记导出器的扩展名。
    /// 返回按字典序升序排列的小写扩展名列表；无导出器时返回空列表。
    pub fn supported_export_extensions(&self) -> Vec<String> {
        let mut exts: Vec<_> = self.exporters.keys().cloned().collect();
        exts.sort();
        exts
    }
}

/// 一次导入行为的配置。
/// 说明源文件的长度单位、图层映射方式、需要保留的实体类别与坐标系统，
/// 以及曲面处理公差；不设置时用 [`Default`] 的毫米+保留图层+全部实体。
#[derive(Debug, Clone)]
pub struct ImportOptions {
    /// 源文件的长度单位，导入时需要按它换算为内部单位。
    pub unit: LengthUnit,
    /// 源文件图层到目标文档图层的映射方式。
    pub layer_mapping: LayerMappingMode,
    /// 参与导入的实体类型白名单，未勾选的实体类型会被跳过。
    pub entity_filter: EntityFilter,
    /// 源数据所处的坐标系统（WCS 或 UCS）。
    pub coordinate_system: CoordinateSystem,
    /// 是否把共面且相邻的面合并为一个面，减少面片数量。
    pub merge_coplanar_faces: bool,
    /// 曲面与曲线离散化的公差，单位与文档长度单位一致；值越小越精确、数据量越大。
    pub tessellation_tolerance: f64,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            unit: LengthUnit::Millimeter,
            layer_mapping: LayerMappingMode::Preserve,
            entity_filter: EntityFilter::all(),
            coordinate_system: CoordinateSystem::WCS,
            merge_coplanar_faces: false,
            tessellation_tolerance: 0.01,
        }
    }
}

/// 图纸长度单位，用于在导入导出时标定文件中的坐标含义。
/// 变体本身不携带换算系数，调用者需按业务约定自行换算到内部单位。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LengthUnit {
    /// 毫米。
    Millimeter,
    /// 厘米。
    Centimeter,
    /// 米。
    Meter,
    /// 千米。
    Kilometer,
    /// 英寸。
    Inch,
    /// 英尺。
    Foot,
    /// 码。
    Yard,
    /// 英里。
    Mile,
}

impl Default for LengthUnit {
    fn default() -> Self {
        LengthUnit::Millimeter
    }
}

/// 导入时源文件图层与目标文档图层的对应策略。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayerMappingMode {
    /// 完整保留源图层：按名称逐一创建或复用目标图层。
    Preserve,
    /// 把源文件的全部图层合并到目标文档的单一图层中。
    MergeAll,
    /// 忽略源图层归属，仅导入几何数据。
    Ignore,
}

/// 导入实体类型白名单：字段为 `true` 表示该类实体参与导入，为 `false` 表示被跳过。
/// 用于在导入大图纸时按需裁剪内容；各字段彼此独立，可任意组合。
#[derive(Debug, Clone, PartialEq)]
pub struct EntityFilter {
    /// 是否导入直线。
    pub include_lines: bool,
    /// 是否导入圆。
    pub include_circles: bool,
    /// 是否导入圆弧。
    pub include_arcs: bool,
    /// 是否导入椭圆。
    pub include_ellipses: bool,
    /// 是否导入多段线。
    pub include_polylines: bool,
    /// 是否导入样条曲线。
    pub include_splines: bool,
    /// 是否导入 NURBS 曲线与曲面。
    pub include_nurbs: bool,
    /// 是否导入曲面实体。
    pub include_surfaces: bool,
    /// 是否导入实体（3D 体）。
    pub include_solids: bool,
    /// 是否导入文字。
    pub include_text: bool,
    /// 是否导入标注。
    pub include_dimensions: bool,
}

impl EntityFilter {
    /// 全选过滤器，所有实体类别均参与导入。
    /// 返回一个全部字段为 `true` 的新实例，适合「原样导入」的默认场景。
    pub fn all() -> Self {
        Self {
            include_lines: true,
            include_circles: true,
            include_arcs: true,
            include_ellipses: true,
            include_polylines: true,
            include_splines: true,
            include_nurbs: true,
            include_surfaces: true,
            include_solids: true,
            include_text: true,
            include_dimensions: true,
        }
    }
    
    /// 仅几何过滤器，只保留线、圆、弧、椭圆、多段线、样条与 NURBS 等曲线类实体。
    /// 返回的新实例中曲面、实体、文字与标注均为 `false`，适合只关心图形轮廓的预览场景。
    pub fn geometry_only() -> Self {
        Self {
            include_lines: true,
            include_circles: true,
            include_arcs: true,
            include_ellipses: true,
            include_polylines: true,
            include_splines: true,
            include_nurbs: true,
            include_surfaces: false,
            include_solids: false,
            include_text: false,
            include_dimensions: false,
        }
    }
    
    /// 关闭曲面、实体、文字与标注四类，其余保持不变，支持链式调用。
    /// 消费并返回自身（builder 风格），原过滤器在调用后不可再用。
    pub fn excludes_solids_and_text(mut self) -> Self {
        self.include_surfaces = false;
        self.include_solids = false;
        self.include_text = false;
        self.include_dimensions = false;
        self
    }
}

/// 坐标系统选择，决定按世界坐标系还是用户坐标系解释图纸坐标。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordinateSystem {
    /// 世界坐标系（World Coordinate System）。
    WCS,
    /// 用户坐标系（User Coordinate System），坐标含义依赖当前 UCS 设置。
    UCS,
}

/// 统一数据交换入口：内部持有一个 [`FormatRegistry`] 和一套默认导入选项，对上层屏蔽具体格式。
/// 各方法按扩展名或显式扩展名参数分派到已注册的导入器/导出器，自身不解析任何文件内容。
pub struct UnifiedDataExchange {
    registry: FormatRegistry,
    default_options: ImportOptions,
}

impl UnifiedDataExchange {
    /// 创建数据交换入口，内部使用空的格式注册表和默认导入选项。
    /// 返回的实例尚未注册任何格式，需先调用 [`UnifiedDataExchange::register_importer`] / [`UnifiedDataExchange::register_exporter`] 才能导入导出。
    pub fn new() -> Self {
        let mut registry = FormatRegistry::new();
        
        let mut exchange = Self {
            registry,
            default_options: ImportOptions::default(),
        };
        
        exchange
    }
    
    /// 按文件扩展名导入磁盘文件，返回新的文档。
    /// - `filename`：源文件路径，扩展名决定使用哪个导入器；缺少扩展名时返回 [`Error::UnsupportedFormat`]。
    /// - `options`：本次导入的配置，传 `None` 时使用该实例的默认选项。
    /// 返回解析得到的文档；无对应导入器或解析失败时返回 [`Error`]。
    /// 注意：选项目前仅决定是否使用默认值，尚未下传给导入器。
    pub fn import_file(&self, filename: &str, options: Option<ImportOptions>) -> Result<Document, Error> {
        let options = options.unwrap_or(self.default_options.clone());
        
        let path = PathBuf::from(filename);
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| Error::UnsupportedFormat("No file extension".to_string()))?;
        
        let importer = self.registry.get_importer(ext)
            .ok_or_else(|| Error::UnsupportedFormat(format!("No importer for '{}'", ext)))?;
        
        importer.import_from_file(filename)
    }
    
    /// 从内存字节导入，扩展名由调用者显式给出。
    /// - `data`：完整文件内容字节。
    /// - `extension`：格式提示，用于选择导入器，可带前导点。
    /// - `options`：本次导入的配置，传 `None` 时使用默认选项。
    /// 返回解析得到的文档；无可匹配导入器或数据非法时返回 [`Error`]。
    pub fn import_from_data(&self, data: &[u8], extension: &str, options: Option<ImportOptions>) -> Result<Document, Error> {
        let options = options.unwrap_or(self.default_options.clone());
        
        let importer = self.registry.get_importer(extension)
            .ok_or_else(|| Error::UnsupportedFormat(format!("No importer for '{}'", extension)))?;
        
        importer.import_from_bytes(data, extension)
    }
    
    /// 按文件扩展名把文档导出到磁盘，同名文件会被覆盖（有写盘副作用）。
    /// - `doc`：源文档，仅读取。
    /// - `filename`：目标路径，扩展名决定使用哪个导出器；缺少扩展名时返回 [`Error::UnsupportedFormat`]。
    /// - `options`：导出配置，当前实现尚未下传给导出器。
    /// 成功返回 `()`；无对应导出器或写盘失败时返回 [`Error`]。
    pub fn export_file(&self, doc: &Document, filename: &str, options: Option<ExportOptions>) -> Result<(), Error> {
        let path = PathBuf::from(filename);
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| Error::UnsupportedFormat("No file extension".to_string()))?;
        
        let exporter = self.registry.get_exporter(ext)
            .ok_or_else(|| Error::UnsupportedFormat(format!("No exporter for '{}'", ext)))?;
        
        exporter.export_to_file(doc, filename)
    }
    
    /// 把文档导出为内存字节，不产生磁盘副作用。
    /// - `doc`：源文档，仅读取。
    /// - `extension`：目标格式提示，用于选择导出器，可带前导点。
    /// - `options`：导出配置，当前实现尚未下传给导出器。
    /// 返回完整的文件内容字节；无对应导出器时返回 [`Error`]。
    pub fn export_to_data(&self, doc: &Document, extension: &str, options: Option<ExportOptions>) -> Result<Vec<u8>, Error> {
        let exporter = self.registry.get_exporter(extension)
            .ok_or_else(|| Error::UnsupportedFormat(format!("No exporter for '{}'", extension)))?;
        
        exporter.export_to_bytes(doc)
    }
    
    /// 把导入器登记到底层注册表，使其扩展名可被本入口分派使用。
    /// - `importer`：导入器实现，所有权转移。
    /// 成功时修改自身；扩展名冲突时返回 [`Error::RegistrationError`]。
    pub fn register_importer<I: Importer + 'static>(&mut self, importer: I) -> Result<(), Error> {
        self.registry.register_importer(importer)
    }
    
    /// 把导出器登记到底层注册表，使其扩展名可被本入口分派使用。
    /// - `exporter`：导出器实现，所有权转移。
    /// 成功时修改自身；扩展名冲突时返回 [`Error::RegistrationError`]。
    pub fn register_exporter<E: Exporter + 'static>(&mut self, exporter: E) -> Result<(), Error> {
        self.registry.register_exporter(exporter)
    }
    
    /// 列出当前可导入的扩展名（升序），可直接用于构建文件选择过滤条件。
    pub fn supported_import_formats(&self) -> Vec<String> {
        self.registry.supported_import_extensions()
    }
    
    /// 列出当前可导出的扩展名（升序）。
    pub fn supported_export_formats(&self) -> Vec<String> {
        self.registry.supported_export_extensions()
    }
    
    /// 猜测数据所属的格式，内容特征优先于文件名。
    /// - `data`：文件起始字节，用于识别二进制签名（如 DWG 的 `AC10xx` 头）。
    /// - `filename`：可选的文件名，仅在内容未命中时按扩展名兜底。
    /// 返回小写的格式扩展名；两者都无法判定时返回 `None`。
    pub fn detect_format(&self, data: &[u8], filename: Option<&str>) -> Option<String> {
        if data.len() >= 4 {
            let header = &data[..4];
            if header == b"AC10" || header == b"AC11" || header == b"AC12" || header == b"AC13" || header == b"AC14" || header == b"AC15" || header == b"AC16" || header == b"AC17" || header == b"AC18" {
                return Some("dwg".to_string());
            }
        }
        
        if let Some(name) = filename {
            let path = PathBuf::from(name);
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                return Some(ext.to_lowercase());
            }
        }
        
        None
    }
}

/// 一次导出行为的配置。
/// 指定目标格式版本、长度单位、坐标系统，以及是否导出不可见实体、是否写图层名、
/// 离散化公差与输出是否为二进制；不设置时用 [`Default`]（毫米 + WCS + 保留图层名 + 文本格式）。
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// 目标格式的版本标识（如 `"R12"`、`"2018"`）；`None` 表示由导出器自行决定默认版本。
    pub version: Option<String>,
    /// 导出使用的长度单位，决定写出的坐标数值量纲。
    pub unit: LengthUnit,
    /// 写出坐标所处的坐标系统。
    pub coordinate_system: CoordinateSystem,
    /// 是否导出被隐藏（不可见）的实体；默认 `false`，即隐藏实体被丢弃。
    pub export_hidden_entities: bool,
    /// 是否写出图层名称信息；关闭后实体将归入目标格式的默认图层。
    pub export_layernames: bool,
    /// 曲面与曲线离散化的公差，单位与文档长度单位一致。
    pub tessellation_tolerance: f64,
    /// 是否输出二进制格式；为 `false` 时输出文本格式。
    pub binary_format: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            version: None,
            unit: LengthUnit::Millimeter,
            coordinate_system: CoordinateSystem::WCS,
            export_hidden_entities: false,
            export_layernames: true,
            tessellation_tolerance: 0.01,
            binary_format: false,
        }
    }
}

impl fmt::Display for LengthUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LengthUnit::Millimeter => write!(f, "mm"),
            LengthUnit::Centimeter => write!(f, "cm"),
            LengthUnit::Meter => write!(f, "m"),
            LengthUnit::Kilometer => write!(f, "km"),
            LengthUnit::Inch => write!(f, "in"),
            LengthUnit::Foot => write!(f, "ft"),
            LengthUnit::Yard => write!(f, "yd"),
            LengthUnit::Mile => write!(f, "mi"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_format() {
        let format = FormatInfo::new("dxf", "DXF", "AutoCAD Format", false);
        assert_eq!(format.extension, "dxf");
        assert!(!format.is_binary);
    }

    #[test]
    fn test_supported_formats() {
        assert!(SUPPORTED_FORMATS.len() > 0);
    }

    #[test]
    fn test_format_registry() {
        let mut registry = FormatRegistry::new();
        assert!(registry.supported_import_extensions().is_empty());
        assert!(registry.supported_export_extensions().is_empty());
    }

    #[test]
    fn test_entity_filter() {
        let filter = EntityFilter::all();
        assert!(filter.include_lines);
        assert!(filter.include_solids);
        
        let geo_filter = EntityFilter::geometry_only();
        assert!(geo_filter.include_lines);
        assert!(!geo_filter.include_solids);
    }
}
