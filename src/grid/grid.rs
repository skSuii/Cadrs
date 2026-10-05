use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GridType {
    Rectangular,
    Isometric,
    Polar,
    None,
}

impl Default for GridType {
    fn default() -> Self {
        GridType::Rectangular
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GridSnapStyle {
    Standard,
    Isometric,
}

impl Default for GridSnapStyle {
    fn default() -> Self {
        GridSnapStyle::Standard
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridSettings {
    pub grid_on: bool,
    pub grid_snap: bool,
    pub grid_type: GridType,
    pub grid_spacing_x: f64,
    pub grid_spacing_y: f64,
    pub grid_spacing_z: f64,
    pub grid_lines: u32,
    pub grid_major_lines: u32,
    pub grid_bounds: (f64, f64, f64, f64),
    pub grid_color: (u8, u8, u8),
    pub grid_axis_color: (u8, u8, u8),
    pub grid_line_weight: i32,
    pub display_grid_beyond_limits: bool,
    pub follow_ucs: bool,
    pub grid_snap_style: GridSnapStyle,
    pub grid_snap_angle: f64,
    pub polar_snap_spacing: f64,
    pub display_polar_tracking_path: bool,
}

impl Default for GridSettings {
    fn default() -> Self {
        Self {
            grid_on: true,
            grid_snap: true,
            grid_type: GridType::Rectangular,
            grid_spacing_x: 1.0,
            grid_spacing_y: 1.0,
            grid_spacing_z: 1.0,
            grid_lines: 100,
            grid_major_lines: 10,
            grid_bounds: (-100.0, -100.0, 100.0, 100.0),
            grid_color: (128, 128, 128),
            grid_axis_color: (255, 0, 0),
            grid_line_weight: 0,
            display_grid_beyond_limits: false,
            follow_ucs: true,
            grid_snap_style: GridSnapStyle::Standard,
            grid_snap_angle: 0.0,
            polar_snap_spacing: 90.0,
            display_polar_tracking_path: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridPoint {
    pub x: f64,
    pub y: f64,
    pub is_major: bool,
    pub is_axis: bool,
}

impl GridPoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            is_major: false,
            is_axis: false,
        }
    }

    pub fn as_major(mut self) -> Self {
        self.is_major = true;
        self
    }

    pub fn as_axis(mut self) -> Self {
        self.is_axis = true;
        self
    }

    pub fn is_major(mut self, value: bool) -> Self {
        self.is_major = value;
        self
    }

    pub fn is_axis(mut self, value: bool) -> Self {
        self.is_axis = value;
        self
    }
}

pub struct Grid {
    settings: GridSettings,
    grid_points: Vec<GridPoint>,
    visible_bounds: (f64, f64, f64, f64),
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            settings: GridSettings::default(),
            grid_points: Vec::new(),
            visible_bounds: (-100.0, -100.0, 100.0, 100.0),
        }
    }
}

impl Grid {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_spacing(spacing_x: f64, spacing_y: f64) -> Self {
        let mut grid = Self::default();
        grid.settings.grid_spacing_x = spacing_x;
        grid.settings.grid_spacing_y = spacing_y;
        grid
    }

    pub fn enable(&mut self, enabled: bool) {
        self.settings.grid_on = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.settings.grid_on
    }

    pub fn enable_snap(&mut self, enabled: bool) {
        self.settings.grid_snap = enabled;
    }

    pub fn is_snap_enabled(&self) -> bool {
        self.settings.grid_snap
    }

    pub fn set_spacing(&mut self, x: f64, y: f64) {
        self.settings.grid_spacing_x = x;
        self.settings.grid_spacing_y = y;
    }

    pub fn get_spacing(&self) -> (f64, f64) {
        (self.settings.grid_spacing_x, self.settings.grid_spacing_y)
    }

    pub fn set_major_spacing(&mut self, major: u32) {
        self.settings.grid_major_lines = major;
    }

    pub fn get_major_spacing(&self) -> u32 {
        self.settings.grid_major_lines
    }

    pub fn set_bounds(&mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) {
        self.settings.grid_bounds = (min_x, min_y, max_x, max_y);
        self.visible_bounds = (min_x, min_y, max_x, max_y);
    }

    pub fn set_visible_bounds(&mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) {
        self.visible_bounds = (min_x, min_y, max_x, max_y);
    }

    pub fn set_type(&mut self, grid_type: GridType) {
        self.settings.grid_type = grid_type;
    }

    pub fn get_type(&self) -> GridType {
        self.settings.grid_type
    }

    pub fn set_color(&mut self, color: (u8, u8, u8)) {
        self.settings.grid_color = color;
    }

    pub fn set_axis_color(&mut self, color: (u8, u8, u8)) {
        self.settings.grid_axis_color = color;
    }

    pub fn settings(&self) -> &GridSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut GridSettings {
        &mut self.settings
    }

    pub fn generate_grid(&mut self) {
        self.grid_points.clear();

        let (min_x, min_y, max_x, max_y) = self.visible_bounds;
        let spacing_x = self.settings.grid_spacing_x;
        let spacing_y = self.settings.grid_spacing_y;
        let major_spacing = self.settings.grid_major_lines;

        let start_x = (min_x / spacing_x).floor() * spacing_x;
        let start_y = (min_y / spacing_y).floor() * spacing_y;
        let end_x = (max_x / spacing_x).ceil() * spacing_x;
        let end_y = (max_y / spacing_y).ceil() * spacing_y;

        let mut x = start_x;
        let mut x_index = 0;
        while x <= end_x {
            let is_major = x_index % major_spacing == 0;
            let is_axis = x == 0.0;

            let mut y = start_y;
            let mut y_index = 0;
            while y <= end_y {
                let point = GridPoint::new(x, y)
                    .as_major()
                    .is_major(is_major && y_index % major_spacing == 0)
                    .is_axis(is_axis || y == 0.0);

                self.grid_points.push(point);

                y += spacing_y;
                y_index += 1;
            }

            x += spacing_x;
            x_index += 1;
        }
    }

    pub fn get_grid_points(&self) -> &[GridPoint] {
        &self.grid_points
    }

    pub fn snap_to_grid(&self, point: crate::geometry::Point) -> crate::geometry::Point {
        if !self.settings.grid_snap {
            return point;
        }

        let spacing_x = self.settings.grid_spacing_x;
        let spacing_y = self.settings.grid_spacing_y;

        let snapped_x = (point.x / spacing_x).round() * spacing_x;
        let snapped_y = (point.y / spacing_y).round() * spacing_y;

        crate::geometry::Point::new(snapped_x, snapped_y, point.z)
    }

    pub fn get_nearest_grid_point(&self, point: crate::geometry::Point) -> Option<GridPoint> {
        let mut nearest: Option<GridPoint> = None;
        let mut min_distance = f64::MAX;

        for grid_point in &self.grid_points {
            let distance = ((grid_point.x - point.x).powi(2) + (grid_point.y - point.y).powi(2)).sqrt();
            if distance < min_distance {
                min_distance = distance;
                nearest = Some(grid_point.clone());
            }
        }

        if min_distance < self.settings.grid_spacing_x.max(self.settings.grid_spacing_y) {
            nearest
        } else {
            None
        }
    }

    pub fn toggle(&mut self) -> bool {
        self.settings.grid_on = !self.settings.grid_on;
        self.settings.grid_on
    }

    pub fn toggle_snap(&mut self) -> bool {
        self.settings.grid_snap = !self.settings.grid_snap;
        self.settings.grid_snap
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Grid(enabled={}, points={}, spacing=({}, {}))",
            self.settings.grid_on,
            self.grid_points.len(),
            self.settings.grid_spacing_x,
            self.settings.grid_spacing_y
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OrthoMode {
    Off,
    Horizontal,
    Vertical,
    Both,
}

impl Default for OrthoMode {
    fn default() -> Self {
        OrthoMode::Off
    }
}

pub struct OrthoSettings {
    pub mode: OrthoMode,
    pub ortho_angle: f64,
    pub ortho_top_direction: f64,
    pub display_tool_tip: bool,
}

impl Default for OrthoSettings {
    fn default() -> Self {
        Self {
            mode: OrthoMode::Off,
            ortho_angle: 90.0,
            ortho_top_direction: 90.0,
            display_tool_tip: true,
        }
    }
}

pub struct Ortho {
    settings: OrthoSettings,
}

impl Default for Ortho {
    fn default() -> Self {
        Self {
            settings: OrthoSettings::default(),
        }
    }
}

impl Ortho {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enable(&mut self, enabled: bool) {
        self.settings.mode = if enabled { OrthoMode::Both } else { OrthoMode::Off };
    }

    pub fn disable(&mut self) {
        self.settings.mode = OrthoMode::Off;
    }

    pub fn is_enabled(&self) -> bool {
        self.settings.mode != OrthoMode::Off
    }

    pub fn set_mode(&mut self, mode: OrthoMode) {
        self.settings.mode = mode;
    }

    pub fn get_mode(&self) -> OrthoMode {
        self.settings.mode
    }

    pub fn toggle(&mut self) -> bool {
        if self.settings.mode == OrthoMode::Off {
            self.settings.mode = OrthoMode::Both;
            true
        } else {
            self.settings.mode = OrthoMode::Off;
            false
        }
    }

    pub fn constrain(
        &self,
        from_point: crate::geometry::Point,
        to_point: crate::geometry::Point,
    ) -> crate::geometry::Point {
        if self.settings.mode == OrthoMode::Off {
            return to_point;
        }

        let dx = to_point.x - from_point.x;
        let dy = to_point.y - from_point.y;

        match self.settings.mode {
            OrthoMode::Horizontal => {
                crate::geometry::Point::new(to_point.x, from_point.y, to_point.z)
            }
            OrthoMode::Vertical => {
                crate::geometry::Point::new(from_point.x, to_point.y, to_point.z)
            }
            OrthoMode::Both => {
                if dx.abs() >= dy.abs() {
                    crate::geometry::Point::new(to_point.x, from_point.y, to_point.z)
                } else {
                    crate::geometry::Point::new(from_point.x, to_point.y, to_point.z)
                }
            }
            OrthoMode::Off => to_point,
        }
    }

    pub fn get_directions(&self) -> Vec<f64> {
        match self.settings.mode {
            OrthoMode::Horizontal => vec![0.0, 180.0],
            OrthoMode::Vertical => vec![90.0, 270.0],
            OrthoMode::Both => vec![0.0, 90.0, 180.0, 270.0],
            OrthoMode::Off => vec![],
        }
    }
}

impl fmt::Display for Ortho {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Ortho(mode={:?})", self.settings.mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_creation() {
        let grid = Grid::new_with_spacing(1.0, 1.0);
        assert!(grid.is_enabled());
        assert!(grid.is_snap_enabled());
    }

    #[test]
    fn test_grid_snap() {
        let grid = Grid::new_with_spacing(1.0, 1.0);
        let point = crate::geometry::Point::new(1.3, 2.7, 0.0);
        let snapped = grid.snap_to_grid(point);
        assert_eq!(snapped.x, 1.0);
        assert_eq!(snapped.y, 3.0);
    }

    #[test]
    fn test_ortho_constrain() {
        let ortho = Ortho::new();
        let from = crate::geometry::Point::new(0.0, 0.0, 0.0);
        let to = crate::geometry::Point::new(1.0, 1.0, 0.0);

        let constrained = ortho.constrain(from, to);
        assert_eq!(constrained.x, 1.0);
        assert_eq!(constrained.y, 0.0);
    }

    #[test]
    fn test_ortho_toggle() {
        let mut ortho = Ortho::new();
        assert!(!ortho.is_enabled());

        ortho.enable(true);
        assert!(ortho.is_enabled());

        ortho.disable();
        assert!(!ortho.is_enabled());

        assert!(ortho.toggle());
        assert!(ortho.is_enabled());

        assert!(!ortho.toggle());
        assert!(!ortho.is_enabled());
    }
}
