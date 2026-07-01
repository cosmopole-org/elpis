// ===========================================================================
// Material UI Kit — a faithful Material Design 3 (Flutter `material` library)
// component SDK for Elpis Miniapps.
//
// Where the Glass UI kit (sdk/glass-ui-kit.js) is a factory-function library
// modeled on Apple's liquid glass, this kit is built as a genuine **object
// hierarchy** mirroring Flutter's own widget classes: every component is a
// class extending a common `Widget` base (exactly `Widget` -> `StatelessWidget`
// / `StatefulWidget` -> concrete widgets, with `build()` returning the render
// tree), and Material's own supporting types are real classes too
// (`ColorScheme`, `TextTheme`, `ThemeData`, `TonalPalette`). Flutter's public
// widget API (constructor properties, default values, shapes, elevations,
// type scale, motion tokens) is reproduced from the Material 3 specification
// as closely as this engine's declarative style/animation surface allows.
//
// It is plain JavaScript that runs inside the Elpis sandbox on the Elpian VM,
// built **entirely** on the Blinc UI builders the host imports into the VM via
// the prelude (`div`, `row`, `column`, `text`, `icon`, `canvas`, …, plus the
// paint/animation helpers: `hex`/`withAlpha`, `linearGradient`, `shadow`,
// `stop`, `on`, `withStyle`, `withAnim`). It never touches the host protocol
// directly, and it never requires the Glass kit (the two are independent,
// prepend whichever one your Miniapp needs).
//
// Usage — module import is denied in the sandbox, so this file is *prepended*
// to a Miniapp, e.g.:
//
//   cargo run --bin elpis -- --lib sdk/material-ui-kit.js miniapps/material-gallery/app.js
//
// It defines one global, `Material`, exposing both the class hierarchy
// (`Material.ElevatedButton`, `Material.Card`, …) and, for every class, a
// lowercase factory wrapper for terse call sites (`Material.elevatedButton`,
// `Material.card`, …) — the two are equivalent:
//
//   function view() {
//     return Material.scaffold({
//       appBar: Material.appBar({ title: "Inbox" }),
//       body: Material.card({ children: [
//         Material.text({ text: "Welcome", variant: "headlineSmall" }),
//         Material.filledButton({ label: "Continue", onClick: "go" })
//       ] })
//     });
//   }
//   render(view());
//
//   // ...or the explicit OOP form:
//   var btn = new Material.FilledButton({ label: "Continue", onClick: "go" }).build();
//
// Every class is also a bare top-level identifier (`ElevatedButton`, `Card`,
// `Widget`, `State`, …), not just a `Material.*` property — because the
// engine's `class X extends Y` only parses a plain identifier after
// `extends`, never a member expression like `Material.Widget`. So to
// subclass anything (build your own `StatefulWidget`/`State`, or specialize
// a button), extend the bare name:
//
//   class CounterState extends State {
//     init() { this.state = { count: 0 }; }
//     build(widget) { return Material.text({ text: "" + this.state.count }); }
//   }
//   var counterState = new CounterState();
//   Material.runApp(function () { return new StatefulWidget({ state: counterState }).build(); });
//   function onEvent(ev) { if (ev.id == "inc") { counterState.setState({ count: counterState.state.count + 1 }); } }
//
// This does mean ~60 fairly generic names (`Text`, `Card`, `Icon`, `Dialog`,
// `Switch`, `State`, …) are bare globals in whatever script this file is
// prepended to, not tucked behind one namespace like the Glass kit's `Glass`
// — an inherent trade-off of supporting real `extends`-based inheritance in
// this engine. Prefer the `Material.*` spelling for everyday calls (keeps
// call sites self-documenting and sidesteps the risk entirely) and reserve
// the bare names for the moments you actually need `extends`.
//
// Conventions
//   * Every widget class takes a single `props` object in its constructor and
//     exposes `build()`, which returns a plain render-tree node.
//   * `props.children` is an array of child nodes (or a single node).
//   * Event handlers are guest handler-id strings (`onClick`, `onChange`, …),
//     exactly like the rest of the Elpis widget surface.
//   * `props.style` is extra style shallow-merged in last; `props.key` sets
//     the reconciliation key; `props.theme` overrides the ambient `ThemeData`
//     for that one widget (defaults to `Material.theme()`).
//   * Colors, elevations, shapes and type styles all flow from the ambient
//     `ThemeData` (`Material.theme()`), exactly as `Theme.of(context)` would
//     in Flutter. Call `Material.setTheme(ThemeData.dark())` to re-theme
//     everything, or generate a custom scheme with
//     `ThemeData.light({ seed: hex("#006494") })` (Material You dynamic color).
//
// A note on fidelity vs. the underlying engine
//   Elpis's native `checkbox`/`switch`/`radio`/`slider` widgets carry a fixed
//   host-side appearance the guest cannot recolor per the protocol
//   (`crates/elpis-protocol/src/node.rs`'s `ToggleSpec`/`RadioSpec`/
//   `SliderSpec` do not carry paint fields). Because exact Material color and
//   shape fidelity is the whole point of this kit, `Checkbox`/`Switch`/`Radio`
//   are reimplemented from scratch as plain divs + icons with click handlers
//   (Material's own toggles are simple tap targets, not drags, so this loses
//   nothing); `Slider`/`TextField`/`Dropdown` genuinely need native drag/caret
//   handling the sandbox can't reimplement, so those wrap the native widgets
//   and layer Material decoration (labels, tracks, fills, shape) around them.
//   Likewise there is no continuous pointer-tracking seam for a true animated
//   ink ripple; `Material._ink` fakes one by mounting a fresh, freshly-keyed
//   decorative circle (via `props.pulse`, a value the host changes on click)
//   whose entrance animation (scale + fade) plays once on mount — a
//   reasonable emulation of `InkWell` given the engine's mount-triggered,
//   spring/tween animation model rather than a stateful gesture arena.
// ===========================================================================

var Material = {};
Material.version = "1.0.0";

// ===========================================================================
// Internal helpers (self-contained; mirrors the Glass kit's own helpers so
// this file has no dependency on it).
// ===========================================================================

Material._opt = function (o) { return o ? o : {}; };
Material._d = function (o, k, d) { return has(o, k) ? get(o, k, d) : d; };
Material._map = function (list, fn) {
  var out = [];
  if (!list) { return out; }
  for (var i = 0; i < len(list); i = i + 1) { push(out, fn(list[i], i)); }
  return out;
};
Material._kids = function (o) {
  if (!has(o, "children")) { return []; }
  var c = o.children;
  if (typeOf(c) == "array") { return c; }
  return [c];
};
Material._len = function (v) {
  if (typeOf(v) == "object") { return v; }
  if (v == "full") { return { unit: "full" }; }
  if (v == "auto") { return { unit: "auto" }; }
  return { unit: "px", value: v };
};
Material._edges = function (v) {
  if (typeOf(v) == "object") { return v; }
  return { top: v, right: v, bottom: v, left: v };
};
Material._corner = function (v) {
  if (typeOf(v) == "object") { return v; }
  return { tl: v, tr: v, br: v, bl: v };
};
Material._cornerTop = function (v) { return { tl: v, tr: v, br: 0, bl: 0 }; };
Material._cornerBottom = function (v) { return { tl: 0, tr: 0, br: v, bl: v }; };

// ===========================================================================
// Color science: sRGB <-> HSL, and a tonal-palette generator approximating
// Material 3's HCT tone ramps (0 = black .. 100 = white at a fixed hue and
// chroma) closely enough to reproduce `ColorScheme.fromSeed`'s role mapping.
// ===========================================================================

Material._rgbToHsl = function (r, g, b) {
  var mx = max(r, max(g, b));
  var mn = min(r, min(g, b));
  var l = (mx + mn) / 2;
  var d = mx - mn;
  var h = 0;
  var s = 0;
  if (d > 0.00001) {
    s = d / (1 - abs(2 * l - 1));
    if (mx == r) { h = 60 * (((g - b) / d) % 6); }
    else if (mx == g) { h = 60 * ((b - r) / d + 2); }
    else { h = 60 * ((r - g) / d + 4); }
  }
  if (h < 0) { h = h + 360; }
  return [h, s, l];
};

Material._hslToColor = function (h, s, l) {
  var hh = h;
  while (hh < 0) { hh = hh + 360; }
  while (hh >= 360) { hh = hh - 360; }
  var c = (1 - abs(2 * l - 1)) * s;
  var x = c * (1 - abs(((hh / 60) % 2) - 1));
  var m = l - c / 2;
  var r = 0; var g = 0; var b = 0;
  if (hh < 60) { r = c; g = x; b = 0; }
  else if (hh < 120) { r = x; g = c; b = 0; }
  else if (hh < 180) { r = 0; g = c; b = x; }
  else if (hh < 240) { r = 0; g = x; b = c; }
  else if (hh < 300) { r = x; g = 0; b = c; }
  else { r = c; g = 0; b = x; }
  return rgb(r + m, g + m, b + m);
};

// A tonal palette: a fixed hue + chroma(saturation), sampled at any tone
// (perceptual lightness) 0 (black) .. 100 (white). Mirrors Material 3's
// `TonalPalette` (a simplified HSL stand-in for the real HCT color space).
class TonalPalette {
  constructor(hue, chroma) {
    this.hue = hue;
    this.chroma = chroma > 1 ? 1 : (chroma < 0 ? 0 : chroma);
  }
  tone(t) {
    var l = t / 100;
    if (l > 1) { l = 1; }
    if (l < 0) { l = 0; }
    return Material._hslToColor(this.hue, this.chroma, l);
  }
  static fromColor(seed) {
    var hsl = Material._rgbToHsl(seed.r, seed.g, seed.b);
    return new TonalPalette(hsl[0], hsl[1]);
  }
}
Material.TonalPalette = TonalPalette;

// Flutter's `Colors` class: the classic Material named swatches (500 shade).
Material.colors = {
  red: hex("#F44336"), pink: hex("#E91E63"), purple: hex("#9C27B0"),
  deepPurple: hex("#673AB7"), indigo: hex("#3F51B5"), blue: hex("#2196F3"),
  lightBlue: hex("#03A9F4"), cyan: hex("#00BCD4"), teal: hex("#009688"),
  green: hex("#4CAF50"), lightGreen: hex("#8BC34A"), lime: hex("#CDDC39"),
  yellow: hex("#FFEB3B"), amber: hex("#FFC107"), orange: hex("#FF9800"),
  deepOrange: hex("#FF5722"), brown: hex("#795548"), grey: hex("#9E9E9E"),
  blueGrey: hex("#607D8B"), black: hex("#000000"), white: hex("#FFFFFF"),
  transparent: rgba(0, 0, 0, 0)
};

// ===========================================================================
// ColorScheme — the 30-odd semantic color roles a Material 3 app is themed
// from, generated from a single seed color exactly like Flutter's
// `ColorScheme.fromSeed`: five tonal palettes (primary, secondary, tertiary,
// error, neutral, neutral-variant) sampled at fixed tones per role, per
// brightness.
// ===========================================================================

// NOTE: a class's own methods cannot call `ClassName.otherMethod()` on
// themselves in this engine (self-reference through the class's own name is
// unreliable — see the module-level comment in the OOP section below); `new
// ClassName(...)` and calling a *different* class's statics both work fine.
// So the named-constructor statics below route through this plain function
// rather than calling `ColorScheme.fromSeed` from within `ColorScheme` itself.
Material._colorSchemeFromSeed = function (opts) {
  opts = Material._opt(opts);
  var seed = has(opts, "seed") ? opts.seed : hex("#6750A4");
  var dark = Material._d(opts, "brightness", "light") == "dark";
  var hsl = Material._rgbToHsl(seed.r, seed.g, seed.b);
  var hue = hsl[0];
  var sat = hsl[1];
  var primary = new TonalPalette(hue, sat);
  var secondary = new TonalPalette(hue, sat * 0.32);
  var tertiary = new TonalPalette(hue + 60, sat * 0.48);
  var error = new TonalPalette(25, 0.84);
  var neutral = new TonalPalette(hue, sat * 0.08);
  var neutralVariant = new TonalPalette(hue, sat * 0.16);
  var roles = { brightness: dark ? "dark" : "light" };
  if (!dark) {
    roles.primary = primary.tone(40); roles.onPrimary = primary.tone(100);
    roles.primaryContainer = primary.tone(90); roles.onPrimaryContainer = primary.tone(10);
    roles.secondary = secondary.tone(40); roles.onSecondary = secondary.tone(100);
    roles.secondaryContainer = secondary.tone(90); roles.onSecondaryContainer = secondary.tone(10);
    roles.tertiary = tertiary.tone(40); roles.onTertiary = tertiary.tone(100);
    roles.tertiaryContainer = tertiary.tone(90); roles.onTertiaryContainer = tertiary.tone(10);
    roles.error = error.tone(40); roles.onError = error.tone(100);
    roles.errorContainer = error.tone(90); roles.onErrorContainer = error.tone(10);
    roles.background = neutral.tone(98); roles.onBackground = neutral.tone(10);
    roles.surface = neutral.tone(98); roles.onSurface = neutral.tone(10);
    roles.surfaceVariant = neutralVariant.tone(90); roles.onSurfaceVariant = neutralVariant.tone(30);
    roles.outline = neutralVariant.tone(50); roles.outlineVariant = neutralVariant.tone(80);
    roles.shadow = neutral.tone(0); roles.scrim = neutral.tone(0);
    roles.inverseSurface = neutral.tone(20); roles.onInverseSurface = neutral.tone(95);
    roles.inversePrimary = primary.tone(80); roles.surfaceTint = roles.primary;
    roles.surfaceContainerLowest = neutral.tone(100); roles.surfaceContainerLow = neutral.tone(96);
    roles.surfaceContainer = neutral.tone(94); roles.surfaceContainerHigh = neutral.tone(92);
    roles.surfaceContainerHighest = neutral.tone(90);
  } else {
    roles.primary = primary.tone(80); roles.onPrimary = primary.tone(20);
    roles.primaryContainer = primary.tone(30); roles.onPrimaryContainer = primary.tone(90);
    roles.secondary = secondary.tone(80); roles.onSecondary = secondary.tone(20);
    roles.secondaryContainer = secondary.tone(30); roles.onSecondaryContainer = secondary.tone(90);
    roles.tertiary = tertiary.tone(80); roles.onTertiary = tertiary.tone(20);
    roles.tertiaryContainer = tertiary.tone(30); roles.onTertiaryContainer = tertiary.tone(90);
    roles.error = error.tone(80); roles.onError = error.tone(20);
    roles.errorContainer = error.tone(30); roles.onErrorContainer = error.tone(90);
    roles.background = neutral.tone(6); roles.onBackground = neutral.tone(90);
    roles.surface = neutral.tone(6); roles.onSurface = neutral.tone(90);
    roles.surfaceVariant = neutralVariant.tone(30); roles.onSurfaceVariant = neutralVariant.tone(80);
    roles.outline = neutralVariant.tone(60); roles.outlineVariant = neutralVariant.tone(30);
    roles.shadow = neutral.tone(0); roles.scrim = neutral.tone(0);
    roles.inverseSurface = neutral.tone(90); roles.onInverseSurface = neutral.tone(20);
    roles.inversePrimary = primary.tone(40); roles.surfaceTint = roles.primary;
    roles.surfaceContainerLowest = neutral.tone(4); roles.surfaceContainerLow = neutral.tone(10);
    roles.surfaceContainer = neutral.tone(12); roles.surfaceContainerHigh = neutral.tone(17);
    roles.surfaceContainerHighest = neutral.tone(22);
  }
  return new ColorScheme(roles);
};

