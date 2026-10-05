//! 内置导出器实现与清单：DXF、SVG、JSON 三种格式的写出逻辑及统一分派。
//! [`DXFExporter`] 直接拼接 ASCII DXF 文本（含 HEADER/TABLES/BLOCKS/ENTITIES 段）；
//! [`SVGExporter`] 以固定画布尺寸输出矢量图元；[`JSONExporter`] 输出便于读取的实体摘要。
//! [`ExporterRegistry`] 按扩展名选择实现，本模块错误同样以中文 `String` 返回给上层。

use super::{Exporter, FormatInfo, Error};
use super::io::SUPPORTED_FORMATS;
use crate::data_structure::{Document, Entity, EntityType, EntityGeometry, ObjectId, Layer, Block};
use crate::geometry::{Point, Line, Circle, Arc, Polyline};
use std::path::Path;
use std::io::{self, Write};
use quick_xml::events::{Event, BytesText};
use quick_xml::writer::Writer;
use std::fs::File;

/// ASCII DXF 导出器，按 [`DXFVersion`] 决定文件头中的 `$ACADVER` 版本号。
/// 依次写出 HEADER/CLASSES/BLOCKS/ENTITIES/OBJECTS 段；当前仅实际输出直线、圆、圆弧与多段线，
/// 其余实体类型会被跳过，块只写出块名占位而不含块内实体。
#[derive(Debug)]
pub struct DXFExporter {
    version: DXFVersion,
}

/// DXF 目标版本，对应文件头 `$ACADVER` 的取值。
#[derive(Debug, Clone, PartialEq)]
pub enum DXFVersion {
    /// R12（`AC1009`），兼容性最好，但仅支持较老的实体类型。
    R12,
    /// AutoCAD 2000（`AC1015`）。
    R2000,
    /// AutoCAD 2018（`AC1032`），[`DXFExporter::new`] 的默认取值。
    R2018,
}

impl DXFExporter {
    /// 创建 DXF 导出器，默认使用 [`DXFVersion::R2018`]。
    /// 返回的实例可立即用于导出，版本无法在创建后修改。
    pub fn new() -> Self {
        Self {
            version: DXFVersion::R2018,
        }
    }

    fn format_version(&self) -> &str {
        match self.version {
            DXFVersion::R12 => "AC1009",
            DXFVersion::R2000 => "AC1015",
            DXFVersion::R2018 => "AC1032",
        }
    }
}

impl Exporter for DXFExporter {
    fn can_export(&self, extension: &str) -> bool {
        extension.to_lowercase() == "dxf"
    }

    fn export_to_file(&self, doc: &Document, filename: &str) -> Result<(), Error> {
        let mut output = String::new();

        self.write_header(doc, &mut output);
        self.write_classes(&mut output);
        self.write_blocks(doc, &mut output);
        self.write_entities(doc, &mut output);
        self.write_objects(&mut output);
        self.write_footer(&mut output);

        std::fs::write(filename, output)
            .map_err(|e| Error::ExportError(format!("Failed to write file: {}", e)))
    }

    fn export_to_bytes(&self, doc: &Document) -> Result<Vec<u8>, Error> {
        let mut output = String::new();

        self.write_header(doc, &mut output);
        self.write_classes(&mut output);
        self.write_blocks(doc, &mut output);
        self.write_entities(doc, &mut output);
        self.write_objects(&mut output);
        self.write_footer(&mut output);

        Ok(output.into_bytes())
    }
}

impl DXFExporter {
    fn write_header(&self, doc: &Document, output: &mut String) {
        output.push_str("SECTION\n");
        output.push_str("  2\n");
        output.push_str("HEADER\n");

        output.push_str("  9\n");
        output.push_str("$ACADVER\n");
        output.push_str("  1\n");
        output.push_str(&format!("{}\n", self.format_version()));

        output.push_str("  9\n");
        output.push_str("$INSBASE\n");
        output.push_str("  10\n");
        output.push_str("0.0\n");
        output.push_str("  20\n");
        output.push_str("0.0\n");
        output.push_str("  30\n");
        output.push_str("0.0\n");

        output.push_str("  9\n");
        output.push_str("$EXTMIN\n");
        output.push_str("  10\n");
        output.push_str("0.0\n");
        output.push_str("  20\n");
        output.push_str("0.0\n");

        output.push_str("  9\n");
        output.push_str("$EXTMAX\n");
        output.push_str("  10\n");
        output.push_str("1000.0\n");
        output.push_str("  20\n");
        output.push_str("1000.0\n");

        output.push_str("ENDSEC\n");
    }

