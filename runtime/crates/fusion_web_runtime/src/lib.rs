use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// DOM abstractions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DomElement {
    pub tag: String,
    pub id: Option<String>,
    pub class_list: Vec<String>,
    pub attributes: HashMap<String, String>,
    pub style: CssStyle,
    pub children: Vec<DomNode>,
    pub event_listeners: Vec<EventListener>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DomNode {
    Element(DomElement),
    Text(String),
    Comment(String),
}

impl DomNode {
    pub fn text(content: &str) -> Self {
        DomNode::Text(content.to_string())
    }

    pub fn comment(content: &str) -> Self {
        DomNode::Comment(content.to_string())
    }
}

// ---------------------------------------------------------------------------
// CSS style abstraction
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CssStyle {
    pub properties: HashMap<String, String>,
}

impl CssStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, property: &str, value: &str) -> &mut Self {
        self.properties.insert(property.to_string(), value.to_string());
        self
    }

    pub fn get(&self, property: &str) -> Option<&str> {
        self.properties.get(property).map(|s| s.as_str())
    }

    pub fn to_css_text(&self) -> String {
        self.properties
            .iter()
            .map(|(k, v)| format!("{}: {};", k, v))
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn display(&mut self, value: &str) -> &mut Self {
        self.set("display", value)
    }

    pub fn position(&mut self, value: &str) -> &mut Self {
        self.set("position", value)
    }

    pub fn width(&mut self, value: &str) -> &mut Self {
        self.set("width", value)
    }

    pub fn height(&mut self, value: &str) -> &mut Self {
        self.set("height", value)
    }

    pub fn background_color(&mut self, value: &str) -> &mut Self {
        self.set("background-color", value)
    }

    pub fn color(&mut self, value: &str) -> &mut Self {
        self.set("color", value)
    }

    pub fn font_size(&mut self, value: &str) -> &mut Self {
        self.set("font-size", value)
    }

    pub fn margin(&mut self, value: &str) -> &mut Self {
        self.set("margin", value)
    }

    pub fn padding(&mut self, value: &str) -> &mut Self {
        self.set("padding", value)
    }
}

