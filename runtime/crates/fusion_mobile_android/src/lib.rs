use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// JNI reference handle (opaque pointer abstraction)
// ---------------------------------------------------------------------------

/// An opaque handle representing a JNI local/global reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JniHandle(pub u64);

impl JniHandle {
    pub const NULL: JniHandle = JniHandle(0);
}

// ---------------------------------------------------------------------------
// Activity lifecycle
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActivityState {
    Created,
    Started,
    Resumed,
    Paused,
    Stopped,
    Destroyed,
}

impl fmt::Display for ActivityState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActivityState::Created => write!(f, "Created"),
            ActivityState::Started => write!(f, "Started"),
            ActivityState::Resumed => write!(f, "Resumed"),
            ActivityState::Paused => write!(f, "Paused"),
            ActivityState::Stopped => write!(f, "Stopped"),
            ActivityState::Destroyed => write!(f, "Destroyed"),
        }
    }
}

/// Valid lifecycle transitions (mirrors Android official diagram).
fn is_valid_transition(from: &ActivityState, to: &ActivityState) -> bool {
    matches!(
        (from, to),
        (ActivityState::Created, ActivityState::Started)
            | (ActivityState::Started, ActivityState::Resumed)
            | (ActivityState::Resumed, ActivityState::Paused)
            | (ActivityState::Paused, ActivityState::Resumed)
            | (ActivityState::Paused, ActivityState::Stopped)
            | (ActivityState::Stopped, ActivityState::Started)
            | (ActivityState::Paused, ActivityState::Destroyed)
            | (ActivityState::Stopped, ActivityState::Destroyed)
    )
}

pub struct ActivityLifecycleManager {
    state: ActivityState,
    history: Vec<(ActivityState, u64)>,
    activity_id: String,
}

impl ActivityLifecycleManager {
    pub fn new(activity_id: &str) -> Self {
        Self {
            state: ActivityState::Created,
            history: Vec::new(),
            activity_id: activity_id.to_string(),
        }
    }

    pub fn current_state(&self) -> &ActivityState {
        &self.state
    }

    pub fn activity_id(&self) -> &str {
        &self.activity_id
    }

    pub fn transition_to(
        &mut self,
        new_state: ActivityState,
        timestamp_ms: u64,
    ) -> Result<(), LifecycleError> {
        if !is_valid_transition(&self.state, &new_state) {
            return Err(LifecycleError::InvalidTransition {
                from: self.state.clone(),
                to: new_state,
            });
        }
        self.history.push((self.state.clone(), timestamp_ms));
        self.state = new_state;
        Ok(())
    }

    pub fn is_alive(&self) -> bool {
        !matches!(
            self.state,
            ActivityState::Destroyed
        )
    }

    pub fn is_visible(&self) -> bool {
        matches!(
            self.state,
            ActivityState::Started | ActivityState::Resumed
        )
    }

