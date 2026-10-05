//! 事件系统：文档与界面之间的解耦通知机制。
//!
//! 事件以 [`EventType`] 分类，载荷放在 [`Event`] 中携带（源对象 id 与任意类型的数据）。
//! [`EventEmitter`] 管理单一事件类型的处理器列表，[`EventBus`] 则按事件类型分发并额外支持
//! 全局处理器。处理器按优先级降序调用，优先级为负数的处理器不参与触发；处理器内调用
//! [`Event::stop_propagation`] 可中止后续处理器的执行。触发只通知订阅者，不直接改动文档。

use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex};
use super::ObjectId;

/// 事件类别，同时用作 [`EventBus`] 中分发器的键。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    /// 选择集发生变化。
    SelectionChanged,
    /// 实体被加入文档。
    EntityAdded,
    /// 实体被移出文档。
    EntityRemoved,
    /// 实体属性或几何被修改。
    EntityModified,
    /// 图层被创建。
    LayerAdded,
    /// 图层被删除。
    LayerRemoved,
    /// 图层属性（颜色、线宽等）被修改。
    LayerModified,
    /// 块定义被创建。
    BlockAdded,
    /// 块定义被删除。
    BlockRemoved,
    /// 块定义被修改。
    BlockModified,
    /// 文档整体发生变化（如新建、打开、另存）。
    DocumentModified,
    /// 视口范围或内容变化。
    ViewportChanged,
    /// 光标位置移动。
    CursorMoved,
    /// 键盘按键按下。
    KeyPressed,
    /// 鼠标点击。
    MouseClicked,
    /// 鼠标拖拽。
    MouseDragged,
    /// 缩放比例变化。
    ZoomChanged,
    /// 捕捉到栅格点。
    GridSnapped,
    /// 当前工具切换。
    ToolChanged,
    /// 图层可见性变化。
    LayerVisibilityChanged,
    /// 图层锁定状态变化。
    LayerLockChanged,
    /// 选择高亮更新。
    SelectionHighlighted,
    /// 鼠标悬停到对象上。
    ObjectHovered,
    /// 事务（一次可撤销操作）开始。
    TransactionStarted,
    /// 事务结束并提交。
    TransactionEnded,
    /// 执行了撤销。
    UndoPerformed,
    /// 执行了重做。
    RedoPerformed,
    /// 自定义事件，按名称区分，可用于插件扩展。
    Custom(String),
}

/// 一次事件实例：类别、发生时间、可选来源对象与可选载荷。
#[derive(Debug, Clone)]
pub struct Event {
    event_type: EventType,
    timestamp: std::time::SystemTime,
    source: Option<ObjectId>,
    data: Option<Box<dyn Any>>,
    propagated: bool,
}

impl Event {
    /// 新建事件，时间戳取当前系统时间，来源与载荷为空，传播未中止。
    ///
    /// - `event_type`：事件类别，决定 [`EventBus`] 用哪个分发器处理。
    #[inline]
    pub fn new(event_type: EventType) -> Self {
        Self {
            event_type,
            timestamp: std::time::SystemTime::now(),
            source: None,
            data: None,
            propagated: false,
        }
    }

    /// 设置来源对象并返回自身，用于链式构造。
    ///
    /// - `source`：触发本次事件的实体/图层等对象标识，消费 `self` 的所有权。
    #[inline]
    pub fn with_source(mut self, source: ObjectId) -> Self {
        self.source = Some(source);
        self
    }

    /// 挂载任意类型的载荷并返回自身，用于链式构造。
    ///
    /// - `data`：载荷数据，按具体类型装箱保存；重复调用会覆盖前一次的数据。
    #[inline]
    pub fn with_data<T: Any>(mut self, data: T) -> Self {
        self.data = Some(Box::new(data));
        self
    }

    /// 事件类别。
    #[inline]
    pub fn event_type(&self) -> &EventType {
        &self.event_type
    }

    /// 事件创建时刻，可用于乱序判定或日志排序。
    #[inline]
    pub fn timestamp(&self) -> &std::time::SystemTime {
        &self.timestamp
    }

    /// 事件来源对象标识；未通过 [`Event::with_source`] 设置时返回 [`None`]。
    #[inline]
    pub fn source(&self) -> Option<&ObjectId> {
        self.source.as_ref()
    }