    fn write_classes(&self, output: &mut String) {
        output.push_str("SECTION\n");
        output.push_str("  2\n");
        output.push_str("CLASSES\n");
        output.push_str("ENDSEC\n");
    }

    fn write_blocks(&self, doc: &Document, output: &mut String) {
        output.push_str("SECTION\n");
        output.push_str("  2\n");
        output.push_str("BLOCKS\n");

        for (_, block) in doc.blocks() {
            output.push_str("  0\n");
            output.push_str("BLOCK\n");
            output.push_str("  8\n");
            output.push_str("0\n");
            output.push_str("  2\n");
            output.push_str(&format!("{}\n", block.name()));
            output.push_str(" 70\n");
            output.push_str("1\n");

            output.push_str("  0\n");
            output.push_str("ENDBLK\n");
            output.push_str("  8\n");
            output.push_str("0\n");
        }

        output.push_str("ENDSEC\n");
    }

    fn write_entities(&self, doc: &Document, output: &mut String) {
        output.push_str("SECTION\n");
        output.push_str("  2\n");
        output.push_str("ENTITIES\n");

        for (_, entity) in doc.entities() {
            self.write_entity(entity, output);
        }

        output.push_str("ENDSEC\n");
    }

    fn write_entity(&self, entity: &Entity, output: &mut String) {
        match &entity.geometry {
            EntityGeometry::Line(line) => {
                self.write_line_entity(&line.start, &line.end, &entity.layer_id.to_string(), output);
            }
            EntityGeometry::Circle(circle) => {
                self.write_circle_entity(&circle.center, circle.radius, &entity.layer_id.to_string(), output);
            }
            EntityGeometry::Arc(arc) => {
                self.write_arc_entity(arc, &entity.layer_id.to_string(), output);
            }
            EntityGeometry::Polyline(polyline) => {
                self.write_polyline_entity(polyline, &entity.layer_id.to_string(), output);
            }
            _ => {}
        }
    }

    fn write_line_entity(&self, start: &Point, end: &Point, layer: &str, output: &mut String) {
        output.push_str("  0\n");
        output.push_str("LINE\n");
        output.push_str("  8\n");
        output.push_str(&format!("{}\n", layer));
        output.push_str(" 10\n");
        output.push_str(&format!("{}\n", start.x));
        output.push_str(" 20\n");
        output.push_str(&format!("{}\n", start.y));
        output.push_str(" 11\n");
        output.push_str(&format!("{}\n", end.x));
        output.push_str(" 21\n");
        output.push_str(&format!("{}\n", end.y));
    }

    fn write_circle_entity(&self, center: &Point, radius: f64, layer: &str, output: &mut String) {
        output.push_str("  0\n");
        output.push_str("CIRCLE\n");
        output.push_str("  8\n");
        output.push_str(&format!("{}\n", layer));
        output.push_str(" 10\n");
        output.push_str(&format!("{}\n", center.x));
        output.push_str(" 20\n");
        output.push_str(&format!("{}\n", center.y));
        output.push_str(" 40\n");
        output.push_str(&format!("{}\n", radius));
    }

    fn write_arc_entity(&self, arc: &Arc, layer: &str, output: &mut String) {
        output.push_str("  0\n");
        output.push_str("ARC\n");
        output.push_str("  8\n");
        output.push_str(&format!("{}\n", layer));
        output.push_str(" 10\n");
        output.push_str(&format!("{}\n", arc.center.x));
        output.push_str(" 20\n");
        output.push_str(&format!("{}\n", arc.center.y));
        output.push_str(" 40\n");
        output.push_str(&format!("{}\n", arc.radius));
        output.push_str(" 50\n");
        output.push_str(&format!("{}\n", arc.start_angle.to_degrees()));
        output.push_str(" 51\n");
        output.push_str(&format!("{}\n", arc.end_angle.to_degrees()));
    }

