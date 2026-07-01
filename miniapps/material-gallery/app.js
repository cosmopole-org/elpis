// Material Gallery — a living showcase of the Material UI kit
// (sdk/material-ui-kit.js), a Material Design 3 / Flutter-faithful SDK built
// as a real widget class hierarchy (`Widget` -> `StatelessWidget` /
// `StatefulWidget`, `ColorScheme`/`ThemeData`, …).
//
// Run it with the kit prepended (module import is denied in the sandbox):
//
//   cargo run --bin elpis -- --lib sdk/material-ui-kit.js miniapps/material-gallery/app.js
//   cargo run --bin elpis -- --lib sdk/material-ui-kit.js miniapps/material-gallery/app.js --event nav:2
//
// All state lives in VM globals and the app re-renders on each event, same
// convention as the other miniapps (counter, showcase, glass-gallery).

var tab = 0;             // active NavigationBar destination
var checked = true;
var switchOn = false;
var radioValue = "b";
var sliderVal = 0.4;
var textValue = "";
var chipSelected = false;
var expanded = true;
var dialogOpen = false;
var stepIndex = 1;

var DESTINATIONS = [
  { icon: "home", label: "Buttons" },
  { icon: "edit", label: "Inputs" },
  { icon: "list", label: "Data" },
  { icon: "notifications", label: "Feedback" }
];

// ---- Sections --------------------------------------------------------------

function buttonsTab() {
  return Material.column({
    spacing: 16,
    children: [
      Material.card({ children: [
        Material.text({ text: "Buttons", variant: "titleLarge" }),
        Material.row({ spacing: 12, children: [
          Material.elevatedButton({ label: "Elevated", onClick: "noop" }),
          Material.filledButton({ label: "Filled", onClick: "noop" }),
          Material.filledTonalButton({ label: "Tonal", onClick: "noop" })
        ] }),
        Material.row({ spacing: 12, children: [
          Material.outlinedButton({ label: "Outlined", onClick: "noop" }),
          Material.textButton({ label: "Text", onClick: "noop" }),
          Material.textButton({ label: "Disabled", disabled: true, onClick: "noop" })
        ] }),
        Material.row({ spacing: 12, children: [
          Material.iconButton({ icon: "favorite", onClick: "noop" }),
          IconButton.filled({ icon: "add", onClick: "noop" }),
          IconButton.filledTonal({ icon: "share", onClick: "noop" }),
          IconButton.outlined({ icon: "close", onClick: "noop" })
        ] })
      ] }),
      Material.card({ children: [
        Material.text({ text: "Chips", variant: "titleLarge" }),
        Material.row({ spacing: 8, children: [
          Material.chip({ label: "Assist" }),
          Material.choiceChip({ label: "Selected", selected: chipSelected, onClick: "chip" }),
          Material.filterChip({ label: "Filter" }),
          Material.actionChip({ label: "Delete", deletable: true, onDeleted: "noop" })
        ] })
      ] })
    ]
  });
}

function inputsTab() {
  return Material.column({
    spacing: 16,
    children: [
      Material.card({ children: [
        Material.text({ text: "Selection controls", variant: "titleLarge" }),
        Material.checkboxListTile({ title: "Subscribe to updates", value: checked, onChanged: "checked" }),
        Material.radioListTile({ title: "Option A", value: "a", groupValue: radioValue, onChanged: "radioA" }),
        Material.radioListTile({ title: "Option B", value: "b", groupValue: radioValue, onChanged: "radioB" }),
        Material.switchListTile({ title: "Airplane mode", value: switchOn, onChanged: "switch" })
      ] }),
      Material.card({ children: [
        Material.text({ text: "Slider: " + round(sliderVal * 100) + "%", variant: "titleLarge" }),
        Material.slider({ value: sliderVal, onChanged: "slide" })
      ] }),
      Material.card({ children: [
        Material.text({ text: "Text field", variant: "titleLarge" }),
        Material.textField({ labelText: "Your name", hintText: "Ada Lovelace", value: textValue, onChanged: "text" })
      ] })
    ]
  });
}

