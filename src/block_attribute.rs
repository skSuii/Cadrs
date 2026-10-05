//! 块属性（Attribute）与块表（Block Table）：属性定义、属性实例、块记录与块参照。
//!
//! 核心概念：
//! - `AttributeDefinition`：块定义中的属性模板（标签、提示、默认值、字高、对齐等）；
//! - `Attribute`：块参照上一次具体的标签/值对，通常由 `AttributeDefinition::create_attribute` 生成；
//! - `BlockTableRecord`：一个块（或布局）所拥有的对象 ID 与属性定义集合；
//! - `BlockTable`：按名称索引全部块表记录并跟踪当前块，`*` 开头的内置记录不可删除或改名；
//! - `InsertEntity`：块参照，记录插入点、比例、旋转角（内部为弧度）与属性值，可展开为行列阵列。
//!
//! 坐标与长度沿用世界单位，`DrawingUnit`/`BlockUnit` 提供到毫米的换算；本模块只维护内存中的数据结构，不读写磁盘。

use serde::{Serialize, Deserialize};
use std::fmt;

/// 属性实例：块参照上一次具体的标签/值对，附带文字排版参数。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    /// 标签名，在块内唯一；查找与更新属性时按它匹配。
    pub tag: String,
    /// 属性值文本。
    pub value: String,
    /// 属性文字的插入点（世界坐标）。
    pub position: crate::geometry::Point,
    /// 文字样式。
    pub text_style: crate::text::TextStyle,
    /// 是否不可见：不绘制，但仍可按标签读取。
    pub is_invisible: bool,
    /// 是否为常量属性：插入后不应被修改。
    pub is_constant: bool,
    /// 插入时是否提示确认取值。
    pub is_verify: bool,
    /// 是否锁定位置：锁定后不可用夹点移动。
    pub is_locked: bool,
    /// 文字对齐方式。
    pub alignment: crate::text::TextAlignment,
    /// 文字高度，单位为图形单位。
    pub height: f64,
    /// 文字旋转角，单位为弧度。
    pub rotation: f64,
    /// 宽度因子；1.0 表示不压缩也不拉伸。
    pub width_factor: f64,
    /// 文字倾斜角，单位为弧度。
    pub oblique_angle: f64,
}

impl Default for Attribute {
    fn default() -> Self {
        Self {
            tag: String::new(),
            value: String::new(),
            position: crate::geometry::Point::origin(),
            text_style: crate::text::TextStyle::default(),
            is_invisible: false,
            is_constant: false,
            is_verify: false,
            is_locked: false,
            alignment: crate::text::TextAlignment::Left,
            height: 2.5,
            rotation: 0.0,
            width_factor: 1.0,
            oblique_angle: 0.0,
        }
    }
}

impl Attribute {
    /// 以标签与取值创建属性实例，其余字段取默认值（字高 2.5、左对齐、可见、未锁定、无旋转）。
    #[inline]
    pub fn new(tag: &str, value: &str) -> Self {
        Self {
            tag: tag.to_string(),
            value: value.to_string(),
            ..Default::default()
        }
    }

    /// 设置属性文字的插入点并返回自身，用于链式构造。
    #[inline]
    pub fn with_position(mut self, position: crate::geometry::Point) -> Self {
        self.position = position;
        self
    }

    /// 修改标签名；不做唯一性校验，调用方需自行保证块内标签不冲突。
    #[inline]
    pub fn set_tag(&mut self, tag: &str) {
        self.tag = tag.to_string();
    }

    /// 覆盖属性值文本；即使 `is_constant` 为真也照常写入。
    #[inline]
    pub fn set_value(&mut self, value: &str) {
        self.value = value.to_string();
    }

    /// 属性是否可见，等价于 `!is_invisible`。
    #[inline]
    pub fn is_visible(&self) -> bool {
        !self.is_invisible
    }
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}={}", self.tag, self.value)
    }
}

