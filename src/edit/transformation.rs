use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TransformType {
    Move,
    Rotate,
    Scale,
    Mirror,
    Array,
    Offset,
    Stretch,
    Align,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform2D {
    pub translation_x: f64,
    pub translation_y: f64,
    pub rotation: f64,
    pub scale_x: f64,
    pub scale_y: f64,
    pub center_x: f64,
    pub center_y: f64,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            translation_x: 0.0,
            translation_y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            center_x: 0.0,
            center_y: 0.0,
        }
    }
}

impl Transform2D {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn translation(dx: f64, dy: f64) -> Self {
        Self {
            translation_x: dx,
            translation_y: dy,
            ..Default::default()
        }
    }

    pub fn rotation(angle: f64) -> Self {
        Self {
            rotation: angle,
            ..Default::default()
        }
    }

    pub fn scale(sx: f64, sy: f64) -> Self {
        Self {
            scale_x: sx,
            scale_y: sy,
            ..Default::default()
        }
    }

    pub fn uniform_scale(s: f64) -> Self {
        Self::scale(s, s)
    }

    pub fn mirror_x() -> Self {
        Self::scale(-1.0, 1.0)
    }

    pub fn mirror_y() -> Self {
        Self::scale(1.0, -1.0)
    }

    pub fn combined(&self, other: Transform2D) -> Self {
        Self {
            translation_x: self.translation_x + other.translation_x,
            translation_y: self.translation_y + other.translation_y,
            rotation: self.rotation + other.rotation,
            scale_x: self.scale_x * other.scale_x,
            scale_y: self.scale_y * other.scale_y,
            center_x: other.center_x,
            center_y: other.center_y,
        }
    }

    pub fn apply_to_point(&self, point: &crate::geometry::Point) -> crate::geometry::Point {
        let (cos_r, sin_r) = (self.rotation.cos(), self.rotation.sin());

        let dx = point.x - self.center_x;
        let dy = point.y - self.center_y;

        let rotated_x = dx * cos_r - dy * sin_r;
        let rotated_y = dx * sin_r + dy * cos_r;

        let scaled_x = rotated_x * self.scale_x;
        let scaled_y = rotated_y * self.scale_y;

        crate::geometry::Point::new(
            self.center_x + scaled_x + self.translation_x,
            self.center_y + scaled_y + self.translation_y,
            0.0,
        )
    }

    pub fn apply_to_vector(&self, vector: &crate::geometry::Vector2) -> crate::geometry::Vector2 {
        let (cos_r, sin_r) = (self.rotation.cos(), self.rotation.sin());

        let rotated_x = vector.x * cos_r - vector.y * sin_r;
        let rotated_y = vector.x * sin_r + vector.y * cos_r;

        crate::geometry::Vector2::new(
            rotated_x * self.scale_x,
            rotated_y * self.scale_y,
        )
    }
}

pub struct MoveTool {
    displacement: crate::geometry::Vector2,
    copy: bool,
    multiple: u32,
}

impl Default for MoveTool {
    fn default() -> Self {
        Self {
            displacement: crate::geometry::Vector2::new(0.0, 0.0),
            copy: false,
            multiple: 1,
        }
    }
}

impl MoveTool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_displacement(&mut self, dx: f64, dy: f64) {
        self.displacement = crate::geometry::Vector2::new(dx, dy);
    }

    pub fn set_base_point(&mut self, base: &crate::geometry::Point, second: &crate::geometry::Point) {
        self.displacement = second.to_vector2() - base.to_vector2();
    }

    pub fn enable_copy(&mut self, enabled: bool) {
        self.copy = enabled;
    }

    pub fn set_multiple(&mut self, count: u32) {
        self.multiple = count;
    }

    pub fn transform_entity(&self, entity: &super::super::data_structure::Entity) -> super::super::data_structure::Entity {
        let mut transformed = entity.clone();

        let mut t = transformed.transform();
        t.translation = crate::geometry::Point::new(
            t.translation.x + self.displacement.x,
            t.translation.y + self.displacement.y,
            t.translation.z,
        );
        transformed.set_transform(t);

        transformed
    }
}

pub struct RotateTool {
    center: crate::geometry::Point,
    angle: f64,
    copy: bool,
    reference_angle: Option<f64>,
}

impl Default for RotateTool {
    fn default() -> Self {
        Self {
            center: crate::geometry::Point::new(0.0, 0.0, 0.0),
            angle: 0.0,
            copy: false,
            reference_angle: None,
        }
    }
}