    fn write_polyline_entity(&self, polyline: &Polyline, layer: &str, output: &mut String) {
        output.push_str("  0\n");
        output.push_str("LWPOLYLINE\n");
        output.push_str("  8\n");
        output.push_str(&format!("{}\n", layer));
        output.push_str(" 90\n");
        output.push_str(&format!("{}\n", polyline.vertices.len()));
        output.push_str(" 70\n");
        output.push_str(&format!("{}\n", if polyline.is_closed { 1 } else { 0 }));

        for (i, vertex) in polyline.vertices.iter().enumerate() {
            output.push_str(" 10\n");
            output.push_str(&format!("{}\n", vertex.x));
            output.push_str(" 20\n");
            output.push_str(&format!("{}\n", vertex.y));
        }
    }

    fn write_objects(&self, output: &mut String) {
        output.push_str("SECTION\n");
        output.push_str("  2\n");
        output.push_str("OBJECTS\n");

        output.push_str("  0\n");
        output.push_str("DICTIONARY\n");
        output.push_str("  5\n");
        output.push_str("ACAD_GROUP\n");
        output.push_str("330\n");
        output.push_str("0\n");
        output.push_str("100\n");
        output.push_str("AcDbDictionary\n");

        output.push_str("ENDSEC\n");
    }

    fn write_footer(&self, output: &mut String) {
        output.push_str("EOF\n");
    }
}

/// SVG 导出器，输出像素尺寸画布，并用 `<g transform="scale(1,-1) ...">` 把图纸 Y 轴翻转为屏幕向下方向。
/// 当前只输出直线、圆、圆弧与多段线，且描边颜色固定为 `#000000`。
#[derive(Debug)]
pub struct SVGExporter {
    width: f64,
    height: f64,
    scale: f64,
}

impl SVGExporter {
    /// 创建 SVG 导出器，默认画布 800×600 像素。
    /// 返回的实例可立即导出；实体坐标按图纸原始数值写出，不做自动缩放。
    pub fn new() -> Self {
        Self {
            width: 800.0,
            height: 600.0,
            scale: 1.0,
        }
    }

    /// 设置输出画布尺寸，返回自身以便链式调用。
    /// - `width`、`height`：画布宽高，单位为像素，同时写入根元素的 `width`/`height` 与 `viewBox`。
    /// 消费并返回自身（builder 风格）；仅改变画布声明，图元坐标与 Y 轴翻转依据的是高度值。
    pub fn with_size(mut self, width: f64, height: f64) -> Self {
        self.width = width;
        self.height = height;
        self
    }
}

impl Exporter for SVGExporter {
    fn can_export(&self, extension: &str) -> bool {
        extension.to_lowercase() == "svg"
    }

    fn export_to_file(&self, doc: &Document, filename: &str) -> Result<(), Error> {
        let mut buffer = Vec::new();
        self.write_svg(doc, &mut buffer);

        let mut file = File::create(filename)
            .map_err(|e| Error::ExportError(format!("Failed to create file: {}", e)))?;

        file.write_all(&buffer)
            .map_err(|e| Error::ExportError(format!("Failed to write file: {}", e)))
    }

    fn export_to_bytes(&self, doc: &Document) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::new();
        self.write_svg(doc, &mut buffer);
        Ok(buffer)
    }
}

impl SVGExporter {
    fn write_svg(&self, doc: &Document, buffer: &mut Vec<u8>) {
        let mut writer = Writer::new_with_indent(buffer, b' ', 2);

        let svg_start = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
            self.width, self.height, self.width, self.height
        );

        writer.write_event(Event::Text(BytesText::new(&svg_start)))
            .expect("Failed to write SVG start");

        writer.write_event(Event::Text(BytesText::new("\n")))
            .expect("Failed to write newline");

        writer.write_event(Event::Text(BytesText::new(&format!("  <!-- Generated by CAD SDK -->\n"))))
            .expect("Failed to write comment");

        writer.write_event(Event::Text(BytesText::new(&format!("  <g transform=\"scale(1, -1) translate(0, -{})\">\n", self.height))))
            .expect("Failed to write transform");

        let mut bbox = self.calculate_bbox(doc);

        for (_, entity) in doc.entities() {
            self.write_entity(entity, &mut writer);
        }

        writer.write_event(Event::Text(BytesText::new("  </g>\n")))
            .expect("Failed to write group end");

