use crate::data_structure::{Document, Block, Layer, Entity, ObjectId, EntityType, EntityGeometry};
use crate::geometry::{Point, Line, Circle, Arc, Ellipse, Polyline, BSpline, NURBS};
use std::io::{BufReader, BufRead};
use crate::io::Error;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum IGESVersion {
    V5_0,
    V5_1,
    V5_2,
    V5_3,
}

#[derive(Debug, Clone)]
struct IGESDirectoryEntry {
    entity_type: usize,
    parameter_data: usize,
    structure: usize,
    line_font_pattern: usize,
    level: usize,
    view: usize,
    transformation: usize,
    label_display: usize,
    status_number: (usize, usize, usize, usize),
    sequence_number: usize,
    entity_type_parameter: usize,
    line_weight: usize,
    color_number: usize,
    parameter_line_count: usize,
    form: usize,
    reserved_1: usize,
    reserved_2: usize,
    entity_label: String,
    entity_subscript: usize,
}

struct IGESParser<'a> {
    lines: Vec<&'a str>,
    current_line: usize,
    directory_entries: Vec<IGESDirectoryEntry>,
    parameter_data: HashMap<usize, Vec<String>>,
    current_section: Option<String>,
}

impl<'a> IGESParser<'a> {
    fn new(lines: Vec<&'a str>) -> Self {
        Self {
            lines,
            current_line: 0,
            directory_entries: Vec::new(),
            parameter_data: HashMap::new(),
            current_section: None,
        }
    }

    fn parse_version(&self) -> IGESVersion {
        if self.lines.is_empty() {
            return IGESVersion::V5_3;
        }

        let first_line = self.lines[0];
        if first_line.contains("5.0") {
            IGESVersion::V5_0
        } else if first_line.contains("5.1") {
            IGESVersion::V5_1
        } else if first_line.contains("5.2") {
            IGESVersion::V5_2
        } else if first_line.contains("5.3") {
            IGESVersion::V5_3
        } else {
            IGESVersion::V5_3
        }
    }

    fn is_section_start(&self, line: &str) -> bool {
        line.starts_with('S') || line.starts_with('G') || line.starts_with('D') ||
        line.starts_with('P') || line.starts_with('T')
    }

    fn parse_header_section(&mut self) -> Result<(), String> {
        let mut header_data = HashMap::new();

        while self.current_line < self.lines.len() {
            let line = self.lines[self.current_line].trim();

            if line.starts_with('S') || line.is_empty() {
                self.current_line += 1;
                continue;
            }

            if line.starts_with(',') {
                if let Some(section) = &self.current_section {
                    if section == "HEADER" {
                        if let Some(eq_idx) = line.find(',') {
                            let key = &line[1..eq_idx].trim();
                            let value = &line[eq_idx + 1..].trim().trim_end_matches(',');
                            header_data.insert(key.to_string(), value.to_string());
                        }
                    }
                }
            }

            if line.starts_with("H,") {
                self.current_section = Some("HEADER".to_string());
            } else if line.starts_with("G,") {
                break;
            }

            self.current_line += 1;
        }

        Ok(())
    }

    fn parse_global_section(&mut self) -> Result<(), String> {
        while self.current_line < self.lines.len() {
            let line = self.lines[self.current_line].trim();

            if line.starts_with('B') || line.is_empty() {
                self.current_line += 1;
                continue;
            }

            if line.starts_with("G,") || line.starts_with("GLOBAL") {
            }

            if line.starts_with('D') {
                break;
            }

            self.current_line += 1;
        }

        Ok(())
    }

    fn parse_directory_section(&mut self) -> Result<(), String> {
        let mut index = self.current_line;

        while index < self.lines.len() {
            let line = self.lines[index].trim();

            if line.starts_with('D') && line.len() >= 80 {
                if let Some(entry) = self.parse_directory_entry(line) {
                    self.directory_entries.push(entry);
                }
            } else if line.starts_with('P') || line.starts_with('T') {
                break;
            }

            index += 1;
        }

        self.current_line = index;
        Ok(())
    }

