//! IGES 导出实现：把 `Document` 的实体写成 IGES 5.x 文本。
//! 输出由头段（H）、全局段（G）、目录段（D）、参数段（P）与结束段（T）组成，
//! 每条实体在目录段占两行（DE 行与 D 行）、在参数段占一行，编号从 1 起按实体顺序递增；
//! 支持点(116)、线(110)、圆与圆弧(100)、椭圆(104)、B 样条(126) 与折线(110)，其余几何（如 NURBS）被整条跳过。
use crate::data_structure::{Document, Entity, ObjectId, EntityType, EntityGeometry, Layer};
use crate::geometry::{Point, Line, Circle, Arc, Ellipse, Polyline, BSpline, NURBS};
use std::io::{Write, BufWriter};
use std::fs::File;
use crate::io::{Exporter, Error, ExportOptions};

/// IGES 版本标记。
#[derive(Debug, Clone, PartialEq)]
pub enum IGESVersion {
    /// IGES 5.0。
    V5_0,
    /// IGES 5.1。
    V5_1,
    /// IGES 5.3，`IGESExporter::new` 的默认取值。
    V5_3,
}

/// IGES 导出器：实现 `Exporter`，把 `Document` 序列化为 IGES 文本。
/// 导出只读取文档，不修改传入的 `doc`；文本可写入文件，也可直接从内存取回。
pub struct IGESExporter {
    version: IGESVersion,
    line_counter: usize,
    parameter_data: Vec<String>,
    directory_entries: Vec<DirectoryEntry>,
    entity_counter: usize,
}

#[derive(Debug, Clone)]
struct DirectoryEntry {
    entity_type: usize,
    parameter_data: usize,
    structure: usize,
    line_font: usize,
    level: usize,
    view: usize,
    transformation: usize,
    display: usize,
    status: (usize, usize, usize, usize),
    entity_number: usize,
    line_weight: usize,
    color: usize,
    param_count: usize,
    form: usize,
    reserved1: usize,
    reserved2: usize,
    entity_label: String,
    subscript: usize,
}

impl IGESExporter {
    /// 创建导出器，版本标记默认为 IGES 5.3，行号与实体编号都从 1 开始。
    pub fn new() -> Self {
        Self::with_version(IGESVersion::V5_3)
    }

    /// 指定版本标记创建导出器。
    /// - `version`：写入实例的版本；当前输出固定为 IGES 5.x 文本结构，该字段仅作记录。
    pub fn with_version(version: IGESVersion) -> Self {
        Self {
            version,
            line_counter: 1,
            parameter_data: Vec::new(),
            directory_entries: Vec::new(),
            entity_counter: 1,
        }
    }

    fn generate_line_number(&mut self) -> usize {
        let line = self.line_counter;
        self.line_counter += 1;
        line
    }

    fn add_parameter(&mut self, param: impl Into<String>) -> usize {
        let line = self.parameter_data.len() + 1;
        self.parameter_data.push(param.into());
        line
    }

    fn write_point(&mut self, point: &Point, entity_id: &ObjectId) -> String {
        let line_num = self.generate_line_number();
        let param_line = format!("1,{},{},{};", point.x(), point.y(), point.z());
        let param_ptr = self.add_parameter(param_line);

        let entry = DirectoryEntry {
            entity_type: 116,
            parameter_data: param_ptr,
            structure: 0,
            line_font: 1,
            level: 0,
            view: 0,
            transformation: 0,
            display: 0,
            status: (0, 1, 0, 0),
            entity_number: self.entity_counter,
            line_weight: 0,
            color: 1,
            param_count: 1,
            form: 0,
            reserved1: 0,
            reserved2: 0,
            entity_label: format!("POINT_{}", entity_id),
            subscript: 0,
        };
        self.directory_entries.push(entry);
        self.entity_counter += 1;

        format!("{}P{}1", line_num, self.entity_counter - 1)
    }

