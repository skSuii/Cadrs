use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub value: ParameterValue,
    pub data_type: ParameterDataType,
    pub unit: Option<String>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub step: Option<f64>,
    pub default_value: Option<ParameterValue>,
    pub expressions: Vec<ParameterExpression>,
    pub constraints: Vec<ParameterConstraint>,
    pub is_read_only: bool,
    pub is_hidden: bool,
    pub is_optional: bool,
    pub category: String,
    pub sub_category: String,
    pub order: i32,
    pub icon: String,
    pub tooltip: String,
    pub validation_rules: Vec<ValidationRule>,
    pub linked_parameters: Vec<String>,
    pub formula: Option<String>,
    pub formula_value: Option<f64>,
}

impl Default for Parameter {
    fn default() -> Self {
        Self::new()
    }
}

impl Parameter {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            display_name: String::new(),
            description: String::new(),
            value: ParameterValue::None,
            data_type: ParameterDataType::String,
            unit: None,
            minimum: None,
            maximum: None,
            step: None,
            default_value: None,
            expressions: Vec::new(),
            constraints: Vec::new(),
            is_read_only: false,
            is_hidden: false,
            is_optional: false,
            category: String::new(),
            sub_category: String::new(),
            order: 0,
            icon: String::new(),
            tooltip: String::new(),
            validation_rules: Vec::new(),
            linked_parameters: Vec::new(),
            formula: None,
            formula_value: None,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn with_display_name(mut self, display_name: &str) -> Self {
        self.display_name = display_name.to_string();
        self
    }

    pub fn with_value(mut self, value: ParameterValue) -> Self {
        self.value = value.clone();
        if self.default_value.is_none() {
            self.default_value = Some(value);
        }
        self
    }

    pub fn with_type(mut self, data_type: ParameterDataType) -> Self {
        self.data_type = data_type;
        self
    }