class ColorScheme {
  constructor(roles) {
    this.brightness = roles.brightness;
    this.primary = roles.primary; this.onPrimary = roles.onPrimary;
    this.primaryContainer = roles.primaryContainer; this.onPrimaryContainer = roles.onPrimaryContainer;
    this.secondary = roles.secondary; this.onSecondary = roles.onSecondary;
    this.secondaryContainer = roles.secondaryContainer; this.onSecondaryContainer = roles.onSecondaryContainer;
    this.tertiary = roles.tertiary; this.onTertiary = roles.onTertiary;
    this.tertiaryContainer = roles.tertiaryContainer; this.onTertiaryContainer = roles.onTertiaryContainer;
    this.error = roles.error; this.onError = roles.onError;
    this.errorContainer = roles.errorContainer; this.onErrorContainer = roles.onErrorContainer;
    this.background = roles.background; this.onBackground = roles.onBackground;
    this.surface = roles.surface; this.onSurface = roles.onSurface;
    this.surfaceVariant = roles.surfaceVariant; this.onSurfaceVariant = roles.onSurfaceVariant;
    this.outline = roles.outline; this.outlineVariant = roles.outlineVariant;
    this.shadow = roles.shadow; this.scrim = roles.scrim;
    this.inverseSurface = roles.inverseSurface; this.onInverseSurface = roles.onInverseSurface;
    this.inversePrimary = roles.inversePrimary; this.surfaceTint = roles.surfaceTint;
    this.surfaceContainerLowest = roles.surfaceContainerLowest;
    this.surfaceContainerLow = roles.surfaceContainerLow;
    this.surfaceContainer = roles.surfaceContainer;
    this.surfaceContainerHigh = roles.surfaceContainerHigh;
    this.surfaceContainerHighest = roles.surfaceContainerHighest;
  }

  static fromSeed(opts) { return Material._colorSchemeFromSeed(opts); }
  static light(seed) { return seed ? Material._colorSchemeFromSeed({ seed: seed, brightness: "light" }) : Material._colorSchemeFromSeed({ brightness: "light" }); }
  static dark(seed) { return seed ? Material._colorSchemeFromSeed({ seed: seed, brightness: "dark" }) : Material._colorSchemeFromSeed({ brightness: "dark" }); }
}
Material.ColorScheme = ColorScheme;

// ===========================================================================
// Typography — the Material 3 type scale (15 named text styles). `height` is
// a line-height *multiplier* (line-height / font-size), matching the ratios
// in the M3 spec's px tables (e.g. Display Large 57/64 -> 1.1228).
// ===========================================================================

Material._typeScale = {
  displayLarge: { size: 57, height: 1.1228, weight: "normal" },
  displayMedium: { size: 45, height: 1.1556, weight: "normal" },
  displaySmall: { size: 36, height: 1.2222, weight: "normal" },
  headlineLarge: { size: 32, height: 1.25, weight: "normal" },
  headlineMedium: { size: 28, height: 1.2857, weight: "normal" },
  headlineSmall: { size: 24, height: 1.3333, weight: "normal" },
  titleLarge: { size: 22, height: 1.2727, weight: "normal" },
  titleMedium: { size: 16, height: 1.5, weight: "medium" },
  titleSmall: { size: 14, height: 1.4286, weight: "medium" },
  bodyLarge: { size: 16, height: 1.5, weight: "normal" },
  bodyMedium: { size: 14, height: 1.4286, weight: "normal" },
  bodySmall: { size: 12, height: 1.3333, weight: "normal" },
  labelLarge: { size: 14, height: 1.4286, weight: "medium" },
  labelMedium: { size: 12, height: 1.3333, weight: "medium" },
  labelSmall: { size: 11, height: 1.4545, weight: "medium" }
};

class TextTheme {
  constructor(scale) { this.scale = scale; }
  style(variant) { return get(this.scale, variant, this.scale.bodyMedium); }
  static m3() { return new TextTheme(Material._typeScale); }
}
Material.TextTheme = TextTheme;

// ===========================================================================
// ThemeData — the ambient theme, mirroring Flutter's `ThemeData`/`Theme.of`.
// ===========================================================================

class ThemeData {
  constructor(opts) {
    opts = Material._opt(opts);
    this.colorScheme = has(opts, "colorScheme") ? opts.colorScheme : ColorScheme.light();
    this.textTheme = has(opts, "textTheme") ? opts.textTheme : TextTheme.m3();
    this.useMaterial3 = Material._d(opts, "useMaterial3", true);
  }
  static light(seed) { return new ThemeData({ colorScheme: seed ? ColorScheme.light(seed) : ColorScheme.light() }); }
  static dark(seed) { return new ThemeData({ colorScheme: seed ? ColorScheme.dark(seed) : ColorScheme.dark() }); }
}
Material.ThemeData = ThemeData;

Material._theme = ThemeData.light();
// Get (no args) or replace (with a ThemeData) the ambient theme — the
// `Theme.of(context)` / `MaterialApp(theme:)` equivalent.
Material.theme = function (next) {
  if (next) { Material._theme = next; }
  return Material._theme;
};
Material.setTheme = function (themeData) { Material._theme = themeData; return Material._theme; };

// ===========================================================================
// Elevation, shape and motion tokens.
// ===========================================================================

// Material 3 elevation levels 0..5 map to fixed dp values; the dual
// ambient+key shadow pair below approximates Flutter's elevation shadow
// recipe (a soft wide ambient shadow plus a tighter directional key shadow),
// scaled by dp rather than reproduced from Flutter's exact per-dp shadow
// table (the engine's single blur+spread+offset `Shadow` primitive can't
// reproduce Flutter's 3-shadow stack exactly, but the silhouette reads the
// same: subtler at low elevation, a soft deep drop at high elevation).
Material._elevationDp = [0, 1, 3, 6, 8, 12];
Material.elevationDp = function (dp) {
  if (!dp || dp <= 0) { return []; }
  var ambient = shadow([0, dp * 0.3], dp * 2.6 + 1, rgba(0, 0, 0, 0.15), dp * 0.05);
  var key = shadow([0, dp * 0.9], dp * 1.6, rgba(0, 0, 0, 0.30), 0);
  return [ambient, key];
};
Material.elevation = function (level) {
  var dp = get(Material._elevationDp, "" + level, level);
  if (typeOf(dp) != "number") { dp = Material._elevationDp[level]; }
  return Material.elevationDp(dp);
};

// Material 3 shape scale.
Material.shape = { none: 0, extraSmall: 4, small: 8, medium: 12, large: 16, extraLarge: 28, full: 9999 };

// Material 3 motion duration + easing tokens.
Material.motion = {
  duration: {
    short1: 50, short2: 100, short3: 150, short4: 200,
    medium1: 250, medium2: 300, medium3: 350, medium4: 400,
    long1: 450, long2: 500, long3: 550, long4: 600
  },
  bezier: {
    standard: [0.2, 0.0, 0.0, 1.0],
    standardAccelerate: [0.3, 0.0, 1.0, 1.0],
    standardDecelerate: [0.0, 0.0, 0.0, 1.0],
    emphasized: [0.2, 0.0, 0.0, 1.0]
  }
};

// Animation builders atop the raw protocol (`AnimProp`/`Curve`/`Animation`).
Material.tween = function (duration, easing, bezierPts) {
  var c = { kind: "tween", duration: duration, easing: easing ? easing : "ease_in_out" };
  if (bezierPts) { c.bezier = bezierPts; c.easing = "cubic_bezier"; }
  return c;
};
Material.spring = function (stiffness, damping, mass) {
  return { kind: "spring", stiffness: stiffness ? stiffness : 170, damping: damping ? damping : 26,
           mass: mass ? mass : 1, initial_velocity: 0 };
};
Material.anim = function (prop, opts) {
  opts = Material._opt(opts);
  var a = { prop: prop, curve: has(opts, "curve") ? opts.curve : Material.tween(Material.motion.duration.medium2, "ease_in_out"),
            repeat: Material._d(opts, "repeat", "once"), delay: Material._d(opts, "delay", 0) };
  if (has(opts, "to")) { a.to = opts.to; }
  if (has(opts, "from")) { a.from = opts.from; }
  if (has(opts, "keyframes")) { a.keyframes = opts.keyframes; }
  if (has(opts, "onComplete")) { a.on_complete = opts.onComplete; }
  return a;
};

// A fake ink ripple: pass a changing `pulse` value (e.g. a click counter the
// host bumps before re-rendering) to mount a fresh decorative circle that
// plays a one-shot scale+fade entrance animation, approximating `InkWell`.
// Only wraps `child` in a stack (and pays for the extra node) when the
// caller actually supplies a `pulse`; otherwise returns `child` untouched.
// The wrapping stack has no explicit size of its own, so it sizes to `child`
// exactly as `child` would size on its own — the ripple overlay (absolute,
// inset 0, `full`) then fills whatever that resolves to.
Material._ink = function (child, opts) {
  opts = Material._opt(opts);
  if (!has(opts, "pulse")) { return child; }
  var color = Material._d(opts, "color", rgba(0, 0, 0, 0.10));
  var radius = has(opts, "radius") ? opts.radius : Material._corner(Material.shape.full);
  var ripple = div({
    key: "ink-" + opts.pulse,
    style: { position: "absolute", inset: { top: 0, right: 0, bottom: 0, left: 0 },
             width: { unit: "full" }, height: { unit: "full" }, radius: radius,
             background: solid(color), opacity: 0 },
    animations: [
      Material.anim("opacity", { from: 0.55, to: 0, curve: Material.tween(Material.motion.duration.long1, "ease_out") }),
      Material.anim("scale", { from: 0.3, to: 1.6, curve: Material.tween(Material.motion.duration.long1, "ease_out") })
    ]
  });
  return stack({ children: [child, ripple] });
};

// ===========================================================================
// Icons — Material icon names, best-effort mapped onto the engine's Tabler
// icon set (`crates/elpis-protocol/src/node.rs`'s `IconSpec` ships Tabler +
// Noto). Unknown names pass through unchanged (in case the host registers a
// "material" icon set, or the caller already knows the Tabler name).
// ===========================================================================

Material._iconMap = {
  add: "plus", remove: "minus", close: "x", check: "check", menu: "menu-2",
  more_vert: "dots-vertical", more_horiz: "dots", search: "search", settings: "settings",
  home: "home", favorite: "heart", favorite_border: "heart", star: "star", star_border: "star",
  delete: "trash", edit: "pencil", share: "share", download: "download", upload: "upload",
  refresh: "refresh", visibility: "eye", visibility_off: "eye-off", lock: "lock",
  lock_open: "lock-open", person: "user", people: "users", notifications: "bell",
  notifications_none: "bell", mail: "mail", phone: "phone", calendar_today: "calendar",
  location_on: "map-pin", info: "info-circle", warning: "alert-triangle", error: "alert-circle",
  help: "help", chevron_left: "chevron-left", chevron_right: "chevron-right",
  expand_more: "chevron-down", expand_less: "chevron-up", arrow_drop_down: "chevron-down",
  arrow_drop_up: "chevron-up", arrow_back: "arrow-left", arrow_forward: "arrow-right",
  play_arrow: "player-play", pause: "player-pause", stop: "player-stop",
  skip_next: "player-skip-forward", skip_previous: "player-skip-back",
  volume_up: "volume", volume_off: "volume-3", camera_alt: "camera", photo: "photo",
  image: "photo", attach_file: "paperclip", send: "send", add_circle: "circle-plus",
  remove_circle: "circle-minus", check_circle: "circle-check", cancel: "circle-x",
  filter_list: "filter", sort: "arrows-sort", dashboard: "layout-dashboard", apps: "apps",
  list: "list", grid_view: "layout-grid", folder: "folder", file_copy: "copy",
  save: "device-floppy", print: "printer", logout: "logout", login: "login"
};
Material.icon = function (o) {
  o = Material._opt(o);
  var name = Material._d(o, "name", "circle");
  var mapped = get(Material._iconMap, name, name);
  return icon(mapped, { size: Material._d(o, "size", 24), color: Material._d(o, "color", Material.theme().colorScheme.onSurface) });
};

