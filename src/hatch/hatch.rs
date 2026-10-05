use crate::geometry::extended_geometry::Point;
use std::f64::consts::PI;

#[derive(Debug, Clone)]
pub struct Hatch {
    pub pattern: Option<HatchPattern>,
    pub gradient: Option<GradientFill>,
    pub boundaries: Vec<BoundaryPath>,
    pub associativity: bool,
    pub origin: Point,
    pub rotation: f64,
    pub scale: f64,
    pub is_solid_fill: bool,
    pub transparency: f64,
}

impl Hatch {
    pub fn new() -> Self {
        Self {
            pattern: None,
            gradient: None,
            boundaries: Vec::new(),
            associativity: false,
            origin: Point::new(0.0, 0.0),
            rotation: 0.0,
            scale: 1.0,
            is_solid_fill: false,
            transparency: 0.0,
        }
    }

    pub fn with_pattern(mut self, pattern: HatchPattern) -> Self {
        self.pattern = Some(pattern);
        self.is_solid_fill = false;
        self
    }

    pub fn with_gradient(mut self, gradient: GradientFill) -> Self {
        self.gradient = Some(gradient);
        self.is_solid_fill = false;
        self
    }

    pub fn as_solid_fill(mut self) -> Self {
        self.is_solid_fill = true;
        self.pattern = None;
        self.gradient = None;
        self
    }

    pub fn with_boundary(mut self, boundary: BoundaryPath) -> Self {
        self.boundaries.push(boundary);
        self
    }

    pub fn with_boundaries(mut self, boundaries: Vec<BoundaryPath>) -> Self {
        self.boundaries.extend(boundaries);
        self
    }

    pub fn with_associativity(mut self, associative: bool) -> Self {
        self.associativity = associative;
        self
    }

    pub fn with_origin(mut self, origin: Point) -> Self {
        self.origin = origin;
        self
    }

    pub fn with_rotation(mut self, rotation: f64) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_scale(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_transparency(mut self, transparency: f64) -> Self {
        self.transparency = transparency.clamp(0.0, 1.0);
        self
    }

    pub fn calculate_coverage(&self) -> f64 {
        let mut total_area = 0.0;
        let mut holes_area = 0.0;

        for boundary in &self.boundaries {
            match boundary.path_type {
                BoundaryPathType::External => {
                    total_area += boundary.calculate_area();
                }
                BoundaryPathType::Hole => {
                    holes_area += boundary.calculate_area();
                }
                BoundaryPathType::TextBox => {}
                BoundaryPathType::CommentBox => {}
            }
        }

        (total_area - holes_area).max(0.0)
    }

    pub fn is_point_inside(&self, point: Point) -> bool {
        let mut inside = false;
        
        for boundary in &self.boundaries {
            if boundary.path_type == BoundaryPathType::External {
                inside = boundary.is_point_inside(point);
                break;
            }
        }

        if inside {
            for boundary in &self.boundaries {
                if boundary.path_type == BoundaryPathType::Hole {
                    if boundary.is_point_inside(point) {
                        return false;
                    }
                }
            }
        }

        inside
    }
}

impl Default for Hatch {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct HatchPattern {
    pub name: String,
    pub description: String,
    pub pattern_type: HatchPatternType,
    pub angle: f64,
    pub scale: f64,
    pub double: bool,
    pub lines: Vec<HatchLine>,
    pub dots: Vec<HatchDot>,
    pub origin: Point,
}

impl HatchPattern {
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            pattern_type: HatchPatternType::UserDefined,
            angle: 0.0,
            scale: 1.0,
            double: false,
            lines: Vec::new(),
            dots: Vec::new(),
            origin: Point::new(0.0, 0.0),
        }
    }

    pub fn with_angle(&mut self, angle: f64) -> &mut Self {
        self.angle = angle;
        self
    }

    pub fn with_scale(&mut self, scale: f64) -> &mut Self {
        self.scale = scale;
        self
    }

    pub fn as_double(&mut self) -> &mut Self {
        self.double = true;
        self
    }

    pub fn add_line(&mut self, line: HatchLine) -> &mut Self {
        self.lines.push(line);
        self
    }

    pub fn add_dot(&mut self, dot: HatchDot) -> &mut Self {
        self.dots.push(dot);
        self
    }

