// Material Gallery — a living showcase of the Material UI kit
// (sdk/material-ui-kit.js), a Material Design 3 / Flutter-faithful SDK built
// as a real widget class hierarchy (`Widget` -> `StatelessWidget` /
// `StatefulWidget`, `ColorScheme`/`ThemeData`, …).
//
// Run it with the kit prepended (module import is denied in the sandbox):
//
//   cargo run --bin elpis -- --lib sdk/material-ui-kit.js miniapps/material-gallery/app.js
//   cargo run --bin elpis -- --lib sdk/material-ui-kit.js miniapps/material-gallery/app.js --event root:1
//
// Unlike the other miniapps (counter, showcase, glass-gallery), this one has
// no hand-written `onEvent`/id-parsing dispatch table at all: every control
// below is wired with an ordinary closure attached directly to its own
// widget (`onClick: () => this.setState({ ... })`), and the kit's own
// `Material.State`/`setState` re-renders exactly the part of the tree that
// actually changed — see the "Reactivity demo" card, which proves it with a
// pair of on-screen build counters.

// A single root `State` holds every piece of app-level UI state (the M3
// destination, dialog visibility, form values, …) — the direct equivalent of
// a Flutter `State` backing a top-level `Scaffold`.
class GalleryState extends State {
  init() {
    this.state = {
      tab: 0, checked: true, switchOn: false, radioValue: "b", sliderVal: 0.4,
      textValue: "", chipSelected: false, expanded: true, dialogOpen: false, stepIndex: 1
    };
    this.rootBuildCount = 0;
  }

  build(widget) {
    this.rootBuildCount = this.rootBuildCount + 1;
    var s = this.state;
    var layers = [
      Material.scaffold({
        appBar: Material.appBar({
          title: "Material Gallery", centerTitle: false,
          actions: [Material.iconButton({ icon: "search", onClick: () => log("search tapped") })]
        }),
        body: Material.singleChildScrollView({ children: [
          div({ style: { padding: { top: 16, right: 16, bottom: 16, left: 16 } }, children: [this._body()] })
        ] }),
        bottomNavigationBar: Material.navigationBar({
          destinations: DESTINATIONS, selectedIndex: s.tab,
          onDestinationSelected: (i) => this.setState({ tab: i })
        }),
        floatingActionButton: Material.fab({ icon: "add", onClick: () => this.setState({ dialogOpen: true }) })
      })
    ];
    if (s.dialogOpen) {
      push(layers, Material.alertDialog({
        title: "Material Dialog", content: "This overlay is a Material 3 AlertDialog.",
        onDismiss: () => this.setState({ dialogOpen: false }),
        actions: [
          Material.textButton({ label: "Cancel", onClick: () => this.setState({ dialogOpen: false }) }),
          Material.filledButton({ label: "OK", onClick: () => this.setState({ dialogOpen: false }) })
        ]
      }));
    }
    return stack({ style: { width: { unit: "full" }, height: { unit: "full" } }, children: layers });
  }

  _body() {
    var t = this.state.tab;
    if (t == 0) { return this._buttonsTab(); }
    if (t == 1) { return this._inputsTab(); }
    if (t == 2) { return this._dataTab(); }
    return this._feedbackTab();
  }

  _buttonsTab() {
    var s = this.state;
    return Material.column({
      spacing: 16,
      children: [
        Material.card({ children: [
          Material.text({ text: "Buttons", variant: "titleLarge" }),
          Material.row({ spacing: 12, children: [
            Material.elevatedButton({ label: "Elevated", onClick: () => log("elevated tapped") }),
            Material.filledButton({ label: "Filled", onClick: () => log("filled tapped") }),
            Material.filledTonalButton({ label: "Tonal", onClick: () => log("tonal tapped") })
          ] }),
          Material.row({ spacing: 12, children: [
            Material.outlinedButton({ label: "Outlined", onClick: () => log("outlined tapped") }),
            Material.textButton({ label: "Text", onClick: () => log("text tapped") }),
            Material.textButton({ label: "Disabled", disabled: true, onClick: () => log("unreachable") })
          ] }),
          Material.row({ spacing: 12, children: [
            Material.iconButton({ icon: "favorite", onClick: () => log("favorite tapped") }),
            IconButton.filled({ icon: "add", onClick: () => log("add tapped") }),
            IconButton.filledTonal({ icon: "share", onClick: () => log("share tapped") }),
            IconButton.outlined({ icon: "close", onClick: () => log("close tapped") })
          ] })
        ] }),
        Material.card({ children: [
          Material.text({ text: "Chips", variant: "titleLarge" }),
          Material.row({ spacing: 8, children: [
            Material.chip({ label: "Assist" }),
            Material.choiceChip({ label: "Selected", selected: s.chipSelected, onClick: () => this.setState({ chipSelected: !this.state.chipSelected }) }),
            Material.filterChip({ label: "Filter" }),
            Material.actionChip({ label: "Delete", deletable: true, onDeleted: () => log("chip deleted") })
          ] })
        ] }),
        new StatefulWidget({ state: reactivityCounterState }).build()
      ]
    });
  }