// ===========================================================================
// Core OOP framework: `Widget` -> `StatelessWidget` / `StatefulWidget`, and a
// minimal `State`/`runApp` loop. Every concrete widget below extends one of
// these, exactly mirroring Flutter's own widget class hierarchy.
//
// Because Elpis's render model has no persistent Element tree (a Miniapp's
// own top-level variables are the only thing that survives across renders —
// see `crates/elpis-host/src/prelude.js`), a `StatefulWidget`'s `State` must
// be created and held by the host Miniapp itself (in a module-level var) and
// threaded back in via `props.state`, exactly as `counter/app.js` holds
// `count` at the top level. `Material.runApp` + `State.setState` wire that
// pattern up so it feels like Flutter's `setState`: mutate, then the kit
// re-renders the whole app for you.
// ===========================================================================

class Widget {
  constructor(props) { this.props = Material._opt(props); }
  theme() { return has(this.props, "theme") ? this.props.theme : Material.theme(); }
  opt(k, d) { return Material._d(this.props, k, d); }
  kids() { return Material._kids(this.props); }
  // Apply the universal props (key/style/events) every widget accepts.
  finish(n) {
    if (has(this.props, "key")) { n.key = this.props.key; }
    if (has(this.props, "style")) { withStyle(n, this.props.style); }
    if (has(this.props, "events")) { bindEvents(n, this.props.events); }
    return n;
  }
  build() { return this.finish(div({})); }
}
Material.Widget = Widget;

class StatelessWidget extends Widget { }
Material.StatelessWidget = StatelessWidget;

// A Flutter-style `State<T>`: override `init()` and `build(widget)`.
class State {
  constructor() { this.state = {}; this._inited = false; }
  init() { }
  // `patch` is either an object shallow-merged into `state`, or a function
  // `(state) -> object` for functional updates — then triggers a full re-render
  // via `Material.runApp`'s stored root builder (Flutter's `setState`).
  setState(patch) {
    var next = typeOf(patch) == "object" ? patch : patch(this.state);
    this.state = merge(this.state, next);
    Material._rebuild();
    return this.state;
  }
  build(widget) { return div({}); }
}
Material.State = State;

// `props.state` must be a `State` instance the host keeps alive across
// renders (a module-level `var`); `createState()` is only consulted the very
// first time (when the host hasn't created one yet).
class StatefulWidget extends Widget {
  createState() { return new State(); }
  build() {
    var s = has(this.props, "state") ? this.props.state : this.createState();
    if (!s._inited) { s._inited = true; s.init(); }
    return this.finish(s.build(this));
  }
}
Material.StatefulWidget = StatefulWidget;

Material._root = null;
// The `runApp(widget)` equivalent: remembers a zero-arg builder function and
// renders it. Call `Material._rebuild()` (or let `State.setState` do it) to
// re-render after mutating state held outside the tree.
Material.runApp = function (rootBuilderFn) {
  Material._root = rootBuilderFn;
  Material._rebuild();
};
Material._rebuild = function () {
  if (Material._root) { render(Material._root()); }
};

// ===========================================================================
// Layout — the structural widgets every Flutter screen is built from.
// ===========================================================================

Material._mainAxis = function (v) {
  if (v == "spaceBetween") { return "between"; }
  if (v == "spaceAround") { return "around"; }
  if (v == "spaceEvenly") { return "evenly"; }
  return v ? v : "start";
};
// "topLeft".."bottomRight" -> [align_items, justify_content].
Material._boxAlign = function (v) {
  if (v == "topLeft") { return ["start", "start"]; }
  if (v == "topCenter") { return ["center", "start"]; }
  if (v == "topRight") { return ["end", "start"]; }
  if (v == "centerLeft") { return ["start", "center"]; }
  if (v == "centerRight") { return ["end", "center"]; }
  if (v == "bottomLeft") { return ["start", "end"]; }
  if (v == "bottomCenter") { return ["center", "end"]; }
  if (v == "bottomRight") { return ["end", "end"]; }
  return ["center", "center"];
};

// `Container` — Flutter's do-everything box: color/gradient, padding, margin,
// fixed size, border, radius, elevation (as a shadow), alignment of children.
class Container extends StatelessWidget {
  build() {
    var st = {};
    if (has(this.props, "width")) { st.width = Material._len(this.props.width); }
    if (has(this.props, "height")) { st.height = Material._len(this.props.height); }
    if (has(this.props, "padding")) { st.padding = Material._edges(this.props.padding); }
    if (has(this.props, "margin")) { st.margin = Material._edges(this.props.margin); }
    if (has(this.props, "color")) { st.background = solid(this.props.color); }
    if (has(this.props, "gradient")) { st.background = this.props.gradient; }
    if (has(this.props, "borderRadius")) { st.radius = Material._corner(this.props.borderRadius); }
    if (has(this.props, "borderWidth")) { st.border_width = this.props.borderWidth; }
    if (has(this.props, "borderColor")) { st.border_color = this.props.borderColor; }
    if (has(this.props, "elevation")) { st.shadows = Material.elevationDp(this.props.elevation); }
    if (has(this.props, "alignment")) {
      var al = Material._boxAlign(this.props.alignment);
      st.align_items = al[0]; st.justify_content = al[1];
    }
    return this.finish(div({ style: st, children: this.kids() }));
  }
}
Material.Container = Container;
Material.container = function (o) { return new Container(o).build(); };

class SizedBox extends StatelessWidget {
  build() {
    var st = {};
    if (has(this.props, "width")) { st.width = Material._len(this.props.width); }
    if (has(this.props, "height")) { st.height = Material._len(this.props.height); }
    return this.finish(div({ style: st, children: this.kids() }));
  }
  static shrink() { return new SizedBox({ width: 0, height: 0 }).build(); }
}
Material.SizedBox = SizedBox;
Material.sizedBox = function (o) { return new SizedBox(o).build(); };

class Center extends StatelessWidget {
  build() {
    var st = { align_items: "center", justify_content: "center",
               width: Material._len(this.opt("width", "full")), height: Material._len(this.opt("height", "full")) };
    return this.finish(column({ style: st, children: this.kids() }));
  }
}
Material.Center = Center;
Material.center = function (o) { return new Center(o).build(); };

class Align extends StatelessWidget {
  build() {
    var al = Material._boxAlign(this.opt("alignment", "center"));
    var st = { align_items: al[0], justify_content: al[1],
               width: Material._len(this.opt("width", "full")), height: Material._len(this.opt("height", "full")) };
    return this.finish(column({ style: st, children: this.kids() }));
  }
}
Material.Align = Align;
Material.align = function (o) { return new Align(o).build(); };

class Padding extends StatelessWidget {
  build() {
    var st = { padding: Material._edges(this.opt("padding", 0)) };
    return this.finish(div({ style: st, children: this.kids() }));
  }
}
Material.Padding = Padding;
Material.padding = function (o) { return new Padding(o).build(); };

// `Expanded`/`Flexible` — flex children that grow to fill the remaining
// main-axis space of their parent `Row`/`Column`.
class Expanded extends StatelessWidget {
  build() {
    var st = { flex_grow: this.opt("flex", 1), width: Material._len("full"), height: Material._len("full") };
    return this.finish(div({ style: st, children: this.kids() }));
  }
}
Material.Expanded = Expanded;
Material.expanded = function (o) { return new Expanded(o).build(); };

class Flexible extends StatelessWidget {
  build() {
    var st = { flex_grow: this.opt("flex", 1), flex_shrink: 1 };
    return this.finish(div({ style: st, children: this.kids() }));
  }
}
Material.Flexible = Flexible;
Material.flexible = function (o) { return new Flexible(o).build(); };

class Spacer extends StatelessWidget {
  build() { return this.finish(spacer({ style: { flex_grow: this.opt("flex", 1) } })); }
}
Material.Spacer = Spacer;
Material.spacerWidget = function (o) { return new Spacer(o).build(); };

class Wrap extends StatelessWidget {
  build() {
    var st = { wrap: true, gap: this.opt("spacing", 8) };
    if (has(this.props, "runSpacing")) { st.row_gap = this.props.runSpacing; }
    if (has(this.props, "alignment")) { st.justify_content = Material._mainAxis(this.props.alignment); }
    var dir = this.opt("direction", "horizontal");
    var n = dir == "vertical" ? column({ style: st, children: this.kids() }) : row({ style: st, children: this.kids() });
    return this.finish(n);
  }
}
Material.Wrap = Wrap;
Material.wrap = function (o) { return new Wrap(o).build(); };

class Row extends StatelessWidget {
  build() {
    var st = {};
    if (has(this.props, "mainAxisAlignment")) { st.justify_content = Material._mainAxis(this.props.mainAxisAlignment); }
    if (has(this.props, "crossAxisAlignment")) { st.align_items = this.props.crossAxisAlignment; }
    if (has(this.props, "spacing")) { st.gap = this.props.spacing; }
    return this.finish(row({ style: st, children: this.kids() }));
  }
}
Material.Row = Row;
Material.row = function (o) { return new Row(o).build(); };

class Column extends StatelessWidget {
  build() {
    var st = {};
    if (has(this.props, "mainAxisAlignment")) { st.justify_content = Material._mainAxis(this.props.mainAxisAlignment); }
    if (has(this.props, "crossAxisAlignment")) { st.align_items = this.props.crossAxisAlignment; }
    if (has(this.props, "spacing")) { st.gap = this.props.spacing; }
    return this.finish(column({ style: st, children: this.kids() }));
  }
}
Material.Column = Column;
Material.column = function (o) { return new Column(o).build(); };

class Stack extends StatelessWidget {
  build() {
    var st = {};
    if (has(this.props, "alignment")) {
      var al = Material._boxAlign(this.props.alignment);
      st.align_items = al[0]; st.justify_content = al[1];
    }
    return this.finish(stack({ style: st, children: this.kids() }));
  }
}
Material.Stack = Stack;
Material.stackWidget = function (o) { return new Stack(o).build(); };

// A `Stack` child pinned by absolute offsets — matches Flutter's `Positioned`
// (unset sides default to `0` rather than "unconstrained", since the
// engine's inset edges are plain floats with no "auto" side).
class Positioned extends StatelessWidget {
  build() {
    var st = { position: "absolute",
               inset: { top: this.opt("top", 0), right: this.opt("right", 0),
                        bottom: this.opt("bottom", 0), left: this.opt("left", 0) } };
    if (has(this.props, "width")) { st.width = Material._len(this.props.width); }
    if (has(this.props, "height")) { st.height = Material._len(this.props.height); }
    return this.finish(div({ style: st, children: this.kids() }));
  }
}
Material.Positioned = Positioned;
Material.positioned = function (o) { return new Positioned(o).build(); };

class ListView extends StatelessWidget {
  build() {
    var st = { gap: this.opt("spacing", 0) };
    if (has(this.props, "padding")) { st.padding = Material._edges(this.props.padding); }
    return this.finish(scroll({ axis: "vertical", style: st, children: this.kids() }));
  }
  static builder(o) {
    o = Material._opt(o);
    var count = Material._d(o, "itemCount", 0);
    var kids = [];
    for (var i = 0; i < count; i = i + 1) { push(kids, o.itemBuilder(i)); }
    return new ListView(merge(o, { children: kids })).build();
  }
  static separated(o) {
    o = Material._opt(o);
    var count = Material._d(o, "itemCount", 0);
    var kids = [];
    for (var i = 0; i < count; i = i + 1) {
      push(kids, o.itemBuilder(i));
      if (i < count - 1) { push(kids, o.separatorBuilder(i)); }
    }
    return new ListView(merge(o, { children: kids })).build();
  }
}
Material.ListView = ListView;
Material.listView = function (o) { return new ListView(o).build(); };

class GridView extends StatelessWidget {
  build() {
    var cols = this.opt("crossAxisCount", 2);
    var tracks = [];
    for (var i = 0; i < cols; i = i + 1) { push(tracks, "1fr"); }
    var st = { grid_template_columns: join(tracks, " "), gap: this.opt("spacing", 8) };
    if (has(this.props, "padding")) { st.padding = Material._edges(this.props.padding); }
    return this.finish(grid({ style: st, children: this.kids() }));
  }
  static count(o) { return new GridView(o).build(); }
  static builder(o) {
    o = Material._opt(o);
    var count = Material._d(o, "itemCount", 0);
    var kids = [];
    for (var i = 0; i < count; i = i + 1) { push(kids, o.itemBuilder(i)); }
    return new GridView(merge(o, { children: kids })).build();
  }
}
Material.GridView = GridView;
Material.gridView = function (o) { return new GridView(o).build(); };

class SingleChildScrollView extends StatelessWidget {
  build() {
    var axis = this.opt("scrollDirection", "vertical");
    return this.finish(scroll({ axis: axis, children: this.kids() }));
  }
}
Material.SingleChildScrollView = SingleChildScrollView;
Material.singleChildScrollView = function (o) { return new SingleChildScrollView(o).build(); };

class Divider extends StatelessWidget {
  build() {
    var th = this.theme();
    var st = { height: Material._len(this.opt("thickness", 1)), width: Material._len("full"),
               background: solid(this.opt("color", th.colorScheme.outlineVariant)) };
    if (has(this.props, "indent") || has(this.props, "endIndent")) {
      st.margin = { top: 0, bottom: 0, left: this.opt("indent", 0), right: this.opt("endIndent", 0) };
    }
    return this.finish(div({ style: st }));
  }
}
Material.Divider = Divider;
Material.divider = function (o) { return new Divider(o).build(); };

class VerticalDivider extends StatelessWidget {
  build() {
    var th = this.theme();
    var st = { width: Material._len(this.opt("thickness", 1)), height: Material._len("full"),
               background: solid(this.opt("color", th.colorScheme.outlineVariant)) };
    if (has(this.props, "indent") || has(this.props, "endIndent")) {
      st.margin = { left: 0, right: 0, top: this.opt("indent", 0), bottom: this.opt("endIndent", 0) };
    }
    return this.finish(div({ style: st }));
  }
}
Material.VerticalDivider = VerticalDivider;
Material.verticalDivider = function (o) { return new VerticalDivider(o).build(); };