    pub fn with_origin(&mut self, origin: Point) -> &mut Self {
        self.origin = origin;
        self
    }

    pub fn add_line_mut(&mut self, line: HatchLine) {
        self.lines.push(line);
    }

    pub fn add_dot_mut(&mut self, dot: HatchDot) {
        self.dots.push(dot);
    }

    pub fn as_double_mut(&mut self) {
        self.double = true;
    }

    pub fn as_single_mut(&mut self) {
        self.double = false;
    }

    pub fn generate_pattern_lines(&self, bounding_box: BoundingBox) -> Vec<PatternLine> {
        let mut pattern_lines = Vec::new();

        for hatch_line in &self.lines {
            let step = hatch_line.spacing * self.scale;
            let rotated_angle = self.angle + hatch_line.angle;

                let num_lines = ((bounding_box.width() + bounding_box.height()) / step) as usize + 2;

                for i in -1i64..=(num_lines as i64) {
                    let offset = i as f64 * step;
                    let pattern_line = PatternLine {
                        start: Point::new(
                            bounding_box.min_x - step,
                            bounding_box.min_y + offset,
                        ),
                        end: Point::new(
                            bounding_box.max_x + step,
                            bounding_box.min_y + offset,
                        ),
                        line_type: hatch_line.line_type.clone(),
                        line_weight: hatch_line.line_weight,
                    };
                    pattern_lines.push(pattern_line);
                }
        }

        if self.double {
            for hatch_line in &self.lines {
                let step = hatch_line.spacing * self.scale;
                let rotated_angle = self.angle + hatch_line.angle + PI / 2.0;

                let num_lines = ((bounding_box.width() + bounding_box.height()) / step) as usize + 2;

                for i in -1i64..=(num_lines as i64) {
                    let offset = i as f64 * step;
                    let pattern_line = PatternLine {
                        start: Point::new(
                            bounding_box.min_x + offset,
                            bounding_box.min_y - step,
                        ),
                        end: Point::new(
                            bounding_box.min_x + offset,
                            bounding_box.max_y + step,
                        ),
                        line_type: hatch_line.line_type.clone(),
                        line_weight: hatch_line.line_weight,
                    };
                    pattern_lines.push(pattern_line);
                }
            }
        }

        pattern_lines
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HatchPatternType {
    UserDefined,
    Predefined,
    Custom,
}

#[derive(Debug, Clone)]
pub struct HatchLine {
    pub angle: f64,
    pub spacing: f64,
    pub line_type: LineType,
    pub line_weight: f64,
}

impl HatchLine {
    pub fn new(angle: f64, spacing: f64) -> Self {
        Self {
            angle,
            spacing,
            line_type: LineType::Solid,
            line_weight: 0.25,
        }
    }

    pub fn with_line_type(mut self, line_type: LineType) -> Self {
        self.line_type = line_type;
        self
    }

    pub fn with_line_weight(mut self, weight: f64) -> Self {
        self.line_weight = weight;
        self
    }
}

#[derive(Debug, Clone)]
pub struct HatchDot {
    pub position: Point,
    pub spacing: f64,
    pub dot_type: DotType,
    pub size: f64,
}

impl HatchDot {
    pub fn new(position: Point, spacing: f64) -> Self {
        Self {
            position,
            spacing,
            dot_type: DotType::Circle,
            size: 0.5,
        }
    }

    pub fn with_dot_type(mut self, dot_type: DotType) -> Self {
        self.dot_type = dot_type;
        self
    }

    pub fn with_size(mut self, size: f64) -> Self {
        self.size = size;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LineType {
    Solid,
    Dashed,
    Dotted,
    DashDot,
    LongDash,
    ShortDash,
    SparseDots,
    UserDefined(Vec<f64>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DotType {
    Circle,
    Square,
    Diamond,
    Cross,
}

#[derive(Debug, Clone)]
pub struct PatternLine {
    pub start: Point,
    pub end: Point,
    pub line_type: LineType,
    pub line_weight: f64,
}

#[derive(Debug, Clone)]
pub struct GradientFill {
    pub gradient_type: GradientType,
    pub color1: Color,
    pub color2: Color,
    pub angle: f64,
    pub shift: f64,
    pub tint: f64,
    pub is_one_color: bool,
}

impl GradientFill {
    pub fn new(gradient_type: GradientType, color1: Color, color2: Color) -> Self {
        Self {
            gradient_type,
            color1,
            color2,
            angle: 0.0,
            shift: 0.0,
            tint: 1.0,
            is_one_color: false,
        }
    }

    pub fn with_angle(mut self, angle: f64) -> Self {
        self.angle = angle;
        self
    }

    pub fn with_shift(mut self, shift: f64) -> Self {
        self.shift = shift;
        self
    }

    pub fn with_tint(mut self, tint: f64) -> Self {
        self.tint = tint;
        self
    }

    pub fn as_one_color(mut self) -> Self {
        self.is_one_color = true;
        self
    }

    pub fn color_at_position(&self, position: Point, bounding_box: BoundingBox) -> Color {
        let normalized_x = (position.x - bounding_box.min_x) / bounding_box.width();
        let normalized_y = (position.y - bounding_box.min_y) / bounding_box.height();

        let gradient_pos = match self.gradient_type {
            GradientType::Linear => {
                let rotated_x = normalized_x * self.angle.cos() + normalized_y * self.angle.sin();
                rotated_x.clamp(0.0, 1.0)
            }
            GradientType::Radial => {
                let center_x = 0.5;
                let center_y = 0.5;
                let dx = normalized_x - center_x;
                let dy = normalized_y - center_y;
                (dx * dx + dy * dy).sqrt() * 2.0
            }
            GradientType::Swirl => {
                let angle = (normalized_y - 0.5).atan2(normalized_x - 0.5);
                let radius = (normalized_x - 0.5).powi(2) + (normalized_y - 0.5).powi(2);
                (angle / PI + radius).fract().abs()
            }
            GradientType::Spherical => {
                let dx = normalized_x - 0.5;
                let dy = normalized_y - 0.5;
                let dz = (0.25 - dx * dx - dy * dy).sqrt();
                (dz + 0.5).clamp(0.0, 1.0)
            }
            GradientType::Hemispherical => {
                let dx = normalized_x - 0.5;
                let dy = normalized_y - 0.5;
                (dx * dx + dy * dy).sqrt() * 2.0
            }
            GradientType::Curved => {
                let t = normalized_x;
                t * t * (3.0 - 2.0 * t)
            }
        };

        let t = (gradient_pos + self.shift).fract().clamp(0.0, 1.0);

        Color {
            r: self.color1.r + (self.color2.r - self.color1.r) * t,
            g: self.color1.g + (self.color2.g - self.color1.g) * t,
            b: self.color1.b + (self.color2.b - self.color1.b) * t,
            a: self.color1.a + (self.color2.a - self.color1.a) * t,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GradientType {
    Linear,
    Radial,
    Swirl,
    Spherical,
    Hemispherical,
    Curved,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
            a: 1.0,
        }
    }

    pub fn with_alpha(mut self, alpha: f64) -> Self {
        self.a = alpha.clamp(0.0, 1.0);
        self
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0)
    }

    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }

    pub fn hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f64 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f64 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f64 / 255.0;
        Self::new(r, g, b)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::new(0.5, 0.5, 0.5)
    }
}

#[derive(Debug, Clone)]
pub struct BoundaryPath {
    pub path_type: BoundaryPathType,
    pub geometry: BoundaryGeometry,
    pub is_closed: bool,
}

impl BoundaryPath {
    pub fn new(path_type: BoundaryPathType, geometry: BoundaryGeometry) -> Self {
        Self {
            path_type,
            geometry,
            is_closed: true,
        }
    }

    pub fn calculate_area(&self) -> f64 {
        match &self.geometry {
            BoundaryGeometry::LineSequence(lines) => {
                let mut area = 0.0;
                let n = lines.len();
                for i in 0..n {
                    let (x1, y1) = if i == 0 {
                        (lines[n-1].start.x, lines[n-1].start.y)
                    } else {
                        (lines[i-1].end.x, lines[i-1].end.y)
                    };
                    let (x2, y2) = (lines[i].end.x, lines[i].end.y);
                    area += (x1 * y2 - x2 * y1);
                }
                area.abs() / 2.0
            }
            BoundaryGeometry::ArcSequence(arcs) => {
                let mut area = 0.0;
                for arc in arcs {
                    let sector_area = 0.5 * arc.radius.powi(2) * 
                        (arc.end_angle - arc.start_angle).sin();
                    area += sector_area;
                }
                area.abs()
            }
            BoundaryGeometry::Spline(points) => {
                let mut area = 0.0;
                let n = points.len();
                for i in 0..n {
                    let j = (i + 1) % n;
                    area += points[i].x * points[j].y - points[j].x * points[i].y;
                }
                area.abs() / 2.0
            }
        }
    }

    pub fn is_point_inside(&self, point: Point) -> bool {
        match &self.geometry {
            BoundaryGeometry::LineSequence(lines) => {
                let mut inside = false;
                let n = lines.len();
                for i in 0..n {
                    let p1 = if i == 0 { lines[n-1].end } else { lines[i-1].end };
                    let p2 = lines[i].start;
                    
                    if ((p1.y > point.y) != (p2.y > point.y)) &&
                        (point.x < (p2.x - p1.x) * (point.y - p1.y) / (p2.y - p1.y) + p1.x) {
                        inside = !inside;
                    }
                }
                inside
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundaryPathType {
    External,
    Hole,
    TextBox,
    CommentBox,
}

#[derive(Debug, Clone)]
pub enum BoundaryGeometry {
    LineSequence(Vec<BoundaryLine>),
    ArcSequence(Vec<BoundaryArc>),
    Spline(Vec<Point>),
}

#[derive(Debug, Clone)]
pub struct BoundaryLine {
    pub start: Point,
    pub end: Point,
}

impl BoundaryLine {
    pub fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone)]
pub struct BoundaryArc {
    pub center: Point,
    pub radius: f64,
    pub start_angle: f64,
    pub end_angle: f64,
    pub is_counter_clockwise: bool,
}

impl BoundaryArc {
    pub fn new(
        center: Point,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
    ) -> Self {
        let sweep = if end_angle >= start_angle {
            end_angle - start_angle
        } else {
            end_angle + 2.0 * PI - start_angle
        };
        
        Self {
            center,
            radius,
            start_angle,
            end_angle,
            is_counter_clockwise: sweep <= PI,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl BoundingBox {
    pub fn new(points: Vec<Point>) -> Self {
        if points.is_empty() {
            return Self {
                min_x: 0.0,
                min_y: 0.0,
                max_x: 0.0,
                max_y: 0.0,
            };
        }

        let mut min_x = points[0].x;
        let mut min_y = points[0].y;
        let mut max_x = points[0].x;
        let mut max_y = points[0].y;

        for point in points.iter().skip(1) {
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
        }

        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    pub fn center(&self) -> Point {
        Point::new(
            (self.min_x + self.max_x) / 2.0,
            (self.min_y + self.max_y) / 2.0,
        )
    }
}

pub struct PatternLibrary;

impl PatternLibrary {
    pub fn get_pattern(&self, name: &str) -> Option<HatchPattern> {
        match name {
            "ANSI31" => Some(Self::create_ansi31()),
            "ANSI32" => Some(Self::create_ansi32()),
            "ANSI33" => Some(Self::create_ansi33()),
            "ANSI34" => Some(Self::create_ansi34()),
            "ANSI35" => Some(Self::create_ansi35()),
            "ANSI36" => Some(Self::create_ansi36()),
            "ANSI37" => Some(Self::create_ansi37()),
            "ANSI38" => Some(Self::create_ansi38()),
            "BRICK" => Some(Self::create_brick()),
            "CROSS" => Some(Self::create_cross()),
            "DASH" => Some(Self::create_dash()),
            "DIAGONAL" => Some(Self::create_diagonal()),
            "GRID" => Some(Self::create_grid()),
            "HOUND" => Some(Self::create_hound()),
            "ISO01" => Some(Self::create_iso01()),
            "ISO02" => Some(Self::create_iso02()),
            "ISO03" => Some(Self::create_iso03()),
            "ISO04" => Some(Self::create_iso04()),
            "ISO05" => Some(Self::create_iso05()),
            "ISO06" => Some(Self::create_iso06()),
            "ISO07" => Some(Self::create_iso07()),
            "ISO08" => Some(Self::create_iso08()),
            "ISO09" => Some(Self::create_iso09()),
            "ISO10" => Some(Self::create_iso10()),
            "PLASTIC" => Some(Self::create_plastic()),
            "STEEL" => Some(Self::create_steel()),
            "ZIGZAG" => Some(Self::create_zigzag()),
            _ => None,
        }
    }

    pub fn create_ansi31() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ANSI31".to_string(),
            "ANSI Iron, Brick, and Masonry".to_string(),
        );
        pattern.add_line(HatchLine::new(45.0, 3.0));
        pattern
    }

    pub fn create_ansi32() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ANSI32".to_string(),
            "ANSI Steel".to_string(),
        );
        pattern.add_line(HatchLine::new(45.0, 8.0));
        pattern.add_line(HatchLine::new(-45.0, 8.0));
        pattern
    }

    pub fn create_ansi33() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ANSI33".to_string(),
            "ANSI Bronze, Brass, Copper".to_string(),
        );
        pattern.add_line(HatchLine::new(45.0, 4.0));
        pattern.add_line(HatchLine::new(135.0, 4.0));
        pattern.as_double();
        pattern
    }

    pub fn create_ansi34() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ANSI34".to_string(),
            "ANSI Plastics".to_string(),
        );
        pattern.add_line(HatchLine::new(45.0, 6.0));
        pattern.add_line(HatchLine::new(135.0, 6.0));
        pattern.as_double();
        pattern
    }

    pub fn create_ansi35() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ANSI35".to_string(),
            "ANSI Hard ROCK".to_string(),
        );
        pattern.add_line(HatchLine::new(45.0, 5.0));
        pattern.add_line(HatchLine::new(135.0, 5.0));
        pattern.as_double();
        pattern
    }

    pub fn create_ansi36() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ANSI36".to_string(),
            "ANSI Earth".to_string(),
        );
        pattern.add_line(HatchLine::new(45.0, 12.0));
        pattern.add_line(HatchLine::new(135.0, 12.0));
        pattern.as_double();
        pattern
    }

    pub fn create_ansi37() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ANSI37".to_string(),
            "ANSI Concrete".to_string(),
        );
        pattern.add_line(HatchLine::new(45.0, 10.0));
        pattern.add_line(HatchLine::new(135.0, 10.0));
        pattern.as_double();
        pattern
    }

    pub fn create_ansi38() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ANSI38".to_string(),
            "ANSI Lead, Zinc, Magnesium, Aluminum".to_string(),
        );
        pattern.add_line(HatchLine::new(45.0, 2.0));
        pattern.add_line(HatchLine::new(135.0, 2.0));
        pattern.as_double();
        pattern
    }

    pub fn create_brick() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "BRICK".to_string(),
            "Brick pattern".to_string(),
        );
        pattern.add_line(HatchLine::new(0.0, 4.0));
        pattern.add_line(HatchLine::new(90.0, 2.5));
        pattern
    }

    pub fn create_cross() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "CROSS".to_string(),
            "Crosshatch pattern".to_string(),
        );
        pattern.add_line(HatchLine::new(0.0, 5.0));
        pattern.add_line(HatchLine::new(90.0, 5.0));
        pattern
    }

    pub fn create_dash() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "DASH".to_string(),
            "Dash pattern".to_string(),
        );
        pattern.add_line(HatchLine::new(45.0, 3.0).with_line_type(LineType::Dashed));
        pattern
    }

    pub fn create_diagonal() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "DIAGONAL".to_string(),
            "Diagonal lines".to_string(),
        );
        pattern.add_line(HatchLine::new(45.0, 4.0));
        pattern
    }

    pub fn create_grid() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "GRID".to_string(),
            "Grid pattern".to_string(),
        );
        pattern.add_line(HatchLine::new(0.0, 5.0));
        pattern.add_line(HatchLine::new(90.0, 5.0));
        pattern
    }

    pub fn create_hound() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "HOUND".to_string(),
            "Houndstooth pattern".to_string(),
        );
        pattern.add_line(HatchLine::new(45.0, 8.0));
        pattern.add_line(HatchLine::new(0.0, 8.0));
        pattern
    }

    pub fn create_iso01() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ISO01".to_string(),
            "ISO Light".to_string(),
        );
        pattern.add_line(HatchLine::new(45.0, 3.0));
        pattern
    }

    pub fn create_iso02() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ISO02".to_string(),
            "ISO Medium".to_string(),
        );
        pattern.add_line(HatchLine::new(45.0, 2.0));
        pattern
    }

    pub fn create_iso03() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ISO03".to_string(),
            "ISO Dense".to_string(),
        );
        pattern.add_line(HatchLine::new(45.0, 1.0));
        pattern
    }

    pub fn create_iso04() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ISO04".to_string(),
            "ISO Light double".to_string(),
        );
        pattern.add_line_mut(HatchLine::new(45.0, 3.0));
        pattern.add_line_mut(HatchLine::new(135.0, 3.0));
        pattern.as_double_mut();
        pattern
    }

    pub fn create_iso05() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ISO05".to_string(),
            "ISO Medium double".to_string(),
        );
        pattern.add_line_mut(HatchLine::new(45.0, 2.0));
        pattern.add_line_mut(HatchLine::new(135.0, 2.0));
        pattern.as_double_mut();
        pattern
    }

    pub fn create_iso06() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ISO06".to_string(),
            "ISO Dense double".to_string(),
        );
        pattern.add_line_mut(HatchLine::new(45.0, 1.0));
        pattern.add_line_mut(HatchLine::new(135.0, 1.0));
        pattern.as_double_mut();
        pattern
    }

    pub fn create_iso07() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ISO07".to_string(),
            "ISO Swirl".to_string(),
        );
        pattern.add_line_mut(HatchLine::new(30.0, 2.0));
        pattern.add_line_mut(HatchLine::new(150.0, 2.0));
        pattern
    }

    pub fn create_iso08() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ISO08".to_string(),
            "ISO Wave".to_string(),
        );
        pattern.add_line_mut(HatchLine::new(45.0, 3.0));
        pattern.add_line_mut(HatchLine::new(45.0, 1.0).with_line_type(LineType::Dashed));
        pattern
    }

    pub fn create_iso09() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ISO09".to_string(),
            "ISO Cobblestone".to_string(),
        );
        pattern.add_line_mut(HatchLine::new(0.0, 3.0));
        pattern.add_line_mut(HatchLine::new(90.0, 3.0));
        pattern.add_line_mut(HatchLine::new(45.0, 3.0));
        pattern.add_line_mut(HatchLine::new(135.0, 3.0));
        pattern
    }

    pub fn create_iso10() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ISO10".to_string(),
            "ISO Weave".to_string(),
        );
        pattern.add_line_mut(HatchLine::new(0.0, 4.0));
        pattern.add_line_mut(HatchLine::new(90.0, 4.0));
        pattern.add_line_mut(HatchLine::new(45.0, 4.0));
        pattern.add_line_mut(HatchLine::new(135.0, 4.0));
        pattern
    }

    pub fn create_plastic() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "PLASTIC".to_string(),
            "Plastic pattern".to_string(),
        );
        pattern.add_line_mut(HatchLine::new(45.0, 2.5));
        pattern.add_line_mut(HatchLine::new(135.0, 2.5));
        pattern.as_double_mut();
        pattern
    }

    pub fn create_steel() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "STEEL".to_string(),
            "Steel pattern".to_string(),
        );
        pattern.add_line_mut(HatchLine::new(45.0, 6.0));
        pattern.add_line_mut(HatchLine::new(135.0, 6.0));
        pattern.as_double_mut();
        pattern
    }

    pub fn create_zigzag() -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ZIGZAG".to_string(),
            "Zigzag pattern".to_string(),
        );
        pattern.add_line_mut(HatchLine::new(0.0, 3.0));
        pattern.add_line_mut(HatchLine::new(90.0, 3.0));
        pattern
    }

    pub fn available_patterns(&self) -> Vec<&'static str> {
        vec![
            "ANSI31", "ANSI32", "ANSI33", "ANSI34", "ANSI35",
            "ANSI36", "ANSI37", "ANSI38", "BRICK", "CROSS",
            "DASH", "DIAGONAL", "GRID", "HOUND", "ISO01",
            "ISO02", "ISO03", "ISO04", "ISO05", "ISO06",
            "ISO07", "ISO08", "ISO09", "ISO10", "PLASTIC",
            "STEEL", "ZIGZAG",
        ]
    }
}