    fn parse_directory_entry(&self, line: &str) -> Option<IGESDirectoryEntry> {
        if line.len() < 80 {
            return None;
        }

        let type_str = line[0..8].trim();
        let entity_type: usize = type_str.parse().ok()?;

        if entity_type == 0 {
            return None;
        }

        let status_str = line[64..72].trim();
        let status_parts: Vec<usize> = status_str
            .chars()
            .filter_map(|c| c.to_string().parse().ok())
            .collect();

        let status_number = (
            status_parts.get(0).copied().unwrap_or(0),
            status_parts.get(1).copied().unwrap_or(0),
            status_parts.get(2).copied().unwrap_or(0),
            status_parts.get(3).copied().unwrap_or(0),
        );

        Some(IGESDirectoryEntry {
            entity_type,
            parameter_data: line[8..16].trim().parse().ok().unwrap_or(0),
            structure: line[16..24].trim().parse().ok().unwrap_or(0),
            line_font_pattern: line[24..32].trim().parse().ok().unwrap_or(0),
            level: line[32..40].trim().parse().ok().unwrap_or(0),
            view: line[40..48].trim().parse().ok().unwrap_or(0),
            transformation: line[48..56].trim().parse().ok().unwrap_or(0),
            label_display: line[56..64].trim().parse().ok().unwrap_or(0),
            status_number,
            sequence_number: line[72..80].trim().parse().ok().unwrap_or(0),
            entity_type_parameter: 0,
            line_weight: 0,
            color_number: 0,
            parameter_line_count: 0,
            form: 0,
            reserved_1: 0,
            reserved_2: 0,
            entity_label: line[64..72].trim().to_string(),
            entity_subscript: 0,
        })
    }

    fn parse_parameter_section(&mut self) -> Result<(), String> {
        let mut current_de_index = 0;

        while self.current_line < self.lines.len() {
            let line = self.lines[self.current_line].trim();

            if line.starts_with('T') || line.is_empty() {
                self.current_line += 1;
                continue;
            }

            if line.starts_with('P') {
                if current_de_index < self.directory_entries.len() {
                    let params = self.parse_parameter_data_line(line);
                    self.parameter_data.insert(current_de_index, params);
                    current_de_index += 1;
                }
            }

            if line.starts_with('T') {
                break;
            }

            self.current_line += 1;
        }

        Ok(())
    }

    fn parse_parameter_data_line(&self, line: &str) -> Vec<String> {
        let mut params = Vec::new();
        let cleaned = match line.find('=') {
            Some(pos) => &line[pos + 1..],
            None => line,
        };

        let mut current = String::new();
        let mut depth = 0;
        let mut in_string = false;

        for c in cleaned.chars() {
            if in_string {
                current.push(c);
                if c == '\'' {
                    in_string = false;
                }
            } else {
                match c {
                    '\'' => {
                        current.push(c);
                        in_string = true;
                    }
                    '(' => {
                        depth += 1;
                        current.push(c);
                    }
                    ')' => {
                        if depth > 0 {
                            depth -= 1;
                            current.push(c);
                        }
                    }
                    ',' if depth == 0 => {
                        let trimmed = current.trim().to_string();
                        if !trimmed.is_empty() {
                            params.push(trimmed);
                        }
                        current.clear();
                    }
                    _ => {
                        current.push(c);
                    }
                }
            }
        }

        if !current.trim().is_empty() {
            params.push(current.trim().to_string());
        }

        params
    }

    fn parse_terminator_section(&mut self) -> Result<(), String> {
        while self.current_line < self.lines.len() {
            let line = self.lines[self.current_line].trim();

            if line.starts_with('T') || line.is_empty() {
                self.current_line += 1;
                continue;
            }

            break;
        }

        Ok(())
    }

