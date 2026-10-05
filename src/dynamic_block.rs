use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParameterType {
    Point,
    Linear,
    Aligned,
    Angle,
    Distance,
    Radius,
    Diameter,
    ArcLength,
    Area,
    Visibility,
    Lookup,
    Xform,
    BlockProperty,
    UserParameter,
}

impl Default for ParameterType {
    fn default() -> Self {
        ParameterType::Point
    }
}

impl ParameterType {
    pub fn name(&self) -> &str {
        match self {
            ParameterType::Point => "Point",
            ParameterType::Linear => "Linear",
            ParameterType::Aligned => "Aligned",
            ParameterType::Angle => "Angle",
            ParameterType::Distance => "Distance",
            ParameterType::Radius => "Radius",
            ParameterType::Diameter => "Diameter",
            ParameterType::ArcLength => "Arc Length",
            ParameterType::Area => "Area",
            ParameterType::Visibility => "Visibility",
            ParameterType::Lookup => "Lookup",
            ParameterType::Xform => "Xform",
            ParameterType::BlockProperty => "Block Property",
            ParameterType::UserParameter => "User Parameter",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            ParameterType::Point => "📍",
            ParameterType::Linear => "↔",
            ParameterType::Aligned => "↗",
            ParameterType::Angle => "∠",
            ParameterType::Distance => "📏",
            ParameterType::Radius => "⭕",
            ParameterType::Diameter => "⊕",
            ParameterType::ArcLength => "⌒",
            ParameterType::Area => "⬜",
            ParameterType::Visibility => "👁",
            ParameterType::Lookup => "📋",
            ParameterType::Xform => "🔄",
            ParameterType::BlockProperty => "📦",
            ParameterType::UserParameter => "🔧",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockParameter {
    pub name: String,
    pub parameter_type: ParameterType,
    pub label: String,
    pub description: String,
    pub value: f64,
    pub default_value: f64,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub step: Option<f64>,
    pub expressions: Vec<Expression>,
    pub is_chain_action: bool,
    pub is_preset: bool,
    pub is_lookup: bool,
    pub lookup_table: Vec<LookupRow>,
    pub position: crate::geometry::Point,
    pub angle: f64,
    pub chain_actions: Vec<String>,
    pub property_id: Option<String>,
}

impl Default for BlockParameter {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockParameter {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            parameter_type: ParameterType::Point,
            label: String::new(),
            description: String::new(),
            value: 0.0,
            default_value: 0.0,
            minimum: None,
            maximum: None,
            step: None,
            expressions: Vec::new(),
            is_chain_action: false,
            is_preset: false,
            is_lookup: false,
            lookup_table: Vec::new(),
            position: crate::geometry::Point::origin(),
            angle: 0.0,
            chain_actions: Vec::new(),
            property_id: None,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn with_type(mut self, parameter_type: ParameterType) -> Self {
        self.parameter_type = parameter_type;
        self
    }

    pub fn with_value(mut self, value: f64) -> Self {
        self.value = value;
        self
    }

    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.minimum = Some(min);
        self.maximum = Some(max);
        self
    }

    pub fn set_value(&mut self, value: f64) -> bool {
        if let Some(min) = self.minimum {
            if value < min {
                return false;
            }
        }
        if let Some(max) = self.maximum {
            if value > max {
                return false;
            }
        }
        self.value = value;
        true
    }

    pub fn add_expression(&mut self, expression: Expression) {
        self.expressions.push(expression);
    }

    pub fn add_chain_action(&mut self, action_id: &str) {
        self.chain_actions.push(action_id.to_string());
    }

    pub fn add_lookup_row(&mut self, row: LookupRow) {
        self.lookup_table.push(row);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LookupRow {
    pub input_values: Vec<f64>,
    pub output_value: f64,
}

impl Default for LookupRow {
    fn default() -> Self {
        Self::new()
    }
}

impl LookupRow {
    pub fn new() -> Self {
        Self {
            input_values: Vec::new(),
            output_value: 0.0,
        }
    }

    pub fn with_values(mut self, inputs: &[f64], output: f64) -> Self {
        self.input_values = inputs.to_vec();
        self.output_value = output;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expression {
    pub name: String,
    pub expression: String,
    pub value: f64,
    pub is_valid: bool,
}

impl Default for Expression {
    fn default() -> Self {
        Self::new()
    }
}

impl Expression {
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
        let expr = self.expression.replace(" ", "");
        self.value = match expr.as_str() {
            "PI" => std::f64::consts::PI,
            "2*PI" => 2.0 * std::f64::consts::PI,
            "PI/2" => std::f64::consts::PI / 2.0,
            "PI/4" => std::f64::consts::PI / 4.0,
            _ => {
                let cleaned = expr.replace("(", "").replace(")", "");
                if let Ok(num) = cleaned.parse::<f64>() {
                    num
                } else {
                    0.0
                }
            }
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    Move,
    Scale,
    Stretch,
    Rotate,
    Mirror,
    Array,
    Polar,
    Lookup,
}

impl Default for ActionType {
    fn default() -> Self {
        ActionType::Move
    }
}

impl ActionType {
    pub fn name(&self) -> &str {
        match self {
            ActionType::Move => "Move",
            ActionType::Scale => "Scale",
            ActionType::Stretch => "Stretch",
            ActionType::Rotate => "Rotate",
            ActionType::Mirror => "Mirror",
            ActionType::Array => "Array",
            ActionType::Polar => "Polar",
            ActionType::Lookup => "Lookup",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            ActionType::Move => "➡",
            ActionType::Scale => "↔",
            ActionType::Stretch => "↗",
            ActionType::Rotate => "🔄",
            ActionType::Mirror => "🪞",
            ActionType::Array => "▦",
            ActionType::Polar => "◎",
            ActionType::Lookup => "📋",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockAction {
    pub id: String,
    pub action_type: ActionType,
    pub name: String,
    pub description: String,
    pub parameter_name: String,
    pub entities: Vec<String>,
    pub connection_points: Vec<ConnectionPoint>,
    pub displacement: crate::geometry::Point,
    pub angle: f64,
    pub scale: f64,
    pub is_dependent: bool,
    pub is_enabled: bool,
    pub flip_action: bool,
    pub base_point: Option<ConnectionPoint>,
    pub action_direction: f64,
    pub action_value: f64,
    pub dependent_actions: Vec<String>,
}

impl Default for BlockAction {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockAction {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            action_type: ActionType::Move,
            name: String::new(),
            description: String::new(),
            parameter_name: String::new(),
            entities: Vec::new(),
            connection_points: Vec::new(),
            displacement: crate::geometry::Point::origin(),
            angle: 0.0,
            scale: 1.0,
            is_dependent: false,
            is_enabled: true,
            flip_action: false,
            base_point: None,
            action_direction: 0.0,
            action_value: 0.0,
            dependent_actions: Vec::new(),
        }
    }

    pub fn with_type(mut self, action_type: ActionType) -> Self {
        self.action_type = action_type;
        self
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn with_parameter(mut self, param_name: &str) -> Self {
        self.parameter_name = param_name.to_string();
        self
    }

    pub fn add_entity(&mut self, entity_id: &str) {
        self.entities.push(entity_id.to_string());
    }

    pub fn add_connection_point(&mut self, point: ConnectionPoint) {
        self.connection_points.push(point);
    }

    pub fn set_displacement(&mut self, displacement: crate::geometry::Point) {
        self.displacement = displacement;
    }

    pub fn set_rotation(&mut self, angle: f64) {
        self.angle = angle;
    }

    pub fn set_scale(&mut self, scale: f64) {
        self.scale = scale;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.is_enabled = enabled;
    }

    pub fn add_dependent_action(&mut self, action_id: &str) {
        self.dependent_actions.push(action_id.to_string());
    }

    pub fn remove_dependent_action(&mut self, action_id: &str) {
        self.dependent_actions.retain(|id| id != action_id);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionPoint {
    pub position: crate::geometry::Point,
    pub entity_id: String,
    pub grip_index: u32,
    pub is_base: bool,
}

impl Default for ConnectionPoint {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionPoint {
    pub fn new() -> Self {
        Self {
            position: crate::geometry::Point::origin(),
            entity_id: String::new(),
            grip_index: 0,
            is_base: false,
        }
    }

    pub fn at_position(position: crate::geometry::Point) -> Self {
        Self {
            position,
            entity_id: String::new(),
            grip_index: 0,
            is_base: false,
        }
    }

    pub fn on_entity(entity_id: &str, position: crate::geometry::Point) -> Self {
        Self {
            position,
            entity_id: entity_id.to_string(),
            grip_index: 0,
            is_base: false,
        }
    }

    pub fn as_base(mut self) -> Self {
        self.is_base = true;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisibilityState {
    Visible,
    Invisible,
    Hidden,
}

impl Default for VisibilityState {
    fn default() -> Self {
        VisibilityState::Visible
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisibilitySetting {
    pub name: String,
    pub description: String,
    pub visible_entities: Vec<String>,
    pub hidden_entities: Vec<String>,
    pub icon: String,
}

impl Default for VisibilitySetting {
    fn default() -> Self {
        Self::new()
    }
}

impl VisibilitySetting {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            visible_entities: Vec::new(),
            hidden_entities: Vec::new(),
            icon: String::new(),
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn set_entity_visible(&mut self, entity_id: &str, visible: bool) {
        if visible {
            if !self.visible_entities.contains(&entity_id.to_string()) {
                self.visible_entities.push(entity_id.to_string());
            }
            self.hidden_entities.retain(|e| e != entity_id);
        } else {
            if !self.hidden_entities.contains(&entity_id.to_string()) {
                self.hidden_entities.push(entity_id.to_string());
            }
            self.visible_entities.retain(|e| e != entity_id);
        }
    }

    pub fn is_entity_visible(&self, entity_id: &str) -> bool {
        self.visible_entities.contains(&entity_id.to_string())
            && !self.hidden_entities.contains(&entity_id.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterSet {
    pub name: String,
    pub description: String,
    pub parameters: Vec<String>,
    pub actions: Vec<String>,
    pub icon: String,
}

impl Default for ParameterSet {
    fn default() -> Self {
        Self::new()
    }
}

impl ParameterSet {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            parameters: Vec::new(),
            actions: Vec::new(),
            icon: String::new(),
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn add_parameter(&mut self, param_name: &str) {
        self.parameters.push(param_name.to_string());
    }

    pub fn add_action(&mut self, action_id: &str) {
        self.actions.push(action_id.to_string());
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GripPoint {
    pub parameter_name: String,
    pub parameter_value: f64,
    pub position: crate::geometry::Point,
    pub grip_type: GripType,
    pub is_visible: bool,
    pub is_enabled: bool,
    pub is_hovered: bool,
    pub tooltip: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GripType {
    Linear,
    Angular,
    Radial,
    Mover,
    Aligned,
}

impl Default for GripType {
    fn default() -> Self {
        GripType::Mover
    }
}

impl GripPoint {
    pub fn new() -> Self {
        Self {
            parameter_name: String::new(),
            parameter_value: 0.0,
            position: crate::geometry::Point::origin(),
            grip_type: GripType::Mover,
            is_visible: true,
            is_enabled: true,
            is_hovered: false,
            tooltip: String::new(),
        }
    }

    pub fn for_parameter(param_name: &str, value: f64, position: crate::geometry::Point) -> Self {
        Self {
            parameter_name: param_name.to_string(),
            parameter_value: value,
            position,
            grip_type: GripType::Mover,
            is_visible: true,
            is_enabled: true,
            is_hovered: false,
            tooltip: format!("{}: {}", param_name, value),
        }
    }

    pub fn set_position(&mut self, position: crate::geometry::Point) {
        self.position = position;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.is_enabled = enabled;
    }

    pub fn set_hover(&mut self, hovered: bool) {
        self.is_hovered = hovered;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DynamicBlock {
    pub name: String,
    pub description: String,
    pub block_units: BlockUnit,
    pub entities: Vec<DynamicBlockEntity>,
    pub parameters: Vec<BlockParameter>,
    pub actions: Vec<BlockAction>,
    pub visibility_states: Vec<VisibilitySetting>,
    pub parameter_sets: Vec<ParameterSet>,
    pub grips: Vec<GripPoint>,
    pub is_dynamic: bool,
    pub is_block_table_record: bool,
    pub scale: f64,
    pub rotation: f64,
    pub allows_exploding: bool,
    pub block_table_record_id: String,
}

impl Default for DynamicBlock {
    fn default() -> Self {
        Self::new()
    }
}

impl DynamicBlock {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            block_units: BlockUnit::Unitless,
            entities: Vec::new(),
            parameters: Vec::new(),
            actions: Vec::new(),
            visibility_states: Vec::new(),
            parameter_sets: Vec::new(),
            grips: Vec::new(),
            is_dynamic: true,
            is_block_table_record: true,
            scale: 1.0,
            rotation: 0.0,
            allows_exploding: true,
            block_table_record_id: String::new(),
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn add_entity(&mut self, entity: DynamicBlockEntity) {
        self.entities.push(entity);
    }

    pub fn add_parameter(&mut self, parameter: BlockParameter) {
        self.parameters.push(parameter);
    }

    pub fn add_action(&mut self, action: BlockAction) {
        self.actions.push(action);
    }

    pub fn add_visibility_state(&mut self, state: VisibilitySetting) {
        self.visibility_states.push(state);
    }

    pub fn add_parameter_set(&mut self, set: ParameterSet) {
        self.parameter_sets.push(set);
    }

    pub fn get_parameter(&self, name: &str) -> Option<&BlockParameter> {
        self.parameters.iter().find(|p| p.name == name)
    }

    pub fn get_action(&self, id: &str) -> Option<&BlockAction> {
        self.actions.iter().find(|a| a.id == id)
    }

    pub fn set_parameter_value(&mut self, name: &str, value: f64) -> bool {
        if let Some(param) = self.parameters.iter_mut().find(|p| p.name == name) {
            param.set_value(value)
        } else {
            false
        }
    }

    pub fn set_visibility_state(&mut self, state_name: &str) -> bool {
        if let Some(state) = self.visibility_states.iter().find(|s| s.name == state_name) {
            for entity in &mut self.entities {
                entity.is_visible = state.is_entity_visible(&entity.id);
            }
            true
        } else {
            false
        }
    }

    pub fn update_grips(&mut self) {
        self.grips.clear();
        for param in &self.parameters {
            let grip = GripPoint::for_parameter(
                &param.name,
                param.value,
                param.position,
            );
            self.grips.push(grip);
        }
    }

    pub fn is_locked(&self) -> bool {
        false
    }

    pub fn set_allows_exploding(&mut self, allows: bool) {
        self.allows_exploding = allows;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DynamicBlockEntity {
    pub id: String,
    pub entity_type: String,
    pub geometry: String,
    pub position: crate::geometry::Point,
    pub rotation: f64,
    pub scale: (f64, f64),
    pub layer: String,
    pub color: (u8, u8, u8),
    pub linetype: String,
    pub lineweight: i32,
    pub is_visible: bool,
    pub is_locked: bool,
    pub transformations: Vec<Transformation>,
}

impl Default for DynamicBlockEntity {
    fn default() -> Self {
        Self::new()
    }
}

impl DynamicBlockEntity {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            entity_type: String::new(),
            geometry: String::new(),
            position: crate::geometry::Point::origin(),
            rotation: 0.0,
            scale: (1.0, 1.0),
            layer: String::new(),
            color: (0, 0, 0),
            linetype: String::new(),
            lineweight: 0,
            is_visible: true,
            is_locked: false,
            transformations: Vec::new(),
        }
    }

    pub fn with_type(mut self, entity_type: &str) -> Self {
        self.entity_type = entity_type.to_string();
        self
    }

    pub fn at_position(mut self, position: crate::geometry::Point) -> Self {
        self.position = position;
        self
    }

    pub fn set_visibility(&mut self, visible: bool) {
        self.is_visible = visible;
    }

    pub fn add_transformation(&mut self, transformation: Transformation) {
        self.transformations.push(transformation);
    }

    pub fn clear_transformations(&mut self) {
        self.transformations.clear();
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transformation {
    pub transformation_type: TransformationType,
    pub parameters: HashMap<String, f64>,
    pub order: u32,
}

impl Default for Transformation {
    fn default() -> Self {
        Self::new()
    }
}

impl Transformation {
    pub fn new() -> Self {
        Self {
            transformation_type: TransformationType::Identity,
            parameters: HashMap::new(),
            order: 0,
        }
    }

    pub fn with_type(mut self, transformation_type: TransformationType) -> Self {
        self.transformation_type = transformation_type;
        self
    }

    pub fn with_param(mut self, name: &str, value: f64) -> Self {
        self.parameters.insert(name.to_string(), value);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformationType {
    Identity,
    Translation,
    Rotation,
    Scaling,
    Mirror,
    Shear,
}

impl Default for TransformationType {
    fn default() -> Self {
        TransformationType::Identity
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockUnit {
    Unitless,
    Inches,
    Feet,
    Miles,
    Millimeters,
    Centimeters,
    Meters,
    Kilometers,
    Microinches,
    Mils,
    Yards,
    Angstroms,
    Nanometers,
    Microns,
    Decimeters,
    Decameters,
    Hectometers,
    Gigameters,
    AstronomicalUnits,
    LightYears,
    Parsecs,
}

impl Default for BlockUnit {
    fn default() -> Self {
        BlockUnit::Unitless
    }
}

impl BlockUnit {
    pub fn name(&self) -> &str {
        match self {
            BlockUnit::Unitless => "Unitless",
            BlockUnit::Inches => "Inches",
            BlockUnit::Feet => "Feet",
            BlockUnit::Miles => "Miles",
            BlockUnit::Millimeters => "Millimeters",
            BlockUnit::Centimeters => "Centimeters",
            BlockUnit::Meters => "Meters",
            BlockUnit::Kilometers => "Kilometers",
            BlockUnit::Microinches => "Microinches",
            BlockUnit::Mils => "Mils",
            BlockUnit::Yards => "Yards",
            BlockUnit::Angstroms => "Angstroms",
            BlockUnit::Nanometers => "Nanometers",
            BlockUnit::Microns => "Microns",
            BlockUnit::Decimeters => "Decimeters",
            BlockUnit::Decameters => "Decameters",
            BlockUnit::Hectometers => "Hectometers",
            BlockUnit::Gigameters => "Gigameters",
            BlockUnit::AstronomicalUnits => "Astronomical Units",
            BlockUnit::LightYears => "Light Years",
            BlockUnit::Parsecs => "Parsecs",
        }
    }

    pub fn conversion_factor(&self) -> f64 {
        match self {
            BlockUnit::Unitless => 1.0,
            BlockUnit::Inches => 25.4,
            BlockUnit::Feet => 304.8,
            BlockUnit::Miles => 1609344.0,
            BlockUnit::Millimeters => 1.0,
            BlockUnit::Centimeters => 10.0,
            BlockUnit::Meters => 1000.0,
            BlockUnit::Kilometers => 1000000.0,
            BlockUnit::Microinches => 0.0000254,
            BlockUnit::Mils => 0.0254,
            BlockUnit::Yards => 914.4,
            BlockUnit::Angstroms => 0.0000001,
            BlockUnit::Nanometers => 0.000001,
            BlockUnit::Microns => 0.001,
            BlockUnit::Decimeters => 100.0,
            BlockUnit::Decameters => 10000.0,
            BlockUnit::Hectometers => 100000.0,
            BlockUnit::Gigameters => 1000000000000.0,
            BlockUnit::AstronomicalUnits => 149597870700000.0,
            BlockUnit::LightYears => 9460730472580800000.0,
            BlockUnit::Parsecs => 30856775814913670000.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DynamicBlockManager {
    blocks: HashMap<String, DynamicBlock>,
    active_block: Option<String>,
    is_editing: bool,
    show_grips: bool,
    show_palette: bool,
}

impl Default for DynamicBlockManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DynamicBlockManager {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            active_block: None,
            is_editing: false,
            show_grips: true,
            show_palette: true,
        }
    }

    pub fn create_block(&mut self, name: &str) -> &mut DynamicBlock {
        let block = DynamicBlock::new().with_name(name);
        self.blocks.insert(name.to_string(), block);
        self.active_block = Some(name.to_string());
        self.blocks.get_mut(name).unwrap()
    }

    pub fn add_block(&mut self, block: DynamicBlock) {
        self.blocks.insert(block.name.clone(), block);
    }

    pub fn get_block(&self, name: &str) -> Option<&DynamicBlock> {
        self.blocks.get(name)
    }

    pub fn get_block_mut(&mut self, name: &str) -> Option<&mut DynamicBlock> {
        self.blocks.get_mut(name)
    }

    pub fn remove_block(&mut self, name: &str) -> bool {
        self.blocks.remove(name).is_some()
    }

    pub fn rename_block(&mut self, old_name: &str, new_name: &str) -> bool {
        if let Some(block) = self.blocks.remove(old_name) {
            let mut new_block = block;
            new_block.name = new_name.to_string();
            self.blocks.insert(new_name.to_string(), new_block);
            true
        } else {
            false
        }
    }

    pub fn add_parameter(&mut self, block_name: &str, parameter: BlockParameter) -> bool {
        if let Some(block) = self.blocks.get_mut(block_name) {
            block.parameters.push(parameter);
            true
        } else {
            false
        }
    }

    pub fn add_action(&mut self, block_name: &str, action: BlockAction) -> bool {
        if let Some(block) = self.blocks.get_mut(block_name) {
            block.actions.push(action);
            true
        } else {
            false
        }
    }

    pub fn set_parameter_value(&mut self, block_name: &str, param_name: &str, value: f64) -> bool {
        if let Some(block) = self.blocks.get_mut(block_name) {
            block.set_parameter_value(param_name, value)
        } else {
            false
        }
    }

    pub fn set_visibility_state(&mut self, block_name: &str, state_name: &str) -> bool {
        if let Some(block) = self.blocks.get_mut(block_name) {
            block.set_visibility_state(state_name)
        } else {
            false
        }
    }

    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    pub fn block_names(&self) -> Vec<&str> {
        self.blocks.keys().map(|s| s.as_str()).collect()
    }

    pub fn set_active_block(&mut self, name: Option<&str>) {
        self.active_block = name.map(|s| s.to_string());
    }

    pub fn active_block(&self) -> Option<&DynamicBlock> {
        self.active_block.as_ref().and_then(|name| self.blocks.get(name))
    }

    pub fn active_block_mut(&mut self) -> Option<&mut DynamicBlock> {
        self.active_block.as_ref().and_then(|name| self.blocks.get_mut(name))
    }

    pub fn set_editing(&mut self, editing: bool) {
        self.is_editing = editing;
    }

    pub fn is_editing(&self) -> bool {
        self.is_editing
    }

    pub fn set_show_grips(&mut self, show: bool) {
        self.show_grips = show;
    }

    pub fn show_grips(&self) -> bool {
        self.show_grips
    }

    pub fn set_show_palette(&mut self, show: bool) {
        self.show_palette = show;
    }

    pub fn show_palette(&self) -> bool {
        self.show_palette
    }

    pub fn clear(&mut self) {
        self.blocks.clear();
        self.active_block = None;
        self.is_editing = false;
    }

    pub fn duplicate_block(&mut self, source_name: &str, new_name: &str) -> bool {
        if let Some(source_block) = self.blocks.get(source_name) {
            let mut new_block = source_block.clone();
            new_block.name = new_name.to_string();
            self.blocks.insert(new_name.to_string(), new_block);
            true
        } else {
            false
        }
    }

    pub fn convert_to_static(&mut self, block_name: &str) -> bool {
        if let Some(block) = self.blocks.get_mut(block_name) {
            block.is_dynamic = false;
            block.parameters.clear();
            block.actions.clear();
            block.parameter_sets.clear();
            block.grips.clear();
            true
        } else {
            false
        }
    }

    pub fn get_statistics(&self, block_name: &str) -> Option<BlockStatistics> {
        self.blocks.get(block_name).map(|block| BlockStatistics {
            parameter_count: block.parameters.len(),
            action_count: block.actions.len(),
            visibility_state_count: block.visibility_states.len(),
            entity_count: block.entities.len(),
            grip_count: block.grips.len(),
            is_dynamic: block.is_dynamic,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockStatistics {
    pub parameter_count: usize,
    pub action_count: usize,
    pub visibility_state_count: usize,
    pub entity_count: usize,
    pub grip_count: usize,
    pub is_dynamic: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn test_parameter_creation() {
        let param = BlockParameter::new()
            .with_name("Length")
            .with_type(ParameterType::Distance)
            .with_value(100.0)
            .with_range(0.0, 1000.0);

        assert_eq!(param.name, "Length");
        assert_eq!(param.parameter_type, ParameterType::Distance);
        assert!((param.value - 100.0).abs() < 1e-10);
    }

    #[test]
    fn test_parameter_range_validation() {
        let mut param = BlockParameter::new()
            .with_name("Length")
            .with_range(0.0, 100.0);

        assert!(param.set_value(50.0));
        assert!(!param.set_value(150.0));
        assert!(!param.set_value(-10.0));
    }

    #[test]
    fn test_action_creation() {
        let action = BlockAction::new()
            .with_type(ActionType::Move)
            .with_name("Move Action")
            .with_parameter("Length");

        assert_eq!(action.action_type, ActionType::Move);
        assert_eq!(action.name, "Move Action");
    }

    #[test]
    fn test_dynamic_block_creation() {
        let mut block = DynamicBlock::new().with_name("Door");

        let param = BlockParameter::new()
            .with_name("Width")
            .with_type(ParameterType::Linear)
            .with_value(900.0);

        block.add_parameter(param);

        assert_eq!(block.parameters.len(), 1);
        assert!(block.get_parameter("Width").is_some());
    }

    #[test]
    fn test_dynamic_block_parameter_value() {
        let mut block = DynamicBlock::new().with_name("Door");
        block.add_parameter(
            BlockParameter::new()
                .with_name("Width")
                .with_type(ParameterType::Linear)
                .with_range(600.0, 1200.0)
        );

        assert!(block.set_parameter_value("Width", 1000.0));
        assert!(!block.set_parameter_value("Width", 1500.0));
    }

    #[test]
    fn test_visibility_state() {
        let mut block = DynamicBlock::new().with_name("Door");

        let mut state = VisibilitySetting::new().with_name("Open");
        state.set_entity_visible("door_panel", false);
        state.set_entity_visible("hinge", true);

        block.add_visibility_state(state);

        assert_eq!(block.visibility_states.len(), 1);
    }

    #[test]
    fn test_grip_points() {
        let grip = GripPoint::for_parameter(
            "Width",
            1000.0,
            Point::new(100.0, 50.0, 0.0),
        );

        assert_eq!(grip.parameter_name, "Width");
        assert!((grip.parameter_value - 1000.0).abs() < 1e-10);
    }

    #[test]
    fn test_connection_point() {
        let point = ConnectionPoint::at_position(Point::new(100.0, 50.0, 0.0));
        assert!((point.position.x - 100.0).abs() < 1e-10);

        let base_point = ConnectionPoint::on_entity("line1", Point::new(0.0, 0.0, 0.0)).as_base();
        assert!(base_point.is_base);
    }

    #[test]
    fn test_expression() {
        let mut expr = Expression::new().with_expression("PI/2");
        assert!((expr.value - std::f64::consts::PI / 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_lookup_row() {
        let row = LookupRow::new()
            .with_values(&[0.0, 50.0, 100.0], 25.0);

        assert_eq!(row.input_values.len(), 3);
        assert!((row.output_value - 25.0).abs() < 1e-10);
    }

    #[test]
    fn test_parameter_set() {
        let mut set = ParameterSet::new().with_name("Linear Set");
        set.add_parameter("Width");
        set.add_parameter("Height");
        set.add_action("move_action");

        assert_eq!(set.parameters.len(), 2);
        assert_eq!(set.actions.len(), 1);
    }

    #[test]
    fn test_dynamic_block_manager() {
        let mut manager = DynamicBlockManager::new();
        manager.create_block("Door");

        assert_eq!(manager.block_count(), 1);
        assert!(manager.get_block("Door").is_some());
    }

    #[test]
    fn test_dynamic_block_duplicate() {
        let mut manager = DynamicBlockManager::new();
        manager.create_block("Door");

        assert!(manager.duplicate_block("Door", "Door Copy"));
        assert_eq!(manager.block_count(), 2);
        assert!(manager.get_block("Door Copy").is_some());
    }

    #[test]
    fn test_dynamic_block_convert_to_static() {
        let mut manager = DynamicBlockManager::new();
        manager.create_block("Door");

        assert!(manager.convert_to_static("Door"));
        let block = manager.get_block("Door").unwrap();
        assert!(!block.is_dynamic);
        assert!(block.parameters.is_empty());
    }

    #[test]
    fn test_block_unit_conversion() {
        assert!((BlockUnit::Millimeters.conversion_factor() - 1.0).abs() < 1e-10);
        assert!((BlockUnit::Centimeters.conversion_factor() - 10.0).abs() < 1e-10);
        assert!((BlockUnit::Inches.conversion_factor() - 25.4).abs() < 1e-10);
    }

    #[test]
    fn test_parameter_type_names() {
        assert_eq!(ParameterType::Point.name(), "Point");
        assert_eq!(ParameterType::Linear.name(), "Linear");
        assert_eq!(ParameterType::Angle.name(), "Angle");
        assert_eq!(ParameterType::Visibility.name(), "Visibility");
    }

    #[test]
    fn test_action_type_names() {
        assert_eq!(ActionType::Move.name(), "Move");
        assert_eq!(ActionType::Rotate.name(), "Rotate");
        assert_eq!(ActionType::Scale.name(), "Scale");
        assert_eq!(ActionType::Stretch.name(), "Stretch");
    }

    #[test]
    fn test_dynamic_block_entity() {
        let entity = DynamicBlockEntity::new()
            .with_type("LINE")
            .at_position(Point::new(100.0, 100.0, 0.0));

        assert_eq!(entity.entity_type, "LINE");
        assert!((entity.position.x - 100.0).abs() < 1e-10);
    }

    #[test]
    fn test_transformation() {
        let transform = Transformation::new()
            .with_type(TransformationType::Rotation)
            .with_param("angle", 45.0);

        assert_eq!(transform.transformation_type, TransformationType::Rotation);
        assert!((transform.parameters["angle"] - 45.0).abs() < 1e-10);
    }

    #[test]
    fn test_block_statistics() {
        let mut manager = DynamicBlockManager::new();
        manager.create_block("Door");
        let block = manager.get_block_mut("Door").unwrap();

        block.add_parameter(BlockParameter::new().with_name("Width"));
        block.add_parameter(BlockParameter::new().with_name("Height"));
        block.add_action(BlockAction::new().with_name("Move"));

        let stats = manager.get_statistics("Door").unwrap();
        assert_eq!(stats.parameter_count, 2);
        assert_eq!(stats.action_count, 1);
    }

    #[test]
    fn test_visibility_setting() {
        let mut setting = VisibilitySetting::new().with_name("State1");
        setting.set_entity_visible("entity1", false);
        setting.set_entity_visible("entity2", true);

        assert!(!setting.is_entity_visible("entity1"));
        assert!(setting.is_entity_visible("entity2"));
    }

    #[test]
    fn test_grip_hover() {
        let mut grip = GripPoint::new();
        grip.set_hover(true);
        assert!(grip.is_hovered);
    }
}
