use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::time::SystemTime;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OleObjectType {
    Embed,
    Link,
    Olev1,
    Olev2,
    Ocx,
    Package,
}

impl Default for OleObjectType {
    fn default() -> Self {
        OleObjectType::Embed
    }
}

impl OleObjectType {
    pub fn name(&self) -> &str {
        match self {
            OleObjectType::Embed => "Embedded",
            OleObjectType::Link => "Linked",
            OleObjectType::Olev1 => "OLE 1.0",
            OleObjectType::Olev2 => "OLE 2.0",
            OleObjectType::Ocx => "OCX Control",
            OleObjectType::Package => "Package",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            OleObjectType::Embed => "📎",
            OleObjectType::Link => "🔗",
            OleObjectType::Olev1 => "📄",
            OleObjectType::Olev2 => "📄",
            OleObjectType::Ocx => "🎛️",
            OleObjectType::Package => "📦",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OleObjectState {
    Closed,
    Open,
    Linked,
    Updated,
    Broken,
}

impl Default for OleObjectState {
    fn default() -> Self {
        OleObjectState::Closed
    }
}

impl OleObjectState {
    pub fn name(&self) -> &str {
        match self {
            OleObjectState::Closed => "Closed",
            OleObjectState::Open => "Open",
            OleObjectState::Linked => "Linked",
            OleObjectState::Updated => "Updated",
            OleObjectState::Broken => "Broken",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OleObject {
    pub id: String,
    pub object_type: OleObjectType,
    pub prog_id: String,
    pub class_id: String,
    pub source_file: PathBuf,
    pub source_item: String,
    pub link_path: Option<PathBuf>,
    pub position: crate::geometry::Point,
    pub size: (f64, f64),
    pub rotation: f64,
    pub is_locked: bool,
    pub is_visible: bool,
    pub is_printable: bool,
    pub is_undoable: bool,
    pub border_style: BorderStyle,
    pub border_width: i32,
    pub scale: (f64, f64),
    pub aspect_ratio: f64,
    pub transparency: f64,
    pub ole_data: Vec<u8>,
    pub preview_data: Option<Vec<u8>>,
    pub object_state: OleObjectState,
    pub last_update: SystemTime,
    pub application_name: String,
}

impl Default for OleObject {
    fn default() -> Self {
        Self::new()
    }
}

impl OleObject {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            object_type: OleObjectType::Embed,
            prog_id: String::new(),
            class_id: String::new(),
            source_file: PathBuf::new(),
            source_item: String::new(),
            link_path: None,
            position: crate::geometry::Point::origin(),
            size: (100.0, 100.0),
            rotation: 0.0,
            is_locked: false,
            is_visible: true,
            is_printable: true,
            is_undoable: true,
            border_style: BorderStyle::None,
            border_width: 0,
            scale: (1.0, 1.0),
            aspect_ratio: 1.0,
            transparency: 0.0,
            ole_data: Vec::new(),
            preview_data: None,
            object_state: OleObjectState::Closed,
            last_update: SystemTime::UNIX_EPOCH,
            application_name: String::new(),
        }
    }

    pub fn with_prog_id(mut self, prog_id: &str) -> Self {
        self.prog_id = prog_id.to_string();
        self
    }

    pub fn with_source(mut self, source: &str) -> Self {
        self.source_file = PathBuf::from(source);
        self
    }

    pub fn at_position(mut self, position: crate::geometry::Point) -> Self {
        self.position = position;
        self
    }

    pub fn with_size(mut self, width: f64, height: f64) -> Self {
        self.size = (width, height);
        self
    }

    pub fn set_position(&mut self, position: crate::geometry::Point) {
        self.position = position;
    }

    pub fn set_size(&mut self, width: f64, height: f64) {
        self.size = (width, height);
        self.aspect_ratio = if height > 0.0 { width / height } else { 1.0 };
    }

    pub fn set_rotation(&mut self, rotation: f64) {
        self.rotation = rotation;
    }

    pub fn set_lock(&mut self, locked: bool) {
        self.is_locked = locked;
    }

    pub fn set_visibility(&mut self, visible: bool) {
        self.is_visible = visible;
    }

    pub fn set_printable(&mut self, printable: bool) {
        self.is_printable = printable;
    }

    pub fn set_border(&mut self, style: BorderStyle, width: i32) {
        self.border_style = style;
        self.border_width = width;
    }

    pub fn set_scale(&mut self, scale_x: f64, scale_y: f64) {
        self.scale = (scale_x, scale_y);
    }

    pub fn set_transparency(&mut self, transparency: f64) {
        self.transparency = transparency.clamp(0.0, 100.0);
    }

    pub fn update_link(&mut self) -> bool {
        if let Some(link_path) = &self.link_path {
            if link_path.exists() {
                self.object_state = OleObjectState::Updated;
                self.last_update = SystemTime::now();
                true
            } else {
                self.object_state = OleObjectState::Broken;
                false
            }
        } else {
            false
        }
    }

    pub fn break_link(&mut self) {
        self.link_path = None;
        self.source_item.clear();
        self.object_state = OleObjectState::Closed;
    }

    pub fn set_aspect_ratio(&mut self, ratio: f64) {
        self.aspect_ratio = ratio;
        self.size.1 = self.size.0 / ratio;
    }

    pub fn contains_point(&self, point: &crate::geometry::Point) -> bool {
        let half_width = self.size.0 / 2.0;
        let half_height = self.size.1 / 2.0;

        point.x >= self.position.x - half_width
            && point.x <= self.position.x + half_width
            && point.y >= self.position.y - half_height
            && point.y <= self.position.y + half_height
    }
}

impl fmt::Display for OleObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "OLE: {} ({}) at ({}, {})",
            self.prog_id,
            self.object_type.name(),
            self.position.x,
            self.position.y
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BorderStyle {
    None,
    Thin,
    Medium,
    Thick,
    Custom(i32),
}

impl Default for BorderStyle {
    fn default() -> Self {
        BorderStyle::None
    }
}

impl BorderStyle {
    pub fn name(&self) -> &str {
        match self {
            BorderStyle::None => "None",
            BorderStyle::Thin => "Thin",
            BorderStyle::Medium => "Medium",
            BorderStyle::Thick => "Thick",
            BorderStyle::Custom(_) => "Custom",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OleApplication {
    pub name: String,
    pub prog_id: String,
    pub class_id: String,
    pub icon_path: PathBuf,
    pub extensions: Vec<String>,
    pub is_installed: bool,
    pub version: String,
}

impl Default for OleApplication {
    fn default() -> Self {
        Self::new()
    }
}

impl OleApplication {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            prog_id: String::new(),
            class_id: String::new(),
            icon_path: PathBuf::new(),
            extensions: Vec::new(),
            is_installed: false,
            version: String::new(),
        }
    }

    pub fn with_prog_id(mut self, prog_id: &str) -> Self {
        self.prog_id = prog_id.to_string();
        self
    }

    pub fn add_extension(&mut self, ext: &str) {
        if !self.extensions.contains(&ext.to_string()) {
            self.extensions.push(ext.to_string());
        }
    }
}

#[derive(Debug, Clone)]
pub struct OleManager {
    objects: Vec<OleObject>,
    applications: Vec<OleApplication>,
    default_apps: Vec<String>,
    is_auto_update_links: bool,
    is_update_remote: bool,
    update_interval: u64,
    last_update_check: SystemTime,
}

impl Default for OleManager {
    fn default() -> Self {
        Self::new()
    }
}

impl OleManager {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            applications: Vec::new(),
            default_apps: vec![
                "Excel.Application".to_string(),
                "Word.Application".to_string(),
                "PowerPoint.Application".to_string(),
            ],
            is_auto_update_links: true,
            is_update_remote: false,
            update_interval: 60,
            last_update_check: SystemTime::UNIX_EPOCH,
        }
    }

    pub fn create_object(&mut self, prog_id: &str, position: crate::geometry::Point, size: (f64, f64)) -> &mut OleObject {
        let object = OleObject::new()
            .with_prog_id(prog_id)
            .at_position(position)
            .with_size(size.0, size.1);

        self.objects.push(object);
        self.objects.last_mut().unwrap()
    }

    pub fn embed_file(&mut self, file_path: &str, position: crate::geometry::Point) -> Result<&mut OleObject, String> {
        if !PathBuf::from(file_path).exists() {
            return Err("File not found".to_string());
        }

        let object = OleObject::new()
            .with_source(file_path)
            .at_position(position)
            .with_size(100.0, 100.0);

        self.objects.push(object);
        Ok(self.objects.last_mut().unwrap())
    }

    pub fn link_file(&mut self, file_path: &str, position: crate::geometry::Point) -> Result<&mut OleObject, String> {
        let path = PathBuf::from(file_path);
        if !path.exists() {
            return Err("File not found".to_string());
        }

        let mut object = OleObject::new()
            .with_source(file_path)
            .at_position(position)
            .with_size(100.0, 100.0);

        object.object_type = OleObjectType::Link;
        object.link_path = Some(path);

        self.objects.push(object);
        Ok(self.objects.last_mut().unwrap())
    }

    pub fn add(&mut self, object: OleObject) {
        self.objects.push(object);
    }

    pub fn get(&self, id: &str) -> Option<&OleObject> {
        self.objects.iter().find(|o| o.id == id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut OleObject> {
        self.objects.iter_mut().find(|o| o.id == id)
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let original_len = self.objects.len();
        self.objects.retain(|o| o.id != id);
        self.objects.len() != original_len
    }

    pub fn find_by_position(&self, point: &crate::geometry::Point) -> Option<&OleObject> {
        self.objects.iter().find(|o| o.contains_point(point))
    }

    pub fn update_all_links(&mut self) -> usize {
        let mut updated = 0;
        for object in &mut self.objects {
            if object.object_type == OleObjectType::Link {
                if object.update_link() {
                    updated += 1;
                }
            }
        }
        self.last_update_check = SystemTime::now();
        updated
    }

    pub fn break_all_links(&mut self) {
        for object in &mut self.objects {
            if object.object_type == OleObjectType::Link {
                object.break_link();
            }
        }
    }

    pub fn update_link(&mut self, object_id: &str) -> bool {
        if let Some(object) = self.get_mut(object_id) {
            object.update_link()
        } else {
            false
        }
    }

    pub fn break_link(&mut self, object_id: &str) {
        if let Some(object) = self.get_mut(object_id) {
            object.break_link();
        }
    }

    pub fn set_auto_update(&mut self, auto: bool) {
        self.is_auto_update_links = auto;
    }

    pub fn set_update_remote(&mut self, remote: bool) {
        self.is_update_remote = remote;
    }

    pub fn object_count(&self) -> usize {
        self.objects.len()
    }

    pub fn objects_by_type(&self, object_type: OleObjectType) -> Vec<&OleObject> {
        self.objects.iter().filter(|o| o.object_type == object_type).collect()
    }

    pub fn embedded_count(&self) -> usize {
        self.objects.iter()
            .filter(|o| o.object_type == OleObjectType::Embed)
            .count()
    }

    pub fn linked_count(&self) -> usize {
        self.objects.iter()
            .filter(|o| o.object_type == OleObjectType::Link)
            .count()
    }

    pub fn broken_link_count(&self) -> usize {
        self.objects.iter()
            .filter(|o| o.object_state == OleObjectState::Broken)
            .count()
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn select_all(&mut self) {
        for object in &mut self.objects {
            object.is_locked = false;
        }
    }

    pub fn lock_all(&mut self) {
        for object in &mut self.objects {
            object.set_lock(true);
        }
    }

    pub fn hide_all(&mut self) {
        for object in &mut self.objects {
            object.set_visibility(false);
        }
    }

    pub fn show_all(&mut self) {
        for object in &mut self.objects {
            object.set_visibility(true);
        }
    }

    pub fn get_statistics(&self) -> OleStatistics {
        OleStatistics {
            total_objects: self.objects.len(),
            embedded: self.embedded_count(),
            linked: self.linked_count(),
            broken_links: self.broken_link_count(),
            hidden: self.objects.iter().filter(|o| !o.is_visible).count(),
            locked: self.objects.iter().filter(|o| o.is_locked).count(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OleStatistics {
    pub total_objects: usize,
    pub embedded: usize,
    pub linked: usize,
    pub broken_links: usize,
    pub hidden: usize,
    pub locked: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn test_ole_object_creation() {
        let object = OleObject::new()
            .with_prog_id("Excel.Application")
            .at_position(Point::new(100.0, 100.0, 0.0))
            .with_size(200.0, 150.0);

        assert_eq!(object.prog_id, "Excel.Application");
        assert!((object.position.x - 100.0).abs() < 1e-10);
        assert!((object.size.0 - 200.0).abs() < 1e-10);
    }

    #[test]
    fn test_ole_object_contains_point() {
        let object = OleObject::new()
            .at_position(Point::new(100.0, 100.0, 0.0))
            .with_size(100.0, 100.0);

        assert!(object.contains_point(&Point::new(100.0, 100.0, 0.0)));
        assert!(object.contains_point(&Point::new(50.0, 50.0, 0.0)));
        assert!(!object.contains_point(&Point::new(200.0, 200.0, 0.0)));
    }

    #[test]
    fn test_ole_object_border() {
        let mut object = OleObject::new();
        object.set_border(BorderStyle::Medium, 2);

        assert_eq!(object.border_style, BorderStyle::Medium);
        assert_eq!(object.border_width, 2);
    }

    #[test]
    fn test_ole_object_transparency() {
        let mut object = OleObject::new();
        object.set_transparency(50.0);
        assert!((object.transparency - 50.0).abs() < 1e-10);

        object.set_transparency(150.0);
        assert!((object.transparency - 100.0).abs() < 1e-10);
    }

    #[test]
    fn test_ole_manager_creation() {
        let manager = OleManager::new();
        assert_eq!(manager.object_count(), 0);
    }

    #[test]
    fn test_ole_manager_embed_file() {
        let mut manager = OleManager::new();
        let result = manager.embed_file("test.xlsx", Point::new(0.0, 0.0, 0.0));

        assert!(result.is_ok());
        assert_eq!(manager.object_count(), 1);
    }

    #[test]
    fn test_ole_manager_link_file() {
        let mut manager = OleManager::new();
        let result = manager.link_file("test.xlsx", Point::new(0.0, 0.0, 0.0));

        assert!(result.is_ok());
        let object = result.unwrap();
        assert_eq!(object.object_type, OleObjectType::Link);
    }

    #[test]
    fn test_ole_manager_remove() {
        let mut manager = OleManager::new();
        manager.create_object("Excel.Application", Point::origin(), (100.0, 100.0));
        let id = manager.objects[0].id.clone();

        assert!(manager.remove(&id));
        assert_eq!(manager.object_count(), 0);
    }

    #[test]
    fn test_ole_manager_find_by_position() {
        let mut manager = OleManager::new();
        manager.create_object("Excel.Application", Point::new(100.0, 100.0, 0.0), (100.0, 100.0));

        let found = manager.find_by_position(&Point::new(100.0, 100.0, 0.0));
        assert!(found.is_some());
    }

    #[test]
    fn test_ole_manager_update_all_links() {
        let mut manager = OleManager::new();
        manager.create_object("Excel.Application", Point::origin(), (100.0, 100.0));

        let updated = manager.update_all_links();
        assert_eq!(updated, 0);
    }

    #[test]
    fn test_ole_manager_break_all_links() {
        let mut manager = OleManager::new();
        let object = manager.create_object("Excel.Application", Point::origin(), (100.0, 100.0));
        object.object_type = OleObjectType::Link;
        object.link_path = Some(PathBuf::from("test.xlsx"));

        manager.break_all_links();

        for object in &manager.objects {
            assert_eq!(object.object_type, OleObjectType::Embed);
        }
    }

    #[test]
    fn test_ole_manager_object_counts() {
        let mut manager = OleManager::new();

        let obj1 = manager.create_object("Excel.Application", Point::origin(), (100.0, 100.0));
        obj1.object_type = OleObjectType::Embed;

        let obj2 = manager.create_object("Word.Application", Point::origin(), (100.0, 100.0));
        obj2.object_type = OleObjectType::Link;

        assert_eq!(manager.embedded_count(), 1);
        assert_eq!(manager.linked_count(), 1);
    }

    #[test]
    fn test_ole_manager_statistics() {
        let mut manager = OleManager::new();
        manager.create_object("Excel.Application", Point::origin(), (100.0, 100.0));

        let stats = manager.get_statistics();
        assert_eq!(stats.total_objects, 1);
        assert_eq!(stats.embedded, 1);
        assert_eq!(stats.linked, 0);
    }

    #[test]
    fn test_ole_object_type_names() {
        assert_eq!(OleObjectType::Embed.name(), "Embedded");
        assert_eq!(OleObjectType::Link.name(), "Linked");
        assert_eq!(OleObjectType::Ocx.name(), "OCX Control");
    }

    #[test]
    fn test_ole_object_state_names() {
        assert_eq!(OleObjectState::Closed.name(), "Closed");
        assert_eq!(OleObjectState::Open.name(), "Open");
        assert_eq!(OleObjectState::Linked.name(), "Linked");
    }

    #[test]
    fn test_border_style() {
        assert_eq!(BorderStyle::None.name(), "None");
        assert_eq!(BorderStyle::Thin.name(), "Thin");
        assert_eq!(BorderStyle::Medium.name(), "Medium");
        assert_eq!(BorderStyle::Thick.name(), "Thick");
    }

    #[test]
    fn test_ole_application() {
        let mut app = OleApplication::new().with_prog_id("Excel.Application");
        app.add_extension(".xlsx");
        app.add_extension(".xls");

        assert_eq!(app.extensions.len(), 2);
        assert!(app.extensions.contains(&".xlsx".to_string()));
    }

    #[test]
    fn test_ole_object_aspect_ratio() {
        let mut object = OleObject::new();
        object.set_aspect_ratio(2.0);
        assert!((object.aspect_ratio - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_ole_object_lock() {
        let mut object = OleObject::new();
        object.set_lock(true);
        assert!(object.is_locked);

        object.set_lock(false);
        assert!(!object.is_locked);
    }

    #[test]
    fn test_ole_object_visibility() {
        let mut object = OleObject::new();
        object.set_visibility(false);
        assert!(!object.is_visible);

        object.set_visibility(true);
        assert!(object.is_visible);
    }

    #[test]
    fn test_ole_object_printable() {
        let mut object = OleObject::new();
        object.set_printable(false);
        assert!(!object.is_printable);
    }
}