    fn create_entity(&self, entry: &IGESDirectoryEntry, params: &[String]) -> Option<Entity> {
        match entry.entity_type {
            100 => self.parse_circular_arc(entry, params),
            102 => self.parse_conic_arc(entry, params),
            104 => self.parse_plane_surface(entry, params),
            108 => self.parse_plane(entry, params),
            110 => self.parse_line(entry, params),
            112 => self.parse_parametric_spline_curve(entry, params),
            114 => self.parse_parametric_spline_surface(entry, params),
            116 => self.parse_point(entry, params),
            118 => self.parse_surface_of_revolution(entry, params),
            120 => self.parse_tabulated_cylinder(entry, params),
            124 => None,
            128 => self.parse_ruled_solid(entry, params),
            130 => self.parse_curve_on_surface(entry, params),
            144 => self.parse_nurbs_curve(entry, params),
            146 => self.parse_nurbs_surface(entry, params),
            402 => None,
            404 => None,
            _ => None,
        }
    }

    fn get_f64_param(&self, params: &[String], index: usize) -> Option<f64> {
        params.get(index).and_then(|p| p.parse::<f64>().ok())
    }

    fn get_point_param(&self, params: &[String], start_idx: usize) -> Option<Point> {
        if start_idx + 2 >= params.len() {
            return None;
        }

        let x = params[start_idx].parse::<f64>().ok()?;
        let y = params[start_idx + 1].parse::<f64>().ok()?;
        let z = params[start_idx + 2].parse::<f64>().ok().unwrap_or(0.0);

        Some(Point::new(x, y, z))
    }

    fn parse_point(&self, entry: &IGESDirectoryEntry, params: &[String]) -> Option<Entity> {
        let point = self.get_point_param(params, 1)?;
        let entity = Entity::new(
            EntityType::Point,
            EntityGeometry::Point(point),
        );
        Some(entity)
    }

    fn parse_line(&self, entry: &IGESDirectoryEntry, params: &[String]) -> Option<Entity> {
        if params.len() < 7 {
            let line = Line::new(Point::origin(), Point::new(1.0, 0.0, 0.0));
            return Some(Entity::new(EntityType::Line, EntityGeometry::Line(line)));
        }

        let p1 = self.get_point_param(params, 1)?;
        let p2 = self.get_point_param(params, 4)?;

        let line = Line::new(p1, p2);
        Some(Entity::new(EntityType::Line, EntityGeometry::Line(line)))
    }

    fn parse_circular_arc(&self, entry: &IGESDirectoryEntry, params: &[String]) -> Option<Entity> {
        if params.len() < 10 {
            let arc = Arc::new(Point::origin(), 1.0, 0.0, std::f64::consts::PI * 2.0);
            return Some(Entity::new(EntityType::Arc, EntityGeometry::Arc(arc)));
        }

        let center = self.get_point_param(params, 1)?;
        let z_axis = self.get_point_param(params, 7)?;
        let x_axis = self.get_point_param(params, 4)?;
        let radius = self.get_f64_param(params, 10)?;

        let start_angle = if entry.form == 1 {
            self.get_f64_param(params, 11).unwrap_or(0.0)
        } else {
            0.0
        };

        let end_angle = if entry.form == 1 {
            self.get_f64_param(params, 12).unwrap_or(std::f64::consts::PI * 2.0)
        } else {
            std::f64::consts::PI * 2.0
        };

        let arc = Arc::new(center, radius, start_angle, end_angle);
        Some(Entity::new(EntityType::Arc, EntityGeometry::Arc(arc)))
    }

