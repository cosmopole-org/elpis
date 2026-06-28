//! The Elpis widget tree.
//!
//! A Miniapp's render function returns a tree of [`Node`]s. Each node has a
//! [`NodeKind`] (which Blinc widget family it is), a [`Style`], optional event
//! bindings and animations, and children. The guest emits this as plain JSON;
//! the host diffs successive trees and the Blinc backend reconciles the live
//! widget tree against the patches.
//!
//! `NodeKind` is `#[serde(tag = "type")]`, so a node serializes as
//! `{"type":"text", "text":"hi", ...}` — a shape that is natural to build from
//! JavaScript.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::animation::{Animation, Transition};
use crate::canvas::CanvasSpec;
use crate::scene3d::Scene3DSpec;
use crate::style::Style;

/// Event bindings on a node: event name (`"click"`, `"change"`, `"input"`,
/// `"pointerdown"`, `"keydown"`, `"submit"`, `"scroll"`, …) → a guest handler
/// id. When the event fires, the host invokes the guest's dispatch function
/// with `{ handler, event }`. A `BTreeMap` keeps ordering deterministic so the
/// differ produces stable patches.
pub type EventMap = BTreeMap<String, String>;

/// A node in the widget tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    /// Stable identity for keyed reconciliation. Two nodes with the same key
    /// across renders are treated as the same widget (state preserved, props
    /// patched); without a key, position is used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    #[serde(flatten)]
    pub kind: NodeKind,

    #[serde(default, skip_serializing_if = "is_default_style")]
    pub style: Style,

    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub events: EventMap,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub animations: Vec<Animation>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transition: Option<Transition>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Node>,
}

fn is_default_style(s: &Style) -> bool {
    *s == Style::default()
}

impl Node {
    pub fn new(kind: NodeKind) -> Node {
        Node {
            key: None,
            kind,
            style: Style::default(),
            events: EventMap::new(),
            animations: Vec::new(),
            transition: None,
            children: Vec::new(),
        }
    }

    pub fn with_children(mut self, children: Vec<Node>) -> Node {
        self.children = children;
        self
    }

    /// The discriminant string used by the differ to decide whether two nodes
    /// are "the same kind" (a kind change forces a full replace).
    pub fn type_tag(&self) -> &'static str {
        self.kind.tag()
    }
}

/// Which Blinc widget family a node represents.
///
/// This enumerates the full Blinc surface: layout containers, text/content,
/// media, the interactive widget set (`blinc_layout::widgets` + `blinc_cn`),
/// the 2D `Canvas`, and the 3D/game `Scene3D`. New families can be added
/// without touching the differ.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NodeKind {
    // ---- Layout containers --------------------------------------------
    /// The generic flex/grid/block container (Blinc `Div`).
    Div,
    /// Convenience row (flex-direction: row).
    Row,
    /// Convenience column (flex-direction: column).
    Column,
    /// Z-stacked layers (Blinc `stack`).
    Stack,
    /// CSS grid container.
    Grid,
    /// A scroll viewport.
    Scroll(ScrollSpec),
    /// Flexible empty space.
    Spacer,
    /// A fixed-position overlay layer (modals, toasts, tooltips).
    Overlay(OverlaySpec),

    // ---- Content -------------------------------------------------------
    Text(TextSpec),
    RichText(RichTextSpec),
    Markdown(MarkdownSpec),
    Image(ImageSpec),
    Svg(SvgSpec),
    Icon(IconSpec),

    // ---- Interactive widgets ------------------------------------------
    Button(ButtonSpec),
    TextInput(InputSpec),
    Checkbox(ToggleSpec),
    Switch(ToggleSpec),
    Radio(RadioSpec),
    Slider(SliderSpec),
    Dropdown(DropdownSpec),
    Tabs(TabsSpec),
    Carousel(CarouselSpec),
    ProgressBar(ProgressSpec),
    Spinner(SpinnerSpec),

    // ---- Graphics ------------------------------------------------------
    /// 2D immediate-mode drawing surface.
    Canvas(CanvasSpec),
    /// 3D scene / game viewport.
    #[serde(rename = "scene3d")]
    Scene3D(Scene3DSpec),

    // ---- Media ---------------------------------------------------------
    Video(MediaSpec),
    Audio(MediaSpec),

    /// A named guest-defined component instance (for tooling / hot-reload).
    Component(ComponentSpec),
}

impl NodeKind {
    pub fn tag(&self) -> &'static str {
        match self {
            NodeKind::Div => "div",
            NodeKind::Row => "row",
            NodeKind::Column => "column",
            NodeKind::Stack => "stack",
            NodeKind::Grid => "grid",
            NodeKind::Scroll(_) => "scroll",
            NodeKind::Spacer => "spacer",
            NodeKind::Overlay(_) => "overlay",
            NodeKind::Text(_) => "text",
            NodeKind::RichText(_) => "rich_text",
            NodeKind::Markdown(_) => "markdown",
            NodeKind::Image(_) => "image",
            NodeKind::Svg(_) => "svg",
            NodeKind::Icon(_) => "icon",
            NodeKind::Button(_) => "button",
            NodeKind::TextInput(_) => "text_input",
            NodeKind::Checkbox(_) => "checkbox",
            NodeKind::Switch(_) => "switch",
            NodeKind::Radio(_) => "radio",
            NodeKind::Slider(_) => "slider",
            NodeKind::Dropdown(_) => "dropdown",
            NodeKind::Tabs(_) => "tabs",
            NodeKind::Carousel(_) => "carousel",
            NodeKind::ProgressBar(_) => "progress_bar",
            NodeKind::Spinner(_) => "spinner",
            NodeKind::Canvas(_) => "canvas",
            NodeKind::Scene3D(_) => "scene3d",
            NodeKind::Video(_) => "video",
            NodeKind::Audio(_) => "audio",
            NodeKind::Component(_) => "component",
        }
    }
}