    fn write_line(&mut self, line: &Line, entity_id: &ObjectId) -> String {
        let line_num = self.generate_line_number();
        let param_line = format!("1,{},{},{},{},{},{};",
            line.start_point().x(), line.start_point().y(), line.start_point().z(),
            line.end_point().x(), line.end_point().y(), line.end_point().z());
        let param_ptr = self.add_parameter(param_line);

        let entry = DirectoryEntry {
            entity_type: 110,
            parameter_data: param_ptr,
            structure: 0,
            line_font: 1,
            level: 0,
            view: 0,
            transformation: 0,
            display: 0,
            status: (0, 1, 0, 0),
            entity_number: self.entity_counter,
            line_weight: 0,
            color: 1,
            param_count: 1,
            form: 0,
            reserved1: 0,
            reserved2: 0,
            entity_label: format!("LINE_{}", entity_id),
            subscript: 0,
        };
        self.directory_entries.push(entry);
        self.entity_counter += 1;

        format!("{}P{}1", line_num, self.entity_counter - 1)
    }

    fn write_circle(&mut self, circle: &Circle, entity_id: &ObjectId, is_arc: bool, start_angle: f64, end_angle: f64) -> String {
        let line_num = self.generate_line_number();
        let center = circle.center();
        let z_axis = Point::new(0.0, 0.0, 1.0);
        let x_axis = Point::new(1.0, 0.0, 0.0);

        let param_line = format!("1,{},{},{},{},{},{},{},{},{},{};",
            center.x(), center.y(), center.z(),
            x_axis.x(), x_axis.y(), x_axis.z(),
            z_axis.x(), z_axis.y(), z_axis.z(),
            circle.radius());
        let param_ptr = self.add_parameter(param_line);

        let entity_type = if is_arc { 100 } else { 100 };
        let label = if is_arc { format!("ARC_{}", entity_id) } else { format!("CIRCLE_{}", entity_id) };

        let entry = DirectoryEntry {
            entity_type,
            parameter_data: param_ptr,
            structure: 0,
            line_font: 1,
            level: 0,
            view: 0,
            transformation: 0,
            display: 0,
            status: (0, 1, 0, 0),
            entity_number: self.entity_counter,
            line_weight: 0,
            color: 1,
            param_count: 1,
            form: if is_arc { 1 } else { 0 },
            reserved1: 0,
            reserved2: 0,
            entity_label: label,
            subscript: 0,
        };
        self.directory_entries.push(entry);
        self.entity_counter += 1;

        format!("{}P{}1", line_num, self.entity_counter - 1)
    }

    fn write_ellipse(&mut self, ellipse: &Ellipse, entity_id: &ObjectId) -> String {
        let line_num = self.generate_line_number();
        let center = ellipse.center();
        let z_axis = Point::new(0.0, 0.0, 1.0);
        let x_axis = Point::new(1.0, 0.0, 0.0);

        let param_line = format!("1,{},{},{},{},{},{},{},{},{},{},{},{};",
            center.x(), center.y(), center.z(),
            x_axis.x(), x_axis.y(), x_axis.z(),
            z_axis.x(), z_axis.y(), z_axis.z(),
            ellipse.major_axis(), ellipse.minor_axis(), 0.0);
        let param_ptr = self.add_parameter(param_line);

        let entry = DirectoryEntry {
            entity_type: 104,
            parameter_data: param_ptr,
            structure: 0,
            line_font: 1,
            level: 0,
            view: 0,
            transformation: 0,
            display: 0,
            status: (0, 1, 0, 0),
            entity_number: self.entity_counter,
            line_weight: 0,
            color: 1,
            param_count: 1,
            form: 0,
            reserved1: 0,
            reserved2: 0,
            entity_label: format!("ELLIPSE_{}", entity_id),
            subscript: 0,
        };
        self.directory_entries.push(entry);
        self.entity_counter += 1;

        format!("{}P{}1", line_num, self.entity_counter - 1)
    }

