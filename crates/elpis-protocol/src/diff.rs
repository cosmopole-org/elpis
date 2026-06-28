//! Keyed tree diffing: turn an old widget tree into a new one with a minimal
//! list of [`Patch`]es.
//!
//! This is the "patch the UI tree partially for minimal computation overhead"
//! half of the bridge. Re-running a Miniapp's render path produces a fresh
//! [`Node`] tree every frame; rather than tearing down and rebuilding the live
//! Blinc widget tree, the host diffs the new tree against the retained one and
//! the backend applies only the patches — preserving widget state, in-flight
//! animations, scroll positions, and focus on the untouched subtrees.
//!
//! A [`Path`] addresses a node by the child indices from the root. The patch
//! script is ordered so it can be applied sequentially to the retained tree;
//! [`apply`] does exactly that, which also lets the headless backend and the
//! test-suite verify the round-trip `apply(old, diff(old, new)) == new`.

use serde::{Deserialize, Serialize};

use crate::animation::{Animation, Transition};
use crate::node::{EventMap, Node, NodeKind};
use crate::style::Style;

/// A path from the root to a node: the child index at each level.
pub type Path = Vec<usize>;

/// A single mutation of the retained tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Patch {
    /// Replace the whole subtree at `path` (kind or key changed).
    Replace { path: Path, node: Box<Node> },
    /// Replace the kind-specific props of the node at `path` (same tag).
    Props { path: Path, kind: Box<NodeKind> },
    /// Replace the style block of the node at `path`.
    Style { path: Path, style: Box<Style> },
    /// Replace the event bindings of the node at `path`.
    Events { path: Path, events: EventMap },
    /// Replace the animation list of the node at `path`.
    Animations { path: Path, animations: Vec<Animation> },
    /// Replace the transition of the node at `path`.
    Transition { path: Path, transition: Option<Transition> },
    /// Insert `node` as child `index` of the node at `path`.
    Insert { path: Path, index: usize, node: Box<Node> },
    /// Remove child `index` of the node at `path`.
    Remove { path: Path, index: usize },
    /// Move child `from` to `index` within the node at `path`.
    Move { path: Path, from: usize, index: usize },
}

/// Diff two trees rooted at `old` / `new`, returning the patch script.
pub fn diff(old: &Node, new: &Node) -> Vec<Patch> {
    let mut out = Vec::new();
    diff_node(&[], old, new, &mut out);
    out
}

fn same_identity(a: &Node, b: &Node) -> bool {
    a.key == b.key && a.type_tag() == b.type_tag()
}

fn diff_node(path: &[usize], old: &Node, new: &Node, out: &mut Vec<Patch>) {
    if !same_identity(old, new) {
        out.push(Patch::Replace { path: path.to_vec(), node: Box::new(new.clone()) });
        return;
    }
    // Same tag + key: patch the differing facets in place.
    if old.kind != new.kind {
        out.push(Patch::Props { path: path.to_vec(), kind: Box::new(new.kind.clone()) });
    }
    if old.style != new.style {
        out.push(Patch::Style { path: path.to_vec(), style: Box::new(new.style.clone()) });
    }
    if old.events != new.events {
        out.push(Patch::Events { path: path.to_vec(), events: new.events.clone() });
    }
    if old.animations != new.animations {
        out.push(Patch::Animations { path: path.to_vec(), animations: new.animations.clone() });
    }
    if old.transition != new.transition {
        out.push(Patch::Transition { path: path.to_vec(), transition: new.transition });
    }
    diff_children(path, &old.children, &new.children, out);
}

fn fully_keyed(nodes: &[Node]) -> bool {
    !nodes.is_empty() && nodes.iter().all(|n| n.key.is_some())
}

fn diff_children(path: &[usize], old: &[Node], new: &[Node], out: &mut Vec<Patch>) {
    if fully_keyed(old) && fully_keyed(new) {
        diff_children_keyed(path, old, new, out);
    } else {
        diff_children_positional(path, old, new, out);
    }
}

/// Index-based reconciliation: recurse over the common prefix, then trim or
/// append the tail.
fn diff_children_positional(path: &[usize], old: &[Node], new: &[Node], out: &mut Vec<Patch>) {
    let common = old.len().min(new.len());
    for i in 0..common {
        let mut child_path = path.to_vec();
        child_path.push(i);
        diff_node(&child_path, &old[i], &new[i], out);
    }
    if new.len() > old.len() {
        for (i, node) in new.iter().enumerate().skip(common) {
            out.push(Patch::Insert { path: path.to_vec(), index: i, node: Box::new(node.clone()) });
        }
    } else if old.len() > new.len() {
        // Remove from the back so earlier indices stay valid.
        for i in (common..old.len()).rev() {
            out.push(Patch::Remove { path: path.to_vec(), index: i });
        }
    }
}