/// 属性定义：块定义中的属性模板，用于在插入块时生成一个个 `Attribute`。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributeDefinition {
    /// 标签名，生成属性实例时原样复制。
    pub tag: String,
    /// 插入块时提示用户输入的文本。
    pub prompt: String,
    /// 属性实例的默认取值。
    pub default_value: String,
    /// 属性文字的默认插入点。
    pub position: crate::geometry::Point,
    /// 默认文字样式。
    pub text_style: crate::text::TextStyle,
    /// 字段显示长度；0 表示不限制。
    pub field_length: u32,
    /// 默认文字对齐方式。
    pub alignment: crate::text::TextAlignment,
    /// 生成的属性是否默认不可见。
    pub is_invisible: bool,
    /// 是否为常量属性定义。
    pub is_constant: bool,
    /// 插入时是否提示校验取值。
    pub is_verify: bool,
    /// 生成的属性是否默认锁定位置。
    pub is_locked: bool,
    /// 默认文字高度，单位为图形单位。
    pub height: f64,
    /// 默认旋转角，单位为弧度。
    pub rotation: f64,
    /// 默认宽度因子。
    pub width_factor: f64,
    /// 默认倾斜角，单位为弧度。
    pub oblique_angle: f64,
    /// 是否为多行属性并以底端作为对齐基准。
    pub mtext_bottom: bool,
}

impl Default for AttributeDefinition {
    fn default() -> Self {
        Self {
            tag: String::new(),
            prompt: String::new(),
            default_value: String::new(),
            position: crate::geometry::Point::origin(),
            text_style: crate::text::TextStyle::default(),
            field_length: 0,
            alignment: crate::text::TextAlignment::Left,
            is_invisible: false,
            is_constant: false,
            is_verify: false,
            is_locked: false,
            height: 2.5,
            rotation: 0.0,
            width_factor: 1.0,
            oblique_angle: 0.0,
            mtext_bottom: false,
        }
    }
}

impl AttributeDefinition {
    /// 以标签与提示文本创建属性定义，其余字段取默认值（字高 2.5、左对齐、可见、未锁定）。
    #[inline]
    pub fn new(tag: &str, prompt: &str) -> Self {
        Self {
            tag: tag.to_string(),
            prompt: prompt.to_string(),
            ..Default::default()
        }
    }

    /// 在 `new` 的基础上同时指定属性文字的默认插入点。
    #[inline]
    pub fn with_defaults(tag: &str, prompt: &str, position: crate::geometry::Point) -> Self {
        Self {
            tag: tag.to_string(),
            prompt: prompt.to_string(),
            position,
            ..Default::default()
        }
    }

    /// 按本定义生成属性实例。
    ///
    /// - `value`：写入新属性的值文本；
    ///
    /// 标签、位置、文字样式、字高、旋转角、对齐以及可见/常量/校验/锁定标志均从定义复制，
    /// 不使用 `default_value`；本方法只读 `self`，不修改定义。
    #[inline]
    pub fn create_attribute(&self, value: &str) -> Attribute {
        Attribute {
            tag: self.tag.clone(),
            value: value.to_string(),
            position: self.position,
            text_style: self.text_style.clone(),
            is_invisible: self.is_invisible,
            is_constant: self.is_constant,
            is_verify: self.is_verify,
            is_locked: self.is_locked,
            alignment: self.alignment,
            height: self.height,
            rotation: self.rotation,
            width_factor: self.width_factor,
            oblique_angle: self.oblique_angle,
        }
    }
}

/// 块表记录：一个块（或布局）所拥有的对象 ID 与属性定义集合。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockTableRecord {
    /// 记录名，在块表中唯一；`*` 开头的是系统内置记录。
    pub name: String,
    /// 属于本记录的对象 ID 列表（实体或其他对象的标识）。
    pub objects: Vec<super::data_structure::ObjectId>,
    /// 本块携带的属性定义。
    pub attribute_defs: Vec<AttributeDefinition>,
    /// 块基点，即块坐标系原点在图纸中的位置。
    pub origin: crate::geometry::Point,
    /// 本记录的图纸单位。
    pub units: DrawingUnit,
    /// X/Y 方向的默认缩放比例，通常为 (1.0, 1.0)。
    pub scaling: (f64, f64),
    /// 记录说明文本。
    pub description: String,
    /// 是否允许分解该块。
    pub is_explodable: bool,
    /// 块的归一化单位，可由 `units` 经 `BlockUnit::from_drawing_unit` 得到。
    pub block_unit: BlockUnit,
    /// 备注文本。
    pub comments: String,
}

