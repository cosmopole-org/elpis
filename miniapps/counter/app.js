// Counter Miniapp — the smallest stateful Elpis app.
//
// State lives in VM globals (persisted across event turns). The top-level code
// renders once; `onEvent` mutates state and re-renders. The host diffs the two
// trees and patches only what changed.

var count = 0;

function view() {
  return column({
    style: { padding: { top: 24, right: 24, bottom: 24, left: 24 }, gap: 16, align_items: "center" },
    children: [
      text("Elpis Counter", { size: 28, weight: "bold", foreground: rgb(0.95, 0.95, 1.0) }),
      text("Count: " + count, { size: 48, weight: "bold", foreground: rgb(0.4, 0.8, 1.0) }),
      row({
        style: { gap: 12 },
        children: [
          on(button("−", { variant: "secondary" }), "click", "dec"),
          on(button("Reset", { variant: "ghost" }), "click", "reset"),
          on(button("+", { variant: "primary" }), "click", "inc")
        ]
      })
    ]
  });
}

function onEvent(ev) {
  if (ev.id == "inc") { count = count + 1; }
  if (ev.id == "dec") { count = count - 1; }
  if (ev.id == "reset") { count = 0; }
  render(view());
  return null;
}

render(view());