impl RotateTool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_center(&mut self, center: crate::geometry::Point) {
        self.center = center;
    }

    pub fn set_angle(&mut self, angle: f64) {
        self.angle = angle;
    }

    pub fn set_reference(&mut self, base: &crate::geometry::Point, second: &crate::geometry::Point) {
        let v1 = second.to_vector2() - base.to_vector2();
        self.reference_angle = Some(v1.angle());
    }

    pub fn rotate_from_reference(&mut self, current: &crate::geometry::Point) {
        if let Some(ref_angle) = self.reference_angle {
            let v = current.to_vector2() - self.center.to_vector2();
            self.angle = v.angle() - ref_angle;
        }
    }

    pub fn enable_copy(&mut self, enabled: bool) {
        self.copy = enabled;
    }

    pub fn transform_entity(&self, entity: &super::super::data_structure::Entity) -> super::super::data_structure::Entity {
        let mut transformed = entity.clone();

        let mut t = transformed.transform();
        t.rotation += self.angle;

        let (cos_r, sin_r) = (self.angle.cos(), self.angle.sin());
        let dx = t.translation.x - self.center.x;
        let dy = t.translation.y - self.center.y;
        t.translation = crate::geometry::Point::new(
            self.center.x + dx * cos_r - dy * sin_r,
            self.center.y + dx * sin_r + dy * cos_r,
            t.translation.z,
        );
        transformed.set_transform(t);

        transformed
    }
}

pub struct ScaleTool {
    base_point: crate::geometry::Point,
    scale_factor: f64,
    copy: bool,
    reference_length: Option<f64>,
}

impl Default for ScaleTool {
    fn default() -> Self {
        Self {
            base_point: crate::geometry::Point::new(0.0, 0.0, 0.0),
            scale_factor: 1.0,
            copy: false,
            reference_length: None,
        }
    }
}

impl ScaleTool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_base_point(&mut self, base: crate::geometry::Point) {
        self.base_point = base;
    }

    pub fn set_scale_factor(&mut self, factor: f64) {
        self.scale_factor = factor;
    }

    pub fn set_uniform_scale(&mut self, base: &crate::geometry::Point, current: &crate::geometry::Point, reference: f64) {
        let current_length = current.distance_to(base);
        if reference > 0.0 {
            self.scale_factor = current_length / reference;
        }
    }

    pub fn set_x_y_scale(&mut self, base: &crate::geometry::Point, current: &crate::geometry::Point, reference: f64) {
        let current_length = current.x - base.x;
        if reference > 0.0 {
            self.scale_factor = current_length / reference;
        }
    }

    pub fn enable_copy(&mut self, enabled: bool) {
        self.copy = enabled;
    }

    pub fn transform_entity(&self, entity: &super::super::data_structure::Entity) -> super::super::data_structure::Entity {
        let mut transformed = entity.clone();

        let mut t = transformed.transform();
        let s = self.scale_factor;
        t.scale *= s;
        t.translation = crate::geometry::Point::new(
            self.base_point.x + (t.translation.x - self.base_point.x) * s,
            self.base_point.y + (t.translation.y - self.base_point.y) * s,
            t.translation.z,
        );
        transformed.set_transform(t);

        transformed
    }
}

pub struct MirrorTool {
    first_point: crate::geometry::Point,
    second_point: crate::geometry::Point,
    copy: bool,
    delete_source: bool,
}

impl Default for MirrorTool {
    fn default() -> Self {
        Self {
            first_point: crate::geometry::Point::new(0.0, 0.0, 0.0),
            second_point: crate::geometry::Point::new(1.0, 0.0, 0.0),
            copy: true,
            delete_source: false,
        }
    }
}

impl MirrorTool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_axis_points(&mut self, p1: crate::geometry::Point, p2: crate::geometry::Point) {
        self.first_point = p1;
        self.second_point = p2;
    }

    pub fn enable_copy(&mut self, enabled: bool) {
        self.copy = enabled;
    }

    pub fn set_delete_source(&mut self, delete: bool) {
        self.delete_source = delete;
    }

    pub fn transform_entity(&self, entity: &super::super::data_structure::Entity) -> super::super::data_structure::Entity {
        let mut transformed = entity.clone();

        let dx = self.second_point.x - self.first_point.x;
        let dy = self.second_point.y - self.first_point.y;
        let d = dx * dx + dy * dy;

        if d > 0.0 {
            let mut t = transformed.transform();

            let tr = t.translation;
            let ex = tr.x - self.first_point.x;
            let ey = tr.y - self.first_point.y;
            let dot = (ex * dx + ey * dy) / d;
            let px = self.first_point.x + dot * dx;
            let py = self.first_point.y + dot * dy;
            t.translation = crate::geometry::Point::new(
                2.0 * px - tr.x,
                2.0 * py - tr.y,
                tr.z,
            );

            let axis_angle = dy.atan2(dx);
            t.rotation = 2.0 * axis_angle - t.rotation;
            t.scale = -t.scale;

            transformed.set_transform(t);
        }

        transformed
    }
}

