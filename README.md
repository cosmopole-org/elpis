# Elpis

**Elpis** is a sandboxed Miniapp framework. The host application instantiates an
**Elpis sandbox** — an isolated instance powered by the [Elpian VM][elpa] — and
runs a **Miniapp written in JavaScript** inside it. The Miniapp controls a
[Blinc][blinc] UI entirely through a set of host-API bridge functions imported
into the VM, so all of Blinc's capabilities are driveable from sandboxed Elpian
programs without the guest ever touching the renderer (or the GPU, the
filesystem, or the network) directly.

```
            ┌──────────────── Elpis sandbox ─────────────────┐
  app.js ─▶ │  elpian-vm ──askHost("ui.render", tree)──▶      │
 (Miniapp)  │     ▲                    │   host bridge         │ ─▶ Blinc UI
            │     └──── onEvent(ev) ◀──┘  (diff + patch)       │ ◀─ UiEvent
            └────────────────────────────────────────────────-┘
```

This mirrors the Elpian-VM ⇄ Flutter bridge from the [Elpa project][elpa] —
rendering a widget DSL tree from the guest, patching the UI tree partially for
minimal overhead, and keeping all application state inside the VM — but targets
the Blinc GPU UI framework instead of Flutter.

## Workspace layout

| Crate | What it is |
|-------|-----------|
| **`crates/elpian-vm`** | The Elpian AST/bytecode VM, **vendored complete** from [`cosmopole-org/elpa`][elpa] (`rust/crates/elpian-vm`). Unmodified; its 93 tests pass as-is. Compiles JS → Elpian AST → bytecode and executes it, pausing on `askHost` to hand host calls back to the embedder. |
| **`crates/elpis-protocol`** | The wire protocol: a serializable **widget DSL tree** (`Node`/`NodeKind`) covering the full Blinc surface, a **keyed tree-diff** that turns two trees into a minimal patch script, and the **host-call envelope**. No Blinc/GPU dependency. |
| **`crates/elpis-host`** | The **sandbox runtime**. Owns an `elpian-vm` instance, runs a JS Miniapp, and services every `askHost` UI call against a retained widget tree and a pluggable backend. Routes UI events back into the VM. Capabilities (net/fs/module-import) are **denied by default**. |
| **`crates/elpis-blinc`** | The **Blinc backend**. A pure, tested lowering from the protocol tree to a blinc-flavored element description (`lower`), plus the live `blinc_layout`/`blinc_core` interpreter and windowed run loop behind the `blinc-backend` feature. |
| **`apps/elpis-app`** | The host binary (`elpis`). Instantiates a sandbox, loads a Miniapp, and runs it — headless by default, or in a real Blinc window with `--features blinc`. Supports `--lib FILE` to prepend reusable SDK sources (e.g. the Glass UI kit). |
| **`sdk/`** | **`glass-ui-kit.js`** — the **Glass UI kit**, a full "liquid glass" component SDK written in sandbox JS on top of the Blinc builders (see below). |
| **`miniapps/`** | Example Miniapps written in JS (`counter`, `showcase`, `glass-gallery`). |

## How the bridge works

The vendored VM is renderer-agnostic: guest code calls
`askHost(apiName, [args])`, the VM pauses, and the embedder services the call and
resumes it with a JSON reply. Elpis rides its **entire UI protocol over this
seam** — `ui.render`, `ui.patch`, `ui.surfaceInfo`, `theme.*`, `router.*`,
`storage.*`, `anim.*`, `canvas.*`, `scene3d.*`, `media.*`, `host.send/request`,
plus `log`/`time.*`/`random.*`. **The VM is not modified**: these names map to
the VM's capability families (the UI/theme/router/storage/messaging families are
all gated by the `Other` capability), and the unused-name seam forwards them
verbatim.

A Miniapp's render path returns a `Node` tree; the host **diffs** it against the
retained tree and applies only the resulting patches, preserving widget state
and in-flight animations on untouched subtrees. Events flow back by the host
invoking the guest's `onEvent` / `onTick` / `onMessage` functions.