impl Default for BlockTableRecord {
    fn default() -> Self {
        Self::new("*Unnamed")
    }
}

/// 图纸单位：描述文档线性长度的单位，`conversion_factor` 给出到毫米的换算系数。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrawingUnit {
    /// 无单位：数值原样使用，换算系数为 1.0。
    Unitless,
    /// 英寸（系数 25.4）。
    Inches,
    /// 英尺（系数 304.8）。
    Feet,
    /// 英里（系数 1609344.0）。
    Miles,
    /// 毫米（基准单位，系数 1.0）。
    Millimeters,
    /// 厘米（系数 10.0）。
    Centimeters,
    /// 米（系数 1000.0）。
    Meters,
    /// 千米（系数 10^6）。
    Kilometers,
    /// 微英寸（系数 2.54e-5）。
    Microinches,
    /// 密尔，即千分之一英寸（系数 0.0254）。
    Mils,
    /// 埃（系数 1e-7）。
    Angstroms,
    /// 纳米（系数 1e-6）。
    Nanometers,
    /// 微米（系数 0.001）。
    Microns,
    /// 分米（系数 100.0）。
    Decimeters,
    /// 十米（系数 10^4）。
    Decameters,
    /// 百米（系数 10^5）。
    Hectometers,
    /// 吉米；本实现的换算系数为 1e9 mm（相当于 10^6 米）。
    Gigameters,
    /// 天文单位（系数 ≈ 1.495978707e14）。
    AstronomicalUnits,
    /// 光年（系数 ≈ 9.4607304725808e18）。
    LightYears,
    /// 秒差距（系数 ≈ 3.0856775814913673e19）。
    Parsecs,
}

impl Default for DrawingUnit {
    fn default() -> Self {
        DrawingUnit::Unitless
    }
}

impl DrawingUnit {
    /// 返回该单位到毫米的换算系数；`Unitless` 与 `Millimeters` 均为 1.0，调用方据此换算长度。
    #[inline]
    pub fn conversion_factor(&self) -> f64 {
        match self {
            DrawingUnit::Unitless => 1.0,
            DrawingUnit::Inches => 25.4,
            DrawingUnit::Feet => 304.8,
            DrawingUnit::Miles => 1609344.0,
            DrawingUnit::Millimeters => 1.0,
            DrawingUnit::Centimeters => 10.0,
            DrawingUnit::Meters => 1000.0,
            DrawingUnit::Kilometers => 1000000.0,
            DrawingUnit::Microinches => 0.0000254,
            DrawingUnit::Mils => 0.0254,
            DrawingUnit::Angstroms => 0.0000001,
            DrawingUnit::Nanometers => 0.000001,
            DrawingUnit::Microns => 0.001,
            DrawingUnit::Decimeters => 100.0,
            DrawingUnit::Decameters => 10000.0,
            DrawingUnit::Hectometers => 100000.0,
            DrawingUnit::Gigameters => 1000000000.0,
            DrawingUnit::AstronomicalUnits => 149597870700000.0,
            DrawingUnit::LightYears => 9460730472580800000.0,
            DrawingUnit::Parsecs => 30856775814913673000.0,
        }
    }

    /// 把以本单位为单位的长度换算为毫米（乘以换算系数）。
    #[inline]
    pub fn to_millimeters(&self, value: f64) -> f64 {
        value * self.conversion_factor()
    }

    /// 把毫米长度换算为以本单位为单位的数值（除以换算系数）。
    #[inline]
    pub fn from_millimeters(&self, value: f64) -> f64 {
        value / self.conversion_factor()
    }
}

/// 块单位：块的归一化长度单位，仅保留常用单位，用于块与图纸之间的换算。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockUnit {
    /// 无单位。
    Unitless,
    /// 英寸（也用于英尺、英里等英制图纸单位）。
    Inches,
    /// 英尺。
    Feet,
    /// 毫米。
    Millimeters,
    /// 厘米。
    Centimeters,
    /// 米。
    Meters,
}

impl Default for BlockUnit {
    fn default() -> Self {
        BlockUnit::Unitless
    }
}