    fn parse_conic_arc(&self, entry: &IGESDirectoryEntry, params: &[String]) -> Option<Entity> {
        if params.len() < 13 {
            let ellipse = Ellipse::new(Point::origin(), 1.0, 0.5, 0.0);
            return Some(Entity::new(EntityType::Ellipse, EntityGeometry::Ellipse(ellipse)));
        }

        let center = self.get_point_param(params, 1)?;
        let semi_axis_1 = self.get_f64_param(params, 10)?;
        let semi_axis_2 = self.get_f64_param(params, 11)?;

        let ellipse = Ellipse::new(center, semi_axis_1, semi_axis_2, 0.0);
        Some(Entity::new(EntityType::Ellipse, EntityGeometry::Ellipse(ellipse)))
    }

    fn parse_parametric_spline_curve(&self, entry: &IGESDirectoryEntry, params: &[String]) -> Option<Entity> {
        self.parse_b_spline_common(params)
    }

    fn parse_parametric_spline_surface(&self, entry: &IGESDirectoryEntry, params: &[String]) -> Option<Entity> {
        self.parse_nurbs_common(params)
    }

    fn parse_nurbs_curve(&self, entry: &IGESDirectoryEntry, params: &[String]) -> Option<Entity> {
        self.parse_b_spline_common(params)
    }

    fn parse_nurbs_surface(&self, entry: &IGESDirectoryEntry, params: &[String]) -> Option<Entity> {
        self.parse_nurbs_common(params)
    }

    fn parse_b_spline_common(&self, params: &[String]) -> Option<Entity> {
        if params.len() < 4 {
            let bspline = BSpline::from_points(vec![Point::origin()], 3);
            return Some(Entity::new(EntityType::BSpline, EntityGeometry::BSpline(bspline)));
        }

        let degree = self.get_f64_param(params, 1).unwrap_or(2.0) as usize;
        let num_control_points = self.get_f64_param(params, 2).unwrap_or(0.0) as usize;

        let mut control_points = Vec::new();
        let mut idx = 3;
        for _ in 0..num_control_points {
            if let Some(point) = self.get_point_param(params, idx) {
                control_points.push(point);
                idx += 3;
            } else {
                break;
            }
        }

        if control_points.is_empty() {
            control_points.push(Point::origin());
        }

        let bspline = BSpline::from_points(control_points, degree.max(1));
        Some(Entity::new(EntityType::BSpline, EntityGeometry::BSpline(bspline)))
    }

    fn parse_nurbs_common(&self, params: &[String]) -> Option<Entity> {
        if params.len() < 4 {
            let nurbs = NURBS::from_points(vec![Point::origin()], 3);
            return Some(Entity::new(EntityType::NURBS, EntityGeometry::NURBS(nurbs)));
        }

        let degree_u = self.get_f64_param(params, 1).unwrap_or(3.0).max(1.0) as usize;

        let nurbs = NURBS::from_points(vec![Point::origin()], degree_u);
        Some(Entity::new(EntityType::NURBS, EntityGeometry::NURBS(nurbs)))
    }

    fn parse_surface_of_revolution(&self, entry: &IGESDirectoryEntry, params: &[String]) -> Option<Entity> {
        self.parse_nurbs_common(params)
    }

    fn parse_tabulated_cylinder(&self, entry: &IGESDirectoryEntry, params: &[String]) -> Option<Entity> {
        self.parse_nurbs_common(params)
    }

    fn parse_plane_surface(&self, entry: &IGESDirectoryEntry, params: &[String]) -> Option<Entity> {
        None
    }

    fn parse_plane(&self, entry: &IGESDirectoryEntry, params: &[String]) -> Option<Entity> {
        None
    }

    fn parse_ruled_solid(&self, entry: &IGESDirectoryEntry, params: &[String]) -> Option<Entity> {
        self.parse_nurbs_common(params)
    }

    fn parse_curve_on_surface(&self, entry: &IGESDirectoryEntry, params: &[String]) -> Option<Entity> {
        self.parse_b_spline_common(params)
    }

    fn parse(&mut self) -> Result<(), String> {
        self.current_line = 0;

        self.parse_header_section()?;
        self.parse_global_section()?;
        self.parse_directory_section()?;
        self.parse_parameter_section()?;
        self.parse_terminator_section()?;

        Ok(())
    }