    /// 取出载荷并要求类型匹配，会修改 `self`（载荷被移出，事件不再持有它）。
    ///
    /// 返回 `T` 类型的载荷；载荷为空或实际类型不是 `T` 时返回 [`None`]，此时载荷已被丢弃。
    /// 同一事件上重复调用只有第一次可能成功。
    #[inline]
    pub fn take_data<T: Any>(&mut self) -> Option<T> {
        self.data.take().and_then(|boxed| boxed.downcast().ok().map(|boxed| *boxed))
    }

    /// 判断载荷是否为 `T` 类型，不取走数据。
    pub fn has_data<T: Any>(&self) -> bool {
        self.data.as_ref().and_then(|d| d.downcast_ref::<T>()).is_some()
    }

    /// 传播是否已被中止；为真时后续处理器不再收到本事件。
    #[inline]
    pub fn is_propagation_stopped(&self) -> bool {
        self.propagated
    }

    /// 中止事件传播，会修改 `self`；调用后同一轮分发中优先级更低的处理器被跳过。
    #[inline]
    pub fn stop_propagation(&mut self) {
        self.propagated = true;
    }
}

/// 事件处理器：可跨线程共享、可重入的闭包，接收事件的可变引用以便读取载荷或中止传播。
pub type EventHandler = Arc<dyn Fn(&mut Event) + Send + Sync>;

/// 一条处理器注册记录：处理器本体、优先级与是否只触发一次。
pub struct EventHandlerRegistration {
    handler: EventHandler,
    priority: i32,
    once: bool,
}

impl EventHandlerRegistration {
    /// 新建一条注册记录。
    ///
    /// - `handler`：处理器闭包，以 [`Arc`] 共享；
    /// - `priority`：优先级，数值越大越先调用，负数表示不触发；
    /// - `once`：为真时该处理器触发一次后即被移除。
    #[inline]
    pub fn new(handler: EventHandler, priority: i32, once: bool) -> Self {
        Self {
            handler,
            priority,
            once,
        }
    }

    /// 处理器本体，可用于比对以注销指定处理器。
    #[inline]
    pub fn handler(&self) -> &EventHandler {
        &self.handler
    }

    /// 注册时指定的优先级，仅参与排序与是否触发的判断。
    #[inline]
    pub fn priority(&self) -> i32 {
        self.priority
    }

    /// 是否为一次性处理器，触发后会被自动移除。
    #[inline]
    pub fn is_once(&self) -> bool {
        self.once
    }
}

/// 单一事件类型的分发器：持有该类型的处理器列表并按优先级调用。
pub struct EventEmitter {
    handlers: Vec<EventHandlerRegistration>,
    event_type: EventType,
}

impl EventEmitter {
    /// 新建分发器，处理器列表为空。
    ///
    /// - `event_type`：本分发器负责的事件类别，仅作为标识字段保存。
    #[inline]
    pub fn new(event_type: EventType) -> Self {
        Self {
            handlers: Vec::new(),
            event_type,
        }
    }