// ===========================================================================
// Text & media.
// ===========================================================================

// `Text` — Material-typography-aware text. `variant` picks a type-scale role
// (`"headlineSmall"`, `"bodyMedium"`, …); any field can be overridden.
class MaterialText extends StatelessWidget {
  build() {
    var th = this.theme();
    var st = th.textTheme.style(this.opt("variant", "bodyMedium"));
    var props = {
      size: this.opt("size", st.size),
      weight: this.opt("weight", st.weight),
      foreground: this.opt("color", th.colorScheme.onSurface),
      align: this.opt("align", "start"),
      line_height: this.opt("lineHeight", st.height)
    };
    if (has(this.props, "italic")) { props.italic = this.props.italic; }
    if (has(this.props, "underline")) { props.underline = this.props.underline; }
    if (has(this.props, "maxLines")) { props.max_lines = this.props.maxLines; }
    if (has(this.props, "letterSpacing")) { props.letter_spacing = this.props.letterSpacing; }
    return this.finish(text(this.opt("text", ""), props));
  }
}
Material.Text = MaterialText;
Material.text = function (o) { return new MaterialText(o).build(); };

// `CircleAvatar` — a circular avatar with an image, initials, or a solo icon.
class CircleAvatar extends StatelessWidget {
  build() {
    var th = this.theme();
    var sz = this.opt("radius", 20) * 2;
    var st = { width: Material._len(sz), height: Material._len(sz), radius: Material._corner(sz),
               align_items: "center", justify_content: "center", overflow_x: "hidden", overflow_y: "hidden",
               background: solid(this.opt("backgroundColor", th.colorScheme.primaryContainer)) };
    var kids;
    if (has(this.props, "backgroundImage")) { kids = [image(this.props.backgroundImage, { fit: "cover" })]; }
    else if (has(this.props, "child")) { kids = [this.props.child]; }
    else {
      kids = [new MaterialText({ text: this.opt("initials", "?"), size: sz * 0.4, weight: "medium",
                                  color: this.opt("foregroundColor", th.colorScheme.onPrimaryContainer) }).build()];
    }
    return this.finish(stack({ style: st, children: kids }));
  }
}
Material.CircleAvatar = CircleAvatar;
Material.circleAvatar = function (o) { return new CircleAvatar(o).build(); };

// ===========================================================================
// Surfaces: Card, Scaffold, AppBar, BottomAppBar.
// ===========================================================================

// `Card` — elevated (default), filled, or outlined; M3 shape/color defaults.
class Card extends StatelessWidget {
  build() {
    var th = this.theme();
    var variant = this.opt("variant", "elevated");
    var st = { radius: Material._corner(this.opt("shape", Material.shape.medium)),
               margin: Material._edges(this.opt("margin", 4)) };
    if (variant == "outlined") {
      st.background = solid(this.opt("color", th.colorScheme.surface));
      st.border_width = 1; st.border_color = th.colorScheme.outlineVariant;
    } else if (variant == "filled") {
      st.background = solid(this.opt("color", th.colorScheme.surfaceContainerHighest));
    } else {
      st.background = solid(this.opt("color", th.colorScheme.surfaceContainerLow));
      st.shadows = Material.elevationDp(this.opt("elevation", 1));
    }
    if (has(this.props, "padding")) { st.padding = Material._edges(this.props.padding); }
    if (has(this.props, "width")) { st.width = Material._len(this.props.width); }
    if (has(this.props, "height")) { st.height = Material._len(this.props.height); }
    return this.finish(column({ style: st, children: this.kids() }));
  }
  static outlined(o) { return new Card(merge(Material._opt(o), { variant: "outlined" })).build(); }
  static filled(o) { return new Card(merge(Material._opt(o), { variant: "filled" })).build(); }
}
Material.Card = Card;
Material.card = function (o) { return new Card(o).build(); };

// `Scaffold` — the top-level page frame: app bar, body, optional FAB,
// bottom navigation bar and drawer (drawer renders only while `drawerOpen`).
class Scaffold extends StatelessWidget {
  build() {
    var th = this.theme();
    var kids = [];
    if (has(this.props, "appBar")) { push(kids, this.props.appBar); }
    var bodyKids = has(this.props, "body") ? [this.props.body] : this.kids();
    push(kids, column({ style: { flex_grow: 1, width: Material._len("full"), overflow_y: "auto" }, children: bodyKids }));
    if (has(this.props, "bottomNavigationBar")) { push(kids, this.props.bottomNavigationBar); }
    var st = { width: Material._len("full"), height: Material._len("full"),
               background: solid(this.opt("backgroundColor", th.colorScheme.background)) };
    var layers = [column({ style: { width: Material._len("full"), height: Material._len("full") }, children: kids })];
    if (has(this.props, "floatingActionButton")) {
      var pos = this.opt("floatingActionButtonLocation", "endFloat");
      var alignItems = pos == "centerFloat" ? "center" : "end";
      push(layers, column({ style: { position: "absolute", inset: { top: 0, right: 16, bottom: 16, left: 0 },
                                     width: Material._len("full"), align_items: alignItems, justify_content: "end" },
                            children: [this.props.floatingActionButton] }));
    }
    if (this.opt("drawerOpen", false) && has(this.props, "drawer")) { push(layers, this.props.drawer); }
    if (this.opt("endDrawerOpen", false) && has(this.props, "endDrawer")) { push(layers, this.props.endDrawer); }
    if (has(this.props, "snackBar")) { push(layers, this.props.snackBar); }
    return this.finish(stack({ style: st, children: layers }));
  }
}
Material.Scaffold = Scaffold;
Material.scaffold = function (o) { return new Scaffold(o).build(); };

// `AppBar` — a top app bar with leading/title/actions, M3 flat by default
// (`elevation: 0`), an optional `centerTitle`.
class AppBar extends StatelessWidget {
  build() {
    var th = this.theme();
    var leading = has(this.props, "leading") ? [this.props.leading] : [];
    var titleNode = null;
    if (has(this.props, "title")) {
      var t = this.props.title;
      titleNode = (typeOf(t) == "object" && has(t, "type")) ? t
        : new MaterialText({ text: t, variant: "titleLarge", color: this.opt("foregroundColor", th.colorScheme.onSurface) }).build();
    }
    var st = { width: Material._len("full"), height: Material._len(64), align_items: "center",
               justify_content: "between",
               padding: { top: 8, right: 16, bottom: 8, left: has(this.props, "leading") ? 4 : 16 },
               background: solid(this.opt("backgroundColor", th.colorScheme.surface)) };
    var elevation = this.opt("elevation", 0);
    if (elevation > 0) { st.shadows = Material.elevationDp(elevation); }
    var actions = has(this.props, "actions") ? this.props.actions : [];
    var right = row({ style: { gap: 4, align_items: "center" }, children: actions });
    var kids;
    if (this.opt("centerTitle", false)) {
      kids = [
        row({ style: { gap: 16, align_items: "center", width: Material._len(96) }, children: leading }),
        row({ style: { flex_grow: 1, align_items: "center", justify_content: "center" }, children: titleNode ? [titleNode] : [] }),
        right
      ];
    } else {
      var left = row({ style: { gap: 16, align_items: "center", flex_grow: 1 }, children: concat(leading, titleNode ? [titleNode] : []) });
      kids = [left, right];
    }
    return this.finish(row({ style: st, children: kids }));
  }
}
Material.AppBar = AppBar;
Material.appBar = function (o) { return new AppBar(o).build(); };

class BottomAppBar extends StatelessWidget {
  build() {
    var th = this.theme();
    var st = { width: Material._len("full"), height: Material._len(80), align_items: "center",
               padding: { top: 8, right: 16, bottom: 8, left: 16 }, gap: 8,
               background: solid(this.opt("color", th.colorScheme.surfaceContainer)) };
    var elevation = this.opt("elevation", 3);
    if (elevation > 0) { st.shadows = Material.elevationDp(elevation); }
    return this.finish(row({ style: st, children: this.kids() }));
  }
}
Material.BottomAppBar = BottomAppBar;
Material.bottomAppBar = function (o) { return new BottomAppBar(o).build(); };

// ===========================================================================
// Buttons — `ElevatedButton`/`FilledButton`/`OutlinedButton`/`TextButton` all
// share one visual recipe (pill shape, `labelLarge` type, an optional leading
// icon) and differ only in their fill/border/elevation, exactly like
// Flutter's own four button classes. `_ButtonBase` holds the shared recipe;
// each concrete class overrides `kind()` — ordinary polymorphism (`this.
// kind()`), not the broken self-reference pattern above.
// ===========================================================================

class _ButtonBase extends StatelessWidget {
  kind() { return "text"; }
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var kind = this.kind();
    var disabled = this.opt("disabled", false);
    var bg = null; var fg = cs.primary; var border = null; var elevation = 0;
    if (kind == "elevated") {
      bg = this.opt("backgroundColor", cs.surfaceContainerLow); fg = this.opt("foregroundColor", cs.primary);
      elevation = this.opt("elevation", 1);
    } else if (kind == "filled") {
      bg = this.opt("backgroundColor", cs.primary); fg = this.opt("foregroundColor", cs.onPrimary);
    } else if (kind == "tonal") {
      bg = this.opt("backgroundColor", cs.secondaryContainer); fg = this.opt("foregroundColor", cs.onSecondaryContainer);
    } else if (kind == "outlined") {
      fg = this.opt("foregroundColor", cs.primary); border = this.opt("borderColor", cs.outline);
    } else {
      fg = this.opt("foregroundColor", cs.primary);
    }
    var hasIcon = has(this.props, "icon");
    var px = kind == "text" ? (hasIcon ? 8 : 12) : (hasIcon ? 16 : 24);
    var st = { padding: { top: 10, right: px, bottom: 10, left: px }, radius: Material._corner(Material.shape.full),
               align_items: "center", justify_content: "center", gap: 8,
               cursor: disabled ? "not-allowed" : "pointer" };
    if (bg) { st.background = solid(disabled ? withAlpha(cs.onSurface, 0.12) : bg); }
    if (border) { st.border_width = 1; st.border_color = disabled ? withAlpha(cs.onSurface, 0.12) : border; }
    if (elevation > 0 && !disabled) { st.shadows = Material.elevationDp(elevation); }
    if (has(this.props, "width")) { st.width = Material._len(this.props.width); }
    var fgc = disabled ? withAlpha(cs.onSurface, 0.38) : fg;
    var content = [];
    if (hasIcon) { push(content, icon(get(Material._iconMap, this.props.icon, this.props.icon), { size: 18, color: fgc })); }
    if (has(this.props, "label")) { push(content, new MaterialText({ text: this.props.label, variant: "labelLarge", color: fgc }).build()); }
    content = concat(content, this.kids());
    var inkOpts = { color: withAlpha(fgc, 0.12) };
    if (has(this.props, "pulse")) { inkOpts.pulse = this.props.pulse; }
    var inner = Material._ink(row({ style: { align_items: "center", justify_content: "center", gap: 8 }, children: content }), inkOpts);
    var n = row({ style: st, children: [inner] });
    if (has(this.props, "onClick") && !disabled) { on(n, "click", this.props.onClick); }
    return this.finish(n);
  }
}

class ElevatedButton extends _ButtonBase {
  kind() { return "elevated"; }
  static icon(o) { return new ElevatedButton(o).build(); }
}
Material.ElevatedButton = ElevatedButton;
Material.elevatedButton = function (o) { return new ElevatedButton(o).build(); };

class FilledButton extends _ButtonBase {
  kind() { return this.opt("_tonal", false) ? "tonal" : "filled"; }
  static tonal(o) { return new FilledButton(merge(Material._opt(o), { _tonal: true })).build(); }
  static icon(o) { return new FilledButton(o).build(); }
}
Material.FilledButton = FilledButton;
Material.filledButton = function (o) { return new FilledButton(o).build(); };
Material.filledTonalButton = function (o) { return FilledButton.tonal(o); };

class OutlinedButton extends _ButtonBase {
  kind() { return "outlined"; }
  static icon(o) { return new OutlinedButton(o).build(); }
}
Material.OutlinedButton = OutlinedButton;
Material.outlinedButton = function (o) { return new OutlinedButton(o).build(); };

class TextButton extends _ButtonBase {
  kind() { return "text"; }
  static icon(o) { return new TextButton(o).build(); }
}
Material.TextButton = TextButton;
Material.textButton = function (o) { return new TextButton(o).build(); };

// `IconButton` — a single tappable icon, `variant`: "standard" (default,
// no fill), "filled", "filledTonal", "outlined".
class IconButton extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var variant = this.opt("variant", "standard");
    var disabled = this.opt("disabled", false);
    var sz = this.opt("size", 40);
    var bg = null; var fg = cs.onSurfaceVariant; var border = null;
    if (variant == "filled") { bg = this.opt("backgroundColor", cs.primary); fg = this.opt("foregroundColor", cs.onPrimary); }
    else if (variant == "filledTonal") { bg = this.opt("backgroundColor", cs.secondaryContainer); fg = this.opt("foregroundColor", cs.onSecondaryContainer); }
    else if (variant == "outlined") { border = cs.outlineVariant; fg = this.opt("foregroundColor", cs.onSurfaceVariant); }
    else { fg = this.opt("foregroundColor", cs.onSurfaceVariant); }
    if (this.opt("selected", false)) { bg = this.opt("selectedBackgroundColor", cs.primary); fg = this.opt("selectedForegroundColor", cs.onPrimary); }
    var fgc = disabled ? withAlpha(cs.onSurface, 0.38) : fg;
    var st = { width: Material._len(sz), height: Material._len(sz), radius: Material._corner(Material.shape.full),
               align_items: "center", justify_content: "center", cursor: disabled ? "not-allowed" : "pointer" };
    if (bg) { st.background = solid(disabled ? withAlpha(cs.onSurface, 0.12) : bg); }
    if (border) { st.border_width = 1; st.border_color = border; }
    var inkOpts = { color: withAlpha(fgc, 0.16) };
    if (has(this.props, "pulse")) { inkOpts.pulse = this.props.pulse; }
    var glyph = icon(get(Material._iconMap, this.opt("icon", "circle"), this.opt("icon", "circle")), { size: sz * 0.55, color: fgc });
    var n = row({ style: st, children: [Material._ink(glyph, inkOpts)] });
    if (has(this.props, "onClick") && !disabled) { on(n, "click", this.props.onClick); }
    return this.finish(n);
  }
  static filled(o) { return new IconButton(merge(Material._opt(o), { variant: "filled" })).build(); }
  static filledTonal(o) { return new IconButton(merge(Material._opt(o), { variant: "filledTonal" })).build(); }
  static outlined(o) { return new IconButton(merge(Material._opt(o), { variant: "outlined" })).build(); }
}
Material.IconButton = IconButton;
Material.iconButton = function (o) { return new IconButton(o).build(); };

