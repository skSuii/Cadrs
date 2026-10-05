use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::any::{TypeId, Any};
use std::hash::{Hash, Hasher};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XData {
    pub app_name: String,
    pub data: Vec<XDataItem>,
}

impl Default for XData {
    fn default() -> Self {
        Self {
            app_name: String::new(),
            data: Vec::new(),
        }
    }
}

impl XData {
    #[inline]
    pub fn new(app_name: &str) -> Self {
        Self {
            app_name: app_name.to_string(),
            data: Vec::new(),
        }
    }

    #[inline]
    pub fn add_string(&mut self, value: &str) {
        self.data.push(XDataItem::String(value.to_string()));
    }

    #[inline]
    pub fn add_real(&mut self, value: f64) {
        self.data.push(XDataItem::Real(value));
    }

    #[inline]
    pub fn add_point(&mut self, point: super::geometry::Point) {
        self.data.push(XDataItem::Point(point));
    }

    #[inline]
    pub fn add_integer(&mut self, value: i32) {
        self.data.push(XDataItem::Integer(value));
    }

    #[inline]
    pub fn add_long(&mut self, value: i64) {
        self.data.push(XDataItem::Long(value));
    }

    #[inline]
    pub fn add_boolean(&mut self, value: bool) {
        self.data.push(XDataItem::Boolean(value));
    }

    #[inline]
    pub fn add_binary(&mut self, data: Vec<u8>) {
        self.data.push(XDataItem::Binary(data));
    }

    #[inline]
    pub fn add_handle(&mut self, handle: &str) {
        self.data.push(XDataItem::Handle(handle.to_string()));
    }

    #[inline]
    pub fn add_3d_point(&mut self, x: f64, y: f64, z: f64) {
        self.data.push(XDataItem::Point3D(x, y, z));
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.data.clear();
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &XDataItem> {
        self.data.iter()
    }

    #[inline]
    pub fn strings(&self) -> Vec<&str> {
        self.data.iter()
            .filter_map(|item| match item {
                XDataItem::String(s) => Some(s.as_str()),
                _ => None,
            })
            .collect()
    }

    #[inline]
    pub fn reals(&self) -> Vec<f64> {
        self.data.iter()
            .filter_map(|item| match item {
                XDataItem::Real(r) => Some(*r),
                _ => None,
            })
            .collect()
    }

    #[inline]
    pub fn integers(&self) -> Vec<i32> {
        self.data.iter()
            .filter_map(|item| match item {
                XDataItem::Integer(i) => Some(*i),
                _ => None,
            })
            .collect()
    }

    #[inline]
    pub fn points(&self) -> Vec<super::geometry::Point> {
        self.data.iter()
            .filter_map(|item| match item {
                XDataItem::Point(p) => Some(*p),
                _ => None,
            })
            .collect()
    }
}

impl fmt::Display for XData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "XData({}): {} items", self.app_name, self.data.len())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum XDataItem {
    String(String),
    Real(f64),
    Point(super::geometry::Point),
    Integer(i32),
    Long(i64),
    Boolean(bool),
    Binary(Vec<u8>),
    Handle(String),
    Point3D(f64, f64, f64),
}

impl XDataItem {
    #[inline]
    pub fn type_code(&self) -> u16 {
        match self {
            XDataItem::String(_) => 1000,
            XDataItem::Real(_) => 1040,
            XDataItem::Point(_) => 1010,
            XDataItem::Integer(_) => 1070,
            XDataItem::Long(_) => 1071,
            XDataItem::Boolean(_) => 1076,
            XDataItem::Binary(_) => 1004,
            XDataItem::Handle(_) => 1005,
            XDataItem::Point3D(..) => 1011,
        }
    }
}

impl fmt::Display for XDataItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            XDataItem::String(s) => write!(f, "String({})", s),
            XDataItem::Real(r) => write!(f, "Real({})", r),
            XDataItem::Point(p) => write!(f, "Point({})", p),
            XDataItem::Integer(i) => write!(f, "Integer({})", i),
            XDataItem::Long(l) => write!(f, "Long({})", l),
            XDataItem::Boolean(b) => write!(f, "Boolean({})", b),
            XDataItem::Binary(b) => write!(f, "Binary({} bytes)", b.len()),
            XDataItem::Handle(h) => write!(f, "Handle({})", h),
            XDataItem::Point3D(x, y, z) => write!(f, "Point3D({}, {}, {})", x, y, z),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XDataSet {
    xdata_list: Vec<XData>,
}

impl Default for XDataSet {
    fn default() -> Self {
        Self {
            xdata_list: Vec::new(),
        }
    }
}

impl XDataSet {
    #[inline]
    pub fn new() -> Self {
        Self {
            xdata_list: Vec::new(),
        }
    }

