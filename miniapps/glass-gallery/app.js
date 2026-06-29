// Glass Gallery — a living showcase of the Glass UI kit (sdk/glass-ui-kit.js).
//
// Run it with the kit prepended (module import is denied in the sandbox):
//
//   cargo run --bin elpis -- --lib sdk/glass-ui-kit.js miniapps/glass-gallery/app.js
//   cargo run --bin elpis -- --lib sdk/glass-ui-kit.js miniapps/glass-gallery/app.js --event tab:2
//
// Every visible surface is liquid glass. The app keeps all state in VM globals
// and re-renders on each event, exactly like the other miniapps.

var tab = 0;            // active section
var switchOn = true;
var sliderVal = 0.6;
var rating = 3;
var count = 2;
var accordionOpen = 1;
var showModal = false;
var spin = 0;          // 3D scene spin
var phase = 0;         // blob animation phase

var TABS = [
  { icon: "layout", label: "Surfaces" },
  { icon: "forms", label: "Inputs" },
  { icon: "list", label: "Data" },
  { icon: "bell", label: "Feedback" },
  { icon: "chart", label: "Charts" },
  { icon: "cube", label: "3D" }
];

// ---- Sections -------------------------------------------------------------

function surfacesTab() {
  return Glass.column({ gap: 16, children: [
    Glass.hero({ title: "Liquid Glass", subtitle: "A full component kit for Elpis · Blinc", children: [
      Glass.row({ gap: 10, wrap: true, children: [
        Glass.button({ label: "Get started", variant: "accent", onClick: "noop" }),
        Glass.button({ label: "Docs", variant: "ghost", onClick: "noop" })
      ]})
    ]}),
    Glass.row({ gap: 16, wrap: true, children: [
      Glass.card({ variant: "regular", children: [ Glass.title({ text: "Regular" }), Glass.subtitle({ text: "Frosted surface" }) ]}),
      Glass.card({ variant: "thin", children: [ Glass.title({ text: "Thin" }), Glass.subtitle({ text: "Subtle blur" }) ]}),
      Glass.card({ variant: "thick", children: [ Glass.title({ text: "Thick" }), Glass.subtitle({ text: "Smoked glass" }) ]}),
      Glass.card({ variant: "accent", children: [ Glass.title({ text: "Accent" }), Glass.subtitle({ text: "Tinted glass" }) ]})
    ]}),
    Glass.row({ gap: 10, wrap: true, children: [
      Glass.badge({ text: "New", color: hex("#34D399") }),
      Glass.chip({ label: "Design", icon: "tag" }),
      Glass.tag({ label: "v1.0", color: hex("#38BDF8") }),
      Glass.avatar({ initials: "EL", size: 40 }),
      Glass.kbd({ text: "Cmd" }),
      Glass.kbd({ text: "K" })
    ]}),
    Glass.segmented({ items: ["Day", "Week", "Month"], selected: 1, onSelect: "seg" })
  ]});
}

function inputsTab() {
  return Glass.column({ gap: 16, children: [
    Glass.card({ children: [
      Glass.field({ label: "Email", control: Glass.textField({ placeholder: "you@example.com", onInput: "email" }) }),
      Glass.field({ label: "Password", control: Glass.passwordField({ placeholder: "••••••••" }) }),
      Glass.search({ placeholder: "Search the kit…", onInput: "search" })
    ]}),
    Glass.card({ gap: 14, children: [
      Glass.row({ justify: "between", align: "center", children: [
        Glass.text({ text: "Notifications" }), Glass.toggle({ checked: switchOn, onChange: "toggle" }) ]}),
      Glass.row({ justify: "between", align: "center", children: [
        Glass.text({ text: "Volume" }), Glass.text({ text: "" + round(sliderVal * 100) + "%", color: Glass.tokens.textDim }) ]}),
      Glass.slider({ value: sliderVal, onChange: "slide" }),
      Glass.row({ justify: "between", align: "center", children: [
        Glass.text({ text: "Quantity" }), Glass.stepper({ value: count, onDecrement: "dec", onIncrement: "inc" }) ]}),
      Glass.row({ justify: "between", align: "center", children: [
        Glass.text({ text: "Rating" }), Glass.rating({ value: rating, onRate: "rate" }) ]}),
      Glass.select({ options: [ { value: "a", label: "Option A" }, { value: "b", label: "Option B" } ], selected: "a", onChange: "sel" })
    ]})
  ]});
}