// `FloatingActionButton` — `size`: "small" (40, radius medium), "regular"
// (56, radius large — the default), "large" (96, radius extraLarge); or set
// `extended: true` with a `label` for the pill-shaped extended FAB.
class FloatingActionButton extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var size = this.opt("size", "regular");
    var dims = { small: 40, regular: 56, large: 96 };
    var radii = { small: Material.shape.medium, regular: Material.shape.large, large: Material.shape.extraLarge };
    var sz = get(dims, size, 56);
    var radius = get(radii, size, Material.shape.large);
    var bg = this.opt("backgroundColor", cs.primaryContainer);
    var fg = this.opt("foregroundColor", cs.onPrimaryContainer);
    var extended = this.opt("extended", false);
    var st = { align_items: "center", justify_content: "center", gap: 12,
               radius: Material._corner(radius), background: solid(bg),
               shadows: Material.elevationDp(this.opt("elevation", 6)), cursor: "pointer" };
    if (extended) { st.padding = { top: 16, right: 20, bottom: 16, left: 16 }; st.height = Material._len(56); }
    else { st.width = Material._len(sz); st.height = Material._len(sz); }
    var content = [];
    if (has(this.props, "icon")) { push(content, icon(get(Material._iconMap, this.props.icon, this.props.icon), { size: sz * 0.32 + (extended ? 24 - sz * 0.32 : 0), color: fg })); }
    if (extended && has(this.props, "label")) { push(content, new MaterialText({ text: this.props.label, variant: "labelLarge", color: fg }).build()); }
    var inkOpts = { color: withAlpha(fg, 0.16), radius: Material._corner(radius) };
    if (has(this.props, "pulse")) { inkOpts.pulse = this.props.pulse; }
    var inner = Material._ink(row({ style: { align_items: "center", justify_content: "center", gap: 12 }, children: content }), inkOpts);
    var n = row({ style: st, children: [inner] });
    if (has(this.props, "onClick")) { on(n, "click", this.props.onClick); }
    return this.finish(n);
  }
  static small(o) { return new FloatingActionButton(merge(Material._opt(o), { size: "small" })).build(); }
  static large(o) { return new FloatingActionButton(merge(Material._opt(o), { size: "large" })).build(); }
  static extended(o) { return new FloatingActionButton(merge(Material._opt(o), { extended: true })).build(); }
}
Material.FloatingActionButton = FloatingActionButton;
Material.fab = function (o) { return new FloatingActionButton(o).build(); };

// `ToggleButtons` — a row of mutually-independent toggle icon/label cells.
class ToggleButtons extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var items = this.opt("children", []);
    var isSelected = this.opt("isSelected", []);
    var cells = Material._map(items, function (child, i) {
      var on_ = isSelected[i];
      var st = { padding: { top: 10, right: 14, bottom: 10, left: 14 }, align_items: "center", justify_content: "center", cursor: "pointer" };
      if (on_) { st.background = solid(cs.secondaryContainer); }
      var n = row({ style: st, children: [child] });
      return n;
    });
    var st2 = { border_width: 1, border_color: cs.outline, radius: Material._corner(Material.shape.small), overflow_x: "hidden", overflow_y: "hidden" };
    var n = row({ style: st2, children: cells });
    if (has(this.props, "onPressed")) { on(n, "click", this.props.onPressed); }
    return this.finish(n);
  }
}
Material.ToggleButtons = ToggleButtons;
Material.toggleButtons = function (o) { return new ToggleButtons(o).build(); };

// `SegmentedButton` — M3's single/multi-select segmented control.
class SegmentedButton extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var segments = this.opt("segments", []);
    var selected = this.opt("selected", []);
    var cells = Material._map(segments, function (seg, i) {
      var active = contains(selected, seg.value);
      var st = { padding: { top: 10, right: 16, bottom: 10, left: 16 }, align_items: "center", justify_content: "center",
                 gap: 6, flex_grow: 1, cursor: "pointer" };
      if (active) { st.background = solid(cs.secondaryContainer); }
      var kids = [];
      if (active) { push(kids, icon("check", { size: 16, color: cs.onSecondaryContainer })); }
      else if (has(seg, "icon")) { push(kids, icon(get(Material._iconMap, seg.icon, seg.icon), { size: 16, color: cs.onSurface })); }
      push(kids, new MaterialText({ text: seg.label, variant: "labelLarge", color: active ? cs.onSecondaryContainer : cs.onSurface }).build());
      var cell = row({ style: st, children: kids });
      return cell;
    });
    var st2 = { border_width: 1, border_color: cs.outline, radius: Material._corner(Material.shape.small), overflow_x: "hidden", overflow_y: "hidden" };
    var track = row({ style: st2, children: cells });
    if (has(this.props, "onSelectionChanged")) { on(track, "change", this.props.onSelectionChanged); }
    return this.finish(track);
  }
}
Material.SegmentedButton = SegmentedButton;
Material.segmentedButton = function (o) { return new SegmentedButton(o).build(); };

// `DropdownButton` — wraps the native `dropdown` widget (native drag/caret
// handling is required, see the module doc) with Material's underline style.
class DropdownButton extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var opts = Material._map(this.opt("items", []), function (it) {
      return { value: "" + it.value, label: it.label, disabled: Material._d(it, "disabled", false) };
    });
    var props = { options: opts };
    if (has(this.props, "value")) { props.selected = "" + this.props.value; }
    if (has(this.props, "hint")) { props.placeholder = this.props.hint; }
    props.disabled = this.opt("disabled", false);
    props.style = { border_width: 0, foreground: cs.onSurface, padding: { top: 4, right: 4, bottom: 8, left: 4 } };
    var n = dropdown(props);
    if (has(this.props, "onChanged")) { on(n, "change", this.props.onChanged); }
    return this.finish(n);
  }
}
Material.DropdownButton = DropdownButton;
Material.dropdownButton = function (o) { return new DropdownButton(o).build(); };

// `PopupMenuButton` — an icon button that, when `open` is true, shows a
// glass-free Material menu card in a popover overlay layer.
class PopupMenuButton extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var trigger = has(this.props, "child") ? this.props.child
      : new IconButton({ icon: this.opt("icon", "more_vert"), onClick: this.opt("onOpen", "") }).build();
    var kids = [trigger];
    if (this.opt("open", false)) {
      var items = Material._map(this.opt("items", []), function (it) {
        var st = { padding: { top: 10, right: 16, bottom: 10, left: 16 }, gap: 10, align_items: "center", cursor: "pointer" };
        var cell = row({ style: st, children: [new MaterialText({ text: it.label, variant: "bodyLarge", color: cs.onSurface }).build()] });
        if (has(it, "onSelect")) { on(cell, "click", it.onSelect); }
        return cell;
      });
      var menu = column({ style: { background: solid(cs.surfaceContainer), radius: Material._corner(Material.shape.extraSmall),
                                    shadows: Material.elevationDp(3), padding: { top: 4, right: 0, bottom: 4, left: 0 },
                                    min_width: Material._len(160) },
                          children: items });
      var ov = overlay({ layer: "popover", backdrop: false, dismissible: true, children: [menu] });
      if (has(this.props, "onDismiss")) { on(ov, "dismiss", this.props.onDismiss); }
      push(kids, ov);
    }
    return this.finish(stack({ children: kids }));
  }
}
Material.PopupMenuButton = PopupMenuButton;
Material.popupMenuButton = function (o) { return new PopupMenuButton(o).build(); };

// ===========================================================================
// Selection controls & text input.
//
// `Checkbox`/`Radio`/`Switch` are simple tap targets (no drag), so they are
// reimplemented from scratch as plain divs + icons for exact Material color
// and shape fidelity (see the module doc). `Slider`/`RangeSlider`/`TextField`
// genuinely need native drag/caret handling, so they wrap the native
// `slider`/`text_input` widgets and layer Material decoration around them.
// ===========================================================================

class Checkbox extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var value = this.opt("value", false);
    var disabled = this.opt("disabled", false);
    var sz = 18;
    var st = { width: Material._len(sz), height: Material._len(sz), radius: Material._corner(2),
               align_items: "center", justify_content: "center", cursor: disabled ? "not-allowed" : "pointer" };
    var kids = [];
    if (value) {
      st.background = solid(disabled ? withAlpha(cs.onSurface, 0.38) : this.opt("activeColor", cs.primary));
      push(kids, icon("check", { size: 14, color: this.opt("checkColor", cs.onPrimary) }));
    } else {
      st.border_width = 2;
      st.border_color = disabled ? withAlpha(cs.onSurface, 0.38) : this.opt("borderColor", cs.outline);
    }
    var inkOpts = { color: withAlpha(cs.primary, 0.16), radius: Material._corner(9999) };
    if (has(this.props, "pulse")) { inkOpts.pulse = this.props.pulse; }
    var n = Material._ink(row({ style: st, children: kids }), inkOpts);
    if (has(this.props, "onChanged") && !disabled) { on(n, "click", this.props.onChanged); }
    return this.finish(n);
  }
}
Material.Checkbox = Checkbox;
Material.checkbox = function (o) { return new Checkbox(o).build(); };

Material._tileRow = function (leading, title, subtitle, trailing, th, onClick) {
  var texts = [];
  if (title != null) { push(texts, title); }
  if (subtitle != null) { push(texts, subtitle); }
  var mid = column({ style: { gap: 2, flex_grow: 1 }, children: texts });
  var kids = [];
  if (leading != null) { push(kids, leading); }
  push(kids, mid);
  if (trailing != null) { push(kids, trailing); }
  var st = { padding: { top: 8, right: 16, bottom: 8, left: 16 }, gap: 16, align_items: "center" };
  if (onClick) { st.cursor = "pointer"; }
  var n = row({ style: st, children: kids });
  if (onClick) { on(n, "click", onClick); }
  return n;
};

class CheckboxListTile extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var value = this.opt("value", false);
    var cbProps = { value: value, disabled: this.opt("disabled", false) };
    if (has(this.props, "onChanged")) { cbProps.onChanged = this.props.onChanged; }
    var cb = new Checkbox(cbProps).build();
    var titleNode = has(this.props, "title") ? new MaterialText({ text: this.props.title, variant: "bodyLarge", color: cs.onSurface }).build() : null;
    var subtitleNode = has(this.props, "subtitle") ? new MaterialText({ text: this.props.subtitle, variant: "bodyMedium", color: cs.onSurfaceVariant }).build() : null;
    var trailing = this.opt("controlAffinity", "leading") == "trailing";
    var leading = trailing ? null : cb;
    var trail = trailing ? cb : null;
    var onClick = has(this.props, "onChanged") ? this.props.onChanged : null;
    return this.finish(Material._tileRow(leading, titleNode, subtitleNode, trail, th, onClick));
  }
}
Material.CheckboxListTile = CheckboxListTile;
Material.checkboxListTile = function (o) { return new CheckboxListTile(o).build(); };

// `Radio<T>` — Flutter's own per-option API: `value`/`groupValue`/`onChanged`
// (a group is just several `Radio`s sharing a `groupValue`).
class Radio extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var selected = this.opt("value", null) == this.opt("groupValue", null);
    var disabled = this.opt("disabled", false);
    var sz = 20;
    var ringColor = disabled ? withAlpha(cs.onSurface, 0.38) : (selected ? this.opt("activeColor", cs.primary) : cs.onSurfaceVariant);
    var st = { width: Material._len(sz), height: Material._len(sz), radius: Material._corner(sz),
               border_width: 2, border_color: ringColor, align_items: "center", justify_content: "center",
               cursor: disabled ? "not-allowed" : "pointer" };
    var kids = [];
    if (selected) {
      push(kids, div({ style: { width: Material._len(10), height: Material._len(10), radius: Material._corner(10), background: solid(ringColor) } }));
    }
    var inkOpts = { color: withAlpha(cs.primary, 0.16), radius: Material._corner(9999) };
    if (has(this.props, "pulse")) { inkOpts.pulse = this.props.pulse; }
    var n = Material._ink(row({ style: st, children: kids }), inkOpts);
    if (has(this.props, "onChanged") && !disabled) { on(n, "click", this.props.onChanged); }
    return this.finish(n);
  }
}
Material.Radio = Radio;
Material.radio = function (o) { return new Radio(o).build(); };

class RadioListTile extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var rProps = { value: this.opt("value", null), groupValue: this.opt("groupValue", null), disabled: this.opt("disabled", false) };
    if (has(this.props, "onChanged")) { rProps.onChanged = this.props.onChanged; }
    var r = new Radio(rProps).build();
    var titleNode = has(this.props, "title") ? new MaterialText({ text: this.props.title, variant: "bodyLarge", color: cs.onSurface }).build() : null;
    var subtitleNode = has(this.props, "subtitle") ? new MaterialText({ text: this.props.subtitle, variant: "bodyMedium", color: cs.onSurfaceVariant }).build() : null;
    var trailing = this.opt("controlAffinity", "leading") == "trailing";
    var leading = trailing ? null : r;
    var trail = trailing ? r : null;
    var onClick = has(this.props, "onChanged") ? this.props.onChanged : null;
    return this.finish(Material._tileRow(leading, titleNode, subtitleNode, trail, th, onClick));
  }
}
Material.RadioListTile = RadioListTile;
Material.radioListTile = function (o) { return new RadioListTile(o).build(); };

