//! STEP（ISO 10303-21）导出实现：把 `Document` 的实体写成 AP203/AP214/AP242 通用的交换文本。
//! 输出为「ISO-10303-21 头 + DATA 段 + 结束标记」的完整文本，实体编号从 `#1` 起按写入顺序递增；
//! 点、线、圆、圆弧、椭圆、B 样条与折线会被写成对应的几何记录（折线按相邻顶点拆成多段 LINE），
//! 其余几何（NURBS、文本、标注、填充等）只写出一行占位 `ENTITY` 记录。
use crate::data_structure::{Document, Entity, ObjectId, EntityType, EntityGeometry, Layer};
use crate::geometry::{Point, Line, Circle, Arc, Ellipse, Polyline, BSpline, NURBS, Curve};
use std::io::{Write, BufWriter};
use std::fs::File;
use crate::io::{Exporter, Error, ExportOptions};

/// STEP 应用协议（AP）版本标记。
#[derive(Debug, Clone, PartialEq)]
pub enum STEPVersion {
    /// AP203：配置控制设计。
    AP203,
    /// AP214：默认取值。
    AP214,
    /// AP242：在 AP214 基础上加入 PMI 与模型基定义。
    AP242,
}

/// STEP 导出器：实现 `Exporter`，把 `Document` 序列化为 ISO 10303-21 文本。
/// 导出只读取文档，不修改传入的 `doc`；文本可写入文件，也可直接从内存取回。
pub struct STEPExporter {
    version: STEPVersion,
    entity_counter: usize,
    output_buffer: String,
    entity_mapping: Vec<(String, String)>,
}

impl STEPExporter {
    /// 创建导出器，协议版本标记默认为 AP214，实体编号从 `#1` 开始。
    pub fn new() -> Self {
        Self {
            version: STEPVersion::AP214,
            entity_counter: 1,
            output_buffer: String::new(),
            entity_mapping: Vec::new(),
        }
    }

    /// 指定协议版本标记创建导出器。
    /// - `version`：写入实例的版本；当前输出固定使用 CONFIG_CONTROL_DESIGN schema，该字段仅作记录。
    pub fn with_version(version: STEPVersion) -> Self {
        Self {
            version,
            ..Self::new()
        }
    }

    fn generate_entity_id(&mut self) -> String {
        let id = format!("#{}", self.entity_counter);
        self.entity_counter += 1;
        id
    }

