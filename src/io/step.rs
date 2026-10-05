use crate::data_structure::{Document, Block, Layer, Entity, ObjectId, EntityType, EntityGeometry};
use crate::geometry::{Point, Line, Circle, Arc, Ellipse, Polyline, BSpline, NURBS};
use std::io::{BufReader, BufRead, Read};
use crate::io::Error;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum STEPVersion {
    AP203,
    AP214,
    AP242,
}

#[derive(Debug)]
struct STEPEntity {
    id: String,
    type_name: String,
    parameters: Vec<String>,
    raw_line: String,
}

struct STEPParser<'a> {
    lines: Vec<&'a str>,
    current_line: usize,
    entities: Vec<STEPEntity>,
    entity_params: HashMap<String, Vec<String>>,
    entity_map: HashMap<String, String>,
    current_entity: Option<STEPEntity>,
}

impl<'a> STEPParser<'a> {
    fn new(lines: Vec<&'a str>) -> Self {
        Self {
            lines,
            current_line: 0,
            entities: Vec::new(),
            entity_params: HashMap::new(),
            entity_map: HashMap::new(),
            current_entity: None,
        }
    }

    fn parse_version(&self) -> STEPVersion {
        if self.lines.is_empty() {
            return STEPVersion::AP214;
        }

        for line in &self.lines {
            if line.contains("AP203") {
                return STEPVersion::AP203;
            } else if line.contains("AP214") {
                return STEPVersion::AP214;
            } else if line.contains("AP242") {
                return STEPVersion::AP242;
            }
        }

        STEPVersion::AP214
    }

    fn is_comment(&self, line: &str) -> bool {
        line.trim_start().starts_with("/*") || line.trim_start().starts_with("/*")
    }

    fn is_header_section(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("FILE_DESCRIPTION") ||
        trimmed.starts_with("FILE_NAME") ||
        trimmed.starts_with("FILE_SCHEMA") ||
        trimmed.starts_with("HEADER")
    }

    fn is_data_section(&self, line: &str) -> bool {
        line.trim().starts_with("DATA")
    }

    fn is_end_section(&self, line: &str) -> bool {
        line.trim().starts_with("ENDSEC")
    }

    fn parse_header(&mut self) -> Result<(), String> {
        let mut in_header = false;

        while self.current_line < self.lines.len() {
            let line = self.lines[self.current_line].trim();

            if self.is_comment(line) {
                self.current_line += 1;
                continue;
            }

            if line.starts_with("HEADER") {
                in_header = true;
                self.current_line += 1;
                continue;
            }

            if line.starts_with("ENDSEC") {
                break;
            }

            if in_header && !line.is_empty() {
                self.parse_header_entity(line);
            }

            self.current_line += 1;
        }

        Ok(())
    }

    fn parse_header_entity(&mut self, _line: &str) {
    }

    fn parse_data(&mut self) -> Result<(), String> {
        let mut current_entity_lines: Vec<String> = Vec::new();
        let mut in_entity = false;

        while self.current_line < self.lines.len() {
            let line = self.lines[self.current_line].trim();

            if self.is_comment(line) || line.is_empty() {
                self.current_line += 1;
                continue;
            }

            if self.is_end_section(line) {
                if in_entity && !current_entity_lines.is_empty() {
                    self.finalize_entity(&current_entity_lines);
                    current_entity_lines.clear();
                }
                break;
            }

            if line.starts_with("DATA") {
                in_entity = true;
                self.current_line += 1;
                continue;
            }

            if in_entity {
                current_entity_lines.push(line.to_string());

                if line.ends_with(';') {
                    self.finalize_entity(&current_entity_lines);
                    current_entity_lines.clear();
                }
            }

            self.current_line += 1;
        }

        Ok(())
    }

    fn finalize_entity(&mut self, lines: &[String]) {
        if lines.is_empty() {
            return;
        }

        let full_line: String = lines.join("");

        if full_line.trim().is_empty() {
            return;
        }

        let parts: Vec<&str> = full_line.splitn(2, '=').collect();
        if parts.len() < 2 {
            return;
        }

        let entity_id = parts[0].trim().to_string();
        let type_and_params = parts[1].trim();

        if entity_id.starts_with('#') {
            let type_end = type_and_params.find('(')
                .unwrap_or(type_and_params.len());
            let type_name = &type_and_params[..type_end];
            let params_str = if type_and_params.ends_with(';') {
                &type_and_params[type_end..type_and_params.len()-1]
            } else {
                &type_and_params[type_end..]
            };

            let params = self.parse_parameters(params_str);
            let entity = STEPEntity {
                id: entity_id.clone(),
                type_name: type_name.to_string(),
                parameters: params,
                raw_line: full_line.clone(),
            };

            self.entities.push(entity);
            self.entity_map.insert(entity_id, full_line);
        }
    }

