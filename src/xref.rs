use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::fmt;
use std::time::{SystemTime, Duration};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum XrefType {
    Dwg,
    Dwf,
    Dxf,
    Pdf,
    Image,
    Dgn,
    Scene,
}

impl Default for XrefType {
    fn default() -> Self {
        XrefType::Dwg
    }
}

impl XrefType {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            ".dwg" => XrefType::Dwg,
            ".dwf" | ".dwfx" => XrefType::Dwf,
            ".dxf" => XrefType::Dxf,
            ".pdf" => XrefType::Pdf,
            ".jpg" | ".jpeg" | ".png" | ".bmp" | ".tif" | ".tiff" | ".gif" | ".webp" => XrefType::Image,
            ".dgn" => XrefType::Dgn,
            ".3ds" | ".obj" | ".fbx" => XrefType::Scene,
            _ => XrefType::Dwg,
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            XrefType::Dwg => ".dwg",
            XrefType::Dwf => ".dwf",
            XrefType::Dxf => ".dxf",
            XrefType::Pdf => ".pdf",
            XrefType::Image => ".jpg",
            XrefType::Dgn => ".dgn",
            XrefType::Scene => ".obj",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum XrefStatus {
    Loaded,
    Unloaded,
    Unresolved,
    Modified,
    Referenced,
    Embedded,
    Overlay,
}

impl Default for XrefStatus {
    fn default() -> Self {
        XrefStatus::Unloaded
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum XrefFormat {
    Full,
    Partial,
    SymbolTableOnly,
    NoMessages,
}

impl Default for XrefFormat {
    fn default() -> Self {
        XrefFormat::Full
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BindType {
    Bind,
    Insert,
    BindNested,
}

impl Default for BindType {
    fn default() -> Self {
        BindType::Insert
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrefDefinition {
    pub name: String,
    pub path: PathBuf,
    pub xref_type: XrefType,
    pub status: XrefStatus,
    pub format: XrefFormat,
    pub loaded_at: Option<SystemTime>,
    pub file_size: u64,
    pub last_modified: SystemTime,
    pub definition_created: SystemTime,
    pub blocks: Vec<String>,
    pub layers: Vec<String>,
    pub linetypes: Vec<String>,
    pub text_styles: Vec<String>,
    pub dimension_styles: Vec<String>,
    pub user_data: HashMap<String, String>,
    pub is_overlay: bool,
    pub is_nested: bool,
    pub attachment_point: AttachmentPoint,
    pub scale: (f64, f64, f64),
    pub rotation: f64,
}

impl Default for XrefDefinition {
    fn default() -> Self {
        Self::new()
    }
}

impl XrefDefinition {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            path: PathBuf::new(),
            xref_type: XrefType::Dwg,
            status: XrefStatus::Unloaded,
            format: XrefFormat::Full,
            loaded_at: None,
            file_size: 0,
            last_modified: SystemTime::UNIX_EPOCH,
            definition_created: SystemTime::UNIX_EPOCH,
            blocks: Vec::new(),
            layers: Vec::new(),
            linetypes: Vec::new(),
            text_styles: Vec::new(),
            dimension_styles: Vec::new(),
            user_data: HashMap::new(),
            is_overlay: false,
            is_nested: false,
            attachment_point: AttachmentPoint::WorldCoordinateSystem,
            scale: (1.0, 1.0, 1.0),
            rotation: 0.0,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn with_path(mut self, path: &str) -> Self {
        self.path = PathBuf::from(path);
        self.xref_type = XrefType::from_extension(path);
        self
    }

    pub fn with_scale(mut self, x: f64, y: f64, z: f64) -> Self {
        self.scale = (x, y, z);
        self
    }

    pub fn with_rotation(mut self, rotation: f64) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn set_overlay(&mut self, is_overlay: bool) {
        self.is_overlay = is_overlay;
    }

    pub fn set_format(&mut self, format: XrefFormat) {
        self.format = format;
    }
}

impl fmt::Display for XrefDefinition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "XREF: {} ({:?}) - {:?}",
            self.name, self.xref_type, self.status
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttachmentPoint {
    WorldCoordinateSystem,
    CurrentUcs,
    CurrentViewport,
}

impl Default for AttachmentPoint {
    fn default() -> Self {
        AttachmentPoint::WorldCoordinateSystem
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrefInstance {
    pub id: String,
    pub definition_name: String,
    pub position: crate::geometry::Point,
    pub rotation: f64,
    pub scale: (f64, f64, f64),
    pub layer: String,
    pub is_visible: bool,
    pub is_clipped: bool,
    pub clip_boundary: Option<Vec<crate::geometry::Point>>,
    pub transparency: f64,
    pub brightness: f64,
    pub contrast: f64,
    pub fade: f64,
    pub is_frozen: bool,
    pub is_locked: bool,
}

impl Default for XrefInstance {
    fn default() -> Self {
        Self::new()
    }
}

impl XrefInstance {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            definition_name: String::new(),
            position: crate::geometry::Point::origin(),
            rotation: 0.0,
            scale: (1.0, 1.0, 1.0),
            layer: String::new(),
            is_visible: true,
            is_clipped: false,
            clip_boundary: None,
            transparency: 0.0,
            brightness: 50.0,
            contrast: 50.0,
            fade: 0.0,
            is_frozen: false,
            is_locked: false,
        }
    }

    pub fn with_definition(mut self, name: &str) -> Self {
        self.definition_name = name.to_string();
        self
    }

    pub fn with_position(mut self, position: crate::geometry::Point) -> Self {
        self.position = position;
        self
    }

    pub fn with_rotation(mut self, rotation: f64) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_scale(mut self, x: f64, y: f64, z: f64) -> Self {
        self.scale = (x, y, z);
        self
    }

    pub fn set_clip_boundary(&mut self, boundary: &[crate::geometry::Point]) {
        self.clip_boundary = Some(boundary.to_vec());
        self.is_clipped = true;
    }

    pub fn clear_clip_boundary(&mut self) {
        self.clip_boundary = None;
        self.is_clipped = false;
    }

    pub fn set_visibility(&mut self, visible: bool) {
        self.is_visible = visible;
    }

    pub fn set_layer(&mut self, layer: &str) {
        self.layer = layer.to_string();
    }

    pub fn set_transparency(&mut self, transparency: f64) {
        self.transparency = transparency.clamp(0.0, 100.0);
    }

    pub fn set_freeze(&mut self, frozen: bool) {
        self.is_frozen = frozen;
    }

    pub fn set_lock(&mut self, locked: bool) {
        self.is_locked = locked;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrefPath {
    pub relative_path: PathBuf,
    pub absolute_path: PathBuf,
    pub found: bool,
    pub last_checked: SystemTime,
}

impl Default for XrefPath {
    fn default() -> Self {
        Self::new()
    }
}

impl XrefPath {
    pub fn new() -> Self {
        Self {
            relative_path: PathBuf::new(),
            absolute_path: PathBuf::new(),
            found: false,
            last_checked: SystemTime::UNIX_EPOCH,
        }
    }

    pub fn from_relative(relative: &str, base: &str) -> Self {
        let relative_path = PathBuf::from(relative);
        let absolute_path = PathBuf::from(base).parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join(&relative_path);

        Self {
            relative_path,
            absolute_path,
            found: false,
            last_checked: SystemTime::UNIX_EPOCH,
        }
    }

    pub fn from_absolute(path: &str) -> Self {
        let absolute_path = PathBuf::from(path);
        Self {
            relative_path: absolute_path.file_name()
                .map(|p| PathBuf::from(p.to_string_lossy().as_ref()))
                .unwrap_or_else(|| absolute_path.clone()),
            absolute_path,
            found: false,
            last_checked: SystemTime::UNIX_EPOCH,
        }
    }

    pub fn exists(&mut self) -> bool {
        self.found = self.absolute_path.exists();
        self.last_checked = SystemTime::now();
        self.found
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestedXref {
    pub path: PathBuf,
    pub definition_name: String,
    pub parent_xref: String,
    pub depth: u32,
    pub is_valid: bool,
}

impl Default for NestedXref {
    fn default() -> Self {
        Self::new()
    }
}

impl NestedXref {
    pub fn new() -> Self {
        Self {
            path: PathBuf::new(),
            definition_name: String::new(),
            parent_xref: String::new(),
            depth: 0,
            is_valid: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrefDependency {
    pub file_name: String,
    pub path: PathBuf,
    pub is_required: bool,
    pub is_missing: bool,
}

impl Default for XrefDependency {
    fn default() -> Self {
        Self::new()
    }
}

impl XrefDependency {
    pub fn new() -> Self {
        Self {
            file_name: String::new(),
            path: PathBuf::new(),
            is_required: true,
            is_missing: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct XrefManager {
    definitions: HashMap<String, XrefDefinition>,
    instances: HashMap<String, XrefInstance>,
    paths: HashMap<String, XrefPath>,
    nested_xrefs: Vec<NestedXref>,
    dependencies: Vec<XrefDependency>,
    search_paths: Vec<PathBuf>,
    is_attach_relative: bool,
    is_load_on_demand: bool,
    max_nested_depth: u32,
    current_nested_depth: u32,
}

impl Default for XrefManager {
    fn default() -> Self {
        Self::new()
    }
}

impl XrefManager {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            instances: HashMap::new(),
            paths: HashMap::new(),
            nested_xrefs: Vec::new(),
            dependencies: Vec::new(),
            search_paths: Vec::new(),
            is_attach_relative: true,
            is_load_on_demand: false,
            max_nested_depth: 10,
            current_nested_depth: 0,
        }
    }

    pub fn attach(
        &mut self,
        name: &str,
        path: &str,
        position: crate::geometry::Point,
        rotation: f64,
        scale: (f64, f64, f64),
    ) -> Result<String, String> {
        let definition = XrefDefinition::new()
            .with_name(name)
            .with_path(path)
            .with_scale(scale.0, scale.1, scale.2)
            .with_rotation(rotation);

        let instance_id = self.create_instance(&definition.name, position, rotation, scale)?;

        self.definitions.insert(name.to_string(), definition);

        Ok(instance_id)
    }

    pub fn attach_overlay(&mut self, name: &str, path: &str) -> Result<String, String> {
        let mut definition = XrefDefinition::new().with_name(name).with_path(path);
        definition.set_overlay(true);

        let instance_id = self.create_instance(name, crate::geometry::Point::origin(), 0.0, (1.0, 1.0, 1.0))?;

        self.definitions.insert(name.to_string(), definition);

        Ok(instance_id)
    }

    fn create_instance(
        &mut self,
        definition_name: &str,
        position: crate::geometry::Point,
        rotation: f64,
        scale: (f64, f64, f64),
    ) -> Result<String, String> {
        let instance = XrefInstance::new()
            .with_definition(definition_name)
            .with_position(position)
            .with_rotation(rotation)
            .with_scale(scale.0, scale.1, scale.2);

        let instance_id = instance.id.clone();
        self.instances.insert(instance_id.clone(), instance);

        Ok(instance_id)
    }

    pub fn detach(&mut self, name: &str) -> bool {
        let removed_def = self.definitions.remove(name);
        let removed_instances: Vec<String> = self.instances.iter()
            .filter(|(_, i)| i.definition_name == name)
            .map(|(id, _)| id.clone())
            .collect();

        for id in &removed_instances {
            self.instances.remove(id);
        }

        removed_def.is_some() || !removed_instances.is_empty()
    }

    pub fn reload(&mut self, name: &str) -> Result<(), String> {
        if let Some(definition) = self.definitions.get_mut(name) {
            definition.loaded_at = Some(SystemTime::now());
            definition.status = XrefStatus::Loaded;

            for instance in self.instances.values_mut() {
                if instance.definition_name == name {
                    instance.clear_clip_boundary();
                }
            }

            Ok(())
        } else {
            Err(format!("XREF '{}' not found", name))
        }
    }

    pub fn unload(&mut self, name: &str) -> Result<(), String> {
        if let Some(definition) = self.definitions.get_mut(name) {
            definition.status = XrefStatus::Unloaded;
            definition.loaded_at = None;

            Ok(())
        } else {
            Err(format!("XREF '{}' not found", name))
        }
    }

    pub fn bind(&mut self, name: &str, bind_type: BindType) -> Result<(), String> {
        if !self.definitions.contains_key(name) {
            return Err(format!("XREF '{}' not found", name));
        }

        self.detach(name);

        match bind_type {
            BindType::Insert => {
            }
            BindType::Bind => {
            }
            BindType::BindNested => {
            }
        }

        Ok(())
    }

    pub fn rename(&mut self, old_name: &str, new_name: &str) -> bool {
        if let Some(definition) = self.definitions.remove(old_name) {
            let mut new_definition = definition;
            new_definition.name = new_name.to_string();

            for instance in self.instances.values_mut() {
                if instance.definition_name == old_name {
                    instance.definition_name = new_name.to_string();
                }
            }

            self.definitions.insert(new_name.to_string(), new_definition);
            true
        } else {
            false
        }
    }

    pub fn get_definition(&self, name: &str) -> Option<&XrefDefinition> {
        self.definitions.get(name)
    }

    pub fn get_definition_mut(&mut self, name: &str) -> Option<&mut XrefDefinition> {
        self.definitions.get_mut(name)
    }

    pub fn get_instance(&self, id: &str) -> Option<&XrefInstance> {
        self.instances.get(id)
    }

    pub fn get_instance_mut(&mut self, id: &str) -> Option<&mut XrefInstance> {
        self.instances.get_mut(id)
    }

    pub fn get_instances_by_definition(&self, name: &str) -> Vec<&XrefInstance> {
        self.instances.values()
            .filter(|i| i.definition_name == name)
            .collect()
    }

    pub fn definition_count(&self) -> usize {
        self.definitions.len()
    }

    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    pub fn definition_names(&self) -> Vec<&str> {
        self.definitions.keys().map(|s| s.as_str()).collect()
    }

    pub fn instance_ids(&self) -> Vec<&str> {
        self.instances.keys().map(|s| s.as_str()).collect()
    }

    pub fn add_search_path(&mut self, path: &str) {
        self.search_paths.push(PathBuf::from(path));
    }

    pub fn clear_search_paths(&mut self) {
        self.search_paths.clear();
    }

    /// 解析外部参照文件名到实际路径。
    ///
    /// 解析顺序：`filename` 本身（绝对路径或包含目录的相对路径）→ 依次与
    /// [`Self::add_search_path`] 注册的搜索目录拼接。
    ///
    /// - 命中磁盘上真实存在的文件时返回该路径；
    /// - 都不存在时返回「首选候选路径」（`filename` 自身，或第一个搜索目录下的同名文件），
    ///   便于调用方提示用户缺失位置；文件名为空时返回 `None`。
    pub fn resolve_path(&mut self, filename: &str) -> Option<PathBuf> {
        if filename.trim().is_empty() {
            return None;
        }

        let direct = PathBuf::from(filename);
        if direct.exists() {
            return Some(direct);
        }

        // 未命中磁盘：优先把第一个搜索目录下的同名文件作为提示位置，
        // 没有配置搜索目录时回退为 `filename` 本身
        let mut first_candidate = PathBuf::from(filename);

        for (index, search_path) in self.search_paths.iter().enumerate() {
            let candidate = search_path.join(filename);
            if candidate.exists() {
                return Some(candidate);
            }
            if index == 0 {
                first_candidate = candidate;
            }
        }

        Some(first_candidate)
    }

    pub fn find_missing_xrefs(&self) -> Vec<&str> {
        self.definitions.values()
            .filter(|d| d.status == XrefStatus::Unresolved)
            .map(|d| d.name.as_str())
            .collect()
    }

    pub fn set_relative_path(&mut self, relative: bool) {
        self.is_attach_relative = relative;
    }

    pub fn set_load_on_demand(&mut self, load: bool) {
        self.is_load_on_demand = load;
    }

    pub fn set_clip_boundary(&mut self, instance_id: &str, boundary: &[crate::geometry::Point]) -> bool {
        if let Some(instance) = self.instances.get_mut(instance_id) {
            instance.set_clip_boundary(boundary);
            true
        } else {
            false
        }
    }

    pub fn remove_clip_boundary(&mut self, instance_id: &str) -> bool {
        if let Some(instance) = self.instances.get_mut(instance_id) {
            instance.clear_clip_boundary();
            true
        } else {
            false
        }
    }

    pub fn update_path(&mut self, name: &str, new_path: &str) -> bool {
        if let Some(definition) = self.definitions.get_mut(name) {
            definition.path = PathBuf::from(new_path);
            definition.xref_type = XrefType::from_extension(new_path);
            definition.status = XrefStatus::Modified;
            true
        } else {
            false
        }
    }

    pub fn verify_all_paths(&mut self) -> Vec<String> {
        let mut missing = Vec::new();

        let names: Vec<String> = self.definition_names().into_iter().map(|s| s.to_string()).collect();
        for name in names {
            if let Some(path) = self.paths.get_mut(&name) {
                if !path.exists() {
                    missing.push(name);
                }
            }
        }

        missing
    }

    pub fn set_visibility(&mut self, name: &str, visible: bool) -> usize {
        let mut count = 0;
        for instance in self.instances.values_mut() {
            if instance.definition_name == name {
                instance.set_visibility(visible);
                count += 1;
            }
        }
        count
    }

    pub fn freeze_all(&mut self) {
        for instance in self.instances.values_mut() {
            instance.set_freeze(true);
        }
    }

    pub fn unfreeze_all(&mut self) {
        for instance in self.instances.values_mut() {
            instance.set_freeze(false);
        }
    }

    pub fn lock_all(&mut self) {
        for instance in self.instances.values_mut() {
            instance.set_lock(true);
        }
    }

    pub fn unlock_all(&mut self) {
        for instance in self.instances.values_mut() {
            instance.set_lock(false);
        }
    }

    pub fn clear(&mut self) {
        self.definitions.clear();
        self.instances.clear();
        self.paths.clear();
        self.nested_xrefs.clear();
        self.dependencies.clear();
    }

    pub fn dump_xref_tree(&self, name: &str, depth: u32) -> Vec<(String, u32)> {
        let mut tree = Vec::new();
        tree.push((name.to_string(), depth));

        if depth < self.max_nested_depth {
            if let Some(definition) = self.definitions.get(name) {
                for nested in &definition.blocks {
                    tree.extend(self.dump_xref_tree(nested, depth + 1));
                }
            }
        }

        tree
    }

    pub fn get_statistics(&self) -> XrefStatistics {
        let mut loaded = 0;
        let mut unloaded = 0;
        let mut unresolved = 0;
        let mut modified = 0;

        for definition in self.definitions.values() {
            match definition.status {
                XrefStatus::Loaded => loaded += 1,
                XrefStatus::Unloaded => unloaded += 1,
                XrefStatus::Unresolved => unresolved += 1,
                XrefStatus::Modified => modified += 1,
                _ => {}
            }
        }

        XrefStatistics {
            total_definitions: self.definitions.len(),
            total_instances: self.instances.len(),
            loaded,
            unloaded,
            unresolved,
            modified,
            search_paths: self.search_paths.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrefStatistics {
    pub total_definitions: usize,
    pub total_instances: usize,
    pub loaded: usize,
    pub unloaded: usize,
    pub unresolved: usize,
    pub modified: usize,
    pub search_paths: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn test_xref_definition_creation() {
        let xref = XrefDefinition::new().with_name("MyXref")
            .with_path("reference.dwg")
            .with_scale(2.0, 2.0, 2.0)
            .with_rotation(45.0);

        assert_eq!(xref.name, "MyXref");
        assert_eq!(xref.xref_type, XrefType::Dwg);
        assert!((xref.scale.0 - 2.0).abs() < 1e-10);
        assert!((xref.rotation - 45.0).abs() < 1e-10);
    }

    #[test]
    fn test_xref_instance_creation() {
        let instance = XrefInstance::new()
            .with_definition("MyXref")
            .with_position(Point::new(100.0, 100.0, 0.0))
            .with_rotation(90.0)
            .with_scale(1.0, 2.0, 1.0);

        assert_eq!(instance.definition_name, "MyXref");
        assert!((instance.position.x - 100.0).abs() < 1e-10);
    }

    #[test]
    fn test_xref_manager_attach() {
        let mut manager = XrefManager::new();
        let result = manager.attach(
            "TestXref",
            "reference.dwg",
            Point::new(0.0, 0.0, 0.0),
            0.0,
            (1.0, 1.0, 1.0),
        );

        assert!(result.is_ok());
        let instance_id = result.unwrap();
        assert!(manager.definition_count() == 1);
        assert!(manager.instance_count() == 1);
    }

    #[test]
    fn test_xref_manager_detach() {
        let mut manager = XrefManager::new();
        manager.attach("TestXref", "reference.dwg", Point::origin(), 0.0, (1.0, 1.0, 1.0)).unwrap();

        assert!(manager.detach("TestXref"));
        assert!(manager.definition_count() == 0);
        assert!(manager.instance_count() == 0);
    }

    #[test]
    fn test_xref_manager_rename() {
        let mut manager = XrefManager::new();
        manager.attach("OldName", "reference.dwg", Point::origin(), 0.0, (1.0, 1.0, 1.0)).unwrap();

        assert!(manager.rename("OldName", "NewName"));
        assert!(manager.definition_names().contains(&"NewName"));
        assert!(!manager.definition_names().contains(&"OldName"));
    }

    #[test]
    fn test_xref_path_resolution() {
        let mut manager = XrefManager::new();
        manager.add_search_path("C:/CAD/Files");
        manager.add_search_path("D:/Projects");

        assert!(manager.resolve_path("existing.dwg").is_some());
    }

    #[test]
    fn test_xref_type_detection() {
        assert_eq!(XrefType::from_extension(".dwg"), XrefType::Dwg);
        assert_eq!(XrefType::from_extension(".PDF"), XrefType::Pdf);
        assert_eq!(XrefType::from_extension(".png"), XrefType::Image);
        assert_eq!(XrefType::from_extension(".dgn"), XrefType::Dgn);
    }

    #[test]
    fn test_xref_clip_boundary() {
        let mut instance = XrefInstance::new();
        let boundary = vec![
            Point::new(0.0, 0.0, 0.0),
            Point::new(100.0, 0.0, 0.0),
            Point::new(100.0, 100.0, 0.0),
            Point::new(0.0, 100.0, 0.0),
        ];

        instance.set_clip_boundary(&boundary);

        assert!(instance.is_clipped);
        assert!(instance.clip_boundary.is_some());
        assert_eq!(instance.clip_boundary.unwrap().len(), 4);
    }

    #[test]
    fn test_xref_visibility() {
        let mut instance = XrefInstance::new();
        instance.set_visibility(false);
        assert!(!instance.is_visible);

        instance.set_visibility(true);
        assert!(instance.is_visible);
    }

    #[test]
    fn test_xref_freeze() {
        let mut instance = XrefInstance::new();
        instance.set_freeze(true);
        assert!(instance.is_frozen);
    }

    #[test]
    fn test_xref_transparency() {
        let mut instance = XrefInstance::new();
        instance.set_transparency(50.0);
        assert!((instance.transparency - 50.0).abs() < 1e-10);

        instance.set_transparency(150.0);
        assert!((instance.transparency - 100.0).abs() < 1e-10);
    }

    #[test]
    fn test_xref_overlay() {
        let mut definition = XrefDefinition::new();
        definition.set_overlay(true);
        assert!(definition.is_overlay);
    }

    #[test]
    fn test_xref_format() {
        let mut definition = XrefDefinition::new();
        definition.set_format(XrefFormat::SymbolTableOnly);
        assert_eq!(definition.format, XrefFormat::SymbolTableOnly);
    }

    #[test]
    fn test_xref_path() {
        let path = XrefPath::from_relative("refs/xref.dwg", "C:/Projects/drawing.dwg");
        assert!(path.relative_path.to_string_lossy().contains("xref.dwg"));
    }

    #[test]
    fn test_xref_statistics() {
        let mut manager = XrefManager::new();
        manager.attach("Xref1", "ref1.dwg", Point::origin(), 0.0, (1.0, 1.0, 1.0)).unwrap();
        manager.attach("Xref2", "ref2.dwg", Point::origin(), 0.0, (1.0, 1.0, 1.0)).unwrap();

        let stats = manager.get_statistics();
        assert_eq!(stats.total_definitions, 2);
        assert_eq!(stats.total_instances, 2);
    }

    #[test]
    fn test_xref_reset_path() {
        let mut manager = XrefManager::new();
        manager.attach("Xref", "old.dwg", Point::origin(), 0.0, (1.0, 1.0, 1.0)).unwrap();

        assert!(manager.update_path("Xref", "new.dwg"));
        let definition = manager.get_definition("Xref").unwrap();
        assert!(definition.path.to_string_lossy().contains("new.dwg"));
    }
}