    fn write_b_spline(&mut self, bspline: &BSpline, entity_id: &ObjectId, is_curve: bool) -> String {
        let line_num = self.generate_line_number();
        let degree = bspline.degree();
        let num_points = bspline.control_points().len();

        let mut control_points_str = String::new();
        for p in bspline.control_points() {
            control_points_str.push_str(&format!("{},{},{},", p.x(), p.y(), p.z()));
        }

        let knots_str = bspline.knots().iter()
            .map(|k| format!("{},", k))
            .collect::<String>();

        let param_line = format!("1,{},{},1,{},{},{},{},{},{},1.0,0,0,1.0,{};",
            degree, num_points, control_points_str, knots_str, 0, 0, 0, 0, 0);
        let param_ptr = self.add_parameter(param_line);

        let entity_type = if is_curve { 126 } else { 128 };

        let entry = DirectoryEntry {
            entity_type,
            parameter_data: param_ptr,
            structure: 0,
            line_font: 1,
            level: 0,
            view: 0,
            transformation: 0,
            display: 0,
            status: (0, 1, 0, 0),
            entity_number: self.entity_counter,
            line_weight: 0,
            color: 1,
            param_count: 1,
            form: 0,
            reserved1: 0,
            reserved2: 0,
            entity_label: format!("BSPLINE_{}", entity_id),
            subscript: 0,
        };
        self.directory_entries.push(entry);
        self.entity_counter += 1;

        format!("{}P{}1", line_num, self.entity_counter - 1)
    }

    fn write_polyline(&mut self, polyline: &Polyline, entity_id: &ObjectId) -> String {
        let line_num = self.generate_line_number();
        let vertices = polyline.vertices();

        let mut coords = String::new();
        for v in vertices {
            coords.push_str(&format!("{},{},{},", v.x, v.y, 0.0));
        }

        let param_line = format!("1,{},{};", vertices.len(), coords);
        let param_ptr = self.add_parameter(param_line);

        let entry = DirectoryEntry {
            entity_type: 110,
            parameter_data: param_ptr,
            structure: 0,
            line_font: 1,
            level: 0,
            view: 0,
            transformation: 0,
            display: 0,
            status: (0, 1, 0, 0),
            entity_number: self.entity_counter,
            line_weight: 0,
            color: 1,
            param_count: 1,
            form: 0,
            reserved1: 0,
            reserved2: 0,
            entity_label: format!("POLYLINE_{}", entity_id),
            subscript: 0,
        };
        self.directory_entries.push(entry);
        self.entity_counter += 1;

        format!("{}P{}1", line_num, self.entity_counter - 1)
    }

    fn write_header(&self, doc: &Document) -> String {
        let mut header = String::new();

        let file_time = chrono::Utc::now().format("%H:%M:%S").to_string();
        let file_date = chrono::Utc::now().format("%Y-%m-%d").to_string();

        header.push_str(&format!("H,{0},CAD SDK IGES Export,{0},,{1},{2},,1.0E-4,1,{3},,", doc.name(), file_date, file_time, file_time));
        header.push('\n');

        header.push_str("H,,CAD SDK, , , , , ,1.0E-4,1,;");
        header.push('\n');

        header.push_str(&format!("H,,{0},,{0},,IGES Export,{0},{1},;", file_date, file_time));
        header.push('\n');

        header
    }

    fn write_global(&self) -> String {
        let mut global = String::new();

        global.push_str("G,1H,1.,1,1,1,1,1,0,1,0,16H$");
        global.push_str(&format!("CAD SDK IGES Export,{};", chrono::Utc::now().format("%Y-%m-%d").to_string()));
        global.push('\n');

        global
    }

    fn write_directory_entry(&self, entry: &DirectoryEntry) -> String {
        let mut de = String::new();

        de.push_str(&format!("{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}",
            entry.entity_type, entry.parameter_data, entry.structure, entry.line_font,
            entry.level, entry.view, entry.transformation, entry.display));

        let status = format!("{}{}{}{}", entry.status.0, entry.status.1, entry.status.2, entry.status.3);
        de.push_str(&format!("{:>8}", status));

        de.push_str(&format!("{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}",
            entry.entity_number, entry.line_weight, entry.color,
            entry.param_count, entry.form, entry.reserved1));

        de.push_str(&format!("{:>8}", entry.reserved2));
        de.push_str(&format!("{:>8}{:>8}", entry.entity_label.len(), entry.subscript));
        de.push('\n');

        de.push_str(&format!("D{:>7}1", entry.entity_number));
        de.push('\n');

        de
    }