function dataTab() {
  return Glass.column({ gap: 16, children: [
    Glass.row({ gap: 16, wrap: true, children: [
      Glass.stat({ label: "Revenue", value: "$48.2k", delta: "12%", deltaUp: true }),
      Glass.stat({ label: "Users", value: "1,204", delta: "3%", deltaUp: false }),
      Glass.stat({ label: "Uptime", value: "99.9%" })
    ]}),
    Glass.list({ items: [
      { icon: "user", title: "Ada Lovelace", subtitle: "Online", chevron: true, onClick: "open" },
      { icon: "user", title: "Alan Turing", subtitle: "Last seen 2h ago", chevron: true, onClick: "open" },
      { icon: "user", title: "Grace Hopper", subtitle: "Away", chevron: true, onClick: "open" }
    ]}),
    Glass.table({
      columns: [ { key: "name", label: "Name" }, { key: "role", label: "Role" }, { key: "score", label: "Score" } ],
      rows: [
        { name: "Ada", role: "Engineer", score: "98" },
        { name: "Alan", role: "Theorist", score: "95" },
        { name: "Grace", role: "Admiral", score: "99" }
      ]
    }),
    Glass.accordion({ selected: accordionOpen, onToggle: "acc", items: [
      { title: "What is liquid glass?", open: (accordionOpen == 0), body: Glass.subtitle({ text: "A translucent, backdrop-blurred material." }) },
      { title: "How do I theme it?", open: (accordionOpen == 1), body: Glass.subtitle({ text: "Call Glass.theme({ accent: hex('#FF6B6B') })." }) }
    ]}),
    Glass.timeline({ items: [
      { title: "Booted sandbox", time: "09:00", color: hex("#34D399") },
      { title: "Rendered UI", time: "09:01", color: hex("#5B8CFF") },
      { title: "Dispatched event", time: "09:02", color: hex("#A66BFF") }
    ]})
  ]});
}

function feedbackTab() {
  return Glass.column({ gap: 16, children: [
    Glass.alert({ kind: "info", title: "Heads up", message: "This is an informational banner." }),
    Glass.alert({ kind: "success", title: "Saved", message: "Your changes were stored." }),
    Glass.alert({ kind: "warning", title: "Careful", message: "This action is irreversible." }),
    Glass.alert({ kind: "danger", title: "Error", message: "Something went wrong." }),
    Glass.card({ gap: 14, children: [
      Glass.label({ text: "Progress" }), Glass.progress({ value: sliderVal }),
      Glass.row({ gap: 16, align: "center", children: [
        Glass.spinner({}), Glass.skeleton({ width: 160 }), Glass.skeleton({ width: 90 }) ]})
    ]}),
    Glass.row({ gap: 10, children: [
      Glass.button({ label: "Open dialog", variant: "accent", onClick: "openModal" }) ]}),
    Glass.emptyState({ icon: "inbox", title: "All caught up", message: "You have no new notifications.", action: "Refresh", onAction: "noop" })
  ]});
}

function chartsTab() {
  return Glass.column({ gap: 16, children: [
    Glass.row({ gap: 16, wrap: true, align: "center", justify: "center", children: [
      Glass.ring({ value: sliderVal, size: 120 }),
      Glass.gauge({ value: sliderVal, size: 160 })
    ]}),
    Glass.card({ children: [
      Glass.title({ text: "Bar chart" }),
      Glass.barChart({ data: [3, 7, 4, 9, 6, 8, 5], width: 300, height: 150 }) ]}),
    Glass.card({ children: [
      Glass.title({ text: "Line chart" }),
      Glass.lineChart({ data: [2, 5, 3, 8, 6, 9, 7, 10], width: 300, height: 120 }) ]})
  ]});
}