  _inputsTab() {
    var s = this.state;
    return Material.column({
      spacing: 16,
      children: [
        Material.card({ children: [
          Material.text({ text: "Selection controls", variant: "titleLarge" }),
          Material.checkboxListTile({ title: "Subscribe to updates", value: s.checked, onChanged: (v) => this.setState({ checked: v }) }),
          Material.radioListTile({ title: "Option A", value: "a", groupValue: s.radioValue, onChanged: () => this.setState({ radioValue: "a" }) }),
          Material.radioListTile({ title: "Option B", value: "b", groupValue: s.radioValue, onChanged: () => this.setState({ radioValue: "b" }) }),
          Material.switchListTile({ title: "Airplane mode", value: s.switchOn, onChanged: (v) => this.setState({ switchOn: v }) })
        ] }),
        Material.card({ children: [
          Material.text({ text: "Slider: " + round(s.sliderVal * 100) + "%", variant: "titleLarge" }),
          Material.slider({ value: s.sliderVal, onChanged: (v) => this.setState({ sliderVal: v }) })
        ] }),
        Material.card({ children: [
          Material.text({ text: "Text field", variant: "titleLarge" }),
          Material.textField({ labelText: "Your name", hintText: "Ada Lovelace", value: s.textValue, onChanged: (v) => this.setState({ textValue: v }) })
        ] })
      ]
    });
  }

  _dataTab() {
    var s = this.state;
    return Material.column({
      spacing: 16,
      children: [
        Material.card({ padding: 0, children: [
          Material.listTile({ leading: Material.circleAvatar({ initials: "AL" }), title: "Ada Lovelace", subtitle: "Mathematician", trailing: Material.icon({ name: "chevron_right" }) }),
          Material.divider({}),
          Material.listTile({ leading: Material.circleAvatar({ initials: "AT" }), title: "Alan Turing", subtitle: "Computer scientist", trailing: Material.icon({ name: "chevron_right" }) })
        ] }),
        Material.expansionTile({
          title: "Advanced settings", expanded: s.expanded,
          onExpansionChanged: (v) => this.setState({ expanded: v }),
          children: [Material.text({ text: "More detail goes here.", variant: "bodyMedium" })]
        }),
        Material.dataTable({
          columns: [{ label: "Name" }, { label: "Role" }, { label: "Score" }],
          rows: [["Ada", "Engineer", "98"], ["Alan", "Theorist", "95"], ["Grace", "Admiral", "99"]]
        }),
        Material.stepper({
          currentStep: s.stepIndex, onStepTapped: (i) => this.setState({ stepIndex: i }),
          steps: [
            { title: "Account" }, { title: "Shipping", content: Material.text({ text: "Address details" }) }, { title: "Payment" }
          ]
        })
      ]
    });
  }

  _feedbackTab() {
    var s = this.state;
    return Material.column({
      spacing: 16,
      children: [
        Material.card({ children: [
          Material.text({ text: "Progress", variant: "titleLarge" }),
          Material.linearProgressIndicator({ value: s.sliderVal }),
          Material.row({ spacing: 24, children: [
            Material.circularProgressIndicator({ value: s.sliderVal }),
            Material.circularProgressIndicator({})
          ] })
        ] }),
        Material.row({ spacing: 12, children: [
          Material.filledButton({ label: "Open dialog", onClick: () => this.setState({ dialogOpen: true }) }),
          Material.badge({ label: 5, child: Material.iconButton({ icon: "notifications" }) })
        ] }),
        Material.card({ children: [
          Material.text({ text: "Root rebuild count: " + this.rootBuildCount, variant: "bodyMedium" }),
          Material.text({ text: "(bumps once per tap anywhere on this tab/dialog/nav — the whole scaffold above is this State's own subtree)", variant: "bodySmall" })
        ] })
      ]
    });
  }
}

// A small nested `StatefulWidget` used to demonstrate that `setState` only
// re-renders *its own* subtree: tapping "+" bumps `reactivityCounterState`
// alone — the card grows its own build count, while `GalleryState`'s
// `rootBuildCount` (shown on the Feedback tab) never moves, because
// `GalleryState.build()` is never re-invoked by this counter's `setState`.
class ReactivityCounterState extends State {
  init() { this.state = { count: 0 }; this.buildCount = 0; }
  build(widget) {
    this.buildCount = this.buildCount + 1;
    return Material.card({ children: [
      Material.text({ text: "Reactivity demo", variant: "titleLarge" }),
      Material.text({ text: "This card's own build count: " + this.buildCount, variant: "bodyMedium" }),
      Material.text({ text: "Tapping + only rebuilds this card — check the Feedback tab's root count.", variant: "bodySmall" }),
      Material.row({ spacing: 12, children: [
        Material.text({ text: "Count: " + this.state.count, variant: "titleMedium" }),
        Material.filledButton({ label: "+", onClick: () => this.setState({ count: this.state.count + 1 }) })
      ] })
    ] });
  }
}

var galleryState = new GalleryState();
var reactivityCounterState = new ReactivityCounterState();

var DESTINATIONS = [
  { icon: "home", label: "Buttons" },
  { icon: "edit", label: "Inputs" },
  { icon: "list", label: "Data" },
  { icon: "notifications", label: "Feedback" }
];

// Boot: mount the whole app under `Material.runApp`. No `onEvent` needed —
// every closure above dispatches itself through the kit's own event
// registry (see sdk/material-ui-kit.js's "Events and setState" doc).
Material.runApp(() => new StatefulWidget({ state: galleryState }).build());
log("material gallery booted");