### A Miniapp

```js
var count = 0;
function view() {
  return column({ style: { gap: 16, align_items: "center" }, children: [
    text("Count: " + count, { size: 48, weight: "bold" }),
    row({ children: [
      on(button("−", {}), "click", "dec"),
      on(button("+", {}), "click", "inc")
    ]})
  ]});
}
function onEvent(ev) {
  if (ev.id == "inc") { count = count + 1; }
  if (ev.id == "dec") { count = count - 1; }
  render(view());
}
render(view());
```

The builder helpers (`column`, `row`, `text`, `button`, `canvas`, `scene3d`, …),
the `render`/`storageSet`/`themeSet`/`hostSend`/… wrappers, and the `on(...)`
event binder all come from a small JS **prelude** prepended to every Miniapp
(`crates/elpis-host/src/prelude.js`).

## Covered Blinc surface

The protocol and lowering cover the full Blinc ecosystem — verified by the
`all_kinds_lower` test (30 widget families) and the protocol round-trip tests:

- **Layout** — flex (row/column/reverse/wrap), grid, stack, block, scroll
  viewports, fixed overlays, spacers; the full Tailwind-like style surface
  (sizing with px/%/fr/vw/em/rem/auto, padding/margin, gap, align/justify,
  position/inset/z-index, overflow).
- **Paint** — solid/linear/radial/conic gradient/image brushes, per-corner
  radii, borders, multi-shadow stacks, opacity, 2D+3D transforms, blur /
  backdrop-blur / glassmorphism filters, CSS classes + raw-CSS overrides.
- **Content** — text (weight/italic/align/underline/line-height/letter-spacing/
  max-lines), rich text runs, markdown, images (fit/placeholder), SVG (inline or
  asset, recolorable), icons (Tabler/Noto sets).
- **Widgets** — button, text input, checkbox, switch, radio, slider (incl.
  range), dropdown (incl. multi), tabs, carousel, progress bar, spinner.
- **2D canvas** — an immediate-mode `DrawOp` list (rects, rounded rects,
  circles, ellipses, lines, polylines, polygons, arcs, arbitrary bezier paths,
  text, images, clip + transform + layer stack) replayed into Blinc's
  `DrawContext`.
- **3D / game** — a declarative scene (perspective/orthographic camera;
  ambient/directional/point/spot lights; cube/sphere/plane/cylinder/cone/torus/
  glTF/custom geometry; PBR materials; sprites; a scene graph; pickable
  entities; optional physics) driven by a per-frame `onTick`.
- **Animation** — spring physics and keyframe/tween timelines per property,
  repeat/ping-pong, delays, completion events, and layout transitions.
- **Theming, router, media, storage, messaging** — `theme.*`, `router.*`,
  `media.*`, sandboxed `storage.*`, and a bidirectional `host.send`/`host.request`
  pipe.

## The Glass UI kit (`sdk/glass-ui-kit.js`)

A full **liquid-glass component SDK** written entirely in sandbox JavaScript on
top of the Blinc builders the host imports into the VM. Every surface renders as
Apple-style *liquid glass*: a translucent, backdrop-blurred panel with a
saturated backdrop, a bright specular rim, and physical depth. The kit defines
one global, `Glass`, with factory functions covering the whole widget space:

- **Layout** — `screen`, `row`/`column`/`stack`/`grid`/`wrap`/`scroll`/`center`,
  `spacer`, `divider`, `surface`.
- **Panels** — `card`, `panel`, `sheet`, `hero`.
- **Typography** — `text`, `display`/`heading`/`title`/`subtitle`/`caption`/
  `label`, `code`, `link`, `markdown`.
- **Actions** — `button` (accent/ghost/destructive/success/pill/sizes),
  `iconButton`, `fab`, `buttonGroup`, `segmented`.
