use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// Touch / input events
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TouchPoint {
    pub x: f64,
    pub y: f64,
    pub pressure: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TouchEvent {
    pub event_type: TouchEventType,
    pub touches: Vec<TouchPoint>,
    pub timestamp: f64,
    pub view_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TouchEventType {
    Began,
    Moved,
    Ended,
    Cancelled,
}

// ---------------------------------------------------------------------------
// UI components (abstractions over UIKit concepts)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiFrame {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl UiColor {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }

    pub fn black() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }

    pub fn clear() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiFont {
    pub name: String,
    pub size: f32,
}

impl UiFont {
    pub fn system(size: f32) -> Self {
        Self {
            name: ".SFUIText".into(),
            size,
        }
    }

    pub fn bold(size: f32) -> Self {
        Self {
            name: ".SFUIText-Bold".into(),
            size,
        }
    }
}

/// Generic UI component representation that maps to UIView subclasses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiComponent {
    pub id: String,
    pub component_type: UiComponentType,
    pub frame: UiFrame,
    pub background_color: UiColor,
    pub hidden: bool,
    pub alpha: f32,
    pub children: Vec<UiComponent>,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UiComponentType {
    View,
    Label,
    Button,
    TextField,
    Image,
    StackView,
    ScrollView,
    TableView,
    Custom(String),
}

// ---------------------------------------------------------------------------
// View hierarchy builder
// ---------------------------------------------------------------------------

pub struct ViewHierarchy {
    root: Option<UiComponent>,
    lookup: HashMap<String, usize>,
}

impl ViewHierarchy {
    pub fn new() -> Self {
        Self {
            root: None,
            lookup: HashMap::new(),
        }
    }

    pub fn set_root(&mut self, component: UiComponent) {
        self.index_component(&component, 0);
        self.root = Some(component);
    }

    pub fn get_root(&self) -> Option<&UiComponent> {
        self.root.as_ref()
    }

    pub fn find_by_id(&self, id: &str) -> Option<&UiComponent> {
        self.root.as_ref().and_then(|r| Self::find_recursive(r, id))
    }

    fn find_recursive<'a>(component: &'a UiComponent, id: &str) -> Option<&'a UiComponent> {
        if component.id == id {
            return Some(component);
        }
        for child in &component.children {
            if let Some(found) = Self::find_recursive(child, id) {
                return Some(found);
            }
        }
        None
    }

    fn index_component(&mut self, comp: &UiComponent, depth: usize) {
        self.lookup.insert(comp.id.clone(), depth);
        for child in &comp.children {
            self.index_component(child, depth + 1);
        }
    }

    pub fn count(&self) -> usize {
        self.root.as_ref().map_or(0, Self::count_recursive)
    }

    fn count_recursive(comp: &UiComponent) -> usize {
        1 + comp.children.iter().map(Self::count_recursive).sum::<usize>()
    }
}

impl Default for ViewHierarchy {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Activity lifecycle (UIApplicationDelegate mapping)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum AppLifecycleState {
    /// App not yet launched.
    NotRunning,
    /// App is in the foreground and receiving events.
    Active,
    /// App is in the foreground but not receiving events (e.g. phone call).
    Inactive,
    /// App is in the background.
    Background,
    /// App is suspended and may be terminated.
    Suspended,
}

impl fmt::Display for AppLifecycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppLifecycleState::NotRunning => write!(f, "NotRunning"),
            AppLifecycleState::Active => write!(f, "Active"),
            AppLifecycleState::Inactive => write!(f, "Inactive"),
            AppLifecycleState::Background => write!(f, "Background"),
            AppLifecycleState::Suspended => write!(f, "Suspended"),
        }
    }
}

pub struct IosLifecycleManager {
    state: AppLifecycleState,
    state_history: Vec<(AppLifecycleState, f64)>,
}

impl IosLifecycleManager {
    pub fn new() -> Self {
        Self {
            state: AppLifecycleState::NotRunning,
            state_history: Vec::new(),
        }
    }

    pub fn current_state(&self) -> &AppLifecycleState {
        &self.state
    }

    pub fn transition_to(&mut self, new_state: AppLifecycleState, timestamp: f64) {
        self.state_history
            .push((self.state.clone(), timestamp));
        self.state = new_state;
    }

    pub fn is_active(&self) -> bool {
        self.state == AppLifecycleState::Active
    }

    pub fn is_background(&self) -> bool {
        self.state == AppLifecycleState::Background
            || self.state == AppLifecycleState::Suspended
    }

    pub fn state_history(&self) -> &[(AppLifecycleState, f64)] {
        &self.state_history
    }
}

impl Default for IosLifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Bridge: thin FFI-safe representation of the iOS runtime
// ---------------------------------------------------------------------------

pub struct IosBridge {
    lifecycle: IosLifecycleManager,
    view_hierarchy: ViewHierarchy,
    pending_events: Vec<TouchEvent>,
}

impl IosBridge {
    pub fn new() -> Self {
        Self {
            lifecycle: IosLifecycleManager::new(),
            view_hierarchy: ViewHierarchy::new(),
            pending_events: Vec::new(),
        }
    }

    pub fn lifecycle(&self) -> &IosLifecycleManager {
        &self.lifecycle
    }

    pub fn lifecycle_mut(&mut self) -> &mut IosLifecycleManager {
        &mut self.lifecycle
    }

