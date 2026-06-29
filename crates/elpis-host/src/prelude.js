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
function audio(src, props) {
  var n = props ? props : {};
  n.type = "audio";
  n.src = src;
  return n;
}
// A named guest-defined component instance (for tooling / hot-reload).
function component(name, propsBlob, props) {
  var n = props ? props : {};
  n.type = "component";
  n.name = name;
  n.props = propsBlob ? propsBlob : {};
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

// ---- Paint helpers --------------------------------------------------------
//
// Foundational color/brush/shadow constructors so a UI kit (or any Miniapp)
// composes paint with named helpers instead of bare object literals.

// Parse a CSS-ish "#rgb"/"#rrggbb"/"#rrggbbaa" hex string into a color
// (channels 0..1). Use `hexA(s, alpha)` to force a specific alpha.
function hex(s) {
  var str = "" + s;
  if (charAt(str, 0) == "#") { str = substring(str, 1, len(str)); }
  var zero = ord("0");
  var nine = ord("9");
  var ach = ord("a");
  var digit = function (ch) {
    var code = ord(lower(ch));
    if (code >= zero && code <= nine) { return code - zero; }
    return code - ach + 10;
  };
  var byteAt = function (i) { return (digit(charAt(str, i)) * 16 + digit(charAt(str, i + 1))) / 255.0; };
  var nibAt = function (i) { var v = digit(charAt(str, i)); return (v * 16 + v) / 255.0; };
  var n = len(str);
  if (n == 3) { return { r: nibAt(0), g: nibAt(1), b: nibAt(2), a: 1.0 }; }
  if (n == 8) { return { r: byteAt(0), g: byteAt(2), b: byteAt(4), a: byteAt(6) }; }
  return { r: byteAt(0), g: byteAt(2), b: byteAt(4), a: 1.0 };
}
function hexA(s, a) { return withAlpha(hex(s), a); }

// A color with its alpha replaced (handy for tints/overlays).
function withAlpha(color, a) { return { r: color.r, g: color.g, b: color.b, a: a }; }

// Gradient stop and gradient brushes.
function stop(offset, color) { return { offset: offset, color: color }; }
function linearGradient(angle, stops) { return { kind: "linear_gradient", angle: angle, stops: stops }; }
function radialGradient(stops, center, radius) {
  return { kind: "radial_gradient", center: center ? center : [0.5, 0.5], radius: radius ? radius : 0.5, stops: stops };
}
function conicGradient(stops, center, startAngle) {
  return { kind: "conic_gradient", center: center ? center : [0.5, 0.5], start_angle: startAngle ? startAngle : 0, stops: stops };
}
function imageBrush(src, fit) { return { kind: "image", src: src, fit: fit ? fit : "cover" }; }

// A drop/inner shadow.
function shadow(offset, blur, color, spread, inset) {
  return { offset: offset ? offset : [0, 0], blur: blur ? blur : 0, spread: spread ? spread : 0,
           color: color ? color : rgba(0, 0, 0, 0.3), inset: inset ? true : false };
}

// ---- Liquid glass ---------------------------------------------------------
//
// A liquid-glass material descriptor for `style.glass_material`. The host
// lowering expands it into a backdrop-blur + saturate filter, a tinted
// translucent background, a specular rim border, a radius and an elevation
// shadow — only for fields the node didn't set itself.
function _isNum(x) { return typeOf(x) == "number"; }
function glassMaterial(opts) {
  var o = opts ? opts : {};
  var m = {};
  if (_isNum(o.blur)) { m.blur = o.blur; }
  if (_isNum(o.saturate)) { m.saturate = o.saturate; }
  if (_isNum(o.brightness)) { m.brightness = o.brightness; }
  if (o.tint) { m.tint = o.tint; }
  if (o.rim) { m.rim = o.rim; }
  if (_isNum(o.rim_width)) { m.rim_width = o.rim_width; }
  if (_isNum(o.radius)) { m.radius = o.radius; }
  if (_isNum(o.elevation)) { m.elevation = o.elevation; }
  if (o.interactive) { m.interactive = true; }
  return m;
}

// ---- Composition helpers --------------------------------------------------

// Set a stable reconciliation key on a node (chainable).
function withKey(n, key) { n.key = key; return n; }

// Shallow-merge `extra` style into a node's style (extra wins). Returns node.
function withStyle(n, extra) {
  if (!extra) { return n; }
  if (!n.style) { n.style = extra; return n; }
  n.style = merge(n.style, extra);
  return n;
}

// Bind several events at once: bindEvents(node, { click: "id", input: "id2" }).
function bindEvents(n, map) {
  if (!map) { return n; }
  var ks = keys(map);
  for (var i = 0; i < len(ks); i = i + 1) { on(n, ks[i], map[ks[i]]); }
  return n;
}

// Attach a keyframe/spring animation (chainable).
function withAnim(n, animation) {
  if (!n.animations) { n.animations = []; }
  push(n.animations, animation);
  return n;
}

// Attach a layout transition (chainable).
function withTransition(n, transition) { n.transition = transition; return n; }
