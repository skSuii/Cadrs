use crate::hatch::hatch::Color;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LayerManager {
    pub layers: HashMap<String, Layer>,
    pub layer_groups: Vec<LayerGroup>,
    pub active_layer: Option<String>,
    pub layer_states: LayerStateManager,
    pub filters: Vec<LayerFilter>,
}

impl LayerManager {
    pub fn new() -> Self {
        let mut manager = Self {
            layers: HashMap::new(),
            layer_groups: Vec::new(),
            active_layer: None,
            layer_states: LayerStateManager::new(),
            filters: Vec::new(),
        };

        manager.add_default_layers();
        manager
    }

    fn add_default_layers(&mut self) {
        let mut layer = Layer::new(String::from("0"));
        layer.color = Color::white();
        layer.is_locked = false;
        layer.is_visible = true;
        layer.is_printable = true;
        self.layers.insert(String::from("0"), layer);
        
        self.active_layer = Some(String::from("0"));

        let mut defpoints = Layer::new(String::from("DEFPOINTS"));
        defpoints.color = Color::white();
        defpoints.description = Some(String::from("Layer for definition points"));
        self.layers.insert(String::from("DEFPOINTS"), defpoints);

        let mut viewport = Layer::new(String::from("VIEWPORT"));
        viewport.color = Color::rgb(255, 255, 0);
        viewport.description = Some(String::from("Viewport layer"));
        self.layers.insert(String::from("VIEWPORT"), viewport);

        let mut construction = Layer::new(String::from("CONSTRUCTION"));
        construction.color = Color::rgb(100, 100, 100);
        construction.description = Some(String::from("Construction geometry"));
        construction.line_type = Some(String::from("CENTER"));
        self.layers.insert(String::from("CONSTRUCTION"), construction);
    }

    pub fn create_layer(&mut self, name: String) -> &mut Layer {
        let layer = Layer::new(name.clone());
        self.layers.insert(name.clone(), layer);
        self.active_layer = Some(name.clone());
        self.layers.get_mut(&name).unwrap()
    }

    pub fn get_layer(&self, name: &str) -> Option<&Layer> {
        self.layers.get(name)
    }

    pub fn get_layer_mut(&mut self, name: &str) -> Option<&mut Layer> {
        self.layers.get_mut(name)
    }

    pub fn delete_layer(&mut self, name: &str) -> bool {
        if name == "0" {
            return false;
        }
        self.layers.remove(name);
        true
    }

    pub fn rename_layer(&mut self, old_name: &str, new_name: &str) -> bool {
        if old_name == "0" || new_name == "0" {
            return false;
        }
        
        if let Some(mut layer) = self.layers.remove(old_name) {
            layer.name = new_name.to_string();
            self.layers.insert(new_name.to_string(), layer);
            true
        } else {
            false
        }
    }

    pub fn set_active_layer(&mut self, name: &str) -> bool {
        if self.layers.contains_key(name) {
            self.active_layer = Some(name.to_string());
            true
        } else {
            false
        }
    }

    pub fn get_active_layer(&self) -> Option<&Layer> {
        self.active_layer.as_ref().and_then(|name| self.layers.get(name))
    }

    pub fn get_active_layer_mut(&mut self) -> Option<&mut Layer> {
        self.active_layer.as_ref().and_then(|name| self.layers.get_mut(name))
    }

    pub fn create_layer_group(&mut self, name: String, description: String) -> &mut LayerGroup {
        let group = LayerGroup::new(name, description);
        self.layer_groups.push(group);
        self.layer_groups.last_mut().unwrap()
    }