    /// 注册一个常驻处理器，会修改 `self`。
    ///
    /// - `priority`：优先级，数值越大越先被调用，负数的处理器在 [`EventEmitter::emit`] 中被跳过；
    /// - `handler`：处理器闭包，需满足 `Send + Sync + 'static`。
    ///
    /// 注册后处理器列表按优先级降序重排；同一闭包重复注册会各自触发。
    #[inline]
    pub fn on<F>(&mut self, priority: i32, handler: F)
    where
        F: Fn(&mut Event) + Send + Sync + 'static,
    {
        self.handlers.push(EventHandlerRegistration::new(
            Arc::new(handler),
            priority,
            false,
        ));
        self.handlers.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// 注册一个一次性处理器，会修改 `self`。
    ///
    /// - `priority`：优先级，语义同 [`EventEmitter::on`]；
    /// - `handler`：处理器闭包。
    ///
    /// 该处理器在首次触发后即被移出列表；若因优先级为负从未触发，则不会被移除。
    #[inline]
    pub fn once<F>(&mut self, priority: i32, handler: F)
    where
        F: Fn(&mut Event) + Send + Sync + 'static,
    {
        self.handlers.push(EventHandlerRegistration::new(
            Arc::new(handler),
            priority,
            true,
        ));
        self.handlers.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// 注销指定处理器，会修改 `self`。
    ///
    /// - `handler`：要移除的处理器，按 [`Arc`] 指针相等比较，须与注册时传入的是同一实例。
    ///
    /// 移除所有匹配项；未找到时静默无操作。
    #[inline]
    pub fn off(&mut self, handler: &EventHandler) {
        self.handlers.retain(|h| &h.handler != handler);
    }

    /// 触发一次事件，会修改 `self` 与传入的 `event`。
    ///
    /// 按优先级从高到低调用处理器：优先级为负的处理器被直接跳过；一次性处理器调用后被移除；
    /// 某处理器调用 [`Event::stop_propagation`] 后，剩余处理器不再收到本事件。
    ///
    /// - `event`：待分发的事件，处理器可通过它读取载荷或中止传播。
    #[inline]
    pub fn emit(&mut self, event: &mut Event) {
        self.handlers.retain_mut(|registration| {
            let mut should_retain = true;
            if registration.priority >= 0 {
                (registration.handler)(event);
                should_retain = !registration.once && !event.is_propagation_stopped();
            }
            should_retain
        });
    }

    /// 当前注册的处理器个数，含尚未触发的一次性处理器与优先级为负的处理器。
    #[inline]
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }

    /// 移除本分发器的全部处理器，会修改 `self`。
    #[inline]
    pub fn clear(&mut self) {
        self.handlers.clear();
    }
}

/// 事件总线：按 [`EventType`] 分发事件，并提供对所有事件生效的全局处理器。
pub struct EventBus {
    emitters: HashMap<EventType, EventEmitter>,
    global_handlers: Vec<EventHandlerRegistration>,
}

impl EventBus {
    /// 新建事件总线，不注册任何分发器与全局处理器。
    #[inline]
    pub fn new() -> Self {
        Self {
            emitters: HashMap::new(),
            global_handlers: Vec::new(),
        }
    }

    /// 为指定事件类型注册常驻处理器，会修改 `self`。
    ///
    /// - `event_type`：关注的事件类别，对应的分发器不存在时自动创建；
    /// - `priority`：优先级，数值越大越先调用，负数则不触发；
    /// - `handler`：处理器闭包。
    ///
    /// 仅当事件类别匹配时该处理器才会被调用。
    #[inline]
    pub fn on<F>(&mut self, event_type: EventType, priority: i32, handler: F)
    where
        F: Fn(&mut Event) + Send + Sync + 'static,
    {
        self.emitters
            .entry(event_type)
            .or_insert_with(|| EventEmitter::new(event_type))
            .on(priority, handler);
    }

    /// 为指定事件类型注册一次性处理器，会修改 `self`。
    ///
    /// - `event_type`：关注的事件类别，对应的分发器不存在时自动创建；
    /// - `priority`：优先级，数值越大越先调用；
    /// - `handler`：处理器闭包，首次触发后自动移除。
    #[inline]
    pub fn once<F>(&mut self, event_type: EventType, priority: i32, handler: F)
    where
        F: Fn(&mut Event) + Send + Sync + 'static,
    {
        self.emitters
            .entry(event_type)
            .or_insert_with(|| EventEmitter::new(event_type))
            .once(priority, handler);
    }