    pub fn history(&self) -> &[(ActivityState, u64)] {
        &self.history
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LifecycleError {
    InvalidTransition { from: ActivityState, to: ActivityState },
}

impl fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LifecycleError::InvalidTransition { from, to } => {
                write!(f, "Invalid lifecycle transition: {} -> {}", from, to)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Touch events (MotionEvent abstraction)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MotionEvent {
    pub action: MotionEventAction,
    pub pointer_id: i32,
    pub x: f32,
    pub y: f32,
    pub pressure: f32,
    pub size: f32,
    pub event_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MotionEventAction {
    Down,
    Move,
    Up,
    Cancel,
    PointerDown(i32),
    PointerUp(i32),
}

// ---------------------------------------------------------------------------
// UI component abstractions (View / ViewGroup)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ViewBounds {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl ViewBounds {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AndroidColor(pub u32);

impl AndroidColor {
    pub fn argb(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self(((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::argb(255, r, g, b)
    }

    pub fn transparent() -> Self {
        Self(0)
    }

    pub fn red(&self) -> u8 {
        ((self.0 >> 16) & 0xFF) as u8
    }

    pub fn green(&self) -> u8 {
        ((self.0 >> 8) & 0xFF) as u8
    }

    pub fn blue(&self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    pub fn alpha(&self) -> u8 {
        ((self.0 >> 24) & 0xFF) as u8
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AndroidView {
    pub handle: JniHandle,
    pub class_name: String,
    pub id: String,
    pub bounds: ViewBounds,
    pub background_color: AndroidColor,
    pub visible: bool,
    pub enabled: bool,
    pub children: Vec<AndroidView>,
    pub layoutParams: HashMap<String, String>,
}

/// Layout parameters that mirror Android's LayoutParams concepts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LayoutWidth {
    MatchParent,
    WrapContent,
    Exact(i32),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LayoutHeight {
    MatchParent,
    WrapContent,
    Exact(i32),
}

pub struct ViewGroupManager {
    activities: HashMap<String, ActivityLifecycleManager>,
    views: HashMap<String, AndroidView>,
    next_handle: u64,
}

impl ViewGroupManager {
    pub fn new() -> Self {
        Self {
            activities: HashMap::new(),
            views: HashMap::new(),
            next_handle: 1,
        }
    }

    pub fn create_activity(&mut self, activity_id: &str) -> &mut ActivityLifecycleManager {
        self.activities
            .entry(activity_id.to_string())
            .or_insert_with(|| ActivityLifecycleManager::new(activity_id))
    }

    pub fn get_activity(&self, activity_id: &str) -> Option<&ActivityLifecycleManager> {
        self.activities.get(activity_id)
    }

    pub fn next_handle(&mut self) -> JniHandle {
        let h = JniHandle(self.next_handle);
        self.next_handle += 1;
        h
    }

    pub fn register_view(&mut self, view: AndroidView) {
        self.views.insert(view.id.clone(), view);
    }

    pub fn get_view(&self, id: &str) -> Option<&AndroidView> {
        self.views.get(id)
    }

    pub fn remove_view(&mut self, id: &str) -> Option<AndroidView> {
        self.views.remove(id)
    }

    pub fn view_count(&self) -> usize {
        self.views.len()
    }

    /// Create a standard TextView.
    pub fn make_text_view(id: &str, text: &str, bounds: ViewBounds) -> AndroidView {
        let mut props = HashMap::new();
        props.insert("text".into(), text.to_string());
        props.insert("textSize".into(), "16sp".into());

        AndroidView {
            handle: JniHandle(0),
            class_name: "android.widget.TextView".into(),
            id: id.into(),
            bounds,
            background_color: AndroidColor::transparent(),
            visible: true,
            enabled: true,
            children: vec![],
            layoutParams: props,
        }
    }

    /// Create a standard Button.
    pub fn make_button(id: &str, text: &str, bounds: ViewBounds) -> AndroidView {
        let mut props = HashMap::new();
        props.insert("text".into(), text.to_string());

        AndroidView {
            handle: JniHandle(0),
            class_name: "android.widget.Button".into(),
            id: id.into(),
            bounds,
            background_color: AndroidColor::rgb(33, 150, 243),
            visible: true,
            enabled: true,
            children: vec![],
            layoutParams: props,
        }
    }

    /// Create a LinearLayout container.
    pub fn make_linear_layout(id: &str, bounds: ViewBounds) -> AndroidView {
        let mut props = HashMap::new();
        props.insert("orientation".into(), "vertical".into());

        AndroidView {
            handle: JniHandle(0),
            class_name: "android.widget.LinearLayout".into(),
            id: id.into(),
            bounds,
            background_color: AndroidColor::transparent(),
            visible: true,
            enabled: true,
            children: vec![],
            layoutParams: props,
        }
    }
}

impl Default for ViewGroupManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Android runtime bridge
// ---------------------------------------------------------------------------

pub struct AndroidBridge {
    manager: ViewGroupManager,
    pending_events: Vec<MotionEvent>,
}

impl AndroidBridge {
    pub fn new() -> Self {
        Self {
            manager: ViewGroupManager::new(),
            pending_events: Vec::new(),
        }
    }

    pub fn manager(&self) -> &ViewGroupManager {
        &self.manager
    }

    pub fn manager_mut(&mut self) -> &mut ViewGroupManager {
        &mut self.manager
    }

    pub fn push_motion_event(&mut self, event: MotionEvent) {
        self.pending_events.push(event);
    }

    pub fn drain_events(&mut self) -> Vec<MotionEvent> {
        std::mem::take(&mut self.pending_events)
    }
}

impl Default for AndroidBridge {
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
    fn activity_lifecycle_happy_path() {
        let mut mgr = ActivityLifecycleManager::new("MainAct");
        assert_eq!(mgr.current_state(), &ActivityState::Created);

        mgr.transition_to(ActivityState::Started, 0).unwrap();
        assert!(mgr.is_visible());

        mgr.transition_to(ActivityState::Resumed, 100).unwrap();
        assert!(mgr.is_alive());
        assert!(mgr.is_visible());

        mgr.transition_to(ActivityState::Paused, 200).unwrap();
        assert!(!mgr.is_visible());

        mgr.transition_to(ActivityState::Stopped, 300).unwrap();
        mgr.transition_to(ActivityState::Destroyed, 400).unwrap();
        assert!(!mgr.is_alive());
        assert_eq!(mgr.history().len(), 5);
    }

    #[test]
    fn activity_invalid_transition() {
        let mut mgr = ActivityLifecycleManager::new("Act");
        let err = mgr.transition_to(ActivityState::Resumed, 0);
        assert!(err.is_err());
        assert_eq!(
            err.unwrap_err(),
            LifecycleError::InvalidTransition {
                from: ActivityState::Created,
                to: ActivityState::Resumed,
            }
        );
    }

    #[test]
    fn view_bounds_dimensions() {
        let bounds = ViewBounds { left: 10, top: 20, right: 110, bottom: 320 };
        assert_eq!(bounds.width(), 100);
        assert_eq!(bounds.height(), 300);
    }

    #[test]
    fn color_argb() {
        let c = AndroidColor::argb(255, 128, 64, 32);
        assert_eq!(c.alpha(), 255);
        assert_eq!(c.red(), 128);
        assert_eq!(c.green(), 64);
        assert_eq!(c.blue(), 32);
    }

    #[test]
    fn view_group_manager_crud() {
        let mut mgr = ViewGroupManager::new();
        let tv = ViewGroupManager::make_text_view("tv1", "Hello", ViewBounds {
            left: 0, top: 0, right: 200, bottom: 50,
        });
        mgr.register_view(tv);

        assert_eq!(mgr.view_count(), 1);
        assert!(mgr.get_view("tv1").is_some());
        mgr.remove_view("tv1");
        assert_eq!(mgr.view_count(), 0);
    }

    #[test]
    fn make_button_has_correct_class() {
        let btn = ViewGroupManager::make_button(
            "btn1",
            "OK",
            ViewBounds { left: 0, top: 0, right: 100, bottom: 44 },
        );
        assert_eq!(btn.class_name, "android.widget.Button");
        assert_eq!(btn.layoutParams.get("text").unwrap(), "OK");
    }

    #[test]
    fn make_linear_layout_orientation() {
        let ll = ViewGroupManager::make_linear_layout(
            "root",
            ViewBounds { left: 0, top: 0, right: 400, bottom: 800 },
        );
        assert_eq!(ll.layoutParams.get("orientation").unwrap(), "vertical");
        assert!(ll.children.is_empty());
    }

    #[test]
    fn bridge_event_queue() {
        let mut bridge = AndroidBridge::new();
        bridge.push_motion_event(MotionEvent {
            action: MotionEventAction::Down,
            pointer_id: 0,
            x: 50.0,
            y: 100.0,
            pressure: 1.0,
            size: 1.0,
            event_time_ms: 0,
        });
        assert_eq!(bridge.drain_events().len(), 1);
        assert!(bridge.drain_events().is_empty());
    }

    #[test]
    fn handle_null() {
        assert_eq!(JniHandle::NULL, JniHandle(0));
    }

    #[test]
    fn serialization_roundtrip() {
        let event = MotionEvent {
            action: MotionEventAction::Move,
            pointer_id: 0,
            x: 10.0,
            y: 20.0,
            pressure: 0.5,
            size: 1.0,
            event_time_ms: 100,
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: MotionEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, parsed);
    }

    #[test]
    fn activity_resume_after_pause() {
        let mut mgr = ActivityLifecycleManager::new("Act");
        mgr.transition_to(ActivityState::Started, 0).unwrap();
        mgr.transition_to(ActivityState::Resumed, 1).unwrap();
        mgr.transition_to(ActivityState::Paused, 2).unwrap();
        // Can resume from paused
        mgr.transition_to(ActivityState::Resumed, 3).unwrap();
        assert!(mgr.is_visible());
    }

    #[test]
    fn activity_restart_from_stopped() {
        let mut mgr = ActivityLifecycleManager::new("Act");
        mgr.transition_to(ActivityState::Started, 0).unwrap();
        mgr.transition_to(ActivityState::Resumed, 1).unwrap();
        mgr.transition_to(ActivityState::Paused, 2).unwrap();
        mgr.transition_to(ActivityState::Stopped, 3).unwrap();
        // Restart
        mgr.transition_to(ActivityState::Started, 4).unwrap();
        assert!(mgr.is_visible());
    }
}
