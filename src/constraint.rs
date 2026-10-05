use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeometricConstraint {
    Coincident,
    Perpendicular,
    Parallel,
    Tangent,
    Horizontal,
    Vertical,
    EqualLength,
    EqualRadius,
    Symmetric,
    Midpoint,
    Center,
    Fix,
}

impl Default for GeometricConstraint {
    fn default() -> Self {
        GeometricConstraint::Coincident
    }
}

impl GeometricConstraint {
    pub fn name(&self) -> &str {
        match self {
            GeometricConstraint::Coincident => "Coincident",
            GeometricConstraint::Perpendicular => "Perpendicular",
            GeometricConstraint::Parallel => "Parallel",
            GeometricConstraint::Tangent => "Tangent",
            GeometricConstraint::Horizontal => "Horizontal",
            GeometricConstraint::Vertical => "Vertical",
            GeometricConstraint::EqualLength => "Equal Length",
            GeometricConstraint::EqualRadius => "Equal Radius",
            GeometricConstraint::Symmetric => "Symmetric",
            GeometricConstraint::Midpoint => "Midpoint",
            GeometricConstraint::Center => "Center",
            GeometricConstraint::Fix => "Fix",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            GeometricConstraint::Coincident => "⭕",
            GeometricConstraint::Perpendicular => "⊥",
            GeometricConstraint::Parallel => "∥",
            GeometricConstraint::Tangent => "◎",
            GeometricConstraint::Horizontal => "—",
            GeometricConstraint::Vertical => "|",
            GeometricConstraint::EqualLength => "=",
            GeometricConstraint::EqualRadius => "≅",
            GeometricConstraint::Symmetric => "◈",
            GeometricConstraint::Midpoint => "◉",
            GeometricConstraint::Center => "⊕",
            GeometricConstraint::Fix => "📌",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstraintEntity {
    pub id: String,
    pub constraint_type: GeometricConstraint,
    pub first_entity: String,
    pub second_entity: Option<String>,
    pub point_on_first: Option<crate::geometry::Point>,
    pub point_on_second: Option<crate::geometry::Point>,
    pub is_applied: bool,
    pub is_dragging: bool,
    pub reference: bool,
}

impl Default for ConstraintEntity {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstraintEntity {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            constraint_type: GeometricConstraint::Coincident,
            first_entity: String::new(),
            second_entity: None,
            point_on_first: None,
            point_on_second: None,
            is_applied: false,
            is_dragging: false,
            reference: false,
        }
    }

    pub fn with_type(mut self, constraint_type: GeometricConstraint) -> Self {
        self.constraint_type = constraint_type;
        self
    }

    pub fn with_first_entity(mut self, entity: &str) -> Self {
        self.first_entity = entity.to_string();
        self
    }

    pub fn with_second_entity(mut self, entity: &str) -> Self {
        self.second_entity = Some(entity.to_string());
        self
    }

    pub fn with_first_point(mut self, point: crate::geometry::Point) -> Self {
        self.point_on_first = Some(point);
        self
    }

    pub fn with_second_point(mut self, point: crate::geometry::Point) -> Self {
        self.point_on_second = Some(point);
        self
    }
}

impl fmt::Display for ConstraintEntity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {} on {}",
            self.constraint_type.name(),
            self.first_entity,
            self.second_entity.as_ref().unwrap_or(&"None".to_string())
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolverState {
    pub solved: bool,
    pub iterations: u32,
    pub error: f64,
    pub max_error: f64,
    pub constrained_dof: u32,
    pub free_dof: u32,
}

impl Default for SolverState {
    fn default() -> Self {
        Self::new()
    }
}

impl SolverState {
    pub fn new() -> Self {
        Self {
            solved: true,
            iterations: 0,
            error: 0.0,
            max_error: 1e-6,
            constrained_dof: 0,
            free_dof: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GeometricSolver {
    pub constraints: Vec<ConstraintEntity>,
    pub entities: HashMap<String, EntityState>,
    pub tolerance: f64,
    pub max_iterations: u32,
    pub state: SolverState,
    pub auto_solve: bool,
    pub inference_constraints: bool,
}

impl Default for GeometricSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl GeometricSolver {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            entities: HashMap::new(),
            tolerance: 1e-6,
            max_iterations: 1000,
            state: SolverState::new(),
            auto_solve: true,
            inference_constraints: true,
        }
    }

    pub fn add_constraint(&mut self, constraint: ConstraintEntity) -> bool {
        if self.validate_constraint(&constraint) {
            self.constraints.push(constraint.clone());
            if self.auto_solve {
                self.solve();
            }
            true
        } else {
            false
        }
    }

    pub fn remove_constraint(&mut self, constraint_id: &str) -> bool {
        let original_len = self.constraints.len();
        self.constraints.retain(|c| c.id != constraint_id);
        if self.auto_solve {
            self.solve();
        }
        self.constraints.len() != original_len
    }

    pub fn get_constraint(&self, constraint_id: &str) -> Option<&ConstraintEntity> {
        self.constraints.iter().find(|c| c.id == constraint_id)
    }

    pub fn constraints_for_entity(&self, entity_id: &str) -> Vec<&ConstraintEntity> {
        self.constraints
            .iter()
            .filter(|c| c.first_entity == entity_id || c.second_entity.as_ref() == Some(&entity_id.to_string()))
            .collect()
    }

    pub fn validate_constraint(&self, constraint: &ConstraintEntity) -> bool {
        match constraint.constraint_type {
            GeometricConstraint::Coincident
            | GeometricConstraint::Perpendicular
            | GeometricConstraint::Parallel
            | GeometricConstraint::Tangent
            | GeometricConstraint::EqualLength
            | GeometricConstraint::EqualRadius
            | GeometricConstraint::Symmetric => constraint.second_entity.is_some(),
            GeometricConstraint::Horizontal
            | GeometricConstraint::Vertical
            | GeometricConstraint::Midpoint
            | GeometricConstraint::Fix => constraint.second_entity.is_none(),
            GeometricConstraint::Center => true,
        }
    }

    pub fn solve(&mut self) -> &SolverState {
        self.state.iterations = 0;
        self.state.error = 0.0;
        self.state.solved = true;

        let mut converged = false;
        let mut error = f64::MAX;

        for _ in 0..self.max_iterations {
            self.state.iterations += 1;
            error = self.evaluate_constraints();

            if error < self.tolerance {
                converged = true;
                break;
            }

            self.apply_corrections(error);
        }

        self.state.error = error;
        self.state.solved = converged;

        self.calculate_dof();
        &self.state
    }

    fn evaluate_constraints(&self) -> f64 {
        let mut max_error: f64 = 0.0;

        for constraint in &self.constraints {
            let error = match constraint.constraint_type {
                GeometricConstraint::Coincident => self.evaluate_coincident(constraint),
                GeometricConstraint::Perpendicular => self.evaluate_perpendicular(constraint),
                GeometricConstraint::Parallel => self.evaluate_parallel(constraint),
                GeometricConstraint::Tangent => self.evaluate_tangent(constraint),
                GeometricConstraint::Horizontal => self.evaluate_horizontal(constraint),
                GeometricConstraint::Vertical => self.evaluate_vertical(constraint),
                GeometricConstraint::EqualLength => self.evaluate_equal_length(constraint),
                GeometricConstraint::EqualRadius => self.evaluate_equal_radius(constraint),
                GeometricConstraint::Symmetric => self.evaluate_symmetric(constraint),
                GeometricConstraint::Midpoint => self.evaluate_midpoint(constraint),
                GeometricConstraint::Center => self.evaluate_center(constraint),
                GeometricConstraint::Fix => self.evaluate_fix(constraint),
            };
            max_error = max_error.max(error);
        }

        max_error
    }

    fn evaluate_coincident(&self, constraint: &ConstraintEntity) -> f64 {
        if let (Some(first), Some(second)) = (
            self.get_entity_position(&constraint.first_entity),
            constraint.second_entity.as_ref().and_then(|e| self.get_entity_position(e)),
        ) {
            first.distance_to(&second)
        } else {
            0.0
        }
    }

    fn evaluate_perpendicular(&self, constraint: &ConstraintEntity) -> f64 {
        if let (Some(first), Some(second)) = (
            self.get_entity_direction(&constraint.first_entity),
            self.get_entity_direction(&constraint.second_entity.as_ref().unwrap()),
        ) {
            let dot = first.x * second.x + first.y * second.y;
            let angle = dot.abs().acos();
            let target = PI / 2.0;
            (angle - target).abs()
        } else {
            0.0
        }
    }

    fn evaluate_parallel(&self, constraint: &ConstraintEntity) -> f64 {
        if let (Some(first), Some(second)) = (
            self.get_entity_direction(&constraint.first_entity),
            self.get_entity_direction(&constraint.second_entity.as_ref().unwrap()),
        ) {
            let dot = first.x * second.x + first.y * second.y;
            let angle = dot.abs().acos();
            angle.min(PI - angle)
        } else {
            0.0
        }
    }

    fn evaluate_tangent(&self, constraint: &ConstraintEntity) -> f64 {
        if let (Some(first), Some(second)) = (
            self.get_entity_direction(&constraint.first_entity),
            self.get_entity_direction(&constraint.second_entity.as_ref().unwrap()),
        ) {
            let center1 = self.get_entity_center(&constraint.first_entity);
            let center2 = self.get_entity_center(&constraint.second_entity.as_ref().unwrap());

            if let (Some(c1), Some(c2)) = (center1, center2) {
                let point1 = self.get_entity_closest_point(&constraint.first_entity, &c2);
                let point2 = self.get_entity_closest_point(&constraint.second_entity.as_ref().unwrap(), &c1);

                point1.distance_to(&point2)
            } else {
                let dot = first.x * second.x + first.y * second.y;
                let angle = dot.abs().acos();
                angle.min(PI - angle) * 100.0
            }
        } else {
            0.0
        }
    }

    fn evaluate_horizontal(&self, constraint: &ConstraintEntity) -> f64 {
        if let Some(direction) = self.get_entity_direction(&constraint.first_entity) {
            direction.y.abs()
        } else {
            0.0
        }
    }

    fn evaluate_vertical(&self, constraint: &ConstraintEntity) -> f64 {
        if let Some(direction) = self.get_entity_direction(&constraint.first_entity) {
            direction.x.abs()
        } else {
            0.0
        }
    }

    fn evaluate_equal_length(&self, constraint: &ConstraintEntity) -> f64 {
        let len1 = self.get_entity_length(&constraint.first_entity);
        let len2 = self.get_entity_length(constraint.second_entity.as_ref().unwrap());
        (len1 - len2).abs()
    }

    fn evaluate_equal_radius(&self, constraint: &ConstraintEntity) -> f64 {
        let r1 = self.get_entity_radius(&constraint.first_entity);
        let r2 = self.get_entity_radius(constraint.second_entity.as_ref().unwrap());
        (r1 - r2).abs()
    }

    fn evaluate_symmetric(&self, constraint: &ConstraintEntity) -> f64 {
        if let (Some(line1), Some(line2)) = (
            self.get_entity_points(&constraint.first_entity),
            constraint.second_entity.as_ref().and_then(|e| self.get_entity_points(e)),
        ) {
            let mid1 = Point::new(
                (line1.0.x + line1.1.x) / 2.0,
                (line1.0.y + line1.1.y) / 2.0,
                0.0,
            );
            let mid2 = Point::new(
                (line2.0.x + line2.1.x) / 2.0,
                (line2.0.y + line2.1.y) / 2.0,
                0.0,
            );
            mid1.distance_to(&mid2)
        } else {
            0.0
        }
    }

    fn evaluate_midpoint(&self, constraint: &ConstraintEntity) -> f64 {
        if let (Some(points), Some(entity_point)) = (
            self.get_entity_points(&constraint.first_entity),
            constraint.point_on_second.as_ref().or(constraint.point_on_first.as_ref()),
        ) {
            let mid = Point::new(
                (points.0.x + points.1.x) / 2.0,
                (points.0.y + points.1.y) / 2.0,
                0.0,
            );
            mid.distance_to(entity_point)
        } else {
            0.0
        }
    }

    fn evaluate_center(&self, constraint: &ConstraintEntity) -> f64 {
        if let Some(center) = self.get_entity_center(&constraint.first_entity) {
            if let Some(point) = constraint.point_on_second.as_ref().or(constraint.point_on_first.as_ref()) {
                center.distance_to(point)
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    fn evaluate_fix(&self, constraint: &ConstraintEntity) -> f64 {
        if let Some(state) = self.entities.get(&constraint.first_entity) {
            state.drag_distance
        } else {
            0.0
        }
    }

    fn apply_corrections(&mut self, _error: f64) {
        let constraints: Vec<_> = self.constraints.clone();
        for constraint in constraints {
            match constraint.constraint_type {
                GeometricConstraint::Coincident => self.apply_coincident_correction(&constraint),
                GeometricConstraint::Horizontal => self.apply_horizontal_correction(&constraint),
                GeometricConstraint::Vertical => self.apply_vertical_correction(&constraint),
                _ => {}
            }
        }
    }

    fn apply_coincident_correction(&mut self, constraint: &ConstraintEntity) {
        if let (Some(first_pos), Some(second_pos)) = (
            self.get_entity_position(&constraint.first_entity),
            constraint.second_entity.as_ref().and_then(|e| self.get_entity_position(e)),
        ) {
            let correction = Point::new(
                (second_pos.x - first_pos.x) / 2.0,
                (second_pos.y - first_pos.y) / 2.0,
                0.0,
            );
            self.move_entity(&constraint.first_entity, &correction);
            self.move_entity(constraint.second_entity.as_ref().unwrap(), &Point::new(-correction.x, -correction.y, 0.0));
        }
    }

    fn apply_horizontal_correction(&mut self, constraint: &ConstraintEntity) {
        if let Some(state) = self.entities.get_mut(&constraint.first_entity) {
            state.angle = 0.0;
        }
    }

    fn apply_vertical_correction(&mut self, constraint: &ConstraintEntity) {
        if let Some(state) = self.entities.get_mut(&constraint.first_entity) {
            state.angle = PI / 2.0;
        }
    }

    fn get_entity_position(&self, entity_id: &str) -> Option<Point> {
        self.entities.get(entity_id).map(|s| s.position)
    }

    fn get_entity_direction(&self, entity_id: &str) -> Option<Point> {
        self.entities.get(entity_id).map(|s| {
            Point::new(s.angle.cos(), s.angle.sin(), 0.0)
        })
    }

    fn get_entity_center(&self, entity_id: &str) -> Option<Point> {
        self.entities.get(entity_id).map(|s| s.center)
    }

    fn get_entity_points(&self, entity_id: &str) -> Option<(Point, Point)> {
        self.entities.get(entity_id).map(|s| {
            let start = Point::new(
                s.position.x + (s.angle - s.length / 2.0).cos() * s.length,
                s.position.y + (s.angle - s.length / 2.0).sin() * s.length,
                0.0,
            );
            let end = Point::new(
                s.position.x + (s.angle + s.length / 2.0).cos() * s.length,
                s.position.y + (s.angle + s.length / 2.0).sin() * s.length,
                0.0,
            );
            (start, end)
        })
    }

    fn get_entity_length(&self, entity_id: &str) -> f64 {
        self.entities.get(entity_id).map(|s| s.length).unwrap_or(0.0)
    }

    fn get_entity_radius(&self, entity_id: &str) -> f64 {
        self.entities.get(entity_id).map(|s| s.radius).unwrap_or(0.0)
    }

    fn get_entity_closest_point(&self, entity_id: &str, point: &Point) -> Point {
        if let Some(state) = self.entities.get(entity_id) {
            let dx = point.x - state.position.x;
            let dy = point.y - state.position.y;
            let angle = dy.atan2(dx);
            Point::new(
                state.position.x + angle.cos() * state.length / 2.0,
                state.position.y + angle.sin() * state.length / 2.0,
                0.0,
            )
        } else {
            *point
        }
    }

    fn move_entity(&mut self, entity_id: &str, delta: &Point) {
        if let Some(state) = self.entities.get_mut(entity_id) {
            state.position.x += delta.x;
            state.position.y += delta.y;
            state.center.x += delta.x;
            state.center.y += delta.y;
        }
    }

    fn calculate_dof(&mut self) {
        let mut total_dof = 0;
        let mut constrained_dof = 0;

        for entity in self.entities.values() {
            total_dof += 3;
            constrained_dof += entity.constrained_dof;
        }

        self.state.constrained_dof = constrained_dof;
        self.state.free_dof = total_dof - constrained_dof;
    }

    pub fn add_entity(&mut self, entity_id: &str, position: Point, angle: f64, length: f64) {
        self.entities.insert(
            entity_id.to_string(),
            EntityState {
                position,
                center: position,
                angle,
                length,
                radius: 0.0,
                constrained_dof: 0,
                drag_distance: 0.0,
            },
        );
    }

    pub fn set_entity_position(&mut self, entity_id: &str, position: Point) {
        if let Some(state) = self.entities.get_mut(entity_id) {
            let dx = position.x - state.position.x;
            let dy = position.y - state.position.y;
            state.position = position;
            state.center.x += dx;
            state.center.y += dy;
        }
    }

    pub fn clear_constraints(&mut self) {
        self.constraints.clear();
        for state in self.entities.values_mut() {
            state.constrained_dof = 0;
        }
        if self.auto_solve {
            self.solve();
        }
    }

    pub fn constraint_count(&self) -> usize {
        self.constraints.len()
    }

    pub fn is_fully_constrained(&self) -> bool {
        self.state.free_dof == 0
    }

    pub fn is_over_constrained(&self) -> bool {
        (self.state.constrained_dof as usize) > self.entities.len() * 3
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityState {
    pub position: Point,
    pub center: Point,
    pub angle: f64,
    pub length: f64,
    pub radius: f64,
    pub constrained_dof: u32,
    pub drag_distance: f64,
}

impl Default for EntityState {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityState {
    pub fn new() -> Self {
        Self {
            position: Point::origin(),
            center: Point::origin(),
            angle: 0.0,
            length: 0.0,
            radius: 0.0,
            constrained_dof: 0,
            drag_distance: 0.0,
        }
    }
}

pub type Point = crate::geometry::Point;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_entity_creation() {
        let constraint = ConstraintEntity::new()
            .with_type(GeometricConstraint::Coincident)
            .with_first_entity("line1")
            .with_second_entity("line2");

        assert_eq!(constraint.constraint_type, GeometricConstraint::Coincident);
        assert_eq!(constraint.first_entity, "line1");
        assert_eq!(constraint.second_entity, Some("line2".to_string()));
    }

    #[test]
    fn test_solver_creation() {
        let solver = GeometricSolver::new();
        assert_eq!(solver.constraint_count(), 0);
        assert!(!solver.is_fully_constrained());
    }

    #[test]
    fn test_add_constraint() {
        let mut solver = GeometricSolver::new();
        solver.auto_solve = false;

        let constraint = ConstraintEntity::new()
            .with_type(GeometricConstraint::Coincident)
            .with_first_entity("line1")
            .with_second_entity("line2");

        assert!(solver.add_constraint(constraint));
        assert_eq!(solver.constraint_count(), 1);
    }

    #[test]
    fn test_validate_constraint() {
        let solver = GeometricSolver::new();

        let valid_constraint = ConstraintEntity::new()
            .with_type(GeometricConstraint::Coincident)
            .with_first_entity("line1")
            .with_second_entity("line2");

        let invalid_constraint = ConstraintEntity::new()
            .with_type(GeometricConstraint::Horizontal)
            .with_first_entity("line1")
            .with_second_entity("line2");

        assert!(solver.validate_constraint(&valid_constraint));
        assert!(!solver.validate_constraint(&invalid_constraint));
    }

    #[test]
    fn test_remove_constraint() {
        let mut solver = GeometricSolver::new();
        solver.auto_solve = false;

        let constraint = ConstraintEntity::new()
            .with_type(GeometricConstraint::Coincident)
            .with_first_entity("line1")
            .with_second_entity("line2");

        solver.add_constraint(constraint.clone());
        assert_eq!(solver.constraint_count(), 1);

        solver.remove_constraint(&constraint.id);
        assert_eq!(solver.constraint_count(), 0);
    }

    #[test]
    fn test_add_entity() {
        let mut solver = GeometricSolver::new();
        solver.add_entity("line1", Point::new(0.0, 0.0, 0.0), 0.0, 10.0);

        assert!(solver.entities.contains_key("line1"));
        let state = solver.entities.get("line1").unwrap();
        assert!((state.position.x - 0.0).abs() < 1e-10);
        assert!((state.length - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_constraint_types() {
        assert_eq!(GeometricConstraint::Coincident.name(), "Coincident");
        assert_eq!(GeometricConstraint::Perpendicular.name(), "Perpendicular");
        assert_eq!(GeometricConstraint::Parallel.name(), "Parallel");
        assert_eq!(GeometricConstraint::Tangent.name(), "Tangent");
        assert_eq!(GeometricConstraint::Horizontal.name(), "Horizontal");
        assert_eq!(GeometricConstraint::Vertical.name(), "Vertical");
    }

    #[test]
    fn test_constraint_icons() {
        assert_eq!(GeometricConstraint::Coincident.icon(), "⭕");
        assert_eq!(GeometricConstraint::Horizontal.icon(), "—");
        assert_eq!(GeometricConstraint::Vertical.icon(), "|");
        assert_eq!(GeometricConstraint::Fix.icon(), "📌");
    }

    #[test]
    fn test_solver_state() {
        let state = SolverState::new();
        assert!(state.solved);
        assert_eq!(state.iterations, 0);
        assert!((state.error - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_constraints_for_entity() {
        let mut solver = GeometricSolver::new();
        solver.auto_solve = false;

        solver.add_constraint(
            ConstraintEntity::new()
                .with_type(GeometricConstraint::Coincident)
                .with_first_entity("line1")
                .with_second_entity("line2"),
        );

        solver.add_constraint(
            ConstraintEntity::new()
                .with_type(GeometricConstraint::Horizontal)
                .with_first_entity("line1"),
        );

        let constraints = solver.constraints_for_entity("line1");
        assert_eq!(constraints.len(), 2);
    }

    #[test]
    fn test_clear_constraints() {
        let mut solver = GeometricSolver::new();
        solver.auto_solve = false;

        solver.add_constraint(ConstraintEntity::new()
            .with_type(GeometricConstraint::Coincident)
            .with_first_entity("line1")
            .with_second_entity("line2"));

        solver.add_constraint(ConstraintEntity::new()
            .with_type(GeometricConstraint::Horizontal)
            .with_first_entity("line1"));

        assert_eq!(solver.constraint_count(), 2);
        solver.clear_constraints();
        assert_eq!(solver.constraint_count(), 0);
    }

    #[test]
    fn test_dof_calculation() {
        let mut solver = GeometricSolver::new();
        solver.add_entity("line1", Point::new(0.0, 0.0, 0.0), 0.0, 10.0);
        solver.add_entity("line2", Point::new(10.0, 0.0, 0.0), PI / 2.0, 10.0);

        solver.auto_solve = false;
        solver.add_constraint(ConstraintEntity::new()
            .with_type(GeometricConstraint::Coincident)
            .with_first_entity("line1")
            .with_second_entity("line2"));

        solver.solve();

        assert!(solver.state.free_dof >= 0);
    }
}