// `Switch` — M3 52x32 track, a 16dp thumb off / 24dp thumb on, sliding via a
// layout `transition` so the thumb *animates* between renders when `value`
// flips (rather than re-implementing drag).
class Switch extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var value = this.opt("value", false);
    var disabled = this.opt("disabled", false);
    var st = { width: Material._len(52), height: Material._len(32), radius: Material._corner(16),
               align_items: "center", cursor: disabled ? "not-allowed" : "pointer",
               justify_content: value ? "end" : "start", padding: { top: 4, right: 4, bottom: 4, left: 4 } };
    if (value) {
      st.background = solid(disabled ? withAlpha(cs.onSurface, 0.12) : this.opt("activeColor", cs.primary));
    } else {
      st.background = solid(disabled ? withAlpha(cs.onSurface, 0.12) : cs.surfaceContainerHighest);
      st.border_width = 2; st.border_color = disabled ? withAlpha(cs.onSurface, 0.38) : cs.outline;
    }
    var thumbSize = value ? 24 : 16;
    var thumbColor = value ? (disabled ? cs.surface : this.opt("thumbColor", cs.onPrimary))
                            : (disabled ? withAlpha(cs.onSurface, 0.38) : cs.outline);
    var thumb = div({
      style: { width: Material._len(thumbSize), height: Material._len(thumbSize), radius: Material._corner(thumbSize), background: solid(thumbColor) },
      transition: { curve: Material.tween(Material.motion.duration.short4, "ease_in_out"), enabled: true }
    });
    var n = row({ style: st, children: [thumb] });
    if (has(this.props, "onChanged") && !disabled) { on(n, "click", this.props.onChanged); }
    return this.finish(n);
  }
}
Material.Switch = Switch;
Material.switchWidget = function (o) { return new Switch(o).build(); };

class SwitchListTile extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var swProps = { value: this.opt("value", false), disabled: this.opt("disabled", false) };
    if (has(this.props, "onChanged")) { swProps.onChanged = this.props.onChanged; }
    var sw = new Switch(swProps).build();
    var titleNode = has(this.props, "title") ? new MaterialText({ text: this.props.title, variant: "bodyLarge", color: cs.onSurface }).build() : null;
    var subtitleNode = has(this.props, "subtitle") ? new MaterialText({ text: this.props.subtitle, variant: "bodyMedium", color: cs.onSurfaceVariant }).build() : null;
    var leading = this.opt("controlAffinity", "trailing") == "leading" ? sw : null;
    var trail = this.opt("controlAffinity", "trailing") == "leading" ? null : sw;
    var onClick = has(this.props, "onChanged") ? this.props.onChanged : null;
    return this.finish(Material._tileRow(leading, titleNode, subtitleNode, trail, th, onClick));
  }
}
Material.SwitchListTile = SwitchListTile;
Material.switchListTile = function (o) { return new SwitchListTile(o).build(); };

// `Slider`/`RangeSlider` wrap the native drag-tracked `slider` widget (see
// module doc — color theming of the native track/thumb isn't exposed by the
// host yet, so these inherit its fixed accent color).
class Slider extends StatelessWidget {
  build() {
    var props = { value: this.opt("value", 0), min: this.opt("min", 0), max: this.opt("max", 1) };
    if (has(this.props, "divisions")) { props.step = (this.opt("max", 1) - this.opt("min", 0)) / this.props.divisions; }
    props.disabled = this.opt("disabled", false);
    var n = slider(props);
    if (has(this.props, "onChanged")) { on(n, "change", this.props.onChanged); }
    return this.finish(n);
  }
}
Material.Slider = Slider;
Material.slider = function (o) { return new Slider(o).build(); };

class RangeSlider extends StatelessWidget {
  build() {
    var v = this.opt("values", { start: 0, end: 1 });
    var props = { value: v.start, value_end: v.end, min: this.opt("min", 0), max: this.opt("max", 1) };
    if (has(this.props, "divisions")) { props.step = (this.opt("max", 1) - this.opt("min", 0)) / this.props.divisions; }
    props.disabled = this.opt("disabled", false);
    var n = slider(props);
    if (has(this.props, "onChanged")) { on(n, "change", this.props.onChanged); }
    return this.finish(n);
  }
}
Material.RangeSlider = RangeSlider;
Material.rangeSlider = function (o) { return new RangeSlider(o).build(); };

// `TextField` — M3 filled or outlined (default) decoration around the
// native `text_input` (native caret/selection handling, see module doc).
class TextField extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var variant = this.opt("variant", "outlined");
    var hasError = this.opt("error", false) || has(this.props, "errorText");
    var props = { value: "" + this.opt("value", "") };
    if (has(this.props, "placeholder")) { props.placeholder = this.props.placeholder; }
    else if (has(this.props, "hintText")) { props.placeholder = this.props.hintText; }
    props.input_type = this.opt("obscureText", false) ? "password" : this.opt("keyboardType", "text");
    props.disabled = !this.opt("enabled", true);
    if (has(this.props, "maxLength")) { props.max_length = this.props.maxLength; }
    var st = { padding: { top: 14, right: 14, bottom: 14, left: 14 }, foreground: cs.onSurface };
    if (variant == "filled") {
      st.background = solid(cs.surfaceContainerHighest);
      st.radius = Material._cornerTop(Material.shape.extraSmall);
    } else {
      st.background = solid(cs.surface);
      st.border_width = 1;
      st.border_color = hasError ? cs.error : cs.outline;
      st.radius = Material._corner(Material.shape.extraSmall);
    }
    props.style = st;
    var input = textInput(props);
    if (has(this.props, "onChanged")) { on(input, "input", this.props.onChanged); }
    if (has(this.props, "onSubmitted")) { on(input, "submit", this.props.onSubmitted); }
    var kids = [];
    if (has(this.props, "labelText")) { push(kids, new MaterialText({ text: this.props.labelText, variant: "bodySmall", color: cs.onSurfaceVariant }).build()); }
    push(kids, input);
    if (has(this.props, "errorText")) { push(kids, new MaterialText({ text: this.props.errorText, variant: "bodySmall", color: cs.error }).build()); }
    else if (has(this.props, "helperText")) { push(kids, new MaterialText({ text: this.props.helperText, variant: "bodySmall", color: cs.onSurfaceVariant }).build()); }
    return this.finish(column({ style: { gap: 4 }, children: kids }));
  }
}
Material.TextField = TextField;
Material.textField = function (o) { return new TextField(o).build(); };

class TextFormField extends TextField { }
Material.TextFormField = TextFormField;
Material.textFormField = function (o) { return new TextFormField(o).build(); };

// ===========================================================================
// Chips — `Chip`/`InputChip`/`ChoiceChip`/`FilterChip`/`ActionChip` share one
// visual recipe (32dp tall, `small` shape, `labelLarge` type) and differ only
// in Flutter by intent; here they're thin subclasses of one `Chip` base.
// ===========================================================================

class Chip extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var selected = this.opt("selected", false);
    var disabled = this.opt("disabled", false);
    var st = { height: Material._len(32), padding: { top: 6, right: 12, bottom: 6, left: 12 },
               radius: Material._corner(Material.shape.small), align_items: "center", justify_content: "center",
               gap: 8, cursor: disabled ? "not-allowed" : (has(this.props, "onClick") ? "pointer" : "default") };
    var fg;
    if (selected) { st.background = solid(cs.secondaryContainer); fg = cs.onSecondaryContainer; }
    else { st.background = solid(cs.surfaceContainerLow); st.border_width = 1; st.border_color = cs.outlineVariant; fg = cs.onSurfaceVariant; }
    if (disabled) { fg = withAlpha(cs.onSurface, 0.38); }
    var kids = [];
    if (selected) { push(kids, icon("check", { size: 16, color: fg })); }
    else if (has(this.props, "avatar")) { push(kids, this.props.avatar); }
    else if (has(this.props, "icon")) { push(kids, icon(get(Material._iconMap, this.props.icon, this.props.icon), { size: 16, color: fg })); }
    push(kids, new MaterialText({ text: this.opt("label", ""), variant: "labelLarge", color: fg }).build());
    if (this.opt("deletable", false) || has(this.props, "onDeleted")) {
      var delIcon = icon("close", { size: 16, color: fg });
      if (has(this.props, "onDeleted")) { on(delIcon, "click", this.props.onDeleted); }
      push(kids, delIcon);
    }
    var n = row({ style: st, children: kids });
    if (has(this.props, "onClick") && !disabled) { on(n, "click", this.props.onClick); }
    return this.finish(n);
  }
}
Material.Chip = Chip;
Material.chip = function (o) { return new Chip(o).build(); };

class InputChip extends Chip { }
Material.InputChip = InputChip;
Material.inputChip = function (o) { return new InputChip(o).build(); };

class ChoiceChip extends Chip { }
Material.ChoiceChip = ChoiceChip;
Material.choiceChip = function (o) { return new ChoiceChip(o).build(); };

class FilterChip extends Chip { }
Material.FilterChip = FilterChip;
Material.filterChip = function (o) { return new FilterChip(o).build(); };

class ActionChip extends Chip { }
Material.ActionChip = ActionChip;
Material.actionChip = function (o) { return new ActionChip(o).build(); };

// ===========================================================================
// Indicators: progress, badge, tooltip.
//
// The determinate progress indicators are custom (div fill / canvas arc) for
// exact color control; the indeterminate case (no `value`) falls back to the
// native `progress_bar` widget so it gets the host's real looping animation
// rather than an approximated one (see module doc on native-widget limits).
// ===========================================================================

class LinearProgressIndicator extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    if (!has(this.props, "value")) { return this.finish(progressBar({ value: null, shape: "linear" })); }
    var value = this.props.value;
    if (value < 0) { value = 0; }
    if (value > 1) { value = 1; }
    var trackColor = this.opt("backgroundColor", cs.surfaceContainerHighest);
    var color = this.opt("color", cs.primary);
    var track = { width: Material._len("full"), height: Material._len(4), radius: Material._corner(2),
                  background: solid(trackColor), position: "relative", overflow_x: "hidden", overflow_y: "hidden" };
    var fill = div({
      style: { position: "absolute", inset: { top: 0, left: 0, bottom: 0, right: 0 },
               width: { unit: "percent", value: value * 100 }, height: Material._len("full"),
               radius: Material._corner(2), background: solid(color) },
      transition: { curve: Material.tween(Material.motion.duration.medium2, "ease_in_out"), enabled: true }
    });
    return this.finish(div({ style: track, children: [fill] }));
  }
}
Material.LinearProgressIndicator = LinearProgressIndicator;
Material.linearProgressIndicator = function (o) { return new LinearProgressIndicator(o).build(); };

class CircularProgressIndicator extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    if (!has(this.props, "value")) { return this.finish(progressBar({ value: null, shape: "circular" })); }
    var value = this.props.value;
    if (value < 0) { value = 0; }
    if (value > 1) { value = 1; }
    var size = this.opt("size", 40);
    var strokeWidth = this.opt("strokeWidth", 4);
    var cx = size / 2;
    var cy = size / 2;
    var r = (size - strokeWidth) / 2;
    var trackColor = this.opt("backgroundColor", rgba(0, 0, 0, 0));
    var color = this.opt("color", cs.primary);
    var ops = [
      { type: "clear", color: rgba(0, 0, 0, 0) },
      { type: "arc", center: { x: cx, y: cy }, radius: r, start_angle: -90, end_angle: 270,
        stroke: { brush: solid(trackColor), width: strokeWidth, cap: "round" } },
      { type: "arc", center: { x: cx, y: cy }, radius: r, start_angle: -90, end_angle: -90 + 360 * value,
        stroke: { brush: solid(color), width: strokeWidth, cap: "round" } }
    ];
    return this.finish(canvas(ops, { style: { width: Material._len(size), height: Material._len(size) } }));
  }
}
Material.CircularProgressIndicator = CircularProgressIndicator;
Material.circularProgressIndicator = function (o) { return new CircularProgressIndicator(o).build(); };

// `Badge` — a small count/dot overlay; wraps `child` when given, else renders
// standalone.
class Badge extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var visible = this.opt("isLabelVisible", true);
    var label = this.opt("label", null);
    var bg = this.opt("backgroundColor", cs.error);
    var fg = this.opt("textColor", cs.onError);
    var child = this.opt("child", null);
    if (!visible) { return this.finish(child ? child : div({})); }
    var dotOnly = label == null;
    var badgeSt = dotOnly
      ? { width: Material._len(6), height: Material._len(6), radius: Material._corner(3), background: solid(bg) }
      : { min_width: Material._len(16), height: Material._len(16), radius: Material._corner(8), background: solid(bg),
          align_items: "center", justify_content: "center", padding: { top: 0, right: 4, bottom: 0, left: 4 } };
    var badgeKids = dotOnly ? [] : [new MaterialText({ text: "" + label, variant: "labelSmall", color: fg }).build()];
    var badge = row({ style: badgeSt, children: badgeKids });
    if (!child) { return this.finish(badge); }
    var positioned = div({ style: { position: "absolute", inset: { top: -4, right: -4, bottom: 0, left: 0 } }, children: [badge] });
    return this.finish(stack({ children: [child, positioned] }));
  }
}
Material.Badge = Badge;
Material.badge = function (o) { return new Badge(o).build(); };

