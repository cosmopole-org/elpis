//! Visual + layout styling for an Elpis node.
//!
//! This mirrors the surface a Blinc `Div`/`Style` exposes (the Tailwind-like
//! chainable builder: flex/grid layout, spacing, sizing, colors, brushes,
//! gradients, borders, radii, shadows, transforms, filters, opacity, overflow)
//! but in a flat, serde-serializable form so a Miniapp's JS can emit it as a
//! plain JSON object. Every field is optional (`#[serde(default)]` +
//! `skip_serializing_if`) so the guest only sends what it sets, keeping the
//! per-frame payload — and the per-frame diff — small.

use serde::{Deserialize, Serialize};

/// An sRGB color with straight (non-premultiplied) alpha, each channel `0.0..=1.0`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    #[serde(default = "one")]
    pub a: f32,
}

fn one() -> f32 {
    1.0
}

impl Color {
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Color { r, g, b, a: 1.0 }
    }
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
    pub const TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

    /// Parse a CSS-style `#rgb` / `#rrggbb` / `#rrggbbaa` hex string.
    pub fn from_hex(s: &str) -> Option<Color> {
        let s = s.trim_start_matches('#');
        let v = u32::from_str_radix(s, 16).ok()?;
        let f = |shift: u32| ((v >> shift) & 0xff) as f32 / 255.0;
        match s.len() {
            6 => Some(Color::rgb(f(16), f(8), f(0))),
            8 => Some(Color::rgba(f(24), f(16), f(8), f(0))),
            3 => {
                let g = |shift: u32| {
                    let nib = (v >> shift) & 0xf;
                    (nib * 16 + nib) as f32 / 255.0
                };
                Some(Color::rgb(g(8), g(4), g(0)))
            }
            _ => None,
        }
    }
}

/// One stop of a gradient.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GradientStop {
    /// Position along the gradient axis, `0.0..=1.0`.
    pub offset: f32,
    pub color: Color,
}

/// A paint source: a flat color, a gradient, or an image pattern. Maps onto
/// Blinc's `Brush`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Brush {
    Solid {
        color: Color,
    },
    LinearGradient {
        /// Angle in degrees, measured clockwise from the +x axis.
        #[serde(default)]
        angle: f32,
        stops: Vec<GradientStop>,
    },
    RadialGradient {
        #[serde(default)]
        center: [f32; 2],
        #[serde(default = "one")]
        radius: f32,
        stops: Vec<GradientStop>,
    },
    /// Conic / sweep gradient (used by Blinc for dials, glass rims, etc.).
    ConicGradient {
        #[serde(default)]
        center: [f32; 2],
        #[serde(default)]
        start_angle: f32,
        stops: Vec<GradientStop>,
    },
    /// An image used as a fill, referenced by asset id or URL.
    Image {
        src: String,
        #[serde(default)]
        fit: ImageFit,
    },
}

impl Brush {
    pub fn solid(color: Color) -> Brush {
        Brush::Solid { color }
    }
}

/// How an image fills its box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageFit {
    #[default]
    Contain,
    Cover,
    Fill,
    None,
    ScaleDown,
}

/// A length, expressing the full set of CSS-like units Blinc understands.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "unit", content = "value", rename_all = "snake_case")]
pub enum Length {
    /// Device-independent pixels.
    Px(f32),
    /// Percentage of the parent's corresponding axis (`0.0..=100.0`).
    Percent(f32),
    /// Fraction of remaining free space (flex/grid `fr`).
    Fr(f32),
    /// Viewport-width / viewport-height percentage.
    Vw(f32),
    Vh(f32),
    /// `em` relative to the current font size.
    Em(f32),
    /// `rem` relative to the root font size.
    Rem(f32),
    /// Size to content.
    Auto,
    /// Fit / max-content.
    Fit,
    /// Fill available space.
    Full,
}

impl Length {
    pub fn px(v: f32) -> Length {
        Length::Px(v)
    }
}

/// Edge insets (padding / margin / inset), in pixels per side.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Edges {
    #[serde(default)]
    pub top: f32,
    #[serde(default)]
    pub right: f32,
    #[serde(default)]
    pub bottom: f32,
    #[serde(default)]
    pub left: f32,
}

impl Edges {
    pub fn all(v: f32) -> Edges {
        Edges { top: v, right: v, bottom: v, left: v }
    }
    pub fn symmetric(x: f32, y: f32) -> Edges {
        Edges { top: y, right: x, bottom: y, left: x }
    }
}

/// Per-corner radii (top-left, top-right, bottom-right, bottom-left), pixels.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct CornerRadius {
    #[serde(default)]
    pub tl: f32,
    #[serde(default)]
    pub tr: f32,
    #[serde(default)]
    pub br: f32,
    #[serde(default)]
    pub bl: f32,
}

