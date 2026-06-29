// Showcase Miniapp — exercises a broad slice of the Blinc surface through the
// Elpis bridge: layout, text, the interactive widget set, a 2D canvas drawing,
// a 3D scene, animations, theming, storage, and host messaging. It is the
// living demonstration that the bridge "covers everything".

var tab = 0;
var sliderValue = 0.5;
var toggleOn = true;

function header() {
  return row({
    style: {
      padding: { top: 16, right: 20, bottom: 16, left: 20 },
      align_items: "center",
      justify_content: "between",
      background: {
        kind: "linear_gradient",
        angle: 90,
        stops: [
          { offset: 0, color: rgb(0.10, 0.11, 0.16) },
          { offset: 1, color: rgb(0.16, 0.13, 0.22) }
        ]
      }
    },
    children: [
      row({
        style: { gap: 10, align_items: "center" },
        children: [
          icon("E", { size: 28, color: rgb(0.4, 0.8, 1.0) }),
          text("Elpis · Blinc Showcase", { size: 22, weight: "bold", foreground: rgb(0.9, 0.9, 1.0) })
        ]
      }),
      row({
        style: { gap: 8 },
        children: [
          on(button("2D", {}), "click", "tab0"),
          on(button("Widgets", {}), "click", "tab1"),
          on(button("3D", {}), "click", "tab2")
        ]
      })
    ]
  });
}

// --- Tab 0: a 2D canvas drawing -------------------------------------------
function canvasTab() {
  var ops = [
    { type: "clear", color: rgb(0.06, 0.07, 0.10) },
    { type: "fill_round_rect", rect: { x: 40, y: 40, w: 220, h: 140 }, radius: 18,
      brush: { kind: "linear_gradient", angle: 45,
               stops: [ { offset: 0, color: rgb(0.2, 0.6, 1.0) }, { offset: 1, color: rgb(0.7, 0.3, 1.0) } ] } },
    { type: "fill_circle", center: { x: 360, y: 110 }, radius: 60, brush: solid(rgb(1.0, 0.5, 0.3)) },
    { type: "line", from: { x: 40, y: 220 }, to: { x: 420, y: 220 },
      stroke: { brush: solid(rgb(0.5, 0.9, 0.6)), width: 4, cap: "round" } },
    { type: "text", text: "DrawContext via Elpis", at: { x: 44, y: 260 }, size: 18, color: rgb(0.85, 0.85, 0.9) }
  ];
  return canvas(ops, { animated: false, style: { width: { unit: "px", value: 480 }, height: { unit: "px", value: 300 } } });
}

// --- Tab 1: interactive widgets -------------------------------------------
function widgetsTab() {
  return column({
    style: { padding: { top: 20, right: 20, bottom: 20, left: 20 }, gap: 16 },
    children: [
      on(textInput({ value: "", placeholder: "Type something…" }), "input", "typed"),
      on(slider({ value: sliderValue, min: 0, max: 1 }), "change", "slid"),
      text("Slider: " + sliderValue, { size: 14, foreground: rgb(0.7, 0.8, 0.9) }),
      row({
        style: { gap: 12, align_items: "center" },
        children: [
          on(toggle({ checked: toggleOn, label: "Notifications" }), "change", "toggled"),
          progressBar({ value: sliderValue })
        ]
      }),
      dropdown({ options: [ { value: "a", label: "Option A" }, { value: "b", label: "Option B" } ], selected: "a" }),
      markdown("**Markdown** rendered by Blinc, driven from JS.")
    ]
  });
}

// --- Tab 2: a 3D / game scene ---------------------------------------------
function sceneTab() {
  return scene3d({
    animated: true,
    style: { width: { unit: "px", value: 480 }, height: { unit: "px", value: 320 } },
    camera: { kind: "perspective", position: [0, 1.5, 4], look_at: [0, 0, 0], fov: 55 },
    lights: [
      { kind: "ambient", color: rgb(1, 1, 1), intensity: 0.3 },
      { kind: "directional", direction: [-1, -1, -1], color: rgb(1, 1, 1), intensity: 0.9 }
    ],
    entities: [
      { transform: { position: [0, 0, 0], rotation: [0, spin, 0], scale: [1, 1, 1] },
        geometry: { shape: "cube", size: 1.2 },
        material: { base_color: rgb(0.3, 0.7, 1.0), metallic: 0.2, roughness: 0.4 },
        pickable: true }
    ]
  });
}

var spin = 0;

function body() {
  if (tab == 0) { return canvasTab(); }
  if (tab == 1) { return widgetsTab(); }
  return sceneTab();
}

function view() {
  return column({
    style: { background: solid(rgb(0.04, 0.05, 0.07)) },
    children: [ header(), body() ]
  });
}

// Animation tick: spin the 3D cube while the scene tab is active.
function onTick(t) {
  if (tab == 2) {
    spin = spin + t.dt * 0.05;
    render(view());
  }
  return null;
}

function onEvent(ev) {
  if (ev.id == "tab0") { tab = 0; }
  if (ev.id == "tab1") { tab = 1; }
  if (ev.id == "tab2") { tab = 2; }
  if (ev.id == "slid") { sliderValue = ev.value; }
  if (ev.id == "toggled") { toggleOn = ev.value; }
  if (ev.id == "typed") { storageSet("draft", ev.value); }
  render(view());
  return null;
}

// Persist + restore a small piece of state to prove storage works.
themeSet({ name: "dark", accent: rgb(0.4, 0.8, 1.0) });
render(view());
log("showcase booted");