// `Tooltip` — a floating label shown while `visible` (no hover-state
// tracking seam, see module doc; the host toggles `visible` explicitly).
class Tooltip extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var child = this.opt("child", null);
    if (!this.opt("visible", false)) { return this.finish(child ? child : div({})); }
    var msgNode = row({
      style: { background: solid(cs.inverseSurface), padding: { top: 6, right: 8, bottom: 6, left: 8 }, radius: Material._corner(4) },
      children: [new MaterialText({ text: this.opt("message", ""), variant: "bodySmall", color: cs.onInverseSurface }).build()]
    });
    var ov = overlay({ layer: "tooltip", backdrop: false, dismissible: false, children: [msgNode] });
    return this.finish(stack({ children: child ? [child, ov] : [ov] }));
  }
}
Material.Tooltip = Tooltip;
Material.tooltip = function (o) { return new Tooltip(o).build(); };

// ===========================================================================
// Dialogs, sheets & snack bars — modal/overlay content. Each widget builds
// the sheet *and* wraps it in the right overlay `layer` in one shot, so
// including one anywhere in the render tree (conditionally, on some app
// state) is exactly Flutter's `showDialog`/`showModalBottomSheet` effect.
// ===========================================================================

class AlertDialog extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var kids = [];
    if (has(this.props, "icon")) {
      push(kids, row({ style: { justify_content: "center" },
                       children: [icon(get(Material._iconMap, this.props.icon, this.props.icon), { size: 24, color: cs.secondary })] }));
    }
    if (has(this.props, "title")) { push(kids, new MaterialText({ text: this.props.title, variant: "headlineSmall", color: cs.onSurface }).build()); }
    if (has(this.props, "content")) {
      var c = this.props.content;
      push(kids, (typeOf(c) == "object" && has(c, "type")) ? c : new MaterialText({ text: c, variant: "bodyMedium", color: cs.onSurfaceVariant }).build());
    }
    if (has(this.props, "actions")) { push(kids, row({ style: { justify_content: "end", gap: 8 }, children: this.props.actions })); }
    var st = { background: solid(cs.surfaceContainerHigh), radius: Material._corner(Material.shape.extraLarge),
               padding: Material._edges(24), gap: 16, shadows: Material.elevationDp(6),
               width: Material._len(this.opt("width", 320)), max_width: Material._len("full") };
    var sheet = column({ style: st, children: kids });
    var ov = overlay({ layer: "modal", backdrop: true, dismissible: this.opt("barrierDismissible", true),
                       style: { align_items: "center", justify_content: "center", padding: Material._edges(24) },
                       children: [sheet] });
    if (has(this.props, "onDismiss")) { on(ov, "dismiss", this.props.onDismiss); }
    return this.finish(ov);
  }
}
Material.AlertDialog = AlertDialog;
Material.alertDialog = function (o) { return new AlertDialog(o).build(); };

// A single row inside a `SimpleDialog` (Flutter's `SimpleDialogOption`).
Material.simpleDialogOption = function (o) {
  o = Material._opt(o);
  var cs = Material.theme().colorScheme;
  var n = row({ style: { padding: { top: 12, right: 24, bottom: 12, left: 24 }, cursor: "pointer" },
                children: [new MaterialText({ text: Material._d(o, "text", ""), variant: "bodyLarge", color: cs.onSurface }).build()] });
  if (has(o, "onClick")) { on(n, "click", o.onClick); }
  return n;
};

class SimpleDialog extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var kids = [];
    if (has(this.props, "title")) {
      push(kids, div({ style: { padding: { top: 0, right: 24, bottom: 12, left: 24 } },
                       children: [new MaterialText({ text: this.props.title, variant: "titleLarge", color: cs.onSurface }).build()] }));
    }
    kids = concat(kids, this.kids());
    var st = { background: solid(cs.surfaceContainerHigh), radius: Material._corner(Material.shape.extraLarge),
               padding: { top: 24, right: 0, bottom: 8, left: 0 }, gap: 4, shadows: Material.elevationDp(6),
               width: Material._len(this.opt("width", 280)), max_width: Material._len("full") };
    var sheet = column({ style: st, children: kids });
    var ov = overlay({ layer: "modal", backdrop: true, dismissible: this.opt("barrierDismissible", true),
                       style: { align_items: "center", justify_content: "center", padding: Material._edges(24) },
                       children: [sheet] });
    if (has(this.props, "onDismiss")) { on(ov, "dismiss", this.props.onDismiss); }
    return this.finish(ov);
  }
}
Material.SimpleDialog = SimpleDialog;
Material.simpleDialog = function (o) { return new SimpleDialog(o).build(); };

// A raw `Dialog` shell (no title/content/actions layout opinion).
class Dialog extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var st = { background: solid(this.opt("backgroundColor", cs.surfaceContainerHigh)),
               radius: Material._corner(this.opt("shape", Material.shape.extraLarge)),
               shadows: Material.elevationDp(this.opt("elevation", 6)),
               padding: Material._edges(this.opt("padding", 24)),
               width: Material._len(this.opt("width", 320)), max_width: Material._len("full") };
    var sheet = column({ style: st, children: this.kids() });
    var ov = overlay({ layer: "modal", backdrop: true, dismissible: this.opt("barrierDismissible", true),
                       style: { align_items: "center", justify_content: "center", padding: Material._edges(24) },
                       children: [sheet] });
    if (has(this.props, "onDismiss")) { on(ov, "dismiss", this.props.onDismiss); }
    return this.finish(ov);
  }
}
Material.Dialog = Dialog;
Material.dialog = function (o) { return new Dialog(o).build(); };

// `showDialog` — the imperative-flavored helper: give it a zero-arg
// `builder` (or a ready-made `child`) and it returns the modal-wrapped node
// to drop into your render tree (there's no imperative overlay stack here —
// see the module doc — so "showing" a dialog just means rendering it).
Material.showDialog = function (o) {
  o = Material._opt(o);
  var content = has(o, "builder") ? o.builder() : (has(o, "child") ? o.child : div({}));
  var ov = overlay({ layer: "modal", backdrop: true, dismissible: Material._d(o, "barrierDismissible", true),
                     style: { align_items: "center", justify_content: "center", padding: Material._edges(24) },
                     children: [content] });
  if (has(o, "onDismiss")) { on(ov, "dismiss", o.onDismiss); }
  return ov;
};

// `BottomSheet` — a drag-handle sheet anchored to the bottom, modal by
// default (dimmed backdrop) or inline (`modal: false`, e.g. a persistent
// sheet at the end of a `Scaffold`).
class BottomSheetWidget extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var handle = div({ style: { width: Material._len(32), height: Material._len(4), radius: Material._corner(2),
                                background: solid(withAlpha(cs.onSurfaceVariant, 0.4)), align_self: "center" } });
    var st = { background: solid(this.opt("backgroundColor", cs.surfaceContainerLow)),
               radius: Material._cornerTop(Material.shape.extraLarge), width: Material._len("full"),
               padding: { top: 12, right: 16, bottom: 24, left: 16 }, gap: 16,
               shadows: Material.elevationDp(this.opt("elevation", 1)) };
    var kids = concat([handle], this.kids());
    var sheet = column({ style: st, children: kids });
    if (!this.opt("modal", true)) { return this.finish(sheet); }
    var ov = overlay({ layer: "modal", backdrop: true, dismissible: this.opt("dismissible", true),
                       style: { align_items: "stretch", justify_content: "end" }, children: [sheet] });
    if (has(this.props, "onDismiss")) { on(ov, "dismiss", this.props.onDismiss); }
    return this.finish(ov);
  }
}
Material.BottomSheetWidget = BottomSheetWidget;
Material.bottomSheet = function (o) { return new BottomSheetWidget(o).build(); };
Material.showModalBottomSheet = function (o) { return new BottomSheetWidget(merge(Material._opt(o), { modal: true })).build(); };

// `SnackBar` — a transient bottom (here: top-of-toast-layer) notification
// with an optional action button.
class SnackBar extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var kids = [new MaterialText({ text: this.opt("content", ""), variant: "bodyMedium", color: cs.onInverseSurface }).build(),
                spacer({ style: { flex_grow: 1 } })];
    if (has(this.props, "actionLabel")) {
      push(kids, new TextButton({ label: this.props.actionLabel, foregroundColor: cs.inversePrimary, onClick: this.opt("onAction", "") }).build());
    }
    var st = { background: solid(cs.inverseSurface), radius: Material._corner(Material.shape.extraSmall),
               padding: { top: 14, right: 8, bottom: 14, left: 16 }, gap: 8, align_items: "center",
               shadows: Material.elevationDp(3), width: Material._len(this.opt("width", 344)), max_width: Material._len("full") };
    var bar = row({ style: st, children: kids });
    var ov = overlay({ layer: "toast", backdrop: false, dismissible: true,
                       style: { align_items: "center", justify_content: "start", padding: Material._edges(16) }, children: [bar] });
    return this.finish(ov);
  }
}
Material.SnackBar = SnackBar;
Material.snackBar = function (o) { return new SnackBar(o).build(); };

// ===========================================================================
// Navigation.
// ===========================================================================

// `BottomNavigationBar` — the legacy (pre-M3) bottom bar: icon + always-on
// label, selected item tinted `primary`.
class BottomNavigationBar extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var items = this.opt("items", []);
    var current = this.opt("currentIndex", 0);
    var onTap = this.opt("onTap", null);
    var cells = Material._map(items, function (it, i) {
      var active = i == current;
      var color = active ? cs.primary : cs.onSurfaceVariant;
      var cell = column({
        style: { align_items: "center", justify_content: "center", gap: 4, flex_grow: 1, cursor: "pointer",
                 padding: { top: 12, right: 0, bottom: 12, left: 0 } },
        children: [icon(get(Material._iconMap, it.icon, it.icon), { size: 24, color: color }),
                   new MaterialText({ text: it.label, variant: "labelSmall", color: color }).build()]
      });
      if (onTap) { on(cell, "click", onTap + ":" + i); }
      return cell;
    });
    var st = { width: Material._len("full"), height: Material._len(80), align_items: "center",
               background: solid(this.opt("backgroundColor", cs.surfaceContainer)), shadows: Material.elevationDp(3) };
    return this.finish(row({ style: st, children: cells }));
  }
}
Material.BottomNavigationBar = BottomNavigationBar;
Material.bottomNavigationBar = function (o) { return new BottomNavigationBar(o).build(); };

// `NavigationBar` — the M3 bottom bar: a pill indicator (`secondaryContainer`)
// behind the selected icon, label always visible below.
class NavigationBar extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var items = this.opt("destinations", this.opt("items", []));
    var selected = this.opt("selectedIndex", 0);
    var onSelect = this.opt("onDestinationSelected", null);
    var cells = Material._map(items, function (it, i) {
      var active = i == selected;
      var iconColor = active ? cs.onSecondaryContainer : cs.onSurfaceVariant;
      var indicatorKids = [icon(get(Material._iconMap, it.icon, it.icon), { size: 24, color: iconColor })];
      var indicator = row({
        style: { width: Material._len(64), height: Material._len(32), radius: Material._corner(16),
                 align_items: "center", justify_content: "center",
                 background: active ? solid(cs.secondaryContainer) : solid(rgba(0, 0, 0, 0)) },
        children: indicatorKids
      });
      var cell = column({
        style: { align_items: "center", justify_content: "center", gap: 4, flex_grow: 1, cursor: "pointer",
                 padding: { top: 12, right: 0, bottom: 16, left: 0 } },
        children: [indicator, new MaterialText({ text: it.label, variant: "labelMedium", color: active ? cs.onSurface : cs.onSurfaceVariant }).build()]
      });
      if (onSelect) { on(cell, "click", onSelect + ":" + i); }
      return cell;
    });
    var st = { width: Material._len("full"), height: Material._len(80), align_items: "center",
               background: solid(this.opt("backgroundColor", cs.surfaceContainer)), shadows: Material.elevationDp(3) };
    return this.finish(row({ style: st, children: cells }));
  }
}
Material.NavigationBar = NavigationBar;
Material.navigationBar = function (o) { return new NavigationBar(o).build(); };

// `NavigationRail` — the vertical sibling of `NavigationBar`, for wide
// layouts; `extended: true` shows the label beside the icon instead of below.
class NavigationRail extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var items = this.opt("destinations", this.opt("items", []));
    var selected = this.opt("selectedIndex", 0);
    var onSelect = this.opt("onDestinationSelected", null);
    var extended = this.opt("extended", false);
    var cells = Material._map(items, function (it, i) {
      var active = i == selected;
      var iconColor = active ? cs.onSecondaryContainer : cs.onSurfaceVariant;
      var indicator = row({
        style: extended
          ? { width: Material._len("full"), height: Material._len(56), radius: Material._corner(28),
              align_items: "center", gap: 12, padding: { top: 0, right: 24, bottom: 0, left: 24 },
              background: active ? solid(cs.secondaryContainer) : solid(rgba(0, 0, 0, 0)) }
          : { width: Material._len(56), height: Material._len(32), radius: Material._corner(16),
              align_items: "center", justify_content: "center",
              background: active ? solid(cs.secondaryContainer) : solid(rgba(0, 0, 0, 0)) },
        children: [icon(get(Material._iconMap, it.icon, it.icon), { size: 24, color: iconColor })]
      });
      var kids = extended ? [indicator, new MaterialText({ text: it.label, variant: "labelLarge", color: active ? cs.onSurface : cs.onSurfaceVariant }).build()]
        : [indicator, new MaterialText({ text: it.label, variant: "labelMedium", color: active ? cs.onSurface : cs.onSurfaceVariant }).build()];
      var cell = extended
        ? row({ style: { align_items: "center", cursor: "pointer" }, children: kids })
        : column({ style: { align_items: "center", gap: 4, cursor: "pointer", padding: { top: 12, right: 0, bottom: 12, left: 0 } }, children: kids });
      if (onSelect) { on(cell, "click", onSelect + ":" + i); }
      return cell;
    });
    var kids2 = [];
    if (has(this.props, "leading")) { push(kids2, this.props.leading); }
    kids2 = concat(kids2, cells);
    var st = { width: Material._len(extended ? 220 : 80), height: Material._len("full"), align_items: extended ? "stretch" : "center",
               gap: 12, padding: Material._edges(8), background: solid(this.opt("backgroundColor", cs.surface)) };
    return this.finish(column({ style: st, children: kids2 }));
  }
}
Material.NavigationRail = NavigationRail;
Material.navigationRail = function (o) { return new NavigationRail(o).build(); };