- **Indicators** — `badge`, `dot`, `chip`, `tag`, `avatar`, `avatarGroup`,
  `kbd`, `icon`.
- **Forms** — `field`, `textField`/`textArea`/`passwordField`/`numberField`/
  `search`, `checkbox`, `toggle` (`switch`), `radioGroup`, `slider`,
  `rangeSlider`, `select`/`multiSelect`, `stepper`, `rating`, `colorSwatch`.
- **Navigation** — `navbar`, `tabBar`, `tabs`, `breadcrumbs`, `pagination`,
  `menu`/`menuItem`, `sidebar`, `drawer`.
- **Overlays** — `modal`, `bottomSheet`, `popover`, `tooltip`, `toast`,
  `snackbar`, `loadingOverlay`.
- **Feedback** — `alert`/`banner`, `progress`/`progressCircle`, `spinner`,
  `skeleton`, `emptyState`.
- **Data display** — `list`/`listItem`, `table`, `stat`, `keyValue`, `timeline`,
  `accordion`/`collapsible`.
- **Media** — `image`, `video`, `audioPlayer`, `carousel`, `gallery`, `scene`
  (glass-framed 3D).
- **Charts (2D canvas)** — `ring`, `gauge`, `barChart`, `lineChart`.
- **Decorative** — `blob` (animated liquid wallpaper), `glow`.
- **Theming** — `Glass.tokens`, `Glass.theme(partial)`, `Glass.material(variant)`.

```js
function view() {
  return Glass.screen({ children: [
    Glass.navbar({ title: "Inbox", trailing: [ Glass.iconButton({ icon: "plus", onClick: "add" }) ] }),
    Glass.card({ children: [
      Glass.heading({ text: "Welcome" }),
      Glass.button({ label: "Continue", variant: "accent", onClick: "go" })
    ]})
  ]});
}
render(view());
```

Because runtime module import is denied in the sandbox, the kit is shared by
**prepending** it to a Miniapp. The host binary does this with `--lib`:

```bash
cargo run --bin elpis -- --lib sdk/glass-ui-kit.js miniapps/glass-gallery/app.js
cargo run --bin elpis -- --lib sdk/glass-ui-kit.js miniapps/glass-gallery/app.js --event tab:4 --ticks 3
```

### Foundational support added for the kit

The kit drove a few additions to Elpis itself:

- **Protocol** — a `GlassMaterial` primitive on `Style` (`glass_material`):
  tint, backdrop blur, saturation, brightness, specular rim, radius and
  elevation. The `Node → Blinc` lowering **expands** it into concrete paint (a
  backdrop-blur + saturate `Filter`, a tinted translucent background, a rim
  border, a radius and an elevation shadow) — only for the fields a node didn't
  set itself — so a glass surface renders on any backend that honors those
  fields. `Style::glass` stays a plain bool for the simplest case.
- **Prelude** — new builders (`audio`, `component`) and paint/composition
  helpers: `hex`/`hexA`/`withAlpha`, `linearGradient`/`radialGradient`/
  `conicGradient`/`imageBrush`, `stop`, `shadow`, `glassMaterial`, and
  `withKey`/`withStyle`/`bindEvents`/`withAnim`/`withTransition`.
- **Host binary** — `--lib FILE` (repeatable) to compose a Miniapp from
  reusable SDK sources, the sandbox-friendly substitute for `import`.

## Running

```bash
# Build + test the whole workspace (no GPU needed).
cargo build
cargo test

# Run a Miniapp headless: boot, render, simulate clicks, drive animation frames.
cargo run --bin elpis -- miniapps/counter/app.js --event inc --event inc --event dec
cargo run --bin elpis -- miniapps/showcase/app.js --event tab2 --ticks 3

# Open a real Blinc window (pulls the Blinc/wgpu stack).
cargo run --bin elpis --features blinc -- miniapps/showcase/app.js
```

### A note on the `blinc-backend` feature

