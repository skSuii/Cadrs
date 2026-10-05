use crate::data_structure::{Document, Block, Layer, Entity, ObjectId, EntityType, EntityGeometry, Visibility, Transform, HatchBoundary, BoundaryType, HatchEdge, EdgeType, DimensionType, TextStyle, TextAlignment};
use crate::geometry::{Point, Line, Circle, Arc, Ellipse, Polyline, BSpline, NURBS};
use std::io::{Read, Seek, SeekFrom};
use thiserror::Error;
use crate::io::Error as ImportError;

#[derive(Debug, Error)]
pub enum DWGError {
    #[error("Failed to parse DWG file: {0}")]
    ParseError(String),
    
    #[error("Invalid DWG version: {0}")]
    InvalidVersion(String),
    
    #[error("Unsupported DWG version: {0}")]
    UnsupportedVersion(String),
    
    #[error("CRC error: {0}")]
    CRCError(String),
    
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DWGVersion {
    R12,
    R13,
    R14,
    R2000,
    R2004,
    R2007,
    R2010,
    R2013,
    R2018,
}

impl DWGVersion {
    pub fn from_magic(magic: &[u8]) -> Option<Self> {
        let magic_str = String::from_utf8_lossy(magic);
        if magic_str.contains("AC1009") {
            Some(DWGVersion::R12)
        } else if magic_str.contains("AC1012") {
            Some(DWGVersion::R13)
        } else if magic_str.contains("AC1014") {
            Some(DWGVersion::R14)
        } else if magic_str.contains("AC1015") {
            Some(DWGVersion::R2000)
        } else if magic_str.contains("AC1018") {
            Some(DWGVersion::R2004)
        } else if magic_str.contains("AC1021") {
            Some(DWGVersion::R2007)
        } else if magic_str.contains("AC1024") {
            Some(DWGVersion::R2010)
        } else if magic_str.contains("AC1027") {
            Some(DWGVersion::R2013)
        } else if magic_str.contains("AC1032") {
            Some(DWGVersion::R2018)
        } else {
            None
        }
    }
}

struct DWGEntity {
    entity_type: String,
    handles: Vec<String>,
    string_data: Vec<String>,
    double_data: Vec<f64>,
    int_data: Vec<i32>,
    point_data: Vec<Point>,
}

struct DWGSection {
    section_id: u16,
    size: usize,
    offset: usize,
    compressed: bool,
}

struct DWGParser<'a> {
    data: &'a [u8],
    cursor: usize,
    version: DWGVersion,
    entities: Vec<Entity>,
    layers: Vec<(String, ObjectId)>,
    blocks: Vec<Block>,
    block_names: Vec<String>,
    current_block: Option<String>,
    object_map: std::collections::HashMap<String, DWGEntity>,
    entity_handles: std::collections::HashMap<String, usize>,
}

