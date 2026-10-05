use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::fmt;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SheetCategory {
    Architectural,
    Structural,
    Mechanical,
    Electrical,
    Plumbing,
    Civil,
    General,
    Custom(String),
}

impl Default for SheetCategory {
    fn default() -> Self {
        SheetCategory::General
    }
}

impl SheetCategory {
    pub fn name(&self) -> &str {
        match self {
            SheetCategory::Architectural => "Architectural",
            SheetCategory::Structural => "Structural",
            SheetCategory::Mechanical => "Mechanical",
            SheetCategory::Electrical => "Electrical",
            SheetCategory::Plumbing => "Plumbing",
            SheetCategory::Civil => "Civil",
            SheetCategory::General => "General",
            SheetCategory::Custom(name) => name.as_str(),
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            SheetCategory::Architectural => "🏛️",
            SheetCategory::Structural => "🏗️",
            SheetCategory::Mechanical => "⚙️",
            SheetCategory::Electrical => "⚡",
            SheetCategory::Plumbing => "🚿",
            SheetCategory::Civil => "🛣️",
            SheetCategory::General => "📄",
            SheetCategory::Custom(_) => "📁",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SheetStatus {
    NotStarted,
    InProgress,
    ForReview,
    IssuedForConstruction,
    Revised,
    Archived,
    Void,
}

impl Default for SheetStatus {
    fn default() -> Self {
        SheetStatus::NotStarted
    }
}

impl SheetStatus {
    pub fn name(&self) -> &str {
        match self {
            SheetStatus::NotStarted => "Not Started",
            SheetStatus::InProgress => "In Progress",
            SheetStatus::ForReview => "For Review",
            SheetStatus::IssuedForConstruction => "Issued for Construction",
            SheetStatus::Revised => "Revised",
            SheetStatus::Archived => "Archived",
            SheetStatus::Void => "Void",
        }
    }

    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            SheetStatus::NotStarted => (128, 128, 128),
            SheetStatus::InProgress => (255, 255, 0),
            SheetStatus::ForReview => (255, 165, 0),
            SheetStatus::IssuedForConstruction => (0, 128, 0),
            SheetStatus::Revised => (0, 0, 255),
            SheetStatus::Archived => (128, 128, 128),
            SheetStatus::Void => (255, 0, 0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetProperties {
    pub title: String,
    pub number: String,
    pub revision: String,
    pub category: SheetCategory,
    pub status: SheetStatus,
    pub description: String,
    pub designer: String,
    pub checker: String,
    pub approver: String,
    pub creation_date: SystemTime,
    pub modification_date: SystemTime,
    pub keywords: Vec<String>,
    pub client: String,
    pub project_name: String,
    pub project_number: String,
    pub drawing_file: PathBuf,
    pub layout_name: String,
    pub scale: f64,
    pub page_setup: String,
    print_settings: String,
    publish_settings: String,
}

impl Default for SheetProperties {
    fn default() -> Self {
        Self::new()
    }
}

impl SheetProperties {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            number: String::new(),
            revision: "A".to_string(),
            category: SheetCategory::General,
            status: SheetStatus::NotStarted,
            description: String::new(),
            designer: String::new(),
            checker: String::new(),
            approver: String::new(),
            creation_date: SystemTime::UNIX_EPOCH,
            modification_date: SystemTime::UNIX_EPOCH,
            keywords: Vec::new(),
            client: String::new(),
            project_name: String::new(),
            project_number: String::new(),
            drawing_file: PathBuf::new(),
            layout_name: String::new(),
            scale: 1.0,
            page_setup: String::new(),
            print_settings: String::new(),
            publish_settings: String::new(),
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn with_number(mut self, number: &str) -> Self {
        self.number = number.to_string();
        self
    }

    pub fn set_revision(&mut self, revision: &str) {
        self.revision = revision.to_string();
    }

    pub fn increment_revision(&mut self) {
        if self.revision.len() == 1 && self.revision.as_bytes()[0] >= b'A' && self.revision.as_bytes()[0] <= b'Z' {
            let next = (self.revision.as_bytes()[0] + 1) as char;
            self.revision = next.to_string();
        } else if self.revision.starts_with('R') {
            let num: i32 = self.revision[1..].parse().unwrap_or(0);
            self.revision = format!("R{}", num + 1);
        } else {
            self.revision.push('1');
        }
    }

    pub fn set_status(&mut self, status: SheetStatus) {
        self.status = status;
    }

    pub fn add_keyword(&mut self, keyword: &str) {
        if !self.keywords.contains(&keyword.to_string()) {
            self.keywords.push(keyword.to_string());
        }
    }

    pub fn set_designer(&mut self, designer: &str) {
        self.designer = designer.to_string();
    }

    pub fn set_checker(&mut self, checker: &str) {
        self.checker = checker.to_string();
    }

    pub fn set_approver(&mut self, approver: &str) {
        self.approver = approver.to_string();
    }

    pub fn full_name(&self) -> String {
        format!("{} - {}", self.number, self.title)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sheet {
    pub id: String,
    pub properties: SheetProperties,
    pub sheet_subset: Option<String>,
    pub layout_references: Vec<LayoutReference>,
    pub xref_references: Vec<XrefSheetReference>,
    pub block_references: Vec<BlockSheetReference>,
    pub custom_data: HashMap<String, String>,
    pub is_selected: bool,
    pub is_locked: bool,
    pub thumbnail: Option<Vec<u8>>,
    pub annotations: Vec<SheetAnnotation>,
}

impl Default for Sheet {
    fn default() -> Self {
        Self::new()
    }
}

impl Sheet {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            properties: SheetProperties::new(),
            sheet_subset: None,
            layout_references: Vec::new(),
            xref_references: Vec::new(),
            block_references: Vec::new(),
            custom_data: HashMap::new(),
            is_selected: false,
            is_locked: false,
            thumbnail: None,
            annotations: Vec::new(),
        }
    }

    pub fn with_properties(mut self, properties: SheetProperties) -> Self {
        self.properties = properties;
        self
    }

    pub fn add_layout_reference(&mut self, reference: LayoutReference) {
        self.layout_references.push(reference);
    }

    pub fn add_xref_reference(&mut self, reference: XrefSheetReference) {
        self.xref_references.push(reference);
    }

    pub fn add_block_reference(&mut self, reference: BlockSheetReference) {
        self.block_references.push(reference);
    }

    pub fn set_custom_data(&mut self, key: &str, value: &str) {
        self.custom_data.insert(key.to_string(), value.to_string());
    }

    pub fn get_custom_data(&self, key: &str) -> Option<&str> {
        self.custom_data.get(key).map(|s| s.as_str())
    }

    pub fn select(&mut self) {
        self.is_selected = true;
    }

    pub fn deselect(&mut self) {
        self.is_selected = false;
    }

    pub fn lock(&mut self) {
        self.is_locked = true;
    }

    pub fn unlock(&mut self) {
        self.is_locked = false;
    }

    pub fn add_annotation(&mut self, annotation: SheetAnnotation) {
        self.annotations.push(annotation);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutReference {
    pub drawing_file: PathBuf,
    pub layout_name: String,
    pub is_required: bool,
}

impl Default for LayoutReference {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutReference {
    pub fn new() -> Self {
        Self {
            drawing_file: PathBuf::new(),
            layout_name: String::new(),
            is_required: true,
        }
    }

    pub fn with_layout(mut self, drawing: &str, layout: &str) -> Self {
        self.drawing_file = PathBuf::from(drawing);
        self.layout_name = layout.to_string();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XrefSheetReference {
    pub xref_name: String,
    pub drawing_file: PathBuf,
    pub attachment_point: AttachmentPoint,
    pub is_required: bool,
}

impl Default for XrefSheetReference {
    fn default() -> Self {
        Self::new()
    }
}

impl XrefSheetReference {
    pub fn new() -> Self {
        Self {
            xref_name: String::new(),
            drawing_file: PathBuf::new(),
            attachment_point: AttachmentPoint::WorldCoordinateSystem,
            is_required: true,
        }
    }

    pub fn with_xref(mut self, name: &str, drawing: &str) -> Self {
        self.xref_name = name.to_string();
        self.drawing_file = PathBuf::from(drawing);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockSheetReference {
    pub block_name: String,
    pub is_required: bool,
    pub quantity: u32,
}

impl Default for BlockSheetReference {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockSheetReference {
    pub fn new() -> Self {
        Self {
            block_name: String::new(),
            is_required: true,
            quantity: 1,
        }
    }

    pub fn with_block(mut self, name: &str) -> Self {
        self.block_name = name.to_string();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetAnnotation {
    pub id: String,
    pub annotation_type: AnnotationType,
    pub content: String,
    pub position: (f64, f64),
    pub date: SystemTime,
    pub author: String,
}

impl Default for SheetAnnotation {
    fn default() -> Self {
        Self::new()
    }
}

impl SheetAnnotation {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            annotation_type: AnnotationType::Note,
            content: String::new(),
            position: (0.0, 0.0),
            date: SystemTime::UNIX_EPOCH,
            author: String::new(),
        }
    }

    pub fn with_content(mut self, content: &str) -> Self {
        self.content = content.to_string();
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnnotationType {
    Note,
    Issue,
    Revision,
    Approval,
}

impl Default for AnnotationType {
    fn default() -> Self {
        AnnotationType::Note
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttachmentPoint {
    WorldCoordinateSystem,
    CurrentUcs,
}

impl Default for AttachmentPoint {
    fn default() -> Self {
        AttachmentPoint::WorldCoordinateSystem
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetSubset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: SheetCategory,
    pub parent_id: Option<String>,
    pub sheet_ids: Vec<String>,
    pub subset_ids: Vec<String>,
    pub properties: HashMap<String, String>,
    pub display_order: i32,
}

impl Default for SheetSubset {
    fn default() -> Self {
        Self::new()
    }
}

impl SheetSubset {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            description: String::new(),
            category: SheetCategory::General,
            parent_id: None,
            sheet_ids: Vec::new(),
            subset_ids: Vec::new(),
            properties: HashMap::new(),
            display_order: 0,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn with_parent(mut self, parent_id: &str) -> Self {
        self.parent_id = Some(parent_id.to_string());
        self
    }

    pub fn add_sheet(&mut self, sheet_id: &str) {
        if !self.sheet_ids.contains(&sheet_id.to_string()) {
            self.sheet_ids.push(sheet_id.to_string());
        }
    }

    pub fn remove_sheet(&mut self, sheet_id: &str) -> bool {
        let len = self.sheet_ids.len();
        self.sheet_ids.retain(|id| id != sheet_id);
        self.sheet_ids.len() != len
    }

    pub fn add_subset(&mut self, subset_id: &str) {
        if !self.subset_ids.contains(&subset_id.to_string()) {
            self.subset_ids.push(subset_id.to_string());
        }
    }

    pub fn set_property(&mut self, key: &str, value: &str) {
        self.properties.insert(key.to_string(), value.to_string());
    }

    pub fn sheet_count(&self) -> usize {
        self.sheet_ids.len()
    }

    pub fn subset_count(&self) -> usize {
        self.subset_ids.len()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetSetProperties {
    pub name: String,
    pub description: String,
    pub project_number: String,
    pub client_name: String,
    pub designer: String,
    pub creator: String,
    pub creation_date: SystemTime,
    pub modification_date: SystemTime,
    pub default_category: SheetCategory,
    pub default_units: String,
    pub storage_location: PathBuf,
    pub publish_destination: PathBuf,
    pub is_read_only: bool,
    pub is_archived: bool,
    pub archive_date: Option<SystemTime>,
    pub custom_data: HashMap<String, String>,
}

impl Default for SheetSetProperties {
    fn default() -> Self {
        Self::new()
    }
}

impl SheetSetProperties {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            project_number: String::new(),
            client_name: String::new(),
            designer: String::new(),
            creator: String::new(),
            creation_date: SystemTime::UNIX_EPOCH,
            modification_date: SystemTime::UNIX_EPOCH,
            default_category: SheetCategory::General,
            default_units: "mm".to_string(),
            storage_location: PathBuf::new(),
            publish_destination: PathBuf::new(),
            is_read_only: false,
            is_archived: false,
            archive_date: None,
            custom_data: HashMap::new(),
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn set_project_number(&mut self, number: &str) {
        self.project_number = number.to_string();
    }

    pub fn set_client_name(&mut self, client: &str) {
        self.client_name = client.to_string();
    }

    pub fn set_storage_location(&mut self, path: &str) {
        self.storage_location = PathBuf::from(path);
    }

    pub fn set_publish_destination(&mut self, path: &str) {
        self.publish_destination = PathBuf::from(path);
    }

    pub fn set_read_only(&mut self, read_only: bool) {
        self.is_read_only = read_only;
    }

    pub fn archive(&mut self) {
        self.is_archived = true;
        self.archive_date = Some(SystemTime::now());
    }

    pub fn unarchive(&mut self) {
        self.is_archived = false;
        self.archive_date = None;
    }

    pub fn set_custom_data(&mut self, key: &str, value: &str) {
        self.custom_data.insert(key.to_string(), value.to_string());
    }
}

#[derive(Debug, Clone)]
pub struct SheetSet {
    pub properties: SheetSetProperties,
    pub sheets: HashMap<String, Sheet>,
    pub subsets: HashMap<String, SheetSubset>,
    pub sheet_order: Vec<String>,
    pub subset_order: Vec<String>,
    pub resource_files: Vec<ResourceFile>,
    pub templates: Vec<SheetTemplate>,
    pub callout_blocks: Vec<CalloutBlock>,
    pub revision_tables: Vec<RevisionTable>,
    pub sheet_set_view_id: String,
    pub is_modified: bool,
}

impl Default for SheetSet {
    fn default() -> Self {
        Self::new()
    }
}

impl SheetSet {
    pub fn new() -> Self {
        Self {
            properties: SheetSetProperties::new(),
            sheets: HashMap::new(),
            subsets: HashMap::new(),
            sheet_order: Vec::new(),
            subset_order: Vec::new(),
            resource_files: Vec::new(),
            templates: Vec::new(),
            callout_blocks: Vec::new(),
            revision_tables: Vec::new(),
            sheet_set_view_id: String::new(),
            is_modified: false,
        }
    }

    pub fn with_properties(mut self, properties: SheetSetProperties) -> Self {
        self.properties = properties;
        self
    }

    pub fn create_sheet(&mut self, number: &str, title: &str) -> &mut Sheet {
        let mut properties = SheetProperties::new()
            .with_number(number)
            .with_title(title);

        properties.creation_date = SystemTime::now();
        properties.modification_date = SystemTime::now();

        let sheet = Sheet::new().with_properties(properties);
        let sheet_id = sheet.id.clone();

        self.sheets.insert(sheet_id.clone(), sheet);
        self.sheet_order.push(sheet_id.clone());

        self.sheets.get_mut(&sheet_id).unwrap()
    }

    pub fn add_sheet(&mut self, sheet: Sheet) {
        let id = sheet.id.clone();
        self.sheets.insert(id.clone(), sheet);
        self.sheet_order.push(id);
        self.is_modified = true;
    }

    pub fn get_sheet(&self, id: &str) -> Option<&Sheet> {
        self.sheets.get(id)
    }

    pub fn get_sheet_mut(&mut self, id: &str) -> Option<&mut Sheet> {
        self.sheets.get_mut(id)
    }

    pub fn find_sheet_by_number(&self, number: &str) -> Option<&Sheet> {
        self.sheets.values().find(|s| s.properties.number == number)
    }

    pub fn find_sheet_by_title(&self, title: &str) -> Option<&Sheet> {
        self.sheets.values().find(|s| s.properties.title == title)
    }

    pub fn remove_sheet(&mut self, id: &str) -> bool {
        if let Some(_sheet) = self.sheets.remove(id) {
            self.sheet_order.retain(|i| i != id);

            for subset in self.subsets.values_mut() {
                subset.remove_sheet(id);
            }

            self.is_modified = true;
            true
        } else {
            false
        }
    }

    pub fn create_subset(&mut self, name: &str) -> &mut SheetSubset {
        let subset = SheetSubset::new().with_name(name);
        let subset_id = subset.id.clone();

        self.subsets.insert(subset_id.clone(), subset);
        self.subset_order.push(subset_id.clone());

        self.subsets.get_mut(&subset_id).unwrap()
    }

    pub fn add_subset(&mut self, subset: SheetSubset) {
        let id = subset.id.clone();
        self.subsets.insert(id.clone(), subset);
        self.subset_order.push(id);
        self.is_modified = true;
    }

    pub fn get_subset(&self, id: &str) -> Option<&SheetSubset> {
        self.subsets.get(id)
    }

    pub fn get_subset_mut(&mut self, id: &str) -> Option<&mut SheetSubset> {
        self.subsets.get_mut(id)
    }

    pub fn add_sheet_to_subset(&mut self, sheet_id: &str, subset_id: &str) -> bool {
        if let Some(sheet) = self.sheets.get_mut(sheet_id) {
            if let Some(subset) = self.subsets.get_mut(subset_id) {
                subset.add_sheet(sheet_id);
                sheet.sheet_subset = Some(subset_id.to_string());
                self.is_modified = true;
                return true;
            }
        }
        false
    }

    pub fn remove_sheet_from_subset(&mut self, sheet_id: &str, subset_id: &str) -> bool {
        if let Some(subset) = self.subsets.get_mut(subset_id) {
            let removed = subset.remove_sheet(sheet_id);
            if removed {
                if let Some(sheet) = self.sheets.get_mut(sheet_id) {
                    sheet.sheet_subset = None;
                }
                self.is_modified = true;
            }
            removed
        } else {
            false
        }
    }

    /// 把标识解析为图纸 id：既接受图纸 id，也接受图纸编号（如 `A-101`）。
    fn resolve_sheet_id(&self, id_or_number: &str) -> Option<String> {
        if self.sheets.contains_key(id_or_number) {
            return Some(id_or_number.to_string());
        }
        self.find_sheet_by_number(id_or_number)
            .map(|s| s.id.clone())
    }

    /// 调整图纸在图纸列表中的次序。
    ///
    /// - `sheet_id`：图纸 id 或图纸编号（如 `A-101`）。
    /// - `new_index`：目标下标（从 0 开始）；越界返回 `false` 且不修改任何状态。
    ///
    /// 返回是否移动成功。移动后 `sheet_order` 中保存的仍是图纸 id。
    pub fn move_sheet(&mut self, sheet_id: &str, new_index: usize) -> bool {
        let Some(sheet_id) = self.resolve_sheet_id(sheet_id) else {
            return false;
        };
        if let Some(current_index) = self.sheet_order.iter().position(|id| *id == sheet_id) {
            if new_index >= self.sheet_order.len() {
                return false;
            }

            let sheet_id = self.sheet_order.remove(current_index);
            self.sheet_order.insert(new_index, sheet_id);
            self.is_modified = true;
            true
        } else {
            false
        }
    }

    /// 按给定次序重排全部图纸。
    ///
    /// - `order`：图纸 id 或图纸编号的完整序列；长度必须与当前图纸数一致，
    ///   且每个元素都能解析到已存在的图纸，否则返回 `false` 且不修改任何状态。
    ///
    /// 返回是否重排成功；成功后 `sheet_order` 中保存的是图纸 id。
    pub fn reorder_sheets(&mut self, order: &[&str]) -> bool {
        if order.len() != self.sheet_order.len() {
            return false;
        }

        let mut new_order: Vec<String> = Vec::with_capacity(order.len());
        for id_or_number in order {
            let Some(id) = self.resolve_sheet_id(id_or_number) else {
                return false;
            };
            if new_order.contains(&id) {
                return false;
            }
            new_order.push(id);
        }

        self.sheet_order = new_order;
        self.is_modified = true;
        true
    }

    /// 复制一张图纸。
    ///
    /// - `source_id`：源图纸的 id 或图纸编号（如 `A-101`）。
    /// - `new_number`：新图纸编号；为空或与原编号重复时返回 `None`。
    ///
    /// 新图纸沿用源的标题与自定义属性，版本号重置为 `A`，创建 / 修改时间取当前时间，
    /// 并追加到图纸列表末尾。返回新图纸的 id；源图纸不存在时返回 `None`。
    pub fn duplicate_sheet(&mut self, source_id: &str, new_number: &str) -> Option<String> {
        let source_id = self.resolve_sheet_id(source_id)?;

        let trimmed_number = new_number.trim();
        if trimmed_number.is_empty() || self.find_sheet_by_number(trimmed_number).is_some() {
            return None;
        }

        let mut new_sheet = self.sheets.get(&source_id)?.clone();
        new_sheet.properties.number = trimmed_number.to_string();
        new_sheet.properties.revision = "A".to_string();
        new_sheet.properties.creation_date = SystemTime::now();
        new_sheet.properties.modification_date = SystemTime::now();
        new_sheet.id = uuid::Uuid::new_v4().to_string();
        new_sheet.is_selected = false;

        self.sheets.insert(new_sheet.id.clone(), new_sheet.clone());
        self.sheet_order.push(new_sheet.id.clone());

        self.is_modified = true;
        Some(new_sheet.id)
    }

    pub fn add_resource_file(&mut self, resource_file: ResourceFile) {
        self.resource_files.push(resource_file);
        self.is_modified = true;
    }

    pub fn add_template(&mut self, template: SheetTemplate) {
        self.templates.push(template);
        self.is_modified = true;
    }

    pub fn add_callout_block(&mut self, callout: CalloutBlock) {
        self.callout_blocks.push(callout);
        self.is_modified = true;
    }

    pub fn add_revision_table(&mut self, table: RevisionTable) {
        self.revision_tables.push(table);
        self.is_modified = true;
    }

    pub fn sheet_count(&self) -> usize {
        self.sheets.len()
    }

    pub fn subset_count(&self) -> usize {
        self.subsets.len()
    }

    pub fn sheet_ids(&self) -> Vec<&str> {
        self.sheet_order.iter().map(|s| s.as_str()).collect()
    }

    pub fn subset_ids(&self) -> Vec<&str> {
        self.subset_order.iter().map(|s| s.as_str()).collect()
    }

    pub fn sheets_in_category(&self, category: &SheetCategory) -> Vec<&Sheet> {
        self.sheets.values()
            .filter(|s| &s.properties.category == category)
            .collect()
    }

    pub fn sheets_by_status(&self, status: &SheetStatus) -> Vec<&Sheet> {
        self.sheets.values()
            .filter(|s| &s.properties.status == status)
            .collect()
    }

    pub fn search_sheets(&self, query: &str) -> Vec<&Sheet> {
        let query_lower = query.to_lowercase();
        self.sheets.values()
            .filter(|s| {
                s.properties.title.to_lowercase().contains(&query_lower)
                    || s.properties.number.to_lowercase().contains(&query_lower)
                    || s.properties.keywords.iter().any(|k| k.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    pub fn get_statistics(&self) -> SheetSetStatistics {
        let mut status_counts = HashMap::new();
        let mut category_counts = HashMap::new();

        for sheet in self.sheets.values() {
            *status_counts.entry(sheet.properties.status.clone()).or_insert(0) += 1;
            *category_counts.entry(sheet.properties.category.clone()).or_insert(0) += 1;
        }

        SheetSetStatistics {
            total_sheets: self.sheets.len(),
            total_subsets: self.subsets.len(),
            status_counts,
            category_counts,
            resource_files: self.resource_files.len(),
            templates: self.templates.len(),
            callout_blocks: self.callout_blocks.len(),
            revision_tables: self.revision_tables.len(),
            is_modified: self.is_modified,
        }
    }

    pub fn mark_as_modified(&mut self) {
        self.is_modified = true;
        self.properties.modification_date = SystemTime::now();
    }

    pub fn clear_modified(&mut self) {
        self.is_modified = false;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetSetStatistics {
    pub total_sheets: usize,
    pub total_subsets: usize,
    pub status_counts: HashMap<SheetStatus, usize>,
    pub category_counts: HashMap<SheetCategory, usize>,
    pub resource_files: usize,
    pub templates: usize,
    pub callout_blocks: usize,
    pub revision_tables: usize,
    pub is_modified: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceFile {
    pub name: String,
    pub file_path: PathBuf,
    pub file_type: String,
    pub description: String,
    pub usage: String,
}

impl Default for ResourceFile {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceFile {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            file_path: PathBuf::new(),
            file_type: String::new(),
            description: String::new(),
            usage: String::new(),
        }
    }

    pub fn with_path(mut self, path: &str) -> Self {
        self.file_path = PathBuf::from(path);
        self.name = self.file_path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());
        self.file_type = self.file_path.extension()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| String::new());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetTemplate {
    pub name: String,
    pub template_file: PathBuf,
    pub description: String,
    pub layout_name: String,
    pub is_default: bool,
}

impl Default for SheetTemplate {
    fn default() -> Self {
        Self::new()
    }
}

impl SheetTemplate {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            template_file: PathBuf::new(),
            description: String::new(),
            layout_name: String::new(),
            is_default: false,
        }
    }

    pub fn with_template(mut self, template: &str, layout: &str) -> Self {
        self.template_file = PathBuf::from(template);
        self.layout_name = layout.to_string();
        self.name = self.template_file.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| template.to_string());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalloutBlock {
    pub name: String,
    pub block_name: String,
    pub callout_type: CalloutType,
    pub description: String,
    pub is_system: bool,
}

impl Default for CalloutBlock {
    fn default() -> Self {
        Self::new()
    }
}

impl CalloutBlock {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            block_name: String::new(),
            callout_type: CalloutType::Detail,
            description: String::new(),
            is_system: false,
        }
    }

    pub fn with_block(mut self, name: &str, block: &str) -> Self {
        self.name = name.to_string();
        self.block_name = block.to_string();
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalloutType {
    Detail,
    Section,
    Elevation,
    PlanView,
    View,
}

impl Default for CalloutType {
    fn default() -> Self {
        CalloutType::Detail
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RevisionTable {
    pub name: String,
    pub description: String,
    pub sheet_id: String,
    pub revisions: Vec<RevisionEntry>,
    pub position: (f64, f64),
    pub width: f64,
    pub height: f64,
}

impl Default for RevisionTable {
    fn default() -> Self {
        Self::new()
    }
}

impl RevisionTable {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            sheet_id: String::new(),
            revisions: Vec::new(),
            position: (0.0, 0.0),
            width: 100.0,
            height: 50.0,
        }
    }

    pub fn add_revision(&mut self, revision: RevisionEntry) {
        self.revisions.push(revision);
    }

    pub fn latest_revision(&self) -> Option<&RevisionEntry> {
        self.revisions.last()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RevisionEntry {
    pub number: String,
    pub description: String,
    pub date: SystemTime,
    pub author: String,
    pub checked_by: String,
}

impl Default for RevisionEntry {
    fn default() -> Self {
        Self::new()
    }
}

impl RevisionEntry {
    pub fn new() -> Self {
        Self {
            number: String::new(),
            description: String::new(),
            date: SystemTime::UNIX_EPOCH,
            author: String::new(),
            checked_by: String::new(),
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }
}

#[derive(Debug, Clone)]
pub struct SheetSetManager {
    sheet_sets: HashMap<String, SheetSet>,
    active_sheet_set: Option<String>,
    backup_enabled: bool,
    backup_interval: u32,
    auto_backup_count: u32,
}

impl Default for SheetSetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SheetSetManager {
    pub fn new() -> Self {
        Self {
            sheet_sets: HashMap::new(),
            active_sheet_set: None,
            backup_enabled: true,
            backup_interval: 300,
            auto_backup_count: 3,
        }
    }

    pub fn create_sheet_set(&mut self, name: &str) -> &mut SheetSet {
        let properties = SheetSetProperties::new().with_name(name);
        let sheet_set = SheetSet::new().with_properties(properties);

        self.sheet_sets.insert(name.to_string(), sheet_set);
        self.active_sheet_set = Some(name.to_string());

        self.sheet_sets.get_mut(name).unwrap()
    }

    pub fn add_sheet_set(&mut self, sheet_set: SheetSet) {
        let name = sheet_set.properties.name.clone();
        self.sheet_sets.insert(name.clone(), sheet_set);
        self.active_sheet_set = Some(name);
    }

    pub fn get_sheet_set(&self, name: &str) -> Option<&SheetSet> {
        self.sheet_sets.get(name)
    }

    pub fn get_sheet_set_mut(&mut self, name: &str) -> Option<&mut SheetSet> {
        self.sheet_sets.get_mut(name)
    }

    pub fn remove_sheet_set(&mut self, name: &str) -> bool {
        self.sheet_sets.remove(name).is_some()
    }

    pub fn rename_sheet_set(&mut self, old_name: &str, new_name: &str) -> bool {
        if let Some(mut sheet_set) = self.sheet_sets.remove(old_name) {
            sheet_set.properties.name = new_name.to_string();
            self.sheet_sets.insert(new_name.to_string(), sheet_set);
            if self.active_sheet_set.as_ref() == Some(&old_name.to_string()) {
                self.active_sheet_set = Some(new_name.to_string());
            }
            true
        } else {
            false
        }
    }

    pub fn set_active_sheet_set(&mut self, name: Option<&str>) {
        self.active_sheet_set = name.map(|s| s.to_string());
    }

    pub fn active_sheet_set(&self) -> Option<&SheetSet> {
        self.active_sheet_set.as_ref().and_then(|name| self.sheet_sets.get(name))
    }

    pub fn active_sheet_set_mut(&mut self) -> Option<&mut SheetSet> {
        self.active_sheet_set.as_ref().and_then(|name| self.sheet_sets.get_mut(name))
    }

    pub fn sheet_set_count(&self) -> usize {
        self.sheet_sets.len()
    }

    pub fn sheet_set_names(&self) -> Vec<&str> {
        self.sheet_sets.keys().map(|s| s.as_str()).collect()
    }

    pub fn duplicate_sheet_set(&mut self, source_name: &str, new_name: &str) -> bool {
        if let Some(source_set) = self.sheet_sets.get(source_name) {
            let mut new_set = source_set.clone();
            new_set.properties.name = new_name.to_string();
            new_set.properties.creation_date = SystemTime::now();
            new_set.properties.modification_date = SystemTime::now();
            new_set.is_modified = true;

            self.sheet_sets.insert(new_name.to_string(), new_set);
            true
        } else {
            false
        }
    }

    pub fn export_sheet_list(&self, sheet_set_name: &str, format: &str) -> Option<String> {
        if let Some(sheet_set) = self.sheet_sets.get(sheet_set_name) {
            match format {
                "csv" | "txt" => {
                    let mut output = String::new();
                    output.push_str("Number,Title,Revision,Category,Status,Designer\n");
                    for sheet in sheet_set.sheets.values() {
                        output.push_str(&format!(
                            "{},{},{},{},{},{}\n",
                            sheet.properties.number,
                            sheet.properties.title,
                            sheet.properties.revision,
                            sheet.properties.category.name(),
                            sheet.properties.status.name(),
                            sheet.properties.designer
                        ));
                    }
                    Some(output)
                }
                "json" => {
                    let json = serde_json::to_string(&sheet_set.sheets).ok()?;
                    Some(json)
                }
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn import_sheets(&mut self, sheet_set_name: &str, sheets: Vec<Sheet>) -> usize {
        if let Some(sheet_set) = self.sheet_sets.get_mut(sheet_set_name) {
            let mut imported = 0;
            for sheet in sheets {
                if sheet_set.sheets.get(&sheet.id).is_none() {
                    let id = sheet.id.clone();
                    sheet_set.sheets.insert(id.clone(), sheet);
                    sheet_set.sheet_order.push(id);
                    imported += 1;
                }
            }
            sheet_set.is_modified = true;
            imported
        } else {
            0
        }
    }

    pub fn set_backup_enabled(&mut self, enabled: bool) {
        self.backup_enabled = enabled;
    }

    pub fn set_backup_interval(&mut self, interval: u32) {
        self.backup_interval = interval;
    }

    pub fn clear(&mut self) {
        self.sheet_sets.clear();
        self.active_sheet_set = None;
    }

    pub fn get_statistics(&self) -> SheetSetManagerStatistics {
        let total_sheets: usize = self.sheet_sets.values().map(|s| s.sheets.len()).sum();
        let total_subsets: usize = self.sheet_sets.values().map(|s| s.subsets.len()).sum();

        SheetSetManagerStatistics {
            total_sheet_sets: self.sheet_sets.len(),
            total_sheets,
            total_subsets,
            modified_sets: self.sheet_sets.values().filter(|s| s.is_modified).count(),
            backup_enabled: self.backup_enabled,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetSetManagerStatistics {
    pub total_sheet_sets: usize,
    pub total_sheets: usize,
    pub total_subsets: usize,
    pub modified_sets: usize,
    pub backup_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sheet_properties_creation() {
        let props = SheetProperties::new()
            .with_number("A-101")
            .with_title("Floor Plan - Level 1");

        assert_eq!(props.number, "A-101");
        assert_eq!(props.title, "Floor Plan - Level 1");
        assert_eq!(props.revision, "A");
    }

    #[test]
    fn test_sheet_properties_revision() {
        let mut props = SheetProperties::new();
        props.increment_revision();
        assert_eq!(props.revision, "B");

        props.revision = "R1".to_string();
        props.increment_revision();
        assert_eq!(props.revision, "R2");
    }

    #[test]
    fn test_sheet_creation() {
        let mut sheet = Sheet::new();
        sheet.properties = SheetProperties::new()
            .with_number("A-101")
            .with_title("Floor Plan");

        assert_eq!(sheet.properties.number, "A-101");
        assert!(!sheet.is_selected);
    }

    #[test]
    fn test_sheet_subset() {
        let mut subset = SheetSubset::new().with_name("Architectural");
        subset.add_sheet("sheet1");
        subset.add_sheet("sheet2");

        assert_eq!(subset.sheet_count(), 2);
        subset.remove_sheet("sheet1");
        assert_eq!(subset.sheet_count(), 1);
    }

    #[test]
    fn test_sheet_set_creation() {
        let mut manager = SheetSetManager::new();
        let sheet_set = manager.create_sheet_set("Project A");

        sheet_set.properties.set_client_name("ABC Corporation");

        assert_eq!(manager.sheet_set_count(), 1);
        assert!(manager.get_sheet_set("Project A").is_some());
    }

    #[test]
    fn test_sheet_set_add_sheet() {
        let mut sheet_set = SheetSet::new();
        let sheet = sheet_set.create_sheet("A-101", "Floor Plan");

        assert_eq!(sheet_set.sheet_count(), 1);
        assert!(sheet_set.find_sheet_by_number("A-101").is_some());
    }

    #[test]
    fn test_sheet_set_add_to_subset() {
        let mut sheet_set = SheetSet::new();
        let sheet_id = sheet_set.create_sheet("A-101", "Floor Plan").id.clone();
        let subset_id = sheet_set.create_subset("Drawings").id.clone();

        assert!(sheet_set.add_sheet_to_subset(&sheet_id, &subset_id));

        let subset = sheet_set.get_subset(&subset_id).unwrap();
        assert_eq!(subset.sheet_count(), 1);
    }

    #[test]
    fn test_sheet_set_move_sheet() {
        let mut sheet_set = SheetSet::new();
        sheet_set.create_sheet("A-101", "Sheet 1");
        sheet_set.create_sheet("A-102", "Sheet 2");
        sheet_set.create_sheet("A-103", "Sheet 3");

        assert_eq!(sheet_set.sheet_order[0], "A-101");
        sheet_set.move_sheet("A-101", 2);
        assert_eq!(sheet_set.sheet_order[2], "A-101");
    }

    #[test]
    fn test_sheet_set_duplicate_sheet() {
        let mut sheet_set = SheetSet::new();
        sheet_set.create_sheet("A-101", "Floor Plan");

        let new_id = sheet_set.duplicate_sheet("A-101", "A-101-COPY").unwrap();

        assert_eq!(sheet_set.sheet_count(), 2);
        assert!(sheet_set.find_sheet_by_number("A-101-COPY").is_some());
    }

    #[test]
    #[test]
    fn test_sheet_search() {
        let mut sheet_set = SheetSet::new();
        sheet_set.create_sheet("A-101", "Floor Plan Level 1");
        sheet_set.create_sheet("A-102", "Electrical Plan");
        sheet_set.create_sheet("A-103", "HVAC Layout");

        let results = sheet_set.search_sheets("floor");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_sheet_set_statistics() {
        let mut sheet_set = SheetSet::new();
        sheet_set.create_sheet("A-101", "Sheet 1");
        sheet_set.create_sheet("A-102", "Sheet 2");
        sheet_set.create_subset("Category");

        let stats = sheet_set.get_statistics();
        assert_eq!(stats.total_sheets, 2);
        assert_eq!(stats.total_subsets, 1);
    }

    #[test]
    fn test_sheet_category() {
        assert_eq!(SheetCategory::Architectural.name(), "Architectural");
        assert_eq!(SheetCategory::Mechanical.icon(), "⚙️");
    }

    #[test]
    fn test_sheet_status() {
        assert_eq!(SheetStatus::InProgress.name(), "In Progress");
        let color = SheetStatus::InProgress.color();
        assert_eq!(color, (255, 255, 0));
    }

    #[test]
    fn test_sheet_properties_full_name() {
        let props = SheetProperties::new()
            .with_number("A-101")
            .with_title("Floor Plan");
        assert_eq!(props.full_name(), "A-101 - Floor Plan");
    }

    #[test]
    fn test_resource_file() {
        let resource = ResourceFile::new().with_path("C:/Resources/Title.dwg");
        assert!(resource.name.contains("Title"));
        assert_eq!(resource.file_type, "dwg");
    }

    #[test]
    fn test_sheet_template() {
        let template = SheetTemplate::new()
            .with_template("templates/A3template.dwt", "A3 Layout");
        assert!(template.name.contains("A3template"));
        assert_eq!(template.layout_name, "A3 Layout");
    }

    #[test]
    fn test_revision_table() {
        let mut table = RevisionTable::new();
        table.add_revision(
            RevisionEntry::new()
                .with_description("Initial issue")
        );
        assert_eq!(table.revisions.len(), 1);
    }

    #[test]
    fn test_sheet_set_export() {
        let mut manager = SheetSetManager::new();
        manager.create_sheet_set("Project");
        let csv = manager.export_sheet_list("Project", "csv");
        assert!(csv.is_some());
        assert!(csv.unwrap().contains("Number,Title"));
    }

    #[test]
    fn test_sheet_set_import() {
        let mut manager = SheetSetManager::new();
        manager.create_sheet_set("Project");

        let sheets = vec![Sheet::new()];

        let imported = manager.import_sheets("Project", sheets);
        assert_eq!(imported, 1);
    }

    #[test]
    fn test_callout_block() {
        let callout = CalloutBlock::new()
            .with_block("Detail A", "DetailBlock");
        assert_eq!(callout.block_name, "DetailBlock");
    }

    #[test]
    fn test_sheet_annotation() {
        let annotation = SheetAnnotation::new()
            .with_content("Check this detail");
        assert_eq!(annotation.content, "Check this detail");
    }

    #[test]
    fn test_custom_data() {
        let mut sheet = Sheet::new();
        sheet.set_custom_data("ProjectPhase", "Design");
        assert_eq!(sheet.get_custom_data("ProjectPhase"), Some("Design"));
    }

    #[test]
    fn test_sheet_selection() {
        let mut sheet = Sheet::new();
        sheet.select();
        assert!(sheet.is_selected);

        sheet.deselect();
        assert!(!sheet.is_selected);
    }

    #[test]
    fn test_sheet_set_mark_modified() {
        let mut sheet_set = SheetSet::new();
        assert!(!sheet_set.is_modified);

        sheet_set.mark_as_modified();
        assert!(sheet_set.is_modified);
    }

    #[test]
    fn test_sheet_set_reorder() {
        let mut sheet_set = SheetSet::new();
        sheet_set.create_sheet("A-101", "Sheet 1");
        sheet_set.create_sheet("A-102", "Sheet 2");

        let order = ["A-102", "A-101"];
        assert!(sheet_set.reorder_sheets(&order));
        assert_eq!(sheet_set.sheet_order[0], "A-102");
    }
}
