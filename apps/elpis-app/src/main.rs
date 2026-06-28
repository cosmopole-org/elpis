//! # elpis
//!
//! The Elpis host application. It instantiates an **Elpis sandbox** — an
//! [`elpis_blinc::Sandbox`] powered by the `elpian-vm` — loads a Miniapp written
//! in JavaScript, and runs it. The Miniapp controls a Blinc UI entirely through
//! the host-API bridge; it never touches the renderer directly.
//!
//! ```text
//!   elpis [MINIAPP.js] [--event ID]... [--ticks N]
//! ```
//!
//! * **Headless (default).** Boots the Miniapp, prints the rendered widget
//!   tree, dispatches any `--event` ids (simulating clicks), drives `--ticks`
//!   animation frames, and prints the resulting tree and instance stats. This
//!   runs anywhere — no GPU required — and is how the sandbox/bridge is
//!   exercised in CI.
//! * **Windowed (`--features blinc`).** Opens a real Blinc window and drives the
//!   sandbox against it (`run_windowed`).

use std::process::ExitCode;

use elpis_blinc::protocol::node::{NodeKind, TextSpec};
use elpis_blinc::{Node, Sandbox, SandboxConfig, SurfaceInfo};

/// A default Miniapp bundled into the binary, so `elpis` runs with no args.
const DEFAULT_MINIAPP: &str = include_str!("../../../miniapps/showcase/app.js");

struct Args {
    path: Option<String>,
    events: Vec<String>,
    ticks: u32,
}

fn parse_args() -> Args {
    let mut path = None;
    let mut events = Vec::new();
    let mut ticks = 0u32;
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--event" | "-e" => {
                if let Some(id) = it.next() {
                    events.push(id);
                }
            }
            "--ticks" | "-t" => {
                ticks = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other if !other.starts_with('-') => path = Some(other.to_string()),
            _ => {}
        }
    }
    Args { path, events, ticks }
}

fn print_help() {
    println!(
        "elpis — run a sandboxed Miniapp on the elpian-vm + Blinc\n\n\
         USAGE:\n  elpis [MINIAPP.js] [--event ID]... [--ticks N]\n\n\
         OPTIONS:\n  \
         -e, --event ID   Dispatch a UI event to handler ID (repeatable)\n  \
         -t, --ticks N    Drive N animation frames after booting\n  \
         -h, --help       Show this help\n\n\
         Build with `--features blinc` to open a real Blinc window instead."
    );
}

fn load_source(args: &Args) -> (String, String) {
    match &args.path {
        Some(p) => match std::fs::read_to_string(p) {
            Ok(src) => (p.clone(), src),
            Err(e) => {
                eprintln!("could not read '{p}': {e}; falling back to bundled showcase");
                ("<bundled showcase>".to_string(), DEFAULT_MINIAPP.to_string())
            }
        },
        None => ("<bundled showcase>".to_string(), DEFAULT_MINIAPP.to_string()),
    }
}

fn main() -> ExitCode {
    let args = parse_args();
    let (name, source) = load_source(&args);
    let surface = SurfaceInfo { width: 960.0, height: 640.0, scale_factor: 1.0 };

    let config = SandboxConfig { surface: Some(surface), ..SandboxConfig::new("elpis-main") };

    #[cfg(feature = "blinc")]
    {
        return run_windowed(config, &name, &source, surface);
    }

    #[cfg(not(feature = "blinc"))]
    {
        run_headless(config, &name, &source, &args)
    }
}