pub struct ArrayTool {
    rows: u32,
    columns: u32,
    row_spacing: f64,
    column_spacing: f64,
    angle: f64,
    associativity: bool,
    array_type: ArrayType,
    center_point: Option<crate::geometry::Point>,
    polar_angle: f64,
    polar_fill_angle: f64,
}

impl Default for ArrayTool {
    fn default() -> Self {
        Self {
            rows: 2,
            columns: 2,
            row_spacing: 1.0,
            column_spacing: 1.0,
            angle: 0.0,
            associativity: false,
            array_type: ArrayType::Rectangular,
            center_point: None,
            polar_angle: 360.0,
            polar_fill_angle: 360.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArrayType {
    Rectangular,
    Polar,
}

impl ArrayTool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_rectangular(&mut self, rows: u32, columns: u32, row_spacing: f64, column_spacing: f64) {
        self.rows = rows;
        self.columns = columns;
        self.row_spacing = row_spacing;
        self.column_spacing = column_spacing;
        self.array_type = ArrayType::Rectangular;
    }

    pub fn set_polar(&mut self, items: u32, angle_between: f64, fill_angle: f64, center: crate::geometry::Point) {
        self.columns = items;
        self.polar_angle = angle_between;
        self.polar_fill_angle = fill_angle;
        self.center_point = Some(center);
        self.array_type = ArrayType::Polar;
    }

    pub fn set_angle(&mut self, angle: f64) {
        self.angle = angle;
    }

    pub fn set_associativity(&mut self, associative: bool) {
        self.associativity = associative;
    }

    pub fn generate_copies(&self, entity: &super::super::data_structure::Entity) -> Vec<super::super::data_structure::Entity> {
        let mut copies = Vec::new();

        for row in 0..self.rows {
            for col in 0..self.columns {
                if row == 0 && col == 0 {
                    continue;
                }

                let mut transformed = entity.clone();

                let mut t = transformed.transform();
                let dx = col as f64 * self.column_spacing;
                let dy = row as f64 * self.row_spacing;

                let (offset_x, offset_y) = if self.angle != 0.0 {
                    let (cos_a, sin_a) = (self.angle.cos(), self.angle.sin());
                    (dx * cos_a - dy * sin_a, dx * sin_a + dy * cos_a)
                } else {
                    (dx, dy)
                };

                t.translation = crate::geometry::Point::new(
                    t.translation.x + offset_x,
                    t.translation.y + offset_y,
                    t.translation.z,
                );
                transformed.set_transform(t);

                copies.push(transformed);
            }
        }

        copies
    }
}

pub struct TransformTool {
    current_type: TransformType,
    transform: Transform2D,
}

impl Default for TransformTool {
    fn default() -> Self {
        Self {
            current_type: TransformType::Move,
            transform: Transform2D::new(),
        }
    }
}

impl TransformTool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_type(&mut self, transform_type: TransformType) {
        self.current_type = transform_type;
        self.transform = Transform2D::new();
    }

    pub fn get_type(&self) -> TransformType {
        self.current_type
    }

    pub fn preview_transform(
        &self,
        entity: &super::super::data_structure::Entity,
        point: crate::geometry::Point,
    ) -> super::super::data_structure::Entity {
        match self.current_type {
            TransformType::Move => {
                let mut tool = MoveTool::new();
                tool.set_displacement(point.x, point.y);
                tool.transform_entity(entity)
            }
            TransformType::Rotate => {
                let mut tool = RotateTool::new();
                tool.set_angle(point.x);
                tool.transform_entity(entity)
            }
            TransformType::Scale => {
                let mut tool = ScaleTool::new();
                tool.set_scale_factor(point.x.max(0.01));
                tool.transform_entity(entity)
            }
            _ => entity.clone(),
        }
    }
}

impl fmt::Display for Transform2D {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Transform2D(trans=({}, {}), rot={}, scale=({}, {}))",
            self.translation_x,
            self.translation_y,
            self.rotation,
            self.scale_x,
            self.scale_y
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_2d() {
        let transform = Transform2D::translation(10.0, 20.0);

        let point = crate::geometry::Point::new(0.0, 0.0, 0.0);
        let transformed = transform.apply_to_point(&point);

        assert!((transformed.x - 10.0).abs() < 0.001);
        assert!((transformed.y - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_rotation_transform() {
        let transform = Transform2D::rotation(std::f64::consts::PI / 2.0);

        let point = crate::geometry::Point::new(1.0, 0.0, 0.0);
        let transformed = transform.apply_to_point(&point);

        assert!((transformed.x - 0.0).abs() < 0.001);
        assert!((transformed.y - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_scale_transform() {
        let transform = Transform2D::uniform_scale(2.0);

        let point = crate::geometry::Point::new(1.0, 2.0, 0.0);
        let transformed = transform.apply_to_point(&point);

        assert!((transformed.x - 2.0).abs() < 0.001);
        assert!((transformed.y - 4.0).abs() < 0.001);
    }
}