        writer.write_event(Event::Text(BytesText::new("</svg>\n")))
            .expect("Failed to write SVG end");
    }

    fn calculate_bbox(&self, doc: &Document) -> (f64, f64, f64, f64) {
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for (_, entity) in doc.entities() {
            match &entity.geometry {
                EntityGeometry::Line(line) => {
                    min_x = min_x.min(line.start.x).min(line.end.x);
                    min_y = min_y.min(line.start.y).min(line.end.y);
                    max_x = max_x.max(line.start.x).max(line.end.x);
                    max_y = max_y.max(line.start.y).max(line.end.y);
                }
                EntityGeometry::Circle(circle) => {
                    min_x = min_x.min(circle.center.x - circle.radius);
                    min_y = min_y.min(circle.center.y - circle.radius);
                    max_x = max_x.max(circle.center.x + circle.radius);
                    max_y = max_y.max(circle.center.y + circle.radius);
                }
                _ => {}
            }
        }

        (min_x, min_y, max_x, max_y)
    }

    fn write_entity<W: Write>(&self, entity: &Entity, writer: &mut Writer<W>) {
        match &entity.geometry {
            EntityGeometry::Line(line) => {
                let stroke = self.get_layer_color(&entity.layer_id);
                let element = format!(
                    r#"    <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1" />"#,
                    line.start.x, line.start.y, line.end.x, line.end.y, stroke
                );
                writer.write_event(Event::Text(BytesText::new(&element)))
                    .expect("Failed to write line");
                writer.write_event(Event::Text(BytesText::new("\n")))
                    .expect("Failed to write newline");
            }
            EntityGeometry::Circle(circle) => {
                let stroke = self.get_layer_color(&entity.layer_id);
                let element = format!(
                    r#"    <circle cx="{}" cy="{}" r="{}" fill="none" stroke="{}" stroke-width="1" />"#,
                    circle.center.x, circle.center.y, circle.radius, stroke
                );
                writer.write_event(Event::Text(BytesText::new(&element)))
                    .expect("Failed to write circle");
                writer.write_event(Event::Text(BytesText::new("\n")))
                    .expect("Failed to write newline");
            }
            EntityGeometry::Arc(arc) => {
                let stroke = self.get_layer_color(&entity.layer_id);
                let start_x = arc.center.x + arc.radius * arc.start_angle.to_radians().cos();
                let start_y = arc.center.y + arc.radius * arc.start_angle.to_radians().sin();
                let end_x = arc.center.x + arc.radius * arc.end_angle.to_radians().cos();
                let end_y = arc.center.y + arc.radius * arc.end_angle.to_radians().sin();

                let large_arc = if arc.angle_span() > std::f64::consts::PI { 1 } else { 0 };
                let sweep = if arc.is_counter_clockwise { 1 } else { 0 };

                let element = format!(
                    r#"    <path d="M {} {} A {} {} 0 {} {} {} {}" fill="none" stroke="{}" stroke-width="1" />"#,
                    start_x, start_y, arc.radius, arc.radius, large_arc, sweep, end_x, end_y, stroke
                );
                writer.write_event(Event::Text(BytesText::new(&element)))
                    .expect("Failed to write arc");
                writer.write_event(Event::Text(BytesText::new("\n")))
                    .expect("Failed to write newline");
            }
            EntityGeometry::Polyline(polyline) => {
                if polyline.vertices.len() < 2 {
                    return;
                }

                let points: String = polyline.vertices.iter()
                    .map(|v| format!("{},{}", v.x, v.y))
                    .collect::<Vec<_>>()
                    .join(" ");

                let stroke = self.get_layer_color(&entity.layer_id);
                let fill = if polyline.is_closed { "none" } else { "none" };
                let element = format!(
                    r#"    <polyline points="{}" fill="{}" stroke="{}" stroke-width="1" />"#,
                    points, fill, stroke
                );
                writer.write_event(Event::Text(BytesText::new(&element)))
                    .expect("Failed to write polyline");
                writer.write_event(Event::Text(BytesText::new("\n")))
                    .expect("Failed to write newline");
            }
            _ => {}
        }
    }

    fn get_layer_color(&self, _layer_id: &ObjectId) -> String {
        "#000000".to_string()
    }
}