    fn get_document(self) -> Document {
        let mut doc = Document::new("Imported from IGES".to_string());

        for (idx, entry) in self.directory_entries.iter().enumerate() {
            let params = self.parameter_data.get(&idx).cloned().unwrap_or_default();

            if let Some(entity) = self.create_entity(&entry, &params) {
                doc.add_entity(entity);
            }
        }

        doc
    }
}

pub struct IGESImporter {
    version: IGESVersion,
}

impl IGESImporter {
    pub fn new() -> Self {
        Self {
            version: IGESVersion::V5_3,
        }
    }

    pub fn with_version(version: IGESVersion) -> Self {
        Self { version }
    }

    fn parse_file(&self, content: &str) -> Result<Document, String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut parser = IGESParser::new(lines);
        parser.parse()?;
        Ok(parser.get_document())
    }

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
}

impl crate::io::Importer for IGESImporter {
    fn can_import(&self, extension: &str) -> bool {
        extension.to_lowercase() == "iges" || extension.to_lowercase() == "igs"
    }

    fn import_from_file(&self, filename: &str) -> Result<Document, Error> {
        let file = std::fs::File::open(filename).map_err(|e| Error::Io(e.to_string()))?;
        let reader = BufReader::new(file);
        let mut content = String::new();
        for line in reader.lines() {
            let line_content = line.map_err(|e| Error::Io(e.to_string()))?;
            content.push_str(&line_content);
            content.push('\n');
        }
        self.parse_file(&content).map_err(|e| Error::ParseError(e))
    }

    fn import_from_bytes(&self, data: &[u8], extension: &str) -> Result<Document, Error> {
        if !self.can_import(extension) {
            return Err(Error::UnsupportedFormat(extension.to_string()));
        }

        let content = String::from_utf8_lossy(data);
        self.parse_file(&content).map_err(|e| Error::ParseError(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::Importer;

    #[test]
    fn test_iges_importer_can_import() {
        let importer = IGESImporter::new();
        assert!(importer.can_import("iges"));
        assert!(importer.can_import("igs"));
        assert!(importer.can_import("IGES"));
        assert!(!importer.can_import("step"));
    }

    #[test]
    fn test_iges_version_detection() {
        let content = "test header 5.0 IGES file\n";
        let lines: Vec<&str> = content.lines().collect();
        let parser = IGESParser::new(lines);
        let version = parser.parse_version();
        assert_eq!(version, IGESVersion::V5_0);
    }

    #[test]
    fn test_iges_parser_creation() {
        let lines: Vec<&str> = Vec::new();
        let parser = IGESParser::new(lines);
        assert!(parser.directory_entries.is_empty());
    }

    #[test]
    fn test_parse_directory_entry() {
        let line = "     110     1     0     1     0     0     0     0     0 1     0     0     1     0     0     0     0    8        0";
        let parser = IGESParser::new(vec![]);
        let entry = parser.parse_directory_entry(line);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().entity_type, 110);
    }

    #[test]
    fn test_parse_point_entity() {
        let params = vec!["1".to_string(), "0.0".to_string(), "0.0".to_string(), "0.0".to_string()];
        let parser = IGESParser::new(vec![]);
        let entry = IGESDirectoryEntry {
            entity_type: 116,
            parameter_data: 1,
            structure: 0,
            line_font_pattern: 0,
            level: 0,
            view: 0,
            transformation: 0,
            label_display: 0,
            status_number: (0, 0, 0, 0),
            sequence_number: 1,
            entity_type_parameter: 0,
            line_weight: 0,
            color_number: 0,
            parameter_line_count: 0,
            form: 0,
            reserved_1: 0,
            reserved_2: 0,
            entity_label: "POINT".to_string(),
            entity_subscript: 0,
        };

        let entity = parser.create_entity(&entry, &params);
        assert!(entity.is_some());
    }
}
