//! # elpis-protocol
//!
//! The wire protocol of the **Elpis** sandboxed Miniapp framework: the shared
//! vocabulary a guest Miniapp (running on the `elpian-vm`) and the host UI
//! backend (driving Blinc) speak to each other.
//!
//! It has three parts:
//!
//! 1. **The widget DSL tree** ([`Node`] / [`NodeKind`] + [`style`], [`canvas`],
//!    [`scene3d`], [`animation`]). A Miniapp's render function returns a tree of
//!    nodes covering the full Blinc surface — flex/grid layout, text, images,
//!    SVG, icons, the interactive widget set, the 2D [`canvas::DrawOp`] drawing
//!    API, the 3D/game [`scene3d`] viewport, and [`animation`]s.
//!
//! 2. **The diff** ([`diff`]). Successive trees are reconciled into a minimal
//!    [`Patch`] script so the live Blinc tree is patched in place rather than
//!    rebuilt, preserving widget state and in-flight animations.
//!
//! 3. **The host-call envelope** ([`hostcall`]). The `askHost(api, ...args)`
//!    request/reply protocol the guest uses to drive the host (`ui.render`,
//!    `ui.patch`, `anim.*`, `canvas.*`, `scene3d.*`, `theme.*`, `router.*`, …).
//!
//! The protocol is renderer-agnostic: it is plain serde types with no Blinc or
//! wgpu dependency, so it serializes identically across the in-process bridge
//! and any out-of-process transport.

pub mod animation;
pub mod canvas;
pub mod diff;
pub mod hostcall;
pub mod node;
pub mod scene3d;
pub mod style;

pub use diff::{apply, diff, Patch, Path};
pub use hostcall::HostCall;
pub use node::{Node, NodeKind};
pub use style::{Brush, Color, Style};

/// Parse a widget tree from the JSON a guest emits.
pub fn parse_tree(json: &str) -> Result<Node, serde_json::Error> {
    serde_json::from_str(json)
}

/// Serialize a widget tree to JSON.
pub fn tree_to_json(node: &Node) -> String {
    serde_json::to_string(node).unwrap_or_else(|_| "null".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::*;
    use crate::style::*;

    fn text(key: &str, s: &str) -> Node {
        let mut n = Node::new(NodeKind::Text(TextSpec {
            text: s.to_string(),
            size: 14.0,
            weight: FontWeight::Normal,
            font: None,
            italic: false,
            align: TextAlign::Start,
            underline: false,
            strikethrough: false,
            line_height: None,
            letter_spacing: None,
            max_lines: 0,
            selectable: false,
        }));
        n.key = Some(key.to_string());
        n
    }

    fn container(children: Vec<Node>) -> Node {
        Node::new(NodeKind::Div).with_children(children)
    }

    /// The core contract: applying the diff of two trees turns one into the
    /// other exactly.
    fn assert_roundtrip(old: &Node, new: &Node) {
        let patches = diff(old, new);
        let mut work = old.clone();
        apply(&mut work, &patches).expect("apply must succeed");
        assert_eq!(&work, new, "patches:\n{patches:#?}");
    }

    #[test]
    fn identical_trees_produce_no_patches() {
        let a = container(vec![text("a", "hello")]);
        assert!(diff(&a, &a).is_empty());
    }

    #[test]
    fn text_change_is_a_props_patch() {
        let a = container(vec![text("a", "hello")]);
        let b = container(vec![text("a", "world")]);
        let patches = diff(&a, &b);
        assert_eq!(patches.len(), 1);
        assert!(matches!(patches[0], Patch::Props { .. }));
        assert_roundtrip(&a, &b);
    }

    #[test]
    fn style_change_is_a_style_patch() {
        let a = container(vec![text("a", "x")]);
        let mut b = a.clone();
        b.style.background = Some(Brush::solid(Color::WHITE));
        let patches = diff(&a, &b);
        assert_eq!(patches.len(), 1);
        assert!(matches!(patches[0], Patch::Style { .. }));
        assert_roundtrip(&a, &b);
    }

    #[test]
    fn appending_a_child_is_an_insert() {
        let a = container(vec![text("a", "1")]);
        let b = container(vec![text("a", "1"), text("b", "2")]);
        let patches = diff(&a, &b);
        assert!(patches.iter().any(|p| matches!(p, Patch::Insert { .. })));
        assert_roundtrip(&a, &b);
    }

    #[test]
    fn removing_a_child_is_a_remove() {
        let a = container(vec![text("a", "1"), text("b", "2")]);
        let b = container(vec![text("a", "1")]);
        assert_roundtrip(&a, &b);
    }

    #[test]
    fn reordering_keyed_children_uses_moves() {
        let a = container(vec![text("a", "1"), text("b", "2"), text("c", "3")]);
        let b = container(vec![text("c", "3"), text("a", "1"), text("b", "2")]);
        let patches = diff(&a, &b);
        assert!(patches.iter().any(|p| matches!(p, Patch::Move { .. })));
        assert_roundtrip(&a, &b);
    }

    #[test]
    fn keyed_insert_remove_and_update_together() {
        let a = container(vec![text("a", "1"), text("b", "2"), text("c", "3")]);
        // Remove b, update a, insert d at front, reorder.
        let b = container(vec![text("d", "4"), text("c", "3"), text("a", "9")]);
        assert_roundtrip(&a, &b);
    }

    #[test]
    fn kind_change_forces_replace() {
        let a = container(vec![text("a", "1")]);
        let mut img = Node::new(NodeKind::Image(ImageSpec {
            src: "logo.png".into(),
            fit: ImageFit::Contain,
            alt: None,
            placeholder: None,
        }));
        img.key = Some("a".into());
        let b = container(vec![img]);
        let patches = diff(&a, &b);
        assert!(matches!(patches[0], Patch::Replace { .. }));
        assert_roundtrip(&a, &b);
    }

    #[test]
    fn tree_json_roundtrips() {
        let a = container(vec![text("a", "hi")]);
        let json = tree_to_json(&a);
        let back = parse_tree(&json).unwrap();
        assert_eq!(a, back);
    }

    #[test]
    fn glass_material_roundtrips_and_diffs() {
        // A liquid-glass surface serializes, parses back identically, and a
        // change to its material is a single style patch.
        let mut a = container(vec![text("a", "x")]);
        a.style.glass_material = Some(GlassMaterial::default());
        let json = tree_to_json(&a);
        assert!(json.contains("glass_material"), "material must serialize: {json}");
        let back = parse_tree(&json).unwrap();
        assert_eq!(a, back);

        let mut b = a.clone();
        b.style.glass_material = Some(GlassMaterial { blur: 40.0, ..GlassMaterial::default() });
        let patches = diff(&a, &b);
        assert_eq!(patches.len(), 1);
        assert!(matches!(patches[0], Patch::Style { .. }));
        assert_roundtrip(&a, &b);
    }

    #[test]
    fn hostcall_parses_args() {
        let raw = r#"{"machineId":"m1","apiName":"ui.render","payload":[{"type":"div"}]}"#;
        let hc = HostCall::parse(raw).unwrap();
        assert_eq!(hc.api_name, "ui.render");
        assert_eq!(hc.args().len(), 1);
    }
}