/// JSON 导出器，把文档名称、版本、单位与实体摘要序列化为文本 JSON。
/// 实体只输出类型、图层与关键坐标：直线给起点/终点、圆给圆心/半径，其余几何一律落为 `"data": {}`；
/// 文本内容不做转义处理，名称或图层名含引号时会破坏 JSON 结构。
#[derive(Debug)]
pub struct JSONExporter;

impl JSONExporter {
    /// 创建 JSON 导出器；结构体无状态，多次导出互不影响。
    pub fn new() -> Self {
        Self
    }
}

impl Exporter for JSONExporter {
    fn can_export(&self, extension: &str) -> bool {
        extension.to_lowercase() == "json"
    }

    fn export_to_file(&self, doc: &Document, filename: &str) -> Result<(), Error> {
        let json = self.to_json(doc);
        std::fs::write(filename, json)
            .map_err(|e| Error::ExportError(format!("Failed to write file: {}", e)))
    }

    fn export_to_bytes(&self, doc: &Document) -> Result<Vec<u8>, Error> {
        Ok(self.to_json(doc).into_bytes())
    }
}

impl JSONExporter {
    fn to_json(&self, doc: &Document) -> String {
        let mut json = String::new();

        json.push_str("{\n");
        json.push_str(&format!("  \"name\": \"{}\",\n", doc.name()));
        json.push_str(&format!("  \"version\": \"{}\",\n", doc.version()));
        json.push_str(&format!("  \"units\": \"{:?}\",\n", doc.units()));
        json.push_str("  \"entities\": [\n");

        let entity_count = doc.entities().len();
        for (i, (_, entity)) in doc.entities().iter().enumerate() {
            json.push_str("    {\n");
            json.push_str(&format!("      \"type\": \"{:?}\",\n", entity.entity_type));
            json.push_str(&format!("      \"layer\": \"{}\",\n", entity.layer_id));

            match &entity.geometry {
                EntityGeometry::Line(line) => {
                    json.push_str(&format!("      \"start\": {{ \"x\": {}, \"y\": {} }},\n", line.start.x, line.start.y));
                    json.push_str(&format!("      \"end\": {{ \"x\": {}, \"y\": {} }}\n", line.end.x, line.end.y));
                }
                EntityGeometry::Circle(circle) => {
                    json.push_str(&format!("      \"center\": {{ \"x\": {}, \"y\": {} }},\n", circle.center.x, circle.center.y));
                    json.push_str(&format!("      \"radius\": {}\n", circle.radius));
                }
                _ => {
                    json.push_str("      \"data\": {}\n");
                }
            }

            json.push_str("    }");
            if i < entity_count - 1 {
                json.push_str(",");
            }
            json.push_str("\n");
        }

        json.push_str("  ]\n");
        json.push_str("}\n");

        json
    }
}

/// 内置导出器清单，持有按注册顺序排列的导出器实例（DXF、SVG、JSON）。
/// 查找按插入顺序进行，返回第一个声明支持该扩展名的实例。
pub struct ExporterRegistry {
    exporters: Vec<Box<dyn Exporter>>,
}

impl ExporterRegistry {
    /// 创建注册表并一次性登记全部内置导出器（DXF、SVG、JSON）。
    /// 返回可直接使用的实例，无需再手工注册。
    /// # 示例
    /// ```
    /// use cadrs::io::exporter_impl::ExporterRegistry;
    /// let registry = ExporterRegistry::new();
    /// assert!(registry.get_exporter("dxf").is_some());
    /// ```
    pub fn new() -> Self {
        let mut registry = Self {
            exporters: Vec::new(),
        };
        registry.register_all();
        registry
    }

    fn register_all(&mut self) {
        self.exporters.push(Box::new(DXFExporter::new()));
        self.exporters.push(Box::new(SVGExporter::new()));
        self.exporters.push(Box::new(JSONExporter::new()));
    }

    /// 按扩展名查找第一个声明支持该扩展名的导出器。
    /// - `extension`：扩展名，由各导出器的 `can_export` 自行做大小写不敏感判断。
    /// 返回导出器的共享引用；内置清单中不存在的格式（如 `"dwg"`）返回 `None`。
    pub fn get_exporter(&self, extension: &str) -> Option<&dyn Exporter> {
        for exporter in &self.exporters {
            if exporter.can_export(extension) {
                return Some(exporter.as_ref());
            }
        }
        None
    }