function dataTab() {
  return Material.column({
    spacing: 16,
    children: [
      Material.card({ padding: 0, children: [
        Material.listTile({ leading: Material.circleAvatar({ initials: "AL" }), title: "Ada Lovelace", subtitle: "Mathematician", trailing: Material.icon({ name: "chevron_right" }) }),
        Material.divider({}),
        Material.listTile({ leading: Material.circleAvatar({ initials: "AT" }), title: "Alan Turing", subtitle: "Computer scientist", trailing: Material.icon({ name: "chevron_right" }) })
      ] }),
      Material.expansionTile({ title: "Advanced settings", expanded: expanded, onExpansionChanged: "expand",
        children: [ Material.text({ text: "More detail goes here.", variant: "bodyMedium" }) ] }),
      Material.dataTable({
        columns: [{ label: "Name" }, { label: "Role" }, { label: "Score" }],
        rows: [["Ada", "Engineer", "98"], ["Alan", "Theorist", "95"], ["Grace", "Admiral", "99"]]
      }),
      Material.stepper({
        currentStep: stepIndex, onStepTapped: "step",
        steps: [
          { title: "Account" }, { title: "Shipping", content: Material.text({ text: "Address details" }) }, { title: "Payment" }
        ]
      })
    ]
  });
}

function feedbackTab() {
  return Material.column({
    spacing: 16,
    children: [
      Material.card({ children: [
        Material.text({ text: "Progress", variant: "titleLarge" }),
        Material.linearProgressIndicator({ value: sliderVal }),
        Material.row({ spacing: 24, children: [
          Material.circularProgressIndicator({ value: sliderVal }),
          Material.circularProgressIndicator({})
        ] })
      ] }),
      Material.row({ spacing: 12, children: [
        Material.filledButton({ label: "Open dialog", onClick: "openDialog" }),
        Material.badge({ label: 5, child: Material.iconButton({ icon: "notifications" }) })
      ] })
    ]
  });
}

function body() {
  if (tab == 0) { return buttonsTab(); }
  if (tab == 1) { return inputsTab(); }
  if (tab == 2) { return dataTab(); }
  return feedbackTab();
}

function view() {
  var layers = [
    Material.scaffold({
      appBar: Material.appBar({ title: "Material Gallery", centerTitle: false,
        actions: [Material.iconButton({ icon: "search", onClick: "noop" })] }),
      body: Material.singleChildScrollView({ children: [
        div({ style: { padding: { top: 16, right: 16, bottom: 16, left: 16 } }, children: [body()] })
      ] }),
      bottomNavigationBar: Material.navigationBar({ destinations: DESTINATIONS, selectedIndex: tab, onDestinationSelected: "nav" }),
      floatingActionButton: Material.fab({ icon: "add", onClick: "openDialog" })
    })
  ];
  if (dialogOpen) {
    push(layers, Material.alertDialog({
      title: "Material Dialog", content: "This overlay is a Material 3 AlertDialog.",
      onDismiss: "closeDialog",
      actions: [
        Material.textButton({ label: "Cancel", onClick: "closeDialog" }),
        Material.filledButton({ label: "OK", onClick: "closeDialog" })
      ]
    }));
  }
  return stack({ style: { width: { unit: "full" }, height: { unit: "full" } }, children: layers });
}

// ---- Event handling ---------------------------------------------------------

function splitId(id) {
  var idx = indexOf(id, ":");
  if (idx < 0) { return [id, ""]; }
  return [substring(id, 0, idx), substring(id, idx + 1, len(id))];
}

function onEvent(ev) {
  var parts = splitId(ev.id);
  var name = parts[0];
  var arg = parts[1];
  if (name == "nav") { tab = int(arg); }
  else if (name == "checked") { checked = !checked; }
  else if (name == "radioA") { radioValue = "a"; }
  else if (name == "radioB") { radioValue = "b"; }
  else if (name == "switch") { switchOn = !switchOn; }
  else if (name == "slide") { sliderVal = ev.value; }
  else if (name == "text") { textValue = ev.value; }
  else if (name == "chip") { chipSelected = !chipSelected; }
  else if (name == "expand") { expanded = !expanded; }
  else if (name == "step") { stepIndex = int(arg); }
  else if (name == "openDialog") { dialogOpen = true; }
  else if (name == "closeDialog") { dialogOpen = false; }
  render(view());
  return null;
}

// First paint.
render(view());
log("material gallery booted");
