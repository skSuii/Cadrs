use std::f64::consts::PI;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    pub fn angle_to(&self, other: Point) -> f64 {
        (other.y - self.y).atan2(other.x - self.x)
    }

    pub fn lerp(&self, other: Point, t: f64) -> Point {
        Point {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }

    pub fn rotate(&self, center: Point, angle: f64) -> Point {
        let dx = self.x - center.x;
        let dy = self.y - center.y;
        let cos = angle.cos();
        let sin = angle.sin();
        Point {
            x: center.x + dx * cos - dy * sin,
            y: center.y + dx * sin + dy * cos,
        }
    }
}

impl Default for Point {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl From<crate::geometry::point::Point> for Point {
    fn from(p: crate::geometry::point::Point) -> Self {
        Self { x: p.x, y: p.y }
    }
}

impl From<&crate::geometry::point::Point> for Point {
    fn from(p: &crate::geometry::point::Point) -> Self {
        Self { x: p.x, y: p.y }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    pub start: Point,
    pub end: Point,
}

impl Line {
    pub fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }

    pub fn length(&self) -> f64 {
        self.start.distance_to(self.end)
    }

    pub fn angle(&self) -> f64 {
        self.start.angle_to(self.end)
    }
}

#[derive(Debug, Clone)]
pub struct Circle {
    pub center: Point,
    pub radius: f64,
}

impl Circle {
    pub fn new(center: Point, radius: f64) -> Self {
        Self { center, radius }
    }

    pub fn circumference(&self) -> f64 {
        2.0 * PI * self.radius
    }

    pub fn area(&self) -> f64 {
        PI * self.radius.powi(2)
    }
}

#[derive(Debug, Clone)]
pub struct Arc {
    pub center: Point,
    pub radius: f64,
    pub start_angle: f64,
    pub end_angle: f64,
    pub is_counter_clockwise: bool,
}

impl Arc {
    pub fn new(center: Point, radius: f64, start_angle: f64, end_angle: f64) -> Self {
        let is_counter_clockwise = if end_angle >= start_angle {
            end_angle - start_angle <= PI
        } else {
            start_angle - end_angle > PI
        };
        
        Self {
            center,
            radius,
            start_angle,
            end_angle,
            is_counter_clockwise,
        }
    }