    /// 按文件扩展名导出到磁盘，同名文件会被覆盖（有写盘副作用）。
    /// - `doc`：源文档，仅读取，不会被修改。
    /// - `filename`：目标路径，扩展名取最后一个点之后的部分并转为小写。
    /// 成功返回 `()`；无扩展名或格式不被支持时返回中文错误描述，写盘失败则透传导出器的错误文本。
    pub fn export_to_file(&self, doc: &Document, filename: &str) -> Result<(), String> {
        let path = Path::new(filename);
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
            .ok_or("无法获取文件扩展名")?;

        if let Some(exporter) = self.get_exporter(&extension) {
            exporter.export_to_file(doc, filename)
                .map_err(|e| e.to_string())
        } else {
            Err(format!("不支持的文件格式: {}", extension))
        }
    }

    /// 把文档导出为内存字节，不产生磁盘副作用。
    /// - `doc`：源文档，仅读取，不会被修改。
    /// - `extension`：目标格式提示，会转为小写后用于选择导出器。
    /// 返回完整的文件内容字节；格式不被支持时返回中文错误描述。
    pub fn export_to_bytes(&self, doc: &Document, extension: &str) -> Result<Vec<u8>, String> {
        let ext = extension.to_lowercase();
        if let Some(exporter) = self.get_exporter(&ext) {
            exporter.export_to_bytes(doc)
                .map_err(|e| e.to_string())
        } else {
            Err(format!("不支持的文件格式: {}", ext))
        }
    }

    /// 列出本注册表实际支持的导出扩展名。
    /// 返回常量列表 `["dxf", "svg", "json"]`，与已登记的导出器集合一致。
    pub fn supported_formats(&self) -> Vec<&'static str> {
        vec!["dxf", "svg", "json"]
    }
}

impl Default for ExporterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 按扩展名直接新建一个导出器实例，无需先构造注册表。
/// - `extension`：扩展名，大小写不敏感。
/// 返回拥有所有权的导出器；`dxf`/`svg`/`json` 之外的扩展名返回 `None`。
/// # 示例
/// ```
/// use cadrs::io::exporter_impl::get_exporter;
/// assert!(get_exporter("dxf").is_some());
/// assert!(get_exporter("dwg").is_none());
/// ```
pub fn get_exporter(extension: &str) -> Option<Box<dyn Exporter>> {
    match extension.to_lowercase().as_str() {
        "dxf" => Some(Box::new(DXFExporter::new())),
        "svg" => Some(Box::new(SVGExporter::new())),
        "json" => Some(Box::new(JSONExporter::new())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_structure::{Document, Entity, EntityType, EntityGeometry, Layer};
    use crate::geometry::{Point, Line, Circle};

    #[test]
    fn test_dxf_exporter() {
        let exporter = DXFExporter::new();
        assert!(exporter.can_export("dxf"));
        assert!(!exporter.can_export("dwg"));
    }

    #[test]
    fn test_svg_exporter() {
        let exporter = SVGExporter::new();
        assert!(exporter.can_export("svg"));
        assert!(!exporter.can_export("dxf"));
    }

    #[test]
    fn test_json_exporter() {
        let exporter = JSONExporter::new();
        assert!(exporter.can_export("json"));
        assert!(!exporter.can_export("svg"));
    }

    #[test]
    fn test_exporter_registry() {
        let registry = ExporterRegistry::new();
        assert!(registry.get_exporter("dxf").is_some());
        assert!(registry.get_exporter("svg").is_some());
        assert!(registry.get_exporter("json").is_some());
        assert!(registry.get_exporter("dwg").is_none());
    }

    #[test]
    fn test_document_export() {
        let mut doc = Document::new("Test".to_string());
        let line = Line::new(Point::new(0.0, 0.0, 0.0), Point::new(100.0, 100.0, 0.0));
        let entity = Entity::new(EntityType::Line, EntityGeometry::Line(line));
        doc.add_entity(entity);

        let exporter = DXFExporter::new();
        let bytes = exporter.export_to_bytes(&doc);

        assert!(bytes.is_ok());
        let content = String::from_utf8(bytes.unwrap()).unwrap();
        assert!(content.contains("LINE"));
        assert!(content.contains("SECTION"));
    }
}