    pub fn with_unit(mut self, unit: &str) -> Self {
        self.unit = Some(unit.to_string());
        self
    }

    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.minimum = Some(min);
        self.maximum = Some(max);
        self
    }

    pub fn with_step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    pub fn set_value(&mut self, value: ParameterValue) -> bool {
        if self.is_read_only {
            return false;
        }

        if !self.validate(&value) {
            return false;
        }

        self.value = value;
        true
    }

    pub fn validate(&self, value: &ParameterValue) -> bool {
        if let Some(min) = self.minimum {
            if value.to_f64() < min {
                return false;
            }
        }

        if let Some(max) = self.maximum {
            if value.to_f64() > max {
                return false;
            }
        }

        for rule in &self.validation_rules {
            if !rule.validate(value) {
                return false;
            }
        }

        true
    }

    pub fn get_f64(&self) -> Option<f64> {
        match &self.value {
            ParameterValue::Real(r) => Some(*r),
            ParameterValue::Integer(i) => Some(*i as f64),
            ParameterValue::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    pub fn set_f64(&mut self, value: f64) -> bool {
        self.set_value(ParameterValue::Real(value))
    }

    pub fn get_string(&self) -> Option<&str> {
        match &self.value {
            ParameterValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn set_string(&mut self, value: &str) -> bool {
        self.set_value(ParameterValue::String(value.to_string()))
    }

    pub fn get_bool(&self) -> Option<bool> {
        match &self.value {
            ParameterValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn set_bool(&mut self, value: bool) -> bool {
        self.set_value(ParameterValue::Boolean(value))
    }

    pub fn reset_to_default(&mut self) -> bool {
        if let Some(default) = &self.default_value {
            self.set_value(default.clone())
        } else {
            false
        }
    }

    pub fn add_constraint(&mut self, constraint: ParameterConstraint) {
        self.constraints.push(constraint);
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn link_to(&mut self, param_name: &str) {
        if !self.linked_parameters.contains(&param_name.to_string()) {
            self.linked_parameters.push(param_name.to_string());
        }
    }

    pub fn set_formula(&mut self, formula: &str) {
        self.formula = Some(formula.to_string());
        self.evaluate_formula();
    }

    pub fn evaluate_formula(&mut self) {
        if let Some(formula) = &self.formula {
            self.formula_value = Some(parse_simple_expression(formula));
        }
    }

    pub fn format_value(&self) -> String {
        match &self.value {
            ParameterValue::Real(r) => {
                if let Some(unit) = &self.unit {
                    format!("{}{}", r, unit)
                } else {
                    format!("{}", r)
                }
            }
            ParameterValue::String(s) => s.clone(),
            ParameterValue::Integer(i) => i.to_string(),
            ParameterValue::Boolean(b) => b.to_string(),
            ParameterValue::Point(p) => format!("({}, {}, {})", p.x, p.y, p.z),
            ParameterValue::Angle(a) => format!("{:.2}°", a.to_degrees()),
            ParameterValue::None => "None".to_string(),
        }
    }
}

fn parse_simple_expression(expr: &str) -> f64 {
    let cleaned = expr.replace(" ", "");

    if let Ok(val) = cleaned.parse::<f64>() {
        return val;
    }

    let expr_lower = cleaned.to_lowercase();

    if expr_lower == "pi" || expr_lower == "π" {
        return std::f64::consts::PI;
    }
    if expr_lower == "2*pi" || expr_lower == "2π" {
        return 2.0 * std::f64::consts::PI;
    }
    if expr_lower == "pi/2" {
        return std::f64::consts::PI / 2.0;
    }
    if expr_lower == "pi/4" {
        return std::f64::consts::PI / 4.0;
    }
    if expr_lower == "e" {
        return std::f64::consts::E;
    }

    0.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterValue {
    None,
    Real(f64),
    Integer(i32),
    String(String),
    Boolean(bool),
    Point(crate::geometry::Point),
    Angle(f64),
}

impl Default for ParameterValue {
    fn default() -> Self {
        ParameterValue::None
    }
}

impl ParameterValue {
    pub fn to_f64(&self) -> f64 {
        match self {
            ParameterValue::Real(r) => *r,
            ParameterValue::Integer(i) => *i as f64,
            ParameterValue::Boolean(b) => if *b { 1.0 } else { 0.0 },
            _ => 0.0,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            ParameterValue::Real(r) => format!("{}", r),
            ParameterValue::Integer(i) => format!("{}", i),
            ParameterValue::String(s) => s.clone(),
            ParameterValue::Boolean(b) => format!("{}", b),
            ParameterValue::Point(p) => format!("({}, {}, {})", p.x, p.y, p.z),
            ParameterValue::Angle(a) => format!("{}", a),
            ParameterValue::None => String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParameterDataType {
    None,
    Real,
    Integer,
    String,
    Boolean,
    Point,
    Angle,
    Distance,
    Area,
    Volume,
    AngleUnitless,
    Reference,
}

impl Default for ParameterDataType {
    fn default() -> Self {
        ParameterDataType::String
    }
}

impl ParameterDataType {
    pub fn name(&self) -> &str {
        match self {
            ParameterDataType::None => "None",
            ParameterDataType::Real => "Real",
            ParameterDataType::Integer => "Integer",
            ParameterDataType::String => "String",
            ParameterDataType::Boolean => "Boolean",
            ParameterDataType::Point => "Point",
            ParameterDataType::Angle => "Angle",
            ParameterDataType::Distance => "Distance",
            ParameterDataType::Area => "Area",
            ParameterDataType::Volume => "Volume",
            ParameterDataType::AngleUnitless => "Angle (Unitless)",
            ParameterDataType::Reference => "Reference",
        }
    }

    pub fn default_value(&self) -> ParameterValue {
        match self {
            ParameterDataType::Real => ParameterValue::Real(0.0),
            ParameterDataType::Integer => ParameterValue::Integer(0),
            ParameterDataType::String => ParameterValue::String(String::new()),
            ParameterDataType::Boolean => ParameterValue::Boolean(false),
            ParameterDataType::Point => ParameterValue::Point(crate::geometry::Point::origin()),
            ParameterDataType::Angle => ParameterValue::Angle(0.0),
            ParameterDataType::Distance => ParameterValue::Real(0.0),
            ParameterDataType::Area => ParameterValue::Real(0.0),
            ParameterDataType::Volume => ParameterValue::Real(0.0),
            ParameterDataType::AngleUnitless => ParameterValue::Real(1.0),
            ParameterDataType::Reference => ParameterValue::None,
            ParameterDataType::None => ParameterValue::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterExpression {
    pub name: String,
    pub expression: String,
    pub value: f64,
    pub is_valid: bool,
}

impl Default for ParameterExpression {
    fn default() -> Self {
        Self::new()
    }
}

impl ParameterExpression {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            expression: String::new(),
            value: 0.0,
            is_valid: true,
        }
    }

    pub fn with_expression(mut self, expr: &str) -> Self {
        self.expression = expr.to_string();
        self.evaluate();
        self
    }

    pub fn evaluate(&mut self) {
        self.value = parse_simple_expression(&self.expression);
        self.is_valid = self.value.is_finite();
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterConstraint {
    pub name: String,
    pub constraint_type: ParameterConstraintType,
    pub value: ParameterValue,
}

impl Default for ParameterConstraint {
    fn default() -> Self {
        Self::new()
    }
}

impl ParameterConstraint {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            constraint_type: ParameterConstraintType::Fixed,
            value: ParameterValue::None,
        }
    }

    pub fn with_type(mut self, constraint_type: ParameterConstraintType) -> Self {
        self.constraint_type = constraint_type;
        self
    }

    pub fn with_value(mut self, value: ParameterValue) -> Self {
        self.value = value;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParameterConstraintType {
    Fixed,
    List,
    Range,
    Expression,
}

impl Default for ParameterConstraintType {
    fn default() -> Self {
        ParameterConstraintType::Fixed
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationRule {
    pub name: String,
    pub rule_type: ValidationRuleType,
    pub message: String,
    pub value: ParameterValue,
}

impl Default for ValidationRule {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationRule {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            rule_type: ValidationRuleType::Required,
            message: String::new(),
            value: ParameterValue::None,
        }
    }

    pub fn with_type(mut self, rule_type: ValidationRuleType) -> Self {
        self.rule_type = rule_type;
        self
    }

    pub fn with_message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    pub fn validate(&self, value: &ParameterValue) -> bool {
        match self.rule_type {
            ValidationRuleType::Required => {
                !matches!(value, ParameterValue::None)
            }
            ValidationRuleType::MinLength => {
                if let ParameterValue::String(s) = value {
                    s.len() >= self.value.to_f64() as usize
                } else {
                    true
                }
            }
            ValidationRuleType::MaxLength => {
                if let ParameterValue::String(s) = value {
                    s.len() <= self.value.to_f64() as usize
                } else {
                    true
                }
            }
            ValidationRuleType::Pattern => {
                if let ParameterValue::String(s) = value {
                    crate::xdata::wildcard_match(s, &self.message)
                } else {
                    true
                }
            }
            ValidationRuleType::Custom => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationRuleType {
    Required,
    MinLength,
    MaxLength,
    Pattern,
    Custom,
}

impl Default for ValidationRuleType {
    fn default() -> Self {
        ValidationRuleType::Required
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterCategory {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub parent_category: Option<String>,
    pub icon: String,
    pub order: i32,
    pub parameters: Vec<String>,
    pub sub_categories: Vec<String>,
}

impl Default for ParameterCategory {
    fn default() -> Self {
        Self::new()
    }
}

impl ParameterCategory {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            display_name: String::new(),
            description: String::new(),
            parent_category: None,
            icon: String::new(),
            order: 0,
            parameters: Vec::new(),
            sub_categories: Vec::new(),
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn add_parameter(&mut self, param_name: &str) {
        if !self.parameters.contains(&param_name.to_string()) {
            self.parameters.push(param_name.to_string());
        }
    }

    pub fn remove_parameter(&mut self, param_name: &str) {
        self.parameters.retain(|p| p != param_name);
    }

    pub fn add_sub_category(&mut self, category_name: &str) {
        if !self.sub_categories.contains(&category_name.to_string()) {
            self.sub_categories.push(category_name.to_string());
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterGroup {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub parameters: Vec<String>,
    pub is_collapsible: bool,
    pub is_expanded: bool,
    pub order: i32,
}

impl Default for ParameterGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl ParameterGroup {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            display_name: String::new(),
            description: String::new(),
            parameters: Vec::new(),
            is_collapsible: true,
            is_expanded: true,
            order: 0,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn add_parameter(&mut self, param_name: &str) {
        if !self.parameters.contains(&param_name.to_string()) {
            self.parameters.push(param_name.to_string());
        }
    }

    pub fn parameter_count(&self) -> usize {
        self.parameters.len()
    }
}

#[derive(Debug, Clone)]
pub struct ParameterManager {
    parameters: HashMap<String, Parameter>,
    categories: HashMap<String, ParameterCategory>,
    groups: HashMap<String, ParameterGroup>,
    active_category: Option<String>,
    active_group: Option<String>,
    is_modified: bool,
    modified_parameters: Vec<String>,
    parameter_order: Vec<String>,
}

impl Default for ParameterManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ParameterManager {
    pub fn new() -> Self {
        Self {
            parameters: HashMap::new(),
            categories: HashMap::new(),
            groups: HashMap::new(),
            active_category: None,
            active_group: None,
            is_modified: false,
            modified_parameters: Vec::new(),
            parameter_order: Vec::new(),
        }
    }

    pub fn add(&mut self, parameter: Parameter) {
        let name = parameter.name.clone();
        self.parameters.insert(name.clone(), parameter);
        self.parameter_order.push(name);
    }

    pub fn create(&mut self, name: &str) -> &mut Parameter {
        let parameter = Parameter::new().with_name(name);
        self.parameters.insert(name.to_string(), parameter);
        self.parameter_order.push(name.to_string());
        self.parameters.get_mut(name).unwrap()
    }

    pub fn get(&self, name: &str) -> Option<&Parameter> {
        self.parameters.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Parameter> {
        self.parameters.get_mut(name)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.parameter_order.retain(|n| n != name);
        self.parameters.remove(name).is_some()
    }

    pub fn rename(&mut self, old_name: &str, new_name: &str) -> bool {
        if let Some(param) = self.parameters.remove(old_name) {
            let mut new_param = param;
            new_param.name = new_name.to_string();
            self.parameters.insert(new_name.to_string(), new_param);
            self.parameter_order.retain(|n| n != old_name);
            self.parameter_order.push(new_name.to_string());
            true
        } else {
            false
        }
    }

    pub fn set_value(&mut self, name: &str, value: ParameterValue) -> bool {
        if let Some(param) = self.parameters.get_mut(name) {
            let result = param.set_value(value);
            if result {
                self.is_modified = true;
                if !self.modified_parameters.contains(&name.to_string()) {
                    self.modified_parameters.push(name.to_string());
                }
            }
            result
        } else {
            false
        }
    }

    pub fn set_f64(&mut self, name: &str, value: f64) -> bool {
        self.set_value(name, ParameterValue::Real(value))
    }

    pub fn set_string(&mut self, name: &str, value: &str) -> bool {
        self.set_value(name, ParameterValue::String(value.to_string()))
    }

    pub fn get_f64(&self, name: &str) -> Option<f64> {
        self.parameters.get(name).and_then(|p| p.get_f64())
    }

    pub fn get_string(&self, name: &str) -> Option<&str> {
        self.parameters.get(name).and_then(|p| p.get_string())
    }

    pub fn parameter_count(&self) -> usize {
        self.parameters.len()
    }

    pub fn parameter_names(&self) -> Vec<&str> {
        self.parameter_order.iter().map(|s| s.as_str()).collect()
    }

    pub fn parameters_in_category(&self, category: &str) -> Vec<&Parameter> {
        self.parameters.values()
            .filter(|p| p.category == category)
            .collect()
    }

    pub fn parameters_in_group(&self, group: &str) -> Vec<&Parameter> {
        self.groups.get(group)
            .map(|g| {
                g.parameters.iter()
                    .filter_map(|name| self.parameters.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn add_category(&mut self, category: ParameterCategory) {
        self.categories.insert(category.name.clone(), category);
    }

    pub fn get_category(&self, name: &str) -> Option<&ParameterCategory> {
        self.categories.get(name)
    }

    pub fn category_names(&self) -> Vec<&str> {
        self.categories.keys().map(|s| s.as_str()).collect()
    }

    pub fn add_group(&mut self, group: ParameterGroup) {
        self.groups.insert(group.name.clone(), group);
    }

    pub fn get_group(&self, name: &str) -> Option<&ParameterGroup> {
        self.groups.get(name)
    }

    pub fn group_names(&self) -> Vec<&str> {
        self.groups.keys().map(|s| s.as_str()).collect()
    }

    pub fn set_active_category(&mut self, category: Option<&str>) {
        self.active_category = category.map(|s| s.to_string());
    }

    pub fn set_active_group(&mut self, group: Option<&str>) {
        self.active_group = group.map(|s| s.to_string());
    }

    pub fn active_category(&self) -> Option<&str> {
        self.active_category.as_deref()
    }

    pub fn active_group(&self) -> Option<&str> {
        self.active_group.as_deref()
    }

    pub fn search(&self, query: &str) -> Vec<&Parameter> {
        let query_lower = query.to_lowercase();
        self.parameters.values()
            .filter(|p| {
                p.name.to_lowercase().contains(&query_lower)
                    || p.display_name.to_lowercase().contains(&query_lower)
                    || p.description.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    pub fn is_modified(&self) -> bool {
        self.is_modified
    }

    pub fn clear_modified(&mut self) {
        self.is_modified = false;
        self.modified_parameters.clear();
    }

    pub fn modified_parameter_names(&self) -> Vec<&str> {
        self.modified_parameters.iter().map(|s| s.as_str()).collect()
    }

    pub fn reset_all_to_defaults(&mut self) -> usize {
        let mut reset_count = 0;
        for param in self.parameters.values_mut() {
            if param.reset_to_default() {
                reset_count += 1;
            }
        }
        reset_count
    }

    pub fn validate_all(&mut self) -> Vec<&str> {
        let mut invalid = Vec::new();
        for name in self.parameters.keys() {
            if let Some(param) = self.parameters.get(name) {
                if !param.validate(&param.value) {
                    invalid.push(name.as_str());
                }
            }
        }
        invalid
    }

    pub fn export_to_json(&self) -> String {
        serde_json::to_string_pretty(&self.parameters).unwrap_or_default()
    }

    pub fn import_from_json(&mut self, json: &str) -> Result<usize, String> {
        let imported: HashMap<String, Parameter> = serde_json::from_str(json)
            .map_err(|e| e.to_string())?;

        let count = imported.len();
        for (name, param) in imported {
            self.parameters.insert(name, param);
        }
        Ok(count)
    }

    pub fn clear(&mut self) {
        self.parameters.clear();
        self.categories.clear();
        self.groups.clear();
        self.parameter_order.clear();
        self.modified_parameters.clear();
        self.active_category = None;
        self.active_group = None;
        self.is_modified = false;
    }

    pub fn duplicate(&mut self, source_name: &str, new_name: &str) -> bool {
        if let Some(source) = self.parameters.get(source_name) {
            let mut new_param = source.clone();
            new_param.name = new_name.to_string();
            self.parameters.insert(new_name.to_string(), new_param);
            self.parameter_order.push(new_name.to_string());
            true
        } else {
            false
        }
    }

    pub fn get_statistics(&self) -> ParameterStatistics {
        let mut category_counts = HashMap::new();
        let mut type_counts = HashMap::new();

        for param in self.parameters.values() {
            *category_counts.entry(param.category.clone()).or_insert(0) += 1;
            *type_counts.entry(param.data_type).or_insert(0) += 1;
        }

        ParameterStatistics {
            total_parameters: self.parameters.len(),
            categories: self.categories.len(),
            groups: self.groups.len(),
            modified: self.modified_parameters.len(),
            category_counts,
            type_counts,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterStatistics {
    pub total_parameters: usize,
    pub categories: usize,
    pub groups: usize,
    pub modified: usize,
    pub category_counts: HashMap<String, usize>,
    pub type_counts: HashMap<ParameterDataType, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_creation() {
        let param = Parameter::new()
            .with_name("Length")
            .with_display_name("Length")
            .with_value(ParameterValue::Real(100.0))
            .with_unit("mm")
            .with_range(0.0, 1000.0);

        assert_eq!(param.name, "Length");
        assert_eq!(param.data_type, ParameterDataType::String);
        assert_eq!(param.unit, Some("mm".to_string()));
    }

    #[test]
    fn test_parameter_value_operations() {
        let mut param = Parameter::new()
            .with_name("Length")
            .with_type(ParameterDataType::Distance)
            .with_value(ParameterValue::Real(100.0));

        assert!(param.set_f64(200.0));
        assert!((param.get_f64().unwrap() - 200.0).abs() < 1e-10);

        assert!(!param.set_f64(-10.0));
    }

    #[test]
    fn test_parameter_range_validation() {
        let mut param = Parameter::new()
            .with_name("Width")
            .with_range(0.0, 100.0);

        assert!(param.set_f64(50.0));
        assert!(!param.set_f64(150.0));
    }

    #[test]
    fn test_parameter_string_operations() {
        let mut param = Parameter::new()
            .with_name("Description")
            .with_type(ParameterDataType::String);

        assert!(param.set_string("Test"));
        assert_eq!(param.get_string(), Some("Test"));
    }

    #[test]
    fn test_parameter_reset_to_default() {
        let mut param = Parameter::new()
            .with_name("Length")
            .with_value(ParameterValue::Real(100.0));

        param.set_f64(200.0);
        param.reset_to_default();
        assert!((param.get_f64().unwrap() - 100.0).abs() < 1e-10);
    }

    #[test]
    fn test_parameter_value_conversions() {
        assert!((ParameterValue::Real(3.14).to_f64() - 3.14).abs() < 1e-10);
        assert!((ParameterValue::Integer(42).to_f64() - 42.0).abs() < 1e-10);
        assert!((ParameterValue::Boolean(true).to_f64() - 1.0).abs() < 1e-10);
        assert!((ParameterValue::Boolean(false).to_f64() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_parameter_data_types() {
        assert_eq!(ParameterDataType::Real.name(), "Real");
        assert_eq!(ParameterDataType::Distance.name(), "Distance");
        assert_eq!(ParameterDataType::Point.name(), "Point");
    }

    #[test]
    fn test_parameter_expression() {
        let mut expr = ParameterExpression::new().with_expression("PI");
        assert!((expr.value - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_parameter_category() {
        let mut category = ParameterCategory::new().with_name("Geometry");
        category.add_parameter("Length");
        category.add_parameter("Width");

        assert_eq!(category.parameters.len(), 2);
    }

    #[test]
    fn test_parameter_group() {
        let mut group = ParameterGroup::new().with_name("Dimensions");
        group.add_parameter("Length");
        group.add_parameter("Width");

        assert_eq!(group.parameter_count(), 2);
    }

    #[test]
    fn test_parameter_manager_creation() {
        let manager = ParameterManager::new();
        assert_eq!(manager.parameter_count(), 0);
    }

    #[test]
    fn test_parameter_manager_add() {
        let mut manager = ParameterManager::new();
        manager.add(
            Parameter::new()
                .with_name("Length")
                .with_value(ParameterValue::Real(100.0))
        );

        assert_eq!(manager.parameter_count(), 1);
        assert!(manager.get("Length").is_some());
    }

    #[test]
    fn test_parameter_manager_set_value() {
        let mut manager = ParameterManager::new();
        manager.add(
            Parameter::new()
                .with_name("Length")
                .with_value(ParameterValue::Real(100.0))
        );

        assert!(manager.set_f64("Length", 200.0));
        assert!((manager.get_f64("Length").unwrap() - 200.0).abs() < 1e-10);
    }

    #[test]
    fn test_parameter_manager_remove() {
        let mut manager = ParameterManager::new();
        manager.add(Parameter::new().with_name("Length"));
        assert_eq!(manager.parameter_count(), 1);

        assert!(manager.remove("Length"));
        assert_eq!(manager.parameter_count(), 0);
    }

    #[test]
    fn test_parameter_manager_rename() {
        let mut manager = ParameterManager::new();
        manager.add(Parameter::new().with_name("Length"));

        assert!(manager.rename("Length", "Width"));
        assert!(manager.get("Width").is_some());
        assert!(manager.get("Length").is_none());
    }

    #[test]
    fn test_parameter_manager_search() {
        let mut manager = ParameterManager::new();
        manager.add(Parameter::new().with_name("Length").with_display_name("Length of beam"));
        manager.add(Parameter::new().with_name("Width").with_display_name("Width of beam"));

        let results = manager.search("beam");
        assert_eq!(results.len(), 2);

        let results = manager.search("beam len");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_parameter_manager_reset_all() {
        let mut manager = ParameterManager::new();
        manager.add(Parameter::new().with_name("P1").with_value(ParameterValue::Real(100.0)));
        manager.add(Parameter::new().with_name("P2").with_value(ParameterValue::Real(200.0)));

        manager.set_f64("P1", 150.0);
        manager.set_f64("P2", 250.0);

        let reset_count = manager.reset_all_to_defaults();
        assert_eq!(reset_count, 2);
    }

    #[test]
    fn test_parameter_manager_validate_all() {
        let mut manager = ParameterManager::new();
        let mut param = Parameter::new().with_name("Length").with_range(0.0, 100.0);
        param.set_f64(50.0);
        manager.add(param);

        let invalid = manager.validate_all();
        assert_eq!(invalid.len(), 0);
    }

    #[test]
    fn test_parameter_manager_export_import() {
        let mut manager = ParameterManager::new();
        manager.add(Parameter::new().with_name("Length").with_value(ParameterValue::Real(100.0)));

        let json = manager.export_to_json();
        assert!(json.contains("Length"));

        let mut manager2 = ParameterManager::new();
        manager2.import_from_json(&json).unwrap();
        assert_eq!(manager2.parameter_count(), 1);
    }

    #[test]
    fn test_parameter_manager_duplicate() {
        let mut manager = ParameterManager::new();
        manager.add(Parameter::new().with_name("Length").with_value(ParameterValue::Real(100.0)));

        assert!(manager.duplicate("Length", "Width"));
        assert_eq!(manager.parameter_count(), 2);
    }

    #[test]
    fn test_parameter_manager_statistics() {
        let mut manager = ParameterManager::new();
        manager.add(Parameter::new().with_name("Length").with_value(ParameterValue::Real(100.0)));
        manager.add(Parameter::new().with_name("Width").with_value(ParameterValue::Real(50.0)));

        let stats = manager.get_statistics();
        assert_eq!(stats.total_parameters, 2);
    }

    #[test]
    fn test_validation_rule() {
        let mut rule = ValidationRule::new()
            .with_type(ValidationRuleType::MinLength)
            .with_message("3");

        assert!(rule.validate(&ParameterValue::String("Hello".to_string())));
        assert!(!rule.validate(&ParameterValue::String("Hi".to_string())));
    }

    #[test]
    fn test_parameter_linked() {
        let mut param = Parameter::new().with_name("Width");
        param.link_to("Length");
        assert_eq!(param.linked_parameters.len(), 1);
    }

    #[test]
    fn test_parameter_format_value() {
        let mut param = Parameter::new()
            .with_name("Length")
            .with_unit("mm");

        param.set_f64(100.0);
        assert_eq!(param.format_value(), "100mm");
    }

    #[test]
    fn test_parameter_constraint() {
        let param = ParameterConstraint::new()
            .with_type(ParameterConstraintType::Range)
            .with_value(ParameterValue::Real(50.0));

        assert_eq!(param.constraint_type, ParameterConstraintType::Range);
    }
}
