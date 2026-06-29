// ===========================================================================
// Glass UI Kit — a "liquid glass" component SDK for Elpis Miniapps.
//
// A full, batteries-included component set (layout, typography, actions, forms,
// navigation, feedback, overlays, data display, media, charts) where every
// surface is rendered as Apple-style *liquid glass*: a translucent,
// backdrop-blurred panel with a saturated backdrop, a bright specular rim, and
// physical depth.
//
// It is plain JavaScript that runs inside the Elpis sandbox on the Elpian VM.
// It is built **entirely** on the Blinc UI builders the host imports into the
// VM via the prelude (`div`, `row`, `column`, `text`, `button`, `canvas`, …,
// plus the `glassMaterial`/`hex`/gradient/`on`/`withStyle` helpers). It never
// touches the host protocol directly.
//
// Usage — module import is denied in the sandbox, so this file is *prepended*
// to a Miniapp (e.g. `elpis --lib sdk/glass-ui-kit.js miniapps/glass-gallery/app.js`,
// or concatenated by the embedder). It defines one global, `Glass`:
//
//   function view() {
//     return Glass.screen({ children: [
//       Glass.navbar({ title: "Inbox", trailing: [ Glass.iconButton({ icon: "plus", onClick: "add" }) ] }),
//       Glass.card({ children: [
//         Glass.heading({ text: "Welcome" }),
//         Glass.button({ label: "Continue", variant: "accent", onClick: "go" })
//       ]})
//     ]});
//   }
//   render(view());
//
// Conventions
//   * Every factory takes a single optional `opts` object and returns a node.
//   * `opts.children` is an array of child nodes.
//   * Event handlers are passed as guest handler-id strings: `onClick`,
//     `onChange`, `onInput`, `onSubmit`, `onDismiss`, `onSelect`, …
//   * `opts.style` is an extra style object shallow-merged in last (you can
//     override any piece of any component).
//   * `opts.key` sets the reconciliation key for stable, stateful diffing.
// ===========================================================================

var Glass = {};
Glass.version = "1.0.0";

// ---- Internal helpers -----------------------------------------------------

Glass._opt = function (o) { return o ? o : {}; };
Glass._num = function (x) { return typeOf(x) == "number"; };
// Default helper: returns o[k] when present, else d (so 0/false are honored).
Glass._d = function (o, k, d) { return has(o, k) ? o[k] : d; };
// Map over an array with a (value, index) callback into a new array.
Glass._map = function (list, fn) {
  var out = [];
  if (!list) { return out; }
  for (var i = 0; i < len(list); i = i + 1) { push(out, fn(list[i], i)); }
  return out;
};
// Finalize a node: apply key, extra style, and an events map from opts.
Glass._fin = function (n, o) {
  if (has(o, "key")) { n.key = o.key; }
  if (has(o, "style")) { withStyle(n, o.style); }
  if (has(o, "events")) { bindEvents(n, o.events); }
  return n;
};
// Coerce children: accept an array, a single node, or nothing.
Glass._kids = function (o) {
  if (!has(o, "children")) { return []; }
  var c = o.children;
  if (typeOf(c) == "array") { return c; }
  return [c];
};

// ---- Design tokens + theming ---------------------------------------------
//
// A dark "midnight" palette by default. `Glass.theme(partial)` overrides any
// token; every component reads from `Glass.tokens` so re-theming is global.

Glass.tokens = {
  // Brand / semantic colors.
  accent: hex("#5B8CFF"),
  accent2: hex("#A66BFF"),
  danger: hex("#FF5B6E"),
  success: hex("#34D399"),
  warning: hex("#FBBF24"),
  info: hex("#38BDF8"),
  // Text.
  text: hex("#EAF0FF"),
  textDim: hexA("#EAF0FF", 0.64),
  textFaint: hexA("#EAF0FF", 0.40),
  onAccent: hex("#0B0E17"),
  // Backdrop (the wallpaper behind the glass).
  bg0: hex("#070912"),
  bg1: hex("#0E1326"),
  bg2: hex("#1B1230"),
  // Glass material defaults.
  glassTint: hexA("#FFFFFF", 0.12),
  glassRim: hexA("#FFFFFF", 0.45),
  // Scales.
  radius: { sm: 10, md: 16, lg: 22, xl: 30, pill: 999, round: 9999 },
  blur: { thin: 8, regular: 18, thick: 32 },
  space: { xs: 4, sm: 8, md: 12, lg: 20, xl: 32, xxl: 48 },
  font: { xs: 12, sm: 14, md: 16, lg: 20, xl: 26, xxl: 34, display: 48 }
};

Glass.theme = function (partial) {
  if (partial) { Glass.tokens = merge(Glass.tokens, partial); }
  return Glass.tokens;
};

// ---- The liquid-glass material -------------------------------------------
//
// `Glass.material(variant)` returns a `glass_material` descriptor. Pass a
// string variant ("regular" | "clear" | "thin" | "thick" | "accent" |
// "danger" | "success" | "warning") or an options object to fine-tune.

Glass.material = function (v) {
  var t = Glass.tokens;
  if (typeOf(v) == "object") { return glassMaterial(v); }
  v = v ? v : "regular";
  var base = { rim: t.glassRim, rim_width: 1, radius: t.radius.lg, elevation: 18 };
  var tinted = function (c, alpha, blur, sat) {
    return glassMaterial(merge(base, { blur: blur, saturate: sat, tint: withAlpha(c, alpha), rim: withAlpha(c, 0.6) }));
  };
  if (v == "clear") { return glassMaterial(merge(base, { blur: 10, saturate: 1.3, brightness: 1.12, tint: hexA("#FFFFFF", 0.05), elevation: 10 })); }
  if (v == "thin") { return glassMaterial(merge(base, { blur: t.blur.thin, saturate: 1.4, tint: hexA("#FFFFFF", 0.08), elevation: 8 })); }
  if (v == "thick") { return glassMaterial(merge(base, { blur: t.blur.thick, saturate: 1.9, tint: hexA("#0B0E17", 0.42), elevation: 28 })); }
  if (v == "accent") { return tinted(t.accent, 0.28, 20, 2.0); }
  if (v == "danger") { return tinted(t.danger, 0.28, 20, 1.9); }
  if (v == "success") { return tinted(t.success, 0.26, 20, 1.9); }
  if (v == "warning") { return tinted(t.warning, 0.26, 20, 1.9); }
  return glassMaterial(merge(base, { blur: t.blur.regular, saturate: 1.8, tint: t.glassTint }));
};

// A raw glass surface: a div carrying a glass material. The building block of
// every other panel-like component.
Glass.surface = function (o) {
  o = Glass._opt(o);
  var mat = has(o, "material") ? o.material : Glass.material(Glass._d(o, "variant", "regular"));
  var st = { glass_material: mat };
  if (has(o, "padding")) { st.padding = Glass._edges(o.padding); }
  if (has(o, "gap")) { st.gap = o.gap; }
  if (has(o, "width")) { st.width = Glass._len(o.width); }
  if (has(o, "height")) { st.height = Glass._len(o.height); }
  if (has(o, "direction")) { st.direction = o.direction; }
  if (has(o, "align")) { st.align_items = o.align; }
  if (has(o, "justify")) { st.justify_content = o.justify; }
  var n = div({ style: st, children: Glass._kids(o) });
  return Glass._fin(n, o);
};

// ---- Length / edge sugar --------------------------------------------------

Glass._len = function (v) {
  if (typeOf(v) == "object") { return v; }            // already {unit,value}
  if (v == "full") { return { unit: "full" }; }
  if (v == "auto") { return { unit: "auto" }; }
  return { unit: "px", value: v };                     // bare number -> px
};
Glass._edges = function (v) {
  if (typeOf(v) == "object") { return v; }
  return { top: v, right: v, bottom: v, left: v };
};