    #[inline]
    pub fn add(&mut self, xdata: XData) {
        self.xdata_list.push(xdata);
    }

    #[inline]
    pub fn get(&self, app_name: &str) -> Option<&XData> {
        self.xdata_list.iter().find(|xd| xd.app_name == app_name)
    }

    #[inline]
    pub fn get_mut(&mut self, app_name: &str) -> Option<&mut XData> {
        self.xdata_list.iter_mut().find(|xd| xd.app_name == app_name)
    }

    #[inline]
    pub fn has(&self, app_name: &str) -> bool {
        self.xdata_list.iter().any(|xd| xd.app_name == app_name)
    }

    #[inline]
    pub fn remove(&mut self, app_name: &str) -> bool {
        let len = self.xdata_list.len();
        self.xdata_list.retain(|xd| xd.app_name != app_name);
        self.xdata_list.len() != len
    }

    #[inline]
    pub fn clear(&mut self) {
        self.xdata_list.clear();
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.xdata_list.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.xdata_list.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &XData> {
        self.xdata_list.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut XData> {
        self.xdata_list.iter_mut()
    }

    #[inline]
    pub fn app_names(&self) -> Vec<&str> {
        self.xdata_list.iter().map(|xd| xd.app_name.as_str()).collect()
    }

    #[inline]
    pub fn get_or_create(&mut self, app_name: &str) -> &mut XData {
        if self.get(app_name).is_none() {
            self.add(XData::new(app_name));
        }
        self.get_mut(app_name).unwrap()
    }

    #[inline]
    pub fn total_item_count(&self) -> usize {
        self.xdata_list.iter().map(|xd| xd.len()).sum()
    }
}

/// 简单通配符匹配，支持 `*` 和 `?`，不依赖 regex
pub(crate) fn wildcard_match(s: &str, pattern: &str) -> bool {
    let s = s.as_bytes();
    let p = pattern.as_bytes();
    let (mut si, mut pi) = (0usize, 0usize);
    let (mut star, mut ss) = (usize::MAX, 0usize);
    while si < s.len() {
        if pi < p.len() && (p[pi] == b'?' || p[pi] == s[si]) {
            si += 1;
            pi += 1;
        } else if pi < p.len() && p[pi] == b'*' {
            star = pi;
            ss = si;
            pi += 1;
        } else if star != usize::MAX {
            pi = star + 1;
            ss += 1;
            si = ss;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == b'*' {
        pi += 1;
    }
    pi == p.len()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomData {
    id: String,
    name: String,
    data_type: DataType,
    value: CustomValue,
    description: String,
    is_required: bool,
    validation: Option<ValidationRule>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    String,
    Integer,
    Real,
    Boolean,
    Point,
    Point3D,
    Handle,
    Enum,
}

impl Default for DataType {
    fn default() -> Self {
        DataType::String
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomValue {
    String(String),
    Integer(i32),
    Real(f64),
    Boolean(bool),
    Point(super::geometry::Point),
    Point3D(f64, f64, f64),
    Handle(String),
    Enum(String),
}

impl fmt::Display for CustomValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CustomValue::String(s) => write!(f, "{}", s),
            CustomValue::Integer(i) => write!(f, "{}", i),
            CustomValue::Real(r) => write!(f, "{}", r),
            CustomValue::Boolean(b) => write!(f, "{}", b),
            CustomValue::Point(p) => write!(f, "{}", p),
            CustomValue::Point3D(x, y, z) => write!(f, "({}, {}, {})", x, y, z),
            CustomValue::Handle(h) => write!(f, "{}", h),
            CustomValue::Enum(e) => write!(f, "{}", e),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    min_value: Option<CustomValue>,
    max_value: Option<CustomValue>,
    pattern: Option<String>,
    enum_values: Vec<String>,
}

impl ValidationRule {
    #[inline]
    pub fn new() -> Self {
        Self {
            min_value: None,
            max_value: None,
            pattern: None,
            enum_values: Vec::new(),
        }
    }

    #[inline]
    pub fn with_range(min: CustomValue, max: CustomValue) -> Self {
        Self {
            min_value: Some(min),
            max_value: Some(max),
            pattern: None,
            enum_values: Vec::new(),
        }
    }

    #[inline]
    pub fn with_pattern(pattern: &str) -> Self {
        Self {
            min_value: None,
            max_value: None,
            pattern: Some(pattern.to_string()),
            enum_values: Vec::new(),
        }
    }

    #[inline]
    pub fn with_enum_values(values: &[&str]) -> Self {
        Self {
            min_value: None,
            max_value: None,
            pattern: None,
            enum_values: values.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[inline]
    pub fn validate(&self, value: &CustomValue) -> bool {
        if let Some(ref min) = self.min_value {
            if !self.ge(value, min) {
                return false;
            }
        }
        if let Some(ref max) = self.max_value {
            if !self.le(value, max) {
                return false;
            }
        }
        if let Some(ref pattern) = self.pattern {
            if let CustomValue::String(s) = value {
                if !wildcard_match(s, pattern) {
                    return false;
                }
            } else {
                return false;
            }
        }
        if !self.enum_values.is_empty() {
            if let CustomValue::Enum(e) = value {
                if !self.enum_values.contains(e) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    fn ge(&self, a: &CustomValue, b: &CustomValue) -> bool {
        match (a, b) {
            (CustomValue::Integer(i1), CustomValue::Integer(i2)) => i1 >= i2,
            (CustomValue::Real(r1), CustomValue::Real(r2)) => r1 >= r2,
            (CustomValue::Real(r1), CustomValue::Integer(i2)) => *r1 >= *i2 as f64,
            (CustomValue::Integer(i1), CustomValue::Real(r2)) => *i1 as f64 >= *r2,
            _ => false,
        }
    }

    fn le(&self, a: &CustomValue, b: &CustomValue) -> bool {
        match (a, b) {
            (CustomValue::Integer(i1), CustomValue::Integer(i2)) => i1 <= i2,
            (CustomValue::Real(r1), CustomValue::Real(r2)) => r1 <= r2,
            (CustomValue::Real(r1), CustomValue::Integer(i2)) => *r1 <= *i2 as f64,
            (CustomValue::Integer(i1), CustomValue::Real(r2)) => *i1 as f64 <= *r2,
            _ => false,
        }
    }
}

impl Default for ValidationRule {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomData {
    #[inline]
    pub fn new(id: &str, name: &str, data_type: DataType) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            data_type,
            value: CustomValue::String(String::new()),
            description: String::new(),
            is_required: false,
            validation: None,
        }
    }

    #[inline]
    pub fn with_value(mut self, value: CustomValue) -> Self {
        self.value = value;
        self
    }

    #[inline]
    pub fn set_value(&mut self, value: CustomValue) -> bool {
        if let Some(ref validation) = self.validation {
            if !validation.validate(&value) {
                return false;
            }
        }
        self.value = value;
        true
    }

    #[inline]
    pub fn description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    #[inline]
    pub fn required(mut self, required: bool) -> Self {
        self.is_required = required;
        self
    }

    #[inline]
    pub fn with_validation(mut self, validation: ValidationRule) -> Self {
        self.validation = Some(validation);
        self
    }
}

impl Default for CustomData {
    fn default() -> Self {
        Self::new("default", "Default", DataType::String)
    }
}

#[derive(Debug, Clone)]
pub struct ExtendedDataRegistry {
    schemas: HashMap<String, ExtendedSchema>,
    app_names: Vec<String>,
}

impl Default for ExtendedDataRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtendedDataRegistry {
    #[inline]
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
            app_names: Vec::new(),
        }
    }

    #[inline]
    pub fn register_schema(&mut self, schema: ExtendedSchema) -> bool {
        if schema.app_name.is_empty() || schema.id.is_empty() {
            return false;
        }
        let app_name = schema.app_name.clone();
        self.schemas.insert(schema.id.clone(), schema);
        self.app_names.push(app_name);
        true
    }

    #[inline]
    pub fn get_schema(&self, id: &str) -> Option<&ExtendedSchema> {
        self.schemas.get(id)
    }

    #[inline]
    pub fn has_app(&self, app_name: &str) -> bool {
        self.app_names.contains(&app_name.to_string())
    }

    #[inline]
    pub fn schemas_for_app(&self, app_name: &str) -> Vec<&ExtendedSchema> {
        self.schemas.values()
            .filter(|s| s.app_name == app_name)
            .collect()
    }

    #[inline]
    pub fn unregister(&mut self, id: &str) -> bool {
        if let Some(schema) = self.schemas.remove(id) {
            self.app_names.retain(|name| name != &schema.app_name);
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.schemas.clear();
        self.app_names.clear();
    }

    #[inline]
    pub fn schema_count(&self) -> usize {
        self.schemas.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedSchema {
    pub id: String,
    pub app_name: String,
    pub name: String,
    pub description: String,
    pub fields: Vec<CustomData>,
    pub version: u32,
    pub is_active: bool,
}

impl ExtendedSchema {
    #[inline]
    pub fn new(id: &str, app_name: &str) -> Self {
        Self {
            id: id.to_string(),
            app_name: app_name.to_string(),
            name: String::new(),
            description: String::new(),
            fields: Vec::new(),
            version: 1,
            is_active: true,
        }
    }

    #[inline]
    pub fn add_field(&mut self, field: CustomData) {
        self.fields.push(field);
    }

    #[inline]
    pub fn get_field(&self, id: &str) -> Option<&CustomData> {
        self.fields.iter().find(|f| f.id == id)
    }

    #[inline]
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }
}

impl Default for ExtendedSchema {
    fn default() -> Self {
        Self::new("default", "default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xdata_creation() {
        let xdata = XData::new("TestApp");
        assert_eq!(xdata.app_name, "TestApp");
        assert!(xdata.is_empty());
    }

    #[test]
    fn test_xdata_operations() {
        let mut xdata = XData::new("TestApp");
        xdata.add_string("Test");
        xdata.add_real(3.14);
        xdata.add_integer(42);
        xdata.add_boolean(true);

        assert_eq!(xdata.len(), 4);
        assert!(!xdata.is_empty());
    }

    #[test]
    fn test_xdata_strings() {
        let mut xdata = XData::new("TestApp");
        xdata.add_string("String1");
        xdata.add_real(3.14);
        xdata.add_string("String2");

        let strings = xdata.strings();
        assert_eq!(strings.len(), 2);
        assert_eq!(strings[0], "String1");
        assert_eq!(strings[1], "String2");
    }

    #[test]
    fn test_xdata_set() {
        let mut xdata_set = XDataSet::new();
        xdata_set.add(XData::new("App1"));
        xdata_set.add(XData::new("App2"));

        assert_eq!(xdata_set.len(), 2);
        assert!(xdata_set.has("App1"));
        assert!(xdata_set.has("App2"));
        assert!(!xdata_set.has("App3"));
    }

    #[test]
    fn test_xdata_get_or_create() {
        let mut xdata_set = XDataSet::new();
        xdata_set.add(XData::new("App1"));

        let xdata = xdata_set.get_or_create("App2");
        assert_eq!(xdata.app_name, "App2");
        assert_eq!(xdata_set.len(), 2);
    }

    #[test]
    fn test_custom_data() {
        let data = CustomData::new("field1", "Field1", DataType::String)
            .with_value(CustomValue::String("Value1".to_string()))
            .description("A test field")
            .required(true);

        assert_eq!(data.id, "field1");
        assert_eq!(data.name, "Field1");
        assert_eq!(data.description, "A test field");
        assert!(data.is_required);
    }

    #[test]
    fn test_validation_rule_range() {
        let rule = ValidationRule::with_range(
            CustomValue::Integer(1),
            CustomValue::Integer(10),
        );

        assert!(rule.validate(&CustomValue::Integer(5)));
        assert!(!rule.validate(&CustomValue::Integer(0)));
        assert!(!rule.validate(&CustomValue::Integer(11)));
    }

    #[test]
    fn test_validation_rule_enum() {
        let rule = ValidationRule::with_enum_values(&["red", "green", "blue"]);

        assert!(rule.validate(&CustomValue::Enum("red".to_string())));
        assert!(!rule.validate(&CustomValue::Enum("yellow".to_string())));
    }

    #[test]
    fn test_extended_schema() {
        let mut schema = ExtendedSchema::new("schema1", "MyApp");
        schema.name = "My Schema".to_string();
        schema.description = "A test schema".to_string();

        schema.add_field(CustomData::new("field1", "Field1", DataType::String));
        schema.add_field(CustomData::new("field2", "Field2", DataType::Integer));

        assert_eq!(schema.field_count(), 2);
    }

    #[test]
    fn test_extended_data_registry() {
        let mut registry = ExtendedDataRegistry::new();
        let schema = ExtendedSchema::new("schema1", "MyApp");

        assert!(registry.register_schema(schema));
        assert!(registry.has_app("MyApp"));
        assert_eq!(registry.schema_count(), 1);

        assert!(registry.unregister("schema1"));
        assert!(!registry.has_app("MyApp"));
    }

    #[test]
    fn test_xdata_item_type_code() {
        assert_eq!(XDataItem::String("test".to_string()).type_code(), 1000);
        assert_eq!(XDataItem::Real(3.14).type_code(), 1040);
        assert_eq!(XDataItem::Integer(42).type_code(), 1070);
        assert_eq!(XDataItem::Boolean(true).type_code(), 1076);
    }

    #[test]
    fn test_xdata_remove() {
        let mut xdata_set = XDataSet::new();
        xdata_set.add(XData::new("App1"));
        xdata_set.add(XData::new("App2"));

        assert!(xdata_set.remove("App1"));
        assert!(!xdata_set.has("App1"));
        assert!(xdata_set.has("App2"));
    }

    #[test]
    fn test_xdata_clear() {
        let mut xdata_set = XDataSet::new();
        xdata_set.add(XData::new("App1"));
        xdata_set.add(XData::new("App2"));

        xdata_set.clear();
        assert!(xdata_set.is_empty());
    }
}