    pub fn view_hierarchy(&self) -> &ViewHierarchy {
        &self.view_hierarchy
    }

    pub fn view_hierarchy_mut(&mut self) -> &mut ViewHierarchy {
        &mut self.view_hierarchy
    }

    pub fn push_touch_event(&mut self, event: TouchEvent) {
        self.pending_events.push(event);
    }

    pub fn drain_events(&mut self) -> Vec<TouchEvent> {
        std::mem::take(&mut self.pending_events)
    }

    /// Create a label component (maps to UILabel).
    pub fn make_label(id: &str, text: &str, frame: UiFrame) -> UiComponent {
        let mut props = HashMap::new();
        props.insert("text".into(), text.to_string());
        props.insert("font".into(), UiFont::system(17.0).name);

        UiComponent {
            id: id.into(),
            component_type: UiComponentType::Label,
            frame,
            background_color: UiColor::clear(),
            hidden: false,
            alpha: 1.0,
            children: vec![],
            properties: props,
        }
    }

    /// Create a button component (maps to UIButton).
    pub fn make_button(id: &str, title: &str, frame: UiFrame) -> UiComponent {
        let mut props = HashMap::new();
        props.insert("title".into(), title.to_string());

        UiComponent {
            id: id.into(),
            component_type: UiComponentType::Button,
            frame,
            background_color: UiColor::new(0.0, 0.478, 1.0, 1.0),
            hidden: false,
            alpha: 1.0,
            children: vec![],
            properties: props,
        }
    }
}

impl Default for IosBridge {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touch_event_creation() {
        let event = TouchEvent {
            event_type: TouchEventType::Began,
            touches: vec![TouchPoint {
                x: 100.0,
                y: 200.0,
                pressure: 1.0,
            }],
            timestamp: 0.0,
            view_id: "main".into(),
        };
        assert_eq!(event.touches.len(), 1);
        assert_eq!(event.event_type, TouchEventType::Began);
    }

    #[test]
    fn lifecycle_transitions() {
        let mut mgr = IosLifecycleManager::new();
        assert_eq!(mgr.current_state(), &AppLifecycleState::NotRunning);

        mgr.transition_to(AppLifecycleState::Active, 0.0);
        assert!(mgr.is_active());
        assert!(!mgr.is_background());

        mgr.transition_to(AppLifecycleState::Background, 1.0);
        assert!(mgr.is_background());
        assert_eq!(mgr.state_history().len(), 2);
    }

    #[test]
    fn view_hierarchy_find() {
        let mut vh = ViewHierarchy::new();
        let child = UiComponent {
            id: "child-1".into(),
            component_type: UiComponentType::Label,
            frame: UiFrame { x: 0.0, y: 0.0, width: 100.0, height: 30.0 },
            background_color: UiColor::clear(),
            hidden: false,
            alpha: 1.0,
            children: vec![],
            properties: HashMap::new(),
        };
        let root = UiComponent {
            id: "root".into(),
            component_type: UiComponentType::View,
            frame: UiFrame { x: 0.0, y: 0.0, width: 400.0, height: 800.0 },
            background_color: UiColor::white(),
            hidden: false,
            alpha: 1.0,
            children: vec![child],
            properties: HashMap::new(),
        };

        vh.set_root(root);
        assert_eq!(vh.count(), 2);
        assert!(vh.find_by_id("child-1").is_some());
        assert!(vh.find_by_id("nonexistent").is_none());
    }

    #[test]
    fn bridge_make_label() {
        let label = IosBridge::make_label(
            "title",
            "Hello",
            UiFrame { x: 0.0, y: 0.0, width: 200.0, height: 44.0 },
        );
        assert_eq!(label.component_type, UiComponentType::Label);
        assert_eq!(label.properties.get("text").unwrap(), "Hello");
    }

    #[test]
    fn bridge_make_button() {
        let btn = IosBridge::make_button(
            "submit",
            "Submit",
            UiFrame { x: 50.0, y: 100.0, width: 200.0, height: 44.0 },
        );
        assert_eq!(btn.component_type, UiComponentType::Button);
        assert_eq!(btn.properties.get("title").unwrap(), "Submit");
    }

    #[test]
    fn bridge_event_queue() {
        let mut bridge = IosBridge::new();
        let event = TouchEvent {
            event_type: TouchEventType::Began,
            touches: vec![],
            timestamp: 0.0,
            view_id: "v".into(),
        };
        bridge.push_touch_event(event);
        assert_eq!(bridge.drain_events().len(), 1);
        assert_eq!(bridge.drain_events().len(), 0);
    }

    #[test]
    fn color_constructors() {
        assert_eq!(UiColor::white(), UiColor::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(UiColor::black(), UiColor::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(UiColor::clear(), UiColor::new(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn serialization_roundtrip() {
        let comp = UiComponent {
            id: "test".into(),
            component_type: UiComponentType::View,
            frame: UiFrame { x: 0.0, y: 0.0, width: 100.0, height: 100.0 },
            background_color: UiColor::white(),
            hidden: false,
            alpha: 1.0,
            children: vec![],
            properties: HashMap::new(),
        };
        let json = serde_json::to_string(&comp).unwrap();
        let parsed: UiComponent = serde_json::from_str(&json).unwrap();
        assert_eq!(comp, parsed);
    }
}