// ===========================================================================
// Layout
// ===========================================================================

Glass._flex = function (builder, o) {
  o = Glass._opt(o);
  var st = {};
  if (has(o, "gap")) { st.gap = o.gap; }
  if (has(o, "align")) { st.align_items = o.align; }
  if (has(o, "justify")) { st.justify_content = o.justify; }
  if (has(o, "wrap")) { st.wrap = o.wrap; }
  if (has(o, "padding")) { st.padding = Glass._edges(o.padding); }
  if (has(o, "width")) { st.width = Glass._len(o.width); }
  if (has(o, "height")) { st.height = Glass._len(o.height); }
  if (has(o, "grow")) { st.flex_grow = o.grow; }
  var n = builder({ style: st, children: Glass._kids(o) });
  return Glass._fin(n, o);
};
Glass.row = function (o) { return Glass._flex(row, o); };
Glass.column = function (o) { return Glass._flex(column, o); };
Glass.stack = function (o) { return Glass._flex(stack, o); };
Glass.wrap = function (o) { o = merge({ wrap: true }, Glass._opt(o)); return Glass._flex(row, o); };
Glass.center = function (o) {
  o = merge({ align: "center", justify: "center" }, Glass._opt(o));
  return Glass._flex(column, o);
};
Glass.grid = function (o) {
  o = Glass._opt(o);
  var st = {};
  if (has(o, "columns")) { st.grid_template_columns = o.columns; }
  if (has(o, "rows")) { st.grid_template_rows = o.rows; }
  if (has(o, "gap")) { st.gap = o.gap; }
  if (has(o, "padding")) { st.padding = Glass._edges(o.padding); }
  var n = grid({ style: st, children: Glass._kids(o) });
  return Glass._fin(n, o);
};
Glass.scroll = function (o) {
  o = Glass._opt(o);
  var n = scroll({ axis: Glass._d(o, "axis", "vertical"), snap: Glass._d(o, "snap", false),
                   style: has(o, "padding") ? { padding: Glass._edges(o.padding) } : {},
                   children: Glass._kids(o) });
  return Glass._fin(n, o);
};
Glass.spacer = function (o) {
  o = Glass._opt(o);
  var st = { flex_grow: Glass._d(o, "grow", 1) };
  if (has(o, "size")) { st.width = Glass._len(o.size); st.height = Glass._len(o.size); st.flex_grow = 0; }
  return Glass._fin(spacer({ style: st }), o);
};
Glass.divider = function (o) {
  o = Glass._opt(o);
  var vertical = Glass._d(o, "vertical", false);
  var st = { background: solid(Glass.tokens.glassRim), opacity: 0.5 };
  if (vertical) { st.width = Glass._len(1); st.height = Glass._len(Glass._d(o, "length", "full")); }
  else { st.height = Glass._len(1); st.width = Glass._len(Glass._d(o, "length", "full")); }
  return Glass._fin(div({ style: st }), o);
};

// The app root: a full-bleed gradient backdrop with the content laid over it.
// Liquid glass only reads as glass against a colorful, textured background.
Glass.screen = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var bg = has(o, "background") ? o.background
    : linearGradient(135, [ stop(0, t.bg0), stop(0.55, t.bg1), stop(1, t.bg2) ]);
  var st = {
    width: Glass._len("full"), height: Glass._len("full"),
    background: bg, padding: Glass._edges(Glass._d(o, "padding", t.space.lg)),
    gap: Glass._d(o, "gap", t.space.lg)
  };
  var n = column({ style: st, children: Glass._kids(o) });
  return Glass._fin(n, o);
};

// ===========================================================================
// Glass panels
// ===========================================================================

Glass.card = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var inner = column({
    style: { gap: Glass._d(o, "gap", t.space.md), padding: Glass._edges(Glass._d(o, "padding", t.space.lg)) },
    children: Glass._kids(o)
  });
  var s = Glass.surface(merge(o, { variant: Glass._d(o, "variant", "regular"), children: [inner],
                                   padding: 0, gap: 0 }));
  return s;
};
Glass.panel = function (o) {
  o = merge({ variant: "thin" }, Glass._opt(o));
  return Glass.card(o);
};
Glass.hero = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var kids = [];
  if (has(o, "title")) { push(kids, Glass.heading({ text: o.title, size: t.font.xxl })); }
  if (has(o, "subtitle")) { push(kids, Glass.text({ text: o.subtitle, color: t.textDim, size: t.font.lg })); }
  kids = concat(kids, Glass._kids(o));
  return Glass.card(merge(o, { variant: Glass._d(o, "variant", "accent"), padding: t.space.xl, gap: t.space.md, children: kids }));
};
// A modal/drawer sheet surface (no overlay; compose with Glass.modal).
Glass.sheet = function (o) {
  o = merge({ variant: "thick" }, Glass._opt(o));
  return Glass.card(o);
};

// ===========================================================================
// Typography
// ===========================================================================

Glass.text = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var props = {
    size: Glass._d(o, "size", t.font.md),
    weight: Glass._d(o, "weight", "normal"),
    foreground: Glass._d(o, "color", t.text),
    align: Glass._d(o, "align", "start")
  };
  if (has(o, "italic")) { props.italic = o.italic; }
  if (has(o, "underline")) { props.underline = o.underline; }
  if (has(o, "maxLines")) { props.max_lines = o.maxLines; }
  if (has(o, "lineHeight")) { props.line_height = o.lineHeight; }
  if (has(o, "letterSpacing")) { props.letter_spacing = o.letterSpacing; }
  return Glass._fin(text(Glass._d(o, "text", ""), props), o);
};
Glass.heading = function (o) {
  o = merge({ weight: "bold", size: Glass.tokens.font.xl }, Glass._opt(o));
  return Glass.text(o);
};
Glass.title = function (o) { return Glass.heading(merge({ size: Glass.tokens.font.lg, weight: "semibold" }, Glass._opt(o))); };
Glass.subtitle = function (o) { return Glass.text(merge({ size: Glass.tokens.font.md, color: Glass.tokens.textDim }, Glass._opt(o))); };
Glass.caption = function (o) { return Glass.text(merge({ size: Glass.tokens.font.xs, color: Glass.tokens.textDim }, Glass._opt(o))); };
Glass.label = function (o) { return Glass.text(merge({ size: Glass.tokens.font.sm, weight: "medium", color: Glass.tokens.textDim }, Glass._opt(o))); };
Glass.display = function (o) { return Glass.heading(merge({ size: Glass.tokens.font.display, weight: "black" }, Glass._opt(o))); };
Glass.code = function (o) {
  o = Glass._opt(o);
  var st = { padding: { top: 2, right: 6, bottom: 2, left: 6 }, radius: { tl: 6, tr: 6, br: 6, bl: 6 },
             background: solid(hexA("#FFFFFF", 0.10)) };
  var n = text(Glass._d(o, "text", ""), { size: Glass.tokens.font.sm, font: "mono", foreground: Glass.tokens.text });
  return Glass._fin(div({ style: st, children: [n] }), o);
};
Glass.link = function (o) {
  o = Glass._opt(o);
  var n = Glass.text(merge({ color: Glass.tokens.accent, underline: true }, o));
  if (has(o, "onClick")) { on(n, "click", o.onClick); }
  n.style = merge(n.style ? n.style : {}, { cursor: "pointer" });
  return n;
};
Glass.markdown = function (o) {
  o = Glass._opt(o);
  return Glass._fin(markdown(Glass._d(o, "source", ""), {}), o);
};

// ===========================================================================
// Actions
// ===========================================================================