impl<'a> DWGParser<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            cursor: 0,
            version: DWGVersion::R12,
            entities: Vec::new(),
            layers: Vec::new(),
            blocks: Vec::new(),
            block_names: Vec::new(),
            current_block: None,
            object_map: std::collections::HashMap::new(),
            entity_handles: std::collections::HashMap::new(),
        }
    }

    fn read_u8(&mut self) -> Result<u8, DWGError> {
        if self.cursor >= self.data.len() {
            return Err(DWGError::ParseError("EOF reached".to_string()));
        }
        let value = self.data[self.cursor];
        self.cursor += 1;
        Ok(value)
    }

    fn read_u16(&mut self) -> Result<u16, DWGError> {
        if self.cursor + 1 >= self.data.len() {
            return Err(DWGError::ParseError("EOF reached".to_string()));
        }
        let value = u16::from_le_bytes([self.data[self.cursor], self.data[self.cursor + 1]]);
        self.cursor += 2;
        Ok(value)
    }

    fn read_u32(&mut self) -> Result<u32, DWGError> {
        if self.cursor + 3 >= self.data.len() {
            return Err(DWGError::ParseError("EOF reached".to_string()));
        }
        let value = u32::from_le_bytes([
            self.data[self.cursor],
            self.data[self.cursor + 1],
            self.data[self.cursor + 2],
            self.data[self.cursor + 3],
        ]);
        self.cursor += 4;
        Ok(value)
    }

    fn read_f64(&mut self) -> Result<f64, DWGError> {
        if self.cursor + 7 >= self.data.len() {
            return Err(DWGError::ParseError("EOF reached".to_string()));
        }
        let bytes: [u8; 8] = [
            self.data[self.cursor],
            self.data[self.cursor + 1],
            self.data[self.cursor + 2],
            self.data[self.cursor + 3],
            self.data[self.cursor + 4],
            self.data[self.cursor + 5],
            self.data[self.cursor + 6],
            self.data[self.cursor + 7],
        ];
        let value = f64::from_le_bytes(bytes);
        self.cursor += 8;
        Ok(value)
    }

    fn read_string(&mut self, length: usize) -> Result<String, DWGError> {
        if self.cursor + length >= self.data.len() {
            return Err(DWGError::ParseError("EOF reached".to_string()));
        }
        let bytes = &self.data[self.cursor..self.cursor + length];
        self.cursor += length;
        let s = String::from_utf8_lossy(bytes).trim_end_matches('\0').to_string();
        Ok(s)
    }

    fn read_crc(&mut self) -> Result<u16, DWGError> {
        self.read_u16()
    }

    fn verify_crc(&self, start: usize, end: usize, expected: u16) -> bool {
        let mut crc: u16 = 0;
        for i in start..end {
            crc ^= self.data[i] as u16;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ 0xA001;
                } else {
                    crc >>= 1;
                }
            }
        }
        crc == expected
    }

    fn parse_version(&mut self) -> Result<(), DWGError> {
        if self.cursor + 6 > self.data.len() {
            return Err(DWGError::ParseError("File too small".to_string()));
        }
        
        let magic = &self.data[self.cursor..self.cursor + 6];
        if let Some(version) = DWGVersion::from_magic(magic) {
            self.version = version;
            self.cursor += 6;
            Ok(())
        } else {
            Err(DWGError::InvalidVersion("Unknown".to_string()))
        }
    }

    fn parse_header_section(&mut self) -> Result<(), DWGError> {
        let section_id = self.read_u16()?;
        let section_size = self.read_u32()? as usize;
        let section_start = self.cursor;
        
        match self.version {
            DWGVersion::R12 | DWGVersion::R13 | DWGVersion::R14 => {
                self.parse_header_variables()?;
            }
            DWGVersion::R2000 | DWGVersion::R2004 | DWGVersion::R2007 | 
            DWGVersion::R2010 | DWGVersion::R2013 | DWGVersion::R2018 => {
                self.parse_section_header()?;
                self.parse_header_variables()?;
            }
        }
        
        self.cursor = section_start + section_size;
        Ok(())
    }

    fn parse_section_header(&mut self) -> Result<(), DWGError> {
        let section_number = self.read_u16()?;
        let section_name_length = self.read_u8()?;
        let _section_name = self.read_string(section_name_length as usize)?;
        let page_count = self.read_u32()?;
        let _page_size = self.read_u32()?;
        let _ = self.read_u32()?;
        Ok(())
    }

    fn parse_header_variables(&mut self) -> Result<(), DWGError> {
        while self.cursor < self.data.len() - 2 {
            let marker = self.read_u8()?;
            if marker == 0x0D || marker == 0x0A {
                continue;
            }
            if marker == 0x03 || marker == 0x3D {
                break;
            }
            
            self.cursor -= 1;
            let data = self.read_string(64)?;
            if data.contains("$ACADVER") {
                if let Some(start) = data.find('$') {
                    if let Some(end) = data[start..].find(|c: char| !c.is_ascii_alphanumeric()) {
                        let version_str = &data[start + 1..start + end];
                        self.cursor -= 1;
                        return Ok(());
                    }
                }
            }
        }
        Ok(())
    }

    fn parse_classes_section(&mut self) -> Result<(), DWGError> {
        let section_size = self.read_u32()? as usize;
        let section_start = self.cursor;
        let class_count = self.read_u32()?;
        
        for _ in 0..class_count {
            let class_num = self.read_u16()?;
            let class_name = self.read_string(64)?;
            let app_name = self.read_string(64)?;
            let _flags = self.read_u8()?;
            let _was_proxy = self.read_u8()?;
        }
        
        self.cursor = section_start + section_size;
        Ok(())
    }

    fn parse_tables_section(&mut self) -> Result<(), DWGError> {
        match self.version {
            DWGVersion::R12 | DWGVersion::R13 | DWGVersion::R14 => {
                self.parse_tables_section_legacy()?;
            }
            _ => {
                self.parse_tables_section_modern()?;
            }
        }
        Ok(())
    }

    fn parse_tables_section_legacy(&mut self) -> Result<(), DWGError> {
        let table_count = self.read_u16()?;
        
        for _ in 0..table_count {
            let table_type = self.read_string(64)?;
            let max_entries = self.read_u16()?;
            let entry_count = self.read_u16()?;
            
            match table_type.as_str() {
                "LAYER" => {
                    for _ in 0..entry_count {
                        let layer_name = self.read_string(64)?;
                        let flags = self.read_u8()?;
                        let _color = self.read_u16()?;
                        let _linetype = self.read_string(64)?;
                        self.layers.push((layer_name, ObjectId::new()));
                    }
                }
                "BLOCK" => {
                    for _ in 0..entry_count {
                        let block_name = self.read_string(64)?;
                        self.block_names.push(block_name);
                    }
                }
                _ => {
                    for _ in 0..entry_count {
                        let _name = self.read_string(64)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn parse_tables_section_modern(&mut self) -> Result<(), DWGError> {
        let section_id = self.read_u16()?;
        let _section_size = self.read_u32()?;
        
        let layer_table_size = self.read_u32()?;
        let _layer_table_start = self.cursor;
        self.cursor += layer_table_size as usize;
        
        let block_table_size = self.read_u32()?;
        let _block_table_start = self.cursor;
        self.cursor += block_table_size as usize;
        
        Ok(())
    }

    fn parse_blocks_section(&mut self) -> Result<(), DWGError> {
        let section_size = self.read_u32()? as usize;
        let section_start = self.cursor;
        
        let num_blocks = self.read_u16()?;
        
        for _ in 0..num_blocks {
            let block_name = self.read_string(64)?;
            let _flags = self.read_u8()?;
            let _layer = self.read_u16()?;
            let _base_point = self.read_f64()?;
            let _base_point_y = self.read_f64()?;
            let _base_point_z = self.read_f64()?;
            let _block_type = self.read_u16()?;
            
            let entity_count = self.read_u16()?;
            
            self.current_block = Some(block_name.clone());
            let mut block = Block::new(block_name);
            
            for _ in 0..entity_count {
                if let Some(entity) = self.parse_entity()? {
                    block.add_entity(entity);
                }
            }
            
            self.blocks.push(block);
        }
        
        self.cursor = section_start + section_size;
        Ok(())
    }

    fn parse_entities_section(&mut self) -> Result<(), DWGError> {
        let section_size = self.read_u32()? as usize;
        let section_start = self.cursor;
        
        match self.version {
            DWGVersion::R12 | DWGVersion::R13 | DWGVersion::R14 => {
                self.parse_entities_legacy()?;
            }
            _ => {
                self.parse_entities_modern()?;
            }
        }
        
        self.cursor = section_start + section_size;
        Ok(())
    }

    fn parse_entities_legacy(&mut self) -> Result<(), DWGError> {
        let num_entities = self.read_u16()?;
        
        for _ in 0..num_entities {
            if let Some(entity) = self.parse_entity()? {
                self.entities.push(entity);
            }
        }
        Ok(())
    }

    fn parse_entities_modern(&mut self) -> Result<(), DWGError> {
        loop {
            let entity_type_id = self.read_u16()?;
            if entity_type_id == 0 {
                break;
            }
            
            if let Some(entity) = self.parse_entity_by_type(entity_type_id)? {
                self.entities.push(entity);
            }
        }
        Ok(())
    }

    fn parse_entity(&mut self) -> Result<Option<Entity>, DWGError> {
        let handle = self.read_u8()?;
        let entity_type_id = self.read_u16()?;
        
        self.parse_entity_by_type(entity_type_id)
    }

    fn parse_entity_by_type(&mut self, entity_type_id: u16) -> Result<Option<Entity>, DWGError> {
        match entity_type_id {
            1 => self.parse_line_entity(),
            2 => self.parse_circle_entity(),
            3 => self.parse_arc_entity(),
            4 => self.parse_text_entity(),
            5 => self.parse_dimension_entity(),
            6 => self.parse_insert_entity(),
            7 => self.parse_polyline_entity(),
            8 => self.parse_spline_entity(),
            9 => self.parse_hatch_entity(),
            10 => self.parse_ellipse_entity(),
            11 => self.parse_point_entity(),
            12 => self.parse_lwpolyline_entity(),
            _ => {
                let _size = self.read_u16()?;
                Ok(None)
            }
        }
    }

    fn parse_line_entity(&mut self) -> Result<Option<Entity>, DWGError> {
        let start = Point::new(
            self.read_f64()?,
            self.read_f64()?,
            self.read_f64()?,
        );
        let end = Point::new(
            self.read_f64()?,
            self.read_f64()?,
            self.read_f64()?,
        );
        let _thickness = self.read_f64()?;
        let _extrusion = self.read_f64()?;
        
        let line = Line::new(start, end);
        Ok(Some(Entity::new(
            EntityType::Line,
            EntityGeometry::Line(line),
        )))
    }

    fn parse_circle_entity(&mut self) -> Result<Option<Entity>, DWGError> {
        let center = Point::new(
            self.read_f64()?,
            self.read_f64()?,
            self.read_f64()?,
        );
        let radius = self.read_f64()?;
        let _thickness = self.read_f64()?;
        let _extrusion = self.read_f64()?;
        
        let circle = Circle::new(center, radius);
        Ok(Some(Entity::new(
            EntityType::Circle,
            EntityGeometry::Circle(circle),
        )))
    }

    fn parse_arc_entity(&mut self) -> Result<Option<Entity>, DWGError> {
        let center = Point::new(
            self.read_f64()?,
            self.read_f64()?,
            self.read_f64()?,
        );
        let radius = self.read_f64()?;
        let start_angle = self.read_f64()?;
        let end_angle = self.read_f64()?;
        let _thickness = self.read_f64()?;
        let _extrusion = self.read_f64()?;
        
        let arc = Arc::new(center, radius, start_angle, end_angle);
        Ok(Some(Entity::new(
            EntityType::Arc,
            EntityGeometry::Arc(arc),
        )))
    }

    fn parse_ellipse_entity(&mut self) -> Result<Option<Entity>, DWGError> {
        let center = Point::new(
            self.read_f64()?,
            self.read_f64()?,
            self.read_f64()?,
        );
        let major_x = self.read_f64()?;
        let major_y = self.read_f64()?;
        let major_z = self.read_f64()?;
        let major_length = (major_x * major_x + major_y * major_y + major_z * major_z).sqrt();
        let ratio = self.read_f64()?;
        let start_angle = self.read_f64()?;
        let end_angle = self.read_f64()?;
        
        let minor_length = major_length * ratio;
        let rotation = if major_x.abs() > major_y.abs() {
            (major_y / major_x).atan()
        } else if major_y != 0.0 {
            std::f64::consts::PI / 2.0 - (major_x / major_y).atan()
        } else {
            0.0
        };
        
        let ellipse = Ellipse::new(center, major_length / 2.0, minor_length / 2.0, rotation);
        Ok(Some(Entity::new(
            EntityType::Ellipse,
            EntityGeometry::Ellipse(ellipse),
        )))
    }

    fn parse_point_entity(&mut self) -> Result<Option<Entity>, DWGError> {
        let x = self.read_f64()?;
        let y = self.read_f64()?;
        let z = self.read_f64()?;
        
        let point = Point::new(x, y, z);
        Ok(Some(Entity::new(
            EntityType::Point,
            EntityGeometry::Point(point),
        )))
    }

    fn parse_text_entity(&mut self) -> Result<Option<Entity>, DWGError> {
        let insertion_point = Point::new(
            self.read_f64()?,
            self.read_f64()?,
            self.read_f64()?,
        );
        let height = self.read_f64()?;
        let width_factor = self.read_f64()?;
        let rotation = self.read_f64()?;
        let _flags = self.read_u8()?;
        let text_length = self.read_u16()?;
        let content = self.read_string(text_length as usize)?;
        
        Ok(Some(Entity::new(
            EntityType::Text,
            EntityGeometry::Text {
                content,
                position: insertion_point,
                height,
                rotation,
                width_factor: 1.0,
                font_name: "Standard".to_string(),
                style: TextStyle {
                    bold: false,
                    italic: false,
                    underline: false,
                    alignment: TextAlignment::Left,
                },
            },
        )))
    }

    fn parse_dimension_entity(&mut self) -> Result<Option<Entity>, DWGError> {
        let _geometry = self.read_u32()?;
        let _attachment = self.read_u16()?;
        let _line_spacing_style = self.read_u16()?;
        let _line_spacing_factor = self.read_f64()?;
        let _actual_measurement = self.read_f64()?;
        let text_length = self.read_u16()?;
        let content = self.read_string(text_length as usize)?;
        let insertion_point = Point::new(
            self.read_f64()?,
            self.read_f64()?,
            self.read_f64()?,
        );
        let _rotation = self.read_f64()?;
        let _horizontal_direction = self.read_f64()?;
        let _vertical_direction = self.read_f64()?;
        
        Ok(Some(Entity::new(
            EntityType::Dimension,
            EntityGeometry::Dimension {
                dim_type: DimensionType::Linear,
                measurement: 0.0,
                text: content.clone(),
                text_position: insertion_point,
                text_height: 2.5,
                text_rotation: 0.0,
                definition_point: insertion_point,
                def_point_1: insertion_point,
                def_point_2: insertion_point,
                def_point_3: insertion_point,
                def_point_4: insertion_point,
                angle: 0.0,
                extension_lines: true,
                center_marks: false,
            },
        )))
    }

    fn parse_insert_entity(&mut self) -> Result<Option<Entity>, DWGError> {
        let block_name_length = self.read_u16()?;
        let block_name = self.read_string(block_name_length as usize)?;
        let insertion_point = Point::new(
            self.read_f64()?,
            self.read_f64()?,
            self.read_f64()?,
        );
        let scale_x = self.read_f64()?;
        let scale_y = self.read_f64()?;
        let scale_z = self.read_f64()?;
        let rotation = self.read_f64()?;
        let _column_count = self.read_u16()?;
        let _row_count = self.read_u16()?;
        let _column_spacing = self.read_f64()?;
        let _row_spacing = self.read_f64()?;
        
        let mut entity = Entity::new(
            EntityType::BlockRef,
            EntityGeometry::BlockRef {
                block_name: block_name.clone(),
                position: insertion_point,
                scale_x,
                scale_y,
                scale_z,
                rotation,
                column_count: _column_count as u32,
                row_count: _row_count as u32,
                column_spacing: _column_spacing,
                row_spacing: _row_spacing,
            },
        );
        entity.set_property("block_name".to_string(), block_name);
        entity.set_property("scale_x".to_string(), scale_x.to_string());
        entity.set_property("scale_y".to_string(), scale_y.to_string());
        entity.set_property("rotation".to_string(), rotation.to_string());
        
        Ok(Some(entity))
    }

    fn parse_polyline_entity(&mut self) -> Result<Option<Entity>, DWGError> {
        let flags = self.read_u8()?;
        let _curve_type = self.read_u16()?;
        let start_width = self.read_f64()?;
        let end_width = self.read_f64()?;
        let _m_count = self.read_u16()?;
        let _n_count = self.read_u16()?;
        let _m_count_smooth = self.read_u16()?;
        let _n_count_smooth = self.read_u16()?;
        let _total_points = self.read_u32()?;
        
        let mut vertices = Vec::new();
        
        if flags & 0x08 != 0 {
            let num_vertices = self.read_u16()?;
            for _ in 0..num_vertices {
                let x = self.read_f64()?;
                let y = self.read_f64()?;
                let z = self.read_f64()?;
                vertices.push(Point::new(x, y, z));
            }
        }
        
        let polyline = Polyline::from_points(&vertices);
        Ok(Some(Entity::new(
            EntityType::Polyline,
            EntityGeometry::Polyline(polyline),
        )))
    }

    fn parse_lwpolyline_entity(&mut self) -> Result<Option<Entity>, DWGError> {
        let num_vertices = self.read_u32()?;
        let _flags = self.read_u32()?;
        let _constant_width = self.read_f64()?;
        let _elevation = self.read_f64()?;
        let _thickness = self.read_f64()?;
        let _extrusion = self.read_f64()?;
        
        let mut vertices = Vec::new();
        
        for _ in 0..num_vertices {
            let x = self.read_f64()?;
            let y = self.read_f64()?;
            let _start_width = self.read_f64()?;
            let _end_width = self.read_f64()?;
            let _bulge = self.read_f64()?;
            vertices.push(Point::new(x, y, 0.0));
        }
        
        let polyline = Polyline::from_points(&vertices);
        Ok(Some(Entity::new(
            EntityType::Polyline,
            EntityGeometry::Polyline(polyline),
        )))
    }

    fn parse_spline_entity(&mut self) -> Result<Option<Entity>, DWGError> {
        let degree = self.read_u32()?;
        let num_knots = self.read_u32()?;
        let num_control_points = self.read_u32()?;
        let _num_fit_points = self.read_u32()?;
        let _knot_tolerance = self.read_f64()?;
        let _control_point_tolerance = self.read_f64()?;
        let _fit_point_tolerance = self.read_f64()?;
        
        let mut knots = Vec::new();
        for _ in 0..num_knots {
            knots.push(self.read_f64()?);
        }
        
        let mut control_points = Vec::new();
        for _ in 0..num_control_points {
            let x = self.read_f64()?;
            let y = self.read_f64()?;
            let z = self.read_f64()?;
            control_points.push(Point::new(x, y, z));
        }
        
        let spline = BSpline::new(control_points, knots, degree as usize);
        Ok(Some(Entity::new(
            EntityType::BSpline,
            EntityGeometry::BSpline(spline),
        )))
    }

    fn parse_hatch_entity(&mut self) -> Result<Option<Entity>, DWGError> {
        let _is_associative = self.read_u32()?;
        let _boundary_paths_count = self.read_u32()?;
        let _style = self.read_u32()?;
        let _pattern_scale = self.read_f64()?;
        let _double_text_flag = self.read_u32()?;
        let _dash_distances_count = self.read_u32()?;
        let _outlier_percent = self.read_f64()?;
        let _outlier_radius = self.read_f64()?;
        let _hatch_color = self.read_u32()?;
        let _background_color = self.read_u32()?;
        let _pattern_angle = self.read_f64()?;
        let _pattern_spacing = self.read_f64()?;
        let _pattern_definition_lines = self.read_u32()?;
        
        let _fill_color = self.read_u32()?;
        
        Ok(None)
    }

    fn parse_objects_section(&mut self) -> Result<(), DWGError> {
        let section_size = self.read_u32()? as usize;
        let _section_start = self.cursor;
        self.cursor += section_size;
        Ok(())
    }

    fn parse(&mut self) -> Result<Document, DWGError> {
        self.parse_version()?;
        
        match self.version {
            DWGVersion::R12 => self.parse_r12_format()?,
            DWGVersion::R13 | DWGVersion::R14 => self.parse_r13_r14_format()?,
            _ => self.parse_modern_format()?,
        }
        
        let mut doc = Document::new(format!("DWG Document ({:?})", self.version));
        
        for (layer_name, _layer_id) in &self.layers {
            let layer = Layer::new(layer_name.clone());
            doc.add_layer(layer);
        }
        
        let mut main_block = Block::new("Model".to_string());
        for entity in &self.entities {
            main_block.add_entity(entity.clone());
        }
        doc.add_block(main_block);
        
        for block in &self.blocks {
            doc.add_block(block.clone());
        }
        
        Ok(doc)
    }

    fn parse_r12_format(&mut self) -> Result<(), DWGError> {
        let _file_maintenance = self.read_u8()?;
        let _version = self.read_u8()?;
        let _app_version = self.read_u16()?;
        let _maint_version = self.read_u8()?;
        let _drawings_unit = self.read_u8()?;
        
        self.parse_entities_section()?;
        Ok(())
    }

    fn parse_r13_r14_format(&mut self) -> Result<(), DWGError> {
        self.parse_tables_section()?;
        self.parse_blocks_section()?;
        self.parse_entities_section()?;
        Ok(())
    }

    fn parse_modern_format(&mut self) -> Result<(), DWGError> {
        let _machine_state = self.read_u32()?;
        
        self.parse_header_section()?;
        self.parse_classes_section()?;
        self.parse_tables_section()?;
        self.parse_blocks_section()?;
        self.parse_entities_section()?;
        self.parse_objects_section()?;
        Ok(())
    }
}

pub struct DWGImporter;

impl DWGImporter {
    pub fn new() -> Self {
        Self
    }

    pub fn detect_version(data: &[u8]) -> Option<DWGVersion> {
        if data.len() < 6 {
            return None;
        }
        DWGVersion::from_magic(&data[0..6])
    }
}

impl crate::io::Importer for DWGImporter {
    fn can_import(&self, extension: &str) -> bool {
        extension.to_lowercase() == "dwg"
    }

    fn import_from_file(&self, filename: &str) -> Result<Document, ImportError> {
        let mut file = std::fs::File::open(filename).map_err(|e| ImportError::Io(e.to_string()))?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).map_err(|e| ImportError::Io(e.to_string()))?;
        self.import_from_bytes(&data, "dwg")
    }

    fn import_from_bytes(&self, data: &[u8], extension: &str) -> Result<Document, ImportError> {
        if !self.can_import(extension) {
            return Err(ImportError::UnsupportedFormat(extension.to_string()));
        }

        if data.len() < 6 {
            return Err(ImportError::ParseError("File too small to be a valid DWG".to_string()));
        }

        let version = DWGImporter::detect_version(data)
            .ok_or_else(|| ImportError::ParseError("Unable to detect DWG version".to_string()))?;

        let mut parser = DWGParser::new(data);
        parser.version = version;
        
        let doc = parser.parse()
            .map_err(|e| ImportError::ParseError(e.to_string()))?;
        
        Ok(doc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::Importer;

    #[test]
    fn test_dwg_version_detection() {
        let r12_data = b"AC1009";
        let version = DWGVersion::from_magic(r12_data);
        assert_eq!(version, Some(DWGVersion::R12));
        
        let r2000_data = b"AC1015";
        let version = DWGVersion::from_magic(r2000_data);
        assert_eq!(version, Some(DWGVersion::R2000));
        
        let r2018_data = b"AC1032";
        let version = DWGVersion::from_magic(r2018_data);
        assert_eq!(version, Some(DWGVersion::R2018));
    }

    #[test]
    fn test_dwg_importer_can_import() {
        let importer = DWGImporter::new();
        assert!(importer.can_import("dwg"));
        assert!(importer.can_import("DWG"));
        assert!(!importer.can_import("dxf"));
    }
}