The default build is fully self-contained and GPU-free: the host, the protocol,
the diff/patch engine, and the complete **Node → Blinc lowering** all compile and
are unit-tested without pulling Blinc. The live renderer (`elpis-blinc`'s
`blinc_backend` module and `elpis-app`'s `--features blinc`) depends on the
[Blinc 0.5 crates][blinc] from crates.io (wgpu, windowing, text, svg, animation,
theming, router) and is built only when that feature is enabled. It is
compile-verified against `blinc_app` 0.5.1 (`cargo check -p elpis-blinc
--features blinc-backend` and `cargo check -p elpis-web --target
wasm32-unknown-unknown`).

The backend renders the **full surface**: every widget family from Blinc
primitives, complete text styling, real linear/radial/conic gradients (for
backgrounds and canvas paint), the entire 2D canvas op set (rects, rounded
rects, circles, ellipses, lines, polylines/polygons, arbitrary bezier paths,
arcs, text, clip + transform + opacity stacks), and a CPU software 3D renderer
that projects, depth-sorts, and Lambert-shades the `Scene3D` geometry so the 3D
tab shows real lit, rotating solids.

Building the windowed path needs the usual Linux desktop/GPU development
libraries that the Blinc/winit/wgpu stack links against — on Debian/Ubuntu:

```bash
apt-get install -y libpango1.0-dev libgdk-pixbuf-2.0-dev libatk1.0-dev \
  libgtk-3-dev libxkbcommon-dev libwayland-dev libasound2-dev libudev-dev
```

(Plus a working Vulkan/GL driver at run time to actually open a window.) The
headless default path needs none of this.

## Demos: desktop, web, and Android

The same Elpis sandbox + bridge + `Node → Blinc` lowering drives three platform
targets; only the run loop differs (each demo crate supplies its own `blinc_app`
platform feature and calls the shared `elpis_blinc::frame_closure`):

| Target | Crate | Run loop | Status |
|--------|-------|----------|--------|
| Desktop | `apps/elpis-app` (`--features blinc`) | `WindowedApp::run` | compiles against blinc 0.5.1 |
| Web (wasm) | `apps/elpis-web` | `WebApp::run` (WebGPU canvas) | **compiles for `wasm32-unknown-unknown`** |
| Android | `apps/elpis-android` | `AndroidApp::run` (NativeActivity) | built in CI via `cargo-ndk` + Gradle |

### GitHub workflows

* **`.github/workflows/web.yml`** — builds `apps/elpis-web` with `wasm-pack`,
  assembles a static site (`index.html` + `pkg/`), and deploys it to **GitHub
  Pages**. One-time setup: repo *Settings → Pages → Source: GitHub Actions*.
  The demo then lives at `https://<owner>.github.io/<repo>/`.

* **`.github/workflows/android.yml`** — cross-compiles `apps/elpis-android` to an
  `arm64-v8a` `.so` with `cargo-ndk`, packages it into a debug APK with the
  Gradle project under `apps/elpis-android/android/`, and **commits the APK to
  the repository root** as `elpis-demo.apk` (also uploaded as a build artifact).

Both also run on `workflow_dispatch`. They trigger on pushes to `main`, so they
take effect once this branch is merged to the default branch.

Build the web demo locally:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
wasm-pack build apps/elpis-web --target web --release --out-dir pkg
python3 -m http.server -d apps/elpis-web   # then open http://localhost:8000
```

## Sandboxing

Each Elpis instance is an isolated VM with its own capability set and resource
governor. By default a Miniapp may render UI, log, read the clock, generate
randomness, use per-instance key/value storage, and exchange host messages —
but **network, filesystem, and runtime module import are denied** unless the
host explicitly grants them in `SandboxConfig`. The VM governor caps CPU steps,
heap, and storage, and the instance can be paused, resumed, or terminated by the
host at any step boundary.

[elpa]: https://github.com/cosmopole-org/elpa/tree/main/rust/crates/elpian-vm
[blinc]: https://github.com/project-blinc/Blinc