Glass.button = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var variant = Glass._d(o, "variant", "regular");
  // Map the kit variant onto a glass material + label color.
  var matVar = variant; var fg = t.text;
  if (variant == "primary" || variant == "accent") { matVar = "accent"; fg = t.text; }
  if (variant == "ghost") { matVar = "clear"; }
  if (variant == "destructive" || variant == "danger") { matVar = "danger"; }
  if (variant == "success") { matVar = "success"; }
  var pill = Glass._d(o, "pill", false);
  var disabled = Glass._d(o, "disabled", false);
  var mat = Glass.material(matVar);
  mat.radius = pill ? t.radius.pill : t.radius.md;
  mat.interactive = true;
  var pad = Glass._d(o, "size", "md");
  var py = pad == "lg" ? 14 : (pad == "sm" ? 6 : 10);
  var px = pad == "lg" ? 24 : (pad == "sm" ? 12 : 18);
  var content = [];
  if (has(o, "icon")) { push(content, icon(o.icon, { size: 18, color: fg })); }
  if (has(o, "label")) { push(content, text(o.label, { size: t.font.sm, weight: "semibold", foreground: fg })); }
  content = concat(content, Glass._kids(o));
  var st = {
    glass_material: mat,
    padding: { top: py, right: px, bottom: py, left: px },
    align_items: "center", justify_content: "center", gap: 8,
    opacity: disabled ? 0.45 : 1.0,
    cursor: disabled ? "not-allowed" : "pointer"
  };
  if (has(o, "width")) { st.width = Glass._len(o.width); }
  var n = row({ style: st, children: content });
  if (has(o, "onClick") && !disabled) { on(n, "click", o.onClick); }
  return Glass._fin(n, o);
};
Glass.iconButton = function (o) {
  o = Glass._opt(o);
  var sz = Glass._d(o, "size", 40);
  var mat = Glass.material(Glass._d(o, "variant", "regular"));
  mat.radius = Glass._d(o, "pill", true) ? Glass.tokens.radius.round : Glass.tokens.radius.md;
  mat.interactive = true;
  var st = { glass_material: mat, width: Glass._len(sz), height: Glass._len(sz),
             align_items: "center", justify_content: "center", cursor: "pointer" };
  var n = row({ style: st, children: [ icon(Glass._d(o, "icon", "circle"), { size: sz * 0.5, color: Glass.tokens.text }) ] });
  if (has(o, "onClick")) { on(n, "click", o.onClick); }
  return Glass._fin(n, o);
};
// Floating action button.
Glass.fab = function (o) {
  o = merge({ size: 56, variant: "accent" }, Glass._opt(o));
  return Glass.iconButton(o);
};
Glass.buttonGroup = function (o) {
  o = Glass._opt(o);
  return Glass.row(merge({ gap: Glass._d(o, "gap", 10) }, o));
};

// Segmented control: a glass track with selectable segments.
Glass.segmented = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var items = Glass._d(o, "items", []);
  var sel = Glass._d(o, "selected", 0);
  var segs = Glass._map(items, function (it, i) {
    var active = (i == sel);
    var lbl = typeOf(it) == "object" ? it.label : it;
    var val = typeOf(it) == "object" ? it.value : ("" + i);
    var st = { padding: { top: 6, right: 14, bottom: 6, left: 14 },
               radius: { tl: t.radius.sm, tr: t.radius.sm, br: t.radius.sm, bl: t.radius.sm },
               align_items: "center", justify_content: "center", cursor: "pointer" };
    if (active) { st.glass_material = Glass.material("accent"); st.glass_material.radius = t.radius.sm; }
    var seg = row({ style: st, children: [ text(lbl, { size: t.font.sm, weight: active ? "semibold" : "medium",
                                                       foreground: active ? t.text : t.textDim }) ] });
    if (has(o, "onSelect")) { on(seg, "click", o.onSelect + ":" + val); }
    return seg;
  });
  var track = row({ style: { glass_material: Glass.material("thin"), padding: { top: 4, right: 4, bottom: 4, left: 4 }, gap: 4 },
                    children: segs });
  return Glass._fin(track, o);
};

// ===========================================================================
// Indicators: badges, chips, tags, avatars
// ===========================================================================

Glass.badge = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var color = Glass._d(o, "color", t.accent);
  var st = { background: solid(withAlpha(color, 0.9)), padding: { top: 2, right: 8, bottom: 2, left: 8 },
             radius: { tl: t.radius.pill, tr: t.radius.pill, br: t.radius.pill, bl: t.radius.pill },
             align_items: "center", justify_content: "center" };
  var n = row({ style: st, children: [ text("" + Glass._d(o, "text", ""), { size: t.font.xs, weight: "bold", foreground: t.onAccent }) ] });
  return Glass._fin(n, o);
};
Glass.dot = function (o) {
  o = Glass._opt(o);
  var sz = Glass._d(o, "size", 8);
  var st = { width: Glass._len(sz), height: Glass._len(sz),
             radius: { tl: sz, tr: sz, br: sz, bl: sz }, background: solid(Glass._d(o, "color", Glass.tokens.success)) };
  return Glass._fin(div({ style: st }), o);
};
Glass.chip = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var mat = Glass.material("thin"); mat.radius = t.radius.pill;
  var content = [];
  if (has(o, "icon")) { push(content, icon(o.icon, { size: 14, color: t.textDim })); }
  push(content, text("" + Glass._d(o, "label", ""), { size: t.font.sm, foreground: t.text }));
  if (Glass._d(o, "removable", false)) { push(content, icon("x", { size: 14, color: t.textDim })); }
  var st = { glass_material: mat, padding: { top: 5, right: 12, bottom: 5, left: 12 }, gap: 6, align_items: "center" };
  if (has(o, "onClick")) { st.cursor = "pointer"; }
  var n = row({ style: st, children: content });
  if (has(o, "onClick")) { on(n, "click", o.onClick); }
  return Glass._fin(n, o);
};
Glass.tag = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var color = Glass._d(o, "color", t.info);
  var st = { background: solid(withAlpha(color, 0.18)), border_width: 1, border_color: withAlpha(color, 0.5),
             padding: { top: 3, right: 10, bottom: 3, left: 10 },
             radius: { tl: 8, tr: 8, br: 8, bl: 8 }, align_items: "center" };
  var n = row({ style: st, children: [ text("" + Glass._d(o, "label", ""), { size: t.font.xs, weight: "semibold", foreground: color }) ] });
  return Glass._fin(n, o);
};
Glass.avatar = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var sz = Glass._d(o, "size", 44);
  var st = { width: Glass._len(sz), height: Glass._len(sz),
             radius: { tl: sz, tr: sz, br: sz, bl: sz }, align_items: "center", justify_content: "center",
             border_width: 2, border_color: t.glassRim, overflow_x: "hidden", overflow_y: "hidden" };
  var inner;
  if (has(o, "src")) { inner = image(o.src, { fit: "cover" }); }
  else {
    st.background = linearGradient(135, [ stop(0, t.accent), stop(1, t.accent2) ]);
    inner = text("" + Glass._d(o, "initials", "?"), { size: sz * 0.4, weight: "bold", foreground: t.text });
  }
  var n = stack({ style: st, children: [inner] });
  return Glass._fin(n, o);
};
Glass.avatarGroup = function (o) {
  o = Glass._opt(o);
  var avatars = Glass._map(Glass._d(o, "items", []), function (it) { return Glass.avatar(it); });
  return Glass._fin(row({ style: { gap: -10 }, children: avatars }), o);
};
Glass.kbd = function (o) {
  o = Glass._opt(o);
  var st = { padding: { top: 2, right: 7, bottom: 2, left: 7 }, radius: { tl: 6, tr: 6, br: 6, bl: 6 },
             background: solid(hexA("#FFFFFF", 0.12)), border_width: 1, border_color: hexA("#FFFFFF", 0.2) };
  return Glass._fin(div({ style: st, children: [ text("" + Glass._d(o, "text", ""), { size: Glass.tokens.font.xs, weight: "semibold", foreground: Glass.tokens.text }) ] }), o);
};
Glass.icon = function (o) {
  o = Glass._opt(o);
  return Glass._fin(icon(Glass._d(o, "name", "circle"), { size: Glass._d(o, "size", 24), color: Glass._d(o, "color", Glass.tokens.text) }), o);
};