    pub fn length(&self) -> f64 {
        let sweep = if self.is_counter_clockwise {
            let sweep = self.end_angle - self.start_angle;
            if sweep < 0.0 { sweep + 2.0 * PI } else { sweep }
        } else {
            let sweep = self.start_angle - self.end_angle;
            if sweep < 0.0 { sweep + 2.0 * PI } else { sweep }
        };
        self.radius * sweep
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ellipse {
    pub center: Point,
    pub semi_major: f64,
    pub semi_minor: f64,
    pub rotation: f64,
}

impl Ellipse {
    pub fn new(center: Point, semi_major: f64, semi_minor: f64, rotation: f64) -> Self {
        Self {
            center,
            semi_major,
            semi_minor,
            rotation,
        }
    }

    pub fn eccentricity(&self) -> f64 {
        (1.0 - (self.semi_minor / self.semi_major).powi(2)).sqrt()
    }

    pub fn area(&self) -> f64 {
        PI * self.semi_major * self.semi_minor
    }

    pub fn circumference_approx(&self) -> f64 {
        let a = self.semi_major;
        let b = self.semi_minor;
        PI * (3.0 * (a + b) - ((3.0 * a + b) * (a + 3.0 * b)).sqrt())
    }
}

#[derive(Debug, Clone)]
pub struct EllipseArc {
    pub center: Point,
    pub semi_major: f64,
    pub semi_minor: f64,
    pub rotation: f64,
    pub start_angle: f64,
    pub end_angle: f64,
    pub is_counter_clockwise: bool,
}

impl EllipseArc {
    pub fn new(
        center: Point,
        semi_major: f64,
        semi_minor: f64,
        rotation: f64,
        start_angle: f64,
        end_angle: f64,
    ) -> Self {
        let is_counter_clockwise = if end_angle >= start_angle {
            end_angle - start_angle <= PI
        } else {
            start_angle - end_angle > PI
        };

        Self {
            center,
            semi_major,
            semi_minor,
            rotation,
            start_angle,
            end_angle,
            is_counter_clockwise,
        }
    }

    pub fn point_at(&self, t: f64) -> Point {
        let angle = if self.is_counter_clockwise {
            self.start_angle + t * (self.end_angle - self.start_angle)
        } else {
            self.start_angle - t * (self.start_angle - self.end_angle)
        };
        
        let local_x = self.semi_major * angle.cos();
        let local_y = self.semi_minor * angle.sin();
        
        let cos_rot = self.rotation.cos();
        let sin_rot = self.rotation.sin();
        
        Point {
            x: self.center.x + local_x * cos_rot - local_y * sin_rot,
            y: self.center.y + local_x * sin_rot + local_y * cos_rot,
        }
    }

    pub fn length_approx(&self, samples: usize) -> f64 {
        let mut length = 0.0;
        let dt = 1.0 / samples as f64;
        
        for i in 0..samples {
            let t1 = i as f64 * dt;
            let t2 = (i + 1) as f64 * dt;
            let p1 = self.point_at(t1);
            let p2 = self.point_at(t2);
            length += p1.distance_to(p2);
        }
        
        length
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rectangle {
    pub corner1: Point,
    pub corner2: Point,
}

impl Rectangle {
    pub fn new(corner1: Point, corner2: Point) -> Self {
        Self { corner1, corner2 }
    }

    pub fn width(&self) -> f64 {
        (self.corner2.x - self.corner1.x).abs()
    }

    pub fn height(&self) -> f64 {
        (self.corner2.y - self.corner1.y).abs()
    }

    pub fn area(&self) -> f64 {
        self.width() * self.height()
    }

    pub fn perimeter(&self) -> f64 {
        2.0 * (self.width() + self.height())
    }

    pub fn center(&self) -> Point {
        Point {
            x: (self.corner1.x + self.corner2.x) / 2.0,
            y: (self.corner1.y + self.corner2.y) / 2.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Polygon {
    pub vertices: Vec<Point>,
}

impl Polygon {
    pub fn new(vertices: Vec<Point>) -> Self {
        Self { vertices }
    }

    pub fn area(&self) -> f64 {
        if self.vertices.len() < 3 {
            return 0.0;
        }
        
        let mut area = 0.0;
        let n = self.vertices.len();
        
        for i in 0..n {
            let j = (i + 1) % n;
            area += self.vertices[i].x * self.vertices[j].y;
            area -= self.vertices[j].x * self.vertices[i].y;
        }
        
        area.abs() / 2.0
    }

    pub fn perimeter(&self) -> f64 {
        if self.vertices.len() < 3 {
            return 0.0;
        }
        
        let mut perimeter = 0.0;
        let n = self.vertices.len();
        
        for i in 0..n {
            let j = (i + 1) % n;
            perimeter += self.vertices[i].distance_to(self.vertices[j]);
        }
        
        perimeter
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Polyline {
    pub vertices: Vec<Point>,
    pub is_closed: bool,
}

impl Polyline {
    pub fn new(vertices: Vec<Point>, is_closed: bool) -> Self {
        Self { vertices, is_closed }
    }

    pub fn push<P: Into<Point>>(&mut self, point: P) {
        self.vertices.push(point.into());
    }

    pub fn close(&mut self) {
        self.is_closed = true;
    }

    pub fn from_points<I, P>(points: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<Point>,
    {
        Self {
            vertices: points.into_iter().map(|p| p.into()).collect(),
            is_closed: false,
        }
    }

    pub fn vertices(&self) -> &Vec<Point> {
        &self.vertices
    }

    pub fn length(&self) -> f64 {
        if self.vertices.len() < 2 {
            return 0.0;
        }
        
        let mut length = 0.0;
        for i in 0..self.vertices.len() - 1 {
            length += self.vertices[i].distance_to(self.vertices[i + 1]);
        }
        
        if self.is_closed && self.vertices.len() > 2 {
            length += self.vertices.last().unwrap().distance_to(self.vertices[0]);
        }
        
        length
    }
}

impl Default for Polyline {
    fn default() -> Self {
        Self {
            vertices: Vec::new(),
            is_closed: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NurbsCurve {
    pub control_points: Vec<Point>,
    pub degree: usize,
    pub knots: Vec<f64>,
    pub weights: Option<Vec<f64>>,
}

impl NurbsCurve {
    pub fn new(control_points: Vec<Point>, degree: usize) -> Self {
        let n = control_points.len();
        let m = n + degree + 1;
        
        let mut knots = Vec::with_capacity(m);
        for _ in 0..=degree {
            knots.push(0.0);
        }
        for i in degree + 1..n {
            let t = (i - degree) as f64 / (n - degree) as f64;
            knots.push(t);
        }
        for _ in 0..=degree {
            knots.push(1.0);
        }
        
        Self {
            control_points,
            degree,
            knots,
            weights: None,
        }
    }

    pub fn with_weights(mut self, weights: Vec<f64>) -> Self {
        self.weights = Some(weights);
        self
    }

    pub fn point_at(&self, t: f64) -> Point {
        let _ = self.control_points.len() - 1;
        let t = t.clamp(0.0, 1.0);
        
        let span = self.find_span(t);
        let basis = self.evaluate_basis(span, t);
        
        let mut x = 0.0;
        let mut y = 0.0;
        let mut w = 0.0;
        
        for i in 0..=self.degree {
            let index = span - self.degree + i;
            let weight = self.weights.as_ref()
                .map_or(1.0, |w| w[index]);
            let b = basis[i];
            
            x += self.control_points[index].x * weight * b;
            y += self.control_points[index].y * weight * b;
            w += weight * b;
        }
        
        Point { x: x / w, y: y / w }
    }

    fn find_span(&self, t: f64) -> usize {
        let n = self.control_points.len() - 1;
        
        if t >= self.knots[n + 1] {
            return n;
        }
        
        let mut low = self.degree;
        let mut high = n + 1;
        let mut mid = (low + high) / 2;
        
        while t < self.knots[mid] || t >= self.knots[mid + 1] {
            if t < self.knots[mid] {
                high = mid;
            } else {
                low = mid;
            }
            mid = (low + high) / 2;
        }
        
        mid
    }

    fn evaluate_basis(&self, span: usize, t: f64) -> Vec<f64> {
        let p = self.degree;
        let n = p + 1;
        
        let mut left = vec![0.0; n];
        let mut right = vec![0.0; n];
        let mut basis = vec![1.0; n];
        
        for i in 1..=p {
            left[i - 1] = t - self.knots[span + 1 - i];
            right[i - 1] = self.knots[span + i] - t;
            
            let mut saved = 0.0;
            for j in 0..i {
                let temp = basis[j] / (right[j] + left[i - 1 - j]);
                basis[j] = saved + right[j] * temp;
                saved = left[i - 1 - j] * temp;
            }
            basis[i] = saved;
        }
        
        basis
    }
}

#[derive(Debug, Clone)]
pub struct InvoluteCurve {
    pub base_circle_radius: f64,
    pub start_point: Point,
    pub direction: f64,
    pub resolution: usize,
}

impl InvoluteCurve {
    pub fn new(base_circle_radius: f64, start_point: Point, direction: f64) -> Self {
        Self {
            base_circle_radius,
            start_point,
            direction,
            resolution: 100,
        }
    }

    pub fn point_at_angle(&self, angle: f64) -> Point {
        let t = (self.base_circle_radius * angle).cos() + angle * (self.base_circle_radius * angle).sin();
        let y_coord = (self.base_circle_radius * angle).sin() - angle * (self.base_circle_radius * angle).cos();
        
        Point {
            x: self.start_point.x + self.base_circle_radius * (t * self.direction.cos() - y_coord * self.direction.sin()),
            y: self.start_point.y + self.base_circle_radius * (t * self.direction.sin() + y_coord * self.direction.cos()),
        }
    }

    pub fn generate_points(&self, end_angle: f64) -> Vec<Point> {
        let mut points = Vec::with_capacity(self.resolution);
        let step = end_angle / self.resolution as f64;
        
        for i in 0..=self.resolution {
            let angle = i as f64 * step;
            points.push(self.point_at_angle(angle));
        }
        
        points
    }
}

#[derive(Debug, Clone)]
pub struct GearProfile {
    pub num_teeth: usize,
    pub module: f64,
    pub pressure_angle: f64,
    pub addendum_coefficient: f64,
    pub dedendum_coefficient: f64,
    pub fillet_radius: f64,
}

impl GearProfile {
    pub fn new(num_teeth: usize, module: f64, pressure_angle: f64) -> Self {
        Self {
            num_teeth,
            module,
            pressure_angle,
            addendum_coefficient: 1.0,
            dedendum_coefficient: 1.25,
            fillet_radius: 0.38 * module,
        }
    }

    pub fn pitch_radius(&self) -> f64 {
        self.module * self.num_teeth as f64 / 2.0
    }

    pub fn base_radius(&self) -> f64 {
        self.pitch_radius() * self.pressure_angle.cos()
    }

    pub fn addendum_radius(&self) -> f64 {
        self.pitch_radius() + self.addendum_coefficient * self.module
    }

    pub fn dedendum_radius(&self) -> f64 {
        self.pitch_radius() - self.dedendum_coefficient * self.module
    }

    pub fn generate_tooth_profile(&self, tooth_index: usize) -> Vec<Point> {
        let mut profile = Vec::new();
        
        let tooth_angle = 2.0 * PI / self.num_teeth as f64;
        let base_angle = tooth_index as f64 * tooth_angle;
        
        let involute = InvoluteCurve::new(
            self.base_radius(),
            Point::new(self.base_radius(), 0.0),
            0.0,
        );
        
        let start_angle = 0.0;
        let end_angle = PI / 2.0 - self.pressure_angle;
        
        let involute_points = involute.generate_points(end_angle);
        
        let addendum_angle = end_angle;
        
        profile.push(Point::new(
            self.addendum_radius() * (base_angle + addendum_angle).cos(),
            self.addendum_radius() * (base_angle + addendum_angle).sin(),
        ));
        
        for p in &involute_points {
            let angle = base_angle + start_angle + (p.y / self.base_radius());
            let radius = (p.x.powi(2) + p.y.powi(2)).sqrt();
            profile.push(Point::new(
                radius * angle.cos(),
                radius * angle.sin(),
            ));
        }
        
        let _dedendum_start_angle = PI / 2.0 - self.pressure_angle;
        let dedendum_end_angle = PI / 2.0 + self.pressure_angle;
        
        profile.push(Point::new(
            self.dedendum_radius() * (base_angle + dedendum_end_angle).cos(),
            self.dedendum_radius() * (base_angle + dedendum_end_angle).sin(),
        ));
        
        let tooth_center_angle = PI / self.num_teeth as f64;
        let next_tooth_start = base_angle + tooth_center_angle;
        
        profile.push(Point::new(
            self.dedendum_radius() * (next_tooth_start - self.pressure_angle).cos(),
            self.dedendum_radius() * (next_tooth_start - self.pressure_angle).sin(),
        ));
        
        profile
    }

    pub fn generate_full_profile(&self) -> Vec<Point> {
        let mut full_profile = Vec::new();
        
        for i in 0..self.num_teeth {
            let tooth_profile = self.generate_tooth_profile(i);
            full_profile.extend(tooth_profile);
        }
        
        full_profile
    }
}

#[derive(Debug, Clone)]
pub struct Helix2D {
    pub center: Point,
    pub radius: f64,
    pub pitch: f64,
    pub rotation_direction: f64,
}

impl Helix2D {
    pub fn new(center: Point, radius: f64, pitch: f64, rotation_direction: f64) -> Self {
        Self {
            center,
            radius,
            pitch,
            rotation_direction,
        }
    }

    pub fn point_at_z(&self, z: f64) -> Point {
        let angle = z * self.rotation_direction * 2.0 * PI / self.pitch;
        
        Point {
            x: self.center.x + self.radius * angle.cos(),
            y: self.center.y + self.radius * angle.sin(),
        }
    }

    pub fn generate_points(&self, height: f64, resolution: usize) -> Vec<Point> {
        let mut points = Vec::with_capacity(resolution);
        let step = height / resolution as f64;
        
        for i in 0..=resolution {
            points.push(self.point_at_z(i as f64 * step));
        }
        
        points
    }
}

#[derive(Debug, Clone)]
pub struct CloudLine {
    pub points: Vec<Point>,
    pub bulge_factor: f64,
    pub amplitude: f64,
    pub frequency: f64,
}

impl CloudLine {
    pub fn new(points: Vec<Point>, bulge_factor: f64) -> Self {
        Self {
            points,
            bulge_factor,
            amplitude: 2.0,
            frequency: 3.0,
        }
    }

    pub fn with_amplitude(mut self, amplitude: f64) -> Self {
        self.amplitude = amplitude;
        self
    }

    pub fn with_frequency(mut self, frequency: f64) -> Self {
        self.frequency = frequency;
        self
    }

    pub fn generate_points(&self, segments_per_edge: usize) -> Vec<Point> {
        let mut result = Vec::new();
        
        for i in 0..self.points.len() {
            let start = self.points[i];
            let end = self.points[(i + 1) % self.points.len()];
            
            let dx = end.x - start.x;
            let dy = end.y - start.y;
            let length = (dx.powi(2) + dy.powi(2)).sqrt();
            
            if length < 1e-6 {
                continue;
            }
            
            let nx = dx / length;
            let ny = dy / length;
            
            let perpendicular_x = -ny;
            let perpendicular_y = nx;
            
            for j in 0..=segments_per_edge {
                let t = j as f64 / segments_per_edge as f64;
                let base_x = start.x + dx * t;
                let base_y = start.y + dy * t;
                
                let normalized_t = t * PI * self.frequency;
                let bulge = (normalized_t.sin() * self.amplitude * self.bulge_factor)
                    * (1.0 - (t - 0.5).powi(2) * 4.0);
                
                result.push(Point {
                    x: base_x + perpendicular_x * bulge,
                    y: base_y + perpendicular_y * bulge,
                });
            }
        }
        
        result
    }
}

#[derive(Debug, Clone)]
pub struct SplineFittedPolyline {
    pub control_points: Vec<Point>,
    pub tolerance: f64,
}

impl SplineFittedPolyline {
    pub fn new(control_points: Vec<Point>, tolerance: f64) -> Self {
        Self {
            control_points,
            tolerance,
        }
    }

    pub fn fit(&self) -> Vec<Point> {
        if self.control_points.len() < 3 {
            return self.control_points.clone();
        }
        
        let nurbs = NurbsCurve::new(self.control_points.clone(), 3);
        
        let total_length = self.calculate_total_length();
        let num_segments = (total_length / self.tolerance).ceil() as usize;
        
        let mut fitted_points = Vec::with_capacity(num_segments + 1);
        
        for i in 0..=num_segments {
            let t = i as f64 / num_segments as f64;
            fitted_points.push(nurbs.point_at(t));
        }
        
        fitted_points
    }

    fn calculate_total_length(&self) -> f64 {
        let mut length = 0.0;
        for i in 0..self.control_points.len() - 1 {
            length += self.control_points[i].distance_to(self.control_points[i + 1]);
        }
        length
    }
}