    /// 注册全局处理器，会修改 `self`。
    ///
    /// - `priority`：优先级，数值越大越先调用，负数则不触发；
    /// - `handler`：处理器闭包。
    ///
    /// 全局处理器对任意类别的事件都会在类型专属处理器之后被调用。
    #[inline]
    pub fn on_global<F>(&mut self, priority: i32, handler: F)
    where
        F: Fn(&mut Event) + Send + Sync + 'static,
    {
        self.global_handlers.push(EventHandlerRegistration::new(
            Arc::new(handler),
            priority,
            false,
        ));
        self.global_handlers.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// 分发一次事件，会修改 `self` 与传入的 `event`。
    ///
    /// 先调用与该事件类别匹配的分发器，再按优先级调用全局处理器。任一环节中止传播后，
    /// 后续处理器（含全局处理器）都会被跳过；总线上没有匹配的分发器时该事件仍会送到全局处理器。
    ///
    /// - `event`：待分发的事件，其载荷与来源由处理器读取。
    #[inline]
    pub fn emit(&mut self, event: &mut Event) {
        if let Some(emitter) = self.emitters.get_mut(event.event_type()) {
            emitter.emit(event);
        }

        for registration in &mut self.global_handlers {
            if registration.priority >= 0 {
                (registration.handler)(event);
                if event.is_propagation_stopped() {
                    break;
                }
            }
        }
    }

    /// 以空载荷事件触发指定类别，会修改 `self`。
    ///
    /// - `event_type`：要触发的事件类别。
    #[inline]
    pub fn emit_event(&mut self, event_type: EventType) {
        let mut event = Event::new(event_type);
        self.emit(&mut event);
    }

    /// 触发指定类别并携带一份载荷，会修改 `self`。
    ///
    /// - `event_type`：要触发的事件类别；
    /// - `data`：载荷，处理器需用 [`Event::take_data`] 以相同类型取出。
    #[inline]
    pub fn emit_with_data<T: Any>(&mut self, event_type: EventType, data: T) {
        let mut event = Event::new(event_type).with_data(data);
        self.emit(&mut event);
    }

    /// 触发 [`EventType::SelectionChanged`]，载荷为新的选中对象列表。
    ///
    /// - `selection`：当前选中的实体标识，会被复制进事件载荷。
    #[inline]
    pub fn emit_selection_changed(&mut self, selection: &[ObjectId]) {
        let mut event = Event::new(EventType::SelectionChanged).with_data(selection.to_vec());
        self.emit(&mut event);
    }

    /// 触发 [`EventType::EntityAdded`]，来源为新增实体。
    ///
    /// - `entity_id`：被加入的实体标识。
    #[inline]
    pub fn emit_entity_added(&mut self, entity_id: &ObjectId) {
        let mut event = Event::new(EventType::EntityAdded).with_source(entity_id.clone());
        self.emit(&mut event);
    }

    /// 触发 [`EventType::EntityRemoved`]，来源为被删除实体。
    ///
    /// - `entity_id`：被移除的实体标识；实体此刻已不在文档中，处理器只能按 id 处理。
    #[inline]
    pub fn emit_entity_removed(&mut self, entity_id: &ObjectId) {
        let mut event = Event::new(EventType::EntityRemoved).with_source(entity_id.clone());
        self.emit(&mut event);
    }

    /// 触发 [`EventType::EntityModified`]，来源为被修改实体。
    ///
    /// - `entity_id`：发生变化实体的标识。
    #[inline]
    pub fn emit_entity_modified(&mut self, entity_id: &ObjectId) {
        let mut event = Event::new(EventType::EntityModified).with_source(entity_id.clone());
        self.emit(&mut event);
    }

    /// 触发 [`EventType::TransactionStarted`]，载荷为事务名。
    ///
    /// - `name`：事务名，会被复制进事件载荷，供撤销/重做界面显示。
    #[inline]
    pub fn emit_transaction_started(&mut self, name: &str) {
        let mut event = Event::new(EventType::TransactionStarted).with_data(name.to_string());
        self.emit(&mut event);
    }

    /// 触发 [`EventType::TransactionEnded`]，不带来源与载荷。
    #[inline]
    pub fn emit_transaction_ended(&mut self) {
        let mut event = Event::new(EventType::TransactionEnded);
        self.emit(&mut event);
    }

    /// 注销指定事件类型下的处理器，会修改 `self`。
    ///
    /// - `event_type`：处理器所在的事件类别，该类别无分发器时静默无操作；
    /// - `handler`：要移除的处理器，按 [`Arc`] 指针相等比较。
    #[inline]
    pub fn off(&mut self, event_type: EventType, handler: &EventHandler) {
        if let Some(emitter) = self.emitters.get_mut(&event_type) {
            emitter.off(handler);
        }
    }

    /// 注销一个全局处理器，会修改 `self`。
    ///
    /// - `handler`：要移除的处理器，须与注册时传入的是同一实例。
    #[inline]
    pub fn off_global(&mut self, handler: &EventHandler) {
        self.global_handlers.retain(|h| &h.handler != handler);
    }

    /// 清空指定事件类型的全部处理器，会修改 `self`。
    ///
    /// - `event_type`：要清理的事件类别；全局处理器不受影响。
    #[inline]
    pub fn clear_event(&mut self, event_type: EventType) {
        if let Some(emitter) = self.emitters.get_mut(&event_type) {
            emitter.clear();
        }
    }

    /// 清空所有分发器的处理器以及全部全局处理器，会修改 `self`。
    #[inline]
    pub fn clear_all(&mut self) {
        for emitter in self.emitters.values_mut() {
            emitter.clear();
        }
        self.global_handlers.clear();
    }

    /// 指定事件类型已注册的处理器个数，不含全局处理器。
    ///
    /// - `event_type`：查询的事件类别；该类别无分发器时返回 0。
    #[inline]
    pub fn handler_count(&self, event_type: EventType) -> usize {
        self.emitters.get(&event_type).map(|e| e.handler_count()).unwrap_or(0)
    }

    /// 总线上注册的处理器总数，等于各事件类型的处理器数之和加全局处理器数。
    #[inline]
    pub fn total_handler_count(&self) -> usize {
        let emitter_count: usize = self.emitters.values().map(|e| e.handler_count()).sum();
        emitter_count + self.global_handlers.len()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// 选择变化事件的载荷：记录本次新增、移除的实体以及变化后的完整选择。
#[derive(Debug, Clone)]
pub struct SelectionChangeEvent {
    /// 本次新加入选择的实体标识。
    pub added: Vec<ObjectId>,
    /// 本次从选择中移除的实体标识。
    pub removed: Vec<ObjectId>,
    /// 变化后的完整选中列表。
    pub current: Vec<ObjectId>,
}

/// 实体变化事件的载荷：记录被改动实体的 id 及改动前后的数据快照。
#[derive(Debug, Clone)]
pub struct EntityChangeEvent {
    /// 发生变化的实体标识。
    pub entity_id: ObjectId,
    /// 改动前的实体数据快照；新建实体时为 [`None`]。
    pub old_data: Option<Box<dyn Any>>,
    /// 改动后的实体数据快照；删除实体时为 [`None`]。
    pub new_data: Option<Box<dyn Any>>,
}

/// 鼠标事件载荷，坐标单位为视口像素，而非文档坐标。
#[derive(Debug, Clone)]
pub struct MouseEvent {
    /// 光标 X 坐标（视口像素）。
    pub x: f64,
    /// 光标 Y 坐标（视口像素）。
    pub y: f64,
    /// 按下的鼠标键。
    pub button: MouseButton,
    /// 同时按下的修饰键。
    pub modifiers: ModifierKeys,
    /// 是否为双击。
    pub double_click: bool,
}

/// 鼠标按键。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// 左键。
    Left,
    /// 右键。
    Right,
    /// 中键（滚轮键）。
    Middle,
    /// 无按键，用于移动类事件。
    None,
}

/// 键盘修饰键的按下状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModifierKeys {
    /// Shift 键是否按下。
    pub shift: bool,
    /// Ctrl 键是否按下。
    pub ctrl: bool,
    /// Alt 键是否按下。
    pub alt: bool,
    /// Meta（Windows 键/Command 键）是否按下。
    pub meta: bool,
}

impl Default for ModifierKeys {
    fn default() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        }
    }
}

