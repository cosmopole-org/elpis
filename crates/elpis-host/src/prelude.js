// ===========================================================================
// Elpis Miniapp prelude.
//
// This is prepended to every Miniapp's source before it is compiled onto the
// elpian-vm. It wraps the raw `askHost(api, [args])` seam in an ergonomic UI
// vocabulary so a Miniapp builds Blinc UIs with plain object literals and never
// touches the host protocol directly.
//
// The host (elpis-host) calls these guest entry points when present:
//   * top-level code  -> runs once at boot (kick off the first render here)
//   * onEvent(ev)     -> a UI event ({ id, type, value }) reached a handler
//   * onTick(t)       -> an animation frame ({ dt, time }) for animated nodes
//   * onMessage(m)    -> a host->guest message ({ channel, message })
// Defining them is optional; the host treats a missing one as a no-op.
// ===========================================================================

// ---- Core seam wrappers ---------------------------------------------------

function render(tree) { return askHost("ui.render", [tree]); }
function applyPatches(patches) { return askHost("ui.patch", [patches]); }
function log(x) { askHost("log", [x]); }
function surfaceInfo() { return askHost("ui.surfaceInfo", []); }

function themeSet(theme) { return askHost("theme.set", [theme]); }
function themeGet() { return askHost("theme.get", []); }

function routerPush(route) { return askHost("router.push", [route]); }
function routerReplace(route) { return askHost("router.replace", [route]); }
function routerPop() { return askHost("router.pop", []); }
function routerCurrent() { return askHost("router.current", []); }

function storageGet(key) { return askHost("storage.get", [key]); }
function storageSet(key, value) { return askHost("storage.set", [key, value]); }
function storageRemove(key) { return askHost("storage.remove", [key]); }

function now() { var r = askHost("time.now", []); if (r) { return r.ms; } return 0; }
function monotonic() { var r = askHost("time.monotonic", []); if (r) { return r.ms; } return 0; }
function random() { return askHost("random.next", []); }

function hostSend(channel, message) { return askHost("host.send", [channel, message]); }
function hostRequest(channel, message) { return askHost("host.request", [channel, message]); }

function animStart(target, animation) { return askHost("anim.start", [target, animation]); }
function animCancel(target) { return askHost("anim.cancel", [target]); }

function mediaPlay(id) { return askHost("media.play", [id]); }
function mediaPause(id) { return askHost("media.pause", [id]); }

// ---- Widget builders ------------------------------------------------------
//
// Every builder returns a plain node object the host parses into a typed
// `Node`. `props` carries style + per-kind fields + an `events`/`children`
// list; helpers below fill in the `type` tag.

function node(type, props) {
  var n = props ? props : {};
  n.type = type;
  return n;
}

function div(props) { return node("div", props); }
function row(props) { return node("row", props); }
function column(props) { return node("column", props); }
function stack(props) { return node("stack", props); }
function grid(props) { return node("grid", props); }
function spacer(props) { return node("spacer", props); }
function scroll(props) { return node("scroll", props); }
function overlay(props) { return node("overlay", props); }

function text(value, props) {
  var n = props ? props : {};
  n.type = "text";
  n.text = value;
  return n;
}
function richText(props) { return node("rich_text", props); }
function markdown(source, props) {
  var n = props ? props : {};
  n.type = "markdown";
  n.source = source;
  return n;
}
function image(src, props) {
  var n = props ? props : {};
  n.type = "image";
  n.src = src;
  return n;
}
function svg(props) { return node("svg", props); }
function icon(name, props) {
  var n = props ? props : {};
  n.type = "icon";
  n.name = name;
  return n;
}

function button(label, props) {
  var n = props ? props : {};
  n.type = "button";
  n.label = label;
  return n;
}
function textInput(props) { return node("text_input", props); }
function checkbox(props) { return node("checkbox", props); }
function toggle(props) { return node("switch", props); }
function radio(props) { return node("radio", props); }
function slider(props) { return node("slider", props); }
function dropdown(props) { return node("dropdown", props); }
function tabs(props) { return node("tabs", props); }
function carousel(props) { return node("carousel", props); }
function progressBar(props) { return node("progress_bar", props); }
function spinner(props) { return node("spinner", props); }

function canvas(ops, props) {
  var n = props ? props : {};
  n.type = "canvas";
  n.ops = ops;
  return n;
}
function scene3d(props) { return node("scene3d", props); }
function video(src, props) {
  var n = props ? props : {};
  n.type = "video";
  n.src = src;
  return n;
}

// ---- Helpers --------------------------------------------------------------

// Attach children to a node (chainable-ish).
function withChildren(n, children) { n.children = children; return n; }

// Attach an event handler id to a node.
function on(n, eventName, handlerId) {
  if (!n.events) { n.events = {}; }
  n.events[eventName] = handlerId;
  return n;
}

// Build a solid-color brush.
function solid(color) { return { kind: "solid", color: color }; }

// Build an rgba color (channels 0..1).
function rgba(r, g, b, a) { return { r: r, g: g, b: b, a: a }; }
function rgb(r, g, b) { return { r: r, g: g, b: b, a: 1.0 }; }
