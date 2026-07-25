//! Fusion Compiler Profiler
//! Addresses: No profiling support, Slow compilation.
//!
//! Tracks phase durations (Lexing, Parsing, Sema, Codegen) and exports
//! to Chrome Tracing Format (JSON) for visualization.
use crate::types::*;

pub struct TraceEvent {
    pub name: FString,
    pub phase: FString, // "B" for begin, "E" for end
    pub timestamp_us: FI64,
    pub thread_id: FSize,
}

pub struct Profiler {
    events: FVec<TraceEvent>,
    active_spans: FVec<FString>,
}


impl Profiler {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            active_spans: Vec::new(),
        }
    }

    fn current_time_us() -> FI64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        (duration.as_secs() as FI64) * 1_000_000 + (duration.subsec_micros() as FI64)
    }

    /// Begins a profiling span (e.g., "Semantic Analysis").
    pub fn begin_span(&mut self, name: FString) {
        self.active_spans.push(name.clone());
        self.events.push(TraceEvent {
            name,
            phase: "B".to_string(),
            timestamp_us: Self::current_time_us(),
            thread_id: 1,
        });
    }

    /// Ends the most recently opened profiling span.
    pub fn end_span(&mut self) {
        if let Some(name) = self.active_spans.pop() {
            self.events.push(TraceEvent {
                name,
                phase: "E".to_string(),
                timestamp_us: Self::current_time_us(),
                thread_id: 1,
            });
        }
    }

    /// Exports collected events to a Chrome Tracing compatible JSON file.
    pub fn export_json(&self) -> FString {
        let mut json = FString::from("[\n");
        for (i, event) in self.events.iter().enumerate() {
            json.push_str(&format!(
                r#"  {{"name": "{}", "ph": "{}", "ts": {}, "pid": 1, "tid": {}}}"#,
                event.name, event.phase, event.timestamp_us, event.thread_id
            ));
            if i < self.events.len() - 1 {
                json.push_str(",\n");
            } else {
                json.push('\n');
            }
        }
        json.push(']');
        json
    }
}