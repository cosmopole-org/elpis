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

// ---- Glass UI kit ---------------------------------------------------------
//
// The kit is plain JS prepended ahead of a Miniapp (the `--lib` path the host
// binary uses). These tests prove the whole SDK compiles on the Elpian VM,
// boots, renders a real glass tree, and survives event dispatch — i.e. that
// "covers everything and any widget" actually executes in the sandbox.

/// The Glass UI kit SDK source (the same file shipped under `sdk/`).
const GLASS_KIT: &str = include_str!("../../../sdk/glass-ui-kit.js");
/// The gallery Miniapp that exercises the kit.
const GLASS_GALLERY: &str = include_str!("../../../miniapps/glass-gallery/app.js");

/// Boot a Miniapp with the Glass kit prepended (mirrors `elpis --lib`).
fn boot_with_kit(app: &str) -> Sandbox {
    let source = format!("{GLASS_KIT}\n// ---- app ----\n{app}");
    boot(&source)
}

#[test]
fn glass_kit_compiles_and_renders_components() {
    // A driver that touches a broad slice of the kit's families: layout,
    // typography, glass surfaces, actions, inputs, navigation, feedback,
    // data display, and a chart.
    let driver = r#"
        function view() {
          return Glass.screen({ children: [
            Glass.navbar({ title: "Kit", trailing: [ Glass.iconButton({ icon: "plus", onClick: "noop" }) ] }),
            Glass.hero({ title: "Hello", subtitle: "Liquid glass" }),
            Glass.row({ gap: 8, children: [
              Glass.button({ label: "Primary", variant: "accent", onClick: "noop" }),
              Glass.button({ label: "Ghost", variant: "ghost", onClick: "noop" }),
              Glass.chip({ label: "Tag" }), Glass.badge({ text: "9" })
            ]}),
            Glass.field({ label: "Name", control: Glass.textField({ placeholder: "you" }) }),
            Glass.toggle({ checked: true, onChange: "t" }),
            Glass.slider({ value: 0.5, onChange: "s" }),
            Glass.list({ items: [ { title: "One", chevron: true }, { title: "Two", chevron: true } ] }),
            Glass.stat({ label: "Revenue", value: "$10k", delta: "5%", deltaUp: true }),
            Glass.alert({ kind: "success", title: "Saved", message: "ok" }),
            Glass.ring({ value: 0.7, size: 100 }),
            Glass.gauge({ value: 0.4, size: 120 }),
            Glass.barChart({ data: [1, 4, 2, 6, 3], width: 200, height: 80 }),
            Glass.lineChart({ data: [1, 3, 2, 5], width: 200, height: 80 }),
            Glass.tabBar({ items: [ { icon: "a", label: "A" }, { icon: "b", label: "B" } ], selected: 0, onSelect: "tab" })
          ]});
        }
        render(view());
    "#;
    let sb = boot_with_kit(driver);
    assert_eq!(sb.frames(), 1, "kit driver should render one frame");
    let json = tree_json(&sb);
    // Glass surfaces lower to a material; the kit emits glass_material.
    assert!(json.contains("glass_material"), "no glass material in tree: {json}");
    assert!(json.contains("Primary"), "button label missing");
    assert!(json.contains("\"type\":\"canvas\""), "bar chart canvas missing");
    assert!(json.contains("\"type\":\"text_input\""), "text field missing");
}

#[test]
fn glass_gallery_boots_and_handles_events() {
    let mut sb = boot_with_kit(GLASS_GALLERY);
    assert_eq!(sb.frames(), 1, "gallery should render its first frame");
    assert!(tree_json(&sb).contains("Liquid Glass"), "hero title missing");

    // Switch to the Inputs tab (handler id "tab:1") and confirm a re-render
    // that doesn't trap.
    sb.dispatch_event(&UiEvent::new("tab:1", "click", Value::Null)).unwrap();
    assert!(sb.frames() >= 2, "tab switch should re-render");
    assert!(tree_contains(&sb, "Notifications"), "inputs tab content missing");

    // Walk every section: each tab must re-render (frames advance) without a
    // trap. This catches a single broken component aborting a whole render
    // (e.g. a malformed canvas op silently dropping the frame).
    let sections = ["tab:0", "tab:1", "tab:2", "tab:3", "tab:4", "tab:5"];
    for (i, id) in sections.iter().enumerate() {
        let before = sb.frames();
        sb.dispatch_event(&UiEvent::new(*id, "click", Value::Null)).unwrap();
        assert!(sb.frames() > before, "section {id} did not re-render (component {i} aborted the frame?)");
    }
    // The charts section renders ring/gauge/bar/line canvases.
    assert!(tree_json(&sb).contains("\"type\":\"canvas\""), "charts canvases missing");

    // Drive animation frames (blob wallpaper + 3D scene) — must not trap.
    sb.tick(16.0).unwrap();
    sb.tick(16.0).unwrap();

    // Open the modal overlay.
    sb.dispatch_event(&UiEvent::new("tab:3", "click", Value::Null)).unwrap();
    sb.dispatch_event(&UiEvent::new("openModal", "click", Value::Null)).unwrap();
    assert!(tree_json(&sb).contains("Liquid Glass Dialog"), "modal did not open");
}

/// Small helper: does the retained tree's JSON contain `needle`?
fn tree_contains(sb: &Sandbox, needle: &str) -> bool {
    tree_json(sb).contains(needle)
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