impl BlockUnit {
    /// 把图纸单位归一化为块单位：Inches/Feet/Miles 归为 `Inches`，毫米、厘米、米各自对应同名单位，
    /// 其余单位（含 `Unitless` 与天文单位等）一律返回 `Unitless`。
    #[inline]
    pub fn from_drawing_unit(unit: DrawingUnit) -> Self {
        match unit {
            DrawingUnit::Unitless => BlockUnit::Unitless,
            DrawingUnit::Inches | DrawingUnit::Feet | DrawingUnit::Miles => BlockUnit::Inches,
            DrawingUnit::Millimeters => BlockUnit::Millimeters,
            DrawingUnit::Centimeters => BlockUnit::Centimeters,
            DrawingUnit::Meters => BlockUnit::Meters,
            _ => BlockUnit::Unitless,
        }
    }
}

impl BlockTableRecord {
    /// 创建空记录：无对象与属性定义，基点在原点，单位无单位，比例 (1.0, 1.0)，允许分解。
    #[inline]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            objects: Vec::new(),
            attribute_defs: Vec::new(),
            origin: crate::geometry::Point::origin(),
            units: DrawingUnit::Unitless,
            scaling: (1.0, 1.0),
            description: String::new(),
            is_explodable: true,
            block_unit: BlockUnit::default(),
            comments: String::new(),
        }
    }

    /// 加入一个对象 ID；已存在时忽略，因此同一对象不会重复登记。
    #[inline]
    pub fn add_object(&mut self, object_id: super::data_structure::ObjectId) {
        if !self.objects.contains(&object_id) {
            self.objects.push(object_id);
        }
    }

    /// 移除指定对象 ID（删除所有匹配项）；不存在时静默无操作。
    #[inline]
    pub fn remove_object(&mut self, object_id: &super::data_structure::ObjectId) {
        self.objects.retain(|id| id != object_id);
    }

    /// 追加一个属性定义；不校验标签是否重复。
    #[inline]
    pub fn add_attribute_def(&mut self, attr_def: AttributeDefinition) {
        self.attribute_defs.push(attr_def);
    }

    /// 按标签移除属性定义（删除所有同名项）；不存在时静默无操作。
    #[inline]
    pub fn remove_attribute_def(&mut self, tag: &str) {
        self.attribute_defs.retain(|def| def.tag != tag);
    }

    /// 按标签查找属性定义，返回首个匹配项的只读引用；不存在时为 `None`。
    #[inline]
    pub fn get_attribute_def(&self, tag: &str) -> Option<&AttributeDefinition> {
        self.attribute_defs.iter().find(|def| def.tag == tag)
    }

    /// 返回本记录登记的对象个数。
    #[inline]
    pub fn object_count(&self) -> usize {
        self.objects.len()
    }

    /// 返回本记录携带的属性定义个数。
    #[inline]
    pub fn attribute_count(&self) -> usize {
        self.attribute_defs.len()
    }

    /// 清空对象与属性定义；名称、基点、单位等字段保持不变。
    #[inline]
    pub fn clear(&mut self) {
        self.objects.clear();
        self.attribute_defs.clear();
    }
}

/// 块表：按名称索引全部块表记录，并跟踪当前块（通常是 `*Model_Space`）。
///
/// 名称以 `*` 开头的是内置记录，`remove`/`rename` 会拒绝操作它们。
#[derive(Debug, Clone)]
pub struct BlockTable {
    records: std::collections::HashMap<String, BlockTableRecord>,
    current_block: Option<String>,
}

impl Default for BlockTable {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockTable {
    /// 创建空表并注册内置记录 `*Model_Space`、`*Paper_Space`、`*Paper_Space0`（初始 `count()` 为 3）。
    #[inline]
    pub fn new() -> Self {
        let mut table = Self {
            records: std::collections::HashMap::new(),
            current_block: None,
        };
        table.register_builtin_blocks();
        table
    }

    fn register_builtin_blocks(&mut self) {
        self.records.insert("*Model_Space".to_string(), BlockTableRecord::new("*Model_Space"));
        self.records.insert("*Paper_Space".to_string(), BlockTableRecord::new("*Paper_Space"));
        self.records.insert("*Paper_Space0".to_string(), BlockTableRecord::new("*Paper_Space0"));
    }

