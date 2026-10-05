//! 数据交换模块：把外部图纸文件与内部 Document 相互转换。
//! 涵盖 DXF/SVG/EPS/PDF/WMF/光栅/DWG/IGES/STEP 的导入与导出实现，以及格式注册与统一入口。
//! 核心抽象见 [`io`] 中的 [`Importer`]、[`Exporter`]、[`FormatRegistry`] 与 [`UnifiedDataExchange`]；
//! 各格式实现位于同名子模块，并在此重新导出，调用者通常只需引用本模块一层。

/// DXF (ASCII) 格式的导入与导出实现。
pub mod dxf;
/// SVG 矢量图形格式的导入与导出实现。
pub mod svg;
/// EPS 格式的导出实现。
pub mod eps;
/// PDF 格式的导出实现。
pub mod pdf;
/// WMF 格式的导出实现。
pub mod wmf;
/// 光栅图像格式（PNG/JPEG 等）的导入与导出实现。
pub mod raster;
/// 默认导入器占位实现，见 [`importer`]。
pub mod importer;
/// 默认导出器占位实现，见 [`exporter`]。
pub mod exporter;
/// DWG 二进制格式的导入实现。
pub mod dwg;
/// IGES 格式的导入实现。
pub mod iges;
/// STEP 格式的导入实现。
pub mod step;
/// 带默认实现清单的导入器注册表（[`importer_impl`]）。
pub mod importer_impl;
/// DXF/SVG/JSON 导出器及注册表（[`exporter_impl`]）。
pub mod exporter_impl;
/// 数据交换的核心抽象：错误类型、Importer/Exporter trait、格式注册表与统一入口。
pub mod io;
/// STEP 导出器实现。
pub mod step_exporter;
/// IGES 导出器实现。
pub mod iges_exporter;

// 以下为常用条目的扁平化重导出，便于调用者直接 use crate::io::{...}。
pub use io::{Importer, Exporter, Error, FormatInfo, SUPPORTED_FORMATS};
pub use io::{FormatRegistry, UnifiedDataExchange, ImportOptions, ExportOptions};
pub use io::{LengthUnit, LayerMappingMode, EntityFilter, CoordinateSystem};
pub use step_exporter::{STEPExporter, STEPVersion};
pub use iges_exporter::{IGESExporter, IGESVersion};
pub use raster::RasterFormat;
