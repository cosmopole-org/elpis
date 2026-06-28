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
| **`apps/elpis-app`** | The host binary (`elpis`). Instantiates a sandbox, loads a Miniapp, and runs it — headless by default, or in a real Blinc window with `--features blinc`. |
| **`miniapps/`** | Example Miniapps written in JS (`counter`, `showcase`). |

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
theming, router) and is built only when that feature is enabled. The lowering it
interprets is exercised by the headless path, so a Miniapp can be developed and
its UI tree validated end-to-end with no GPU.

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