    pub fn add_layer_to_group(&mut self, layer_name: &str, group_name: &str) -> bool {
        if let Some(group) = self.layer_groups.iter_mut().find(|g| g.name == group_name) {
            if self.layers.contains_key(layer_name) {
                group.layer_names.push(layer_name.to_string());
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn remove_layer_from_group(&mut self, layer_name: &str, group_name: &str) -> bool {
        if let Some(group) = self.layer_groups.iter_mut().find(|g| g.name == group_name) {
            let original_len = group.layer_names.len();
            group.layer_names.retain(|n| n != layer_name);
            group.layer_names.len() < original_len
        } else {
            false
        }
    }

    pub fn get_group(&self, name: &str) -> Option<&LayerGroup> {
        self.layer_groups.iter().find(|g| g.name == name)
    }

    pub fn apply_filter(&self, filter: &LayerFilter) -> Vec<&Layer> {
        self.layers.values()
            .filter(|layer| filter.matches(layer))
            .collect()
    }

    pub fn create_filter(&mut self, name: String, criteria: FilterCriteria) -> &mut LayerFilter {
        let filter = LayerFilter::new(name, criteria);
        self.filters.push(filter);
        self.filters.last_mut().unwrap()
    }

    pub fn save_state(&mut self, state_name: String) {
        self.layer_states.save_state(state_name, &self.layers);
    }

    pub fn restore_state(&mut self, state_name: &str) -> bool {
        if let Some(saved_layers) = self.layer_states.get_state(state_name) {
            for (name, saved_layer) in saved_layers {
                if let Some(layer) = self.layers.get_mut(name) {
                    layer.is_visible = saved_layer.is_visible;
                    layer.is_locked = saved_layer.is_locked;
                    layer.is_printable = saved_layer.is_printable;
                    layer.color = saved_layer.color;
                    layer.line_weight = saved_layer.line_weight;
                    layer.line_type = saved_layer.line_type.clone();
                }
            }
            true
        } else {
            false
        }
    }

    pub fn available_states(&self) -> Vec<&String> {
        self.layer_states.available_states()
    }

    pub fn all_layers(&self) -> Vec<&Layer> {
        self.layers.values().collect()
    }

    pub fn visible_layers(&self) -> Vec<&Layer> {
        self.layers.values()
            .filter(|l| l.is_visible)
            .collect()
    }

    pub fn locked_layers(&self) -> Vec<&Layer> {
        self.layers.values()
            .filter(|l| l.is_locked)
            .collect()
    }

    pub fn printable_layers(&self) -> Vec<&Layer> {
        self.layers.values()
            .filter(|l| l.is_printable)
            .collect()
    }

    pub fn freeze_all(&mut self) {
        for layer in self.layers.values_mut() {
            if layer.name != "0" {
                layer.is_visible = false;
                layer.is_frozen = true;
            }
        }
    }

    pub fn thaw_all(&mut self) {
        for layer in self.layers.values_mut() {
            layer.is_frozen = false;
        }
    }

    pub fn lock_all(&mut self) {
        for layer in self.layers.values_mut() {
            if layer.name != "0" {
                layer.is_locked = true;
            }
        }
    }

    pub fn unlock_all(&mut self) {
        for layer in self.layers.values_mut() {
            layer.is_locked = false;
        }
    }
}

impl Default for LayerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Layer {
    pub name: String,
    pub color: Color,
    pub line_type: Option<String>,
    pub line_weight: f64,
    pub is_visible: bool,
    pub is_locked: bool,
    pub is_printable: bool,
    pub is_frozen: bool,
    pub plot_style: Option<PlotStyle>,
    pub description: Option<String>,
    pub extended_properties: HashMap<String, LayerProperty>,
}

impl Layer {
    pub fn new(name: String) -> Self {
        Self {
            name,
            color: Color::rgb(255, 255, 255),
            line_type: None,
            line_weight: -1.0,
            is_visible: true,
            is_locked: false,
            is_printable: true,
            is_frozen: false,
            plot_style: None,
            description: None,
            extended_properties: HashMap::new(),
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_line_type(mut self, line_type: String) -> Self {
        self.line_type = Some(line_type);
        self
    }

    pub fn with_line_weight(mut self, weight: f64) -> Self {
        self.line_weight = weight;
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_plot_style(mut self, plot_style: PlotStyle) -> Self {
        self.plot_style = Some(plot_style);
        self
    }

    pub fn add_property(mut self, name: String, property: LayerProperty) -> Self {
        self.extended_properties.insert(name, property);
        self
    }

    pub fn is_editable(&self) -> bool {
        !self.is_locked && self.is_visible && !self.is_frozen
    }
}

#[derive(Debug, Clone)]
pub struct PlotStyle {
    pub name: String,
    pub color: Option<Color>,
    pub screen: u8,
    pub screen_pattern: ScreenPattern,
    pub linetype_scale: f64,
    pub adaptive_linetype: bool,
    pub lineweight: Option<f64>,
    pub end_style: LineEndStyle,
    pub join_style: LineJoinStyle,
    pub fill_style: FillStyle,
}

impl PlotStyle {
    pub fn new(name: String) -> Self {
        Self {
            name,
            color: None,
            screen: 100,
            screen_pattern: ScreenPattern::Solid,
            linetype_scale: 1.0,
            adaptive_linetype: true,
            lineweight: None,
            end_style: LineEndStyle::Round,
            join_style: LineJoinStyle::Round,
            fill_style: FillStyle::Solid,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_screen(mut self, screen: u8) -> Self {
        self.screen = screen.clamp(0, 100);
        self
    }

    pub fn with_linetype_scale(mut self, scale: f64) -> Self {
        self.linetype_scale = scale;
        self
    }

    pub fn with_lineweight(mut self, weight: f64) -> Self {
        self.lineweight = Some(weight);
        self
    }

    pub fn with_end_style(mut self, style: LineEndStyle) -> Self {
        self.end_style = style;
        self
    }

    pub fn with_join_style(mut self, style: LineJoinStyle) -> Self {
        self.join_style = style;
        self
    }

    pub fn with_fill_style(mut self, style: FillStyle) -> Self {
        self.fill_style = style;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScreenPattern {
    Solid,
    Crosshatch,
    Diagonal,
    Horizontal,
    Vertical,
    Grid,
    Dots,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineEndStyle {
    Butt,
    Round,
    Square,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineJoinStyle {
    Miter,
    Round,
    Bevel,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FillStyle {
    Solid,
    Hatch,
    OutlinedHatch,
    Background,
}

#[derive(Debug, Clone)]
pub enum LayerProperty {
    String(String),
    Number(f64),
    Boolean(bool),
    Color(Color),
    ColorIndex(u16),
}

#[derive(Debug, Clone)]
pub struct LayerGroup {
    pub name: String,
    pub description: String,
    pub layer_names: Vec<String>,
    pub is_expanded: bool,
}

impl LayerGroup {
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            layer_names: Vec::new(),
            is_expanded: true,
        }
    }

    pub fn add_layer(mut self, layer_name: String) -> Self {
        self.layer_names.push(layer_name);
        self
    }

    pub fn toggle_expand(&mut self) {
        self.is_expanded = !self.is_expanded;
    }

    pub fn get_layers<'a>(&self, layer_manager: &'a LayerManager) -> Vec<&'a Layer> {
        self.layer_names.iter()
            .filter_map(|name| layer_manager.get_layer(name))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct LayerFilter {
    pub name: String,
    pub criteria: FilterCriteria,
    pub is_group_filter: bool,
    pub nested_filters: Vec<LayerFilter>,
}

impl LayerFilter {
    pub fn new(name: String, criteria: FilterCriteria) -> Self {
        Self {
            name,
            criteria,
            is_group_filter: false,
            nested_filters: Vec::new(),
        }
    }

    pub fn as_group_filter(mut self) -> Self {
        self.is_group_filter = true;
        self
    }

    pub fn add_nested_filter(mut self, filter: LayerFilter) -> Self {
        self.nested_filters.push(filter);
        self
    }

    pub fn matches(&self, layer: &Layer) -> bool {
        if self.is_group_filter {
            self.nested_filters.iter()
                .any(|f| f.matches(layer))
        } else {
            self.criteria.matches(layer)
        }
    }
}

#[derive(Debug, Clone)]
pub struct FilterCriteria {
    pub name_pattern: Option<NamePattern>,
    pub color_filter: Option<ColorFilter>,
    pub visibility_filter: Option<VisibilityFilter>,
    pub lock_filter: Option<LockFilter>,
    pub plot_filter: Option<PlotFilter>,
    pub line_type_filter: Option<LineTypeFilter>,
    pub description_pattern: Option<DescriptionPattern>,
    pub custom_property_filters: Vec<CustomPropertyFilter>,
    pub combination_operator: CombinationOperator,
}

impl FilterCriteria {
    pub fn new() -> Self {
        Self {
            name_pattern: None,
            color_filter: None,
            visibility_filter: None,
            lock_filter: None,
            plot_filter: None,
            line_type_filter: None,
            description_pattern: None,
            custom_property_filters: Vec::new(),
            combination_operator: CombinationOperator::And,
        }
    }

    pub fn with_name_pattern(mut self, pattern: NamePattern) -> Self {
        self.name_pattern = Some(pattern);
        self
    }

    pub fn with_color(mut self, color: Color, match_type: ColorMatchType) -> Self {
        self.color_filter = Some(ColorFilter { color, match_type });
        self
    }

    pub fn with_visibility(mut self, visible: bool) -> Self {
        self.visibility_filter = Some(VisibilityFilter { visible });
        self
    }

    pub fn with_locked(mut self, locked: bool) -> Self {
        self.lock_filter = Some(LockFilter { locked });
        self
    }

    pub fn with_printable(mut self, printable: bool) -> Self {
        self.plot_filter = Some(PlotFilter { printable });
        self
    }

    pub fn with_line_type(mut self, line_type: String) -> Self {
        self.line_type_filter = Some(LineTypeFilter { line_type });
        self
    }

    pub fn with_description_pattern(mut self, pattern: DescriptionPattern) -> Self {
        self.description_pattern = Some(pattern);
        self
    }

    pub fn with_custom_property(
        mut self,
        name: String,
        value: LayerProperty,
        match_type: PropertyMatchType,
    ) -> Self {
        self.custom_property_filters.push(CustomPropertyFilter {
            name,
            value,
            match_type,
        });
        self
    }

    pub fn with_or_operator(mut self) -> Self {
        self.combination_operator = CombinationOperator::Or;
        self
    }

    pub fn matches(&self, layer: &Layer) -> bool {
        match self.combination_operator {
            CombinationOperator::And => {
                let mut all_match = true;
                
                if let Some(ref pattern) = self.name_pattern {
                    all_match &= pattern.matches(&layer.name);
                }
                if let Some(ref filter) = self.color_filter {
                    all_match &= filter.matches(&layer.color);
                }
                if let Some(ref filter) = self.visibility_filter {
                    all_match &= filter.matches(layer.is_visible);
                }
                if let Some(ref filter) = self.lock_filter {
                    all_match &= filter.matches(layer.is_locked);
                }
                if let Some(ref filter) = self.plot_filter {
                    all_match &= filter.matches(layer.is_printable);
                }
                if let Some(ref filter) = self.line_type_filter {
                    all_match &= filter.matches(layer.line_type.as_ref());
                }
                if let Some(ref pattern) = self.description_pattern {
                    all_match &= pattern.matches(layer.description.as_ref());
                }
                
                all_match
            }
            CombinationOperator::Or => {
                let mut any_match = false;
                
                if let Some(ref pattern) = self.name_pattern {
                    any_match |= pattern.matches(&layer.name);
                }
                if let Some(ref filter) = self.color_filter {
                    any_match |= filter.matches(&layer.color);
                }
                if let Some(ref filter) = self.visibility_filter {
                    any_match |= filter.matches(layer.is_visible);
                }
                if let Some(ref filter) = self.lock_filter {
                    any_match |= filter.matches(layer.is_locked);
                }
                if let Some(ref filter) = self.plot_filter {
                    any_match |= filter.matches(layer.is_printable);
                }
                if let Some(ref filter) = self.line_type_filter {
                    any_match |= filter.matches(layer.line_type.as_ref());
                }
                if let Some(ref pattern) = self.description_pattern {
                    any_match |= pattern.matches(layer.description.as_ref());
                }
                
                any_match
            }
        }
    }
}

impl Default for FilterCriteria {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct NamePattern {
    pub pattern: String,
    pub use_wildcards: bool,
    pub case_sensitive: bool,
}

impl NamePattern {
    pub fn new(pattern: String) -> Self {
        Self {
            pattern,
            use_wildcards: true,
            case_sensitive: false,
        }
    }

    pub fn matches(&self, name: &str) -> bool {
        if self.use_wildcards {
            wildcard_match(&self.pattern, name, self.case_sensitive)
        } else {
            if self.case_sensitive {
                name.contains(&self.pattern)
            } else {
                name.to_lowercase().contains(&self.pattern.to_lowercase())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColorFilter {
    pub color: Color,
    pub match_type: ColorMatchType,
}

impl ColorFilter {
    pub fn matches(&self, layer_color: &Color) -> bool {
        match self.match_type {
            ColorMatchType::Exact => {
                (self.color.r - layer_color.r).abs() < 0.01 &&
                (self.color.g - layer_color.g).abs() < 0.01 &&
                (self.color.b - layer_color.b).abs() < 0.01
            }
            ColorMatchType::Similar => {
                (self.color.r - layer_color.r).abs() < 0.1 &&
                (self.color.g - layer_color.g).abs() < 0.1 &&
                (self.color.b - layer_color.b).abs() < 0.1
            }
            ColorMatchType::HueMatch => {
                let self_hsv = rgb_to_hsv(self.color.r, self.color.g, self.color.b);
                let layer_hsv = rgb_to_hsv(layer_color.r, layer_color.g, layer_color.b);
                (self_hsv.0 - layer_hsv.0).abs() < 10.0
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorMatchType {
    Exact,
    Similar,
    HueMatch,
}

#[derive(Debug, Clone)]
pub struct VisibilityFilter {
    pub visible: bool,
}

impl VisibilityFilter {
    pub fn matches(&self, is_visible: bool) -> bool {
        self.visible == is_visible
    }
}

#[derive(Debug, Clone)]
pub struct LockFilter {
    pub locked: bool,
}

impl LockFilter {
    pub fn matches(&self, is_locked: bool) -> bool {
        self.locked == is_locked
    }
}

#[derive(Debug, Clone)]
pub struct PlotFilter {
    pub printable: bool,
}

impl PlotFilter {
    pub fn matches(&self, is_printable: bool) -> bool {
        self.printable == is_printable
    }
}

#[derive(Debug, Clone)]
pub struct LineTypeFilter {
    pub line_type: String,
}

impl LineTypeFilter {
    pub fn matches(&self, layer_line_type: Option<&String>) -> bool {
        match layer_line_type {
            Some(lt) => lt.to_lowercase() == self.line_type.to_lowercase(),
            None => self.line_type.is_empty(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DescriptionPattern {
    pub pattern: String,
    pub case_sensitive: bool,
}

impl DescriptionPattern {
    pub fn matches(&self, description: Option<&String>) -> bool {
        match description {
            Some(desc) => {
                if self.case_sensitive {
                    desc.contains(&self.pattern)
                } else {
                    desc.to_lowercase().contains(&self.pattern.to_lowercase())
                }
            }
            None => self.pattern.is_empty(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CustomPropertyFilter {
    pub name: String,
    pub value: LayerProperty,
    pub match_type: PropertyMatchType,
}

impl CustomPropertyFilter {
    pub fn matches(&self, properties: &HashMap<String, LayerProperty>) -> bool {
        if let Some(prop) = properties.get(&self.name) {
            match (prop, &self.value) {
                (LayerProperty::String(s1), LayerProperty::String(s2)) => {
                    s1 == s2
                }
                (LayerProperty::Number(n1), LayerProperty::Number(n2)) => {
                    (n1 - n2).abs() < 0.001
                }
                (LayerProperty::Boolean(b1), LayerProperty::Boolean(b2)) => {
                    b1 == b2
                }
                (LayerProperty::Color(c1), LayerProperty::Color(c2)) => {
                    (c1.r - c2.r).abs() < 0.01 &&
                    (c1.g - c2.g).abs() < 0.01 &&
                    (c1.b - c2.b).abs() < 0.01
                }
                (LayerProperty::ColorIndex(i1), LayerProperty::ColorIndex(i2)) => {
                    i1 == i2
                }
                _ => false,
            }
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PropertyMatchType {
    Exact,
    Contains,
    StartsWith,
    EndsWith,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CombinationOperator {
    And,
    Or,
}

#[derive(Debug, Clone)]
pub struct LayerStateManager {
    states: HashMap<String, HashMap<String, Layer>>,
}

impl LayerStateManager {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    pub fn save_state(&mut self, state_name: String, layers: &HashMap<String, Layer>) {
        let mut state = HashMap::new();
        for (name, layer) in layers {
            state.insert(name.clone(), layer.clone());
        }
        self.states.insert(state_name, state);
    }

    pub fn get_state(&self, state_name: &str) -> Option<&HashMap<String, Layer>> {
        self.states.get(state_name)
    }

    pub fn delete_state(&mut self, state_name: &str) -> bool {
        self.states.remove(state_name).is_some()
    }

    pub fn available_states(&self) -> Vec<&String> {
        self.states.keys().collect()
    }

    pub fn rename_state(&mut self, old_name: &str, new_name: String) -> bool {
        if let Some(state) = self.states.remove(old_name) {
            self.states.insert(new_name, state);
            true
        } else {
            false
        }
    }
}

fn wildcard_match(pattern: &str, text: &str, case_sensitive: bool) -> bool {
    let text = if case_sensitive { text.to_string() } else { text.to_lowercase() };
    let pattern = if case_sensitive { pattern.to_string() } else { pattern.to_lowercase() };
    
    let mut pattern_chars = pattern.chars().collect::<Vec<_>>();
    let mut text_chars = text.chars().collect::<Vec<_>>();
    
    let mut dp = vec![vec![false; text_chars.len() + 1]; pattern_chars.len() + 1];
    dp[0][0] = true;
    
    for i in 1..=pattern_chars.len() {
        if pattern_chars[i-1] == '*' {
            dp[i][0] = dp[i-1][0];
        }
    }
    
    for i in 1..=pattern_chars.len() {
        for j in 1..=text_chars.len() {
            match pattern_chars[i-1] {
                '*' => {
                    dp[i][j] = dp[i-1][j] || dp[i][j-1];
                }
                '?' => {
                    dp[i][j] = dp[i-1][j-1];
                }
                _ => {
                    dp[i][j] = dp[i-1][j-1] && pattern_chars[i-1] == text_chars[j-1];
                }
            }
        }
    }
    
    dp[pattern_chars.len()][text_chars.len()]
}

fn rgb_to_hsv(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    
    let h = if delta < 0.001 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };
    
    let s = if max < 0.001 { 0.0 } else { delta / max };
    let v = max;
    
    (h, s * 100.0, v * 100.0)
}