#[cfg(not(feature = "blinc"))]
fn run_headless(config: SandboxConfig, name: &str, source: &str, args: &Args) -> ExitCode {
    use elpis_blinc::{HeadlessBackend, UiEvent};

    println!("┌─ Elpis sandbox ─────────────────────────────────");
    println!("│ miniapp : {name}");
    println!("│ vm      : elpian-vm (JS → bytecode → execute)");
    println!("│ backend : headless (no GPU)  [build --features blinc for a window]");
    println!("└─────────────────────────────────────────────────");

    let mut sandbox = match Sandbox::from_js(config, source, Box::new(HeadlessBackend::new())) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("✗ {e}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(e) = sandbox.boot() {
        eprintln!("✗ boot failed: {e}");
        return ExitCode::FAILURE;
    }
    println!("\n● initial render (frame {}):", sandbox.frames());
    print_tree(&sandbox);

    for id in &args.events {
        println!("\n● dispatch event → handler '{id}'");
        if let Err(e) = sandbox.dispatch_event(&UiEvent::click(id.clone())) {
            eprintln!("✗ event failed: {e}");
            return ExitCode::FAILURE;
        }
        print_tree(&sandbox);
    }

    if args.ticks > 0 {
        println!("\n● driving {} animation frame(s)…", args.ticks);
        for _ in 0..args.ticks {
            if let Err(e) = sandbox.tick(16.0) {
                eprintln!("✗ tick failed: {e}");
                return ExitCode::FAILURE;
            }
        }
        print_tree(&sandbox);
    }

    let outbox = sandbox.take_outbox();
    if !outbox.is_empty() {
        println!("\n● host.send outbox:");
        for m in outbox {
            println!("    [{}] {}", m.channel, m.message);
        }
    }

    if let Some(u) = sandbox.usage() {
        println!("\n● instance stats: frames={} {:?}", sandbox.frames(), u);
    } else {
        println!("\n● instance stats: frames={}", sandbox.frames());
    }

    ExitCode::SUCCESS
}

/// Pretty-print the retained widget tree as an indented outline.
fn print_tree(sandbox: &Sandbox) {
    match sandbox.tree() {
        Some(root) => print_node(root, 0),
        None => println!("    (nothing rendered)"),
    }
}

fn print_node(node: &Node, depth: usize) {
    let indent = "  ".repeat(depth + 2);
    let mut line = format!("{indent}{}", node.type_tag());
    if let Some(k) = &node.key {
        line.push_str(&format!(" #{k}"));
    }
    if let Some(summary) = content_summary(&node.kind) {
        line.push_str(&format!("  {summary}"));
    }
    if !node.events.is_empty() {
        let names: Vec<&str> = node.events.keys().map(String::as_str).collect();
        line.push_str(&format!("  ⟨{}⟩", names.join(",")));
    }
    println!("{line}");
    for child in &node.children {
        print_node(child, depth + 1);
    }
}

fn content_summary(kind: &NodeKind) -> Option<String> {
    match kind {
        NodeKind::Text(TextSpec { text, .. }) => Some(format!("\"{text}\"")),
        NodeKind::Button(b) => Some(format!("[{}]", b.label)),
        NodeKind::Image(i) => Some(format!("<{}>", i.src)),
        NodeKind::Canvas(c) => Some(format!("({} draw ops)", c.ops.len())),
        NodeKind::Scene3D(s) => Some(format!("({} entities, {} lights)", s.entities.len(), s.lights.len())),
        NodeKind::Markdown(_) => Some("(markdown)".to_string()),
        _ => None,
    }
}

#[cfg(feature = "blinc")]
fn run_windowed(config: SandboxConfig, name: &str, source: &str, surface: SurfaceInfo) -> ExitCode {
    use elpis_blinc::BlincBackend;

    println!("Opening Blinc window for miniapp: {name}");
    let (backend, shared) = BlincBackend::new(surface);
    let mut sandbox = match Sandbox::from_js(config, source, Box::new(backend)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("✗ {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = sandbox.boot() {
        eprintln!("✗ boot failed: {e}");
        return ExitCode::FAILURE;
    }
    match elpis_blinc::run_windowed(name, surface.width as u32, surface.height as u32, sandbox, shared) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("✗ {e}");
            ExitCode::FAILURE
        }
    }
}