// ---------------------------------------------------------------------------
// Event system
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventListener {
    pub event_type: String,
    pub handler_id: String,
    pub capture: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DomEvent {
    pub event_type: String,
    pub target_id: Option<String>,
    pub bubbles: bool,
    pub cancelable: bool,
    pub default_prevented: bool,
    pub timestamp_ms: f64,
    pub detail: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyboardEvent {
    pub key: String,
    pub code: String,
    pub alt_key: bool,
    pub ctrl_key: bool,
    pub meta_key: bool,
    pub shift_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MouseEvent {
    pub client_x: f64,
    pub client_y: f64,
    pub page_x: f64,
    pub page_y: f64,
    pub button: i32,
    pub buttons: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScrollEvent {
    pub scroll_x: f64,
    pub scroll_y: f64,
}

// ---------------------------------------------------------------------------
// DOM tree / virtual DOM
// ---------------------------------------------------------------------------

pub struct DomTree {
    root: DomNode,
    element_index: HashMap<String, usize>,
}

impl DomTree {
    pub fn new(root: DomNode) -> Self {
        let mut tree = Self {
            root,
            element_index: HashMap::new(),
        };
        tree.rebuild_index();
        tree
    }

    pub fn root(&self) -> &DomNode {
        &self.root
    }

    pub fn root_mut(&mut self) -> &mut DomNode {
        &mut self.root
    }

    pub fn find_element_by_id(&self, id: &str) -> Option<&DomNode> {
        Self::find_recursive(&self.root, id)
    }

    pub fn element_count(&self) -> usize {
        Self::count_elements(&self.root)
    }

    fn find_recursive<'a>(node: &'a DomNode, id: &str) -> Option<&'a DomNode> {
        match node {
            DomNode::Element(el) => {
                if el.id.as_deref() == Some(id) {
                    return Some(node);
                }
                for child in &el.children {
                    if let Some(found) = Self::find_recursive(child, id) {
                        return Some(found);
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn count_elements(node: &DomNode) -> usize {
        match node {
            DomNode::Element(el) => 1 + el.children.iter().map(Self::count_elements).sum::<usize>(),
            _ => 0,
        }
    }

    fn rebuild_index(&mut self) {
        self.element_index.clear();
        Self::index_recursive(&self.root, &mut self.element_index, &mut 0);
    }

    fn index_recursive(node: &DomNode, index: &mut HashMap<String, usize>, counter: &mut usize) {
        match node {
            DomNode::Element(el) => {
                if let Some(ref id) = el.id {
                    index.insert(id.clone(), *counter);
                }
                *counter += 1;
                for child in &el.children {
                    Self::index_recursive(child, index, counter);
                }
            }
            _ => {}
        }
    }

    /// Diff two DOM trees and produce a list of patches.
    pub fn diff(old: &DomNode, new: &DomNode) -> Vec<DomPatch> {
        let mut patches = Vec::new();
        Self::diff_recursive(old, new, "", &mut patches);
        patches
    }

    fn diff_recursive(old: &DomNode, new: &DomNode, path: &str, patches: &mut Vec<DomPatch>) {
        match (old, new) {
            (DomNode::Element(old_el), DomNode::Element(new_el)) => {
                if old_el.tag != new_el.tag {
                    patches.push(DomPatch::Replace {
                        path: path.to_string(),
                        new_node: new.clone(),
                    });
                    return;
                }
                if old_el.attributes != new_el.attributes {
                    patches.push(DomPatch::UpdateAttributes {
                        path: path.to_string(),
                        attributes: new_el.attributes.clone(),
                    });
                }
                if old_el.style != new_el.style {
                    patches.push(DomPatch::UpdateStyle {
                        path: path.to_string(),
                        style: new_el.style.clone(),
                    });
                }
                // Diff children
                let max = old_el.children.len().max(new_el.children.len());
                for i in 0..max {
                    let child_path = format!("{}/{}", path, i);
                    match (old_el.children.get(i), new_el.children.get(i)) {
                        (Some(old_child), Some(new_child)) => {
                            Self::diff_recursive(old_child, new_child, &child_path, patches);
                        }
                        (None, Some(new_child)) => {
                            patches.push(DomPatch::Insert {
                                path: child_path,
                                node: new_child.clone(),
                            });
                        }
                        (Some(_), None) => {
                            patches.push(DomPatch::Remove { path: child_path });
                        }
                        (None, None) => {}
                    }
                }
            }
            (DomNode::Text(a), DomNode::Text(b)) => {
                if a != b {
                    patches.push(DomPatch::ReplaceText {
                        path: path.to_string(),
                        text: b.clone(),
                    });
                }
            }
            _ => {
                if old != new {
                    patches.push(DomPatch::Replace {
                        path: path.to_string(),
                        new_node: new.clone(),
                    });
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DomPatch {
    Replace { path: String, new_node: DomNode },
    Insert { path: String, node: DomNode },
    Remove { path: String },
    ReplaceText { path: String, text: String },
    UpdateAttributes { path: String, attributes: HashMap<String, String> },
    UpdateStyle { path: String, style: CssStyle },
}

// ---------------------------------------------------------------------------
// WebSocket abstraction
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WebSocketState {
    Connecting,
    Open,
    Closing,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WebSocketMessage {
    pub data: Vec<u8>,
    pub is_binary: bool,
}

impl WebSocketMessage {
    pub fn text(payload: &str) -> Self {
        Self {
            data: payload.as_bytes().to_vec(),
            is_binary: false,
        }
    }

    pub fn binary(payload: Vec<u8>) -> Self {
        Self {
            data: payload,
            is_binary: true,
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        if !self.is_binary {
            std::str::from_utf8(&self.data).ok()
        } else {
            None
        }
    }
}

pub struct WebSocketConnection {
    url: String,
    state: WebSocketState,
    send_buffer: Vec<WebSocketMessage>,
    receive_buffer: Vec<WebSocketMessage>,
    on_open: Option<Box<dyn Fn() + Send + Sync>>,
    on_message: Option<Box<dyn Fn(&WebSocketMessage) + Send + Sync>>,
    on_close: Option<Box<dyn Fn(u16, &str) + Send + Sync>>,
    on_error: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

impl WebSocketConnection {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            state: WebSocketState::Connecting,
            send_buffer: Vec::new(),
            receive_buffer: Vec::new(),
            on_open: None,
            on_message: None,
            on_close: None,
            on_error: None,
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn state(&self) -> &WebSocketState {
        &self.state
    }

    pub fn open(&mut self) {
        self.state = WebSocketState::Open;
        if let Some(ref cb) = self.on_open {
            cb();
        }
    }

    pub fn send(&mut self, msg: WebSocketMessage) {
        if self.state == WebSocketState::Open {
            self.send_buffer.push(msg);
        }
    }

    pub fn send_text(&mut self, text: &str) {
        self.send(WebSocketMessage::text(text));
    }

    pub fn enqueue_receive(&mut self, msg: WebSocketMessage) {
        self.receive_buffer.push(msg);
    }

    pub fn process_receive(&mut self) {
        let messages: Vec<WebSocketMessage> = self.receive_buffer.drain(..).collect();
        if let Some(ref cb) = self.on_message {
            for msg in &messages {
                cb(msg);
            }
        }
    }

    pub fn close(&mut self, code: u16, reason: &str) {
        self.state = WebSocketState::Closed;
        if let Some(ref cb) = self.on_close {
            cb(code, reason);
        }
    }

    pub fn set_on_open<F: Fn() + Send + Sync + 'static>(&mut self, cb: F) {
        self.on_open = Some(Box::new(cb));
    }

    pub fn set_on_message<F: Fn(&WebSocketMessage) + Send + Sync + 'static>(&mut self, cb: F) {
        self.on_message = Some(Box::new(cb));
    }

    pub fn set_on_close<F: Fn(u16, &str) + Send + Sync + 'static>(&mut self, cb: F) {
        self.on_close = Some(Box::new(cb));
    }

    pub fn set_on_error<F: Fn(&str) + Send + Sync + 'static>(&mut self, cb: F) {
        self.on_error = Some(Box::new(cb));
    }

    pub fn buffered_amount(&self) -> usize {
        self.send_buffer.len()
    }
}

// ---------------------------------------------------------------------------
// DOM builder helpers
// ---------------------------------------------------------------------------

pub struct DomBuilder {
    elements: Vec<DomElement>,
}

impl DomBuilder {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    pub fn element(tag: &str) -> DomElementBuilder {
        DomBuilder::element_with_id(tag, "")
    }

    pub fn element_with_id(tag: &str, id: &str) -> DomElementBuilder {
        DomElementBuilder {
            element: DomElement {
                tag: tag.to_string(),
                id: if id.is_empty() { None } else { Some(id.to_string()) },
                class_list: Vec::new(),
                attributes: HashMap::new(),
                style: CssStyle::new(),
                children: Vec::new(),
                event_listeners: Vec::new(),
            },
        }
    }

    pub fn text_node(content: &str) -> DomNode {
        DomNode::Text(content.to_string())
    }
}

impl Default for DomBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DomElementBuilder {
    element: DomElement,
}

impl DomElementBuilder {
    pub fn id(mut self, id: &str) -> Self {
        self.element.id = Some(id.to_string());
        self
    }

    pub fn class(mut self, class: &str) -> Self {
        self.element.class_list.push(class.to_string());
        self
    }

    pub fn attr(mut self, key: &str, value: &str) -> Self {
        self.element.attributes.insert(key.to_string(), value.to_string());
        self
    }

    pub fn child(mut self, node: DomNode) -> Self {
        self.element.children.push(node);
        self
    }

    pub fn text_child(mut self, text: &str) -> Self {
        self.element.children.push(DomNode::Text(text.to_string()));
        self
    }

    pub fn on_event(mut self, event_type: &str, handler_id: &str) -> Self {
        self.element.event_listeners.push(EventListener {
            event_type: event_type.to_string(),
            handler_id: handler_id.to_string(),
            capture: false,
        });
        self
    }

    pub fn style<F: FnOnce(&mut CssStyle)>(mut self, f: F) -> Self {
        let mut style = CssStyle::new();
        f(&mut style);
        self.element.style = style;
        self
    }

    pub fn build(self) -> DomNode {
        DomNode::Element(self.element)
    }
}

// ---------------------------------------------------------------------------
// Web runtime
// ---------------------------------------------------------------------------

pub struct WebRuntime {
    dom: DomTree,
    ws_connections: HashMap<String, WebSocketConnection>,
    event_handlers: HashMap<String, Box<dyn Fn(&DomEvent) + Send + Sync>>,
}

impl WebRuntime {
    pub fn new(root: DomNode) -> Self {
        Self {
            dom: DomTree::new(root),
            ws_connections: HashMap::new(),
            event_handlers: HashMap::new(),
        }
    }

    pub fn dom(&self) -> &DomTree {
        &self.dom
    }

    pub fn dom_mut(&mut self) -> &mut DomTree {
        &mut self.dom
    }

    pub fn open_websocket(&mut self, id: &str, url: &str) {
        let mut ws = WebSocketConnection::new(url);
        ws.open();
        self.ws_connections.insert(id.to_string(), ws);
    }

    pub fn send_ws_text(&mut self, id: &str, text: &str) {
        if let Some(ws) = self.ws_connections.get_mut(id) {
            ws.send_text(text);
        }
    }

    pub fn get_ws_state(&self, id: &str) -> Option<&WebSocketState> {
        self.ws_connections.get(id).map(|ws| ws.state())
    }

    pub fn register_event_handler<F: Fn(&DomEvent) + Send + Sync + 'static>(
        &mut self,
        handler_id: &str,
        handler: F,
    ) {
        self.event_handlers
            .insert(handler_id.to_string(), Box::new(handler));
    }

    pub fn dispatch_event(&self, event: &DomEvent) {
        for listener in self.find_listeners(&event.target_id) {
            if let Some(handler) = self.event_handlers.get(&listener.handler_id) {
                handler(event);
            }
        }
    }

    fn find_listeners(&self, _target_id: &Option<String>) -> Vec<EventListener> {
        // Simplified: return all listeners. A real implementation would walk the DOM.
        Vec::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dom_node_text() {
        let node = DomNode::text("hello");
        assert_eq!(node, DomNode::Text("hello".into()));
    }

    #[test]
    fn css_style_set_get() {
        let mut style = CssStyle::new();
        style.set("color", "red");
        assert_eq!(style.get("color"), Some("red"));
        assert_eq!(style.to_css_text(), "color: red;");
    }

    #[test]
    fn css_style_chaining() {
        let mut style = CssStyle::new();
        style.display("flex")
            .width("100%")
            .height("100vh")
            .background_color("white");
        assert_eq!(style.get("display"), Some("flex"));
        assert_eq!(style.get("width"), Some("100%"));
        assert_eq!(style.get("height"), Some("100vh"));
        assert_eq!(style.get("background-color"), Some("white"));
    }

    #[test]
    fn dom_tree_find_by_id() {
        let child = DomElement {
            tag: "span".into(),
            id: Some("label".into()),
            class_list: vec![],
            attributes: HashMap::new(),
            style: CssStyle::new(),
            children: vec![DomNode::text("hi")],
            event_listeners: vec![],
        };
        let root = DomElement {
            tag: "div".into(),
            id: Some("root".into()),
            class_list: vec![],
            attributes: HashMap::new(),
            style: CssStyle::new(),
            children: vec![DomNode::Element(child)],
            event_listeners: vec![],
        };
        let tree = DomTree::new(DomNode::Element(root));
        assert_eq!(tree.element_count(), 2);
        assert!(tree.find_element_by_id("label").is_some());
        assert!(tree.find_element_by_id("missing").is_none());
    }

    #[test]
    fn dom_diff_text_change() {
        let old = DomNode::text("old");
        let new = DomNode::text("new");
        let patches = DomTree::diff(&old, &new);
        assert_eq!(patches.len(), 1);
        assert!(matches!(patches[0], DomPatch::ReplaceText { .. }));
    }

    #[test]
    fn dom_diff_same_is_noop() {
        let node = DomNode::text("same");
        let patches = DomTree::diff(&node, &node);
        assert!(patches.is_empty());
    }

    #[test]
    fn websocket_lifecycle() {
        let mut ws = WebSocketConnection::new("ws://localhost:8080");
        assert_eq!(ws.state(), &WebSocketState::Connecting);
        ws.open();
        assert_eq!(ws.state(), &WebSocketState::Open);

        ws.send_text("hello");
        assert_eq!(ws.buffered_amount(), 1);

        ws.close(1000, "bye");
        assert_eq!(ws.state(), &WebSocketState::Closed);
    }

    #[test]
    fn websocket_message_types() {
        let text_msg = WebSocketMessage::text("hi");
        assert!(!text_msg.is_binary);
        assert_eq!(text_msg.as_text(), Some("hi"));

        let bin_msg = WebSocketMessage::binary(vec![0, 1, 2]);
        assert!(bin_msg.is_binary);
        assert!(bin_msg.as_text().is_none());
    }

    #[test]
    fn websocket_callbacks() {
        use std::sync::Arc;
        let mut ws = WebSocketConnection::new("ws://test");
        let open_called = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag = Arc::clone(&open_called);
        ws.set_on_open(move || {
            flag.store(true, std::sync::atomic::Ordering::Relaxed);
        });
        ws.open();
        assert!(open_called.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn dom_builder_creates_element() {
        let node = DomBuilder::element_with_id("div", "app")
            .class("container")
            .class("main")
            .attr("role", "main")
            .text_child("Hello")
            .build();

        if let DomNode::Element(el) = node {
            assert_eq!(el.tag, "div");
            assert_eq!(el.id, Some("app".into()));
            assert_eq!(el.class_list, vec!["container", "main"]);
            assert_eq!(el.attributes.get("role").unwrap(), "main");
            assert_eq!(el.children.len(), 1);
        } else {
            panic!("Expected element");
        }
    }

    #[test]
    fn websocket_enqueue_receive() {
        let mut ws = WebSocketConnection::new("ws://test");
        ws.open();
        ws.enqueue_receive(WebSocketMessage::text("msg1"));
        ws.enqueue_receive(WebSocketMessage::text("msg2"));
        assert_eq!(ws.buffered_amount(), 0); // send buffer empty

        // Process receive buffer - callback clears it
        ws.set_on_message(|_msg| {});
        ws.process_receive();
    }

    #[test]
    fn serialization_roundtrip() {
        let event = DomEvent {
            event_type: "click".into(),
            target_id: Some("btn".into()),
            bubbles: true,
            cancelable: true,
            default_prevented: false,
            timestamp_ms: 1000.0,
            detail: HashMap::new(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: DomEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, parsed);
    }
}