/// Keyed reconciliation. Maintains a working copy `cur` of the child list that
/// evolves in lockstep with the emitted patches, so every index a `Move` /
/// `Insert` / `Remove` references is valid at the moment it is applied.
fn diff_children_keyed(path: &[usize], old: &[Node], new: &[Node], out: &mut Vec<Patch>) {
    let mut cur: Vec<Node> = old.to_vec();

    // 1. Remove old children whose key is absent from `new` (descending so the
    //    surviving indices remain stable).
    let new_keys: std::collections::HashSet<&Option<String>> =
        new.iter().map(|n| &n.key).collect();
    let mut i = cur.len();
    while i > 0 {
        i -= 1;
        if !new_keys.contains(&cur[i].key) {
            out.push(Patch::Remove { path: path.to_vec(), index: i });
            cur.remove(i);
        }
    }

    // 2. Walk the target order, inserting / moving / recursing as needed.
    for (target_idx, new_child) in new.iter().enumerate() {
        // Where does this key currently sit in `cur` (at or after target_idx)?
        let found = cur.iter().position(|c| c.key == new_child.key);
        match found {
            Some(j) if j == target_idx => {
                let mut child_path = path.to_vec();
                child_path.push(target_idx);
                diff_node(&child_path, &cur[target_idx], new_child, out);
            }
            Some(j) => {
                out.push(Patch::Move { path: path.to_vec(), from: j, index: target_idx });
                let moved = cur.remove(j);
                cur.insert(target_idx, moved);
                let mut child_path = path.to_vec();
                child_path.push(target_idx);
                // `cur[target_idx]` is the moved (old) node; diff against new.
                let old_child = cur[target_idx].clone();
                diff_node(&child_path, &old_child, new_child, out);
                cur[target_idx] = new_child.clone();
            }
            None => {
                out.push(Patch::Insert {
                    path: path.to_vec(),
                    index: target_idx,
                    node: Box::new(new_child.clone()),
                });
                cur.insert(target_idx, new_child.clone());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Application (the inverse): mutate a retained tree by a patch script.
// ---------------------------------------------------------------------------

/// Apply a patch script to `root` in place. Returns `Err` if a path is invalid.
pub fn apply(root: &mut Node, patches: &[Patch]) -> Result<(), String> {
    for patch in patches {
        apply_one(root, patch)?;
    }
    Ok(())
}

fn node_at<'a>(root: &'a mut Node, path: &[usize]) -> Result<&'a mut Node, String> {
    let mut node = root;
    for &idx in path {
        node = node
            .children
            .get_mut(idx)
            .ok_or_else(|| format!("path index {idx} out of bounds"))?;
    }
    Ok(node)
}

fn apply_one(root: &mut Node, patch: &Patch) -> Result<(), String> {
    match patch {
        Patch::Replace { path, node } => {
            *node_at(root, path)? = (**node).clone();
        }
        Patch::Props { path, kind } => {
            node_at(root, path)?.kind = (**kind).clone();
        }
        Patch::Style { path, style } => {
            node_at(root, path)?.style = (**style).clone();
        }
        Patch::Events { path, events } => {
            node_at(root, path)?.events = events.clone();
        }
        Patch::Animations { path, animations } => {
            node_at(root, path)?.animations = animations.clone();
        }
        Patch::Transition { path, transition } => {
            node_at(root, path)?.transition = *transition;
        }
        Patch::Insert { path, index, node } => {
            let parent = node_at(root, path)?;
            if *index > parent.children.len() {
                return Err("insert index out of bounds".into());
            }
            parent.children.insert(*index, (**node).clone());
        }
        Patch::Remove { path, index } => {
            let parent = node_at(root, path)?;
            if *index >= parent.children.len() {
                return Err("remove index out of bounds".into());
            }
            parent.children.remove(*index);
        }
        Patch::Move { path, from, index } => {
            let parent = node_at(root, path)?;
            if *from >= parent.children.len() {
                return Err("move source out of bounds".into());
            }
            let node = parent.children.remove(*from);
            let dest = (*index).min(parent.children.len());
            parent.children.insert(dest, node);
        }
    }
    Ok(())
}