// ---------------------------------------------------------------------------
// Per-kind specs.
// ---------------------------------------------------------------------------

/// Text alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextAlign {
    #[default]
    Start,
    Center,
    End,
    Justify,
}

/// Font weight (CSS numeric scale).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FontWeight {
    Thin,
    Light,
    #[default]
    Normal,
    Medium,
    Semibold,
    Bold,
    Black,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextSpec {
    pub text: String,
    #[serde(default = "default_text_size")]
    pub size: f32,
    #[serde(default)]
    pub weight: FontWeight,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,
    #[serde(default)]
    pub italic: bool,
    #[serde(default)]
    pub align: TextAlign,
    #[serde(default)]
    pub underline: bool,
    #[serde(default)]
    pub strikethrough: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_height: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub letter_spacing: Option<f32>,
    /// Truncate to N lines with an ellipsis (0 = unlimited).
    #[serde(default)]
    pub max_lines: u32,
    /// Whether the text is user-selectable.
    #[serde(default)]
    pub selectable: bool,
}

fn default_text_size() -> f32 {
    14.0
}

/// A run of styled text inside a `RichText`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextRun {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<f32>,
    #[serde(default)]
    pub weight: FontWeight,
    #[serde(default)]
    pub italic: bool,
    #[serde(default)]
    pub underline: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<crate::style::Color>,
    /// Optional link target / guest handler id (makes the run interactive).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RichTextSpec {
    pub runs: Vec<TextRun>,
    #[serde(default)]
    pub align: TextAlign,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_height: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkdownSpec {
    pub source: String,
    /// Enable GitHub-flavored extensions (tables, task lists, code fences).
    #[serde(default = "default_true")]
    pub gfm: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageSpec {
    /// Asset id or URL.
    pub src: String,
    #[serde(default)]
    pub fit: crate::style::ImageFit,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,
    /// Blur-hash / low-res placeholder while loading.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SvgSpec {
    /// Inline SVG markup, or (when `src` set) an asset id / URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,
    /// Override the SVG's fill (Blinc supports recoloring monochrome icons).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<crate::style::Color>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IconSpec {
    /// Icon name within a set (Blinc ships Tabler + Noto symbol sets).
    pub name: String,
    #[serde(default = "default_icon_size")]
    pub size: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<crate::style::Color>,
    /// Icon set (`"tabler"`, `"noto"`, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub set: Option<String>,
}

fn default_icon_size() -> f32 {
    24.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ButtonSpec {
    #[serde(default)]
    pub label: String,
    /// Visual variant (`"primary"`, `"secondary"`, `"ghost"`, `"destructive"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    #[serde(default)]
    pub disabled: bool,
    /// Optional leading icon name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default)]
    pub loading: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputSpec {
    #[serde(default)]
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    /// `"text"`, `"password"`, `"number"`, `"email"`, `"multiline"`, …
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_type: Option<String>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub readonly: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u32>,
    #[serde(default)]
    pub autofocus: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToggleSpec {
    #[serde(default)]
    pub checked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadioSpec {
    pub options: Vec<OptionItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected: Option<String>,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionItem {
    pub value: String,
    pub label: String,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SliderSpec {
    #[serde(default)]
    pub value: f32,
    #[serde(default)]
    pub min: f32,
    #[serde(default = "one_f")]
    pub max: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<f32>,
    #[serde(default)]
    pub disabled: bool,
    /// Two-thumb range slider.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_end: Option<f32>,
}

fn one_f() -> f32 {
    1.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DropdownSpec {
    pub options: Vec<OptionItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub disabled: bool,
    /// Multi-select mode.
    #[serde(default)]
    pub multi: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TabsSpec {
    pub tabs: Vec<TabItem>,
    #[serde(default)]
    pub selected: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TabItem {
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CarouselSpec {
    #[serde(default)]
    pub index: u32,
    #[serde(default = "default_true")]
    pub indicators: bool,
    #[serde(default)]
    pub autoplay: bool,
    /// Autoplay interval in milliseconds.
    #[serde(default)]
    pub interval: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressSpec {
    /// `0.0..=1.0`, or `null` for indeterminate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<f32>,
    /// `"linear"` or `"circular"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpinnerSpec {
    #[serde(default = "default_icon_size")]
    pub size: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<crate::style::Color>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScrollSpec {
    /// `"vertical"`, `"horizontal"`, or `"both"`.
    #[serde(default = "default_scroll_axis")]
    pub axis: String,
    /// Snap children to viewport edges.
    #[serde(default)]
    pub snap: bool,
    /// Programmatic scroll offset request `[x, y]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scroll_to: Option<[f32; 2]>,
}

fn default_scroll_axis() -> String {
    "vertical".to_string()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OverlaySpec {
    /// Stacking layer (`"modal"`, `"toast"`, `"tooltip"`, `"popover"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer: Option<String>,
    /// Dim / capture the backdrop.
    #[serde(default)]
    pub backdrop: bool,
    /// Whether clicking the backdrop dismisses (emits a `dismiss` event).
    #[serde(default = "default_true")]
    pub dismissible: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaSpec {
    pub src: String,
    #[serde(default)]
    pub autoplay: bool,
    #[serde(default)]
    pub loop_: bool,
    #[serde(default)]
    pub muted: bool,
    #[serde(default = "default_true")]
    pub controls: bool,
    #[serde(default)]
    pub fit: crate::style::ImageFit,
    /// Seek request in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seek: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentSpec {
    pub name: String,
    /// Opaque props blob forwarded to the guest component.
    #[serde(default)]
    pub props: serde_json::Value,
}