    fn write_parameter_data(&self) -> String {
        let mut pd = String::new();
        for (i, param) in self.parameter_data.iter().enumerate() {
            pd.push_str(&format!("P{}={}", i + 1, param));
            pd.push('\n');
        }
        pd
    }

    fn write_terminator(&self) -> String {
        let mut term = String::new();

        let total_lines = self.line_counter - 1;
        term.push_str(&format!("S{:>7}G{:>7}D{:>7}P{:>7}",
            total_lines, self.directory_entries.len() * 2, self.directory_entries.len(), self.parameter_data.len()));
        term.push('\n');

        term.push_str("T$");
        term.push('\n');

        term
    }
}

impl Default for IGESExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Exporter for IGESExporter {
    fn can_export(&self, extension: &str) -> bool {
        extension.to_lowercase() == "iges" || extension.to_lowercase() == "igs"
    }

    fn export_to_file(&self, doc: &Document, filename: &str) -> Result<(), Error> {
        let content = self.export_to_string(doc).map_err(Error::ExportError)?;

        let file = File::create(filename).map_err(|e| Error::Io(e.to_string()))?;
        let mut writer = BufWriter::new(file);
        writer.write_all(content.as_bytes())
            .map_err(|e| Error::Io(e.to_string()))?;

        Ok(())
    }

    fn export_to_bytes(&self, doc: &Document) -> Result<Vec<u8>, Error> {
        self.export_to_string(doc)
            .map(|s| s.into_bytes())
            .map_err(Error::ExportError)
    }
}

impl IGESExporter {
    /// 返回本格式的注册信息：扩展名 `iges`、名称 IGES、文本（非二进制），版本列表为 5.0/5.1/5.3。
    pub fn get_format_info(&self) -> crate::io::FormatInfo {
        crate::io::FormatInfo::new(
            "iges",
            "IGES",
            "IGES V5.3 Initial Graphics Exchange Specification",
            false,
        )
        .with_version("5.0")
        .with_version("5.1")
        .with_version("5.3")
    }