    /// 注册一条记录，以 `record.name` 为键；同名记录会被覆盖。
    ///
    /// 返回是否写入成功；`record.name` 为空时返回 `false` 且不做任何修改。
    #[inline]
    pub fn register(&mut self, record: BlockTableRecord) -> bool {
        if record.name.is_empty() {
            return false;
        }
        self.records.insert(record.name.clone(), record);
        true
    }

    /// 按名称获取记录的只读引用；不存在时为 `None`。
    #[inline]
    pub fn get(&self, name: &str) -> Option<&BlockTableRecord> {
        self.records.get(name)
    }

    /// 按名称获取记录的可变引用；不存在时为 `None`。
    #[inline]
    pub fn get_mut(&mut self, name: &str) -> Option<&mut BlockTableRecord> {
        self.records.get_mut(name)
    }

    /// 判断表中是否存在同名记录。
    #[inline]
    pub fn has(&self, name: &str) -> bool {
        self.records.contains_key(name)
    }

    /// 删除指定记录。
    ///
    /// 返回是否删除成功；名称以 `*` 开头的内置记录不可删除，记录不存在时同样返回 `false`。
    #[inline]
    pub fn remove(&mut self, name: &str) -> bool {
        if name.starts_with('*') {
            return false;
        }
        self.records.remove(name).is_some()
    }

    /// 重命名记录并同步其 `name` 字段。
    ///
    /// 返回是否成功：新旧名任一以 `*` 开头、或 `old_name` 不存在时返回 `false`；目标名已存在会被覆盖。
    #[inline]
    pub fn rename(&mut self, old_name: &str, new_name: &str) -> bool {
        if old_name.starts_with('*') || new_name.starts_with('*') {
            return false;
        }
        if let Some(record) = self.records.remove(old_name) {
            let mut new_record = record;
            new_record.name = new_name.to_string();
            self.records.insert(new_name.to_string(), new_record);
            true
        } else {
            false
        }
    }

    /// 设置当前块。
    ///
    /// 返回是否成功；记录不存在时返回 `false` 且保持原当前块不变。
    #[inline]
    pub fn set_current(&mut self, name: &str) -> bool {
        if self.records.contains_key(name) {
            self.current_block = Some(name.to_string());
            true
        } else {
            false
        }
    }

    /// 返回当前块名；尚未设置时为 `None`。
    #[inline]
    pub fn current(&self) -> Option<&str> {
        self.current_block.as_deref()
    }

    /// 返回当前块记录的可变引用；未设置当前块或记录已被删除时为 `None`。
    #[inline]
    pub fn current_mut(&mut self) -> Option<&mut BlockTableRecord> {
        if let Some(ref name) = self.current_block {
            self.records.get_mut(name)
        } else {
            None
        }
    }

    /// 返回全部记录名（含内置记录）；顺序取决于哈希表键序，不保证稳定。
    #[inline]
    pub fn names(&self) -> Vec<&str> {
        self.records.keys().map(|s| s.as_str()).collect()
    }

    /// 返回记录总数，包含三个内置记录。
    #[inline]
    pub fn count(&self) -> usize {
        self.records.len()
    }

    /// 清空全部记录、把当前块复位为 `None`，随后重新注册三个内置记录。
    #[inline]
    pub fn clear(&mut self) {
        self.records.clear();
        self.current_block = None;
        self.register_builtin_blocks();
    }
}

/// 块参照（INSERT）：某块在图纸中的一次插入，含插入点、比例、旋转与属性值。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InsertEntity {
    /// 被插入的块名，对应 `BlockTableRecord::name`。
    pub block_name: String,
    /// 插入点（世界坐标）。
    pub position: crate::geometry::Point,
    /// X/Y/Z 三向缩放比例，通常为 (1.0, 1.0, 1.0)。
    pub scale: (f64, f64, f64),
    /// 旋转角，单位为弧度；`with_transform` 接受角度值并自动换算后写入这里。
    pub rotation: f64,
    /// 阵列列数；1 表示不阵列，仅记录参数，需调用 `create_array` 才生成实例。
    pub columns: u32,
    /// 阵列行数；1 表示不阵列，需调用 `create_array` 才生成实例。
    pub rows: u32,
    /// 阵列列间距，世界单位。
    pub column_spacing: f64,
    /// 阵列行间距，世界单位。
    pub row_spacing: f64,
    /// 本次插入的属性值列表，按标签查找与更新。
    pub attributes: Vec<Attribute>,
}