impl CornerRadius {
    pub fn all(v: f32) -> CornerRadius {
        CornerRadius { tl: v, tr: v, br: v, bl: v }
    }
}

/// A drop / inner shadow.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Shadow {
    #[serde(default)]
    pub offset: [f32; 2],
    #[serde(default)]
    pub blur: f32,
    #[serde(default)]
    pub spread: f32,
    pub color: Color,
    #[serde(default)]
    pub inset: bool,
}

/// An affine + perspective transform, matching Blinc's `Transform`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    #[serde(default)]
    pub translate: [f32; 2],
    #[serde(default = "one_pair")]
    pub scale: [f32; 2],
    /// Rotation in degrees about the z axis (2D) — see also 3D rotate fields.
    #[serde(default)]
    pub rotate: f32,
    #[serde(default)]
    pub rotate_x: f32,
    #[serde(default)]
    pub rotate_y: f32,
    #[serde(default)]
    pub skew: [f32; 2],
    /// CSS `transform-origin` as a fraction of the box (`0.5,0.5` = center).
    #[serde(default = "half_pair")]
    pub origin: [f32; 2],
    /// Perspective distance for 3D card flips (0 = none).
    #[serde(default)]
    pub perspective: f32,
}

fn one_pair() -> [f32; 2] {
    [1.0, 1.0]
}
fn half_pair() -> [f32; 2] {
    [0.5, 0.5]
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            translate: [0.0, 0.0],
            scale: [1.0, 1.0],
            rotate: 0.0,
            rotate_x: 0.0,
            rotate_y: 0.0,
            skew: [0.0, 0.0],
            origin: [0.5, 0.5],
            perspective: 0.0,
        }
    }
}

/// A blur / backdrop-blur / glassmorphism filter stack. Blinc's signature
/// glass material is `Filter { backdrop_blur, saturate, .. }`.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Filter {
    #[serde(default)]
    pub blur: f32,
    #[serde(default)]
    pub backdrop_blur: f32,
    #[serde(default)]
    pub brightness: Option<f32>,
    #[serde(default)]
    pub contrast: Option<f32>,
    #[serde(default)]
    pub saturate: Option<f32>,
    #[serde(default)]
    pub grayscale: Option<f32>,
    #[serde(default)]
    pub hue_rotate: Option<f32>,
}

/// Flex direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

/// Display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Display {
    #[default]
    Flex,
    Block,
    Grid,
    Stack,
    Hidden,
}

/// Cross-axis alignment (`align-items`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Align {
    #[default]
    Stretch,
    Start,
    Center,
    End,
    Baseline,
}

/// Main-axis distribution (`justify-content`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Justify {
    #[default]
    Start,
    Center,
    End,
    Between,
    Around,
    Evenly,
}

/// Overflow behavior on one axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
    Scroll,
    Auto,
    Clip,
}

/// Positioning scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Position {
    #[default]
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

/// The complete style block for a node. Flat and fully optional so the guest
/// emits a compact JSON object and the differ compares cheaply.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Style {
    // ---- Box model -----------------------------------------------------
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<Length>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<Length>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_width: Option<Length>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_height: Option<Length>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_width: Option<Length>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_height: Option<Length>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<Edges>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin: Option<Edges>,

    // ---- Flex / grid layout -------------------------------------------
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<Display>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<FlexDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_items: Option<Align>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_self: Option<Align>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justify_content: Option<Justify>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_gap: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_gap: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_grow: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_shrink: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_basis: Option<Length>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap: Option<bool>,
    /// Grid track template, as CSS-ish track strings (e.g. `"1fr 2fr 1fr"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_template_columns: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_template_rows: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_column: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_row: Option<String>,

    // ---- Positioning ---------------------------------------------------
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inset: Option<Edges>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub z_index: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overflow_x: Option<Overflow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overflow_y: Option<Overflow>,

    // ---- Paint ---------------------------------------------------------
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<Brush>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreground: Option<Color>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_color: Option<Color>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius: Option<CornerRadius>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub shadows: Vec<Shadow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<Transform>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Filter>,
    /// Marks this node as a glassmorphism surface (Blinc's signature material).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glass: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,

    // ---- Identity / theming -------------------------------------------
    /// CSS classes (Blinc supports class-based theming + overrides).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub classes: Vec<String>,
    /// Raw CSS override string applied last (Blinc's escape hatch).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub css: Option<String>,
}

impl Style {
    pub fn new() -> Style {
        Style::default()
    }
}