// `Drawer`/`NavigationDrawer` — a modal side sheet listing selectable rows.
class Drawer extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var items = this.opt("items", []);
    var selected = this.opt("selectedIndex", -1);
    var rows = Material._map(items, function (it, i) {
      var active = i == selected;
      var st = { padding: { top: 14, right: 16, bottom: 14, left: 16 }, gap: 12, align_items: "center", cursor: "pointer" };
      if (active) { st.background = solid(cs.secondaryContainer); st.radius = Material._corner(Material.shape.full); }
      var kids = [];
      if (has(it, "icon")) { push(kids, icon(get(Material._iconMap, it.icon, it.icon), { size: 24, color: active ? cs.onSecondaryContainer : cs.onSurfaceVariant })); }
      push(kids, new MaterialText({ text: it.label, variant: "labelLarge", color: active ? cs.onSecondaryContainer : cs.onSurfaceVariant }).build());
      var row_ = row({ style: st, children: kids });
      if (has(it, "onClick")) { on(row_, "click", it.onClick); }
      return row_;
    });
    var kids2 = [];
    if (has(this.props, "header")) { push(kids2, this.props.header); }
    kids2 = concat(kids2, rows);
    var st2 = { width: Material._len(this.opt("width", 320)), height: Material._len("full"),
                background: solid(this.opt("backgroundColor", cs.surface)),
                radius: Material._corner(this.opt("end", false) ? 0 : 0),
                padding: Material._edges(12), gap: 4 };
    var sheet = column({ style: st2, children: kids2 });
    if (!this.opt("modal", true)) { return this.finish(sheet); }
    var ov = overlay({ layer: "modal", backdrop: true, dismissible: this.opt("dismissible", true),
                       style: { align_items: "stretch", justify_content: this.opt("end", false) ? "end" : "start" }, children: [sheet] });
    if (has(this.props, "onDismiss")) { on(ov, "dismiss", this.props.onDismiss); }
    return this.finish(ov);
  }
}
Material.Drawer = Drawer;
Material.drawer = function (o) { return new Drawer(o).build(); };
class NavigationDrawer extends Drawer { }
Material.NavigationDrawer = NavigationDrawer;
Material.navigationDrawer = function (o) { return new NavigationDrawer(o).build(); };

// `TabBar` — equal-width tabs with an M3 underline indicator (a 3dp strip
// under the active tab, built as a flex sibling rather than an absolutely
// percent-positioned overlay, since the engine's inset edges are plain
// pixels with no percent unit — see the module doc on engine constraints).
class TabBar extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var tabsList = this.opt("tabs", []);
    var selected = this.opt("selected", 0);
    var onSelect = this.opt("onSelect", null);
    var cells = Material._map(tabsList, function (t, i) {
      var active = i == selected;
      var label = typeOf(t) == "object" ? t.label : t;
      var content = [];
      if (typeOf(t) == "object" && has(t, "icon")) { push(content, icon(get(Material._iconMap, t.icon, t.icon), { size: 20, color: active ? cs.primary : cs.onSurfaceVariant })); }
      push(content, new MaterialText({ text: label, variant: "titleSmall", color: active ? cs.primary : cs.onSurfaceVariant }).build());
      var indicator = div({ style: { width: Material._len("full"), height: Material._len(3),
                                     radius: Material._cornerTop(3), background: solid(active ? cs.primary : rgba(0, 0, 0, 0)) } });
      var cell = column({
        style: { align_items: "center", justify_content: "center", gap: 8, flex_grow: 1, cursor: "pointer",
                 padding: { top: 12, right: 16, bottom: 0, left: 16 } },
        children: [row({ style: { gap: 8, align_items: "center" }, children: content }), indicator]
      });
      if (onSelect) { on(cell, "click", onSelect + ":" + i); }
      return cell;
    });
    var st = { width: Material._len("full"), align_items: "stretch",
               border_width: 0, background: solid(this.opt("backgroundColor", cs.surface)) };
    return this.finish(row({ style: st, children: cells }));
  }
}
Material.TabBar = TabBar;
Material.tabBar = function (o) { return new TabBar(o).build(); };

// `Stepper` — a vertical numbered sequence with a connecting line, the
// active step's content expanded beneath its header.
class Stepper extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var steps = this.opt("steps", []);
    var current = this.opt("currentStep", 0);
    var onTap = this.opt("onStepTapped", null);
    var n = len(steps);
    var rows = Material._map(steps, function (s, i) {
      var done = i < current;
      var active = i == current;
      var circleColor = done || active ? cs.primary : rgba(0, 0, 0, 0);
      var circleBorder = done || active ? cs.primary : cs.outline;
      var numColor = done || active ? cs.onPrimary : cs.onSurfaceVariant;
      var circle = row({
        style: { width: Material._len(28), height: Material._len(28), radius: Material._corner(14),
                 background: solid(circleColor), border_width: 2, border_color: circleBorder,
                 align_items: "center", justify_content: "center" },
        children: [done ? icon("check", { size: 16, color: numColor }) : new MaterialText({ text: "" + (i + 1), variant: "labelLarge", color: (done || active) ? numColor : cs.onSurfaceVariant }).build()]
      });
      var line = i < n - 1 ? div({ style: { width: Material._len(2), flex_grow: 1, background: solid(i < current ? cs.primary : cs.outlineVariant) } }) : null;
      var marker = column({ style: { align_items: "center", gap: 0 }, children: line ? [circle, line] : [circle] });
      var header = row({ style: { gap: 8, align_items: "center", cursor: onTap ? "pointer" : "default" },
                          children: [new MaterialText({ text: s.title, variant: "titleMedium", color: cs.onSurface }).build()] });
      if (onTap) { on(header, "click", onTap + ":" + i); }
      var bodyKids = [header];
      if (active && has(s, "content")) { push(bodyKids, div({ style: { padding: { top: 8, right: 0, bottom: 16, left: 0 } }, children: [s.content] })); }
      var body = column({ style: { gap: 4, flex_grow: 1, padding: { top: 0, right: 0, bottom: 8, left: 16 } }, children: bodyKids });
      return row({ style: { gap: 0 }, children: [marker, body] });
    });
    return this.finish(column({ style: { gap: 0 }, children: rows }));
  }
}
Material.Stepper = Stepper;
Material.stepper = function (o) { return new Stepper(o).build(); };

// ===========================================================================
// Lists & data display.
// ===========================================================================

// `ListTile` — leading/title/subtitle/trailing in a standard 56-72dp row.
class ListTile extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var selected = this.opt("selected", false);
    var enabled = this.opt("enabled", true);
    var titleColor = selected ? cs.primary : (enabled ? cs.onSurface : withAlpha(cs.onSurface, 0.38));
    var titleNode = has(this.props, "title") ? new MaterialText({ text: this.props.title, variant: "bodyLarge", color: titleColor }).build() : null;
    var subtitleNode = has(this.props, "subtitle") ? new MaterialText({ text: this.props.subtitle, variant: "bodyMedium", color: cs.onSurfaceVariant }).build() : null;
    var texts = [];
    if (titleNode) { push(texts, titleNode); }
    if (subtitleNode) { push(texts, subtitleNode); }
    var mid = column({ style: { gap: 2, flex_grow: 1 }, children: texts });
    var kids = [];
    if (has(this.props, "leading")) { push(kids, this.props.leading); }
    push(kids, mid);
    if (has(this.props, "trailing")) { push(kids, this.props.trailing); }
    var st = { padding: { top: 8, right: 16, bottom: 8, left: 16 }, gap: 16, align_items: "center" };
    if (selected) { st.background = solid(cs.secondaryContainer); }
    var clickable = has(this.props, "onClick") && enabled;
    if (clickable) { st.cursor = "pointer"; }
    var n = row({ style: st, children: kids });
    if (clickable) { on(n, "click", this.props.onClick); }
    return this.finish(n);
  }
}
Material.ListTile = ListTile;
Material.listTile = function (o) { return new ListTile(o).build(); };

// `ExpansionTile` — a `ListTile`-like header that reveals its children below
// when `expanded`.
class ExpansionTile extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var expanded = this.opt("expanded", false);
    var leftKids = [];
    if (has(this.props, "leading")) { push(leftKids, this.props.leading); }
    push(leftKids, new MaterialText({ text: this.opt("title", ""), variant: "titleMedium", color: cs.onSurface }).build());
    var header = row({
      style: { padding: { top: 12, right: 16, bottom: 12, left: 16 }, gap: 16, align_items: "center", justify_content: "between", cursor: "pointer" },
      children: [row({ style: { gap: 16, align_items: "center" }, children: leftKids }),
                 icon(expanded ? "expand_less" : "expand_more", { size: 24, color: cs.onSurfaceVariant })]
    });
    if (has(this.props, "onExpansionChanged")) { on(header, "click", this.props.onExpansionChanged); }
    var kids = [header];
    if (expanded) { push(kids, column({ style: { padding: { top: 0, right: 16, bottom: 16, left: 16 }, gap: 8 }, children: this.kids() })); }
    return this.finish(column({ style: { gap: 0 }, children: kids }));
  }
}
Material.ExpansionTile = ExpansionTile;
Material.expansionTile = function (o) { return new ExpansionTile(o).build(); };

// `ExpansionPanelList` — several `ExpansionTile`s, each panel item shaped
// like `{ title, leading?, expanded?, children? }`.
class ExpansionPanelList extends StatelessWidget {
  build() {
    var panels = this.opt("children", this.opt("panels", []));
    var onChanged = this.opt("expansionCallback", this.opt("onExpansionChanged", null));
    var rows = Material._map(panels, function (p, i) {
      var props = merge({}, p);
      if (onChanged) { props.onExpansionChanged = onChanged + ":" + i; }
      return new ExpansionTile(props).build();
    });
    return this.finish(column({ style: { gap: 8 }, children: rows }));
  }
}
Material.ExpansionPanelList = ExpansionPanelList;
Material.expansionPanelList = function (o) { return new ExpansionPanelList(o).build(); };

// `DataTable` — a bordered header + rows table; each row is either an array
// of cells or `{ cells: [...], selected?, onSelectChanged? }`.
class DataTable extends StatelessWidget {
  build() {
    var th = this.theme();
    var cs = th.colorScheme;
    var columns = this.opt("columns", []);
    var rows = this.opt("rows", []);
    var headerCells = Material._map(columns, function (c) {
      var label = typeOf(c) == "object" ? c.label : c;
      return row({ style: { flex_grow: 1, padding: { top: 12, right: 8, bottom: 12, left: 8 } },
                   children: [new MaterialText({ text: label, variant: "labelLarge", color: cs.onSurfaceVariant }).build()] });
    });
    var header = row({ children: headerCells });
    var divider = function () { return div({ style: { width: Material._len("full"), height: Material._len(1), background: solid(cs.outlineVariant) } }); };
    var flatRows = [header, divider()];
    for (var i = 0; i < len(rows); i = i + 1) {
      var r = rows[i];
      var rIsObj = typeOf(r) == "object";
      var cellsArr = (rIsObj && has(r, "cells")) ? r.cells : r;
      var cells = Material._map(cellsArr, function (c) {
        var node = (typeOf(c) == "object" && has(c, "type")) ? c : new MaterialText({ text: "" + c, variant: "bodyMedium", color: cs.onSurface }).build();
        return row({ style: { flex_grow: 1, padding: { top: 12, right: 8, bottom: 12, left: 8 } }, children: [node] });
      });
      var rowSt = { align_items: "center" };
      if (rIsObj && has(r, "selected") && r.selected) { rowSt.background = solid(cs.secondaryContainer); }
      var rowNode = row({ style: rowSt, children: cells });
      if (rIsObj && has(r, "onSelectChanged")) { on(rowNode, "click", r.onSelectChanged); }
      push(flatRows, rowNode);
      if (i < len(rows) - 1) { push(flatRows, divider()); }
    }
    var st = { background: solid(cs.surface), radius: Material._corner(Material.shape.medium), overflow_x: "hidden", overflow_y: "hidden" };
    return this.finish(column({ style: st, children: flatRows }));
  }
}
Material.DataTable = DataTable;
Material.dataTable = function (o) { return new DataTable(o).build(); };

// `Image` — a simple fit/rounding wrapper around the native `image` widget.
class MaterialImage extends StatelessWidget {
  build() {
    var st = {};
    if (has(this.props, "width")) { st.width = Material._len(this.props.width); }
    if (has(this.props, "height")) { st.height = Material._len(this.props.height); }
    if (has(this.props, "borderRadius")) { st.radius = Material._corner(this.props.borderRadius); st.overflow_x = "hidden"; st.overflow_y = "hidden"; }
    var props = { fit: this.opt("fit", "cover"), style: st };
    return this.finish(image(this.opt("src", ""), props));
  }
}
Material.Image = MaterialImage;
Material.image = function (o) { return new MaterialImage(o).build(); };

// `TabBarView` — pairs with `TabBar`: renders whichever child is at `index`.
class TabBarView extends StatelessWidget {
  build() {
    var idx = this.opt("index", 0);
    var kids = this.kids();
    var child = (idx >= 0 && idx < len(kids)) ? kids[idx] : null;
    return this.finish(column({ style: { width: Material._len("full"), flex_grow: 1 }, children: child ? [child] : [] }));
  }
}
Material.TabBarView = TabBarView;
Material.tabBarView = function (o) { return new TabBarView(o).build(); };