impl Default for InsertEntity {
    fn default() -> Self {
        Self {
            block_name: String::new(),
            position: crate::geometry::Point::origin(),
            scale: (1.0, 1.0, 1.0),
            rotation: 0.0,
            columns: 1,
            rows: 1,
            column_spacing: 0.0,
            row_spacing: 0.0,
            attributes: Vec::new(),
        }
    }
}

impl InsertEntity {
    /// 以块名创建块参照，其余字段取默认值：原点、比例 (1.0, 1.0, 1.0)、无旋转、行列数均为 1。
    #[inline]
    pub fn new(block_name: &str) -> Self {
        Self {
            block_name: block_name.to_string(),
            ..Default::default()
        }
    }

    /// 设置插入点的位置、缩放与旋转。
    ///
    /// - `position`：块参照插入点（世界坐标）。
    /// - `scale`：X/Y/Z 三向缩放比例，通常为 `(1.0, 1.0, 1.0)`。
    /// - `rotation`：旋转角，**单位为度**（与 DXF 组码 50 一致），内部换算为弧度存储。
    #[inline]
    pub fn with_transform(mut self, position: crate::geometry::Point, scale: (f64, f64, f64), rotation_degrees: f64) -> Self {
        self.position = position;
        self.scale = scale;
        self.rotation = rotation_degrees.to_radians();
        self
    }

    /// 设置插入点的位置、缩放与旋转，旋转角**单位为弧度**。
    #[inline]
    pub fn with_transform_radians(
        mut self,
        position: crate::geometry::Point,
        scale: (f64, f64, f64),
        rotation: f64,
    ) -> Self {
        self.position = position;
        self.scale = scale;
        self.rotation = rotation;
        self
    }

    /// 追加一个属性值；不检查标签是否重复，同名属性可能同时存在。
    #[inline]
    pub fn add_attribute(&mut self, attribute: Attribute) {
        self.attributes.push(attribute);
    }

    /// 按标签改写已存在属性的值；标签不存在时静默无操作（不会新建属性）。
    #[inline]
    pub fn set_attribute(&mut self, tag: &str, value: &str) {
        if let Some(attr) = self.attributes.iter_mut().find(|a| a.tag == tag) {
            attr.value = value.to_string();
        }
    }

    /// 按标签查找属性，返回首个匹配项的只读引用；不存在时为 `None`。
    #[inline]
    pub fn get_attribute(&self, tag: &str) -> Option<&Attribute> {
        self.attributes.iter().find(|a| a.tag == tag)
    }

    /// 返回本次插入携带的属性个数。
    #[inline]
    pub fn attribute_count(&self) -> usize {
        self.attributes.len()
    }

    /// 把块参照展开为行列阵列。
    ///
    /// - `columns` / `rows`：列数与行数；
    /// - `col_spacing` / `row_spacing`：列、行间距（世界单位）；
    ///
    /// 返回按行优先顺序排列的块参照：首个元素是原位置的副本，其余沿 +X/+Y 平移，Z 保持不变；
    /// 属性等其余字段逐份克隆。本方法不修改 `self`，也不写回 `columns`/`rows` 字段。
    #[inline]
    pub fn create_array(&self, columns: u32, rows: u32, col_spacing: f64, row_spacing: f64) -> Vec<InsertEntity> {
        let mut array = Vec::new();
        for row in 0..rows {
            for col in 0..columns {
                if row == 0 && col == 0 {
                    array.push(self.clone());
                } else {
                    let offset_x = col as f64 * col_spacing;
                    let offset_y = row as f64 * row_spacing;
                    let new_position = crate::geometry::Point::new(
                        self.position.x + offset_x,
                        self.position.y + offset_y,
                        self.position.z,
                    );
                    let mut new_insert = self.clone();
                    new_insert.position = new_position;
                    array.push(new_insert);
                }
            }
        }
        array
    }