function sceneTab() {
  return Glass.card({ children: [
    Glass.title({ text: "Glass-framed 3D scene" }),
    Glass.scene({
      height: 300, animated: true,
      camera: { kind: "perspective", position: [0, 1.5, 4], look_at: [0, 0, 0], fov: 55 },
      lights: [
        { kind: "ambient", color: rgb(1, 1, 1), intensity: 0.3 },
        { kind: "directional", direction: [-1, -1, -1], color: rgb(1, 1, 1), intensity: 0.9 }
      ],
      entities: [
        { transform: { position: [0, 0, 0], rotation: [0, spin, 0.3], scale: [1, 1, 1] },
          geometry: { shape: "cube", size: 1.2 },
          material: { base_color: rgb(0.36, 0.55, 1.0), metallic: 0.3, roughness: 0.3 }, pickable: true }
      ]
    })
  ]});
}

function body() {
  if (tab == 0) { return surfacesTab(); }
  if (tab == 1) { return inputsTab(); }
  if (tab == 2) { return dataTab(); }
  if (tab == 3) { return feedbackTab(); }
  if (tab == 4) { return chartsTab(); }
  return sceneTab();
}

function view() {
  // A single top-to-bottom column over the screen's animated wallpaper, which
  // sits behind the content (via z-index) so every glass surface backdrop-blurs
  // it. The content is centred to a comfortable max width: full-bleed on
  // phones, a centred column on wide screens, so nothing overflows.
  var kids = [
    Glass.navbar({ title: "Glass UI Kit",
      leading: [ icon("sparkles", { size: 24, color: Glass.tokens.accent }) ],
      trailing: [ Glass.iconButton({ icon: "search", onClick: "noop" }),
                  Glass.iconButton({ icon: "settings", onClick: "noop" }) ] }),
    Glass.scroll({ children: [ body() ] }),
    Glass.tabBar({ items: TABS, selected: tab, onSelect: "tab" })
  ];
  if (showModal) {
    push(kids, Glass.modal({ title: "Liquid Glass Dialog",
      onDismiss: "closeModal", dismissible: true,
      children: [ Glass.subtitle({ text: "A glass sheet floating over a dimmed backdrop." }) ],
      actions: [ Glass.button({ label: "Cancel", variant: "ghost", onClick: "closeModal" }),
                 Glass.button({ label: "Confirm", variant: "accent", onClick: "closeModal" }) ] }));
  }
  return Glass.screen({ maxWidth: 760, phase: phase, animated: true, children: kids });
}

// ---- Event handling -------------------------------------------------------

// Split a handler id of the form "name:arg" into [name, arg].
function splitId(id) {
  var idx = indexOf(id, ":");
  if (idx < 0) { return [id, ""]; }
  return [ substring(id, 0, idx), substring(id, idx + 1, len(id)) ];
}

function onEvent(ev) {
  var parts = splitId(ev.id);
  var name = parts[0];
  var arg = parts[1];
  if (name == "tab") { tab = int(arg); }
  else if (name == "toggle") { switchOn = ev.value; }
  else if (name == "slide") { sliderVal = ev.value; }
  else if (name == "inc") { count = count + 1; }
  else if (name == "dec") { count = count - 1; }
  else if (name == "rate") { rating = int(arg); }
  else if (name == "acc") { accordionOpen = int(arg); }
  else if (name == "openModal") { showModal = true; }
  else if (name == "closeModal") { showModal = false; }
  render(view());
  return null;
}

function onTick(t) {
  phase = phase + t.dt * 0.001;
  if (tab == 5) { spin = spin + t.dt * 0.05; }
  render(view());
  return null;
}

// Theme + first paint.
Glass.theme({ accent: hex("#5B8CFF"), accent2: hex("#A66BFF") });
render(view());
log("glass gallery booted");