// ===========================================================================
// Forms & inputs
// ===========================================================================

// A labeled field wrapper: label + control + optional helper/error text.
Glass.field = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var kids = [];
  if (has(o, "label")) { push(kids, Glass.label({ text: o.label })); }
  if (has(o, "control")) { push(kids, o.control); }
  kids = concat(kids, Glass._kids(o));
  if (has(o, "error")) { push(kids, Glass.text({ text: o.error, size: t.font.xs, color: t.danger })); }
  else if (has(o, "help")) { push(kids, Glass.caption({ text: o.help })); }
  return Glass._fin(column({ style: { gap: 6 }, children: kids }), o);
};

Glass.textField = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var mat = Glass.material("thin"); mat.radius = t.radius.md;
  var props = { value: "" + Glass._d(o, "value", ""), input_type: Glass._d(o, "type", "text") };
  if (has(o, "placeholder")) { props.placeholder = o.placeholder; }
  if (has(o, "disabled")) { props.disabled = o.disabled; }
  if (has(o, "readonly")) { props.readonly = o.readonly; }
  if (has(o, "maxLength")) { props.max_length = o.maxLength; }
  props.style = { glass_material: mat, padding: { top: 10, right: 14, bottom: 10, left: 14 }, foreground: t.text };
  var n = textInput(props);
  if (has(o, "onInput")) { on(n, "input", o.onInput); }
  if (has(o, "onChange")) { on(n, "change", o.onChange); }
  if (has(o, "onSubmit")) { on(n, "submit", o.onSubmit); }
  if (has(o, "label") || has(o, "error") || has(o, "help")) {
    return Glass.field(merge(o, { control: Glass._fin(n, {}) }));
  }
  return Glass._fin(n, o);
};
Glass.textArea = function (o) { return Glass.textField(merge({ type: "multiline" }, Glass._opt(o))); };
Glass.passwordField = function (o) { return Glass.textField(merge({ type: "password" }, Glass._opt(o))); };
Glass.numberField = function (o) { return Glass.textField(merge({ type: "number" }, Glass._opt(o))); };
Glass.search = function (o) {
  o = merge({ placeholder: "Search…", type: "text" }, Glass._opt(o));
  return Glass.textField(o);
};

Glass.checkbox = function (o) {
  o = Glass._opt(o);
  var props = { checked: Glass._d(o, "checked", false) };
  if (has(o, "label")) { props.label = o.label; }
  if (has(o, "disabled")) { props.disabled = o.disabled; }
  var n = checkbox(props);
  if (has(o, "onChange")) { on(n, "change", o.onChange); }
  return Glass._fin(n, o);
};
// A switch / toggle (named `toggle` to avoid the reserved word `switch`;
// `Glass["switch"]` is provided below as a convenience alias).
Glass.toggle = function (o) {
  o = Glass._opt(o);
  var props = { checked: Glass._d(o, "checked", false) };
  if (has(o, "label")) { props.label = o.label; }
  if (has(o, "disabled")) { props.disabled = o.disabled; }
  var n = toggle(props);
  if (has(o, "onChange")) { on(n, "change", o.onChange); }
  return Glass._fin(n, o);
};
Glass["switch"] = Glass.toggle;
Glass.radioGroup = function (o) {
  o = Glass._opt(o);
  var props = { options: Glass._d(o, "options", []) };
  if (has(o, "selected")) { props.selected = o.selected; }
  if (has(o, "disabled")) { props.disabled = o.disabled; }
  var n = radio(props);
  if (has(o, "onChange")) { on(n, "change", o.onChange); }
  return Glass._fin(n, o);
};
Glass.slider = function (o) {
  o = Glass._opt(o);
  var props = { value: Glass._d(o, "value", 0.5), min: Glass._d(o, "min", 0), max: Glass._d(o, "max", 1) };
  if (has(o, "step")) { props.step = o.step; }
  if (has(o, "disabled")) { props.disabled = o.disabled; }
  var n = slider(props);
  if (has(o, "onChange")) { on(n, "change", o.onChange); }
  return Glass._fin(n, o);
};
Glass.rangeSlider = function (o) {
  o = Glass._opt(o);
  var props = { value: Glass._d(o, "start", 0.2), value_end: Glass._d(o, "end", 0.8),
                min: Glass._d(o, "min", 0), max: Glass._d(o, "max", 1) };
  var n = slider(props);
  if (has(o, "onChange")) { on(n, "change", o.onChange); }
  return Glass._fin(n, o);
};
Glass.select = function (o) {
  o = Glass._opt(o);
  var props = { options: Glass._d(o, "options", []), multi: Glass._d(o, "multi", false) };
  if (has(o, "selected")) { props.selected = o.selected; }
  if (has(o, "placeholder")) { props.placeholder = o.placeholder; }
  if (has(o, "disabled")) { props.disabled = o.disabled; }
  var n = dropdown(props);
  if (has(o, "onChange")) { on(n, "change", o.onChange); }
  return Glass._fin(n, o);
};
Glass.multiSelect = function (o) { return Glass.select(merge({ multi: true }, Glass._opt(o))); };

// A +/- numeric stepper built from two icon buttons and a value.
Glass.stepper = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var dec = Glass.iconButton({ icon: "minus", size: 34, onClick: Glass._d(o, "onDecrement", "") });
  var inc = Glass.iconButton({ icon: "plus", size: 34, onClick: Glass._d(o, "onIncrement", "") });
  var val = row({ style: { width: Glass._len(48), align_items: "center", justify_content: "center" },
                  children: [ text("" + Glass._d(o, "value", 0), { size: t.font.md, weight: "semibold", foreground: t.text }) ] });
  return Glass._fin(row({ style: { gap: 4, align_items: "center" }, children: [dec, val, inc] }), o);
};

// A 0..max star rating row.
Glass.rating = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var max = Glass._d(o, "max", 5);
  var value = Glass._d(o, "value", 0);
  var stars = [];
  for (var i = 0; i < max; i = i + 1) {
    var filled = (i < value);
    var s = icon(filled ? "star-filled" : "star", { size: Glass._d(o, "size", 22), color: filled ? t.warning : t.textFaint });
    if (has(o, "onRate")) { var st2 = { cursor: "pointer" }; s = withStyle(s, st2); on(s, "click", o.onRate + ":" + (i + 1)); }
    push(stars, s);
  }
  return Glass._fin(row({ style: { gap: 4 }, children: stars }), o);
};

Glass.colorSwatch = function (o) {
  o = Glass._opt(o);
  var sz = Glass._d(o, "size", 28);
  var st = { width: Glass._len(sz), height: Glass._len(sz), radius: { tl: 8, tr: 8, br: 8, bl: 8 },
             background: solid(Glass._d(o, "color", Glass.tokens.accent)), border_width: 2,
             border_color: Glass._d(o, "selected", false) ? Glass.tokens.text : Glass.tokens.glassRim, cursor: "pointer" };
  var n = div({ style: st });
  if (has(o, "onClick")) { on(n, "click", o.onClick); }
  return Glass._fin(n, o);
};

// ===========================================================================
// Navigation
// ===========================================================================

// Top app bar: leading, title, trailing actions, on a glass surface.
Glass.navbar = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var leading = [];
  if (has(o, "leading")) { leading = concat(leading, o.leading); }
  if (has(o, "title")) { push(leading, Glass.title({ text: o.title })); }
  var trailing = has(o, "trailing") ? o.trailing : [];
  var mat = Glass.material(Glass._d(o, "variant", "regular")); mat.radius = t.radius.lg;
  var bar = row({
    style: { glass_material: mat, padding: { top: 12, right: 16, bottom: 12, left: 16 },
             align_items: "center", justify_content: "between" },
    children: [
      row({ style: { gap: 12, align_items: "center" }, children: leading }),
      row({ style: { gap: 8, align_items: "center" }, children: trailing })
    ]
  });
  return Glass._fin(bar, o);
};

