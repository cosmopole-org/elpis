//! UI events flowing from the backend back into the guest.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// An event produced by the Blinc backend (a click, text change, scroll, key
/// press, 3D pick, animation completion, …) addressed to a guest handler id
/// that the Miniapp attached to a node via its `events` map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiEvent {
    /// The guest handler id from the node's `events` map. Serialized as `id`,
    /// the field name the guest's `onEvent(ev)` reads (`ev.id`).
    #[serde(rename = "id")]
    pub handler: String,
    /// Event kind, e.g. `"click"`, `"change"`, `"input"`, `"submit"`,
    /// `"scroll"`, `"keydown"`, `"pick"`, `"animationend"`, `"dismiss"`.
    #[serde(rename = "type")]
    pub kind: String,
    /// Event-specific payload (the new value for `change`/`input`, pointer
    /// coordinates for `pointerdown`, the key for `keydown`, …).
    #[serde(default)]
    pub value: Value,
}

impl UiEvent {
    pub fn new(handler: impl Into<String>, kind: impl Into<String>, value: Value) -> UiEvent {
        UiEvent { handler: handler.into(), kind: kind.into(), value }
    }

    /// A bare click with no payload.
    pub fn click(handler: impl Into<String>) -> UiEvent {
        UiEvent::new(handler, "click", Value::Null)
    }
}