    /// 把文档导出为 IGES 文本，不写盘。
    /// - `doc`：待导出的文档；只读取文档名与实体，不修改文档本身；不支持的几何类型不会产生任何输出行。
    /// 返回 `Ok`：完整的 IGES 文本（H/G/D/P/T 各段）；`Err` 目前不会被触发，接口为兼容导出框架保留。
    /// 头段与全局段使用当前 UTC 日期，因此同一文档两次导出的内容可能不同。
    /// # 示例
    /// ```
    /// use cadrs::Document;
    /// use cadrs::io::iges_exporter::IGESExporter;
    /// let doc = Document::new("Test".to_string());
    /// let text = IGESExporter::new().export_to_string(&doc).unwrap();
    /// assert!(text.contains("HDSW1"));
    /// ```
    pub fn export_to_string(&self, doc: &Document) -> Result<String, String> {
        let mut exporter = IGESExporter::new();
        exporter.line_counter = 1;
        exporter.parameter_data.clear();
        exporter.directory_entries.clear();
        exporter.entity_counter = 1;

        let mut content = String::new();
        content.push_str("500HDSW1,1\n");
        content.push_str(&format!("H,CAD SDK IGES Export,{},\n", chrono::Utc::now().format("%Y-%m-%d")));
        content.push_str("H,,CAD SDK, , , , , ,1.0E-4,1,\n");

        content.push_str("G,1H,1.,1,1,1,1,1,0,1,0,16H$CAD SDK Export;\n");
        content.push_str("B,1,1\n");

        let mut dir_entries = Vec::new();
        let mut params = Vec::new();
        let mut entity_num = 1;

        for (entity_id, entity) in doc.entities() {
            match entity.geometry() {
                EntityGeometry::Point(p) => {
                    let param_line = format!("1,{},{},{};", p.x(), p.y(), p.z());
                    params.push(format!("P{}={}", params.len() + 1, param_line));
                    dir_entries.push(format!("     116     1     0     1     0     0     0     0     0 1     0     0     1     0     0     0     0    8        0"));
                    dir_entries.push(format!("D{}     1", entity_num));
                    entity_num += 1;
                },
                EntityGeometry::Line(l) => {
                    let param_line = format!("1,{},{},{},{},{},{};",
                        l.start_point().x(), l.start_point().y(), l.start_point().z(),
                        l.end_point().x(), l.end_point().y(), l.end_point().z());
                    params.push(format!("P{}={}", params.len() + 1, param_line));
                    dir_entries.push(format!("     110     1     0     1     0     0     0     0     0 1     0     0     1     0     0     0     0    8        0"));
                    dir_entries.push(format!("D{}     1", entity_num));
                    entity_num += 1;
                },
                EntityGeometry::Circle(c) => {
                    let center = c.center();
                    let param_line = format!("1,{},{},{},1.,0.,0.,0.,1.,{};", center.x(), center.y(), center.z(), c.radius());
                    params.push(format!("P{}={}", params.len() + 1, param_line));
                    dir_entries.push(format!("     100     1     0     1     0     0     0     0     0 1     0     0     1     0     0     0     0    8        0"));
                    dir_entries.push(format!("D{}     1", entity_num));
                    entity_num += 1;
                },
                EntityGeometry::Arc(a) => {
                    let center = a.center();
                    let param_line = format!("1,{},{},{},1.,0.,0.,0.,1.,{},{},{};", center.x(), center.y(), center.z(), a.radius(), a.start_angle(), a.end_angle());
                    params.push(format!("P{}={}", params.len() + 1, param_line));
                    dir_entries.push(format!("     100     1     0     1     0     0     0     0     0 1     0     0     1     1     0     0     0    8        0"));
                    dir_entries.push(format!("D{}     1", entity_num));
                    entity_num += 1;
                },
                EntityGeometry::Ellipse(e) => {
                    let center = e.center();
                    let param_line = format!("1,{},{},{},1.,0.,0.,0.,1.,{},{},0.;", center.x(), center.y(), center.z(), e.major_axis(), e.minor_axis());
                    params.push(format!("P{}={}", params.len() + 1, param_line));
                    dir_entries.push(format!("     104     1     0     1     0     0     0     0     0 1     0     0     1     0     0     0     0    8        0"));
                    dir_entries.push(format!("D{}     1", entity_num));
                    entity_num += 1;
                },
                EntityGeometry::BSpline(b) => {
                    let param_line = format!("1,{},0,1.,{},0,1.0E+00,0,0,1.0;", b.degree(), b.control_points().len());
                    params.push(format!("P{}={}", params.len() + 1, param_line));
                    dir_entries.push(format!("     126     1     0     1     0     0     0     0     0 1     0     0     1     0     0     0     0    8        0"));
                    dir_entries.push(format!("D{}     1", entity_num));
                    entity_num += 1;
                },
                EntityGeometry::Polyline(p) => {
                    let mut param_line = format!("1,{},", p.vertices().len());
                    for v in p.vertices() {
                        param_line.push_str(&format!("{},{},{},", v.x, v.y, 0.0));
                    }
                    params.push(format!("{};", param_line));
                    dir_entries.push(format!("     110     1     0     1     0     0     0     0     0 1     0     0     1     0     0     0     0    8        0"));
                    dir_entries.push(format!("D{}     1", entity_num));
                    entity_num += 1;
                },
                _ => {}
            }
        }

        for de in &dir_entries {
            content.push_str(de);
            content.push('\n');
        }

        content.push_str("B,1,1\n");

        for param in &params {
            content.push_str(param);
            content.push('\n');
        }

        let total_lines = content.lines().count();
        content.push_str(&format!("S{:>7}G{:>7}D{:>7}P{:>7}\n", total_lines, dir_entries.len(), dir_entries.len(), params.len()));
        content.push_str("T$");

        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iges_exporter_creation() {
        let exporter = IGESExporter::new();
        assert!(exporter.can_export("iges"));
        assert!(exporter.can_export("igs"));
        assert!(!exporter.can_export("step"));
    }

    #[test]
    fn test_export_empty_document() {
        let doc = Document::new("Test".to_string());
        let exporter = IGESExporter::new();
        let result = exporter.export_to_string(&doc);
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("HDSW1"));
    }
}