// Bottom tab bar: an array of { icon, label } items, with a selected index.
Glass.tabBar = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var items = Glass._d(o, "items", []);
  var sel = Glass._d(o, "selected", 0);
  var cells = Glass._map(items, function (it, i) {
    var active = (i == sel);
    var col = active ? t.accent : t.textDim;
    var cell = column({ style: { align_items: "center", justify_content: "center", gap: 3, flex_grow: 1, cursor: "pointer" },
      children: [
        icon(Glass._d(it, "icon", "circle"), { size: 22, color: col }),
        text(Glass._d(it, "label", ""), { size: t.font.xs, weight: active ? "semibold" : "normal", foreground: col })
      ] });
    if (has(o, "onSelect")) { on(cell, "click", o.onSelect + ":" + i); }
    return cell;
  });
  var mat = Glass.material("thick"); mat.radius = t.radius.xl;
  return Glass._fin(row({ style: { glass_material: mat, padding: { top: 10, right: 8, bottom: 10, left: 8 } }, children: cells }), o);
};

// Content tabs (uses the native tabs widget with a glass frame).
Glass.tabs = function (o) {
  o = Glass._opt(o);
  var n = tabs({ tabs: Glass._d(o, "tabs", []), selected: Glass._d(o, "selected", 0) });
  if (has(o, "onSelect")) { on(n, "change", o.onSelect); }
  return Glass._fin(n, o);
};

Glass.breadcrumbs = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var items = Glass._d(o, "items", []);
  var parts = [];
  for (var i = 0; i < len(items); i = i + 1) {
    var it = items[i];
    var last = (i == len(items) - 1);
    var lbl = typeOf(it) == "object" ? it.label : it;
    var txt = Glass.text({ text: lbl, size: t.font.sm, color: last ? t.text : t.textDim, weight: last ? "semibold" : "normal" });
    if (!last && has(o, "onSelect") && typeOf(it) == "object") { on(txt, "click", o.onSelect + ":" + it.value); txt.style = merge(txt.style, { cursor: "pointer" }); }
    push(parts, txt);
    if (!last) { push(parts, Glass.text({ text: "/", size: t.font.sm, color: t.textFaint })); }
  }
  return Glass._fin(row({ style: { gap: 8, align_items: "center" }, children: parts }), o);
};

Glass.pagination = function (o) {
  o = Glass._opt(o);
  var pages = Glass._d(o, "pages", 1);
  var cur = Glass._d(o, "current", 0);
  var cells = [];
  push(cells, Glass.iconButton({ icon: "chevron-left", size: 34, onClick: Glass._d(o, "onPrev", "") }));
  for (var i = 0; i < pages; i = i + 1) {
    var active = (i == cur);
    var c = Glass.iconButton({ icon: "circle", size: 34, variant: active ? "accent" : "thin" });
    c.children = [ text("" + (i + 1), { size: Glass.tokens.font.sm, weight: active ? "bold" : "normal",
                                        foreground: Glass.tokens.text }) ];
    if (has(o, "onSelect")) { on(c, "click", o.onSelect + ":" + i); }
    push(cells, c);
  }
  push(cells, Glass.iconButton({ icon: "chevron-right", size: 34, onClick: Glass._d(o, "onNext", "") }));
  return Glass._fin(row({ style: { gap: 6, align_items: "center" }, children: cells }), o);
};

// A vertical menu of selectable rows on a glass surface.
Glass.menu = function (o) {
  o = Glass._opt(o);
  var items = Glass._map(Glass._d(o, "items", []), function (it) { return Glass.menuItem(it); });
  return Glass.card(merge({ variant: "regular", padding: 6, gap: 2, children: items }, o));
};
Glass.menuItem = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var content = [];
  if (has(o, "icon")) { push(content, icon(o.icon, { size: 18, color: t.textDim })); }
  push(content, text("" + Glass._d(o, "label", ""), { size: t.font.sm, foreground: Glass._d(o, "danger", false) ? t.danger : t.text }));
  if (has(o, "shortcut")) { push(content, Glass.spacer({})); push(content, Glass.kbd({ text: o.shortcut })); }
  var st = { padding: { top: 9, right: 12, bottom: 9, left: 12 }, radius: { tl: 10, tr: 10, br: 10, bl: 10 },
             gap: 10, align_items: "center", cursor: "pointer" };
  if (Glass._d(o, "active", false)) { st.background = solid(hexA("#FFFFFF", 0.08)); }
  var n = row({ style: st, children: content });
  if (has(o, "onClick")) { on(n, "click", o.onClick); }
  return Glass._fin(n, o);
};

// A side navigation rail.
Glass.sidebar = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var items = Glass._map(Glass._d(o, "items", []), function (it, i) {
    var mi = merge(it, { active: (i == Glass._d(o, "selected", 0)) });
    if (has(o, "onSelect")) { mi.onClick = o.onSelect + ":" + i; }
    return Glass.menuItem(mi);
  });
  var kids = [];
  if (has(o, "header")) { push(kids, o.header); push(kids, Glass.divider({})); }
  kids = concat(kids, items);
  var mat = Glass.material("thin"); mat.radius = t.radius.xl;
  return Glass._fin(column({ style: { glass_material: mat, width: Glass._len(Glass._d(o, "width", 240)),
                                      height: Glass._len("full"), padding: { top: 12, right: 10, bottom: 12, left: 10 }, gap: 4 },
                             children: kids }), o);
};

// ===========================================================================
// Overlays
// ===========================================================================

// Modal dialog: a dimmed backdrop overlay holding a glass sheet.
Glass.modal = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var kids = [];
  if (has(o, "title")) { push(kids, Glass.title({ text: o.title })); }
  kids = concat(kids, Glass._kids(o));
  if (has(o, "actions")) { push(kids, row({ style: { gap: 10, justify_content: "end" }, children: o.actions })); }
  var sheet = Glass.card({ variant: "thick", padding: t.space.xl, gap: t.space.lg,
                           style: { width: Glass._len(Glass._d(o, "width", 420)), max_width: Glass._len("full") }, children: kids });
  var ov = overlay({ layer: "modal", backdrop: true, dismissible: Glass._d(o, "dismissible", true),
                     style: { align_items: "center", justify_content: "center", padding: { top: 24, right: 24, bottom: 24, left: 24 } },
                     children: [sheet] });
  if (has(o, "onDismiss")) { on(ov, "dismiss", o.onDismiss); }
  return Glass._fin(ov, o);
};

// A sheet anchored to the bottom of the screen.
Glass.bottomSheet = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var handle = div({ style: { width: Glass._len(40), height: Glass._len(4), radius: { tl: 4, tr: 4, br: 4, bl: 4 },
                              background: solid(t.glassRim), align_self: "center" } });
  var body = Glass.card({ variant: "thick", padding: t.space.xl, gap: t.space.md,
                          style: { width: Glass._len("full") }, children: concat([handle], Glass._kids(o)) });
  var ov = overlay({ layer: "modal", backdrop: true, dismissible: Glass._d(o, "dismissible", true),
                     style: { align_items: "stretch", justify_content: "end" }, children: [body] });
  if (has(o, "onDismiss")) { on(ov, "dismiss", o.onDismiss); }
  return Glass._fin(ov, o);
};

