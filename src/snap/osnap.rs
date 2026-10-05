use serde::{Serialize, Deserialize};
use std::fmt;
use super::snap_point::SnapCalculator;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OsnapMode {
    Endpoint,
    Midpoint,
    Center,
    Node,
    Quadrant,
    Intersection,
    Perpendicular,
    Tangent,
    Parallel,
    Extension,
    Insertion,
    Nearest,
    ApparentIntersection,
    None,
}

impl Default for OsnapMode {
    fn default() -> Self {
        OsnapMode::None
    }
}

impl OsnapMode {
    pub fn to_snap_type(&self) -> super::snap_point::SnapType {
        match self {
            OsnapMode::Endpoint => super::snap_point::SnapType::EndPoint,
            OsnapMode::Midpoint => super::snap_point::SnapType::MidPoint,
            OsnapMode::Center => super::snap_point::SnapType::Center,
            OsnapMode::Node => super::snap_point::SnapType::Node,
            OsnapMode::Quadrant => super::snap_point::SnapType::Quadrant,
            OsnapMode::Intersection => super::snap_point::SnapType::Intersection,
            OsnapMode::Perpendicular => super::snap_point::SnapType::Perpendicular,
            OsnapMode::Tangent => super::snap_point::SnapType::Tangent,
            OsnapMode::Parallel => super::snap_point::SnapType::Parallel,
            OsnapMode::Extension => super::snap_point::SnapType::Extension,
            OsnapMode::Nearest => super::snap_point::SnapType::Nearest,
            OsnapMode::ApparentIntersection => super::snap_point::SnapType::Intersection,
            OsnapMode::Insertion => super::snap_point::SnapType::Nearest,
            OsnapMode::None => super::snap_point::SnapType::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsnapSettings {
    pub display_osnap_marker: bool,
    pub display_osnap_tooltip: bool,
    pub display_osnap_cursor_tip: bool,
    pub marker_color: (u8, u8, u8),
    pub marker_size: f64,
    pub tooltip_color: (u8, u8, u8),
    pub aperture_box_size: f64,
    pub is_running: bool,
}

impl Default for OsnapSettings {
    fn default() -> Self {
        Self {
            display_osnap_marker: true,
            display_osnap_tooltip: true,
            display_osnap_cursor_tip: true,
            marker_color: (0, 255, 0),
            marker_size: 5.0,
            tooltip_color: (255, 255, 0),
            aperture_box_size: 30.0,
            is_running: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsnapMarker {
    pub position: crate::geometry::Point,
    pub mode: OsnapMode,
    pub entity_id: super::super::data_structure::ObjectId,
    pub display_text: String,
}

impl Default for OsnapMarker {
    fn default() -> Self {
        Self {
            position: crate::geometry::Point::new(0.0, 0.0, 0.0),
            mode: OsnapMode::None,
            entity_id: super::super::data_structure::ObjectId::null(),
            display_text: String::new(),
        }
    }
}

impl OsnapMarker {
    pub fn new(position: crate::geometry::Point, mode: OsnapMode, entity_id: super::super::data_structure::ObjectId) -> Self {
        let display_text = format!("{:?}", mode);
        Self {
            position,
            mode,
            entity_id,
            display_text,
        }
    }
}

pub struct OsnapTracker {
    settings: OsnapSettings,
    active_modes: Vec<OsnapMode>,
    current_marker: Option<OsnapMarker>,
    tooltip_text: String,
}

impl Default for OsnapTracker {
    fn default() -> Self {
        Self {
            settings: OsnapSettings::default(),
            active_modes: vec![
                OsnapMode::Endpoint,
                OsnapMode::Midpoint,
                OsnapMode::Center,
                OsnapMode::Intersection,
            ],
            current_marker: None,
            tooltip_text: String::new(),
        }
    }
}

impl OsnapTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enable(&mut self, enabled: bool) {
        self.settings.is_running = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.settings.is_running
    }

    pub fn enable_mode(&mut self, mode: OsnapMode, enabled: bool) {
        if enabled {
            if !self.active_modes.contains(&mode) {
                self.active_modes.push(mode);
            }
        } else {
            self.active_modes.retain(|&m| m != mode);
        }
    }

    pub fn get_active_modes(&self) -> &[OsnapMode] {
        &self.active_modes
    }

    pub fn set_marker(&mut self, marker: Option<OsnapMarker>) {
        self.current_marker = marker;
    }

    pub fn get_marker(&self) -> Option<&OsnapMarker> {
        self.current_marker.as_ref()
    }

    pub fn get_tooltip_text(&self) -> &str {
        &self.tooltip_text
    }

    pub fn set_tooltip_text(&mut self, text: impl Into<String>) {
        self.tooltip_text = text.into();
    }

    pub fn settings(&self) -> &OsnapSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut OsnapSettings {
        &mut self.settings
    }

    pub fn track(
        &mut self,
        cursor_point: crate::geometry::Point,
        entities: &[super::super::data_structure::Entity],
        snap_manager: &super::snap_point::SnapManager,
    ) -> Option<OsnapMarker> {
        if !self.settings.is_running {
            return None;
        }

        let mut best_marker: Option<OsnapMarker> = None;
        let mut best_distance = self.settings.aperture_box_size;

        let calculator = super::snap_point::SnapCalculatorImpl;

        for entity in entities {
            for mode in &self.active_modes {
                let snapshot = match mode {
                    OsnapMode::Endpoint => calculator.calculate_endpoint(entity),
                    OsnapMode::Midpoint => calculator.calculate_midpoint(entity),
                    OsnapMode::Center => calculator.calculate_center(entity),
                    OsnapMode::Node => calculator.calculate_node(entity),
                    OsnapMode::Quadrant => calculator.calculate_quadrant(entity).first().cloned(),
                    OsnapMode::Perpendicular => calculator.calculate_perpendicular(entity, cursor_point),
                    OsnapMode::Tangent => calculator.calculate_tangent(entity, cursor_point),
                    OsnapMode::Nearest => calculator.calculate_nearest(entity, cursor_point),
                    OsnapMode::Extension => calculator.calculate_extension(entity, cursor_point, 10.0),
                    _ => None,
                };

                if let Some(snap) = snapshot {
                    let distance = snap.point.distance_to(&cursor_point);
                    if distance < best_distance {
                        best_marker = Some(OsnapMarker::new(snap.point, *mode, snap.entity_id));
                        best_distance = distance;
                    }
                }
            }
        }

        if let Some(ref marker) = best_marker {
            self.set_tooltip_text(format!("{}: {:?}", marker.position, marker.mode));
        } else {
            self.set_tooltip_text(String::new());
        }

        self.current_marker = best_marker.clone();
        best_marker
    }

    pub fn clear(&mut self) {
        self.current_marker = None;
        self.tooltip_text.clear();
    }
}

impl fmt::Display for OsnapTracker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "OsnapTracker(enabled={}, modes={})",
            self.settings.is_running,
            self.active_modes.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_osnap_mode_conversion() {
        assert_eq!(OsnapMode::Endpoint.to_snap_type(), super::super::snap_point::SnapType::EndPoint);
        assert_eq!(OsnapMode::Midpoint.to_snap_type(), super::super::snap_point::SnapType::MidPoint);
        assert_eq!(OsnapMode::Center.to_snap_type(), super::super::snap_point::SnapType::Center);
    }

    #[test]
    fn test_osnap_settings() {
        let settings = OsnapSettings::default();
        assert!(settings.is_running);
        assert_eq!(settings.marker_size, 5.0);
    }

    #[test]
    fn test_osnap_tracker() {
        let mut tracker = OsnapTracker::new();
        assert!(!tracker.is_enabled());

        tracker.enable(true);
        assert!(tracker.is_enabled());

        tracker.enable_mode(OsnapMode::Endpoint, false);
        assert!(!tracker.get_active_modes().contains(&OsnapMode::Endpoint));
    }
}
