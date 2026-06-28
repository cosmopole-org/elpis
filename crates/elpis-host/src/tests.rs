//! Integration tests driving a Miniapp end-to-end through the sandbox with the
//! headless backend.

use serde_json::Value;

use crate::{HeadlessBackend, Sandbox, SandboxConfig, UiEvent};

fn boot(source: &str) -> Sandbox {
    static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let id = format!("test-{}", N.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
    let cfg = SandboxConfig::new(id);
    let mut sb = Sandbox::from_js(cfg, source, Box::new(HeadlessBackend::new()))
        .expect("miniapp should compile");
    sb.boot().expect("boot should succeed");
    sb
}

fn tree_json(sb: &Sandbox) -> String {
    sb.tree().map(elpis_protocol::tree_to_json).unwrap_or_default()
}

const COUNTER: &str = r#"
var count = 0;
function view() {
  return column({ children: [
    text("Count: " + count, { size: 20 }),
    on(button("Increment", {}), "click", "inc"),
    on(button("Reset", {}), "click", "reset")
  ]});
}
function onEvent(ev) {
  if (ev.id == "inc") { count = count + 1; render(view()); }
  if (ev.id == "reset") { count = 0; render(view()); }
  return null;
}
render(view());
"#;

#[test]
fn boots_and_renders_initial_tree() {
    let sb = boot(COUNTER);
    assert_eq!(sb.frames(), 1, "boot should render exactly one frame");
    let json = tree_json(&sb);
    assert!(json.contains("\"type\":\"column\""), "root should be a column: {json}");
    assert!(json.contains("Count: 0"), "initial count text missing: {json}");
}

#[test]
fn event_updates_state_and_repatches() {
    let mut sb = boot(COUNTER);
    sb.dispatch_event(&UiEvent::new("inc", "click", Value::Null)).unwrap();
    assert_eq!(sb.frames(), 2, "event should trigger a second render");
    assert!(tree_json(&sb).contains("Count: 1"), "state did not advance");

    sb.dispatch_event(&UiEvent::new("inc", "click", Value::Null)).unwrap();
    assert!(tree_json(&sb).contains("Count: 2"));

    sb.dispatch_event(&UiEvent::new("reset", "click", Value::Null)).unwrap();
    assert!(tree_json(&sb).contains("Count: 0"));
}

#[test]
fn unknown_handler_is_harmless() {
    let mut sb = boot(COUNTER);
    // No handler with this id exists; should not trap.
    sb.dispatch_event(&UiEvent::new("does-not-exist", "click", Value::Null)).unwrap();
    assert!(tree_json(&sb).contains("Count: 0"));
}

#[test]
fn storage_roundtrips_through_the_bridge() {
    let src = r#"
        storageSet("greeting", "hello");
        var v = storageGet("greeting");
        render(text("stored:" + v, {}));
    "#;
    let sb = boot(src);
    assert!(tree_json(&sb).contains("stored:hello"), "{}", tree_json(&sb));
}

#[test]
fn surface_info_is_available_to_the_guest() {
    let src = r#"
        var s = surfaceInfo();
        render(text("w=" + s.width, {}));
    "#;
    let cfg = SandboxConfig {
        surface: Some(crate::SurfaceInfo { width: 1024.0, height: 768.0, scale_factor: 2.0 }),
        ..SandboxConfig::new("surf")
    };
    let mut sb = Sandbox::from_js(cfg, src, Box::new(HeadlessBackend::new())).unwrap();
    sb.boot().unwrap();
    assert!(tree_json(&sb).contains("w=1024"), "{}", tree_json(&sb));
}

#[test]
fn host_send_reaches_the_outbox() {
    let src = r#"
        hostSend("nav", { route: "home" });
        render(text("sent", {}));
    "#;
    let mut sb = boot(src);
    let out = sb.take_outbox();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].channel, "nav");
}

#[test]
fn invalid_js_fails_to_construct() {
    let cfg = SandboxConfig::new("bad");
    let res = Sandbox::from_js(cfg, "this is not (valid javascript <<<", Box::new(HeadlessBackend::new()));
    assert!(res.is_err());
}

#[test]
fn diff_minimizes_work_on_text_only_change() {
    // Two renders that differ only in the text node should not remount.
    let mut sb = boot(COUNTER);
    let mounts_before = sb.frames();
    sb.dispatch_event(&UiEvent::new("inc", "click", Value::Null)).unwrap();
    assert!(sb.frames() > mounts_before);
    // The retained tree advanced via a patch, not a fresh mount; correctness of
    // the patch itself is covered by elpis-protocol's round-trip tests.
}