// A slide-in drawer (left or right) as a modal overlay.
Glass.drawer = function (o) {
  o = Glass._opt(o);
  var side = Glass._d(o, "side", "left");
  var body = Glass.sidebar(merge(o, { width: Glass._d(o, "width", 280) }));
  var ov = overlay({ layer: "modal", backdrop: true, dismissible: Glass._d(o, "dismissible", true),
                     style: { align_items: "stretch", justify_content: side == "right" ? "end" : "start" }, children: [body] });
  if (has(o, "onDismiss")) { on(ov, "dismiss", o.onDismiss); }
  return Glass._fin(ov, o);
};

// A popover anchored layer (positioned by the host's popover layer).
Glass.popover = function (o) {
  o = Glass._opt(o);
  var body = Glass.card({ variant: "regular", padding: Glass.tokens.space.md, children: Glass._kids(o) });
  var ov = overlay({ layer: "popover", backdrop: false, dismissible: Glass._d(o, "dismissible", true), children: [body] });
  if (has(o, "onDismiss")) { on(ov, "dismiss", o.onDismiss); }
  return Glass._fin(ov, o);
};

// A tooltip layer.
Glass.tooltip = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var body = row({ style: { glass_material: Glass.material("thick"), padding: { top: 6, right: 10, bottom: 6, left: 10 } },
                   children: [ text("" + Glass._d(o, "text", ""), { size: t.font.xs, foreground: t.text }) ] });
  return Glass._fin(overlay({ layer: "tooltip", backdrop: false, dismissible: false, children: [body] }), o);
};

// A toast (transient notification) in the toast layer.
Glass.toast = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var color = Glass._d(o, "color", t.accent);
  var content = [ Glass.dot({ color: color, size: 10 }) ];
  if (has(o, "title")) { push(content, text(o.title, { size: t.font.sm, weight: "semibold", foreground: t.text })); }
  if (has(o, "message")) { push(content, text(o.message, { size: t.font.sm, foreground: t.textDim })); }
  var body = row({ style: { glass_material: Glass.material("thick"), padding: { top: 12, right: 16, bottom: 12, left: 16 },
                            gap: 10, align_items: "center" }, children: content });
  return Glass._fin(overlay({ layer: "toast", backdrop: false, dismissible: true, children: [body] }), o);
};

// A persistent snackbar (toast with an action button).
Glass.snackbar = function (o) {
  o = Glass._opt(o);
  var content = [ text("" + Glass._d(o, "message", ""), { size: Glass.tokens.font.sm, foreground: Glass.tokens.text }), Glass.spacer({}) ];
  if (has(o, "action")) { push(content, Glass.button({ label: o.action, variant: "ghost", size: "sm", onClick: Glass._d(o, "onAction", "") })); }
  var body = row({ style: { glass_material: Glass.material("thick"), padding: { top: 10, right: 12, bottom: 10, left: 16 },
                            gap: 12, align_items: "center", width: Glass._len("full") }, children: content });
  return Glass._fin(overlay({ layer: "toast", backdrop: false, dismissible: true, children: [body] }), o);
};

// An inline banner / alert.
Glass.alert = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var kind = Glass._d(o, "kind", "info");
  var palette = { info: t.info, success: t.success, warning: t.warning, danger: t.danger };
  var color = get(palette, kind, t.info);
  var iconName = kind == "success" ? "check" : (kind == "danger" ? "alert-triangle" : (kind == "warning" ? "alert-circle" : "info-circle"));
  var body = [];
  if (has(o, "title")) { push(body, text(o.title, { size: t.font.sm, weight: "bold", foreground: t.text })); }
  if (has(o, "message")) { push(body, text(o.message, { size: t.font.sm, foreground: t.textDim })); }
  var st = { background: solid(withAlpha(color, 0.14)), border_width: 1, border_color: withAlpha(color, 0.4),
             radius: { tl: t.radius.md, tr: t.radius.md, br: t.radius.md, bl: t.radius.md },
             padding: { top: 12, right: 14, bottom: 12, left: 14 }, gap: 12, align_items: "start" };
  var n = row({ style: st, children: [ icon(iconName, { size: 20, color: color }), column({ style: { gap: 3 }, children: body }) ] });
  return Glass._fin(n, o);
};
Glass.banner = Glass.alert;

// A full-surface loading overlay.
Glass.loadingOverlay = function (o) {
  o = Glass._opt(o);
  var body = column({ style: { glass_material: Glass.material("thick"), padding: { top: 24, right: 32, bottom: 24, left: 32 },
                               gap: 14, align_items: "center" },
                      children: [ spinner({ size: 36, color: Glass.tokens.accent }),
                                  text("" + Glass._d(o, "text", "Loading…"), { size: Glass.tokens.font.sm, foreground: Glass.tokens.textDim }) ] });
  return Glass._fin(overlay({ layer: "modal", backdrop: true, dismissible: false,
                              style: { align_items: "center", justify_content: "center" }, children: [body] }), o);
};

// ===========================================================================
// Feedback: progress, spinners, skeletons, empty states
// ===========================================================================

Glass.progress = function (o) {
  o = Glass._opt(o);
  var n = progressBar({ value: has(o, "value") ? o.value : null, shape: "linear" });
  return Glass._fin(n, o);
};
Glass.progressCircle = function (o) {
  o = Glass._opt(o);
  var n = progressBar({ value: has(o, "value") ? o.value : null, shape: "circular" });
  return Glass._fin(n, o);
};
Glass.spinner = function (o) {
  o = Glass._opt(o);
  return Glass._fin(spinner({ size: Glass._d(o, "size", 28), color: Glass._d(o, "color", Glass.tokens.accent) }), o);
};
Glass.skeleton = function (o) {
  o = Glass._opt(o);
  var st = { width: Glass._len(Glass._d(o, "width", "full")), height: Glass._len(Glass._d(o, "height", 16)),
             radius: { tl: 8, tr: 8, br: 8, bl: 8 }, background: solid(hexA("#FFFFFF", 0.08)) };
  return Glass._fin(div({ style: st }), o);
};
Glass.emptyState = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var kids = [ icon(Glass._d(o, "icon", "inbox"), { size: 48, color: t.textFaint }) ];
  if (has(o, "title")) { push(kids, Glass.title({ text: o.title, align: "center" })); }
  if (has(o, "message")) { push(kids, Glass.subtitle({ text: o.message, align: "center" })); }
  if (has(o, "action")) { push(kids, Glass.button({ label: o.action, variant: "accent", onClick: Glass._d(o, "onAction", "") })); }
  return Glass._fin(column({ style: { align_items: "center", justify_content: "center", gap: 12,
                                      padding: { top: 40, right: 24, bottom: 40, left: 24 } }, children: kids }), o);
};

// ===========================================================================
// Data display
// ===========================================================================

Glass.list = function (o) {
  o = Glass._opt(o);
  var items = Glass._d(o, "items", []);
  var rows = [];
  for (var i = 0; i < len(items); i = i + 1) {
    push(rows, Glass.listItem(items[i]));
    if (Glass._d(o, "dividers", true) && i < len(items) - 1) { push(rows, Glass.divider({ style: { opacity: 0.25 } })); }
  }
  return Glass.card(merge({ variant: "thin", padding: 6, gap: 0, children: rows }, o));
};
Glass.listItem = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var leading = [];
  if (has(o, "avatar")) { push(leading, Glass.avatar(o.avatar)); }
  else if (has(o, "icon")) { push(leading, icon(o.icon, { size: 22, color: t.textDim })); }
  var texts = [ text("" + Glass._d(o, "title", ""), { size: t.font.md, weight: "medium", foreground: t.text }) ];
  if (has(o, "subtitle")) { push(texts, text(o.subtitle, { size: t.font.sm, foreground: t.textDim })); }
  var mid = column({ style: { gap: 2, flex_grow: 1 }, children: texts });
  var trailing = [];
  if (has(o, "trailing")) { trailing = concat(trailing, typeOf(o.trailing) == "array" ? o.trailing : [o.trailing]); }
  else if (Glass._d(o, "chevron", false)) { push(trailing, icon("chevron-right", { size: 18, color: t.textFaint })); }
  var children = concat(concat(leading, [mid]), trailing);
  var st = { padding: { top: 10, right: 12, bottom: 10, left: 12 }, gap: 12, align_items: "center",
             radius: { tl: 12, tr: 12, br: 12, bl: 12 } };
  if (has(o, "onClick")) { st.cursor = "pointer"; }
  var n = row({ style: st, children: children });
  if (has(o, "onClick")) { on(n, "click", o.onClick); }
  return Glass._fin(n, o);
};