    /// 构造块参照的 4×4 变换矩阵。
    ///
    /// 旋转量取 `rotation`（弧度），2×2 部分由旋转与 `scale` 的 X/Y 分量合成，
    /// 平移分量取 `position`，Z 轴仅按 `scale.2` 缩放；调用方用它把块内坐标映射到世界坐标。
    #[inline]
    pub fn transformation_matrix(&self) -> crate::math::Matrix4 {
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        let (sx, sy, sz) = self.scale;

        crate::math::Matrix4::new(
            sx * cos_r, sy * -sin_r, 0.0, self.position.x,
            sx * sin_r, sy * cos_r, 0.0, self.position.y,
            0.0, 0.0, sz, self.position.z,
            0.0, 0.0, 0.0, 1.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;
    use crate::data_structure::ObjectId;

    #[test]
    fn test_attribute_creation() {
        let attr = Attribute::new("TAG1", "Value1");
        assert_eq!(attr.tag, "TAG1");
        assert_eq!(attr.value, "Value1");
    }

    #[test]
    fn test_attribute_definition() {
        let def = AttributeDefinition::new("TAG1", "Enter value:");
        assert_eq!(def.tag, "TAG1");
        assert_eq!(def.prompt, "Enter value:");
    }

    #[test]
    fn test_attribute_definition_create_attribute() {
        let def = AttributeDefinition::new("TAG1", "Enter value:");
        let attr = def.create_attribute("TestValue");
        assert_eq!(attr.tag, "TAG1");
        assert_eq!(attr.value, "TestValue");
    }

    #[test]
    fn test_block_table_record() {
        let mut record = BlockTableRecord::new("TestBlock");
        assert_eq!(record.name, "TestBlock");
        assert!(record.object_count() == 0);
    }

    #[test]
    fn test_block_table_record_operations() {
        let mut record = BlockTableRecord::new("TestBlock");
        record.add_object(ObjectId::new());
        record.add_object(ObjectId::new());
        assert_eq!(record.object_count(), 2);
        record.clear();
        assert_eq!(record.object_count(), 0);
    }

    #[test]
    fn test_block_table() {
        let table = BlockTable::new();
        assert!(table.has("*Model_Space"));
        assert_eq!(table.count(), 3);
    }

    #[test]
    fn test_block_table_operations() {
        let mut table = BlockTable::new();
        let record = BlockTableRecord::new("MyBlock");
        assert!(table.register(record));
        assert!(table.has("MyBlock"));
        assert!(table.remove("MyBlock"));
        assert!(!table.has("MyBlock"));
    }

    #[test]
    fn test_insert_entity() {
        let insert = InsertEntity::new("TestBlock")
            .with_transform(Point::new(100.0, 100.0, 0.0), (2.0, 2.0, 1.0), 45.0);
        assert_eq!(insert.block_name, "TestBlock");
        assert_eq!(insert.scale, (2.0, 2.0, 1.0));
        assert!((insert.rotation - 45.0 * std::f64::consts::PI / 180.0).abs() < 1e-10);
    }

    #[test]
    fn test_insert_entity_attributes() {
        let mut insert = InsertEntity::new("TestBlock");
        insert.add_attribute(Attribute::new("ATTR1", "Value1"));
        insert.add_attribute(Attribute::new("ATTR2", "Value2"));
        assert_eq!(insert.attribute_count(), 2);
        insert.set_attribute("ATTR1", "NewValue");
        assert_eq!(insert.get_attribute("ATTR1").unwrap().value, "NewValue");
    }

    #[test]
    fn test_insert_entity_array() {
        let insert = InsertEntity::new("TestBlock")
            .with_transform(Point::origin(), (1.0, 1.0, 1.0), 0.0);
        let array = insert.create_array(2, 2, 10.0, 20.0);
        assert_eq!(array.len(), 4);
    }

    #[test]
    fn test_drawing_unit_conversion() {
        assert!((DrawingUnit::Inches.to_millimeters(1.0) - 25.4).abs() < 1e-10);
        assert!((DrawingUnit::Millimeters.to_millimeters(25.4) - 25.4).abs() < 1e-10);
    }

    #[test]
    fn test_block_unit_from_drawing_unit() {
        assert_eq!(BlockUnit::from_drawing_unit(DrawingUnit::Millimeters), BlockUnit::Millimeters);
        assert_eq!(BlockUnit::from_drawing_unit(DrawingUnit::Inches), BlockUnit::Inches);
    }
}
