use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaderType {
    Straight,
    Spline,
}

impl Default for LeaderType {
    fn default() -> Self {
        LeaderType::Straight
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaderAttachmentType {
    AttachmentTop,
    AttachmentMiddle,
    AttachmentBottom,
}

impl Default for LeaderAttachmentType {
    fn default() -> Self {
        LeaderAttachmentType::AttachmentTop
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaderBranchType {
    Left,
    Right,
}

impl Default for LeaderBranchType {
    fn default() -> Self {
        LeaderBranchType::Left
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaderLine {
    pub points: Vec<crate::geometry::Point>,
    pub leader_type: LeaderType,
    pub branch_type: LeaderBranchType,
    pub last_segment_angle: f64,
}

impl Default for LeaderLine {
    fn default() -> Self {
        Self::new()
    }
}

impl LeaderLine {
    #[inline]
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            leader_type: LeaderType::Straight,
            branch_type: LeaderBranchType::Left,
            last_segment_angle: 0.0,
        }
    }

    #[inline]
    pub fn with_points(points: &[crate::geometry::Point]) -> Self {
        let last_segment_angle = if points.len() >= 2 {
            let dx = points[points.len() - 1].x - points[points.len() - 2].x;
            let dy = points[points.len() - 1].y - points[points.len() - 2].y;
            dy.atan2(dx)
        } else {
            0.0
        };

        Self {
            points: points.to_vec(),
            leader_type: LeaderType::Straight,
            branch_type: LeaderBranchType::Left,
            last_segment_angle,
        }
    }

    #[inline]
    pub fn add_point(&mut self, point: crate::geometry::Point) {
        if self.points.len() >= 2 {
            let last_idx = self.points.len() - 1;
            let dx = point.x - self.points[last_idx].x;
            let dy = point.y - self.points[last_idx].y;
            self.last_segment_angle = dy.atan2(dx);
        }
        self.points.push(point);
    }

    #[inline]
    pub fn point_count(&self) -> usize {
        self.points.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    #[inline]
    pub fn length(&self) -> f64 {
        if self.points.len() < 2 {
            return 0.0;
        }
        let mut total = 0.0;
        for i in 1..self.points.len() {
            total += self.points[i].distance_to(&self.points[i - 1]);
        }
        total
    }

    #[inline]
    pub fn start_point(&self) -> Option<&crate::geometry::Point> {
        self.points.first()
    }

    #[inline]
    pub fn end_point(&self) -> Option<&crate::geometry::Point> {
        self.points.last()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arrowhead {
    pub position: crate::geometry::Point,
    pub size: f64,
    pub arrowhead_type: ArrowheadType,
    pub angle: f64,
}

impl Default for Arrowhead {
    fn default() -> Self {
        Self::new()
    }
}

impl Arrowhead {
    #[inline]
    pub fn new() -> Self {
        Self {
            position: crate::geometry::Point::origin(),
            size: 2.5,
            arrowhead_type: ArrowheadType::ArrowClosedFilled,
            angle: 0.0,
        }
    }

    #[inline]
    pub fn with_position(mut self, position: crate::geometry::Point) -> Self {
        self.position = position;
        self
    }

    #[inline]
    pub fn with_size(mut self, size: f64) -> Self {
        self.size = size;
        self
    }

    #[inline]
    pub fn with_type(mut self, arrow_type: ArrowheadType) -> Self {
        self.arrowhead_type = arrow_type;
        self
    }

    #[inline]
    pub fn set_angle(&mut self, angle: f64) {
        self.angle = angle;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArrowheadType {
    None,
    DotSmall,
    Dot,
    ArrowOpen,
    ArrowOpen90,
    ArrowOpen45,
    ArrowClosed,
    ArrowClosedFilled,
    Tick,
    Tick2x,
    Triangle,
    Triangle90,
    Triangle45,
    Square,
    Hexagon,
    Circle,
    CircleFilled,
    DotBlank,
    Origin,
    Origin2,
    NoneFilled,
    User,
}

impl Default for ArrowheadType {
    fn default() -> Self {
        ArrowheadType::ArrowClosedFilled
    }
}

impl ArrowheadType {
    #[inline]
    pub fn name(&self) -> &str {
        match self {
            ArrowheadType::None => "None",
            ArrowheadType::DotSmall => "Dot Small",
            ArrowheadType::Dot => "Dot",
            ArrowheadType::ArrowOpen => "Open",
            ArrowheadType::ArrowOpen90 => "Open 90",
            ArrowheadType::ArrowOpen45 => "Open 45",
            ArrowheadType::ArrowClosed => "Closed",
            ArrowheadType::ArrowClosedFilled => "Closed Filled",
            ArrowheadType::Tick => "Tick",
            ArrowheadType::Tick2x => "Tick 2x",
            ArrowheadType::Triangle => "Triangle",
            ArrowheadType::Triangle90 => "Triangle 90",
            ArrowheadType::Triangle45 => "Triangle 45",
            ArrowheadType::Square => "Square",
            ArrowheadType::Hexagon => "Hexagon",
            ArrowheadType::Circle => "Circle",
            ArrowheadType::CircleFilled => "Circle Filled",
            ArrowheadType::DotBlank => "Dot Blank",
            ArrowheadType::Origin => "Origin",
            ArrowheadType::Origin2 => "Origin 2",
            ArrowheadType::NoneFilled => "None Filled",
            ArrowheadType::User => "User Defined",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MTextContent {
    pub text: String,
    pub width: f64,
    pub text_style: crate::text::TextStyle,
    pub alignment: crate::text::TextAlignment,
    pub line_spacing: f64,
}

impl Default for MTextContent {
    fn default() -> Self {
        Self {
            text: String::new(),
            width: 0.0,
            text_style: crate::text::TextStyle::default(),
            alignment: crate::text::TextAlignment::Left,
            line_spacing: 1.0,
        }
    }
}

impl MTextContent {
    #[inline]
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            ..Default::default()
        }
    }

    #[inline]
    pub fn with_width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    #[inline]
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    #[inline]
    pub fn append_text(&mut self, text: &str) {
        self.text.push_str(text);
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockContent {
    pub block_name: String,
    pub position: crate::geometry::Point,
    pub scale: (f64, f64),
    pub rotation: f64,
    pub attribute_values: Vec<(String, String)>,
}

impl Default for BlockContent {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockContent {
    #[inline]
    pub fn new() -> Self {
        Self {
            block_name: String::new(),
            position: crate::geometry::Point::origin(),
            scale: (1.0, 1.0),
            rotation: 0.0,
            attribute_values: Vec::new(),
        }
    }

    #[inline]
    pub fn with_block(mut self, block_name: &str) -> Self {
        self.block_name = block_name.to_string();
        self
    }

    #[inline]
    pub fn set_attribute(&mut self, tag: &str, value: &str) {
        if let Some((_, val)) = self.attribute_values.iter_mut().find(|(t, _)| t == tag) {
            *val = value.to_string();
        } else {
            self.attribute_values.push((tag.to_string(), value.to_string()));
        }
    }

    #[inline]
    pub fn get_attribute(&self, tag: &str) -> Option<&str> {
        self.attribute_values.iter()
            .find(|(t, _)| t == tag)
            .map(|(_, v)| v.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MultileaderContent {
    MText(MTextContent),
    Block(BlockContent),
}

impl Default for MultileaderContent {
    fn default() -> Self {
        MultileaderContent::MText(MTextContent::default())
    }
}

impl MultileaderContent {
    #[inline]
    pub fn mtext(text: &str) -> Self {
        MultileaderContent::MText(MTextContent::new(text))
    }

    #[inline]
    pub fn block(block_name: &str) -> Self {
        MultileaderContent::Block(BlockContent::new().with_block(block_name))
    }

    #[inline]
    pub fn is_mtext(&self) -> bool {
        matches!(self, MultileaderContent::MText(_))
    }

    #[inline]
    pub fn is_block(&self) -> bool {
        matches!(self, MultileaderContent::Block(_))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Multileader {
    pub object_id: super::data_structure::ObjectId,
    pub style_name: String,
    pub leaders: Vec<LeaderLine>,
    pub arrowheads: Vec<Arrowhead>,
    pub content: MultileaderContent,
    pub content_attachment: LeaderAttachmentType,
    pub last_leader_point: crate::geometry::Point,
    pub dogleg_vector: crate::geometry::Point,
    pub dogleg_length: f64,
    pub branch_angle: f64,
    pub is_landing_set: bool,
    pub landing_gap: f64,
    pub enable_frame_text: bool,
    pub text_height: f64,
    pub text_rotation: f64,
}

impl Default for Multileader {
    fn default() -> Self {
        Self::new()
    }
}

impl Multileader {
    #[inline]
    pub fn new() -> Self {
        Self {
            object_id: super::data_structure::ObjectId::new(),
            style_name: "Standard".to_string(),
            leaders: Vec::new(),
            arrowheads: Vec::new(),
            content: MultileaderContent::MText(MTextContent::new("")),
            content_attachment: LeaderAttachmentType::AttachmentTop,
            last_leader_point: crate::geometry::Point::origin(),
            dogleg_vector: crate::geometry::Point::new(1.0, 0.0, 0.0),
            dogleg_length: 8.0,
            branch_angle: 90.0_f64.to_radians(),
            is_landing_set: true,
            landing_gap: 2.0,
            enable_frame_text: false,
            text_height: 2.5,
            text_rotation: 0.0,
        }
    }

    #[inline]
    pub fn with_content(mut self, content: MultileaderContent) -> Self {
        self.content = content;
        self
    }

    #[inline]
    pub fn add_leader(&mut self, leader: LeaderLine) {
        self.leaders.push(leader);
    }

    #[inline]
    pub fn add_leader_with_arrowhead(&mut self, leader: LeaderLine, arrowhead: Arrowhead) {
        self.leaders.push(leader.clone());
        self.arrowheads.push(arrowhead);
    }

    #[inline]
    pub fn leader_count(&self) -> usize {
        self.leaders.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.leaders.is_empty()
    }

    #[inline]
    pub fn set_style(&mut self, style_name: &str) {
        self.style_name = style_name.to_string();
    }

    #[inline]
    pub fn set_text(&mut self, text: &str) {
        if let MultileaderContent::MText(mtext) = &mut self.content {
            mtext.set_text(text);
        }
    }

    #[inline]
    pub fn get_text(&self) -> Option<&str> {
        match &self.content {
            MultileaderContent::MText(mtext) => Some(&mtext.text),
            _ => None,
        }
    }

    #[inline]
    pub fn set_dogleg_length(&mut self, length: f64) {
        self.dogleg_length = length;
    }

    #[inline]
    pub fn set_branch_angle(&mut self, angle: f64) {
        self.branch_angle = angle;
    }

    #[inline]
    pub fn set_content_attachment(&mut self, attachment: LeaderAttachmentType) {
        self.content_attachment = attachment;
    }

    #[inline]
    pub fn rebuild_leaders(&mut self) {
        for (idx, leader) in self.leaders.iter_mut().enumerate() {
            if let Some(end_point) = leader.points.last() {
                let dx = end_point.x - self.last_leader_point.x;
                let dy = end_point.y - self.last_leader_point.y;
                let base_angle = dy.atan2(dx);

                let dogleg_angle = base_angle + self.branch_angle / 2.0;

                let new_point = crate::geometry::Point::new(
                    end_point.x + dogleg_angle.cos() * self.dogleg_length,
                    end_point.y + dogleg_angle.sin() * self.dogleg_length,
                    end_point.z,
                );

                if idx == 0 {
                    leader.points.insert(leader.points.len() - 1, new_point);
                } else {
                    leader.points.insert(1, new_point);
                }
            }
        }
    }
}

impl fmt::Display for Multileader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MLEADER: {} leaders, {} arrowheads", self.leaders.len(), self.arrowheads.len())
    }
}

#[derive(Debug, Clone)]
pub struct MultileaderBuilder {
    multileader: Multileader,
}

impl Default for MultileaderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl MultileaderBuilder {
    #[inline]
    pub fn new() -> Self {
        Self {
            multileader: Multileader::new(),
        }
    }

    #[inline]
    pub fn style(mut self, style_name: &str) -> Self {
        self.multileader.set_style(style_name);
        self
    }

    #[inline]
    pub fn content(mut self, content: MultileaderContent) -> Self {
        self.multileader.content = content;
        self
    }

    #[inline]
    pub fn text(mut self, text: &str) -> Self {
        self.multileader.set_text(text);
        self
    }

    #[inline]
    pub fn add_leader_point(&mut self, point: crate::geometry::Point) {
        if let Some(last_leader) = self.multileader.leaders.last_mut() {
            last_leader.add_point(point);
        } else {
            let mut leader = LeaderLine::new();
            leader.add_point(point);
            self.multileader.leaders.push(leader);
        }
        self.multileader.last_leader_point = point;
    }

    #[inline]
    pub fn start_leader(&mut self, point: crate::geometry::Point) {
        let leader = LeaderLine::new();
        self.multileader.leaders.push(leader);
        self.multileader.last_leader_point = point;
    }

    #[inline]
    pub fn add_arrowhead(&mut self, arrowhead: Arrowhead) {
        self.multileader.arrowheads.push(arrowhead);
    }

    #[inline]
    pub fn set_dogleg_length(mut self, length: f64) -> Self {
        self.multileader.set_dogleg_length(length);
        self
    }

    #[inline]
    pub fn set_branch_angle(mut self, angle: f64) -> Self {
        self.multileader.set_branch_angle(angle);
        self
    }

    #[inline]
    pub fn set_content_attachment(mut self, attachment: LeaderAttachmentType) -> Self {
        self.multileader.set_content_attachment(attachment);
        self
    }

    #[inline]
    pub fn build(self) -> Multileader {
        self.multileader
    }

    #[inline]
    pub fn build_mut(&mut self) -> &mut Multileader {
        &mut self.multileader
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultileaderStyle {
    pub name: String,
    pub description: String,
    pub arrowhead_type: ArrowheadType,
    pub arrowhead_size: f64,
    pub leader_type: LeaderType,
    pub content_type: ContentType,
    pub text_style: crate::text::TextStyle,
    pub text_height: f64,
    pub text_rotation: f64,
    pub text_alignment: crate::text::TextAlignment,
    pub text_attachment: TextAttachment,
    pub text_frame_enabled: bool,
    pub landing_gap: f64,
    pub dogleg_length: f64,
    pub branch_angle: f64,
    pub enable_frame_text: bool,
    pub scale_factor: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    MText,
    Block,
    CopyContent,
}

impl Default for ContentType {
    fn default() -> Self {
        ContentType::MText
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAttachment {
    TopOfTop,
    MiddleOfTop,
    Middle,
    BottomOfTop,
    BottomOfBottom,
    MiddleOfBottom,
    AttachmentBottom,
}

impl Default for TextAttachment {
    fn default() -> Self {
        TextAttachment::TopOfTop
    }
}

impl Default for MultileaderStyle {
    fn default() -> Self {
        Self::new("Standard")
    }
}

impl MultileaderStyle {
    #[inline]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            arrowhead_type: ArrowheadType::ArrowClosedFilled,
            arrowhead_size: 2.5,
            leader_type: LeaderType::Straight,
            content_type: ContentType::MText,
            text_style: crate::text::TextStyle::default(),
            text_height: 2.5,
            text_rotation: 0.0,
            text_alignment: crate::text::TextAlignment::Left,
            text_attachment: TextAttachment::TopOfTop,
            text_frame_enabled: false,
            landing_gap: 2.0,
            dogleg_length: 8.0,
            branch_angle: 90.0_f64.to_radians(),
            enable_frame_text: false,
            scale_factor: 1.0,
        }
    }

    #[inline]
    pub fn set_arrowhead_type(&mut self, arrow_type: ArrowheadType) {
        self.arrowhead_type = arrow_type;
    }

    #[inline]
    pub fn set_arrowhead_size(&mut self, size: f64) {
        self.arrowhead_size = size;
    }

    #[inline]
    pub fn set_text_height(&mut self, height: f64) {
        self.text_height = height;
    }

    #[inline]
    pub fn set_dogleg_length(&mut self, length: f64) {
        self.dogleg_length = length;
    }

    #[inline]
    pub fn set_branch_angle(&mut self, angle: f64) {
        self.branch_angle = angle;
    }

    #[inline]
    pub fn create_multileader(&self) -> Multileader {
        Multileader::new()
            .with_content(MultileaderContent::MText(MTextContent::new("")))
    }
}

#[derive(Debug, Clone)]
pub struct MultileaderManager {
    multileaders: std::collections::HashMap<super::data_structure::ObjectId, Multileader>,
    styles: std::collections::HashMap<String, MultileaderStyle>,
    current_style: String,
    active_multileader: Option<super::data_structure::ObjectId>,
}

impl Default for MultileaderManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MultileaderManager {
    #[inline]
    pub fn new() -> Self {
        let mut manager = Self {
            multileaders: std::collections::HashMap::new(),
            styles: std::collections::HashMap::new(),
            current_style: "Standard".to_string(),
            active_multileader: None,
        };
        manager.register_builtin_styles();
        manager
    }

    fn register_builtin_styles(&mut self) {
        self.styles.insert("Standard".to_string(), MultileaderStyle::new("Standard"));
        self.styles.insert("StandardWithFrame".to_string(), {
            let mut style = MultileaderStyle::new("StandardWithFrame");
            style.text_frame_enabled = true;
            style
        });
    }

    #[inline]
    pub fn add(&mut self, multileader: Multileader) -> super::data_structure::ObjectId {
        let object_id = multileader.object_id.clone();
        self.multileaders.insert(object_id.clone(), multileader);
        object_id
    }

    #[inline]
    pub fn create(&mut self) -> super::data_structure::ObjectId {
        let style = self.styles.get(&self.current_style).cloned().unwrap_or_default();
        let multileader = style.create_multileader();
        self.add(multileader)
    }

    #[inline]
    pub fn get(&self, object_id: &super::data_structure::ObjectId) -> Option<&Multileader> {
        self.multileaders.get(object_id)
    }

    #[inline]
    pub fn get_mut(&mut self, object_id: &super::data_structure::ObjectId) -> Option<&mut Multileader> {
        self.multileaders.get_mut(object_id)
    }

    #[inline]
    pub fn remove(&mut self, object_id: &super::data_structure::ObjectId) -> bool {
        self.multileaders.remove(object_id).is_some()
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.multileaders.len()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.multileaders.clear();
    }

    #[inline]
    pub fn set_style(&mut self, style_name: &str) -> bool {
        if self.styles.contains_key(style_name) {
            self.current_style = style_name.to_string();
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn current_style(&self) -> &str {
        &self.current_style
    }

    #[inline]
    pub fn add_style(&mut self, style: MultileaderStyle) -> bool {
        if style.name.is_empty() {
            return false;
        }
        self.styles.insert(style.name.clone(), style);
        true
    }

    #[inline]
    pub fn get_style(&self, name: &str) -> Option<&MultileaderStyle> {
        self.styles.get(name)
    }

    #[inline]
    pub fn style_names(&self) -> Vec<&str> {
        self.styles.keys().map(|s| s.as_str()).collect()
    }

    #[inline]
    pub fn set_active(&mut self, object_id: Option<super::data_structure::ObjectId>) {
        self.active_multileader = object_id;
    }

    #[inline]
    pub fn active(&self) -> Option<&Multileader> {
        self.active_multileader.as_ref().and_then(|id| self.get(id))
    }

    #[inline]
    pub fn active_mut(&mut self) -> Option<&mut Multileader> {
        let id = self.active_multileader.clone()?;
        self.get_mut(&id)
    }

    #[inline]
    pub fn add_leader_to_active(&mut self, point: crate::geometry::Point) -> bool {
        if let Some(mleader) = self.active_mut() {
            if let Some(last_leader) = mleader.leaders.last_mut() {
                last_leader.add_point(point);
            } else {
                let mut leader = LeaderLine::new();
                leader.add_point(point);
                mleader.leaders.push(leader);
            }
            mleader.last_leader_point = point;
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn add_leader_point(&mut self, object_id: &super::data_structure::ObjectId, point: crate::geometry::Point) -> bool {
        if let Some(mleader) = self.get_mut(object_id) {
            if let Some(last_leader) = mleader.leaders.last_mut() {
                last_leader.add_point(point);
            } else {
                let mut leader = LeaderLine::new();
                leader.add_point(point);
                mleader.leaders.push(leader);
            }
            mleader.last_leader_point = point;
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn all(&self) -> &std::collections::HashMap<super::data_structure::ObjectId, Multileader> {
        &self.multileaders
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn test_leader_line() {
        let leader = LeaderLine::new();
        assert!(leader.is_empty());

        let points = vec![
            Point::new(0.0, 0.0, 0.0),
            Point::new(10.0, 0.0, 0.0),
            Point::new(10.0, 10.0, 0.0),
        ];
        let leader = LeaderLine::with_points(&points);
        assert_eq!(leader.point_count(), 3);
        assert!((leader.length() - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_leader_line_add_point() {
        let mut leader = LeaderLine::new();
        leader.add_point(Point::new(0.0, 0.0, 0.0));
        leader.add_point(Point::new(10.0, 0.0, 0.0));
        assert_eq!(leader.point_count(), 2);
        assert!((leader.length() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_arrowhead() {
        let arrowhead = Arrowhead::new()
            .with_position(Point::new(100.0, 100.0, 0.0))
            .with_size(3.0)
            .with_type(ArrowheadType::ArrowClosedFilled);

        assert!((arrowhead.position.x - 100.0).abs() < 1e-10);
        assert_eq!(arrowhead.size, 3.0);
    }

    #[test]
    fn test_multileader() {
        let mut mleader = Multileader::new();
        let leader = LeaderLine::with_points(&[
            Point::new(0.0, 0.0, 0.0),
            Point::new(5.0, 0.0, 0.0),
            Point::new(10.0, 10.0, 0.0),
        ]);
        mleader.add_leader(leader);
        assert_eq!(mleader.leader_count(), 1);
        assert!(!mleader.is_empty());
    }

    #[test]
    fn test_multileader_builder() {
        let builder = MultileaderBuilder::new()
            .text("Test Label")
            .set_dogleg_length(10.0)
            .set_branch_angle(90.0_f64.to_radians());

        let mleader = builder.build();
        assert_eq!(mleader.get_text(), Some("Test Label"));
        assert!((mleader.dogleg_length - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_multileader_content() {
        assert!(MultileaderContent::mtext("Test").is_mtext());
        assert!(!MultileaderContent::mtext("Test").is_block());

        assert!(MultileaderContent::block("MyBlock").is_block());
        assert!(!MultileaderContent::block("MyBlock").is_mtext());
    }

    #[test]
    fn test_multileader_style() {
        let style = MultileaderStyle::new("TestStyle");
        assert_eq!(style.name, "TestStyle");
        assert_eq!(style.arrowhead_size, 2.5);
    }

    #[test]
    fn test_multileader_style_operations() {
        let mut style = MultileaderStyle::new("TestStyle");
        style.set_arrowhead_type(ArrowheadType::Dot);
        style.set_arrowhead_size(5.0);
        style.set_text_height(3.0);
        assert_eq!(style.arrowhead_type, ArrowheadType::Dot);
        assert!((style.arrowhead_size - 5.0).abs() < 1e-10);
        assert!((style.text_height - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_multileader_manager() {
        let manager = MultileaderManager::new();
        assert!(manager.count() == 0);
        assert!(manager.get_style("Standard").is_some());
    }

    #[test]
    fn test_multileader_manager_operations() {
        let mut manager = MultileaderManager::new();
        let object_id = manager.create();

        assert_eq!(manager.count(), 1);
        assert!(manager.get(&object_id).is_some());

        assert!(manager.remove(&object_id));
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_multileader_style_management() {
        let mut manager = MultileaderManager::new();
        let mut style = MultileaderStyle::new("Custom");
        style.description = "Custom multileader style".to_string();
        assert!(manager.add_style(style));
        assert!(manager.get_style("Custom").is_some());

        assert_eq!(manager.set_style("Custom"), true);
        assert_eq!(manager.current_style(), "Custom");
    }

    #[test]
    fn test_multileader_text_content() {
        let mut mleader = Multileader::new();
        mleader.set_text("Hello\nWorld");
        assert_eq!(mleader.get_text(), Some("Hello\nWorld"));

        mleader.set_text("Updated");
        assert_eq!(mleader.get_text(), Some("Updated"));
    }

    #[test]
    fn test_leader_types() {
        let straight_leader = LeaderLine::with_points(&[
            Point::new(0.0, 0.0, 0.0),
            Point::new(10.0, 0.0, 0.0),
        ]);
        assert_eq!(straight_leader.leader_type, LeaderType::Straight);
    }

    #[test]
    fn test_multileader_rebuild() {
        let mut mleader = Multileader::new();
        let leader = LeaderLine::with_points(&[
            Point::new(0.0, 0.0, 0.0),
            Point::new(10.0, 0.0, 0.0),
        ]);
        mleader.add_leader(leader);
        mleader.dogleg_length = 8.0;
        mleader.branch_angle = 90.0_f64.to_radians();
        mleader.last_leader_point = Point::new(10.0, 0.0, 0.0);

        mleader.rebuild_leaders();

        if let Some(leader) = mleader.leaders.first() {
            assert!(leader.point_count() >= 3);
        }
    }
}