// A simple data table: columns [{ key, label }] + rows [{ ... }].
Glass.table = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var cols = Glass._d(o, "columns", []);
  var headerCells = Glass._map(cols, function (c) {
    var lbl = typeOf(c) == "object" ? Glass._d(c, "label", "") : c;
    return row({ style: { flex_grow: 1 }, children: [ text("" + lbl, { size: t.font.xs, weight: "bold", foreground: t.textDim }) ] });
  });
  var header = row({ style: { padding: { top: 8, right: 12, bottom: 8, left: 12 }, gap: 12 }, children: headerCells });
  var bodyRows = Glass._map(Glass._d(o, "rows", []), function (rowData) {
    var cells = Glass._map(cols, function (c) {
      var key = typeOf(c) == "object" ? c.key : c;
      return row({ style: { flex_grow: 1 }, children: [ text("" + get(rowData, key, ""), { size: t.font.sm, foreground: t.text }) ] });
    });
    return row({ style: { padding: { top: 9, right: 12, bottom: 9, left: 12 }, gap: 12,
                          border_width: 1, border_color: hexA("#FFFFFF", 0.06) }, children: cells });
  });
  return Glass.card(merge({ variant: "thin", padding: 4, gap: 0, children: concat([header], bodyRows) }, o));
};

// A KPI / metric tile.
Glass.stat = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var kids = [];
  if (has(o, "label")) { push(kids, Glass.label({ text: o.label })); }
  push(kids, text("" + Glass._d(o, "value", "—"), { size: t.font.xxl, weight: "bold", foreground: t.text }));
  if (has(o, "delta")) {
    var up = Glass._d(o, "deltaUp", true);
    push(kids, row({ style: { gap: 4, align_items: "center" }, children: [
      icon(up ? "trending-up" : "trending-down", { size: 16, color: up ? t.success : t.danger }),
      text("" + o.delta, { size: t.font.sm, weight: "semibold", foreground: up ? t.success : t.danger }) ] }));
  }
  return Glass.card(merge({ variant: "regular", gap: 6, children: kids }, o));
};

// A label : value row pair.
Glass.keyValue = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var n = row({ style: { justify_content: "between", align_items: "center", padding: { top: 6, right: 0, bottom: 6, left: 0 } },
    children: [ text("" + Glass._d(o, "label", ""), { size: t.font.sm, foreground: t.textDim }),
                text("" + Glass._d(o, "value", ""), { size: t.font.sm, weight: "semibold", foreground: t.text }) ] });
  return Glass._fin(n, o);
};

// A vertical timeline of events [{ title, time, color }].
Glass.timeline = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var rows = Glass._map(Glass._d(o, "items", []), function (it, i) {
    var marker = column({ style: { align_items: "center", gap: 0 }, children: [
      Glass.dot({ color: Glass._d(it, "color", t.accent), size: 12 }),
      div({ style: { width: Glass._len(2), height: Glass._len(28), background: solid(t.glassRim), opacity: 0.4 } }) ] });
    var body = column({ style: { gap: 2 }, children: [
      text("" + Glass._d(it, "title", ""), { size: t.font.sm, weight: "semibold", foreground: t.text }),
      text("" + Glass._d(it, "time", ""), { size: t.font.xs, foreground: t.textFaint }) ] });
    return row({ style: { gap: 12, align_items: "start" }, children: [marker, body] });
  });
  return Glass._fin(column({ style: { gap: 0 }, children: rows }), o);
};

// Accordion: an array of { title, body, open } panels.
Glass.accordion = function (o) {
  o = Glass._opt(o);
  var panels = Glass._map(Glass._d(o, "items", []), function (it, i) {
    var pit = merge(it, {});
    if (has(o, "onToggle")) { pit.onToggle = o.onToggle + ":" + i; }
    return Glass.collapsible(pit);
  });
  return Glass._fin(column({ style: { gap: 8 }, children: panels }), o);
};
Glass.collapsible = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var open = Glass._d(o, "open", false);
  var header = row({ style: { justify_content: "between", align_items: "center", cursor: "pointer",
                              padding: { top: 12, right: 14, bottom: 12, left: 14 } }, children: [
    text("" + Glass._d(o, "title", ""), { size: t.font.md, weight: "semibold", foreground: t.text }),
    icon(open ? "chevron-up" : "chevron-down", { size: 18, color: t.textDim }) ] });
  if (has(o, "onToggle")) { on(header, "click", o.onToggle); }
  var kids = [header];
  if (open) {
    var body = has(o, "body") ? o.body : column({ style: { gap: 8 }, children: Glass._kids(o) });
    push(kids, div({ style: { padding: { top: 0, right: 14, bottom: 14, left: 14 } }, children: [body] }));
  }
  return Glass.card(merge({ variant: "thin", padding: 0, gap: 0, children: kids }, o));
};

// ===========================================================================
// Media
// ===========================================================================

Glass.image = function (o) {
  o = Glass._opt(o);
  var st = { radius: { tl: Glass._d(o, "radius", Glass.tokens.radius.md), tr: Glass._d(o, "radius", Glass.tokens.radius.md),
                       br: Glass._d(o, "radius", Glass.tokens.radius.md), bl: Glass._d(o, "radius", Glass.tokens.radius.md) },
             overflow_x: "hidden", overflow_y: "hidden" };
  if (has(o, "width")) { st.width = Glass._len(o.width); }
  if (has(o, "height")) { st.height = Glass._len(o.height); }
  var props = { fit: Glass._d(o, "fit", "cover"), style: st };
  if (has(o, "alt")) { props.alt = o.alt; }
  if (has(o, "placeholder")) { props.placeholder = o.placeholder; }
  return Glass._fin(image(Glass._d(o, "src", ""), props), o);
};
Glass.video = function (o) {
  o = Glass._opt(o);
  var props = { autoplay: Glass._d(o, "autoplay", false), loop_: Glass._d(o, "loop", false),
                muted: Glass._d(o, "muted", false), controls: Glass._d(o, "controls", true), fit: Glass._d(o, "fit", "contain") };
  return Glass._fin(video(Glass._d(o, "src", ""), props), o);
};
Glass.audioPlayer = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var media = audio(Glass._d(o, "src", ""), { controls: false, autoplay: Glass._d(o, "autoplay", false) });
  var controls = row({ style: { gap: 12, align_items: "center", flex_grow: 1 }, children: [
    Glass.iconButton({ icon: Glass._d(o, "playing", false) ? "pause" : "play", variant: "accent", size: 44, onClick: Glass._d(o, "onToggle", "") }),
    column({ style: { gap: 4, flex_grow: 1 }, children: [
      text("" + Glass._d(o, "title", "Audio"), { size: t.font.sm, weight: "semibold", foreground: t.text }),
      progressBar({ value: Glass._d(o, "progress", 0), shape: "linear" }) ] }) ] });
  return Glass.card(merge({ variant: "regular", children: [media, controls] }, o));
};
Glass.carousel = function (o) {
  o = Glass._opt(o);
  var n = carousel({ index: Glass._d(o, "index", 0), indicators: Glass._d(o, "indicators", true),
                     autoplay: Glass._d(o, "autoplay", false), interval: Glass._d(o, "interval", 0) });
  n.children = Glass._kids(o);
  if (has(o, "onChange")) { on(n, "change", o.onChange); }
  return Glass._fin(n, o);
};
Glass.gallery = function (o) {
  o = Glass._opt(o);
  var imgs = Glass._map(Glass._d(o, "items", []), function (src) {
    return Glass.image({ src: src, width: Glass._d(o, "size", 120), height: Glass._d(o, "size", 120) });
  });
  return Glass._fin(row({ style: { gap: 10, wrap: true }, children: imgs }), o);
};