    fn parse_parameters(&self, params_str: &str) -> Vec<String> {
        let mut params = Vec::new();
        let mut current = String::new();
        let mut depth = 0;
        let mut in_string = false;

        for c in params_str.chars() {
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
                        if !current.trim().is_empty() {
                            params.push(current.trim().to_string());
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

    fn get_parameter_as_f64(&self, params: &[String], index: usize) -> Option<f64> {
        params.get(index).and_then(|p| p.parse::<f64>().ok())
    }

    fn get_parameter_as_point(&self, params: &[String], start_idx: usize) -> Option<Point> {
        if start_idx + 2 >= params.len() {
            return None;
        }

        let x = params[start_idx].parse::<f64>().ok()?;
        let y = params[start_idx + 1].parse::<f64>().ok()?;
        let z = params[start_idx + 2].parse::<f64>().ok().unwrap_or(0.0);

        Some(Point::new(x, y, z))
    }

    fn create_entity_from_type(&self, entity: &STEPEntity) -> Option<Entity> {
        match entity.type_name.as_str() {
            "CARTESIAN_POINT" => self.create_point(&entity.parameters),
            "DIRECTION" => self.create_direction(&entity.parameters),
            "AXIS2_PLACEMENT_3D" => self.create_axis2_placement_3d(&entity.parameters),
            "AXIS2_PLACEMENT_2D" => self.create_axis2_placement_2d(&entity.parameters),
            "LINE" => self.create_line(&entity.parameters),
            "CIRCLE" => self.create_circle(&entity.parameters),
            "ELLIPSE" => self.create_ellipse(&entity.parameters),
            "VERTEX_POINT" => self.create_vertex_point(&entity.parameters),
            "EDGE_CURVE" | "ORIENTED_EDGE" => self.create_edge_curve(&entity.parameters),
            "B_SPLINE_CURVE" | "B_SPLINE_CURVE_WITH_KNOTS" => self.create_b_spline_curve(&entity.parameters),
            "BEZIER_CURVE" => self.create_bezier_curve(&entity.parameters),
            "RATIONAL_B_SPLINE_CURVE" => self.create_rational_b_spline_curve(&entity.parameters),
            "TRIMMED_CURVE" => self.create_trimmed_curve(&entity.parameters),
            "COMPOSITE_CURVE" => self.create_composite_curve(&entity.parameters),
            "B_SPLINE_SURFACE" | "B_SPLINE_SURFACE_WITH_KNOTS" => self.create_b_spline_surface(&entity.parameters),
            "BEZIER_SURFACE" => self.create_bezier_surface(&entity.parameters),
            "PLANE" => self.create_plane(&entity.parameters),
            "CYLINDRICAL_SURFACE" => self.create_cylindrical_surface(&entity.parameters),
            _ => None,
        }
    }

    fn create_point(&self, params: &[String]) -> Option<Entity> {
        let location_idx = self.find_param_start(params, "CARTESIAN_POINT")?;
        let point = self.get_parameter_as_point(params, location_idx)?;

        let entity = Entity::new(
            EntityType::Point,
            EntityGeometry::Point(point),
        );
        Some(entity)
    }

    fn create_direction(&self, params: &[String]) -> Option<Entity> {
        let dir_x = self.get_parameter_as_f64(params, 0)?;
        let dir_y = self.get_parameter_as_f64(params, 1)?;
        let dir_z = self.get_parameter_as_f64(params, 2).unwrap_or(0.0);

        let point = Point::new(dir_x, dir_y, dir_z);
        let entity = Entity::new(
            EntityType::Point,
            EntityGeometry::Point(point),
        );
        Some(entity)
    }

    fn create_axis2_placement_3d(&self, params: &[String]) -> Option<Entity> {
        None
    }

    fn create_axis2_placement_2d(&self, params: &[String]) -> Option<Entity> {
        None
    }

    fn create_line(&self, params: &[String]) -> Option<Entity> {
        if params.len() < 2 {
            let line = Line::new(Point::origin(), Point::new(1.0, 0.0, 0.0));
            return Some(Entity::new(EntityType::Line, EntityGeometry::Line(line)));
        }

        let p1_ref = self.resolve_entity_reference(params.get(0)?)?;
        let p2_ref = self.resolve_entity_reference(params.get(1)?)?;

        let p1 = self.extract_point(&p1_ref);
        let p2 = self.extract_point(&p2_ref);

        if p1.is_none() || p2.is_none() {
            let line = Line::new(Point::origin(), Point::new(1.0, 0.0, 0.0));
            return Some(Entity::new(EntityType::Line, EntityGeometry::Line(line)));
        }

        let line = Line::new(p1.unwrap(), p2.unwrap());
        Some(Entity::new(EntityType::Line, EntityGeometry::Line(line)))
    }

    fn create_circle(&self, params: &[String]) -> Option<Entity> {
        if params.len() < 3 {
            let arc = Arc::new(Point::origin(), 1.0, 0.0, std::f64::consts::PI * 2.0);
            return Some(Entity::new(EntityType::Circle, EntityGeometry::Arc(arc)));
        }

        let placement_ref = self.resolve_entity_reference(params.get(0)?)?;
        let radius = self.get_parameter_as_f64(params, 2).unwrap_or(1.0);

        let center = self.extract_axis_placement_center(&placement_ref)
            .unwrap_or(Point::origin());

        let arc = Arc::new(center, radius, 0.0, std::f64::consts::PI * 2.0);
        Some(Entity::new(EntityType::Circle, EntityGeometry::Arc(arc)))
    }

    fn create_ellipse(&self, params: &[String]) -> Option<Entity> {
        if params.len() < 4 {
            let ellipse = Ellipse::new(Point::origin(), 1.0, 0.5, 0.0);
            return Some(Entity::new(EntityType::Ellipse, EntityGeometry::Ellipse(ellipse)));
        }

        let placement_ref = self.resolve_entity_reference(params.get(0)?)?;
        let center = self.extract_axis_placement_center(&placement_ref)
            .unwrap_or(Point::origin());

        let semi_axis_1 = self.get_parameter_as_f64(params, 1).unwrap_or(1.0);
        let semi_axis_2 = self.get_parameter_as_f64(params, 2).unwrap_or(0.5);

        let ellipse = Ellipse::new(center, semi_axis_1, semi_axis_2, 0.0);
        Some(Entity::new(EntityType::Ellipse, EntityGeometry::Ellipse(ellipse)))
    }

    fn create_vertex_point(&self, params: &[String]) -> Option<Entity> {
        self.create_point(params)
    }

    fn create_edge_curve(&self, params: &[String]) -> Option<Entity> {
        self.create_line(params)
    }

    fn create_b_spline_curve(&self, params: &[String]) -> Option<Entity> {
        let degree = self.get_parameter_as_f64(params, 0).unwrap_or(2.0) as usize;

        let control_points_idx = self.find_array_start(params, 1)?;
        let num_control_points = self.get_parameter_as_f64(params, control_points_idx).unwrap_or(0.0) as usize;

        let mut control_points = Vec::new();
        for i in 0..num_control_points {
            if let Some(point) = self.get_parameter_as_point(params, control_points_idx + 1 + i * 3) {
                control_points.push(point);
            }
        }

        if control_points.is_empty() {
            control_points.push(Point::origin());
        }

        let bspline = BSpline::from_points(control_points, degree.max(1));
        Some(Entity::new(EntityType::BSpline, EntityGeometry::BSpline(bspline)))
    }

    fn create_bezier_curve(&self, params: &[String]) -> Option<Entity> {
        self.create_b_spline_curve(params)
    }

    fn create_rational_b_spline_curve(&self, params: &[String]) -> Option<Entity> {
        self.create_b_spline_curve(params)
    }

    fn create_trimmed_curve(&self, params: &[String]) -> Option<Entity> {
        self.create_line(params)
    }

    fn create_composite_curve(&self, params: &[String]) -> Option<Entity> {
        None
    }

    fn create_b_spline_surface(&self, params: &[String]) -> Option<Entity> {
        let degree_u = self.get_parameter_as_f64(params, 0).unwrap_or(3.0).max(1.0) as usize;

        let nurbs = NURBS::from_points(vec![Point::origin()], degree_u);
        Some(Entity::new(EntityType::NURBS, EntityGeometry::NURBS(nurbs)))
    }

    fn create_bezier_surface(&self, params: &[String]) -> Option<Entity> {
        self.create_b_spline_surface(params)
    }

    fn create_plane(&self, params: &[String]) -> Option<Entity> {
        None
    }

    fn create_cylindrical_surface(&self, params: &[String]) -> Option<Entity> {
        let nurbs = NURBS::from_points(vec![Point::origin()], 3);
        Some(Entity::new(EntityType::NURBS, EntityGeometry::NURBS(nurbs)))
    }

    fn find_param_start(&self, params: &[String], _type_name: &str) -> Option<usize> {
        params.iter().position(|p| p.starts_with('#') || p.parse::<f64>().is_ok())
    }

    fn find_array_start(&self, params: &[String], start: usize) -> Option<usize> {
        for i in start..params.len() {
            if params[i].parse::<f64>().is_ok() {
                return Some(i);
            }
        }
        None
    }

    fn resolve_entity_reference(&self, param: &str) -> Option<String> {
        let cleaned = param.trim();
        if cleaned.starts_with('#') {
            if let Some(line) = self.entity_map.get(cleaned) {
                return Some(line.clone());
            }
        }
        Some(param.to_string())
    }

    fn extract_point(&self, data: &str) -> Option<Point> {
        let coords: Vec<f64> = data
            .trim()
            .trim_start_matches('(')
            .trim_end_matches(')')
            .split(',')
            .filter_map(|s| s.parse::<f64>().ok())
            .collect();

        match coords.len() {
            2 => Some(Point::new(coords[0], coords[1], 0.0)),
            3 => Some(Point::new(coords[0], coords[1], coords[2])),
            _ => None,
        }
    }

    fn extract_axis_placement_center(&self, entity_data: &str) -> Option<Point> {
        if !entity_data.contains("CARTESIAN_POINT") {
            return None;
        }

        self.extract_point(entity_data)
    }

    fn parse(&mut self) -> Result<(), String> {
        self.current_line = 0;

        while self.current_line < self.lines.len() {
            let line = self.lines[self.current_line].trim();

            if self.is_header_section(line) {
                self.parse_header()?;
            } else if self.is_data_section(line) {
                self.parse_data()?;
            }

            self.current_line += 1;
        }

        Ok(())
    }

    fn get_document(mut self) -> Document {
        let mut doc = Document::new("Imported from STEP".to_string());

        for entity in &self.entities {
            if let Some(cad_entity) = self.create_entity_from_type(entity) {
                doc.add_entity(cad_entity);
            }
        }

        doc
    }
}

pub struct STEPImporter {
    version: STEPVersion,
}

impl STEPImporter {
    pub fn new() -> Self {
        Self {
            version: STEPVersion::AP214,
        }
    }

    pub fn with_version(version: STEPVersion) -> Self {
        Self { version }
    }

    fn parse_file(&self, content: &str) -> Result<Document, String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut parser = STEPParser::new(lines);
        parser.parse()?;
        Ok(parser.get_document())
    }

    fn detect_version(&self, content: &str) -> STEPVersion {
        if content.contains("AP203") {
            return STEPVersion::AP203;
        } else if content.contains("AP242") {
            return STEPVersion::AP242;
        }
        STEPVersion::AP214
    }

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
}

impl crate::io::Importer for STEPImporter {
    fn can_import(&self, extension: &str) -> bool {
        extension.to_lowercase() == "step" || extension.to_lowercase() == "stp"
    }

    fn import_from_file(&self, filename: &str) -> Result<Document, Error> {
        let file = std::fs::File::open(filename).map_err(|e| Error::Io(e.to_string()))?;
        let mut content = String::new();
        let mut reader = BufReader::new(file);
        reader.read_to_string(&mut content).map_err(|e| Error::Io(e.to_string()))?;

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
    fn test_step_importer_can_import() {
        let importer = STEPImporter::new();
        assert!(importer.can_import("step"));
        assert!(importer.can_import("stp"));
        assert!(!importer.can_import("iges"));
    }

    #[test]
    fn test_step_version_detection() {
        let importer = STEPImporter::new();
        assert_eq!(importer.detect_version("/* AP203 file */"), STEPVersion::AP203);
        assert_eq!(importer.detect_version("/* AP214 file */"), STEPVersion::AP214);
        assert_eq!(importer.detect_version("/* AP242 file */"), STEPVersion::AP242);
        assert_eq!(importer.detect_version("/* unknown */"), STEPVersion::AP214);
    }

    #[test]
    fn test_parse_cartesian_point() {
        let content = "#100 = CARTESIAN_POINT('Origin',(0.0,0.0,0.0));\n";
        let lines: Vec<&str> = content.lines().collect();
        let parser = STEPParser::new(lines);
        assert!(!parser.entities.is_empty() || parser.entity_map.contains_key("#100"));
    }

    #[test]
    fn test_parse_line_entity() {
        let content = "#100 = CARTESIAN_POINT('P1',(0.0,0.0,0.0));\n#101 = CARTESIAN_POINT('P2',(1.0,1.0,0.0));\n#102 = LINE('L1',#100,#101);\n";
        let lines: Vec<&str> = content.lines().collect();
        let parser = STEPParser::new(lines);
        assert!(parser.entity_map.contains_key("#102"));
    }
}