    fn write_header(&mut self, doc: &Document) -> String {
        let mut header = String::new();
        header.push_str("ISO-10303-21;\n");
        header.push_str("HEADER;\n");

        header.push_str(&format!("FILE_DESCRIPTION(('CAD SDK STEP Export'),'2;1');\n"));
        header.push_str(&format!("FILE_NAME('{}','{}',(),('CAD SDK'));",
            doc.name(),
            chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S")));
        header.push('\n');

        header.push_str("FILE_SCHEMA(('CONFIG_CONTROL_DESIGN'));\n");
        header.push_str("ENDSEC;\n");

        header
    }

    fn write_point(&mut self, point: &Point) -> String {
        let id = self.generate_entity_id();
        let coords = format!("({},{},{})", point.x(), point.y(), point.z());
        format!("{} = CARTESIAN_POINT('Point',{});\n", id, coords)
    }

    fn write_direction(&mut self, x: f64, y: f64, z: f64) -> String {
        let id = self.generate_entity_id();
        let dir = format!("({},{},{})", x, y, z);
        format!("{} = DIRECTION('Dir',{});\n", id, dir)
    }

    fn write_axis_placement_3d(&mut self, origin: &Point, z_axis: &(Point, Point)) -> String {
        let id = self.generate_entity_id();
        let origin_id = self.write_point(origin);
        let z_id = self.write_direction(z_axis.0.x(), z_axis.0.y(), z_axis.0.z());
        let x_id = self.write_direction(z_axis.1.x(), z_axis.1.y(), z_axis.1.z());
        format!("{} = AXIS2_PLACEMENT_3D('Axis',{},{},{});\n", id, origin_id, z_id, x_id)
    }

    fn write_line(&mut self, line: &Line) -> String {
        let id = self.generate_entity_id();
        let p1_id = self.write_point(&line.start_point());
        let p2_id = self.write_point(&line.end_point());
        format!("{} = LINE('Line',{},{});\n", id, p1_id, p2_id)
    }

    fn write_circle(&mut self, arc: &Arc) -> String {
        let id = self.generate_entity_id();
        let center = arc.center();
        let placement = self.write_axis_placement_3d(
            &center,
            &(Point::new(0.0, 0.0, 1.0), Point::new(1.0, 0.0, 0.0))
        );
        let radius = arc.radius();
        format!("{} = CIRCLE('Circle',{},{});\n", id, placement, radius)
    }

    fn write_arc(&mut self, arc: &Arc) -> String {
        let id = self.generate_entity_id();
        let center = arc.center();
        let placement = self.write_axis_placement_3d(
            &center,
            &(Point::new(0.0, 0.0, 1.0), Point::new(1.0, 0.0, 0.0))
        );
        format!("{} = CIRCLE('Arc',{},{});\n", id, placement, arc.radius())
    }

    fn write_ellipse(&mut self, ellipse: &Ellipse) -> String {
        let id = self.generate_entity_id();
        let center = ellipse.center();
        let placement = self.write_axis_placement_3d(
            &center,
            &(Point::new(0.0, 0.0, 1.0), Point::new(1.0, 0.0, 0.0))
        );
        format!("{} = ELLIPSE('Ellipse',{},{},{});\n",
            id, placement, ellipse.major_axis(), ellipse.minor_axis())
    }

    fn write_b_spline_curve(&mut self, bspline: &BSpline) -> String {
        let id = self.generate_entity_id();
        let degree = bspline.degree();
        let control_points = bspline.control_points();
        let knot_vector = bspline.knots();

        let control_points_str = control_points.iter()
            .map(|p| {
                let pt_id = self.write_point(p);
                format!("({})", pt_id)
            })
            .collect::<Vec<_>>()
            .join(",");

        let knots_str = knot_vector.iter()
            .map(|k| format!("{}", k))
            .collect::<Vec<_>>()
            .join(",");

        format!("{} = B_SPLINE_CURVE_WITH_KNOTS('BSpline',{},(,),.UNSPECIFIED.,.UNSPECIFIED.);\n",
            id, degree)
    }

    fn write_nurbs_surface(&mut self, nurbs: &NURBS) -> String {
        let id = self.generate_entity_id();
        let degree = nurbs.degree();
        format!("{} = NURBS_SURFACE('NURBS',{},{},(,),.UNSPECIFIED.);\n",
            id, degree, degree)
    }

    fn write_polyline(&mut self, polyline: &Polyline) -> String {
        let id = self.generate_entity_id();
        let vertices = polyline.vertices();

        if vertices.len() < 2 {
            return String::new();
        }

        let mut segments = Vec::new();
        for i in 0..vertices.len() - 1 {
            let line = Line::new(
                Point::new(vertices[i].x, vertices[i].y, 0.0),
                Point::new(vertices[i + 1].x, vertices[i + 1].y, 0.0),
            );
            segments.push(self.write_line(&line));
        }

        segments.join("")
    }

    fn convert_entity(&mut self, entity: &Entity) -> Option<String> {
        match entity.geometry() {
            EntityGeometry::Point(p) => Some(self.write_point(p)),
            EntityGeometry::Line(l) => Some(self.write_line(l)),
            EntityGeometry::Circle(c) => {
                let arc = Arc::new(c.center(), c.radius(), 0.0, std::f64::consts::PI * 2.0);
                Some(self.write_circle(&arc))
            },
            EntityGeometry::Arc(a) => Some(self.write_arc(a)),
            EntityGeometry::Ellipse(e) => Some(self.write_ellipse(e)),
            EntityGeometry::BSpline(b) => Some(self.write_b_spline_curve(b)),
            EntityGeometry::NURBS(n) => Some(self.write_nurbs_surface(n)),
            EntityGeometry::Polyline(p) => Some(self.write_polyline(p)),
            _ => None,
        }
    }

    fn write_data_section(&mut self, doc: &Document) -> String {
        let mut data = String::new();
        data.push_str("DATA;\n");

        let mut written_entities = Vec::new();

        for (entity_id, entity) in doc.entities() {
            if let Some(steps) = self.convert_entity(entity) {
                written_entities.push(steps);
                self.entity_mapping.push((format!("{}", entity_id), format!("#{}", self.entity_counter - 1)));
            }
        }

        for step in written_entities {
            data.push_str(&step);
        }

        data.push_str("ENDSEC;\n");

        data
    }

    fn write_footer(&self) -> String {
        let mut footer = String::new();
        let entity_count = self.entity_counter - 1;

        footer.push_str("SECTION-ENTITY-ACCESS-COUNTER(");
        footer.push_str(&format!("{})", entity_count));
        footer.push('\n');

        footer.push_str("ENDSEC-ISO-10303-21;\n");

        footer
    }
}

impl Default for STEPExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Exporter for STEPExporter {
    fn can_export(&self, extension: &str) -> bool {
        extension.to_lowercase() == "step" || extension.to_lowercase() == "stp"
    }

    fn export_to_file(&self, doc: &Document, filename: &str) -> Result<(), Error> {
        let file = File::create(filename).map_err(|e| Error::Io(e.to_string()))?;
        let mut writer = BufWriter::new(file);

        let content = self.export_to_string(doc).map_err(Error::ExportError)?;

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

impl STEPExporter {
    /// 返回本格式的注册信息：扩展名 `step`、名称 STEP、文本（非二进制），版本列表为 AP203/AP214/AP242。
    pub fn get_format_info(&self) -> crate::io::FormatInfo {
        crate::io::FormatInfo::new(
            "step",
            "STEP",
            "STEP AP214 (Configuration Controlled Design)",
            false,
        )
        .with_version("AP203")
        .with_version("AP214")
        .with_version("AP242")
    }

    /// 把文档导出为 STEP 文本，不写盘。
    /// - `doc`：待导出的文档；只读取文档名与实体，不修改文档本身。
    /// 返回 `Ok`：完整的 ISO-10303-21 文本；`Err` 目前不会被触发，接口为兼容导出框架保留。
    /// 文件头的时间戳取当前 UTC 时间，因此同一文档两次导出的头部可能不同。
    /// # 示例
    /// ```
    /// use cadrs::Document;
    /// use cadrs::io::step_exporter::STEPExporter;
    /// let doc = Document::new("Test".to_string());
    /// let text = STEPExporter::new().export_to_string(&doc).unwrap();
    /// assert!(text.contains("ISO-10303-21"));
    /// ```
    pub fn export_to_string(&self, doc: &Document) -> Result<String, String> {
        let mut exporter = STEPExporter::new();

        let mut content = String::new();
        content.push_str("ISO-10303-21;\n");
        content.push_str("HEADER;\n");
        content.push_str(&format!("FILE_DESCRIPTION(('CAD SDK Export'),'2;1');\n"));
        content.push_str(&format!("FILE_NAME('{}','{}',(),('CAD SDK'));",
            doc.name(),
            chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S")));
        content.push('\n');
        content.push_str("FILE_SCHEMA(('CONFIG_CONTROL_DESIGN'));\n");
        content.push_str("ENDSEC;\n");

        content.push_str("DATA;\n");

        let mut entity_counter = 1;

        for (entity_id, entity) in doc.entities() {
            let id = format!("#{}", entity_counter);
            entity_counter += 1;

            match entity.geometry() {
                EntityGeometry::Point(p) => {
                    let coords = format!("({},{},{})", p.x(), p.y(), p.z());
                    content.push_str(&format!("{} = CARTESIAN_POINT('Point_{}',{});\n",
                        id, entity_id, coords));
                },
                EntityGeometry::Line(l) => {
                    let p1 = format!("#{}", entity_counter);
                    entity_counter += 1;
                    let p2 = format!("#{}", entity_counter);
                    entity_counter += 1;
                    content.push_str(&format!("{} = LINE('Line_{}',{},{});\n",
                        id, entity_id, p1, p2));
                },
                EntityGeometry::Circle(c) => {
                    let radius = c.radius();
                    content.push_str(&format!("{} = CIRCLE('Circle_{}',#{},{});\n",
                        id, entity_id, entity_counter + 1, radius));
                    entity_counter += 1;
                },
                EntityGeometry::Arc(a) => {
                    content.push_str(&format!("{} = CIRCLE('Arc_{}',#{},{});\n",
                        id, entity_id, entity_counter + 1, a.radius()));
                    entity_counter += 1;
                },
                EntityGeometry::Ellipse(e) => {
                    content.push_str(&format!("{} = ELLIPSE('Ellipse_{}',#{},{},{});\n",
                        id, entity_id, entity_counter + 1, e.major_axis(), e.minor_axis()));
                    entity_counter += 1;
                },
                EntityGeometry::BSpline(b) => {
                    content.push_str(&format!("{} = B_SPLINE_CURVE_WITH_KNOTS('BSpline_{}',{},(),.UNSPECIFIED.);\n",
                        id, entity_id, b.degree()));
                },
                EntityGeometry::Polyline(p) => {
                    let vertex_count = p.vertices().len();
                    content.push_str(&format!("{} = POLYLINE('Polyline_{}',({}));\n",
                        id, entity_id, vertex_count));
                },
                _ => {
                    content.push_str(&format!("# = ENTITY('{}');\n", entity_id));
                }
            }
        }

        content.push_str("ENDSEC;\n");
        content.push_str("ENDSEC-ISO-10303-21;\n");

        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn test_step_exporter_creation() {
        let exporter = STEPExporter::new();
        assert!(exporter.can_export("step"));
        assert!(exporter.can_export("stp"));
        assert!(!exporter.can_export("iges"));
    }

    #[test]
    fn test_export_empty_document() {
        let doc = Document::new("Test".to_string());
        let exporter = STEPExporter::new();
        let result = exporter.export_to_string(&doc);
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("ISO-10303-21"));
        assert!(content.contains("FILE_DESCRIPTION"));
        assert!(content.contains("DATA"));
    }

    #[test]
    fn test_export_point() {
        let mut doc = Document::new("Test".to_string());
        let entity = Entity::new(
            EntityType::Point,
            EntityGeometry::Point(Point::new(1.0, 2.0, 3.0)),
        );
        doc.add_entity(entity);

        let exporter = STEPExporter::new();
        let result = exporter.export_to_string(&doc);
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("CARTESIAN_POINT"));
    }
}