// A glass-framed 3D scene viewport.
Glass.scene = function (o) {
  o = Glass._opt(o);
  var props = merge({ animated: Glass._d(o, "animated", true) }, {});
  if (has(o, "camera")) { props.camera = o.camera; }
  if (has(o, "lights")) { props.lights = o.lights; }
  if (has(o, "entities")) { props.entities = o.entities; }
  props.style = { width: Glass._len(Glass._d(o, "width", "full")), height: Glass._len(Glass._d(o, "height", 320)),
                  radius: { tl: 22, tr: 22, br: 22, bl: 22 }, overflow_x: "hidden", overflow_y: "hidden",
                  border_width: 1, border_color: Glass.tokens.glassRim };
  return Glass._fin(scene3d(props), o);
};

// ===========================================================================
// Charts (immediate-mode 2D canvas)
// ===========================================================================

// A circular progress ring, drawn with arcs.
Glass.ring = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var size = Glass._d(o, "size", 120);
  var value = Glass._d(o, "value", 0.66);
  var width = Glass._d(o, "thickness", 12);
  var cx = size / 2; var cy = size / 2; var r = (size - width) / 2;
  var color = Glass._d(o, "color", t.accent);
  var ops = [
    { type: "clear", color: rgba(0, 0, 0, 0) },
    { type: "arc", center: { x: cx, y: cy }, radius: r, start_angle: -90, end_angle: 270,
      stroke: { brush: solid(hexA("#FFFFFF", 0.12)), width: width, cap: "round" } },
    { type: "arc", center: { x: cx, y: cy }, radius: r, start_angle: -90, end_angle: -90 + 360 * value,
      stroke: { brush: solid(color), width: width, cap: "round" } }
  ];
  var label = text("" + round(value * 100) + "%", { size: t.font.lg, weight: "bold", foreground: t.text });
  var st = { width: Glass._len(size), height: Glass._len(size), align_items: "center", justify_content: "center" };
  return Glass._fin(stack({ style: st, children: [ canvas(ops, { style: { width: Glass._len(size), height: Glass._len(size) } }), label ] }), o);
};

// A vertical bar chart.
Glass.barChart = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var data = Glass._d(o, "data", []);
  var w = Glass._d(o, "width", 320); var h = Glass._d(o, "height", 160);
  var pad = 12; var n = len(data);
  var maxV = 1;
  for (var i = 0; i < n; i = i + 1) { if (data[i] > maxV) { maxV = data[i]; } }
  var bw = n > 0 ? (w - pad * 2) / n * 0.62 : 0;
  var gap = n > 0 ? (w - pad * 2) / n * 0.38 : 0;
  var ops = [ { type: "clear", color: rgba(0, 0, 0, 0) } ];
  for (var j = 0; j < n; j = j + 1) {
    var bh = (h - pad * 2) * (data[j] / maxV);
    var x = pad + j * (bw + gap);
    var y = h - pad - bh;
    push(ops, { type: "fill_round_rect", rect: { x: x, y: y, w: bw, h: bh }, radius: 6,
      brush: linearGradient(90, [ stop(0, Glass._d(o, "color", t.accent)), stop(1, t.accent2) ]) });
  }
  return Glass._fin(canvas(ops, { style: { width: Glass._len(w), height: Glass._len(h) } }), o);
};

// A line / area sparkline.
Glass.lineChart = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var data = Glass._d(o, "data", []);
  var w = Glass._d(o, "width", 320); var h = Glass._d(o, "height", 120);
  var pad = 10; var n = len(data);
  var maxV = 0.0001; var minV = 0;
  for (var i = 0; i < n; i = i + 1) { if (data[i] > maxV) { maxV = data[i]; } }
  var pts = [];
  for (var j = 0; j < n; j = j + 1) {
    var x = pad + (n > 1 ? (w - pad * 2) * j / (n - 1) : 0);
    var y = h - pad - (h - pad * 2) * (data[j] - minV) / (maxV - minV);
    push(pts, { x: x, y: y });
  }
  var ops = [ { type: "clear", color: rgba(0, 0, 0, 0) } ];
  if (n > 1) { push(ops, { type: "polyline", points: pts, stroke: { brush: solid(Glass._d(o, "color", t.accent)), width: 3, cap: "round", join: "round" } }); }
  return Glass._fin(canvas(ops, { style: { width: Glass._len(w), height: Glass._len(h) } }), o);
};

// A semicircular gauge.
Glass.gauge = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var size = Glass._d(o, "size", 160);
  var value = Glass._d(o, "value", 0.5);
  var cx = size / 2; var cy = size * 0.7; var r = size * 0.4; var width = 14;
  var ops = [
    { type: "clear", color: rgba(0, 0, 0, 0) },
    { type: "arc", center: { x: cx, y: cy }, radius: r, start_angle: 180, end_angle: 360,
      stroke: { brush: solid(hexA("#FFFFFF", 0.12)), width: width, cap: "round" } },
    { type: "arc", center: { x: cx, y: cy }, radius: r, start_angle: 180, end_angle: 180 + 180 * value,
      stroke: { brush: linearGradient(0, [ stop(0, t.success), stop(0.5, t.warning), stop(1, t.danger) ]), width: width, cap: "round" } }
  ];
  return Glass._fin(canvas(ops, { style: { width: Glass._len(size), height: Glass._len(size * 0.8) } }), o);
};

// ===========================================================================
// Decorative
// ===========================================================================

// A soft "liquid blob" backdrop drawn on a canvas — animate by passing a
// phase that shifts each frame.
Glass.blob = function (o) {
  o = Glass._opt(o);
  var t = Glass.tokens;
  var w = Glass._d(o, "width", 480); var h = Glass._d(o, "height", 320);
  var phase = Glass._d(o, "phase", 0);
  var ops = [ { type: "clear", color: rgba(0, 0, 0, 0) } ];
  var blobs = [
    { cx: w * 0.3, cy: h * 0.35, r: 120, c: t.accent },
    { cx: w * 0.7, cy: h * 0.6, r: 150, c: t.accent2 },
    { cx: w * 0.5, cy: h * 0.5, r: 90, c: t.info }
  ];
  for (var i = 0; i < len(blobs); i = i + 1) {
    var b = blobs[i];
    var dx = sin(phase + i) * 30; var dy = cos(phase * 0.8 + i) * 24;
    push(ops, { type: "fill_circle", center: { x: b.cx + dx, y: b.cy + dy }, radius: b.r, brush: solid(withAlpha(b.c, 0.5)) });
  }
  return Glass._fin(canvas(ops, { animated: Glass._d(o, "animated", false),
                                  style: { width: Glass._len(w), height: Glass._len(h), overflow_x: "hidden", overflow_y: "hidden" } }), o);
};

// A radial glow halo behind a focal element.
Glass.glow = function (o) {
  o = Glass._opt(o);
  var color = Glass._d(o, "color", Glass.tokens.accent);
  var st = { width: Glass._len(Glass._d(o, "size", 220)), height: Glass._len(Glass._d(o, "size", 220)),
             radius: { tl: 9999, tr: 9999, br: 9999, bl: 9999 },
             background: radialGradient([ stop(0, withAlpha(color, 0.5)), stop(1, withAlpha(color, 0)) ]) };
  return Glass._fin(div({ style: st }), o);
};
