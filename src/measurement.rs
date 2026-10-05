use crate::geometry::{Point, Circle, Arc, Curve};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasurementResult {
    pub value: f64,
    pub unit: MeasurementUnit,
    pub display: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MeasurementUnit {
    Millimeters,
    Centimeters,
    Meters,
    Inches,
    Feet,
    Degrees,
    Radians,
}

impl MeasurementUnit {
    pub fn abbreviation(&self) -> &str {
        match self {
            MeasurementUnit::Millimeters => "mm",
            MeasurementUnit::Centimeters => "cm",
            MeasurementUnit::Meters => "m",
            MeasurementUnit::Inches => "\"",
            MeasurementUnit::Feet => "'",
            MeasurementUnit::Degrees => "°",
            MeasurementUnit::Radians => "rad",
        }
    }
    
    pub fn conversion_factor(&self) -> f64 {
        match self {
            MeasurementUnit::Millimeters => 1.0,
            MeasurementUnit::Centimeters => 10.0,
            MeasurementUnit::Meters => 1000.0,
            MeasurementUnit::Inches => 25.4,
            MeasurementUnit::Feet => 304.8,
            MeasurementUnit::Degrees => 1.0,
            MeasurementUnit::Radians => 1.0,
        }
    }
}

pub trait MeasurementTool {
    fn measure(&self) -> MeasurementResult;
    fn measure_with_precision(&self, decimals: u32) -> MeasurementResult;
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DistanceMeasurement {
    pub start_point: Point,
    pub end_point: Point,
    pub distance: f64,
    pub delta_x: f64,
    pub delta_y: f64,
    pub angle: f64,
    pub unit: MeasurementUnit,
}

impl DistanceMeasurement {
    pub fn new(start: Point, end: Point, unit: MeasurementUnit) -> Self {
        let delta = end - start;
        let distance = (delta.x.powi(2) + delta.y.powi(2)).sqrt();
        let angle = delta.to_vector2().angle();
        
        Self {
            start_point: start,
            end_point: end,
            distance,
            delta_x: delta.x,
            delta_y: delta.y,
            angle,
            unit,
        }
    }
    
    pub fn distance_in_unit(&self, unit: MeasurementUnit) -> f64 {
        let base_distance = self.distance * self.unit.conversion_factor();
        base_distance / unit.conversion_factor()
    }
    
    pub fn format(&self, decimals: u32) -> String {
        format!(
            "Distance: {:.4$} | Δx: {:.4$} | Δy: {:.4$} | Angle: {:.4$}°",
            self.distance,
            self.delta_x,
            self.delta_y,
            self.angle,
            decimals as usize
        )
    }
}

impl MeasurementTool for DistanceMeasurement {
    fn measure(&self) -> MeasurementResult {
        MeasurementResult {
            value: self.distance,
            unit: self.unit,
            display: format!("{:.3} {}", self.distance, self.unit.abbreviation()),
        }
    }
    
    fn measure_with_precision(&self, decimals: u32) -> MeasurementResult {
        MeasurementResult {
            value: self.distance,
            unit: self.unit,
            display: format!("{:.prec$} {}", self.distance, self.unit.abbreviation(), prec = decimals as usize),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AngleMeasurement {
    pub vertex: Point,
    pub start_point: Point,
    pub end_point: Point,
    pub angle: f64,
    pub angle_rad: f64,
    pub is_reflex: bool,
    pub unit: MeasurementUnit,
}

impl AngleMeasurement {
    pub fn new(vertex: Point, p1: Point, p2: Point, unit: MeasurementUnit) -> Self {
        let v1 = (p1 - vertex).to_vector2();
        let v2 = (p2 - vertex).to_vector2();
        
        let dot = v1.x * v2.x + v1.y * v2.y;
        let cross = v1.x * v2.y - v1.y * v2.x;
        let angle_rad = cross.atan2(dot);
        
        let angle_deg = angle_rad.to_degrees();
        let normalized_angle = if angle_deg < 0.0 { angle_deg + 360.0 } else { angle_deg };
        let is_reflex = normalized_angle > 180.0;
        
        Self {
            vertex,
            start_point: p1,
            end_point: p2,
            angle: normalized_angle,
            angle_rad,
            is_reflex,
            unit,
        }
    }
    
    pub fn get_smallest_angle(&self) -> f64 {
        self.angle.min(360.0 - self.angle)
    }
    
    pub fn angle_in_unit(&self, unit: MeasurementUnit) -> f64 {
        if unit == MeasurementUnit::Radians {
            self.angle_rad.abs()
        } else {
            self.angle
        }
    }
    
    pub fn format(&self, decimals: u32) -> String {
        if self.is_reflex {
            format!(
                "Angle: {:.2$}° (reflex: {:.2$}°)",
                self.angle,
                360.0 - self.angle,
                decimals as usize
            )
        } else {
            format!("Angle: {:.1$}°", self.angle, decimals as usize)
        }
    }
}

impl MeasurementTool for AngleMeasurement {
    fn measure(&self) -> MeasurementResult {
        MeasurementResult {
            value: self.angle,
            unit: self.unit,
            display: format!("{:.2}°", self.angle),
        }
    }
    
    fn measure_with_precision(&self, decimals: u32) -> MeasurementResult {
        MeasurementResult {
            value: self.angle,
            unit: self.unit,
            display: format!("{:.1$}°", self.angle, decimals as usize),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AreaMeasurement {
    pub area: f64,
    pub perimeter: f64,
    pub centroid: Point,
    pub unit: MeasurementUnit,
    pub points: Vec<Point>,
}

impl AreaMeasurement {
    pub fn new(points: Vec<Point>, unit: MeasurementUnit) -> Self {
        let area = Self::calculate_area(&points);
        let perimeter = Self::calculate_perimeter(&points);
        let centroid = Self::calculate_centroid(&points);
        
        Self {
            area,
            perimeter,
            centroid,
            unit,
            points,
        }
    }
    
    fn calculate_area(points: &[Point]) -> f64 {
        if points.len() < 3 {
            return 0.0;
        }
        
        let mut area = 0.0;
        let n = points.len();
        
        for i in 0..n {
            let j = (i + 1) % n;
            area += points[i].x * points[j].y;
            area -= points[j].x * points[i].y;
        }
        
        area.abs() / 2.0
    }
    
    fn calculate_perimeter(points: &[Point]) -> f64 {
        let mut perimeter = 0.0;
        
        for i in 0..points.len() {
            let next = (i + 1) % points.len();
            perimeter += points[i].distance_to(&points[next]);
        }
        
        perimeter
    }
    
    fn calculate_centroid(points: &[Point]) -> Point {
        if points.is_empty() {
            return Point::origin();
        }
        
        let cx = points.iter().map(|p| p.x).sum::<f64>() / points.len() as f64;
        let cy = points.iter().map(|p| p.y).sum::<f64>() / points.len() as f64;
        
        Point::new(cx, cy, 0.0)
    }
    
    pub fn area_in_unit(&self, unit: MeasurementUnit) -> f64 {
        let base_area = self.area * self.unit.conversion_factor().powi(2);
        base_area / unit.conversion_factor().powi(2)
    }
    
    pub fn perimeter_in_unit(&self, unit: MeasurementUnit) -> f64 {
        let base_perimeter = self.perimeter * self.unit.conversion_factor();
        base_perimeter / unit.conversion_factor()
    }
    
    pub fn format(&self, decimals: u32) -> String {
        format!(
            "Area: {:.4$} {}² | Perimeter: {:.4$} {}",
            self.area,
            self.unit.abbreviation(),
            self.perimeter,
            self.unit.abbreviation(),
            decimals as usize
        )
    }
}

impl MeasurementTool for AreaMeasurement {
    fn measure(&self) -> MeasurementResult {
        MeasurementResult {
            value: self.area,
            unit: self.unit,
            display: format!("{:.3} {}^2", self.area, self.unit.abbreviation()),
        }
    }
    
    fn measure_with_precision(&self, decimals: u32) -> MeasurementResult {
        MeasurementResult {
            value: self.area,
            unit: self.unit,
            display: format!("{:.prec$} {}^2", self.area, self.unit.abbreviation(), prec = decimals as usize),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadiusMeasurement {
    pub center: Point,
    pub point: Point,
    pub radius: f64,
    pub diameter: f64,
    pub unit: MeasurementUnit,
}

impl RadiusMeasurement {
    pub fn new(center: Point, point: Point, unit: MeasurementUnit) -> Self {
        let radius = center.distance_to(&point);
        
        Self {
            center,
            point,
            radius,
            diameter: radius * 2.0,
            unit,
        }
    }
    
    pub fn from_circle(circle: &Circle, unit: MeasurementUnit) -> Self {
        Self {
            center: circle.center,
            point: Point::new(
                circle.center.x + circle.radius,
                circle.center.y,
                0.0,
            ),
            radius: circle.radius,
            diameter: circle.radius * 2.0,
            unit,
        }
    }
    
    pub fn from_arc(arc: &Arc, unit: MeasurementUnit) -> Self {
        Self {
            center: arc.center,
            point: Point::new(
                arc.center.x + arc.radius,
                arc.center.y,
                0.0,
            ),
            radius: arc.radius,
            diameter: arc.radius * 2.0,
            unit,
        }
    }
    
    pub fn radius_in_unit(&self, unit: MeasurementUnit) -> f64 {
        let base_radius = self.radius * self.unit.conversion_factor();
        base_radius / unit.conversion_factor()
    }
    
    pub fn format(&self, decimals: u32) -> String {
        format!(
            "Radius: {:.2$} | Diameter: {:.2$}",
            self.radius,
            self.diameter,
            decimals as usize
        )
    }
}

impl MeasurementTool for RadiusMeasurement {
    fn measure(&self) -> MeasurementResult {
        MeasurementResult {
            value: self.radius,
            unit: self.unit,
            display: format!("R {:.3} {}", self.radius, self.unit.abbreviation()),
        }
    }
    
    fn measure_with_precision(&self, decimals: u32) -> MeasurementResult {
        MeasurementResult {
            value: self.radius,
            unit: self.unit,
            display: format!("R {:.prec$} {}", self.radius, self.unit.abbreviation(), prec = decimals as usize),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArcLengthMeasurement {
    pub arc: Arc,
    pub arc_length: f64,
    pub chord_length: f64,
    pub sweep_angle: f64,
    pub unit: MeasurementUnit,
}

impl ArcLengthMeasurement {
    pub fn new(arc: Arc, unit: MeasurementUnit) -> Self {
        let arc_length = arc.radius * (arc.end_angle - arc.start_angle).abs();
        
        let start = Point::new(
            arc.center.x + arc.radius * arc.start_angle.cos(),
            arc.center.y + arc.radius * arc.start_angle.sin(),
            0.0,
        );
        let end = Point::new(
            arc.center.x + arc.radius * arc.end_angle.cos(),
            arc.center.y + arc.radius * arc.end_angle.sin(),
            0.0,
        );
        let chord_length = start.distance_to(&end);
        
        let sweep_angle = (arc.end_angle - arc.start_angle).abs().to_degrees();
        
        Self {
            arc,
            arc_length,
            chord_length,
            sweep_angle,
            unit,
        }
    }
    
    pub fn format(&self, decimals: u32) -> String {
        format!(
            "Arc Length: {:.3$} | Chord: {:.3$} | Angle: {:.3$}°",
            self.arc_length,
            self.chord_length,
            self.sweep_angle,
            decimals as usize
        )
    }
}

impl MeasurementTool for ArcLengthMeasurement {
    fn measure(&self) -> MeasurementResult {
        MeasurementResult {
            value: self.arc_length,
            unit: self.unit,
            display: format!("{:.3} {}", self.arc_length, self.unit.abbreviation()),
        }
    }
    
    fn measure_with_precision(&self, decimals: u32) -> MeasurementResult {
        MeasurementResult {
            value: self.arc_length,
            unit: self.unit,
            display: format!("{:.prec$} {}", self.arc_length, self.unit.abbreviation(), prec = decimals as usize),
        }
    }
}

pub struct MeasurementCalculator;

impl MeasurementCalculator {
    pub fn calculate_distance(start: Point, end: Point, unit: MeasurementUnit) -> DistanceMeasurement {
        DistanceMeasurement::new(start, end, unit)
    }
    
    pub fn calculate_angle(vertex: Point, p1: Point, p2: Point, unit: MeasurementUnit) -> AngleMeasurement {
        AngleMeasurement::new(vertex, p1, p2, unit)
    }
    
    pub fn calculate_area_from_points(points: Vec<Point>, unit: MeasurementUnit) -> AreaMeasurement {
        AreaMeasurement::new(points, unit)
    }
    
    pub fn calculate_radius(center: Point, point: Point, unit: MeasurementUnit) -> RadiusMeasurement {
        RadiusMeasurement::new(center, point, unit)
    }
    
    pub fn calculate_circle_measurements(circle: &Circle, unit: MeasurementUnit) -> (RadiusMeasurement, AreaMeasurement) {
        let radius = RadiusMeasurement::from_circle(circle, unit);
        
        let circumference_points = vec![
            Point::new(circle.center.x + circle.radius, circle.center.y, 0.0),
            Point::new(circle.center.x, circle.center.y + circle.radius, 0.0),
            Point::new(circle.center.x - circle.radius, circle.center.y, 0.0),
            Point::new(circle.center.x, circle.center.y - circle.radius, 0.0),
        ];
        let area = AreaMeasurement::new(circumference_points, unit);
        
        (radius, area)
    }
    
    pub fn calculate_curve_length(curve: &dyn Curve, tolerance: f64, unit: MeasurementUnit) -> f64 {
        curve.length(tolerance) * unit.conversion_factor()
    }
    
    pub fn convert_distance(value: f64, from: MeasurementUnit, to: MeasurementUnit) -> f64 {
        let base = value * from.conversion_factor();
        base / to.conversion_factor()
    }
    
    pub fn convert_area(value: f64, from: MeasurementUnit, to: MeasurementUnit) -> f64 {
        let base = value * from.conversion_factor().powi(2);
        base / to.conversion_factor().powi(2)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasurementSession {
    pub measurements: Vec<MeasurementResult>,
    pub session_start: std::time::SystemTime,
    pub unit: MeasurementUnit,
}

impl MeasurementSession {
    pub fn new(unit: MeasurementUnit) -> Self {
        Self {
            measurements: Vec::new(),
            session_start: std::time::SystemTime::now(),
            unit,
        }
    }
    
    pub fn add_measurement(&mut self, measurement: MeasurementResult) {
        self.measurements.push(measurement);
    }
    
    pub fn clear(&mut self) {
        self.measurements.clear();
    }
    
    pub fn get_statistics(&self) -> MeasurementStatistics {
        if self.measurements.is_empty() {
            return MeasurementStatistics::empty();
        }
        
        let values: Vec<f64> = self.measurements.iter().map(|m| m.value).collect();
        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let avg = sum / count;
        
        let min = *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max = *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        
        MeasurementStatistics {
            count: self.measurements.len(),
            sum,
            average: avg,
            minimum: min,
            maximum: max,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MeasurementStatistics {
    pub count: usize,
    pub sum: f64,
    pub average: f64,
    pub minimum: f64,
    pub maximum: f64,
}

impl MeasurementStatistics {
    fn empty() -> Self {
        Self {
            count: 0,
            sum: 0.0,
            average: 0.0,
            minimum: 0.0,
            maximum: 0.0,
        }
    }
    
    pub fn format(&self, unit: MeasurementUnit) -> String {
        if self.count == 0 {
            "No measurements".to_string()
        } else {
            format!(
                "Count: {} | Sum: {:.3} | Avg: {:.3} | Min: {:.3} | Max: {:.3} {}",
                self.count,
                self.sum,
                self.average,
                self.minimum,
                self.maximum,
                unit.abbreviation()
            )
        }
    }
}

// ---------------- 实体测量 ----------------

fn polyline_length(pts: &[Point]) -> f64 {
    pts.windows(2).map(|w| w[0].distance_to(&w[1])).sum()
}

fn polygon_area(pts: &[Point]) -> f64 {
    let n = pts.len();
    if n < 3 {
        return 0.0;
    }
    let mut s = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        s += pts[i].x * pts[j].y - pts[j].x * pts[i].y;
    }
    (s / 2.0).abs()
}

/// 实体长度（曲线长度；点/文字/标注返回 0）
pub fn entity_length(entity: &crate::data_structure::Entity) -> f64 {
    use crate::data_structure::EntityGeometry;
    use crate::render::tessellation::{bspline_points, nurbs_points};
    match entity.geometry() {
        EntityGeometry::Line(l) => l.length(),
        EntityGeometry::Circle(c) => c.circumference(),
        EntityGeometry::Arc(a) => a.length(),
        EntityGeometry::Ellipse(e) => e.circumference_approx(),
        EntityGeometry::Polyline(p) => {
            let pts: Vec<Point> = p.vertices.iter().map(|v| Point::new2d(v.x, v.y)).collect();
            let mut len = polyline_length(&pts);
            if p.is_closed && pts.len() > 2 {
                len += pts[pts.len() - 1].distance_to(&pts[0]);
            }
            len
        }
        EntityGeometry::BSpline(s) => polyline_length(&bspline_points(s, 256)),
        EntityGeometry::NURBS(n) => polyline_length(&nurbs_points(n, 256)),
        _ => 0.0,
    }
}

/// 实体面积（闭合形状；开放形状返回 0）
pub fn entity_area(entity: &crate::data_structure::Entity) -> f64 {
    use crate::data_structure::EntityGeometry;
    match entity.geometry() {
        EntityGeometry::Circle(c) => c.area(),
        EntityGeometry::Ellipse(e) => e.area(),
        EntityGeometry::Polyline(p) => {
            if !p.is_closed {
                return 0.0;
            }
            let pts: Vec<Point> = p.vertices.iter().map(|v| Point::new2d(v.x, v.y)).collect();
            polygon_area(&pts)
        }
        EntityGeometry::Solid { points, .. } => polygon_area(points),
        EntityGeometry::Hatch { boundary_paths, .. } => boundary_paths
            .iter()
            .filter(|bp| bp.is_polyline)
            .map(|bp| polygon_area(&bp.edges.iter().map(|e| e.start_point).collect::<Vec<_>>()))
            .sum(),
        _ => 0.0,
    }
}