impl ModifierKeys {
    /// 按各键的按下状态构造。
    ///
    /// - `shift`：Shift 是否按下；
    /// - `ctrl`：Ctrl 是否按下；
    /// - `alt`：Alt 是否按下；
    /// - `meta`：Meta 是否按下。
    #[inline]
    pub fn new(shift: bool, ctrl: bool, alt: bool, meta: bool) -> Self {
        Self {
            shift,
            ctrl,
            alt,
            meta,
        }
    }

    /// 四个修饰键均未按下，等价于 [`ModifierKeys::default`]。
    #[inline]
    pub fn none() -> Self {
        Self::default()
    }
}

/// 键盘事件载荷。
#[derive(Debug, Clone)]
pub struct KeyEvent {
    /// 平台相关的按键码。
    pub key_code: u32,
    /// 按键的可读名称，如 `"A"`。
    pub key: String,
    /// 同时按下的修饰键；按键本身作为修饰键时同样记录在此。
    pub modifiers: ModifierKeys,
    /// 是否为按住不放产生的重复事件。
    pub repeat: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = Event::new(EventType::SelectionChanged);
        assert_eq!(event.event_type(), &EventType::SelectionChanged);
        assert!(event.source().is_none());
        assert!(!event.is_propagation_stopped());
    }

    #[test]
    fn test_event_with_source() {
        let id = ObjectId::new();
        let event = Event::new(EventType::EntityAdded).with_source(id.clone());
        assert_eq!(event.source(), Some(&id));
    }

    #[test]
    fn test_event_with_data() {
        let event = Event::new(EventType::MouseClicked).with_data(42i32);
        assert!(event.has_data::<i32>());
        assert!(!event.has_data::<String>());
    }

    #[test]
    fn test_event_bus_creation() {
        let bus = EventBus::new();
        assert_eq!(bus.total_handler_count(), 0);
    }

    #[test]
    fn test_event_handler_registration() {
        let mut bus = EventBus::new();
        let mut call_count = 0;
        
        bus.on(EventType::SelectionChanged, 0, |_| {
            call_count += 1;
        });
        
        assert_eq!(bus.handler_count(EventType::SelectionChanged), 1);
        
        let mut event = Event::new(EventType::SelectionChanged);
        bus.emit(&mut event);
        
        assert_eq!(call_count, 1);
    }

    #[test]
    fn test_event_propagation() {
        let mut bus = EventBus::new();
        let mut call_count = 0;
        let mut propagation_stopped = false;
        
        bus.on(EventType::SelectionChanged, 0, |e| {
            call_count += 1;
            e.stop_propagation();
        });
        
        bus.on(EventType::SelectionChanged, 0, |_| {
            call_count += 1;
        });
        
        let mut event = Event::new(EventType::SelectionChanged);
        bus.emit(&mut event);
        
        assert_eq!(call_count, 1);
    }

    #[test]
    fn test_global_event_handlers() {
        let mut bus = EventBus::new();
        let mut global_call_count = 0;
        let mut specific_call_count = 0;
        
        bus.on_global(0, |_| {
            global_call_count += 1;
        });
        
        bus.on(EventType::SelectionChanged, 0, |_| {
            specific_call_count += 1;
        });
        
        let mut event1 = Event::new(EventType::SelectionChanged);
        bus.emit(&mut event1);
        
        let mut event2 = Event::new(EventType::EntityAdded);
        bus.emit(&mut event2);
        
        assert_eq!(specific_call_count, 1);
        assert_eq!(global_call_count, 2);
    }

    #[test]
    fn test_once_handler() {
        let mut bus = EventBus::new();
        let mut call_count = 0;
        
        bus.once(EventType::SelectionChanged, 0, |_| {
            call_count += 1;
        });
        
        assert_eq!(bus.handler_count(EventType::SelectionChanged), 1);
        
        let mut event = Event::new(EventType::SelectionChanged);
        bus.emit(&mut event);
        
        assert_eq!(call_count, 1);
        assert_eq!(bus.handler_count(EventType::SelectionChanged), 0);
    }

    #[test]
    fn test_event_clear() {
        let mut bus = EventBus::new();
        
        bus.on(EventType::SelectionChanged, 0, |_| {});
        bus.on(EventType::EntityAdded, 0, |_| {});
        bus.on_global(0, |_| {});
        
        assert_eq!(bus.total_handler_count(), 3);
        
        bus.clear_all();
        
        assert_eq!(bus.total_handler_count(), 0);
    }

    #[test]
    fn test_selection_change_event() {
        let added = vec![ObjectId::new(), ObjectId::new()];
        let removed = vec![ObjectId::new()];
        let current = vec![added[0].clone()];
        
        let event = SelectionChangeEvent {
            added: added.clone(),
            removed: removed.clone(),
            current: current.clone(),
        };
        
        assert_eq!(event.added.len(), 2);
        assert_eq!(event.removed.len(), 1);
        assert_eq!(event.current.len(), 1);
    }

    #[test]
    fn test_mouse_event() {
        let mouse_event = MouseEvent {
            x: 100.5,
            y: 200.75,
            button: MouseButton::Left,
            modifiers: ModifierKeys::new(true, false, false, false),
            double_click: false,
        };
        
        assert_eq!(mouse_event.x, 100.5);
        assert_eq!(mouse_event.button, MouseButton::Left);
        assert!(mouse_event.modifiers.shift);
        assert!(!mouse_event.modifiers.ctrl);
    }

    #[test]
    fn test_key_event() {
        let key_event = KeyEvent {
            key_code: 65,
            key: "A".to_string(),
            modifiers: ModifierKeys::none(),
            repeat: false,
        };
        
        assert_eq!(key_event.key_code, 65);
        assert_eq!(key_event.key, "A");
        assert!(!key_event.repeat);
    }
}
